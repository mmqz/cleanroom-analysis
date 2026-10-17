#!/usr/bin/env python3
"""R-O2 回归套件（r-o2-t3）：DOCX → LibreOffice headless → PDF 渲染管线。

确定性、CI 可跑的回归测试：14 用例 = 9 有效夹具（要素级断言）+
5 畸形输入（行为钉定）。每用例冷/暖双次转换（隔离
-env:UserInstallation profile 目录），逐断言 expected/actual 记录，
逐转换墙钟 + 子进程峰值 RSS（rss_wrapper.py 口径），全量 JSON 报告。

用法：
  python3 regress.py [--filter SUBSTR] [--timeout SECS] [--json-out PATH] [--list]

净室纪律：python stdlib only；soffice/pdftotext/pdfinfo/pdfimages/pdffonts
为环境工具（poppler 仅作**检视面**，非替代渲染器）；本套件为**要素级**
覆盖（非视觉级）——通过 ≠ 通用 DOCX 保真。

行为钉定语义（kind=malformed-pinned）：通过 = 钉定行为被确认（LibreOffice
25.2.3.2 实测行为）。LO 版本升级（上游 26.8.1）导致钉定破裂 = 版本差
信号，需复核后更新钉定——不是套件缺陷；此语义在报告 meta 与
REGRESSION.md 双登记。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ro2_fixtures as fx  # noqa: E402  确定性夹具构建器（本目录）

FIXTURES_DIR = HERE / "fixtures"        # 已提交的二进制夹具（import_sample.docx）
OUT_DIR = HERE / "out"                  # 运行产物（gitignored）
PROFILE_ROOT = Path("/tmp/ro2-regress-profiles")  # 隔离 LO profile（任务口径）

LO_CAVEAT = ("LibreOffice 本机 25.2.3.2 vs 上游 26.8.1 版本差（office-f §5.2 登记）——"
             "全部断言（含钉定）在上游版本复跑；钉定破裂 = 版本差信号，需复核后更新。")
RSS_METHOD = ("per-conversion getrusage(RUSAGE_CHILDREN).ru_maxrss（Linux: kB），由独立 "
              "rss_wrapper.py 子进程在等待 soffice 结束后读取；每次转换全新 wrapper 进程，"
              "故为该次转换的子进程峰值（含被等待后代）。/usr/bin/time 不存在于本环境"
              "（已核实），以此为诚实替代口径。")
BOUNDARIES = [
    "要素级（非视觉级）覆盖：通过 ≠ 通用 DOCX 保真（字体替换视觉近似/表格度量/色彩管理不主张）",
    "poppler 工具仅作 PDF 检视面，非替代渲染器；渲染管线 = DOCX → LibreOffice headless → PDF",
    "mdc-pdf 消费腿未覆盖（Q3-gated 实施轮项）——本套件止步于 PDF 产出与检视",
    "小夹具 KB 级口径；大文档/并发转换未覆盖（office-f §5.1 遗留维持）",
    "夹具字节级确定性为同平台同环境口径（zip 条目固定时间戳 1980-01-01）；跨平台字节级一致不主张",
    "LO 25.2.3.2 本机实测；上游 26.8.1 复跑为版本差核对前置（LO_CAVEAT）",
]


# ── poppler 检视面 ──────────────────────────────────────────────────────────


def _run(cmd: list[str]) -> str:
    return subprocess.run(cmd, capture_output=True, text=True, errors="replace").stdout


def pdf_pages(pdf: Path) -> int | None:
    for line in _run(["pdfinfo", str(pdf)]).splitlines():
        if line.startswith("Pages:"):
            return int(line.split(":", 1)[1].strip())
    return None


def page_sizes(pdf: Path, n_pages: int) -> list[list[int]]:
    """逐页尺寸（pt，四舍五入整数）——pdfinfo -f 1 -l N。"""
    sizes: list[list[int]] = []
    for line in _run(["pdfinfo", "-f", "1", "-l", str(n_pages), str(pdf)]).splitlines():
        if "size:" in line and "pts" in line:
            parts = line.split("size:", 1)[1].strip().split(" pts")[0]
            w, h = parts.split(" x ")
            sizes.append([round(float(w)), round(float(h))])
    return sizes


def page_text(pdf: Path, page: int) -> str:
    return _run(["pdftotext", "-f", str(page), "-l", str(page), str(pdf), "-"])


def pdf_text(pdf: Path) -> str:
    return _run(["pdftotext", str(pdf), "-"])


def norm(s: str) -> str:
    """断言归一化：剥全部空白——pdftotext 在 CJK/拉丁边界插空格（登记发现②）。"""
    return "".join(ch for ch in s if not ch.isspace())


def pdf_image_count(pdf: Path) -> int:
    lines = [l for l in _run(["pdfimages", "-list", str(pdf)]).splitlines() if l.strip()]
    return max(0, len(lines) - 2)  # 首行 header + 次行列名，其后每行一图


def pdf_font_names(pdf: Path) -> list[str]:
    fonts = []
    for line in _run(["pdffonts", str(pdf)]).splitlines()[2:]:
        if line.strip():
            fonts.append(line.split()[0])
    return fonts


def inspect_pdf(pdf: Path) -> dict:
    info: dict = {"pdf": str(pdf), "exists": pdf.exists()}
    if not pdf.exists():
        return info
    info["bytes"] = pdf.stat().st_size
    info["pages"] = pdf_pages(pdf)
    text = pdf_text(pdf)
    info["text_chars"] = len(text)
    info["ntext"] = norm(text)
    if info["pages"]:
        info["sizes"] = page_sizes(pdf, info["pages"])
    info["images"] = pdf_image_count(pdf)
    info["fonts"] = pdf_font_names(pdf)
    return info


# ── 转换执行（超时治理 + 峰值 RSS） ─────────────────────────────────────────


def find_soffice_pids() -> set[int]:
    """扫描 /proc/*/comm 找 soffice 族进程（stdlib，无 ps 依赖）。"""
    pids: set[int] = set()
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            comm = Path(f"/proc/{entry}/comm").read_text().strip()
        except OSError:
            continue
        if comm in ("soffice", "soffice.bin", "oosplash"):
            pids.add(int(entry))
    return pids


def convert(docx: Path, outdir: Path, profile: Path, timeout: float,
            baseline_soffice: set[int]) -> dict:
    """单次转换：独立 wrapper 进程（进程组组长）跑 soffice，超时杀整组。"""
    outdir.mkdir(parents=True, exist_ok=True)
    cmd = [
        "soffice", "--headless", "--norestore",
        f"-env:UserInstallation={profile.as_uri()}",
        "--convert-to", "pdf", "--outdir", str(outdir), str(docx),
    ]
    wrapper = [sys.executable, str(HERE / "rss_wrapper.py"), json.dumps(cmd)]
    t0 = time.perf_counter()
    proc = subprocess.Popen(wrapper, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            start_new_session=True)
    try:
        out, _err = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        # 杀整个进程组（wrapper + soffice 同组；soffice 启动脚本 exec 为 soffice.bin）
        kill_note = "killed_pg"
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            kill_note = "killpg_failed_fallback_proc_kill"
            proc.kill()
        proc.communicate()
        time.sleep(0.2)  # SIGKILL 后短暂等待内核收尸，再核残留
        leftover = sorted(find_soffice_pids() - baseline_soffice)
        return {
            "timeout": True,
            "kill": kill_note,
            "soffice_leftover_pids": leftover,
            "wall_s": round(time.perf_counter() - t0, 3),
            "exit_code": None,
            "pdf": False,
            "maxrss_kb": None,
            "stdout_digest": "",
            "stderr_digest": "",
        }
    wall = time.perf_counter() - t0
    rec: dict = {"timeout": False, "wall_s": round(wall, 3)}
    try:
        data = json.loads(out.decode("utf-8", "replace"))
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        rec.update({"wrapper_error": f"json_decode: {exc!r}", "exit_code": None,
                    "pdf": False, "maxrss_kb": None,
                    "stdout_digest": "", "stderr_digest": ""})
        return rec
    if "wrapper_error" in data:
        rec.update({"wrapper_error": data["wrapper_error"], "exit_code": None,
                    "pdf": False, "maxrss_kb": None,
                    "stdout_digest": "", "stderr_digest": ""})
        return rec
    stdout = base64_decode(data.get("stdout_b64", ""))
    stderr = base64_decode(data.get("stderr_b64", ""))
    rec.update({
        "exit_code": data["returncode"],
        # 墙钟口径：wrapper 内测得的 soffice 专属墙钟（排除 wrapper python 启动）
        "soffice_wall_s": data.get("wall_s"),
        "harness_wall_s": rec.pop("wall_s"),
        "wall_s": data.get("wall_s"),
        "maxrss_kb": data.get("maxrss_kb"),
        "stdout_digest": digest(stdout),
        "stderr_digest": digest(stderr),
        "combined_digest": digest(stdout + "\n" + stderr),
    })
    rec["pdf"] = (outdir / (docx.stem + ".pdf")).exists()
    return rec


def base64_decode(b64: str) -> str:
    import base64 as _b64
    try:
        return _b64.b64decode(b64).decode("utf-8", "replace")
    except Exception:
        return ""


def digest(s: str, n: int = 240) -> str:
    return " ".join(s.split())[:n]


# ── 断言记录器 ──────────────────────────────────────────────────────────────


class Rec:
    """逐断言 expected/actual 记录。"""

    def __init__(self) -> None:
        self.items: list[dict] = []

    def value(self, name: str, op: str, expected, actual, note: str | None = None) -> bool:
        ops = {
            "eq": lambda: actual == expected,
            "ne": lambda: actual != expected,
            "ge": lambda: actual is not None and actual >= expected,
            "le": lambda: actual is not None and actual <= expected,
        }
        passed = bool(ops.get(op, lambda: False)())
        item = {"name": name, "op": op, "expected": expected, "actual": actual, "pass": passed}
        if note:
            item["note"] = note
        self.items.append(item)
        return passed

    def contains(self, name: str, substr: str, present: bool, *, expect: bool = True,
                 detail: str | None = None, note: str | None = None) -> bool:
        passed = present is expect
        item = {
            "name": name, "op": "contains" if expect else "absent",
            "expected": substr, "actual": bool(present), "pass": passed,
        }
        if detail:
            item["detail"] = detail
        if note:
            item["note"] = note
        self.items.append(item)
        return passed

    def contains_norm(self, name: str, kw: str, ntext: str, *,
                      note: str | None = None) -> bool:
        """归一化文本包含断言：期望串与抽取文本**双侧**剥空白后比对
        （pdftotext CJK/拉丁边界插空格——登记发现②的断言口径）。"""
        return self.contains(name, kw, norm(kw) in ntext,
                             note=note or "双侧空白归一化后比对")

    @property
    def all_pass(self) -> bool:
        return all(i["pass"] for i in self.items)

    @property
    def fail_count(self) -> int:
        return sum(0 if i["pass"] else 1 for i in self.items)


# ── 夹具注册表（确定性构建） ────────────────────────────────────────────────


def build_all_fixtures() -> dict[str, Path]:
    """全量构建（不受 --filter 影响——sha256 台账每次全量复验）。"""
    fdir = OUT_DIR / "fixtures"
    fdir.mkdir(parents=True, exist_ok=True)
    fx.build_basic_structure(fdir / "basic-structure.docx")
    fx.build_explicit_page_breaks(fdir / "explicit-page-breaks.docx")
    fx.build_table_spanning_pages(fdir / "table-spanning-pages.docx")
    fx.build_headers_footers(fdir / "headers-footers.docx")
    fx.build_section_boundaries(fdir / "section-boundaries.docx")
    fx.build_missing_font(fdir / "missing-font.docx")
    fx.build_embedded_image(fdir / "embedded-image.docx")
    fx.build_composite(fdir / "composite.docx")
    fx.copy_real_fixture(fdir / "real-fixture.docx", FIXTURES_DIR / "import_sample.docx")
    # 畸形输入（truncated-zip 派生自 basic-structure 字节）
    fx.build_garbage_bytes(fdir / "garbage-bytes.docx")
    fx.build_truncated_zip(fdir / "truncated-zip.docx",
                           (fdir / "basic-structure.docx").read_bytes())
    fx.build_bad_xml(fdir / "bad-xml.docx")
    fx.build_missing_content_types(fdir / "missing-content-types.docx")
    fx.build_drawing_at_paragraph_level(fdir / "drawing-at-paragraph-level.docx")
    return {p.stem: p for p in sorted(fdir.glob("*.docx"))}


def check_fixture_determinism(fixtures: dict[str, Path]) -> dict:
    """sha256 台账：与上次运行（同工作区）对比——字节级确定性回归门。"""
    ledger_path = OUT_DIR / "fixture-hashes.json"
    current = {name: hashlib.sha256(p.read_bytes()).hexdigest()
               for name, p in sorted(fixtures.items())}
    report: dict = {"current_sha256": current}
    if ledger_path.exists():
        try:
            prev = json.loads(ledger_path.read_text())
        except json.JSONDecodeError:
            prev = None
        if prev is None:
            report.update({"all_match": None, "note": "旧台账不可解析——本轮重写为基线"})
        else:
            mismatches = {n: {"prev": prev.get(n), "curr": current.get(n)}
                          for n in set(prev) | set(current)
                          if prev.get(n) != current.get(n)}
            report["all_match"] = not mismatches
            report["mismatches"] = mismatches
            report["note"] = ("同工作区重跑确定性复验（非首次）" if not mismatches
                              else "夹具字节漂移——非确定性回归！")
    else:
        report.update({"all_match": None, "note": "首次运行——台账基线落档"})
    ledger_path.write_text(json.dumps(current, indent=2, ensure_ascii=False))
    return report


# ── 用例断言逻辑 ────────────────────────────────────────────────────────────


def common_valid(rec: Rec, conv: dict, info: dict) -> None:
    rec.value("no_timeout_cold", "eq", True, not conv["cold"]["timeout"])
    rec.value("no_timeout_warm", "eq", True, not conv["warm"]["timeout"])
    rec.value("exit_code_cold", "eq", 0, conv["cold"]["exit_code"])
    rec.value("exit_code_warm", "eq", 0, conv["warm"]["exit_code"])
    rec.value("pdf_produced_warm", "eq", True, info["exists"])


def common_no_timeout(rec: Rec, conv: dict) -> None:
    rec.value("no_timeout_cold", "eq", True, not conv["cold"]["timeout"])
    rec.value("no_timeout_warm", "eq", True, not conv["warm"]["timeout"])


def checks_basic_structure(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    rec.value("page_count", "eq", 2, info.get("pages"))
    for kw in ["结构样本标题段", "二级标题", "粗体", "斜体", "指标", "收入", "成本"]:
        rec.contains_norm(f"text_{kw}", kw, info.get("ntext", ""))
    p1, p2 = norm(page_text(pdf, 1)), norm(page_text(pdf, 2)) if info.get("pages", 0) >= 2 else ""
    rec.contains("table_on_page1", "指标", "指标" in p1)
    rec.contains("post_break_text_on_page2", "分页符之后的段落", "分页符之后的段落" in p2)


def checks_explicit_page_breaks(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    rec.value("page_count", "eq", 6, info.get("pages"), note="5 个显式分页符 → 恰 6 页（硬断言）")
    found = {}
    for pg in range(1, (info.get("pages") or 0) + 1):
        found[pg] = f"第{pg}页内容" in norm(page_text(pdf, pg))
    rec.value("per_page_text_on_its_page", "eq", True,
              all(found.values()) and len(found) == 6,
              note=f"逐页定位：{found}")


def checks_table_spanning_pages(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    n = info.get("pages") or 0
    rec.value("page_count_ge_2", "ge", 2, n, note="80 行表 → 至少 2 页")
    first_row_page = next((pg for pg in range(1, n + 1)
                           if "行001-甲" in norm(page_text(pdf, pg))), None)
    last_row_page = next((pg for pg in range(1, n + 1)
                          if "行080-丙" in norm(page_text(pdf, pg))), None)
    rec.value("first_row_on_page_1", "eq", 1, first_row_page,
              note="首行（行001-甲）所在页（逐页文本抽取 + 空白归一化）")
    rec.value("last_row_page_exists", "ne", None, last_row_page)
    rec.value("last_row_on_different_page", "eq", True,
              last_row_page is not None and last_row_page != first_row_page,
              note=f"末行（行080-丙）在页 {last_row_page} ≠ 首行页 {first_row_page}（跨页实证）")
    rec.contains_norm("header_text_present", "列一", info.get("ntext", ""))
    if last_row_page:
        rec.contains("tbl_header_repeated_on_last_row_page", "列一",
                     "列一" in norm(page_text(pdf, last_row_page)),
                     detail=f"页 {last_row_page}（tblHeader 重复表头）")
    rec.contains_norm("mid_row_present", "行040-乙", info.get("ntext", ""))


def checks_headers_footers(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    rec.value("page_count", "eq", 3, info.get("pages"))
    n = info.get("pages") or 0
    hdr = {pg: fx.HDR_TEXT in norm(page_text(pdf, pg)) for pg in range(1, n + 1)}
    ftr = {pg: fx.FTR_TEXT in norm(page_text(pdf, pg)) for pg in range(1, n + 1)}
    rec.value("header_on_every_page", "eq", True, bool(hdr) and all(hdr.values()),
              note=f"逐页页眉：{hdr}")
    rec.value("footer_on_every_page", "eq", True, bool(ftr) and all(ftr.values()),
              note=f"逐页页脚：{ftr}")


def checks_section_boundaries(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    sizes = info.get("sizes") or []
    rec.value("page_count", "eq", 2, info.get("pages"))
    rec.value("page1_size_A4_portrait", "eq", [595, 842], sizes[0] if len(sizes) >= 1 else None,
              note="节 1 = 11906×16838 twip 纵向 → 595×842pt（pdfinfo -f/-l）")
    rec.value("page2_size_landscape", "eq", [842, 595], sizes[1] if len(sizes) >= 2 else None,
              note="节 2 = 16838×11906 twip 横向 → 842×595pt（宽>高）")
    n = info.get("pages") or 0
    p1 = norm(page_text(pdf, 1)) if n >= 1 else ""
    p2 = norm(page_text(pdf, 2)) if n >= 2 else ""
    rec.contains("sect1_text_on_page1", "第一节正文", "第一节正文" in p1)
    rec.contains("sect2_text_on_page2", "第二节正文", "第二节正文" in p2)


def checks_missing_font(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    rec.contains_norm("missing_font_text_still_present", "此行文本应仍可读",
                      info.get("ntext", ""),
                      note="rFonts=TotallyMissingFont123 → 替换不丢内容")
    rec.contains_norm("control_line_present", "常规字体对照行", info.get("ntext", ""))
    fonts = info.get("fonts") or []
    rec.value("no_phantom_font_embedded", "eq", True,
              all("TotallyMissingFont123" not in f for f in fonts),
              note=f"pdffonts：{fonts}")
    rec.value("substitute_fonts_embedded", "ge", 1, len(fonts))


def checks_embedded_image(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    rec.value("image_count_ge_1", "ge", 1, info.get("images"),
              note="8×8 stdlib PNG + run 级 w:drawing inline → pdfimages ≥1")
    rec.contains_norm("text_after_image_present", "图片后段落文本", info.get("ntext", ""))
    rec.contains_norm("text_before_image_present", "图片样本", info.get("ntext", ""))


def checks_composite(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_valid(rec, conv, info)
    n = info.get("pages") or 0
    rec.value("page_count_ge_4", "ge", 4, n, note="全要素组合（表格跨页+分节+分页符）")
    hdr = {pg: fx.HDR_TEXT in norm(page_text(pdf, pg)) for pg in range(1, n + 1)}
    rec.value("header_on_every_page", "eq", True, bool(hdr) and all(hdr.values()),
              note=f"逐页页眉：{hdr}")
    sizes = info.get("sizes") or []
    rec.value("page1_size_A4_portrait", "eq", [595, 842], sizes[0] if len(sizes) >= 1 else None)
    rec.value("landscape_page_present", "eq", True, [842, 595] in sizes,
              note=f"逐页尺寸：{sizes}")
    for kw in ["组合夹具第一节", "组合表头A", "组合行55-右", "此行文本应仍可读",
               "第二节第 2 页内容"]:
        rec.contains_norm(f"text_{kw[:8]}", kw, info.get("ntext", ""))
    rec.value("image_count_ge_1", "ge", 1, info.get("images"))


def checks_real_fixture(rec: Rec, conv: dict, info: dict, pdf: Path,
                        committed_sha: str, fixture_sha: str) -> None:
    common_valid(rec, conv, info)
    rec.value("fixture_sha256_matches_committed_copy", "eq", committed_sha, fixture_sha,
              note="out/fixtures 副本须与已提交 fixtures/import_sample.docx 字节一致")
    rec.value("page_count", "eq", 1, info.get("pages"))
    for kw in ["Imported Title", "Header Slot", "Footer Slot", "Bullet one",
               "Nested step", "alpha", "plain bold and italic and docs", "Value"]:
        rec.contains_norm(f"text_{kw[:12]}", kw, info.get("ntext", ""),
                          note="mdcore t16 导入链真件（docx-rs 生成）关键串，双侧归一化比对")
    rec.value("image_count_ge_1", "ge", 1, info.get("images"),
              note="真件 1×1px PNG → image+smask 双对象（登记发现③口径）")


# ── 畸形输入（行为钉定） ────────────────────────────────────────────────────

PIN_NOTE_LOAD_FAIL = ("钉定：rc=0 但 PDF 不产出 + 输出含 'Error: source file could not be "
                      "loaded'——**退出码不是失败信号**（LO 25.2.3.2 实测），检出策略 = "
                      "PDF 产出核验 + 输出扫描。rc 作为观察值记录（版本差敏感）。")


def checks_garbage_bytes(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_no_timeout(rec, conv)
    rec.value("pin_exit_code_cold", "eq", 0, conv["cold"]["exit_code"],
              note="钉定：LO 文本导入兜底——垃圾字节按 Writer 文本文档打开（新发现，见 REGRESSION.md）")
    rec.value("pin_exit_code_warm", "eq", 0, conv["warm"]["exit_code"])
    rec.value("pin_pdf_produced_warm", "eq", True, info["exists"],
              note="钉定：乱码 PDF 仍产出（无报错）——敌意输入面关注点")
    rec.value("pin_pdf_produced_cold", "eq", True, conv["cold"]["pdf"])


def _pins_load_failure(rec: Rec, conv: dict, info: dict) -> None:
    common_no_timeout(rec, conv)
    rec.value("pin_pdf_not_produced_warm", "eq", False, info["exists"], note=PIN_NOTE_LOAD_FAIL)
    rec.value("pin_pdf_not_produced_cold", "eq", False, conv["cold"]["pdf"])
    rec.value("pin_load_error_message_warm", "eq", True,
              "source file could not be loaded" in conv["warm"]["combined_digest"],
              note="输出含 load 失败消息（stdout+stderr 合并扫描）")
    rec.value("pin_load_error_message_cold", "eq", True,
              "source file could not be loaded" in conv["cold"]["combined_digest"])


def checks_truncated_zip(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    _pins_load_failure(rec, conv, info)


def checks_bad_xml(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    _pins_load_failure(rec, conv, info)


def checks_missing_content_types(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    _pins_load_failure(rec, conv, info)


def checks_drawing_at_paragraph_level(rec: Rec, conv: dict, info: dict, pdf: Path) -> None:
    common_no_timeout(rec, conv)
    rec.value("pin_exit_code_cold", "eq", 0, conv["cold"]["exit_code"],
              note="钉定（r-o2-t2 登记发现①）：w:drawing 直接置于 w:p = 非法 OOXML →"
                   " LO 静默丢弃图片，退出码 0、转换『成功』")
    rec.value("pin_exit_code_warm", "eq", 0, conv["warm"]["exit_code"])
    rec.value("pin_pdf_produced_warm", "eq", True, info["exists"])
    rec.value("pin_image_count_eq_0", "eq", 0, info.get("images"),
              note="钉定核心：图片被静默丢弃（pdfimages 计数恰 0）——**检出策略证明："
                   "敌意/畸形输入必须用转换后要素计数断言，不能信退出码**")
    rec.contains_norm("pin_text_intact", "图片后段落文本", info.get("ntext", ""),
                      note="文本完好（静默丢弃只丢图片，不丢文本）")


# ── 用例注册表 ──────────────────────────────────────────────────────────────

CASES = [
    {"name": "basic-structure", "kind": "valid", "checks": checks_basic_structure,
     "desc": "标题 + 粗/斜 run + 2×3 表 + 显式分页（承第一轮 A）"},
    {"name": "explicit-page-breaks", "kind": "valid", "checks": checks_explicit_page_breaks,
     "desc": "5 个显式分页符 → 恰 6 页 + 逐页关键串（承 B）"},
    {"name": "table-spanning-pages", "kind": "valid", "checks": checks_table_spanning_pages,
     "desc": "80 行表 + tblHeader 重复表头 → 跨页实证（承 D）"},
    {"name": "headers-footers", "kind": "valid", "checks": checks_headers_footers,
     "desc": "header1/footer1 + sectPr 引用，3 页逐页页眉页脚（承 E）"},
    {"name": "section-boundaries", "kind": "valid", "checks": checks_section_boundaries,
     "desc": "节 1 A4 纵 595×842pt → 节 2 横向 842×595pt（承 F）"},
    {"name": "missing-font", "kind": "valid", "checks": checks_missing_font,
     "desc": "rFonts=不存在字体 → 替换不丢内容 + 无幻影嵌入（承 G）"},
    {"name": "embedded-image", "kind": "valid", "checks": checks_embedded_image,
     "desc": "8×8 stdlib PNG + run 级 w:drawing inline（承 H）"},
    {"name": "composite", "kind": "valid", "checks": checks_composite,
     "desc": "全要素组合：页眉页脚+分节+跨页表+图片+缺失字体+分页（承 I）"},
    {"name": "real-fixture", "kind": "valid", "checks": checks_real_fixture,
     "desc": "mdcore import_sample.docx 真件（承第一轮 C；fixtures/ 已提交副本）"},
    {"name": "garbage-bytes", "kind": "malformed-pinned", "checks": checks_garbage_bytes,
     "desc": "4096B 确定性伪随机字节（非 zip）——钉定 LO 文本导入兜底"},
    {"name": "truncated-zip", "kind": "malformed-pinned", "checks": checks_truncated_zip,
     "desc": "有效夹具前 60% 字节——钉定加载失败行为"},
    {"name": "bad-xml", "kind": "malformed-pinned", "checks": checks_bad_xml,
     "desc": "document.xml 未闭合标签——钉定加载失败行为"},
    {"name": "missing-content-types", "kind": "malformed-pinned", "checks": checks_missing_content_types,
     "desc": "缺 [Content_Types].xml 部件——钉定加载失败行为"},
    {"name": "drawing-at-paragraph-level", "kind": "malformed-pinned",
     "checks": checks_drawing_at_paragraph_level,
     "desc": "w:p 直接子级 w:drawing（非法 OOXML）——钉定静默丢弃（登记发现①）"},
]


# ── 用例执行 ────────────────────────────────────────────────────────────────


def run_case(case: dict, timeout: float, baseline_soffice: set[int],
             fixtures: dict[str, Path]) -> dict:
    name = case["name"]
    fixture = fixtures[name]
    case_dir = OUT_DIR / "cases" / name
    shutil.rmtree(case_dir, ignore_errors=True)
    case_dir.mkdir(parents=True)
    profile = PROFILE_ROOT / name

    # 冷：全新 profile（含 LO 首启/profile 建档开销）；暖：同 profile 复用
    shutil.rmtree(profile, ignore_errors=True)
    cold = convert(fixture, case_dir / "cold", profile, timeout, baseline_soffice)
    cold_pages = None
    cold_pdf = case_dir / "cold" / f"{name}.pdf"
    if cold_pdf.exists():
        cold_pages = pdf_pages(cold_pdf)
    warm = convert(fixture, case_dir / "warm", profile, timeout, baseline_soffice)

    warm_pdf = case_dir / "warm" / f"{name}.pdf"
    info = inspect_pdf(warm_pdf)

    rec = Rec()
    try:
        if name == "real-fixture":
            committed_sha = hashlib.sha256(
                (FIXTURES_DIR / "import_sample.docx").read_bytes()).hexdigest()
            fixture_sha = hashlib.sha256(fixture.read_bytes()).hexdigest()
            case["checks"](rec, {"cold": cold, "warm": warm}, info, warm_pdf,
                           committed_sha, fixture_sha)
        else:
            case["checks"](rec, {"cold": cold, "warm": warm}, info, warm_pdf)
    except Exception as exc:  # 断言器自身异常：记录而非崩溃（套件永不因单用例崩）
        rec.value("assertion_harness_error", "eq", None, repr(exc),
                  note="断言器异常——按失败记录并继续")

    all_pass = rec.all_pass

    fixture_sha = hashlib.sha256(fixture.read_bytes()).hexdigest()
    record = {
        "case": name,
        "kind": case["kind"],
        "desc": case["desc"],
        "fixture": str(fixture.relative_to(HERE)),
        "fixture_bytes": fixture.stat().st_size,
        "fixture_sha256": fixture_sha,
        "conversion": {
            "cold": {**cold, "pdf_pages": cold_pages},
            "warm": {**warm, "pdf_pages": info.get("pages")},
            "cold_warm_page_count_consistent": (
                cold_pages is None and not info.get("exists")
                or cold_pages == info.get("pages")),
        },
        "pdf_inspection_warm": {k: v for k, v in info.items() if k != "ntext"},
        "ntext_excerpt": info.get("ntext", "")[:120],
        "assertions": rec.items,
        "assertion_count": len(rec.items),
        "failed_assertions": rec.fail_count,
        "result": None,  # 由下方一致性门后回填
        **({"pin_broken": True,
            "pin_note": "钉定破裂——LO 行为变化（版本差信号）：复核后更新钉定"}
           if case["kind"] == "malformed-pinned" and not all_pass else {}),
    }
    # e-review NOTE 6 收口：冷/暖页数一致性 = 硬门（非确定性转换 → FAIL；
    # 此前仅警告打印——套件语义收严）
    if not record["conversion"]["cold_warm_page_count_consistent"]:
        all_pass = False
    if case["kind"] == "valid":
        record["result"] = "pass" if all_pass else "fail"
    else:
        record["result"] = "pinned" if all_pass else "fail"
    return record


# ── 环境 meta ───────────────────────────────────────────────────────────────


def env_meta() -> dict:
    def first_line(cmd: list[str]) -> str:
        try:
            proc = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            return (proc.stdout + proc.stderr).strip().splitlines()[0] if (proc.stdout + proc.stderr).strip() else ""
        except Exception as exc:
            return f"<unavailable: {exc!r}>"

    return {
        "soffice_path": shutil.which("soffice") or "soffice",
        "soffice_version": first_line(["soffice", "--version"]),
        "pdftotext_version": first_line(["pdftotext", "-v"]),
        "pdfimages_version": first_line(["pdfimages", "-v"]),
        "pdffonts_version": first_line(["pdffonts", "-v"]),
        "python_version": sys.version.split()[0],
        "usr_bin_time_exists": Path("/usr/bin/time").exists(),
    }


# ── main ────────────────────────────────────────────────────────────────────


def main() -> int:
    parser = argparse.ArgumentParser(description="R-O2 DOCX→PDF 渲染管线回归套件（r-o2-t3）")
    parser.add_argument("--filter", default="", help="用例名子串过滤")
    parser.add_argument("--timeout", type=float, default=60.0,
                        help="每次转换超时秒数（默认 60；杀整进程组）")
    parser.add_argument("--json-out", default=str(OUT_DIR / "regression-report.json"),
                        help="JSON 报告路径（默认 out/regression-report.json）")
    parser.add_argument("--list", action="store_true", help="列出用例后退出")
    args = parser.parse_args()

    if args.list:
        print(f"{'用例':<30} {'类别':<18} 说明")
        for c in CASES:
            print(f"{c['name']:<30} {c['kind']:<18} {c['desc']}")
        return 0

    OUT_DIR.mkdir(exist_ok=True)
    meta = env_meta()
    baseline_soffice = find_soffice_pids()

    fixtures = build_all_fixtures()
    determinism = check_fixture_determinism(fixtures)

    selected = [c for c in CASES if args.filter in c["name"]]
    if not selected:
        print(f"错误：--filter '{args.filter}' 无匹配用例（--list 查看全表）")
        return 2

    print("R-O2 回归套件（r-o2-t3）：DOCX → LibreOffice headless → PDF（要素级）")
    print(f"环境: {meta['soffice_version']} / {meta['pdftotext_version']} / "
          f"python {meta['python_version']}")
    print(f"夹具: {len(fixtures)} 件确定性构建"
          + (f"，sha256 台账复验：{determinism.get('note', '')}"
             if determinism.get("all_match") is not None else "，台账基线落档"))
    if baseline_soffice:
        print(f"警告：套件启动前已有 soffice 进程在场 {sorted(baseline_soffice)}"
              "（残留检测将排除这些 PID）")
    print(f"用例: {len(selected)}/{len(CASES)}（--filter '{args.filter}'，"
          f"转换超时 {args.timeout:g}s）")
    print()

    results = []
    for idx, case in enumerate(selected, 1):
        t0 = time.perf_counter()
        record = run_case(case, args.timeout, baseline_soffice, fixtures)
        case_wall = time.perf_counter() - t0
        results.append(record)

        cold, warm = record["conversion"]["cold"], record["conversion"]["warm"]
        tag = {"pass": "PASS", "pinned": "PINNED", "fail": "FAIL"}[record["result"]]
        rss = warm.get("maxrss_kb") or cold.get("maxrss_kb")
        rss_s = f"{rss / 1024:.0f}MB" if rss else "n/a"
        pages = record["conversion"]["warm"]["pdf_pages"]
        ctime = cold.get("wall_s")
        wtime = warm.get("wall_s")
        ctime_s = f"{ctime:.2f}s" if ctime is not None else "timeout"
        wtime_s = f"{wtime:.2f}s" if wtime is not None else "timeout"
        print(f"[{idx:>2}/{len(selected)}] {record['case']:<32} {tag:<7} "
              f"冷 {ctime_s:>7} 暖 {wtime_s:>7} 峰值RSS {rss_s:>7} "
              f"页 {pages if pages is not None else '-'}  ({record['assertion_count']} 断言)")
        if record["result"] == "fail":
            for item in record["assertions"]:
                if not item["pass"]:
                    print(f"       ✗ {item['name']} [{item['op']}] "
                          f"expected={item['expected']!r} actual={item['actual']!r}")
        if cold.get("timeout") or warm.get("timeout"):
            for run_name, run in (("cold", cold), ("warm", warm)):
                if run.get("timeout"):
                    print(f"       ⏱ {run_name} 转换超时（{args.timeout:g}s）："
                          f"kill={run.get('kill')}，soffice 残留 PID "
                          f"{run.get('soffice_leftover_pids')}")
        if not record["conversion"]["cold_warm_page_count_consistent"]:
            print(f"       ⚠ 冷/暖页数不一致（flake 信号）："
                  f"cold={cold.get('pdf_pages')} warm={warm.get('pdf_pages')}")

    # 汇总
    n_pass = sum(1 for r in results if r["result"] == "pass")
    n_pinned = sum(1 for r in results if r["result"] == "pinned")
    n_fail = sum(1 for r in results if r["result"] == "fail")
    det_ok = determinism.get("all_match") is not False

    report = {
        "suite": "r-o2-t3",
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "reproduce_cmd": f"python3 {' '.join(sys.argv)}",
        "env": meta,
        "lo_version_caveat": LO_CAVEAT,
        "rss_method": RSS_METHOD,
        "boundaries": BOUNDARIES,
        "pin_semantics": ("malformed-pinned 用例通过 = 钉定行为（LO 25.2.3.2 实测）被确认；"
                          "版本升级导致破裂 = 版本差信号，复核后更新钉定"),
        "fixture_determinism": determinism,
        "suite_timeout_s": args.timeout,
        "soffice_baseline_pids": sorted(baseline_soffice),
        "cases": results,
        "summary": {
            "selected": len(results),
            "total_registered": len(CASES),
            "pass": n_pass,
            "pinned": n_pinned,
            "fail": n_fail,
            "fixture_determinism_gate": "pass" if det_ok else "FAIL",
        },
    }
    json_path = Path(args.json_out)
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False))

    print()
    print(f"汇总: {len(results)} 用例 = {n_pass} pass + {n_pinned} pinned + {n_fail} fail"
          f"（夹具确定性门 {'pass' if det_ok else 'FAIL'}）")
    print(f"JSON 报告: {json_path}")
    suite_ok = n_fail == 0 and det_ok
    print("SUITE " + ("PASS" if suite_ok else "FAIL"))
    return 0 if suite_ok else 1


if __name__ == "__main__":
    sys.exit(main())
