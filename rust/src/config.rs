//! Daemon yapilandirmasi, dosya duzeni ve parola tabanli anahtar turetme.
//!
//! # Guvenlik modeli
//!
//! Onceki surumde oturum anahtari `aterkeep.key` dosyasinda, `session.enc`'in
//! tam yaninda duruyordu. Klasoru kopyalayan biri hem sifreli veriyi hem de onu
//! acan anahtari ele geciriyordu — AES-256 fiilen hicbir sey korumuyordu.
//!
//! Artik anahtar DISKE YAZILMAZ. Kullanicinin belirledigi paroladan her
//! calistirmada PBKDF2-HMAC-SHA256 ile yeniden turetilir:
//!
//!   oturum anahtari = PBKDF2(parola, kdf_salt,  600_000)   -> hicbir yerde saklanmaz
//!   giris dogrulama = PBKDF2(parola, auth_salt, 600_000)   -> aterkeep.json'da saklanir
//!
//! Iki AYRI rastgele salt kullanilmasi kritik: ayni salt kullanilsaydi
//! `aterkeep.json`'da sakladigimiz dogrulama ozeti, oturumu acan anahtarin
//! ta kendisi olurdu. Farkli saltlarla, config dosyasi calinsa bile ondan
//! oturum anahtari uretilemez.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;

/// PBKDF2 tur sayisi. crypto.rs'teki oturum sifrelemesiyle ayni buyuklukte
/// tutuluyor — dusurmek kaba kuvvet saldirisini ucuzlatir.
const ITERS: u32 = 600_000;
const SALT_LEN: usize = 16;

/// Tum calisma dosyalarinin (config, oturum, bot durumu) tutuldugu dizin.
/// `ATERKEEP_DIR` ile degistirilebilir; varsayilan olarak calisma dizinindeki
/// `config/`. Boylece exe'nin yanina dagilmis dosyalar olusmaz.
pub fn data_dir() -> PathBuf {
    match std::env::var("ATERKEEP_DIR") {
        Ok(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => PathBuf::from("config"),
    }
}

pub fn config_path() -> PathBuf {
    data_dir().join("aterkeep.json")
}

pub fn session_path() -> PathBuf {
    data_dir().join("session.enc")
}

/// Panel giris dogrulamasi icin saklanan veri. Parolanin kendisi DEGIL,
/// yavas bir KDF'ten gecmis ozeti tutulur.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Auth {
    /// Oturum anahtarini turetmekte kullanilan salt (base64).
    pub kdf_salt: String,
    /// Giris dogrulama ozetinin salti (base64) — kdf_salt'tan FARKLI olmali.
    pub auth_salt: String,
    /// PBKDF2(parola, auth_salt) ozeti (base64).
    pub auth_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// Panel arayuz dili (kurulumda secilir). translations.rs'teki kodlardan biri.
    pub lang: String,
    /// Panelin dinledigi port.
    pub port: u16,
    /// Dinlenen adres. Varsayilan 127.0.0.1 — disariya acmak icin degistirilir.
    pub bind: String,
    /// Kurulumda olusturulur. None ise panel korumasizdir (eski kurulumlar).
    pub auth: Option<Auth>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // Kurulum ekrani kullanici henuz dil secmemisken gorunur; urun
            // karari olarak ilk temas Ingilizce. Sihirbazin 1. adiminda secilen
            // dil aninda tum arayuze (sihirbaz dahil) uygulanir.
            lang: "en".into(),
            port: 4041,
            bind: "127.0.0.1".into(),
            auth: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pretty = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, pretty).map_err(|e| e.to_string())
    }

    /// Panel girisi zorunlu mu? (kurulum tamamlanmissa evet)
    pub fn auth_enabled(&self) -> bool {
        self.auth.is_some()
    }

    /// Girilen parola dogru mu? Sabit zamanli karsilastirma kullanir.
    pub fn verify_password(&self, password: &str) -> bool {
        let Some(auth) = &self.auth else {
            return false;
        };
        let Ok(salt) = B64.decode(&auth.auth_salt) else {
            return false;
        };
        let Ok(expected) = B64.decode(&auth.auth_hash) else {
            return false;
        };
        let mut got = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, ITERS, &mut got);
        constant_time_eq(&got, &expected)
    }

    /// Bu kurulumun salt'i ile oturum anahtarini paroladan turetir.
    /// Auth yoksa (kurulum yapilmamis) None doner.
    pub fn derive_session_key(&self, password: &str) -> Option<[u8; 32]> {
        let auth = self.auth.as_ref()?;
        let salt = B64.decode(&auth.kdf_salt).ok()?;
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, ITERS, &mut key);
        Some(key)
    }
}

/// Yeni bir kurulum icin rastgele saltlar uretip parolanin dogrulama ozetini
/// hesaplar; ayni anda oturum anahtarini da dondurur (diske yazilmaz).
pub fn new_auth(password: &str) -> (Auth, [u8; 32]) {
    let mut rng = rand::thread_rng();
    let mut kdf_salt = [0u8; SALT_LEN];
    let mut auth_salt = [0u8; SALT_LEN];
    rng.fill_bytes(&mut kdf_salt);
    rng.fill_bytes(&mut auth_salt);

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &kdf_salt, ITERS, &mut key);

    let mut hash = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &auth_salt, ITERS, &mut hash);

    (
        Auth {
            kdf_salt: B64.encode(kdf_salt),
            auth_salt: B64.encode(auth_salt),
            auth_hash: B64.encode(hash),
        },
        key,
    )
}

/// Rastgele panel oturum jetonu (giris sonrasi cerezde tasinir).
pub fn new_session_token() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    B64.encode(buf)
}

/// Uzunluk ve icerik farkini erken donmeden karsilastirir — zamanlama
/// sizintisiyla parola ozeti tahmin edilmesini zorlastirir.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_roundtrip_verifies() {
        let (auth, _key) = new_auth("gizli-parola");
        let cfg = AppConfig {
            auth: Some(auth),
            ..Default::default()
        };
        assert!(cfg.verify_password("gizli-parola"));
        assert!(!cfg.verify_password("yanlis"));
        assert!(!cfg.verify_password(""));
    }

    #[test]
    fn stored_hash_is_not_the_session_key() {
        // En kritik ozellik: config'te sakladigimiz ozet, oturumu acan
        // anahtarin AYNISI olmamali. Ayni olsaydi config dosyasini ele
        // geciren kisi oturumu da cozerdi.
        let (auth, key) = new_auth("ayni-parola");
        let stored = B64.decode(&auth.auth_hash).unwrap();
        assert_ne!(stored, key.to_vec());
        assert_ne!(auth.kdf_salt, auth.auth_salt);
    }

    #[test]
    fn derived_key_is_deterministic_per_install() {
        let (auth, key) = new_auth("parola123");
        let cfg = AppConfig {
            auth: Some(auth),
            ..Default::default()
        };
        // Ayni parola + ayni salt -> ayni anahtar (oturum tekrar acilabilmeli)
        assert_eq!(cfg.derive_session_key("parola123"), Some(key));
        // Farkli parola -> farkli anahtar
        assert_ne!(cfg.derive_session_key("baska"), Some(key));
    }

    #[test]
    fn two_installs_derive_different_keys() {
        // Salt rastgele oldugu icin ayni parola bile kurulumdan kuruluma
        // farkli anahtar uretmeli (rainbow table savunmasi).
        let (a1, k1) = new_auth("ortak-parola");
        let (a2, k2) = new_auth("ortak-parola");
        assert_ne!(k1, k2);
        assert_ne!(a1.kdf_salt, a2.kdf_salt);
    }

    #[test]
    fn no_auth_means_no_access() {
        let cfg = AppConfig::default();
        assert!(!cfg.auth_enabled());
        assert!(!cfg.verify_password("herhangi"));
        assert_eq!(cfg.derive_session_key("herhangi"), None);
    }

    #[test]
    fn session_tokens_are_unique() {
        assert_ne!(new_session_token(), new_session_token());
    }
}
