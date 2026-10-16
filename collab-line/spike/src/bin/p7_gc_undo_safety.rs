//! 探针 7 GC × Undo × 快照交互面（D1-a 集成边界证据轮，#676 面专项）：
//!
//! P7a  #676 组合面复现（默认 GC）：上游 issue #676（2026-09-29 报告，评估基线
//!      ac33960 时仍 open）在 ASan 下确定性 UAF（BlockSlice::encode 读已释放
//!      内存，释放者 = 事务 commit 期 GCCollector）。本探针逐字移植其最小
//!      复现形状，两种口径：
//!      - MODE=asan（单发，默认 GC）：预期在 nightly + `-Zsanitizer=address`
//!        构建下确定性触发 ASan 报告（进程非零退出/abort）。
//!      - MODE=asan_mitigated（单发，双端 skip_gc: true）：预期同构建下干净
//!        退出（缓解面实证）。
//!      - release 非消毒口径见 MODE=full（上游口径：无声读释放内存、偶发
//!        段错误——本环境无 nightly 时仅登记行为面）。
//! P7b  缓解与部署边界矩阵：b1 双端 skip_gc 收敛；b2 混合模式（release 仅验
//!      收敛——ASan 口径 MIXED=1 实测仍确定性 UAF → 缓解不可单端部署，
//!      skip_gc 为全编队不变量）；b3 快照编码门（skip_gc → Ok；默认 GC →
//!      Err(Error::Gc)，与 D1③ 权威快照对齐）；b4 显式压缩点（clear_all
//!      后 txn.gc(None) 在 skip_gc 文档上可用且不 panic）。
//! P7c  skip_gc 代价定量：N 轮「嵌套 map 插入 + 删除」循环，对照默认 GC 的
//!      终态内存（VmRSS）与全量 update 尺寸；显式 gc(None) 后两值复测
//!      （内存回收受分配器保持策略影响，诚实登记；update 尺寸应实降）。
//!
//! 口径：release（full 模式）/ nightly+ASan（asan* 模式）；SCALE 环境变量
//! 调节 a 轮数（默认 200）与 c 循环数（默认 20000）。

use std::collections::HashSet;
use std::sync::Arc;

use yrs::types::ToJson;
use yrs::undo::Options as UndoOptions;
use yrs::updates::decoder::Decode;
use yrs::{
    Any, ArrayPrelim, ClientID, Doc, In, Map, MapPrelim, Options as DocOptions, Out, ReadTxn,
    StateVector, Text, Transact, UndoManager, Update,
};

fn vm(field: &str) -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{}:", field)) {
            return rest.trim().trim_end_matches(" kB").parse().unwrap();
        }
    }
    0
}

/// #676 形状的 peer 构造（逐字对齐上游复现）：items 根 map + UndoManager
/// （捕获窗 0 = 逐事务成栈项；timestamp 恒 0 = 关闭时间合并面）。
fn peer(skip_gc: bool, client_id: u64) -> (Doc, UndoManager) {
    let mut opts = DocOptions::with_client_id(ClientID::new(client_id));
    opts.skip_gc = skip_gc;
    let doc = Doc::with_options(opts);
    let _items = doc.get_or_insert_map("items");
    let mut undo = UndoManager::with_options(UndoOptions {
        capture_timeout_millis: 0,
        tracked_origins: HashSet::new(),
        capture_transaction: None,
        timestamp: Arc::new(|| 0u64),
        init_undo_stack: vec![],
        init_redo_stack: vec![],
    });
    undo.expand_scope(&doc, &doc.get_or_insert_map("items"));
    undo.include_origin("local");
    (doc, undo)
}

fn insert(doc: &Doc, undo: &mut UndoManager, key: &str) {
    let items = doc.get_or_insert_map("items");
    let fields = MapPrelim::from([
        ("z", In::Any(Any::from("a0"))),
        ("erase", In::Array(ArrayPrelim::default())),
    ]);
    items.insert(&mut doc.transact_mut_with("local"), key, fields);
    undo.reset();
}

fn set_color(doc: &Doc, undo: &mut UndoManager, key: &str) {
    let items = doc.get_or_insert_map("items");
    let mut t = doc.transact_mut_with("local");
    if let Some(Out::YMap(m)) = items.get(&t, key) {
        m.insert(&mut t, "color", 1.0);
    }
    drop(t);
    undo.reset();
}

fn remove(doc: &Doc, undo: &mut UndoManager, key: &str) {
    let items = doc.get_or_insert_map("items");
    items.remove(&mut doc.transact_mut_with("local"), key);
    undo.reset();
}

/// `to` takes what it lacks from `from`.
fn sync(from: &Doc, to: &Doc) {
    let d = from
        .transact()
        .encode_diff_v1(&to.transact().state_vector());
    to.transact_mut_with("remote")
        .apply_update(Update::decode_v1(&d).unwrap())
        .unwrap();
}

/// #676 逐字复现序列。返回（a 终态 JSON, b 终态 JSON）。
fn repro_676(a_skip_gc: bool, b_skip_gc: bool) -> (Any, Any) {
    let (a, mut ua) = peer(a_skip_gc, 1);
    let (b, mut ub) = peer(b_skip_gc, 2);
    insert(&a, &mut ua, "1");
    insert(&a, &mut ua, "2");
    insert(&a, &mut ua, "3");
    insert(&b, &mut ub, "b");
    sync(&b, &a);
    sync(&a, &b);
    ua.undo_blocking(); // a takes back "3"
    set_color(&b, &mut ub, "2");
    remove(&b, &mut ub, "3");
    ua.undo_blocking(); // a takes back "2"
    ub.undo_blocking(); // b takes back its removal of "3"
    set_color(&b, &mut ub, "2");
    ub.undo_blocking(); // b takes back the second colour
    remove(&b, &mut ub, "1");
    // a 的两笔 undo 到达 b：b 的 GC 在 commit 期释放 map "2"/"3"，而 b 自身
    // （已 undo 的）编辑块仍指向它们；b 为 a 编码 diff 时读已释放内存。
    sync(&a, &b);
    sync(&b, &a); // ← #676 UAF 点（ASan 下确定性触发）
    let ja = a.get_or_insert_map("items").to_json(&a.transact());
    let jb = b.get_or_insert_map("items").to_json(&b.transact());
    (ja, jb)
}

fn main() {
    let mode = std::env::var("MODE").unwrap_or_else(|_| "full".to_string());
    match mode.as_str() {
        "asan" => {
            // 默认 GC 单发：ASan 构建下预期 abort（UAF 实证）；若干净退出，
            // 如实登记（说明该形状在本版本/平台未触发——与上游主张相悖时登记）。
            let (ja, jb) = repro_676(false, false);
            println!("P7a-asan: 默认 GC 单发完成（未触发 abort）；收敛 = {}", ja == jb);
        }
        "asan_mitigated" => {
            let (ja, jb) = repro_676(true, true);
            assert_eq!(ja, jb, "P7b1 双端 skip_gc 收敛");
            println!("P7b1-asan: 双端 skip_gc: true 干净完成且收敛（缓解面实证）");
        }
        "asan_mixed" => {
            // 混合模式（单端缓解部署）在消毒构建下的两面：
            // MIXED=1 → a=skip_gc, b=GC；MIXED=2 → a=GC, b=skip_gc。
            let which = std::env::var("MIXED").unwrap_or_else(|_| "1".to_string());
            match which.as_str() {
                "1" => {
                    let (ja, jb) = repro_676(true, false);
                    assert_eq!(ja, jb, "P7b2 混合（a=skip_gc, b=GC）收敛");
                    println!("P7b2-asan: 混合 a=skip_gc/b=GC 干净完成且收敛");
                }
                "2" => {
                    let (ja, jb) = repro_676(false, true);
                    assert_eq!(ja, jb, "P7b2 混合（a=GC, b=skip_gc）收敛");
                    println!("P7b2-asan: 混合 a=GC/b=skip_gc 干净完成且收敛");
                }
                other => panic!("未知 MIXED: {}", other),
            }
        }
        "full" => full(),
        other => panic!("未知 MODE: {}", other),
    }
}

fn full() {
    let rounds: u32 = std::env::var("SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let cycles: u32 = std::env::var("CYCLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);

    // ---------- P7a：#676 形状 × rounds（release 非消毒口径） ----------
    let mut diverged = 0u32;
    for i in 0..rounds {
        let (ja, jb) = repro_676(false, false);
        if ja != jb {
            diverged += 1;
        }
        if (i + 1) % 50 == 0 {
            println!("  P7a 进度 {}/{}（发散 {}）", i + 1, rounds, diverged);
        }
    }
    // 收敛面断言：CRDT 收敛性质在 UAF 读释放内存但未崩的轮次应保持。
    // （若有发散轮：上方计数登记，此处不 assert——发散即发现，不粉饰。）
    println!("P7a: {} 轮完成，终态发散轮数 = {}（非消毒口径：UAF 读释放内存多为无声）", rounds, diverged);

    // ---------- P7b：缓解与部署边界矩阵 ----------
    // b1 双端 skip_gc
    let (ja, jb) = repro_676(true, true);
    assert_eq!(ja, jb, "P7b1 双端 skip_gc 收敛");
    println!("P7b1: 双端 skip_gc: true → 收敛成立");

    // b2 混合模式（release 仅验收敛；内存安全结论以 ASan 口径为准：
    //    MIXED=1 方向 GC 端确定性 UAF → 「单端部署缓解」被证伪，
    //    skip_gc 是全编队不变量；MIXED=2 方向本形状未触发【触发条件 =
    //    接收远端删除+本地 undo 栈+GC 的组合，任何 peer 都可能进入 b 角色】）
    let (ja, jb) = repro_676(true, false);
    assert_eq!(ja, jb, "P7b2 混合模式收敛（a=skip_gc, b=GC）");
    let (ja, jb) = repro_676(false, true);
    assert_eq!(ja, jb, "P7b2 混合模式收敛（a=GC, b=skip_gc）");
    println!("P7b2: 混合两方向 release 收敛；但 ASan 口径 MIXED=1 确定性 UAF → 缓解不可单端部署，skip_gc 须全编队一致");

    // b3 快照编码门：skip_gc → Ok；默认 GC → Err(Error::Gc)
    let mut o = DocOptions::with_client_id(ClientID::new(42));
    o.skip_gc = true;
    let d_skip = Doc::with_options(o);
    let t = d_skip.get_or_insert_text("text");
    t.insert(&mut d_skip.transact_mut(), 0, "hello");
    let snap = d_skip.transact_mut().snapshot();
    let mut enc = yrs::updates::encoder::EncoderV1::new();
    let r1 = d_skip
        .transact_mut()
        .encode_state_from_snapshot(&snap, &mut enc);
    assert!(r1.is_ok(), "P7b3 skip_gc 快照编码 Ok");
    let d_gc = Doc::with_client_id(43);
    let t2 = d_gc.get_or_insert_text("text");
    t2.insert(&mut d_gc.transact_mut(), 0, "hello");
    let snap2 = d_gc.transact_mut().snapshot();
    let mut enc2 = yrs::updates::encoder::EncoderV1::new();
    let r2 = d_gc
        .transact_mut()
        .encode_state_from_snapshot(&snap2, &mut enc2);
    assert!(r2.is_err(), "P7b3 默认 GC 快照编码必须 Err");
    match r2 {
        Err(yrs::error::Error::Gc) => println!("P7b3: 快照编码门钉定（skip_gc → Ok；默认 GC → Err(Error::Gc)）"),
        other => panic!("P7b3 默认 GC 错误变体非 Gc: {:?}", other),
    }

    // b4 显式压缩点：skip_gc 文档 + UndoManager 编辑序列 + clear_all 后
    //     txn.gc(None) 可用不 panic，且文档此后仍可正常编码/同步。
    let (d, mut u) = peer(true, 7);
    insert(&d, &mut u, "x");
    insert(&d, &mut u, "y");
    remove(&d, &mut u, "x");
    u.clear_all(); // ← 压缩前置：undo 栈不再引用任何块
    {
        let mut txn = d.transact_mut();
        txn.gc(None); // 显式全量压缩（transaction.rs gc API）
    }
    let bytes = d.transact().encode_diff_v1(&StateVector::default());
    let remote = Doc::with_client_id(8);
    remote
        .transact_mut()
        .apply_update(Update::decode_v1(&bytes).unwrap())
        .unwrap();
    let j1 = d.get_or_insert_map("items").to_json(&d.transact());
    let j2 = remote.get_or_insert_map("items").to_json(&remote.transact());
    assert_eq!(j1, j2, "P7b4 压缩后编码→apply 收敛");
    println!("P7b4: clear_all 后显式 gc(None) 可用；压缩后全量同步收敛（{}B）", bytes.len());

    // ---------- P7c：skip_gc 代价定量 ----------
    let mk_ops = |doc: &Doc| {
        let items = doc.get_or_insert_map("items");
        for i in 0..cycles {
            let key = format!("k{}", i);
            let fields = MapPrelim::from([
                ("z", In::Any(Any::from("a0"))),
                ("erase", In::Array(ArrayPrelim::default())),
            ]);
            items.insert(&mut doc.transact_mut_with("local"), key.as_str(), fields);
            items.remove(&mut doc.transact_mut_with("local"), key.as_str());
        }
    };
    let base = vm("VmRSS");
    let d_gc2 = Doc::with_client_id(51);
    mk_ops(&d_gc2);
    let gc_rss = vm("VmRSS") - base;
    let gc_upd = d_gc2
        .transact()
        .encode_diff_v1(&StateVector::default())
        .len();

    let base = vm("VmRSS");
    let mut o = DocOptions::with_client_id(ClientID::new(52));
    o.skip_gc = true;
    let d_skip2 = Doc::with_options(o);
    mk_ops(&d_skip2);
    let skip_rss = vm("VmRSS") - base;
    let skip_upd = d_skip2
        .transact()
        .encode_diff_v1(&StateVector::default())
        .len();

    // 显式压缩后复测（分配器保持策略 → RSS 降幅可能小于真实释放量；update
    // 尺寸是硬指标：GC 块只编 ID 不编内容，应实降）。
    {
        let mut txn = d_skip2.transact_mut();
        txn.gc(None);
    }
    let skip_rss_after = vm("VmRSS") - base;
    let skip_upd_after = d_skip2
        .transact()
        .encode_diff_v1(&StateVector::default())
        .len();
    let j_skip = d_skip2.get_or_insert_map("items").to_json(&d_skip2.transact());
    assert!(
        matches!(&j_skip, Any::Map(m) if m.is_empty()),
        "P7c 压缩后 items 应为空 map"
    );
    assert!(skip_upd_after < skip_upd, "P7c 压缩后 update 尺寸应下降");

    println!("P7c skip_gc 代价（CYCLES={} 轮插入+删除）：", cycles);
    println!("  默认 GC : 终态 RSS +{} kB, 全量 update {} B", gc_rss, gc_upd);
    println!("  skip_gc : 终态 RSS +{} kB, 全量 update {} B", skip_rss, skip_upd);
    println!("  显式 gc(None) 后: RSS +{} kB（分配器保持策略口径）, update {} B", skip_rss_after, skip_upd_after);

    println!("P7 PASS: a 复现面登记 + b 缓解矩阵四断言 + c 代价定量");
}
