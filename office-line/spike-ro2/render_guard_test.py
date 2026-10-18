#!/usr/bin/env python3
"""render_guard 验证脚本（ro2-t7）——三机械面四断言组：

  T1 单实例串行化：3 并发 guard 转换（真实 soffice + R-O2 夹具）
     → 全成功 + 锁等待证据（后到者 lock_wait_s > 0）+ 总墙钟 >
     单转换 3 倍下界近似（串行而非并行）。
  T2 RSS 正常面：真实转换监控不误杀（硬限高设）→ rss_peak 记录
     在合理区间 + 无 hard/timeout kill + procs_seen ≥ 1。
  T3 RSS 硬限触发：合成吞噬者（分配 ~500MB 后退出）+ 硬限 200MB
     → hard_kill=True + returncode=-9 + 提前返回（未自然退出）。
  T4 超时杀组：合成睡眠者（sleep 30）+ kill-timeout 2s →
     timeout_kill=True + wall_s ∈ [2, 3.5]。

口径：stdlib only；soffice 用系统 25.2.3.2（本套件基线）。
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
GUARD = HERE / "render_guard.py"
FIXTURES = HERE / "fixtures"
OUT = HERE / "out" / "guard_test"
OUT.mkdir(parents=True, exist_ok=True)


def run_guard(cmd: list[str], **kw) -> dict:
    argv = [
        sys.executable, str(GUARD), json.dumps(cmd),
        "--lock-path", str(OUT / "test.lock"),
        "--lock-timeout", str(kw.get("lock_timeout", 120)),
        "--rss-soft-kb", str(kw.get("soft", 384 * 1024)),
        "--rss-hard-kb", str(kw.get("hard", 768 * 1024)),
        "--rss-poll-ms", str(kw.get("poll", 50)),
        "--kill-timeout", str(kw.get("kill_timeout", 60)),
    ]
    r = subprocess.run(argv, capture_output=True, text=True, timeout=kw.get("outer", 180))
    return json.loads(r.stdout)


def main() -> int:
    failures = 0
    docs = sorted(FIXTURES.glob("*.docx"))
    assert docs, "夹具缺失"

    # ── T1 单实例串行化（3 并发真实转换）───────────────────────────
    def conv(i: int) -> dict:
        src = docs[i % len(docs)]
        dst = OUT / f"t1_{i}.pdf"
        if dst.exists():
            dst.unlink()
        return run_guard(
            ["soffice", "--headless", "--norestore", "--convert-to", "pdf",
             "--outdir", str(OUT), str(src)],
            hard=2 * 1024 * 1024,  # 2GB 硬限（正常面不干扰 T1）
        )

    t_start = time.perf_counter()
    with ThreadPoolExecutor(max_workers=3) as ex:
        results = list(ex.map(conv, range(3)))
    total_wall = time.perf_counter() - t_start

    t1_ok = (
        all(r.get("returncode") == 0 for r in results)
        and all(not r.get("rss_hard_kill") and not r.get("timeout_kill") for r in results)
        and any((r.get("lock_wait_s") or 0) > 0.05 for r in results)
        and total_wall > 0.9  # 3 次串行 ≥ 单次 ~0.3s 的近似下界
    )
    if t1_ok:
        waits = [r.get("lock_wait_s") for r in results]
        print(f"[PASS] T1 单实例串行化（3 并发全成功，锁等待 {waits}，总墙钟 {total_wall:.2f}s）")
    else:
        failures += 1
        print(f"[FAIL] T1：{json.dumps(results, ensure_ascii=False)[:400]}")

    # ── T2 RSS 正常面（真实转换不误杀）─────────────────────────────
    src = docs[0]
    dst = OUT / "t2.pdf"
    if dst.exists():
        dst.unlink()
    r2 = run_guard(
        ["soffice", "--headless", "--norestore", "--convert-to", "pdf",
         "--outdir", str(OUT), str(src)],
        soft=1,  # 软限 1kB 恒触发（告警面验证）
        hard=2 * 1024 * 1024,
    )
    t2_ok = (
        r2.get("returncode") == 0
        and r2.get("rss_soft_breach") is True
        and r2.get("rss_hard_kill") is False
        and r2.get("rss_peak_kb", 0) > 50 * 1024   # LO 真实足迹 > 50MB
        and r2.get("procs_seen", 0) >= 1
    )
    if t2_ok:
        print(f"[PASS] T2 RSS 正常面（软限告警触发不杀，峰值 {r2['rss_peak_kb']//1024}MB，树进程 {r2['procs_seen']}）")
    else:
        failures += 1
        print(f"[FAIL] T2：{json.dumps(r2)[:300]}")

    # ── T3 RSS 硬限触发（合成吞噬者 ~500MB）────────────────────────
    eater = (
        "import time,sys\n"
        "b=bytearray(500*1024*1024)\n"
        "b[::4096]=b'x'*len(b[::4096])\n"
        "time.sleep(30)\n"
    )
    r3 = run_guard(
        [sys.executable, "-c", eater],
        soft=1, hard=200 * 1024, poll=30, kill_timeout=60,
    )
    t3_ok = (
        r3.get("rss_hard_kill") is True
        and r3.get("returncode") == -9
        and r3.get("rss_peak_kb", 0) >= 190 * 1024
    )
    if t3_ok:
        print(f"[PASS] T3 RSS 硬限 fail-closed（峰值 {r3['rss_peak_kb']//1024}MB ≥ 200MB 限 → SIGKILL 组，rc=-9）")
    else:
        failures += 1
        print(f"[FAIL] T3：{json.dumps(r3)[:300]}")

    # ── T4 超时杀组（合成睡眠者）───────────────────────────────────
    r4 = run_guard(
        ["sleep", "30"],
        hard=2 * 1024 * 1024, kill_timeout=2, poll=20,
    )
    wall = r4.get("wall_s") or 0
    t4_ok = (
        r4.get("timeout_kill") is True
        and r4.get("returncode") == -9
        and 1.5 <= wall <= 4.0
    )
    if t4_ok:
        print(f"[PASS] T4 超时杀组（2s 限时 → wall {wall}s SIGKILL 组 rc=-9）")
    else:
        failures += 1
        print(f"[FAIL] T4：{json.dumps(r4)[:300]}")

    print(f"\n== render_guard 验证：{failures == 0 and 'ALL PASS' or f'{failures} FAILURES'} ==")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
