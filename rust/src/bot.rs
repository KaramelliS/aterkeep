//! Minecraft anti-idle bot yoneticisi.
//!
//! Rust daemon, mevcut Node.js (mineflayer) bot surecini spawn eder/yonetir.
//! Bot `config/bot.json`'dan ayarlarini okur, `config/bot-status.json`'a
//! canli durum yazar. Bu mod sadece surec yonetimi + config persistence yapar;
//! botun icerigi (index.js) ayri bir agent (B3) tarafindan yazilir.

use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

pub use aterkeep_core::BotConfig;

/// PATCH guncellemesi — tum alanlar opsiyonel. Sadece verilen alanlar uzerine yazilir.
/// frontend -> POST /api/bot/config icin.
#[derive(Clone, Debug, Deserialize)]
pub struct BotConfigPatch {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub vanish_x: Option<f64>,
    pub vanish_y: Option<f64>,
    pub vanish_z: Option<f64>,
}

pub struct BotManager {
    config: Mutex<BotConfig>,
    child: Mutex<Option<Child>>,
    repo_root: PathBuf,
}

impl BotManager {
    pub fn new(repo_root: PathBuf) -> Self {
        let config = Self::load_config(&repo_root).unwrap_or_default();
        Self {
            config: Mutex::new(config),
            child: Mutex::new(None),
            repo_root,
        }
    }

    fn config_path(root: &Path) -> PathBuf {
        root.join("config").join("bot.json")
    }

    fn status_path(&self) -> PathBuf {
        self.repo_root.join("config").join("bot-status.json")
    }

    fn load_config(root: &Path) -> Option<BotConfig> {
        let path = Self::config_path(root);
        let raw = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    fn persist_config_locked(cfg: &BotConfig, root: &Path) -> Result<(), String> {
        let path = Self::config_path(root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pretty = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, pretty).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn config(&self) -> BotConfig {
        self.config.lock().await.clone()
    }

    /// Patch'i mevcut config'in uzerine yaz, persist et.
    /// Eger host/port/name degisti VE bot su an calisiyorsa restart yapar.
    pub async fn update_config(&self, patch: BotConfigPatch) -> Result<(), String> {
        let changed = {
            let mut cfg = self.config.lock().await;
            let prev = cfg.clone();
            if let Some(v) = patch.name {
                cfg.name = v;
            }
            if let Some(v) = patch.host {
                cfg.host = v;
            }
            if let Some(v) = patch.port {
                cfg.port = v;
            }
            if let Some(v) = patch.vanish_x {
                cfg.vanish_x = v;
            }
            if let Some(v) = patch.vanish_y {
                cfg.vanish_y = v;
            }
            if let Some(v) = patch.vanish_z {
                cfg.vanish_z = v;
            }
            Self::persist_config_locked(&cfg, &self.repo_root)?;
            
            cfg.name != prev.name || cfg.host != prev.host || cfg.port != prev.port
            // NOT: `child` kilidi BURADA ALINMAZ. Alinsaydi kilit sirasi
            // config -> child olurdu; start() ise child -> config sirasiyla
            // aliyor. Iki yol ayni anda calistiginda bu klasik kilit sirasi
            // tersligi bot yoneticisini ve keepalive'i KALICI olarak
            // kitleyebilirdi. Config kilidi burada birakiliyor.
        };
        let was_running = self.is_running().await;
        let need_restart = changed && was_running;
        if need_restart && was_running {
            let cfg = self.config().await;
            self.stop().await;
            self.start(cfg.host.clone(), cfg.port).await?;
        }
        Ok(())
    }

    pub async fn enabled(&self) -> bool {
        self.config.lock().await.enabled
    }

    /// enabled bayragini gunceller ve config'i persist eder.
    /// Bot surecini baslatmaz/durdurmaz — sadece tercih kaydedilir.
    pub async fn set_enabled(&self, on: bool) -> Result<(), String> {
        let mut cfg = self.config.lock().await;
        cfg.enabled = on;
        let snapshot = cfg.clone();
        drop(cfg);
        Self::persist_config_locked(&snapshot, &self.repo_root)
    }

    /// Bot surecini baslatir.
    /// - Zaten calisiyorsa no-op (ama host/port degisse config'i gunceller).
    /// - `node bot/index.js` cwd=repo_root, env ATERKEEP_BOT_DIR=repo_root ile spawn.
    pub async fn start(&self, host: String, port: u16) -> Result<(), String> {
        // host/port degistiyse once config'i guncelle + persist et. Log icin
        // gereken degerler BURADA yerel degiskenlere alinir; child kilidi
        // altindayken config kilidini almak (child -> config) update_config'in
        // sirasiyla (config -> child) ters duser ve kilitlenme uretir.
        let (log_host, log_port, log_name) = {
            let mut cfg = self.config.lock().await;
            if cfg.host != host || cfg.port != port {
                cfg.host = host;
                cfg.port = port;
                let snapshot = cfg.clone();
                Self::persist_config_locked(&snapshot, &self.repo_root)?;
            }
            (cfg.host.clone(), cfg.port, cfg.name.clone())
        };

        let mut child_guard = self.child.lock().await;
        if let Some(child) = child_guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => { /* exited — asagida yeniden spawn */ }
                Ok(None) => {
                    // hala calisiyor — no-op
                    return Ok(());
                }
                Err(_) => { /* durum okunamadi — re-spawn guvenli */ }
            }
        }
        // Exited/none: (re)spawn.
        // Config degerleri child kilidi ALINMADAN once okunmus olmali; burada
        // `self.config()` cagirmak child -> config sirasi yaratir ve
        // update_config'in ters sirasiyla kilitlenme riski dogururdu.
        let child = self.spawn()?;
        *child_guard = Some(child);
        eprintln!(
            "[bot] spawn edildi: node bot/index.js (host={log_host}, port={log_port}, name={log_name})"
        );
        Ok(())
    }

    /// bot/bot.log'u ekleme modunda acar ve stdout+stderr icin iki tutamac doner.
    /// Dosya 2 MB'i gecerse sifirlanir — gunlerce calisan bir daemon'da log
    /// sinirsiz buyumemeli.
    fn open_bot_log(root: &Path) -> Option<(std::process::Stdio, std::process::Stdio)> {
        let path = root.join("bot").join("bot.log");
        if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > 2 * 1024 * 1024 {
            let _ = std::fs::write(&path, b"");
        }
        let f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok()?;
        let f2 = f.try_clone().ok()?;
        Some((std::process::Stdio::from(f), std::process::Stdio::from(f2)))
    }

    fn spawn(&self) -> Result<Child, String> {
        let root_str = self.repo_root.to_string_lossy().to_string();
        let mut cmd = tokio::process::Command::new("node");
        cmd.arg("bot/index.js")
            .current_dir(&self.repo_root)
            .env("ATERKEEP_BOT_DIR", &root_str)
            // Master parolayi cocuk surece MIRAS BIRAKMA. Bot, mineflayer ile
            // birlikte genis bir npm bagimlilik agaci calistiriyor; oradaki
            // herhangi bir paket `process.env` okuyabilir. Botun bu parolaya
            // hicbir ihtiyaci yok.
            .env_remove("ATERKEEP_KEY");
        // Bot ciktisi bir dosyaya yazilir. Onceden /dev/null'a gidiyordu: bot
        // sunucudan atildiginda ya da baglanamadiginda geriye hicbir iz kalmiyor,
        // "bot girmiyor" sikayetinin sebebini gormek imkansiz oluyordu. Daemon'in
        // kendi loguna karismasin diye ayri dosya.
        let (out, err) = match Self::open_bot_log(&self.repo_root) {
            Some(pair) => pair,
            None => (std::process::Stdio::null(), std::process::Stdio::null()),
        };
        cmd.stdout(out).stderr(err);
        // kill_on_drop: BotManager drop olursa child da olmesi icin sinyal
        // (daemon cikinca bot da ciksin). Windows'ta drop -> kill degildir ama
        // stop() acik kill cagirir.
        cmd.kill_on_drop(true);
        cmd.spawn().map_err(|e| format!("node spawn hatasi: {e}"))
    }

    /// Bot surecini oldurur ve handle'i temizler.
    pub async fn stop(&self) {
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            // once normal kill (SIGTERM/Linux, TerminateProcess/Windows)
            let _ = child.start_kill();
            // best-effort bekle — surec olmezse orphan kalir ama daemon cikinca
            // kill_on_drop tetiklenir.
            let _ = tokio::time::timeout(std::time::Duration::from_secs(2), child.wait()).await;
        }
        eprintln!("[bot] durduruldu");
    }

    /// Bot sureci hala yasiyor mu? (try_wait: bazilari exited olabilir)
    pub async fn is_running(&self) -> bool {
        let mut child_guard = self.child.lock().await;
        if let Some(child) = child_guard.as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    // exited — handle'i temizle
                    *child_guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// config/bot-status.json'i best-effort okur + parse eder.
    /// Dosya yoksa/gecersizse {connected:false, state:"stopped"} doner.
    pub async fn read_status(&self) -> Value {
        let path = self.status_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| {
                json!({ "connected": false, "state": "stopped" })
            }),
            Err(_) => json!({ "connected": false, "state": "stopped" }),
        }
    }

    /// API'nin dondugu birlesik durum nesnesi.
    /// bot-status.json + mevcut config (enabled/host/port/name) birlestirir.
    /// Sabit sema (frontend sozlesmesi):
    /// { enabled, state, name, host, port, vanished,
    ///   server_version, max_supported_version, error }
    pub async fn status(&self) -> Value {
        let cfg = self.config().await;
        let file_status = self.read_status().await;
        // Eger surec hic yoksa (kapali), file_status ne derse desin state'i
        // "stopped" olarak goster — stale veriyi maskele.
        let running = self.is_running().await;
        // "kapali" ile "etkin ama sunucu bekliyor" AYNI SEY DEGIL. Bot sureci
        // yalnizca sunucu online oldugunda yasar (keepalive sunucu kapanirken
        // botu durdurur), dolayisiyla enabled=true + running=false NORMAL bir
        // durumdur. Ikisini "stopped" olarak gostermek kullaniciya "botu actim
        // ama offline yaziyor" diye yansiyor — panele hangi durumda oldugunu
        // dogru soylemek icin ayri bir state veriyoruz.
        let state = if !running {
            if cfg.enabled {
                "waiting_server".to_string()
            } else {
                "stopped".to_string()
            }
        } else {
            file_status
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("stopped")
                .to_string()
        };
        let server_version = file_status
            .get("server_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let max_supported_version = file_status
            .get("max_supported_version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.21.11")
            .to_string();
        let vanished = file_status
            .get("vanished")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let error = if !running {
            None
        } else {
            file_status
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        };
        // Bot, cozumu bilinen hatalari makine tarafindan okunabilir bir kodla
        // bildirir (orn. "online_mode"). Panel bunu kullanicinin dilinde ve ne
        // yapmasi gerektigini soyleyerek gosterir; ham kick metni degil.
        let error_code = file_status
            .get("error_code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        json!({
            "enabled": cfg.enabled,
            // Panel "etkin mi?" ile "sureci yasiyor mu?" ayrimini gosterir.
            "running": running,
            "node_available": node_available(),
            "state": state,
            "name": cfg.name,
            "host": cfg.host,
            "port": cfg.port,
            "vanished": vanished,
            "server_version": server_version,
            "max_supported_version": max_supported_version,
            "error": error,
            "error_code": if running { error_code } else { None },
        })
    }
}

/// `node` calistirilabilir durumda mi? Bot Node.js gerektirir (mineflayer);
/// yoksa spawn sessizce basarisiz olur ve panelde sebebi gorunmez. Panel bu
/// bilgiyi gosterip kullaniciyi "npm install"e yonlendirir.
///
/// Sonuc process omru boyunca cache'lenir — panel bu ucu 4 saniyede bir
/// cagiriyor, her seferinde surec spawn etmek israf olur.
pub fn node_available() -> bool {
    static CACHE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHE.get_or_init(|| {
        std::process::Command::new("node")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

/// Kolaylik: AppCtx ve keepalive her yerde Arc<BotManager> tasiyor.
pub type SharedBot = Arc<BotManager>;

#[async_trait::async_trait]
impl aterkeep_core::BotRunner for BotManager {
    async fn is_running(&self) -> bool {
        self.is_running().await
    }
    async fn enabled(&self) -> bool {
        self.enabled().await
    }
    async fn config(&self) -> aterkeep_core::BotConfig {
        self.config().await
    }
    async fn start(&self, host: String, port: u16) -> Result<(), String> {
        self.start(host, port).await
    }
    async fn stop(&self) {
        self.stop().await
    }
}
