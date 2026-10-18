mod common;

use common::{generate_test_pdf, generate_test_pdf_with_info, generate_test_png};
use podpdf::config::Config;
use podpdf::create_app;
use podpdf::pdf::page_count;
use tower::ServiceExt;

struct MultipartBuilder {
    boundary: String,
    data: Vec<u8>,
}

impl MultipartBuilder {
    fn new() -> Self {
        Self {
            boundary: "----PodPdfTestBoundaryXYZ123".to_string(),
            data: Vec::new(),
        }
    }

    fn add_file(mut self, name: &str, filename: &str, content: &[u8]) -> Self {
        self.data
            .extend_from_slice(format!("--{}\r\n", self.boundary).as_bytes());
        self.data.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                name, filename
            )
            .as_bytes(),
        );
        self.data
            .extend_from_slice(b"Content-Type: application/pdf\r\n\r\n");
        self.data.extend_from_slice(content);
        self.data.extend_from_slice(b"\r\n");
        self
    }

    fn add_image(mut self, name: &str, filename: &str, content: &[u8]) -> Self {
        self.data
            .extend_from_slice(format!("--{}\r\n", self.boundary).as_bytes());
        self.data.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                name, filename
            )
            .as_bytes(),
        );
        self.data
            .extend_from_slice(b"Content-Type: image/png\r\n\r\n");
        self.data.extend_from_slice(content);
        self.data.extend_from_slice(b"\r\n");
        self
    }

    fn add_text(mut self, name: &str, value: &str) -> Self {
        self.data
            .extend_from_slice(format!("--{}\r\n", self.boundary).as_bytes());
        self.data.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
        );
        self.data.extend_from_slice(value.as_bytes());
        self.data.extend_from_slice(b"\r\n");
        self
    }

    fn finish(mut self) -> (String, axum::body::Body) {
        self.data
            .extend_from_slice(format!("--{}--\r\n", self.boundary).as_bytes());
        let content_type = format!("multipart/form-data; boundary={}", self.boundary);
        (content_type, axum::body::Body::from(self.data))
    }
}

fn test_config() -> Config {
    Config {
        auth_enabled: false,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_app(&test_config());

    let req = axum::http::Request::builder()
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["app"], "podpdf");
}

#[tokio::test]
async fn test_serve_frontend() {
    let app = create_app(&test_config());

    let req = axum::http::Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("PodPdf"));
    assert!(html.contains("Merge PDFs"));
}

#[tokio::test]
async fn test_api_merge() {
    let app = create_app(&test_config());

    let doc1 = generate_test_pdf(1);
    let doc2 = generate_test_pdf(2);

    let (content_type, body) = MultipartBuilder::new()
        .add_file("files", "doc1.pdf", &doc1)
        .add_file("files", "doc2.pdf", &doc2)
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/merge")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 3);
}

#[tokio::test]
async fn test_api_rotate() {
    let app = create_app(&test_config());

    let doc = generate_test_pdf(2);
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .add_text("angle", "180")
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/rotate")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let parsed = podpdf::pdf::load_document(&bytes).unwrap();
    for (_, page_id) in parsed.get_pages() {
        let page_dict = parsed.get_object(page_id).unwrap().as_dict().unwrap();
        assert_eq!(page_dict.get(b"Rotate").unwrap().as_i64().unwrap(), 180);
    }
}

#[tokio::test]
async fn test_api_reverse() {
    let app = create_app(&test_config());
    let doc = generate_test_pdf(3);
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/reverse")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 3);
}

#[tokio::test]
async fn test_api_watermark() {
    let app = create_app(&test_config());
    let doc = generate_test_pdf(2);
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .add_text("text", "DRAFT")
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/watermark")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 2);
}

#[tokio::test]
async fn test_api_page_numbers() {
    let app = create_app(&test_config());
    let doc = generate_test_pdf(2);
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .add_text("position", "bottom-right")
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/page-numbers")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 2);
}

#[tokio::test]
async fn test_api_images_to_pdf() {
    let app = create_app(&test_config());
    let img = generate_test_png(100, 100);
    let (content_type, body) = MultipartBuilder::new()
        .add_image("files", "test.png", &img)
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/images-to-pdf")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 1);
}

#[tokio::test]
async fn test_api_text_to_pdf() {
    let app = create_app(&test_config());
    let (content_type, body) = MultipartBuilder::new()
        .add_text("text", "Hello PodPdf World!")
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/text-to-pdf")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 1);
}

#[tokio::test]
async fn test_api_compress() {
    let app = create_app(&test_config());
    let doc = generate_test_pdf(2);
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/compress")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&bytes).unwrap(), 2);
}

#[tokio::test]
async fn test_api_security_encrypt_decrypt() {
    let app = create_app(&test_config());
    let doc = generate_test_pdf(1);

    // 1. Encrypt
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .add_text("password", "safe123")
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/encrypt")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let enc_bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();

    // 2. Decrypt
    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &enc_bytes)
        .add_text("password", "safe123")
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/decrypt")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let dec_bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(page_count(&dec_bytes).unwrap(), 1);
}

#[tokio::test]
async fn test_api_compare() {
    let app = create_app(&test_config());
    let doc1 = generate_test_pdf(1);
    let doc2 = generate_test_pdf(2);

    let (content_type, body) = MultipartBuilder::new()
        .add_file("file1", "doc1.pdf", &doc1)
        .add_file("file2", "doc2.pdf", &doc2)
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/compare")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let comp: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(comp["page_count_doc1"], 1);
    assert_eq!(comp["page_count_doc2"], 2);
}

#[tokio::test]
async fn test_api_metadata() {
    let app = create_app(&test_config());
    let doc = generate_test_pdf_with_info("Api Title", "Api Author");

    let (content_type, body) = MultipartBuilder::new()
        .add_file("file", "doc.pdf", &doc)
        .finish();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/pdf/metadata/read")
        .header("Content-Type", content_type)
        .body(body)
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let meta: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(meta["title"], "Api Title");
}
