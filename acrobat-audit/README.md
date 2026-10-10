# acrobat-audit · Adobe Acrobat Pro DC 安装器架构级安全审计

对官方 Acrobat Pro DC 26.002.21931（Windows）安装包的**数据库级逆向 + 安装行为 + 网络遥测面 + 二进制能力面 + AGS 运行期取证**审计。承接 Task 1 的下载链/Authenticode 验证，回答：*签名合法的官方安装器，会对系统做什么、向谁报告*。

## 核心结论（60 秒版）

1. **安装链路干净**：36 个内嵌 PE 的导入表证明**零网络外联能力、零进程注入 API**（R5），字符串层零隐藏端点（R1/R3）；无加壳（节熵全部<7.2）。
2. **一次默认安装的常驻清单**（全部有表级明示证据）：`AdobeARMservice`（SYSTEM 服务）+ `Adobe Acrobat Update Task`（登录计划任务）+ Run 键 `Acrobat Assistant 8.0`(Acrotray.exe) + WER 崩溃挂接(CRClient.dll) + 3,306 条注册表写入 + 5,280 个文件。
3. **更新面收敛于单域** `armmf.adobe.com`（4 清单 + 3 个 ARM 自更新 MSI，72h 周期）——可控但通道是活的。
4. **AGS（AdobeGCClient）不在本 MSI 静态载荷内**——由运行期链路引入（CA 中的 `gccustomhook.exe` 引用为耦合点）；2026-10 检索确认 AGS 仍活跃、社区高 CPU 投诉持续。
5. **浏览器集成面**（R5 新发现）：默认安装注册 Chrome 扩展 + Chrome 原生消息宿主（`com.adobe.acrobat.chrome_webcapture`），扩展经 `clients2.google.com/service/update2/crx` 标准通道自更新。
6. **AGS 证据链（R6）**：经范围请求取证（4MB+128MB 而非整包 1.58GB）取得 AGS 本体——`AGSService.exe`/`gccustomhook.exe` 均为 Adobe 签名且未篡改；`gc_dispatchtable` 揭示完整上报面（`genuine.adobe.com/server/services/{check,machineevents,appevents,…}` + `na1e.services.adobe.com`），二进制含 `MACAddress` 指纹采集；**`SKIP_GC=1` 为官方 AGS 阻断开关**；ARM 活体通道（清单+更新包）全链验签通过，`armsvc.exe`（SYSTEM 服务）零网络导入。
7. 风险定级：安装器**低**（R5 二进制层复核后证据链闭合）/ ARM 与 AGS**中** / 运行时遥测**中**——处置路径见 analysis/04（企业 FeatureLockdown 策略矩阵 + 个人级措施 + 只读自审计脚本）。

## 仓库结构

```
analysis/
  01-安装器数据库逆向.md     OLE 138流/字符串池破解/列块格式/11表重建
  02-安装行为面分析.md       服务/计划任务/Run键/336定制动作/落盘清单
  03-网络与遥测面.md         端点清单/8组外部检索交叉/AGS现状
  04-风险定级与处置建议.md   分级结论/处置矩阵/自审计指南
  05-二进制能力面.md         36个PE导入/导出/能力矩阵/身份修正/时间戳分群(R5)
  06-AGS取证.md              ARM活体通道/AGS本体验签/网络面全景/SKIP_GC开关(R6)
evidence/
  msi-tables/               11张重建表JSON(共~2.8MB)
  pe-capability/            R5: pe_capabilities+capability_matrix+strings_hits
  msi_manifest.json         138流清单(含SHA256前16位)
  msi_pe_meta.json          36个PE的版本资源身份
  msi_strings_mining.json   PE字符串挖掘(URL/域/服务/注册表)
  _behavior_summary.json    行为汇总(自启动命中30行等)
  external-research/        8组检索原始JSON(2026-10-09)
tools/
  msi_audit.py              核心可复现管线(单一脚本, 见下)
  zip_range_probe/index/fetch.py  ZIP范围取证三件套(免整包下载)
  libarchive_cab_extract.py  CAB(LZX)提取
  verify_msi_sig.py          signify验签(MSI/MSP)
  msi_pe_audit.py           R5: PE导入/导出/能力/字符串审计
  push_to_github.sh         token不落盘推送脚本
  Collect-AdobeAutostart.ps1  Windows端只读自审计收集器
source/
  AcroPro.msi(13MB) setup.exe setup.ini ABCPY.INI artifact-hashes.txt
  ags/                      ARM清单×3+ARM更新包+GC三件套+ARM载荷(R6)
WORKLOG.md                  逐轮工作记录
```

## 复现

```bash
pip install olefile pefile    # 依赖(第二个仅 R5 需要)
python3 tools/msi_audit.py    # 输出到 evidence/ (11表/行为/36PE挖掘)
python3 tools/msi_pe_audit.py # R5: PE能力面 -> evidence/pe-capability/
# 验证已内置: 字符串池长度断言 + 表规格断言
```

复现一致性：11 张表与首次导出 10 张字节级一致（Binary 因管线补全 Data 列为预期差异）。Windows 端配合 `tools/Collect-AdobeAutostart.ps1` 可将本审计的静态结论落到自己的实机。

## 方法论说明

- 证据纪律：每条结论可回溯到具体流/表/字符串/检索记录；无法证明的不写。
- 无 sudo/msitools 环境下的纯 Python 逆向：字符串池格式由唯一闭合数值检验锁定（Σ长度=781,290B）；表身份由内容特征判定（绕过 InstallShield 流名混淆）。
- 已知局限在 analysis/01 §3 诚实声明（CustomAction/File 变长列对齐）。
