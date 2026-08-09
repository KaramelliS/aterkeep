use crate::bot::{BotConfigPatch, SharedBot};
use crate::config::{self, AppConfig};
use aterkeep_core::{log, split_addr, AternosClient, Cookie, LogTx, Session, SharedConsole, SharedState};
use axum::extract::Path;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

pub struct AppCtx {
    pub client: Arc<AternosClient>,
    pub state: SharedState,
    pub tx: LogTx,
    pub bot: SharedBot,
    /// WS (Hermes) canli akis console buffer'i. ws.rs tarafindan doldurulur,
    /// /api/console tarafindan okunur. Yoksa (örn. setup modu) None olabilir.
    pub console: SharedConsole,
    /// Daemon yapilandirmasi (dil, port, panel parolasi ozeti).
    pub cfg: tokio::sync::Mutex<AppConfig>,
    /// Gecerli panel oturum jetonu. Giris yapilinca uretilir, cikista silinir.
    /// Tek jeton tutulur: yeni giris eskisini gecersiz kilar.
    pub auth_token: tokio::sync::Mutex<Option<String>>,
}

/// Panel cerezinden oturum jetonunu okur.
fn token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    parse_cookies(raw)
        .into_iter()
        .find(|c| c.name == AUTH_COOKIE)
        .map(|c| c.value)
}

const AUTH_COOKIE: &str = "aterkeep_auth";

/// Istek yetkili mi? Auth kapaliysa (kurulum yapilmamis) her istek gecer.
async fn is_authed(ctx: &AppCtx, headers: &axum::http::HeaderMap) -> bool {
    if !ctx.cfg.lock().await.auth_enabled() {
        return true;
    }
    let Some(tok) = token_from_headers(headers) else {
        return false;
    };
    ctx.auth_token.lock().await.as_deref() == Some(tok.as_str())
}

/// Tum /api/* uclarini koruyan katman. Giris, durum sorgusu ve statik
/// dosyalar disinda her sey jeton ister; yoksa 401 doner ve panel giris
/// ekranini gosterir.
async fn auth_layer(
    ctx: Arc<AppCtx>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = req.uri().path().to_string();
    // Giris akisinin kendisi ve statik varliklar korumasiz olmak zorunda.
    let open = path == "/"
        || path.starts_with("/static/")
        || path == "/api/login"
        || path == "/api/boot"
        || path == "/api/needs-setup"
        || path == "/api/setup"
        || path == "/api/i18n"
        || path.starts_with("/api/i18n/");
    if open || is_authed(&ctx, req.headers()).await {
        return next.run(req).await;
    }
    (
        axum::http::StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "giris gerekli", "auth_required": true })),
    )
        .into_response()
}

pub fn router(ctx: Arc<AppCtx>) -> Router {
    let c1 = ctx.clone();
    let c2 = ctx.clone();
    let c3 = ctx.clone();
    let c4 = ctx.clone();
    let c5 = ctx.clone();
    let c6 = ctx.clone();
    let c7 = ctx.clone();
    let c8 = ctx.clone();
    let c9 = ctx.clone();
    let c10 = ctx.clone();
    let c11 = ctx.clone();
    let c12 = ctx.clone();
    let c13 = ctx.clone();
    let c14 = ctx.clone();
    let c15 = ctx.clone();
    let c16 = ctx.clone();
    let c17 = ctx.clone();
    let c18 = ctx.clone();
    let c19 = ctx.clone();
    let c20 = ctx.clone();
    let c21 = ctx.clone();
    Router::new()
        .route("/", get(index))
        .route("/static/style.css", get(style_css))
        .route("/static/app.js", get(app_js))
        .route("/static/logo.svg", get(logo_svg))
        .route("/api/status", get(move |s| api_status(c1, s)))
        .route("/api/action/{action}", get(move |p| api_action(c2, p)))
        .route("/api/stream", get(move |s| api_stream(c3, s)))
        .route("/api/options", get(move |s| api_options(c4, s)))
        .route("/api/options/set", get(move |q| api_option_set(c5, q)))
        .route("/api/players", get(move |s| api_players(c6, s)))
        .route("/api/console", get(move |s| api_console(c7, s)))
        .route("/api/toggle", get(move |q| api_toggle(c8, q)))
        .route("/api/i18n/{lang}", get(move |p| api_i18n(c9, p)))
        .route("/api/i18n", get(api_i18n_list))
        .route("/api/needs-setup", get(api_needs_setup))
        .route("/api/boot", get(api_boot))
        .route("/api/setup", post(api_setup))
        .route("/api/setup/reset", post(api_setup_reset))
        .route("/api/bot", get(move |s| api_bot_status(c10, s)))
        // Panel bot durumunu /api/bot/status'tan, config+durum birlesimini
        // GET /api/bot/config'ten okur; toggle'i JSON POST ile yollar. GET ?on=
        // formu da korunuyor (eski panel/otomasyon uyumlulugu).
        .route("/api/bot/status", get(move |s| api_bot_status_wrapped(c15, s)))
        .route(
            "/api/bot/toggle",
            get(move |q| api_bot_toggle(c11, q)).post(move |b| api_bot_toggle_post(c16, b)),
        )
        .route(
            "/api/bot/config",
            get(move |s| api_bot_config_get(c17, s)).post(move |b| api_bot_config(c12, b)),
        )
        .route("/api/cancel", get(move |s| api_cancel(c13, s)))
        .route("/api/extend", get(move |s| api_extend(c14, s)))
        .route("/api/confirm", get(move |s| api_confirm(c21, s)))
        .route("/api/login", post(move |b| api_login(c18, b)))
        .route("/api/logout", post(move |_: axum::extract::Request| api_logout(c19)))
        .layer(axum::middleware::from_fn(move |req, next| {
            auth_layer(c20.clone(), req, next)
        }))
}

/// POST /api/login  body: { "password": "..." }
/// Dogruysa rastgele jeton uretir, HttpOnly cerez olarak yazar.
async fn api_login(ctx: Arc<AppCtx>, Json(body): Json<Value>) -> impl IntoResponse {
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let ok = ctx.cfg.lock().await.verify_password(password);
    if !ok {
        // Kaba kuvvet denemelerini yavaslatmak icin kucuk bir gecikme.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "parola hatali" })),
        )
            .into_response();
    }
    let token = config::new_session_token();
    *ctx.auth_token.lock().await = Some(token.clone());
    // HttpOnly: JavaScript okuyamaz (XSS ile jeton calinmasini zorlastirir).
    // SameSite=Strict: baska sitelerden gelen isteklerde cerez gonderilmez.
    let cookie = format!("{AUTH_COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict");
    (
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(json!({ "ok": true })),
    )
        .into_response()
}

/// POST /api/logout -> jetonu dusur, cerezi sil.
async fn api_logout(ctx: Arc<AppCtx>) -> impl IntoResponse {
    *ctx.auth_token.lock().await = None;
    let cookie = format!("{AUTH_COOKIE}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0");
    (
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(json!({ "ok": true })),
    )
        .into_response()
}

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
async fn api_i18n_public(Path(lang): Path<String>) -> impl IntoResponse {
    Json(crate::translations::get(&lang))
}

/// Statik varliklar binary'ye gomulu oldugu icin surum atlandiginda dosya adi
/// degismez; cache basligi olmadan tarayici eski panelde takili kalir (yeni
/// surumde kurulum ekranini hic gormemek gibi teshis edilmesi zor sonuclar
/// dogurur). Her yanit yeniden dogrulansin.
const NO_CACHE: (&str, &str) = ("Cache-Control", "no-cache, must-revalidate");

async fn index() -> impl IntoResponse {
    (
        [NO_CACHE],
        axum::response::Html(include_str!("../static/index.html")),
    )
}
async fn style_css() -> impl IntoResponse {
    (
        [("Content-Type", "text/css"), NO_CACHE],
        include_str!("../static/style.css"),
    )
}
async fn app_js() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript"), NO_CACHE],
        include_str!("../static/app.js"),
    )
}
async fn logo_svg() -> impl IntoResponse {
    (
        [("Content-Type", "image/svg+xml"), NO_CACHE],
        include_str!("../static/logo.svg"),
    )
}

async fn api_status(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    // anlik durum: son cekim 2sn'den eskiyse gercek bir start() cagir (throttle)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    {
        let mut s = ctx.state.lock().await;
        let stale = now.saturating_sub(s.last_poll) >= 2;
        // Oto-baslat KAPALIYSA start() cagirma — kullanici sunucuyu durdurmus olabilir
        // (orn. surum degistirmek icin). Sadece bilinen durumu don, Aternos'u rahatsiz etme.
        // AyrIca keepalive dongusu zaten kapali; status poll'un da start atmasi anlamsiz.
        if stale && !s.busy && s.auto {
            s.last_poll = now;
            s.busy = true;
            drop(s);
            let mut resp = ctx.client.start(false).await;
            if let Ok(v) = &resp {
                if v.pointer("/data/status").and_then(|x| x.as_str()) == Some("eula") {
                    resp = ctx.client.start(true).await;
                }
            }
            let mut s = ctx.state.lock().await;
            if let Ok(v) = resp {
                let status = v
                    .pointer("/data/status")
                    .and_then(|x| x.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                s.server_state = status;
                s.last_check = now;
                // frontend request inspector'in canli calismasi icin last_request'i guncelle.
                s.last_request = Some(json!({"action": "status", "response": v}));
            }
            s.busy = false;
        }
    }
    let s = ctx.state.lock().await;
    // Adresi cache'den al; bossa (session'da yoktu) arka planda bir kez tespit et
    // (sonraki poll'de gelir).
    let addr = s.server_addr.clone();
    if s.server_addr.is_none() && !s.busy {
        let client = ctx.client.clone();
        let state = ctx.state.clone();
        tokio::spawn(async move {
            match client.get_server_addr().await {
                Ok(a) => {
                    let mut s = state.lock().await;
                    s.server_addr = Some(a);
                }
                Err(_) => { /* sessiz — sonraki poll tekrar dener */ }
            }
        });
    }
    Json(json!({
        "state": s.server_state,
        "auto": s.auto,
        "last_check": s.last_check,
        "running": s.busy,
        "server_id": ctx.client.session.server_id,
        "server_addr": addr,
        "last_request": s.last_request,
        // WS (Hermes) canli alanlar — ws.rs besler, kapaliysa varsayilan degerler.
        "ws_connected": s.ws_connected,
        "players": s.players,
        "slots": s.slots,
        "label": s.label,
        "tps": s.tps,
        // Sunucunun kullandigi RAM (MB). Hermes akisi besler; baglanti yoksa 0.
        "heap": s.heap,
        "queue": s.queue,
    }))
}

async fn api_action(
    ctx: Arc<AppCtx>,
    Path(action): Path<String>,
) -> impl IntoResponse {
    let valid = ["start", "stop", "restart"].contains(&action.as_str());
    if !valid {
        return Json(json!({"ok": false, "error": "gecersiz"}));
    }
    let busy = {
        let s = ctx.state.lock().await;
        s.busy
    };
    if busy {
        return Json(json!({"ok": false, "error": "mesgul"}));
    }
    {
        let mut s = ctx.state.lock().await;
        s.busy = true;
    }
    let client = ctx.client.clone();
    let state = ctx.state.clone();
    let tx = ctx.tx.clone();
    tokio::spawn(async move {
        // Inner logic: herhangi bir early-return veya beklenmedik durumda bile
        // aşağıdaki busy=false set'inin kesinlikle çalışması için bloğa alıyoruz.
        let inner = async {
            log(&tx, "cmd", format!("$ aternos {action}"));
            if action == "stop" {
                let mut s = state.lock().await;
                s.auto = false;
                log(&tx, "sys", "oto-baslat KAPALI (durdurma — sunucu kapali kalir)".into());
            }
            let resp = match action.as_str() {
                "start" => client.start(false).await,
                "stop" => client.stop().await,
                _ => client.restart().await,
            };
            let mut req = json!({"action": action, "response": resp.clone().unwrap_or(json!({"error": "istek hatasi"}))});
            {
                let mut s = state.lock().await;
                s.last_request = Some(req.take());
            }
            match resp {
                Ok(v) => {
                    let status = v.pointer("/data/status").and_then(|x| x.as_str()).unwrap_or("");
                    if action == "start" && status == "eula" {
                        log(&tx, "warn", "EULA gerekli, tekrar deneniyor".into());
                        let r2 = client.start(true).await;
                        let mut s = state.lock().await;
                        s.last_request = Some(json!({"action": "start+eula", "response": r2.clone().unwrap_or(json!({}))}));
                        drop(s);
                        log(&tx, "ok", format!("start+eula -> {}", serde_json::to_string(&r2.unwrap_or(json!({}))).unwrap_or_default()));
                    } else if action == "start" && status == "already" {
                        log(&tx, "ok", "sunucu zaten calisiyor (already)".into());
                        let mut s = state.lock().await;
                        s.server_state = "already".into();
                    } else if action == "stop" {
                        log(&tx, "ok", "durma komutu gonderildi".into());
                        let mut s = state.lock().await;
                        s.server_state = "offline".into();
                    } else if action == "restart" {
                        log(&tx, "ok", "yeniden baslatma gonderildi".into());
                    } else if v.get("success").and_then(|x| x.as_bool()) == Some(true) {
                        log(&tx, "ok", "sunucu baslatiliyor".into());
                        let mut s = state.lock().await;
                        s.server_state = if status.is_empty() { "starting".into() } else { status.to_string() };
                    } else if action == "start" {
                        log(&tx, "warn", format!("start kabul edilmedi (state: {status})"));
                        let mut s = state.lock().await;
                        s.server_state = status.to_string();
                    }
                }
                Err(e) => log(&tx, "err", format!("istek hatasi: {e}")),
            }
        };
        inner.await;
        // HER DURUMDA busy'yi serbest birak (early-exit/panic-sonrasi state korunmasi).
        let mut s = state.lock().await;
        s.busy = false;
    });
    Json(json!({"ok": true}))
}

/// Kuyruktaki/baslamakta olan sunucuyu iptal et (cancel). Aternos /ajax/cancel.
async fn api_cancel(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    let tx = ctx.tx.clone();
    log(&tx, "cmd", "$ aternos cancel".into());
    match ctx.client.cancel().await {
        Ok(v) => {
            log(&tx, "ok", "kuyruk iptali gonderildi".into());
            let mut s = ctx.state.lock().await;
            s.last_request = Some(json!({"action": "cancel", "response": v.clone()}));
            Json(json!({"ok": true, "response": v}))
        }
        Err(e) => {
            log(&tx, "err", format!("cancel hatasi: {e}"));
            Json(json!({"ok": false, "error": e}))
        }
    }
}

/// Kuyruk sirasi geldiginde Aternos'un bekledigi onayi elle gonder.
/// keepalive dongusu bunu "pending" gordugunde otomatik yapar; bu uc, kullanici
/// panelden hemen tetiklemek isterse diye var.
async fn api_confirm(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    let tx = ctx.tx.clone();
    log(&tx, "cmd", "$ aternos confirm".into());
    match ctx.client.confirm().await {
        Ok(v) => {
            log(&tx, "ok", "kuyruk onayi gonderildi".into());
            let mut s = ctx.state.lock().await;
            s.last_request = Some(json!({"action": "confirm", "response": v.clone()}));
            Json(json!({"ok": true, "response": v}))
        }
        Err(e) => {
            log(&tx, "err", format!("onay hatasi: {e}"));
            Json(json!({"ok": false, "error": e}))
        }
    }
}

/// Online sunucunun idle kapanma suresini uzat (extend). Aternos /ajax/extend.
async fn api_extend(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    let tx = ctx.tx.clone();
    log(&tx, "cmd", "$ aternos extend".into());
    match ctx.client.extend_end().await {
        Ok(v) => {
            log(&tx, "ok", "idle suresi uzatildi".into());
            let mut s = ctx.state.lock().await;
            s.last_extend = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            s.last_request = Some(json!({"action": "extend", "response": v.clone()}));
            Json(json!({"ok": true, "response": v}))
        }
        Err(e) => {
            log(&tx, "err", format!("extend hatasi: {e}"));
            Json(json!({"ok": false, "error": e}))
        }
    }
}

async fn api_stream(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    let rx = ctx.tx.subscribe();
    let stream = async_stream::stream! {
        let mut rx = rx;
        loop {
            match rx.recv().await {
                Ok(v) => yield Ok::<Event, Infallible>(Event::default().data(v.to_string())),
                Err(RecvError::Lagged(_)) => {
                    // geciken mesajlar: keepalive comment gonder, devam et.
                    yield Ok::<Event, Infallible>(Event::default().data(":keepalive"));
                }
                Err(RecvError::Closed) => {
                    // channel tamamen kapandi (sender yok) — task temiz kapansin.
                    break;
                }
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::new())
}

async fn api_options(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    match ctx.client.get_options().await {
        Ok(v) => Json(v),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn api_option_set(
    ctx: Arc<AppCtx>,
    q: axum::extract::Query<Value>,
) -> impl IntoResponse {
    let name = q.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let value = q.get("value").and_then(|v| v.as_str()).unwrap_or("");
    if name.is_empty() {
        return Json(json!({"ok": false, "error": "name gerekli"}));
    }
    match ctx.client.set_option(name, value).await {
        Ok(v) => Json(v),
        Err(e) => Json(json!({"ok": false, "error": e})),
    }
}

async fn api_players(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    match ctx.client.get_players().await {
        Ok(v) => Json(v),
        Err(e) => Json(json!({"error": e})),
    }
}

async fn api_console(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    // Oncelik: WS (Hermes) canli console buffer'i. ws.rs besliyorsa onu don;
    // bu en anlik kaynak (HTTP'den daha taze). Bossa HTTP fallback (get_console_log).
    {
        let buf = ctx.console.lock().await;
        if !buf.lines.is_empty() {
            return Json(json!({"lines": buf.lines.clone(), "source": "ws"}));
        }
    }
    match ctx.client.get_console_log(60).await {
        Ok(v) => {
            // ws_connect degilse veya buffer bossa kaynak "http"
            let ws_connected = ctx.state.lock().await.ws_connected;
            Json(json!({"lines": v, "source": if ws_connected { "ws" } else { "http"}}))
        }
        Err(e) => Json(json!({"error": e})),
    }
}

async fn api_toggle(ctx: Arc<AppCtx>, q: axum::extract::Query<Value>) -> impl IntoResponse {
    let on = q.get("on").and_then(|v| v.as_str()) == Some("true");
    {
        let mut s = ctx.state.lock().await;
        s.auto = on;
    }
    log(
        &ctx.tx,
        "sys",
        format!("oto-baslat {}", if on { "ACIK" } else { "KAPALI" }),
    );
    Json(json!({"ok": true, "auto": on}))
}

async fn api_i18n(
    _ctx: Arc<AppCtx>,
    Path(lang): Path<String>,
) -> impl IntoResponse {
    Json(crate::translations::get(&lang))
}

async fn api_i18n_list() -> impl IntoResponse {
    Json(crate::translations::list())
}

// ---------- bot backend ----------

/// GET /api/bot -> bot durum nesnesi (sabit sema, frontend sozlesmesi).
async fn api_bot_status(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    Json(ctx.bot.status().await)
}

/// GET /api/bot/status -> { status: {...} }
/// Panel durumu bu sargi altinda bekler (`b.status`), duz nesne degil.
async fn api_bot_status_wrapped(
    ctx: Arc<AppCtx>,
    _: axum::extract::Request,
) -> impl IntoResponse {
    Json(json!({ "status": ctx.bot.status().await }))
}

/// GET /api/bot/config -> { config, status, running, node_available }
/// Panelin bot sekmesini tek istekte doldurdugu birlesik uc.
async fn api_bot_config_get(
    ctx: Arc<AppCtx>,
    _: axum::extract::Request,
) -> impl IntoResponse {
    let cfg = ctx.bot.config().await;
    Json(json!({
        "config": {
            "enabled": cfg.enabled,
            "name": cfg.name,
            "host": cfg.host,
            "port": cfg.port,
            "vanish_x": cfg.vanish_x,
            "vanish_y": cfg.vanish_y,
            "vanish_z": cfg.vanish_z,
        },
        "status": ctx.bot.status().await,
        "running": ctx.bot.is_running().await,
        "node_available": crate::bot::node_available(),
    }))
}

/// POST /api/bot/toggle  body: { "enabled": bool }
/// GET ?on=true|false ile ayni isi yapar; panel JSON POST kullanir.
async fn api_bot_toggle_post(ctx: Arc<AppCtx>, Json(body): Json<Value>) -> impl IntoResponse {
    let on = body
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    bot_set_enabled(ctx, on).await
}

/// GET /api/bot/toggle?on=true|false
/// enabled'i kaydeder; sunucu ONLINE ise botu baslatir, degilse durdurur.
async fn api_bot_toggle(ctx: Arc<AppCtx>, q: axum::extract::Query<Value>) -> impl IntoResponse {
    let on = q.get("on").and_then(|v| v.as_str()) == Some("true");
    bot_set_enabled(ctx, on).await
}

/// Toggle'in ortak mantigi — GET (?on=) ve POST ({enabled}) ayni yoldan gecer.
async fn bot_set_enabled(ctx: Arc<AppCtx>, on: bool) -> Json<Value> {
    if let Err(e) = ctx.bot.set_enabled(on).await {
        return Json(json!({ "ok": false, "error": e }));
    }
    if on {
        // Sunucu ONLINE ise botu baslat. Host = state.server_addr (host kismi),
        // port = bot config port (veya adresten parse edilen).
        let (server_state, server_addr) = {
            let s = ctx.state.lock().await;
            (s.server_state.clone(), s.server_addr.clone())
        };
        let online = matches!(server_state.as_str(), "already" | "online");
        if online {
            // Online iken adresi yeniden tespit et: Aternos dinamik portu SADECE
            // sunucu calisirken gosterir. Cache portsuzsa port su an gelmis olabilir.
            let mut addr = server_addr.clone();
            let need_refresh = addr.as_deref().map_or(true, |a| !a.contains(':'));
            if need_refresh {
                if let Ok(full) = ctx.client.get_server_addr().await {
                    if full.contains(':') {
                        log(&ctx.tx, "sys", format!("adres (portlu) tespit edildi: {full}"));
                        let mut s = ctx.state.lock().await;
                        s.server_addr = Some(full.clone());
                        addr = Some(full);
                    }
                }
            }
            let cfg = ctx.bot.config().await;
            let host = addr
                .as_deref()
                .map(|a| split_addr(a).0)
                .unwrap_or(cfg.host.clone());
            let port_from_addr = addr
                .as_deref()
                .and_then(|a| split_addr(a).1);
            let port = port_from_addr.unwrap_or(cfg.port);
            log(&ctx.tx, "sys", format!("bot baslatiliyor: {host}:{port}"));
            if let Err(e) = ctx.bot.start(host, port).await {
                log(&ctx.tx, "err", format!("bot baslatma hatasi: {e}"));
            }
        }
    } else {
        ctx.bot.stop().await;
    }
    let enabled = ctx.bot.enabled().await;
    Json(json!({ "ok": true, "enabled": enabled }))
}

/// POST /api/bot/config  body: {name?, port?, vanish_x?, vanish_y?, vanish_z?, host?}
async fn api_bot_config(ctx: Arc<AppCtx>, Json(body): Json<Value>) -> impl IntoResponse {
    let patch = BotConfigPatch {
        name: body.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        host: body.get("host").and_then(|v| v.as_str()).map(|s| s.to_string()),
        port: body.get("port").and_then(|v| v.as_u64()).and_then(|n| u16::try_from(n).ok()),
        vanish_x: body.get("vanish_x").and_then(|v| v.as_f64()),
        vanish_y: body.get("vanish_y").and_then(|v| v.as_f64()),
        vanish_z: body.get("vanish_z").and_then(|v| v.as_f64()),
    };
    match ctx.bot.update_config(patch).await {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_cookies, split_token};

    #[test]
    fn splits_combined_token_and_sec() {
        // panelin tarif ettigi AJAX_TOKEN|generateAjaxToken() formati
        let (t, s) = split_token("LLK12U2TNWWp6KoZj7gn|zii7mb2mqn000000:gpnl9xr52o000000");
        assert_eq!(t, "LLK12U2TNWWp6KoZj7gn");
        assert_eq!(s, "zii7mb2mqn000000:gpnl9xr52o000000");
    }

    #[test]
    fn token_without_separator_leaves_sec_empty() {
        let (t, s) = split_token("  PLAINTOKEN  ");
        assert_eq!(t, "PLAINTOKEN");
        assert_eq!(s, "");
    }

    #[test]
    fn split_token_trims_around_separator() {
        let (t, s) = split_token(" abc | def ");
        assert_eq!(t, "abc");
        assert_eq!(s, "def");
    }

    #[test]
    fn split_token_keeps_extra_separators_in_sec() {
        // sec kisminda '|' varsa bolunmemeli — sadece ilk ayirici sayilir
        let (t, s) = split_token("tok|a|b");
        assert_eq!(t, "tok");
        assert_eq!(s, "a|b");
    }

    #[test]
    fn parses_standard_cookie_header() {
        let c = parse_cookies("ATERNOS_SESSION=abc; ATERNOS_SERVER=xyz");
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].name, "ATERNOS_SESSION");
        assert_eq!(c[0].value, "abc");
        assert_eq!(c[1].name, "ATERNOS_SERVER");
        assert_eq!(c[1].value, "xyz");
    }

    #[test]
    fn trims_whitespace_around_pairs() {
        let c = parse_cookies("  A = 1 ;  B=2  ");
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].name, "A");
        assert_eq!(c[0].value, "1");
        assert_eq!(c[1].name, "B");
        assert_eq!(c[1].value, "2");
    }

    #[test]
    fn skips_pairs_without_equals_and_empty_names() {
        // "garbage" (no =) and a leading "=novalue" (empty name) are dropped
        let c = parse_cookies("garbage; =novalue; OK=1");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].name, "OK");
    }

    #[test]
    fn keeps_equals_in_value() {
        // base64/JWT values often contain '=' padding; only first '=' splits
        let c = parse_cookies("TOK=a=b=c");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].value, "a=b=c");
    }

    #[test]
    fn empty_input_yields_no_cookies() {
        assert!(parse_cookies("").is_empty());
        assert!(parse_cookies("   ").is_empty());
    }
}

// ---------- setup-mode backend ----------

/// Kurulum ekranindaki tek token alanini (token, sec) ikilisine ayirir.
///
/// Panel kullaniciya `window.AJAX_TOKEN + "|" + window.generateAjaxToken()`
/// ciktisini yapistirmasini soyler; bu iki degeri '|' ile ayrilmis tek string
/// olarak getirir. Ayirici yoksa tamami token'dir, sec bos kalir.
fn split_token(raw: &str) -> (String, String) {
    match raw.split_once('|') {
        Some((t, s)) => (t.trim().to_string(), s.trim().to_string()),
        None => (raw.trim().to_string(), String::new()),
    }
}

fn parse_cookies(raw: &str) -> Vec<Cookie> {
    raw.split(';').filter_map(|pair| {
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
    }).collect()
}

/// setup tamamlandiktan sonra ayni binary'yi ayni argumanlarla yeniden baslatir
/// ve mevcut process'i kapatir. session.enc artik mevcut oldugu icin yeni process
/// normal (kurulu client ile) baslayacaktir.
/// `password`: yeniden baslayan surece parolayi ATERKEEP_KEY ile aktarir.
/// Anahtar diskte tutulmadigi icin cocuk surec oturumu baska turlu acamaz.
/// Parola sadece cocugun ortaminda kalir, diske hic yazilmaz.
fn self_restart(password: Option<&str>) {
    let exe = std::env::current_exe().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(exe) = exe {
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(&args);
        if let Some(p) = password {
            cmd.env("ATERKEEP_KEY", p);
        }
        let _ = cmd.spawn();
    }
    println!("[setup] process yeniden baslatiliyor");
    std::process::exit(0);
}

/// Kurulum gerekli mi? Oturum dosyasi yoksa ya da panel parolasi hic
/// belirlenmemisse evet.
async fn api_needs_setup() -> impl IntoResponse {
    let cfg = AppConfig::load();
    let needed = !config::session_path().exists() || !cfg.auth_enabled();
    Json(json!({ "needs_setup": needed }))
}

/// GET /api/boot -> { setup_mode: bool }
/// Panel acilir acilmaz bunu sorar: true ise kurulum overlay'ini gosterir ve
/// sekmeleri kilitler. `needs_setup` ile ayni kaynaktan beslenir.
/// Kimlik dogrulamasi ISTEMEZ — panel acilirken hangi ekrani gosterecegini
/// (kurulum / giris / panel) buradan ogrenir.
async fn api_boot(headers: axum::http::HeaderMap) -> impl IntoResponse {
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
    }))
}

/// POST /api/setup/reset -> oturum dosyasini siler ve process'i yeniden baslatir.
/// Aternos oturumu ~30 gunde bir doldugu icin bu, panelden tek tikla yeniden
/// kurulum akisina donmeyi saglar.
async fn api_setup_reset() -> impl IntoResponse {
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
async fn api_setup(Json(body): Json<Value>) -> impl IntoResponse {
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
    if token.is_empty() && cookies.is_empty() {
        return Json(json!({ "ok": false, "error": "token veya cookie gerekli" }));
    }

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
    if let Err(e) = cfg.save() {
        return Json(json!({ "ok": false, "error": format!("config yazilamadi: {e}") }));
    }

    let strings = match aterkeep_core::Strings::decrypt_all() {
        Ok(s) => s,
        Err(e) => return Json(json!({ "ok": false, "error": format!("strings: {e}") })),
    };
    // server_id: ATERNOS_SERVER cookie'sinden tespit et, yoksa body'den al
    let sid = cookies
        .iter()
        .find(|c| c.name == strings.c_server)
        .map(|c| c.value.clone())
        .unwrap_or(server_id);
    let mut sess = Session {
        token,
        sec,
        cookies,
        server_id: sid,
        server_addr: None,
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
