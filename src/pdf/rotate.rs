use crate::error::PdfError;
use crate::pdf::{load_document, save_document};
use lopdf::Object;
use std::collections::HashSet;

/// Rotates specified pages (or all pages if None) by the given angle in degrees.
pub fn rotate_pages(
    pdf_bytes: &[u8],
    page_numbers: Option<&[u32]>,
    angle: i64,
) -> Result<Vec<u8>, PdfError> {
    if angle % 90 != 0 {
        return Err(PdfError::InvalidPage(
            "Rotation angle must be a multiple of 90 degrees".to_string(),
        ));
    }

    let mut doc = load_document(pdf_bytes)?;
    let pages = doc.get_pages();

    if pages.is_empty() {
        return Err(PdfError::EmptyResult);
    }

    let target_set: Option<HashSet<u32>> = page_numbers.map(|nums| nums.iter().copied().collect());

    for (page_num, page_id) in pages {
        if let Some(ref set) = target_set
            && !set.contains(&page_num)
        {
            continue;
        }

        let page_dict = doc
            .get_object_mut(page_id)
            .map_err(|err| PdfError::Parse(err.to_string()))?
            .as_dict_mut()
            .map_err(|err| PdfError::Parse(err.to_string()))?;

        let current_rotation = page_dict
            .get(b"Rotate")
            .and_then(|obj| obj.as_i64())
            .unwrap_or(0);

        let new_rotation = (current_rotation + angle).rem_euclid(360);
        page_dict.set("Rotate", Object::Integer(new_rotation));
    }

    save_document(&mut doc)
}
