//! B1.6 入向字节限额**全入口边界矩阵**（d1e-t2，2026-10-17）。
//!
//! 用户指令 Task 3b：「Harden resource-limit coverage across every
//! write entry point」。本文件 = 适配层全部入向入口的**精确边界**两钉：
//! 每入口 `== limit` 恰过门（状态可推进/到达后续语义面）与
//! `limit+1` 恰拒（`Err(Limit)` 且 size/limit 字段精确）。
//!
//! d1e-t2 补检三面（本文件新锁，此前无限额检查）：
//! - `RemoveBlock` 查找键（与 InsertBlockBefore 的 before 键同纪律——
//!   e-review 先例「查找键同为入向输入」）；
//! - `bind` 语义锚字符串（写入 anchor_table 并经侧车导出；绑定**条数**
//!   上限维持 yrs-d §4 开放裁决面，本检查不预设）；
//! - `encode_diff` remote_sv（读面但解码入向输入——纪律统一）。
//!
//! 边界工件策略：算术口径路径（load_blocks/InsertBlock/InsertBlockBefore
//! 的 `id+text+64[+before]`）按精确账目构造；wire 口径路径
//! （apply_update/encode_diff/import_sidecar）用**真实工件**钉（差量
//! 57B/全零 SV/32B 侧车——先观测尺寸再定限额，避免手搓编码假设）。

use collab_adapter::{
    AdapterError, BlockSpec, CollabAdapter, EditOrigin, SemanticAnchor, SessionOptions,
    StreamEdit,
};

const L: usize = 256;

fn sess_limited(client_id: u64, limit: usize) -> CollabAdapter {
    CollabAdapter::open(SessionOptions {
        client_id,
        input_byte_limit: limit,
        ..Default::default()
    })
    .expect("open")
}

fn is_limit_err(r: Result<(), AdapterError>, size: usize, limit: usize) -> bool {
    matches!(r, Err(AdapterError::Limit { size: s, limit: l }) if s == size && l == limit)
}

// ── 装载/编辑流五路径（算术口径）─────────────────────────────────

#[test]
fn boundary_load_bulk_text_exact_and_over() {
    let ok = sess_limited(7_110_001, L);
    let exact = "x".repeat(L);
    ok.load_bulk_text(EditOrigin::Ai, &exact).expect("== limit 恰过门");
    assert_eq!(ok.__probe_text().len(), L, "内容完整入文档");

    let over = sess_limited(7_110_002, L);
    assert!(
        is_limit_err(over.load_bulk_text(EditOrigin::Ai, &"x".repeat(L + 1)), L + 1, L),
        "limit+1 恰拒（size/limit 精确）"
    );
}

#[test]
fn boundary_load_blocks_exact_and_over() {
    // 账目口径：block_id.len + text.len + 64（d1c-t1 既有估算面）
    // 两块：id(1) + text(63) + 64 = 128B × 2 = 256 == limit
    let exact: Vec<BlockSpec> = (0..2)
        .map(|_| BlockSpec {
            block_id: "b".into(),
            text: "t".repeat(63),
        })
        .collect();
    let ok = sess_limited(7_110_003, L);
    ok.load_blocks(EditOrigin::Ai, &exact).expect("== limit 恰过门");
    assert_eq!(ok.__probe_block_ids().len(), 2);

    let over: Vec<BlockSpec> = (0..2)
        .map(|_| BlockSpec {
            block_id: "b".into(),
            text: "t".repeat(64), // +1B/块 → 总 258
        })
        .collect();
    let s = sess_limited(7_110_004, L);
    assert!(
        is_limit_err(s.load_blocks(EditOrigin::Ai, &over), 258, L),
        "总量超限恰拒（size = 总账目）"
    );
}

#[test]
fn boundary_append_text_exact_and_over() {
    let ok = sess_limited(7_110_005, L);
    ok.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "y".repeat(L) },
    )
    .expect("== limit 恰过门");
    assert_eq!(ok.__probe_text().len(), L);

    let s = sess_limited(7_110_006, L);
    assert!(
        is_limit_err(
            s.edit_stream(
                EditOrigin::Human,
                StreamEdit::AppendText { text: "y".repeat(L + 1) }
            ),
            L + 1,
            L
        ),
        "limit+1 恰拒"
    );
}

#[test]
fn boundary_insert_block_exact_and_over() {
    // id(2) + text(190) + 64 = 256 == limit
    let ok = sess_limited(7_110_007, L);
    ok.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock {
            block: BlockSpec {
                block_id: "id".into(),
                text: "t".repeat(190),
            },
        },
    )
    .expect("== limit 恰过门");
    assert_eq!(ok.__probe_block_ids(), vec!["id".to_string()]);

    let s = sess_limited(7_110_008, L);
    assert!(
        is_limit_err(
            s.edit_stream(
                EditOrigin::Human,
                StreamEdit::InsertBlock {
                    block: BlockSpec {
                        block_id: "id".into(),
                        text: "t".repeat(191), // 257
                    },
                }
            ),
            257,
            L
        ),
        "limit+1 恰拒（账目含 64B 常量）"
    );
}

#[test]
fn boundary_insert_block_before_exact_and_over() {
    // 前置目标块：id(1) + text(1) + 64 = 66B（每次调用独立限额账目）
    let ok = sess_limited(7_110_009, L);
    ok.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock {
            block: BlockSpec {
                block_id: "t".into(),
                text: "x".into(),
            },
        },
    )
    .expect("target insert");
    // before(1) + id(2) + text(189) + 64 = 256 == limit
    ok.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlockBefore {
            before: "t".into(),
            block: BlockSpec {
                block_id: "id".into(),
                text: "t".repeat(189),
            },
        },
    )
    .expect("== limit 恰过门（含 before 键账目）");
    assert_eq!(ok.__probe_block_ids().len(), 2);

    let s = sess_limited(7_110_010, L);
    s.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock {
            block: BlockSpec {
                block_id: "t".into(),
                text: "x".into(),
            },
        },
    )
    .expect("target insert");
    assert!(
        is_limit_err(
            s.edit_stream(
                EditOrigin::Human,
                StreamEdit::InsertBlockBefore {
                    before: "t".into(),
                    block: BlockSpec {
                        block_id: "id".into(),
                        text: "t".repeat(190), // 257
                    },
                }
            ),
            257,
            L
        ),
        "limit+1 恰拒（e-review 发现 2 回归面的精确边界）"
    );
}

// ── d1e-t2 补检三面（此前无限额检查）────────────────────────────

#[test]
fn boundary_remove_block_key_exact_and_over() {
    let ok = sess_limited(7_110_011, L);
    ok.load_blocks(
        EditOrigin::Ai,
        &vec![BlockSpec {
            block_id: "r".into(),
            text: "x".into(),
        }],
    )
    .expect("load");
    // == limit 键：过门后到达搜索面（未命中 → InvalidInput 而非 Limit）
    let miss_exact = "k".repeat(L);
    match ok.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: miss_exact },
    ) {
        Err(AdapterError::InvalidInput(m)) => assert!(m.contains("not found"), "过门后到达搜索面"),
        other => panic!("== limit 应过门到达搜索（InvalidInput），实得 {:?}", other),
    }
    // limit+1 键：恰拒（检查先于搜索——即使键必然未命中也先拒）
    let s = sess_limited(7_110_012, L);
    assert!(
        is_limit_err(
            s.edit_stream(
                EditOrigin::Human,
                StreamEdit::RemoveBlock { block_id: "k".repeat(L + 1) }
            ),
            L + 1,
            L
        ),
        "查找键超限恰拒（与 before 键同纪律）"
    );
    // 命中面不受影响：正常尺寸键删除成功
    let normal = sess_limited(7_110_013, L);
    normal
        .load_blocks(
            EditOrigin::Ai,
            &vec![BlockSpec {
                block_id: "r".into(),
                text: "x".into(),
            }],
        )
        .expect("load");
    normal
        .edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: "r".into() },
        )
        .expect("正常键删除不受新检查影响");
    assert!(normal.__probe_block_ids().is_empty());
}

#[test]
fn boundary_bind_semantic_exact_and_over() {
    let mut ok = sess_limited(7_110_014, L);
    ok.load_blocks(
        EditOrigin::Ai,
        &vec![BlockSpec {
            block_id: "b".into(),
            text: "x".into(),
        }],
    )
    .expect("load");
    // OfficeBookmark 名 == limit：过门并成功绑定（位置快照面）
    ok.bind(&SemanticAnchor::OfficeBookmark {
        name: "n".repeat(L),
        block_index: 0,
    })
    .expect("== limit 恰过门并绑定成功");
    assert_eq!(
        ok.anchor_of(&SemanticAnchor::OfficeBookmark {
            name: "n".repeat(L),
            block_index: 0,
        }),
        Some(ok.anchor_of(&SemanticAnchor::OfficeBookmark {
            name: "n".repeat(L),
            block_index: 0,
        }).expect("已绑定")),
        "映射表持有该绑定"
    );

    let mut s = sess_limited(7_110_015, L);
    assert!(
        matches!(
            s.bind(&SemanticAnchor::OfficeBookmark {
                name: "n".repeat(L + 1),
                block_index: 0,
            }),
            Err(AdapterError::Limit { size, limit }) if size == L + 1 && limit == L
        ),
        "语义锚字符串超限恰拒"
    );
    // MdBlock 变体：超限 block_id 先拒（即使块不存在）
    assert!(
        matches!(
            s.bind(&SemanticAnchor::MdBlock {
                block_id: "m".repeat(L + 1),
            }),
            Err(AdapterError::Limit { .. })
        ),
        "MdBlock 键超限同拒"
    );
}

#[test]
fn boundary_encode_diff_sv_exact_and_over() {
    let ok = sess_limited(7_110_017, L);
    // 全零 256B = 合法空 SV 编码（观测钉定）→ 过门 → Ok
    ok.encode_diff(&vec![0u8; L]).expect("== limit 恰过门（全零 = 空 SV）");

    let s = sess_limited(7_110_018, L);
    assert!(
        matches!(
            s.encode_diff(&vec![0u8; L + 1]),
            Err(AdapterError::Limit { size, limit }) if size == L + 1 && limit == L
        ),
        "remote_sv 超限恰拒（先拒后解码）"
    );
}

// ── wire 工件路径（真实工件钉边界）───────────────────────────────

#[test]
fn boundary_apply_update_exact_and_over() {
    // 真实差量工件：1 块内容对空 SV 的全量 diff（观测 57B）
    let src = CollabAdapter::open(SessionOptions {
        client_id: 7_110_019,
        ..Default::default()
    })
    .expect("open");
    src.load_blocks(
        EditOrigin::Ai,
        &vec![BlockSpec {
            block_id: "u".into(),
            text: "x".repeat(8),
        }],
    )
    .expect("load");
    let probe = sess_limited(7_110_020, usize::MAX / 2);
    let diff = src
        .encode_diff(&probe.state_vector())
        .expect("diff");
    let dlen = diff.len();
    assert!(dlen > 9 && dlen < L, "工件尺寸假设（实得 {dlen}B）");

    let ok = sess_limited(7_110_021, dlen);
    ok.apply_update(&diff).expect("== 工件尺寸恰过门");
    assert_eq!(ok.__probe_block_ids(), vec!["u".to_string()], "内容收敛");

    let s = sess_limited(7_110_022, dlen - 1);
    assert!(
        matches!(
            s.apply_update(&diff),
            Err(AdapterError::Limit { size, limit }) if size == dlen && limit == dlen - 1
        ),
        "limit-1 恰拒（真实工件边界）"
    );
}

#[test]
fn boundary_import_sidecar_exact_and_over() {
    // 真实侧车工件：1 条 OfficeBookmark 绑定（观测 32B）
    let mut src = CollabAdapter::open(SessionOptions {
        client_id: 7_110_023,
        ..Default::default()
    })
    .expect("open");
    src.load_blocks(
        EditOrigin::Ai,
        &vec![BlockSpec {
            block_id: "s".into(),
            text: "x".into(),
        }],
    )
    .expect("load");
    src.bind(&SemanticAnchor::OfficeBookmark {
        name: "bk".into(),
        block_index: 0,
    })
    .expect("bind");
    let sidecar = src.export_anchor_sidecar();
    let slen = sidecar.len();
    assert_eq!(slen, 32, "工件尺寸假设（1 条绑定 = 32B）");

    let mut ok = sess_limited(7_110_024, slen);
    let n = ok.import_anchor_sidecar(&sidecar).expect("== 工件尺寸恰过门");
    assert_eq!(n, 1, "导入条数（导入不依赖内容同步——解析面）");

    let mut s = sess_limited(7_110_025, slen - 1);
    assert!(
        matches!(
            s.import_anchor_sidecar(&sidecar),
            Err(AdapterError::Limit { size, limit }) if size == slen && limit == slen - 1
        ),
        "limit-1 恰拒（e-review 发现 1 修复面的精确边界）"
    );
}
