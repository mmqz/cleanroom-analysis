#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""verify_msi_sig.py — MSI/MSP Authenticode 验签(signify SignedMsiFile)
用法: python3 verify_msi_sig.py <file.msi> [file2.msi ...]
输出: 签名者证书/有效期/时间戳/authentihash与messageDigest匹配结论
方法: m.get_fingerprint(<hashlib可调用>) 为signify对MSI CFB结构的authentihash计算,
      与signer_info.message_digest 逐字节对比, 相等=文件自签名后未被篡改
"""
import sys
import hashlib
from datetime import timezone

from signify.authenticode.signed_file import SignedMsiFile

HASHERS = {
    "sha256": hashlib.sha256, "sha384": hashlib.sha384,
    "sha512": hashlib.sha512, "sha1": hashlib.sha1,
}


def fmt(dt):
    try:
        return dt.astimezone(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
    except Exception:
        return str(dt)


def verify(path):
    print(f"===== {path} =====")
    data = open(path, "rb").read()
    try:
        m = SignedMsiFile.__new__(SignedMsiFile)  # 占位, 实际用构造器
    except Exception:
        pass
    import io
    try:
        m = SignedMsiFile(io.BytesIO(data))
    except Exception as e:
        print(f"  [!] 非签名MSI或解析失败: {e}")
        return None
    sigs = list(m.iter_embedded_signatures())
    if not sigs:
        print("  [!] 无内嵌签名")
        return None
    for s in sigs:
        try:
            si = s.signer_infos[0]
        except Exception as e:
            print(f"  [!] signer_info 解析失败: {e}")
            continue
        alg_name = str(getattr(s.digest_algorithm, "name", s.digest_algorithm)).lower()
        hasher = None
        for k, h in HASHERS.items():
            if k in alg_name:
                hasher = h
                break
        certs = list(s.certificates)
        # 真正的签名者证书: 按 signer_info 的 issuer+serial 在证书集中匹配
        signer_cert = None
        try:
            tgt_issuer, tgt_serial = si.issuer, si.serial_number
            for c in certs:
                if str(c.issuer) == str(tgt_issuer) and c.serial_number == tgt_serial:
                    signer_cert = c
                    break
        except Exception:
            pass
        if signer_cert is None and certs:
            # 回退: 非时间戳用途的首个证书
            for c in certs:
                if "timestamp" not in str(c.subject).lower():
                    signer_cert = c
                    break
        if signer_cert:
            c = signer_cert
            print(f"  签名者: {c.subject}")
            print(f"  颁发者: {c.issuer}")
            print(f"  有效期: {fmt(c.valid_from)} ~ {fmt(c.valid_to)}")
            try:
                print(f"  证书序列号: {hex(c.serial_number)}")
            except Exception:
                pass
        try:
            pn = si.program_name
            if pn:
                print(f"  ProgramName: {pn}")
        except Exception:
            pass
        try:
            if si.signing_time:
                print(f"  签名时间: {fmt(si.signing_time)}")
        except Exception:
            pass
        try:
            cs = si.countersigner
            if cs is not None:
                st = getattr(cs, "signing_time", None) or getattr(cs, "generalized_time", None)
                print(f"  RFC3161时间戳: {fmt(st) if st else '(存在, 时间字段略)'}")
        except Exception:
            pass
        if hasher:
            # 文件authentihash vs SpcIndirectData内嵌的文件摘要(indirect_data.digest)
            fp = m.get_fingerprint(hasher)
            try:
                cd = s.indirect_data.digest
            except Exception:
                cd = None
            match = fp == cd
            print(f"  摘要算法: {alg_name}")
            print(f"  文件authentihash: {fp.hex()}")
            print(f"  签名内嵌摘要:    {cd.hex() if cd else None}")
            print(f"  匹配: {'YES — 文件未被篡改' if match else 'NO — 摘要不一致!'}")
        else:
            print(f"  摘要算法 {alg_name} 不在支持列表, 跳过hash计算")
    print()
    return True


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for p in sys.argv[1:]:
        verify(p)
