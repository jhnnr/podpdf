use crate::error::PdfError;
use crate::pdf::pages::extract_pages;
use crate::pdf::{load_document, page_count};
use std::io::Write;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Splits a PDF into single-page documents.
pub fn split_into_single_pages(pdf_bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, PdfError> {
    let total = page_count(pdf_bytes)?;
    if total == 0 {
        return Err(PdfError::EmptyResult);
    }

    let mut result = Vec::with_capacity(total);
    for page in 1..=total {
        let single_page_pdf = extract_pages(pdf_bytes, &[page as u32])?;
        result.push((format!("page_{}.pdf", page), single_page_pdf));
    }

    Ok(result)
}

/// Splits a PDF according to range specifications such as "1-2, 3, 4-5".
pub fn split_by_ranges(
    pdf_bytes: &[u8],
    range_spec: &str,
) -> Result<Vec<(String, Vec<u8>)>, PdfError> {
    let doc = load_document(pdf_bytes)?;
    let total_pages = doc.get_pages().len();

    let ranges = parse_ranges(range_spec, total_pages)?;
    if ranges.is_empty() {
        return Err(PdfError::InvalidPage(
            "No valid page ranges specified".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(ranges.len());
    for (i, page_group) in ranges.into_iter().enumerate() {
        let split_pdf = extract_pages(pdf_bytes, &page_group)?;
        result.push((format!("split_{}.pdf", i + 1), split_pdf));
    }

    Ok(result)
}

/// Parses comma-separated ranges (e.g. "1-3, 5, 7-9") into lists of 1-based page indices.
pub fn parse_ranges(spec: &str, total_pages: usize) -> Result<Vec<Vec<u32>>, PdfError> {
    let mut groups = Vec::new();

    for part in spec.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((start_s, end_s)) = trimmed.split_once('-') {
            let start: usize = start_s
                .trim()
                .parse()
                .map_err(|_| PdfError::InvalidPage(format!("Invalid range start: {}", start_s)))?;
            let end: usize = end_s
                .trim()
                .parse()
                .map_err(|_| PdfError::InvalidPage(format!("Invalid range end: {}", end_s)))?;

            if start == 0 || end == 0 || start > end || end > total_pages {
                return Err(PdfError::InvalidPage(format!(
                    "Invalid range {}-{} for document with {} pages",
                    start, end, total_pages
                )));
            }

            let pages: Vec<u32> = (start..=end).map(|p| p as u32).collect();
            groups.push(pages);
        } else {
            let single: usize = trimmed
                .parse()
                .map_err(|_| PdfError::InvalidPage(format!("Invalid page number: {}", trimmed)))?;
            if single == 0 || single > total_pages {
                return Err(PdfError::InvalidPage(format!(
                    "Page {} out of range 1..={}",
                    single, total_pages
                )));
            }
            groups.push(vec![single as u32]);
        }
    }

    Ok(groups)
}

/// Packages a set of named file buffers into a ZIP archive.
pub fn create_zip_archive(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, PdfError> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for (name, data) in files {
        zip.start_file(name, options)
            .map_err(|e| PdfError::Zip(e.to_string()))?;
        zip.write_all(data)
            .map_err(|e| PdfError::Zip(e.to_string()))?;
    }

    zip.finish().map_err(|e| PdfError::Zip(e.to_string()))?;
    Ok(buffer.into_inner())
}
