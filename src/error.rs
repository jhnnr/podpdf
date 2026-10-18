use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("Failed to parse PDF document: {0}")]
    Parse(String),
    #[error("Failed to save PDF document: {0}")]
    Save(String),
    #[error("Invalid page number or range: {0}")]
    InvalidPage(String),
    #[error("No documents provided")]
    EmptyInput,
    #[error("Operation produced an empty document")]
    EmptyResult,
    #[error("ZIP packaging error: {0}")]
    Zip(String),
}

impl From<lopdf::Error> for PdfError {
    fn from(err: lopdf::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Pdf(#[from] PdfError),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Multipart error: {0}")]
    Multipart(String),
    #[error("Payload too large: maximum allowed size is {0} bytes")]
    PayloadTooLarge(usize),
    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponseBody {
    error: String,
    code: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Pdf(PdfError::InvalidPage(_)) => {
                (StatusCode::BAD_REQUEST, "INVALID_PAGE_SPECIFICATION")
            }
            Self::Pdf(PdfError::EmptyInput) => (StatusCode::BAD_REQUEST, "EMPTY_INPUT"),
            Self::Pdf(PdfError::Parse(_)) => (StatusCode::UNPROCESSABLE_ENTITY, "INVALID_PDF"),
            Self::Pdf(_) => (StatusCode::INTERNAL_SERVER_ERROR, "PDF_PROCESSING_ERROR"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Multipart(_) => (StatusCode::BAD_REQUEST, "MULTIPART_ERROR"),
            Self::PayloadTooLarge(_) => (StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = ErrorResponseBody {
            error: self.to_string(),
            code,
        };

        (status, Json(body)).into_response()
    }
}
