//! Ceviri uclari. Dil dosyalari binary'ye gomuludur (translations.rs).

use super::AppCtx;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

pub(super) async fn api_i18n(
    _ctx: Arc<AppCtx>,
    Path(lang): Path<String>,
) -> impl IntoResponse {
    Json(crate::translations::get(&lang))
}

pub(super) async fn api_i18n_list() -> impl IntoResponse {
    Json(crate::translations::list())
}
