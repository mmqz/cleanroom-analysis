//! PDF-a spike S6 探针——formfill 能力面定谳复测（2026-10-11 复原重建）。
//!
//! 缺口缘由：S6 定谳登记（SECURITY_POLICY.md S6 行、提交 010b519）把
//! `src/bin/formfill_probe.rs` 列为证据，但该文件从未随提交入树（前一
//! 环境产物缺失，历史登记与仓不符）——F12 可复现性缺口的实证。本文件
//! 按登记语义重建，两段：
//!
//! - 段 a 生成：lopdf 现场构造 fixtures/form_probe.pdf（复用 fixtures.rs
//!   `build_form_pdf` 构造惯例）——1 页 + catalog /AcroForm（/Fields 一个
//!   /Tx 文本域）+ 页 /Annots 直挂该域 /Widget。这是 mdcore 15 号锁
//!   （`mount_preserves_existing_page_annots`）的前置：页 /Annots 已有
//!   /Widget，挂载追加不得破坏它。
//! - 段 b 探针（S6 定谳复测）：pdfium 打开该 fixture——form() = Some、
//!   form_type = Acrobat、field_values 枚举 /Tx 字段名值可读；对照组
//!   （无表单的 small.pdf）form() = None。
//!
//! 安全语义（S6 处置纪律）：表单字段是**被动可读面**而非执行面——本探针
//! 只读 field_values；交互路径不存在（pdfium-render 绑定层的
//! FPDF_FORMFILLINFO 全部回调置 None、m_pJsPlatform 空、XFA 禁用——
//! 环境初始化仅为使字段数据可读，无任何交互回调可被文档触发）。
//!
//! 运行：先 ./fetch-pdfium.sh，再 cargo run --bin formfill_probe --release。

use std::fs;
use std::path::Path;
use std::time::Instant;

use pdfium_render::prelude::*;

use pdf_a_spike::fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = here.join("fixtures");
    fs::create_dir_all(&fixtures_dir)?;

    // ================= 段 a：生成 form_probe.pdf（lopdf 现场构造） =================
    let form_probe = fixtures_dir.join("form_probe.pdf");
    let t = Instant::now();
    fixtures::build_form_pdf(&form_probe)?;
    let gen_ms = t.elapsed().as_millis();
    let probe_bytes = fs::metadata(&form_probe)?.len();
    println!(
        "| fixture 生成（lopdf /AcroForm） | form_probe.pdf 1p={}B | {} ms |",
        probe_bytes, gen_ms
    );

    // 对照组基准：small.pdf（无表单 3 页档）——缺失则按同一构造惯例补生成
    // （净克隆可复现：探针不依赖 main.rs 先跑过）。
    let small = fixtures_dir.join("small.pdf");
    if !small.exists() {
        fixtures::build_pdf(3, &small)?;
    }

    // ---- fixture 形状自检（lopdf 重载；mdcore 15 号锁的前置形状逐项在位）----
    {
        let doc = lopdf::Document::load(&form_probe)?;
        assert_eq!(doc.get_pages().len(), 1, "form_probe 恰 1 页");
        let page_id = *doc.get_pages().get(&1).expect("页 1 对象号");
        let page = doc.get_object(page_id)?.as_dict()?;
        let annots = page.get(b"Annots")?.as_array()?;
        assert_eq!(annots.len(), 1, "页 /Annots 直挂数组恰 1 条");
        let widget = doc.get_object(annots[0].as_reference()?)?.as_dict()?;
        assert_eq!(
            widget.get(b"Subtype")?.as_name()?,
            b"Widget",
            "注释子型 /Widget"
        );
        assert_eq!(widget.get(b"FT")?.as_name()?, b"Tx", "域类型 /Tx（文本域）");
        let acro = doc.catalog()?.get(b"AcroForm")?.as_dict()?;
        let fields = acro.get(b"Fields")?.as_array()?;
        assert_eq!(fields.len(), 1, "catalog /AcroForm /Fields 恰 1 域");
        assert_eq!(
            fields[0].as_reference()?,
            annots[0].as_reference()?,
            "/Fields 与页 /Annots 指向同一 widget 对象（单引用挂载惯例）"
        );
        println!(
            "| fixture 形状自检（lopdf 重载） | /AcroForm/Fields=1 + 页 /Annots=[Widget /FT /Tx] 直挂 | OK |"
        );
    }

    // ---- 绑定 pdfium（non-V8 构建；bin/ 由 fetch-pdfium.sh 取件）----
    let lib = here.join("bin/lib/libpdfium.so");
    let bindings = Pdfium::bind_to_library(&lib)?;
    let pdfium = Pdfium::new(bindings);

    // ================= 段 b：S6 探针（formfill 能力面定谳复测） =================
    {
        // --- b1. form_probe.pdf：表单在位、类型 Acrobat、/Tx 域名值可读 ---
        let t = Instant::now();
        let doc = pdfium.load_pdf_from_file(&form_probe, None)?;
        let open_ms = t.elapsed().as_millis();

        let form = doc.form().expect(
            "S6 定谳断言：带 /AcroForm 文档 form() 必须 Some（formfill 模块编入 non-V8 构建）",
        );
        let ty = form.form_type();
        assert_eq!(
            ty,
            PdfFormType::Acrobat,
            "S6 定谳断言：form_type 必须 Acrobat（非 XFA——S1.2）"
        );

        let t = Instant::now();
        let values = form.field_values(doc.pages());
        let enum_ms = t.elapsed().as_millis();
        assert_eq!(values.len(), 1, "field_values 恰枚举 1 个域");
        let val = values
            .get("probe_text")
            .expect("/Tx 域名 probe_text 可枚举");
        assert_eq!(
            val.as_deref(),
            Some("hello form"),
            "/Tx 域值可读（写入什么读出什么）"
        );

        // pdfium 视角的 widget 本体：页注释恰 1 条、类型 /Widget——15 号锁
        // 挂载前形态（挂载追加后应为 2 条并存）。
        let page = doc.pages().first()?;
        let annots = page.annotations();
        assert_eq!(
            annots.len(),
            1,
            "pdfium 视角：页注释恰 1 条（form widget 本体）"
        );
        let first = annots.first()?;
        assert_eq!(
            first.annotation_type(),
            PdfPageAnnotationType::Widget,
            "注释类型 /Widget"
        );

        println!(
            "| 探针S6 表单读（form_probe.pdf） | form()=Some, type={:?}, /Tx 域名值可读（probe_text=\"hello form\"） | {} ms（load）+ {} ms（field_values 枚举）|",
            ty, open_ms, enum_ms
        );

        // --- b2. 对照组：small.pdf（无表单）form() = None ---
        let doc = pdfium.load_pdf_from_file(&small, None)?;
        assert!(
            doc.form().is_none(),
            "对照组断言：无 /AcroForm 文档 form() 必须 None"
        );
        println!("| 探针S6 对照（small.pdf 无表单） | form()=None | OK |");
    }

    println!("\nS6 探针（formfill）全部通过——定谳复测与登记一致。");
    Ok(())
}
