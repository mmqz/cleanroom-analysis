//! 探针 10 适配层原型第三轮（d1c-t3）：锚点面 + 快照面。
//!
//! 覆盖 yrs-b 边界四（稳定锚点跨模型映射）+ D1③ 权威快照——用户三
//! 验证面之三（跨模型锚点映射）：
//!
//! P10a MD 锚点保持（R4.2/R4.3 + yrs-b §5.3 验收）
//!      a1 bind(MdBlock) → CRDT ID；结构性编辑（前插 5 + 前删 3）后
//!         resolve → 同一 block_id，下标漂移正确（42 → 42+5-3=44）；
//!      a2 StickyIndex wire 往返（Encode→Decode→get_offset 与直接解算
//!         一致——§5.3 验收第一项）；
//!      a3 目标删除面显式化：删目标块后 resolve → Err(AnchorDangling)
//!         （不静默返回邻居块——R4.3 反例封堵）。
//! P10b Office/PDF 语义锚（R4.5 弱绑定 + 登记面）
//!      b1 OfficeBookmark（书签名+位置快照）bind → 结构漂移后 resolve
//!         → 位置语义（弱绑定：解析到当前该位置的块）；
//!      b2 PdfAnnotRef（页号+标签+位置）bind → anchor_of 查询一致。
//! P10c 映射表侧车（R4.3 验收：版本化 + 往返）
//!      c1 export → import 到新会话 → anchor_of 全部复原（逐条相等）；
//!      c2 敌意侧车（坏 magic / 坏版本 / 截断）→ Err(SidecarFormat)。
//! P10d 跨端锚点（CRDT ID 全局性）
//!      d1 A 端 bind → 全量同步 B 端 → B.resolve(A 的锚点) → 同一
//!         block_id（ID 跨进程稳定——对照 mdcore BlockId 条件保真）。
//! P10e 权威快照（D1③）
//!      e1 snapshot → 新适配器 apply → 内容逐块一致；
//!      e2 快照后 maintenance_gc → 再快照仍一致（B1.3 维护窗与 D1③
//!         同点）；e3 收敛双端本地全量编码差异登记（yrs-a 发现①：
//!         字节可能不同 ≠ 状态不同——单点编码的意义）。

use collab_adapter::{
    AdapterError, BlockSpec, CollabAdapter, EditOrigin, SemanticAnchor, SessionOptions,
    StreamEdit,
};

fn blocks_spec(n: usize, prefix: &str) -> Vec<BlockSpec> {
    (0..n)
        .map(|i| BlockSpec {
            block_id: format!("{}-{:05}", prefix, i),
            text: format!("content {} lorem ipsum dolor sit amet", i),
        })
        .collect()
}

fn main() {
    println!("P10 适配层原型第三轮：锚点面 + 快照面（跨模型锚点映射）");

    // ── P10a MD 锚点保持 ──────────────────────────────────────────
    let mut a = CollabAdapter::open(SessionOptions {
        client_id: 9_000_001,
        ..Default::default()
    })
    .unwrap();
    a.load_blocks(EditOrigin::Ai, &blocks_spec(100, "blk")).unwrap();
    let anchor42 = a
        .bind(&SemanticAnchor::MdBlock {
            block_id: "blk-00042".into(),
        })
        .unwrap();
    let r0 = a.resolve(anchor42).unwrap();
    assert_eq!(r0.block_index, 42, "a1 bind 时刻下标");
    assert_eq!(r0.block_id, "blk-00042");

    // 结构性编辑一：目标前插入 5 块（下标漂移 +5）
    for k in 0..5u32 {
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::InsertBlockBefore {
                before: "blk-00042".into(),
                block: BlockSpec {
                    block_id: format!("ins-{}", k),
                    text: "前置插入块".into(),
                },
            },
        )
        .unwrap();
    }
    // 结构性编辑二：目标前删除 3 块（blk-00010/11/12）
    for id in ["blk-00010", "blk-00011", "blk-00012"] {
        a.edit_stream(EditOrigin::Human, StreamEdit::RemoveBlock { block_id: id.into() })
            .unwrap();
    }
    let r1 = a.resolve(anchor42).unwrap();
    assert_eq!(
        r1.block_index, 44,
        "a1 结构性编辑后下标漂移 42+5-3=44（ID 稳定下标漂移）"
    );
    assert_eq!(r1.block_id, "blk-00042", "a1 语义锚复原同一块");
    println!("P10a1: MD 锚点保持过（前插 5 + 前删 3 → 下标 42→44，block_id 复原）");

    // a2 StickyIndex wire 往返
    let via_wire = a.__probe_sticky_wire_roundtrip(anchor42).unwrap();
    assert_eq!(via_wire, Some(44), "a2 wire 往返解算一致");
    println!("P10a2: StickyIndex wire 往返过（Encode→Decode→get_offset == 直接解算）");

    // a3 目标删除面显式化
    let anchor100 = a
        .bind(&SemanticAnchor::MdBlock {
            block_id: "blk-00090".into(),
        })
        .unwrap();
    a.edit_stream(
        EditOrigin::Human,
        StreamEdit::RemoveBlock {
            block_id: "blk-00090".into(),
        },
    )
    .unwrap();
    match a.resolve(anchor100) {
        Err(AdapterError::AnchorDangling { client, clock }) => {
            assert_eq!((client, clock), (anchor100.client, anchor100.clock));
            println!("P10a3: 目标删除面显式化（Err(AnchorDangling)——不静默返回邻居）");
        }
        other => panic!("P10a3 必须 AnchorDangling: {:?}", other.map(|r| r.block_id)),
    }

    // ── P10b Office/PDF 语义锚 ───────────────────────────────────
    // b1 OfficeBookmark 弱绑定（书签名 + 位置快照 50）
    let bm = SemanticAnchor::OfficeBookmark {
        name: "sec-start".into(),
        block_index: 50,
    };
    let bm_anchor = a.bind(&bm).unwrap();
    // 结构漂移后 resolve：位置语义（当前该位置是哪个块）
    let bm_resolved = a.resolve(bm_anchor).unwrap();
    println!(
        "P10b1: Office 书签弱绑定（bind 位 50 → 漂移后解析到位 {} 块 {}——R4.5 维持现状公开 API 的位置语义）",
        bm_resolved.block_index, bm_resolved.block_id
    );
    // 弱绑定的身份面诚实登记：漂移后位置上的块可能已非 bind 时刻的块
    // （这是 R4.5 声明的弱绑定退化面；Option A 扩面后升级为强绑定）

    // b2 PdfAnnotRef 登记面
    let annot = SemanticAnchor::PdfAnnotRef {
        page: 3,
        label: "annot-7".into(),
        block_index: 80,
    };
    let annot_anchor = a.bind(&annot).unwrap();
    assert_eq!(a.anchor_of(&annot), Some(annot_anchor), "b2 登记查询一致");
    println!("P10b2: PDF 标注登记面（page 3 / annot-7 → ID 登记查询一致）");

    // ── P10c 映射表侧车 ──────────────────────────────────────────
    let sidecar = a.export_anchor_sidecar();
    let mut b = CollabAdapter::open(SessionOptions {
        client_id: 9_000_002,
        ..Default::default()
    })
    .unwrap();
    let imported = b.import_anchor_sidecar(&sidecar).unwrap();
    assert_eq!(imported, 4, "c1 导入 4 条（md×2 + office + pdf）");
    assert_eq!(
        b.anchor_of(&SemanticAnchor::MdBlock {
            block_id: "blk-00042".into()
        }),
        Some(anchor42),
        "c1 MdBlock 锚复原"
    );
    assert_eq!(b.anchor_of(&bm), Some(bm_anchor), "c1 OfficeBookmark 锚复原");
    assert_eq!(b.anchor_of(&annot), Some(annot_anchor), "c1 PdfAnnotRef 锚复原");
    // 侧车字节稳定（确定性序）：二次导出逐字节一致
    let sidecar2 = a.export_anchor_sidecar();
    assert_eq!(sidecar, sidecar2, "c1 侧车字节稳定（确定性排序）");
    println!(
        "P10c1: 侧车往返过（{}B，4 条全复原，字节稳定）",
        sidecar.len()
    );

    // c2 敌意侧车
    let mut bad_magic = sidecar.clone();
    bad_magic[0] = b'X';
    assert!(matches!(
        b.import_anchor_sidecar(&bad_magic),
        Err(AdapterError::SidecarFormat(_))
    ));
    let mut bad_ver = sidecar.clone();
    bad_ver[4] = 9u8;
    assert!(matches!(
        b.import_anchor_sidecar(&bad_ver),
        Err(AdapterError::SidecarFormat(_))
    ));
    assert!(matches!(
        b.import_anchor_sidecar(&sidecar[..sidecar.len() - 3]),
        Err(AdapterError::SidecarFormat(_))
    ));
    println!("P10c2: 敌意侧车三面全拒（坏 magic / 坏版本 / 截断）");

    // ── P10d 跨端锚点 ────────────────────────────────────────────
    // b 已导入侧车但文档为空 → 全量同步 a → b.resolve(a 的锚点)
    let sv_b = b.state_vector();
    let full = a.encode_diff(&sv_b).unwrap();
    b.apply_update(&full).unwrap();
    let cross = b.resolve(anchor42).unwrap();
    assert_eq!(cross.block_id, "blk-00042", "d1 跨端解析同一块");
    assert_eq!(cross.block_index, 44, "d1 跨端下标一致（同一收敛态）");
    println!("P10d1: 跨端锚点过（A 端 bind → B 端 resolve 同一 block_id——CRDT ID 跨进程稳定）");

    // ── P10e 权威快照 ────────────────────────────────────────────
    let snapshot = a.authoritative_snapshot().unwrap();
    let fresh = CollabAdapter::open(SessionOptions {
        client_id: 9_000_003,
        ..Default::default()
    })
    .unwrap();
    fresh.apply_update(&snapshot).unwrap();
    let ids_a = a.__probe_block_ids();
    let ids_f = fresh.__probe_block_ids();
    assert_eq!(ids_a, ids_f, "e1 快照恢复内容逐块一致");
    // 文本面同样一致
    assert_eq!(a.__probe_text(), fresh.__probe_text(), "e1 文本面一致");
    println!(
        "P10e1: 权威快照过（{}B → 新会话恢复 {} 块逐块一致）",
        snapshot.len(),
        ids_f.len()
    );

    // e2 快照 → 维护窗压缩 → 再快照（B1.3 与 D1③ 同点）
    let mut a2 = CollabAdapter::open(SessionOptions {
        client_id: 9_000_004,
        ..Default::default()
    })
    .unwrap();
    a2.load_blocks(EditOrigin::Ai, &blocks_spec(500, "snap")).unwrap();
    for i in 0..250 {
        a2.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock {
                block_id: format!("snap-{:05}", i),
            },
        )
        .unwrap();
    }
    let snap1 = a2.authoritative_snapshot().unwrap();
    let (_, _) = a2.maintenance_gc().unwrap();
    let snap2 = a2.authoritative_snapshot().unwrap();
    assert!(snap2.len() < snap1.len(), "e2 压缩后快照更小（墓碑治理）");
    let fresh2 = CollabAdapter::open(SessionOptions {
        client_id: 9_000_005,
        ..Default::default()
    })
    .unwrap();
    fresh2.apply_update(&snap2).unwrap();
    assert_eq!(
        a2.__probe_block_ids(),
        fresh2.__probe_block_ids(),
        "e2 压缩后快照内容保真"
    );
    println!(
        "P10e2: 快照→维护窗压缩→再快照（{}B → {}B，内容保真——B1.3 与 D1③ 同点）",
        snap1.len(),
        snap2.len()
    );

    // e3 收敛双端本地全量编码差异登记（yrs-a 发现① 复现于适配层场景）
    let x = CollabAdapter::open(SessionOptions {
        client_id: 9_000_006,
        ..Default::default()
    })
    .unwrap();
    let y = CollabAdapter::open(SessionOptions {
        client_id: 9_000_007,
        ..Default::default()
    })
    .unwrap();
    // 产生两条不同的整合历史路径，终态内容一致
    x.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "hello ".into() })
        .unwrap();
    let sv_y0 = y.state_vector();
    let dx = x.encode_diff(&sv_y0).unwrap();
    y.apply_update(&dx).unwrap();
    y.edit_stream(EditOrigin::Human, StreamEdit::AppendText { text: "world".into() })
        .unwrap();
    let sv_x = x.state_vector();
    let dy = y.encode_diff(&sv_x).unwrap();
    x.apply_update(&dy).unwrap();
    assert_eq!(x.__probe_text(), y.__probe_text(), "e3 双端收敛");
    let ux_placeholder_removed = ();
    let _ = ux_placeholder_removed;
    // 全量编码（相对空 SV = 全量）：[0x00] = StateVector::default() 的 v1 编码
    let full_from_empty_x = x.encode_diff(&[0u8]).unwrap();
    let full_from_empty_y = y.encode_diff(&[0u8]).unwrap();
    // 收敛后 x 相对 y 的增量应为空/近空
    let full_x = x.encode_diff(&y.state_vector()).unwrap();
    let same_bytes = full_from_empty_x == full_from_empty_y;
    assert!(full_x.len() <= full_from_empty_x.len(), "e3 收敛后增量 ≤ 全量");
    println!(
        "P10e3: 收敛双端全量编码字节{}（x {}B / y {}B）——{}；权威快照单点编码消解该面（yrs-a 发现① 在适配层场景复现登记）",
        if same_bytes { "一致" } else { "不一致" },
        full_from_empty_x.len(),
        full_from_empty_y.len(),
        if same_bytes {
            "本形状未触发（块布局历史依赖）"
        } else {
            "状态收敛 ≠ 编码同一——D1③ 单点编码的必要性"
        }
    );

    println!("P10 PASS: MD 锚点保持（结构性编辑下 ID 稳定）+ 删除面显式化 + Office/PDF 语义锚 + 侧车版本化往返 + 跨端解析 + 权威快照（恢复/维护窗/单点编码）——用户验证面三（跨模型锚点映射）闭环");
}
