//! xlsx-r1 t10-d — XLSX 读骨架原型（mdcore IO 模块直移形态）。
//!
//! 三段结构（xlsx-r1 PLAN §2 t10-d 规格）：
//!   1. 容器预扫门（SECURITY_POLICY §2.1 四检查 + §3 限额值接线）：
//!      条目数上限 / 单条目声明尺寸 / 声明求和 / 路径逃逸——typed reject，
//!      零条目数据读取（zip 中央目录声明值直读）。
//!   2. calamine 读取包装：表名单 / 值矩阵 / 公式文本 / 合并范围四 API 面，
//!      统一提取结构（直移 mdcore 的领域形状）。
//!   3. 统一错误面：拒绝（门）/ 容器损坏 / 解析错误三分 + Ok。
//!
//! 本文件同时是验证 harness：17 件语料（t10-b 全集）逐件过 read_book，
//! 断言三分归类与既有读侧行为锚点一致，**零 panic**（任何 panic = harness
//! 崩溃即失败信号）。敌意件的实际归类为**实证钉定**（版本锁定
//! calamine 0.36.1 / zip 9.0.2；升版须重跑重钉）。

use std::collections::BTreeMap;
use std::fmt::Write as _;

use calamine::{Data, Dimensions, Range, Reader, Xlsx};

// ---------------------------------------------------------------------------
// §3 限额常量（SECURITY_POLICY v1 接线——xlsx 族）
// ---------------------------------------------------------------------------

/// zip 声明总量上限（xlsx 族 300MB；office-b default.json 一手参照）。
pub const LIMIT_TOTAL_DECLARED: u64 = 300 * 1024 * 1024;
/// 单条目声明尺寸上限（= 族总量上限，对称设计）。
pub const LIMIT_SINGLE_ENTRY: u64 = LIMIT_TOTAL_DECLARED;
/// 条目数上限（v1 新设 10,000；本 harness 对 mal6d 实测校准）。
pub const LIMIT_ENTRY_COUNT: usize = 10_000;

// ---------------------------------------------------------------------------
// 统一错误面（读路径公共类型——mdcore 直移形状）
// ---------------------------------------------------------------------------

/// 读路径统一错误。三分与 harness 断言面一一对应：
/// 拒绝（预扫门，零数据读取）/ 容器损坏 / 解析错误（容器过门后）。
#[derive(Debug)]
pub enum ReadError {
    /// 预扫门拒绝（typed reject）——敌意/超额输入未进解析层。
    PrescanRejected {
        reason: &'static str,
        entry: Option<String>,
    },
    /// zip 容器结构损坏（EOCD/中央目录不可读）。
    ContainerCorrupt(String),
    /// 容器过门后的解析层错误（calamine / zip 读界）。
    ParseError(String),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::PrescanRejected { reason, entry } => {
                write!(f, "prescan rejected: {reason}")?;
                if let Some(e) = entry {
                    write!(f, " (entry: {e})")?;
                }
                Ok(())
            }
            ReadError::ContainerCorrupt(m) => write!(f, "container corrupt: {m}"),
            ReadError::ParseError(m) => write!(f, "parse error: {m}"),
        }
    }
}

impl std::error::Error for ReadError {}

// ---------------------------------------------------------------------------
// 1. 容器预扫门
// ---------------------------------------------------------------------------

/// 预扫报告（过门后的正常路径附带，审计/日志面）。
#[derive(Debug)]
pub struct PrescanReport {
    pub entries: usize,
    pub total_declared: u64,
    pub max_single_declared: u64,
}

/// 容器预扫：四检查（§2.1），全部基于 zip 中央目录声明值，零条目数据读取。
///
/// 检查序 = 拒绝最快路径优先：条目数（O(1)，len() 直读）→ 逐条目
/// 单尺寸/路径逃逸（单趟）→ 求和总量（同趟累计）。
pub fn prescan(path: &str) -> Result<PrescanReport, ReadError> {
    let f = std::fs::File::open(path)
        .map_err(|e| ReadError::ContainerCorrupt(format!("open: {e}")))?;
    let mut za = zip::ZipArchive::new(f)
        .map_err(|e| ReadError::ContainerCorrupt(format!("{e}")))?;

    if za.len() > LIMIT_ENTRY_COUNT {
        return Err(ReadError::PrescanRejected {
            reason: "entry count over limit",
            entry: None,
        });
    }

    let mut total: u64 = 0;
    let mut max_single: u64 = 0;
    for i in 0..za.len() {
        let zf = za
            .by_index(i)
            .map_err(|e| ReadError::ContainerCorrupt(format!("{e}")))?;
        let declared = zf.size(); // 声明 uncompressed_size（types.rs pub 暴露）
        if declared > LIMIT_SINGLE_ENTRY {
            return Err(ReadError::PrescanRejected {
                reason: "single entry declared size over limit",
                entry: Some(zf.name().unwrap_or_default().to_string()),
            });
        }
        // 路径逃逸：enclosed_name None = 绝对路径/../逃逸/盘符位
        if zf.enclosed_name().is_none() {
            return Err(ReadError::PrescanRejected {
                reason: "path escape entry",
                entry: Some(zf.name().unwrap_or_default().to_string()),
            });
        }
        max_single = max_single.max(declared);
        total = total.saturating_add(declared);
    }
    if total > LIMIT_TOTAL_DECLARED {
        return Err(ReadError::PrescanRejected {
            reason: "total declared size over limit",
            entry: None,
        });
    }
    Ok(PrescanReport {
        entries: za.len(),
        total_declared: total,
        max_single_declared: max_single,
    })
}

// ---------------------------------------------------------------------------
// 2. calamine 读取包装（四 API 面 → 统一提取结构）
// ---------------------------------------------------------------------------

/// 单表快照：calamine 公开 API 的全部可观测面（值/公式/合并）。
#[derive(Debug, Default)]
pub struct SheetData {
    pub name: String,
    /// 值矩阵（含公式缓存值）。
    pub values: Range<Data>,
    /// 公式文本（calamine 形态：无 = 前缀）。
    pub formulas: Range<String>,
    /// 合并范围。
    pub merges: Vec<Dimensions>,
}

/// 整档提取：表序 + 逐表三面（C1 读取能力的产品面形状）。
#[derive(Debug)]
pub struct Book {
    pub sheets: Vec<SheetData>,
}

impl Book {
    pub fn sheet_names(&self) -> Vec<&str> {
        self.sheets.iter().map(|s| s.name.as_str()).collect()
    }

    /// 非空格计数（harness 摘要用）。
    pub fn non_empty_cells(&self) -> usize {
        self.sheets
            .iter()
            .map(|s| {
                s.values
                    .rows()
                    .map(|r| r.iter().filter(|c| !matches!(c, Data::Empty)).count())
                    .sum::<usize>()
            })
            .sum()
    }
}

/// 读入口：预扫门 → calamine 全量提取。
///
/// 门在前（敌意输入零解析层接触）；门过后的错误全部归 ParseError
/// （含 zip 读界：实际膨胀超声明的谎言头由 zip 9.x with_size_limit
/// 内建拒绝——T3 3b 实测，策略层不重复实现）。
pub fn read_book(path: &str) -> Result<Book, ReadError> {
    let _report = prescan(path)?; // 门：拒绝即出，不进解析层

    let mut book: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| ReadError::ParseError(format!("open: {e}")))?;
    let names = book
        .sheet_names()
        .to_vec();
    let mut sheets = Vec::with_capacity(names.len());
    for name in &names {
        let values = book
            .worksheet_range(name)
            .map_err(|e| ReadError::ParseError(format!("range[{name}]: {e}")))?;
        let formulas = book
            .worksheet_formula(name)
            .map_err(|e| ReadError::ParseError(format!("formula[{name}]: {e}")))?;
        let merges = book
            .merge_cells_by_sheet_name(name)
            .map_err(|e| ReadError::ParseError(format!("merge[{name}]: {e}")))?;
        sheets.push(SheetData {
            name: name.clone(),
            values,
            formulas,
            merges,
        });
    }
    Ok(Book { sheets })
}

// ---------------------------------------------------------------------------
// 3. harness：17 件语料三分断言（零 panic）
// ---------------------------------------------------------------------------

#[derive(PartialEq, Debug, Clone, Copy)]
enum Class {
    Ok,
    Reject,
    Err,
}

impl Class {
    fn tag(&self) -> &'static str {
        match self {
            Class::Ok => "OK",
            Class::Reject => "REJECT",
            Class::Err => "ERR",
        }
    }
}

/// 实测归类函数（三分映射：ReadError::PrescanRejected → REJECT，
/// 其余 Err → ERR，Ok → OK）。
fn classify(r: &Result<Book, ReadError>) -> Class {
    match r {
        Ok(_) => Class::Ok,
        Err(ReadError::PrescanRejected { .. }) => Class::Reject,
        Err(_) => Class::Err,
    }
}

fn main() {
    let corpus = "fixtures/corpus";
    // 期望表：三分 + 正向件附加锚点断言（与 manifest 校验一致）。
    // mal7* 五件 = 实证钉定（2026-10-13，calamine 0.36.1 / zip 9.0.2）：
    // 初跑观察 → 按实测钉入本表；升版须重跑重钉（文件头注纪律）。
    let cases: &[(&str, Class, fn(&Book) -> bool)] = &[
        ("ok7_control_normal.xlsx", Class::Ok, |b| {
            b.sheets.len() == 1 && b.non_empty_cells() > 0
        }),
        ("dim1_formulas.xlsx", Class::Ok, |b| {
            b.sheets.len() == 1
                && b.sheets[0]
                    .formulas
                    .rows()
                    .map(|r| r.iter().filter(|c| !c.is_empty()).count())
                    .sum::<usize>()
                    == 4
        }),
        ("dim2_styles.xlsx", Class::Ok, |b| b.non_empty_cells() == 6),
        ("dim3_merged.xlsx", Class::Ok, |b| {
            b.sheets[0].merges.len() == 3 && b.non_empty_cells() == 4
        }),
        ("dim4_multisheet.xlsx", Class::Ok, |b| {
            b.sheet_names() == vec!["Data", "RefSheet", "Calc", "TailEmpty"]
        }),
        ("dim5_relationships.xlsx", Class::Ok, |b| b.non_empty_cells() == 5),
        // 敌意 ⑥：容器门四检查逐件对位
        ("mal6a_lie_header.zip", Class::Err, |_| true), // 声明 16B 过门；zip 读界内建拒（T1 层）
        ("mal6b_path_escape.zip", Class::Reject, |_| true), // enclosed_name None
        ("mal6c_declared_huge.zip", Class::Reject, |_| true), // 320MB > 300MB
        ("mal6d_too_many_entries.zip", Class::Reject, |_| true), // 10,001 > 10,000
        ("mal6e_eocd_corrupt.zip", Class::Err, |_| true), // EOCD 损坏 = 容器损坏
        ("mal6f_no_content_types.zip", Class::Err, |_| true), // zip 合法，xlsx 语义损坏
        // 敌意 ⑦：异常 XML 五件（实证钉定，见上注）
        // 7a 截断 / 7b 非法实体 / 7d 编码错 → 解析层拒绝（ERR）
        ("mal7a_xml_truncated.xlsx", Class::Err, |_| true),
        ("mal7b_bad_entity.xlsx", Class::Err, |_| true),
        // 7c 实证发现（钉定 2026-10-13）：quick-xml 文本面容忍 XML 1.0
        // 非法的 0x00——读回值含 NUL 字节且非空。安全含义：读侧不过滤，
        // 再输出必须过 mdc-office xml_control（消毒链载荷锚点）。
        ("mal7c_control_char.xlsx", Class::Ok, |b| {
            matches!(b.sheets[0].values.get((0, 0)), Some(Data::String(s)) if s.contains('\u{0}'))
        }),
        ("mal7d_bad_encoding.xlsx", Class::Err, |_| true),
        // 7e 实证发现（钉定 2026-10-13）：二进制垃圾部件被容忍，提取为
        // 1 表 0 格（无崩溃无错误）。安全含义：损坏输入静默降级为空结果，
        // 无信号——登记（严格模式属后续裁决，非本轮）。
        ("mal7e_not_xml.xlsx", Class::Ok, |b| {
            b.sheets.len() == 1 && b.non_empty_cells() == 0
        }),
    ];

    let mut table = String::new();
    let mut pass = 0usize;
    let mut fail: Vec<String> = Vec::new();
    let _ = writeln!(table, "| 语料 | 期望 | 实测 | 锚点 | 判定 |");
    let _ = writeln!(table, "|------|------|------|------|------|");

    for (file, expected, anchor) in cases {
        let r = read_book(&format!("{corpus}/{file}"));
        let actual = classify(&r);
        // 锚点断言仅对 Ok 件生效
        let anchor_ok = if actual == Class::Ok {
            match &r {
                Ok(b) => anchor(b),
                Err(_) => unreachable!(),
            }
        } else {
            true
        };
        let verdict = if actual == *expected && anchor_ok {
            pass += 1;
            "PASS"
        } else {
            let detail = match &r {
                Ok(_) => "unexpected Ok".to_string(),
                Err(e) => format!("{e}"),
            };
            fail.push(format!("{file}: expected {:?} got {:?} [{detail}]", expected, actual));
            "FAIL"
        };
        let detail = match &r {
            Ok(b) => format!("{} sheets / {} cells", b.sheets.len(), b.non_empty_cells()),
            Err(e) => {
                // 截断长错误串入表
                let s = e.to_string();
                let s = if s.chars().count() > 60 {
                    let t: String = s.chars().take(60).collect();
                    format!("{t}...")
                } else {
                    s
                };
                s
            }
        };
        let _ = writeln!(
            table,
            "| {file} | {} | {} | {detail} | {verdict} |",
            expected.tag(),
            actual.tag()
        );
    }

    println!("{table}");
    println!("pass {pass}/{total}", total = cases.len());
    if !fail.is_empty() {
        for f in &fail {
            eprintln!("FAIL: {f}");
        }
        std::process::exit(1);
    }
    // 零 panic 由本进程正常退出自证（任何 panic = harness 崩溃）。
    println!("zero-panic: yes (process exited normally)");
}

// 备忘：BTreeMap 保留给后续 mdcore 落地时的关系面扩展（.rels 提取），
// 本原型四 API 面不含关系面（calamine 无公开 API——FIDELITY.md §4）。
#[allow(dead_code)]
type _RelIndex = BTreeMap<String, Vec<String>>;
