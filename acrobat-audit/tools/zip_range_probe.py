#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""zip_range_probe.py — 远程ZIP中央目录解析(不经整包下载)
用法: python3 zip_range_probe.py <url> [--save manifest.json]
输出: 每个条目 {name, offset, csize, usize, method}
"""
import argparse
import json
import struct
import sys
import urllib.request


def fetch_range(url, start, end):
    req = urllib.request.Request(
        url, headers={"Range": f"bytes={start}-{end}",
                      "User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=120) as r:
        return r.read()


def head(url):
    req = urllib.request.Request(url, method="HEAD",
                                 headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return int(r.headers["Content-Length"]), r.headers.get("Accept-Ranges", "")


def parse_central_dir(data):
    """data: 含 EOCD 与中央目录的字节段"""
    # 找 EOCD: 0x06054b50
    eocd_off = data.rfind(b"PK\x05\x06")
    if eocd_off < 0:
        sys.exit("未找到 EOCD")
    (sig, disk, cd_disk, n_disk, n_total, cd_size, cd_off,
     cmt_len) = struct.unpack_from("<IHHHHIIH", data, eocd_off)
    entries = []
    cd_start = len(data) - (len(data) - eocd_off) + cd_size  # cd 在段内偏移
    # 中央目录通常正好在 EOCD 前
    pos = eocd_off - cd_size
    if pos < 0:
        sys.exit(f"中央目录尺寸异常: cd_size={cd_size}")
    for _ in range(n_total):
        (sig, ver_made, ver_need, flags, method, mtime, mdate, crc,
         csize, usize, nlen, elen, clen, disk_no, iattr, eattr, lho) = \
            struct.unpack_from("<IHHHHHHIIIHHHHHII", data, pos)
        name = data[pos + 46: pos + 46 + nlen].decode(errors="replace")
        entries.append({"name": name, "method": method, "crc": f"{crc:08x}",
                        "csize": csize, "usize": usize, "local_header_off": lho})
        pos += 46 + nlen + elen + clen
    return entries, cd_size, cd_off


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("url")
    ap.add_argument("--save")
    ap.add_argument("--tail", type=int, default=262144)
    args = ap.parse_args()
    total, ranges = head(args.url)
    print(f"总大小: {total:,}B | Range支持: {ranges}")
    tail = fetch_range(args.url, max(0, total - args.tail), total - 1)
    entries, cd_size, cd_off = parse_central_dir(tail)
    print(f"条目数: {len(entries)} | 中央目录: {cd_size:,}B @ {cd_off:,}")
    for e in entries:
        print(f"  {e['name']:28s} method={e['method']} csize={e['csize']:>12,} "
              f"usize={e['usize']:>12,} LHO={e['local_header_off']:,}")
    if args.save:
        json.dump({"total": total, "entries": entries},
                  open(args.save, "w"), indent=1)
        print(f"[saved] {args.save}")


if __name__ == "__main__":
    main()
