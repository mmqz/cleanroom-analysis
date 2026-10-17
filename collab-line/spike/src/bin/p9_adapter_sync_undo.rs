//! 探针 9 适配层原型第二轮（d1c-t2）：同步/撤销深验 + B1.3 受控压缩点。
//!
//! 覆盖 yrs-b 边界规则与用户验证面（撤销语义 = 用户三验证面之二）：
//!
//! P9a 受控压缩点（B1.3 + B3.3 内置前置）
//!      a1 装载 + 批量删除后全量 update 含墓碑膨胀；maintenance_gc()
//!         （内部先 clear_all 全栈再 gc）后 update 尺寸显著回落
//!         （P7c 口径：显式压缩后 update 回到等价基线）；
//!      a2 压缩后双 origin 撤销栈深度 = 0（clear_all 内置——B3.3 不依赖
//!         调用方纪律）；
//!      a3 压缩内容保真（block_ids 与压缩前一致——wire 治理不以内容
//!         损失为代价）；a4 幂等（二次调用无害）。
//! P9b 撤销语义深验（B3.1 栈本地 + B3.2 origin/捕获窗）
//!      b1 栈本地性 + 效果全局：peer1 human 编辑同步 peer2 → peer1 undo
//!         → 差量再同步 → 双端收敛于回退后状态（撤销产生的 op 随 update
//!         同步，栈本身不出进程）；
//!      b2 捕获窗合并/分离：窗内连续编辑 = 单栈项（一笔 undo 全回退）；
//!         窗外（>1.1s 间隔）编辑 = 独立栈项（逐项 undo）；
//!      b3 "一笔装载"语义（B2.2）：load_blocks 2k 块（Ai）= 捕获窗内
//!         4 小批事务合并为 1 栈项；undo(Ai) 一次清空全部装载。
//! P9c 同步深验（B1.6）
//!      c1 截断面：合法 update 在 32 个随机偏移截断 → 全部 Err（Decode
//!         或 Panic 隔离），进程存活，引擎每次仍可用（未毒化）；
//!      c2 幂等复交换：同一 update 双次 apply → 零变化零错误；
//!      c3 并发冲突合并：双端同位并发编辑 → 交换后收敛一致（CRDT 域）；
//!      c4 限额边界：len == limit 不被 Limit 拒（后续面另判）；
//!         len == limit+1 → Err(Limit)。
//! P9d B3.4 API 面定谳登记（源码核验，非运行时）
//!      yrs 0.28 UndoManager：observer_* 全私有、无公开 observe、无栈
//!      逐出/容量 API；undo_stack()/redo_stack() 公开 getter 在（深度
//!      监控可行）。结论：栈上限不可公开 API 实现——呈 B3.4/R-3 协同
//!      裁决（选项：上游请求 / 粗粒度 clear_all / 无限维持）。

use std::thread::sleep;
use std::time::Duration;

use collab_adapter::{
    BlockSpec, CollabAdapter, EditOrigin, SessionOptions, StreamEdit,
};

fn main() {
    println!("P9 适配层原型第二轮：同步/撤销深验 + 压缩点");

    // ── P9a 受控压缩点 ────────────────────────────────────────────
    let mut a = CollabAdapter::open(SessionOptions {
        client_id: 8_000_001,
        ..Default::default()
    })
    .unwrap();
    let blocks: Vec<BlockSpec> = (0..2_000)
        .map(|i| BlockSpec {
            block_id: format!("blk-{:05}", i),
            text: format!("content-of-block-{:05}-xxxxxxxx", i),
        })
        .collect();
    a.load_blocks(EditOrigin::Ai, &blocks).unwrap();
    // human 面批量删除（每块独立事务 → 捕获窗合并为少数栈项）
    for i in 0..1_000 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock {
                block_id: format!("blk-{:05}", i),
            },
        )
        .unwrap();
    }
    let depth_human_before = a.undo_depth(EditOrigin::Human);
    assert!(depth_human_before >= 1, "删除操作应入 human 栈");
    let (pre_gc, post_gc) = a.maintenance_gc().unwrap();
    let a2_h = a.undo_depth(EditOrigin::Human);
    let a2_ai = a.undo_depth(EditOrigin::Ai);
    assert_eq!((a2_h, a2_ai), (0, 0), "P9a2 压缩后双栈清零（B3.3 内置前置）");
    println!(
        "P9a1: 全量 update {}B → {}B（墓碑治理 {:.1}%，P7c 口径 wire 治理）",
        pre_gc,
        post_gc,
        100.0 * (1.0 - post_gc as f64 / pre_gc as f64)
    );
    assert!(
        post_gc < pre_gc,
        "P9a1 压缩必须显著降低 update 尺寸（skip_gc 墓碑被 gc(None) 收集）"
    );
    let ids: Vec<String> = a.__probe_block_ids();
    assert_eq!(ids.len(), 1_000, "P9a3 压缩内容保真（剩余块数）");
    assert_eq!(ids[0], "blk-01000", "P9a3 剩余首块正确");
    assert_eq!(ids[999], "blk-01999", "P9a3 剩余末块正确");
    let (p2, p3) = a.maintenance_gc().unwrap();
    assert_eq!(p2, p3, "P9a4 二次压缩幂等（无新墓碑）");
    println!("P9a: 受控压缩点全过（B1.3 + B3.3 内置前置 + 保真 + 幂等）");

    // ── P9b 撤销语义深验 ──────────────────────────────────────────
    // b1 栈本地 + 效果全局
    let mut p1 = CollabAdapter::open(SessionOptions {
        client_id: 8_000_002,
        ..Default::default()
    })
    .unwrap();
    let p2a = CollabAdapter::open(SessionOptions {
        client_id: 8_000_003,
        ..Default::default()
    })
    .unwrap();
    p1.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "SHARED-".into() })
        .unwrap();
    let sv2 = p2a.state_vector();
    let d1 = p1.encode_diff(&sv2).unwrap();
    p2a.apply_update(&d1).unwrap();
    assert_eq!(p2a.__probe_text(), "SHARED-", "b1 同步后对端可见");
    // p1 undo → 产生新 op → 再同步 → 双端收敛于回退态
    assert!(p1.undo(EditOrigin::Human));
    let sv2b = p2a.state_vector();
    let d2 = p1.encode_diff(&sv2b).unwrap();
    p2a.apply_update(&d2).unwrap();
    assert_eq!(p1.__probe_text(), "", "b1 发起端回退");
    assert_eq!(p2a.__probe_text(), "", "b1 对端随 update 收敛于回退态（栈本地效果全局）");
    println!("P9b1: 撤销栈本地 + 撤销效果经 update 全局收敛");

    // b2 捕获窗合并/分离
    let mut w = CollabAdapter::open(SessionOptions {
        client_id: 8_000_004,
        ..Default::default()
    })
    .unwrap();
    for s in ["w1 ", "w2 ", "w3 "] {
        w.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: s.into() })
            .unwrap();
    }
    let merged = w.undo_depth(EditOrigin::Human);
    assert_eq!(merged, 1, "b2 窗内三笔合并单栈项（捕获窗 1s = COALESCE_MS）");
    assert!(w.undo(EditOrigin::Human));
    assert_eq!(w.__probe_text(), "", "b2 单次 undo 全回退");
    // 窗外独立栈项
    w.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "SLOW-1 ".into() })
        .unwrap();
    sleep(Duration::from_millis(1_150));
    w.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "SLOW-2".into() })
        .unwrap();
    let split = w.undo_depth(EditOrigin::Human);
    assert_eq!(split, 2, "b2 窗外两笔独立栈项（逐项 undo）");
    assert!(w.undo(EditOrigin::Human));
    assert_eq!(w.__probe_text(), "SLOW-1 ", "b2 逐项回退（仅最近项）");
    println!("P9b2: 捕获窗合并（窗内单栈项）与分离（窗外逐项）双面实证");

    // b3 一笔装载语义（B2.2）
    let mut g = CollabAdapter::open(SessionOptions {
        client_id: 8_000_005,
        ..Default::default()
    })
    .unwrap();
    let gen_blocks: Vec<BlockSpec> = (0..2_048)
        .map(|i| BlockSpec {
            block_id: format!("gen-{:04}", i),
            text: "AI 生成块内容。".repeat(8),
        })
        .collect();
    g.load_blocks(EditOrigin::Ai, &gen_blocks).unwrap();
    let gen_depth = g.undo_depth(EditOrigin::Ai);
    assert_eq!(gen_depth, 1, "b3 4 小批事务合并为单栈项（一笔装载）");
    assert!(g.undo(EditOrigin::Ai));
    assert_eq!(g.__probe_block_ids().len(), 0, "b3 单次 undo 清空全部装载");
    println!("P9b3: 一笔装载 = 单 undo 栈项（B2.2 生成态语义，2048 块 4 批合并）");

    // ── P9c 同步深验 ─────────────────────────────────────────────
    // c1 截断面
    let s = CollabAdapter::open(SessionOptions {
        client_id: 8_000_006,
        ..Default::default()
    })
    .unwrap();
    let t = CollabAdapter::open(SessionOptions {
        client_id: 8_000_007,
        ..Default::default()
    })
    .unwrap();
    let mut corpus = String::new();
    for i in 0..2_000 {
        corpus.push_str(&format!("line-{:04} abcdefgh\r\n", i));
    }
    t.load_bulk_text(EditOrigin::Human, &corpus).unwrap();
    let full = t.encode_diff(&s.state_vector()).unwrap();
    let full_len = full.len();
    // 确定性伪随机截断偏移（32 点，覆盖头部/中部/尾部）
    let offsets: Vec<usize> = (0..32)
        .map(|k| {
            let x = (k as u64 * 6_364_136_223_846_793_005 + 1_442_695_040_888_963_407)
                % (full_len as u64);
            (x as usize).max(1).min(full_len - 1)
        })
        .collect();
    let mut rejected = 0;
    for &off in &offsets {
        let cut = &full[..off];
        match t.apply_update(cut) {
            Ok(()) => panic!("c1 截断 update 不应 Ok（offset {}）", off),
            Err(_) => rejected += 1,
        }
        // 引擎未毒化：截断后仍可正常编码
        let _ = t.state_vector();
    }
    assert_eq!(rejected, 32, "c1 全部截断面被拒");
    let text_after = t.__probe_text();
    assert_eq!(text_after, corpus, "c1 截断 apply 零内容变化（拒绝面不落部分状态）");
    println!(
        "P9c1: 32/32 截断面全拒（偏移 1..{}，进程存活引擎未毒化内容零变化）",
        full_len - 1
    );

    // c2 幂等复交换
    let d_again = t.encode_diff(&s.state_vector()).unwrap();
    s.apply_update(&d_again).unwrap();
    let sv_s = s.state_vector();
    let d_again2 = t.encode_diff(&sv_s).unwrap();
    s.apply_update(&d_again2).unwrap();
    assert_eq!(s.__probe_text(), corpus, "c2 双次 apply 幂等（内容恒定）");
    println!("P9c2: 幂等复交换过（同 update 双次 apply 零变化）");

    // c3 并发冲突合并
    let u = CollabAdapter::open(SessionOptions {
        client_id: 8_000_008,
        ..Default::default()
    })
    .unwrap();
    let v = CollabAdapter::open(SessionOptions {
        client_id: 8_000_009,
        ..Default::default()
    })
    .unwrap();
    u.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "U-EDIT ".into() })
        .unwrap();
    v.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "V-EDIT ".into() })
        .unwrap();
    let sv_v = v.state_vector();
    let du = u.encode_diff(&sv_v).unwrap();
    v.apply_update(&du).unwrap();
    let sv_u = u.state_vector();
    let dv = v.encode_diff(&sv_u).unwrap(); // v 相对 u 的差量（含 u 缺的 v 编辑）
    u.apply_update(&dv).unwrap();
    assert_eq!(u.__probe_text(), v.__probe_text(), "c3 并发编辑收敛一致");
    assert!(
        u.__probe_text().contains("U-EDIT ") && u.__probe_text().contains("V-EDIT "),
        "c3 双方编辑都在场"
    );
    println!("P9c3: 并发冲突合并收敛（双端内容一致且双方编辑在场）");

    // c4 限额边界
    let limit = SessionOptions::default().input_byte_limit;
    let lim = CollabAdapter::open(SessionOptions {
        client_id: 8_000_010,
        ..Default::default()
    })
    .unwrap();
    let at_limit = vec![0u8; limit];
    match lim.apply_update(&at_limit) {
        Err(collab_adapter::AdapterError::Limit { .. }) => {
            panic!("c4 len == limit 不应被 Limit 拒")
        }
        Err(other) => println!(
            "P9c4: len == limit 过限额门（后续面另判：{:?}）",
            other
        ),
        Ok(()) => println!("P9c4: len == limit 过限额门（全零 = 合法空 update 编码，Ok 无操作）"),
    }
    assert_eq!(lim.__probe_text(), "", "c4 空面无内容变化");
    let over = vec![0u8; limit + 1];
    match lim.apply_update(&over) {
        Err(collab_adapter::AdapterError::Limit { size, limit: l }) => {
            assert_eq!((size, l), (limit + 1, limit));
            println!("P9c4: 限额边界钉定（== limit 过门，+1 → Err(Limit)）");
        }
        other => panic!("c4 +1 必须 Err(Limit): {:?}", other),
    }

    // ── P9d B3.4 定谳登记（源码核验结论，无运行时断言）────────────
    println!("P9d: B3.4 API 面定谳（源码核验）——yrs 0.28 UndoManager observer_* 全私有/无公开 observe/无逐出容量 API；undo_stack() getter 公开（深度监控可行）。栈上限 = 上游请求 / 粗粒度 clear_all / 无限维持三选项，呈 B3.4×R-3 协同裁决");

    println!("P9 PASS: 压缩点（B1.3/B3.3）+ 撤销深验（B3.1 栈本地/B3.2 捕获窗双向/B2.2 一笔装载）+ 同步深验（截断 32 面/幂等/并发收敛/限额边界）+ B3.4 定谳登记");
}
