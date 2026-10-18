use crate::error::AppError;
use crate::pdf::merge::merge_documents;
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn merge_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut files = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Multipart(e.to_string()))?
    {
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Multipart(e.to_string()))?;

        if !data.is_empty() {
            files.push(data.to_vec());
        }
    }

    if files.is_empty() {
        return Err(AppError::BadRequest(
            "At least one PDF file must be provided".to_string(),
        ));
    }

    let merged_bytes = merge_documents(&files)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"merged.pdf\"",
        ),
    ];

    Ok((headers, merged_bytes))
}
