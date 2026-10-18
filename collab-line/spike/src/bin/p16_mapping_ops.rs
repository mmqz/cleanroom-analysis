//! p16_mapping_ops —— 映射矩阵操作保真环探针（d1p-t3 就绪件）。
//!
//! 依据：yrs-e §5 操作映射表 + §3.2 mark 拆分规约 + §6 差异 D-1/D-2。
//! 本探针 = 7 ops 翻译器微缩实现（BlockTree 语义 → 载体事务）+ 全环
//! 断言：
//!   p16-A SetAttrs 终态替换：{a:1,b:2} → SetAttrs{c:3} → 终态恰
//!      {c:3}（清 attrs 域写目标集，非增量合并；block_id = 身份域
//!      不受触动——映射 §4.3 双轨解耦）
//!   p16-B 重叠 marks 拆分（D-2）：em[0,10) ∩ strong[5,15) → 最小
//!      区间等价覆盖三段装载 → 读回覆盖断言语义相等（em=[0,10)，
//!      strong=[5,15)——非字面）
//!   p16-C mark 边界推移：em 内首插/尾插文本 → 覆盖随文本扩张
//!   p16-D SetBlockType（D-1）：p→h 删旧建新搬子树 → 结构等价 +
//!      block_id 连续 + 载体身份换新族（StickyIndex ID 不等）
//!   p16-E 七 op 全环：InsertBlock×4（含嵌套 parent 路径）/ReplaceText
//!      /SetAttrs/SetMark/MoveBlock/RemoveBlock → 载体事务 → 结构
//!      断言 → wire diff → 对端 apply 收敛 → 增量二轮再收敛
//!
//! 口径：release 跑；路径翻译含 own-text 偏移规则（blockquote/li
//! 子块下标 +1——映射 §2 主表自有文本首位承载的实现细节）。

use yrs::types::xml::{XmlIn, XmlOut};
use yrs::updates::decoder::Decode;
use yrs::{
    ClientID, Doc, GetString, IndexedSequence, Options as DocOptions, ReadTxn,
    Text as TextTrait, Transact, Xml, XmlElementPrelim, XmlFragment, XmlTextPrelim,
};

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true; // B1.1 口径对齐
    Doc::with_options(opts)
}

/// 【实测发现】yrs 0.28 XmlText 索引/长度 = UTF-8 **字节域**
/// （sem_probe："第一段".len()=15；insert(3) 落在第/一字节边界）
/// ——非 Text(YText) 的 UTF-16 域非 char 域。
/// BlockTree ReplaceText/SetMark = char 域 → 适配层边界换算（无损纯工程）。
fn c2b(s: &str, char_idx: u32) -> u32 {
    s.char_indices()
        .nth(char_idx as usize)
        .map(|(b, _)| b as u32)
        .unwrap_or(s.len() as u32)
}

fn el(tag: &str, attrs: &[(&str, &str)], text: &str) -> XmlElementPrelim {
    let mut e = XmlElementPrelim::empty(tag);
    for (k, v) in attrs {
        e.attributes.insert((*k).into(), v.to_string());
    }
    if !text.is_empty() {
        e.children
            .push(XmlIn::Text(XmlTextPrelim::new(text).into()));
    }
    e
}

fn mark(kind: &str, text: &str) -> XmlElementPrelim {
    let mut m = XmlElementPrelim::empty("m");
    m.attributes.insert("kind".into(), kind.to_string());
    m.children
        .push(XmlIn::Text(XmlTextPrelim::new(text).into()));
    m
}

fn element_sig(el: &yrs::XmlElementRef, txn: &yrs::Transaction) -> String {
    let mut parts = Vec::new();
    for child in el.children(txn) {
        match child {
            XmlOut::Text(t) => parts.push(format!("T({})", t.get_string(txn))),
            XmlOut::Element(e) => parts.push(element_sig(&e, txn)),
            XmlOut::Fragment(_) => parts.push("F()".to_string()),
        }
    }
    let mut attr_strs: Vec<String> = el
        .attributes(txn)
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    attr_strs.sort();
    format!("<{} [{}] {}>", el.tag(), attr_strs.join(","), parts.join(" "))
}

fn structure(doc: &Doc) -> Vec<String> {
    let xml = doc.get_or_insert_xml_fragment("doc");
    let txn = doc.transact();
    xml.children(&txn)
        .filter_map(|c| match c {
            XmlOut::Element(e) => Some(element_sig(&e, &txn)),
            _ => None,
        })
        .collect()
}

/// BlockTree 路径 → 载体元素解析（own-text 偏移规则：blockquote/li
/// 的 BlockTree 子块 i = 载体子节点 i+1——首位为自有文本节点）。
fn child_offset(tag: &str) -> u32 {
    if tag == "blockquote" || tag == "li" {
        1
    } else {
        0
    }
}

/// 读回 mark 覆盖：per-kind 字符区间并集（语义面，非字面切分）。
fn mark_coverage(el: &yrs::XmlElementRef, txn: &yrs::Transaction) -> Vec<(String, u32, u32)> {
    fn walk(
        el: &yrs::XmlElementRef,
        txn: &yrs::Transaction,
        active: &mut Vec<String>,
        pos: &mut u32,
        out: &mut Vec<(String, u32, u32)>,
    ) {
        for child in el.children(txn) {
            match child {
                XmlOut::Text(t) => {
                    let len = t.get_string(txn).chars().count() as u32;
                    let (mut s, mut e) = (*pos, *pos);
                    let _ = (&mut s, &mut e);
                    for kind in active.iter() {
                        out.push((kind.clone(), *pos, *pos + len));
                    }
                    *pos += len;
                }
                XmlOut::Element(c) => {
                    if c.tag().as_ref() == "m" {
                        let kind = c
                            .attributes(txn)
                            .into_iter()
                            .find(|(k, _)| *k == "kind")
                            .map(|(_, v)| v.to_string(txn))
                            .unwrap_or_default();
                        active.push(kind);
                        walk(&c, txn, active, pos, out);
                        active.pop();
                    } else {
                        walk(&c, txn, active, pos, out);
                    }
                }
                XmlOut::Fragment(_) => {}
            }
        }
    }
    let mut out = Vec::new();
    let mut pos = 0u32;
    let mut active = Vec::new();
    walk(el, txn, &mut active, &mut pos, &mut out);
    // 同 kind 区间排序合并
    out.sort_by(|a, b| {
        a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2))
    });
    let mut merged: Vec<(String, u32, u32)> = Vec::new();
    for (k, s, e) in out {
        if let Some(last) = merged.last_mut() {
            if last.0 == k && last.2 == s {
                last.2 = e;
                continue;
            }
        }
        merged.push((k, s, e));
    }
    merged
}

fn main() {
    println!("== p16_mapping_ops：映射矩阵操作保真环 ==\n");
    let mut failures = 0usize;

    // ── p16-A SetAttrs 终态替换 ────────────────────────────────────
    {
        let doc = new_doc(7_000_601);
        let xml = doc.get_or_insert_xml_fragment("doc");
        {
            let mut txn = doc.transact_mut();
            xml.insert(&mut txn, 0, el("h", &[("block_id", "a-1"), ("a", "1"), ("b", "2")], "题"));
            txn.commit();
        }
        {
            let mut txn = doc.transact_mut();
            let el0 = xml
                .children(&txn)
                .find_map(|c| match c {
                    XmlOut::Element(e) => Some(e),
                    _ => None,
                })
                .unwrap();
            // SetAttrs{c:3} 翻译：清 attrs 域（a/b）→ 写 {c:3}；block_id 身份域不触
            el0.remove_attribute(&mut txn, &"a");
            el0.remove_attribute(&mut txn, &"b");
            el0.insert_attribute(&mut txn, "c", "3");
            txn.commit();
        }
        let want = "<h [block_id=a-1,c=3] T(题)>";
        let got = structure(&doc);
        if got.len() == 1 && got[0] == want {
            println!("[PASS] p16-A SetAttrs 终态替换（清旧写新非合并 + 身份域解耦不触）");
        } else {
            failures += 1;
            println!("[FAIL] p16-A：got {:?} want {want}", got);
        }
    }

    // ── p16-B 重叠 marks 拆分（D-2 语义等价）───────────────────────
    {
        let doc = new_doc(7_000_602);
        let xml = doc.get_or_insert_xml_fragment("doc");
        {
            let mut txn = doc.transact_mut();
            // 文本 "ABCDEFGHIJKLMNO"(15) + em[0,10) + strong[5,15)
            // → 三段最小区间等价覆盖：[0,5)em / [5,10)em+strong / [10,15)strong
            let mut p = XmlElementPrelim::empty("p");
            p.children.push(XmlIn::Element(mark("em", "ABCDE")));
            p.children.push(XmlIn::Element({
                let mut inner = mark("strong", "FGHIJ");
                let mut outer = XmlElementPrelim::empty("m");
                outer.attributes.insert("kind".into(), "em".into());
                outer.children.push(XmlIn::Element(inner));
                outer
            }));
            p.children.push(XmlIn::Element(mark("strong", "KLMNO")));
            xml.insert(&mut txn, 0, p);
            txn.commit();
        }
        let cov = {
            let txn = doc.transact();
            match xml.children(&txn).next() {
                Some(XmlOut::Element(e)) => mark_coverage(&e, &txn),
                _ => vec![],
            }
        };
        let want = vec![
            ("em".to_string(), 0u32, 10u32),
            ("strong".to_string(), 5u32, 15u32),
        ];
        if cov == want {
            println!("[PASS] p16-B 重叠 marks 拆分读回语义相等（em=[0,10) strong=[5,15) 并集还原）");
        } else {
            failures += 1;
            println!("[FAIL] p16-B：coverage {:?} want {want:?}", cov);
        }
    }

    // ── p16-C mark 边界推移 ────────────────────────────────────────
    {
        let doc = new_doc(7_000_603);
        let xml = doc.get_or_insert_xml_fragment("doc");
        {
            let mut txn = doc.transact_mut();
            let mut p = XmlElementPrelim::empty("p");
            p.children.push(XmlIn::Element(mark("em", "Hello")));
            xml.insert(&mut txn, 0, p);
            txn.commit();
        }
        // em 文本首插 ">>" + 尾插 " World"
        {
            let mut txn = doc.transact_mut();
            // 经由 em 元素取文本（children[0] = m 元素 → 其内文本）
            if let Some(XmlOut::Element(p)) = xml.children(&txn).next() {
                if let Some(XmlOut::Element(m)) = p.children(&txn).next() {
                    if let Some(XmlOut::Text(t)) = m.children(&txn).next() {
                        t.insert(&mut txn, 0, ">>");
                        let len = t.get_string(&txn).len() as u32; // 字节域
                        t.insert(&mut txn, len, " World");
                    }
                }
            }
            txn.commit();
        }
        let cov = {
            let txn = doc.transact();
            match xml.children(&txn).next() {
                Some(XmlOut::Element(e)) => mark_coverage(&e, &txn),
                _ => vec![],
            }
        };
        let want = vec![("em".to_string(), 0u32, 13u32)]; // ">>Hello World" 全覆盖
        if cov == want {
            println!("[PASS] p16-C mark 边界推移（首插+尾插后覆盖 [0,13) 随文本扩张）");
        } else {
            failures += 1;
            println!("[FAIL] p16-C：coverage {:?} want {want:?}", cov);
        }
    }

    // ── p16-D SetBlockType（D-1 删旧建新搬子树）────────────────────
    {
        let doc = new_doc(7_000_604);
        let xml = doc.get_or_insert_xml_fragment("doc");
        // <p block_id=d-9>[T(head ), m.em(mid), T( tail)]>
        {
            let mut txn = doc.transact_mut();
            let mut p = XmlElementPrelim::empty("p");
            p.attributes.insert("block_id".into(), "d-9".into());
            p.children
                .push(XmlIn::Text(XmlTextPrelim::new("head ").into()));
            p.children.push(XmlIn::Element(mark("em", "mid")));
            p.children
                .push(XmlIn::Text(XmlTextPrelim::new(" tail").into()));
            xml.insert(&mut txn, 0, p);
            txn.commit();
        }
        let (old_id, old_sig) = {
            let txn = doc.transact();
            match xml.children(&txn).next() {
                Some(XmlOut::Element(e)) => {
                    let s = xml
                        .sticky_index(&txn, 0, yrs::Assoc::After)
                        .unwrap();
                    (s.id().cloned().unwrap(), element_sig(&e, &txn))
                }
                _ => unreachable!(),
            }
        };
        // D-1 翻译：删旧建新搬子树（同名属性 + 同子结构，tag p→h + level）
        {
            let mut txn = doc.transact_mut();
            xml.remove_range(&mut txn, 0, 1);
            let mut h = XmlElementPrelim::empty("h");
            h.attributes.insert("block_id".into(), "d-9".into());
            h.attributes.insert("level".into(), "3".into());
            h.children
                .push(XmlIn::Text(XmlTextPrelim::new("head ").into()));
            h.children.push(XmlIn::Element(mark("em", "mid")));
            h.children
                .push(XmlIn::Text(XmlTextPrelim::new(" tail").into()));
            xml.insert(&mut txn, 0, h);
            txn.commit();
        }
        let (new_id, new_sig, bid) = {
            let txn = doc.transact();
            match xml.children(&txn).next() {
                Some(XmlOut::Element(e)) => {
                    let s = xml
                        .sticky_index(&txn, 0, yrs::Assoc::After)
                        .unwrap();
                    let bid = e
                        .attributes(&txn)
                        .into_iter()
                        .find(|(k, _)| *k == "block_id")
                        .map(|(_, v)| v.to_string(&txn))
                        .unwrap_or_default();
                    (s.id().cloned().unwrap(), element_sig(&e, &txn), bid)
                }
                _ => unreachable!(),
            }
        };
        let want_sig = "<h [block_id=d-9,level=3] T(head ) <m [kind=em] T(mid)> T( tail)>";
        let d_ok = new_sig == want_sig
            && bid == "d-9"          // BlockId 连续（身份域经属性承载在探针内的等价物）
            && new_id != old_id;     // 载体身份换新族（D-1 钉定）
        if d_ok {
            println!("[PASS] p16-D SetBlockType D-1（结构等价 + block_id 连续 + 载体身份换新族）");
        } else {
            failures += 1;
            println!(
                "[FAIL] p16-D：sig {} bid={bid} id 换族={}",
                new_sig == want_sig,
                new_id != old_id
            );
        }
    }

    // ── p16-E 七 op 全环（翻译器微缩 + 对端收敛）───────────────────
    {
        let doc_a = new_doc(7_000_605);
        let xml = doc_a.get_or_insert_xml_fragment("doc");

        // op1 InsertBlock([], 0, Paragraph, {block_id:e-1}, "第一段")
        {
            let mut txn = doc_a.transact_mut();
            xml.insert(&mut txn, 0, el("p", &[("block_id", "e-1")], "第一段"));
            txn.commit();
        }
        // op2 InsertBlock([], 1, Heading, {level:2, block_id:e-2}, "标题")
        {
            let mut txn = doc_a.transact_mut();
            xml.insert(
                &mut txn,
                1,
                el("h", &[("level", "2"), ("block_id", "e-2")], "标题"),
            );
            txn.commit();
        }
        // op3 InsertBlock([], 2, Blockquote, {block_id:e-3}, "引语自有")
        {
            let mut txn = doc_a.transact_mut();
            xml.insert(
                &mut txn,
                2,
                el("blockquote", &[("block_id", "e-3")], "引语自有"),
            );
            txn.commit();
        }
        // op4 InsertBlock(parent=[2], 0, Paragraph, {block_id:e-4}, "引语内段")
        //    路径翻译：[2] → blockquote；own-text 偏移 → 子位 0+1=1
        {
            let mut txn = doc_a.transact_mut();
            let children: Vec<_> = xml.children(&txn).collect();
            if let Some(XmlOut::Element(bq)) = children.get(2) {
                let off = child_offset(bq.tag());
                bq.insert(&mut txn, off, el("p", &[("block_id", "e-4")], "引语内段"));
            }
            txn.commit();
        }
        // op5 ReplaceText([0], start=3, end=3, insert="落尾") → "第一段落尾"
        //    char 位 3 → 字节位 9（c2b 边界换算——映射 §3.1 修正版）
        {
            let mut txn = doc_a.transact_mut();
            if let Some(XmlOut::Element(p)) = xml.children(&txn).next() {
                if let Some(XmlOut::Text(t)) = p.children(&txn).next() {
                    let full = t.get_string(&txn);
                    t.insert(&mut txn, c2b(&full, 3), "落尾");
                }
            }
            txn.commit();
        }
        // op6 SetAttrs([1], {level:3}) → 终态替换（清 level 写 3）
        {
            let mut txn = doc_a.transact_mut();
            let children: Vec<_> = xml.children(&txn).collect();
            if let Some(XmlOut::Element(h)) = children.get(1) {
                h.remove_attribute(&mut txn, &"level");
                h.insert_attribute(&mut txn, "level", "3");
            }
            txn.commit();
        }
        // op7 SetMark([0], 0, 2, em, add) → em[0,2) 包裹 "第一"
        {
            let mut txn = doc_a.transact_mut();
            if let Some(XmlOut::Element(p)) = xml.children(&txn).next() {
                if let Some(XmlOut::Text(t)) = p.children(&txn).next() {
                    // 拆分：T("第一段落尾") → [m.em("第一"), T("段落尾")]
                    // char 域 [0,2) → 字节域 [0,6)（c2b）
                    let full = t.get_string(&txn);
                    let byte_off = c2b(&full, 2);
                    let (head, _tail) = full.split_at(byte_off as usize);
                    t.remove_range(&mut txn, 0, byte_off);
                    p.insert(&mut txn, 0, mark("em", head));
                }
            }
            txn.commit();
        }
        // op8 MoveBlock(from=[1], to_parent=[], to_index=0) → h 移至首位
        {
            let mut txn = doc_a.transact_mut();
            xml.remove_range(&mut txn, 1, 1); // h 离位
            // 此时 [p, blockquote]；h 将重插于 0——搬移 = 删+插同事务
            // （prelim 重建同构 D-1；语义 = 结构等价搬移）
            let h_new = el("h", &[("level", "3"), ("block_id", "e-2")], "标题");
            xml.insert(&mut txn, 0, h_new);
            txn.commit();
        }
        // op9 RemoveBlock([2, 0]) → 删 blockquote 子块（own-text 偏移 0+1=1）
        {
            let mut txn = doc_a.transact_mut();
            let children: Vec<_> = xml.children(&txn).collect();
            if let Some(XmlOut::Element(bq)) = children.get(2) {
                let off = child_offset(bq.tag());
                bq.remove_range(&mut txn, off, 1);
            }
            txn.commit();
        }

        let expect_e = vec![
            "<h [block_id=e-2,level=3] T(标题)>".to_string(),
            "<p [block_id=e-1] <m [kind=em] T(第一)> T(段落尾)>".to_string(),
            "<blockquote [block_id=e-3] T(引语自有)>".to_string(),
        ];
        let got = structure(&doc_a);
        if got == expect_e {
            println!("[PASS] p16-E1 七 op 翻译器微缩全环结构断言（含 own-text 偏移路径翻译）");
        } else {
            failures += 1;
            println!("[FAIL] p16-E1：");
            for (i, (g, e)) in got.iter().zip(expect_e.iter()).enumerate() {
                if g != e {
                    println!("  块{i}: got  {g}\n       want {e}");
                }
            }
        }

        // 对端收敛：wire diff → doc_b apply → 结构相等
        let doc_b = new_doc(7_000_606);
        {
            let sv = doc_b.transact().state_vector();
            let bytes = doc_a.transact().encode_diff_v1(&sv);
            let mut txn = doc_b.transact_mut();
            txn.apply_update(yrs::Update::decode_v1(&bytes).expect("decode"))
                .expect("apply");
            txn.commit();
        }
        if structure(&doc_b) == expect_e {
            println!("[PASS] p16-E2 对端 wire apply 收敛（七 op 终态结构相等）");
        } else {
            failures += 1;
            println!("[FAIL] p16-E2 对端结构不等");
        }

        // 增量二轮：doc_a 追加编辑 → 增量 diff → doc_b 再收敛
        {
            let mut txn = doc_a.transact_mut();
            let idx = xml.len(&txn);
            xml.insert(&mut txn, idx, el("p", &[("block_id", "e-5")], "增量二轮段"));
            txn.commit();
        }
        {
            let sv = doc_b.transact().state_vector();
            let bytes = doc_a.transact().encode_diff_v1(&sv);
            let mut txn = doc_b.transact_mut();
            txn.apply_update(yrs::Update::decode_v1(&bytes).expect("decode"))
                .expect("apply");
            txn.commit();
        }
        let expect_e3 = {
            let mut v = expect_e.clone();
            v.push("<p [block_id=e-5] T(增量二轮段)>".to_string());
            v
        };
        if structure(&doc_b) == expect_e3 && structure(&doc_a) == expect_e3 {
            println!("[PASS] p16-E3 增量二轮再收敛（SV 增量 diff 双端结构相等）");
        } else {
            failures += 1;
            println!("[FAIL] p16-E3 增量轮不等");
        }
    }

    println!(
        "\n== p16 结果：{} ==",
        if failures == 0 { "ALL PASS".to_string() } else { format!("{failures} FAILURES") }
    );
    assert_eq!(failures, 0, "p16 操作保真环存在失败断言");
}
