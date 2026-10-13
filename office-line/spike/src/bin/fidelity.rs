//! xlsx-r1 t10-c v2 — 已有工作簿修改最小技术验证 + 保真对比（分析仓写面）。
//!
//! 验证目标（xlsx-r1 PLAN §1 C3）：实证"calamine 读 + rust_xlsxwriter
//! 从零新建"组合的语义 = **重建而非修改**——即使用尽力而为路径（值 +
//! 公式文本 + 缓存值 + 合并范围 + 超链接全重放），未触及内容仍丢失哪些维度。
//!
//! v2 勘误（2026-10-13）：v1 曾按「calamine 无超链接 API」登记超链接为
//! 预期丢失维度——**误判**。calamine 0.36.1 hyperlinks_by_sheet_name
//! 完整可观测（target/location/displayed_text/tooltip/range 五字段）。
//! v2 补超链接面提取与重放，语料 dim5 同步扩展（tooltip + file 子锚）。
//!
//! 方法（对 dim1..dim5 五个正向语料档逐件执行）：
//!   1. calamine 提取全部可观测面（表序/值矩阵/公式文本/合并/超链接）；
//!   2. 模拟修改：首表 (8,8) 写入 MODIFIED 标记（各档数据区外统一位）；
//!   3. rust_xlsxwriter 从零新建，重放全部提取面 + 修改；
//!      超链接回放规则 = 值面驱动：锚点值面==回放 URL → 纯链接形态；
//!      否则 write_url_with_text（自定义文本）；tooltip 透传。
//!   4. 双重对比：
//!      条目级 —— zip 条目名集合/内容字节对比（新增/丢失/变化三分）；
//!      语义级 —— 值逐格、公式文本、合并范围、超链接逐字段、样式/关系面可观测性。
//!   5. 输出丢失矩阵（R1 报告事实基座）。
//!
//! 产物：fixtures/fidelity_report.md + stdout 摘要。

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read;

use calamine::{Data, Dimensions, Hyperlink, Range, Reader, Xlsx};
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
    /// (sheet, Vec<Hyperlink>) —— 超链接（v2）
    hyperlinks: Vec<(String, Vec<Hyperlink>)>,
}

fn extract(path: &str) -> Result<Extracted, Box<dyn std::error::Error>> {
    let mut book: Xlsx<_> = calamine::open_workbook(path)?;
    let sheet_names = book.sheet_names().to_vec();
    let mut values = Vec::new();
    let mut formulas = Vec::new();
    let mut merges = Vec::new();
    let mut hyperlinks = Vec::new();
    for name in &sheet_names {
        let v = book.worksheet_range(name)?;
        values.push((name.clone(), v));
        let f = book.worksheet_formula(name)?;
        formulas.push((name.clone(), f));
        let m = book.merge_cells_by_sheet_name(name)?;
        merges.push((name.clone(), m));
        let h = book.hyperlinks_by_sheet_name(name)?;
        hyperlinks.push((name.clone(), h));
    }
    Ok(Extracted { sheet_names, values, formulas, merges, hyperlinks })
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
    // 超链接索引（v2）：sheet -> Vec<Hyperlink>
    let hyperlink_idx: BTreeMap<String, Vec<Hyperlink>> =
        ex.hyperlinks.iter().cloned().collect();

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
        // 超链接重放（v2）：值面驱动形态判定 + tooltip 透传
        if let Some(hls) = hyperlink_idx.get(name) {
            for hl in hls {
                let (hr, hc) = (hl.range.start.0, hl.range.start.1 as u16);
                let url: String = match (&hl.target, &hl.location) {
                    (Some(t), Some(loc)) => format!("{}#{}", xml_unescape5(t), xml_unescape5(loc)), // 外部文件 + 子锚（file#anchor）
                    (Some(t), None) => xml_unescape5(t),
                    (None, Some(loc)) => format!("internal:{}", xml_unescape5(loc)),
                    (None, None) => continue, // 无目标无位置：不可达形态，跳过
                };
                let val = vr
                    .get(((hl.range.start.0 - start.0) as usize, (hl.range.start.1 - start.1) as usize))
                    .cloned()
                    .unwrap_or(Data::Empty);
                let val_str = match &val {
                    Data::String(s) => Some(s.clone()),
                    _ => None,
                };
                let tip = hl.tooltip.clone().unwrap_or_default();
                if let Some(text) = val_str.filter(|t| t != &url) {
                    // 锚点值面 != URL → 自定义文本形态（write_url_with_text 语义）
                    ws.write_url_with_options(hr, hc, url.as_str(), text.as_str(), tip.as_str(), None)?;
                } else {
                    // 值面==URL（或非字符串）：纯链接形态
                    ws.write_url_with_options(hr, hc, url.as_str(), url.as_str(), tip.as_str(), None)?;
                }
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
        let name = zf.name().to_string(); // zip 8.6：name() 返回 &str（9.x 为 Result——API 差异三处校准之一）
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
        // 超链接（v2）：按锚点对齐逐字段对比
        {
            let hl_of = |ex: &Extracted| -> BTreeMap<(u32, u32), Hyperlink> {
                let mut m = BTreeMap::new();
                for (_, hls) in &ex.hyperlinks {
                    for h in hls {
                        m.insert(h.range.start, h.clone());
                    }
                }
                m
            };
            let a_map = hl_of(&ex);
            let b_map = hl_of(&ex2);
            let h_total = a_map.len();
            let mut field_diffs = 0usize;
            let mut details: Vec<String> = Vec::new();
            for (anchor, oh) in &a_map {
                match b_map.get(anchor) {
                    None => {
                        field_diffs += 1;
                        details.push(format!("锚点 {anchor:?} 丢失"));
                    }
                    Some(nh) => {
                        let tgt = oh.target.clone().unwrap_or_default()
                            == nh.target.clone().unwrap_or_default();
                        let loc = oh.location.clone().unwrap_or_default()
                            == nh.location.clone().unwrap_or_default();
                        // displayed_text 归一：None ≡ ""（rust_xlsxwriter 内链 display 必发、
                        // 外链不发——形态等价比较）
                        let disp = oh.displayed_text.clone().unwrap_or_default()
                            == nh.displayed_text.clone().unwrap_or_default();
                        let tip = oh.tooltip.clone().unwrap_or_default()
                            == nh.tooltip.clone().unwrap_or_default();
                        if !(tgt && loc && disp && tip) {
                            field_diffs += 1;
                            details.push(format!(
                                "锚点 {anchor:?}：target{}/location{}/display{}/tooltip{}（原 target={:?} 新 target={:?}）",
                                yn(tgt),
                                yn(loc),
                                yn(disp),
                                yn(tip),
                                oh.target,
                                nh.target
                            ));
                        }
                    }
                }
            }
            if b_map.len() > a_map.len() {
                details.push(format!("重建多出 {} 条（幽灵链接）", b_map.len() - a_map.len()));
            }
            let _ = writeln!(
                report,
                "- 超链接面：原 {h_total} 条 → 重放后 {} 条，字段差异 {field_diffs}{}{}",
                b_map.len(),
                if field_diffs == 0 && h_total > 0 {
                    "（保）"
                } else if h_total == 0 {
                    "（本档无超链接）"
                } else {
                    ""
                },
                if details.is_empty() { String::new() } else { format!("——{}", details.join("；")) }
            );
        }
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

fn yn(b: bool) -> &'static str {
    if b { "✓" } else { "✗" }
}

/// calamine 0.36.1 读侧怪癖补偿：hyperlink rels Target（及 location）
/// 属性值未做 XML 实体反转义（utils.rs:117 unescape_xml 只处理 _x00HH_
/// 约定，非该前缀原样返回）——`&` 以 `&amp;` 字面量读出。回放前把
/// 5 实体还原为逻辑字符，重建字节面与原档一致。注意 &amp; 必须最后
/// 替换（&amp;amp; 正确解为字面 "&amp;"）。登记边界：真含字面
/// "&amp;" 文本的 URL 在此补偿下有歧义（双解不可分，实际概率≈0）。
fn xml_unescape5(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn data_eq(a: &Data, b: &Data) -> bool {
    match (a, b) {
        (Data::Empty, Data::Empty) => true,
        (Data::String(x), Data::String(y)) => x == y,
        (Data::Float(x), Data::Float(y)) => x == y,
        (Data::Float(x), Data::Int(y)) => *x == (*y as f64),
        (Data::Int(x), Data::Float(y)) => (*x as f64) == *y,
        (Data::Int(x), Data::Int(y)) => x == y,
        (Data::Bool(x), Data::Bool(y)) => x == y,
        (Data::Error(x), Data::Error(y)) => x == y,
        _ => false,
    }
}
