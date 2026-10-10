#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""acrobat-audit 核心可复现管线: AcroPro.msi 数据库级取证
依赖: olefile (pip install olefile), binutils strings
用法: python3 msi_audit.py [--msi path] [--outdir dir] [--phase all|extract|tables|behavior|strings]
输出: 流清单/表JSON/行为汇总/PE字符串挖掘, 与 evidence/ 下结构一致
来源: 对 2026-10-09 会话中逆向工程的整合(见 WORKLOG.md)
"""
import argparse
import hashlib
import json
import os
import re
import struct
import subprocess
import sys
from collections import Counter, defaultdict

try:
    import olefile
except ImportError:
    sys.exit("需要 olefile: pip install olefile")

HERE = os.path.dirname(os.path.abspath(__file__))


# ---------- 第1阶段: OLE流转储与分类 ----------
def phase_extract(msi, outdir):
    sdir = os.path.join(outdir, "msi_streams")
    os.makedirs(sdir, exist_ok=True)
    ole = olefile.OleFileIO(msi)
    manifest = []
    idx = 0
    for e in ole.listdir(streams=True, storages=False):
        path = "/".join(e)
        size = ole.get_size(path) if ole.exists(path) else -1
        if size <= 0:
            continue
        data = ole.openstream(path).read()
        idx += 1
        head = data[:16]
        if head[:2] == b"MZ":
            kind = "PE"
        elif head[:2] == b"BM":
            kind = "BMP"
        elif head[:3] == b"\xff\xd8\xff":
            kind = "JPEG"
        elif head[:4] == b"MSCF":
            kind = "CAB"
        else:
            sample = data[:4096]
            pr = sum(1 for b in sample if 32 <= b < 127 or b in (9, 10, 13))
            ratio = pr / max(1, len(sample))
            kind = f"ASCII({ratio:.0%})" if ratio > 0.85 else "bin"
        sha = hashlib.sha256(data).hexdigest()[:16]
        fname = f"{idx:03d}_{size}.bin"
        with open(os.path.join(sdir, fname), "wb") as f:
            f.write(data)
        manifest.append({"idx": idx, "ole_name": path.encode("unicode_escape").decode()[:48],
                         "size": size, "class": kind, "sha256_16": sha, "file": fname})
    json.dump(manifest, open(os.path.join(outdir, "msi_manifest.json"), "w"),
              ensure_ascii=False, indent=1)
    cnt = Counter(m["class"].split("(")[0] for m in manifest)
    print(f"[extract] {len(manifest)} 流 -> {sdir}; 分类: {dict(cnt)}")
    # 定位 StringData(最大ASCII流) 与 StringPool(次大, 与其共享前缀族)
    ascii_streams = [m for m in manifest if m["class"].startswith("ASCII")]
    sd_file = max(ascii_streams, key=lambda m: m["size"])["file"]
    print(f"[extract] StringData = {sd_file}")
    return sdir, sd_file


# ---------- 第2阶段: 字符串池解码 + 表重建 ----------
TABLE_SPEC = [
    ("_Tables", "069_156.bin", 78, 1, ["TableName"]),
    ("Property", "128_1252.bin", 313, 2, ["Property", "Value"]),
    ("CustomAction", "133_2688.bin", 336, 4,
     ["Action", "Type", "Source", "Target"]),
    ("InstallExecuteSequence", "105_1512.bin", 252, 3,
     ["Action", "Condition", "Sequence"]),
    ("ServiceControl", "091_84.bin", 7, 6,
     ["ServiceControl", "Name", "Event", "Argument", "Wait", "Component_"]),
    ("Media", "085_72.bin", 6, 6,
     ["DiskId", "LastSequence", "DiskPrompt", "Cabinet", "VolumeLabel", "Source"]),
    ("Directory", "098_4836.bin", 806, 3,
     ["Directory", "Directory_Parent", "DefaultDir"]),
    ("Component", "109_27696.bin", 2308, 6,
     ["Component", "ComponentId", "Directory_", "Attributes", "KeyPath", "Condition"]),
    ("Registry", "088_39672.bin", 3306, 6,
     ["Registry", "Root", "Key", "Name", "Value", "Component_"]),
    ("File", "131_84480.bin", 5280, 8,
     ["File", "Component_", "FileName", "FileSize", "Version", "Language",
      "Attributes", "Sequence"]),
    ("Binary", "097_156.bin", 39, 2, ["Name", "Data"]),
]


def load_strings(sdir, sd_file):
    sd = open(os.path.join(sdir, sd_file), "rb").read()
    # StringPool = 次大且名字前缀族相邻的bin流; 经验定位: 117716字节
    pool_file = None
    for fn in os.listdir(sdir):
        if fn.endswith("_117716.bin"):
            pool_file = fn
            break
    if pool_file is None:
        sys.exit("未找到 StringPool(117716B) 流")
    pool = open(os.path.join(sdir, pool_file), "rb").read()
    n = len(pool) // 4
    strings, off = [], 0
    for i in range(n):
        ln, ref = struct.unpack_from("<HH", pool, i * 4)
        strings.append(sd[off:off + ln].decode("latin-1"))
        off += ln
    assert off == len(sd), f"字符串池长度不吻合: {off} != {len(sd)}"
    print(f"[tables] 字符串池: {n} 条, {len(sd):,} 字节")
    return strings


def decode_cell(v, strings):
    n = len(strings)
    if v == 0:
        return None
    if v & 0x8000:
        return v & 0x7FFF
    return strings[v] if v < n else f"#{v}"


def phase_tables(sdir, sd_file, outdir):
    strings = load_strings(sdir, sd_file)
    tdir = os.path.join(outdir, "msi_tables")
    os.makedirs(tdir, exist_ok=True)
    ok = {}
    for name, fn, rows, cols, colnames in TABLE_SPEC:
        path = os.path.join(sdir, fn)
        if not os.path.exists(path):
            print(f"[tables] 跳过 {name}: 缺 {fn}")
            continue
        d = open(path, "rb").read()
        cells = struct.unpack(f"<{len(d)//2}H", d)
        if len(cells) != rows * cols:
            print(f"[tables] 跳过 {name}: {len(cells)} != {rows}x{cols}")
            continue
        table = []
        for r in range(rows):
            table.append({(colnames[c] if c < len(colnames) else f"col{c+1}"):
                          decode_cell(cells[c * rows + r], strings)
                          for c in range(cols)})
        json.dump(table, open(os.path.join(tdir, f"{name}.json"), "w"),
                  ensure_ascii=False, indent=1)
        ok[name] = {"stream": fn, "rows": rows, "cols": cols}
    print(f"[tables] 重建 {len(ok)} 张表 -> {tdir}")
    return tdir, strings


# ---------- 第3阶段: 行为汇总 ----------
def phase_behavior(sdir, tdir, strings, outdir):
    def load(n):
        p = os.path.join(tdir, f"{n}.json")
        return json.load(open(p)) if os.path.exists(p) else []

    ca = load("CustomAction")
    ies = load("InstallExecuteSequence")
    reg = load("Registry")
    binary = [r["Name"] for r in load("Binary") if r.get("Name")]

    # ServiceInstall #092 手工cells
    si_path = os.path.join(sdir, "092_32.bin")
    si_cells = []
    if os.path.exists(si_path):
        d = open(si_path, "rb").read()
        si_cells = [decode_cell(v, strings) for v in struct.unpack(f"<{len(d)//2}H", d)]

    # Registry 自启动/服务/ARM/AGS 过滤
    interesting = []
    for r in reg:
        blob = "||".join(str(r.get(k) or "") for k in
                         ("Key", "Name", "Value")).lower()
        if any(s in blob for s in (
                "\\run", "runonce", "adobegc", "genuine", "armsvc",
                "adobe arm", "update service", "crashreporter",
                "collabsync", "acrotray", "ipcbroker", "ccxprocess")):
            interesting.append(r)

    type_dist = Counter(str(r.get("Type")) for r in ca)
    summary = {
        "serviceinstall_cells": si_cells,
        "servicecontrol": load("ServiceControl"),
        "custom_action_count": len(ca),
        "ca_type_distribution": dict(type_dist),
        "ca_source_counts": dict(Counter(
            str(r.get("Source")) for r in ca
            if isinstance(r.get("Source"), str))),
        "ies_action_count": len(ies),
        "registry_rows": len(reg),
        "registry_autostart_hits": interesting,
        "binary_keys": binary,
    }
    json.dump(summary, open(os.path.join(outdir, "_behavior_summary.json"), "w"),
              ensure_ascii=False, indent=1)
    print(f"[behavior] CA={len(ca)} IES={len(ies)} Registry={len(reg)} "
          f"Binary={len(binary)} 自启动命中={len(interesting)}")


# ---------- 第4阶段: PE身份与字符串挖掘 ----------
VER_KEYS = ["CompanyName", "FileDescription", "InternalName", "OriginalFileName",
            "OriginalFilename", "ProductName", "ProductVersion", "FileVersion"]
URL_RE = re.compile(r"(?:https?://|ftp://)[A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%-]{4,}")
DOM_RE = re.compile(r"\b(?:[A-Za-z0-9-]+\.)+(?:adobe|adobegenuine|demdex)\.(?:com|io)\b",
                    re.IGNORECASE)
SVC_RE = re.compile(r"(?i)(schtasks|CreateService|OpenSCManager|AdobeARMservice|armsvc"
                    r"|AdobeGCClient|AdobeGCInvoker|Genuine)[^\x00]{0,80}")


def phase_strings(sdir, outdir):
    result = {}
    for fn in sorted(os.listdir(sdir)):
        p = os.path.join(sdir, fn)
        with open(p, "rb") as f:
            if f.read(2) != b"MZ":
                continue
        vi = {}
        try:
            txt = subprocess.run(["strings", "-el", "-n", "4", p],
                                 capture_output=True, text=True, timeout=60).stdout
        except Exception:
            txt = ""
        lines = [l.strip() for l in txt.splitlines() if l.strip()]
        for i, l in enumerate(lines):
            if l in VER_KEYS:
                for j in range(i + 1, min(i + 4, len(lines))):
                    if lines[j] not in VER_KEYS and len(lines[j]) <= 120:
                        vi.setdefault(l, lines[j])
                        break
        urls, doms, svcs = set(), set(), set()
        for args in (["strings", "-a", "-n", "6", p],):
            try:
                t2 = subprocess.run(args, capture_output=True, text=True,
                                    timeout=60).stdout
            except Exception:
                continue
            urls.update(URL_RE.findall(t2))
            for m in DOM_RE.finditer(t2):
                doms.add(m.group(0))
            for m in SVC_RE.finditer(t2):
                svcs.add(m.group(0).strip()[:110])
        result[fn] = {"identity": vi.get("InternalName",
                                         vi.get("OriginalFileName", "?")),
                      "version_info": vi, "urls": sorted(urls),
                      "domains": sorted(doms), "svc": sorted(svcs)}
    json.dump(result, open(os.path.join(outdir, "msi_strings_mining.json"), "w"),
              ensure_ascii=False, indent=1)
    print(f"[strings] 挖掘 {len(result)} 个PE流")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--msi", default=os.path.join(HERE, "..", "source", "AcroPro.msi"))
    ap.add_argument("--outdir", default=os.path.join(HERE, "..", "evidence"))
    ap.add_argument("--phase", default="all",
                    choices=["all", "extract", "tables", "behavior", "strings"])
    a = ap.parse_args()
    msi = os.path.abspath(a.msi)
    outdir = os.path.abspath(a.outdir)
    os.makedirs(outdir, exist_ok=True)
    sha = hashlib.sha256(open(msi, "rb").read()).hexdigest()
    print(f"[init] MSI SHA-256 = {sha}")
    sdir, sd_file = phase_extract(msi, outdir)
    if a.phase in ("all", "tables"):
        tdir, strings = phase_tables(sdir, sd_file, outdir)
    else:
        tdir, strings = os.path.join(outdir, "msi_tables"), []
    if a.phase == "all":
        phase_behavior(sdir, tdir, strings, outdir)
        phase_strings(sdir, outdir)
    print("[done] 证据输出目录:", outdir)


if __name__ == "__main__":
    main()
