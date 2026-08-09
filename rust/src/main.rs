mod bot;
mod config;
mod translations;
mod web;

use aterkeep_core::{run_keepalive, Cookie, LogTx, Session, SharedState, State};
use config::AppConfig;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Oturumu acacak parolayi bulur.
///
/// Sirasiyla: `ATERKEEP_KEY` ortam degiskeni -> interaktif terminalde sorma.
/// Arka planda (servis/systemd) calisirken terminal olmadigi icin ortam
/// degiskeni ZORUNLUDUR — anahtar diskte tutulmadigindan baska kaynak yok.
fn acquire_password(cfg: &AppConfig) -> Result<String, String> {
    if let Ok(p) = std::env::var("ATERKEEP_KEY") {
        if !p.is_empty() {
            return Ok(p);
        }
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Err(
            "parola yok: ATERKEEP_KEY ortam degiskenini ayarla (arka planda calisirken sorulamaz)"
                .into(),
        );
    }
    for attempt in 0..3 {
        let p = prompt("Panel parolasi");
        if p.is_empty() {
            continue;
        }
        if cfg.verify_password(&p) {
            return Ok(p);
        }
        eprintln!("[!] parola hatali ({}/3)", attempt + 1);
    }
    Err("parola 3 kez hatali girildi".into())
}

fn load_session(cfg: &AppConfig, password: &str) -> Result<(Session, [u8; 32]), String> {
    let key = cfg
        .derive_session_key(password)
        .ok_or("kurulum tamamlanmamis (config/aterkeep.json icinde auth yok)")?;
    let sess = Session::load_encrypted(&config::session_path(), &key)?;
    Ok((sess, key))
}

/// Kurulum icin parola alir ve config'e auth kaydi yazar; oturum anahtarini
/// dondurur. Mevcut bir kurulum varsa parolayi dogrular (yeniden uretmez —
/// yoksa eski session.enc cozulemez hale gelirdi).
fn setup_password(cfg: &mut AppConfig) -> Result<[u8; 32], String> {
    if cfg.auth_enabled() {
        let p = acquire_password(cfg)?;
        return cfg
            .derive_session_key(&p)
            .ok_or_else(|| "anahtar turetilemedi".into());
    }
    let p = match std::env::var("ATERKEEP_KEY") {
        Ok(p) if !p.is_empty() => p,
        _ => {
            println!("\nPanel parolasi belirle. Bu parola hem paneli korur hem de");
            println!("oturumunu sifreler — anahtar diske YAZILMAZ, kaybedersen oturum gider.");
            let p = prompt("Yeni parola");
            if p.len() < 4 {
                return Err("parola en az 4 karakter olmali".into());
            }
            let again = prompt("Parola (tekrar)");
            if p != again {
                return Err("parolalar eslesmiyor".into());
            }
            p
        }
    };
    let (auth, key) = config::new_auth(&p);
    cfg.auth = Some(auth);
    cfg.save()?;
    Ok(key)
}

fn cmd_import(json_path: &str) -> Result<(), String> {
    let mut cfg = AppConfig::load();
    let key = setup_password(&mut cfg)?;
    let raw = std::fs::read_to_string(json_path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let cookies: Vec<Cookie> = v
        .get("cookies")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    Some(Cookie {
                        name: c.get("name")?.as_str()?.to_string(),
                        value: c.get("value")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let sess = Session {
        token: v.get("token").and_then(|t| t.as_str()).unwrap_or("").to_string(),
        sec: v.get("sec").and_then(|t| t.as_str()).unwrap_or("").to_string(),
        cookies,
        server_id: v
            .get("server_id")
            .or_else(|| v.pointer("/cookies/0/value"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        server_addr: v
            .get("server_addr")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string()),
    };
    // server_id fallback: ATERNOS_SERVER cookie
    let strings = aterkeep_core::Strings::decrypt_all()?;
    let sid = sess
        .cookies
        .iter()
        .find(|c| c.name == strings.c_server)
        .map(|c| c.value.clone())
        .unwrap_or(sess.server_id.clone());
    let sess = Session {
        server_id: sid,
        ..sess
    };
    sess.save_encrypted(&config::session_path(), &key)?;
    println!("{} yazildi (AES-256-GCM)", config::session_path().display());
    Ok(())
}

fn prompt(label: &str) -> String {
    use std::io::{self, Write};
    print!("{label}: ");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    s.trim().to_string()
}

fn parse_cookies(raw: &str) -> Vec<Cookie> {
    raw.split(';')
        .filter_map(|pair| {
            let pair = pair.trim();
            let (name, value) = pair.split_once('=')?;
            let name = name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(Cookie {
                name,
                value: value.trim().to_string(),
            })
        })
        .collect()
}

async fn run_wizard() -> Result<(), String> {
    let mut cfg = AppConfig::load();

    // Dil secimi — panel bu dille acilir, config'e yazilir.
    let codes: Vec<&str> = translations::LANGS.iter().map(|(c, _)| *c).collect();
    println!("\nDiller: {}", codes.join(", "));
    let lang = prompt(&format!("Panel dili [{}]", cfg.lang));
    if !lang.is_empty() && codes.contains(&lang.as_str()) {
        cfg.lang = lang;
    }

    let key = setup_password(&mut cfg)?;
    cfg.save()?;

    println!("\naternos.org'da F12 -> Application -> Cookies / Console.");
    let token = prompt("AJAX TOKEN (window.AJAX_TOKEN degerini yapistir)");
    let sec = prompt("SEC (bos birakabilirsin, cookie'den turetilecek)");
    let server_id = prompt("Server ID (ATERNOS_SERVER cookie degeri)");
    let cookies_raw = prompt("Cookie header (tum cookie'leri yapistir: ATERNOS_SESSION=...; ...)");

    let cookies = parse_cookies(&cookies_raw);

    // Validasyon: token veya cookie yoksa kurulum anlamsiz — bos session.enc
    // uretip daemon'i sahte oturumla calistirmayalim (arka planda/stdin kapaliyken
    // prompt'lar bos doner). Web wizard akisini da bozmamak icin burada dur.
    if token.is_empty() && cookies.is_empty() {
        return Err("token veya cookie girilmedi — kurulum iptal".into());
    }

    let sess = Session {
        token,
        sec,
        cookies,
        server_id,
        server_addr: None,
    };

    // server_id fallback: ATERNOS_SERVER cookie'sinden turet
    let strings = aterkeep_core::Strings::decrypt_all()?;
    let sid = sess
        .cookies
        .iter()
        .find(|c| c.name == strings.c_server)
        .map(|c| c.value.clone())
        .unwrap_or_else(|| sess.server_id.clone());
    let mut sess = Session {
        server_id: sid,
        ..sess
    };

    // sunucu adresini tespit et (best-effort)
    println!("[*] sunucu adresi tespit ediliyor...");
    match aterkeep_core::new_client(sess.clone()) {
        Ok(c) => match c.get_server_addr().await {
            Ok(addr) => {
                println!("[+] sunucu adresi: {addr}");
                sess.server_addr = Some(addr);
            }
            Err(e) => println!("[!] adres tespit edilemedi (panelde sonra doldurulur): {e}"),
        },
        Err(e) => println!("[!] adres tespit atlandi: {e}"),
    }

    sess.save_encrypted(&config::session_path(), &key)?;
    println!(
        "[+] {} yazildi (AES-256-GCM). aterkeep basliyor...",
        config::session_path().display()
    );
    println!("[!] Bu klasoru kimseyle paylasma — sifreli de olsa oturum verisi icerir.");
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
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

    let (sess, _key) = match load_session(&cfg, &password) {
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
        auto: true,
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

    // keepalive dongusu: status-only polling + bot entegrasyonu + extend/zombie.
    // probe_host/port: Aternos sunucu adresi (status online iken MC handshake ile
    // dogrulamak icin). server_addr tespit edilene kadar bos — keepalive online
    // oldugunda adresi yeniden tespit edip probe yapar (simdilik best-effort).
    let probe_host = {
        let s = state.lock().await;
        s.server_addr
            .as_deref()
            .and_then(|a| a.split(':').next())
            .unwrap_or("localhost")
            .to_string()
    };
    tokio::spawn(run_keepalive(
        client.clone(),
        state.clone(),
        tx.clone(),
        bot.clone(),
        30,
        probe_host.clone(),
        25565,
    ));

    // WS canli akis: Aternos Hermes WebSocket'ine baglan, anlik durum/kuyruk/console
    // guncellemelerini state + console buffer'a besle. keepalive'den bagimsiz calisir.
    tokio::spawn(aterkeep_core::run_hermes(
        client.clone(),
        state.clone(),
        console.clone(),
        tx.clone(),
    ));

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
