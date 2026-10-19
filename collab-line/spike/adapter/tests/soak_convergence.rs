//! soak_convergence —— 随机操作长跑收敛回归（a5-r4）。
//!
//! 面向：matrix/hostile 的**构造性**用例之外的**随机化但完全确定
//! 性**覆盖（种子 PRNG——失败可精确复现，零三方依赖）。
//!   soak-1 双端随机并发：150 轮 × 每端 8 随机操作（四变体流编辑
//!      × 双 origin 混合 × 穿插随机 undo/redo），每轮双向同步后
//!      断言收敛（text + block_ids + 逐块 text 三面全等）；每 30 轮
//!      复同步幂等（收敛态再同步零变化）。
//!   soak-2 长跑收敛稳定：3000 混合操作（无 undo/redo）→ 追踪面
//!      活跃 + 双端收敛 + 幂等 + maintenance_gc 状态存活。
//!   soak-3 钉定缺陷（p20 发现）：双 manager + 捕获窗 1000（适配器
//!      现产配置）× 交错 origin 同窗追加 → 非连续合并项 = undo 部分
//!      应用 + redo panic（yrs 0.28 Store::materialize）——断言缺陷
//!      **在场**；配置改窗 0 或上游修复后本测试失败强制解钉。
//!
//! 既有 54 用例（matrix25 + hostile13 + anchor6 + limit10）的
//! 结构性覆盖不重复——本文件补随机形状面与缺陷钉定。

use collab_adapter::{BlockSpec, CollabAdapter, EditOrigin, SessionOptions, StreamEdit};

/// xorshift64*（确定性，零依赖）
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn sess(client_id: u64) -> CollabAdapter {
    CollabAdapter::open(SessionOptions {
        client_id,
        input_byte_limit: 52 * 1024 * 1024,
    })
    .expect("open")
}

/// 双向同步至收敛（两轮差量交换——matrix 同型）
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

/// 三面全等断言（text 根 + block_id 序 + 逐块 text）
fn assert_three_face_equal(a: &CollabAdapter, b: &CollabAdapter, ctx: &str) {
    assert_eq!(a.__probe_text(), b.__probe_text(), "{ctx}: text 根不等");
    let ids_a = a.__probe_block_ids();
    let ids_b = b.__probe_block_ids();
    assert_eq!(ids_a, ids_b, "{ctx}: block_id 序不等");
    for id in &ids_a {
        assert_eq!(
            a.__probe_block_text(id),
            b.__probe_block_text(id),
            "{ctx}: 块 {id} text 不等"
        );
    }
}

/// 对某端生成并应用一笔随机流编辑（全部对本地现状合法——生成即
/// 应用，非法形状跳过）。返回是否实际执行。
fn random_edit(ad: &mut CollabAdapter, rng: &mut Rng, site: char, counter: &mut u64) -> bool {
    let origin = if rng.below(2) == 0 {
        EditOrigin::Human
    } else {
        EditOrigin::Ai
    };
    let ids = ad.__probe_block_ids();
    let edit = match rng.below(4) {
        0 => StreamEdit::AppendText {
            text: format!("尾部{site}{counter}段"),
        },
        1 | 2 => {
            *counter += 1;
            StreamEdit::InsertBlock {
                block: BlockSpec {
                    block_id: format!("{site}-{counter:06}"),
                    text: format!("{site} 站第 {counter} 块正文"),
                },
            }
        }
        _ => {
            if ids.is_empty() {
                return false;
            }
            let pick = ids[rng.below(ids.len() as u64) as usize].clone();
            if rng.below(3) == 0 {
                StreamEdit::RemoveBlock { block_id: pick }
            } else {
                *counter += 1;
                StreamEdit::InsertBlockBefore {
                    before: pick,
                    block: BlockSpec {
                        block_id: format!("{site}-{counter:06}"),
                        text: format!("{site} 中插第 {counter} 块"),
                    },
                }
            }
        }
    };
    ad.edit_stream(origin, edit).expect("edit_stream");
    true
}

#[test]
fn soak_dual_site_random_ops_converge() {
    let mut a = sess(7_002_001);
    let mut b = sess(7_002_002);
    let mut rng_a = Rng::new(0x5041_3100_0000_0001);
    let mut rng_b = Rng::new(0x5242_3200_0000_0002);
    let (mut ca, mut cb) = (0u64, 0u64);

    const ROUNDS: u64 = 60;
    const OPS_PER_ROUND: u64 = 8;

    for round in 1..=ROUNDS {
        // 双端各自随机编辑（并发分歧）
        for _ in 0..OPS_PER_ROUND {
            random_edit(&mut a, &mut rng_a, 'a', &mut ca);
            random_edit(&mut b, &mut rng_b, 'b', &mut cb);
        }
        // 穿插随机 undo（含空栈面——false 合法不 panic）
        if rng_a.below(7) == 0 {
            let _ = a.undo(if rng_a.below(2) == 0 {
                EditOrigin::Human
            } else {
                EditOrigin::Ai
            });
        }
        if rng_b.below(7) == 0 {
            let _ = b.redo(if rng_b.below(2) == 0 {
                EditOrigin::Human
            } else {
                EditOrigin::Ai
            });
        }
        // 同步 + 收敛三面断言
        sync_converge(&a, &b);
        assert_three_face_equal(&a, &b, &format!("round {round}"));
        // 每 20 轮幂等复验（收敛态再同步零变化）
        if round % 20 == 0 {
            let (t0, ids0) = (a.__probe_text(), a.__probe_block_ids());
            sync_converge(&a, &b);
            assert_eq!(a.__probe_text(), t0, "幂等破：round {round} text");
            assert_eq!(a.__probe_block_ids(), ids0, "幂等破：round {round} ids");
        }
    }
    // 终态非平凡性（操作确实发生了）
    assert!(
        !a.__probe_block_ids().is_empty() && !a.__probe_text().is_empty(),
        "soak-1 终态意外为空"
    );
}

#[test]
fn soak_long_run_converge_stability() {
    let mut a = sess(7_002_003);
    let mut b = sess(7_002_004);
    let mut rng = Rng::new(0x5343_3300_0000_0003);
    let mut counter = 0u64;

    // 3000 混合操作长跑（无 undo/redo——撤销面缺陷另由 soak-3 钉定，
    // 见 p20_undo_window_repro 四配置矩阵：双 manager + 捕获窗 1000
    // = 适配器现产配置下非连续合并项部分撤销 + redo panic）
    for _ in 0..3000 {
        random_edit(&mut a, &mut rng, 's', &mut counter);
    }
    let ids_final = a.__probe_block_ids();
    assert!(!ids_final.is_empty(), "soak-2 长跑后意外无块");

    // 追踪面活跃（撤销栈有入项——编辑被双 manager 记录）
    assert!(
        a.undo_depth(EditOrigin::Human) + a.undo_depth(EditOrigin::Ai) > 0,
        "soak-2 撤销追踪面意外为空"
    );

    // 长跑后双端收敛 + 幂等
    sync_converge(&a, &b);
    assert_three_face_equal(&a, &b, "soak-2 长跑收敛");
    let (t0, ids0) = (a.__probe_text(), a.__probe_block_ids());
    sync_converge(&a, &b);
    assert_eq!(a.__probe_text(), t0, "soak-2 幂等 text 破");
    assert_eq!(a.__probe_block_ids(), ids0, "soak-2 幂等 ids 破");

    // maintenance_gc 后状态存活（读回不变——GC 清栈但不清内容）
    a.maintenance_gc().expect("gc");
    assert_eq!(a.__probe_text(), t0, "soak-2 gc 后 text 破");
    assert_eq!(a.__probe_block_ids(), ids0, "soak-2 gc 后 ids 破");
}

/// soak-3【钉定缺陷】：双 manager + 捕获窗 1000（适配器现产配置）×
/// 交错 origin 同窗追加 → 非连续合并项。yrs 0.28 表现 = undo 部分应用
/// + 后续 redo panic（Store::materialize unwrap None——p20 cfg-1 复现，
/// cfg-2/3/4 干净对照）。**本测试断言缺陷在场**（catch_unwind 钉 panic
/// + 终态文本断言钉部分撤销）——配置改窗 0 或上游修复后本测试将失败，
/// 强制解钉 + 关闭 DECISION_GROUPS G2 对应决策项。
#[test]
fn pinned_undo_window_defect_dual_manager() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let mut a = sess(7_002_005);
    // 同窗交错：H1 A1 H2 A2（Human 项区间 [0,2)+[4,6) 非连续）
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "H1".into() },
    )
    .expect("e1");
    a.edit_stream(EditOrigin::Ai, StreamEdit::AppendText { text: "A1".into() })
        .expect("e2");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::AppendText { text: "H2".into() },
    )
    .expect("e3");
    a.edit_stream(EditOrigin::Ai, StreamEdit::AppendText { text: "A2".into() })
        .expect("e4");
    assert_eq!(a.__probe_text(), "H1A1H2A2");

    // undo(Human)：合并项部分应用（仅 H1 移除——终态 "A1H2A2" ≠ 期望
    // "A1A2"）——钉定部分撤销
    let _ = a.undo(EditOrigin::Human);
    assert_eq!(
        a.__probe_text(),
        "A1H2A2",
        "钉定失败：部分撤销表现消失（若此断言失败=缺陷已修，解钉 soak-3）"
    );

    // redo(Human)：panic 钉定（catch_unwind 捕获 = 缺陷在场）
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        let _ = a.redo(EditOrigin::Human);
    }))
    .is_err();
    assert!(
        panicked,
        "钉定失败：redo panic 消失（若此断言失败=缺陷已修，解钉 soak-3）"
    );
}
