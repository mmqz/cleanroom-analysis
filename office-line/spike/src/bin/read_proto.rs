//! xlsx-r1 t10-d — XLSX 读骨架 harness（产品面已收束至 lib `io` 模块）。
//!
//! 演进注记：原型三段（预扫门 / calamine 包装 / 统一错误面）已直移
//! `src/io.rs`（lib 形态，阶段 B → mdc-office-io 直移源）；本文件保留
//! 为 17 件全语料集成 harness（mdcore 集成测试同构）。
//!
//! 门结构（lib io::read_book）：①prescan 声明值四检查（SECURITY_POLICY
//! §2.1 + §3 限额）→ ②verify_actual_sizes 谎言头闭环（zip 8.6 读界
//! 无内建强制[lie86 探针 2026-10-13 实测]，策略层补）→ ③calamine
//! 四 API 面（表名单/值矩阵/公式文本/合并范围）。
//!
//! 17 件语料逐件三分断言 + 锚点闭包，**零 panic**（panic = harness
//! 崩溃即失败）。敌意件归类为**实证钉定**（版本锁定 calamine 0.36.1 /
//! zip 8.6.0；升版须重跑重钉）。

use std::fmt::Write as _;

use calamine::Data;
use office_impl_spike::io::{self, Book, ReadError};

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
        // 敌意 ⑥：容器门检查逐件对位（门 1 声明值 + 门 2 实际尺寸）
        // mal6a：声明 16B 过门 1；门 2 实测计数 1MB > 16B → REJECT
        //（2026-10-13 定谳：zip 8.6 读界无内建声明值强制[lie86 探针]，
        // zip 9 的拒绝[T3 3b]为 9.x 独有——策略层 verify_actual_sizes 补）
        ("mal6a_lie_header.zip", Class::Reject, |_| true),
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
        let r = io::read_book(&format!("{corpus}/{file}"));
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
