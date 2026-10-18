pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod pdf;

use auth::AuthManager;
use axum::Router;
use config::Config;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Creates the primary application router with all API routes, state, and middleware.
pub fn create_app(config: &Config) -> Router {
    let auth = AuthManager::new(&config.auth_store_path);
    let cors = CorsLayer::permissive();

    Router::new()
        .merge(api::routes(config, auth))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
