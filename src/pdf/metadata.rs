use crate::error::PdfError;
use crate::pdf::{load_document, save_document};
use lopdf::{Document, Object, ObjectId, dictionary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub page_count: usize,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfMetadataUpdate {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
}

/// Reads metadata from a PDF document.
pub fn read_metadata(pdf_bytes: &[u8]) -> Result<PdfMetadata, PdfError> {
    let doc = load_document(pdf_bytes)?;
    let page_count = doc.get_pages().len();
    let version = doc.version.clone();

    let mut metadata = PdfMetadata {
        page_count,
        version,
        ..Default::default()
    };

    if let Ok(info_dict) = get_info_dictionary(&doc) {
        metadata.title = extract_string_field(&info_dict, b"Title");
        metadata.author = extract_string_field(&info_dict, b"Author");
        metadata.subject = extract_string_field(&info_dict, b"Subject");
        metadata.keywords = extract_string_field(&info_dict, b"Keywords");
        metadata.creator = extract_string_field(&info_dict, b"Creator");
        metadata.producer = extract_string_field(&info_dict, b"Producer");
        metadata.creation_date = extract_string_field(&info_dict, b"CreationDate");
        metadata.modification_date = extract_string_field(&info_dict, b"ModDate");
    }

    Ok(metadata)
}

/// Updates document metadata fields and returns the modified PDF document.
pub fn update_metadata(pdf_bytes: &[u8], update: &PdfMetadataUpdate) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;

    let info_id = ensure_info_dictionary(&mut doc)?;
    let info_obj = doc
        .get_object_mut(info_id)
        .map_err(|e| PdfError::Parse(e.to_string()))?;
    let dict = info_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Parse(e.to_string()))?;

    if let Some(ref val) = update.title {
        dict.set("Title", Object::string_literal(val.as_str()));
    }
    if let Some(ref val) = update.author {
        dict.set("Author", Object::string_literal(val.as_str()));
    }
    if let Some(ref val) = update.subject {
        dict.set("Subject", Object::string_literal(val.as_str()));
    }
    if let Some(ref val) = update.keywords {
        dict.set("Keywords", Object::string_literal(val.as_str()));
    }
    if let Some(ref val) = update.creator {
        dict.set("Creator", Object::string_literal(val.as_str()));
    }
    if let Some(ref val) = update.producer {
        dict.set("Producer", Object::string_literal(val.as_str()));
    }

    save_document(&mut doc)
}

/// Strips all metadata from the PDF document and returns the cleaned PDF bytes.
pub fn strip_metadata(pdf_bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;

    if let Ok(info_ref) = doc.trailer.get(b"Info")
        && let Ok(id) = info_ref.as_reference()
    {
        doc.objects.remove(&id);
    }
    doc.trailer.remove(b"Info");

    if let Ok(root_ref) = doc.trailer.get(b"Root")
        && let Ok(root_id) = root_ref.as_reference()
        && let Ok(root_obj) = doc.get_object_mut(root_id)
        && let Ok(dict) = root_obj.as_dict_mut()
    {
        dict.remove(b"Metadata");
    }

    doc.prune_objects();
    save_document(&mut doc)
}

fn get_info_dictionary(doc: &Document) -> Result<lopdf::Dictionary, PdfError> {
    let info_ref = doc
        .trailer
        .get(b"Info")
        .map_err(|_| PdfError::Parse("No Info entry in trailer".to_string()))?;

    match info_ref {
        Object::Reference(id) => {
            let obj = doc
                .get_object(*id)
                .map_err(|e| PdfError::Parse(e.to_string()))?;
            let dict = obj.as_dict().map_err(|e| PdfError::Parse(e.to_string()))?;
            Ok(dict.clone())
        }
        Object::Dictionary(dict) => Ok(dict.clone()),
        _ => Err(PdfError::Parse("Invalid Info object type".to_string())),
    }
}

fn ensure_info_dictionary(doc: &mut Document) -> Result<ObjectId, PdfError> {
    if let Ok(info_ref) = doc.trailer.get(b"Info")
        && let Ok(id) = info_ref.as_reference()
    {
        return Ok(id);
    }

    let info_id = doc.add_object(dictionary! {});
    doc.trailer.set("Info", info_id);
    Ok(info_id)
}

fn extract_string_field(dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
    dict.get(key).ok().and_then(|obj| match obj {
        Object::String(bytes, _) => {
            if bytes.starts_with(&[0xFE, 0xFF]) {
                // UTF-16BE encoding
                let u16_chars: Vec<u16> = bytes[2..]
                    .chunks_exact(2)
                    .map(|c| ((c[0] as u16) << 8) | (c[1] as u16))
                    .collect();
                String::from_utf16(&u16_chars).ok()
            } else {
                // PDFDocEncoding / ASCII fallback
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
        }
        _ => None,
    })
}
