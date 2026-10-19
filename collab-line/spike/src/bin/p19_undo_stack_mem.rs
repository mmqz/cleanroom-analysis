//! p19_undo_stack_mem —— 撤销栈无界增长内存量化探针（a5-r3）。
//!
//! 依据：已知风险 #2「撤销栈无界增长（p12 新输入）」+ B3.4 定谳
//! （yrs 0.28 UndoManager 无公开栈上限/长度 API——DECISION_GROUPS
//! G3 栈上限裁决的证据前置）。本探针以**计数分配器**量化：
//!   同一 N 操作序列（XmlFragment 载体 + 双 origin 交替 + 每操作
//!   独立事务 = 每事务一栈项·最坏粒度），带撤销追踪 vs 不带的
//!   存活字节差 = 撤销栈侧成本。
//!   p19-A 基线 vs 追踪：per-op 栈成本 + 6M 操作窗口投影
//!   p19-B 重载撤销功能面：3 万操作后 undo_blocking 20 次 +
//!        redo_blocking 5 次，结构计数精确复原
//!   p19-C 栈保留性格：undo 后存活字节 ≥ 追踪态 90%（栈项转
//!        redo + tombstone 增项——内存不随 undo 释放，无界增长
//!        性格实证）
//!
//! 口径：release 跑；capture_timeout=0（p17 探针口径——最坏粒度；
//! 适配器 1000ms 窗口在交替 origin 下同不合并，量级等价）；
//! skip_gc=true（B1.1 对齐）。计数含全进程分配（事务/Vec 等噪声
//! 经 N 摊薄），报差值口径。

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use yrs::types::xml::{XmlOut, XmlTextPrelim};
use yrs::{
    ClientID, Doc, GetString, Options as DocOptions, Origin, Transact, UndoManager,
    XmlElementPrelim, XmlFragment,
};

// ── 计数分配器 ──────────────────────────────────────────────
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn live() -> u64 {
    ALLOC_BYTES
        .load(Ordering::Relaxed)
        .saturating_sub(DEALLOC_BYTES.load(Ordering::Relaxed))
}

fn new_doc(id: u64) -> Doc {
    let mut opts = DocOptions::with_client_id(ClientID::new(id));
    opts.skip_gc = true; // B1.1 口径对齐
    Doc::with_options(opts)
}

fn el_text(tag: &str, text: &str, bid: &str) -> XmlElementPrelim {
    let mut e = XmlElementPrelim::empty(tag);
    e.attributes.insert("block_id".into(), bid.to_string());
    e.children.push(XmlTextPrelim::new(text).into());
    e
}

fn new_mgr(doc: &Doc, origin: &str, xml: &yrs::XmlFragmentRef) -> UndoManager {
    let mut m = UndoManager::with_options(yrs::undo::Options {
        capture_timeout_millis: 0,
        ..Default::default()
    });
    m.expand_scope(doc, xml);
    m.include_origin(Origin::from(origin));
    m
}

fn child_count(doc: &Doc, xml: &yrs::XmlFragmentRef) -> usize {
    let txn = doc.transact();
    xml.children(&txn)
        .filter(|c| matches!(c, XmlOut::Element(_)))
        .count()
}

fn main() {
    println!("== p19_undo_stack_mem：撤销栈无界增长内存量化 ==\n");
    let mut failures = 0usize;
    let mut pass = |name: &str, ok: bool, detail: String| {
        if ok {
            println!("[PASS] {name}  {detail}");
        } else {
            failures += 1;
            println!("[FAIL] {name}  {detail}");
        }
    };

    const N: usize = 30_000; // 每文档操作数（H/A 交替各半）
    const OP_TEXT: &str = "段落文本内容示例第若干段落正文内容"; // 30 B UTF-8

    // ── p19-A 基线（无撤销追踪）vs 追踪（双 manager）─────────────
    let base_delta = {
        let doc = new_doc(7_001_901);
        let xml = doc.get_or_insert_xml_fragment("doc");
        let start = live();
        for i in 0..N {
            let origin = if i % 2 == 0 { "human" } else { "ai" };
            let mut txn = doc.transact_mut_with(Origin::from(origin));
            let idx = xml.len(&txn);
            xml.insert(&mut txn, idx, el_text("p", OP_TEXT, &format!("b-{i}")));
            txn.commit();
        }
        live() - start
    };

    let (tracked_delta, doc, xml, mut mgr_h) = {
        let doc = new_doc(7_001_902);
        let xml = doc.get_or_insert_xml_fragment("doc");
        let mgr_h = new_mgr(&doc, "human", &xml);
        let mgr_a = new_mgr(&doc, "ai", &xml);
        let _ = mgr_a; // ai 栈对称存在（交替 origin 双栈各 N/2 项）
        let start = live();
        for i in 0..N {
            let origin = if i % 2 == 0 { "human" } else { "ai" };
            let mut txn = doc.transact_mut_with(Origin::from(origin));
            let idx = xml.len(&txn);
            xml.insert(&mut txn, idx, el_text("p", OP_TEXT, &format!("b-{i}")));
            txn.commit();
        }
        let d = live() - start;
        (d, doc, xml, mgr_h)
    };

    let overhead = tracked_delta.saturating_sub(base_delta);
    let per_op = overhead as f64 / N as f64;
    let proj_6m = per_op * 6_000_000.0 / (1024.0 * 1024.0);
    println!(
        "  A-数据：基线存活增量 {base_delta} B / 追踪存活增量 {tracked_delta} B\n\
         \x20\x20\x20\x20\x20\x20\x20撤销追踪开销 {overhead} B → per-op {per_op:.1} B\
         （N={N}，最坏粒度每事务一栈项）\n\
         \x20\x20\x20\x20\x20\x20\x20  6M 操作窗口投影 ≈ {proj_6m:.0} MiB（单端单文档）"
    );

    pass(
        "p19-A1 追踪态严格大于基线（撤销追踪有实成本）",
        tracked_delta > base_delta,
        format!("Δ={overhead} B"),
    );
    pass(
        "p19-A2 per-op 成本在合理带内（8..8192 B）",
        (8.0..8192.0).contains(&per_op),
        format!("{per_op:.1} B/op"),
    );

    // ── p19-B 重载撤销功能面 ────────────────────────────────────
    let before_undo = child_count(&doc, &xml);
    let mut undone = 0usize;
    for _ in 0..20 {
        if mgr_h.undo_blocking() {
            undone += 1;
        }
    }
    let after_undo_count = child_count(&doc, &xml);
    pass(
        "p19-B1 3 万操作后撤销 20 次（human 栈重载下功能完好）",
        undone == 20 && before_undo == N && after_undo_count == N - 20,
        format!("{before_undo} → {after_undo_count}（undo {undone}）"),
    );

    let mut redone = 0usize;
    for _ in 0..5 {
        if mgr_h.redo_blocking() {
            redone += 1;
        }
    }
    let after_redo_count = child_count(&doc, &xml);
    pass(
        "p19-B2 redo 5 次复原",
        redone == 5 && after_redo_count == N - 15,
        format!("redo {redone} → {after_redo_count}"),
    );

    // ── p19-C 栈保留性格（无界增长实证）──────────────────────────
    let live_pre_undo = live();
    let mut mgr_h2 = mgr_h;
    for _ in 0..200 {
        if !mgr_h2.undo_blocking() {
            break;
        }
    }
    let live_post_undo = live();
    // undo 200 项后：栈项转 redo + 删除产生 tombstone——存活字节
    // 不降反稳/升（无界增长性格：撤销不释放内存）
    let ratio = live_post_undo as f64 / live_pre_undo as f64;
    pass(
        "p19-C1 undo 200 项后存活字节 ≥ 90%（栈保留 + tombstone，不释放）",
        ratio >= 0.90,
        format!("pre={live_pre_undo} post={live_post_undo} ratio={ratio:.3}"),
    );

    // 末态内容抽查：首块文本零畸变
    let first_text = {
        let txn = doc.transact();
        match xml.children(&txn).next() {
            Some(XmlOut::Element(e)) => {
                let mut s = String::new();
                for c in e.children(&txn) {
                    if let XmlOut::Text(t) = c {
                        s.push_str(&t.get_string(&txn));
                    }
                }
                s
            }
            _ => String::new(),
        }
    };
    pass(
        "p19-C2 首块文本零畸变（重撤销后内容完好）",
        first_text == OP_TEXT,
        format!("{first_text:?}"),
    );

    println!(
        "\n== p19 结果：{} ==",
        if failures == 0 {
            "ALL PASS".to_string()
        } else {
            format!("{failures} FAILURES")
        }
    );
    assert_eq!(failures, 0, "p19 撤销栈量化存在失败断言");
}
