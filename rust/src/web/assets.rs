//! Binary'ye gomulu statik dosyalar.
//!
//! Panel tek dosyalik bir urun: HTML/CSS/JS derleme aninda gomulur, diskte
//! aranmaz. Onbellek KAPALI — varliklar surum degisse de ayni URL'de kalir,
//! yani tarayici eski surumu suresiz saklayabilirdi.

use axum::response::IntoResponse;

/// Statik varliklar binary'ye gomulu oldugu icin surum atlandiginda dosya adi
/// degismez; cache basligi olmadan tarayici eski panelde takili kalir (yeni
/// surumde kurulum ekranini hic gormemek gibi teshis edilmesi zor sonuclar
/// dogurur). Her yanit yeniden dogrulansin.
pub(super) const NO_CACHE: (&str, &str) = ("Cache-Control", "no-cache, must-revalidate");

pub(super) async fn index() -> impl IntoResponse {
    (
        [NO_CACHE],
        axum::response::Html(include_str!("../../static/index.html")),
    )
}

pub(super) async fn style_css() -> impl IntoResponse {
    (
        [("Content-Type", "text/css"), NO_CACHE],
        include_str!("../../static/style.css"),
    )
}

pub(super) async fn app_js() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript"), NO_CACHE],
        include_str!("../../static/app.js"),
    )
}

pub(super) async fn logo_svg() -> impl IntoResponse {
    (
        [("Content-Type", "image/svg+xml"), NO_CACHE],
        include_str!("../../static/logo.svg"),
    )
}
