use crate::error::PdfError;
use crate::pdf::{load_document, save_document};
use lopdf::{Document, Object, ObjectId};
use std::collections::HashSet;

/// Deletes the specified pages (1-based indices) and returns the remaining document.
pub fn remove_pages(pdf_bytes: &[u8], page_numbers: &[u32]) -> Result<Vec<u8>, PdfError> {
    if page_numbers.is_empty() {
        return Err(PdfError::EmptyInput);
    }

    let mut doc = load_document(pdf_bytes)?;
    let total_pages = doc.get_pages().len();

    let unique_to_delete: HashSet<u32> = page_numbers.iter().copied().collect();
    for &page in &unique_to_delete {
        if page == 0 || page as usize > total_pages {
            return Err(PdfError::InvalidPage(format!(
                "Page {} is out of range 1..={}",
                page, total_pages
            )));
        }
    }

    if unique_to_delete.len() >= total_pages {
        return Err(PdfError::EmptyResult);
    }

    doc.delete_pages(page_numbers);
    doc.prune_objects();

    save_document(&mut doc)
}

/// Extracts only the specified pages (1-based indices) in the requested order.
pub fn extract_pages(pdf_bytes: &[u8], page_numbers: &[u32]) -> Result<Vec<u8>, PdfError> {
    if page_numbers.is_empty() {
        return Err(PdfError::EmptyInput);
    }

    reorder_or_extract_pages(pdf_bytes, page_numbers)
}

/// Reorders pages according to the specified 1-based index order.
pub fn reorder_pages(pdf_bytes: &[u8], order: &[u32]) -> Result<Vec<u8>, PdfError> {
    if order.is_empty() {
        return Err(PdfError::EmptyInput);
    }

    let doc = load_document(pdf_bytes)?;
    let total_pages = doc.get_pages().len();

    if order.len() != total_pages {
        return Err(PdfError::InvalidPage(format!(
            "Order length ({}) does not match total page count ({})",
            order.len(),
            total_pages
        )));
    }

    let unique: HashSet<u32> = order.iter().copied().collect();
    if unique.len() != total_pages {
        return Err(PdfError::InvalidPage(
            "Page order contains duplicates".to_string(),
        ));
    }

    reorder_or_extract_pages(pdf_bytes, order)
}

/// Constructs a new PDF containing only the specified pages in the exact sequence requested.
fn reorder_or_extract_pages(pdf_bytes: &[u8], page_indices: &[u32]) -> Result<Vec<u8>, PdfError> {
    let mut source_doc = load_document(pdf_bytes)?;
    let total_pages = source_doc.get_pages().len();

    for &idx in page_indices {
        if idx == 0 || idx as usize > total_pages {
            return Err(PdfError::InvalidPage(format!(
                "Page {} is out of range 1..={}",
                idx, total_pages
            )));
        }
    }

    let source_pages = source_doc.get_pages();
    let mut target_doc = Document::with_version("1.5");
    let mut max_id = 1;

    source_doc.renumber_objects_with(max_id);
    max_id = source_doc.max_id + 1;

    let mut selected_pages: Vec<(ObjectId, Object)> = Vec::new();
    let mut selected_ids = Vec::new();

    for &page_num in page_indices {
        let &page_id = source_pages.get(&page_num).ok_or_else(|| {
            PdfError::InvalidPage(format!("Page {} not found in document", page_num))
        })?;

        // In lopdf, page objects can be re-queried after renumbering
        let page_obj = source_doc
            .get_object(page_id)
            .map_err(|err| PdfError::Parse(err.to_string()))?
            .clone();

        selected_pages.push((page_id, page_obj));
        selected_ids.push(Object::Reference(page_id));
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in source_doc.objects.into_iter() {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                if catalog_object.is_none() {
                    catalog_object = Some((object_id, object));
                }
            }
            b"Pages" => {
                if pages_object.is_none() {
                    pages_object = Some((object_id, object));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                target_doc.objects.insert(object_id, object);
            }
        }
    }

    let (catalog_id, catalog_object) = catalog_object
        .ok_or_else(|| PdfError::Parse("Catalog root not found in document".to_string()))?;
    let (pages_id, pages_object) = pages_object
        .ok_or_else(|| PdfError::Parse("Pages root not found in document".to_string()))?;

    for (object_id, object) in selected_pages {
        if let Ok(dict) = object.as_dict() {
            let mut dict = dict.clone();
            dict.set("Parent", pages_id);
            target_doc
                .objects
                .insert(object_id, Object::Dictionary(dict));
        }
    }

    if let Ok(dict) = pages_object.as_dict() {
        let mut dict = dict.clone();
        dict.set("Count", selected_ids.len() as i64);
        dict.set("Kids", selected_ids);
        target_doc
            .objects
            .insert(pages_id, Object::Dictionary(dict));
    }

    if let Ok(dict) = catalog_object.as_dict() {
        let mut dict = dict.clone();
        dict.set("Pages", pages_id);
        dict.remove(b"Outlines");
        target_doc
            .objects
            .insert(catalog_id, Object::Dictionary(dict));
    }

    target_doc.trailer.set("Root", catalog_id);
    target_doc.max_id = max_id;
    target_doc.prune_objects();
    target_doc.renumber_objects();

    save_document(&mut target_doc)
}
