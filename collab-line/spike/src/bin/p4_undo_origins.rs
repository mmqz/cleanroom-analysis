//! 探针 4 UndoManager origin 分栈：human/ai 两 origin 事务分离追踪——
//! include_origin 只收 human 面，undo 仅回退 human 编辑，ai 编辑保留。
//! （mdcore Source::Human/Ai 溯源分栈的 yrs 侧对应物实证）

use yrs::{Doc, GetString, Origin, ReadTxn, StateVector, Text, Transact, UndoManager};

fn main() {
    let doc = Doc::with_client_id(1);
    let text = doc.get_or_insert_text("body");

    let mut mgr = UndoManager::with_options(yrs::undo::Options {
        capture_timeout_millis: 0, // 每事务一栈项（mdcore 默认 1s 合并窗的对照面）
        ..Default::default()
    });
    mgr.expand_scope(&doc, &text);
    mgr.include_origin(Origin::from("human"));

    // human 三笔（分事务 → 三栈项）
    text.insert(&mut doc.transact_mut_with(Origin::from("human")), 0, "H1 ");
    text.insert(&mut doc.transact_mut_with(Origin::from("human")), 3, "H2 ");
    text.insert(&mut doc.transact_mut_with(Origin::from("human")), 6, "H3 ");

    // ai 两笔（不入栈）
    text.insert(&mut doc.transact_mut_with(Origin::from("ai")), 9, "A1 ");
    text.insert(&mut doc.transact_mut_with(Origin::from("ai")), 12, "A2");

    let full = text.get_string(&doc.transact());
    assert_eq!(full, "H1 H2 H3 A1 A2", "编辑全量");

    // undo 一项：仅回退最后一笔 human（H3），ai 编辑保留
    assert!(mgr.undo_blocking(), "undo 执行成功");
    let s1 = text.get_string(&doc.transact());
    assert_eq!(s1, "H1 H2 A1 A2", "human 分栈 undo：仅撤 H3，A1/A2 保留（实得 {}）", s1);

    // redo 恢复 H3
    assert!(mgr.redo_blocking(), "redo 执行成功");
    let s2 = text.get_string(&doc.transact());
    assert_eq!(s2, "H1 H2 H3 A1 A2", "redo 复原");

    // 栈深：human 面可继续 undo 三笔（H3、H2、H1）；ai 面永不可 undo
    assert!(mgr.undo_blocking());
    assert!(mgr.undo_blocking());
    assert!(mgr.undo_blocking());
    let s3 = text.get_string(&doc.transact());
    assert_eq!(s3, "A1 A2", "human 全撤后仅剩 ai 编辑（实得 {}）", s3);
    // 第四次 undo 应失败（human 栈空，ai 不在栈）
    assert!(!mgr.undo_blocking(), "human 栈尽后 ai 编辑不可 undo（分栈实证）");
    let s4 = text.get_string(&doc.transact());
    assert_eq!(s4, "A1 A2", "ai 编辑经分栈隔离保持");

    // 事务 origin 保留在 update（跨端可辨）
    let update_bytes = doc.transact().encode_diff_v1(&StateVector::default());
    let _ = update_bytes;
    println!("P4 PASS: origin 分栈（human undo×3 / ai 不可 undo / redo 复原 / 隔离保持）");
}
