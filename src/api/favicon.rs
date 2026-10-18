use axum::http::header;
use axum::response::IntoResponse;

pub const FAVICON_ICO: &[u8] = include_bytes!("../static/favicon.ico");

pub async fn serve_favicon() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/x-icon")], FAVICON_ICO)
}
