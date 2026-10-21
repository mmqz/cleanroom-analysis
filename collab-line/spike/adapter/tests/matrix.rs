//! collab-adapter 可复现测试矩阵（d1d-t1 加固轮）。
//!
//! 用户口径：「A reproducible test matrix for bulk loading, incremental
//! updates, cross-peer synchronization, concurrent edits, deletion,
//! snapshot restoration, undo/redo, and maintenance GC.」
//!
//! 本文件 = 把 d1c-t1..t4 探针证据（p8/p9/p10 手跑 bin）固化为
//! `cargo test -p collab-adapter` 可重复矩阵。断言全部确定性（零时间敏感
//! 断言——捕获窗 1000ms→0 缓解后无窗语义面，见 s0-t2/p20）。
//!
//! 矩阵章节（与用户八类一一对应 + 锚点/侧车面）：
//!   §1 bulk loading（bulk_text / blocks / 限额边界 / UTF-8 切分）
//!   §2 incremental updates（edit_stream 四变体 + 错误面）
//!   §3 cross-peer synchronization（单向收敛 / 幂等复交换）
//!   §4 concurrent edits（双端并发 + 冲突收敛）
//!   §5 deletion（删除 / 重加 / 撤销复原）
//!   §6 snapshot restoration（快照恢复 / gc 后快照）
//!   §7 undo/redo（origin 分栈 / 逐事务粒度 / 一笔装载 / 效果全局 / redo）
//!   §8 maintenance GC（墓碑治理 / 幂等 / 内容保真 / 栈清零）
//!   §9 anchors & sidecar（ID 稳定 / 悬空显式化 / 撤销复原 / 侧车往返）

use collab_adapter::{
    BlockSpec, CollabAdapter, EditOrigin, SemanticAnchor, SessionOptions, StreamEdit,
};

// ── 测试辅助 ─────────────────────────────────────────────────────

fn opts(client_id: u64) -> SessionOptions {
    SessionOptions {
        client_id,
        input_byte_limit: 52 * 1024 * 1024,
    }
}

fn sess(client_id: u64) -> CollabAdapter {
    CollabAdapter::open(opts(client_id)).expect("open")
}

fn blocks(n: usize, prefix: &str) -> Vec<BlockSpec> {
    (0..n)
        .map(|i| BlockSpec {
            block_id: format!("{prefix}-{i:05}"),
            text: format!("text of {prefix} block {i}"),
        })
        .collect()
}

/// 双向同步至收敛（两轮差量交换——CRDT 收敛标准形状）。
fn sync_converge(a: &CollabAdapter, b: &CollabAdapter) {
    for _ in 0..2 {
        let sv_b = b.state_vector();
        let diff_a = a.encode_diff(&sv_b).expect("diff a->b");
        b.apply_update(&diff_a).expect("apply a->b");
        let sv_a = a.state_vector();
        let diff_b = b.encode_diff(&sv_a).expect("diff b->a");
        a.apply_update(&diff_b).expect("apply b->a");
    }
}

// ── §1 bulk loading ──────────────────────────────────────────────

#[test]
fn bulk_text_loads_and_reads_back() {
    let a = sess(7_000_101);
    let content = "你好世界 hello world ".repeat(1000); // 多字节 UTF-8
    a.load_bulk_text(EditOrigin::Ai, &content).expect("load");
    assert_eq!(a.__probe_text(), content);
}

#[test]
fn bulk_text_chunk_split_on_char_boundary() {
    // 8KiB 切分点落在多字节字符中间 → 回退到字符边界（不 panic、内容零损）
    let a = sess(7_000_102);
    let content = "αβγδ".repeat(4096); // 2 字节/字符，切点必落字符中间
    a.load_bulk_text(EditOrigin::Ai, &content).expect("load");
    assert_eq!(a.__probe_text().len(), content.len());
    assert_eq!(a.__probe_text(), content);
}

#[test]
fn bulk_blocks_load_all() {
    let a = sess(7_000_103);
    a.load_blocks(EditOrigin::Ai, &blocks(2000, "blk")).expect("load");
    let ids = a.__probe_block_ids();
    assert_eq!(ids.len(), 2000);
    assert_eq!(ids[0], "blk-00000");
    assert_eq!(ids[1999], "blk-01999");
}

#[test]
fn bulk_limit_boundary_pass_and_reject() {
    // == limit 过门、+1 拒（B1.6 边界钉定——p9 c4 口径的矩阵化）
    // 小限额会话：256B 内容过门，257B 拒
    let small = CollabAdapter::open(SessionOptions {
        client_id: 7_000_104,
        input_byte_limit: 256,
    })
    .expect("open small");
    assert!(small.load_bulk_text(EditOrigin::Ai, &"x".repeat(256)).is_ok());
    let err = small.load_bulk_text(EditOrigin::Ai, &"x".repeat(257)).unwrap_err();
    assert!(matches!(
        err,
        collab_adapter::AdapterError::Limit { size: 257, limit: 256 }
    ));
    // 默认限额会话：52MiB+1 拒
    let big = sess(7_000_105);
    assert!(big
        .load_bulk_text(EditOrigin::Ai, &"x".repeat(52 * 1024 * 1024 + 1))
        .is_err());
}

// ── §2 incremental updates ───────────────────────────────────────

#[test]
fn edit_stream_all_variants() {
    let a = sess(7_000_107);
    a.load_blocks(EditOrigin::Ai, &blocks(3, "e")).expect("load");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "tail".into() },
    )
    .expect("append");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock {
            block: BlockSpec { block_id: "e-new".into(), text: "new".into() },
        },
    )
    .expect("insert tail");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlockBefore {
            before: "e-00001".into(),
            block: BlockSpec { block_id: "e-mid".into(), text: "mid".into() },
        },
    )
    .expect("insert before");
    let ids = a.__probe_block_ids();
    assert_eq!(
        ids,
        vec![
            "e-00000".to_string(),
            "e-mid".to_string(),
            "e-00001".to_string(),
            "e-00002".to_string(),
            "e-new".to_string()
        ]
    );
    assert_eq!(a.__probe_text(), "tail");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "e-mid".into() },
    )
    .expect("remove");
    assert_eq!(a.__probe_block_ids().len(), 4);
}

#[test]
fn edit_stream_error_faces() {
    let a = sess(7_000_108);
    a.load_blocks(EditOrigin::Ai, &blocks(2, "err")).expect("load");
    // 不存在的 before → InvalidInput
    let e1 = a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlockBefore {
            before: "no-such".into(),
            block: BlockSpec { block_id: "x".into(), text: String::new() },
        },
    );
    assert!(matches!(e1, Err(collab_adapter::AdapterError::InvalidInput(_))));
    // 不存在的 block_id 删除 → InvalidInput
    let e2 = a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "no-such".into() },
    );
    assert!(matches!(e2, Err(collab_adapter::AdapterError::InvalidInput(_))));
    // 失败后状态零变化
    assert_eq!(a.__probe_block_ids().len(), 2);
}

// ── §3 cross-peer synchronization ────────────────────────────────

#[test]
fn cross_peer_single_direction_converges() {
    let a = sess(7_000_109);
    let b = sess(7_000_110);
    a.load_blocks(EditOrigin::Ai, &blocks(50, "sync")).expect("load a");
    a.load_bulk_text(EditOrigin::Ai, "shared text").expect("text a");
    sync_converge(&a, &b);
    assert_eq!(a.__probe_block_ids(), b.__probe_block_ids());
    assert_eq!(a.__probe_text(), b.__probe_text());
}

#[test]
fn cross_peer_idempotent_reexchange() {
    let a = sess(7_000_111);
    let b = sess(7_000_112);
    a.load_blocks(EditOrigin::Ai, &blocks(20, "idem")).expect("load");
    sync_converge(&a, &b);
    let before_ids = b.__probe_block_ids();
    sync_converge(&a, &b); // 复交换（幂等性）
    sync_converge(&a, &b);
    assert_eq!(b.__probe_block_ids(), before_ids);
    assert_eq!(a.__probe_block_ids(), before_ids);
}

// ── §4 concurrent edits ──────────────────────────────────────────

#[test]
fn concurrent_disjoint_edits_converge() {
    // 双端在同步前各自编辑（逻辑并发），双向交换后收敛
    let a = sess(7_000_113);
    let b = sess(7_000_114);
    a.load_blocks(EditOrigin::Ai, &blocks(10, "cc")).expect("base a");
    sync_converge(&a, &b);
    // 并发：A 追加块，B 同时删块 + 追加文本
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock {
            block: BlockSpec { block_id: "cc-a-only".into(), text: "from a".into() },
        },
    )
    .expect("a edit");
    b.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "cc-00003".into() },
    )
    .expect("b remove");
    b.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "from b".into() },
    )
    .expect("b append");
    sync_converge(&a, &b);
    let ids_a = a.__probe_block_ids();
    let ids_b = b.__probe_block_ids();
    assert_eq!(ids_a, ids_b, "双端收敛（块面）");
    assert_eq!(a.__probe_text(), b.__probe_text(), "双端收敛（文本面）");
    assert!(ids_a.contains(&"cc-a-only".to_string()));
    assert!(!ids_a.contains(&"cc-00003".to_string()));
}

#[test]
fn concurrent_conflicting_insert_same_position_converges() {
    // 同位置冲突插入：双端在同名块前各插一块 → 收敛且双块在场、顺序一致
    let a = sess(7_000_115);
    let b = sess(7_000_116);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "cf")).expect("base");
    sync_converge(&a, &b);
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlockBefore {
            before: "cf-00002".into(),
            block: BlockSpec { block_id: "cf-ins-a".into(), text: "A".into() },
        },
    )
    .expect("a ins");
    b.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlockBefore {
            before: "cf-00002".into(),
            block: BlockSpec { block_id: "cf-ins-b".into(), text: "B".into() },
        },
    )
    .expect("b ins");
    sync_converge(&a, &b);
    let ids_a = a.__probe_block_ids();
    let ids_b = b.__probe_block_ids();
    assert_eq!(ids_a, ids_b, "冲突收敛：双端序列一致");
    assert!(ids_a.contains(&"cf-ins-a".to_string()));
    assert!(ids_a.contains(&"cf-ins-b".to_string()));
    // 两块都插在 cf-00002 之前（位置语义在双端一致）
    let pos_target = ids_a.iter().position(|x| x == "cf-00002").unwrap();
    let pos_a = ids_a.iter().position(|x| x == "cf-ins-a").unwrap();
    let pos_b = ids_a.iter().position(|x| x == "cf-ins-b").unwrap();
    assert!(pos_a < pos_target && pos_b < pos_target);
}

// ── §5 deletion ──────────────────────────────────────────────────

#[test]
fn deletion_removes_and_readd_works() {
    let a = sess(7_000_117);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "del")).expect("load");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "del-00002".into() },
    )
    .expect("remove");
    assert!(!a.__probe_block_ids().contains(&"del-00002".to_string()));
    // 同名重加：内容层面合法（新 CRDT ID——BlockId 撞号是 R4.4 已登记的
    // mdcore 本地域面，适配层 ID 本位币不受影响）
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock {
            block: BlockSpec { block_id: "del-00002".into(), text: "reborn".into() },
        },
    )
    .expect("re-add");
    assert_eq!(a.__probe_block_text("del-00002").as_deref(), Some("reborn"));
}

#[test]
fn deletion_undo_restores_block() {
    let mut a = sess(7_000_118);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "undel")).expect("load");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "undel-00001".into() },
    )
    .expect("remove");
    assert!(a.undo(EditOrigin::Human));
    assert!(a.__probe_block_ids().contains(&"undel-00001".to_string()));
    assert_eq!(
        a.__probe_block_text("undel-00001").as_deref(),
        Some("text of undel block 1")
    );
}

// ── §6 snapshot restoration ──────────────────────────────────────

#[test]
fn snapshot_restores_into_fresh_session() {
    let a = sess(7_000_119);
    a.load_blocks(EditOrigin::Ai, &blocks(101, "snap")).expect("load");
    a.load_bulk_text(EditOrigin::Ai, "snapshot text").expect("text");
    let snap = a.authoritative_snapshot().expect("snapshot");
    let b = sess(7_000_120);
    b.apply_update(&snap).expect("apply snapshot");
    assert_eq!(a.__probe_block_ids(), b.__probe_block_ids());
    assert_eq!(a.__probe_text(), b.__probe_text());
}

#[test]
fn snapshot_after_maintenance_gc_still_restores() {
    // B1.3 与 D1③ 同点：维护窗压缩不打断快照产物线（p10 e2 口径矩阵化）
    let mut a = sess(7_000_121);
    a.load_blocks(EditOrigin::Ai, &blocks(100, "sgc")).expect("load");
    for i in 0..50 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: format!("sgc-{i:05}") },
        )
        .expect("remove");
    }
    a.maintenance_gc().expect("gc");
    let snap = a.authoritative_snapshot().expect("snapshot after gc");
    let b = sess(7_000_122);
    b.apply_update(&snap).expect("apply");
    assert_eq!(a.__probe_block_ids(), b.__probe_block_ids());
}

// ── §7 undo/redo ─────────────────────────────────────────────────

#[test]
fn undo_origin_stacks_are_independent() {
    let mut a = sess(7_000_123);
    a.load_bulk_text(EditOrigin::Ai, "ai base").expect("ai load");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: " H1".into() },
    )
    .expect("h1");
    a.edit_stream(
        EditOrigin::Ai,
        StreamEdit::AppendText { text: " A1".into() },
    )
    .expect("a1");
    let h_depth = a.undo_depth(EditOrigin::Human);
    let a_depth = a.undo_depth(EditOrigin::Ai);
    assert!(h_depth >= 1 && a_depth >= 1);
    // Human 撤销耗尽后 Ai 栈深度不变（origin 分栈——B3.2）
    while a.undo(EditOrigin::Human) {}
    assert_eq!(a.undo_depth(EditOrigin::Human), 0);
    assert_eq!(
        a.undo_depth(EditOrigin::Ai),
        a_depth,
        "Ai 栈不受 Human 消耗影响"
    );
    // Ai 面独立回退
    assert!(a.undo(EditOrigin::Ai));
    assert!(!a.__probe_text().contains(" A1"));
}

#[test]
fn undo_consecutive_ops_are_separate_items() {
    // 逐事务粒度（p20 缓解后语义，窗 0）：连续三笔追加 = 三个独立栈项，
    // 单次 undo 只回退最后一笔（修复前窗 1000 会三笔合一、一次全清——
    // 即交错场景下合并非连续项触发 yrs 0.28 undo 部分应用/下溢的根源）
    let mut a = sess(7_000_124);
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "1".into() },
    )
    .expect("e1");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "2".into() },
    )
    .expect("e2");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "3".into() },
    )
    .expect("e3");
    assert_eq!(a.undo_depth(EditOrigin::Human), 3, "逐事务粒度：三笔 = 三栈项");
    // LIFO：单次 undo 只撤最后一笔
    assert!(a.undo(EditOrigin::Human));
    assert_eq!(a.__probe_text(), "12", "单次 undo 仅回退最后一笔");
    // 撤尽 + redo 全复原（深度对称）
    assert!(a.undo(EditOrigin::Human));
    assert!(a.undo(EditOrigin::Human));
    assert_eq!(a.__probe_text(), "");
    assert_eq!(a.redo_depth(EditOrigin::Human), 3, "redo 深度镜像");
    while a.redo(EditOrigin::Human) {}
    assert_eq!(a.__probe_text(), "123", "redo 全复原");
}

#[test]
fn undo_bulk_load_is_single_stack_item() {
    // 一笔装载语义（B2.2）：AI 生成态整段装载 = 1 栈项，单次 undo 清空
    let mut a = sess(7_000_126);
    a.load_bulk_text(EditOrigin::Ai, &"bulk content ".repeat(1000)).expect("load");
    assert_eq!(a.undo_depth(EditOrigin::Ai), 1, "一笔装载 = 单栈项");
    assert!(a.undo(EditOrigin::Ai));
    assert_eq!(a.__probe_text(), "");
    assert_eq!(a.undo_depth(EditOrigin::Ai), 0);
}

#[test]
fn undo_effect_propagates_to_peer() {
    // 栈本地 + 效果全局（B3.1）：undo 产生新 op，差量同步后对端收敛于回退态
    let mut a = sess(7_000_127);
    let b = sess(7_000_128);
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "shared".into() },
    )
    .expect("edit");
    sync_converge(&a, &b);
    assert_eq!(b.__probe_text(), "shared");
    assert!(a.undo(EditOrigin::Human));
    sync_converge(&a, &b);
    assert_eq!(b.__probe_text(), "", "对端随差量收敛于回退态");
}

#[test]
fn redo_restores_undone_content() {
    let mut a = sess(7_000_129);
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "redo-me".into() },
    )
    .expect("edit");
    assert!(a.undo(EditOrigin::Human));
    assert_eq!(a.__probe_text(), "");
    assert!(a.redo(EditOrigin::Human));
    assert_eq!(a.__probe_text(), "redo-me");
}

// ── §8 maintenance GC ────────────────────────────────────────────

#[test]
fn maintenance_gc_shrinks_tombstones_and_preserves_content() {
    let mut a = sess(7_000_130);
    a.load_blocks(EditOrigin::Ai, &blocks(200, "gc")).expect("load");
    for i in 0..100 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: format!("gc-{i:05}") },
        )
        .expect("remove");
    }
    let ids_before = a.__probe_block_ids();
    let (before, after) = a.maintenance_gc().expect("gc");
    assert!(
        after < before,
        "墓碑治理收缩全量 update：{before}B → {after}B"
    );
    assert_eq!(a.__probe_block_ids(), ids_before, "内容保真");
    // 撤销栈已清（B3.3 前置内置）
    assert_eq!(a.undo_depth(EditOrigin::Human), 0);
    assert_eq!(a.undo_depth(EditOrigin::Ai), 0);
    // 二次幂等
    let (b2, a2) = a.maintenance_gc().expect("gc2");
    assert_eq!(b2, a2, "二次 gc 尺寸稳定（幂等）");
}

// ── §9 anchors & sidecar ─────────────────────────────────────────

#[test]
fn anchor_id_stable_across_structural_edits() {
    let mut a = sess(7_000_131);
    a.load_blocks(EditOrigin::Ai, &blocks(10, "anc")).expect("load");
    let anchor = a
        .bind(&SemanticAnchor::MdBlock { block_id: "anc-00005".into() })
        .expect("bind");
    // 前插 5 块
    for i in 0..5 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::InsertBlockBefore {
                before: "anc-00000".into(),
                block: BlockSpec {
                    block_id: format!("anc-pre-{i}"),
                    text: "pre".into(),
                },
            },
        )
        .expect("pre insert");
    }
    // 前删 3 块（删的是新插入的 pre 块中前三块）
    for i in 0..3 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: format!("anc-pre-{i}") },
        )
        .expect("pre remove");
    }
    let resolved = a.resolve(anchor).expect("resolve after edits");
    assert_eq!(
        resolved.block_id,
        "anc-00005",
        "ID 稳定：语义复原（下标漂移无关）"
    );
    assert_eq!(resolved.block_index, 7, "下标漂移 5 → 7（前插 5 - 前删 3）");
}

#[test]
fn anchor_dangling_explicit_on_deletion_and_new_id_on_undo() {
    // 语义钉定（d1d-t1 实测发现）：yrs undo 对删除的复原 = 重插**新 ID**
    // 内容项（ItemPtr::redo 走 redone 指针，原墓碑不复活）——原锚点
    // 在 undo 后**保持悬空**；复活块需按 block_id 重新绑定取得新锚点。
    // （含义：锚点跟随复活内容需要 redone 链映射——yrs 0.28 公开 API
    // 无此面，登记为上游 API 请求候选 + 事件面联动遗留。）
    let mut a = sess(7_000_132);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "dang")).expect("load");
    let anchor = a
        .bind(&SemanticAnchor::MdBlock { block_id: "dang-00002".into() })
        .expect("bind");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "dang-00002".into() },
    )
    .expect("remove");
    let err = a.resolve(anchor).unwrap_err();
    assert!(
        matches!(err, collab_adapter::AdapterError::AnchorDangling { client, clock } if client == anchor.client && clock == anchor.clock),
        "删除面显式化：AnchorDangling（不静默返回邻居）"
    );
    // 撤销复原：块内容回归（block_id 复原），但原锚点保持悬空（新 ID 语义）
    assert!(a.undo(EditOrigin::Human));
    assert!(a.__probe_block_ids().contains(&"dang-00002".to_string()));
    assert!(
        matches!(a.resolve(anchor), Err(collab_adapter::AdapterError::AnchorDangling { .. })),
        "undo 复活 = 新 ID 内容项：原锚点保持悬空（不静默漂移到新项）"
    );
    // 复活块重新绑定 → 新锚点（与原锚点不同）且可解析
    let anchor2 = a
        .bind(&SemanticAnchor::MdBlock { block_id: "dang-00002".into() })
        .expect("rebind revived block");
    assert_ne!(
        (anchor2.client, anchor2.clock),
        (anchor.client, anchor.clock),
        "复活块锚点 = 新 CRDT ID"
    );
    let resolved = a.resolve(anchor2).expect("resolve new anchor");
    assert_eq!(resolved.block_id, "dang-00002");
}

#[test]
fn anchor_cross_peer_resolution_is_stable() {
    // 跨端：A bind → 全量同步 → B resolve 同一 block_id（ID 跨进程稳定）
    let mut a = sess(7_000_133);
    let b = sess(7_000_134);
    a.load_blocks(EditOrigin::Ai, &blocks(10, "xpeer")).expect("load");
    let anchor = a
        .bind(&SemanticAnchor::MdBlock { block_id: "xpeer-00007".into() })
        .expect("bind");
    sync_converge(&a, &b);
    let resolved = b.resolve(anchor).expect("peer resolve");
    assert_eq!(resolved.block_id, "xpeer-00007");
    assert_eq!(resolved.block_index, 7);
}

#[test]
fn anchor_sidecar_roundtrip_byte_stable_and_importable() {
    let mut a = sess(7_000_135);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "sc")).expect("load");
    let a1 = a
        .bind(&SemanticAnchor::MdBlock { block_id: "sc-00001".into() })
        .expect("bind1");
    let a2 = a
        .bind(&SemanticAnchor::OfficeBookmark { name: "bk-top".into(), block_index: 2 })
        .expect("bind2");
    let a3 = a
        .bind(&SemanticAnchor::PdfAnnotRef { page: 3, label: "note-9".into(), block_index: 4 })
        .expect("bind3");
    let bytes1 = a.export_anchor_sidecar();
    let bytes2 = a.export_anchor_sidecar();
    assert_eq!(bytes1, bytes2, "侧车字节稳定（确定性排序）");
    // 导入到「经同步持有同源内容」的新会话（锚点 ID 引用 A 的 item——
    // 独立装载的会话内容同形但 ID 不同属，侧车跨会话的正确消费形状
    // = 同步 + 侧车导入，p10 c 口径）
    let mut b = sess(7_000_136);
    let diff = a.encode_diff(&b.state_vector()).expect("sync diff");
    b.apply_update(&diff).expect("sync apply");
    let n = b.import_anchor_sidecar(&bytes1).expect("import");
    assert_eq!(n, 3);
    assert_eq!(
        b.anchor_of(&SemanticAnchor::MdBlock { block_id: "sc-00001".into() }),
        Some(a1)
    );
    assert_eq!(
        b.anchor_of(&SemanticAnchor::OfficeBookmark { name: "bk-top".into(), block_index: 2 }),
        Some(a2)
    );
    assert_eq!(
        b.anchor_of(&SemanticAnchor::PdfAnnotRef { page: 3, label: "note-9".into(), block_index: 4 }),
        Some(a3)
    );
    // 导入后可解析
    let r = b.resolve(a2).expect("resolve imported");
    assert_eq!(r.block_index, 2);
}
