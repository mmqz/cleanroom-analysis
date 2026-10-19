//! p20_undo_window_repro —— yrs 0.28 撤销捕获窗交叉 origin 缺陷最小
//! 复现与定谳探针（a5-r4，soak-2 失败追因）。
//!
//! 触发背景（适配器层 zz 观测用例）：双 UndoManager（human/ai）+
//! capture_timeout=1000（适配器口径）+ 交错同型追加 H1/A1/H2/A2：
//!   ① 合并栈项部分撤销（"H1"+"H2" 同窗合并为一项，undo 仅移除
//!      H1——"A1H2A2" ≠ 期望 "A1A2"）
//!   ② 后续 redo 在 yrs::store::Store::materialize panic
//!      （Option::unwrap on None——经 UndoManager::pop_blocking）
//! 对照：p17 探针（timeout=0 + 双 manager + XmlFragment）全绿——
//! 嫌疑变量 = 捕获窗合并。
//!
//! 四配置矩阵（每配置独立 Doc，catch_unwind 隔离 panic 面）：
//!   cfg-1 双 manager + timeout=1000（适配器镜像——预期复现）
//!   cfg-2 双 manager + timeout=0（p17 口径——预期干净）
//!   cfg-3 单 manager（双 origin）+ timeout=1000
//!   cfg-4 单 manager（双 origin）+ timeout=0
//! 判定：配置间差异 → 缺陷触发面定谳。

use std::panic::{catch_unwind, AssertUnwindSafe};

use yrs::{ClientID, Doc, GetString, Options as DocOptions, Origin, Text, Transact, UndoManager};

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true;
    Doc::with_options(opts)
}

fn append(doc: &Doc, origin: &str, s: &str) {
    // 纪律（p18-F 同族）：先取根（store 访问），后开事务——
    // 事务内 get_or_insert_* = 死锁挂起
    let t = doc.get_or_insert_text("text");
    let mut txn = doc.transact_mut_with(Origin::from(origin));
    let end = t.len(&txn);
    t.insert(&mut txn, end, s);
    txn.commit();
}

fn text(doc: &Doc) -> String {
    doc.get_or_insert_text("text").get_string(&doc.transact())
}

/// 单 manager 配置：track 双 origin + 作用域展开（适配器 open() 镜像）
fn mgr(doc: &Doc, timeout: u64, origins: &[&str]) -> UndoManager {
    let mut m = UndoManager::with_options(yrs::undo::Options {
        capture_timeout_millis: timeout,
        ..Default::default()
    });
    for o in origins {
        m.include_origin(Origin::from(*o));
    }
    m.expand_scope(doc, &doc.get_or_insert_text("text"));
    m
}

fn main() {
    println!("== p20_undo_window_repro：捕获窗交叉 origin 缺陷定谳 ==\n");

    let configs: Vec<(&str, bool, u64, Vec<&str>)> = vec![
        (
            "cfg-1 双mgr+win1000（适配器镜像）",
            true,
            1000,
            vec!["human"],
        ),
        ("cfg-2 双mgr+win0（p17 口径）", true, 0, vec!["human"]),
        (
            "cfg-3 单mgr双origin+win1000",
            false,
            1000,
            vec!["human", "ai"],
        ),
        ("cfg-4 单mgr双origin+win0", false, 0, vec!["human", "ai"]),
    ];

    for (name, dual, timeout, tracked) in &configs {
        let dual = *dual;
        let timeout = *timeout;
        let tracked = tracked.clone();
        let outcome = catch_unwind(AssertUnwindSafe(move || {
            let doc = new_doc(7_003_001);

            let mut m_h = mgr(&doc, timeout, &tracked);
            let mut m_a = if dual {
                Some(mgr(&doc, timeout, &["ai"]))
            } else {
                None
            };

            // 交错追加 H1/A1/H2/A2（同窗内同 origin 两次 → 合并候选）
            append(&doc, "human", "H1");
            append(&doc, "ai", "A1");
            append(&doc, "human", "H2");
            append(&doc, "ai", "A2");
            let initial = text(&doc);
            assert_eq!(initial, "H1A1H2A2", "初始装载");

            // undo human 至尽（cfg-3 单 mgr：undo 即全撤双 origin）
            let mut steps = vec![];
            if dual {
                while m_h.undo_blocking() {}
                steps.push(format!("undo-H 后 {:?}", text(&doc)));
                if let Some(m) = m_a.as_mut() {
                    while m.undo_blocking() {}
                }
                steps.push(format!("undo-A 后 {:?}", text(&doc)));
            } else {
                while m_h.undo_blocking() {}
                steps.push(format!("undo 尽后 {:?}", text(&doc)));
            }

            // redo human 至尽 + redo ai 至尽
            if dual {
                let r = catch_unwind(AssertUnwindSafe(|| while m_h.redo_blocking() {}));
                if r.is_err() {
                    return (steps.join(" | ") + " | redo-H PANIC", true);
                }
                steps.push(format!("redo-H 后 {:?}", text(&doc)));
                if let Some(m) = m_a.as_mut() {
                    let r = catch_unwind(AssertUnwindSafe(|| while m.redo_blocking() {}));
                    if r.is_err() {
                        return (steps.join(" | ") + " | redo-A PANIC", true);
                    }
                }
            } else {
                let r = catch_unwind(AssertUnwindSafe(|| while m_h.redo_blocking() {}));
                if r.is_err() {
                    return (steps.join(" | ") + " | redo PANIC", true);
                }
            }
            (steps.join(" | "), false)
        }));

        match outcome {
            Ok((trace, panicked)) => {
                println!("[{name}] panic={panicked}\n    {trace}");
            }
            Err(_) => println!("[{name}] 外层 panic（装载/undo 段）"),
        }
    }

    println!("\n== p20 结果：观测矩阵完成（判定见输出） ==");
}
