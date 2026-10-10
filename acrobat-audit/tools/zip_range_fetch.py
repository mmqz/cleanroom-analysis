#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""zip_range_fetch.py — 对ZIP内单个条目做分块范围下载+流式解压
用法: python3 zip_range_fetch.py <url> <manifest.json> <entry-name-suffix> <outfile>
"""
import json
import struct
import sys
import zlib


def fetch_range(url, start, end):
    import urllib.request
    req = urllib.request.Request(url, headers={
        "Range": f"bytes={start}-{end}", "User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=300) as r:
        return r.read()


def main():
    url, manifest, suffix, outfile = sys.argv[1:5]
    man = json.load(open(manifest))
    e = next(x for x in man["entries"] if x["name"].endswith(suffix))
    lho, csize = e["local_header_off"], e["csize"]
    print(f"条目 {e['name']}: LHO={lho:,} csize={csize:,} method={e['method']}")
    # 先取本地头确定数据起点
    hdr = fetch_range(url, lho, lho + 128)
    (sig, ver, flags, method, mtime, mdate, crc, cs, us,
     nlen, elen) = struct.unpack_from("<IHHHHHIIIHH", hdr, 0)
    data_off = lho + 30 + nlen + elen
    print(f"本地头: method={method}, 数据起点={data_off:,}")
    d = zlib.decompressobj(-15) if method == 8 else None
    out = open(outfile, "wb")
    total_out = 0
    CH = 32 * 1024 * 1024
    pos = data_off
    end = data_off + csize - 1
    while pos <= end:
        to = min(pos + CH - 1, end)
        chunk = fetch_range(url, pos, to)
        if not chunk:
            sys.exit(f"范围 {pos}-{to} 返回空")
        if d:
            out.write(d.decompress(chunk))
        else:
            out.write(chunk)
        total_out += len(chunk)
        pos = to + 1
        print(f"  已拉 {total_out:,}/{csize:,}B", flush=True)
    if d:
        out.write(d.flush())
    out.close()
    import os
    print(f"完成: {outfile} = {os.path.getsize(outfile):,}B (期望usize {e['usize']:,})")


if __name__ == "__main__":
    main()
