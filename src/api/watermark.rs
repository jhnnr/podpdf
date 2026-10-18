use crate::error::AppError;
use crate::pdf::watermark::{
    PageNumberPosition, WatermarkOptions, add_page_numbers, add_watermark,
};
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn watermark_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut text = "CONFIDENTIAL".to_string();
    let mut font_size = 48.0;
    let mut angle_deg = 45.0;
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
        } else if name == "text" {
            let t = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            if !t.trim().is_empty() {
                text = t;
            }
        } else if name == "font_size" {
            if let Ok(val) = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?
                .trim()
                .parse::<f64>()
            {
                font_size = val;
            }
        } else if name == "angle" {
            if let Ok(val) = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?
                .trim()
                .parse::<f64>()
            {
                angle_deg = val;
            }
        } else if name == "pages" {
            let p = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            if !p.trim().is_empty() {
                pages_spec = Some(p);
            }
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let parsed_pages: Option<Vec<u32>> = if let Some(ref spec) = pages_spec {
        let nums = spec
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|_| AppError::BadRequest(format!("Invalid page number '{}'", s)))
            })
            .collect::<Result<Vec<u32>, _>>()?;
        Some(nums)
    } else {
        None
    };

    let options = WatermarkOptions {
        text,
        font_size,
        opacity_rgb: [0.75, 0.75, 0.75],
        angle_deg,
    };

    let result = add_watermark(&pdf_bytes, &options, parsed_pages.as_deref())?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"watermarked.pdf\"",
        ),
    ];

    Ok((headers, result))
}

pub async fn page_numbers_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut position = PageNumberPosition::BottomCenter;
    let mut format_template = "Page {n} of {total}".to_string();

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
        } else if name == "position" {
            let pos_str = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            position = match pos_str.trim().to_lowercase().as_str() {
                "bottom-right" | "bottomright" => PageNumberPosition::BottomRight,
                "top-right" | "topright" => PageNumberPosition::TopRight,
                "top-center" | "topcenter" => PageNumberPosition::TopCenter,
                _ => PageNumberPosition::BottomCenter,
            };
        } else if name == "format" {
            let fmt_str = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            if !fmt_str.trim().is_empty() {
                format_template = fmt_str;
            }
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let result = add_page_numbers(&pdf_bytes, position, &format_template)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"numbered.pdf\"",
        ),
    ];

    Ok((headers, result))
}
