pub mod compare;
pub mod convert;
pub mod merge;
pub mod metadata;
pub mod optimize;
pub mod pages;
pub mod rotate;
pub mod security;
pub mod split;
pub mod transform;
pub mod watermark;

use crate::error::PdfError;
use lopdf::Document;

/// Loads and validates a PDF document from an in-memory byte slice.
pub fn load_document(bytes: &[u8]) -> Result<Document, PdfError> {
    Document::load_mem(bytes).map_err(|err| PdfError::Parse(err.to_string()))
}

/// Serializes an in-memory PDF document to raw bytes.
pub fn save_document(doc: &mut Document) -> Result<Vec<u8>, PdfError> {
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)
        .map_err(|err| PdfError::Save(err.to_string()))?;
    Ok(buffer)
}

/// Returns the total page count for a PDF document.
pub fn page_count(bytes: &[u8]) -> Result<usize, PdfError> {
    let doc = load_document(bytes)?;
    Ok(doc.get_pages().len())
}
