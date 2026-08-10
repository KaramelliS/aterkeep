//! Sunucu durumu ve aksiyonlar (start/stop/restart/cancel/confirm/extend).
//!
//! ONEMLI: durum sorgusu bir AKSIYON DEGILDIR. Bu modulun okuma yolu Aternos'a
//! yalnizca `get_status` gonderir; eskiden buradan `start()` cagriliyordu ve
//! panel acik durdukca 2 saniyede bir gercek baslatma istegi gidiyordu.

use super::{now_unix, AppCtx};
use aterkeep_core::log;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) async fn api_status(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
    // anlik durum: son cekim 2sn'den eskiyse gercek bir start() cagir (throttle)
    let now = now_unix();
    // Panel acikken durumu tazele — ama YALNIZCA OKUYARAK.
    //
    // Burada eskiden `client.start(false)` cagriliyordu: panel acik durdugu
    // surece Aternos'a 2 saniyede bir GERCEK bir baslatma istegi gidiyordu.
    // Uc ayri sonucu vardi: (1) Aternos'un "yapay aktivite" denetimi icin
    // bariz bir imza — hesabin askiya alinma riski, (2) kullanici Durdur'a
    // bastiginda sunucu kendiliginden geri aciliyordu, (3) oturum dustukten
    // sonra bile atmaya devam ediyordu. Durum sorgusu bir AKSIYON degildir.
    //
    // Mesgul bayragi tek kilit altinda kontrol edilip set ediliyor: onceden
    // kontrol ve set ayri kilitlerdeydi ve keepalive ile bu yol ayni anda
    // gecebiliyordu. (Panik durumunda bayragin asili kalmasi mumkun degil:
    // release profilinde `panic = "abort"`, surec zaten oluyor.)
    {
        let mut s = ctx.state.lock().await;
        let claim = now.saturating_sub(s.last_poll) >= 2 && !s.busy;
        if claim {
            s.last_poll = now;
            s.busy = true;
        }
        drop(s);

        if claim {
            let resp = ctx.client.get_status().await;
            let mut s = ctx.state.lock().await;
            match resp {
                Ok(v) => {
                    s.server_state = v
                        .get("status")
                        .and_then(|x| x.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    s.last_check = now;
                    s.session_expired = false;
                    if matches!(
                        s.server_state.as_str(),
                        "queue" | "inline" | "waiting" | "pending"
                    ) {
                        s.queue = Some(v.clone());
                    } else {
                        s.queue = None;
                    }
                    s.last_request = Some(json!({"action": "status", "response": v}));
                }
                Err(e) if e.starts_with(aterkeep_core::ERR_SESSION_EXPIRED) => {
                    s.session_expired = true;
                    s.server_state = "session_expired".into();
                    s.last_check = now;
                }
                Err(_) => { /* gecici ag hatasi — bilinen durumu koru */ }
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
    let (session_age, last_lifetime) = {
        let cfg = ctx.cfg.lock().await;
        (
            cfg.session_started.map(|t| now.saturating_sub(t)),
            cfg.last_session_lifetime,
        )
    };
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
        // Cerezler gecersizse panel bunu "sunucu kapali" diye degil, ne
        // yapilmasi gerektigini soyleyerek gosterir (bkz. app.js).
        "session_expired": s.session_expired,
        // Cerezlerin kac saniyedir ayakta oldugu ve bir onceki oturumun ne
        // kadar dayandigi. Aternos cerez omrunu ilan etmedigi icin bu sayilar
        // tahminin yerini alan tek gercek veri.
        "session_age": session_age,
        "last_session_lifetime": last_lifetime,
    }))
}

pub(super) async fn api_action(
    ctx: Arc<AppCtx>,
    Path(action): Path<String>,
) -> impl IntoResponse {
    let valid = ["start", "stop", "restart"].contains(&action.as_str());
    if !valid {
        return Json(json!({"ok": false, "error": "gecersiz"}));
    }
    // Mesgul bayragini TEK kilit altinda kontrol et ve sahiplen. Onceden
    // kontrol ve set ayri kilitlerdeydi: keepalive dongusu ile bu yol ayni
    // anda gecebiliyor, kullanicinin "Durdur"u ile dongunun "start"i cakisip
    // sunucu kendiliginden geri aciliyordu.
    {
        let mut s = ctx.state.lock().await;
        if s.busy {
            return Json(json!({"ok": false, "error": "mesgul"}));
        }
        s.busy = true;
    }
    let client = ctx.client.clone();
    let state = ctx.state.clone();
    let tx = ctx.tx.clone();
    // AppCtx'in kendisini tasi: cfg ondan okunur. (Arc klonu yeterli.)
    let ctx_for_task = ctx.clone();
    tokio::spawn(async move {
        // Inner logic: herhangi bir early-return veya beklenmedik durumda bile
        // aşağıdaki busy=false set'inin kesinlikle çalışması için bloğa alıyoruz.
        let inner = async {
            log(&tx, "cmd", format!("$ aternos {action}"));
            if action == "stop" {
                {
                    let mut s = state.lock().await;
                    s.auto = false;
                }
                // Kalici yaz — yoksa yeniden baslatmada sunucu geri aciliyordu.
                {
                    let mut c = ctx_for_task.cfg.lock().await;
                    c.auto_start = false;
                    let _ = c.save();
                }
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
pub(super) async fn api_cancel(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
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
pub(super) async fn api_confirm(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
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
pub(super) async fn api_extend(ctx: Arc<AppCtx>, _: axum::extract::Request) -> impl IntoResponse {
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

pub(super) async fn api_toggle(ctx: Arc<AppCtx>, q: axum::extract::Query<Value>) -> impl IntoResponse {
    let on = q.get("on").and_then(|v| v.as_str()) == Some("true");
    {
        let mut s = ctx.state.lock().await;
        s.auto = on;
    }
    // Tercihi KALICI yaz: yoksa daemon her yeniden basladiginda 7/24 modu
    // sessizce geri geliyor ve kullanicinin bilerek kapattigi sunucu aciliyordu.
    {
        let mut cfg = ctx.cfg.lock().await;
        cfg.auto_start = on;
        if let Err(e) = cfg.save() {
            log(&ctx.tx, "err", format!("oto-baslat tercihi yazilamadi: {e}"));
        }
    }
    log(
        &ctx.tx,
        "sys",
        format!("oto-baslat {}", if on { "ACIK" } else { "KAPALI" }),
    );
    Json(json!({"ok": true, "auto": on}))
}
