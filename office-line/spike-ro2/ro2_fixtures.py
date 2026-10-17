#!/usr/bin/env python3
"""r-o2-t3 回归套件的确定性 DOCX 夹具构建器（stdlib only）。

设计口径：
- 手搓 OOXML（承 lo_convert_spike.py / lo_convert_spike2.py 的构建器，
  参数化 + 回归化适配；原 spike 脚本保持原样不动，本模块为抽取副本）；
- **确定性**：夹具内容零时间戳/零随机；zip 条目统一 date_time=
  (1980,1,1,0,0,0)，同平台下重复构建字节级一致（套件内置 sha256 复验）；
- 畸形输入构建器：garbage / truncated-zip / bad-xml / missing-content-types /
  drawing-at-paragraph-level（登记的静默丢弃钉定样本）。
"""

from __future__ import annotations

import hashlib
import struct
import zipfile
import zlib
from pathlib import Path

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
WP = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
A = "http://schemas.openxmlformats.org/drawingml/2006/main"
PIC = "http://schemas.openxmlformats.org/drawingml/2006/picture"

MISSING_FONT = '<w:rFonts w:ascii="TotallyMissingFont123" w:hAnsi="TotallyMissingFont123"/>'
PAGE_BREAK = '<w:p><w:r><w:br w:type="page"/></w:r></w:p>'

# ── 包级部件模板 ────────────────────────────────────────────────────────────

CT_PNG = '<Default Extension="png" ContentType="image/png"/>'
CT_HDR = (
    '<Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>'
    '<Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>'
)

RELS = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
    "</Relationships>"
)

STYLES = (
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    '<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
    '<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/>'
    "<w:pPr><w:outlineLvl w:val=\"0\"/></w:pPr>"
    '<w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:style>'
    '<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/>'
    "<w:pPr><w:outlineLvl w:val=\"1\"/></w:pPr>"
    '<w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:style>'
    "</w:styles>"
)

# ── OOXML 片段构建 ──────────────────────────────────────────────────────────


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


def p(runs: str, ppr: str = "") -> str:
    return f"<w:p>{ppr}{runs}</w:p>"


def r(text: str, fonts: str | None = None, bold: bool = False, italic: bool = False) -> str:
    props = ""
    if fonts:
        props += fonts
    if bold:
        props += "<w:b/>"
    if italic:
        props += "<w:i/>"
    rpr = f"<w:rPr>{props}</w:rPr>" if props else ""
    return f'<w:r>{rpr}<w:t xml:space="preserve">{text}</w:t></w:r>'


def table(rows: list[list[str]], repeat_header: bool = False) -> str:
    tbl_pr = (
        "<w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>"
        '<w:tblBorders><w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/>'
        '<w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/>'
        '<w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/></w:tblBorders>'
        "</w:tblPr>"
    )
    trs = ""
    for i, row in enumerate(rows):
        trpr = '<w:trPr><w:tblHeader/></w:trPr>' if (repeat_header and i == 0) else ""
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


def hf_xml(tag: str, text: str) -> str:
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<w:{tag} xmlns:w="{W}">{p(r(text))}</w:{tag}>'
    )


def inline_image(rid: str, cx_emu: int = 457200, cy_emu: int = 457200) -> str:
    # 457200 EMU = 0.5 英寸
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


# ── zip 写入（确定性：固定条目时间戳） ──────────────────────────────────────

_ZIP_EPOCH = (1980, 1, 1, 0, 0, 0)  # zip 时间戳下限——固定值保字节级确定性


def _zinfo(name: str) -> zipfile.ZipInfo:
    zi = zipfile.ZipInfo(name, date_time=_ZIP_EPOCH)
    zi.compress_type = zipfile.ZIP_DEFLATED
    return zi


def write_zip(path: Path, entries: dict[str, str | bytes]) -> None:
    with zipfile.ZipFile(path, "w") as z:
        for name, data in entries.items():
            if isinstance(data, str):
                data = data.encode("utf-8")
            z.writestr(_zinfo(name), data)


def write_docx(path: Path, body: str, final_sect: str, *,
               header_text: str | None = None, footer_text: str | None = None,
               with_image: bool = False) -> None:
    """标准最小 docx 包（含可选页眉页脚 / 图片接线）。"""
    doc_rels = [
        '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
    ]
    if header_text is not None:
        doc_rels.append('<Relationship Id="rIdHdr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>')
    if footer_text is not None:
        doc_rels.append('<Relationship Id="rIdFtr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>')
    if with_image:
        doc_rels.append('<Relationship Id="rIdImg" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>')
    rels_xml = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        + "".join(doc_rels) + "</Relationships>"
    )
    entries: dict[str, str | bytes] = {
        "[Content_Types].xml": content_types(
            CT_PNG if with_image else "",
            CT_HDR if (header_text is not None or footer_text is not None) else "",
        ),
        "_rels/.rels": RELS,
        "word/document.xml": doc_xml(body, final_sect),
        "word/_rels/document.xml.rels": rels_xml,
        "word/styles.xml": STYLES,
    }
    if header_text is not None:
        entries["word/header1.xml"] = hf_xml("hdr", header_text)
    if footer_text is not None:
        entries["word/footer1.xml"] = hf_xml("ftr", footer_text)
    if with_image:
        entries["word/media/image1.png"] = make_png()
    write_zip(path, entries)


# ── 有效夹具（回归用例 9 件） ───────────────────────────────────────────────

HDR_TEXT = "REG-HDR-每页页眉锚"
FTR_TEXT = "REG-FTR-每页页脚锚"


def build_basic_structure(path: Path) -> None:
    """用例 basic-structure（承第一轮 A：标题 + 粗斜 run + 2×3 表 + 显式分页）。"""
    body = (
        p(r("结构样本标题段", bold=True), '<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>')
        + p(r("二级标题", bold=True), '<w:pPr><w:pStyle w:val="Heading2"/></w:pPr>')
        + p(r("普通段落文本，含") + r("粗体", bold=True) + r("、") + r("斜体", italic=True) + r("与混排收尾。"))
        + table([
            ["指标", "Q1", "Q2"],
            ["收入", "100", "180"],
            ["成本", "60", "70"],
        ])
        + PAGE_BREAK
        + p(r("分页符之后的段落——应出现在第 2 页。"))
    )
    write_docx(path, body, sect_pr(11906, 16838))


def build_explicit_page_breaks(path: Path) -> None:
    """用例 explicit-page-breaks（承 B：5 个显式分页符 → 恰 6 页）。"""
    body = p(r("第 1 页内容。"))
    for i in range(2, 7):
        body += PAGE_BREAK + p(r(f"第 {i} 页内容。"))
    write_docx(path, body, sect_pr(11906, 16838))


def build_table_spanning_pages(path: Path) -> None:
    """用例 table-spanning-pages（承 D：80 行表 + tblHeader 重复表头）。"""
    rows = [["列一", "列二", "列三"]] + [
        [f"行{i:03d}-甲", f"行{i:03d}-乙", f"行{i:03d}-丙"] for i in range(1, 81)
    ]
    body = p(r("跨页表格样本（80 行 + 重复表头）。")) + table(rows, repeat_header=True)
    write_docx(path, body, sect_pr(11906, 16838))


def build_headers_footers(path: Path) -> None:
    """用例 headers-footers（承 E：header1/footer1 + sectPr 引用，3 页）。"""
    body = (
        p(r("页眉页脚样本第 1 页正文。"))
        + PAGE_BREAK
        + p(r("第 2 页正文。"))
        + PAGE_BREAK
        + p(r("第 3 页正文。"))
    )
    write_docx(path, body, sect_pr(11906, 16838, header_rid="rIdHdr", footer_rid="rIdFtr"),
               header_text=HDR_TEXT, footer_text=FTR_TEXT)


def build_section_boundaries(path: Path) -> None:
    """用例 section-boundaries（承 F：节 1 A4 纵 11906×16838 → 节 2 横向
    16838×11906，即 595×842pt → 842×595pt）。

    命名注记：节 2 尺寸为 A4 旋转对（842×595pt），承 spike2 的「A5 横」
    措辞属沿用误差——本套件按实测 pt 断言（595×842 → 842×595），
    命名偏差在 REGRESSION.md 登记。
    """
    sect1 = sect_pr(11906, 16838)
    body = (
        p(r("分节样本——第一节正文（纵向页 595×842pt）。"))
        + f"<w:p><w:pPr>{sect1}</w:pPr></w:p>"  # 段内分节符（节 1 结束）
        + p(r("第二节正文（横向页 842×595pt）。"))
    )
    write_docx(path, body, sect_pr(16838, 11906, orient="landscape"))


def build_missing_font(path: Path) -> None:
    """用例 missing-font（承 G：rFonts 指定不存在字体 TotallyMissingFont123）。"""
    body = (
        p(r("缺失字体样本——下一行使用不存在字体 TotallyMissingFont123。"))
        + p(r("此行文本应仍可读（字体替换检验）。", fonts=MISSING_FONT))
        + p(r("常规字体对照行。"))
    )
    write_docx(path, body, sect_pr(11906, 16838))


def build_embedded_image(path: Path) -> None:
    """用例 embedded-image（承 H：8×8 stdlib PNG + run 级 w:drawing inline）。"""
    body = (
        p(r("图片样本——下含一张内联 PNG。"))
        + f"<w:p><w:r>{inline_image('rIdImg')}</w:r></w:p>"
        + p(r("图片后段落文本——应仍可读。"))
    )
    write_docx(path, body, sect_pr(11906, 16838), with_image=True)


def build_composite(path: Path) -> None:
    """用例 composite（承 I：页眉页脚 + 分节 + 跨页表格 + 图片 + 缺失字体
    + 显式分页全要素）。"""
    sect1 = sect_pr(11906, 16838, header_rid="rIdHdr", footer_rid="rIdFtr")
    rows = [["组合表头A", "组合表头B"]] + [
        [f"组合行{i:02d}-左", f"组合行{i:02d}-右"] for i in range(1, 61)
    ]
    body = (
        p(r("组合夹具第一节（A4 纵，带页眉页脚，含图片与跨页表格）。"))
        + f"<w:p><w:r>{inline_image('rIdImg')}</w:r></w:p>"
        + table(rows, repeat_header=True)
        + p(r("此行文本应仍可读（字体替换检验）。", fonts=MISSING_FONT))
        + f"<w:p><w:pPr>{sect1}</w:pPr></w:p>"
        + p(r("组合夹具第二节（横向页）——分节后内容。"))
        + PAGE_BREAK
        + p(r("第二节第 2 页内容。"))
    )
    write_docx(path, body, sect_pr(16838, 11906, orient="landscape"),
               header_text=HDR_TEXT, footer_text=FTR_TEXT, with_image=True)


def copy_real_fixture(path: Path, src: Path) -> None:
    """用例 real-fixture：已提交的 mdcore import_sample.docx 副本（二进制原样）。"""
    data = src.read_bytes()
    path.write_bytes(data)


# ── 畸形输入夹具 ───────────────────────────────────────────────────────────


def pseudo_random_bytes(n: int) -> bytes:
    """确定性伪随机字节（sha256 计数器流）——非 zip、无 PK 头。"""
    out = bytearray()
    i = 0
    while len(out) < n:
        out += hashlib.sha256(b"ro2-regress-garbage-" + i.to_bytes(4, "big")).digest()
        i += 1
    return bytes(out[:n])


def build_garbage_bytes(path: Path) -> None:
    """畸形：纯垃圾字节（4096B，非 zip）。"""
    path.write_bytes(pseudo_random_bytes(4096))


def build_truncated_zip(path: Path, src_bytes: bytes) -> None:
    """畸形：有效夹具的前 60% 字节（zip 尾/中央目录缺失）。"""
    keep = max(1024, (len(src_bytes) * 60) // 100)
    path.write_bytes(src_bytes[:keep])


def build_bad_xml(path: Path) -> None:
    """畸形：包结构有效但 document.xml 含未闭合标签。"""
    bad_doc = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        f'<w:document xmlns:w="{W}"><w:body>'
        "<w:p><w:r><w:t>未闭合标签样本"
    )
    write_zip(path, {
        "[Content_Types].xml": content_types(),
        "_rels/.rels": RELS,
        "word/document.xml": bad_doc,
        "word/_rels/document.xml.rels": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
            "</Relationships>"
        ),
        "word/styles.xml": STYLES,
    })


def build_missing_content_types(path: Path) -> None:
    """畸形：有效 zip 但缺 [Content_Types].xml 部件。"""
    write_zip(path, {
        "_rels/.rels": RELS,
        "word/document.xml": doc_xml(p(r("缺 Content_Types 样本。")), sect_pr(11906, 16838)),
        "word/_rels/document.xml.rels": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
            "</Relationships>"
        ),
        "word/styles.xml": STYLES,
    })


def build_drawing_at_paragraph_level(path: Path) -> None:
    """畸形（行为钉定样本）：w:drawing 直接置于 w:p 下（缺 w:r 包装）。

    非法 OOXML——LibreOffice（25.2.3.2 实测）**静默丢弃图片**：退出码 0、
    PDF 正常产出、文本完好、pdfimages 计数 0。这是 r-o2-t2 登记的
    静默丢弃面（office-f §8.2 发现①）——本套件将其钉定为回归锚：若未来
    LO 版本改变该行为（报错 / 渲染图片），本用例失败并提示更新钉定。
    """
    body = (
        p(r("段级 drawing 样本——图片置于 w:p 直接子级（非法 OOXML）。"))
        + f"<w:p>{inline_image('rIdImg')}</w:p>"  # 缺 w:r 包装——静默丢弃面
        + p(r("图片后段落文本——文本应完好。"))
    )
    write_docx(path, body, sect_pr(11906, 16838), with_image=True)
