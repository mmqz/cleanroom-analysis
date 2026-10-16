//! 探针 5 规模 sanity 与提交内存模式对比（非基准战役——量级数字供评估）：
//!
//! 三装载模式对比（同一逻辑内容量级）：
//!   A1 单事务 N 笔小串追加（编辑流重放型）——实测提交期内存超线性放大
//!   A2 单事务大块追加（每块 8000B，AI 生成态典型）——对照面
//!   A3 N 笔小串分事务追加（交互编辑流）——对照面
//! 其后 B 随机位插入 / C 块数组 / D 全量编码+远端 apply / E 增量交换收敛。
//! 口径 = release + Instant 墙钟 + VmHWM。SCALE 可经环境变量调节（默认 10000）。

use std::time::Instant;

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Array, Doc, GetString, ReadTxn, StateVector, Text, Transact, Update};

fn vm_hwm() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest.trim().trim_end_matches(" kB").parse().unwrap();
        }
    }
    0
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

fn main() {
    let scale: u32 = std::env::var("SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    let doc = Doc::with_client_id(1);
    let text = doc.get_or_insert_text("sheet");
    let array = doc.get_or_insert_array("blocks");

    // A1: 单事务 scale 笔 8B 追加
    let base = vm_hwm();
    let t = Instant::now();
    {
        let mut txn = doc.transact_mut();
        for i in 0..scale {
            text.push(&mut txn, &format!("cell{:04}", i % 10));
        }
    }
    let a1_time = t.elapsed();
    let a1_mem = vm_hwm() - base;
    assert_eq!(text.len(&doc.transact()), scale * 8, "A1 总长");

    // A2: 单事务 scale/1000 笔 8000B 大块追加（等量文本）
    let doc2 = Doc::with_client_id(11);
    let text2 = doc2.get_or_insert_text("sheet");
    let base2 = vm_hwm();
    let t = Instant::now();
    {
        let mut txn = doc2.transact_mut();
        let big = "c".repeat(8000);
        for _ in 0..(scale / 1000).max(1) {
            text2.push(&mut txn, &big);
        }
    }
    let a2_time = t.elapsed();
    let a2_mem = vm_hwm() - base2;

    // A3: scale 笔 8B 分事务追加
    let doc3 = Doc::with_client_id(12);
    let text3 = doc3.get_or_insert_text("sheet");
    let base3 = vm_hwm();
    let t = Instant::now();
    for i in 0..scale {
        text3.insert(&mut doc3.transact_mut(), i * 8, &format!("cell{:04}", i % 10));
    }
    let a3_time = t.elapsed();
    let a3_mem = vm_hwm() - base3;
    assert_eq!(text3.len(&doc3.transact()), scale * 8, "A3 总长");

    // B: 2000 笔随机位中途插入（独立事务）
    let t = Instant::now();
    let mut rng = Lcg(0xDEADBEEF);
    let mut cur_len = scale * 8;
    for _ in 0..2_000 {
        let pos = (rng.next() % cur_len as u64) as u32;
        text.insert(&mut doc.transact_mut(), pos, "X");
        cur_len += 1;
    }
    let b_time = t.elapsed();

    // C: 1 万块 array 追加
    let t = Instant::now();
    {
        let mut txn = doc.transact_mut();
        for i in 0..10_000u32 {
            array.insert(&mut txn, i, format!("blk-{}", i));
        }
    }
    let c_time = t.elapsed();
    let c_mem = vm_hwm();

    // D: 全量编码 + 远端 apply
    let t = Instant::now();
    let bytes = doc.transact().encode_diff_v1(&StateVector::default());
    let d_enc = t.elapsed();
    let remote = Doc::with_client_id(2);
    let t = Instant::now();
    remote
        .transact_mut()
        .apply_update(Update::decode_v1(&bytes).unwrap())
        .unwrap();
    let d_app = t.elapsed();
    assert_eq!(
        remote.get_or_insert_text("sheet").len(&remote.transact()),
        scale * 8 + 2_000,
        "远端等长"
    );

    // E: 增量交换（SV 差量互传）+ 收敛
    let t = Instant::now();
    text.insert(&mut doc.transact_mut(), 0, "LOCAL-EDIT ");
    let rtext = remote.get_or_insert_text("sheet");
    rtext.insert(&mut remote.transact_mut(), 0, "REMOTE-EDIT ");
    let sv_doc = doc.transact().state_vector().encode_v1();
    let sv_remote = remote.transact().state_vector().encode_v1();
    let ua = doc
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&sv_remote).unwrap());
    let ub = remote
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&sv_doc).unwrap());
    doc.transact_mut().apply_update(Update::decode_v1(&ub).unwrap()).unwrap();
    remote.transact_mut().apply_update(Update::decode_v1(&ua).unwrap()).unwrap();
    let e_time = t.elapsed();
    let s1 = doc.get_or_insert_text("sheet").get_string(&doc.transact());
    let s2 = remote.get_or_insert_text("sheet").get_string(&remote.transact());
    assert_eq!(s1, s2, "交换后两端收敛");

    println!("P5 装载模式对比（SCALE={}，release 墙钟，VmHWM 增量）：", scale);
    println!("  A1 单事务 {} 笔 8B 小串: {:>8.1} ms, 提交期内存 +{} kB", scale, a1_time.as_secs_f64() * 1000.0, a1_mem);
    println!("  A2 单事务 {} 笔 8KB 大块: {:>8.1} ms, 提交期内存 +{} kB", (scale / 1000).max(1), a2_time.as_secs_f64() * 1000.0, a2_mem);
    println!("  A3 分事务 {} 笔 8B 小串: {:>8.1} ms, 提交期内存 +{} kB", scale, a3_time.as_secs_f64() * 1000.0, a3_mem);
    println!("  B  2000 笔随机位插入:    {:>8.1} ms", b_time.as_secs_f64() * 1000.0);
    println!("  C  1 万块 array 追加:    {:>8.1} ms, 全程峰值 {} kB", c_time.as_secs_f64() * 1000.0, c_mem);
    println!("  D  全量编码 {}B: {:>8.1} ms; 远端 apply: {:>8.1} ms", bytes.len(), d_enc.as_secs_f64() * 1000.0, d_app.as_secs_f64() * 1000.0);
    println!("  E  增量交换(差量 {}B)+合并: {:>8.1} ms", ua.len(), e_time.as_secs_f64() * 1000.0);
    println!("P5 PASS: 模式对比 + 全链断言过（含收敛）");
}
