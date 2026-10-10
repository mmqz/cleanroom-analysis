//! PDF-a spike — 三探针（打开 / 渲染 / 标注写）
//!
//! 目的（pdf_carrier_plan.md §5 PDF-a 门禁）：为 ADR-P1 选型定谳提供实测数据。
//! - 探针 1 打开：pdfium（chromium/8086 non-V8 构建）load 3 页/500 页文档
//!   + 文本抽取，计时；
//! - 探针 2 渲染：pdfium 光栅化 1200px 宽位图，计时 + 尺寸断言；
//! - 探针 3 标注写：lopdf IncrementalDocument 增量写 Highlight 标注 →
//!   ① 原字节前缀保持断言（增量语义）② lopdf 重载标注在位 ③ pdfium
//!   交叉读回标注类型（引擎互通预证）。
//!
//! 运行：先 ./fetch-pdfium.sh，再 cargo run --release。

use std::fs;
use std::path::Path;
use std::time::Instant;

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, IncrementalDocument, Document, Object, Stream, StringFormat};

use pdf_a_spike::fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = here.join("fixtures");
    fs::create_dir_all(&fixtures_dir)?;

    // ---- fixtures：lopdf 现场生成（3 页 + 500 页，无外部素材依赖）----
    let small = fixtures_dir.join("small.pdf");
    let large = fixtures_dir.join("large.pdf");
    let t0 = Instant::now();
    fixtures::build_pdf(3, &small)?;
    fixtures::build_pdf(500, &large)?;
    let gen_ms = t0.elapsed().as_millis();
    let small_bytes = fs::metadata(&small)?.len();
    let large_bytes = fs::metadata(&large)?.len();
    println!("| fixture 生成（lopdf） | 3p={}B + 500p={}B | {} ms |", small_bytes, large_bytes, gen_ms);

    // ---- 绑定 pdfium（non-V8 构建；bin/ 由 fetch-pdfium.sh 取件）----
    use pdfium_render::prelude::*;
    let lib = here.join("bin/lib/libpdfium.so");
    let bindings = Pdfium::bind_to_library(&lib)?;
    let pdfium = Pdfium::new(bindings);

    // ================= 探针 1：打开 + 文本抽取 =================
    for (name, path) in [("3页", &small), ("500页", &large)] {
        let t = Instant::now();
        let doc = pdfium.load_pdf_from_file(path, None)?;
        let open_ms = t.elapsed().as_millis();
        let pages = doc.pages().len();
        assert_eq!(pages, if name == "3页" { 3 } else { 500 }, "页数");

        let t = Instant::now();
        let text = doc.pages().first()?.text()?.all();
        let text_ms = t.elapsed().as_millis();
        assert!(
            text.contains("Spike fixture page 1 of"),
            "文本抽取必须命中首页 fixture 文本（实测前 60 字：{}）",
            &text.chars().take(60).collect::<String>()
        );
        println!("| 探针1 打开（{}） | pages={} | {} ms（load）+ {} ms（首页文本抽取）|", name, pages, open_ms, text_ms);
    }

    // ================= 探针 2：光栅化渲染 =================
    {
        let doc = pdfium.load_pdf_from_file(&large, None)?;
        let page = doc.pages().first()?;
        let cfg = PdfRenderConfig::new().set_target_width(1200);
        let t = Instant::now();
        let bitmap = page.render_with_config(&cfg)?;
        let (w, h) = (bitmap.width(), bitmap.height());
        // 强制完成整页光栅化（as_image 逐像素转换）
        let img = bitmap.as_image()?;
        let render_ms = t.elapsed().as_millis();
        assert!(w >= 1190 && w <= 1210, "目标宽 1200 自适应：实测 {w}");
        assert!(h > 1400 && h < 1700, "Letter 纵横比位图高：实测 {h}");
        let _ = img; // 位图有效（保留变量供调试断点）
        println!("| 探针2 渲染（500页档首页） | {}x{} px | {} ms（光栅化+RGB 转换）|", w, h, render_ms);

        // 连续渲染 5 页观察稳定性（页缓存行为初判）
        let t = Instant::now();
        for i in 0..5i32 {
            let p = doc.pages().get(i)?;
            let b = p.render_with_config(&cfg)?;
            let _ = b.as_image()?;
        }
        println!("| 探针2 追加（连渲 5 页） | 5 页 | {} ms |", t.elapsed().as_millis());
    }

    // ================= 探针 3：lopdf 增量标注写 + 双引擎验证 =================
    {
        let original = fs::read(&small)?;

        // --- 3a. 增量写 Highlight 标注 ---
        let t = Instant::now();
        let mut inc = IncrementalDocument::load(&small)?;
        let page_id = *inc
            .get_prev_documents()
            .get_pages()
            .get(&1)
            .expect("page 1 object id");
        // 克隆页对象到增量修订（同 id 覆盖语义）
        inc.opt_clone_object_to_new_document(page_id)?;

        // 标注对象（PDF 32000 标准子型，零私有扩展——plan §7 互通约束）
        let annot_id = inc
            .new_document
            .add_object(dictionary! {
                "Type" => Object::Name(b"Annot".to_vec()),
                "Subtype" => Object::Name(b"Highlight".to_vec()),
                "Rect" => vec![Object::Integer(72), Object::Integer(692),
                                Object::Integer(330), Object::Integer(716)],
                "QuadPoints" => vec![Object::Integer(72), Object::Integer(712),
                                      Object::Integer(330), Object::Integer(712),
                                      Object::Integer(72), Object::Integer(696),
                                      Object::Integer(330), Object::Integer(696)],
                "Contents" => Object::String(b"mdc-pdf spike: highlight annotation".to_vec(), StringFormat::Literal),
                "T" => Object::String(b"mdc-pdf-spike".to_vec(), StringFormat::Literal),
                "C" => vec![Object::Real(1.0), Object::Real(1.0), Object::Real(0.0)],
                "F" => Object::Integer(4),
            });
        // 页对象挂 /Annots
        {
            let page = inc
                .new_document
                .get_object_mut(page_id)?
                .as_dict_mut()?;
            page.set("Annots", Object::Array(vec![Object::Reference(annot_id)]));
        }
        let out = fixtures_dir.join("small_annotated.pdf");
        inc.save(&out)?;
        let write_ms = t.elapsed().as_millis();

        // --- 3b. 增量语义断言：原字节前缀原样保持 ---
        let updated = fs::read(&out)?;
        assert!(
            updated.len() > original.len(),
            "增量更新必然追加（{} > {}）",
            updated.len(),
            original.len()
        );
        assert_eq!(
            &updated[..original.len()],
            &original[..],
            "增量语义：原字节必须原样作为前缀保持（签名文档友好性的基础）"
        );
        let delta = updated.len() - original.len();
        println!("| 探针3 标注写（lopdf 增量） | 原 {}B → {}B（+{}B）| {} ms |", original.len(), updated.len(), delta, write_ms);

        // --- 3c. lopdf 重载：标注在位 ---
        let reloaded = Document::load(&out)?;
        let page1 = reloaded.get_object(*reloaded.get_pages().get(&1).unwrap())?.as_dict()?;
        let annots = page1.get(b"Annots")?.as_array()?;
        assert_eq!(annots.len(), 1, "重载后 /Annots 恰 1 条");
        let annot_obj = reloaded.get_object(annots[0].as_reference()?)?.as_dict()?;
        assert_eq!(annot_obj.get(b"Subtype")?.as_name()?, b"Highlight");
        println!("| 探针3 lopdf 重载 | /Annots=1, Subtype=Highlight | OK |");

        // --- 3d. pdfium 交叉读回（引擎互通预证）---
        let doc = pdfium.load_pdf_from_file(&out, None)?;
        let page = doc.pages().first()?;
        let annotations = page.annotations();
        let n = annotations.len();
        assert_eq!(n, 1, "pdfium 必须读到 lopdf 写入的标注（互通性）");
        let first = annotations.first()?;
        let ty = first.annotation_type();
        println!(
            "| 探针3 pdfium 交叉读回 | annotations={} type={:?} | OK（lopdf 写 → pdfium 读互通）|",
            n, ty
        );
    }

    println!("\nPDF-a 三探针全部通过。");
    Ok(())
}

// 防未使用告警（fixtures 模块在 main 中经路径使用）
#[allow(unused_imports)]
use fixtures as _fixtures_ref;

// Content/Operation/Stream/dictionary 的再导出使用点（避免误删 import）
#[allow(dead_code)]
fn _api_surface_guard(c: Content, o: Operation, s: Stream, d: lopdf::Dictionary) -> (Content, Operation, Stream, lopdf::Dictionary) {
    let _ = (&c, &o, &s, &d);
    (c, o, s, d)
}
