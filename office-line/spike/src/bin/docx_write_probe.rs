//! t20-a 探针：docx-rs 0.4.22 写面四件 API（numbering / rFonts / pBdr / ind）
//! + 块级 code 等宽段落的一手实证核验（t18 四件降级项的库能力边界）。
//!
//! 产物：生成一个包含全部五类表达面的 .docx 到临时文件，随后**进程内读回
//! 解包**（spike 既有 zip 8.6 依赖；docx-rs 打包为 Stored 无压缩，读回零
//! 特性需求），对每个结构面做子串断言并打印 PASS/FAIL，退出码 0=全过 /
//! 1=有失败。核验项：
//!   1. numbering.xml part 存在 + document.xml.rels 含 numbering 关系
//!      （has_numbering 接线）+ abstractNum（decimal 有序 / bullet 无序）+
//!      num numId 桥 + document.xml 段落层 numPr（numId + ilvl，含二级）
//!   2. 行内等宽：run 层 rPr 内 `<w:rFonts w:ascii="Consolas" …/>`
//!   3. hr：段落 pPr 内 `<w:pBdr>` + `<w:bottom …/>`（下边框水平线表达）
//!   4. quote：`<w:ind w:left=…/>` 缩进 + `<w:pBdr>` 内 `<w:left …/>`
//!   5. 块级 code：段落级 rPr/rFonts（pPr 首子 rPr）+ 每 run rFonts 双通道
//!
//! 分析仓 spike 探针（office-line/spike），不进 mdcore。

use docx_rs::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// 行内 code mark 的等宽字体（t18 降级项 2）。
const MONO_INLINE: &str = "Consolas";
/// 块级 code 的等宽字体（与行内分名，断言可区分两条通道）。
const MONO_BLOCK: &str = "Courier New";

/// 有序列表：abstractNumId = numId = 100（两 id 同值简化读回断言）。
const ORDERED_ID: usize = 100;
/// 无序列表：abstractNumId = numId = 101。
const BULLET_ID: usize = 101;

fn mono(font: &str) -> RunFonts {
    RunFonts::new().ascii(font).hi_ansi(font).cs(font)
}

/// 有序（decimal）两级 abstract numbering——多级列表支持实证（ilvl 0/1）。
fn ordered_abstract() -> AbstractNumbering {
    AbstractNumbering::new(ORDERED_ID)
        .add_level(
            Level::new(
                0,
                Start::new(1),
                NumberFormat::new("decimal"),
                LevelText::new("%1."),
                LevelJc::new("left"),
            )
            .indent(Some(720), Some(SpecialIndentType::Hanging(360)), None, None),
        )
        .add_level(
            Level::new(
                1,
                Start::new(1),
                NumberFormat::new("lowerLetter"),
                LevelText::new("%1.%2."),
                LevelJc::new("left"),
            )
            .indent(
                Some(1440),
                Some(SpecialIndentType::Hanging(360)),
                None,
                None,
            ),
        )
}

/// 无序（bullet）abstract numbering——lvlText 携带项目符号字形。
fn bullet_abstract() -> AbstractNumbering {
    AbstractNumbering::new(BULLET_ID).add_level(
        Level::new(
            0,
            Start::new(1),
            NumberFormat::new("bullet"),
            LevelText::new("\u{2022}"),
            LevelJc::new("left"),
        )
        .indent(Some(720), Some(SpecialIndentType::Hanging(360)), None, None),
    )
}

fn build_probe_doc() -> Docx {
    let hr_paragraph = {
        let mut p = Paragraph::new(); // 空段：OOXML 无原生 hr，惯用 = 段落下边框
        p.property = p.property.set_borders(
            ParagraphBorders::with_empty().set(
                ParagraphBorder::new(ParagraphBorderPosition::Bottom)
                    .size(6) // eighths of a point = 0.75pt
                    .space(1)
                    .color("auto"),
            ),
        );
        p
    };

    let quote_paragraph = {
        let mut p = Paragraph::new()
            .add_run(Run::new().add_text("quoted text with left indent and left border"))
            .indent(Some(720), None, None, None);
        p.property = p.property.set_borders(
            ParagraphBorders::with_empty().set(
                ParagraphBorder::new(ParagraphBorderPosition::Left)
                    .size(12)
                    .space(4)
                    .color("auto"),
            ),
        );
        p
    };

    let block_code_paragraph = Paragraph::new()
        .fonts(mono(MONO_BLOCK)) // 段落级 rPr/rFonts（段落标记 + 缺省面）
        .add_run(Run::new().add_text("let x = 1;").fonts(mono(MONO_BLOCK)))
        .add_run(
            Run::new()
                .add_text(" // each run mono too")
                .fonts(mono(MONO_BLOCK)),
        );

    Docx::new()
        // 1a. 有序列表 ×2（ilvl 0）
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("first ordered item"))
                .numbering(NumberingId::new(ORDERED_ID), IndentLevel::new(0)),
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("second ordered item"))
                .numbering(NumberingId::new(ORDERED_ID), IndentLevel::new(0)),
        )
        // 1b. 多级：同一 numId 的 ilvl 1 嵌套项
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("nested level-1 item"))
                .numbering(NumberingId::new(ORDERED_ID), IndentLevel::new(1)),
        )
        // 1c. 无序列表 ×2
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("first bullet item"))
                .numbering(NumberingId::new(BULLET_ID), IndentLevel::new(0)),
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("second bullet item"))
                .numbering(NumberingId::new(BULLET_ID), IndentLevel::new(0)),
        )
        // 2. 行内等宽：普通 run + Consolas run + 普通 run
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("before "))
                .add_run(Run::new().add_text("inline_code").fonts(mono(MONO_INLINE)))
                .add_run(Run::new().add_text(" after")),
        )
        // 3. hr：段落下边框表达
        .add_paragraph(hr_paragraph)
        // 4. quote：缩进 + 左边框
        .add_paragraph(quote_paragraph)
        // 5. 块级 code：段落级 + run 级双通道等宽
        .add_paragraph(block_code_paragraph)
        // 注册 numbering 定义（document 级）
        .add_abstract_numbering(ordered_abstract())
        .add_abstract_numbering(bullet_abstract())
        .add_numbering(Numbering::new(ORDERED_ID, ORDERED_ID))
        .add_numbering(Numbering::new(BULLET_ID, BULLET_ID))
}

fn read_part(
    archive: &mut zip::ZipArchive<File>,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut entry = archive.by_name(name)?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() -> std::process::ExitCode {
    let out_path: PathBuf = std::env::temp_dir().join("t20a_docx_write_probe.docx");
    let out_path_display = out_path.display();

    // --- 生成 ---
    let file = match File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FAIL create {}: {e}", out_path_display);
            return std::process::ExitCode::FAILURE;
        }
    };
    if let Err(e) = build_probe_doc().build().pack(file) {
        eprintln!("FAIL pack: {e:?}");
        return std::process::ExitCode::FAILURE;
    }
    println!("generated: {out_path_display}");

    // --- 读回解包 ---
    let file = match File::open(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FAIL reopen: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("FAIL open zip: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    let names: Vec<String> = archive.file_names().map(|n| n.to_string()).collect();
    let document = match read_part(&mut archive, "word/document.xml") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("FAIL read word/document.xml: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let numbering_xml = read_part(&mut archive, "word/numbering.xml").unwrap_or_default();
    let rels = read_part(&mut archive, "word/_rels/document.xml.rels").unwrap_or_default();

    // --- 结构面断言 ---
    let checks: Vec<(&str, bool)> = vec![
        (
            "numbering.xml part exists in zip",
            names.iter().any(|n| n == "word/numbering.xml"),
        ),
        (
            "document.xml.rels has numbering relationship (has_numbering wiring)",
            rels.contains("relationships/numbering") && rels.contains("numbering.xml"),
        ),
        (
            "numbering.xml: ordered abstractNum (decimal fmt + lvlText %1.)",
            numbering_xml.contains(r#"<w:abstractNum w:abstractNumId="100""#)
                && numbering_xml.contains(r#"w:numFmt w:val="decimal""#)
                && numbering_xml.contains(r#"w:lvlText w:val="%1.""#),
        ),
        (
            "numbering.xml: bullet abstractNum (bullet fmt + • glyph)",
            numbering_xml.contains(r#"<w:abstractNum w:abstractNumId="101""#)
                && numbering_xml.contains(r#"w:numFmt w:val="bullet""#),
        ),
        (
            "numbering.xml: concrete num bridges (numId 100/101 -> abstract)",
            numbering_xml.contains(r#"<w:num w:numId="100"><w:abstractNumId w:val="100" /></w:num>"#)
                && numbering_xml
                    .contains(r#"<w:num w:numId="101"><w:abstractNumId w:val="101" /></w:num>"#),
        ),
        (
            "numbering.xml: multi-level (ilvl=1 level with lowerLetter fmt)",
            numbering_xml.contains(r#"w:numFmt w:val="lowerLetter""#)
                && numbering_xml.contains(r#"w:lvlText w:val="%1.%2.""#),
        ),
        (
            "document.xml: ordered numPr (numId=100 + ilvl=0, exactly two items)",
            document
                .matches(r#"<w:numId w:val="100" /><w:ilvl w:val="0" />"#)
                .count()
                == 2,
        ),
        (
            "document.xml: nested numPr (numId=100 + ilvl=1, exactly one)",
            document
                .matches(r#"<w:numId w:val="100" /><w:ilvl w:val="1" />"#)
                .count()
                == 1,
        ),
        (
            "document.xml: bullet numPr (numId=101 + ilvl=0, exactly two items)",
            document
                .matches(r#"<w:numId w:val="101" /><w:ilvl w:val="0" />"#)
                .count()
                == 2,
        ),
        (
            "document.xml: inline mono run rFonts (Consolas)",
            document.contains(&format!(
                r#"<w:rFonts w:ascii="{MONO_INLINE}" w:hAnsi="{MONO_INLINE}" w:cs="{MONO_INLINE}" />"#
            )),
        ),
        (
            "document.xml: hr paragraph border (pBdr + w:bottom)",
            document.contains("<w:pBdr>") && document.contains(r#"<w:bottom "#),
        ),
        (
            "document.xml: quote indent (w:ind w:left=\"720\")",
            document.contains(r#"w:ind w:left="720""#),
        ),
        (
            "document.xml: quote left border (pBdr + w:left)",
            document.contains("<w:pBdr>") && document.contains(r#"<w:left "#),
        ),
        (
            "document.xml: block-code paragraph-level rFonts (pPr>rPr>rFonts Courier New)",
            document.contains(&format!(
                r#"<w:pPr><w:rPr><w:rFonts w:ascii="{MONO_BLOCK}" w:hAnsi="{MONO_BLOCK}" w:cs="{MONO_BLOCK}" />"#
            )),
        ),
        (
            "document.xml: block-code run-level rFonts (Courier New, two runs)",
            document.matches(&format!(
                r#"<w:rFonts w:ascii="{MONO_BLOCK}" w:hAnsi="{MONO_BLOCK}" w:cs="{MONO_BLOCK}" />"#
            ))
            .count()
                >= 2,
        ),
        (
            "document.xml: list item texts survived (5 items + inline + quote + code)",
            document.contains("first ordered item")
                && document.contains("second ordered item")
                && document.contains("nested level-1 item")
                && document.contains("first bullet item")
                && document.contains("second bullet item")
                && document.contains("inline_code")
                && document.contains("quoted text with left indent and left border")
                && document.contains("let x = 1;"),
        ),
    ];

    let total = checks.len();
    let passed = checks.iter().filter(|(_, ok)| *ok).count();
    let mut failed_any = false;
    for (label, ok) in &checks {
        println!("{} {label}", if *ok { "PASS" } else { "FAIL" });
        if !ok {
            failed_any = true;
        }
    }
    println!("{passed}/{total} checks passed");

    if failed_any {
        // 失败时落两段 XML 证据，便于诊断
        eprintln!("--- word/document.xml (first 4000 bytes) ---");
        eprintln!("{}", &document[..document.len().min(4000)]);
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
