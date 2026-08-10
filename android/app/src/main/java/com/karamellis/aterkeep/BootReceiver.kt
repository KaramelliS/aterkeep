package com.karamellis.aterkeep

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.core.content.ContextCompat

/**
 * Cihaz yeniden baslarsa servisi geri getirir.
 *
 * YALNIZCA parola "hatirla" ile saklanmissa. Aksi halde daemon oturumu
 * cozemeyeceginden ayaga kalkip hemen olurdu; kullaniciya hicbir sey
 * kazandirmayan, sadece bildirim gosterip duran bir servis olurdu.
 *
 * Servis tipi `specialUse` — bu, boot'tan baslatilabilmesi icin de gerekli:
 * Android 15, BOOT_COMPLETED'dan `dataSync` (ve phoneCall/mediaPlayback/
 * mediaProjection) tipi foreground service baslatmayi yasakliyor ve
 * ForegroundServiceStartNotAllowedException atiyor.
 */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent?) {
        val action = intent?.action ?: return
        if (action != Intent.ACTION_BOOT_COMPLETED &&
            action != Intent.ACTION_MY_PACKAGE_REPLACED
        ) {
            return
        }
        val pw = Prefs.password(context) ?: return
        ContextCompat.startForegroundService(
            context,
            Intent(context, DaemonService::class.java)
                .setAction(DaemonService.ACTION_START)
                .putExtra(DaemonService.EXTRA_PASSWORD, pw),
        )
    }
}
