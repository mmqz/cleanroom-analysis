//! xlsx-r1 补充探针——C2 样式写入 XML 级验证（CAPABILITIES C2 边界
//! 缺口 #2 闭项）。
//!
//! 背景：dim2_styles.xlsx 由 rust_xlsxwriter 生成（bold/红字/黄底/
//! 细边框/¥ 货币/百分比），calamine 读侧无样式可观测面（FIDELITY
//! §4），此前仅值面验证。本探针直查生成档的 xl/styles.xml +
//! sheet1.xml cellXfs，逐属性断言样式写入正确性——验证器 = zip 8.6
//! + 字符串级 XML 检查（不引第三方 XML 解析，净室纪律）。

use std::io::Read;

fn read_entry(path: &str, want: &str) -> Result<String, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let mut za = zip::ZipArchive::new(f)?;
    for i in 0..za.len() {
        let mut zf = za.by_index(i)?;
        if zf.name() == want {
            let mut s = String::new();
            zf.read_to_string(&mut s)?;
            return Ok(s);
        }
    }
    Err(format!("entry not found: {want}").into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = "fixtures/corpus/dim2_styles.xlsx";
    let styles = read_entry(p, "xl/styles.xml")?;
    let sheet = read_entry(p, "xl/worksheets/sheet1.xml")?;

    let mut checks: Vec<(&str, bool)> = Vec::new();

    // 1. 字体面：bold + 红字（FF0000）
    checks.push((
        "font bold (b/) present",
        styles.contains("<b/>") || styles.contains("<b "),
    ));
    checks.push(("font color FF0000", styles.contains("rgb=\"FFFF0000\"")));

    // 2. 填充面：黄底（FFFF00）
    checks.push(("fill FFFF00", styles.contains("rgb=\"FFFFFF00\"")));

    // 3. 边框面：细边框（thin）
    checks.push(("border thin style", styles.contains("style=\"thin\"")));

    // 4. 数字格式面：¥ 货币（用户格式 numFmtId ≥ 164 自定义区）
    checks.push((
        "custom numFmt (currency/pct in numFmts)",
        styles.contains("<numFmts") && styles.contains("numFmtId=\"164\""),
    ));
    // ¥ 符号在格式码内
    checks.push((
        "currency code contains ¥",
        styles.contains("¥") || styles.contains("&quot;¥") || styles.contains("&#165;"),
    ));
    // 百分比格式码
    checks.push(("percent format 0.0%", styles.contains("0.0%")));

    // 5. 应用面：cellXfs 非空且 sheet 引用样式（s= 属性）
    checks.push((
        "cellXfs has xf entries",
        styles.contains("<cellXfs") && styles.matches("<xf ").count() >= 2,
    ));
    checks.push(("sheet cells reference style (s=)", sheet.contains("s=\"")));

    let mut pass = 0;
    for (name, ok) in &checks {
        println!("[{}] {name}", if *ok { "PASS" } else { "FAIL" });
        if *ok {
            pass += 1;
        }
    }
    println!("style probe: {pass}/{}", checks.len());
    if pass != checks.len() {
        std::process::exit(1);
    }
    Ok(())
}
