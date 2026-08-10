//! Panel HTTP katmani.
//!
//! Sorumluluklar ayri modullerde: kimlik dogrulama, statik varliklar, durum ve
//! aksiyonlar, sunucu verileri, bot, kurulum, ceviri. Bu dosyada yalnizca
//! paylasilan durum (AppCtx), rota kurulumu ve iki kucuk ayristirici kalir.

mod assets;
mod auth;
mod bot;
mod i18n;
mod server;
mod setup;
mod status;

pub use setup::{run_setup_mode, self_restart};

use crate::config::AppConfig;
use crate::bot::SharedBot;
use aterkeep_core::{AternosClient, Cookie, LogTx, SharedConsole, SharedState};
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

// Router bu uclari alt modullerden toplar.
use assets::{app_js, index, logo_svg, style_css};
use auth::{api_login, api_logout, auth_layer};
use bot::{
    api_bot_config, api_bot_config_get, api_bot_status, api_bot_status_wrapped,
    api_bot_toggle, api_bot_toggle_post,
};
use i18n::{api_i18n, api_i18n_list};
use server::{api_console, api_option_set, api_options, api_players, api_stream};
use setup::{api_boot, api_needs_setup, api_setup, api_setup_reset};
use status::{api_action, api_cancel, api_confirm, api_extend, api_status, api_toggle};

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

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Kurulum ekranindaki tek token alanini (token, sec) ikilisine ayirir.
///
/// Panel kullaniciya `window.AJAX_TOKEN + "|" + window.generateAjaxToken()`
/// ciktisini yapistirmasini soyler; bu iki degeri '|' ile ayrilmis tek string
/// olarak getirir. Ayirici yoksa tamami token'dir, sec bos kalir.
pub(crate) fn split_token(raw: &str) -> (String, String) {
    match raw.split_once('|') {
        Some((t, s)) => (t.trim().to_string(), s.trim().to_string()),
        None => (raw.trim().to_string(), String::new()),
    }
}

pub(crate) fn parse_cookies(raw: &str) -> Vec<Cookie> {
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
