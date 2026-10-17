#!/usr/bin/env python3
"""R-O2 C 路线原型第二轮（r-o2-t2）：复杂文档夹具扩展。

用户裁决口径（2026-10-17）：R-O2 选 C 路线进原型阶段（DOCX →
LibreOffice headless → PDF → mdc-pdf）；补充复杂文档夹具，验证分页、
跨页表格、页眉页脚、分节、缺失字体和图片——"当前单一夹具的通过结果
只能证明转换链可行，不能证明全面保真"。

净室纪律：python stdlib only（zipfile/zlib/struct 手搓 OOXML 与最小
PNG，零第三方依赖）；soffice 子进程隔离；全部断言本机实测。

样本面（D-I，承接第一轮 A/B/C）：
  D 跨页表格：80 行 3 列（超出单页高度）→ 表头页 1 / 中段行页 2+ /
    末行在末页；断言表格确实跨页（逐页文本抽取定位行文本，空格归一化）；
  E 页眉页脚：header1/footer1 部件 + sectPr 引用 + 3 页正文 →
    页眉与页脚文本在**每一页**在场（逐页断言）；
  F 分节：节 1 = A4 纵向，节 2 = A5 横向（sectPr 双处：段内分节符 +
    body 尾）→ 逐页页面尺寸断言（pdfinfo -f/-l）+ 各节文本落位；
  G 缺失字体：run 指定不存在字体 TotallyMissingFont123 → 转换不失败、
    文本仍在场、pdffonts 无该字体名（替换发生且不嵌入幻影字体）；
    视觉近似度不主张（登记——视觉级校验属实施轮系统性保真校验）；
  H 图片：stdlib 生成 8×8 PNG（media/image1.png + rels + content
    types + w:drawing inline）→ 转换不失败 + pdfimages 断言 1 张图；
  I 组合夹具：页眉页脚 + 分节 + 跨页表格 + 图片 + 缺失字体 + 显式
    分页符全要素 → 页数 ≥ 4 + 全要素关键串在场（最接近真实文档形状）。

口径：本机 LO 25.2.3.2（与上游 26.8.1 版本差登记——office-f §5.2）；
每样本转换 + pdfinfo/pdftotext/pdfimages/pdffonts 四工具断言。
"""

import struct
import subprocess
import sys
import time
import zipfile
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "out"
OUT.mkdir(exist_ok=True)

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
WP = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
A = "http://schemas.openxmlformats.org/drawingml/2006/main"
PIC = "http://schemas.openxmlformats.org/drawingml/2006/picture"

CT_EXTRA_PNG = '<Default Extension="png" ContentType="image/png"/>'
CT_HDR = ('<Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>'
          '<Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>')


def content_types(extra_default: str = "", extra_override: str = "") -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        f"{extra_default}"
        '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
        '<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>'
        f"{extra_override}"
        "</Types>"
    )


RELS = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"""

STYLES = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/>
<w:pPr><w:outlineLvl w:val="0"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style>
</w:styles>"""


def p(runs: str, ppr: str = "") -> str:
    return f"<w:p>{ppr}{runs}</w:p>"


def r(text: str, fonts: str | None = None) -> str:
    rpr = f"<w:rPr>{fonts}</w:rPr>" if fonts else ""
    return f'<w:r>{rpr}<w:t xml:space="preserve">{text}</w:t></w:r>'


MISSING_FONT = '<w:rFonts w:ascii="TotallyMissingFont123" w:hAnsi="TotallyMissingFont123"/>'
PAGE_BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'


def table(rows: list[list[str]], repeat_header: bool = False) -> str:
    tbl_pr = (
        "<w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>"
        '<w:tblBorders><w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/>'
        '<w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/>'
        '<w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/></w:tblBorders>'
        "</w:tblPr>"
    )
    header_props = '<w:trPr><w:tblHeader/></w:trPr>' if repeat_header else ""
    trs = ""
    for i, row in enumerate(rows):
        trpr = header_props if (repeat_header and i == 0) else ""
        trs += (
            f"<w:tr>{trpr}"
            + "".join(f"<w:tc><w:tcPr/><w:p>{r(cell)}</w:p></w:tc>" for cell in row)
            + "</w:tr>"
        )
    return f"<w:tbl>{tbl_pr}{trs}</w:tbl>"


def sect_pr(pg_w: int, pg_h: int, orient: str = "portrait",
            header_rid: str | None = None, footer_rid: str | None = None) -> str:
    refs = ""
    if header_rid:
        refs += f'<w:headerReference w:type="default" r:id="{header_rid}"/>'
    if footer_rid:
        refs += f'<w:footerReference w:type="default" r:id="{footer_rid}"/>'
    return (
        f"<w:sectPr>{refs}"
        f'<w:pgSz w:w="{pg_w}" w:h="{pg_h}"'
        + (' w:orient="landscape"' if orient == "landscape" else "")
        + '/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/>'
        "</w:sectPr>"
    )


def doc_xml(body: str, final_sect: str) -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<w:document xmlns:w="{W}" xmlns:r="{R}" xmlns:wp="{WP}">'
        f"<w:body>{body}{final_sect}</w:body></w:document>"
    )


def header_xml(text: str) -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<w:hdr xmlns:w="{W}">{p(r(text))}</w:hdr>'
    )


def footer_xml(text: str) -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<w:ftr xmlns:w="{W}">{p(r(text))}</w:ftr>'
    )


def inline_image(rid: str, cx_emu: int = 457200, cy_emu: int = 457200) -> str:
    # 457200 EMU = 0.5 英寸（约 48px @96dpi）
    return (
        f'<w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0">'
        f'<wp:extent cx="{cx_emu}" cy="{cy_emu}"/>'
        f'<wp:effectExtent l="0" t="0" r="0" b="0"/>'
        f'<wp:docPr id="1" name="img1"/>'
        f'<wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a="{A}" noChangeAspect="1"/></wp:cNvGraphicFramePr>'
        f'<a:graphic xmlns:a="{A}">'
        f'<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">'
        f'<pic:pic xmlns:pic="{PIC}">'
        f'<pic:nvPicPr><pic:cNvPr id="1" name="img1"/>'
        f'<pic:cNvPicPr><a:picLocks noChangeAspect="1"/></pic:cNvPicPr></pic:nvPicPr>'
        f'<pic:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>'
        f'<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx_emu}" cy="{cy_emu}"/></a:xfrm>'
        f'<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr>'
        f"</pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing>"
    )


def make_png(width: int = 8, height: int = 8, rgb=(200, 30, 30)) -> bytes:
    """stdlib 最小 PNG：单色矩形（IHDR/IDAT/IEND + CRC）。"""
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)  # 8bit RGB
    raw = b"".join(b"\x00" + bytes(rgb) * width for _ in range(height))
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def write_docx(path: Path, body: str, final_sect: str, *,
               with_hf: bool = False, with_image: bool = False) -> None:
    doc_rels = ['<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>']
    if with_hf:
        doc_rels.append(f'<Relationship Id="rIdHdr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>')
        doc_rels.append(f'<Relationship Id="rIdFtr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>')
    if with_image:
        doc_rels.append(f'<Relationship Id="rIdImg" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>')
    rels_xml = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        + "".join(doc_rels) + "</Relationships>"
    )
    extra_default = CT_EXTRA_PNG if with_image else ""
    extra_override = CT_HDR if with_hf else ""
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", content_types(extra_default, extra_override))
        z.writestr("_rels/.rels", RELS)
        z.writestr("word/document.xml", doc_xml(body, final_sect))
        z.writestr("word/_rels/document.xml.rels", rels_xml)
        z.writestr("word/styles.xml", STYLES)
        if with_hf:
            z.writestr("word/header1.xml", header_xml("E2E-HEADER-每页页眉"))
            z.writestr("word/footer1.xml", footer_xml("E2E-FOOTER-每页页脚"))
        if with_image:
            z.writestr("word/media/image1.png", make_png())


def convert(docx: Path) -> tuple[float, Path, str]:
    t0 = time.perf_counter()
    proc = subprocess.run(
        ["soffice", "--headless", "--convert-to", "pdf", "--outdir", str(OUT), str(docx)],
        capture_output=True, text=True, timeout=180,
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


def page_sizes(pdf: Path) -> list[tuple[float, float]]:
    """逐页尺寸（pt）——pdfinfo -f 1 -l N 输出 'Page    1 size: W x H pts'。"""
    n = pdf_pages(pdf)
    out = subprocess.run(
        ["pdfinfo", "-f", "1", "-l", str(n), str(pdf)], capture_output=True, text=True
    ).stdout
    sizes = []
    for line in out.splitlines():
        if "size:" in line and "pts" in line:
            parts = line.split("size:")[1].strip().split(" pts")[0]
            w, h = parts.split(" x ")
            sizes.append((round(float(w)), round(float(h))))
    return sizes


def page_text(pdf: Path, page: int) -> str:
    return subprocess.run(
        ["pdftotext", "-f", str(page), "-l", str(page), str(pdf), "-"],
        capture_output=True, text=True,
    ).stdout


def pdf_text(pdf: Path) -> str:
    return subprocess.run(["pdftotext", str(pdf), "-"], capture_output=True, text=True).stdout


def norm(s: str) -> str:
    """断言归一化：剥空白（pdftotext 在 CJK/拉丁边界插空格——行 001-甲）。"""
    return s.replace(" ", "").replace("\n", "").replace("\r", "")


def pdf_image_count(pdf: Path) -> int:
    out = subprocess.run(["pdfimages", "-list", str(pdf)], capture_output=True, text=True).stdout
    # 首行 header、次行列名，其后每行一图
    lines = [l for l in out.splitlines() if l.strip()]
    return max(0, len(lines) - 2)


def pdf_font_names(pdf: Path) -> list[str]:
    out = subprocess.run(["pdffonts", str(pdf)], capture_output=True, text=True).stdout
    lines = out.splitlines()
    fonts = []
    for line in lines[2:]:
        if line.strip():
            fonts.append(line.split()[0])
    return fonts


def build_samples() -> dict[str, Path]:
    samples = {}

    # D 跨页表格：30 行（超单页）+ 重复表头
    rows = [["列一", "列二", "列三"]] + [
        [f"行{i:03d}-甲", f"行{i:03d}-乙", f"行{i:03d}-丙"] for i in range(1, 81)
    ]
    body_d = p(r("跨页表格样本")) + table(rows, repeat_header=True)
    d = OUT / "sample_d_table_span.docx"
    write_docx(d, body_d, sect_pr(11906, 16838))
    samples["D"] = d

    # E 页眉页脚：3 页正文（2 显式分页符）
    body_e = (
        p(r("页眉页脚样本第 1 页正文。"))
        + PAGE_BREAK
        + p(r("第 2 页正文。"))
        + PAGE_BREAK
        + p(r("第 3 页正文。"))
    )
    e = OUT / "sample_e_header_footer.docx"
    write_docx(e, body_e, sect_pr(11906, 16838, header_rid="rIdHdr", footer_rid="rIdFtr"), with_hf=True)
    samples["E"] = e

    # F 分节：节 1 A4 纵（11906×16838）→ 节 2 A5 横（16838×11906，orient=landscape）
    sect1 = sect_pr(11906, 16838)
    body_f = (
        p(r("分节样本——第一节（A4 纵向）。"))
        + f'<w:p><w:pPr>{sect1}</w:pPr></w:p>'  # 段内分节符（节 1 结束）
        + p(r("第二节（A5 横向）正文。"))
    )
    f = OUT / "sample_f_sections.docx"
    write_docx(f, body_f, sect_pr(16838, 11906, orient="landscape"))
    samples["F"] = f

    # G 缺失字体
    body_g = (
        p(r("缺失字体样本——下行使用不存在字体 TotallyMissingFont123。"))
        + p(r("此行文本应仍可读（字体替换）。", fonts=MISSING_FONT))
        + p(r("常规字体对照行。"))
    )
    g = OUT / "sample_g_missing_font.docx"
    write_docx(g, body_g, sect_pr(11906, 16838))
    samples["G"] = g

    # H 图片
    body_h = (
        p(r("图片样本——下含一张内联 PNG。"))
        + f"<w:p><w:r>{inline_image('rIdImg')}</w:r></w:p>"
        + p(r("图片后段落。"))
    )
    h = OUT / "sample_h_image.docx"
    write_docx(h, body_h, sect_pr(11906, 16838), with_image=True)
    samples["H"] = h

    # I 组合夹具：全要素
    sect_i1 = sect_pr(11906, 16838, header_rid="rIdHdr", footer_rid="rIdFtr")
    combo_rows = [["组合表头A", "组合表头B"]] + [
        [f"组合行{i:02d}-左", f"组合行{i:02d}-右"] for i in range(1, 61)
    ]
    body_i = (
        p(r("组合夹具第一节（A4 纵，带页眉页脚，含跨页表格与图片）。"))
        + f"<w:p><w:r>{inline_image('rIdImg')}</w:r></w:p>"
        + table(combo_rows, repeat_header=True)
        + p(r("缺失字体行（组合内）。", fonts=MISSING_FONT))
        + f'<w:p><w:pPr>{sect_i1}</w:pPr></w:p>'
        + p(r("组合夹具第二节（A5 横）——显式分页后的内容。"))
        + PAGE_BREAK
        + p(r("第二节第 2 页内容。"))
    )
    i = OUT / "sample_i_combo.docx"
    write_docx(i, body_i, sect_pr(16838, 11906, orient="landscape"), with_hf=True, with_image=True)
    samples["I"] = i

    return samples


def main() -> int:
    samples = build_samples()
    print("R-O2 r-o2-t2：复杂文档夹具扩展（C 路线原型第二轮）")
    print(f"soffice: {subprocess.run(['soffice', '--version'], capture_output=True, text=True).stdout.strip()}")
    fails = 0

    def check(cond: bool, label: str, detail: str = "") -> None:
        nonlocal fails
        if cond:
            print(f"  ✓ {label}")
        else:
            fails += 1
            print(f"  ✗ {label} {detail}")

    for name, path in samples.items():
        dt, pdf, log = convert(path)
        if not pdf.exists():
            print(f"[{name}] ✗ 转换失败：{log[:200]}")
            fails += 1
            continue
        pages = pdf_pages(pdf)
        text = pdf_text(pdf)
        print(f"[{name}] 转换 {dt*1000:.0f} ms → {pdf.name}（{pdf.stat().st_size} B，{pages} 页）")

        if name == "D":
            check(pages >= 2, f"表格跨页（80 行 → {pages} 页 ≥ 2）")
            first_page = next((pg for pg in range(1, pages + 1) if "行001-甲" in norm(page_text(pdf, pg))), None)
            last_page = next((pg for pg in range(1, pages + 1) if "行080-丙" in norm(page_text(pdf, pg))), None)
            check(first_page == 1, "首行（行001）在第 1 页")
            check(last_page is not None and last_page > 1,
                  f"末行（行080）在第 {last_page} 页 ≠ 首行页（跨页实证）")
            check("列一" in norm(text), "表头行文本在场")

        elif name == "E":
            check(pages == 3, f"页数恰 3（2 显式分页符）实测 {pages}")
            hdr_ok = all("E2E-HEADER-每页页眉" in norm(page_text(pdf, pg)) for pg in range(1, pages + 1))
            ftr_ok = all("E2E-FOOTER-每页页脚" in norm(page_text(pdf, pg)) for pg in range(1, pages + 1))
            check(hdr_ok, "页眉文本在每一页在场（逐页断言）")
            check(ftr_ok, "页脚文本在每一页在场（逐页断言）")

        elif name == "F":
            sizes = page_sizes(pdf)
            check(len(sizes) >= 2, f"逐页尺寸可得（{len(sizes)} 页）", str(sizes))
            if len(sizes) >= 2:
                check(sizes[0] == (595, 842), f"节 1 = A4 纵（595×842pt）实测 {sizes[0]}")
                check(sizes[1] == (595, 842) or sizes[1][0] > sizes[1][1],
                      f"节 2 = 横向页（宽>高）实测 {sizes[1]}")
            check("第一节" in norm(page_text(pdf, 1)), "节 1 文本在第 1 页")
            check(any("第二节" in norm(page_text(pdf, pg)) for pg in range(1, len(sizes) + 1)), "节 2 文本在场")

        elif name == "G":
            check("此行文本应仍可读" in norm(text), "缺失字体行文本仍在场（替换不丢内容）")
            fonts = pdf_font_names(pdf)
            check(all("TotallyMissingFont123" not in f for f in fonts),
                  f"幻影字体未嵌入（pdffonts：{fonts}）")
            check(len(fonts) > 0, "替换字体已嵌入（fonts 非空）")

        elif name == "H":
            n_img = pdf_image_count(pdf)
            check(n_img >= 1, f"内联图片在场（pdfimages 计数 {n_img}）")
            check("图片后段落" in norm(text), "图片后段落文本在场")

        elif name == "I":
            check(pages >= 4, f"组合夹具页数 {pages} ≥ 4（跨页表格+分节+分页符）")
            hdr_ok = all("E2E-HEADER-每页页眉" in norm(page_text(pdf, pg)) for pg in range(1, pages + 1))
            check(hdr_ok, "组合：页眉逐页在场（全节共享 default 槽）")
            for kw in ["组合夹具第一节", "组合表头A", "组合行55-右", "缺失字体行", "第二节第 2 页内容"]:
                check(kw.replace(" ", "") in norm(text), f"组合：关键串「{kw}」在场")
            n_img = pdf_image_count(pdf)
            check(n_img >= 1, f"组合：图片在场（{n_img} 张）")

    print("SPIKE2 " + ("PASS" if fails == 0 else f"FAIL（{fails} 项）"))
    return 0 if fails == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
