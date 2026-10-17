#!/usr/bin/env python3
"""每次转换一个独立 wrapper 进程：运行 soffice 并以 JSON 报告退出码/
stdout/stderr/墙钟/子进程峰值 RSS。

峰值 RSS 口径（诚实登记）：本环境无 /usr/bin/time（已核实不存在）。
wrapper 在 subprocess.run(soffice) 返回后读取
getrusage(RUSAGE_CHILDREN).ru_maxrss——Linux 上单位为 kB；
RUSAGE_CHILDREN 覆盖该 wrapper 已等待的全部子进程（/usr/bin/soffice
启动脚本 exec 为 soffice.bin，即直接子进程；若存在被其等待的后代，
亦计入 max）。**每次转换使用全新 wrapper 进程**，故数值为该次转换的
子进程峰值 RSS，而非跨用例累积高水位。

用法：python3 rss_wrapper.py '<JSON 命令数组>'
"""

import base64
import json
import resource
import subprocess
import sys
import time


def main() -> int:
    cmd = json.loads(sys.argv[1])
    t0 = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True)
    except Exception as exc:  # wrapper 自身异常也以 JSON 报告——父进程永不因 wrapper 崩
        sys.stdout.write(json.dumps({"wrapper_error": repr(exc)}))
        return 0
    wall_s = time.perf_counter() - t0
    maxrss_kb = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    sys.stdout.write(json.dumps({
        "returncode": proc.returncode,
        "stdout_b64": base64.b64encode(proc.stdout).decode("ascii"),
        "stderr_b64": base64.b64encode(proc.stderr).decode("ascii"),
        "wall_s": round(wall_s, 3),
        "maxrss_kb": maxrss_kb,
    }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
