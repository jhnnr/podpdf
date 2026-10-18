use crate::error::AppError;
use crate::pdf::transform::{CropMargins, PageSize, crop_pdf, reverse_pages_order, scale_pages};
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn reverse_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
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

    let result = reverse_pages_order(&pdf_bytes)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"reversed.pdf\"",
        ),
    ];

    Ok((headers, result))
}

pub async fn crop_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut top = 0.0;
    let mut right = 0.0;
    let mut bottom = 0.0;
    let mut left = 0.0;

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
        } else if name == "top" {
            if let Ok(v) = field.text().await.unwrap_or_default().trim().parse() {
                top = v;
            }
        } else if name == "right" {
            if let Ok(v) = field.text().await.unwrap_or_default().trim().parse() {
                right = v;
            }
        } else if name == "bottom" {
            if let Ok(v) = field.text().await.unwrap_or_default().trim().parse() {
                bottom = v;
            }
        } else if name == "left"
            && let Ok(v) = field.text().await.unwrap_or_default().trim().parse()
        {
            left = v;
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let margins = CropMargins {
        top,
        right,
        bottom,
        left,
    };
    let result = crop_pdf(&pdf_bytes, &margins)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"cropped.pdf\"",
        ),
    ];

    Ok((headers, result))
}

pub async fn scale_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut target_size = PageSize::A4;

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
        } else if name == "size" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            target_size = match text.trim().to_lowercase().as_str() {
                "letter" => PageSize::Letter,
                "a3" => PageSize::A3,
                "legal" => PageSize::Legal,
                _ => PageSize::A4,
            };
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let result = scale_pages(&pdf_bytes, target_size)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"scaled.pdf\"",
        ),
    ];

    Ok((headers, result))
}
