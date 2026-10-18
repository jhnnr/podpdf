use crate::error::PdfError;
use crate::pdf::load_document;
use lopdf::SaveOptions;

/// Compresses and optimizes a PDF document by recompressing streams, pruning dead objects, and using object streams.
pub fn compress_pdf(pdf_bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;

    // Compress all streams
    doc.compress();

    // Remove orphaned objects
    doc.prune_objects();

    let mut output = Vec::new();
    let options = SaveOptions::builder()
        .use_object_streams(true)
        .use_xref_streams(true)
        .max_objects_per_stream(200)
        .compression_level(9)
        .build();

    doc.save_with_options(&mut output, options)
        .map_err(|e| PdfError::Save(e.to_string()))?;

    Ok(output)
}
