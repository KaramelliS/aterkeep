//! Parola alma ve sifreli oturumu acma.
//!
//! Anahtar diske YAZILMAZ: her calistirmada paroladan turetilir. Bu yuzden
//! parolayi bulmak baslangicin ilk adimi ve arka planda calisirken
//! `ATERKEEP_KEY` zorunlu.

use crate::config::{self, AppConfig};
use crate::cli::prompt;
use aterkeep_core::Session;

/// Oturumu acacak parolayi bulur.
///
/// Sirasiyla: `ATERKEEP_KEY` ortam degiskeni -> interaktif terminalde sorma.
/// Arka planda (servis/systemd) calisirken terminal olmadigi icin ortam
/// degiskeni ZORUNLUDUR — anahtar diskte tutulmadigindan baska kaynak yok.
pub(crate) fn acquire_password(cfg: &AppConfig) -> Result<String, String> {
    if let Ok(p) = std::env::var("ATERKEEP_KEY") {
        if !p.is_empty() {
            // Ortamdan gelen parolayi da DOGRULA. Onceden dogrudan kabul
            // ediliyordu: yanlis bir ATERKEEP_KEY ile `import` calistirmak
            // session.enc'i bir daha cozulemeyecek bir anahtarla uzerine
            // yaziyordu. Sessiz ve geri donusu olmayan bir veri kaybi.
            if cfg.auth_enabled() && !cfg.verify_password(&p) {
                return Err(
                    "ATERKEEP_KEY yanlis — panel parolasiyla eslesmiyor (hicbir sey yazilmadi)"
                        .into(),
                );
            }
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

pub(crate) fn load_session(cfg: &AppConfig, password: &str) -> Result<(Session, [u8; 32]), String> {
    let key = cfg
        .derive_session_key(password)
        .ok_or("kurulum tamamlanmamis (config/aterkeep.json icinde auth yok)")?;
    let sess = Session::load_encrypted(&config::session_path(), &key)?;
    Ok((sess, key))
}

/// Kurulum icin parola alir ve config'e auth kaydi yazar; oturum anahtarini
/// dondurur. Mevcut bir kurulum varsa parolayi dogrular (yeniden uretmez —
/// yoksa eski session.enc cozulemez hale gelirdi).
pub(crate) fn setup_password(cfg: &mut AppConfig) -> Result<[u8; 32], String> {
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
