use crate::error::PdfError;
use crate::pdf::save_document;
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, dictionary};
use std::io::Cursor;

/// Converts a list of image byte buffers (PNG, JPEG, WebP, BMP) into a PDF.
pub fn images_to_pdf(images: &[Vec<u8>]) -> Result<Vec<u8>, PdfError> {
    if images.is_empty() {
        return Err(PdfError::EmptyInput);
    }

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_ids = Vec::new();

    for (idx, img_bytes) in images.iter().enumerate() {
        let dynamic_img = image::load_from_memory(img_bytes)
            .map_err(|e| PdfError::Parse(format!("Failed to decode image #{}: {}", idx + 1, e)))?;
        let rgb_img = dynamic_img.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        let image_stream_dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => width as i64,
            "Height" => height as i64,
            "ColorSpace" => "DeviceRGB",
            "BitsPerComponent" => 8,
        };

        let mut image_stream = Stream::new(image_stream_dict, rgb_img.into_raw());
        let _ = image_stream.compress();
        let image_id = doc.add_object(image_stream);

        let img_name = format!("Im{}", idx + 1);
        let resources_id = doc.add_object(dictionary! {
            "XObject" => dictionary! {
                img_name.as_str() => image_id,
            },
        });

        // Content stream draws the image scaled to the page MediaBox dimensions
        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        (width as f64).into(),
                        0.into(),
                        0.into(),
                        (height as f64).into(),
                        0.into(),
                        0.into(),
                    ],
                ),
                Operation::new("Do", vec![Object::Name(img_name.into_bytes())]),
                Operation::new("Q", vec![]),
            ],
        };

        let content_encoded = content
            .encode()
            .map_err(|e| PdfError::Save(e.to_string()))?;
        let content_id = doc.add_object(Stream::new(dictionary! {}, content_encoded));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), (width as f64).into(), (height as f64).into()],
        });

        page_ids.push(Object::Reference(page_id));
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids,
        "Count" => images.len() as i64,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    save_document(&mut doc)
}

/// Converts plain text into a paginated PDF document.
pub fn text_to_pdf(text: &str) -> Result<Vec<u8>, PdfError> {
    if text.trim().is_empty() {
        return Err(PdfError::EmptyInput);
    }

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });

    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if raw_line.len() <= 80 {
            lines.push(raw_line.to_string());
        } else {
            // Soft-wrap lines exceeding 80 characters
            let mut current = raw_line;
            while !current.is_empty() {
                let split_at = current
                    .char_indices()
                    .nth(80)
                    .map_or(current.len(), |(i, _)| i);
                lines.push(current[..split_at].to_string());
                current = &current[split_at..];
            }
        }
    }

    const LINES_PER_PAGE: usize = 50;
    let pages_chunks: Vec<&[String]> = lines.chunks(LINES_PER_PAGE).collect();
    let mut page_ids = Vec::new();

    for chunk in pages_chunks {
        let mut ops = vec![
            Operation::new("q", vec![]),
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 11.0.into()]),
            Operation::new("Td", vec![50.0.into(), 780.0.into()]),
            Operation::new("TL", vec![14.0.into()]),
        ];

        for line in chunk {
            ops.push(Operation::new(
                "Tj",
                vec![Object::string_literal(line.as_bytes())],
            ));
            ops.push(Operation::new("T*", vec![]));
        }

        ops.push(Operation::new("ET", vec![]));
        ops.push(Operation::new("Q", vec![]));

        let content = Content { operations: ops };
        let content_encoded = content
            .encode()
            .map_err(|e| PdfError::Save(e.to_string()))?;
        let content_id = doc.add_object(Stream::new(dictionary! {}, content_encoded));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.0.into(), 842.0.into()], // A4
        });

        page_ids.push(Object::Reference(page_id));
    }

    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids,
        "Count" => lines.chunks(LINES_PER_PAGE).len() as i64,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    save_document(&mut doc)
}

/// Extracts text content across all pages in the PDF document.
pub fn pdf_to_text(pdf_bytes: &[u8]) -> Result<String, PdfError> {
    let doc = crate::pdf::load_document(pdf_bytes)?;
    let pages = doc.get_pages();
    let mut full_text = String::new();

    for (page_num, _) in pages {
        if let Ok(text) = doc.extract_text(&[page_num]) {
            full_text.push_str(&format!("--- Page {} ---\n", page_num));
            full_text.push_str(&text);
            full_text.push('\n');
        }
    }

    Ok(full_text)
}

/// Extracts embedded raster images from the PDF document streams.
pub fn extract_images_from_pdf(pdf_bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, PdfError> {
    let doc = crate::pdf::load_document(pdf_bytes)?;
    let mut extracted = Vec::new();
    let mut image_counter = 1;

    for (_id, object) in doc.objects.iter() {
        if let Object::Stream(stream) = object {
            let dict = &stream.dict;
            let is_image = dict
                .get(b"Subtype")
                .ok()
                .and_then(|obj| obj.as_name().ok())
                .map(|name| name == b"Image")
                .unwrap_or(false);

            if !is_image {
                continue;
            }

            let filter = dict
                .get(b"Filter")
                .ok()
                .and_then(|obj| obj.as_name().ok())
                .unwrap_or(b"");

            if filter == b"DCTDecode" {
                // Direct JPEG stream
                extracted.push((
                    format!("image_{:03}.jpg", image_counter),
                    stream.content.clone(),
                ));
                image_counter += 1;
            } else if let Ok(decompressed) = stream.decompressed_content() {
                // Raw pixel data: check dimensions and color space
                let width = dict.get(b"Width").and_then(|o| o.as_i64()).unwrap_or(0) as u32;
                let height = dict.get(b"Height").and_then(|o| o.as_i64()).unwrap_or(0) as u32;

                if width > 0 && height > 0 {
                    let mut img_buf = Vec::new();
                    if decompressed.len() == (width * height * 3) as usize {
                        if let Some(rgb) = image::RgbImage::from_raw(width, height, decompressed) {
                            let _ = image::DynamicImage::ImageRgb8(rgb)
                                .write_to(&mut Cursor::new(&mut img_buf), image::ImageFormat::Png);
                        }
                    } else if decompressed.len() == (width * height * 4) as usize {
                        if let Some(rgba) = image::RgbaImage::from_raw(width, height, decompressed)
                        {
                            let _ = image::DynamicImage::ImageRgba8(rgba)
                                .write_to(&mut Cursor::new(&mut img_buf), image::ImageFormat::Png);
                        }
                    } else if decompressed.len() == (width * height) as usize
                        && let Some(gray) = image::GrayImage::from_raw(width, height, decompressed)
                    {
                        let _ = image::DynamicImage::ImageLuma8(gray)
                            .write_to(&mut Cursor::new(&mut img_buf), image::ImageFormat::Png);
                    }

                    if !img_buf.is_empty() {
                        extracted.push((format!("image_{:03}.png", image_counter), img_buf));
                        image_counter += 1;
                    }
                }
            }
        }
    }

    Ok(extracted)
}
