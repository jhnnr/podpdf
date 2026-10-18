mod common;

use common::{generate_test_pdf, generate_test_pdf_with_info, generate_test_png};
use podpdf::pdf::compare::compare_pdfs;
use podpdf::pdf::convert::{images_to_pdf, pdf_to_text, text_to_pdf};
use podpdf::pdf::merge::merge_documents;
use podpdf::pdf::metadata::{PdfMetadataUpdate, read_metadata, strip_metadata, update_metadata};
use podpdf::pdf::optimize::compress_pdf;
use podpdf::pdf::pages::{extract_pages, remove_pages, reorder_pages};
use podpdf::pdf::rotate::rotate_pages;
use podpdf::pdf::security::{decrypt_pdf, encrypt_pdf, flatten_pdf, redact_terms};
use podpdf::pdf::split::{create_zip_archive, split_by_ranges, split_into_single_pages};
use podpdf::pdf::transform::{CropMargins, PageSize, crop_pdf, reverse_pages_order, scale_pages};
use podpdf::pdf::watermark::{
    PageNumberPosition, WatermarkOptions, add_page_numbers, add_watermark,
};
use podpdf::pdf::{load_document, page_count};

#[test]
fn test_merge_two_documents() {
    let doc1 = generate_test_pdf(2);
    let doc2 = generate_test_pdf(3);

    let merged = merge_documents(&[doc1, doc2]).expect("Merge failed");
    let count = page_count(&merged).expect("Failed to get page count");
    assert_eq!(count, 5);
}

#[test]
fn test_merge_empty_input() {
    let res = merge_documents(&[]);
    assert!(res.is_err());
}

#[test]
fn test_split_into_single_pages() {
    let doc = generate_test_pdf(3);
    let splits = split_into_single_pages(&doc).expect("Split failed");
    assert_eq!(splits.len(), 3);

    for (name, data) in splits {
        assert!(name.ends_with(".pdf"));
        let count = page_count(&data).expect("Failed to read split page");
        assert_eq!(count, 1);
    }
}

#[test]
fn test_split_by_ranges() {
    let doc = generate_test_pdf(5);
    let splits = split_by_ranges(&doc, "1-2, 3, 4-5").expect("Split by range failed");
    assert_eq!(splits.len(), 3);

    assert_eq!(page_count(&splits[0].1).unwrap(), 2);
    assert_eq!(page_count(&splits[1].1).unwrap(), 1);
    assert_eq!(page_count(&splits[2].1).unwrap(), 2);
}

#[test]
fn test_zip_archive() {
    let doc = generate_test_pdf(2);
    let splits = split_into_single_pages(&doc).unwrap();
    let zip_data = create_zip_archive(&splits).expect("ZIP creation failed");
    assert!(!zip_data.is_empty());
    assert_eq!(&zip_data[0..2], b"PK");
}

#[test]
fn test_rotate_all_pages() {
    let doc = generate_test_pdf(2);
    let rotated = rotate_pages(&doc, None, 90).expect("Rotate failed");

    let parsed = load_document(&rotated).expect("Failed to parse rotated PDF");
    for (_, page_id) in parsed.get_pages() {
        let page_dict = parsed.get_object(page_id).unwrap().as_dict().unwrap();
        let rot = page_dict.get(b"Rotate").unwrap().as_i64().unwrap();
        assert_eq!(rot, 90);
    }
}

#[test]
fn test_rotate_specific_page() {
    let doc = generate_test_pdf(2);
    let rotated = rotate_pages(&doc, Some(&[1]), 180).expect("Rotate failed");

    let parsed = load_document(&rotated).expect("Failed to parse rotated PDF");
    let pages = parsed.get_pages();

    let page1_dict = parsed.get_object(pages[&1]).unwrap().as_dict().unwrap();
    assert_eq!(page1_dict.get(b"Rotate").unwrap().as_i64().unwrap(), 180);

    let page2_dict = parsed.get_object(pages[&2]).unwrap().as_dict().unwrap();
    let p2_rot = page2_dict
        .get(b"Rotate")
        .and_then(|o| o.as_i64())
        .unwrap_or(0);
    assert_eq!(p2_rot, 0);
}

#[test]
fn test_extract_pages() {
    let doc = generate_test_pdf(4);
    let extracted = extract_pages(&doc, &[1, 3]).expect("Extract failed");
    assert_eq!(page_count(&extracted).unwrap(), 2);
}

#[test]
fn test_remove_pages() {
    let doc = generate_test_pdf(4);
    let remaining = remove_pages(&doc, &[2]).expect("Remove failed");
    assert_eq!(page_count(&remaining).unwrap(), 3);
}

#[test]
fn test_remove_all_pages_error() {
    let doc = generate_test_pdf(2);
    let res = remove_pages(&doc, &[1, 2]);
    assert!(res.is_err());
}

#[test]
fn test_reorder_pages() {
    let doc = generate_test_pdf(3);
    let reordered = reorder_pages(&doc, &[3, 1, 2]).expect("Reorder failed");
    assert_eq!(page_count(&reordered).unwrap(), 3);
}

#[test]
fn test_reverse_pages() {
    let doc = generate_test_pdf(3);
    let reversed = reverse_pages_order(&doc).expect("Reverse failed");
    assert_eq!(page_count(&reversed).unwrap(), 3);
}

#[test]
fn test_crop_pdf() {
    let doc = generate_test_pdf(1);
    let margins = CropMargins {
        top: 20.0,
        right: 15.0,
        bottom: 25.0,
        left: 10.0,
    };
    let cropped = crop_pdf(&doc, &margins).expect("Crop failed");
    assert_eq!(page_count(&cropped).unwrap(), 1);
}

#[test]
fn test_scale_pages() {
    let doc = generate_test_pdf(1);
    let scaled = scale_pages(&doc, PageSize::Letter).expect("Scale failed");
    assert_eq!(page_count(&scaled).unwrap(), 1);
}

#[test]
fn test_watermark_pdf() {
    let doc = generate_test_pdf(2);
    let opts = WatermarkOptions::default();
    let watermarked = add_watermark(&doc, &opts, None).expect("Watermark failed");
    assert_eq!(page_count(&watermarked).unwrap(), 2);
}

#[test]
fn test_page_numbers() {
    let doc = generate_test_pdf(2);
    let numbered = add_page_numbers(&doc, PageNumberPosition::BottomRight, "Page {n} of {total}")
        .expect("Add page numbers failed");
    assert_eq!(page_count(&numbered).unwrap(), 2);
}

#[test]
fn test_images_to_pdf() {
    let img1 = generate_test_png(100, 100);
    let img2 = generate_test_png(120, 80);
    let pdf = images_to_pdf(&[img1, img2]).expect("Images to PDF failed");
    assert_eq!(page_count(&pdf).unwrap(), 2);
}

#[test]
fn test_text_to_pdf_and_back() {
    let sample = "Line 1 of text document\nLine 2 with some information\nLine 3 concludes.";
    let pdf = text_to_pdf(sample).expect("Text to PDF failed");
    assert_eq!(page_count(&pdf).unwrap(), 1);

    let extracted_text = pdf_to_text(&pdf).expect("PDF to text failed");
    assert!(extracted_text.contains("Line 1"));
    assert!(extracted_text.contains("Line 2"));
}

#[test]
fn test_encrypt_and_decrypt() {
    let doc = generate_test_pdf(1);
    let encrypted = encrypt_pdf(&doc, "userPass123", "ownerPass456").expect("Encryption failed");

    let parsed_enc = load_document(&encrypted).unwrap();
    assert!(parsed_enc.is_encrypted());

    let decrypted = decrypt_pdf(&encrypted, "userPass123").expect("Decryption failed");
    let parsed_dec = load_document(&decrypted).unwrap();
    assert!(!parsed_dec.is_encrypted());
    assert_eq!(parsed_dec.get_pages().len(), 1);
}

#[test]
fn test_flatten_pdf() {
    let doc = generate_test_pdf(2);
    let flattened = flatten_pdf(&doc).expect("Flatten failed");
    assert_eq!(page_count(&flattened).unwrap(), 2);
}

#[test]
fn test_redact_terms() {
    let doc = text_to_pdf("The secret code is AlphaBravo123 in the vault").unwrap();
    let redacted = redact_terms(&doc, &["AlphaBravo123".to_string()]).expect("Redaction failed");
    let text = pdf_to_text(&redacted).unwrap();
    assert!(text.contains("[REDACTED]"));
    assert!(!text.contains("AlphaBravo123"));
}

#[test]
fn test_compress_pdf() {
    let doc = generate_test_pdf(3);
    let compressed = compress_pdf(&doc).expect("Compress failed");
    assert_eq!(page_count(&compressed).unwrap(), 3);
}

#[test]
fn test_compare_pdfs() {
    let doc1 = generate_test_pdf(2);
    let doc2 = generate_test_pdf(3);

    let comparison = compare_pdfs(&doc1, &doc2).expect("Compare failed");
    assert_eq!(comparison.page_count_doc1, 2);
    assert_eq!(comparison.page_count_doc2, 3);
    assert!(!comparison.text_identical);
}

#[test]
fn test_metadata_operations() {
    let doc = generate_test_pdf_with_info("Original Title", "Test Author");

    let meta = read_metadata(&doc).expect("Read metadata failed");
    assert_eq!(meta.title.as_deref(), Some("Original Title"));
    assert_eq!(meta.author.as_deref(), Some("Test Author"));

    let update = PdfMetadataUpdate {
        title: Some("Updated Title".to_string()),
        author: Some("New Author".to_string()),
        subject: Some("Security & Privacy".to_string()),
        keywords: Some("rust, pdf, privacy".to_string()),
        creator: Some("PodPdf Test".to_string()),
        producer: Some("PodPdf Engine".to_string()),
    };

    let updated_doc = update_metadata(&doc, &update).expect("Update metadata failed");
    let updated_meta = read_metadata(&updated_doc).expect("Read updated metadata failed");

    assert_eq!(updated_meta.title.as_deref(), Some("Updated Title"));
    assert_eq!(updated_meta.author.as_deref(), Some("New Author"));
    assert_eq!(updated_meta.subject.as_deref(), Some("Security & Privacy"));
    assert_eq!(updated_meta.keywords.as_deref(), Some("rust, pdf, privacy"));

    let stripped_doc = strip_metadata(&updated_doc).expect("Strip metadata failed");
    let stripped_meta = read_metadata(&stripped_doc).expect("Read stripped metadata failed");

    assert_eq!(stripped_meta.title, None);
    assert_eq!(stripped_meta.author, None);
}
