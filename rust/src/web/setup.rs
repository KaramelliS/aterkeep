//! Ilk kurulum, acilis durumu ve oturum sifirlama.
//!
//! Kurulumun iki yolu var: Aternos hesabiyla giris (varsayilan — cerezi biz
//! uretiriz ve 30 gunde bir kendimiz yenileriz) ve cerez yapistirma (2FA ya da
//! captcha cikan hesaplar icin yedek).

use super::{now_unix, parse_cookies, split_token};
use crate::config::{self, AppConfig};
use aterkeep_core::Session;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use axum::Json;
use serde_json::{json, Value};

use super::assets::{app_js, index, logo_svg, style_css};
use super::auth::token_from_headers;
use super::i18n::api_i18n_list;

/// Setup modu: session.enc yoksa paneli client olmadan acar. Sadece statik dosyalar,
/// setup endpoint'leri ve needs-setup serve edilir. /api/setup basarili olunca
/// session.enc yazilip self-restart yapilir; process normal modda yeniden baslar.
pub async fn run_setup_mode(cfg: AppConfig) {
    let app = Router::new()
        .route("/", get(index))
        .route("/static/style.css", get(style_css))
        .route("/static/app.js", get(app_js))
        .route("/static/logo.svg", get(logo_svg))
        .route("/api/needs-setup", get(api_needs_setup))
        // Panel acilirken /api/boot'u sorgular — setup modunda da cevap vermeli,
        // yoksa 404 alip setup overlay'ini hic gostermez.
        .route("/api/boot", get(api_boot))
        .route("/api/i18n/{lang}", get(api_i18n_public))
        .route("/api/i18n", get(api_i18n_list))
        .route("/api/setup", post(api_setup));
    let addr = format!("{}:{}", cfg.bind, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("{addr} dinlenemiyor: {e}"));
    axum::serve(listener, app).await.expect("server hatasi");
}

/// Kurulum modunda ctx yok — dil dosyalarini ctx'siz servis eden surum.
pub(super) async fn api_i18n_public(Path(lang): Path<String>) -> impl IntoResponse {
    Json(crate::translations::get(&lang))
}

/// setup tamamlandiktan sonra ayni binary'yi ayni argumanlarla yeniden baslatir
/// ve mevcut process'i kapatir. session.enc artik mevcut oldugu icin yeni process
/// normal (kurulu client ile) baslayacaktir.
/// `password`: yeniden baslayan surece parolayi ATERKEEP_KEY ile aktarir.
/// Anahtar diskte tutulmadigi icin cocuk surec oturumu baska turlu acamaz.
/// Parola sadece cocugun ortaminda kalir, diske hic yazilmaz.
///
/// UNIX'TE spawn+exit DEGIL, exec KULLANILIR. spawn+exit ucu birden bozuyordu:
///
///   1. Bir gozetmen (Android foreground service, systemd, launchd) bu process'i
///      izliyorsa, exit(0) ona "oldu" der ve YENI bir kopya baslatir. Ama
///      spawn edilen torun hala yasiyor ve 4041'i tutuyor; gozetmenin actigi
///      kopya bind edemeyip panic ediyor (bkz. main.rs: "{addr} dinlenemiyor").
///      Yani kurulum sonrasi kendini onaran degil, kendini kiran bir dongu.
///   2. Torun, gozetmenin surec agacindan KOPUYOR. Android'de bu olumcul:
///      izlenmeyen bir cocuk surec, phantom-process avcisinin tam hedefi.
///   3. Iki process kisa bir an ayni config/ uzerinde birlikte yasiyor.
///
/// exec() surec goruntusunu YERINDE degistirir: PID ayni kalir, gozetmenin
/// tuttugu handle gecerli kalir, parola yeni goruntunun ortaminda tasinir ve
/// diske hicbir sey yazilmaz. Yalnizca hata durumunda geri doner.
///
/// Windows'ta execve yok; orada eski spawn+exit davranisi korunuyor (Windows'ta
/// process'i gozeten bir servis katmani da kullanmiyoruz).
pub fn self_restart(password: Option<&str>) {
    let exe = std::env::current_exe().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("[setup] process yeniden baslatiliyor");
    if let Some(exe) = exe {
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(&args);
        if let Some(p) = password {
            cmd.env("ATERKEEP_KEY", p);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // Basarili olursa buradan DONMEZ: bu process artik yeni goruntu.
            let err = cmd.exec();
            // Buraya dusuyorsak exec basarisiz oldu (exe silinmis, izin yok...).
            // Sessizce exit(0) etmek gozetmene "duzgun kapandim" der ve gercek
            // sebep kaybolur; hatayi soyleyip basarisizlik koduyla cikiyoruz.
            eprintln!("[setup] exec basarisiz: {err} — gozetmen yeniden baslatmali");
            std::process::exit(1);
        }
        #[cfg(not(unix))]
        {
            let _ = cmd.spawn();
        }
    }
    std::process::exit(0);
}

/// Kurulum gerekli mi? Oturum dosyasi yoksa ya da panel parolasi hic
/// belirlenmemisse evet.
pub(super) async fn api_needs_setup() -> impl IntoResponse {
    let cfg = AppConfig::load();
    let needed = !config::session_path().exists() || !cfg.auth_enabled();
    Json(json!({ "needs_setup": needed }))
}

/// GET /api/boot -> { setup_mode: bool }
/// Panel acilir acilmaz bunu sorar: true ise kurulum overlay'ini gosterir ve
/// sekmeleri kilitler. `needs_setup` ile ayni kaynaktan beslenir.
/// Kimlik dogrulamasi ISTEMEZ — panel acilirken hangi ekrani gosterecegini
/// (kurulum / giris / panel) buradan ogrenir.
pub(super) async fn api_boot(headers: axum::http::HeaderMap) -> impl IntoResponse {
    let cfg = AppConfig::load();
    let setup_mode = !config::session_path().exists() || !cfg.auth_enabled();
    // Giris gerekiyor mu? Jetonu burada dogrulayamayiz (ctx yok) — panel
    // korumali bir uca istek atip 401 alirsa giris ekranini acar. Yine de
    // cerez hic yoksa dogrudan giris gerektigini bildirebiliriz.
    let has_cookie = token_from_headers(&headers).is_some();
    Json(json!({
        "setup_mode": setup_mode,
        "auth_enabled": cfg.auth_enabled(),
        "has_session_cookie": has_cookie,
        "lang": cfg.lang,
        // Panelde ve destek taleplerinde hangi surumun calistigini gormek sart.
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// POST /api/setup/reset -> oturum dosyasini siler ve process'i yeniden baslatir.
/// Aternos oturumu ~30 gunde bir doldugu icin bu, panelden tek tikla yeniden
/// kurulum akisina donmeyi saglar.
pub(super) async fn api_setup_reset() -> impl IntoResponse {
    let path = config::session_path();
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            return Json(json!({ "ok": false, "error": format!("session silinemedi: {e}") }));
        }
    }
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        // Parola aktarilmaz: oturum silindi, yeni kurulumda yenisi belirlenecek.
        self_restart(None);
    });
    Json(json!({ "ok": true }))
}

/// Cookie'leri al, key uret/yukle, session.enc yaz, adres tespit et, sonra self-restart.
pub(super) async fn api_setup(Json(body): Json<Value>) -> impl IntoResponse {
    let raw_token = body.get("token").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let server_id = body.get("server_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    // Cerez alani iki isimle de gelebilir: "cookies" (form) veya "cookie" (tek
    // textarea'ya yapistirilan ham Cookie header'i). Ikisini de kabul et.
    let cookies_raw = body
        .get("cookies")
        .or_else(|| body.get("cookie"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let cookies = parse_cookies(&cookies_raw);
    // Token "AJAX_TOKEN|SEC" birlesik formatinda gelebilir (panelin tarif ettigi
    // window.AJAX_TOKEN + "|" + window.generateAjaxToken() ciktisi). Bolup ayir;
    // ayri "sec" alani gonderildiyse o oncelikli.
    let (token, split_sec) = split_token(&raw_token);
    let sec = body
        .get("sec")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(split_sec);
    // Not: girdi dogrulamasi asagida, oturum yolu secildikten sonra yapiliyor —
    // hesapla giris yapan kullanicinin token/cookie vermesi gerekmiyor.

    // Panel parolasi: hem paneli korur hem oturumu sifreler. Anahtar diske
    // yazilmadigi icin bu parola olmadan kurulum tamamlanamaz.
    let password = body
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut cfg = AppConfig::load();
    // Kurulum zaten yapilmissa mevcut parolayi dogrula ve anahtari ondan turet;
    // yeniden uretmek eski verileri cozulemez hale getirirdi.
    let key = if cfg.auth_enabled() {
        match cfg.derive_session_key(&password) {
            Some(k) if cfg.verify_password(&password) => k,
            _ => {
                return Json(json!({ "ok": false, "error": "panel parolasi hatali" }));
            }
        }
    } else {
        if password.len() < 4 {
            return Json(
                json!({ "ok": false, "error": "panel parolasi en az 4 karakter olmali" }),
            );
        }
        let (auth, k) = config::new_auth(&password);
        cfg.auth = Some(auth);
        k
    };

    // Dil secimi (kurulum ekranindan) — config'e yazilir, panel bununla acilir.
    if let Some(lang) = body.get("lang").and_then(|v| v.as_str()) {
        if crate::translations::LANGS.iter().any(|(c, _)| *c == lang) {
            cfg.lang = lang.to_string();
        }
    }
    // Cerezlerin girildigi ani yaz: oturumun ne kadar dayandigini olcmenin
    // baslangic noktasi bu. Aternos cerez omrunu ilan etmiyor; tahmin etmek
    // yerine her kurulumda kendi verimizi topluyoruz.
    cfg.session_started = Some(now_unix());
    if let Err(e) = cfg.save() {
        return Json(json!({ "ok": false, "error": format!("config yazilamadi: {e}") }));
    }

    let strings = match aterkeep_core::Strings::decrypt_all() {
        Ok(s) => s,
        Err(e) => return Json(json!({ "ok": false, "error": format!("strings: {e}") })),
    };

    // --- OTURUM: iki yol ---
    // 1) Aternos hesabi (varsayilan): cerezi biz uretiriz. Kullanici DevTools
    //    acmaz ve cerez 30 gunde dolunca daemon kendi yeniler.
    // 2) Cerez yapistirma (yedek): 2FA'li hesaplar ve captcha cikan durumlar
    //    icin eski akis duruyor.
    let a_user = body
        .get("aternos_user")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let a_pass = body
        .get("aternos_pass")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut sess = if !a_user.is_empty() && !a_pass.is_empty() {
        let want_sid = if server_id.is_empty() {
            None
        } else {
            Some(server_id.clone())
        };
        let mut s = match aterkeep_core::login(&a_user, &a_pass, want_sid.clone()).await {
            Ok(s) => s,
            Err(e) => return Json(json!({ "ok": false, "error": e.to_string() })),
        };
        // Hesapta birden fazla sunucu varsa ve kullanici secmediyse SORALIM.
        // "ilkini al" varsayimi sessizce yanlis sunucuyu yonetmeye yol acardi.
        if want_sid.is_none() {
            if let Ok(c) = aterkeep_core::new_client(s.clone()) {
                if let Ok(list) = aterkeep_core::list_servers(&c.cookie_hdr()).await {
                    if list.len() > 1 {
                        return Json(json!({
                            "ok": false,
                            "need_server": list.iter()
                                .map(|(id, name)| json!({ "id": id, "name": name }))
                                .collect::<Vec<_>>()
                        }));
                    }
                }
            }
        }
        // Otomatik yenileme icin sakla (session.enc sifreli — bkz. Session).
        s.username = Some(a_user);
        s.password = Some(a_pass);
        s
    } else {
        if cookies.is_empty() {
            return Json(json!({
                "ok": false,
                "error": "Aternos hesap bilgileri ya da cerezler gerekli"
            }));
        }
        // server_id: ATERNOS_SERVER cookie'sinden tespit et, yoksa body'den al
        let sid = cookies
            .iter()
            .find(|c| c.name == strings.c_server)
            .map(|c| c.value.clone())
            .unwrap_or(server_id);
        Session {
            token,
            sec,
            cookies,
            server_id: sid,
            server_addr: None,
            username: None,
            password: None,
        }
    };
    // adres tespiti (best-effort): yeni client kur, /server/ sayfasindan adresi cek.
    if let Ok(c) = aterkeep_core::new_client(sess.clone()) {
        if let Ok(addr) = c.get_server_addr().await {
            sess.server_addr = Some(addr);
        }
    }
    if let Err(e) = sess.save_encrypted(&config::session_path(), &key) {
        return Json(json!({ "ok": false, "error": format!("kayit: {e}") }));
    }
    // 800ms sonra self-restart (response gitsin diye). Parolayi cocuk surece
    // ATERKEEP_KEY ile aktar — aksi halde oturumu acamaz ve kilitli baslar.
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        self_restart(Some(&password));
    });
    Json(json!({ "ok": true, "server_addr": sess.server_addr }))
}
