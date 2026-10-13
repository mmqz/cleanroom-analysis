//! xlsx-r1 t10-b — 7 维真实测试语料构造（office-line/spike，分析仓写面）。
//!
//! 语料 = xlsx-r1 PLAN §2 t10-b 规格：
//!   正向五维（rust_xlsxwriter 生成，构造正确性以 calamine 打开断言自证）：
//!     dim1 formulas（SUM/跨表/嵌套 IF/字符串拼接/缓存值有无比对）
//!     dim2 styles（bold/字色/填充/边框/数字格式）
//!     dim3 merged（merge_range 多范围）
//!     dim4 multi-sheet（4 表 + 跨表公式）
//!     dim5 relationships（外部 URL 超链接 + 内部 sheet 链接）
//!   敌意两维（净室自构造，循 T3 纪律零外部样本）：
//!     dim6 malicious ZIP（谎言头/路径逃逸/真声明巨量/条目数超限/
//!            EOCD 损坏/缺 Content_Types）
//!     dim7 abnormal XML（截断/非法实体/控制字符/错误编码/非 XML 二进制）
//!
//! 产物：fixtures/corpus/（.gitignore）+ manifest.md（逐件构造意图 + 校验）。

use std::fmt::Write as _;
use std::io::Write as _;

use calamine::{Data, Reader, Xlsx};
use rust_xlsxwriter::{Format, Workbook};

const OUT: &str = "fixtures/corpus";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(OUT)?;
    let mut manifest = String::new();
    let _ = writeln!(manifest, "# 7 维语料 manifest（t10-b 构造产物）\n");

    dim1_formulas(&mut manifest)?;
    dim2_styles(&mut manifest)?;
    dim3_merged(&mut manifest)?;
    dim4_multisheet(&mut manifest)?;
    dim5_relationships(&mut manifest)?;
    dim6_malicious_zip(&mut manifest)?;
    dim7_abnormal_xml(&mut manifest)?;

    std::fs::write(format!("{OUT}/manifest.md"), &manifest)?;
    println!("\nok -> {OUT}/manifest.md");
    Ok(())
}

// ---------------------------------------------------------------------------
// dim1 公式：含/无缓存值对照（calamine 读回行为差异 = 验证点）
// ---------------------------------------------------------------------------

fn dim1_formulas(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{OUT}/dim1_formulas.xlsx");
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Formulas")?;
    // 数据列 A1:A5
    for i in 0..5u32 {
        ws.write_number(i, 0, f64::from(i) * 10.0)?;
    }
    // 带缓存值的公式：calamine worksheet_range 应读回缓存结果
    ws.write_formula(0, 1, "=SUM(A1:A5)")?;
    ws.set_formula_result(0, 1, "100"); // 0+10+20+30+40 = 100（无 Result 返回）
    // 无缓存值公式：worksheet_range 读回行为待实测（预期 0/空）
    ws.write_formula(1, 1, "=AVERAGE(A1:A5)")?;
    // 嵌套
    ws.write_formula(2, 1, "=IF(SUM(A1:A5)>50,MAX(A1:A5),MIN(A1:A5))")?;
    ws.set_formula_result(2, 1, "40");
    // 字符串拼接公式
    ws.write_formula(3, 1, r#"=CONCAT("总","计")"#)?;
    ws.set_formula_result(3, 1, "总计");
    wb.save(&p)?;

    // 校验：值面读缓存、公式面读文本
    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let rg = book.worksheet_range("Formulas")?;
    assert!(
        matches!(rg.get((0, 1)), Some(Data::Float(v)) if *v == 100.0),
        "缓存值公式读回：{:?}",
        rg.get((0, 1))
    );
    let no_cache = rg.get((1, 1)).cloned().unwrap_or(Data::Empty);
    let nf = book.worksheet_formula("Formulas")?;
    // 注意：Range::get 是相对坐标（公式 Range start=(0,1)，B 列即本 Range 第 0 列）
    assert!(
        matches!(nf.get((0, 0)), Some(s) if s.contains("SUM")),
        "公式文本读回（相对坐标 (0,0)）：{:?}",
        nf.get((0, 0))
    );
    let concat_form = nf.get((3, 0)).cloned().unwrap_or_default();
    let _ = writeln!(
        m,
        "| dim1_formulas.xlsx | 公式（SUM/AVG/嵌套 IF/CONCAT；2 带缓存值 2 不带） | 校验过：缓存值=100 读回 ✓ 公式文本读回 ✓ 无缓存值读回 = {no_cache:?}（读侧行为登记）；公式面三发现：文本不带 = 前缀、CONCAT 存储为 _xlfn.CONCAT、Range::get 为相对坐标 |"
    );
    let _ = concat_form;
    Ok(())
}

// ---------------------------------------------------------------------------
// dim2 样式：bold/字色/填充/边框/数字格式（值在位 + 样式可观测性登记）
// ---------------------------------------------------------------------------

fn dim2_styles(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{OUT}/dim2_styles.xlsx");
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Styles")?;

    let bold = Format::new().set_bold();
    let red_text = Format::new().set_font_color("#FF0000");
    let fill = Format::new().set_background_color("#FFFF00");
    let boxed = Format::new().set_border(rust_xlsxwriter::FormatBorder::Thin);
    let currency = Format::new().set_num_format("¥#,##0.00");
    let percent = Format::new().set_num_format("0.0%");

    ws.write_string_with_format(0, 0, "加粗标题", &bold)?;
    ws.write_string_with_format(0, 1, "红字", &red_text)?;
    ws.write_string_with_format(0, 2, "黄底", &fill)?;
    ws.write_string_with_format(0, 3, "边框", &boxed)?;
    ws.write_number_with_format(1, 0, 1234567.891, &currency)?;
    ws.write_number_with_format(1, 1, 0.8765, &percent)?;
    wb.save(&p)?;

    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let rg = book.worksheet_range("Styles")?;
    assert!(
        matches!(rg.get((0, 0)), Some(Data::String(s)) if s == "加粗标题"),
        "值在位"
    );
    assert!(matches!(rg.get((1, 0)), Some(Data::Float(_))), "数字在位");
    let _ = writeln!(
        m,
        "| dim2_styles.xlsx | 样式（bold/红字/黄底/细边框/¥货币格式/百分比格式） | 校验过：值面全部读回 ✓（样式面 calamine Range 不可观测——保真对比的预期丢失维度） |"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// dim3 合并单元格：多范围 merge_range
// ---------------------------------------------------------------------------

fn dim3_merged(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{OUT}/dim3_merged.xlsx");
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Merged")?;
    let center = Format::new().set_align(rust_xlsxwriter::FormatAlign::Center);

    ws.merge_range(0, 0, 1, 1, "2x2 合并", &center)?;
    ws.merge_range(0, 3, 0, 5, "1x3 横向", &center)?;
    ws.merge_range(3, 0, 6, 0, "4x1 纵向", &center)?;
    // 合并区外的散点值
    ws.write_number(3, 3, 42.0)?;
    wb.save(&p)?;

    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let rg = book.worksheet_range("Merged")?;
    assert!(
        matches!(rg.get((0, 0)), Some(Data::String(s)) if s == "2x2 合并"),
        "合并左上值在位"
    );
    assert!(
        matches!(rg.get((3, 3)), Some(Data::Float(v)) if *v == 42.0),
        "散点值在位"
    );
    let merges = book.merge_cells_by_sheet_name("Merged")?;
    assert_eq!(merges.len(), 3, "合并范围数：{merges:?}");
    let _ = writeln!(
        m,
        "| dim3_merged.xlsx | 合并（2x2 + 1x3 + 4x1 三范围 + 散点值） | 校验过：左上值/散点值读回 ✓ merge_cells_by_sheet_name = 3 范围 ✓ |"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// dim4 多工作表：4 表 + 跨表公式
// ---------------------------------------------------------------------------

fn dim4_multisheet(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{OUT}/dim4_multisheet.xlsx");
    let mut wb = Workbook::new();
    let data = wb.add_worksheet();
    data.set_name("Data")?;
    for i in 0..10u32 {
        data.write_number(i, 0, f64::from(i) * 7.0)?;
    }
    let ref_sheet = wb.add_worksheet();
    ref_sheet.set_name("RefSheet")?;
    ref_sheet.write_number(0, 0, 999.0)?;
    let calc = wb.add_worksheet();
    calc.set_name("Calc")?;
    // 跨表公式（带缓存值）
    calc.write_formula(0, 0, "=SUM(Data!A1:A10)")?;
    calc.set_formula_result(0, 0, "315"); // 7*(0+..+9)=315
    calc.write_formula(1, 0, "=RefSheet!A1*2")?;
    calc.set_formula_result(1, 0, "1998");
    let empty_last = wb.add_worksheet();
    empty_last.set_name("TailEmpty")?;
    wb.save(&p)?;

    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let names = book.sheet_names().to_vec();
    assert_eq!(
        names,
        vec!["Data", "RefSheet", "Calc", "TailEmpty"],
        "表序对账"
    );
    let rg = book.worksheet_range("Calc")?;
    assert!(
        matches!(rg.get((0, 0)), Some(Data::Float(v)) if *v == 315.0),
        "跨表公式缓存值"
    );
    let nf = book.worksheet_formula("Calc")?;
    assert!(
        matches!(nf.get((1, 0)), Some(s) if s.contains("RefSheet")),
        "跨表公式文本"
    );
    let _ = writeln!(
        m,
        "| dim4_multisheet.xlsx | 多表（Data/RefSheet/Calc/TailEmpty 四表 + 跨表公式 2 条 + 尾空表） | 校验过：表序 ✓ 跨表缓存值 315 ✓ 跨表公式文本 ✓ |"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// dim5 关系引用：超链接（外部 URL + 内部 sheet 链接）
// ---------------------------------------------------------------------------

fn dim5_relationships(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{OUT}/dim5_relationships.xlsx");
    let mut wb = Workbook::new();
    {
        let ws = wb.add_worksheet();
        ws.set_name("Links")?;
        let link_fmt = Format::new()
            .set_font_color("#0563C1")
            .set_underline(rust_xlsxwriter::FormatUnderline::Single);
        ws.write_url_with_format(0, 0, "https://example.com/docs", &link_fmt)?;
        ws.write_url_with_text(1, 0, "https://example.org/path?q=1&x=2", "带文本链接")?;
        ws.write_url(2, 0, "internal:Target!A1")?;
        // URL 样式字符串（非超链接单元格，纯值面）
        ws.write_string(3, 0, "https://example.net/plain")?;
        // t10-c v2：tooltip 面 + 外部工作簿子锚形态（PLAN 规格"外部工作簿引用"）
        ws.write_url_with_options(
            4,
            0,
            "https://example.com/tip",
            "悬停有提示的链接",
            "悬停提示文本",
            None,
        )?;
        ws.write_url(5, 0, "file:///C:/refs/Book2.xlsx#Sheet1!A1")?;
    }
    let target = wb.add_worksheet();
    target.set_name("Target")?;
    target.write_string(0, 0, "目标单元格")?;
    wb.save(&p)?;

    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let rg = book.worksheet_range("Links")?;
    assert!(
        matches!(rg.get((1, 0)), Some(Data::String(s)) if s == "带文本链接"),
        "带文本链接值"
    );
    let a1 = rg.get((0, 0)).cloned().unwrap_or(Data::Empty);
    // v2：超链接关系面可观测（calamine 0.36.1 hyperlinks_by_sheet_name）
    let hls = book.hyperlinks_by_sheet_name("Links")?;
    assert_eq!(hls.len(), 5, "超链接条数：{hls:?}");
    let file_link = hls
        .iter()
        .find(|h| h.range.start == (5, 0))
        .expect("file 子锚链接在位");
    assert!(
        file_link.target.as_deref().unwrap_or("").starts_with("file:///"),
        "外部工作簿 target：{file_link:?}"
    );
    assert_eq!(
        file_link.location.as_deref(),
        Some("Sheet1!A1"),
        "子锚 location：{file_link:?}"
    );
    let tip_link = hls
        .iter()
        .find(|h| h.range.start == (4, 0))
        .expect("tooltip 链接在位");
    assert_eq!(
        tip_link.tooltip.as_deref(),
        Some("悬停提示文本"),
        "tooltip：{tip_link:?}"
    );
    let _ = writeln!(
        m,
        "| dim5_relationships.xlsx | 关系引用（外链 URL×2 + 内部 Target!A1 链接 + 纯文本 URL 值 + tooltip 面 + file 子锚外部工作簿引用；rels 部件 = sheet1.xml.rels） | 校验过：值面读回 ✓ 超链接 API 读回 5 条 ✓（target/location/tooltip 三字段断言过；v1 manifest 曾误记「calamine 无公开 API」——v2 修正：0.36.1 hyperlinks_by_sheet_name 完整可观测）；A1 读回 = {a1:?} |"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 公共：最小 xlsx 骨架（T3 3c 提取复用）+ CRC-32
// ---------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = table[((crc ^ u32::from(b)) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

const CT: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#;
const RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;
const WB: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="S1" sheetId="1" r:id="rId1"/></sheets></workbook>"#;
const WB_RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#;

/// 手构最小 xlsx：四部件 zip + 调用方给定的 sheet1.xml 字节。
fn build_min_xlsx(path: &str, sheet: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    build_zip(
        path,
        &[
            ("[Content_Types].xml", CT.to_vec()),
            ("_rels/.rels", RELS.to_vec()),
            ("xl/workbook.xml", WB.to_vec()),
            ("xl/_rels/workbook.xml.rels", WB_RELS.to_vec()),
            ("xl/worksheets/sheet1.xml", sheet.to_vec()),
        ],
    )
}

/// 手构 zip（STORED 不压缩——敌意样本构造确定性优先，内容可控）。
fn build_zip(path: &str, entries: &[(&str, Vec<u8>)]) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut zw = zip::ZipWriter::new(f);
    let mut opts: zip::write::SimpleFileOptions = Default::default();
    opts = opts.compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        zw.start_file(*name, opts)?;
        zw.write_all(data)?;
    }
    zw.finish()?;
    Ok(())
}

/// 内存构造 zip 字节（供字节级破坏类样本）。
fn build_zip_bytes(entries: &[(&str, Vec<u8>)]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut cw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let mut opts: zip::write::SimpleFileOptions = Default::default();
    opts = opts.compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        cw.start_file(*name, opts)?;
        cw.write_all(data)?;
    }
    Ok(cw.finish()?.into_inner())
}

// ---------------------------------------------------------------------------
// dim6 恶意 ZIP 族（净室自构造）
// ---------------------------------------------------------------------------

fn dim6_malicious_zip(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    // 6a 谎言头炸弹（T3 3b 同构：DEFLATE 声明 16B 实际 1MB）
    let payload = vec![b'A'; 1 << 20];
    let mut enc = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&payload)?;
    let deflated = enc.finish()?;
    let crc = crc32(&payload);
    let name = b"bomb.txt";
    let lie: u32 = 16;
    let mut z = Vec::new();
    // 本地头
    z.extend_from_slice(b"PK\x03\x04");
    z.extend_from_slice(&20u16.to_le_bytes()); // version needed
    z.extend_from_slice(&0u16.to_le_bytes()); // flags
    z.extend_from_slice(&8u16.to_le_bytes()); // method = deflate
    z.extend_from_slice(&0u16.to_le_bytes()); // time
    z.extend_from_slice(&0u16.to_le_bytes()); // date
    z.extend_from_slice(&crc.to_le_bytes());
    z.extend_from_slice(&(deflated.len() as u32).to_le_bytes()); // compressed（真）
    z.extend_from_slice(&lie.to_le_bytes()); // uncompressed（谎）
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // extra len
    z.extend_from_slice(name);
    z.extend_from_slice(&deflated);
    let cd_off = z.len() as u32;
    // 中央目录
    z.extend_from_slice(b"PK\x01\x02");
    z.extend_from_slice(&20u16.to_le_bytes()); // version made by
    z.extend_from_slice(&20u16.to_le_bytes()); // version needed
    z.extend_from_slice(&0u16.to_le_bytes()); // flags
    z.extend_from_slice(&8u16.to_le_bytes()); // method
    z.extend_from_slice(&0u16.to_le_bytes()); // time
    z.extend_from_slice(&0u16.to_le_bytes()); // date
    z.extend_from_slice(&crc.to_le_bytes());
    z.extend_from_slice(&(deflated.len() as u32).to_le_bytes());
    z.extend_from_slice(&lie.to_le_bytes());
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // extra
    z.extend_from_slice(&0u16.to_le_bytes()); // comment
    z.extend_from_slice(&0u16.to_le_bytes()); // disk
    z.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
    z.extend_from_slice(&0u32.to_le_bytes()); // external attrs
    z.extend_from_slice(&0u32.to_le_bytes()); // local header offset
    z.extend_from_slice(name);
    let cd_size = z.len() as u32 - cd_off;
    // EOCD
    z.extend_from_slice(b"PK\x05\x06");
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&cd_size.to_le_bytes());
    z.extend_from_slice(&cd_off.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    let p6a = format!("{OUT}/mal6a_lie_header.zip");
    std::fs::write(&p6a, &z)?;
    assert!(
        std::fs::metadata(&p6a)?.len() < 4096,
        "谎言头件本体应极小"
    );

    // 6b 路径逃逸条目族
    let p6b = format!("{OUT}/mal6b_path_escape.zip");
    build_zip(
        &p6b,
        &[
            ("[Content_Types].xml", CT.to_vec()),
            ("../evil.txt", b"escape".to_vec()),
            ("/abs/path.txt", b"abs".to_vec()),
            ("ok.txt", b"fine".to_vec()),
        ],
    )?;

    // 6c 真声明巨量（zip64 STORED 条目声明 320MB > xlsx 族 300MB 上限；
    //    本地头 zip64 extra 声明大 usize、中央目录镜像同值——预扫只看声明）
    let p6c = format!("{OUT}/mal6c_declared_huge.zip");
    {
        let mut z = Vec::new();
        let name = b"blob.bin";
        let declared: u64 = 320_000_000; // 320MB > 300MB xlsx 族上限
        // 本地头（zip64）
        z.extend_from_slice(b"PK\x03\x04");
        z.extend_from_slice(&45u16.to_le_bytes()); // version needed (zip64)
        z.extend_from_slice(&0u16.to_le_bytes()); // flags
        z.extend_from_slice(&0u16.to_le_bytes()); // STORED
        z.extend_from_slice(&0u16.to_le_bytes()); // time
        z.extend_from_slice(&0u16.to_le_bytes()); // date
        z.extend_from_slice(&0u32.to_le_bytes()); // crc
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // csize zip64 标记
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // usize zip64 标记
        z.extend_from_slice(&(name.len() as u16).to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // extra len 占位（下方 zip64 extra 20B）
        let extra_len_pos = z.len() - 2;
        z.extend_from_slice(name);
        // zip64 extra field（usize 320MB + csize 320MB，读界用）
        z.extend_from_slice(&0x0001u16.to_le_bytes()); // header id
        z.extend_from_slice(&16u16.to_le_bytes()); // data size
        z.extend_from_slice(&declared.to_le_bytes()); // usize
        z.extend_from_slice(&declared.to_le_bytes()); // csize
        let real_extra_len = 20u16; // 2(id)+2(size)+16(data)
        z[extra_len_pos..extra_len_pos + 2].copy_from_slice(&real_extra_len.to_le_bytes());
        let lh_size = z.len();
        let cd_off = lh_size as u64;
        // 中央目录（zip64：offset 用 extra 承载）
        z.extend_from_slice(b"PK\x01\x02");
        z.extend_from_slice(&45u16.to_le_bytes()); // version made by
        z.extend_from_slice(&45u16.to_le_bytes()); // version needed
        z.extend_from_slice(&0u16.to_le_bytes()); // flags
        z.extend_from_slice(&0u16.to_le_bytes()); // STORED
        z.extend_from_slice(&0u16.to_le_bytes()); // time
        z.extend_from_slice(&0u16.to_le_bytes()); // date
        z.extend_from_slice(&0u32.to_le_bytes()); // crc
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // csize 标记
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // usize 标记
        z.extend_from_slice(&(name.len() as u16).to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // extra len 占位
        z.extend_from_slice(&0u16.to_le_bytes()); // comment
        z.extend_from_slice(&0u16.to_le_bytes()); // disk
        z.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        z.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // offset 标记
        z.extend_from_slice(name);
        // zip64 extra（中央目录位：usize/csize/offset 三槽）
        z.extend_from_slice(&0x0001u16.to_le_bytes());
        z.extend_from_slice(&24u16.to_le_bytes());
        z.extend_from_slice(&declared.to_le_bytes()); // usize
        z.extend_from_slice(&declared.to_le_bytes()); // csize
        z.extend_from_slice(&0u64.to_le_bytes()); // offset
        let real_cd_extra_len = 28u16; // 2+2+24
        // extralen 字段位于 CD 记录 offset 30（sig4+26 定段尾）：从末尾回溯
        // 52 = 28(extra) + 8(name) + 16(46-30 定段尾段)。v1 曾用 z.len()-30
        // （差 22 字节，补丁写进 name 尾部 + extralen 恒 0 → 容器不可解析，
        // python/zip9 双重实证 "Bad offset for central directory"）。
        let cd_extra_pos = z.len() - 52;
        z[cd_extra_pos..cd_extra_pos + 2].copy_from_slice(&real_cd_extra_len.to_le_bytes());
        let cd_size = z.len() as u32 - cd_off as u32;
        // zip64 EOCD（v2 修正：size 字段 u64 8 字节——APPNOTE 4.4.21 +
        // zip-rs write.rs:1504（record_size = ext + 44）；v1 曾误写 u32 4 字节）
        let z64_off = z.len() as u64;
        z.extend_from_slice(b"PK\x06\x06");
        z.extend_from_slice(&44u64.to_le_bytes()); // size of record（8 字节）
        z.extend_from_slice(&45u16.to_le_bytes()); // version made by
        z.extend_from_slice(&45u16.to_le_bytes()); // version needed
        z.extend_from_slice(&0u32.to_le_bytes()); // this disk
        z.extend_from_slice(&0u32.to_le_bytes()); // disk with cd
        z.extend_from_slice(&1u64.to_le_bytes()); // entries this disk
        z.extend_from_slice(&1u64.to_le_bytes()); // entries total
        z.extend_from_slice(&(cd_size as u64).to_le_bytes());
        z.extend_from_slice(&cd_off.to_le_bytes());
        // 注意：无 extensible data len 字段——记录定长 56B（python '<IQHHIIQQQQ'
        // 模型 + zip-rs record_size=44=2+2+4+4+8+8+8+8；ext len 属 extensible
        // data 段，有 ext 数据时才随数据写。v1 曾多写 4B → 记录 60B，
        // python extrasz=4 → check2 失败"Corrupt zip64 EOCD"）
        // zip64 EOCD Locator（PK\x06\x07，20B）——v2 补：缺失则普通 EOCD 的
        // 0xFFFFFFFF cd offset 无法回溯 zip64 记录（v1 败因之二）
        z.extend_from_slice(b"PK\x06\x07");
        z.extend_from_slice(&0u32.to_le_bytes()); // disk with cd
        z.extend_from_slice(&z64_off.to_le_bytes()); // zip64 EOCD offset
        z.extend_from_slice(&1u32.to_le_bytes()); // number of disks
        // 普通 EOCD（disk 字段实值 0——zip-rs 写侧同构；cd 三字段 0xFFFF 标记）
        z.extend_from_slice(b"PK\x05\x06");
        z.extend_from_slice(&0u16.to_le_bytes()); // disk
        z.extend_from_slice(&0u16.to_le_bytes()); // cd disk
        z.extend_from_slice(&0xFFFFu16.to_le_bytes()); // entries 标记
        z.extend_from_slice(&0xFFFFu16.to_le_bytes()); // total 标记
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // cd size 标记
        z.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // cd off 标记
        z.extend_from_slice(&0u16.to_le_bytes()); // comment
        std::fs::write(&p6c, &z)?;
    }

    // 6d 条目数超限（10,001 条目 > SECURITY_POLICY §3 上限 10,000）
    let p6d = format!("{OUT}/mal6d_too_many_entries.zip");
    {
        let f = std::io::BufWriter::new(std::fs::File::create(&p6d)?);
        let mut zw = zip::ZipWriter::new(f);
        let mut opts: zip::write::SimpleFileOptions = Default::default();
        opts = opts.compression_method(zip::CompressionMethod::Stored);
        for i in 0..10_001u32 {
            zw.start_file(format!("e{i}.txt"), opts)?;
            zw.write_all(b"x")?;
        }
        zw.finish()?;
    }

    // 6e EOCD 损坏（中央目录偏移低字节破坏）
    let p6e = format!("{OUT}/mal6e_eocd_corrupt.zip");
    {
        let mut z = build_zip_bytes(&[
            ("[Content_Types].xml", CT.to_vec()),
            ("a.txt", b"a".to_vec()),
        ])?;
        let n = z.len();
        z[n - 6] = 0xEE; // EOCD cd offset 低字节
        std::fs::write(&p6e, &z)?;
    }

    // 6f 缺 [Content_Types].xml（合法 zip、xlsx 语义损坏）
    let p6f = format!("{OUT}/mal6f_no_content_types.zip");
    build_zip(
        &p6f,
        &[
            ("_rels/.rels", RELS.to_vec()),
            ("xl/workbook.xml", WB.to_vec()),
            ("xl/worksheets/sheet1.xml", b"<worksheet/>".to_vec()),
        ],
    )?;

    let _ = writeln!(
        m,
        "| mal6a_lie_header.zip | 谎言头炸弹（DEFLATE 声明 16B 实际 1MB；本体 <4KB） | 敌意——读界行为验证件 |"
    );
    let _ = writeln!(
        m,
        "| mal6b_path_escape.zip | 路径逃逸（../ 与 /abs 两形态 + 正常件对照） | 敌意——预扫路径校验件 |"
    );
    let _ = writeln!(
        m,
        "| mal6c_declared_huge.zip | 真声明巨量（zip64 声明 320MB > xlsx 族 300MB 上限） | 敌意——声明求和门件 |"
    );
    let _ = writeln!(
        m,
        "| mal6d_too_many_entries.zip | 条目数 10,001 > 上限 10,000 | 敌意——条目数门件 |"
    );
    let _ = writeln!(
        m,
        "| mal6e_eocd_corrupt.zip | EOCD 中央目录偏移破坏 | 敌意——容器损坏件 |"
    );
    let _ = writeln!(
        m,
        "| mal6f_no_content_types.zip | 缺 [Content_Types].xml（zip 合法、xlsx 语义损坏） | 敌意——语义损坏件 |"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// dim7 异常 XML 族（最小骨架 + sheet1.xml 篡改）
// ---------------------------------------------------------------------------

fn dim7_abnormal_xml(m: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let head: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>"#;
    let tail: &[u8] = br#"</t></is></c></row></sheetData></worksheet>"#;

    // 7a 截断（尾闭合缺失）
    let p7a = format!("{OUT}/mal7a_xml_truncated.xlsx");
    let mut sheet = head.to_vec();
    sheet.extend_from_slice(b"val");
    sheet.extend_from_slice(br#"</t></is></c></row></sheetData"#); // 缺 </worksheet>
    build_min_xlsx(&p7a, &sheet)?;

    // 7b 非法实体
    let p7b = format!("{OUT}/mal7b_bad_entity.xlsx");
    let mut sheet = head.to_vec();
    sheet.extend_from_slice(b"&bogus;");
    sheet.extend_from_slice(tail);
    build_min_xlsx(&p7b, &sheet)?;

    // 7c 非法控制字符（XML 1.0 禁 0x00）
    let p7c = format!("{OUT}/mal7c_control_char.xlsx");
    let mut sheet = head.to_vec();
    sheet.extend_from_slice(b"a\x00b");
    sheet.extend_from_slice(tail);
    build_min_xlsx(&p7c, &sheet)?;

    // 7d 错误编码声明（声明 UTF-8、实写 GBK 字节：中=D6D0 文=CEC4）
    let p7d = format!("{OUT}/mal7d_bad_encoding.xlsx");
    let mut sheet = head.to_vec();
    sheet.extend_from_slice(&[0xD6, 0xD0, 0xCE, 0xC4]);
    sheet.extend_from_slice(tail);
    build_min_xlsx(&p7d, &sheet)?;

    // 7e 非 XML 二进制垃圾
    let p7e = format!("{OUT}/mal7e_not_xml.xlsx");
    let sheet: Vec<u8> = (0..256u16).map(|i| i as u8).cycle().take(1024).collect();
    build_min_xlsx(&p7e, &sheet)?;

    // 对照组：正常最小档（敌意件的行为基准）
    let p7ok = format!("{OUT}/ok7_control_normal.xlsx");
    let mut sheet = head.to_vec();
    sheet.extend_from_slice(b"normal");
    sheet.extend_from_slice(tail);
    build_min_xlsx(&p7ok, &sheet)?;
    let mut book: Xlsx<_> = calamine::open_workbook(&p7ok)?;
    let rg = book.worksheet_range("S1")?;
    assert!(
        matches!(rg.get((0, 0)), Some(Data::String(s)) if s == "normal"),
        "对照档构造正确性"
    );

    let _ = writeln!(
        m,
        "| mal7a_xml_truncated.xlsx | sheet XML 截断（缺尾闭合） | 敌意——解析器健壮性件 |"
    );
    let _ = writeln!(
        m,
        "| mal7b_bad_entity.xlsx | 非法实体 &bogus; | 敌意——实体解析件 |"
    );
    let _ = writeln!(
        m,
        "| mal7c_control_char.xlsx | 文本体嵌 0x00（XML 1.0 非法） | 敌意——字符合法性件 |"
    );
    let _ = writeln!(
        m,
        "| mal7d_bad_encoding.xlsx | 声明 UTF-8 实写 GBK 字节 | 敌意——编码件 |"
    );
    let _ = writeln!(
        m,
        "| mal7e_not_xml.xlsx | 部件为二进制垃圾 | 敌意——非 XML 件 |"
    );
    let _ = writeln!(
        m,
        "| ok7_control_normal.xlsx | 正常最小档（inlineStr normal） | 对照基准（calamine 读回 ✓） |"
    );
    Ok(())
}
