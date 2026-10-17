//! p12_sync_undo_cycles —— 重复 同步/撤销 循环下的峰值 RSS 与 wire 尺寸
//! 测量探针（d1d-t1 加固轮）。
//!
//! 用户口径：「Measure peak RSS and wire size under identical datasets
//! and operations, including repeated sync/undo cycles.」
//!
//! 设计：双端编队（A/B 两适配器，同进程——进程级 RSS 覆盖双端，登记
//! 口径）；固定数据集（SCALE 块）；每循环（CYCLES 轮）执行固定操作
//! 序列：A 端 5 笔编辑（Human origin：2 文本追加 + 2 块插入 + 1 块
//! 删除）→ 差量同步 A→B → A 端 undo（回退本轮编辑）→ 差量同步 A→B
//! （undo 效果传播）→ A 端 redo → 差量同步 A→B。每循环记录：
//!   - 三次差量 wire 字节数（编辑后 / undo 后 / redo 后）
//!   - VmHWM（进程峰值 RSS）/ VmRSS（当前 RSS）
//!   - 双端内容一致性断言（每循环必须收敛）
//!
//! 收尾：双端 maintenance_gc 前后的全量 update 尺寸 + RSS——墓碑治理
//! 面与撤销栈清零效果的资源侧证据。
//!
//! 口径：release 跑（`cargo run --release --bin p12_sync_undo_cycles`）；
//! SCALE / CYCLES 环境变量可调（默认 2000 / 20）。全部数字本机实测。
//!
//! 无界增长登记面（报告落档）：撤销栈（B3.4——无逐出 API，本轮实测
//! 其 RSS 贡献）；skip_gc 墓碑（维护窗前线性累积——本轮量化）；
//! anchor_table（本轮不绑定锚点，零贡献面）。

use std::time::Instant;

use collab_adapter::{BlockSpec, CollabAdapter, EditOrigin, SemanticAnchor, SessionOptions, StreamEdit};

fn vm(field: &str) -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{}:", field)) {
            return rest.trim().trim_end_matches(" kB").parse().unwrap();
        }
    }
    0
}

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
            text: format!("text of block {prefix} {i}"),
        })
        .collect()
}

/// A→B 单向差量同步，返回 wire 字节数。
fn push(a: &CollabAdapter, b: &CollabAdapter) -> usize {
    let sv = b.state_vector();
    let diff = a.encode_diff(&sv).expect("diff");
    b.apply_update(&diff).expect("apply");
    diff.len()
}

fn main() {
    let scale: usize = std::env::var("SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    let cycles: usize = std::env::var("CYCLES").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    println!("== p12_sync_undo_cycles：重复同步/撤销循环 RSS + wire 测量 ==");
    println!("参数：SCALE={scale} CYCLES={cycles}（release，进程级 RSS 覆盖双端编队）\n");

    let mut a = sess(7_000_301);
    let mut b = sess(7_000_302);

    let rss0 = vm("VmRSS");
    let t0 = Instant::now();
    a.load_blocks(EditOrigin::Ai, &blocks(scale, "cyc")).expect("load");
    let load_ms = t0.elapsed().as_millis();
    let full0 = push(&a, &b); // 全量同步（B 从零追平）
    let hwm_after_load = vm("VmHWM");
    let rss_after_load = vm("VmRSS");
    println!(
        "装载：{scale} 块 {load_ms}ms | 全量同步 wire {full0}B | RSS Δ{:+} kB（HWM {hwm_after_load} kB）",
        rss_after_load as i64 - rss0 as i64
    );
    // 锚点绑定 1 条（anchor_table 常驻面）——绑定首块
    let anchor = a
        .bind(&SemanticAnchor::MdBlock { block_id: "cyc-00000".into() })
        .expect("bind");
    let _ = a.resolve(anchor).expect("resolve");

    println!("\n循环：编辑(5 笔)→同步→undo→同步→redo→同步（每循环固定操作序列）");
    println!(
        "{:>4} {:>10} {:>10} {:>10} {:>12} {:>10} {:>8}",
        "轮", "wire编辑", "wire撤销", "wire重做", "全量尺寸", "VmRSS kB", "收敛"
    );

    let mut wire_edit_total = 0usize;
    let mut wire_undo_total = 0usize;
    let mut wire_redo_total = 0usize;
    let mut last_full = full0;

    for c in 0..cycles {
        // 本轮固定操作序列（identical operations）
        for k in 0..2 {
            a.edit_stream(
                EditOrigin::Human,
                StreamEdit::AppendText { text: format!("cycle-{c}-{k} ") },
            )
            .expect("append");
        }
        for k in 0..2 {
            a.edit_stream(
                EditOrigin::Human,
                StreamEdit::InsertBlock {
                    block: BlockSpec {
                        block_id: format!("cyc-add-{c:03}-{k}"),
                        text: format!("added in cycle {c} #{k}"),
                    },
                },
            )
            .expect("insert");
        }
        // 删除本轮插入的首块（同轮 insert→remove——每轮操作序列恒定，
        // 净效果 +1 块/轮；基线块与锚点块不参与删除面）
        let victim = format!("cyc-add-{c:03}-0");
        a.edit_stream(
            EditOrigin::Human,
            StreamEdit::RemoveBlock { block_id: victim },
        )
        .expect("remove");

        let w_edit = push(&a, &b);
        assert_eq!(
            a.__probe_block_ids(),
            b.__probe_block_ids(),
            "cycle {c}: 编辑后收敛"
        );

        // undo：回退本轮全部 5 笔（窗内合并 = 1 栈项）
        assert!(a.undo(EditOrigin::Human), "cycle {c}: undo");
        let w_undo = push(&a, &b);
        assert_eq!(
            a.__probe_block_ids(),
            b.__probe_block_ids(),
            "cycle {c}: undo 后收敛"
        );

        // redo：重做本轮编辑
        assert!(a.redo(EditOrigin::Human), "cycle {c}: redo");
        let w_redo = push(&a, &b);
        assert_eq!(
            a.__probe_block_ids(),
            b.__probe_block_ids(),
            "cycle {c}: redo 后收敛"
        );

        // 全量尺寸（观测墓碑累积面）：对全新会话 SV 编码 = 全量 diff
        let fresh = sess(7_000_399);
        last_full = a.encode_diff(&fresh.state_vector()).expect("full diff").len();

        wire_edit_total += w_edit;
        wire_undo_total += w_undo;
        wire_redo_total += w_redo;

        println!(
            "{:>4} {:>10} {:>10} {:>10} {:>12} {:>10} {:>8}",
            c,
            format!("{w_edit}B"),
            format!("{w_undo}B"),
            format!("{w_redo}B"),
            format!("{last_full}B"),
            vm("VmRSS"),
            "OK"
        );
    }

    let hwm_peak = vm("VmHWM");
    let rss_end = vm("VmRSS");
    let undo_depth = a.undo_depth(EditOrigin::Human);
    println!("\n循环段汇总：{cycles} 轮累计 wire 编辑 {wire_edit_total}B / 撤销 {wire_undo_total}B / 重做 {wire_redo_total}B");
    println!(
        "资源面：进程峰值 RSS（VmHWM）{hwm_peak} kB | 循环末当前 RSS {rss_end} kB | 循环末 Human 撤销栈深度 {undo_depth}"
    );

    // 维护窗：双端 maintenance_gc（墓碑治理 + 栈清零）
    let full_before = last_full;
    let (before_a, after_a) = a.maintenance_gc().expect("gc a");
    let (before_b, after_b) = b.maintenance_gc().expect("gc b");
    let rss_after_gc = vm("VmRSS");
    println!(
        "\n维护窗：A 全量 {before_a}B → {after_a}B（治理 {:.1}%）| B 全量 {before_b}B → {after_b}B（治理 {:.1}%）",
        100.0 * (before_a as f64 - after_a as f64) / before_a as f64,
        100.0 * (before_b as f64 - after_b as f64) / before_b as f64,
    );
    println!(
        "维护窗后：RSS {rss_end} → {rss_after_gc} kB（高水位不回吐属分配器行为，登记不主张）| 双端撤销栈清零 | 内容保真断言：{}",
        a.__probe_block_ids() == b.__probe_block_ids()
    );
    // 撤销栈清零后 undo 无效
    assert!(!a.undo(EditOrigin::Human), "gc 后撤销栈已清");
    // 锚点仍可解析（首块未被删）
    let r = a.resolve(anchor).expect("anchor survives maintenance");
    assert_eq!(r.block_id, "cyc-00000");

    println!(
        "\n== P12 PASS：{cycles} 轮同步/撤销循环全收敛 | 峰值 RSS {hwm_peak} kB @ {scale} 块双端 | 全量 wire {full0}B → 循环末 {last_full}B → 维护窗 {after_a}B（墓碑治理回基线族） =="
    );
    let _ = full_before;
}
