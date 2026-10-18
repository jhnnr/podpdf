use crate::error::PdfError;
use crate::pdf::{load_document, save_document};
use lopdf::encryption::crypt_filters::{Aes128CryptFilter, CryptFilter};
use lopdf::{Document, EncryptionState, EncryptionVersion, Object, Permissions};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Encrypts the PDF document with AES-128 encryption using user and owner passwords.
pub fn encrypt_pdf(
    pdf_bytes: &[u8],
    user_password: &str,
    owner_password: &str,
) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;
    if doc.is_encrypted() {
        return Err(PdfError::Parse("Document is already encrypted".to_string()));
    }

    if !doc.trailer.has(b"ID") {
        let id_val = Object::String(
            b"1234567890abcdef".to_vec(),
            lopdf::StringFormat::Hexadecimal,
        );
        doc.trailer.set("ID", vec![id_val.clone(), id_val]);
    }

    let permissions = Permissions::PRINTABLE
        | Permissions::COPYABLE
        | Permissions::COPYABLE_FOR_ACCESSIBILITY
        | Permissions::PRINTABLE_IN_HIGH_QUALITY;

    let crypt_filter: Arc<dyn CryptFilter> = Arc::new(Aes128CryptFilter);

    let version = EncryptionVersion::V4 {
        document: &doc,
        encrypt_metadata: true,
        crypt_filters: BTreeMap::from([(b"StdCF".to_vec(), crypt_filter)]),
        stream_filter: b"StdCF".to_vec(),
        string_filter: b"StdCF".to_vec(),
        owner_password,
        user_password,
        permissions,
    };

    let state = EncryptionState::try_from(version)
        .map_err(|e| PdfError::Parse(format!("Encryption initialization error: {:?}", e)))?;

    doc.encrypt(&state)
        .map_err(|e| PdfError::Save(format!("Encryption failure: {:?}", e)))?;

    save_document(&mut doc)
}

/// Decrypts an encrypted PDF document using the supplied password.
pub fn decrypt_pdf(pdf_bytes: &[u8], password: &str) -> Result<Vec<u8>, PdfError> {
    let mut doc = match Document::load_mem_with_options(
        pdf_bytes,
        lopdf::LoadOptions::with_password(password),
    ) {
        Ok(d) => d,
        Err(_) => {
            let mut d = load_document(pdf_bytes)?;
            if d.is_encrypted() {
                d.decrypt(password)
                    .map_err(|e| PdfError::Parse(format!("Decryption failure: {:?}", e)))?;
            }
            d
        }
    };

    doc.encryption_state = None;
    doc.trailer.remove(b"Encrypt");

    save_document(&mut doc)
}

/// Flattens a PDF by removing interactive annotations, comments, and form fields.
pub fn flatten_pdf(pdf_bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;

    let pages = doc.get_pages();
    for (_, page_id) in pages {
        if let Ok(page_obj) = doc.get_object_mut(page_id)
            && let Ok(dict) = page_obj.as_dict_mut()
        {
            dict.remove(b"Annots");
        }
    }

    if let Ok(root_ref) = doc.trailer.get(b"Root")
        && let Ok(root_id) = root_ref.as_reference()
        && let Ok(root_obj) = doc.get_object_mut(root_id)
        && let Ok(dict) = root_obj.as_dict_mut()
    {
        dict.remove(b"AcroForm");
    }

    doc.prune_objects();
    save_document(&mut doc)
}

/// Redacts sensitive terms by replacing matched text occurrences in content streams with redaction placeholders.
pub fn redact_terms(pdf_bytes: &[u8], terms: &[String]) -> Result<Vec<u8>, PdfError> {
    if terms.is_empty() {
        return Ok(pdf_bytes.to_vec());
    }

    let mut doc = load_document(pdf_bytes)?;
    let terms_lower: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();

    for (_, object) in doc.objects.iter_mut() {
        if let Object::Stream(stream) = object
            && let Ok(mut content) = stream.decode_content()
        {
            let mut modified = false;
            for op in &mut content.operations {
                if (op.operator == "Tj" || op.operator == "'" || op.operator == "\"")
                    && let Some(Object::String(bytes, _)) = op.operands.first_mut()
                {
                    let text = String::from_utf8_lossy(bytes).to_string();
                    let mut text_redacted = text.clone();
                    for term in &terms_lower {
                        if text.to_lowercase().contains(term) {
                            text_redacted = "[REDACTED]".to_string();
                            modified = true;
                            break;
                        }
                    }
                    if modified {
                        *bytes = text_redacted.into_bytes();
                    }
                }
            }

            if modified && let Ok(encoded) = content.encode() {
                stream.set_content(encoded);
                let _ = stream.compress();
            }
        }
    }

    save_document(&mut doc)
}
