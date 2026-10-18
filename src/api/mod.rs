pub mod auth;
pub mod compare;
pub mod convert;
pub mod favicon;
pub mod health;
pub mod merge;
pub mod metadata;
pub mod middleware;
pub mod optimize;
pub mod pages;
pub mod rotate;
pub mod security;
pub mod split;
pub mod transform;
pub mod watermark;

use crate::auth::AuthManager;
use crate::config::Config;
use auth::AppState;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware as axum_middleware;
use axum::response::Html;
use axum::routing::{delete, get, post};
use std::sync::Arc;
use tokio::sync::RwLock;

const INDEX_HTML: &str = include_str!("../static/index.html");

async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Constructs the application routes, API handlers, authentication state, and middleware.
pub fn routes(config: &Config, auth: Arc<AuthManager>) -> Router {
    let app_state = AppState {
        config: Arc::new(RwLock::new(config.clone())),
        auth,
    };

    let pdf_routes = Router::new()
        .route("/merge", post(merge::merge_handler))
        .route("/split", post(split::split_handler))
        .route("/rotate", post(rotate::rotate_handler))
        .route("/extract", post(pages::extract_handler))
        .route("/remove", post(pages::remove_handler))
        .route("/reorder", post(pages::reorder_handler))
        .route("/reverse", post(transform::reverse_handler))
        .route("/crop", post(transform::crop_handler))
        .route("/scale", post(transform::scale_handler))
        .route("/watermark", post(watermark::watermark_handler))
        .route("/page-numbers", post(watermark::page_numbers_handler))
        .route("/images-to-pdf", post(convert::images_to_pdf_handler))
        .route("/text-to-pdf", post(convert::text_to_pdf_handler))
        .route("/pdf-to-text", post(convert::pdf_to_text_handler))
        .route("/extract-images", post(convert::extract_images_handler))
        .route("/encrypt", post(security::encrypt_handler))
        .route("/decrypt", post(security::decrypt_handler))
        .route("/flatten", post(security::flatten_handler))
        .route("/redact", post(security::redact_handler))
        .route("/compress", post(optimize::compress_handler))
        .route("/compare", post(compare::compare_handler))
        .route("/metadata/read", post(metadata::read_metadata_handler))
        .route("/metadata/update", post(metadata::update_metadata_handler))
        .route("/metadata/strip", post(metadata::strip_metadata_handler))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    let auth_routes = Router::new()
        .route("/login", post(auth::login_handler))
        .route("/logout", post(auth::logout_handler))
        .route("/status", get(auth::auth_status_handler))
        .route(
            "/change-credentials",
            post(auth::change_credentials_handler),
        )
        .route(
            "/api-keys",
            get(auth::list_api_keys_handler).post(auth::create_api_key_handler),
        )
        .route("/api-keys/{id}", delete(auth::revoke_api_key_handler));

    let system_routes = Router::new()
        .route("/info", get(auth::system_info_handler))
        .route("/security-mode", post(auth::toggle_security_mode_handler))
        .route(
            "/headers",
            get(auth::get_custom_headers_handler).post(auth::update_custom_headers_handler),
        );

    let api_v1 = Router::new()
        .route("/health", get(health::health_check))
        .nest("/auth", auth_routes)
        .nest("/pdf", pdf_routes)
        .nest("/system", system_routes);

    Router::new()
        .route("/", get(serve_index))
        .route("/favicon.ico", get(favicon::serve_favicon))
        .route("/health", get(health::health_check))
        .nest("/api/v1", api_v1)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::custom_headers_middleware,
        ))
        .layer(DefaultBodyLimit::max(config.max_payload_size))
        .with_state(app_state)
}
