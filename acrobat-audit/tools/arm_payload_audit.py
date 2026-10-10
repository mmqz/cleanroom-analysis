#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""arm_payload_audit.py — R6: ARM 渠道载荷(adobearm/adobearmhelper/armsvc + ARM MSI 内2个CA PE)
     + ARM MSI 内 2 个 CA PE 的能力审计
输出: evidence/pe-capability/arm_payload_audit.json
"""
import io
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from msi_pe_audit import audit_one  # 复用R5能力审计核心

import olefile

REPO = os.path.abspath(os.path.join(HERE, ".."))
PAYLOAD = os.path.join(REPO, "source", "ags", "arm_payload")
ARM_MSI = os.path.join(REPO, "source", "ags", "AdobeARM_1824214663.msi")
OUT = os.path.join(REPO, "evidence", "pe-capability", "arm_payload_audit.json")


def main():
    targets = []
    for fn in sorted(os.listdir(PAYLOAD)):
        targets.append((fn, open(os.path.join(PAYLOAD, fn), "rb").read()))
    # ARM MSI 内的独立 CA PE 流
    ole = olefile.OleFileIO(ARM_MSI)
    idx = 0
    for e in ole.listdir(streams=True, storages=False):
        p = "/".join(e)
        if not ole.exists(p):
            continue
        d = ole.openstream(p).read()
        idx += 1
        if d[:2] == b"MZ":
            targets.append((f"armmsi_s{idx:03d}_ca", d))

    full, matrix = [], []
    for label, data in targets:
        rec, caps, strs = audit_one(data, label)
        full_rec = dict(rec)
        if caps:
            full_rec["capabilities_detail"] = caps
        full.append(full_rec)
        ident = (rec.get("version_info", {}).get("InternalName")
                 or rec.get("version_info", {}).get("OriginalFilename")
                 or label)
        hi = []
        capset = set(caps.keys())
        if "net_wininet" in capset or "net_winhttp" in capset or "net_socket" in capset or "net_urldownload" in capset:
            hi.append("network_capability")
        if "proc_inject" in capset:
            hi.append("proc_inject")
        if "service_ctl" in capset:
            hi.append("service_ctl")
        if "reg_write" in capset and any(strs.get(k) for k in ("autoexec", "autoexec_w")):
            hi.append("reg_write+autoexec")
        matrix.append({
            "name": label, "identity": ident, "machine": rec.get("machine"),
            "size": rec["size"], "sha256_12": rec["sha256"][:12],
            "capabilities": sorted(caps.keys()), "flags": hi,
            "embedded_signature": bool(rec.get("embedded_signature")),
            "timestamp": rec.get("timestamp"),
        })

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    json.dump(full, open(OUT, "w"), ensure_ascii=False, indent=1)
    json.dump(matrix, open(OUT.replace(".json", "_matrix.json"), "w"),
              ensure_ascii=False, indent=1)
    for m in matrix:
        print(f"{m['name']:22s} {m['identity'] or '?':16s} {m['machine']} "
              f"sig={str(m['embedded_signature'])[0]} caps={','.join(m['capabilities'])[:80]}")
        if m["flags"]:
            print(f"    FLAGS: {m['flags']}")
    print(f"[out] {OUT}")


if __name__ == "__main__":
    main()
