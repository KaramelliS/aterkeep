//! Minecraft anti-idle bot yoneticisi.
//!
//! Bot artik AYRI BIR SUREC DEGIL: protokol istemcisi daemon'in icinde
//! (`crate::mcbot`) ve burada bir tokio gorevi olarak yasiyor. Eskiden
//! `node bot/index.js` spawn ediliyordu; Node bir APK'nin icine sigmadigi icin
//! bot telefonda hic calismiyor, masaustunde de Node kurulumu + genis bir npm
//! agaci gerektiriyordu.
//!
//! Bu mod ayarlarin kalici yazimini (`config/bot.json`) ve gorev yasam
//! dongusunu yonetir; canli durum yine `config/bot-status.json`'a yazilir
//! (panel sozlesmesi degismedi).

use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub use aterkeep_core::BotConfig;

/// Calisan bot gorevi ve onu durdurmak icin bayrak.
struct Running {
    handle: JoinHandle<()>,
    stop: Arc<AtomicBool>,
}

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
    running: Mutex<Option<Running>>,
    repo_root: PathBuf,
}

impl BotManager {
    pub fn new(repo_root: PathBuf) -> Self {
        let config = Self::load_config(&repo_root).unwrap_or_default();
        Self {
            config: Mutex::new(config),
            running: Mutex::new(None),
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

    /// Bot gorevini baslatir.
    /// - Zaten calisiyorsa no-op (ama host/port degisse config'i gunceller).
    /// - Yerel protokol istemcisini (`crate::mcbot::run`) bir tokio gorevi olarak
    ///   baslatir. Harici surec, Node ya da npm YOK.
    pub async fn start(&self, host: String, port: u16) -> Result<(), String> {
        // host/port degistiyse once config'i guncelle + persist et. Goreve
        // gereken TUM degerler BURADA, running kilidi ALINMADAN once aliniyor:
        // running kilidi altindayken config kilidini almak (running -> config)
        // update_config'in sirasiyla (config -> running) ters duser ve
        // kilitlenme uretir.
        let cfg = {
            let mut cfg = self.config.lock().await;
            if cfg.host != host || cfg.port != port {
                cfg.host = host;
                cfg.port = port;
                let snapshot = cfg.clone();
                Self::persist_config_locked(&snapshot, &self.repo_root)?;
            }
            cfg.clone()
        };
        let (log_host, log_port, log_name) = (cfg.host.clone(), cfg.port, cfg.name.clone());

        let mut guard = self.running.lock().await;
        if let Some(r) = guard.as_ref() {
            if !r.handle.is_finished() {
                // hala calisiyor — no-op
                return Ok(());
            }
        }

        let stop = Arc::new(AtomicBool::new(false));
        let handle = tokio::spawn(crate::mcbot::run(
            cfg,
            self.repo_root.clone(),
            stop.clone(),
        ));
        *guard = Some(Running { handle, stop });
        eprintln!(
            "[bot] baslatildi (host={log_host}, port={log_port}, name={log_name})"
        );
        Ok(())
    }

    /// Bot gorevini durdurur.
    ///
    /// Once bayragi set edip gorevin KENDI kapanmasini bekliyoruz; bot bu
    /// sirada durumu `stopped` yaziyor ve varsa acik soketi kapatiyor. Dogrudan
    /// `abort()` etmek bunlari atlar ve panelde eski durum asili kalir.
    /// Bekleme sinirli: takilirsa yine de abort ediyoruz.
    pub async fn stop(&self) {
        let mut guard = self.running.lock().await;
        if let Some(r) = guard.take() {
            r.stop.store(true, Ordering::Relaxed);
            if tokio::time::timeout(std::time::Duration::from_secs(3), r.handle)
                .await
                .is_err()
            {
                eprintln!("[bot] gorev zamaninda kapanmadi");
            }
        }
        eprintln!("[bot] durduruldu");
    }

    /// Bot gorevi hala yasiyor mu?
    pub async fn is_running(&self) -> bool {
        let mut guard = self.running.lock().await;
        match guard.as_ref() {
            Some(r) if !r.handle.is_finished() => true,
            Some(_) => {
                // bitmis — handle'i temizle
                *guard = None;
                false
            }
            None => false,
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
            // Panel "etkin mi?" ile "gorev yasiyor mu?" ayrimini gosterir.
            "running": running,
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

// node_available() KALDIRILDI: bot artik daemon'in icinde, Node.js gerekmiyor.
// Panelde "Node.js: mevcut/EKSIK" satiri da kaldirildi — her zaman "mevcut"
// yazan bir satir, satiri hic gostermemekten daha kotuydu.

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
