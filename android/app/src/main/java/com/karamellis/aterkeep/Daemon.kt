package com.karamellis.aterkeep

import android.content.Context
import java.io.File

/**
 * Daemon surecinin yol ve ortam sozlesmesi — tek yerde.
 *
 * Rust daemon'i APK icinde `lib*.so` adiyla tasiniyor ama bir kutuphane DEGIL,
 * calistirilabilir bir ELF. Sebep: Android 10+ uygulamanin veri dizininde
 * exec'i yasakliyor (W^X). Calistirma izniyle diske acilan tek yer, APK'dan
 * cikarilan `nativeLibraryDir` — ve oraya yalnizca `lib` ile baslayip `.so` ile
 * biten dosyalar aciliyor. Bu yuzden isimler bu bicimde.
 */
object Daemon {
    /** Rust keep-alive daemon'i (aarch64 ELF, `lib*.so` adiyla paketlendi). */
    const val DAEMON_LIB = "libaterkeep.so"

    /** Statik curl (aarch64 static-PIE ELF, ayni sebeple `lib*.so`). */
    const val CURL_LIB = "libcurlx.so"

    /** Panelin dinledigi port — daemon varsayilaniyla ayni (config.rs). */
    const val PANEL_PORT = 4041

    const val PANEL_URL = "http://127.0.0.1:$PANEL_PORT"

    fun daemonBin(ctx: Context) = File(ctx.applicationInfo.nativeLibraryDir, DAEMON_LIB)

    fun curlBin(ctx: Context) = File(ctx.applicationInfo.nativeLibraryDir, CURL_LIB)

    /** Daemon'in calisma dizini. `bot/` ve `config/` buranin altinda aranir. */
    fun workDir(ctx: Context): File = ctx.filesDir

    /** ATERKEEP_DIR: session.enc + aterkeep.json + bot durumu. */
    fun dataDir(ctx: Context) = File(ctx.filesDir, "config")

    /** Servisin ve daemon'in ciktisini biriktirdigi kayit dosyasi. */
    fun logFile(ctx: Context) = File(ctx.filesDir, "aterkeep.log")

    /**
     * CA demeti. Assets'ten kopyalanir cunku assets APK icinde SIKISTIRILMIS
     * durur — gercek bir dosya yolu yoktur, curl'e `--cacert` ile verilemez.
     */
    fun caBundle(ctx: Context) = File(ctx.filesDir, "cacert.pem")

    /**
     * cacert.pem'i assets'ten diske acar. Zaten varsa ve boyu ayniysa dokunmaz.
     */
    fun installCaBundle(ctx: Context) {
        val dst = caBundle(ctx)
        val bytes = ctx.assets.open("cacert.pem").use { it.readBytes() }
        if (dst.exists() && dst.length() == bytes.size.toLong()) return
        dst.writeBytes(bytes)
    }

    /**
     * Daemon surecinin ortami.
     *
     * Android'e ozel olan her sey burada toplandi; Rust tarafi bu degiskenleri
     * okuyup davranisini ayarliyor, yani platforma ozel dallanma koda gomulu
     * degil.
     */
    fun env(ctx: Context, password: String): Map<String, String> {
        val ca = caBundle(ctx).absolutePath
        return mapOf(
            "ATERKEEP_DIR" to dataDir(ctx).absolutePath,
            // Parola YALNIZCA cocuk surecin ortaminda. Daemon oturum anahtarini
            // bundan turetiyor ve anahtari diske yazmiyor.
            "ATERKEEP_KEY" to password,
            "ATERKEEP_CURL" to curlBin(ctx).absolutePath,
            // --doh-url: Android'de /etc/resolv.conf YOK. Statik curl sistem
            //   cozumleyicisini kullanamaz ve hicbir adi cozemez. DoH'u IP
            //   literal'e sorunca bootstrap DNS'e de ihtiyac kalmiyor.
            // --cacert: statik binary'ye gomulu sertifika yolu Android'de
            //   mevcut degil; demeti kendimiz veriyoruz.
            "ATERKEEP_CURL_EXTRA" to "--doh-url https://1.1.1.1/dns-query --cacert $ca",
            // ws.rs kendi TLS'ini vendored OpenSSL ile kuruyor; onun da varsayilan
            // sertifika yolu Android'de yok. OpenSSL bu degiskeni okuyor.
            // (ws.rs'te DNS sorunu yok: o, bionic'e bagli Rust binary'si icinde
            // getaddrinfo kullaniyor ve sistem cozumleyicisi calisiyor.)
            "SSL_CERT_FILE" to ca,
            "HOME" to ctx.filesDir.absolutePath,
            "TMPDIR" to ctx.cacheDir.absolutePath,
        )
    }

    /** APK'nin binary tasidigi ABI'ler. */
    private val SHIPPED_ABIS = setOf("arm64-v8a", "armeabi-v7a", "x86_64")

    /**
     * Cihaz bu APK'nin tasidigi binary'leri calistirabiliyor mu?
     *
     * Pratikte her Android cihaz bunlardan birini destekliyor; kontrol yine de
     * duruyor cunku alternatif, kurulduktan sonra sebebi gorunmeyen bir
     * "hicbir sey olmuyor" durumu.
     */
    fun abiSupported(): Boolean =
        android.os.Build.SUPPORTED_ABIS.any { it in SHIPPED_ABIS }
}
