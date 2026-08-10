package com.karamellis.aterkeep

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Daemon'i gozeten foreground service.
 *
 * NEDEN FOREGROUND SERVICE: Android'de bir arka plan surecini 7/24 yasatmanin
 * guvenilir tek yolu bu. Termux + `termux-wake-lock` yetmiyor — Android 12'den
 * beri "phantom process" avcisi, uygulamanin izlenmeyen cocuk sureclerini
 * (ozellikle CPU yakanlari) olduruyor; Android 15'te wake-lock'a ragmen bu hala
 * boyle. Kalici bildirimi olan bir foreground service ise sistemin gozunde
 * kullanici tarafindan baslatilmis ve gorunur bir istir.
 *
 * Surec agaci ONEMLI: daemon bu servisin DOGRUDAN cocugu olarak kalmali.
 * Kurulum sonrasi daemon kendini `exec` ile yeniliyor (bkz. rust/web/setup.rs) —
 * exec PID'i korudugu icin buradaki `Process` handle'i gecerli kalir. Eskiden
 * spawn+exit yapiliyordu; o, surecin agactan kopmasina ve hem cift kopya hem
 * phantom-kill riskine yol aciyordu.
 */
class DaemonService : Service() {

    companion object {
        const val ACTION_START = "com.karamellis.aterkeep.START"
        const val ACTION_STOP = "com.karamellis.aterkeep.STOP"
        const val EXTRA_PASSWORD = "password"

        private const val CHANNEL_ID = "aterkeep_daemon"
        private const val NOTIF_ID = 1001
        private const val LOG_CAP_BYTES = 512L * 1024L

        /** UI'nin servise sormasi icin; surec gercekten yasiyor mu. */
        @Volatile
        var alive: Boolean = false
            private set

        /** Panelde/kapida gosterilecek son anlamli satir. */
        @Volatile
        var lastLine: String = ""
            private set
    }

    private val stopRequested = AtomicBoolean(false)
    private var proc: Process? = null
    private var supervisor: Thread? = null
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        createChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            shutdown()
            return START_NOT_STICKY
        }

        val password = intent?.getStringExtra(EXTRA_PASSWORD)
            ?: Prefs.password(this)
            // Parola yoksa oturum acilamaz; sessizce calisan ama hicbir sey
            // yapamayan bir servis birakmak yerine hemen duruyoruz.
            ?: run {
                shutdown()
                return START_NOT_STICKY
            }

        startForegroundNow(getString(R.string.waiting_panel))

        if (supervisor?.isAlive == true) return START_STICKY

        acquireWakeLock()
        stopRequested.set(false)
        supervisor = Thread({ supervise(password) }, "aterkeep-supervisor").apply {
            isDaemon = false
            start()
        }
        return START_STICKY
    }

    override fun onDestroy() {
        shutdown()
        super.onDestroy()
    }

    /**
     * Daemon'i baslatir, ciktiyi kayda alir, olurse geri getirir.
     *
     * Yeniden baslatma araligi artan (backoff) sekilde: derhal ve sonsuz
     * yeniden deneme, kalici bir hatada (orn. binary calistirilamiyor) pili
     * bitiren sonsuz bir dongude sonuclanirdi.
     */
    private fun supervise(password: String) {
        val bin = Daemon.daemonBin(this)
        if (!bin.canExecute()) {
            log("[servis] daemon calistirilamiyor: ${bin.absolutePath}")
            updateNotification("Hata: daemon calistirilamiyor")
            return
        }
        Daemon.dataDir(this).mkdirs()
        try {
            Daemon.installCaBundle(this)
        } catch (e: Exception) {
            log("[servis] cacert.pem acilamadi: ${e.message}")
        }

        var backoffMs = 2_000L
        while (!stopRequested.get()) {
            log("[servis] daemon baslatiliyor")
            val started = System.currentTimeMillis()
            try {
                val pb = ProcessBuilder(bin.absolutePath)
                    .directory(Daemon.workDir(this))
                    .redirectErrorStream(true)
                pb.environment().putAll(Daemon.env(this, password))
                val p = pb.start()
                proc = p
                alive = true

                // Cikti okunmazsa boru dolar ve daemon yazarken BLOKE olur.
                // Yani bu okuma sadece kayit tutmak icin degil, surecin
                // yasamasi icin gerekli.
                BufferedReader(InputStreamReader(p.inputStream)).use { r ->
                    var line = r.readLine()
                    while (line != null) {
                        log(line)
                        if (line.isNotBlank()) {
                            lastLine = line
                            updateNotification(line.take(120))
                        }
                        line = r.readLine()
                    }
                }
                val code = p.waitFor()
                alive = false
                log("[servis] daemon cikti (kod $code)")
            } catch (e: Exception) {
                alive = false
                log("[servis] daemon hatasi: ${e.message}")
            }

            if (stopRequested.get()) break

            // Uzun sure ayakta kaldiysa bu gecici bir cokmedir; backoff'u
            // sifirla ki tek bir kotu gunun ardindan dakikalarca beklemeyelim.
            if (System.currentTimeMillis() - started > 60_000) backoffMs = 2_000L

            updateNotification("Yeniden baslatiliyor (${backoffMs / 1000} sn)")
            try {
                Thread.sleep(backoffMs)
            } catch (_: InterruptedException) {
                break
            }
            backoffMs = (backoffMs * 2).coerceAtMost(60_000L)
        }
        alive = false
    }

    private fun shutdown() {
        stopRequested.set(true)
        supervisor?.interrupt()
        proc?.destroy()
        proc = null
        alive = false
        releaseWakeLock()
        ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    /**
     * Partial wake lock: ekran kapaliyken de CPU'yu ayakta tutar.
     * Keep-alive 30 saniyede bir yokluyor; CPU uykuya girerse yoklama durur ve
     * Aternos sunucuyu bos sayip kapatir.
     *
     * Wake lock TEK BASINA yetmez — pil optimizasyonu muafiyeti de gerekiyor,
     * onu kullanicidan MainActivity istiyor.
     */
    private fun acquireWakeLock() {
        if (wakeLock?.isHeld == true) return
        val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "aterkeep:daemon").apply {
            setReferenceCounted(false)
            acquire()
        }
    }

    private fun releaseWakeLock() {
        if (wakeLock?.isHeld == true) wakeLock?.release()
        wakeLock = null
    }

    private fun createChannel() {
        val nm = getSystemService(NotificationManager::class.java)
        val ch = NotificationChannel(
            CHANNEL_ID,
            getString(R.string.notif_channel_name),
            // LOW: kalici bildirim gerekli ama her guncellemede ses/titreme
            // cikarmasi kullaniciyi bogar.
            NotificationManager.IMPORTANCE_LOW,
        ).apply {
            description = getString(R.string.notif_channel_desc)
            setShowBadge(false)
        }
        nm.createNotificationChannel(ch)
    }

    private fun buildNotification(text: String): Notification {
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val stop = PendingIntent.getService(
            this,
            1,
            Intent(this, DaemonService::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(getString(R.string.notif_title))
            .setContentText(text)
            .setStyle(NotificationCompat.BigTextStyle().bigText(text))
            .setSmallIcon(R.drawable.ic_stat_aterkeep)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setContentIntent(open)
            .addAction(0, getString(R.string.notif_stop), stop)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
    }

    private fun startForegroundNow(text: String) {
        ServiceCompat.startForeground(
            this,
            NOTIF_ID,
            buildNotification(text),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
            } else {
                0
            },
        )
    }

    private var lastNotifAt = 0L

    private fun updateNotification(text: String) {
        // Daemon saniyede birkac satir yazabiliyor; her satirda bildirim
        // guncellemek sistem tarafindan kisilmaya (rate limit) yol acar.
        val now = System.currentTimeMillis()
        if (now - lastNotifAt < 2_000) return
        lastNotifAt = now
        getSystemService(NotificationManager::class.java)
            .notify(NOTIF_ID, buildNotification(text))
    }

    private fun log(line: String) {
        val f: File = Daemon.logFile(this)
        try {
            // Kayit dosyasi sinirsiz buyurse cihazin depolamasini yer.
            if (f.length() > LOG_CAP_BYTES) {
                val tail = f.readBytes().takeLast((LOG_CAP_BYTES / 2).toInt()).toByteArray()
                f.writeBytes(tail)
            }
            f.appendText(line + "\n")
        } catch (_: Exception) {
            // Kayit tutamamak servisi durdurmak icin sebep degil.
        }
    }
}
