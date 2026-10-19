//! p18_char_byte_conv —— D-3 偏移域边界换算完备 PoC（a5-r2）。
//!
//! 依据：yrs-e §3.1 修正版 + §6 D-3 工程项（p16 实测发现：yrs 0.28
//! XmlText 索引/长度 = UTF-8 **字节域**；BlockTree ReplaceText/SetMark
//! = char 域 → 适配层边界无损换算）。p16 仅有单向临时 c2b（clamp
//! 语义）；本探针升格为**完备双向换算模块候选**，五面：
//!   p18-A 纯换算向量断言：六语料（空/纯 ASCII/BMP/非 BMP/组合符/
//!      混合）char→字节精确对全表
//!   p18-B 活体 XmlText 一致性：len=字节域 + c2b 偏移 insert 落点 +
//!      remove_range 字节域口径——模块结论与载体实测互证
//!   p18-C 全位置往返性质：∀char_idx: b2c(c2b(i))==i + 末哨兵==len
//!      + 非边界字节偏移全数拒止（穷举非抽样）
//!   p18-D ReplaceText 真实路径：char 区间 → 字节区间删除+插入，与
//!      Rust 侧 char 域区间删除逐字相等（含区间内/边界处多字节与
//!      组合符）
//!   p18-E 错误面三态：char 越界 None / 字节越界 Err / 字节中码点
//!      Err——边界拒止显式化（适配层不得静默错位）
//!
//! 口径：release 跑；skip_gc=true（B1.1 对齐）。组合符语料使
//! char ≠ UTF-16 ≠ byte 三维分立（Yjs Text=UTF-16 域、Rust str=
//! char 域、yrs XmlText=字节域——三域并立证据）。

use yrs::types::xml::{XmlFragmentRef, XmlOut, XmlTextRef};
use yrs::{
    ClientID, Doc, GetString, Options as DocOptions, ReadTxn, Text as TextTrait, Transact,
    XmlElementPrelim, XmlFragment, XmlTextPrelim,
};

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true; // B1.1 口径对齐
    Doc::with_options(opts)
}

/// char 域 → 字节域（D-3 正向）。
/// char_idx > char_count → None（显式错误面——调用方边界裁断；
/// 与 p16 临时实现的 clamp 语义差异见 p18-E 注记）。
/// 末哨兵：char_idx == char_count → s.len()（append 位）。
fn c2b(s: &str, char_idx: u32) -> Option<u32> {
    let count = s.chars().count() as u32;
    if char_idx > count {
        return None;
    }
    s.char_indices()
        .nth(char_idx as usize)
        .map(|(b, _)| b as u32)
        .or(Some(s.len() as u32))
}

/// 字节域 → char 域（D-3 逆向）。三态：
///   Err("byte-out-of-range") 字节越界
///   Err("mid-codepoint")     落在码点中间（is_char_boundary false）
///   Ok(char_idx)             恰在字符边界
fn b2c(s: &str, byte_idx: u32) -> Result<u32, &'static str> {
    let i = byte_idx as usize;
    if i > s.len() {
        return Err("byte-out-of-range");
    }
    if !s.is_char_boundary(i) {
        return Err("mid-codepoint");
    }
    Ok(s[..i].chars().count() as u32)
}

/// char 域区间删除（Rust 侧真值参照——BlockTree ReplaceText 语义）
fn char_range_delete(s: &str, start: u32, end: u32) -> String {
    s.chars()
        .enumerate()
        .filter(|(i, _)| (*i as u32) < start || (*i as u32) >= end)
        .map(|(_, c)| c)
        .collect()
}

fn el_text(tag: &str, text: &str) -> XmlElementPrelim {
    let mut e = XmlElementPrelim::empty(tag);
    if !text.is_empty() {
        e.children.push(XmlTextPrelim::new(text).into());
    }
    e
}

/// 取首个子元素 own-text 的 XmlText（p16 同型访问；XmlOut 为自有
/// 引用包装——可跨作用域持有，读写事务均可作读侧）
fn own_text<R: ReadTxn>(xml: &XmlFragmentRef, txn: &R) -> Option<XmlTextRef> {
    let children: Vec<_> = xml.children(txn).collect();
    if let Some(XmlOut::Element(p)) = children.first() {
        if let Some(XmlOut::Text(t)) = p.children(txn).next() {
            return Some(t);
        }
    }
    None
}

fn main() {
    let mut failures = 0u32;
    let mut pass = |name: &str, ok: bool| {
        if ok {
            println!("[PASS] {name}");
        } else {
            failures += 1;
            println!("[FAIL] {name}");
        }
    };

    // 六语料：名字 / 串 / char 数 / 字节数
    // combining = 'e'+U+0301（2 char/3 byte——char 与 UTF-16 单元数一致
    // 但与字节分立；非 BMP 🚀 = 4 byte/2 UTF-16 单元/1 char——三维全分立）
    let corpus: Vec<(&str, &str)> = vec![
        ("empty", ""),
        ("ascii", "hello"),
        ("bmp", "中文字"),
        ("nonbmp", "🚀aβ"),
        ("combining", "e\u{301}"),
        ("mixed", "a中🚀βe\u{301}"),
    ];

    // ---------- p18-A 纯换算向量断言 ----------
    {
        // 精确对表（手算真值）：(语料, [(char_idx, byte_idx)])
        let vectors: Vec<(&str, Vec<(u32, u32)>)> = vec![
            ("empty", vec![(0, 0)]),
            ("ascii", vec![(0, 0), (1, 1), (4, 4), (5, 5)]),
            ("bmp", vec![(0, 0), (1, 3), (2, 6), (3, 9)]),
            ("nonbmp", vec![(0, 0), (1, 4), (2, 5), (3, 7)]),
            ("combining", vec![(0, 0), (1, 1), (2, 3)]),
            (
                "mixed",
                vec![(0, 0), (1, 1), (2, 4), (3, 8), (4, 10), (5, 11), (6, 13)],
            ),
        ];
        let mut ok = true;
        for (name, table) in &vectors {
            let s = corpus.iter().find(|(n, _)| n == name).unwrap().1;
            let count = s.chars().count() as u32;
            for (ci, bi) in table {
                match c2b(s, *ci) {
                    Some(got) if got == *bi => {}
                    other => {
                        ok = false;
                        println!("  A 向量错 {name}: c2b({ci}) 期望 {bi} 实得 {other:?}");
                    }
                }
                let _ = count;
            }
        }
        pass("p18-A1 六语料 char→字节精确对全表", ok);
    }

    // ---------- p18-B 活体 XmlText 一致性 ----------
    // 【实测发现（p18-F）】TransactionMut::commit() 不释放写锁——锁随
    // Drop 释放。同一作用域保活已 commit 的写事务再开任何事务 = 死锁
    // （首版即踩中：B2 读回挂起）。纪律：每写事务独立块作用域。
    {
        let mixed = "a中🚀βe\u{301}";
        let doc = new_doc(1);
        let xml = doc.get_or_insert_xml_fragment("doc");
        {
            let mut txn = doc.transact_mut();
            xml.insert(&mut txn, 0, el_text("p", mixed));
            txn.commit();
        }
        let (len_ok, read_ok) = {
            let txn = doc.transact();
            let t = own_text(&xml, &txn).expect("own-text");
            // B1 len = 字节域（p16 实测在模块语境复证：13 ≠ 6 char ≠ 7 UTF-16）
            let len_ok = t.len(&txn) as usize == mixed.len();
            let read_ok = t.get_string(&txn) == mixed;
            (len_ok, read_ok)
        };

        // B2 c2b 偏移 insert 落点：char 3（🚀 后）→ 字节 8，插 "X"
        {
            let mut txn = doc.transact_mut();
            let t = own_text(&xml, &txn).expect("own-text");
            t.insert(&mut txn, c2b(mixed, 3).unwrap(), "X");
            txn.commit();
        }
        let after = {
            let txn = doc.transact();
            let t = own_text(&xml, &txn).expect("own-text");
            t.get_string(&txn)
        };
        let (h, tl) = mixed.split_at(8);
        let expected = format!("{h}X{tl}");
        let ins_ok = after == expected;

        // B3 remove_range 字节域口径：删 "中🚀"（char [1,3) → 字节 [1,8)）
        {
            let mut txn = doc.transact_mut();
            let t = own_text(&xml, &txn).expect("own-text");
            let b0 = c2b(&expected, 1).unwrap();
            let b1 = c2b(&expected, 3).unwrap();
            t.remove_range(&mut txn, b0, b1 - b0);
            txn.commit();
        }
        let after2 = {
            let txn = doc.transact();
            let t = own_text(&xml, &txn).expect("own-text");
            t.get_string(&txn)
        };
        let rm_expected: String = expected
            .chars()
            .enumerate()
            .filter(|(i, _)| *i < 1 || *i >= 3)
            .map(|(_, c)| c)
            .collect();
        let rm_ok = after2 == rm_expected;

        pass(
            "p18-B1 活体 len=字节域（13 byte / 6 char / 7 UTF-16 三域分立）",
            len_ok,
        );
        pass("p18-B2 活体读回零畸变（组合符保序）", read_ok);
        pass("p18-B3 c2b 偏移 insert 落点精确（char3→byte8）", ins_ok);
        pass(
            "p18-B4 remove_range 字节域口径（char[1,3)→byte[1,8)）",
            rm_ok,
        );
    }

    // ---------- p18-C 全位置往返性质（穷举） ----------
    {
        let mut rt_ok = true;
        let mut sentinel_ok = true;
        let mut mid_reject_ok = true;
        for (name, s) in &corpus {
            let count = s.chars().count() as u32;
            // C1 往返：∀char_idx: b2c(c2b(i)) == i
            for ci in 0..=count {
                let b = c2b(s, ci).unwrap_or(u32::MAX);
                match b2c(s, b) {
                    Ok(back) if back == ci => {}
                    other => {
                        rt_ok = false;
                        println!("  C1 往返破 {name}: char {ci} → byte {b} → {other:?}");
                    }
                }
            }
            // C2 末哨兵：c2b(count) == s.len()
            if c2b(s, count) != Some(s.len() as u32) {
                sentinel_ok = false;
                println!("  C2 末哨兵错 {name}");
            }
            // C3 非边界字节偏移全数拒止（穷举 0..=len）
            for bi in 0..=(s.len() as u32) {
                let is_boundary = s.is_char_boundary(bi as usize);
                match b2c(s, bi) {
                    Ok(_) if !is_boundary => {
                        mid_reject_ok = false;
                        println!("  C3 中码点漏拒 {name}: byte {bi}");
                    }
                    Err(_) if is_boundary => {
                        mid_reject_ok = false;
                        println!("  C3 边界误拒 {name}: byte {bi}");
                    }
                    _ => {}
                }
            }
        }
        pass("p18-C1 全位置往返恒等（六语料穷举）", rt_ok);
        pass("p18-C2 末哨兵 == len（append 位语义）", sentinel_ok);
        pass(
            "p18-C3 非边界字节全数拒止 + 边界零误拒（穷举）",
            mid_reject_ok,
        );
    }

    // ---------- p18-D ReplaceText 真实路径 ----------
    {
        let cases: Vec<(&str, &str, u32, u32, &str)> = vec![
            ("D-a 区间中多字节", "a中🚀βe\u{301}", 2, 5, "替代"), // 删 🚀βe 留孤立 U+0301
            ("D-b 边界恰多字节", "中文字", 1, 2, "**"),           // 删「文」
            ("D-c 全区间清空", "中🚀文", 0, 3, ""),               // 删尽
            ("D-d 纯 ASCII 回退", "hello", 1, 4, "EL"),           // 字节=char 退化恒等
        ];
        let mut ok = true;
        for (name, s, cs, ce, ins) in &cases {
            let doc = new_doc(2);
            let xml = doc.get_or_insert_xml_fragment("doc");
            {
                let mut txn = doc.transact_mut();
                xml.insert(&mut txn, 0, el_text("p", s));
                txn.commit();
            }
            {
                let txn = doc.transact();
                let t = own_text(&xml, &txn).expect("own-text");
                drop(txn);
                let b0 = c2b(s, *cs).unwrap();
                let b1 = c2b(s, *ce).unwrap();
                let mut txn = doc.transact_mut();
                t.remove_range(&mut txn, b0, b1 - b0);
                if !ins.is_empty() {
                    t.insert(&mut txn, b0, ins);
                }
                txn.commit();
            }
            let after = {
                let txn = doc.transact();
                let t = own_text(&xml, &txn).expect("own-text");
                t.get_string(&txn)
            };
            // Rust 侧 char 域真值：删 [cs,ce) 后原位插入
            let mut expect = char_range_delete(s, *cs, *ce);
            let ins_byte_at = expect
                .char_indices()
                .nth(*cs as usize)
                .map(|(b, _)| b)
                .unwrap_or(expect.len());
            expect.insert_str(ins_byte_at, ins);
            if after != expect {
                ok = false;
                println!("  D 路径错 {name}: 载体 {after:?} ≠ 真值 {expect:?}");
            }
        }
        pass("p18-D ReplaceText char 区间→字节区间四案逐字相等", ok);
    }

    // ---------- p18-E 错误面三态 ----------
    {
        let mut over_ok = true;
        let mut byte_over_ok = true;
        let mut mid_msg_ok = true;
        for (name, s) in &corpus {
            let count = s.chars().count() as u32;
            // char 越界 → None（注意：与 p16 临时 clamp 的差异——本模块
            // 显式 None 交调用方裁断，p16 的 unwrap_or(len) 为探针简化）
            if c2b(s, count + 1).is_some() {
                over_ok = false;
                println!("  E char 越界漏拒 {name}");
            }
            // 字节越界 → Err
            if b2c(s, s.len() as u32 + 1).is_ok() {
                byte_over_ok = false;
                println!("  E 字节越界漏拒 {name}");
            }
        }
        // 中码点样例错误信息精确区分（bmp 第 1 字节）
        if b2c("中文字", 1) != Err("mid-codepoint") {
            mid_msg_ok = false;
        }
        if b2c("hello", 9) != Err("byte-out-of-range") {
            mid_msg_ok = false;
        }
        pass("p18-E1 char 越界 None（六语料 count+1）", over_ok);
        pass("p18-E2 字节越界 Err（六语料 len+1）", byte_over_ok);
        pass("p18-E3 中码点/越界错误信息精确区分", mid_msg_ok);
    }

    println!(
        "\n== p18 结果：{} ==",
        if failures == 0 {
            "ALL PASS".to_string()
        } else {
            format!("{failures} FAILURES")
        }
    );
    assert_eq!(failures, 0, "p18 D-3 换算 PoC 存在失败断言");
}
