#!/usr/bin/env python3
"""R-O2 渲染级读侧评估 spike：LibreOffice headless docx→PDF 转换路线（C 路线）实证。

净室纪律：python stdlib only（zipfile 手搓最小 OOXML，零第三方依赖）；
soffice 子进程隔离调用（ComfyUI 进程外集成先例）；全部数字本机实测。

样本面：
  A 结构样本（手搓 OOXML）：Heading1/2（styles.xml 最小定义）+ 粗/斜/混排
    run + 2×3 表 + 显式分页符 + 分页后段落；
  B 分页样本（手搓 OOXML）：5 个显式分页符 → 期望恰 6 页（显式分页保真
    的硬断言——office-d §6.3 目标线「分页只信文档显式标记」）；
  C 真实夹具：mdcore web/tests/fixtures/import_sample.docx（t16 导入链
    二进制夹具，含标题/列表/超链接/表格）。

验证面（每样本）：转换耗时（冷/热）、pdfinfo 页数、pdftotext 文本关键串
在场断言；B 样本页数硬断言。
"""

import subprocess
import sys
import time
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "out"
OUT.mkdir(exist_ok=True)

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"""

RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"""

DOC_RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"""

STYLES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/>
<w:pPr><w:outlineLvl w:val="0"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/>
<w:pPr><w:outlineLvl w:val="1"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:style>
</w:styles>"""


def doc_xml(body: str) -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
        f"<w:body>{body}"
        '<w:sectPr><w:pgSz w:w="11906" w:h="16838"/>'
        '<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>'
        "</w:body></w:document>"
    )


def p(runs: str, style: str | None = None) -> str:
    ppr = f'<w:pPr><w:pStyle w:val="{style}"/></w:pPr>' if style else ""
    return f"<w:p>{ppr}{runs}</w:p>"


def r(text: str, bold=False, italic=False) -> str:
    rpr = "<w:rPr>" + ("<w:b/>" if bold else "") + ("<w:i/>" if italic else "") + "</w:rPr>"
    return f"<w:r>{rpr}<w:t xml:space=\"preserve\">{text}</w:t></w:r>"


PAGE_BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'


def table(rows: list[list[str]]) -> str:
    trs = "".join(
        "<w:tr>"
        + "".join(f"<w:tc><w:tcPr/><w:p>{r(cell)}</w:p></w:tc>" for cell in row)
        + "</w:tr>"
        for row in rows
    )
    return (
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>"
        '<w:tblBorders><w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/>'
        '<w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/>'
        '<w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/></w:tblBorders>'
        f"</w:tblPr>{trs}</w:tbl>"
    )


def write_docx(path: Path, body: str) -> None:
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", CONTENT_TYPES)
        z.writestr("_rels/.rels", RELS)
        z.writestr("word/document.xml", doc_xml(body))
        z.writestr("word/_rels/document.xml.rels", DOC_RELS)
        z.writestr("word/styles.xml", STYLES)


def build_samples() -> dict[str, Path]:
    # A 结构样本
    body_a = (
        p(r("结构样本标题段", bold=True), "Heading1")
        + p(r("二级标题", bold=True), "Heading2")
        + p(r("普通段落文本，含") + r("粗体", bold=True) + r("、") + r("斜体", italic=True) + r("与混排收尾。"))
        + table([
            ["指标", "Q1", "Q2"],
            ["收入", "100", "180"],
            ["成本", "60", "70"],
        ])
        + PAGE_BREAK
        + p(r("分页符之后的段落——应出现在第 2 页。"))
    )
    a = OUT / "sample_a_structure.docx"
    write_docx(a, body_a)

    # B 分页样本：5 个显式分页符 → 6 页
    body_b = p(r("第 1 页内容。"))
    for i in range(2, 7):
        body_b += PAGE_BREAK + p(r(f"第 {i} 页内容。"))
    b = OUT / "sample_b_pagination.docx"
    write_docx(b, body_b)

    # C 真实夹具（复制只读）
    src = Path("/home/z/my-project/mdcore/web/tests/fixtures/import_sample.docx")
    c = OUT / "sample_c_real_fixture.docx"
    c.write_bytes(src.read_bytes())
    return {"A": a, "B": b, "C": c}


def convert(docx: Path) -> tuple[float, Path, str]:
    t0 = time.perf_counter()
    proc = subprocess.run(
        ["soffice", "--headless", "--convert-to", "pdf", "--outdir", str(OUT), str(docx)],
        capture_output=True, text=True, timeout=120,
    )
    dt = time.perf_counter() - t0
    pdf = OUT / (docx.stem + ".pdf")
    return dt, pdf, (proc.stdout + proc.stderr).strip()


def pdf_pages(pdf: Path) -> int:
    out = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True).stdout
    for line in out.splitlines():
        if line.startswith("Pages:"):
            return int(line.split(":")[1].strip())
    return -1


def pdf_text(pdf: Path) -> str:
    return subprocess.run(["pdftotext", str(pdf), "-"], capture_output=True, text=True).stdout


def main() -> int:
    samples = build_samples()
    print("R-O2 spike：LibreOffice headless docx→PDF（C 路线）实证")
    print(f"soffice: {subprocess.run(['soffice', '--version'], capture_output=True, text=True).stdout.strip()}")

    fails = 0
    first = True
    for name, path in samples.items():
        dt, pdf, log = convert(path)
        if not pdf.exists():
            print(f"[{name}] ✗ 转换失败：{log[:200]}")
            fails += 1
            continue
        pages = pdf_pages(pdf)
        text = pdf_text(pdf)
        size = pdf.stat().st_size
        cold_tag = "（冷启动含进程开销）" if first else ""
        first = False
        print(f"[{name}] 转换 {dt*1000:.0f} ms{cold_tag} → {pdf.name}（{size} B，{pages} 页）")
        if name == "A":
            checks = ["结构样本标题段", "二级标题", "粗体", "斜体", "指标", "收入", "成本", "分页符之后的段落"]
            missing = [c for c in checks if c not in text]
            if missing:
                print(f"[A] ✗ 文本缺失：{missing}"); fails += 1
            else:
                print("[A] ✓ 全部关键串在场（标题/run 格式文本/表格单元格/分页后段落）")
            if pages != 2:
                print(f"[A] ✗ 页数期望 2（显式分页符 1 处）实测 {pages}"); fails += 1
            else:
                print("[A] ✓ 显式分页符生效（2 页）")
        elif name == "B":
            if pages != 6:
                print(f"[B] ✗ 页数期望 6（5 显式分页符）实测 {pages}"); fails += 1
            else:
                print("[B] ✓ 显式分页保真硬断言过（恰 6 页）")
            missing = [f"第 {i} 页内容" for i in range(1, 7) if f"第 {i} 页内容" not in text]
            if missing:
                print(f"[B] ✗ 各页文本缺失：{missing}"); fails += 1
        elif name == "C":
            ok = pdf.stat().st_size > 0 and pages >= 1
            print(f"[C] {'✓' if ok else '✗'} 真实夹具转换（import_sample.docx → {pages} 页，文本抽取 {len(text)} 字符）")
            if not ok:
                fails += 1

    # 热转换复跑（A 样本）——稳态单文档延迟口径
    dts = []
    for _ in range(3):
        dt, _, _ = convert(samples["A"])
        dts.append(dt)
    print(f"[热转换] A 样本复跑 3 次：{[f'{d*1000:.0f} ms' for d in dts]}（中位 {sorted(dts)[1]*1000:.0f} ms）")

    print("SPIKE " + ("PASS" if fails == 0 else f"FAIL（{fails} 项）"))
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
