//! Panel kimlik dogrulama: giris, cikis ve tum /api/* uclarini koruyan katman.

use super::{parse_cookies, AppCtx};
use crate::config;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

/// Panel cerezinden oturum jetonunu okur.
pub(super) fn token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    parse_cookies(raw)
        .into_iter()
        .find(|c| c.name == AUTH_COOKIE)
        .map(|c| c.value)
}

pub(super) const AUTH_COOKIE: &str = "aterkeep_auth";

/// Istek yetkili mi? Auth kapaliysa (kurulum yapilmamis) her istek gecer.
pub(super) async fn is_authed(ctx: &AppCtx, headers: &axum::http::HeaderMap) -> bool {
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
pub(super) async fn auth_layer(
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

/// POST /api/login  body: { "password": "..." }
/// Dogruysa rastgele jeton uretir, HttpOnly cerez olarak yazar.
pub(super) async fn api_login(ctx: Arc<AppCtx>, Json(body): Json<Value>) -> impl IntoResponse {
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
pub(super) async fn api_logout(ctx: Arc<AppCtx>) -> impl IntoResponse {
    *ctx.auth_token.lock().await = None;
    let cookie = format!("{AUTH_COOKIE}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0");
    (
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(json!({ "ok": true })),
    )
        .into_response()
}
