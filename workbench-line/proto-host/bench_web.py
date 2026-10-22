#!/usr/bin/env python3
"""阶段1宿主选型 — 原型A（web 候选）基准驱动器。

流程：数据集复制 → dist/ 静态服务 → Playwright Chromium headless
（1280×800，与原生侧 Xvfb 屏幕同尺寸）→ 依序执行场景 → 结果落档。

拖拽 = page.mouse 真实 CDP 输入（与原生侧 XTEST 对称）；
启动 = 页面装载即测（startup 指标页面侧产出）。
"""
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
DATA = HERE / "data"
WEB_DIST = Path("../../../mdcore/web/dist").resolve()
RESULTS = HERE / "results"
PORT = 8124

SCEN_ORDER = [
    "doc_open", "doc_edit", "doc_jump_end",
    "evidence_render", "evidence_jump",
    "table_open", "table_scroll", "table_edit",
    "canvas_render", "canvas_hit", "fps_sustain",
]

N_DRAG_STEPS = 60


def proc_tree_rss_kb(_browser) -> int:
    """本沙箱唯一 chromium 实例 = 本驱动所启——按可执行名扫描 /proc 求
    RSS 和（browser+renderer+gpu）。Playwright browser.process() 在当前
    启动模式下返回 None，故用进程名口径（沙箱内无其他 chrome 实例）。"""
    import subprocess as sp
    total = 0
    for proc in Path("/proc").iterdir():
        if not proc.name.isdigit():
            continue
        try:
            exe = (proc / "comm").read_text().strip()
            if "chrom" not in exe:
                continue
            for line in (proc / "status").read_text().splitlines():
                if line.startswith("VmRSS:"):
                    total += int(line.split()[1])
        except Exception:
            continue
    return total


def main() -> int:
    RESULTS.mkdir(exist_ok=True)
    # 数据集 → dist/bench-data/
    bd = WEB_DIST / "bench-data"
    bd.mkdir(exist_ok=True)
    for name in ["doc.json", "evidence.json", "table.json", "canvas.json"]:
        shutil.copy2(DATA / name, bd / name)
        print(f"[data] {name} -> dist/bench-data/")

    server = subprocess.Popen(
        [sys.executable, "-m", "http.server", str(PORT), "--directory", str(WEB_DIST)],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    time.sleep(1.0)
    try:
        with sync_playwright() as pw:
            browser = pw.chromium.launch(headless=True)
            page = browser.new_page(viewport={"width": 1280, "height": 800})
            page.goto(f"http://127.0.0.1:{PORT}/bench.html")
            page.wait_for_function("window.__hostBench && window.__hostBench.ready", timeout=60_000)
            print("[ready] bench page up")

            metrics = [page.evaluate("window.__hostBench.startup()")]

            for name in SCEN_ORDER:
                if name == "canvas_drag":
                    continue  # 单独驱动（真实输入）
                m = page.evaluate(f"window.__hostBench.run('{name}')")
                metrics.append(m)
                extra = m.get("extra", {})
                print(f"[scen] {name:16s} {m['ms']:9.1f} ms  {json.dumps(extra, ensure_ascii=False)[:100]}")

            # canvas_drag：CDP 真实鼠标（与原生 XTEST 对称）
            pos = page.evaluate("window.__hostBench.beginDrag()")
            page.mouse.move(pos["x"], pos["y"])
            page.mouse.down()
            for k in range(1, N_DRAG_STEPS + 1):
                f = k / N_DRAG_STEPS
                page.mouse.move(pos["x"] + 37 * f, pos["y"] + 21 * f)
                page.wait_for_timeout(16)
            page.mouse.up()
            page.wait_for_timeout(120)
            m = page.evaluate("window.__hostBench.endDrag()")
            metrics.append(m)
            print(f"[scen] canvas_drag      {m['ms']:9.1f} ms  {json.dumps(m['extra'], ensure_ascii=False)[:120]}")

            mem = page.evaluate("window.__hostBench.memory()")
            ua = page.evaluate("window.__hostBench.env()")
            # 浏览器进程树 RSS（browser+renderer+gpu 求和——与原生侧单进程
            # VmRSS 的公平对比面；JS heap 单列）
            mem["browser_tree_rss_kb"] = proc_tree_rss_kb(browser)

            # 按原生侧输出顺序重排（startup, settle 序, drag, hit, fps）
            out = {
                "side": "web",
                "env": {"renderer": ua, "engine": "Chromium headless（软渲染）"},
                "scenarios": metrics,
                "memory": mem,
            }
            (RESULTS / "web.json").write_text(json.dumps(out, ensure_ascii=False, indent=2))
            print(f"\n[done] {len(metrics)} scenarios -> {RESULTS / 'web.json'}")

            browser.close()
    finally:
        server.terminate()
    return 0


if __name__ == "__main__":
    sys.exit(main())
