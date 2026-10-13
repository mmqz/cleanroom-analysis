//! xlsx-r1 t10-c — 已有工作簿修改最小技术验证 + 保真对比（分析仓写面）。
//!
//! 验证目标（xlsx-r1 PLAN §1 C3）：实证"calamine 读 + rust_xlsxwriter
//! 从零新建"组合的语义 = **重建而非修改**——即使用尽力而为路径（值 +
//! 公式文本 + 缓存值 + 合并范围全重放），未触及内容仍丢失哪些维度。
//!
//! 方法（对 dim1..dim5 五个正向语料档逐件执行）：
//!   1. calamine 提取全部可观测面（表序/值矩阵/公式文本/合并范围）；
//!   2. 模拟修改：首表 (8,8) 写入 MODIFIED 标记（各档数据区外统一位）；
//!   3. rust_xlsxwriter 从零新建，重放全部提取面 + 修改；
//!   4. 双重对比：
//!      条目级 —— zip 条目名集合/内容字节对比（新增/丢失/变化三分）；
//!      语义级 —— 值逐格、公式文本、合并范围、样式/关系面可观测性。
//!   5. 输出丢失矩阵（R1 报告事实基座）。
//!
//! 产物：fixtures/fidelity_report.md + stdout 摘要。

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read;

use calamine::{Data, Dimensions, Range, Reader, Xlsx};
use rust_xlsxwriter::Workbook;

const CORPUS: &str = "fixtures/corpus";

/// 从档中提取的可观测面（calamine 能看到的全部）。
struct Extracted {
    sheet_names: Vec<String>,
    /// (sheet, Range<Data>) —— 值矩阵（含缓存值）
    values: Vec<(String, Range<Data>)>,
    /// (sheet, Range<String>) —— 公式文本
    formulas: Vec<(String, Range<String>)>,
    /// (sheet, Vec<Dimensions>) —— 合并范围
    merges: Vec<(String, Vec<Dimensions>)>,
}

fn extract(path: &str) -> Result<Extracted, Box<dyn std::error::Error>> {
    let mut book: Xlsx<_> = calamine::open_workbook(path)?;
    let sheet_names = book.sheet_names().to_vec();
    let mut values = Vec::new();
    let mut formulas = Vec::new();
    let mut merges = Vec::new();
    for name in &sheet_names {
        let v = book.worksheet_range(name)?;
        values.push((name.clone(), v));
        let f = book.worksheet_formula(name)?;
        formulas.push((name.clone(), f));
        let m = book.merge_cells_by_sheet_name(name)?;
        merges.push((name.clone(), m));
    }
    Ok(Extracted { sheet_names, values, formulas, merges })
}

/// 尽力而为重建：重放 calamine 全部可观测面 + 首表 (8,8) 修改标记。
fn rebuild_best_effort(ex: &Extracted) -> Result<Workbook, Box<dyn std::error::Error>> {
    let mut wb = Workbook::new();
    // 公式索引：sheet -> (绝对 row, col) -> 公式文本
    let mut formula_idx: BTreeMap<String, BTreeMap<(usize, usize), String>> = BTreeMap::new();
    for (name, fr) in &ex.formulas {
        let start = fr.start().unwrap_or((0, 0));
        let mut m = BTreeMap::new();
        for (r, row) in fr.rows().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if !cell.is_empty() {
                    let abs = (start.0 as usize + r, start.1 as usize + c);
                    m.insert(abs, cell.clone());
                }
            }
        }
        formula_idx.insert(name.clone(), m);
    }
    // 合并索引：sheet -> Vec<Dimensions>
    let merge_idx: BTreeMap<String, Vec<Dimensions>> =
        ex.merges.iter().cloned().collect();

    for (name, vr) in &ex.values {
        let ws = wb.add_worksheet();
        ws.set_name(name)?;
        let start = vr.start().unwrap_or((0, 0));
        let fidx = formula_idx.get(name).cloned().unwrap_or_default();
        let mut merged_cells: BTreeMap<(usize, usize), ()> = BTreeMap::new();
        if let Some(ms) = merge_idx.get(name) {
            for d in ms {
                for r in d.start.0..=d.end.0 {
                    for c in d.start.1..=d.end.1 {
                        merged_cells.insert((r as usize, c as usize), ());
                    }
                }
            }
        }
        for (r, row) in vr.rows().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let abs = (start.0 as usize + r, start.1 as usize + c);
                if cell == &Data::Empty {
                    continue; // 空格不写（合并区非左上格也在此列——左上格有值）
                }
                let (ar, ac) = (abs.0 as u32, abs.1 as u16);
                if let Some(f) = fidx.get(&abs) {
                    // 公式格：重放公式文本（calamine 无 = 前缀 → 还原）
                    let formula = if f.starts_with('=') { f.clone() } else { format!("={f}") };
                    ws.write_formula(ar, ac, formula.as_str())?;
                    // 缓存值重放（若值面非空且非 0——0 可能是真值也可能是无缓存值，
                    // 无法区分：两形态都重放为 "0" 与原行为一致）
                    if let Some(cache) = data_to_formula_result(cell) {
                        ws.set_formula_result(ar, ac, &cache);
                    }
                } else {
                    match cell {
                        Data::String(s) => {
                            // 合并区左上格且属于合并范围 → 用 merge_range 重放
                            // （简化：合并统一在值重放后按范围重放，见下方；
                            //  此处先写值，合并范围左上格已在 merged_cells 集内）
                            let _ = merged_cells;
                            ws.write_string(ar, ac, s)?;
                        }
                        Data::Float(v) => {
                            ws.write_number(ar, ac, *v)?;
                        }
                        Data::Int(v) => {
                            ws.write_number(ar, ac, *v as f64)?;
                        }
                        Data::Bool(b) => {
                            ws.write(ar, ac, *b)?;
                        }
                        Data::DateTime(dt) => {
                            // 尽力而为：按浮点序列值重放（日期面本身就是丢失维度候选）
                            ws.write_number(ar, ac, dt.as_f64())?;
                        }
                        Data::DateTimeIso(s) => {
                            ws.write_string(ar, ac, s.to_string())?;
                        }
                        Data::DurationIso(s) => {
                            ws.write_string(ar, ac, s.to_string())?;
                        }
                        Data::Error(e) => {
                            // 错误值按字符串形态重放（#DIV/0! 等）
                            ws.write_string(ar, ac, format!("{e:?}"))?;
                        }
                        Data::Empty => unreachable!(),
                    }
                }
            }
        }
        // 合并范围重放（merge_range 需要值——用提取的左上值或空串）
        if let Some(ms) = merge_idx.get(name) {
            for d in ms {
                let (r0, c0) = (d.start.0, d.start.1);
                let (r1, c1) = (d.end.0, d.end.1);
                let val = vr
                    .get(((r0 - start.0) as usize, (c0 - start.1) as usize))
                    .cloned()
                    .unwrap_or(Data::Empty);
                let text = match val {
                    Data::String(s) => s,
                    Data::Float(v) => format!("{v}"),
                    Data::Int(v) => format!("{v}"),
                    Data::Bool(b) => format!("{b}"),
                    _ => String::new(),
                };
                let fmt = rust_xlsxwriter::Format::new();
                ws.merge_range(r0, c0 as u16, r1, c1 as u16, &text, &fmt)?;
            }
        }
        // 修改标记：首表 (8,8) —— 数据区外统一位
        if name == &ex.sheet_names[0] {
            ws.write_string(8, 8, "R1-MODIFIED")?;
        }
    }
    Ok(wb)
}

fn data_to_formula_result(d: &Data) -> Option<String> {
    match d {
        Data::Float(v) => Some(format!("{v}")),
        Data::Int(v) => Some(format!("{v}")),
        Data::Bool(b) => Some(format!("{b}")),
        Data::String(s) => Some(s.clone()),
        _ => None, // Empty = 无缓存值；其他形态尽力而为跳过
    }
}

/// zip 条目快照：名字 -> 内容字节。
fn zip_entries(path: &str) -> Result<BTreeMap<String, Vec<u8>>, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let mut za = zip::ZipArchive::new(f)?;
    let mut m = BTreeMap::new();
    for i in 0..za.len() {
        let mut zf = za.by_index(i)?;
        // zip 8.6：name() 直接返回 &str（9.x 为 Result<Cow<str>>——
        // 对齐 8.6 后简化；条目名对比语义不变）。
        let name = zf.name().to_string();
        let mut buf = Vec::new();
        zf.read_to_end(&mut buf)?;
        m.insert(name, buf);
    }
    Ok(m)
}

/// 条目级对比：三分（仅新增/仅丢失/内容变化/相同）。
fn compare_entries(
    a: &BTreeMap<String, Vec<u8>>,
    b: &BTreeMap<String, Vec<u8>>,
) -> (Vec<String>, Vec<String>, Vec<String>, usize) {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut same = 0usize;
    for (name, data) in a {
        match b.get(name) {
            None => removed.push(name.clone()),
            Some(d) if d == data => same += 1,
            Some(_) => changed.push(name.clone()),
        }
    }
    for name in b.keys() {
        if !a.contains_key(name) {
            added.push(name.clone());
        }
    }
    (added, removed, changed, same)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dims = [
        "dim1_formulas.xlsx",
        "dim2_styles.xlsx",
        "dim3_merged.xlsx",
        "dim4_multisheet.xlsx",
        "dim5_relationships.xlsx",
    ];
    let mut report = String::new();
    let _ = writeln!(
        report,
        "# 已有工作簿修改最小技术验证（t10-c）\n\n\
         路径：calamine 全量提取（值/公式/合并 = 全部可观测面）→ 首表(8,8)写\n\
         R1-MODIFIED → rust_xlsxwriter 从零新建重放 → 与原档双重对比。\n\
         语义 = 「尽力而为重建」——此路径已是该组合的理论上限。\n"
    );

    for dim in dims {
        let src = format!("{CORPUS}/{dim}");
        let dst = format!("fixtures/fidelity/{dim}");
        std::fs::create_dir_all("fixtures/fidelity")?;
        let _ = writeln!(report, "\n## {dim}\n");

        // 1. 提取 + 2. 重建（含修改）
        let ex = extract(&src)?;
        let mut wb = rebuild_best_effort(&ex)?;
        wb.save(&dst)?;

        // 3a. 条目级对比
        let a = zip_entries(&src)?;
        let b = zip_entries(&dst)?;
        let (added, removed, changed, same) = compare_entries(&a, &b);
        let _ = writeln!(
            report,
            "### 条目级（原 {} 条目 → 新 {} 条目）\n",
            a.len(),
            b.len()
        );
        let _ = writeln!(
            report,
            "- 相同：{same}；新增：{}；丢失：{}；变化：{}",
            added.len(),
            removed.len(),
            changed.len()
        );
        if !removed.is_empty() {
            let _ = writeln!(report, "- 丢失条目：{}", removed.join("、"));
        }
        if !added.is_empty() {
            let _ = writeln!(report, "- 新增条目：{}", added.join("、"));
        }
        if !changed.is_empty() {
            let _ = writeln!(report, "- 内容变化条目：{}", changed.join("、"));
        }

        // 3b. 语义级对比（重开两档逐面对比）
        let ex2 = extract(&dst)?;
        // 表序
        let _ = writeln!(
            report,
            "- 表序：{}",
            if ex.sheet_names == ex2.sheet_names { "保" } else { "变" },
        );
        // 值逐格
        let mut value_diffs = 0usize;
        let mut value_cells = 0usize;
        for (i, (name, vr)) in ex.values.iter().enumerate() {
            let vr2 = &ex2.values[i].1;
            let (r0, c0) = vr.start().unwrap_or((0, 0));
            let (r1, c1) = vr2.start().unwrap_or((0, 0));
            for (r, row) in vr.rows().enumerate() {
                for (c, cell) in row.iter().enumerate() {
                    if cell == &Data::Empty {
                        continue;
                    }
                    value_cells += 1;
                    // 绝对坐标对齐
                    let abs = (r0 as usize + r, c0 as usize + c);
                    let rel = (abs.0 - r1 as usize, abs.1 - c1 as usize);
                    let other = vr2.get(rel).cloned().unwrap_or(Data::Empty);
                    if !data_eq(cell, &other) {
                        value_diffs += 1;
                    }
                }
            }
            let _ = name;
        }
        let _ = writeln!(
            report,
            "- 值面：{value_cells} 非空格中 {value_diffs} 格差异{}",
            if value_diffs == 0 { "（保）" } else { "" }
        );
        // 公式
        let mut f_total = 0usize;
        let mut f_lost = 0usize;
        for (i, (name, fr)) in ex.formulas.iter().enumerate() {
            let fr2 = &ex2.formulas[i].1;
            let (r0, c0) = fr.start().unwrap_or((0, 0));
            let (r1, c1) = fr2.start().unwrap_or((0, 0));
            for (r, row) in fr.rows().enumerate() {
                for (c, cell) in row.iter().enumerate() {
                    if cell.is_empty() {
                        continue;
                    }
                    f_total += 1;
                    let abs = (r0 as usize + r, c0 as usize + c);
                    let rel = (abs.0 - r1 as usize, abs.1 - c1 as usize);
                    let other = fr2.get(rel).cloned().unwrap_or_default();
                    // _xlfn 前缀差异容忍（写读同前缀即保）
                    if !cell.eq(&other) {
                        f_lost += 1;
                    }
                }
            }
            let _ = name;
        }
        let _ = writeln!(
            report,
            "- 公式面：{f_total} 条中 {f_lost} 条文本差异{}",
            if f_lost == 0 && f_total > 0 {
                "（文本保）"
            } else if f_total == 0 {
                "（本档无公式）"
            } else {
                ""
            }
        );
        // 合并
        let m_total: usize = ex.merges.iter().map(|(_, v)| v.len()).sum();
        let m_kept: usize = ex2.merges.iter().map(|(_, v)| v.len()).sum();
        let _ = writeln!(
            report,
            "- 合并面：原 {m_total} 范围 → 重放后 {m_kept}{}",
            if m_total == m_kept { "（保）" } else { "（差异）" }
        );
        // 修改标记在位验证
        let first2 = &ex2.values[0];
        let mod_cell = first2.1.get((8, 8)).cloned().unwrap_or(Data::Empty);
        let _ = writeln!(
            report,
            "- 修改格(8,8)：{}",
            if matches!(mod_cell, Data::String(s) if s == "R1-MODIFIED") {
                "在位 ✓"
            } else {
                "丢失 ✗"
            }
        );
    }

    std::fs::write("fixtures/fidelity_report.md", &report)?;
    println!("{report}");
    println!("ok -> fixtures/fidelity_report.md");
    Ok(())
}

fn data_eq(a: &Data, b: &Data) -> bool {
    match (a, b) {
        (Data::Empty, Data::Empty) => true,
        (Data::String(x), Data::String(y)) => x == y,
        (Data::Float(x), Data::Float(y)) => x == y,
        (Data::Float(x), Data::Int(y)) => *x == *y as f64,
        (Data::Int(x), Data::Float(y)) => *x as f64 == *y,
        (Data::Int(x), Data::Int(y)) => x == y,
        (Data::Bool(x), Data::Bool(y)) => x == y,
        (Data::Error(x), Data::Error(y)) => x == y,
        _ => false,
    }
}
