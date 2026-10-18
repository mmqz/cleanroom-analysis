#!/usr/bin/env python3
"""render_guard —— 渲染管线守护器就绪原型（ro2-t7，批次 3 机械项预置）。

批次 3 G5 #7（Q3 子进程 + 宿主预装）裁决后的实施轮机械项，本文件
= spike 就绪原型（决策无关预置，零产品代码接触）：

  G1 单实例并发锁：flock 排他锁文件——并发转换请求**串行化**
     （阻塞等锁，默认 300s 超时 fail-closed），内存上界 = 单转换
     进程 footprint（LO 单实例并发 = RSS 上界钳制的实施形态）。
  G2 RSS 看门狗：转换期间轮询 soffice 进程树（/proc 后代扫描）
     VmRSS——软限告警记录（不杀）/ 硬限 SIGKILL 整进程组（fail-
     closed，防 OOM 拖垮宿主）。
  G3 超时杀组：墙钟超限 → SIGKILL 进程组（r-o2-t3 既有语义的
     守护器化收编）。

口径：python stdlib only（fcntl/subprocess/proc 扫描）；每转换
全新 guard 进程 = 隔离不变量维持；输出 JSON 单行（父进程解析）。

用法：
  python3 render_guard.py '<JSON 命令数组>' \
      [--lock-path PATH] [--lock-timeout S] [--rss-soft-kb N]
      [--rss-hard-kb N] [--rss-poll-ms M] [--kill-timeout S]
"""

from __future__ import annotations

import argparse
import base64
import fcntl
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path


def descendants_rss_kb(root_pid: int) -> tuple[int, int]:
    """进程树 RSS 扫描：返回 (树总 RSS kB, 树活进程数)。

    /proc/<pid>/status VmRSS 直读 + /proc/<pid>/task/*/children 沿
    展开后代（stdlib，无 ps 依赖——regress.py find_soffice_pids
    同族口径，扩展为树遍历）。
    """
    children_map: dict[int, list[int]] = {}
    try:
        for p in Path("/proc").iterdir():
            if not p.name.isdigit():
                continue
            try:
                for t in (p / "task").iterdir():
                    with open(t / "children") as f:
                        kids = [int(x) for x in f.read().split()]
                        children_map.setdefault(int(p.name), []).extend(kids)
            except OSError:
                continue
    except OSError:
        pass

    total = 0
    live = 0
    stack = [root_pid]
    seen = set()
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        try:
            with open(f"/proc/{pid}/status") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        total += int(line.split()[1])
                        live += 1
                        break
        except OSError:
            continue
        stack.extend(children_map.get(pid, []))
    return total, live


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd_json")
    ap.add_argument("--lock-path", default="/tmp/render_guard.lock")
    ap.add_argument("--lock-timeout", type=float, default=300.0)
    ap.add_argument("--rss-soft-kb", type=int, default=384 * 1024)
    ap.add_argument("--rss-hard-kb", type=int, default=768 * 1024)
    ap.add_argument("--rss-poll-ms", type=int, default=50)
    ap.add_argument("--kill-timeout", type=float, default=60.0)
    args = ap.parse_args()

    cmd = json.loads(args.cmd_json)
    report: dict = {
        "lock_wait_s": None,
        "wall_s": None,
        "returncode": None,
        "rss_peak_kb": 0,
        "rss_soft_breach": False,
        "rss_hard_kill": False,
        "timeout_kill": False,
        "procs_seen": 0,
    }

    # ── G1 单实例并发锁（阻塞串行化，超时 fail-closed）─────────────
    t_lock = time.perf_counter()
    lock_fh = open(args.lock_path, "w")
    try:
        while True:
            try:
                fcntl.flock(lock_fh, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.perf_counter() - t_lock > args.lock_timeout:
                    report["lock_error"] = "lock_timeout"
                    sys.stdout.write(json.dumps(report))
                    return 0
                time.sleep(0.02)
        report["lock_wait_s"] = round(time.perf_counter() - t_lock, 3)

        # ── 转换执行 + G2/G3 看门狗（独立进程组）────────────────────
        t0 = time.perf_counter()
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,  # 独立进程组（组杀语义）
        )
        hard_kill = False
        timeout_kill = False
        while True:
            rc = proc.poll()
            # RSS 树扫描（含 guard 自身外的转换树——以 proc.pid 为根）
            rss_kb, live = descendants_rss_kb(proc.pid)
            if rss_kb > report["rss_peak_kb"]:
                report["rss_peak_kb"] = rss_kb
            report["procs_seen"] = max(report["procs_seen"], live)
            if rss_kb >= args.rss_soft_kb:
                report["rss_soft_breach"] = True
            if rss_kb >= args.rss_hard_kb:
                hard_kill = True
            elapsed = time.perf_counter() - t0
            if rc is not None:
                break
            if hard_kill or elapsed > args.kill_timeout:
                timeout_kill = not hard_kill
                try:
                    os.killpg(proc.pid, signal.SIGKILL)
                except OSError:
                    pass
                proc.wait()
                break
            time.sleep(args.rss_poll_ms / 1000.0)

        report["wall_s"] = round(time.perf_counter() - t0, 3)
        report["returncode"] = proc.returncode
        report["rss_hard_kill"] = hard_kill
        report["timeout_kill"] = timeout_kill
        try:
            report["stdout_b64"] = base64.b64encode(proc.stdout.read()).decode("ascii")
            report["stderr_b64"] = base64.b64encode(proc.stderr.read()).decode("ascii")
        except Exception:
            pass
    finally:
        try:
            fcntl.flock(lock_fh, fcntl.LOCK_UN)
        except OSError:
            pass
        lock_fh.close()

    sys.stdout.write(json.dumps(report))
    return 0


if __name__ == "__main__":
    sys.exit(main())
