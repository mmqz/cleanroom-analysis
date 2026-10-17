//! 探针 11 结构载体选型证据轮（d1c-t4 前半）：XmlFragment vs
//! Array-of-Map（yrs-b §7 启动条件③——结构载体选型裁决的证据输入）。
//!
//! 口径：直接 years 评估探针（p1-p7 先例——载体问题是适配层内部设计题，
//! 证据采集在 spike 层，结论呈报后由适配层实施采用）。两载体承载
//! **同一内容**（1000 混合块 + 嵌套结构 + 行内混排），对照六维：
//!
//! C1 wire 体积：同内容全量 update 字节数（skip_gc 口径）；
//! C2 装载时延：批量装载 1000 块；
//! C3 读面：全量遍历 block_id（1000 块）时延与 API 形状；
//! C4 结构操作：中段插入 / 中段删除（时延 + 正确性）；
//! C5 锚点面：StickyIndex 锚定中段块 → 结构性编辑 → 保持（两载体）；
//! C6 树性表达：嵌套结构（列表含子项）+ 行内混排（段落内粗体 run）——
//!    XmlFragment 原生树/混排 vs Array-of-Map 扁平 depth 字段/文本
//!    单字段（表达力差异实证——mdcore BlockTree 映射直接性）。

use std::time::Instant;

use yrs::updates::decoder::Decode;

use yrs::types::xml::XmlIn;
use yrs::{
    Assoc, Array, ClientID, Doc, GetString, IndexedSequence, In, Map, MapPrelim,
    Options as DocOptions, Out, ReadTxn, StateVector, Transact, Xml, XmlElementPrelim,
    XmlFragment, XmlTextPrelim,
};

fn vm(field: &str) -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{}:", field)) {
            return rest.trim().trim_end_matches(" kB").parse().unwrap();
        }
    }
    0
}

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true; // B1.1 口径对齐（对照公平性：两载体同 skip_gc）
    Doc::with_options(opts)
}

const N: usize = 1_000;

fn block_id(i: usize) -> String {
    format!("blk-{:05}", i)
}

fn block_text(i: usize) -> String {
    format!("content of block {} with some text payload", i)
}

// ── 载体 A：Array-of-Map（适配层 interim 形状）────────────────────

fn aom_load(doc: &Doc) {
    let array = doc.get_or_insert_array("blocks");
    let mut txn = doc.transact_mut();
    for i in 0..N {
        let map = MapPrelim::from([
            ("block_id", In::Any(block_id(i).into())),
            ("kind", In::Any(if i % 10 == 0 { "heading" } else { "paragraph" }.into())),
            ("text", In::Any(block_text(i).into())),
        ]);
        let idx = array.len(&txn);
        array.insert(&mut txn, idx, map);
    }
    txn.commit();
}

fn aom_read_all(doc: &Doc) -> Vec<String> {
    let array = doc.get_or_insert_array("blocks");
    let txn = doc.transact();
    let mut ids = Vec::with_capacity(array.len(&txn) as usize);
    for i in 0..array.len(&txn) {
        if let Some(Out::YMap(m)) = array.get(&txn, i) {
            if let Some(Out::Any(yrs::Any::String(id))) = m.get(&txn, "block_id") {
                ids.push(id.to_string());
            }
        }
    }
    ids
}

// ── 载体 B：XmlFragment（元素 = 块，tag = kind，属性 = 元数据）────

fn xml_load(doc: &Doc) {
    let xml = doc.get_or_insert_xml_fragment("doc");
    let mut txn = doc.transact_mut();
    for i in 0..N {
        let tag = if i % 10 == 0 { "heading" } else { "paragraph" };
        let mut elem = XmlElementPrelim::empty(tag);
        elem.attributes
            .insert("block_id".into(), block_id(i));
        elem.attributes.insert("level".into(), "2".to_string());
        elem.children.push(XmlIn::Text(XmlTextPrelim::new(block_text(i)).into()));
        let idx = xml.len(&txn);
        xml.insert(&mut txn, idx, elem);
    }
    txn.commit();
}

fn xml_read_all(doc: &Doc) -> Vec<String> {
    let xml = doc.get_or_insert_xml_fragment("doc");
    let txn = doc.transact();
    let mut ids = Vec::with_capacity(xml.len(&txn) as usize);
    for child in xml.children(&txn) {
        if let yrs::XmlOut::Element(e) = child {
            if let Some(Out::Any(yrs::Any::String(id))) = e.get_attribute(&txn, "block_id") {
                ids.push(id.to_string());
            }
        }
    }
    ids
}

fn full_update_len(doc: &Doc) -> usize {
    doc.transact()
        .encode_diff_v1(&StateVector::decode_v1(&[0u8]).unwrap())
        .len()
}

fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() {
    println!("P11 结构载体对照（{} 块，两载体同内容同 skip_gc）", N);

    // ── C1+C2：装载 + wire 体积 ────────────────────────────────────
    let doc_a = new_doc(11_000_001);
    let rss0 = vm("VmRSS");
    let t0 = Instant::now();
    aom_load(&doc_a);
    let aom_load_ms = ms(t0.elapsed());
    let aom_rss = vm("VmRSS") as i64 - rss0 as i64;
    let aom_bytes = full_update_len(&doc_a);

    let doc_b = new_doc(11_000_002);
    let rss0 = vm("VmRSS");
    let t0 = Instant::now();
    xml_load(&doc_b);
    let xml_load_ms = ms(t0.elapsed());
    let xml_rss = vm("VmRSS") as i64 - rss0 as i64;
    let xml_bytes = full_update_len(&doc_b);

    println!(
        "C1/C2 装载与 wire：Array-of-Map {:.0}ms/ΔRSS {:+}kB/update {}B | XmlFragment {:.0}ms/ΔRSS {:+}kB/update {}B（{:.2}×）",
        aom_load_ms,
        aom_rss,
        aom_bytes,
        xml_load_ms,
        xml_rss,
        xml_bytes,
        xml_bytes as f64 / aom_bytes as f64
    );

    // ── C3：读面遍历 ─────────────────────────────────────────────
    let t0 = Instant::now();
    let ids_a = aom_read_all(&doc_a);
    let aom_read_ms = ms(t0.elapsed());
    let t0 = Instant::now();
    let ids_b = xml_read_all(&doc_b);
    let xml_read_ms = ms(t0.elapsed());
    assert_eq!(ids_a.len(), N);
    assert_eq!(ids_b.len(), N);
    assert_eq!(ids_a, ids_b, "两载体 block_id 序列一致（同内容）");
    println!(
        "C3 全量遍历：Array-of-Map {:.2}ms（索引 get 链）| XmlFragment {:.2}ms（children 迭代器）",
        aom_read_ms, xml_read_ms
    );

    // ── C4：中段插入 / 删除 ──────────────────────────────────────
    // Array-of-Map：定位 500 → insert
    let t0 = Instant::now();
    {
        let array = doc_a.get_or_insert_array("blocks");
        let mut txn = doc_a.transact_mut();
        let map = MapPrelim::from([
            ("block_id", In::Any("ins-mid".into())),
            ("kind", In::Any("paragraph".into())),
            ("text", In::Any("mid insert".into())),
        ]);
        array.insert(&mut txn, 500, map);
        txn.commit();
    }
    let aom_ins_ms = ms(t0.elapsed());
    // XmlFragment：同位插入
    let t0 = Instant::now();
    {
        let xml = doc_b.get_or_insert_xml_fragment("doc");
        let mut txn = doc_b.transact_mut();
        let mut elem = XmlElementPrelim::empty("paragraph");
        elem.attributes.insert("block_id".into(), "ins-mid".to_string());
        elem.children
            .push(XmlTextPrelim::new("mid insert").into());
        xml.insert(&mut txn, 500, elem);
        txn.commit();
    }
    let xml_ins_ms = ms(t0.elapsed());
    assert_eq!(aom_read_all(&doc_a)[500], "ins-mid");
    assert_eq!(xml_read_all(&doc_b)[500], "ins-mid");
    println!(
        "C4 中段插入@500：Array-of-Map {:.2}ms | XmlFragment {:.2}ms（两载体插入后 block_id 序列仍一致）",
        aom_ins_ms, xml_ins_ms
    );
    // 中段删除
    {
        let array = doc_a.get_or_insert_array("blocks");
        let mut txn = doc_a.transact_mut();
        array.remove(&mut txn, 500);
        txn.commit();
        let xml = doc_b.get_or_insert_xml_fragment("doc");
        let mut txn = doc_b.transact_mut();
        xml.remove(&mut txn, 500);
        txn.commit();
    }
    assert_eq!(aom_read_all(&doc_a), xml_read_all(&doc_b), "删除后两载体仍一致");

    // ── C5：锚点面（两载体 StickyIndex 保持）─────────────────────
    // Array-of-Map：锚 blk-00500（现下标 500）
    let aom_anchor = {
        let array = doc_a.get_or_insert_array("blocks");
        let txn = doc_a.transact();
        array
            .sticky_index(&txn, 500, Assoc::After)
            .unwrap()
            .id()
            .cloned()
            .unwrap()
    };
    // XmlFragment：同位锚
    let xml_anchor = {
        let xml = doc_b.get_or_insert_xml_fragment("doc");
        let txn = doc_b.transact();
        xml.sticky_index(&txn, 500, Assoc::After)
            .unwrap()
            .id()
            .cloned()
            .unwrap()
    };
    // 结构性编辑：两侧各删 3 块（< 500）
    for i in 100..103 {
        let array = doc_a.get_or_insert_array("blocks");
        let mut txn = doc_a.transact_mut();
        array.remove(&mut txn, i);
        txn.commit();
        let xml = doc_b.get_or_insert_xml_fragment("doc");
        let mut txn = doc_b.transact_mut();
        xml.remove(&mut txn, i);
        txn.commit();
    }
    let aom_off = yrs::StickyIndex::from_id(aom_anchor.clone(), Assoc::After)
        .get_offset(&doc_a.transact())
        .map(|o| o.index);
    let xml_off = yrs::StickyIndex::from_id(xml_anchor.clone(), Assoc::After)
        .get_offset(&doc_b.transact())
        .map(|o| o.index);
    assert_eq!(aom_off, Some(497), "A 载体锚点漂移 500-3=497");
    assert_eq!(xml_off, Some(497), "B 载体锚点漂移 500-3=497");
    // 锚定块身份不变
    assert_eq!(aom_read_all(&doc_a)[497], "blk-00500");
    assert_eq!(xml_read_all(&doc_b)[497], "blk-00500");
    println!("C5 锚点保持：两载体 StickyIndex 同漂移同复原（500→497，block_id 不变）");

    // ── C6：树性表达（嵌套 + 行内混排）───────────────────────────
    // XmlFragment：原生树——<ul><li>文本<b>粗体</b>尾</li><li>子项</li></ul>
    let doc_c = new_doc(11_000_003);
    {
        let xml = doc_c.get_or_insert_xml_fragment("doc");
        let mut txn = doc_c.transact_mut();
        let li1 = {
            let mut e = XmlElementPrelim::empty("list-item");
            e.attributes.insert("block_id".into(), "li-1".to_string());
            e.children.push(XmlTextPrelim::new("文本").into());
            e.children.push({
                let mut b = XmlElementPrelim::empty("bold");
                b.children.push(XmlTextPrelim::new("粗体").into());
                XmlIn::Element(b)
            });
            e.children.push(XmlTextPrelim::new("尾").into());
            e
        };
        let li2 = {
            let mut e = XmlElementPrelim::empty("list-item");
            e.attributes.insert("block_id".into(), "li-2".to_string());
            e.children.push(XmlTextPrelim::new("子项").into());
            e
        };
        let mut ul = XmlElementPrelim::empty("bullet-list");
        ul.attributes.insert("block_id".into(), "ul-1".to_string());
        ul.children.push(XmlIn::Element(li1));
        ul.children.push(XmlIn::Element(li2));
        xml.push_back(&mut txn, ul);
        txn.commit();
    }
    // 遍历：ul-1 → children → li-1（含行内混排三段）/li-2
    let (ul_children, li1_string) = {
        let xml = doc_c.get_or_insert_xml_fragment("doc");
        let txn = doc_c.transact();
        let ul = match xml.first_child() {
            Some(yrs::XmlOut::Element(e)) if e.tag().as_ref() == "bullet-list" => e,
            other => panic!("首子非 bullet-list: {:?}", other.map(|_| ())),
        };
        let kids: Vec<String> = ul
            .children(&txn)
            .filter_map(|c| match c {
                yrs::XmlOut::Element(e) => e.get_attribute(&txn, "block_id").map(|v| match v {
                    Out::Any(yrs::Any::String(s)) => s.to_string(),
                    _ => "?".into(),
                }),
                _ => None,
            })
            .collect();
        let li1 = match ul.children(&txn).next() {
            Some(yrs::XmlOut::Element(e)) => e,
            _ => panic!("li-1 缺失"),
        };
        (kids, li1.get_string(&txn))
    };
    assert_eq!(ul_children, vec!["li-1", "li-2"], "嵌套子项遍历");
    assert!(
        li1_string.contains("文本") && li1_string.contains("<bold>粗体</bold>") && li1_string.ends_with("尾</list-item>"),
        "行内混排结构在场（文本+粗体 run+尾文本交织）：{}",
        li1_string
    );
    let xml_c_bytes = full_update_len(&doc_c);

    // Array-of-Map 对照：同结构需扁平化 depth 字段 + 行内混排无原生表达
    // （文本单字段；粗体 run 需另设 marks 子结构——表达力差异实证）
    let doc_d = new_doc(11_000_004);
    {
        let array = doc_d.get_or_insert_array("blocks");
        let mut txn = doc_d.transact_mut();
        let mk = |id: &str, kind: &str, depth: u32, text: &str| {
            MapPrelim::from([
                ("block_id", In::Any(id.into())),
                ("kind", In::Any(kind.into())),
                ("depth", In::Any((depth as i64).into())),
                ("text", In::Any(text.into())),
            ])
        };
        array.insert(&mut txn, 0, mk("ul-1", "bullet-list", 0, ""));
        array.insert(&mut txn, 1, mk("li-1", "list-item", 1, "文本粗体尾"));
        array.insert(&mut txn, 2, mk("li-2", "list-item", 1, "子项"));
        txn.commit();
    }
    let aom_c_bytes = full_update_len(&doc_d);
    println!(
        "C6 树性表达：XmlFragment 原生（嵌套遍历 [li-1,li-2] + 行内混排三段交织：{}）| Array-of-Map 扁平 depth 字段（混排退化为单文本字段，marks 需另设子结构）；同结构 wire {}B vs {}B",
        li1_string, xml_c_bytes, aom_c_bytes
    );

    // ── 汇总 ─────────────────────────────────────────────────────
    println!(
        "P11 PASS: 载体六维证据采集完成——wire {:.2}× / 装载 {:.1}ms vs {:.1}ms / 读 {:.2}ms vs {:.2}ms / 结构操作一致 / 锚点双载体同保持 / 树性表达力差异实证（XML 原生树+混排，AoM 扁平+单文本）",
        xml_bytes as f64 / aom_bytes as f64,
        aom_load_ms,
        xml_load_ms,
        aom_read_ms,
        xml_read_ms
    );
}
