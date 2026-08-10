//! Anti-idle bot uclari.

use super::AppCtx;
use crate::bot::BotConfigPatch;
use aterkeep_core::{log, split_addr};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /api/bot -> bot durum nesnesi (sabit sema, frontend sozlesmesi).
pub(super) async fn api_bot_status(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    Json(ctx.bot.status().await)
}

/// GET /api/bot/status -> { status: {...} }
/// Panel durumu bu sargi altinda bekler (`b.status`), duz nesne degil.
pub(super) async fn api_bot_status_wrapped(
    ctx: Arc<AppCtx>,
    _: axum::extract::Request,
) -> impl IntoResponse {
    Json(json!({ "status": ctx.bot.status().await }))
}

/// GET /api/bot/config -> { config, status, running }
/// Panelin bot sekmesini tek istekte doldurdugu birlesik uc.
pub(super) async fn api_bot_config_get(
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
    }))
}

/// POST /api/bot/toggle  body: { "enabled": bool }
/// GET ?on=true|false ile ayni isi yapar; panel JSON POST kullanir.
pub(super) async fn api_bot_toggle_post(ctx: Arc<AppCtx>, Json(body): Json<Value>) -> impl IntoResponse {
    let on = body
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    bot_set_enabled(ctx, on).await
}

/// GET /api/bot/toggle?on=true|false
/// enabled'i kaydeder; sunucu ONLINE ise botu baslatir, degilse durdurur.
pub(super) async fn api_bot_toggle(ctx: Arc<AppCtx>, q: axum::extract::Query<Value>) -> impl IntoResponse {
    let on = q.get("on").and_then(|v| v.as_str()) == Some("true");
    bot_set_enabled(ctx, on).await
}

/// Toggle'in ortak mantigi — GET (?on=) ve POST ({enabled}) ayni yoldan gecer.
pub(super) async fn bot_set_enabled(ctx: Arc<AppCtx>, on: bool) -> Json<Value> {
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
            let need_refresh = addr.as_deref().is_none_or(|a| !a.contains(':'));
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
pub(super) async fn api_bot_config(ctx: Arc<AppCtx>, Json(body): Json<Value>) -> impl IntoResponse {
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
