//! 撤销/重做 × 锚点身份回归（yrs-d 后续钉定轮，2026-10-17）。
//!
//! 覆盖面（用户指令：「Add regression coverage for deleted-anchor
//! undo/redo and logical-ID mapping, without inventing unsupported yrs
//! behavior」）：全部断言**先观测后钉定**（探针实录见 worklog 本轮），
//! 不发明 yrs 未支持的行为——复活=新 ID 族是上游语义（ItemPtr::redo
//! 走 redone 指针，原墓碑不复活），本套件钉定的是**适配层在该语义
//! 上的可依赖行为面**：
//! 1. 旧锚点在 delete/undo/redo 全链上永不静默漂移（恒 AnchorDangling
//!    或精确解析，无邻居错配）；
//! 2. 每次复活铸造新 ID → 重新绑定是锚点跟随复活的唯一路径
//!    （redone 链映射 = 上游 API 请求候选，非本层职责）；
//! 3. 深度记账对称守恒（undo↔redo 双栈镜像；p12「redo 返回栈项」
//!    B3.4 观测面的确定性形状）；
//! 4. 复活的新 ID 语义跨端收敛（对端同步后旧锚点同样悬空、重绑定
//!    得到与作者端相同的 CRDT ID——逻辑 ID 映射跨端稳定）；
//! 5. 侧车陈旧映射显式悬空（导入不校验存活，解析是唯一权威面）；
//! 6. maintenance_gc 收走墓碑后悬空仍显式（不 panic 不返回邻居）。

use collab_adapter::{
    AdapterError, BlockSpec, CollabAdapter, EditOrigin, SemanticAnchor, SessionOptions,
    StreamEdit,
};

fn sess(client_id: u64) -> CollabAdapter {
    CollabAdapter::open(SessionOptions {
        client_id,
        ..Default::default()
    })
    .expect("open")
}

fn one(id: &str) -> BlockSpec {
    BlockSpec {
        block_id: id.into(),
        text: format!("text of {id}"),
    }
}

fn bind_md(a: &mut CollabAdapter, block_id: &str) -> collab_adapter::CrdtAnchor {
    a.bind(&SemanticAnchor::MdBlock {
        block_id: block_id.into(),
    })
    .expect("bind")
}

fn is_dangling(a: &CollabAdapter, anchor: collab_adapter::CrdtAnchor) -> bool {
    matches!(
        a.resolve(anchor),
        Err(AdapterError::AnchorDangling { client, clock }) if client == anchor.client && clock == anchor.clock
    )
}

/// 撤销插入 → 重做插入：块内容回归但锚点恒悬空（重做同样铸造新 ID 族
/// ——探针观测 1：bind (c,3) → undo 后悬空 → redo 后仍悬空 → 重绑定
/// (c,6)）。原锚点在整条 undo/redo 链上**永不复活**。
#[test]
fn redo_of_undone_insert_mints_new_id_old_anchor_stays_dangling() {
    let mut a = sess(7_100_201);
    // 装载（Ai 栈）与插入（Human 栈）分源：Human 面可独立撤销插入
    a.load_blocks(EditOrigin::Ai, &vec![one("ri-0")])
        .expect("load");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock { block: one("ri-1") },
    )
    .expect("insert");
    let anchor = bind_md(&mut a, "ri-1");

    assert!(a.undo(EditOrigin::Human), "撤插入");
    assert!(!a.__probe_block_ids().contains(&"ri-1".to_string()));
    assert!(
        is_dangling(&a, anchor),
        "撤销插入后锚点悬空（内容缺席）"
    );

    assert!(a.redo(EditOrigin::Human), "重做插入");
    assert!(
        a.__probe_block_ids().contains(&"ri-1".to_string()),
        "重做后块内容回归"
    );
    assert!(
        is_dangling(&a, anchor),
        "重做插入 = 新 ID 内容项：原锚点保持悬空（不静默漂移到新项）"
    );

    // 重绑定是跟随复活的唯一路径：新锚点 ≠ 原锚点且可解析
    let anchor2 = bind_md(&mut a, "ri-1");
    assert_ne!(
        (anchor2.client, anchor2.clock),
        (anchor.client, anchor.clock),
        "重做复活的块 = 新 CRDT ID"
    );
    let resolved = a.resolve(anchor2).expect("resolve new anchor");
    assert_eq!(resolved.block_id, "ri-1");
}

/// 逐事务分离项的 delete/undo/redo 深度记账：双栈镜像对称、项数守恒
/// （探针观测 2 精修：[A]1/0 [B]2/0 [C]1/1 [D]0/2 [E]1/1 [F]2/0；
/// 捕获窗 0 后逐事务分离是固有语义，无需 sleep 制造窗外分离）。
/// 「redo 返回栈项」= 重做会同时弹 redo 项并压新 undo 项——B3.4
/// 无界增长观测面的确定性形状（栈深与编辑史同阶，见 yrs-d §3/§4）。
#[test]
fn delete_undo_redo_cycle_depth_bookkeeping_is_symmetric() {
    let mut a = sess(7_100_202);
    a.load_blocks(EditOrigin::Ai, &vec![one("db-0")]).expect("load");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::InsertBlock { block: one("db-1") },
    )
    .expect("insert");
    assert_eq!((a.undo_depth(EditOrigin::Human), a.redo_depth(EditOrigin::Human)), (1, 0), "[A] 插入后");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "db-1".into() },
    )
    .expect("remove");
    assert_eq!((a.undo_depth(EditOrigin::Human), a.redo_depth(EditOrigin::Human)), (2, 0), "[B] 删除后（逐事务分离项）");

    a.undo(EditOrigin::Human); // 撤删除 → 复活（新 ID）
    assert_eq!((a.undo_depth(EditOrigin::Human), a.redo_depth(EditOrigin::Human)), (1, 1), "[C] 撤删除后");
    assert!(a.__probe_block_ids().contains(&"db-1".to_string()));

    a.undo(EditOrigin::Human); // 撤插入 → 消失
    assert_eq!((a.undo_depth(EditOrigin::Human), a.redo_depth(EditOrigin::Human)), (0, 2), "[D] 撤插入后");
    assert!(!a.__probe_block_ids().contains(&"db-1".to_string()));

    a.redo(EditOrigin::Human); // 重做插入 → 回归（新 ID）
    assert_eq!((a.undo_depth(EditOrigin::Human), a.redo_depth(EditOrigin::Human)), (1, 1), "[E] 重做插入后：redo 弹 1 压 1");
    assert!(a.__probe_block_ids().contains(&"db-1".to_string()));

    a.redo(EditOrigin::Human); // 重做删除 → 再消失
    assert_eq!((a.undo_depth(EditOrigin::Human), a.redo_depth(EditOrigin::Human)), (2, 0), "[F] 重做删除后");
    assert!(!a.__probe_block_ids().contains(&"db-1".to_string()));
}

/// 多循环删除/撤销：每次复活都铸造**互异**的新 ID；全部历史锚点恒
/// 悬空；重绑定逐次替换逻辑映射表（anchor_of 恒返回最新绑定）。
#[test]
fn revive_cycles_mint_distinct_ids_and_rebind_replaces_logical_mapping() {
    let mut a = sess(7_100_203);
    a.load_blocks(EditOrigin::Ai, &vec![one("rv-0"), one("rv-1")])
        .expect("load");
    let semantic = SemanticAnchor::MdBlock { block_id: "rv-1".into() };
    let a1 = a.bind(&semantic).expect("bind1");
    let mut seen = vec![a1];

    for cycle in 0..3 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: "rv-1".into() },
        )
        .expect("remove");
        assert!(is_dangling(&a, *seen.last().unwrap()), "删除后锚点悬空（cycle {cycle}）");
        assert!(a.undo(EditOrigin::Human), "撤删除（cycle {cycle}）");
        assert!(
            a.__probe_block_ids().contains(&"rv-1".to_string()),
            "复活（cycle {cycle}）"
        );
        // 重绑定 → 新锚点，且与此前所有锚点互异
        let an = a.bind(&semantic).expect("rebind");
        for old in &seen {
            assert_ne!(
                (an.client, an.clock),
                (old.client, old.clock),
                "cycle {cycle} 复活锚点与历史锚点互异（新 ID 族）"
            );
        }
        seen.push(an);
        // 逻辑映射表替换：anchor_of 恒返回最新绑定
        assert_eq!(a.anchor_of(&semantic), Some(an), "映射表最新绑定（cycle {cycle}）");
        // 最新锚点可解析；全部历史锚点恒悬空
        assert_eq!(a.resolve(an).expect("resolve latest").block_id, "rv-1");
        for old in &seen[..seen.len() - 1] {
            assert!(is_dangling(&a, *old), "历史锚点恒悬空（cycle {cycle}）");
        }
    }
}

/// 跨端复活收敛（探针观测 4）：删除态同步 → 对端旧锚点悬空；作者端
/// 撤销复活 → 同步 → 对端看到复活块、旧锚点**同样**悬空（新 ID 语义
/// 收敛而非本地怪癖）；对端重绑定得到与作者端**完全相同**的 CRDT ID
/// （复活块由作者端铸造，逻辑 ID 映射跨端稳定）。
#[test]
fn cross_peer_revive_convergence_and_stable_logical_mapping() {
    let mut author = sess(7_100_204);
    let mut peer = sess(7_100_205);
    author
        .load_blocks(EditOrigin::Ai, &vec![one("cp-0"), one("cp-1")])
        .expect("load");
    let old_anchor = bind_md(&mut author, "cp-1");

    author
        .edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: "cp-1".into() },
        )
        .expect("remove");
    // 同步删除态
    let sv = peer.state_vector();
    peer.apply_update(&author.encode_diff(&sv).expect("diff")).expect("apply");
    assert!(is_dangling(&peer, old_anchor), "对端看到删除：旧锚点悬空");

    // 作者端撤销 → 复活（新 ID）→ 同步复活态
    assert!(author.undo(EditOrigin::Human), "作者端撤删除");
    let author_rebind = bind_md(&mut author, "cp-1");
    let sv2 = peer.state_vector();
    peer.apply_update(&author.encode_diff(&sv2).expect("diff2")).expect("apply2");
    assert!(
        peer.__probe_block_ids().contains(&"cp-1".to_string()),
        "对端看到复活块"
    );
    assert!(
        is_dangling(&peer, old_anchor),
        "复活 = 新 ID 族在对端同样悬空（跨端收敛）"
    );
    let peer_rebind = bind_md(&mut peer, "cp-1");
    assert_eq!(
        (peer_rebind.client, peer_rebind.clock),
        (author_rebind.client, author_rebind.clock),
        "对端重绑定 = 作者端新锚点（复活块 ID 由作者端铸造，逻辑映射跨端一致）"
    );
}

/// 侧车陈旧映射：目标已删除的绑定导出后导入对端——导入不校验存活
/// （n 计数含陈旧项），解析是唯一权威面（显式悬空，不静默邻居）。
#[test]
fn stale_sidecar_mapping_imports_and_dangles_explicitly() {
    let mut a = sess(7_100_206);
    let mut b = sess(7_100_207);
    a.load_blocks(EditOrigin::Ai, &vec![one("st-0"), one("st-1")])
        .expect("load");
    let stale = bind_md(&mut a, "st-1");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "st-1".into() },
    )
    .expect("remove");
    let sidecar = a.export_anchor_sidecar(); // 含陈旧映射
    // 对端同步删除态 + 导入侧车
    let sv = b.state_vector();
    b.apply_update(&a.encode_diff(&sv).expect("diff")).expect("apply");
    let n = b.import_anchor_sidecar(&sidecar).expect("import");
    assert_eq!(n, 1, "陈旧映射可导入（存活校验不在导入面）");
    assert_eq!(
        b.anchor_of(&SemanticAnchor::MdBlock { block_id: "st-1".into() }),
        Some(stale),
        "逻辑映射原样还原"
    );
    assert!(
        is_dangling(&b, stale),
        "解析面显式悬空（权威面在 resolve，不在导入）"
    );
}

/// maintenance_gc 收走墓碑后：已删锚点解析仍显式悬空（不 panic、
/// 不返回邻居——探针观测 3）。GC 后 wire 已压缩（110→73B 形状），
/// 悬空判定不依赖墓碑在场与否。
#[test]
fn dangling_anchor_explicit_after_maintenance_gc() {
    let mut a = sess(7_100_208);
    a.load_blocks(EditOrigin::Ai, &vec![one("gc-0"), one("gc-1")])
        .expect("load");
    let anchor = bind_md(&mut a, "gc-1");
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock { block_id: "gc-1".into() },
    )
    .expect("remove");
    let (before, after) = a.maintenance_gc().expect("gc");
    assert!(after < before, "维护窗压缩生效（{before} → {after}B）");
    assert!(
        is_dangling(&a, anchor),
        "GC 收走墓碑后悬空仍显式（AnchorDangling，非邻居非 panic）"
    );
    // 存活块锚点不受维护窗影响
    let alive = bind_md(&mut a, "gc-0");
    assert_eq!(a.resolve(alive).expect("alive resolve").block_id, "gc-0");
}
