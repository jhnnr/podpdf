use crate::error::PdfError;
use crate::pdf::{load_document, save_document};
use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeMap;

/// Merges multiple PDF documents into a single document preserving page sequence.
pub fn merge_documents(documents: &[Vec<u8>]) -> Result<Vec<u8>, PdfError> {
    if documents.is_empty() {
        return Err(PdfError::EmptyInput);
    }

    if documents.len() == 1 {
        let mut doc = load_document(&documents[0])?;
        return save_document(&mut doc);
    }

    let mut merged_doc = Document::with_version("1.5");
    let mut max_id = 1;

    let mut documents_pages: Vec<(ObjectId, Object)> = Vec::new();
    let mut documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();

    for pdf_data in documents {
        let mut doc = load_document(pdf_data)?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        let pages = doc.get_pages();
        for (_, page_id) in pages {
            if let Ok(page_obj) = doc.get_object(page_id) {
                documents_pages.push((page_id, page_obj.clone()));
            }
        }

        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects.into_iter() {
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
                merged_doc.objects.insert(object_id, object);
            }
        }
    }

    let (catalog_id, catalog_object) = catalog_object
        .ok_or_else(|| PdfError::Parse("Catalog root not found in source documents".to_string()))?;

    let (pages_id, pages_object) = pages_object
        .ok_or_else(|| PdfError::Parse("Pages root not found in source documents".to_string()))?;

    for (object_id, object) in documents_pages.iter() {
        if let Ok(dict) = object.as_dict() {
            let mut dict = dict.clone();
            dict.set("Parent", pages_id);
            merged_doc
                .objects
                .insert(*object_id, Object::Dictionary(dict));
        }
    }

    if let Ok(dict) = pages_object.as_dict() {
        let mut dict = dict.clone();
        dict.set("Count", documents_pages.len() as i64);
        dict.set(
            "Kids",
            documents_pages
                .iter()
                .map(|(id, _)| Object::Reference(*id))
                .collect::<Vec<_>>(),
        );
        merged_doc
            .objects
            .insert(pages_id, Object::Dictionary(dict));
    }

    if let Ok(dict) = catalog_object.as_dict() {
        let mut dict = dict.clone();
        dict.set("Pages", pages_id);
        dict.remove(b"Outlines");
        merged_doc
            .objects
            .insert(catalog_id, Object::Dictionary(dict));
    }

    merged_doc.trailer.set("Root", catalog_id);
    merged_doc.max_id = merged_doc.objects.len() as u32;
    merged_doc.renumber_objects();

    save_document(&mut merged_doc)
}
