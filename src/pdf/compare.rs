use crate::error::PdfError;
use crate::pdf::convert::pdf_to_text;
use crate::pdf::metadata::{PdfMetadata, read_metadata};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfComparison {
    pub size_doc1_bytes: usize,
    pub size_doc2_bytes: usize,
    pub page_count_doc1: usize,
    pub page_count_doc2: usize,
    pub metadata_doc1: PdfMetadata,
    pub metadata_doc2: PdfMetadata,
    pub text_identical: bool,
    pub summary: String,
}

/// Compares two PDF documents across metadata, page counts, file sizes, and extracted textual content.
pub fn compare_pdfs(doc1_bytes: &[u8], doc2_bytes: &[u8]) -> Result<PdfComparison, PdfError> {
    let meta1 = read_metadata(doc1_bytes)?;
    let meta2 = read_metadata(doc2_bytes)?;

    let text1 = pdf_to_text(doc1_bytes)?;
    let text2 = pdf_to_text(doc2_bytes)?;

    let text_identical = text1.trim() == text2.trim();

    let mut differences = Vec::new();
    if meta1.page_count != meta2.page_count {
        differences.push(format!(
            "Page count mismatch: {} vs {}",
            meta1.page_count, meta2.page_count
        ));
    }
    if !text_identical {
        differences.push("Extracted text contents differ".to_string());
    }
    if doc1_bytes.len() != doc2_bytes.len() {
        differences.push(format!(
            "File size difference: {} bytes",
            (doc1_bytes.len() as isize - doc2_bytes.len() as isize).abs()
        ));
    }

    let summary = if differences.is_empty() {
        "Documents are identical in structure and text content.".to_string()
    } else {
        differences.join("; ")
    };

    Ok(PdfComparison {
        size_doc1_bytes: doc1_bytes.len(),
        size_doc2_bytes: doc2_bytes.len(),
        page_count_doc1: meta1.page_count,
        page_count_doc2: meta2.page_count,
        metadata_doc1: meta1,
        metadata_doc2: meta2,
        text_identical,
        summary,
    })
}
