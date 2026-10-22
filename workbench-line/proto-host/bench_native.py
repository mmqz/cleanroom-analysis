#!/usr/bin/env python3
"""阶段1宿主选型 — 原型B（原生 egui 候选）基准驱动器。

前置：
  - Xvfb :99 在跑（或本脚本自动拉起）
  - proto-native release 构建（cleanroom-analysis/workbench-line/proto-native）
  - 沙箱补丁库 libxkbcommon-x11/libxcb-xkb（apt download 无 root 提取到
    /tmp/xkbx11 —— 仅沙箱缺件，真实 Linux 桌面标配）

流程：Xvfb 确认 → 运行 --bench → stdout 标记段提取 JSON → 落档。
"""
import json
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROTO = (HERE / "../proto-native").resolve()
DATA = HERE / "data"
RESULTS = HERE / "results"
XKB = Path("/tmp/xkbx11/usr/lib/x86_64-linux-gnu")

MARK_BEGIN = "===BENCH_JSON==="
MARK_END = "===END==="


def ensure_xvfb() -> None:
    r = subprocess.run(["pgrep", "-x", "Xvfb"], capture_output=True)
    if r.returncode == 0:
        print("[xvfb] already running")
        return
    subprocess.Popen(
        ["Xvfb", ":99", "-screen", "0", "1280x800x24", "-nolisten", "tcp"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    time.sleep(1.5)
    print("[xvfb] started :99 (1280x800x24)")


def main() -> int:
    RESULTS.mkdir(exist_ok=True)
    bin_path = PROTO / "target/release/proto-native"
    if not bin_path.exists():
        print(f"[err] missing {bin_path} — cargo build --release first", file=sys.stderr)
        return 1

    ensure_xvfb()

    import os
    env = dict(os.environ)
    env["DISPLAY"] = ":99"
    if XKB.exists():
        env["LD_LIBRARY_PATH"] = str(XKB)

    print(f"[run] {bin_path} --bench {DATA}")
    r = subprocess.run(
        [str(bin_path), "--bench", str(DATA)],
        capture_output=True, text=True, env=env, timeout=300,
    )
    if MARK_BEGIN not in r.stdout:
        print("[err] no bench JSON in stdout", file=sys.stderr)
        print(r.stdout[-2000:], file=sys.stderr)
        print(r.stderr[-2000:], file=sys.stderr)
        return 1

    payload = r.stdout.split(MARK_BEGIN, 1)[1].split(MARK_END, 1)[0].strip()
    out = json.loads(payload)
    (RESULTS / "native.json").write_text(json.dumps(out, ensure_ascii=False, indent=2))

    for m in out["scenarios"]:
        extra = m.get("extra", {})
        print(f"[scen] {m['name']:16s} {m['ms']:9.1f} ms  {json.dumps(extra, ensure_ascii=False)[:100]}")
    print(f"[mem ] {out['memory']}")
    print(f"\n[done] {len(out['scenarios'])} scenarios -> {RESULTS / 'native.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
