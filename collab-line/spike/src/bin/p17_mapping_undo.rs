//! p17_mapping_undo —— 映射矩阵撤销栈就绪探针（d1p-t4）。
//!
//! 依据：yrs-e §4.3 身份双轨解耦 + §7 撤销行（B3.2 per-origin 分栈
//! 在 XmlFragment 载体复验 + redo 新 ID 族）。适配层既有形态
//! （d1c-t2：双 UndoManager[undo_human/undo_ai] + include_origin +
//! expand_scope 根 + 捕获窗）在 **XmlFragment 载体**上的等价复验：
//!   p17-A per-origin 分栈：human/ai 双栈互不干扰（undo human 仅撤
//!      human 面，ai 编辑保留；反之亦然）
//!   p17-B redo 新 ID 族：undo→redo 复原 = 结构等价重插，载体身份
//!      新族（StickyIndex ID 不等——p13-V5 同构，经 UndoManager
//!      路径复验）
//!   p17-C 物化解耦（§4.3 两轴）：InsertBlock.id=Some 物化路径模拟
//!      ——redo 后 block_id 属性（BlockTree 身份轴）保持 + 载体
//!      ID（协同轴）换新族 → 双轴各自连续互不污染
//!   p17-D 混合序列跨栈交叉：H/A/H 交替 → human 栈 LIFO 两撤、
//!      ai 栈一撤，双 redo 各自复原
//!
//! 口径：release 跑；capture_timeout=0（每事务一栈项，p4 对照面）
//! ——1000ms 合并窗语义已由 p4/适配层锁定，本探针不重复。

use yrs::types::xml::XmlOut;
use yrs::{
    ClientID, Doc, GetString, IndexedSequence, Options as DocOptions, Origin, ReadTxn, Transact,
    UndoManager, Xml, XmlElementPrelim, XmlFragment, XmlTextPrelim,
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
            .push(XmlInText(text));
    }
    e
}

fn XmlInText(text: &str) -> yrs::types::xml::XmlIn {
    yrs::types::xml::XmlIn::Text(XmlTextPrelim::new(text).into())
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

fn new_mgr(doc: &Doc, origin: &str, xml: &yrs::XmlFragmentRef) -> UndoManager {
    let mut m = UndoManager::with_options(yrs::undo::Options {
        capture_timeout_millis: 0,
        ..Default::default()
    });
    m.expand_scope(doc, xml);
    m.include_origin(Origin::from(origin));
    m
}

fn main() {
    println!("== p17_mapping_undo：撤销栈 B3.2 分栈在 XmlFragment 载体复验 ==\n");
    let mut failures = 0usize;

    // ── p17-A/B/C per-origin 分栈 + redo 新 ID 族 + 物化解耦 ────────
    {
        let doc = new_doc(7_000_701);
        let xml = doc.get_or_insert_xml_fragment("doc");
        let mut mgr_h = new_mgr(&doc, "human", &xml);
        let mut mgr_a = new_mgr(&doc, "ai", &xml);

        // human 插 p-1（block_id = BlockTree 身份轴模拟）
        {
            let mut txn = doc.transact_mut_with(Origin::from("human"));
            xml.insert(&mut txn, 0, el("p", &[("block_id", "p-1")], "人类段"));
            txn.commit();
        }
        // ai 插 p-2
        {
            let mut txn = doc.transact_mut_with(Origin::from("ai"));
            xml.insert(&mut txn, 1, el("p", &[("block_id", "p-2")], "AI 段"));
            txn.commit();
        }
        let expect_full = vec![
            "<p [block_id=p-1] T(人类段)>".to_string(),
            "<p [block_id=p-2] T(AI 段)>".to_string(),
        ];
        if structure(&doc) == expect_full {
            println!("[PASS] p17-A1 双 origin 编辑全量在场");
        } else {
            failures += 1;
            println!("[FAIL] p17-A1 全量：{:?}", structure(&doc));
        }

        // 锚定 p-1（human 块）载体身份（协同轴）
        let old_carrier_id = {
            let txn = doc.transact();
            xml.sticky_index(&txn, 0, yrs::Assoc::After)
                .unwrap()
                .id()
                .cloned()
                .unwrap()
        };

        // undo human：仅撤 p-1，p-2（ai）保留
        assert!(mgr_h.undo_blocking(), "human undo 执行");
        let expect_after_hu = vec!["<p [block_id=p-2] T(AI 段)>".to_string()];
        if structure(&doc) == expect_after_hu {
            println!("[PASS] p17-A2 human 分栈 undo 仅撤 human 面（ai 编辑保留）");
        } else {
            failures += 1;
            println!("[FAIL] p17-A2：{:?}", structure(&doc));
        }

        // undo ai：撤 p-2 → 空
        assert!(mgr_a.undo_blocking(), "ai undo 执行");
        if structure(&doc).is_empty() {
            println!("[PASS] p17-A3 ai 分栈独立可撤（跨栈互不阻塞）");
        } else {
            failures += 1;
            println!("[FAIL] p17-A3：{:?}", structure(&doc));
        }

        // redo human：p-1 复原 = 结构等价 + block_id 保持 + 载体 ID 新族
        assert!(mgr_h.redo_blocking(), "human redo 执行");
        let (new_sig, new_bid, new_carrier_id) = {
            let txn = doc.transact();
            match xml.children(&txn).next() {
                Some(XmlOut::Element(e)) => {
                    let bid = e
                        .attributes(&txn)
                        .into_iter()
                        .find(|(k, _)| *k == "block_id")
                        .map(|(_, v)| v.to_string(&txn))
                        .unwrap_or_default();
                    (
                        element_sig(&e, &txn),
                        bid,
                        xml.sticky_index(&txn, 0, yrs::Assoc::After)
                            .unwrap()
                            .id()
                            .cloned()
                            .unwrap(),
                    )
                }
                _ => unreachable!(),
            }
        };
        let bc_ok = new_sig == "<p [block_id=p-1] T(人类段)>"
            && new_bid == "p-1"
            && new_carrier_id != old_carrier_id;
        if bc_ok {
            println!("[PASS] p17-B/C redo 复原：block_id 保持（身份轴连续）+ 载体 ID 换新族（协同轴独立）——双轴解耦钉定");
        } else {
            failures += 1;
            println!(
                "[FAIL] p17-B/C：sig={new_sig} bid={new_bid} 换族={}",
                new_carrier_id != old_carrier_id
            );
        }

        // redo ai：p-2 复原（序：p-1 在前 p-2 在后）
        assert!(mgr_a.redo_blocking(), "ai redo 执行");
        if structure(&doc) == expect_full {
            println!("[PASS] p17-A4 双 redo 后全量复原（跨栈序保持）");
        } else {
            failures += 1;
            println!("[FAIL] p17-A4：{:?}", structure(&doc));
        }
    }

    // ── p17-D 混合序列跨栈交叉 LIFO ────────────────────────────────
    {
        let doc = new_doc(7_000_702);
        let xml = doc.get_or_insert_xml_fragment("doc");
        let mut mgr_h = new_mgr(&doc, "human", &xml);
        let mut mgr_a = new_mgr(&doc, "ai", &xml);

        // H(p-1) → A(p-2) → H(p-3) 交替
        {
            let mut txn = doc.transact_mut_with(Origin::from("human"));
            xml.insert(&mut txn, 0, el("p", &[("block_id", "p-1")], "一"));
            txn.commit();
        }
        {
            let mut txn = doc.transact_mut_with(Origin::from("ai"));
            xml.insert(&mut txn, 1, el("p", &[("block_id", "p-2")], "二"));
            txn.commit();
        }
        {
            let mut txn = doc.transact_mut_with(Origin::from("human"));
            xml.insert(&mut txn, 2, el("p", &[("block_id", "p-3")], "三"));
            txn.commit();
        }

        // human 栈两撤（LIFO：先 p-3 后 p-1），ai 面保留
        assert!(mgr_h.undo_blocking());
        let s1 = structure(&doc);
        let want1 = vec![
            "<p [block_id=p-1] T(一)>".to_string(),
            "<p [block_id=p-2] T(二)>".to_string(),
        ];
        assert!(mgr_h.undo_blocking());
        let s2 = structure(&doc);
        let want2 = vec!["<p [block_id=p-2] T(二)>".to_string()];
        // ai 一撤 → 空
        assert!(mgr_a.undo_blocking());
        let s3 = structure(&doc);
        // 双 redo：human redo×2 + ai redo×1 → 全量复原
        assert!(mgr_a.redo_blocking());
        assert!(mgr_h.redo_blocking());
        assert!(mgr_h.redo_blocking());
        let s4 = structure(&doc);
        let want4 = vec![
            "<p [block_id=p-1] T(一)>".to_string(),
            "<p [block_id=p-2] T(二)>".to_string(),
            "<p [block_id=p-3] T(三)>".to_string(),
        ];

        if s1 == want1 && s2 == want2 && s3.is_empty() && s4 == want4 {
            println!("[PASS] p17-D 混合序列跨栈交叉（human LIFO 两撤/ai 独立一撤/双 redo 全量复原）");
        } else {
            failures += 1;
            println!(
                "[FAIL] p17-D：\n  s1={s1:?}\n  s2={s2:?}\n  s3={s3:?}\n  s4={s4:?}"
            );
        }
    }

    println!(
        "\n== p17 结果：{} ==",
        if failures == 0 { "ALL PASS".to_string() } else { format!("{failures} FAILURES") }
    );
    assert_eq!(failures, 0, "p17 撤销栈复验存在失败断言");
}
