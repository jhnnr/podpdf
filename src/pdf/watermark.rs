use crate::error::PdfError;
use crate::pdf::{load_document, save_document};
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, ObjectId, Stream, dictionary};

#[derive(Debug, Clone)]
pub struct WatermarkOptions {
    pub text: String,
    pub font_size: f64,
    pub opacity_rgb: [f64; 3], // [r, g, b] between 0.0 and 1.0
    pub angle_deg: f64,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            text: "CONFIDENTIAL".to_string(),
            font_size: 48.0,
            opacity_rgb: [0.75, 0.75, 0.75],
            angle_deg: 45.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageNumberPosition {
    #[default]
    BottomCenter,
    BottomRight,
    TopRight,
    TopCenter,
}

/// Stamps a text watermark onto all or selected pages of the PDF.
pub fn add_watermark(
    pdf_bytes: &[u8],
    options: &WatermarkOptions,
    pages_filter: Option<&[u32]>,
) -> Result<Vec<u8>, PdfError> {
    if options.text.trim().is_empty() {
        return Err(PdfError::EmptyInput);
    }

    let mut doc = load_document(pdf_bytes)?;
    let font_id = ensure_helvetica_font(&mut doc)?;

    let pages = doc.get_pages();
    let filter_set: Option<std::collections::HashSet<u32>> =
        pages_filter.map(|p| p.iter().copied().collect());

    let rad = options.angle_deg.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();

    for (page_num, page_id) in pages {
        if let Some(ref set) = filter_set
            && !set.contains(&page_num)
        {
            continue;
        }

        let media_box = get_page_media_box(&doc, page_id)?;
        let center_x = (media_box.0 + media_box.2) / 2.0;
        let center_y = (media_box.1 + media_box.3) / 2.0;

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F_WM".into(), options.font_size.into()]),
                Operation::new(
                    "rg",
                    vec![
                        options.opacity_rgb[0].into(),
                        options.opacity_rgb[1].into(),
                        options.opacity_rgb[2].into(),
                    ],
                ),
                Operation::new(
                    "Tm",
                    vec![
                        cos.into(),
                        sin.into(),
                        (-sin).into(),
                        cos.into(),
                        (center_x - (options.text.len() as f64 * options.font_size * 0.25)).into(),
                        center_y.into(),
                    ],
                ),
                Operation::new("Tj", vec![Object::string_literal(options.text.as_bytes())]),
                Operation::new("ET", vec![]),
                Operation::new("Q", vec![]),
            ],
        };

        append_page_content(&mut doc, page_id, font_id, content)?;
    }

    save_document(&mut doc)
}

/// Adds page numbers to pages with customizable formatting and position.
pub fn add_page_numbers(
    pdf_bytes: &[u8],
    position: PageNumberPosition,
    format_template: &str, // e.g. "Page {n} of {total}" or "{n}"
) -> Result<Vec<u8>, PdfError> {
    let mut doc = load_document(pdf_bytes)?;
    let font_id = ensure_helvetica_font(&mut doc)?;

    let pages = doc.get_pages();
    let total_pages = pages.len();

    for (page_num, page_id) in pages {
        let text = format_template
            .replace("{n}", &page_num.to_string())
            .replace("{total}", &total_pages.to_string());

        let media_box = get_page_media_box(&doc, page_id)?;
        let (x, y) = match position {
            PageNumberPosition::BottomCenter => (
                (media_box.0 + media_box.2) / 2.0 - (text.len() as f64 * 3.0),
                media_box.1 + 25.0,
            ),
            PageNumberPosition::BottomRight => (
                media_box.2 - (text.len() as f64 * 7.0) - 30.0,
                media_box.1 + 25.0,
            ),
            PageNumberPosition::TopRight => (
                media_box.2 - (text.len() as f64 * 7.0) - 30.0,
                media_box.3 - 35.0,
            ),
            PageNumberPosition::TopCenter => (
                (media_box.0 + media_box.2) / 2.0 - (text.len() as f64 * 3.0),
                media_box.3 - 35.0,
            ),
        };

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F_WM".into(), 10.0.into()]),
                Operation::new("rg", vec![0.2.into(), 0.2.into(), 0.2.into()]),
                Operation::new("Td", vec![x.into(), y.into()]),
                Operation::new("Tj", vec![Object::string_literal(text.as_bytes())]),
                Operation::new("ET", vec![]),
                Operation::new("Q", vec![]),
            ],
        };

        append_page_content(&mut doc, page_id, font_id, content)?;
    }

    save_document(&mut doc)
}

fn ensure_helvetica_font(doc: &mut Document) -> Result<ObjectId, PdfError> {
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
    });
    Ok(font_id)
}

fn get_page_media_box(doc: &Document, page_id: ObjectId) -> Result<(f64, f64, f64, f64), PdfError> {
    let page_obj = doc
        .get_object(page_id)
        .map_err(|e| PdfError::Parse(e.to_string()))?;
    let dict = page_obj
        .as_dict()
        .map_err(|e| PdfError::Parse(e.to_string()))?;

    if let Ok(mb) = dict.get(b"MediaBox")
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
        return Ok((to_f(&arr[0]), to_f(&arr[1]), to_f(&arr[2]), to_f(&arr[3])));
    }

    // Default A4 MediaBox in points
    Ok((0.0, 0.0, 595.0, 842.0))
}

fn append_page_content(
    doc: &mut Document,
    page_id: ObjectId,
    font_id: ObjectId,
    content: Content,
) -> Result<(), PdfError> {
    let encoded = content
        .encode()
        .map_err(|e| PdfError::Save(e.to_string()))?;
    let stream_id = doc.add_object(Stream::new(dictionary! {}, encoded));

    let page_obj = doc
        .get_object_mut(page_id)
        .map_err(|e| PdfError::Parse(e.to_string()))?;
    let page_dict = page_obj
        .as_dict_mut()
        .map_err(|e| PdfError::Parse(e.to_string()))?;

    // Ensure Resources dictionary exists and registers F_WM font
    if !page_dict.has(b"Resources") {
        page_dict.set("Resources", dictionary! {});
    }

    if let Ok(resources_obj) = page_dict.get_mut(b"Resources")
        && let Ok(resources_dict) = resources_obj.as_dict_mut()
    {
        if !resources_dict.has(b"Font") {
            resources_dict.set("Font", dictionary! {});
        }
        if let Ok(font_dict_obj) = resources_dict.get_mut(b"Font")
            && let Ok(font_dict) = font_dict_obj.as_dict_mut()
        {
            font_dict.set("F_WM", font_id);
        }
    }

    // Append stream to Contents
    match page_dict.get_mut(b"Contents") {
        Ok(Object::Array(arr)) => {
            arr.push(Object::Reference(stream_id));
        }
        Ok(Object::Reference(existing_id)) => {
            let existing = *existing_id;
            page_dict.set(
                "Contents",
                vec![Object::Reference(existing), Object::Reference(stream_id)],
            );
        }
        _ => {
            page_dict.set("Contents", Object::Reference(stream_id));
        }
    }

    Ok(())
}
