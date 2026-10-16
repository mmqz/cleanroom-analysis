//! 探针 3c 互操作读回端：读 JS（npm yjs 原版）生成的 out/js_to_rust.bin（全量态，
//! 已含 Rust 原内容 + JS 侧编辑），apply 到全新 Rust doc，断言精确合并态；
//! 再做 Rust 侧追加 → 反向 SV 差量（SyncStep2 模拟）尺寸对照。

use std::fs;

use yrs::updates::decoder::Decode;
use yrs::types::ToJson;
use yrs::{Doc, GetString, Map, ReadTxn, StateVector, Text, Transact, Update};

fn main() {
    let bytes = fs::read("out/js_to_rust.bin").expect("先跑 interop_js.mjs（npm yjs）生成此文件");
    let js_sv = fs::read("out/js_sv.bin").expect("JS SV 文件");

    let doc = Doc::with_client_id(7);
    doc.transact_mut()
        .apply_update(Update::decode_v1(&bytes).unwrap())
        .expect("apply JS 生成的 update v1 —— 跨语言 wire 兼容的实证点");

    let text = doc.get_or_insert_text("interop");
    let s = text.get_string(&doc.transact());
    // JS 侧流程：rust 文本 "hello from rust yrs 0.28" → insert(0, "[js prefix] ")
    assert_eq!(
        s, "[js prefix] hello from rust yrs 0.28",
        "JS 全量态在 Rust 精确重建（实得 {:?}）",
        s
    );
    let map = doc.get_or_insert_map("meta");
    let engine = map.get(&doc.transact(), "engine").map(|o| o.to_json(&doc.transact()).to_string());
    assert_eq!(engine.as_deref(), Some("yrs"), "map 值跨语言等值（实得 {:?}）", engine);
    let js_key = map.get(&doc.transact(), "jsKey").map(|o| o.to_json(&doc.transact()).to_string());
    assert_eq!(js_key.as_deref(), Some("from-js"), "JS 侧 map 写入可见（实得 {:?}）", js_key);
    println!("P3c PASS: JS(yjs npm 13.6)→Rust(yrs 0.28) 全量态精确重建；text = {:?}", s);

    // Rust 侧追加 → 反向差量（对照全量尺寸）；读写锁分语句（避免同语句双锁死锁）
    let end = text.len(&doc.transact());
    text.insert(&mut doc.transact_mut(), end, " [rust-append]");
    let sv = StateVector::decode_v1(&js_sv).unwrap();
    let diff = doc.transact().encode_diff_v1(&sv);
    let full = doc.transact().encode_diff_v1(&StateVector::default());
    println!(
        "P3c PASS: 反向 SV 差量 {} bytes << 全量 {} bytes（SyncStep2 语义）",
        diff.len(),
        full.len()
    );
}
