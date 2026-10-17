//! 探针 8 适配层原型第一轮（d1c-t1）：构造/装载面不变量验证。
//!
//! 用户裁决口径（2026-10-17 会话）：D1 有条件确认边界——允许可撤回的
//! 适配层原型，不批准全面集成；验证面 = 内存上限、撤销语义、跨模型
//! 锚点映射。本轮覆盖前两者的装载/构造面 + 同步/撤销冒烟：
//!
//! P8a 构造不变量（B1.1/B1.2）
//!      a1 skip_gc 外部可观测 = true（DocOptions 单点真源直读）；
//!      a2 快照门 Ok（skip_gc 与权威快照同向，yrs-b §2.1 事实 6）；
//!      a3 SessionOptions 无 skip_gc 字段（API 面事实：编译器保证调用方
//!         不可覆盖——登记面）。
//! P8b 装载纪律（B2.1——用户"验证内存上限"的装载面）
//!      b1 edit_stream 20k×8B 逐笔事务 → ΔRSS 断言 < 32MB（A3 形状；
//!         对照面：直接 yrs A1 形状 10k 笔即 +415MB，yrs-a §4.5 在档）；
//!      b2 load_bulk_text 20k×8B 拼接 → 8KiB 大块单事务 → ΔRSS 断言
//!         < 32MB（A2 形状）；
//!      b3 load_blocks 20k 块（512/批分事务）→ ΔRSS 断言 < 64MB
//!         （结构化小批路径——块数多、每块两字段，上限口径放宽）；
//!      b4 禁止形状（单事务 N 笔小串）在适配层 API 面不可表达
//!         （B2.1 纪律的编译期证明——无裸长事务入口，登记面）；
//!      b5 输入限额：load_bulk_text 超 52MiB → Err(Limit)（B1.6 预览）。
//! P8c origin 冒烟（B3.2——深验在 d1c-t2）
//!      c1 Human×3 + Ai×2 后 undo(Human) → human 面回退、ai 面不动；
//!      c2 redo 复原。
//! P8d 同步冒烟（B1.6 预览——panic 隔离深验在 d1c-t2）
//!      d1 双适配器差量交换收敛（内容一致）；
//!      d2 apply_update 敌意垃圾字节 → Err 且进程存活（panic 隔离）。
//!
//! 口径：release（装载内存面）；SCALE 环境变量调节 b 轮规模（默认 20000）。

use std::time::Instant;

use collab_adapter::{
    AdapterError, BlockSpec, CollabAdapter, EditOrigin, SessionOptions, StreamEdit,
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

fn main() {
    let scale: usize = std::env::var("SCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    println!("P8 适配层原型第一轮（scale={}）", scale);

    // ── P8a 构造不变量 ────────────────────────────────────────────
    let a = CollabAdapter::open(SessionOptions::default()).expect("open");
    assert!(a.__probe_skip_gc(), "P8a1 skip_gc 必须为 true（B1.1 全编队不变量）");
    println!("P8a1: skip_gc 构造期强制 = true（DocOptions 单点直读）");

    a.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "hello ".into() })
        .unwrap();
    match a.__probe_snapshot_gate() {
        Ok(bytes) => println!("P8a2: 快照门 Ok（{}B——skip_gc 与权威快照同向）", bytes.len()),
        Err(e) => panic!("P8a2 快照门必须 Ok（skip_gc 文档）: {:?}", e),
    }
    println!("P8a3: SessionOptions 无 skip_gc 字段（API 面不可覆盖——编译期事实）");

    // ── P8b 装载纪律（内存面）─────────────────────────────────────
    // 口径注：8B 小串 = "cell0000" 形（对齐 yrs-a §4.5 A1/A3 口径）

    // b1: edit_stream 逐笔事务（A3 形状）
    let a = CollabAdapter::open(SessionOptions::default()).unwrap();
    let rss0 = vm("VmRSS");
    let t0 = Instant::now();
    for i in 0..scale {
        let s = format!("cell{:04}", i % 10_000);
        a.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: s })
            .unwrap_or_else(|e| panic!("b1 edit {}: {:?}", i, e));
    }
    let b1_rss = vm("VmRSS") as i64 - rss0 as i64;
    println!(
        "  b1 edit_stream {} 笔×8B: ΔRSS {:+} kB, {} ms（A3 形状，逐事务）",
        scale,
        b1_rss,
        t0.elapsed().as_millis()
    );
    assert!(b1_rss < 32 * 1024, "b1 ΔRSS {} kB 超断言上限 32MB", b1_rss);
    assert_eq!(
        a.__probe_text().len(),
        scale * 8,
        "b1 内容长度"
    );

    // b2: load_bulk_text 大块单事务（A2 形状）——同样 8B×scale 的总内容
    let b = CollabAdapter::open(SessionOptions::default()).unwrap();
    let rss0 = vm("VmRSS");
    let t0 = Instant::now();
    let mut bulk = String::with_capacity(scale * 8);
    for i in 0..scale {
        bulk.push_str(&format!("cell{:04}", i % 10_000));
    }
    b.load_bulk_text(&bulk).unwrap();
    let b2_rss = vm("VmRSS") as i64 - rss0 as i64;
    println!(
        "  b2 load_bulk_text {}B（8KiB 大块单事务）: ΔRSS {:+} kB, {} ms（A2 形状）",
        bulk.len(),
        b2_rss,
        t0.elapsed().as_millis()
    );
    assert!(b2_rss < 32 * 1024, "b2 ΔRSS {} kB 超断言上限 32MB", b2_rss);
    assert_eq!(b.__probe_text(), bulk, "b2 内容逐字一致");

    // b3: load_blocks 结构化小批（512/批分事务）
    let c = CollabAdapter::open(SessionOptions::default()).unwrap();
    let rss0 = vm("VmRSS");
    let t0 = Instant::now();
    let blocks: Vec<BlockSpec> = (0..scale)
        .map(|i| BlockSpec {
            block_id: format!("blk-{:06}", i),
            text: format!("cell{:04}", i % 10_000),
        })
        .collect();
    c.load_blocks(&blocks).unwrap();
    let b3_rss = vm("VmRSS") as i64 - rss0 as i64;
    println!(
        "  b3 load_blocks {} 块（512/批分事务）: ΔRSS {:+} kB, {} ms",
        scale,
        b3_rss,
        t0.elapsed().as_millis()
    );
    assert!(b3_rss < 64 * 1024, "b3 ΔRSS {} kB 超断言上限 64MB", b3_rss);
    assert_eq!(c.__probe_block_ids().len(), scale, "b3 块数");
    assert_eq!(c.__probe_block_text("blk-000000").as_deref(), Some("cell0000"));

    println!("P8b1-b3: 装载三路径 ΔRSS 全部在断言上限内（对照：直接 yrs A1 形状 10k 笔 +415MB，yrs-a §4.5 在档——本断言阈值给 32/64MB 量级余量防环境噪声）");
    println!("P8b4: 禁止形状（单事务 N 笔小串）API 面不可表达——CollabAdapter 无裸事务入口（B2.1 编译期证明，登记面）");

    // b5: 输入限额
    let limit = SessionOptions::default().input_byte_limit;
    let big = "x".repeat(limit + 1);
    match b.load_bulk_text(&big) {
        Err(AdapterError::Limit { size, limit: l }) => {
            assert_eq!(size, limit + 1);
            assert_eq!(l, limit);
            println!("P8b5: 输入限额生效（{}B > {}B → Err(Limit)）", size, l);
        }
        other => panic!("P8b5 必须 Err(Limit): {:?}", other.is_ok()),
    }

    // ── P8c origin 冒烟（B3.2）───────────────────────────────────
    let mut d = CollabAdapter::open(SessionOptions {
        client_id: 7_000_002,
        ..Default::default()
    })
    .unwrap();
    for s in ["H1 ", "H2 ", "H3 "] {
        d.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: s.into() })
            .unwrap();
    }
    for s in ["A1 ", "A2"] {
        d.edit_stream(EditOrigin::Ai, StreamEdit::AppendText { text: s.into() })
            .unwrap();
    }
    assert_eq!(d.__probe_text(), "H1 H2 H3 A1 A2");
    // 捕获窗 1000ms 内的连续 human 编辑合并为少数栈项——循环 undo 到 human 面清空
    let mut undos = 0;
    while d.undo(EditOrigin::Human) {
        undos += 1;
        if undos > 10 {
            panic!("c1 undo 轮次异常");
        }
    }
    assert_eq!(d.__probe_text(), "A1 A2", "c1 human 面全部回退");
    let ai_depth_before = d.undo_depth(EditOrigin::Ai);
    assert!(ai_depth_before > 0, "c1 ai 栈仍在（human undo 不消耗 ai 栈）");
    while d.undo(EditOrigin::Ai) {}
    assert_eq!(d.__probe_text(), "", "c1 ai 面随后独立回退");
    let mut redos = 0;
    while d.redo(EditOrigin::Human) {
        redos += 1;
        if redos > 10 {
            panic!("c1 redo 轮次异常");
        }
    }
    assert_eq!(d.__probe_text(), "H1 H2 H3 ", "c2 redo 复原 human 面");
    println!(
        "P8c: origin 分栈冒烟过（human undo×{} 后 ai 面不动；redo 复原；捕获窗 1s = COALESCE_MS 语义连续性）",
        undos
    );

    // ── P8d 同步冒烟 ─────────────────────────────────────────────
    let e = CollabAdapter::open(SessionOptions {
        client_id: 7_000_003,
        ..Default::default()
    })
    .unwrap();
    e.load_bulk_text(&bulk).unwrap();
    let sv_local = c.state_vector();
    let diff = e.encode_diff(&sv_local).expect("encode_diff");
    c.apply_update(&diff).expect("apply_update");
    // e 侧同步 c 的 blocks（结构化面也收敛）——差量方向 = c 相对 e 的 SV
    let sv_e = e.state_vector();
    let diff_c = c.encode_diff(&sv_e).unwrap();
    e.apply_update(&diff_c).unwrap();
    assert_eq!(
        e.__probe_block_ids().len(),
        scale,
        "d1 结构化面收敛（块数）"
    );
    assert_eq!(e.__probe_text(), bulk, "d1 文本面收敛（逐字）");
    println!(
        "P8d1: 双适配器双向差量交换收敛（text 逐字 + blocks {} 项）",
        scale
    );

    // d2: 敌意垃圾字节 → panic 隔离
    let garbage = vec![0xFFu8; 4096];
    match c.apply_update(&garbage) {
        Err(AdapterError::Decode(_)) => println!("P8d2: 敌意 update 被解码层拒绝（Err(Decode)——进程存活）"),
        Err(AdapterError::Panic(m)) => println!("P8d2: 敌意 update 引发引擎 panic 已隔离（Err(Panic): {}——进程存活）", m),
        other => panic!("P8d2 必须 Err: {:?}", other),
    }
    // 超限 update（B1.6 限额）
    let huge = vec![0u8; limit + 1];
    match c.apply_update(&huge) {
        Err(AdapterError::Limit { .. }) => println!("P8d2b: update 字节限额生效（52MiB 面同 load_bulk_text）"),
        other => panic!("P8d2b 必须 Err(Limit): {:?}", other),
    }
    // 隔离后适配层仍可用（B1.6 隔离有效性——引擎未被毒化）
    c.edit_stream(EditOrigin::Ai, StreamEdit::AppendText { text: "STILL-OK".into() })
        .unwrap();
    assert!(c.__probe_text().ends_with("STILL-OK"));
    println!("P8d2c: panic 隔离后引擎继续可用（未被毒化）");

    println!("P8 PASS: 构造不变量（B1.1/B1.2）+ 装载纪律（B2.1 三路径内存）+ 限额（B1.6）+ origin 分栈冒烟（B3.2）+ 同步收敛/敌意隔离冒烟");
}
