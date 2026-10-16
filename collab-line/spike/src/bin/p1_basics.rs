//! 探针 1 基本面：四类型根操作 / 更新 v1·v2 编码往返 / SV 差量 / 子文档 guid 占位。
//! 全部断言过 = PASS（探针纪律：零 panic 零 unwrap 失败）。
//! API 校准记录：encode_diff_v1(&SV::default()) = 全量状态（Y.encodeStateAsUpdate 同义）；
//! insert/get_string/len 均为 trait 方法（Text/Map/Array/XmlFragment/GetString/ReadTxn）。

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::types::ToJson;
use yrs::{
    Array, Doc, GetString, In, Map, Number, ReadTxn, StateVector, Text, Transact, Update,
    XmlFragment, XmlTextPrelim,
};

fn full_v1(doc: &Doc) -> Vec<u8> {
    doc.transact().encode_diff_v1(&StateVector::default())
}

fn main() {
    let doc = Doc::with_client_id(1);
    let text = doc.get_or_insert_text("md-text");
    let map = doc.get_or_insert_map("md-meta");
    let array = doc.get_or_insert_array("md-blocks");
    let xml = doc.get_or_insert_xml_fragment("md-xml");

    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "hello ");
        text.insert(&mut txn, 6, "world");
        text.remove_range(&mut txn, 0, 1);
        map.insert(&mut txn, "kind", "paragraph");
        map.insert(&mut txn, "level", 2i64);
        array.insert(&mut txn, 0, "block-a");
        array.insert(&mut txn, 1, "block-b");
        xml.insert(&mut txn, 0, XmlTextPrelim::new("xml text"));
    }

    let txn = doc.transact();
    assert_eq!(text.get_string(&txn), "ello world", "text 编辑面");
    let level = map
        .get(&txn, "level")
        .map(|o| o.to_json(&txn))
        .expect("map.get(level)");
    assert_eq!(level, yrs::Any::Number(Number::Int(2)), "map 数值面");
    assert_eq!(array.len(&txn), 2, "array 长度面");
    assert!(xml.get_string(&txn).contains("xml text"), "xmlFragment+XmlText");
    drop(txn);

    // 更新往返：v1 与 v2 两格式
    let bytes_v1 = full_v1(&doc);
    let bytes_v2 = doc.transact().encode_diff_v2(&StateVector::default());
    for (label, bytes, v2) in [
        ("v1", bytes_v1.clone(), false),
        ("v2", bytes_v2.clone(), true),
    ] {
        let decoded = if v2 {
            Update::decode_v2(&bytes).unwrap()
        } else {
            Update::decode_v1(&bytes).unwrap()
        };
        let fresh = Doc::with_client_id(9);
        fresh.transact_mut().apply_update(decoded).expect("apply");
        let s = fresh.get_or_insert_text("md-text").get_string(&fresh.transact());
        assert_eq!(s, "ello world", "{} 格式跨 doc 往返等值", label);
    }

    // SV 差量：空 SV 差量 = 全量；已同步 SV 差量 = 近空
    let empty_sv = StateVector::default();
    let full_diff = doc.transact().encode_diff_v1(&empty_sv);
    let self_sv_bytes = doc.transact().state_vector().encode_v1();
    let self_diff = doc
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&self_sv_bytes).unwrap());
    assert!(
        full_diff.len() > self_diff.len(),
        "全量差量 {} > 自差量 {}",
        full_diff.len(),
        self_diff.len()
    );

    // 子文档：父文档引用为 guid 占位（collab-a §1 subdoc 语义）
    let sub = Doc::new();
    sub.get_or_insert_text("sub")
        .insert(&mut sub.transact_mut(), 0, "inner");
    let parent = Doc::new();
    let arr = parent.get_or_insert_array("children"); // 先取引用（锁在方法内释）
    {
        let mut pt = parent.transact_mut(); // 再开写事务（避免写写重入死锁）
        arr.insert(&mut pt, 0, In::Doc(sub));
    }
    let parent_update = full_v1(&parent);
    let restored = Update::decode_v1(&parent_update).unwrap();
    let p2 = Doc::new();
    p2.transact_mut().apply_update(restored).expect("父文档 guid 引用往返");

    println!(
        "P1 PASS: 四类型根({}B update v1) + v1/v2 往返 + SV 差量 + 子文档 guid 占位",
        bytes_v1.len()
    );
}
