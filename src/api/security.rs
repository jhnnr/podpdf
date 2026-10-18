use crate::error::AppError;
use crate::pdf::security::{decrypt_pdf, encrypt_pdf, flatten_pdf, redact_terms};
use axum::extract::Multipart;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn encrypt_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut password = None;
    let mut owner_password = None;

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
        } else if name == "password" || name == "user_password" {
            password = Some(
                field
                    .text()
                    .await
                    .map_err(|e| AppError::Multipart(e.to_string()))?,
            );
        } else if name == "owner_password" {
            owner_password = Some(
                field
                    .text()
                    .await
                    .map_err(|e| AppError::Multipart(e.to_string()))?,
            );
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let user_pwd = password.ok_or_else(|| {
        AppError::BadRequest("Password is required in 'password' field".to_string())
    })?;
    let owner_pwd = owner_password.unwrap_or_else(|| user_pwd.clone());

    let encrypted = encrypt_pdf(&pdf_bytes, &user_pwd, &owner_pwd)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"protected.pdf\"",
        ),
    ];

    Ok((headers, encrypted))
}

pub async fn decrypt_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut password = String::new();

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
        } else if name == "password" {
            password = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    let decrypted = decrypt_pdf(&pdf_bytes, &password)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"unlocked.pdf\"",
        ),
    ];

    Ok((headers, decrypted))
}

pub async fn flatten_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
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

    let flattened = flatten_pdf(&pdf_bytes)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"flattened.pdf\"",
        ),
    ];

    Ok((headers, flattened))
}

pub async fn redact_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut file_data = None;
    let mut terms = Vec::new();

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
        } else if name == "terms" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::Multipart(e.to_string()))?;
            terms = text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    let pdf_bytes = file_data.ok_or_else(|| {
        AppError::BadRequest("PDF file is required in 'file' form field".to_string())
    })?;

    if terms.is_empty() {
        return Err(AppError::BadRequest(
            "At least one term must be provided in 'terms' field".to_string(),
        ));
    }

    let redacted = redact_terms(&pdf_bytes, &terms)?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"redacted.pdf\"",
        ),
    ];

    Ok((headers, redacted))
}
