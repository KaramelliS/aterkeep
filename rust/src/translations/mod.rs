// Panel cevirileri — 14 dil.
//
// TASARIM: EN *taban* tablodur ve TUM anahtarlari icerir. Diger diller yalnizca
// cevirdikleri anahtarlari listeler; `get()` once EN'i yazar, sonra dilin
// tablosuyla uzerine yazar. Boylece:
//   * eksik bir anahtar hicbir zaman ekranda ham anahtar adi ("setup_step3")
//     olarak gorunmez — en kotu durumda Ingilizce gorunur,
//   * yeni bir anahtar eklemek icin 14 tabloyu birden guncellemek gerekmez.
//
// Kurulum sihirbazi kasten Ingilizce'dir (urun karari): kullanici daha hicbir
// dil secmemisken gordugu ilk ekran budur. Dil, sihirbazin ilk adiminda
// secilir ve secim aninda tum arayuze (sihirbaz dahil) uygulanir.
mod en;
mod tr;
mod de;
mod fr;
mod es;
mod it;
mod pt;
mod ru;
mod ar;
mod zh;
mod ja;
mod ko;
mod nl;
mod pl;

use serde_json::{json, Map, Value};

use en::EN;
use tr::TR;
use de::DE;
use fr::FR;
use es::ES;
use it::IT;
use pt::PT;
use ru::RU;
use ar::AR;
use zh::ZH;
use ja::JA;
use ko::KO;
use nl::NL;
use pl::PL;

pub const LANGS: &[(&str, &str)] = &[
    ("en", "English"),
    ("tr", "Türkçe"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
    ("ru", "Русский"),
    ("ar", "العربية"),
    ("zh", "中文"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
];

/// Tarih/saat bicimlemesi icin BCP-47 etiketi. Panel saatleri bununla yazilir;
/// aksi halde her dilde Turkce saat bicimi gorunurdu.
fn locale_for(lang: &str) -> &'static str {
    match lang {
        "tr" => "tr-TR",
        "de" => "de-DE",
        "fr" => "fr-FR",
        "es" => "es-ES",
        "it" => "it-IT",
        "pt" => "pt-BR",
        "ru" => "ru-RU",
        "ar" => "ar-SA",
        "zh" => "zh-CN",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        "nl" => "nl-NL",
        "pl" => "pl-PL",
        _ => "en-GB",
    }
}

pub fn list() -> Value {
    let arr: Vec<Value> = LANGS
        .iter()
        .map(|(c, n)| json!({"code": c, "name": n}))
        .collect();
    Value::Array(arr)
}

pub(crate) type T = &'static [(&'static str, &'static str)];

/// Dil kodu -> ceviri tablosu. Bilinmeyen kod None doner (yalnizca EN kullanilir).
fn overrides(lang: &str) -> Option<T> {
    Some(match lang {
        "tr" => TR,
        "de" => DE,
        "fr" => FR,
        "es" => ES,
        "it" => IT,
        "pt" => PT,
        "ru" => RU,
        "ar" => AR,
        "zh" => ZH,
        "ja" => JA,
        "ko" => KO,
        "nl" => NL,
        "pl" => PL,
        _ => return None,
    })
}

pub fn get(lang: &str) -> Value {
    let mut m = Map::new();
    // 1) Taban: Ingilizce (tam kume).
    for (k, v) in EN {
        m.insert(k.to_string(), Value::String(v.to_string()));
    }
    // 2) Dilin cevirdigi anahtarlar uzerine yazar.
    if let Some(table) = overrides(lang) {
        for (k, v) in table {
            m.insert(k.to_string(), Value::String(v.to_string()));
        }
    }
    let code = if overrides(lang).is_some() || lang == "en" {
        lang
    } else {
        "en"
    };
    m.insert("lang".into(), Value::String(code.to_string()));
    m.insert("locale".into(), Value::String(locale_for(code).into()));
    // Arapca sagdan sola yazilir; panel <html dir> degerini buradan okur.
    m.insert(
        "dir".into(),
        Value::String(if code == "ar" { "rtl" } else { "ltr" }.into()),
    );
    m.insert("langs".into(), list());
    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn list_has_all_langs() {
        let v = list();
        let arr = v.as_array().expect("list() must be an array");
        assert_eq!(arr.len(), LANGS.len());
        for item in arr {
            assert!(item.get("code").and_then(|c| c.as_str()).is_some());
            assert!(item.get("name").and_then(|n| n.as_str()).is_some());
        }
    }

    #[test]
    fn get_known_lang_returns_object_with_langs() {
        let v = get("en");
        let obj = v.as_object().expect("get() must be an object");
        assert!(obj.contains_key("app_name"));
        assert!(obj.get("langs").and_then(|l| l.as_array()).is_some());
    }

    /// Bilinmeyen dil kodu Ingilizce'ye duser (Turkce'ye DEGIL): kurulum ekrani
    /// ve tanimsiz diller icin urun karari Ingilizce.
    #[test]
    fn unknown_lang_falls_back_to_english() {
        let unknown = get("zzz");
        let en = get("en");
        assert_eq!(unknown.get("app_sub"), en.get("app_sub"));
        assert_eq!(unknown.get("lang").and_then(|v| v.as_str()), Some("en"));
    }

    /// Her dil TAM kumeyi dondurur: eksik ceviri Ingilizce'ye duser, hicbir
    /// zaman ham anahtar adi ekranda gorunmez.
    #[test]
    fn every_lang_has_every_english_key() {
        for (code, _) in LANGS {
            let v = get(code);
            let obj = v.as_object().unwrap();
            for (k, _) in EN {
                assert!(
                    obj.contains_key(*k),
                    "dil {code} icin {k} anahtari eksik"
                );
            }
        }
    }

    /// Ceviri tablosunda EN'de bulunmayan bir anahtar varsa bu bir yazim
    /// hatasidir: hicbir yerde kullanilmayacagi icin sessizce olu kalir.
    #[test]
    fn no_override_key_is_missing_from_english() {
        let base: HashSet<&str> = EN.iter().map(|(k, _)| *k).collect();
        for (code, _) in LANGS {
            if let Some(table) = overrides(code) {
                for (k, _) in table {
                    assert!(
                        base.contains(*k),
                        "{code} tablosundaki {k} anahtari EN tabaninda yok (yazim hatasi?)"
                    );
                }
            }
        }
    }

    /// Ayni tabloda bir anahtar iki kez tanimlanmis olmamali — ikincisi
    /// birincisini sessizce eziyor olurdu.
    #[test]
    fn no_duplicate_keys_within_a_table() {
        let mut tables: Vec<(&str, T)> = vec![("en", EN)];
        for (code, _) in LANGS {
            if let Some(t) = overrides(code) {
                tables.push((code, t));
            }
        }
        for (code, table) in tables {
            let mut seen = HashSet::new();
            for (k, _) in table {
                assert!(seen.insert(*k), "{code} tablosunda {k} iki kez var");
            }
        }
    }

    #[test]
    fn arabic_is_rtl_others_are_ltr() {
        assert_eq!(get("ar").get("dir").and_then(|v| v.as_str()), Some("rtl"));
        assert_eq!(get("tr").get("dir").and_then(|v| v.as_str()), Some("ltr"));
    }
}
