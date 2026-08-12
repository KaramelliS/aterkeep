//! Komut satiri: surum/yardim bayraklari, `import` komutu ve kucuk yardimcilar.

use crate::config::{self, AppConfig};
use crate::unlock::setup_password;
use aterkeep_core::{Cookie, Session};

pub(crate) fn cmd_import(json_path: &str) -> Result<(), String> {
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
        // Ice aktarma disaridan gelen bir oturum dosyasidir; hesap bilgisi
        // tasimaz. Otomatik yenileme icin panelden kurulum gerekir.
        username: None,
        password: None,
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

pub(crate) fn prompt(label: &str) -> String {
    use std::io::{self, Write};
    print!("{label}: ");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    s.trim().to_string()
}
