use crate::error::AppError;
use crate::pdf::compare::{PdfComparison, compare_pdfs};
use axum::Json;
use axum::extract::Multipart;

pub async fn compare_handler(mut multipart: Multipart) -> Result<Json<PdfComparison>, AppError> {
    let mut doc1 = None;
    let mut doc2 = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Multipart(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Multipart(e.to_string()))?;

        if !data.is_empty() {
            if name == "file1" || doc1.is_none() {
                doc1 = Some(data.to_vec());
            } else {
                doc2 = Some(data.to_vec());
            }
        }
    }

    let b1 =
        doc1.ok_or_else(|| AppError::BadRequest("First PDF file is required (file1)".to_string()))?;
    let b2 = doc2
        .ok_or_else(|| AppError::BadRequest("Second PDF file is required (file2)".to_string()))?;

    let comparison = compare_pdfs(&b1, &b2)?;
    Ok(Json(comparison))
}
