use crate::api::auth::AppState;
use axum::Json;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Middleware to enforce authentication on protected endpoints when auth_enabled is set.
pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let auth_enabled = state.config.read().await.auth_enabled;
    if !auth_enabled {
        return next.run(req).await;
    }

    let headers = req.headers();

    let mut authenticated = false;

    if let Some(auth_header) = headers.get("authorization")
        && let Ok(s) = auth_header.to_str()
        && state.auth.validate_any(s).await
    {
        authenticated = true;
    }

    if !authenticated
        && let Some(api_key) = headers.get("x-api-key")
        && let Ok(s) = api_key.to_str()
        && state.auth.validate_api_key(s.trim()).await
    {
        authenticated = true;
    }

    if !authenticated
        && let Some(cookie) = headers.get("cookie")
        && let Ok(s) = cookie.to_str()
    {
        for part in s.split(';') {
            let part = part.trim();
            if let Some(token) = part.strip_prefix("podpdf_session=")
                && state.auth.validate_session(token).await.is_some()
            {
                authenticated = true;
                break;
            }
        }
    }

    if authenticated {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized: Please log in or supply a valid API key (X-API-Key)",
                "code": "UNAUTHORIZED"
            })),
        )
            .into_response()
    }
}

/// Middleware that injects custom configured HTTP response headers into all responses.
pub async fn custom_headers_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let headers_to_inject = state.config.read().await.custom_headers.clone();
    let mut response = next.run(req).await;

    for (k, v) in headers_to_inject {
        if let Ok(name) = HeaderName::from_bytes(k.as_bytes())
            && let Ok(val) = HeaderValue::from_str(&v)
        {
            response.headers_mut().insert(name, val);
        }
    }

    response
}
