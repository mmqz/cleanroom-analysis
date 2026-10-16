//! 探针 3a 互操作写出端：Rust 侧构造文档（text + map + array），写 update v1 到
//! out/rust_to_js.bin；JS 侧（npm yjs 原版）将读回并 apply。

use std::fs;
use std::path::Path;

use yrs::updates::encoder::Encode;
use yrs::{Array, Doc, Map, ReadTxn, StateVector, Text, Transact};

fn main() {
    let out_dir = Path::new("out");
    fs::create_dir_all(out_dir).unwrap();

    let doc = Doc::with_client_id(42);
    let text = doc.get_or_insert_text("interop");
    let map = doc.get_or_insert_map("meta");
    let array = doc.get_or_insert_array("blocks");
    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "hello from rust ");
        text.insert(&mut txn, 16, "yrs 0.28");
        map.insert(&mut txn, "engine", "yrs");
        map.insert(&mut txn, "answer", 42i64);
        array.insert(&mut txn, 0, "para-1");
        array.insert(&mut txn, 1, "para-2");
    }
    let bytes = doc.transact().encode_diff_v1(&StateVector::default());
    fs::write(out_dir.join("rust_to_js.bin"), &bytes).unwrap();

    // 同时写 SV（供 JS 侧练习 encodeStateVector 差量流）
    let sv = doc.transact().state_vector().encode_v1();
    fs::write(out_dir.join("rust_sv.bin"), &sv).unwrap();

    println!("P3a PASS: Rust→JS 更新写出 ({} bytes) + SV ({} bytes)", bytes.len(), sv.len());
}
