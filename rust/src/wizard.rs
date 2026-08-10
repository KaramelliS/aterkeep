//! Terminal kurulum sihirbazi (TTY varken). Web sihirbazi asil yoldur; bu,
//! tarayicisiz bir sunucuda kurulum yapabilmek icin duruyor.

use crate::cli::{parse_cookies, prompt};
use crate::config::{self, AppConfig};
use crate::translations;
use crate::unlock::setup_password;
use aterkeep_core::Session;

pub(crate) async fn run_wizard() -> Result<(), String> {
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
        // CLI sihirbazi cerez yapistirmaya dayanir; otomatik giris panel
        // kurulumundan gecer.
        username: None,
        password: None,
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
