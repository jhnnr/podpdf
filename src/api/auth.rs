use crate::auth::{ApiKeyEntry, AuthManager};
use crate::config::Config;
use crate::error::AppError;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub auth: Arc<AuthManager>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    #[serde(alias = "username")]
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct AuthStatusResponse {
    pub auth_enabled: bool,
    pub logged_in: bool,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangeCredentialsRequest {
    #[serde(alias = "new_username")]
    pub new_email: Option<String>,
    pub current_password: String,
    pub new_password: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key: String,
    pub key_prefix: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct SystemInfoResponse {
    pub app_name: &'static str,
    pub version: &'static str,
    pub auth_enabled: bool,
    pub telemetry: &'static str,
    pub status: &'static str,
}

#[derive(Deserialize)]
pub struct ToggleSecurityRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct ToggleSecurityResponse {
    pub success: bool,
    pub auth_enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateCustomHeadersRequest {
    pub headers: HashMap<String, String>,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let token = state
        .auth
        .login(&payload.email, &payload.password)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(LoginResponse {
        token,
        email: payload.email,
    }))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SuccessResponse>, AppError> {
    if let Some(token) = extract_token_from_headers(&headers) {
        state.auth.logout(&token).await;
    }

    Ok(Json(SuccessResponse {
        success: true,
        message: "Successfully logged out".to_string(),
    }))
}

pub async fn auth_status_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<AuthStatusResponse> {
    let token = extract_token_from_headers(&headers);
    let email = if let Some(ref t) = token {
        state.auth.validate_session(t).await
    } else {
        None
    };

    let logged_in = email.is_some();
    let auth_enabled = state.config.read().await.auth_enabled;

    Json(AuthStatusResponse {
        auth_enabled,
        logged_in,
        email,
    })
}

pub async fn change_credentials_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangeCredentialsRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
    let current_email = require_authenticated_user(&state, &headers).await?;

    state
        .auth
        .change_credentials(
            &current_email,
            payload.new_email,
            &payload.current_password,
            payload.new_password,
        )
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "Credentials successfully updated".to_string(),
    }))
}

pub async fn list_api_keys_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ApiKeyEntry>>, AppError> {
    let _ = require_authenticated_user(&state, &headers).await?;
    let keys = state.auth.list_api_keys().await;
    Ok(Json(keys))
}

pub async fn create_api_key_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, AppError> {
    let _ = require_authenticated_user(&state, &headers).await?;

    let (raw_key, entry) = state
        .auth
        .generate_api_key(&payload.name)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(CreateApiKeyResponse {
        id: entry.id,
        name: entry.name,
        key: raw_key,
        key_prefix: entry.key_prefix,
        created_at: entry.created_at,
    }))
}

pub async fn revoke_api_key_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse>, AppError> {
    let _ = require_authenticated_user(&state, &headers).await?;

    state
        .auth
        .revoke_api_key(&id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "API key successfully revoked".to_string(),
    }))
}

pub async fn toggle_security_mode_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ToggleSecurityRequest>,
) -> Result<Json<ToggleSecurityResponse>, AppError> {
    let _ = require_authenticated_user(&state, &headers).await?;

    let mut config = state.config.write().await;
    config.auth_enabled = payload.enabled;
    config.save_to_env();

    Ok(Json(ToggleSecurityResponse {
        success: true,
        auth_enabled: payload.enabled,
    }))
}

pub async fn get_custom_headers_handler(
    State(state): State<AppState>,
) -> Json<HashMap<String, String>> {
    let conf = state.config.read().await;
    Json(conf.custom_headers.clone())
}

pub async fn update_custom_headers_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateCustomHeadersRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
    let _ = require_authenticated_user(&state, &headers).await?;

    let mut conf = state.config.write().await;
    conf.custom_headers = payload.headers;
    conf.save_to_env();

    Ok(Json(SuccessResponse {
        success: true,
        message: "Custom HTTP headers updated successfully".to_string(),
    }))
}

pub async fn system_info_handler(State(state): State<AppState>) -> Json<SystemInfoResponse> {
    let auth_enabled = state.config.read().await.auth_enabled;
    Json(SystemInfoResponse {
        app_name: "PodPdf",
        version: env!("CARGO_PKG_VERSION"),
        auth_enabled,
        telemetry: "None (100% Offline & Private)",
        status: "Operational",
    })
}

fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization")
        && let Ok(s) = auth.to_str()
    {
        let token = s
            .trim_start_matches("Bearer ")
            .trim_start_matches("bearer ")
            .trim();
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    if let Some(key) = headers.get("x-api-key")
        && let Ok(s) = key.to_str()
    {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(cookie) = headers.get("cookie")
        && let Ok(s) = cookie.to_str()
    {
        for part in s.split(';') {
            let part = part.trim();
            if let Some(token) = part.strip_prefix("podpdf_session=") {
                return Some(token.to_string());
            }
        }
    }

    None
}

async fn require_authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<String, AppError> {
    if let Some(token) = extract_token_from_headers(headers) {
        if let Some(email) = state.auth.validate_session(&token).await {
            return Ok(email);
        }
        if state.auth.validate_api_key(&token).await {
            return Ok("api-key-user".to_string());
        }
    }

    let auth_enabled = state.config.read().await.auth_enabled;
    if !auth_enabled {
        return Ok("admin@podpdf.local".to_string());
    }

    Err(AppError::BadRequest(
        "Authentication required to perform this action".to_string(),
    ))
}
