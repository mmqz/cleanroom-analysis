# acrobat-audit WORKLOG

遵循被审计对象同源的证据纪律（agents-md-audit 系列规范）：记录做了什么、证据在哪、什么没能做到。

## 轮次 1 · MSI 数据库解码（Task 4）

- olefile 枚举 AcroPro.msi（SHA-256 089dc9a2…）：138 流 / 13,050,541 字节；36 PE + 3 BMP + 1 JPEG + 3 系统 + 95 数据库流；无标准表名（InstallShield 2012 流名混淆）
- 转储全部流（msi_extract.py）；StringData 定位为 #067（781,290B，100% 可打印，无 NUL 分隔）
- 字符串池格式破解：#068 = (u16 长度, u16 引用计数) × 29,429；假设检验唯一闭合（Σ长度 == 781,290 == StringData 字节数）
- 表流物理格式逆向：列块布局（column-major）+ u16 细胞 + 0x8000 整数标记 + 0=NULL
- 内容特征匹配定位 11 张表并全量重建：Property(313) / CustomAction(336) / IES(252) / Binary(39) / ServiceControl(7) / Media(6) / Directory(806) / Component(2,308) / Registry(3,306) / File(5,280) / _Tables(78)
- PE 身份识别：ADelRCP×2 / ArmInst / AbcpyDll / AdobeIsf / AcroSup64 / AcroTgts / ANCUtili / InstUtil / InstOOBE / InstPDFM / PrintInf / PerfInst / EMIOrien / IWActs / IS 系×3 / _IsIcoRes×17
- 诚实记录：CustomAction/File 存在变长列异质段（逐行对齐不完美，聚合可信）；流名混淆算法未破解（被内容判定绕过）

## 轮次 2 · 安装行为面（Task 5）

- ServiceInstall（#092, 16 细胞）：AdobeARMservice / "Adobe Acrobat Update Service" / "Adobe Acrobat Updater keeps your software up to date." / ARM_armsvc_exe 组件
- ServiceControl 7 行全表：ARM 安装时停(2)+卸载时删(128)；stisvc/Spooler/wsearch 安装期停启
- Registry 3,306 行过滤：Run 键 `Acrobat Assistant 8.0` = Acrotray.exe（唯一传统自启动）；ElevationPolicy(AppName=AdobeCollabSync.exe)；WER RuntimeExceptionHelper → CRClient.dll；ARM 配置键群（Check=72h / iCheck / PolicyDisableUpdater / ManifestURL×3）
- CustomAction 336 聚合：Type 51×54 / 65×24 / 3137×22 / 19×3；Source 引力中心 ADelRCP×80；EXE 引用 acroservicesupdater.exe / SingleClientServicesUpdater.exe / acrobat_sl.exe / gccustomhook.exe / FullTrustNotifier.exe 等
- IES 252 行：Adobe 定制动作分布于标准序列，未见休眠/延时触发器
- File 5,280 行 / 55 EXE-DLL；setup.ini（26 语言 + PATCH=AcrobatDCx64Upd2600221931.msp）；ABCPY.INI（OEM 部署模板）

## 轮次 3 · 网络遥测面（Task 6）

- Property 表端点：armmf.adobe.com ×7（4 manifest + 3 ARM 自更新 MSI）+ ARMCheckFrequency=72 + DEFAULT_UPDATE_MODE=3 + ARPURLUPDATEINFO
- 36 PE + setup.exe strings：仅证书链 URL + adobe.com/go 帮助链接 + ADelRCP 内 clients2.google.com（CEF 残留引用）；安装器静态载荷无遥测端点
- 外部检索 8 组（2026-10-09）：AGS 无停用公告、GCClient 2025-2026 高 CPU 投诉持续、CC 再注入、官方卸载链不完整；armsvc 描述与 file.net 逐字一致（交叉验证闭合）；官方 endpoints 文档 + FeatureLockdown 处置路径定位
- 风险分级：安装器低 / ARM 中（自更新）/ AGS 中（登录校验+CPU）/ 运行时遥测中

## 轮次 4 · 仓库整合（Task 7）

- tools/msi_audit.py 整合管线（提取→池→表→行为→PE 挖掘），仓库内复现验证：10/11 表字节级一致（Binary 为预期结构增强）
- tools/Collect-AdobeAutostart.ps1：Windows 只读自审计收集器（任务/服务/Run键/进程/DNS缓存/文件签名）
- 产出 4 篇分析 + README + 证据包（11 表 JSON / 流清单 / PE 元数据 / 字符串挖掘 / 8 组检索）
- source/ 收录 AcroPro.msi(13MB)+setup.exe+双 INI+SHA-256 清单（保证完全可复现）
- GitHub 推送：**BLOCKED**——token 已在上一会话用毕且按纪律未持久化（磁盘检索 ghp_ 无残留命中，仅脚本内检测正则）；待用户提供新 token 或自行推送（参见 tools/push_to_github.sh）

## 轮次 5 · 二进制能力面（Task 8）

- 新工具 tools/msi_pe_audit.py（olefile+pefile 双依赖）：36 PE 全量解析导入/导出/节区熵/时间戳/安全目录 + 能力分类(20类) + ASCII/UTF16LE 双通道字符串挖掘
- **零网络导入**（wininet/winhttp/ws2_32/urlmon/winsock 全缺席，36/36）+ **零注入 API**（WriteProcessMemory/CreateRemoteThread/SetWindowsHookEx 全缺席）；节熵全部 <7.2（无加壳）
- R1 身份修正：s023(606KB,60导出)=AdobeIsf 引擎(版本块损坏: ProductName="NoFeatureError"), s025=AcrobatCA 主力, s028=真 ADelRCP(CoexistAcrobat/HandleInterOp)
- R1 两个未知 EXE 落定：s030=BlockEntryPoints(AcroInstEntryPointBlock.log/Global\Acro Update Mutex/MSIServer忙探/SDDL低完整性标签), s033=ExecuteInst_Cmd(PDB路径铁证 D:\DCB\CBT_Main\Acrobat\Installers\...; msiexec二级安装 REINSTALL=ALL IS_SEC_INSTALL=1; cmd /c del 临时清理)
- ADelRCP s023 唯一非证书 URL = clients2.google.com/service/update2/crx → Registry 表交叉定位到 HKLM\SOFTWARE\Google\Chrome\Extensions\[CHROME_EXT_ID].update_url(Chrome扩展标准注册) + NativeMessagingHosts\com.adobe.acrobat.chrome_webcapture
- crypto=仅 CryptHashData(哈希非加密); privilege×4=安装期令牌调整; proc_enum×3=CloseApplications/KillProcess(关运行中进程); com_schtasks×6 中真实任务创建者=ArmInst(AddTask/RemoveTask)
- 时间戳分群: 2012-04 Flexera×19 / 2021-01-26 ArmInst / 2021-02-02 Adobe CA×16(与FileVersion 21.1.20135 吻合)——CA 代码库 2021 定型复用于 2026-09 产品
- 证据: evidence/pe-capability/{pe_capabilities,capability_matrix,strings_hits}.json; 分析: analysis/05-二进制能力面.md
- 推送状态: 仍 BLOCKED(token 未随本轮消息提供; push_to_github.sh 就绪)

## 轮次 6 · AGS 取证（Task 9-10）

- 推送先行: Bearer/token方案401→三方案对照定位根因(本网关git端点仅接受Basic)→Basic内存头推送成功, 远端HEAD=2e1ac15一致, 零token落盘
- ARM活体通道: 下载3个manifest MSI+AdobeARM_1824214663.msi自update MSI全部signify验签通过(Adobe EV链, authentihash逐字节匹配); 清单目录解出: msiexec /qn静默自更新/MSI引擎黑名单/NGL+HBEOLSDK模块行/AppsInUse关闭清单
- ARM载荷: libarchive(ctypes)解LZX CAB得adobearm/adobearmhelper/armsvc; armsvc(SYSTEM服务)零网络导入(纯ShellExecuteExW派生); adobearm=wininet+URLDownloadToFile+BITS策略+LogReportDataIntoAnalytics更新遥测(UsageCC/Usage配置键)
- AGS获取: ZIP范围取证(中央目录298KB→6个CAB的CFFILE索引扫描→Intermediate.cab 128MB), 未整包下载; ETag内MD5与R0记录一致证明同源文件
- GC三件套: AGSService.exe 6.6.0.204(3.1MB)+gccustomhook.exe 6.0.0.168(2.4MB)+gc_dispatchtable(2KB); signify验签双双匹配(自研脚本对32位PE误报已如实记录)
- 网络面全景: dispatchtable带DTSignature签名保护; genuine.adobe.com/server/services/{update,rules v1/v2,check,installation,appevents,machineevents,clientconfiguration}/v1 + na1e.services.adobe.com/hsmessaging/rest; 二进制内嵌dispatch/v1+prod/stage.adobegenuine.com; MACAddress指纹关键词
- 处置发现: ExecuteGCHook CA(Type65,序列6130)与GC_AGSService/GC_CustomHook组件条件均为"Not SKIP_GC" → msiexec SKIP_GC=1官方阻断开关
- R2结论修正: AGS本体在Intermediate.cab(File表GC_*组件, [ACROBAT]\GC\customhook)非运行期下载; "不在MSI载荷"仅指数据库文件
- 企业端点官方文档530域清单交叉验证: AGS段(genuine.adobegenuine/prod.adobegenuine/gocart-web ELB/lcs-cops/agsupdate)与二进制证据一致闭合
- 提交ded0987: analysis/06 + 工具5件(zip范围取证三件套/libarchive CAB/MSI验签) + 证据(ags-forensics/+检索JSON) + source/ags/(7.8MB工件)
