//! fixture 生成：lopdf 现场构造标准 PDF（无外部素材依赖）。

use std::path::Path;

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream, StringFormat};

/// 生成 N 页 Letter 文档，每页一段可抽取文本。
pub fn build_pdf(pages: usize, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::with_version("1.7");

    let font_id = doc.add_object(dictionary! {
        "Type" => Object::Name(b"Font".to_vec()),
        "Subtype" => Object::Name(b"Type1".to_vec()),
        "BaseFont" => Object::Name(b"Helvetica".to_vec()),
    });

    let pages_id = doc.new_object_id();
    let mut kids = Vec::with_capacity(pages);
    for i in 0..pages {
        let text = format!("Spike fixture page {} of {}.", i + 1, pages);
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new(
                    "Tf",
                    vec![Object::Name(b"F1".to_vec()), Object::Integer(24)],
                ),
                Operation::new("Td", vec![Object::Integer(72), Object::Integer(720)]),
                Operation::new(
                    "Tj",
                    vec![Object::String(text.into_bytes(), StringFormat::Literal)],
                ),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode()?));
        let page = dictionary! {
            "Type" => Object::Name(b"Page".to_vec()),
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![
                Object::Integer(0), Object::Integer(0),
                Object::Integer(612), Object::Integer(792),
            ],
            "Resources" => dictionary! {
                "Font" => dictionary!{ "F1" => Object::Reference(font_id) },
            },
            "Contents" => Object::Reference(content_id),
        };
        kids.push(Object::Reference(doc.add_object(page)));
    }

    doc.set_object(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => Object::Name(b"Pages".to_vec()),
            "Kids" => Object::Array(kids),
            "Count" => Object::Integer(pages as i64),
        }),
    );

    let catalog_id = doc.add_object(dictionary! {
        "Type" => Object::Name(b"Catalog".to_vec()),
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    doc.save(path)?;
    Ok(())
}

/// S6 探针 fixture：1 页 + /AcroForm（/Tx 文本域 widget）——行为级
/// formfill 能力探测的最小载体（PDF-b 登记、PDF-c 轮补做的欠账）。
pub fn build_form_pdf(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::with_version("1.7");

    let font_id = doc.add_object(dictionary! {
        "Type" => Object::Name(b"Font".to_vec()),
        "Subtype" => Object::Name(b"Type1".to_vec()),
        "BaseFont" => Object::Name(b"Helvetica".to_vec()),
    });

    let pages_id = doc.new_object_id();
    let content_id = doc.add_object(Stream::new(
        dictionary! {},
        Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Integer(24)]),
                Operation::new("Td", vec![Object::Integer(72), Object::Integer(720)]),
                Operation::new(
                    "Tj",
                    vec![Object::String(b"AcroForm probe.".to_vec(), StringFormat::Literal)],
                ),
                Operation::new("ET", vec![]),
            ],
        }
        .encode()?,
    ));

    // 文本域 widget（/FT /Tx；/DA 默认外观；/Rect 在页内）。
    let field_id = doc.add_object(dictionary! {
        "Type" => Object::Name(b"Annot".to_vec()),
        "Subtype" => Object::Name(b"Widget".to_vec()),
        "FT" => Object::Name(b"Tx".to_vec()),
        "T" => Object::String(b"probe_text".to_vec(), StringFormat::Literal),
        "V" => Object::String(b"hello form".to_vec(), StringFormat::Literal),
        "DA" => Object::String(b"/F1 12 Tf 0 g".to_vec(), StringFormat::Literal),
        "Rect" => vec![
            Object::Integer(72), Object::Integer(600),
            Object::Integer(300), Object::Integer(630),
        ],
        "P" => Object::Reference(pages_id),
    });

    let page_id = doc.add_object(dictionary! {
        "Type" => Object::Name(b"Page".to_vec()),
        "Parent" => Object::Reference(pages_id),
        "MediaBox" => vec![
            Object::Integer(0), Object::Integer(0),
            Object::Integer(612), Object::Integer(792),
        ],
        "Resources" => dictionary! {
            "Font" => dictionary!{ "F1" => Object::Reference(font_id) },
        },
        "Contents" => Object::Reference(content_id),
        "Annots" => vec![Object::Reference(field_id)],
    });

    doc.set_object(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => Object::Name(b"Pages".to_vec()),
            "Kids" => vec![Object::Reference(page_id)],
            "Count" => Object::Integer(1),
        }),
    );

    let catalog_id = doc.add_object(dictionary! {
        "Type" => Object::Name(b"Catalog".to_vec()),
        "Pages" => Object::Reference(pages_id),
        "AcroForm" => dictionary! {
            "Fields" => vec![Object::Reference(field_id)],
            "DA" => Object::String(b"/F1 12 Tf 0 g".to_vec(), StringFormat::Literal),
        },
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    doc.save(path)?;
    Ok(())
}
