//! office-impl T3 — 三探针（office-line/spike，分析仓写面）。
//!
//! 探针 1 xlsx 读面：rust_xlsxwriter 生成 fixtures（跨件互证）→
//!   calamine 打开 + 全量提取 + 抽点断言 + 计时（常规 + 大表量级）。
//! 探针 2 xlsx 写面：constant_memory 10 万行写 + 产物体积 + calamine
//!   重开往返互证（ADR-O1 O1-1/O1-2 推荐件的组合证据）。
//! 探针 3 敌意输入：
//!   3a 声明尺寸预扫（SECURITY_POLICY §2.1 机制演示）；
//!   3b 谎言头炸弹（deflate 声明 16B 实际膨胀 1MB → zip 读行为观察）；
//!   3c 巨量 dimension 声明（手构最小 xlsx → calamine 行为观察）。
//!
//! 产物：fixtures/ 本地（不入 git）；数字进 office-e 报告与
//! SECURITY_POLICY §3 限额表回填。

use std::fmt::Write as _;
use std::io::Read;
use std::io::Write as _;
use std::path::Path;
use std::time::Instant;

use calamine::{Data, Reader, Xlsx};
use rust_xlsxwriter::Workbook;

const FIX: &str = "fixtures";

fn ms(d: std::time::Duration) -> String {
    format!("{:.2} ms", d.as_secs_f64() * 1000.0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(FIX)?;
    let mut report = String::new();

    probe1_read(&mut report)?;
    probe2_write(&mut report)?;
    probe3_hostile(&mut report)?;

    println!("\n==== 探针报告 ====\n{report}");
    std::fs::write(format!("{FIX}/probe_report.md"), &report)?;
    println!("ok -> {FIX}/probe_report.md");
    Ok(())
}

// ---------------------------------------------------------------------------
// 探针 1：xlsx 读面（calamine）
// ---------------------------------------------------------------------------

fn make_fixture(path: &Path, sheets: usize, rows: u32, cols: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut wb = Workbook::new();
    for s in 0..sheets {
        let ws = wb.add_worksheet();
        ws.set_name(format!("Sheet{}", s + 1))?;
        for r in 0..rows {
            for c in 0..cols {
                match (r + u32::from(c)) % 3 {
                    0 => {
                        ws.write_string(r, c, format!("r{r}c{c}文本"))?;
                    }
                    1 => {
                        ws.write_number(r, c, f64::from(c) * 1000.0 + f64::from(r))?;
                    }
                    _ => {
                        ws.write(r, c, (r % 2) == 0)?;
                    }
                }
            }
        }
    }
    wb.save(path)?;
    Ok(())
}

fn probe1_read(report: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    // 常规 fixture：3 sheet × 2000 行 × 26 列
    let p = format!("{FIX}/mixed.xlsx");
    make_fixture(Path::new(&p), 3, 2000, 26)?;

    let t = Instant::now();
    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let t_open = t.elapsed();
    let names = book.sheet_names().to_vec();

    let mut cells = 0usize;
    let t = Instant::now();
    for name in &names {
        let rg = book.worksheet_range(name)?;
        for row in rg.rows() {
            for c in row {
                if !matches!(c, Data::Empty) {
                    cells += 1;
                }
            }
        }
    }
    let t_extract = t.elapsed();

    // 抽点断言（写读互证）
    let rg = book.worksheet_range("Sheet1")?;
    let a1 = rg.get((0, 0)).expect("A1 在位");
    assert!(matches!(a1, Data::String(s) if s.contains("r0c0")), "A1 应为字符串：{a1:?}");
    let b1 = rg.get((0, 1)).expect("B1 在位");
    assert!(matches!(b1, Data::Float(_)), "B1 应为数字：{b1:?}");
    let c1 = rg.get((0, 2)).expect("C1 在位");
    assert!(matches!(c1, Data::Bool(_)), "C1 应为布尔：{c1:?}");
    let expect_cells = 3usize * 2000 * 26;
    assert_eq!(cells, expect_cells, "非空单元格全量对账");

    // 大表量级：1 sheet × 50_000 行 × 10 列
    let p2 = format!("{FIX}/large.xlsx");
    make_fixture(Path::new(&p2), 1, 50_000, 10)?;
    let t = Instant::now();
    let mut book2: Xlsx<_> = calamine::open_workbook(&p2)?;
    let t2_open = t.elapsed();
    let t = Instant::now();
    let rg = book2.worksheet_range("Sheet1")?;
    let mut rows_n = 0usize;
    for _row in rg.rows() {
        rows_n += 1;
    }
    let t2_extract = t.elapsed();
    assert_eq!(rows_n, 50_000, "大表行数对账");

    let _ = writeln!(
        report,
        "| 探针 | 内容 | 结果 | 时间 |\n|---|---|---|---|\n\
         | 1 读·常规 | 3 sheet×2000 行×26 列（156,000 格） | 打开+全量提取+抽点断言+全量对账通过 | {} + {} |\n\
         | 1 读·大表 | 1 sheet×50,000 行×10 列（500,000 格） | 提取 5 万行对账通过 | {} + {} |",
        ms(t_open), ms(t_extract), ms(t2_open), ms(t2_extract)
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 探针 2：xlsx 写面（rust_xlsxwriter constant_memory）+ 写读往返
// ---------------------------------------------------------------------------

fn probe2_write(report: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{FIX}/big_write.xlsx");
    const ROWS: u32 = 100_000;
    const COLS: u16 = 10;

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet_with_constant_memory();
    let t = Instant::now();
    for r in 0..ROWS {
        for c in 0..COLS {
            if c == 0 {
                ws.write_string(r, c, format!("行{r}"))?;
            } else {
                ws.write_number(r, c, f64::from(r) * 10.0 + f64::from(c))?;
            }
        }
    }
    wb.save(&p)?;
    let t_write = t.elapsed();
    let size = std::fs::metadata(&p)?.len();

    // 写读往返互证（ADR-O1 O1-1 + O1-2 组合证据）
    let t = Instant::now();
    let mut book: Xlsx<_> = calamine::open_workbook(&p)?;
    let rg = book.worksheet_range("Sheet1")?;
    let last = rg.get(((ROWS - 1) as usize, (COLS - 1) as usize)).expect("末格在位");
    assert!(
        matches!(last, Data::Float(v) if (*v - (f64::from(ROWS - 1) * 10.0 + 9.0)).abs() < 1e-9),
        "末格往返值：{last:?}"
    );
    let rows_n = rg.rows().count();
    assert_eq!(rows_n, ROWS as usize, "往返行数对账");
    let t_roundtrip = t.elapsed();

    let _ = writeln!(
        report,
        "| 2 写·流式 | constant_memory {} 行×{} 列（100 万格） | 写+保存通过，产物 {} | {} |\n\
         | 2 写读往返 | calamine 重开+末格断言+行数对账 | 通过 | {} |",
        ROWS, COLS, humansize(size), ms(t_write), ms(t_roundtrip)
    );
    Ok(())
}

fn humansize(b: u64) -> String {
    if b >= 1 << 20 {
        format!("{:.2} MB", b as f64 / (1u64 << 20) as f64)
    } else {
        format!("{:.1} KB", b as f64 / 1024.0)
    }
}

// ---------------------------------------------------------------------------
// 探针 3：敌意输入（预扫 / 谎言头 / 巨量 dimension）
// ---------------------------------------------------------------------------

/// 3a：声明尺寸预扫（SECURITY_POLICY §2.1 机制演示）。
fn probe3a(report: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let p = format!("{FIX}/prescan.zip");
    let f = std::io::BufWriter::new(std::fs::File::create(&p)?);
    let mut zw = zip::ZipWriter::new(f);
    let opts: zip::write::SimpleFileOptions = Default::default();
    // 三个条目：正常小件 + 路径逃逸名 + 大声明件（写 2MB 数据）
    zw.start_file("normal.txt", opts)?;
    zw.write_all(b"hello")?;
    zw.start_file("../evil.txt", opts)?;
    zw.write_all(b"escape")?;
    zw.start_file("blob.bin", opts)?;
    let blob = vec![0xABu8; 2 << 20]; // 2 MB
    zw.write_all(&blob)?;
    zw.finish()?;

    let mut za = zip::ZipArchive::new(std::fs::File::open(&p)?)?;
    let n = za.len();
    let mut declared_sum = 0u64;
    let mut path_escape = 0u32;
    let t = Instant::now();
    for i in 0..n {
        let zf = za.by_index(i)?;
        declared_sum += zf.size(); // 声明尺寸（types.rs uncompressed_size）
        if zf.enclosed_name().is_none() {
            path_escape += 1; // 路径逃逸名（enclosed_name None = 逃逸）
        }
    }
    let t_scan = t.elapsed();
    // 断言：预扫未读任何条目数据即得声明总量；逃逸名被识别
    assert_eq!(n, 3, "条目数");
    assert!(declared_sum > (2 << 20), "声明求和应含 2MB 大件：{declared_sum}");
    assert_eq!(path_escape, 1, "路径逃逸名恰一处被识别");

    let _ = writeln!(
        report,
        "| 3a 声明预扫 | 3 条目（含 2MB 件与路径逃逸名）| 声明求和 {} + 逃逸名识别 1（零数据读取）| {} |",
        humansize(declared_sum), ms(t_scan)
    );
    Ok(())
}

/// 3b：谎言头炸弹——手构 zip：单条目 DEFLATE，声明 uncompressed=16，
/// 实际流膨胀 1MB。观察 zip 9.x 读行为（with_size_limit 是否按声明截断）。
fn probe3b(report: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    // 原始 deflate 流：1MB 'A' 压到约 1KB
    let payload = vec![b'A'; 1 << 20];
    let mut enc = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&payload)?;
    let deflated = enc.finish()?;
    let crc = crc32(&payload);

    let name = b"bomb.txt";
    let lie_declared: u32 = 16; // 谎言：声明 16B，实际 1MB

    // 手构 zip（本地头 + 数据 + 中央目录 + EOCD）
    let mut z = Vec::new();
    let lh_off = 0u32;
    z.extend_from_slice(b"PK\x03\x04");
    z.extend_from_slice(&20u16.to_le_bytes()); // version needed
    z.extend_from_slice(&0u16.to_le_bytes()); // flags
    z.extend_from_slice(&8u16.to_le_bytes()); // method = deflate
    z.extend_from_slice(&0u16.to_le_bytes()); // time
    z.extend_from_slice(&0u16.to_le_bytes()); // date
    z.extend_from_slice(&crc.to_le_bytes());
    z.extend_from_slice(&(deflated.len() as u32).to_le_bytes()); // compressed（真）
    z.extend_from_slice(&lie_declared.to_le_bytes()); // uncompressed（谎）
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // extra len
    z.extend_from_slice(name);
    let data_off = z.len() as u32;
    z.extend_from_slice(&deflated);
    let cd_off = z.len() as u32;
    z.extend_from_slice(b"PK\x01\x02");
    z.extend_from_slice(&20u16.to_le_bytes()); // version made by
    z.extend_from_slice(&20u16.to_le_bytes()); // version needed
    z.extend_from_slice(&0u16.to_le_bytes()); // flags
    z.extend_from_slice(&8u16.to_le_bytes()); // method
    z.extend_from_slice(&0u16.to_le_bytes()); // time
    z.extend_from_slice(&0u16.to_le_bytes()); // date
    z.extend_from_slice(&crc.to_le_bytes());
    z.extend_from_slice(&(deflated.len() as u32).to_le_bytes());
    z.extend_from_slice(&lie_declared.to_le_bytes());
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // extra
    z.extend_from_slice(&0u16.to_le_bytes()); // comment
    z.extend_from_slice(&0u16.to_le_bytes()); // disk
    z.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
    z.extend_from_slice(&0u32.to_le_bytes()); // external attrs
    z.extend_from_slice(&lh_off.to_le_bytes()); // local header offset
    z.extend_from_slice(name);
    let cd_size = z.len() as u32 - cd_off;
    z.extend_from_slice(b"PK\x05\x06");
    z.extend_from_slice(&0u16.to_le_bytes()); // disk
    z.extend_from_slice(&0u16.to_le_bytes()); // cd disk
    z.extend_from_slice(&1u16.to_le_bytes()); // entries this disk
    z.extend_from_slice(&1u16.to_le_bytes()); // entries total
    z.extend_from_slice(&cd_size.to_le_bytes());
    z.extend_from_slice(&cd_off.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // comment len
    let _ = data_off;

    // 读行为观察：声明 16B、实际膨胀 1MB → ？
    let mut za = zip::ZipArchive::new(std::io::Cursor::new(&z))?;
    let mut zf = za.by_index(0)?;
    let mut out = Vec::new();
    let outcome = match zf.read_to_end(&mut out) {
        Ok(n) => format!("成功（读到 {n} 字节）"),
        Err(e) => format!("错误（{e}）"),
    };
    let behavior = if out.len() as u32 == lie_declared && out.len() < (1 << 20) {
        format!("{outcome} = 按声明截断（读界生效）")
    } else if out.len() >= 1 << 20 {
        format!("{outcome} = 全量膨胀读入（读界未拦）")
    } else {
        outcome
    };

    let _ = writeln!(
        report,
        "| 3b 谎言头 | deflate 声明 16B/实际 1MB | {behavior} | — |"
    );
    Ok(())
}

fn crc32(data: &[u8]) -> u32 {
    // 标准 CRC-32（IEEE 802.3），查表法（探针自含，不引三方件）
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

/// 3c：巨量 dimension 声明——手构最小 xlsx（dimension A1:ZZZ9999999，
/// 约 1.7e12 格声明），观察 calamine 行为。
fn probe3c(report: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let mk = |dimension: &str, path: &str| -> Result<(), Box<dyn std::error::Error>> {
        let f = std::io::BufWriter::new(std::fs::File::create(path)?);
        let mut zw = zip::ZipWriter::new(f);
        let opts: zip::write::SimpleFileOptions = Default::default();
        zw.start_file("[Content_Types].xml", opts)?;
        zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>")?;
        zw.start_file("_rels/.rels", opts)?;
        zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>")?;
        zw.start_file("xl/workbook.xml", opts)?;
        zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"S1\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>")?;
        zw.start_file("xl/_rels/workbook.xml.rels", opts)?;
        zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>")?;
        zw.start_file("xl/worksheets/sheet1.xml", opts)?;
        let sheet = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><dimension ref=\"{dimension}\"/><sheetData><row r=\"1\"><c r=\"A1\" t=\"inlineStr\"><is><t>hello</t></is></c></row><row r=\"2\"><c r=\"B2\"><v>42</v></c></row></sheetData></worksheet>"
        );
        zw.write_all(sheet.as_bytes())?;
        zw.finish()?;
        Ok(())
    };

    // 对照组：正常 dimension
    let normal = format!("{FIX}/dim_normal.xlsx");
    mk("A1:B2", &normal)?;
    let mut book: Xlsx<_> = calamine::open_workbook(&normal)?;
    let rg = book.worksheet_range("S1")?;
    assert_eq!(rg.get_size(), (2, 2), "正常档 Range 尺寸");
    let hello = rg.get((0, 0)).cloned().unwrap_or(Data::Empty);
    assert!(
        matches!(&hello, Data::String(s) if s == "hello"),
        "inlineStr 读回：{hello:?}"
    );

    // 巨量声明组：A1:ZZZ9999999
    let hostile = format!("{FIX}/dim_huge.xlsx");
    mk("A1:ZZZ9999999", &hostile)?;
    let t = Instant::now();
    let behavior = (|| -> String {
        match calamine::open_workbook::<Xlsx<_>, _>(&hostile) {
            Ok(mut b) => match b.worksheet_range("S1") {
                Ok(rg) => {
                    let sz = rg.get_size();
                    format!(
                        "打开+提取成功，Range {} 行×{} 列（{}），单元格值照常可读",
                        sz.0, sz.1, humansize((sz.0 as u64) * (sz.1 as u64) * 8)
                    )
                }
                Err(e) => format!("打开成功但提取错误（{e}）"),
            },
            Err(e) => format!("打开错误（{e}）"),
        }
    })();
    let t_huge = t.elapsed();

    let _ = writeln!(
        report,
        "| 3c 巨量 dimension | 声明 A1:ZZZ9999999（≈1.7e12 格）对照 A1:B2 | {behavior} | {} |",
        ms(t_huge)
    );
    Ok(())
}

fn probe3_hostile(report: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let _ = writeln!(report, "| 探针 3：敌意输入 |\n|---|---|---|---|");
    probe3a(report)?;
    probe3b(report)?;
    probe3c(report)?;
    Ok(())
}
