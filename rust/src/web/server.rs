//! Sunucu verileri: server.properties, oyuncu listesi, konsol ve canli akis.

use super::AppCtx;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

pub(super) async fn api_stream(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
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

pub(super) async fn api_options(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    match ctx.client.get_options().await {
        Ok(v) => Json(v),
        Err(e) => Json(json!({"error": e})),
    }
}

pub(super) async fn api_option_set(
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

pub(super) async fn api_players(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    match ctx.client.get_players().await {
        Ok(v) => Json(v),
        Err(e) => Json(json!({"error": e})),
    }
}

pub(super) async fn api_console(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
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
