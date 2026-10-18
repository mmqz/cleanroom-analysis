//! p15_mapping_structure —— 映射矩阵结构实证探针（d1p-t2 就绪件）。
//!
//! 依据：yrs-e §2 结构映射主表（BlockTree→XmlFragment 九 kind 承载
//! 形态）。本轮按映射表**精确形状**装载并断言：
//!   p15-A1 九 kind 全量装载读回签名全等（Document→fragment 根，
//!      p/h[level]/pre[lang] 多行/hr 空/tbl GFM 文本叶/blockquote
//!      自有文本+子块交织/li[marker] 首行文本+嵌套子项）
//!   p15-A2 wire 快照→新文档 apply 结构相等
//!   p15-A3 属性开放面透传（未知键/转义敏感值/非 ASCII 零丢失）
//!   p15-B  邻居编辑稳定：全部顶层块 StickyIndex 锚定 → 前插 3
//!      前删 2（删新插者，原块全存活）→ 各锚点 ID 等值（映射 §4.2）
//!   p15-C  交织容器子块序：blockquote 子块前插/删除后序保持
//!
//! 口径：release 跑；断言本机实测；元素名集合 = 映射 §2.2
//! {p, h, pre, hr, tbl, blockquote, li}；API 形态对齐 p13 先例。

use yrs::types::xml::{XmlIn, XmlOut};
use yrs::updates::decoder::Decode;
use yrs::{
    ClientID, Doc, GetString, IndexedSequence, Options as DocOptions, ReadTxn, StateVector,
    Transact, Xml, XmlElementPrelim, XmlFragment, XmlTextPrelim,
};

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true; // B1.1 口径对齐
    Doc::with_options(opts)
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

/// 确定性结构签名（p13 element_sig 同构）：属性排序 + 子节点递归。
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

fn main() {
    println!("== p15_mapping_structure：映射矩阵结构实证（九 kind 全量）==\n");
    let mut failures = 0usize;

    // ── p15-A1 九 kind 全量装载 ────────────────────────────────────
    // 顶层序列覆盖 8 个非根 kind（Document = fragment 根本尊）。
    let doc_a = new_doc(7_000_501);
    {
        let xml = doc_a.get_or_insert_xml_fragment("doc");
        let mut txn = doc_a.transact_mut();

        let p1 = el("p", &[("block_id", "b-001")], "第一段普通文本");
        let h = el("h", &[("level", "2"), ("block_id", "b-002")], "二级标题");
        let pre = el(
            "pre",
            &[("lang", "rust"), ("block_id", "b-003")],
            "fn main() {\n    println!(\"多行\");\n}",
        );
        let hr = el("hr", &[("block_id", "b-004")], "");
        let tbl = el(
            "tbl",
            &[("block_id", "b-005")],
            "| a | b |\n|---|---|\n| 1 | 2 |",
        );

        // blockquote：自有文本首位 + p 子块 + pre 子块交织
        let mut bq = el("blockquote", &[("block_id", "b-006")], "引语自有首段");
        bq.children
            .push(XmlIn::Element(el("p", &[("block_id", "b-007")], "引语内段落")));
        bq.children
            .push(XmlIn::Element(el("pre", &[("lang", "sh")], "echo hi")));

        // li：marker 有序 + 首行文本 + 嵌套 li 子项（二级）+ 项内段落
        let li_inner = el("li", &[("marker", "-")], "嵌套无序子项");
        let mut li = el("li", &[("marker", "1."), ("block_id", "b-008")], "首行文本");
        li.children.push(XmlIn::Element(li_inner));
        li.children
            .push(XmlIn::Element(el("p", &[("block_id", "b-009")], "项内段落")));

        let p2 = el("p", &[("block_id", "b-010")], "末段");

        for (i, node) in [p1, h, pre, hr, tbl, bq, li, p2].into_iter().enumerate() {
            let idx = xml.len(&txn);
            let _ = i;
            xml.insert(&mut txn, idx, node);
        }
        txn.commit();
    }

    let expect_a: Vec<String> = vec![
        "<p [block_id=b-001] T(第一段普通文本)>".into(),
        "<h [block_id=b-002,level=2] T(二级标题)>".into(),
        "<pre [block_id=b-003,lang=rust] T(fn main() {\n    println!(\"多行\");\n})>".into(),
        "<hr [block_id=b-004] >".into(),
        "<tbl [block_id=b-005] T(| a | b |\n|---|---|\n| 1 | 2 |)>".into(),
        "<blockquote [block_id=b-006] T(引语自有首段) <p [block_id=b-007] T(引语内段落)> <pre [lang=sh] T(echo hi)>>".into(),
        "<li [block_id=b-008,marker=1.] T(首行文本) <li [marker=-] T(嵌套无序子项)> <p [block_id=b-009] T(项内段落)>>".into(),
        "<p [block_id=b-010] T(末段)>".into(),
    ];
    let got_a = structure(&doc_a);
    if got_a == expect_a {
        println!("[PASS] p15-A1 九 kind 装载读回签名全等（8 顶层块 × 深层结构）");
    } else {
        failures += 1;
        println!("[FAIL] p15-A1 签名不等：");
        for (i, (g, e)) in got_a.iter().zip(expect_a.iter()).enumerate() {
            if g != e {
                println!("  块{i}: got  {g}\n       want {e}");
            }
        }
    }

    // ── p15-A2 wire 快照 → 新文档 apply → 结构相等 ─────────────────
    let bytes = doc_a.transact().encode_diff_v1(&StateVector::default());
    let doc_b = new_doc(7_000_502);
    {
        let mut txn = doc_b.transact_mut();
        txn.apply_update(yrs::Update::decode_v1(&bytes).expect("decode"))
            .expect("apply");
        txn.commit();
    }
    if structure(&doc_b) == expect_a {
        println!("[PASS] p15-A2 wire 快照→新文档 apply 结构相等（九 kind 形态全保真）");
    } else {
        failures += 1;
        println!("[FAIL] p15-A2 快照往返结构不等");
    }

    // ── p15-A3 属性开放面：未知键透传 ──────────────────────────────
    {
        let doc_o = new_doc(7_000_503);
        {
            let xml = doc_o.get_or_insert_xml_fragment("doc");
            let mut txn = doc_o.transact_mut();
            xml.insert(
                &mut txn,
                0,
                el(
                    "p",
                    &[
                        ("custom_key", "自定义值"),
                        ("weird", "a<b>&\"'"),
                        ("emoji", "非ASCII键值透传"),
                    ],
                    "开放键透传测试",
                ),
            );
            txn.commit();
        }
        let want = "<p [custom_key=自定义值,emoji=非ASCII键值透传,weird=a<b>&\"'] T(开放键透传测试)>";
        let got = structure(&doc_o);
        if got.len() == 1 && got[0] == want {
            println!("[PASS] p15-A3 属性开放面透传（未知键/转义敏感值/非 ASCII 零丢失）");
        } else {
            failures += 1;
            println!("[FAIL] p15-A3 开放面：got {:?} want {want}", got);
        }
    }

    // ── p15-B 邻居编辑稳定（全顶层块 StickyIndex 锚定）──────────────
    {
        let doc_s = new_doc(7_000_504);
        let xml = doc_s.get_or_insert_xml_fragment("doc");
        {
            let mut txn = doc_s.transact_mut();
            for i in 0..5 {
                let node = el(
                    "p",
                    &[("block_id", &format!("s-{i}"))],
                    &format!("段{i}"),
                );
                xml.insert(&mut txn, i, node);
            }
            txn.commit();
        }
        // 锚定全部 5 块（fragment 子下标位置锚定，p13-V3 同构）
        let anchors: Vec<(yrs::StickyIndex, yrs::ID)> = {
            let txn = doc_s.transact();
            (0..5)
                .map(|i| {
                    let s = xml.sticky_index(&txn, i, yrs::Assoc::After).expect("sticky");
                    let id = s.id().cloned().expect("id");
                    (s, id)
                })
                .collect()
        };
        // 前插 3（位 0/1/2）+ 前删 2（删新插的前两块——原 5 块全存活）
        {
            let mut txn = doc_s.transact_mut();
            for (pos, t) in [(0usize, "前插甲"), (1, "前插乙"), (2, "前插丙")] {
                xml.insert(&mut txn, pos as u32, el("h", &[("level", "3")], t));
            }
            xml.remove_range(&mut txn, 0, 2);
            txn.commit();
        }
        // 断言：5 锚点全部 ID 等值（p13-V3 判定形态：get_offset → 重锚 → id 比较）
        let mut b_ok = true;
        {
            let txn = doc_s.transact();
            for (i, (sticky, sid)) in anchors.iter().enumerate() {
                let off = sticky.get_offset(&txn).expect("offset after edits");
                let s2 = xml.sticky_index(&txn, off.index, yrs::Assoc::After).unwrap();
                let id2 = s2.id().cloned().unwrap();
                if id2 != *sid {
                    b_ok = false;
                    println!("  段{i} 锚点 ID 漂移：{sid:?} → {id2:?}");
                }
            }
        }
        // 附带结构断言：现存 6 块 = h前插丙 + 段0..段4（序保持）
        let want_b = vec![
            "<h [level=3] T(前插丙)>".to_string(),
            "<p [block_id=s-0] T(段0)>".to_string(),
            "<p [block_id=s-1] T(段1)>".to_string(),
            "<p [block_id=s-2] T(段2)>".to_string(),
            "<p [block_id=s-3] T(段3)>".to_string(),
            "<p [block_id=s-4] T(段4)>".to_string(),
        ];
        if structure(&doc_s) != want_b {
            b_ok = false;
            println!("  p15-B 结构不符：{:?}", structure(&doc_s));
        }
        if b_ok {
            println!("[PASS] p15-B 前插3前删2 后全部 5 锚点 ID 等值 + 序保持（映射 §4.2 基础）");
        } else {
            failures += 1;
            println!("[FAIL] p15-B 邻居编辑稳定性");
        }
    }

    // ── p15-C 交织容器子块序 ───────────────────────────────────────
    {
        let doc_i = new_doc(7_000_505);
        let xml = doc_i.get_or_insert_xml_fragment("doc");
        // blockquote 子块 [p0, p1, p2]（自有文本首位）→ 子块 1 前插 + 删子块 0
        {
            let mut txn = doc_i.transact_mut();
            let mut bq = el("blockquote", &[], "自有文本");
            for t in ["子块零", "子块壹", "子块贰"] {
                bq.children.push(XmlIn::Element(el("p", &[], t)));
            }
            xml.insert(&mut txn, 0, bq);
            txn.commit();
        }
        {
            let mut txn = doc_i.transact_mut();
            if let Some(XmlOut::Element(bq)) = xml.children(&txn).next() {
                // 子节点：[T(自有文本), p0, p1, p2] → 位 1 插前插子块
                bq.insert(&mut txn, 1, el("p", &[], "前插子块"));
                // 删原 p0（现位 2）：remove_range(start=2, len=1)
                bq.remove_range(&mut txn, 2, 1);
            }
            txn.commit();
        }
        let want = "<blockquote [] T(自有文本) <p [] T(前插子块)> <p [] T(子块壹)> <p [] T(子块贰)>>";
        let got = structure(&doc_i);
        if got.len() == 1 && got[0] == want {
            println!("[PASS] p15-C 交织容器子块前插/删除后序保持（自有文本首位恒定）");
        } else {
            failures += 1;
            println!("[FAIL] p15-C 交织序：got {:?}", got);
        }
    }

    println!(
        "\n== p15 结果：{} ==",
        if failures == 0 { "ALL PASS".to_string() } else { format!("{failures} FAILURES") }
    );
    assert_eq!(failures, 0, "p15 结构实证存在失败断言");
}
