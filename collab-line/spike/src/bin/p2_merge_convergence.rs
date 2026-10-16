//! 探针 2 双端合并收敛：两 client 并发冲突编辑（同 text 位置 / 同 map 键 / 同 array 槽）
//! 双向交换 update 后，两端状态必须收敛等值；三方迟到加入者经 SV 差量同步后同态。

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::types::ToJson;
use yrs::{Array, Doc, GetString, Map, ReadTxn, StateVector, Text, Transact, Update};

fn full_v1(doc: &Doc) -> Vec<u8> {
    doc.transact().encode_diff_v1(&StateVector::default())
}

fn exchange(a: &Doc, b: &Doc) {
    let ua = full_v1(a);
    let ub = full_v1(b);
    a.transact_mut().apply_update(Update::decode_v1(&ub).unwrap()).unwrap();
    b.transact_mut().apply_update(Update::decode_v1(&ua).unwrap()).unwrap();
}

fn main() {
    let d1 = Doc::with_client_id(1);
    let d2 = Doc::with_client_id(2);
    let t1 = d1.get_or_insert_text("body");
    let t2 = d2.get_or_insert_text("body");
    let m1 = d1.get_or_insert_map("meta");
    let m2 = d2.get_or_insert_map("meta");

    // 阶段 0：共同基线
    t1.insert(&mut d1.transact_mut(), 0, "base");
    exchange(&d1, &d2);
    assert_eq!(t2.get_string(&d2.transact()), "base", "基线同步");

    // 阶段 1：并发冲突（同位置插入 / 同键写入 / 同槽插入）
    t1.insert(&mut d1.transact_mut(), 4, "-from-1"); // "base-from-1"
    t2.insert(&mut d2.transact_mut(), 4, "-from-2"); // "base-from-2"
    m1.insert(&mut d1.transact_mut(), "owner", "client1");
    m2.insert(&mut d2.transact_mut(), "owner", "client2");
    let a1 = d1.get_or_insert_array("blocks");
    let a2 = d2.get_or_insert_array("blocks");
    a1.insert(&mut d1.transact_mut(), 0, "block-A");
    a2.insert(&mut d2.transact_mut(), 0, "block-B");

    // 交换前：两端状态不同（必然）
    let s1_before = t1.get_string(&d1.transact());
    let s2_before = t2.get_string(&d2.transact());
    assert_ne!(s1_before, s2_before, "交换前应发散");

    exchange(&d1, &d2);

    // 收敛断言：两端可见状态完全一致
    let s1 = t1.get_string(&d1.transact());
    let s2 = t2.get_string(&d2.transact());
    assert_eq!(s1, s2, "并发 text 插入收敛（实得 {} vs {}）", s1, s2);
    let o1 = m1
        .get(&d1.transact(), "owner")
        .map(|o| o.to_json(&d1.transact()).to_string());
    let o2 = m2
        .get(&d2.transact(), "owner")
        .map(|o| o.to_json(&d2.transact()).to_string());
    assert_eq!(o1, o2, "并发 map 同键收敛（LWW 平局由 clientID 定序）");
    assert_eq!(
        a1.len(&d1.transact()),
        a2.len(&d2.transact()),
        "并发 array 同槽收敛（长度）"
    );
    let v1 = a1
        .get(&d1.transact(), 0)
        .map(|o| o.to_json(&d1.transact()).to_string());
    let v2 = a2
        .get(&d2.transact(), 0)
        .map(|o| o.to_json(&d2.transact()).to_string());
    assert_eq!(v1, v2, "并发 array 同槽收敛（元素）");

    // 编码面观察（非判据）：块布局随整合历史不同——d1 本端拆分 vs d2 合并整合，
    // 状态等值但全量编码字节可异。正确收敛判据 = 幂等复交换（下）。
    // 该发现支持「权威快照单点编码」设计（快照由单一整合者产出）。
    let u1 = full_v1(&d1);
    let u2 = full_v1(&d2);
    println!(
        "P2 NOTE: 状态收敛成立但全量编码 {}B vs {}B（块布局随整合历史异形——登记为发现）",
        u1.len(),
        u2.len()
    );

    // 幂等复交换：互灌对方全量 update，状态必须不变且保持等值
    d1.transact_mut().apply_update(Update::decode_v1(&u2).unwrap()).unwrap();
    d2.transact_mut().apply_update(Update::decode_v1(&u1).unwrap()).unwrap();
    let r1 = t1.get_string(&d1.transact());
    let r2 = t2.get_string(&d2.transact());
    assert_eq!(r1, r2, "复交换后两端仍收敛");
    assert_eq!(r1, s1, "复交换幂等（d1 状态不变）");
    assert_eq!(r2, s2, "复交换幂等（d2 状态不变）");

    // 阶段 2：第三方迟到加入（SV 差量同步 = SyncStep1/2 模拟）
    let d3 = Doc::with_client_id(3);
    let sv3 = d3.transact().state_vector().encode_v1();
    let diff_for_3 = d1
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&sv3).unwrap());
    d3.transact_mut()
        .apply_update(Update::decode_v1(&diff_for_3).unwrap())
        .unwrap();
    let s3 = d3.get_or_insert_text("body").get_string(&d3.transact());
    assert_eq!(s3, s1, "迟到者经 SV 差量同步达到同态");
    // d3 反向增量回传（d1 只应收到 d3 的新内容，d1 状态不动）
    let sv1 = d1.transact().state_vector().encode_v1();
    let diff_for_1 = d3
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&sv1).unwrap());
    let before = d1.get_or_insert_text("body").get_string(&d1.transact());
    d1.transact_mut()
        .apply_update(Update::decode_v1(&diff_for_1).unwrap())
        .unwrap();
    let after = d1.get_or_insert_text("body").get_string(&d1.transact());
    assert_eq!(before, after, "d3 无新内容时回传差量为空（幂等）");

    println!(
        "P2 PASS: 双端并发冲突收敛（幂等复交换全过，全量 {}B）+ 三方 SV 差量同步 + 幂等空差量",
        u1.len()
    );
}
