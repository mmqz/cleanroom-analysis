# 06 · AGS 取证：Adobe Genuine Service 的获取、验证与逆向

> 轮次：第 6 轮。目标：拿 到 AGS（AdobeGClient/AGSService）本体二进制并审计——直击最初会话的两个疑点：**CPU 异常**与**侵入式遥测**。
> 方法亮点：对 1.58GB 官方 ZIP 做**范围请求取证**（中央目录 298KB + CAB 索引扫描 4MB + Intermediate.cab 128MB），全程未整包下载。
> 证据：`source/ags/`（三清单 MSI + ARM 更新包 + GC 三件套 + ARM 载荷）、`evidence/pe-capability/ags_*.json`、`evidence/external-research/`

## 1. 获取链：四段全部验签通过

| 工件 | 来源 | 签名 | 时间戳 | authentihash |
|---|---|---|---|---|
| ArmManifest.msi | armmf.adobe.com（活体） | Adobe EV (Symantec G2) | 2017-04-05 | **匹配** |
| AcroManifest.msi | armmf.adobe.com（活体） | Adobe AcrobatXI (Symantec) | 2015-02-26 | **匹配** |
| AcrobatDCManifest.msi | armmf.adobe.com（活体） | Adobe Inc. (DigiCert G4, 2023) | 2024-07-11 | **匹配**（sha256） |
| AdobeARM_1824214663.msi | armmf.adobe.com（活体） | Adobe EV (Symantec G2) | 2017-02-03 | **匹配** |
| AGSService.exe 6.6.0.204 | Intermediate.cab | Adobe Inc. OU=AAM 256 (DigiCert) | — | **匹配**（signify） |
| gccustomhook.exe 6.0.0.168 | Intermediate.cab | Adobe Systems OU=AAM 256 (DigiCert) | — | **匹配**（signify） |

外加 AdobeARM 更新包内三个载荷（adobearm/adobearmhelper/armsvc，1.824.21.4663，均带内嵌签名）。
**结论：ARM→AGS 全链条无篡改注入点。**（诚实记录：自研验签脚本对两个 32 位 GC PE 出现摘要误报，signify 权威复验为匹配；脚本局限已记录。）

## 2. ARM 活体通道情报（安装后真正运行的更新面）

清单 MSI 不是摆设——是 ARM 的**运行期目录**：

- **ARM 自更新**：`ARM.msi` → `https://armmf.adobe.com/arm-updates/win/ARM/1.8.x/AdobeARM_1824214663.msi`，安装参数 **`/qn`（完全静默）**，版本 `1.824.21.4663`；
- **MSI 引擎黑名单**：`MSIBlacklist=2;16;17;18;19;52;53;54;63;68;81;…;97`（规避已知 msiexec 缺陷版本，防御性设计）；
- **产品更新目录**：`AcrobatDCUpd*.msp` 经 `ardownload3.adobe.com` 分发；模块行包含 **NGL**（Next Generation Licensing）与 **HBEOLSDK** 模块；
- **AppsInUse 关闭清单**：`Distiller;AdobeCollabSynchronizer;AcrobatTrayIcon;PDApp.exe`（更新前强制关闭面）。

**ARM 载荷能力审计**（修正最初认知）：

| 二进制 | 网络 | 定性 |
|---|---|---|
| armsvc.exe（**SYSTEM 服务**） | **零网络导入** | 纯 `ShellExecuteExW` 派生器——服务本体不上网，只启动 adobearm.exe |
| adobearm.exe | wininet 全套 + `URLDownloadToFileW` + BITS 策略感知 | 真正的下载器/更新器；`SetWindowsHookExW` 为 UI 消息钩子（非注入）；`CryptProtectData`（DPAPI） |
| adobearmhelper.exe | 零网络 | 关闭应用/清理辅助（`KillProcess` 系导出） |

**更新遥测实锤**（adobearm.exe 宽字符串）：`LogReportDataIntoAnalytics`、`analytics disabled for product version`（版本门控）、`ArmReport.ini`、`ReportSessionData`、注册表读取 `Software\Adobe\CommonFiles\UsageCC\` 与 `Software\Adobe\CommonFiles\Usage\`（Adobe 使用统计配置位）。**ARM 会在更新会话后上报分析数据，除非产品版本门控禁用。**

## 3. AGS 本体：从安装器 ZIP 到二进制

**R2 结论修正**：AGS 本体并非运行期下载——`AGSService.exe`/`gccustomhook.exe`/`gc_dispatchtable` 就装在安装器 ZIP 的 **Intermediate.cab**（File 表 `GC_AGSService`/`GC_CustomHook` 组件，目录 `[ACROBAT]\GC\customhook\`）。R2 所说"不在 MSI 静态载荷"仅指 MSI 数据库文件，修正为"**在包内 CAB、经条件组件安装**"。

**引入与阻断机制（本轮最重要的处置发现）**：

```
CustomAction: ExecuteGCHook (Type 65, 序列 6130)
  命令: gccustomhook.exe --source=Acrobat
  条件: Not (SKIP_GC) And Not REMOVE And SINGLECLIENTAPPMODE="1"
Component: GC_AGSService / GC_CustomHook → Condition: "Not SKIP_GC"
```

→ **`msiexec /i AcroPro.msi SKIP_GC=1` 即可整体阻断 AGS 安装**（官方内置开关，无需事后清理）。

**二进制能力面**：

| | AGSService.exe 6.6.0.204 | gccustomhook.exe 6.0.0.168 |
|---|---|---|
| 描述 | "Adobe Genuine Software Integrity Service" | "Adobe GC Custom Hook" |
| 网络 | **WinHTTP 全套** | **WinHTTP 全套** |
| 服务 | CreateServiceW/DeleteServiceW/ControlService（自管理） | + **StartServiceW**（装完即启） |
| 进程 | CreateProcessW + 进程枚举 | — |
| 令牌 | OpenProcessToken + **SetTokenInformation** | — |
| 注入/键盘钩子/截屏/凭据 | **零** | **零** |

## 4. AGS 网络面全景（“侵入式遥测”的具体答案）

**`gc_dispatchtable`（XML，带 `DTSignature` 签名保护，防篡改调度表）**——两个 scope（PR/GM）各含：

| 端点 | 功能 | 定性 |
|---|---|---|
| `genuine.adobe.com/server/services/update/v1` | CFU 自更新 | 更新通道 |
| `genuine.adobe.com/server/services/rules/v1`、`/v2` | 拉取验证规则 | 策略下发 |
| `genuine.adobe.com/server/services/check/v1` | **PostRulesData——回传机器数据做真伪校验** | **指纹上报** |
| `genuine.adobe.com/server/services/installation/v1` | 安装通知 | 事件上报 |
| `genuine.adobe.com/server/services/consequence/appevents/v1` | 应用内事件 | 遥测 |
| `genuine.adobe.com/server/services/machineevents/v1` | **机器级事件上报** | **遥测核心** |
| `genuine.adobe.com/server/services/clientconfiguration/v1` | 客户端配置拉取 | 配置 |
| `na1e.services.adobe.com/hsmessaging/rest` | ETSEndPoint（宿主状态消息） | 消息通道 |

二进制内嵌域名补充：`genuine.adobe.com/server/services/dispatch/v1`（调度表自身拉取）、`prod.adobegenuine.com`、`stage.adobegenuine.com`、`gc-stage.licenses.adobe.com`（预发环境）。**两个二进制均含 `MACAddress` 关键词**——机器指纹包含 MAC 地址。静态链接 OpenSSL + zlib（字符串中的 openssl.org/winimage.com 为库归属声明）。

与企业端点官方文档（530 域清单）交叉：AGS 官方段 = `genuine.adobe.com` + `prod.adobegenuine.com` + `gocart-web-prod-*.elb.amazonaws.com` + `lcs-cops.adobe.io` + `agsupdate.adobe.com` —— **与二进制证据一致闭合**。

## 5. CPU 异常疑点的机制定位

社区投诉（R3 检索：2025-2026 持续）指向 AGSService.exe 高 CPU。本轮静态证据给出机制候选：AGSService 自带**进程枚举**（`CreateToolhelp32Snapshot` 循环扫描）+ WinHTTP 轮询（`machineevents`/`check` 周期上报）+ `runThisThread` 字符串暗示常驻线程。**静态证据不能证明卡顿，但"周期轮询 + 进程快照 + 网络重试"的组合与用户观察一致**；真伪校验失败或网络受限（企业防火墙掐断 genuine.adobe.com）时重试风暴是高 CPU 的最合理解释——这与"禁网环境投诉更多"的社区模式吻合。

## 6. 风险定级（R6 增量）

| 面 | 定级 | 依据 |
|---|---|---|
| ARM/AGS 供应链完整性 | **低** | 全链 Adobe 签名 + 摘要逐字节匹配 |
| AGS 隐私面 | **中** | MACAddress 指纹 + machineevents/check 定期回传；签名合法≠内容透明 |
| AGS 可控性 | **中→低** | `SKIP_GC=1` 官方阻断开关；端点集中于 `genuine.adobe.com`/`na1e.services.adobe.com` 两域，防火墙可收敛 |
| CPU 异常 | **中（机制假设）** | 进程枚举+轮询重试风暴假设，与社区投诉模式吻合，待实机验证（Collect-AdobeAutostart.ps1 可采集） |

**处置矩阵更新**（叠加 analysis/04）：
1. 安装期：`msiexec /i AcroPro.msi SKIP_GC=1`（阻断 AGS 组件）
2. 已装环境：禁用 `AGSService` 服务 + 防火墙阻断 `genuine.adobe.com`/`na1e.services.adobe.com`/`adobegenuine.com` 域族
3. 更新面：`PolicyDisableUpdater`/FeatureLockdown（见 analysis/04）+ 可选 BITS 策略

## 7. 遗留与下一轮（R7）

- `SingleClientServicesUpdater.exe`（20MB，Core.cab）——Acrobat 服务更新器，未审计；
- ZIP 内 `AcrobatDCx64Upd26002221931.msp`（953MB）补丁变换分析；
- 范围取证工具链（zip_range_probe/cab_index/range_fetch）已入库，可复用于任意 Adobe ZIP 包。
