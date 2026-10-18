use crate::error::AppError;
use crate::pdf::pages::{extract_pages, remove_pages, reorder_pages};
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

fn parse_page_numbers(spec: &str) -> Result<Vec<u32>, AppError> {
    spec.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u32>()
                .map_err(|_| AppError::BadRequest(format!("Invalid page number: {}", s)))
        })
        .collect()
}

pub async fn extract_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut pages_spec = None;

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
        } else if name == "pages" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            pages_spec = Some(text);
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let spec = pages_spec.ok_or_else(|| {
        AppError::BadRequest("Page specification is required in 'pages' field".to_string())
    })?;

    let pages = parse_page_numbers(&spec)?;
    let result_bytes = extract_pages(&pdf_bytes, &pages)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"extracted.pdf\"",
        ),
    ];

    Ok((headers, result_bytes))
}

pub async fn remove_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut pages_spec = None;

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
        } else if name == "pages" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            pages_spec = Some(text);
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let spec = pages_spec.ok_or_else(|| {
        AppError::BadRequest("Page numbers to remove required in 'pages' field".to_string())
    })?;

    let pages = parse_page_numbers(&spec)?;
    let result_bytes = remove_pages(&pdf_bytes, &pages)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"pages_removed.pdf\"",
        ),
    ];

    Ok((headers, result_bytes))
}

pub async fn reorder_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut order_spec = None;

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
        } else if name == "order" || name == "pages" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            order_spec = Some(text);
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let spec = order_spec.ok_or_else(|| {
        AppError::BadRequest("Page sequence is required in 'order' field".to_string())
    })?;

    let order = parse_page_numbers(&spec)?;
    let result_bytes = reorder_pages(&pdf_bytes, &order)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"reordered.pdf\"",
        ),
    ];

    Ok((headers, result_bytes))
}
