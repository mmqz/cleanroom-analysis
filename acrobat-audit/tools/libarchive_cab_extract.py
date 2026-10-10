#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""libarchive_cab_extract.py — 经 ctypes 调系统 libarchive 解 CAB(支持 MSZIP/LZX)
用法: python3 libarchive_cab_extract.py <file.cab> <outdir>
"""
import ctypes
import os
import sys

lib = ctypes.CDLL("libarchive.so.13")

# 函数原型(注意 64 位类型)
lib.archive_read_new.restype = ctypes.c_void_p
lib.archive_read_support_format_all.restype = ctypes.c_int
lib.archive_read_support_filter_all.restype = ctypes.c_int
lib.archive_read_open_memory.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t]
lib.archive_read_open_memory.restype = ctypes.c_int
lib.archive_read_next_header.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
lib.archive_read_next_header.restype = ctypes.c_int
lib.archive_read_data.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t]
lib.archive_read_data.restype = ctypes.c_ssize_t
lib.archive_entry_pathname.argtypes = [ctypes.c_void_p]
lib.archive_entry_pathname.restype = ctypes.c_char_p
lib.archive_entry_size.argtypes = [ctypes.c_void_p]
lib.archive_entry_size.restype = ctypes.c_int64
lib.archive_read_free.argtypes = [ctypes.c_void_p]
lib.archive_read_free.restype = ctypes.c_int
lib.archive_error_string.argtypes = [ctypes.c_void_p]
lib.archive_error_string.restype = ctypes.c_char_p


def extract(cab_path, outdir):
    os.makedirs(outdir, exist_ok=True)
    data = open(cab_path, "rb").read()
    a = lib.archive_read_new()
    assert a, "archive_read_new 失败"
    lib.archive_read_support_format_all(a)
    lib.archive_read_support_filter_all(a)
    r = lib.archive_read_open_memory(a, data, len(data))
    if r != 0:
        print("open失败:", lib.archive_error_string(a))
        return []
    out = []
    entry = ctypes.c_void_p()
    while True:
        r = lib.archive_read_next_header(a, ctypes.byref(entry))
        if r == 1:   # EOF
            break
        if r < 0:
            print("错误:", lib.archive_error_string(a))
            break
        name = lib.archive_entry_pathname(entry)
        if not name:
            continue
        name = name.decode(errors="replace").replace("\\", "/")
        size = lib.archive_entry_size(entry)
        # 流式读取
        chunks = []
        buf = ctypes.create_string_buffer(1 << 20)
        while True:
            n = lib.archive_read_data(a, buf, len(buf))
            if n <= 0:
                break
            chunks.append(buf.raw[:n])
        payload = b"".join(chunks)
        safe = os.path.basename(name)
        dest = os.path.join(outdir, safe)
        with open(dest, "wb") as f:
            f.write(payload)
        out.append((safe, len(payload), size))
        print(f"  {safe:28s} {len(payload):9d}B (entry声称 {size})")
    lib.archive_read_free(a)
    return out


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    files = extract(sys.argv[1], sys.argv[2])
    print(f"共 {len(files)} 个文件")
