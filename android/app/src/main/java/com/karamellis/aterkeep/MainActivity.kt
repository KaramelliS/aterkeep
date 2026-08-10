package com.karamellis.aterkeep

import android.Manifest
import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.PowerManager
import android.provider.Settings
import android.view.View
import android.webkit.WebView
import android.widget.Button
import android.widget.CheckBox
import android.widget.EditText
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import java.net.InetSocketAddress
import java.net.Socket

/**
 * Tek ekran: parola kapisi, sonra panelin kendisi.
 *
 * Arayuz YENIDEN YAZILMADI — daemon zaten 14 dilli, tam ozellikli bir web
 * paneli servis ediyor. Uygulama onu WebView'de gosteriyor; boylece panelde
 * yapilan her iyilestirme telefonda da bedava geliyor ve iki arayuzun birbirinden
 * ayrisma riski hic olusmuyor.
 */
class MainActivity : AppCompatActivity() {

    private lateinit var web: WebView
    private lateinit var gate: ScrollView
    private lateinit var status: TextView
    private val handler = Handler(Looper.getMainLooper())

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        web = findViewById(R.id.web)
        gate = findViewById(R.id.gate)
        status = findViewById(R.id.status)
        val password = findViewById<EditText>(R.id.password)
        val remember = findViewById<CheckBox>(R.id.remember)
        val startBtn = findViewById<Button>(R.id.startBtn)
        val batteryNote = findViewById<TextView>(R.id.batteryNote)
        val batteryBtn = findViewById<Button>(R.id.batteryBtn)

        web.settings.javaScriptEnabled = true
        web.settings.domStorageEnabled = true

        if (!Daemon.abiSupported()) {
            status.text = getString(R.string.unsupported_abi)
            startBtn.isEnabled = false
            return
        }

        requestNotificationPermission()

        // Pil muafiyeti: wake lock tek basina yetmiyor, Doze yine de kisiyor.
        if (!isIgnoringBatteryOptimizations()) {
            batteryNote.visibility = View.VISIBLE
            batteryBtn.visibility = View.VISIBLE
            batteryBtn.setOnClickListener { requestBatteryExemption() }
        }

        startBtn.setOnClickListener {
            val pw = password.text.toString()
            if (pw.isEmpty()) {
                Toast.makeText(this, R.string.password_hint, Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }
            if (remember.isChecked) Prefs.savePassword(this, pw) else Prefs.forget(this)
            launch(pw)
        }

        // Parola hatirlanmissa kapida bekletmenin anlami yok.
        val saved = Prefs.password(this)
        if (saved != null) {
            remember.isChecked = true
            launch(saved)
        }
    }

    private fun launch(pw: String) {
        val intent = Intent(this, DaemonService::class.java)
            .setAction(DaemonService.ACTION_START)
            .putExtra(DaemonService.EXTRA_PASSWORD, pw)
        ContextCompat.startForegroundService(this, intent)
        status.text = getString(R.string.waiting_panel)
        waitForPanel(attempt = 0)
    }

    /**
     * Panel ayaga kalkana kadar bekler.
     *
     * Neden yoklama: daemon'in dinlemeye baslamasi anlik degil (oturum cozumu
     * PBKDF2 ile 600.000 tur suruyor ve ilk aciliste kurulum modu devreye
     * girebiliyor). WebView'i hemen yuklemek ERR_CONNECTION_REFUSED gosterirdi.
     */
    private fun waitForPanel(attempt: Int) {
        if (portOpen()) {
            gate.visibility = View.GONE
            web.visibility = View.VISIBLE
            web.loadUrl(Daemon.PANEL_URL)
            return
        }
        // ~60 saniye (120 x 500ms). Bundan sonrasi gercekten bir hatadir.
        if (attempt > 120) {
            status.text = getString(R.string.panel_failed) + "\n\n" + tailLog()
            return
        }
        val last = DaemonService.lastLine
        if (last.isNotBlank()) status.text = last
        handler.postDelayed({ waitForPanel(attempt + 1) }, 500)
    }

    private fun portOpen(): Boolean = try {
        Socket().use {
            it.connect(InetSocketAddress("127.0.0.1", Daemon.PANEL_PORT), 300)
            true
        }
    } catch (_: Exception) {
        false
    }

    private fun tailLog(): String = try {
        val f = Daemon.logFile(this)
        if (!f.exists()) "" else f.readLines().takeLast(15).joinToString("\n")
    } catch (_: Exception) {
        ""
    }

    private fun isIgnoringBatteryOptimizations(): Boolean {
        val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
        return pm.isIgnoringBatteryOptimizations(packageName)
    }

    @SuppressLint("BatteryLife")
    private fun requestBatteryExemption() {
        // Muafiyet KULLANICI onayiyla verilir; sistem diyalogunu aciyoruz.
        try {
            startActivity(
                Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS)
                    .setData(Uri.parse("package:$packageName")),
            )
        } catch (_: Exception) {
            // Bazi ROM'lar bu intent'i tasimiyor; genel pil ayarlarina dus.
            try {
                startActivity(Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS))
            } catch (_: Exception) {
            }
        }
    }

    private fun requestNotificationPermission() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
            == PackageManager.PERMISSION_GRANTED
        ) {
            return
        }
        // Bildirim reddedilirse servis calismaya devam eder ama kullanici
        // durumu goremez; bu yuzden istiyoruz, sart kosmuyoruz.
        requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 42)
    }

    @Deprecated("Basit geri gezinme icin yeterli")
    override fun onBackPressed() {
        if (web.visibility == View.VISIBLE && web.canGoBack()) {
            web.goBack()
        } else {
            @Suppress("DEPRECATION")
            super.onBackPressed()
        }
    }
}
