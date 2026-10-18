use podpdf::config::Config;
use podpdf::create_app;
use std::collections::HashMap;
use tower::ServiceExt;

#[tokio::test]
async fn test_login_flow() {
    let auth_store =
        std::env::temp_dir().join(format!("podpdf_auth_test_{}.json", rand::random::<u64>()));

    let config = Config {
        auth_store_path: auth_store.to_str().unwrap().to_string(),
        auth_enabled: true,
        ..Default::default()
    };
    let app = create_app(&config);

    // 1. Wrong credentials should fail
    let bad_login = serde_json::json!({
        "email": "admin@podpdf.local",
        "password": "wrongpassword"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&bad_login).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::BAD_REQUEST);

    // 2. Default admin email/password login succeeds
    let good_login = serde_json::json!({
        "email": "admin@podpdf.local",
        "password": "admin"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&good_login).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    // 2b. Logging in with "admin" alias also succeeds
    let alias_login = serde_json::json!({
        "email": "admin",
        "password": "admin"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&alias_login).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = json["token"].as_str().unwrap().to_string();
    assert!(!token.is_empty());

    // 3. Status endpoint shows logged in with valid token
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/auth/status")
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let status_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(status_json["logged_in"], true);
    assert_eq!(status_json["email"], "admin@podpdf.local");
}

#[tokio::test]
async fn test_api_key_lifecycle() {
    let auth_store =
        std::env::temp_dir().join(format!("podpdf_auth_test_{}.json", rand::random::<u64>()));

    let config = Config {
        auth_store_path: auth_store.to_str().unwrap().to_string(),
        auth_enabled: true,
        ..Default::default()
    };
    let app = create_app(&config);

    // Login as admin
    let login_payload = serde_json::json!({
        "email": "admin@podpdf.local",
        "password": "admin"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&login_payload).unwrap(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // 1. Create API key
    let create_key_payload = serde_json::json!({
        "name": "Integration Test Key"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/api-keys")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&create_key_payload).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let key_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let key_id = key_json["id"].as_str().unwrap().to_string();
    let raw_api_key = key_json["key"].as_str().unwrap().to_string();
    assert!(raw_api_key.starts_with("podpdf_"));

    // 2. List API keys
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/auth/api-keys")
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    // 3. System info endpoint works
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/system/info")
        .body(axum::body::Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    // 4. Revoke API key
    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/auth/api-keys/{}", key_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_change_credentials() {
    let auth_store =
        std::env::temp_dir().join(format!("podpdf_auth_test_{}.json", rand::random::<u64>()));

    let config = Config {
        auth_store_path: auth_store.to_str().unwrap().to_string(),
        auth_enabled: true,
        ..Default::default()
    };
    let app = create_app(&config);

    // Login as admin
    let login_payload = serde_json::json!({
        "email": "admin@podpdf.local",
        "password": "admin"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&login_payload).unwrap(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Change credentials to operator@podpdf.local / newSecret999
    let change_payload = serde_json::json!({
        "new_email": "operator@podpdf.local",
        "current_password": "admin",
        "new_password": "newSecret999"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/change-credentials")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&change_payload).unwrap(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    // Login with new credentials succeeds
    let new_login = serde_json::json!({
        "email": "operator@podpdf.local",
        "password": "newSecret999"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&new_login).unwrap(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_auth_middleware_enforcement() {
    let auth_store =
        std::env::temp_dir().join(format!("podpdf_auth_test_{}.json", rand::random::<u64>()));

    let config = Config {
        auth_store_path: auth_store.to_str().unwrap().to_string(),
        auth_enabled: true,
        ..Default::default()
    };
    let app = create_app(&config);

    // 1. Unauthenticated request to protected PDF route returns 401
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/reverse")
        .body(axum::body::Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_toggle_security_mode() {
    let auth_store =
        std::env::temp_dir().join(format!("podpdf_auth_test_{}.json", rand::random::<u64>()));

    let config = Config {
        auth_store_path: auth_store.to_str().unwrap().to_string(),
        auth_enabled: true,
        ..Default::default()
    };
    let app = create_app(&config);

    // Login as admin
    let login_payload = serde_json::json!({
        "email": "admin@podpdf.local",
        "password": "admin"
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&login_payload).unwrap(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Toggle security mode to false (disabled)
    let toggle_payload = serde_json::json!({
        "enabled": false
    });
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/system/security-mode")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&toggle_payload).unwrap(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    // Now protected endpoint can be reached without auth (returns 400 Bad Request because body is empty, NOT 401)
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/reverse")
        .body(axum::body::Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_custom_http_headers() {
    let auth_store =
        std::env::temp_dir().join(format!("podpdf_auth_test_{}.json", rand::random::<u64>()));

    let mut custom_headers = HashMap::new();
    custom_headers.insert(
        "X-Custom-Security".to_string(),
        "PodPdfVerified".to_string(),
    );
    custom_headers.insert("X-Frame-Options".to_string(), "DENY".to_string());

    let config = Config {
        auth_store_path: auth_store.to_str().unwrap().to_string(),
        auth_enabled: false,
        custom_headers,
        ..Default::default()
    };
    let app = create_app(&config);

    // Verify response contains configured custom headers
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    assert_eq!(
        res.headers().get("X-Custom-Security").unwrap(),
        "PodPdfVerified"
    );
    assert_eq!(res.headers().get("X-Frame-Options").unwrap(), "DENY");
}
