use axum::http::header;
use axum::response::IntoResponse;

pub const FAVICON_SVG: &str = include_str!("../static/favicon.svg");

pub async fn serve_favicon() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}
