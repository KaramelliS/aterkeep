//! aterkeep — Aternos 7/24 sunucu bekcisi.
//!
//! Bu dosya YALNIZCA baslangici baglar: komutlari dagitir, oturumu acar,
//! paylasilan durumu kurar, arka plan gorevlerini baslatir ve paneli dinlemeye
//! sokar. Is mantigi modullerde:
//!
//!   unlock    parola alma, oturumun cozulmesi
//!   cli       surum/yardim, `import` komutu
//!   wizard    terminal kurulum sihirbazi
//!   watch     oturum omru olcumu + otomatik yeniden giris
//!   web/      HTTP katmani (kendi icinde ayrilmis)
//!   bot       anti-idle bot sureci
//!   config    dosya duzeni ve parola tabanli anahtar turetme

mod bot;
mod cli;
mod config;
mod mcbot;
mod translations;
mod unlock;
mod watch;
mod web;
mod wizard;

use aterkeep_core::{run_keepalive, LogTx, SharedState, State};
use config::AppConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use cli::cmd_import;
use unlock::{acquire_password, load_session};
use wizard::run_wizard;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Hangi surumun calistigini soylemenin bir yolu olmali: destek talebi
    // gelince "hangi surum?" sorusunun cevabi bulunabilsin.
    if let Some(a) = args.get(1) {
        match a.as_str() {
            "--version" | "-V" | "version" => {
                println!("aterkeep {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" | "help" => {
                println!("aterkeep {} — Aternos 7/24 sunucu bekcisi\n", env!("CARGO_PKG_VERSION"));
                println!("KULLANIM:");
                println!("  aterkeep                 paneli baslat (http://127.0.0.1:4041)");
                println!("  aterkeep import <dosya>  hazir bir session.json'u sifreleyip ice aktar");
                println!("  aterkeep --version       surumu yazdir\n");
                println!("  aterkeep bot-probe <host> [port]  anti-idle botu tek basina calistir");
                println!("ORTAM DEGISKENLERI:");
                println!("  ATERKEEP_KEY   panel parolasi (arka planda calisirken ZORUNLU)");
                println!("  ATERKEEP_DIR   config klasoru (varsayilan: ./config)");
                return;
            }
            _ => {}
        }
    }

    // Botu oturum/panel olmadan, dogrudan bir sunucuya karsi calistirir.
    //
    // NEDEN VAR: bot artik daemon'in icinde oldugu icin "bot girmiyor"
    // sikayetini elle yeniden uretmenin baska yolu yok. Node surumunde
    // `node bot/index.js` calistirilabiliyordu (bkz. docs/BOT.md); bu komut
    // onun yerini aliyor. Ayni zamanda protokol degisikliklerini gercek bir
    // sunucuya karsi dogrulamanin yolu.
    if args.len() > 1 && args[1] == "bot-probe" {
        let host = args.get(2).cloned().unwrap_or_else(|| "127.0.0.1".into());
        let port: u16 = args
            .get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(25565);
        let mut cfg = aterkeep_core::BotConfig::default();
        cfg.host = host.clone();
        cfg.port = port;
        println!("bot-probe: {host}:{port} (Ctrl-C ile cik)");
        match mcbot::ping(&host, port).await {
            Ok(i) => println!(
                "ping: {} (proto {}) — bot max: {} (proto {})",
                i.version_name,
                i.protocol,
                mcbot::MAX_SUPPORTED_VERSION,
                mcbot::MAX_SUPPORTED_PROTOCOL
            ),
            Err(e) => println!("ping basarisiz: {e}"),
        }
        let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        mcbot::run(cfg, dir, stop).await;
        return;
    }
    if args.len() > 1 && args[1] == "import" {
        let path = args.get(2).map(|s| s.as_str()).unwrap_or("session.json");
        match cmd_import(path) {
            Ok(()) => return,
            Err(e) => {
                eprintln!("import hatasi: {e}");
                std::process::exit(1);
            }
        }
    }

    let mut cfg = AppConfig::load();

    // session.enc yoksa:
    //   - interaktif terminal (TTY) → CLI wizard calistir
    //   - degilse (arka plan/service/systemd) → wizard'i atla, panel setup-modunda acilsin
    //     (web wizard /api/setup ile session.enc uretir, sonra self-restart).
    if !config::session_path().exists() && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        println!("=== aterkeep ilk kurulum ===");
        println!("Cerezlerini girmen gerek. aternos.org'da F12 -> Application -> Cookies.");
        if let Err(e) = run_wizard().await {
            eprintln!("kurulum hatasi: {e}");
            std::process::exit(1);
        }
        cfg = AppConfig::load();
    }

    // session.enc hala yoksa → setup modu: client olmadan paneli ac, /api/setup beklenir.
    if !config::session_path().exists() {
        println!("=== aterkeep kurulum modu ===");
        println!(
            "session yok — panel setup sihirbazini aciyor: http://{}:{}",
            cfg.bind, cfg.port
        );
        web::run_setup_mode(cfg).await;
        return;
    }

    // Anahtar diskte tutulmadigi icin oturumu acmak parola gerektirir.
    let password = match acquire_password(&cfg) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("kilit acilamadi: {e}");
            std::process::exit(1);
        }
    };

    let (sess, session_key_val) = match load_session(&cfg, &password) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("session yuklenemedi: {e}");
            eprintln!("parola yanlis olabilir; ya da: aterkeep import session.json");
            std::process::exit(1);
        }
    };

    // Adresi client'a move etmeden once al — State cache'ine koyacagiz.
    let initial_addr = sess.server_addr.clone();

    let client = Arc::new(match aterkeep_core::new_client(sess) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("client kurulamadi: {e}");
            std::process::exit(1);
        }
    });

    // Bot yoneticisi: repo_root = mevcut calisma dizini (cwd). Calisma dosyalari
    // config/ altinda tutuluyor; bot/ dizini de daemon'un baslatildigi yerde olmali.
    // (current_exe() kullanma — release build'de bu target/release/ olur, bot/
    // orada DEGIL, daemon'un baslatildigi dizinde.)
    let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let bot: bot::SharedBot = Arc::new(bot::BotManager::new(repo_root));

    let state: SharedState = Arc::new(Mutex::new(State {
        // Kullanicinin son tercihi. Sabit `true` idi: her yeniden baslatma,
        // kullanicinin bilerek kapattigi 7/24 modunu sessizce geri aciyordu.
        auto: cfg.auto_start,
        server_state: "boot".into(),
        status_num: -1,
        last_check: 0,
        last_poll: 0,
        busy: false,
        last_request: None,
        server_addr: initial_addr,
        queue: None,
        last_zombie_break: 0,
        last_extend: 0,
        session_expired: false,
        ws_connected: false,
        players: 0,
        slots: 20,
        playerlist: serde_json::json!([]),
        label: String::new(),
        tps: 0.0,
        heap: 0,
    }));
    let (tx, _) = tokio::sync::broadcast::channel::<serde_json::Value>(512);
    let tx: LogTx = tx;

    // WS canli akis console buffer'i (Hermes tarafindan doldurulur, panelde gosterilir)
    let console: aterkeep_core::SharedConsole = Arc::new(Mutex::new(aterkeep_core::ConsoleBuf {
        lines: Vec::new(),
        cap: 400,
    }));

    let ctx = Arc::new(web::AppCtx {
        client: client.clone(),
        state: state.clone(),
        tx: tx.clone(),
        bot: bot.clone(),
        console: console.clone(),
        cfg: Mutex::new(cfg.clone()),
        auth_token: Mutex::new(None),
    });

    // keepalive dongusu: yalnizca durum okur; start/stop kararlarini o verir.
    // Zombie probe hedefi ARTIK parametre degil — canli State.server_addr'dan
    // okunuyor (bkz. core/keepalive.rs).
    tokio::spawn(run_keepalive(
        client.clone(),
        state.clone(),
        tx.clone(),
        bot.clone(),
        30,
    ));

    // WS canli akis: Aternos Hermes WebSocket'ine baglan, anlik durum/kuyruk/console
    // guncellemelerini state + console buffer'a besle. keepalive'den bagimsiz calisir.
    tokio::spawn(aterkeep_core::run_hermes(
        client.clone(),
        state.clone(),
        console.clone(),
        tx.clone(),
    ));

    // Oturum omru olcumu + otomatik yeniden giris (bkz. watch.rs).
    watch::spawn(watch::Deps {
        state: state.clone(),
        ctx: ctx.clone(),
        tx: tx.clone(),
        session_key: session_key_val,
        panel_password: password.clone(),
    });

    let app = web::router(ctx);
    let addr = format!("{}:{}", cfg.bind, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("{addr} dinlenemiyor: {e}"));
    println!("aterkeep panel: http://{addr}");
    if cfg.bind != "127.0.0.1" && cfg.bind != "localhost" {
        println!("[!] UYARI: panel disariya acik ({}). Panel parolasi tek", cfg.bind);
        println!("[!] savunmadir — guclu bir parola kullan ve mumkunse VPN arkasina al.");
    }
    axum::serve(listener, app).await.expect("server hatasi");
}
