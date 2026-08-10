package com.karamellis.aterkeep

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

/**
 * Panel parolasinin (istege bagli) saklanmasi.
 *
 * URUNUN VAADIYLE ILGILI NOT: aterkeep'in duruşu, oturum anahtarinin diske hic
 * yazilmamasi ve parolanin her aciliste sorulmasidir. Telefonda bunu birebir
 * korumak, cihaz her yeniden baslatildiginda 7/24'un durmasi ve kullanicinin
 * uygulamayi elle acmasi demek olurdu.
 *
 * Bu yuzden secim KULLANICIYA birakildi ve varsayilan KAPALI:
 *  - kapali  → parola yalnizca bellekte, davranis masaustuyle ayni, boot'ta
 *              otomatik baslamaz.
 *  - acik    → parola Android Keystore destekli EncryptedSharedPreferences'ta
 *              saklanir (anahtar donanim destekli keystore'da, uygulama
 *              disindan okunamaz) ve servis boot'ta kendi basina kalkar.
 *
 * Acik secenek vaadi gevsetiyor; bu yuzden varsayilan degil ve arayuzde ne
 * yaptigi yaziyor.
 */
object Prefs {
    private const val FILE = "aterkeep_secure"
    private const val KEY_PASSWORD = "panel_password"

    private fun prefs(ctx: Context): SharedPreferences {
        val master = MasterKey.Builder(ctx)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        return EncryptedSharedPreferences.create(
            ctx,
            FILE,
            master,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    /** Saklanmis parola, yoksa null. */
    fun password(ctx: Context): String? = try {
        prefs(ctx).getString(KEY_PASSWORD, null)
    } catch (_: Exception) {
        // Keystore anahtari gecersizlestiyse (cihaz kilidi degisimi, yedekten
        // donme) cozme patlar. Bu, parolanin tekrar sorulmasi gereken bir
        // durumdur — cokmek degil.
        null
    }

    fun savePassword(ctx: Context, password: String) {
        try {
            prefs(ctx).edit().putString(KEY_PASSWORD, password).apply()
        } catch (_: Exception) {
        }
    }

    fun forget(ctx: Context) {
        try {
            prefs(ctx).edit().remove(KEY_PASSWORD).apply()
        } catch (_: Exception) {
        }
    }
}
