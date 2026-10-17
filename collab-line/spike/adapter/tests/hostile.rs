//! collab-adapter 敌意输入与不变量测试（d1d-t1 加固轮）。
//!
//! 用户口径：「Tests for malformed/oversized updates, invalid anchors,
//! deleted-anchor resolution, and incompatible skip_gc configurations.
//! Enforce or explicitly validate the all-peers skip_gc invariant at the
//! adapter boundary where possible. Fail closed on detectable mismatches;
//! document what cannot be enforced.」
//!
//! 章节：
//!   §1 malformed updates（垃圾字节 / 截断 / 每前缀截断面 / 引擎未毒化）
//!   §2 oversized inputs（apply_update / 装载三路径 / 边界 +1）
//!   §3 invalid anchors（不存在块 / 越界下标 / 伪造 ID）
//!   §4 deleted-anchor resolution（删除 → 悬空显式化；引擎面细节见 matrix §9）
//!   §5 sidecar 敌意三面 + 尾部冗余（fail-closed）
//!   §6 skip_gc 不变量（构造面强制 / GC 型对端内容级互通 / 盲区登记）
//!
//! §6 诚实边界（p14 实证对齐）：GC 型对端（skip_gc=false 构造或显式 gc）
//! 产出的 update 与干净源在公开 API 观测面（insertions/state_vector/
//! delete_set/extends）完全一致——**wire 级证据存在（墓碑内容被
//! ItemContent::Deleted 标记替换）但 yrs 0.28 公开 API 不可见**；且
//! 适配层自身的 maintenance_gc（B1.3 受控压缩点）产出**同族 wire 形状**
//! （p14 case D：skip_gc=true + 显式 gc = 483B 与未授权 GC 源一致）——
//! 即使可见也无法区分「未授权逐笔 GC」与「受批准维护窗压缩」。故
//! 强制面 = 构造期（B1.1 无条件 skip_gc）+ 部署策略 + 快照门（P7b3）；
//! 本节测试钉定**可执行面**并显式登记盲区。

use collab_adapter::{
    BlockSpec, CollabAdapter, EditOrigin, SemanticAnchor, SessionOptions, StreamEdit,
};

fn sess(client_id: u64) -> CollabAdapter {
    CollabAdapter::open(SessionOptions {
        client_id,
        input_byte_limit: 52 * 1024 * 1024,
    })
    .expect("open")
}

fn blocks(n: usize, prefix: &str) -> Vec<BlockSpec> {
    (0..n)
        .map(|i| BlockSpec {
            block_id: format!("{prefix}-{i:05}"),
            text: format!("text {i}"),
        })
        .collect()
}

/// 确定性伪随机字节流（xorshift64——无 rand 依赖，跨运行稳定）。
fn pseudo_random(n: usize, seed: u64) -> Vec<u8> {
    let mut s = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        out.push((s & 0xFF) as u8);
    }
    out
}

/// 构造一个 GC 型对端源（yrs 直接构造——集成测试可触适配层私有依赖面）：
/// skip_gc=false + 编辑 + 删除 + 显式 gc，产出全量 update。
fn gced_source_update() -> Vec<u8> {
    use yrs::{ClientID, Doc, Options, ReadTxn, StateVector, Text, Transact};
    let mut opts = Options::with_client_id(ClientID::new(7_900_001));
    opts.skip_gc = false;
    let doc = Doc::with_options(opts);
    let text = doc.get_or_insert_text("text");
    for i in 0..50u32 {
        let mut txn = doc.transact_mut();
        let pos = text.len(&txn);
        text.insert(&mut txn, pos, &format!("g{i:02}#"));
        txn.commit();
    }
    {
        let mut txn = doc.transact_mut();
        text.remove_range(&mut txn, 0, 20); // 删前 4 块
        txn.commit();
    }
    {
        let mut txn = doc.transact_mut();
        txn.gc(None);
        txn.commit();
    }
    let bytes = doc
        .transact()
        .encode_diff_v1(&StateVector::default());
    bytes
}

// ── §1 malformed updates ─────────────────────────────────────────

#[test]
fn malformed_garbage_bytes_rejected_or_noop_engine_healthy() {
    let a = sess(7_000_201);
    a.load_blocks(EditOrigin::Ai, &blocks(3, "mg")).expect("load");
    let ids_before = a.__probe_block_ids();

    let inputs: Vec<(&str, Vec<u8>)> = vec![
        ("空字节串", Vec::new()),
        ("全零 64B", vec![0u8; 64]),
        ("伪随机 64B seed1", pseudo_random(64, 1)),
        ("伪随机 256B seed2", pseudo_random(256, 2)),
        ("高字节 64B", vec![0xFFu8; 64]),
    ];
    for (label, bytes) in &inputs {
        // 契约：解码层拒绝（Decode）或无操作（空/合法空更新）——永不
        // panic、永不毒化引擎（后续 apply 正常）。全零面 = lib0 空更新
        // 形状（首 var 0 = 0 客户端）——合法 no-op 是已钉定行为。
        let r = a.apply_update(bytes);
        match r {
            Ok(()) => {
                // no-op 断言：状态零变化
                assert_eq!(a.__probe_block_ids(), ids_before, "{label}: no-op 面状态零变化");
            }
            Err(e) => {
                assert!(
                    matches!(e, collab_adapter::AdapterError::Decode(_)),
                    "{label}: 非 Decode 错误面：{e:?}"
                );
            }
        }
    }
    // 引擎未毒化：合法 update 仍可应用
    let b = sess(7_000_202);
    b.load_blocks(EditOrigin::Ai, &blocks(2, "ok")).expect("load b");
    let diff = b.encode_diff(&a.state_vector()).expect("diff");
    a.apply_update(&diff).expect("poison check: 合法 update 仍可用");
}

#[test]
fn malformed_truncated_update_every_prefix_is_safe() {
    // 取真实 update 的每个前缀（1..len）逐一 apply：要么 Decode 拒、要么
    // 可解码子集安全应用——永不 panic、永不挂起（p9 c1 的 32/32 口径
    // 全前缀矩阵化），引擎始终未毒化。
    let a = sess(7_000_203);
    let src = sess(7_000_204);
    src.load_blocks(EditOrigin::Ai, &blocks(10, "tr")).expect("load");
    let full = src.encode_diff(&a.state_vector()).expect("diff");
    let mut decoded_ok = 0usize;
    let mut rejected = 0usize;
    for cut in 0..full.len() {
        let r = a.apply_update(&full[..cut]);
        match r {
            Ok(()) => decoded_ok += 1,
            Err(collab_adapter::AdapterError::Decode(_)) => rejected += 1,
            Err(other) => panic!("cut={cut}: 意外错误面 {other:?}"),
        }
    }
    // 语义断言：每个前缀都被安全处理（两类之一），无 panic 无挂起
    assert_eq!(
        decoded_ok + rejected,
        full.len(),
        "全前缀安全处理（decoded={decoded_ok} rejected={rejected}）"
    );
    // 引擎健康：全量 update 仍可应用且收敛
    a.apply_update(&full).expect("全量应用（毒化检查）");
    assert_eq!(a.__probe_block_ids().len(), 10);
}

#[test]
fn malformed_valid_update_corrupted_tail() {
    // 合法 update 末字节翻转 → Decode 拒或安全处理；引擎未毒化
    let a = sess(7_000_205);
    let src = sess(7_000_206);
    src.load_blocks(EditOrigin::Ai, &blocks(5, "ct")).expect("load");
    let mut bytes = src.encode_diff(&a.state_vector()).expect("diff");
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    let r = a.apply_update(&bytes);
    if let Err(e) = r {
        assert!(matches!(e, collab_adapter::AdapterError::Decode(_)), "尾翻转错误面：{e:?}");
    }
    let fresh = src.encode_diff(&a.state_vector()).expect("diff fresh");
    a.apply_update(&fresh).expect("毒化检查：合法 update 仍可用");
}

// ── §2 oversized inputs ──────────────────────────────────────────

#[test]
fn oversized_apply_update_rejected_by_limit() {
    let small = CollabAdapter::open(SessionOptions {
        client_id: 7_000_207,
        input_byte_limit: 512,
    })
    .expect("open");
    let big = pseudo_random(513, 42);
    let err = small.apply_update(&big).unwrap_err();
    assert!(matches!(
        err,
        collab_adapter::AdapterError::Limit { size: 513, limit: 512 }
    ));
    // == limit 过门（去到解码层，伪随机字节在 512B 处大概率 Decode 拒——
    // 两面都合法：限额门前置语义 = 「不超限的输入交给解码层」）
    let at_limit = pseudo_random(512, 43);
    let _ = small.apply_update(&at_limit); // 不 panic 即契约满足
}

#[test]
fn oversized_load_paths_rejected() {
    let small = CollabAdapter::open(SessionOptions {
        client_id: 7_000_208,
        input_byte_limit: 256,
    })
    .expect("open");
    // load_bulk_text 超限
    assert!(matches!(
        small.load_bulk_text(EditOrigin::Ai, &"x".repeat(257)),
        Err(collab_adapter::AdapterError::Limit { size: 257, limit: 256 })
    ));
    // AppendText 超限
    assert!(matches!(
        small.edit_stream(
            EditOrigin::Human,
            StreamEdit::AppendText { text: "y".repeat(257) }
        ),
        Err(collab_adapter::AdapterError::Limit { .. })
    ));
    // InsertBlock 超限（block_id + text + 64 估算口径）
    assert!(matches!(
        small.edit_stream(
            EditOrigin::Human,
            StreamEdit::InsertBlock {
                block: BlockSpec {
                    block_id: "z".repeat(200),
                    text: "w".repeat(200),
                }
            }
        ),
        Err(collab_adapter::AdapterError::Limit { .. })
    ));
    // load_blocks 总量超限
    let many: Vec<BlockSpec> = (0..20)
        .map(|i| BlockSpec {
            block_id: format!("ov-{i}"),
            text: "t".repeat(200),
        })
        .collect();
    assert!(matches!(
        small.load_blocks(EditOrigin::Ai, &many),
        Err(collab_adapter::AdapterError::Limit { .. })
    ));
    // InsertBlockBefore 超限（e-review 发现 2 回归锁：六写入路径六查——
    // 此前该路径漏检，同输入 InsertBlock 拒而 InsertBlockBefore 放行）
    assert!(matches!(
        small.edit_stream(
            EditOrigin::Human,
            StreamEdit::InsertBlockBefore {
                before: "any".into(),
                block: BlockSpec {
                    block_id: "z".repeat(200),
                    text: "w".repeat(200),
                }
            }
        ),
        Err(collab_adapter::AdapterError::Limit { .. })
    ));
}

// ── §3 invalid anchors ───────────────────────────────────────────

#[test]
fn invalid_anchor_bind_faces() {
    let mut a = sess(7_000_209);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "ia")).expect("load");
    // MdBlock 不存在
    assert!(matches!(
        a.bind(&SemanticAnchor::MdBlock { block_id: "no-such".into() }),
        Err(collab_adapter::AdapterError::InvalidInput(_))
    ));
    // OfficeBookmark 下标越界（len=5）
    assert!(matches!(
        a.bind(&SemanticAnchor::OfficeBookmark { name: "bk".into(), block_index: 5 }),
        Err(collab_adapter::AdapterError::InvalidInput(_))
    ));
    // PdfAnnotRef 下标越界
    assert!(matches!(
        a.bind(&SemanticAnchor::PdfAnnotRef { page: 0, label: "l".into(), block_index: 99 }),
        Err(collab_adapter::AdapterError::InvalidInput(_))
    ));
    // 空文档绑定任意 MdBlock → InvalidInput（空集合无锚）
    let mut empty = sess(7_000_210);
    assert!(matches!(
        empty.bind(&SemanticAnchor::MdBlock { block_id: "any".into() }),
        Err(collab_adapter::AdapterError::InvalidInput(_))
    ));
}

#[test]
fn invalid_anchor_resolve_fabricated_ids_dangle() {
    let mut a = sess(7_000_211);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "fa")).expect("load");
    // 伪造 client/clock（从未存在）→ 悬空显式化
    let fabricated = collab_adapter::CrdtAnchor { client: 999_999, clock: 12345 };
    assert!(matches!(
        a.resolve(fabricated),
        Err(collab_adapter::AdapterError::AnchorDangling { client: 999_999, clock: 12345 })
    ));
    // 存在 client 但 clock 越界
    let real = a
        .bind(&SemanticAnchor::MdBlock { block_id: "fa-00000".into() })
        .expect("bind");
    let beyond = collab_adapter::CrdtAnchor { client: real.client, clock: u32::MAX };
    assert!(matches!(
        a.resolve(beyond),
        Err(collab_adapter::AdapterError::AnchorDangling { .. })
    ));
}

// ── §4 deleted-anchor resolution ─────────────────────────────────

#[test]
fn deleted_anchor_resolution_never_returns_neighbor() {
    // R4.3 反例钉定：删除目标块后 resolve 绝不静默解析到邻居块
    let mut a = sess(7_000_212);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "nb")).expect("load");
    let anchor = a
        .bind(&SemanticAnchor::MdBlock { block_id: "nb-00002".into() })
        .expect("bind");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "nb-00002".into() },
    )
    .expect("remove");
    let err = a.resolve(anchor).unwrap_err();
    assert!(matches!(err, collab_adapter::AdapterError::AnchorDangling { .. }));
    // 邻居不受影响（nb-00001 / nb-00003 仍可独立绑定解析）
    let n1 = a
        .bind(&SemanticAnchor::MdBlock { block_id: "nb-00001".into() })
        .expect("neighbor 1");
    let n3 = a
        .bind(&SemanticAnchor::MdBlock { block_id: "nb-00003".into() })
        .expect("neighbor 3");
    assert_eq!(a.resolve(n1).unwrap().block_id, "nb-00001");
    assert_eq!(a.resolve(n3).unwrap().block_id, "nb-00003");
}

// ── §5 sidecar 敌意面 ────────────────────────────────────────────

#[test]
fn sidecar_hostile_faces_fail_closed() {
    let mut a = sess(7_000_213);
    a.load_blocks(EditOrigin::Ai, &blocks(5, "sh")).expect("load");
    let _ = a
        .bind(&SemanticAnchor::MdBlock { block_id: "sh-00001".into() })
        .expect("bind");
    let good = a.export_anchor_sidecar();

    // ① 坏 magic
    let mut bad_magic = good.clone();
    bad_magic[0] = b'X';
    assert!(matches!(
        a.import_anchor_sidecar(&bad_magic),
        Err(collab_adapter::AdapterError::SidecarFormat(_))
    ));
    // ② 坏版本
    let mut bad_ver = good.clone();
    bad_ver[4] = 9;
    assert!(matches!(
        a.import_anchor_sidecar(&bad_ver),
        Err(collab_adapter::AdapterError::SidecarFormat(_))
    ));
    // ③ 截断（逐长度面：每一点截断都被拒）
    for cut in 0..good.len() {
        let r = a.import_anchor_sidecar(&good[..cut]);
        assert!(
            matches!(r, Err(collab_adapter::AdapterError::SidecarFormat(_))),
            "cut={cut}: 截断面必须 fail-closed"
        );
    }
    // ④ 未知 tag（首条目 tag 位改为 9——magic/ver/count 之后）
    let mut bad_tag = good.clone();
    bad_tag[9] = 9;
    assert!(matches!(
        a.import_anchor_sidecar(&bad_tag),
        Err(collab_adapter::AdapterError::SidecarFormat(_))
    ));
    // ⑤ 声明 count 与实际条目不符（count 改大 → 载荷越界拒）
    let mut bad_count = good.clone();
    let n = u32::from_le_bytes([bad_count[5], bad_count[6], bad_count[7], bad_count[8]]);
    let inflated = n + 10;
    bad_count[5..9].copy_from_slice(&inflated.to_le_bytes());
    assert!(matches!(
        a.import_anchor_sidecar(&bad_count),
        Err(collab_adapter::AdapterError::SidecarFormat(_))
    ));
    // ⑤b 敌意极端 count（e-review 发现 1 回归锁：u32::MAX 预分配放大
    // abort 反例——13B 载荷声明 40 亿条目必须先拒后分配，绝不 abort）
    let mut extreme = good.clone();
    extreme[5..9].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(
        matches!(
            a.import_anchor_sidecar(&extreme),
            Err(collab_adapter::AdapterError::SidecarFormat(_))
        ),
        "极端 count 必须拒（物理上界钳制）而非预分配 abort"
    );
    // ⑤c 侧车字节限额（B1.6 同面收口——小限额会话拒绝超限侧车）
    let mut small = CollabAdapter::open(SessionOptions {
        client_id: 7_000_222,
        input_byte_limit: 32,
    })
    .expect("open small");
    assert!(good.len() > 32, "夹具尺寸需大于小限额（实际 {}B）", good.len());
    assert!(matches!(
        small.import_anchor_sidecar(&good),
        Err(collab_adapter::AdapterError::Limit { .. })
    ));
    // ⑥ 尾部冗余字节（fail-closed 硬化面：拒绝而非静默忽略）
    let mut trailing = good.clone();
    trailing.extend_from_slice(&[0x00, 0x01, 0x02]);
    assert!(
        matches!(
            a.import_anchor_sidecar(&trailing),
            Err(collab_adapter::AdapterError::SidecarFormat(_))
        ),
        "尾部冗余字节必须 fail-closed（不静默忽略）"
    );
    // 敌意面后映射表不被破坏：原 good 仍可导入
    assert_eq!(a.import_anchor_sidecar(&good).unwrap(), 1);
}

// ── §6 skip_gc 不变量 ────────────────────────────────────────────

#[test]
fn construction_enforces_skip_gc_unconditionally() {
    // B1.1：构造面强制——SessionOptions 无该开关（类型面即证明），
    // 任意构造的会话 skip_gc 恒真（探针观测）。client_id 取 53 位域内
    // 代表值（含大幅值）。
    for client in [1u64, 42, 7_000_214, 1 << 52] {
        let a = CollabAdapter::open(SessionOptions {
            client_id: client,
            input_byte_limit: 1024,
        })
        .expect("open");
        assert!(a.__probe_skip_gc(), "client={client}: skip_gc 无条件生效");
    }
    // 默认选项同样生效
    assert!(sess(7_000_215).__probe_skip_gc());
}

#[test]
fn invalid_client_id_over_53_bits_rejected() {
    // fail-closed 硬化面：client_id 超 53 位（yjs 兼容域）→ 构造期拒绝
    // （yrs ClientID::new 仅 debug_assert——release 构建下超界值会静默
    // 破坏 ID 编码；适配层补上运行期显式拒绝）
    for bad in [1u64 << 53, (1u64 << 53) + 1, u64::MAX] {
        let r = CollabAdapter::open(SessionOptions {
            client_id: bad,
            input_byte_limit: 1024,
        });
        assert!(
            matches!(r, Err(collab_adapter::AdapterError::InvalidInput(_))),
            "client_id={bad} 必须构造期拒绝"
        );
    }
    // 边界内最大值恰好可过
    assert!(CollabAdapter::open(SessionOptions {
        client_id: (1u64 << 53) - 1,
        input_byte_limit: 1024,
    })
    .is_ok());
}

#[test]
fn gced_peer_update_applies_and_content_converges() {
    // 「incompatible skip_gc configurations」的可执行面：GC 型对端的
    // update 在内容层可应用、收敛、且本端（skip_gc）保持健康。
    // （GC 型源的构造经 yrs 直接进行——集成测试依赖面，非公开 API 面。）
    let mut a = sess(7_000_216);
    let gc_bytes = gced_source_update();
    a.apply_update(&gc_bytes).expect("GC 型对端 update 应用（内容层）");
    // 内容收敛：50 块 × 4 字符（"gNN#"）- 删前 20 字符 = 前 5 块
    // （g00#-g04#）删除，存活面自 g05# 起
    let text = a.__probe_text();
    assert!(text.contains("g05#"), "GC 源存活内容在场（g05#）");
    assert!(!text.contains("g00#"), "GC 源已删内容不在场（g00#）");
    assert!(!text.contains("g04#"), "删除边界恰好落在第 5 块尾（g04# 不在场）");
    // 本端保持健康：继续编辑 / 撤销面 / 维护 gc / 快照全部可用
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "local-after-gc".into() },
    )
    .expect("edit after gc-source apply");
    a.maintenance_gc().expect("maintenance_gc after gc-source apply");
    let snap = a.authoritative_snapshot().expect("snapshot after gc-source apply");
    let b = sess(7_000_218);
    b.apply_update(&snap).expect("snapshot restores");
    assert_eq!(b.__probe_text(), a.__probe_text(), "快照恢复内容一致");
}

#[test]
fn skip_gc_blind_spot_is_documented_reality() {
    // 盲区钉定（p14 实证）：适配层公开 API 无法区分 GC 型对端与干净
    // 对端的 update——两者 apply 结果同为 Ok 且内容层同收敛。本测试
    // 即该事实的活文档：两源同内容形状，apply 均 Ok。
    // （强制面 = 构造期 B1.1 + 部署策略 + 快照门 P7b3；wire 级证据
    // ItemContent::Deleted 标记对 yrs 0.28 公开 API 不可见，且
    // maintenance_gc 产出同族形状——见文件头诚实边界注。）
    let clean_dst = sess(7_000_219);
    let gced_dst = sess(7_000_220);

    // 干净源（skip_gc 适配层对端）
    let clean_src = sess(7_000_221);
    clean_src.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "same".into() },
    )
    .expect("clean edit");
    let clean_bytes = clean_src.encode_diff(&clean_dst.state_vector()).expect("diff");
    // GC 型源（yrs 直接构造 + gc）
    let gc_bytes = gced_source_update();

    let r1 = clean_dst.apply_update(&clean_bytes);
    let r2 = gced_dst.apply_update(&gc_bytes);
    assert!(r1.is_ok() && r2.is_ok(), "两源 apply 同为 Ok——公开 API 无区分面（盲区事实）");
}
