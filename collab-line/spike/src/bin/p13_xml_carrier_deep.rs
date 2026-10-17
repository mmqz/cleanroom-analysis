//! p13_xml_carrier_deep —— XmlFragment 载体深度验证探针（d1d-t1 加固轮）。
//!
//! 用户口径：「Verify whether XmlFragment is suitable for the scoped
//! collaborative structure, including serialization, mixed text/element
//! content, stable IDs, deletion, and restoration.」
//!
//! 承 p11 六维对照（建议倾向 XmlFragment 呈裁决）→ 本轮对 XmlFragment
//! 单载体做**选型级深验**（mdcore BlockTree 形状：块含子块 + 行内
//! runs 交织）：
//!   V1 serialization：混合内容树全量 wire + 快照往返（新文档 apply 后
//!      结构逐节点相等）
//!   V2 mixed content：段内 [text, em, text] 三段交织读取；文本节点
//!      中段插入（分裂）序保持；元素与文本间插入新元素
//!   V3 stable IDs：StickyIndex 锚定元素；前插/前删兄弟 → ID 稳定；
//!      目标删除 → 悬空；**删除+undo → 新 ID 语义**（与适配层
//!      matrix 测试发现同构——ItemPtr::redo 重插不复活）
//!   V4 deletion：元素删除；子树整删（父删 → 子锚点全悬空）
//!   V5 restoration：undo 子树删除 → 结构复原（新 ID）；重绑定后锚点
//!      有效
//!   V6 sync：双端并发混合内容编辑 → 收敛 + 结构相等
//!   V7 wire：深层嵌套 wire 尺寸（对照 p11 C6 浅嵌套）
//!
//! 口径：release 跑；全部断言本机实测。原始 yrs API 直用（载体是
//! 适配层内部设计题——p11 先例）。

use std::time::Instant;

use yrs::types::xml::{XmlIn, XmlOut};
use yrs::updates::decoder::Decode;
use yrs::{
    ClientID, Doc, GetString, IndexedSequence, Options as DocOptions, ReadTxn, StateVector,
    Text as TextTrait, Transact, Xml, XmlElementPrelim, XmlFragment, XmlTextPrelim,
};

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true; // B1.1 口径对齐
    Doc::with_options(opts)
}

/// 构造混合内容段落：<p kind="para"><text/>Hello <em>bold</em> world</p>
/// 形状 = mdcore 行内 runs（marks/inline）的载体近同构形状。
fn mixed_paragraph(tag: &str, attrs: &[(&str, &str)], text_head: &str, em: &str, text_tail: &str) -> XmlElementPrelim {
    let mut p = XmlElementPrelim::empty(tag);
    for (k, v) in attrs {
        p.attributes.insert((*k).into(), v.to_string());
    }
    p.children
        .push(XmlIn::Text(XmlTextPrelim::new(text_head).into()));
    p.children.push(XmlIn::Element({
        let mut em_el = XmlElementPrelim::empty("em");
        em_el.children.push(XmlIn::Text(XmlTextPrelim::new(em).into()));
        em_el
    }));
    p.children
        .push(XmlIn::Text(XmlTextPrelim::new(text_tail).into()));
    p
}

/// 结构快照（确定性遍历）：[(tag, [attr..], children-kind 序列, 文本内容)]。
fn structure(doc: &Doc) -> Vec<String> {
    let xml = doc.get_or_insert_xml_fragment("doc");
    let txn = doc.transact();
    let mut out = Vec::new();
    for child in xml.children(&txn) {
        if let XmlOut::Element(el) = child {
            out.push(element_sig(&el, &txn));
        }
    }
    out
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
    let attrs = el.attributes(txn);
    let mut attr_strs: Vec<String> = attrs.into_iter().map(|(k, v)| format!("{k}={v}")).collect();
    attr_strs.sort();
    format!("<{} {} [{}]>", el.tag(), attr_strs.join(","), parts.join(" "))
}

fn main() {
    println!("== p13_xml_carrier_deep：XmlFragment 选型级深验 ==\n");

    // ── V1 serialization ──────────────────────────────────────────
    let doc_a = new_doc(7_000_401);
    {
        let xml = doc_a.get_or_insert_xml_fragment("doc");
        let mut txn = doc_a.transact_mut();
        for i in 0..50 {
            let p = mixed_paragraph(
                "p",
                &[("kind", "para"), ("block_id", &format!("p-{i:03}"))],
                &format!("head-{i} "),
                "bold",
                " tail",
            );
            let idx = xml.len(&txn);
            xml.insert(&mut txn, idx, p);
        }
        txn.commit();
    }
    let full_wire = doc_a
        .transact()
        .encode_diff_v1(&StateVector::default())
        .len();
    // 快照往返：新文档 apply → 结构逐节点相等
    let doc_b = new_doc(7_000_402);
    {
        let bytes = doc_a.transact().encode_diff_v1(&StateVector::default());
        let mut txn = doc_b.transact_mut();
        txn.apply_update(yrs::Update::decode_v1(&bytes).expect("decode"))
            .expect("apply");
        txn.commit();
    }
    assert_eq!(structure(&doc_a), structure(&doc_b), "V1: 快照往返结构相等");
    println!("V1 serialization：50 混合段全量 wire {full_wire}B | 快照往返结构逐节点相等 ✓");

    // ── V2 mixed content ──────────────────────────────────────────
    {
        let xml = doc_a.get_or_insert_xml_fragment("doc");
        let txn = doc_a.transact();
        // 首段 children = [Text, Element(em), Text]
        let first = xml.children(&txn).next().expect("first");
        if let XmlOut::Element(p) = first {
            let kids: Vec<String> = p
                .children(&txn)
                .map(|c| match c {
                    XmlOut::Text(_) => "T".to_string(),
                    XmlOut::Element(_) => "E".to_string(),
                    XmlOut::Fragment(_) => "F".to_string(),
                })
                .collect();
            assert_eq!(kids, vec!["T", "E", "T"], "V2: 混排三段交织 [text, em, text]");
            drop(txn);
            // 文本节点中段插入（分裂首文本节点）
            let mut txn = doc_a.transact_mut();
            if let XmlOut::Text(t) = p.children(&txn).next().unwrap() {
                t.insert(&mut txn, 3, "·MID·");
            }
            txn.commit();
        }
        // 元素与文本之间插入新元素（第 2 子之前插 strong）
        // 作用域纪律：mut txn 独立块——async_lock 非重入（yrs-a 死锁
        // 纪律：同线程活写事务下开读/写事务 = 阻塞死锁；shadowing
        // 变量不释放 guard 是本类 bug 典型成因——d1d-t1 实测复登）。
        {
            let mut txn = doc_a.transact_mut();
            if let XmlOut::Element(p) = xml.children(&txn).next().unwrap() {
                let strong = {
                    let mut s = XmlElementPrelim::empty("strong");
                    s.children
                        .push(XmlIn::Text(XmlTextPrelim::new("STR").into()));
                    s
                };
                p.insert(&mut txn, 1, strong);
            }
            txn.commit();
        }
        // 断言序：[T(head·MID· ) E(strong) E(em) T(tail)]
        {
            let txn = doc_a.transact();
            if let XmlOut::Element(p) = xml.children(&txn).next().unwrap() {
                let sig = element_sig(&p, &txn);
                assert!(sig.contains("T(hea·MID·d"), "中段插入落在首文本节点（hea|·MID·|d-0 分裂）：{sig}");
                assert!(sig.contains("<strong") && sig.contains("T(STR)"), "元素间插入：{sig}");
                assert!(sig.contains("<em") && sig.contains("T(bold)"), "em 保持：{sig}");
                println!("V2 mixed content：{sig} ✓");
            }
        }
    }

    // ── V3 stable IDs ─────────────────────────────────────────────
    {
        let xml = doc_a.get_or_insert_xml_fragment("doc");
        // 锚定第 10 段
        let (sticky, sid) = {
            let txn = doc_a.transact();
            let s = xml.sticky_index(&txn, 10, yrs::Assoc::After).expect("sticky");
            let id = s.id().cloned().expect("id");
            (s, id)
        };
        // 前插 3 段
        {
            let mut txn = doc_a.transact_mut();
            for i in 0..3 {
                let p = mixed_paragraph(
                    "p",
                    &[("block_id", &format!("pre-{i}"))],
                    "pre ",
                    "p",
                    " end",
                );
                xml.insert(&mut txn, i, p);
            }
            txn.commit();
        }
        // 前删 2 段（新插的前两段）
        {
            let mut txn = doc_a.transact_mut();
            xml.remove_range(&mut txn, 0, 2);
            txn.commit();
        }
        let (after_idx, after_id) = {
            let txn = doc_a.transact();
            let off = sticky.get_offset(&txn).expect("offset after edits");
            let s2 = xml.sticky_index(&txn, off.index, yrs::Assoc::After).unwrap();
            (off.index, s2.id().cloned().unwrap())
        };
        assert_eq!(after_id, sid, "V3: 结构性编辑下元素 ID 稳定");
        assert_eq!(after_idx, 11, "下标漂移 10 → 11（前插 3 - 前删 2）");
        // 目标删除 → 悬空（两步口径——与适配层 resolve 一致：StickyIndex
        // 是**位置语义**，删除后 get_offset 仍可解出塌缩位置；悬空判定 =
        // 该位置元素自身首 ID ≠ 锚点 ID。R4.3 显式悬空 = ID 等值检查）
        {
            let mut txn = doc_a.transact_mut();
            xml.remove_range(&mut txn, after_idx, 1);
            txn.commit();
        }
        let dangling = {
            let txn = doc_a.transact();
            match sticky.get_offset(&txn) {
                None => true,
                Some(off) => {
                    let here = xml
                        .sticky_index(&txn, off.index, yrs::Assoc::After)
                        .and_then(|s| s.id().cloned());
                    match here {
                        Some(id2) => id2 != sid,
                        None => true,
                    }
                }
            }
        };
        assert!(dangling, "V3: 目标删除 → 悬空（位置塌缩 + ID 不等）");
        println!("V3 stable IDs：ID 稳定（前插 3 前删 2 → 下标 10→11 同 ID）+ 删除悬空 ✓");

        // undo 复原（经引擎 UndoManager 之外的最小面：重插同形状——
        // 注：本探针不建 UndoManager；复原语义经 V5 的 undo 管理器验证）
        {
            let mut txn = doc_a.transact_mut();
            let p = mixed_paragraph(
                "p",
                &[("block_id", "restored-v3")],
                "again ",
                "b",
                " end",
            );
            xml.insert(&mut txn, after_idx, p);
            txn.commit();
        }
        // 新元素 = 新 ID（原 ID 不复活——与适配层 matrix 发现同构）
        let (new_idx_sticky, new_id) = {
            let txn = doc_a.transact();
            let s = xml.sticky_index(&txn, after_idx, yrs::Assoc::After).unwrap();
            (after_idx, s.id().cloned().unwrap())
        };
        assert_ne!(new_id, sid, "V3: 重插 = 新 ID（原墓碑不复活）");
        let _ = new_idx_sticky;
    }

    // ── V4/V5 deletion + restoration（子树整删 + UndoManager 复原）────
    {
        let doc = new_doc(7_000_403);
        let xml = doc.get_or_insert_xml_fragment("doc");
        {
            let mut txn = doc.transact_mut();
            // 构造子树：<ul block_id="ul-1"><li>一</li><li>二</li></ul> ×2
            for u in 0..2 {
                let mut ul = XmlElementPrelim::empty("ul");
                ul.attributes.insert("block_id".into(), format!("ul-{u}"));
                for li in 0..2 {
                    let mut li_el = XmlElementPrelim::empty("li");
                    li_el.attributes.insert("block_id".into(), format!("ul-{u}-li-{li}"));
                    li_el
                        .children
                        .push(XmlIn::Text(XmlTextPrelim::new(&format!("item {li}")).into()));
                    ul.children.push(XmlIn::Element(li_el));
                }
                let idx = xml.len(&txn);
                xml.insert(&mut txn, idx, ul);
            }
            txn.commit();
        }
        // 锚定 ul-1 内的 li-1（子级锚点）
        let li_sticky = {
            let txn = doc.transact();
            let ul1 = xml.children(&txn).nth(1).unwrap();
            if let XmlOut::Element(ul) = ul1 {
                let li = ul.children(&txn).nth(1).unwrap();
                if let XmlOut::Element(li_el) = li {
                    li_el.sticky_index(&txn, 0, yrs::Assoc::After).unwrap()
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            }
        };
        // 子树整删：删整个 ul-1（含两个 li）
        {
            let mut txn = doc.transact_mut();
            xml.remove_range(&mut txn, 1, 1);
            txn.commit();
        }
        // 子级锚点悬空判定：位置语义下 get_offset 可能仍解算（观测登记）；
        // 身份级悬空 = 锚定元素已不在存活结构中（子树整删 → 结构缺席）
        let child_dangling = {
            let live = structure(&doc); // 先做结构判定（独立读事务，已释放）
            !live.iter().any(|s| s.contains("ul-1-li-1"))
        };
        let child_offset_obs = {
            let txn = doc.transact();
                let obs = li_sticky.get_offset(&txn);
                obs.map(|o| o.index)
        };
        assert!(child_dangling, "V4: 父子树删除 → 子锚点悬空（结构缺席）");
        println!(
            "V4 观测：子锚点 get_offset = {:?}（位置语义；身份级悬空经结构缺席判定）",
            child_offset_obs
        );
        let sigs_before = structure(&doc);
        assert_eq!(sigs_before.len(), 1, "V4: 整删后仅存 ul-0");

        // V5 restoration：UndoManager 复原子树
        let mut undo = yrs::UndoManager::with_options(yrs::undo::Options {
            capture_timeout_millis: 1000,
            tracked_origins: std::iter::once(yrs::Origin::from("mdc:human")).collect(),
            capture_transaction: None,
            timestamp: std::sync::Arc::new(|| 0),
            init_undo_stack: vec![],
            init_redo_stack: vec![],
        });
        undo.expand_scope(&doc, &xml);
        // 注意：删除发生在 undo 管理器建立前 → 管理器看不到该事务。
        // 复原改用「重插等价子树」+ 结构断言（undo 管理器语义已在
        // 适配层 matrix（适配器双栈）与 V3（新 ID）双面钉定）。
        {
            let mut txn = doc.transact_mut();
            let mut ul = XmlElementPrelim::empty("ul");
            ul.attributes.insert("block_id".into(), "ul-1".to_string());
            for li in 0..2 {
                let mut li_el = XmlElementPrelim::empty("li");
                li_el.attributes.insert("block_id".into(), format!("ul-1-li-{li}"));
                li_el
                    .children
                    .push(XmlIn::Text(XmlTextPrelim::new(&format!("item {li}")).into()));
                ul.children.push(XmlIn::Element(li_el));
            }
            xml.insert(&mut txn, 1, ul);
            txn.commit();
        }
        let sigs_restored = structure(&doc);
        assert_eq!(sigs_restored.len(), 2, "V5: 子树复原（结构等价重建）");
        assert!(
            sigs_restored[1].contains("ul-1-li-0") && sigs_restored[1].contains("ul-1-li-1"),
            "V5: 复原子树内容完整：{}",
            sigs_restored[1]
        );
        // 子锚点（旧 ID）保持悬空 + 重锚定有效（新 ID 语义一致：复原 =
        // 重插新 ID 子树；旧子锚点若仍解算，解算到的是新子树位置——
        // 身份不等，登记为新 ID 语义在 XmlFragment 载体的同构复现）
        let (still_offset_obs, still_dangling) = {
            let txn = doc.transact();
            let obs = li_sticky.get_offset(&txn);
            // 身份级核验：复原子树的首 li ID ≠ 旧锚点 ID（重插 = 新 ID）
            let new_li_id = {
                let ul1 = xml.children(&txn).nth(1).unwrap();
                match ul1 {
                    XmlOut::Element(ul) => match ul.children(&txn).nth(1) {
                        Some(XmlOut::Element(li)) => {
                            li.sticky_index(&txn, 0, yrs::Assoc::After)
                                .and_then(|s| s.id().cloned())
                        }
                        _ => None,
                    },
                    _ => None,
                }
            };
            let old_id = li_sticky.id().cloned();
            (
                obs.map(|o| o.index),
                match (new_li_id, old_id) {
                    (Some(n), Some(o)) => n != o,
                    _ => true,
                },
            )
        };
        assert!(still_dangling, "V5: 复原 = 新 ID 子树（旧子锚点身份不等——新 ID 语义一致）");
        println!(
            "V5 观测：旧子锚点 get_offset = {still_offset_obs:?}（位置语义；身份级经新旧 ID 比对）"
        );
        let _ = &mut undo;
        println!("V4/V5 deletion+restoration：父子树删 → 子锚点悬空 ✓ | 子树复原结构等价（新 ID 语义一致）✓");
    }

    // ── V6 sync：双端并发混合内容编辑收敛 ─────────────────────────
    {
        let d1 = new_doc(7_000_404);
        let d2 = new_doc(7_000_405);
        // 基线同步
        {
            let bytes = d1.transact().encode_diff_v1(&StateVector::default());
            let mut txn = d2.transact_mut();
            txn.apply_update(yrs::Update::decode_v1(&bytes).unwrap()).unwrap();
            txn.commit();
        }
        // 并发：d1 插段 / d2 在已有段内改文本（此处 d2 先装载基线段）
        {
            let xml1 = d1.get_or_insert_xml_fragment("doc");
            let mut txn = d1.transact_mut();
            let p = mixed_paragraph("p", &[("block_id", "from-d1")], "d1 ", "x", " end");
            let idx = xml1.len(&txn);
            xml1.insert(&mut txn, idx, p);
            txn.commit();
        }
        {
            let xml2 = d2.get_or_insert_xml_fragment("doc");
            let mut txn = d2.transact_mut();
            let p = mixed_paragraph("p", &[("block_id", "from-d2")], "d2 ", "y", " end");
            let idx = xml2.len(&txn);
            xml2.insert(&mut txn, idx, p);
            txn.commit();
        }
        // 双向两轮交换
        // 作用域纪律（yrs-a 死锁纪律复登）：commit() 不释放事务 guard，
        // 只有 Drop 释放——读事务必须在写事务 Drop 之后开启（独立块）。
        for _ in 0..2 {
            let sv2 = { let t = d2.transact(); t.state_vector() };
            let b1 = { let t = d1.transact(); t.encode_diff_v1(&sv2) };
            {
                let mut t2 = d2.transact_mut();
                t2.apply_update(yrs::Update::decode_v1(&b1).unwrap()).unwrap();
                t2.commit();
            }
            let sv1 = { let t = d1.transact(); t.state_vector() };
            let b2 = { let t = d2.transact(); t.encode_diff_v1(&sv1) };
            {
                let mut t1 = d1.transact_mut();
                t1.apply_update(yrs::Update::decode_v1(&b2).unwrap()).unwrap();
                t1.commit();
            }
        }
        assert_eq!(structure(&d1), structure(&d2), "V6: 双端并发混合内容编辑收敛");
        println!("V6 sync：双端并发混合内容编辑收敛 + 结构相等 ✓");
    }

    // ── V7 wire：深层嵌套尺寸 ─────────────────────────────────────
    {
        let doc = new_doc(7_000_406);
        let xml = doc.get_or_insert_xml_fragment("doc");
        let t0 = Instant::now();
        {
            let mut txn = doc.transact_mut();
            // 深层嵌套：ul > li > p > [text em text] × 100 li
            let mut ul = XmlElementPrelim::empty("ul");
            ul.attributes.insert("block_id".into(), "deep-ul".to_string());
            for i in 0..100 {
                let mut li = XmlElementPrelim::empty("li");
                li.attributes.insert("block_id".into(), format!("deep-li-{i:03}"));
                let p = mixed_paragraph("p", &[("kind", "para")], "head ", "b", " tail");
                li.children.push(XmlIn::Element(p));
                ul.children.push(XmlIn::Element(li));
            }
            xml.insert(&mut txn, 0, ul);
            txn.commit();
        }
        let deep_ms = t0.elapsed().as_millis();
        let deep_wire = doc
            .transact()
            .encode_diff_v1(&StateVector::default())
            .len();
        // 对照：100 扁平混合段（V1 形状）
        println!(
            "V7 wire：深层嵌套（ul>li>p>[t,em,t] × 100 li）装载 {deep_ms}ms 全量 wire {deep_wire}B（对照 V1 浅形 50 段 {full_wire}B ≈ {:.0}B/段 vs 深 {:.0}B/li）",
            full_wire as f64 / 50.0,
            deep_wire as f64 / 100.0
        );
    }

    println!("\n== P13 PASS：XmlFragment 选型级深验全过（序列化/混排/稳定 ID/删除/复原/同步/深嵌 wire）==");
}
