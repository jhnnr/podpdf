use crate::error::AppError;
use crate::pdf::split::{create_zip_archive, split_by_ranges, split_into_single_pages};
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn split_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut ranges = None;

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
        } else if name == "ranges" || name == "pages" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            ranges = Some(text);
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let files = match ranges {
        Some(ref r) if !r.trim().is_empty() => split_by_ranges(&pdf_bytes, r.trim())?,
        _ => split_into_single_pages(&pdf_bytes)?,
    };

    if files.len() == 1 {
        let (filename, content) = &files[0];
        let headers = [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ];
        return Ok((headers, content.clone()));
    }

    let zip_bytes = create_zip_archive(&files)?;
    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"split_pages.zip\"".to_string(),
        ),
    ];

    Ok((headers, zip_bytes))
}
