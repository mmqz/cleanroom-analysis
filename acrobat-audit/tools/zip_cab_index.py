#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""zip_cab_index.py — 对ZIP内各CAB做范围请求+流式解压, 解析CFFILE索引
用法: python3 zip_cab_index.py <manifest.json> <url> [--fetch 786432] [--all]
输出: 每个CAB的文件名清单(前若干) + GC/AGS相关文件定位
"""
import argparse
import json
import struct
import sys
import zlib


def fetch_range(url, start, end):
    import urllib.request
    req = urllib.request.Request(url, headers={
        "Range": f"bytes={start}-{end}", "User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=180) as r:
        return r.read()


def local_header(buf):
    assert buf[:4] == b"PK\x03\x04", "非本地文件头"
    (sig, ver, flags, method, mtime, mdate, crc, csize, usize,
     nlen, elen) = struct.unpack_from("<IHHHHHIIIHH", buf, 0)
    name = buf[30:30 + nlen].decode(errors="replace")
    return 30 + nlen + elen, name, method


def stream_decompress_prefix(url, lho, fetch_n, want_n):
    raw = fetch_range(url, lho, lho + 64 + fetch_n)
    off, name, method = local_header(raw)
    payload = raw[off:]
    if method == 0:
        return name, payload[:want_n]
    d = zlib.decompressobj(-15)
    out = b""
    try:
        out = d.decompress(payload, want_n)
    except Exception as e:
        print(f"  [!] 流式解压中断({e}), 已取 {len(out)}B", file=sys.stderr)
    return name, out


def parse_mscf_index(cab):
    if cab[:4] != b"MSCF":
        return {"error": f"非MSCF: {cab[:8]!r}", "files": []}
    (sig, r1, cbCab, r2, coffFiles, r3, cFolders, cFiles,
     flags, setID) = struct.unpack_from("<4sIIIIIHHHH", cab, 0)
    # 修正: MSCF头 = 4s+4u32(csum)+4u32(cbCab)+4u32(csum)+4u32(coffFiles)+4u32(res)+2u16(cFolders)+2u16(cFiles)+2u16(flags)+2u16(setID)
    names = []
    pos = coffFiles
    n = 0
    while n < cFiles and pos + 20 <= len(cab):
        cbFile, uoff, iFolder = struct.unpack_from("<IIH", cab, pos)
        date, tm, attr = struct.unpack_from("<HHH", cab, pos + 10)
        end = cab.find(b"\x00", pos + 16)
        if end < 0:
            break
        name = cab[pos + 16:end].decode(errors="replace")
        names.append({"name": name, "usize": cbFile, "iFolder": iFolder})
        pos = end + 1
        n += 1
    return {"cFiles": cFiles, "cFolders": cFolders, "coffFiles": coffFiles,
            "flags": flags, "indexed": len(names), "files": names,
            "error": None if len(names) == cFiles else
            f"部分索引: {len(names)}/{cFiles} (解压前缀不足)"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("manifest")
    ap.add_argument("url")
    ap.add_argument("--fetch", type=int, default=786432)
    ap.add_argument("--all", action="store_true")
    args = ap.parse_args()
    man = json.load(open(args.manifest))
    out = {}
    for e in man["entries"]:
        nm = e["name"].split("/")[-1]
        if not nm.endswith(".cab"):
            continue
        print(f"-- {e['name']} (usize {e['usize']:,}) @LHO {e['local_header_off']:,}")
        zname, prefix = stream_decompress_prefix(
            args.url, e["local_header_off"], args.fetch, args.fetch)
        idx = parse_mscf_index(prefix)
        if idx.get("error"):
            print("   ", idx["error"])
            continue
        print(f"    cFiles={idx['cFiles']} indexed={idx['indexed']} "
              f"cFolders={idx['cFolders']} flags={idx['flags']}")
        hits = [f for f in idx["files"] if any(
            k in f["name"].lower() for k in
            ("agsservice", "gccustomhook", "gclient", "adobegc", "dispatch",
             "ngl", "genuine", "ags", "gc_dispatch"))]
        for h in hits:
            print(f"    *** {h['name']} ({h['usize']:,}B, folder={h['iFolder']})")
        if not hits:
            print("    (无GC相关命中) 首批:", [f["name"] for f in idx["files"][:3]])
        out[nm] = {"index": {k: v for k, v in idx.items() if k != "files"},
                   "gc_hits": hits,
                   "all_names_sample": [f["name"] for f in idx["files"][:5]]}
    json.dump(out, open("/tmp/cab_index.json", "w"), ensure_ascii=False, indent=1)
    print("[saved] /tmp/cab_index.json")


if __name__ == "__main__":
    main()
