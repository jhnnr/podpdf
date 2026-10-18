use crate::error::AppError;
use crate::pdf::convert::{extract_images_from_pdf, images_to_pdf, pdf_to_text, text_to_pdf};
use crate::pdf::split::create_zip_archive;
use axum::Json;
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct TextExtractionResponse {
    pub text: String,
}

pub async fn images_to_pdf_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut images = Vec::new();

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
            images.push(data.to_vec());
        }
    }

    if images.is_empty() {
        return Err(AppError::BadRequest(
            "At least one image must be provided".to_string(),
        ));
    }

    let pdf_bytes = images_to_pdf(&images)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"converted_images.pdf\"",
        ),
    ];

    Ok((headers, pdf_bytes))
}

pub async fn text_to_pdf_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut text_content = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Multipart(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "text" || name == "file" {
            let t = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            if !t.trim().is_empty() {
                text_content = Some(t);
            }
        }
    }

    let text = text_content.ok_or_else(|| {
        AppError::BadRequest("Text content is required in 'text' field".to_string())
    })?;

    let pdf_bytes = text_to_pdf(&text)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"text_document.pdf\"",
        ),
    ];

    Ok((headers, pdf_bytes))
}

pub async fn pdf_to_text_handler(
    mut multipart: Multipart,
) -> Result<Json<TextExtractionResponse>, AppError> {
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

    let text = pdf_to_text(&pdf_bytes)?;
    Ok(Json(TextExtractionResponse { text }))
}

pub async fn extract_images_handler(
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

    let images = extract_images_from_pdf(&pdf_bytes)?;
    if images.is_empty() {
        return Err(AppError::BadRequest(
            "No extractable raster images found in the PDF".to_string(),
        ));
    }

    let zip_bytes = create_zip_archive(&images)?;

    let headers = [
        (header::CONTENT_TYPE, "application/zip"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"extracted_images.zip\"",
        ),
    ];

    Ok((headers, zip_bytes))
}
