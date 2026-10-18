use crate::error::PdfError;
use crate::pdf::pages::reorder_pages;
use crate::pdf::{load_document, save_document};
use lopdf::Object;

#[derive(Debug, Clone, Copy)]
pub struct CropMargins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    A4,     // 595 x 842
    Letter, // 612 x 792
    A3,     // 842 x 1191
    Legal,  // 612 x 1008
    Custom(f64, f64),
}

impl PageSize {
    pub fn dimensions(&self) -> (f64, f64) {
        match *self {
            Self::A4 => (595.0, 842.0),
            Self::Letter => (612.0, 792.0),
            Self::A3 => (842.0, 1191.0),
            Self::Legal => (612.0, 1008.0),
            Self::Custom(w, h) => (w, h),
        }
    }
}

/// Inverts the page order of a PDF document (e.g. 5, 4, 3, 2, 1).
pub fn reverse_pages_order(pdf_bytes: &[u8]) -> Result<Vec<u8>, PdfError> {
    let doc = load_document(pdf_bytes)?;
    let count = doc.get_pages().len();
    if count <= 1 {
        return Ok(pdf_bytes.to_vec());
    }

    let reversed_order: Vec<u32> = (1..=count as u32).rev().collect();
    reorder_pages(pdf_bytes, &reversed_order)
}

/// Crops page margins by the specified offsets in points.
pub fn crop_pdf(pdf_bytes: &[u8], margins: &CropMargins) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;
    let pages = doc.get_pages();

    for (_, page_id) in pages {
        let page_obj = doc
            .get_object_mut(page_id)
            .map_err(|e| PdfError::Parse(e.to_string()))?;
        let dict = page_obj
            .as_dict_mut()
            .map_err(|e| PdfError::Parse(e.to_string()))?;

        let (x0, y0, x1, y1) = if let Ok(mb) = dict.get(b"MediaBox")
            && let Ok(arr) = mb.as_array()
            && arr.len() >= 4
        {
            let to_f = |obj: &Object| -> f64 {
                match obj {
                    Object::Real(f) => *f as f64,
                    Object::Integer(i) => *i as f64,
                    _ => 0.0,
                }
            };
            (to_f(&arr[0]), to_f(&arr[1]), to_f(&arr[2]), to_f(&arr[3]))
        } else {
            (0.0, 0.0, 595.0, 842.0)
        };

        let new_x0 = x0 + margins.left;
        let new_y0 = y0 + margins.bottom;
        let new_x1 = x1 - margins.right;
        let new_y1 = y1 - margins.top;

        if new_x0 >= new_x1 || new_y0 >= new_y1 {
            return Err(PdfError::InvalidPage(
                "Crop margins exceed page dimensions".to_string(),
            ));
        }

        let crop_box = vec![
            Object::Real(new_x0 as f32),
            Object::Real(new_y0 as f32),
            Object::Real(new_x1 as f32),
            Object::Real(new_y1 as f32),
        ];

        dict.set("CropBox", crop_box.clone());
        dict.set("MediaBox", crop_box);
    }

    save_document(&mut doc)
}

/// Scales/resizes the MediaBox of all pages to the target page dimensions.
pub fn scale_pages(pdf_bytes: &[u8], target_size: PageSize) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;
    let (target_w, target_h) = target_size.dimensions();

    let pages = doc.get_pages();
    for (_, page_id) in pages {
        let page_obj = doc
            .get_object_mut(page_id)
            .map_err(|e| PdfError::Parse(e.to_string()))?;
        let dict = page_obj
            .as_dict_mut()
            .map_err(|e| PdfError::Parse(e.to_string()))?;

        let media_box = vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(target_w as f32),
            Object::Real(target_h as f32),
        ];

        dict.set("MediaBox", media_box);
        dict.remove(b"CropBox");
    }

    save_document(&mut doc)
}
