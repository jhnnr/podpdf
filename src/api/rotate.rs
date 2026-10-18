use crate::error::AppError;
use crate::pdf::rotate::rotate_pages;
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn rotate_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut angle: i64 = 90;
    let mut pages = None;

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
        } else if name == "angle" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            if let Ok(parsed_angle) = text.trim().parse() {
                angle = parsed_angle;
            }
        } else if name == "pages" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            pages = Some(text);
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let parsed_pages: Option<Vec<u32>> = if let Some(ref p_str) = pages {
        if p_str.trim().is_empty() {
            None
        } else {
            let nums = p_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.parse::<u32>()
                        .map_err(|_| AppError::BadRequest(format!("Invalid page number '{}'", s)))
                })
                .collect::<Result<Vec<u32>, _>>()?;
            Some(nums)
        }
    } else {
        None
    };

    let rotated_bytes = rotate_pages(&pdf_bytes, parsed_pages.as_deref(), angle)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"rotated.pdf\"",
        ),
    ];

    Ok((headers, rotated_bytes))
}
