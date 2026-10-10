#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""msi_pe_audit.py — 第5轮: AcroPro.msi 内嵌 36 个 PE 的二进制能力面审计
依赖: olefile, pefile (pip install olefile pefile)
用法: python3 tools/msi_pe_audit.py [--msi PATH] [--outdir DIR]
      默认 --msi source/AcroPro.msi --outdir evidence/pe-capability (仓库相对)
输出:
  pe_capabilities.json  每个 PE 的机器类型/时间戳/节区熵/导入/导出/延迟导入/安全目录
  capability_matrix.json 每 PE 能力位 + 敏感组合判定
  strings_hits.json     ASCII+UTF16LE 字符串挖掘(URL/注册表/命令行/互斥体/关键词)
      b. 能力分类依据导入表函数名, 字符串证据独立归档, 两者交叉引证
"""
import argparse
import hashlib
import json
import os
import re
import sys
from collections import OrderedDict

try:
    import olefile
    import pefile
except ImportError:
    sys.exit("需要依赖: pip install olefile pefile")

HERE = os.path.dirname(os.path.abspath(__file__))

# ---------------- 能力映射表: (能力名, 判定函数名的集合/正则) ----------------
CAP_RULES = OrderedDict([
    ("proc_inject",   {  # 远程写入/注入代码到他人进程
        "WriteProcessMemory", "VirtualAllocEx", "CreateRemoteThread",
        "CreateRemoteThreadEx", "QueueUserAPC", "SetWindowsHookExA",
        "SetWindowsHookExW", "GetThreadContext", "NtMapViewOfSection"}),
    ("proc_exec",     {  # 派生进程
        "CreateProcessA", "CreateProcessW", "WinExec", "_popen", "system",
        "ShellExecuteA", "ShellExecuteW", "ShellExecuteExA", "ShellExecuteExW"}),
    ("proc_term",     {"TerminateProcess", "TerminateThread", "ExitProcess"}),
    ("proc_enum",     {"CreateToolhelp32Snapshot", "Process32First", "Process32FirstW",
                       "EnumProcesses", "OpenProcess"}),
    ("reg_write",     {"RegCreateKeyExA", "RegCreateKeyExW", "RegSetValueExA",
                       "RegSetValueExW", "RegDeleteKeyA", "RegDeleteKeyW",
                       "RegDeleteValueA", "RegDeleteValueW", "RegSetKeyValue",
                       "RegRestoreKey", "RegReplaceKey", "RegLoadKey"}),
    ("reg_read",      {"RegOpenKeyExA", "RegOpenKeyExW", "RegQueryValueExA",
                       "RegQueryValueExW", "RegEnumKeyExA", "RegEnumValueA"}),
    ("file_del",      {"DeleteFileA", "DeleteFileW", "SHFileOperationA", "SHFileOperationW"}),
    ("file_enum",     {"FindFirstFileA", "FindFirstFileW", "FindNextFileA", "FindNextFileW"}),
    ("net_wininet",   {"InternetOpenA", "InternetOpenW", "InternetConnectA",
                       "InternetConnectW", "HttpOpenRequestA", "HttpOpenRequestW",
                       "HttpSendRequestA", "HttpSendRequestW", "InternetReadFile",
                       "InternetCloseHandle", "InternetOpenUrlA", "InternetOpenUrlW"}),
    ("net_winhttp",   {"WinHttpOpen", "WinHttpConnect", "WinHttpOpenRequest",
                       "WinHttpSendRequest", "WinHttpReceiveResponse", "WinHttpReadData"}),
    ("net_socket",    {"WSAStartup", "socket", "connect", "send", "recv",
                       "gethostbyname", "getaddrinfo", "WSASocket", "sendto", "recvfrom"}),
    ("net_urldownload", {"URLDownloadToFileA", "URLDownloadToFileW", "DownloadFile"}),
    ("service_ctl",   {"OpenSCManagerA", "OpenSCManagerW", "CreateServiceA", "CreateServiceW",
                       "StartServiceA", "StartServiceW", "ControlService", "DeleteService",
                       "ChangeServiceConfigA", "ChangeServiceConfigW", "LockServiceDatabase"}),
    ("privilege",     {"AdjustTokenPrivileges", "OpenProcessToken",
                       "LookupPrivilegeValueA", "LookupPrivilegeValueW", "LogonUserA",
                       "LogonUserW", "ImpersonateLoggedOnUser", "SetTokenInformation"}),
    ("crypto",        {"CryptAcquireContextA", "CryptAcquireContextW", "CryptEncrypt",
                       "CryptDecrypt", "CryptHashData", "BCryptOpenAlgorithmProvider",
                       "BCryptEncrypt", "CryptStringToBinaryA", "CryptProtectData"}),
    ("com_schtasks",  {"CoCreateInstance", "CoCreateInstanceEx", "CoInitialize",
                       "CoInitializeEx", "CLSIDFromProgID"}),
    ("debug_anti",    {"IsDebuggerPresent", "CheckRemoteDebuggerPresent",
                       "GetTickCount", "QueryPerformanceCounter"}),
    ("screen",        {"BitBlt", "GetDC", "GetWindowDC", "CreateCompatibleDC",
                       "CreateCompatibleBitmap", "StretchBlt"}),
    ("cred",          {"CredReadA", "CredReadW", "CredWriteA", "CredWriteW",
                       "CredEnumerateA", "CredEnumerateW"}),
    ("wer",           {"WerRegisterFile", "WerRegisterMemoryBlock", "WerUnregisterFile",
                       "WerRegisterRuntimeModule", "ReportFault"}),
    ("sfc_protect",   {"SfcFileException", "SfcIsKeyProtected"}),
    ("hook_kb",       {"SetWindowsHookExA", "SetWindowsHookExW", "GetAsyncKeyState",
                       "GetKeyState", "MapVirtualKey"}),
])

# 敏感组合判定(能力位×字符串证据)
SUSPICIOUS_COMBOS = [
    ("proc_inject+proc_enum", "枚举他人进程并向其远程写入/执行 — 注入能力", "high"),
    ("proc_inject+privilege", "提权后注入 — 高危组合", "high"),
    ("reg_write+autoexec_path", "写注册表且字符串含 Run/RunOnce 自启动路径 — 持久化写入", "medium"),
    ("net_wininet+file_write", "网络下载且写文件 — 下载落地能力", "medium"),
    ("net_winhttp+file_write", "HTTP(S)下载且写文件 — 下载落地能力", "medium"),
    ("proc_exec+script_cmd", "派生进程且字符串含脚本宿主(cmd/powershell/mshta) — 命令执行链", "high"),
    ("file_del+self_path", "自删除模式字符(如 cmd /c del 自身) — 闪避痕迹", "high"),
    ("debug_anti_only", "仅反调试不足以定罪, 结合上下文复核", "info"),
]

# ---------------- 字符串挖掘正则 ----------------
STR_PATTERNS = OrderedDict([
    ("url",          re.compile(rb"https?://[!-~]{4,120}")),
    ("domain",       re.compile(rb"(?i)(?:[a-z0-9-]+\.)+(?:adobe|adobegenuine|macromedia|typekit|behance|ftcdn|demdex|amazonaws|akamai|adobestats|eulerian|2o7)\.(?:com|io|net)[/a-zA-Z0-9._?=&-]*")),
    ("registry",     re.compile(rb"(?i)(?:Software\\\\|CurrentVersion\\\\(?:Run|RunOnce|Uninstall|Explorer|Windows Error Reporting|Group Policy|Policies)|System\\\\CurrentControlSet\\\\Services)[!-~]{0,120}")),
    ("cmdline",      re.compile(rb"(?i)\b(schtasks|sc\.exe|reg\.exe|cmd\.exe|powershell|mshta|wscript|cscript|rundll32|msiexec|netsh|wmic|taskkill|regsvr32|cacls|icacls|takeown)\b[!-~]{0,80}")),
    ("autoexec",     re.compile(rb"(?i)(?:Run|RunOnce|RunOnceEx|Startup|Approved)[\\\\/]?[!-~]{0,40}")),
    ("globalobj",    re.compile(rb"(?i)(?:Global|Local)\\\\[A-Za-z0-9_.\-]{4,60}")),
    ("service",      re.compile(rb"(?i)\b(Armsvc|AdobeARMservice|Adobe[A-Za-z]*Service|ARMService)\b")),
    ("taskname",     re.compile(rb"(?i)Adobe[A-Za-z ]{0,30}(?:Task|Update Task)\b")),
    ("schtask_clsid", re.compile(rb"\{0F87369F-A4E5-4CFC-BD3E-73E6154572(?:DD|2DD)\}")),
    ("keyword",      re.compile(rb"(?i)\b(telemetry|crashrpt|genuine|ngl|adobegc|gclient|login|sign.?in|analytics|tracking|fingerprint|mac.?address|machine.?id|serial|hwid)\b[!-~]{0,60}")),
])
# UTF-16LE: 提取宽字符串解码后复用上方 ASCII 正则(mine_strings 内处理)


def decode_version_info(pe):
    info = {}
    try:
        if hasattr(pe, "FileInfo") and pe.FileInfo:
            for lst in pe.FileInfo:
                for entry in (lst if isinstance(lst, list) else [lst]):
                    if getattr(entry, "Key", "") in ("StringFileInfo", b"StringFileInfo"):
                        for st in getattr(entry, "StringTable", []):
                            for k, v in st.entries.items():
                                if isinstance(k, bytes):
                                    k = k.decode(errors="replace")
                                if isinstance(v, bytes):
                                    v = v.decode(errors="replace")
                                if isinstance(v, str):
                                    info[k] = v[:80]
    except Exception:
        pass
    return info


def section_table(pe):
    secs = []
    try:
        for s in pe.sections:
            data = s.get_data()
            import math
            if data:
                freq = [0] * 256
                for b in data[:200000]:
                    freq[b] += 1
                n = min(len(data), 200000)
                ent = -sum((c / n) * math.log2(c / n) for c in freq if c)
            else:
                ent = 0.0
            secs.append({
                "name": s.Name.decode(errors="replace").rstrip("\x00"),
                "vsize": int(s.Misc_VirtualSize), "rsize": int(s.SizeOfRawData),
                "entropy": round(ent, 2),
                "executable": bool(s.Characteristics & 0x20000000),
                "writable": bool(s.Characteristics & 0x80000000)})
    except Exception as e:
        secs.append({"error": str(e)})
    return secs


def imports_map(pe):
    """返回 {dll: [funcs]}, 含普通与延迟导入"""
    out = {}
    try:
        if hasattr(pe, "DIRECTORY_ENTRY_IMPORT"):
            for entry in pe.DIRECTORY_ENTRY_IMPORT:
                out[entry.dll.decode(errors="replace")] = [
                    (imp.name.decode(errors="replace") if imp.name else f"#ord{imp.ordinal}")
                    for imp in entry.imports]
    except Exception:
        pass
    try:
        if hasattr(pe, "DIRECTORY_ENTRY_DELAY_IMPORT"):
            for entry in pe.DIRECTORY_ENTRY_DELAY_IMPORT:
                dll = entry.dll.decode(errors="replace")
                out.setdefault(dll + " (delay)", [])
                out[dll + " (delay)"] = [
                    (imp.name.decode(errors="replace") if imp.name else f"#ord{imp.ordinal}")
                    for imp in entry.imports]
    except Exception:
        pass
    return out


def exports_list(pe):
    names = []
    try:
        if hasattr(pe, "DIRECTORY_ENTRY_EXPORT"):
            names = [e.name.decode(errors="replace") for e in
                     pe.DIRECTORY_ENTRY_EXPORT.symbols if e.name][:60]
    except Exception:
        pass
    return names


def capabilities(imports):
    flat = set()
    for funcs in imports.values():
        flat.update(funcs)
    caps = {}
    for cap, ruleset in CAP_RULES.items():
        hit = sorted(flat & ruleset)
        if hit:
            caps[cap] = hit
    return caps


def mine_strings(data):
    """ASCII + UTF16LE 双通道挖掘, 返回 {pattern: [去重命中]}"""
    hits = {}
    # ASCII
    for name, rx in STR_PATTERNS.items():
        found = []
        for m in rx.findall(data):
            s = m if isinstance(m, str) else m.decode("latin-1")
            found.append(s)
        if found:
            hits[name] = sorted(set(found))[:40]
    # UTF16LE: 粗提取宽字符串
    wide = re.findall((b"(?:[ -~]\x00){4,}"), data)
    wide_text = "\n".join(w.decode("latin-1", errors="ignore").replace("\x00", "")
                          for w in wide)
    wide_bytes = wide_text.encode("latin-1", errors="ignore")
    for name, rx in STR_PATTERNS.items():
        found = []
        for m in rx.findall(wide_bytes):
            s = m if isinstance(m, str) else m.decode("latin-1")
            found.append(s)
        if found:
            key = name + "_w"
            hits[key] = sorted(set(found))[:40]
    return hits


def rich_header(pe):
    try:
        rh = pe.parse_data_directories(directories=[
            pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_RESOURCE"]])
    except Exception:
        pass
    # pefile 的 Rich header 需要手动解析 DOS stub 之后
    try:
        return {"present": hasattr(pe, "RICH_HEADER") and bool(getattr(pe, "RICH_HEADER"))}
    except Exception:
        return {"present": False}


def audit_one(data, label):
    rec = {"stream": label, "size": len(data),
           "sha256": hashlib.sha256(data).hexdigest()}
    try:
        pe = pefile.PE(data=data, fast_load=False)
    except Exception as e:
        rec["parse_error"] = str(e)[:120]
        return rec, {}, {}
    fh = pe.FILE_HEADER
    rec["machine"] = {0x8664: "x64", 0x14c: "x86"}.get(fh.Machine, hex(fh.Machine))
    rec["timestamp"] = fh.TimeDateStamp
    rec["sections"] = section_table(pe)
    rec["subsystem"] = getattr(pe.OPTIONAL_HEADER, "SUBSYSTEM", 0)
    rec["is_dll"] = bool(fh.Characteristics & 0x2000)
    # 安全目录(内嵌签名)
    sec_dir = pe.OPTIONAL_HEADER.DATA_DIRECTORY[4]
    rec["embedded_signature"] = {"size": int(sec_dir.Size)} if sec_dir.Size else None
    rec["version_info"] = decode_version_info(pe)
    rec["imports"] = imports_map(pe)
    rec["exports"] = exports_list(pe)
    rec["rich_header"] = rich_header(pe)
    caps = capabilities(rec["imports"])
    strs = mine_strings(data)
    rec["import_dlls"] = sorted(rec["imports"].keys())
    rec.pop("imports", None)  # 完整导入表放capabilities文件, 矩阵只留dll清单
    try:
        pe.close()
    except Exception:
        pass
    return rec, caps, strs


def suspicious_flags(caps, strs):
    flag = []
    capset = set(caps.keys())
    strkeys = set(strs.keys())
    has = lambda k: k in strkeys and strs.get(k)
    if {"proc_inject"} <= capset and ({"proc_enum"} <= capset or "proc_enum" in capset):
        flag.append(("proc_inject+proc_enum", "high"))
    if "proc_inject" in capset and "privilege" in capset:
        flag.append(("proc_inject+privilege", "high"))
    if "reg_write" in capset and (has("autoexec") or "autoexec_w" in strkeys):
        flag.append(("reg_write+autoexec_path", "medium"))
    if ("net_wininet" in capset or "net_winhttp" in capset or
            "net_urldownload" in capset):
        w = any(("file_write" in c or "WriteFile" in str(caps.get("file_write", [])))
                for c in caps)
        if "file_write" in caps or "file_enum" in caps:
            flag.append(("net_download+file_io", "medium"))
    if "proc_exec" in capset:
        joined = " ".join(strs.get("cmdline", []) + strs.get("cmdline_w", []))
        if any(x in joined.lower() for x in ("powershell", "mshta", "wscript",
                                             "cmd.exe", "rundll32")):
            flag.append(("proc_exec+script_cmd", "high"))
    return flag


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--msi", default=os.path.join(HERE, "..", "source", "AcroPro.msi"))
    ap.add_argument("--outdir", default=os.path.join(HERE, "..", "evidence", "pe-capability"))
    args = ap.parse_args()
    msi = os.path.abspath(args.msi)
    outdir = os.path.abspath(args.outdir)
    os.makedirs(outdir, exist_ok=True)

    ole = olefile.OleFileIO(msi)
    pes = []
    for i, e in enumerate(ole.listdir(streams=True, storages=False)):
        path = "/".join(e)
        size = ole.get_size(path) if ole.exists(path) else 0
        if size <= 0:
            continue
        data = ole.openstream(path).read()
        if data[:2] != b"MZ":
            continue
        pes.append((f"s{i+1:03d}_{size}", data))
    print(f"[extract] {len(pes)} 个 PE 流自 MSI 提取")

    full, matrix, strings_all = [], [], {}
    for label, data in pes:
        rec, caps, strs = audit_one(data, label)
        full_rec = dict(rec)
        if caps:
            full_rec["capabilities_detail"] = caps
        full.append(full_rec)
        flags = suspicious_flags(caps, strs)
        ident = (rec.get("version_info", {}).get("InternalName")
                 or rec.get("version_info", {}).get("OriginalFilename")
                 or "?")
        matrix.append({
            "stream": label, "identity": ident, "machine": rec.get("machine"),
            "is_dll": rec.get("is_dll"), "size": rec["size"],
            "sha256_12": rec["sha256"][:12],
            "capabilities": sorted(caps.keys()),
            "suspicious": [f[0] for f in flags],
            "suspicious_level": max((f[1] for f in flags), default="none"),
            "embedded_signature": bool(rec.get("embedded_signature")),
            "import_dll_count": len(rec.get("import_dlls", [])),
        })
        if strs:
            strings_all[label] = {"identity": ident, "hits": strs}

    json.dump(full, open(os.path.join(outdir, "pe_capabilities.json"), "w"),
              ensure_ascii=False, indent=1)
    json.dump(matrix, open(os.path.join(outdir, "capability_matrix.json"), "w"),
              ensure_ascii=False, indent=1)
    json.dump(strings_all, open(os.path.join(outdir, "strings_hits.json"), "w"),
              ensure_ascii=False, indent=1)

    # 控制台摘要
    print(f"[parse] x64={sum(1 for m in matrix if m['machine']=='x64')} "
          f"x86={sum(1 for m in matrix if m['machine']=='x86')} "
          f"带签名={sum(1 for m in matrix if m['embedded_signature'])}")
    print("[caps] 能力覆盖:")
    from collections import Counter
    cc = Counter(c for m in matrix for c in m["capabilities"])
    for cap, n in cc.most_common():
        print(f"    {cap:18s} {n:2d}/{len(matrix)}")
    hi = [m for m in matrix if m["suspicious_level"] in ("high", "medium")]
    print(f"[flags] 敏感组合命中 {len(hi)} 个 PE:")
    for m in hi:
        print(f"    {m['identity']:20s} {m['suspicious_level']:6s} {m['suspicious']}")
    print(f"[out] {outdir}/pe_capabilities.json (+matrix +strings)")


if __name__ == "__main__":
    main()
