use crate::error::AppError;
use crate::pdf::metadata::{
    PdfMetadata, PdfMetadataUpdate, read_metadata, strip_metadata, update_metadata,
};
use axum::Json;
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn read_metadata_handler(
    mut multipart: Multipart,
) -> Result<Json<PdfMetadata>, AppError> {
    let mut file_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Multipart(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "pdf" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            file_data = Some(data.to_vec());
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let meta = read_metadata(&pdf_bytes)?;
    Ok(Json(meta))
}

pub async fn update_metadata_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut update = PdfMetadataUpdate::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Multipart(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "pdf" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            file_data = Some(data.to_vec());
        } else {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            match name.as_str() {
                "title" => update.title = Some(text),
                "author" => update.author = Some(text),
                "subject" => update.subject = Some(text),
                "keywords" => update.keywords = Some(text),
                "creator" => update.creator = Some(text),
                "producer" => update.producer = Some(text),
                _ => {}
            }
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let updated_bytes = update_metadata(&pdf_bytes, &update)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"updated_metadata.pdf\"",
        ),
    ];

    Ok((headers, updated_bytes))
}

pub async fn strip_metadata_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Multipart(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "pdf" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            file_data = Some(data.to_vec());
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let stripped_bytes = strip_metadata(&pdf_bytes)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"stripped_metadata.pdf\"",
        ),
    ];

    Ok((headers, stripped_bytes))
}
