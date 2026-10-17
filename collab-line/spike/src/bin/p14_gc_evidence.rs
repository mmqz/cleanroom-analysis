//! p14_gc_evidence —— skip_gc 全编队不变量的 wire 级可检测性实证探针。
//!
//! 背景（用户加固指令）：「Enforce or explicitly validate the all-peers
//! skip_gc invariant at the adapter boundary where possible. Fail closed on
//! detectable mismatches; document what cannot be enforced.」
//!
//! 待实证问题：GC 型对端（skip_gc=false 构造 + txn.gc()）产出的 update
//! 在 wire 上是否携带**公开 API 可检测**的 GC 证据（Block::GC 块 /
//! Skip 块 / delete_set 形状），使适配层能在 apply 前识别「来源端已 GC」？
//!
//! 四源对照（同形状同操作序列）：
//!   A. skip_gc=true 源（干净对照——预期零证据）
//!   B. skip_gc=false + 默认 GC 源（未显式 gc——事务 commit 期自动 GC）
//!   C. skip_gc=false + 显式 txn.gc(None) 源
//!   D. skip_gc=true + 显式 gc 源（适配层 maintenance_gc 引擎面形状）
//!
//! 检测面（全部经 yrs 0.28 公开 API，不触私有字段）：
//!   - update.insertions(true) vs insertions(false) 的 IdSet 差
//!   - update.state_vector()（连续上界——遇 Skip 停止计数）
//!   - update.state_vector_lower()（首块下界）
//!   - update.delete_set()（IdSet）
//!   - update.extends()（信息量面）
//!
//! 口径：release 跑（`cargo run --release --bin p14_gc_evidence`）；
//! 全部断言本机实测，输出逐源对照表。

use std::collections::BTreeMap;

use yrs::updates::decoder::Decode;
use yrs::{
    ClientID, Doc, GetString, IdSet, Options, ReadTxn, StateVector, Text, Transact,
};

/// 构造同形状文档：插入 100 段小串（client 固定），删除第 10..21 段
/// （制造墓碑/GC 面），可选显式 gc。
fn build_source(client: u64, skip_gc: bool, explicit_gc: bool) -> Doc {
    let mut opts = Options::with_client_id(ClientID::new(client));
    opts.skip_gc = skip_gc;
    let doc = Doc::with_options(opts);
    let text = doc.get_or_insert_text("text");
    // 100 笔小串（每事务一笔 = 每笔一个 item——独立可删）
    for i in 0..100u32 {
        let mut txn = doc.transact_mut();
        let pos = text.len(&txn);
        text.insert(&mut txn, pos, &format!("s{:03}#", i));
        txn.commit();
    }
    // 删除 s010..s020 共 11 段（每段 5 字符 → 字符区间 [50,105)）
    {
        let mut txn = doc.transact_mut();
        text.remove_range(&mut txn, 50, 55);
        txn.commit();
    }
    if explicit_gc {
        let mut txn = doc.transact_mut();
        txn.gc(None);
        txn.commit();
    }
    doc
}

/// IdSet 逐区间字符串化（u32 时钟区间序列，供对照表）。
fn ranges_of(idset: &IdSet) -> String {
    let mut parts = Vec::new();
    for (client, ranges) in idset.iter() {
        for r in ranges.iter() {
            parts.push(format!("c{}:[{}..{}]", client.get(), r.start, r.end));
        }
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(" ")
    }
}

fn sv_of(sv: &StateVector) -> String {
    let mut s = Vec::new();
    for (c, clock) in sv.iter() {
        s.push(format!("c{}:{}", c.get(), clock));
    }
    if s.is_empty() {
        "-".to_string()
    } else {
        s.join(" ")
    }
}

fn main() {
    println!("== p14_gc_evidence：GC 证据 wire 可检测性实证 ==\n");

    let cases: Vec<(&str, u64, bool, bool)> = vec![
        ("A skip_gc=true 干净源", 7_100_001, true, false),
        ("B skip_gc=false 默认GC", 7_100_002, false, false),
        ("C skip_gc=false 显式gc", 7_100_003, false, true),
        ("D skip_gc=true 显式gc", 7_100_004, true, true),
    ];

    let mut summary: BTreeMap<&str, String> = BTreeMap::new();

    // 干净源（skip_gc=true 同形状）全量 wire 尺寸基线——中继传播对照用
    let clean_doc = build_source(7_100_099, true, false);
    let clean_full_len = clean_doc
        .transact()
        .encode_diff_v1(&StateVector::default())
        .len();

    for (label, client, skip_gc, explicit_gc) in &cases {
        let doc = build_source(*client, *skip_gc, *explicit_gc);
        let bytes = doc.transact().encode_diff_v1(&StateVector::default());
        let update = yrs::Update::decode_v1(&bytes).expect("全量 diff 解码");

        let ins_all = update.insertions(true);
        let ins_live = update.insertions(false);
        let sv_upper = update.state_vector();
        let sv_lower = update.state_vector_lower();
        let ds = update.delete_set().clone();

        // 差集：ins_all − ins_live（候选 GC/墓碑面）
        let mut cand = ins_all.clone();
        cand.diff_with(&ins_live);

        println!("── {}（wire {}B）", label, bytes.len());
        println!("   insertions(true)  : {}", ranges_of(&ins_all));
        println!("   insertions(false) : {}", ranges_of(&ins_live));
        println!("   差集(候选GC/墓碑) : {}", ranges_of(&cand));
        println!("   state_vector 连续 : {}", sv_of(&sv_upper));
        println!("   state_vector_lower: {}", sv_of(&sv_lower));
        println!("   delete_set        : {}", ranges_of(&ds));
        println!(
            "   extends(empty sv) : {}",
            update.extends(&StateVector::default())
        );

        // 应用到全新 skip_gc 接收端：内容与后续健康面
        let mut ropts = Options::with_client_id(ClientID::new(7_200_001));
        ropts.skip_gc = true;
        let recv = Doc::with_options(ropts);
        {
            let mut txn = recv.transact_mut();
            txn.apply_update(yrs::Update::decode_v1(&bytes).expect("重解码"))
                .expect("接收端 apply");
            txn.commit();
        }
        let recv_text = recv.get_or_insert_text("text");
        let got = recv_text.get_string(&recv.transact());
        let src_text = doc.get_or_insert_text("text").get_string(&doc.transact());
        println!(
            "   接收端内容一致     : {}（src {} 字符 / recv {} 字符）",
            got == src_text,
            src_text.chars().count(),
            got.chars().count()
        );
        // 接收端后续可继续编辑（健康面）
        {
            let mut txn = recv.transact_mut();
            let pos = recv_text.len(&txn);
            recv_text.insert(&mut txn, pos, "AFTER");
            txn.commit();
        }
        // 中继传播面：接收端自身再编码全量 diff——若证据（Deleted 内容标记）
        // 传播，则中继产物的 wire 尺寸与 GC 源同族而非干净源同族
        let relay_bytes = recv
            .transact()
            .encode_diff_v1(&StateVector::default());
        println!(
            "   中继再编码 wire    : {}B（对照：干净源全量 {}B / 本源 {}B）",
            relay_bytes.len(),
            clean_full_len,
            bytes.len()
        );
        println!("   接收端后续编辑     : OK\n");

        summary.insert(
            label,
            format!("cand=[{}] ds=[{}]", ranges_of(&cand), ranges_of(&ds)),
        );
    }

    println!("== 汇总 ==");
    for (k, v) in &summary {
        println!("  {:<24} {}", k, v);
    }
}
