# Worklog

---
Task ID: 1
Agent: main (Super Z)
Task: 验证 Adobe Acrobat Pro 两个 trials.adobe.com 下载链接的安全性并确认版本

Work Log:
- HEAD 请求验证两个 URL 有效（Win: 1.58GB zip, Mac: 1.59GB dmg），服务器 AkamaiNetStorage，Last-Modified 2026-09-18/09-08
- 验证 trials.adobe.com TLS 证书：CN=*.adobe.com, O=Adobe Inc (San Jose)，DigiCert 签发，2026 年有效
- 下载文件头/尾片段：ZIP 魔数 PK\x03\x04 ✓；DMG 含 Apple Driver Map / Apple Partition Map / Apple_HFS + koly UDIF 尾块 ✓
- 完整下载两个文件，MD5 与服务器 Content-MD5 完全匹配（Win: a33cf85176e0b0c64be237c12e46216d; Mac: 31038a04cd514755679d9e33553171d7）
- 解析 Windows ZIP：AcroPro.msi + setup.exe + AcrobatDCx64Upd2600221931.msp + 多语言 .mst，版本 26.002.21931（2026-09-16 构建）
- 编写 /home/z/my-project/scripts/verify_authenticode.py 完整验证 setup.exe 的 Authenticode：
  - 签名者证书 Adobe Inc. (OU=Acrobat DC, San Jose, serial 2748129)，DigiCert Trusted G4 Code Signing RSA4096 SHA384 2021 CA1 签发，链到 DigiCert Trusted Root G4，有效期 2025-10-06~2027-10-05
  - RSA-4096 私钥对 SignedAttributes 签名验证通过
  - messageDigest == SHA256(SPC_INDIRECT_DATA 内容字节) ✓（调试中修正了 CMS 嵌套结构的 DER 提取 bug）
- 文件摘要校准实验：先下载微软 procexp64.exe（已知有效签名）校准算法，发现正确算法是"跳过"而非"清零"校验和/安全目录项/证书表（参照 signify 源码）
- 用 signify(python3.13) 计算authentihash：setup.exe = fae57bc4...3cc079，与签名内嵌摘要完全一致 → 文件未被篡改
- setup.exe 的 RFC3161 时间戳：2026-02-16 23:48:22 UTC（引导程序 2 月签名，配合 9 月补丁属正常模式）
- 用 signify 的 SignedMsiFile 验证 AcroPro.msi（af10c6c6...）和 AcrobatDCx64Upd2600221931.msp（47ce3671...）签名摘要均与文件一致 → 未被篡改
- CAB 文件（Core/Intermediate）无内嵌 PKCS#7 签名（Adobe 布局常态，靠 TLS 通道+已签名安装器保护）
- Web 搜索确认：26.002.21931 = Adobe 2026-09-17 发布的 out-of-cycle 热修复更新，为当前分发版本（get.adobe.com 亦为此版本）
- 清理大文件，保留 scripts/verify_authenticode.py、extracted/{setup.exe, AcroPro.msi, *.ini} 作为证据

Stage Summary:
- 结论：两个链接均为 Adobe 官方（TLS 证书+Akamai CDN+MD5+数字签名四重验证）最新版 Acrobat Pro DC 26.002.21931 安装包，Windows 包内 setup.exe/AcroPro.msi/.msp 三级签名全部验证通过，文件自签名以来未被篡改，可安全使用
- 关键证据文件保留在 /home/z/my-project/scripts/extracted/，验证脚本 /home/z/my-project/scripts/verify_authenticode.py 可复用
- Mac DMG 结构验证通过（UDIF+koly+MD5），内部 app 代码签名需 macOS 端 Gatekeeper 验证（Linux 无法挂载）

---
Task ID: 2
Agent: main (Super Z)
Task: AGENTS-EN.md 完整架构级安全审计（用户要求：全量、自上而下、警惕休眠恶意行为、成果上传GitHub、回答软件设计/组件/架构）

Work Log:
- 字符级取证(scripts/agents_forensic_scan.py): 11类隐藏载荷攻击全阴性(零宽U+200B-200F/Bidi U+202A-202E/Tag U+E0000-E007F/同形字/ANSI/Base64/Hex/URL/注入语义/控制字符/逐节熵4.0-4.8均匀)；非ASCII仅47个→+7个—+4引号
- 结构提取(scripts/agents_structure_extract.py): 38节点树、59条指令句(36一般/9写/7验证/5技能/2git/2网络/1安全门)、25代码块中仅6条真实命令且全为git只读
- 供应链验证: API限流403后改用git ls-remote+浅克隆7仓库(共250MB,保存于agents-audit/_upstream)；7仓库全部真实活跃(2026-06~10均有提交,3个一周内活跃)；内容扫描命中逐条核查全部为防御性代码(vercel bash.ts凭据黑名单、PracticalSwan skillguard注入扫描器、bun官方安装脚本)
- 交叉验证: open-agents的CLAUDE.md→AGENTS.md软链证实跨宿主惯例；被审计文件与其自带AGENTS.md无派生关系(44共同行均为短语)
- 架构逆向: 重建六层架构(宿主运行时/代理内核/持久状态/工作流引擎/质量保证/技能插件/Git交付)、16个组件、完整数据流、15个休眠触发器、7条设计哲学
- 产出agents-audit/仓库: README+5篇分析MD+4个证据JSON+4个可复现工具+源文件副本+WORKLOG(遵循被审计文件§21规范)；工具已相对路径化并验证仓库内可复现；git init+commit 43b41b4
- GitHub推送: 环境无凭证(无gh/token/SSH,泄露token未存储应已撤销)，网络连通正常；按被审计文件§20纪律如实报告push状态=BLOCKED(待用户提供新token或自行推送)

Stage Summary:
- 核心答案: 该文件不是软件而是"AI编程代理宪法"(agents.md惯例系统提示词)，定义六层/16组件的代理运行时：仓库即数据库、证据优于断言、YOLO有界自治、15个条件触发子协议
- 安全结论: 字符层零载荷、行为面全部工程纪律导向、唯一供应链入口(§6)当前全良性、总体风险低/中低；建议宿主层硬机制兜底YOLO+锁定外部仓库commit+补提示注入防御条款
- 交付物: /home/z/my-project/agents-audit/ (git仓库,commit 43b41b4,待推GitHub)

---
Task ID: 3
Agent: main (Super Z)
Task: 用用户提供的token推送仓库至GitHub + 深化各部分分析

Work Log:
- 验证token(账号mmqz,经典token,权限极广含repo/delete_repo/admin:org——已提醒用毕撤销)
- 创建仓库mmqz/agents-md-audit(公开)并以http.extraheader方式推送(未在.git/config持久化token)
- 推送1: 43b41b4(阶段1全部成果) → git ls-remote验证一致
- 实抓agents.md官方规范(60k+项目/30+宿主)作为08对照基准: 官方=描述性项目说明书, 被审计文件=规范性代理行为法典, 二者互补分层
- 新增tools/agents_metrics.py: 量化画像2259词/≈3050token/42否定式/35条件式/TTR0.349/135列表行/21代码块
- 新增analysis/06上下篇: 25单元×意图/机制/深析三层逐节解析+三个结构韵律(管线镜像/双章定则/否定密度曲线)
- 新增analysis/07: 六项提示工程技术(首尾双锚/伪代码策略语言/休眠触发/封闭词汇表反幻觉/反断言三连/统计性负面清单)+逐节否定密度地图
- 新增analysis/08: 四层生态定位(官方规范层-项目指南层-行为宪法层★被审计文件-宿主运行时层)+三点遗留差距(无注入防御/无宿主机制对接/无版本元数据)
- 提交89cfcae(8文件+710行)并推送, git ls-remote验证一致

Stage Summary:
- 仓库https://github.com/mmqz/agents-md-audit已含两次提交(43b41b4+89cfcae), 9篇分析MD+5证据JSON+5可复现工具
- 待办: 用户需撤销已泄露的ghp_ token(权限含delete_repo/admin:org,风险极高)
- 可选下一步: 对抗性红队测试或生成补防御条款的改进版AGENTS.md

---
Task ID: 4-7
Agent: main (Super Z)
Task: 按用户指令用token环境继续Acrobat Pro深度分析(至少3轮), 成果入Git仓库

Work Log:
- 轮次1(Task4) MSI数据库逆向: olefile转储138流(36PE/3图/95数据库流); 发现InstallShield 2012流名混淆(无明文表名); StringData定位#067(781,290B); 字符串池格式破解#068=(u16长度,u16引用计数)×29,429, Σ长度唯一闭合检验; 表流列块布局+u16细胞+0x8000整数标记; 内容特征匹配重建11表(Property313/CA336/IES252/Binary39/ServiceControl7/Media6/Directory806/Component2308/Registry3306/File5280/_Tables78); PE身份全识别(ADelRCP/ArmInst/AbcpyDll/AdobeIsf等19个Adobe DLL+17个_IsIcoRes)
- 轮次2(Task5) 安装行为面: ServiceInstall=AdobeARMservice("Adobe Acrobat Update Service"+armsvc); ServiceControl7行(ARM安装停/卸载删+stisvc/Spooler/wsearch安装期停启); Registry3306行过滤→Run键Acrobat Assistant 8.0=Acrotray.exe+WER挂接CRClient.dll+ElevationPolicy(AdobeCollabSync)+ARM配置键群(Check=72h/PolicyDisableUpdater); CA336聚合(ADelRCP承载80动作/Type51×54/65×24); ArmInst.dll字符串证实计划任务"Adobe Acrobat Update Task"; CA引用gccustomhook.exe定位AGS耦合点(AGS本体不在MSI静态载荷)
- 轮次3(Task6) 网络遥测面: Property表ARM端点全量(armmf.adobe.com×7: 4manifest+3自更新MSI+72h周期); 36PE+setup.exe字符串挖掘=仅证书链+帮助链接(安装器静态载荷零隐藏端点); web-search 8组外部检索(AGS无停用公告且2025-2026高CPU投诉持续/CC再注入/官方卸载链不完整; armsvc描述与file.net逐字一致交叉验证闭合; 官方endpoints文档+FeatureLockdown处置路径)
- 轮次4(Task7) 仓库整合: tools/msi_audit.py整合管线(仓库内复现验证10/11表字节级一致)+Collect-AdobeAutostart.ps1(Windows只读自审计)+push_to_github.sh(token不落盘); acrobat-audit仓库40文件(4分析MD+25证据+5源工件+3工具)commit 8aba2c5
- GitHub推送: BLOCKED——token仅存在于已压缩的上一会话上下文中, 磁盘检索ghp_仅命中脚本内检测正则(前次会话按纪律未持久化); 已提供push脚本待用户提供新token

Stage Summary:
- 核心答案: 官方安装器(签名链干净)一次默认安装落下1个SYSTEM服务+1个计划任务+1个Run自启动+ARM自更新通道(单域armmf.adobe.com)+336定制动作(无休眠触发器)+3306注册表写入; AGS/遥测为运行期引入面
- 交付物: /home/z/my-project/acrobat-audit/(本地git仓库, commit 8aba2c5, 待token推送; push脚本已就绪)
- 风险定级: 安装器低/ARM中(自更新)/AGS中(登录校验+CPU+再注入)/运行时遥测中; 处置矩阵见analysis/04

---
Task ID: 8
Agent: main (Super Z)
Task: 用户指令: token只用于推送成果(沙盒随时清空不用再提醒)+设计并执行下一轮Acrobat分析(第5轮)

Work Log:
- 确认token策略: token仅用于推送工作成果, 不再提醒撤销; 本轮消息未附token, 推送保持BLOCKED(push_to_github.sh就绪, token只经环境变量单次传递)
- 设计第5-7轮计划: R5=MSI内嵌36PE二进制能力面 / R6=AGS(AdobeGClient)官方包取证 / R7=.msp补丁变换+处置矩阵终版
- R5执行: 新工具acrobat-audit/tools/msi_pe_audit.py(olefile+pefile), 36PE导入/导出/节熵/时间戳/能力分类/双通道字符串
- 核心发现: 36/36零网络导入+零注入API; 节熵全<7.2无加壳; R1身份修正(s023=AdobeIsf引擎坏版本块/s025=AcrobatCA/s028=真ADelRCP); 两个未知EXE落定(s030=BlockEntryPoints, s033=ExecuteInst_Cmd含PDB路径铁证); ADelRCP唯一非证书URL=clients2.google.com CRX更新→Registry表交叉定位为Chrome扩展标准注册+NativeMessagingHosts; crypto仅哈希; 时间戳三群(2012 Flexera/2021 Adobe CA×16与FileVersion 21.1.20135吻合)
- 产出: analysis/05-二进制能力面.md + evidence/pe-capability/三JSON + 工具; README更新; commit 2e1ac15

Stage Summary:
- 安装器证据链四层闭合(数据库→行为→网络→二进制), 二进制层风险定级=低
- 新知情点: 默认安装注册Chrome扩展+原生消息宿主; Adobe构建管线存在坏版本块卫生缺陷
- 下一轮R6: AGS官方分发包取证(直击CPU异常与侵入式遥测根因)

---
Task ID: 9
Agent: main (Super Z)
Task: 用token推送acrobat-audit至GitHub(推送成功) + 启动R6 AGS取证

Work Log:
- 推送诊断: 首次Bearer方案401→token方案仍401; 定位根因=本网关git smart HTTP端点仅接受Basic方案(curl三方案对照: token/Bearer→401, mmqz:token Basic→200)
- 修订push_to_github.sh: base64(user:token)仅内存计算经http.extraheader传递, 不写任何文件
- 推送成功: mmqz/acrobat-pro-audit main分支, 远端HEAD=2e1ac15与本地一致(ls-remote验证), .git/config零token, 磁盘ghp_检索无残留
- 仓库现含R1-R5全部成果(5篇分析+evidence/pe-capability+tools)

---
Task ID: 9-10
Agent: main (Super Z)
Task: 用户指令: 推送结果(附token) + 继续深入分析 → 执行R6 AGS取证

Work Log:
- 推送调试: Bearer 401→token方案仍401→curl三方案对照定位根因=本网关git smart HTTP仅接受Basic(user:token)→修订push_to_github.sh为内存计算Basic头→推送成功(远端HEAD与本地一致, ls-remote+API树验证, 零token落盘)
- R6-A ARM活体通道: 3清单MSI+AdobeARM更新MSI下载, signify验签全通过(Adobe EV/DigiCert链+authentihash逐字节匹配); 清单目录解构(msiexec /qn自更新/MSIBlacklist/NGL模块/AppsInUse); ARM载荷3PE能力审计(armsvc零网络, adobearm=wininet下载器+LogReportDataIntoAnalytics更新遥测)
- R6-B AGS本体: ZIP范围取证(中央目录298KB+6CAB索引扫描4MB+Intermediate.cab 128MB, ETag内MD5与R0一致)→GC三件套; AGSService/gccustomhook signify验签匹配(自研脚本32位PE误报已记录); gc_dispatchtable带签名网络面全景(check/machineevents上报+MACAddress指纹); ExecuteGCHook CA+GC组件条件"Not SKIP_GC"→SKIP_GC=1官方阻断开关
- R2结论修正: AGS在Intermediate.cab非运行期引入; 企业端点文档530域交叉闭合
- 提交ded0987+246fc0f并推送成功(R6全部成果上线)

Stage Summary:
- 原始问题闭环: "AdobeGClient侵入式遥测"=genuine.adobe.com/server/services/{check,machineevents,appevents}+na1e hsmessaging+MACAddress指纹; CPU异常机制假设=进程枚举+网络轮询重试风暴(静态证据+社区模式吻合)
- 交付: analysis/06-AGS取证.md + 5工具(范围取证三件套/CAB提取/MSI验签) + source/ags/(7.8MB含GC本体) + 证据包
- 待办R7: SingleClientServicesUpdater.exe(20MB)审计 + .msp补丁变换分析 + 处置矩阵终版

---
Task ID: pdfcraft-analysis-1
Agent: Super Z (main agent)
Task: 对 storytold/pdfcraft 做净室架构深度剖析（MD 文档，聚焦数据完整性/渲染管线/格式转换，不含代码，供用户后续 Rust 自研参考）

Work Log:
- 浅克隆 pdfcraft 仓库到 scripts/pdfcraft_research/repo（37MB，956 文件）
- 通读 ROADMAP.md（平价计量/诚实评估/里程碑 M0-M14/log）
- 提取全部 30 个 crate 的 lib.rs 头注释与 16 个 README 的设计语义
- 从 xtask/src/layers.rs 提取 CI 强制执行的 L0-L8 分层表与依赖规则
- 从 vendor/README.md 提取 34 条敌意输入加固补丁清单
- 核实数据完整性机制：cos 增量/全量写出、COW、engine working file/rebase/恢复快照
- 核实渲染管线：RenderPool、hayro bootstrap（ADR-0004）、Device 文本提取、Type3 字体路线
- 核实格式转换：filters 行为表、xfdf/export/print/ocr/optimize 语义
- 撰写并保存 /home/z/my-project/download/PdfCraft架构深度剖析.md（359 行，约 1.7 万字）

Stage Summary:
- 交付物：download/PdfCraft架构深度剖析.md（0 净室纪律/1 分层架构/2 数据完整性/3 渲染管线/4 格式转换/5 质量体系/6 施工映射/附录 ABC）
- 关键事实：工作区 50 槽位分层表（30 已实现+15 预留）、CI 机器执行分层规则、增量写保持原始字节不变以保签名、RenderPool 六设计点、Type3+ToUnicode 的 CJK 零字体资产路线、34 条 vendor 加固补丁、oracle 网（qpdf/pdftotext F1 0.98/pdfsig/Poppler/pdf.js 958 文件语料）

---
Task ID: gap-b
Agent: general-purpose 子代理
Task: 调研开源 PDF 生态各项目的能力边界与缺口（不写代码/不写交付文档，纯网络调研）

Work Log:
- 读取 worklog.md：主代理正撰写"开源 PDF 生态缺口分析"，前期已完成 Acrobat 26.002.21931 安装包五层取证（R1-R6）与 pdfcraft 架构剖析，本任务提供开源侧能力边界事实清单
- z-ai web_search 共 19 次查询（num=5，含 1 次 429 限流重试、查询间 sleep 8s 串行避限流），覆盖 6 大调研域：①qpdf/pikepdf ②pdf.js(XFA/JS/注释/性能) ③MuPDF/PDFium(许可/渲染/编辑API) ④Poppler/Ghostscript(AGPL/PDF-A/pdfsig) ⑤商业独占域 8 子项（preflight/色彩/Tagged PDF/PAdES-LTV/redaction/视觉diff/OCR/导出Word） ⑥Acrobat 2026 AI 功能面
- 证据源分级：官方一手源优先（qpdf GitHub、mupdf.com、artifex.com、ghostscript.com/readthedocs、pdfium.googlesource 头文件、ocrmypdf readthedocs+issue#1696、docs.verapdf.org、poppler 邮件列表、helpx/blog.adobe.com、pdfa.org）；商业对比文（nutrient.io/pdf4.dev/pdfix.net/IDR）仅作旁证并标注立场
- 汇总为 6 节结构化能力边界清单（事实/推断分离、每条附 1 URL），直接返回主代理，未产出独立交付文件（遵任务约束）

Stage Summary:
- 总体格局（推断）: 开源生态在"结构层"成熟（qpdf/pikepdf 对象级变换与修复、pdf.js 网页渲染、MuPDF/PDFium 高速渲染+基础注释/表单/增量保存），在"语义与行业工作流层"系统缺口：印前 preflight 自动 fixup（callas/Enfocus/Acrobat 三强垄断，veraPDF 仅验证 PDF/A/UA 子集）、专业色彩（overprint 软打样仅 Ghostscript 分色设备、DeviceLink/专色预览无对标）、Tagged PDF 树编辑（修复 PDF/UA 错误全是商业工具）、PAdES LTV 全链（pdfsig 无吊销检查、AATL 信任缺失）、语义级文档比较（开源仅 pdftoppm+像素diff DIY）、PDF→Word 高保真（2026 实测 Acrobat 第一）、OCRmyPDF 只做归档文本层不做编辑、对话式 AI（Acrobat 2026.10 联网问答/文档转音频视觉）完全无对标
- 许可证约束事实: MuPDF=AGPL/商业双许可(artifex)、Ghostscript=AGPL/商业(服务端商用须付费)、PDFium=BSD式+Apache2(宽松)、qpdf=Apache 2.0
- 关键证据锚点: qpdf 官方自述"不渲染不提取文本、无页面内容高层接口"；pdf.js 注释支持 2025 年才逐步引入、XFA 仅有限支持；pdfium fpdf_signature.h 只读枚举签名对象；OCRmyPDF issue#1696 自述面向批量归档、文本层处理需全页光栅化

---
Task ID: gap-d
Agent: general-purpose 子代理
Task: 调研 PDF 格式转换开源格局与保真度差距

Work Log:
- 读 worklog.md 了解背景（主代理《开源 PDF 生态缺口深度分析》、pdfcraft 转换面覆盖：图像/XFDF/FDF/XML/CSV/OCR/打印拼版/启发式 Word-HTML-RTF 导出）
- 16 次 web_search 覆盖 8 个方向：pdf2docx 局限与维护状态、LibreOffice Draw PDF 导入质量、Acrobat/Word 导出 Word 口碑、pdf2htmlEX 维护与字体机制、pdftohtml/pdf.js textLayer 定位、LibreOffice headless 保真度、Aspose 商业排版引擎、Ghostscript PDF/A 与 veraPDF 验证失败模式、callas pdfaPilot 自动纠错、Scribus PDF/X、OCRmyPDF 管线与 Ghostscript 许可证、PostScript 解释器格局、OmniDocBench/DocLayNet/DocX-Bench/DocAccessible 评测基准
- 关键交叉验证：pdf2docx 官方声明停止维护+MIT relicense（PyPI 与 GitHub 双源）；OCRmyPDF maintainers.md 确认 Ghostscript 已非严格依赖；LibreOffice Word 分页差异案例（39 vs 42 页）

Stage Summary:
- PDF→Word：开源主力 pdf2docx 已被 Artifex 弃养（改 MIT），局限=仅文本型 PDF/无旋转文字/复杂表格列合并/RTL 反转；Acrobat 口碑最强（表格识别+字体+内置 OCR）；核心差距在语义重建（表格/多栏/连字/RTL）而非文本提取
- PDF→HTML：pdf2htmlEX（社区分支维持，Docker 分发，FontForge 转 webfont 做像素级重现）≠语义 HTML；pdftohtml 粗糙；pdf.js textLayer 是阅读器选择层非转换器；语义 HTML 开源空白
- Office→PDF：LibreOffice 与 Word 分页系统性不一致；Aspose 优势=自研高保真排版引擎；OOXML 流式格式原理上必须经排版引擎
- PDF/A：veraPDF 验证强但验证≠转换；Ghostscript 生成常被 veraPDF 挑错（DeviceRGB 无 ICC）；callas/Acrobat 有自动纠错合规化，开源没有
- 图像方向（pdftoppm/img2pdf/OCRmyPDF）开源已解决；OCRmyPDF 固定管线 rotate→bg→deskew→clean，新版 Ghostscript 非严格依赖（解 AGPL 耦合），JBIG2 因专利缺
- Ghostscript=开源唯一完整 PS Level3 解释器（30+年），单点垄断
- 基准：OmniDocBench（CVPR25，1651页）为主流，DocLayNet 布局，DocX-Bench 专测 PDF→DOCX 保真度（刚起步），评测整体偏"解析为 Markdown"而非"还原可编辑 Office"
- 缺口八点：pdf2docx 弃养无接棒/语义重建缺失/语义 HTML 空白/高保真 OOXML 排版引擎缺/PDF-A 自动纠错转换器缺/连字-RIL-竖排-旋转重灾/PS 单点+AGPL/还原式评测基准刚起步

---
Task ID: gap-c
Agent: general-purpose 子代理
Task: 调研 Rust PDF 生态各环节现状与空白（为主代理《Rust PDF 生态空白地图》章节提供数据）

Work Log:
- 读 worklog.md 了解背景（pdfcraft 净室剖析已完成，本任务为生态侧调研）
- crates.io API 批量拉取 ~35 个 crate 的下载量/最近更新/版本：lopdf、pdf、pdf-writer、krilla、hayro 系 9 crate、printpdf、pdf_oxide、zpdf、oxidize-pdf、stet、pdfrs、flpdf、pdf-extract、pdfium-render、mupdf、ocrs、rten、rustybuzz、harfrust、swash、ab_glyph、rusttype、unicode-bidi、flate2、miniz_oxide、weezl、underskrift、pdf-signer、prismpdf-crypto、pdfluent-sign、acroform、pdfer_forms 等
- GitHub stars 经 shields.io 获取（直连 API 被限流）：lopdf 2.3k / pdf-rs 1.7k / typst 57k / hayro 786 / krilla 457 / pdf-writer 743 / ocrs 1.9k / pdfium-render 714 / mupdf-rs 209 / pdf_oxide 1.1k
- 12 次 web_search：生态综述、lopdf 内存模型与维护、krilla 特性路线、hayro 设计、pdf-rs render 状态、Rust 加密/签名(PAdES/CMS)、CJK/Type0/CMap、绑定许可证(PDFium BSD/MuPDF AGPL)、XFA/JS、ICC、qpdf 等价物、typst 0.15→krilla 切换
- 抓取并精读关键 README：hayro（自述 most feature-complete、WIP、CPU-only、不支持 knockout/未嵌入 CID 字体）、krilla（PDF/A-1/2/3/4+PDF/UA+tagged、明确排除加密签名）、lopdf（Rust2024+对象流、空密码解密、RUSTSEC-2026-0187 无界递归）、pdf-rs（读改写仍实验性、pdf_render 依赖已归档的 pathfinder）、ocrs（仅拉丁、early preview）
- 发现 2025-2026 新生力量：stet（PostScript L3 解释器，21.5 万下载）、Lege（应用内渲染器，hayro #1358 实测比 pdfium 快 50%~10x）、harfrust（harfbuzz 官方 Rust 接班 rustybuzz）、VellumPDF/flpdf/pdfrs（qpdf 类工具，千级下载）；发现 pdf-render/pdfluent-sign 为 pdfluent.com 商业闭源占名
- 汇总为 9 节结构化报告 + 16 条空白清单，直接随响应交付主代理

Stage Summary:
- 总格局：解析(lopdf/pdf-rs)老而稳但弱在编辑，生成端被 typst 系（pdf-writer/krilla，typst 0.15 已切换 krilla 后端）做到成品级（tagged+PDF/A+子集化），渲染端 hayro 是唯一接近可用的纯 Rust 库（WIP、CPU-only、性能未优化），生产环境实际依赖 pdfium-render(310 万下载,BSD)/mupdf-rs(AGPL) 绑定
- 关键数据：lopdf 2335 万下载仍在维护但全量内存模型+RUSTSEC 递归漏洞+增量保存 /Prev bug；harfrust(738 万/90天) 接班冻结的 rustybuzz；underskrift/pdf-signer 等签名库全部 <半岁且 <1.1 万下载；ocrs 仅拉丁字母
- 真空清单核心：qpdf 级重写/线性化、容错修复、XFA、PDF JS 沙箱、纯 Rust ICC CMM、结构树编辑、PDF/A 验证器、CJK OCR、JBIG2/JP2/CCITT 编码器、加密写出、非 Identity CMap 全链路、GPU 渲染（zpdf wgpu 刚起步）
- 警示：2026 出现大量 AI 风格营销话术新 crate（pdf_oxide "最快/100% 通过"、oxidize-pdf 等），数据需打折看待

---
Task ID: gap-a
Agent: Explore 子代理
Task: 挖掘 pdfcraft 本地仓库的未完成项/短板/缺口证据（只读调查，不改代码）

Work Log:
- 读 worklog.md 前情（pdfcraft-analysis-1 架构剖析），确认仓库 HEAD=7c35809 (2026-10-09)
- 通读 README.md（531 行）与 ROADMAP.md（261 行，含 Estimate/Honest assessment/Milestones M0-M14/Risks/Log 全节）
- 用只读 Python 解析 parity/acrobat-features.toml（830 个 [[feature]]，含 23 个 na）：按 tier×status、area×status、milestone×status 交叉统计；导出 P0 partial 24 项及 notes、P0/P1 planned 113 项清单、na 23 项清单
- Grep 全 crates/ 搜 TODO/FIXME/unimplemented（0 命中，各 lib.rs 有 #![deny(clippy::todo...)] 禁令）；改搜 "not yet/planned/unsupported/deferred" 231 处并按 crate 归类，人工核对 engine/catalog.rs 的 34 个 Planned() 标记
- 精读关键 crate 自我声明：render/lib.rs（hayro+lopdf 双 bootstrap）、xfa/README（Deliberately not done 清单）、js/lib.rs+xfa.rs（Acrobat JS 对象模型子集）、forms/README、sign/README（PAdES 级别+Not yet）、preflight/lib.rs（仅 PDF/A-2b/3b）、a11y/README（31/32 规则）、compare/lib.rs（纯文本 diff）、ocr/lib.rs（LANGUAGES 仅 en）、edit/lib.rs+text.rs（行级替换+字体替换声明）、vendor/README（34 补丁）
- 统计测试面：自产代码 #[test]=1356（154 文件，不含 vendor）；evidence 单条引用=146/417 shipped；外部 oracle 引用仅 4 条（qpdf×2+OpenSSL×2）；自产 Rust LOC≈130.7k vs ROADMAP 自称 72k
- 交叉验证文档漂移：cos/lib.rs "Not yet: encryption" vs M1 已完成；annot/forms README Not-yet 清单 vs ROADMAP log 已做；engine catalog Planned(comment.cloud/stamp) vs TOML shipped；ROADMAP M9 "Missing: B-T/LTV" vs sign/README 已实现 B-T/B-LTA/DSS（parity 标 partial）
- 全程只读（Read/Grep/Glob+只读 Python 解析），未运行任何构建/测试

Stage Summary:
- 总量：parity TOML 现有 830 项（417 shipped/74 partial/316 planned/23 云服务 na），P0 224/251=89.2%（仍有 3 planned+24 partial），P1 175/326=53.7%，P2 17/187=9.1%，P3 1/43=2.3%；ROADMAP 自评 effort 加权仅 30–35%
- 最低类目：OCR 19%、Misc 24%、Optimize 26%、Create 34%、Print 38%；最低里程碑：M14 5%、M2 15%（自研渲染器）、M7 17%（编辑既有文本）、M11 18%（preflight）、M12 23%（XFA/a11y 面板/compare）
- 根基性短板（项目自认）：渲染借 hayro+检查借 lopdf（ADR-0004 bootstrap，34 条 vendor 补丁待上游）；Acrobat 保真度"未测量"；fuzz 持续发现 OOM/hang；OCR 仅拉丁字母无重音（LANGUAGES=[("en","English")]）；XFA 显式绑定/xfa.template 等未做
- 质量会计：146/417 shipped 功能仅 1 条测试证据、外部 oracle 仅 4 处；README 称"200+ tests"、ROADMAP 称"≈600"、实测 1356 个 #[test]（口径不一）
- 文档漂移证据：cos/annot/forms/engine-catalog 存在滞后于实际进度的"未做"声明（反向夸大缺口），ROADMAP M9 行滞后于 sign crate 实际 B-T/LTA 能力

---
Task ID: pdfcraft-analysis-2
Agent: Super Z (main agent)
Task: 用户指令——继续分析开源生态缺失/差异项，用子代理加速，yolo 到完成，产出 MD 深度分析

Work Log:
- 复核前篇交付物 download/PdfCraft架构深度剖析.md 与本地克隆（HEAD 7c35809）可用性，确认研究基线
- 并行派发 4 个子代理：gap-a（Explore，本地仓库缺口挖掘：parity TOML 逐项统计/ROADMAP 诚实评估/crate 自我声明/vendor 补丁/测试会计/文档漂移）、gap-b（general-purpose，qpdf/pdf.js/MuPDF/PDFium/Poppler/GS/商业独占域/Acrobat 2026 AI 的能力边界与证据 URL）、gap-c（general-purpose，Rust crates 全景：lopdf/pdf-rs/krilla/hayro/harfrust/underskrift/pdfium-render 等实测下载量与版本，产出 16 条真空/半真空清单）、gap-d（general-purpose，格式转换格局：pdf2docx 弃养/pdf2htmlEX/LibreOffice/veraPDF/OCRmyPDF/OmniDocBench 基准）
- 汇总交叉验证：parity 口径漂移（830 vs 806 项）、测试数三口径（200+/600/实测 1356）、"72k kept"vs 130.7k 自产行、krilla 生成侧成品级 vs 编辑侧真空的不对称、AGPL 墙与 PDFium 统治生产的因果关系
- 三段式撰写（scripts/gap_doc_part1/2/3.md）并合并为 download/开源PDF生态缺口与差异项深度分析.md（430 行，48.8KB）
- 笔误修正：krilla 星标口径、EATL 西里尔字符混入、5.5 节语句重构

Stage Summary:
- 交付物：/home/z/my-project/download/开源PDF生态缺口与差异项深度分析.md，结构：0 三层判定标准（Acrobat功能面/ISO规范面/工程面）/1 全景分层蛋糕图+中间层真空四成因/2 pdfcraft 金字塔缺口（830 项实测、14 领域表、六大自认短板、146/417 单测试、34 补丁借用债务）/3 Rust 生态 16 条真空清单+新生军现象/4 全生态差异矩阵（9 项目×18 能力域）+AGPL 墙/pdf.js 边界/AI 零对标/5 十大无人区深挖（preflight fixup、语义重建、autotag、LTV、CJK 全链为最高机会项；XFA/完整 AcroJS 建议跳过）/6 红海蓝海错峰决策框架（四条规则+组合拳咬合）/7 结语+证据索引附录
- 核心结论：开源呈哑铃形——结构端（qpdf）与生成端（typst/krilla）成品级、渲染端三巨头、语义中间层全生态真空；pdfcraft 是中间层第一个住户但 effort 仅 30–35%；对自研者的策略是"跟着 pdfcraft 的骨架建，选它没圈的地活"（CJK 全链/autotag+语义重建/PDF-A fixup+LTV/qpdf 等价层四条最空旷谷地）

---
Task ID: fin-a
Agent: Explore 子代理
Task: 挖掘 ADR 全集/治理文件/净室证据链/资产审计体系

Work Log:
- 读 worklog.md 前情（pdfcraft-analysis-1/2、gap-a/b/c/d 已覆盖架构/生态/缺口），确认本轮只挖治理与证据链盲区
- Glob/Grep 全仓搜 ADR：发现 plan/ 在 .gitignore 第 2 行被忽略（CLAUDE.md:12 自述 local-only），ADR 正文不在公共仓库，仅有 6 个编号共 18 处引用（0001×3、0002×1、0004×5、0006×1、0009×7、0010×1）
- 完整读 CLAUDE.md（48 行）与 AGENTS.md（140 行），提取净室红线/Never panic/分层/提交/资产绝对政策/agent 控制条款
- 读 xtask/src/main.rs + gates.rs + .github/workflows/ci.yml，比对治理条款与机器门：xtask ci 8 步 vs GitHub CI 缺 parity 门
- 读 contributors/people.toml（10 条目）与 docs/contributors.md；用只读 Python 解析 contributors.json（252 commits、5 人、4 模型，96% commits AI co-author）
- 读 deny.toml（11 许可白名单+RUSTSEC-2023-0071 唯一豁免带 ADR-0009 理由）与 NOTICE（含 plan/adr/0002、0006 维护义务）
- 只读 Python 解析 ATTRIBUTION.toml：284 条目四段（asset 255/bundled 6/fetched 22/build_input 1），adobe_data=true 恰 2 条与 AGENTS §1.1 闭清单精确一致；核对 ATTRIBUTION.md 313 行为生成物
- Grep 统计 crates/ 下 .rs 规范出处密度：ISO 32000×59、RFC×56、§x.y×123、PDF/A×11、ISO 19444×1、ISO 19005×2；22/30 crate 有引用，8 个零引用 crate 改引公开 Acrobat 文档；Arlington 在 .rs 中 0 命中
- 读 docs/development.md、docs/releasing.md、docs/localization.md；比对 ROADMAP/CLAUDE/AGENTS 漂移点（wasm 条件句过时、Honest assessment 本地化行滞后、contributors.json head 落后 HEAD 两天）
- 全程 Read/Grep/Glob+只读 Python，未运行构建/测试，未写任何仓库文件

Stage Summary:
- ADR 全集对公众不可见：plan/ 整目录 gitignored，公共仓库只有 6 个 ADR 编号的 18 处交叉引用（最大编号 0010，即私有 ADR 至少 10 个，0003/0005/0007/0008 从未被公开引用）——净室"决策可审计性"存在结构性缺口：决策靠编号可追溯但正文不可审计
- 治理三层宪法：AGENTS.md（冲突时优先，资产绝对政策+净室+Never crash）> CLAUDE.md（会话协议+质量门+提交纪律）> craftrules（内部不可见）；机器执行面=xtask ci 8 门+GitHub CI（但 parity 门只在本地、不在 CI）+crate 级 clippy deny
- AI 治理物证：contributors.json 252 commits 中 242（96%）由模型 co-author（Claude Opus 5.5 239/Codex 1/GLM 1），About▸Models 窗口公开展示；people.toml 仅自愿自报条目
- 证据链完备度：ATTRIBUTION.toml 284 条每条含 SHA-256+许可文件+来源 URL；deny.toml copyleft 零容忍+唯一 advisory 豁免附 ADR 理由；Adobe 数据闭清单 2 条精确落地；规范出处标注密度 22/30 crate、§x.y 引用 123 处，但 sign/cos/annot/filters 之外的 crate 稀疏、Arlington 仅存在于文档层（crate 未建）
- 漂移事实：CLAUDE.md wasm 条件句过时；ROADMAP Honest assessment 本地化行（10-05）滞后于 M14 行与 HEAD；contributors.json head d3db998（10-07）落后仓库 HEAD 7c35809（10-09）两天；资产许可白名单（AGENTS）与代码依赖白名单（deny.toml）为两张不同清单（分层设计非矛盾）

---
Task ID: fin-b
Agent: general-purpose 子代理
Task: 调研 pdfcraft 社区健康度与 AI 开发模式可持续性风险

Work Log:
- 读 worklog.md（pdfcraft-analysis-1/2、gap-a/b/c/d 节），确认主代理收官报告缺"社区健康度"与"AI 模式成本/维护性"两块
- GitHub API 拉取 pdfcraft 仓库主档：created 2026-09-30T13:39:37Z、5735★/1997 fork/244 open issues/Apache-2.0/当日仍在 push
- 拉取 releases（5 个：v0.1.0→v0.4.0，7 天内；v0.2.x 名称仍为 PrintCraft，v0.4.0 改名 PdfCraft）、contributors（59 人，echelon 269≈60.6%、前 5 占 73.6%）、近 100 提交作者分布（社区 PR 每日被合并）
- search API 统计 issue/PR 开闭：issue 167 open/75 closed（69% 未关），PR 320 中 229 merged（94.6% 关闭即合并）
- 查 storytold 组织全 43 仓库：2 名成员（echelon=Brandon Thomas，ArtCraft/Storyteller.ai/FakeYou 创始人，2008 年注册 GitHub；bflatastic）；photocraft/pdfcraft/filmcraft/vectorcraft/lightcraft 均于 2026-09-30 同日 13:37-14:09 集中创建（协同发布），10-01/06/07 又加 designcraft/effectcraft/cadcraft/deckcraft/gridcraft/soundcraft/wordcraft/craft-fonts；旗舰 artcraft(2022,10.4k★) 4.5 年仅 2 个贡献者
- 核实 ROADMAP.md 原文：72k 行/97 agent tools/≈600 测试=65-75 agent-hours（Opus 5.5 连续编码，不含人工审查），~1k 行/agent-hour，剩余估 600-1100 agent-hours；本地 Cargo.lock 解析：672 包、零 GPL/AGPL/CC 条目
- 逐一核实依赖许可证：hayro=Apache-2.0(活跃)、vello_cpu=Apache-2.0 OR MIT、ocrs 代码=Apache-2.0、rten=Apache-2.0(活跃)、swash=Apache-2.0(2026-07 后放缓)、lopdf=MIT；ocrs-models README 承认训练数据含 HierText(CC-BY-SA 4.0)且仓库无 LICENSE 文件、HF 模型库无显式许可（灰区）
- z-ai web_search 共 9 次查询：pdfcraft 报道、ArtCraft 批评、GitClear 2026 维护性研究、vibe coding 技术债、HN 讨论、成本基准、创始人背景、AI 项目坟场、AI 代码存活率
- 撰写 5 节结论并交付主代理

Stage Summary:
- ① 活跃度：9 天 444 提交/5 发布/59 贡献者/320 PR(229 已合并)——超高频但 69% issue 未关、issue 积压与 PR 合并速率远超 2 人团队的常规审查吞吐（推断：agent 辅助审阅）；② 维护者：storytold 仅 2 人（echelon 主导 60.6% 提交），43 仓库中 16 个"craft"系 1 周内集中创建、合计 1556 个 open issue，资金靠 AI token 销售业务交叉补贴（创始人自述"七个免费 app 怎么养活是另一个问题"），单点风险极高；③ 公众评价两极：媒体惊叹 vs HN"非常破碎、标题欺诈性"、Reddit 质疑 clean-room 抗辩与 Adobe 专利；GitClear 2026 重复块 40.3→73.0/M 行(+81%)、Veracode ~45% AI 代码样本安全测试不及格、CSA 74 个 AI 关联 CVE；④ 许可证：依赖树全宽松许可无传染，唯一灰区=ocrs 模型权重（CC-BY-SA 训练数据+HF 无显式许可）；⑤ 存活率：2025 年 60+ AI app builder 到 2026 一半死亡（Taskade 坟场），craft 家族 9 天太早下结论，关键考验在热度消退后的 2-6 个月（推断）

---
Task ID: fin-c
Agent: general-purpose 子代理
Task: 调研净室重实现历史先例与法律边界

Work Log:
- 读 worklog.md 了解背景（主代理撰写"净室方法论历史先例与法律边界"章节+净室合规检查清单，pdfcraft 净室纪律已有描述）
- z-ai web_search 共 17 次有效查询（num=5、查询间 sleep 5s 避限流），覆盖：①Compaq/Columbia/Phoenix BIOS、Wine、ReactOS、Samba、FreeDOS、OpenBIOS/coreboot、DR-DOS ②CA v. Altai、Sega v. Accolade、Sony v. Connectix、Atari v. Nintendo、净室证据链 ③Oracle v. Google、SAS v. WPL、EU 2009/24/EC ④Adobe 专利许可（32000-1/32000-2）、商标指南、iText/ElcomSoft 诉讼史 ⑤失效案例（Atari 欺诈获码、IBM-日立 FBI 钓鱼、ReactOS 2006 审计、chardet 7.0 AI 重写争议、Claude Code leak 讨论）⑥实务指南（Wikipedia、IPWatchdog、Chip Law Group、Reed Smith、Santa Clara Law 论文）
- 汇总为 6 节中文调研报告（每节附来源 URL、区分"判例确立原则"与"业界惯例"、推断标注），直接随响应交付主代理，未产出独立交付文件（遵任务约束）
- 注：本条目曾于并发写入时被其他子代理（fin-a/fin-b 全量重写）覆盖一次，改用 shell 追加模式重新写入

Stage Summary:
- 判例确立（可写进清单"法律依据"栏）：功能/思想不受版权保护（CA v. Altai 抽象-过滤-比较法；SAS v. WPL：功能/语言/数据格式不可版权）；兼容性中间复制可属合理使用（Sega v. Accolade 1992、Sony v. Connectix 2000）；API 重实现可为合理使用（Oracle v. Google 2021，SCOTUS 回避可版权性）；欧盟 2009/24/EC Art.6 互操作反编译成文豁免且违约条款无效；欺诈获取源码即净室污染败诉（Atari v. Nintendo）
- 业界惯例（非法定强制，写进清单"纪律"栏）：双团队隔离、规范不含代码级细节、律师审规范、贡献者无接触声明、规范出处与行为对照记录留存、定期审计（ReactOS 2006 先例）、AI 污染声明（2026 chardet 争议新实践）
- PDF 特有边界：Adobe 对 ISO 32000-1:2008 发布 Public Patent License（免版税）；LoC 记载该许可"明确仅覆盖 PDF 1.7"，PDF 2.0 无同等声明但文本由 PDF Association 免费提供；Adobe 商标指南禁第三方用 Adobe PDF 图标；无 Adobe 追责 PDF 兼容库的公开记录（ElcomSoft 案是 DMCA 规避非兼容问题）
- 失效规律（推断）：净室失效均源于隔离墙被穿透——非法购码（IBM/日立）、欺诈获码（Atari）、贡献者先前接触（ReactOS 审计）、AI 语料携带原代码（chardet 7.0/Claude Code leak 争议）
- 对 pdfcraft 型项目的判断（推断）：以 ISO 文本+oracle 行为对照的"纯规范驱动"路线是风险最低档，与 OpenBIOS（IEEE 1275 公开标准）同档

---
Task ID: pdfcraft-analysis-3 (FINAL)
Agent: Super Z (main agent)
Task: 用户指令——客观文档写入，继续深挖至"净室开发分析工作无剩余事项"为结束节点，用子代理加速

Work Log:
- 盘点两份已交付文档（架构剖析+生态缺口）的未覆盖维度，确定收官补齐面：净室证据链审计/AI 治理剖析/可持续风险画像/历史先例与法律边界/自研合规清单
- 并行派发 3 个子代理：fin-a（Explore：ADR 交叉引用实测[6 编号 18 处、plan/ 被 gitignore 即正文私有]、CLAUDE/AGENTS 治理全纪律、ATTRIBUTION 284 条账本、deny.toml、贡献者会计[252 commits/96% AI co-author]、规范出处密度[ISO×59/RFC×56/§×123、22/30 crate]、CI 门 vs 文档承诺缺口[parity 门不在 GitHub CI]）；fin-b（GitHub API 实测：5735★/1997 fork/9 天 444 commits/69% issue 未关/2 人组织/16 仓 1556 open issues/副业资金；行业研究：GitClear +81% 重复、Veracode 45% 安全不及格、74 AI-CVE；许可证终审：672 包零 copyleft、唯一灰区 ocrs 模型 CC-BY-SA+无 LICENSE）；fin-c（17 次检索：四代净室谱系、五判例、Adobe 专利许可仅覆盖 32000-1、chardet 7.0 AI 污染新失效模式）
- 三段式撰写（scripts/fin_doc_part1/2/3.md）合并为 download/净室开发分析收官报告.md（242 行，28KB）
- 收官声明：6.1 十六维度覆盖矩阵（三文档全映射）、6.2 有意不覆盖项四条（Agent 自动化=用户排除/子系统细节=规格外/不可审计项=已声明/未来预测=不做）、6.3 正式结论"分析工作无剩余事项"

Stage Summary:
- 交付物：/home/z/my-project/download/净室开发分析收官报告.md（0 三问定位/1 净室证据链五层审计+唯一结构性空洞[ADR 私有]/2 AI 治理七组纪律+可移植性/3 风险画像[活跃表面 vs 结构底面，先行指标三条]/4 四代谱系+判例边界+PDF 特有四事实/5 18 条自研合规检查清单[判例/惯例/pdfcraft 三级标注]/6 收官声明）
- 系列终态：三份文档闭环——①架构（怎么做）→ ②缺口（做什么值钱）→ ③信任（能否信、如何合规照做）；净室开发分析工作按用户规格全部完成，无剩余事项；后续性质为增量跟踪/施工规划/按需深读，不再是分析


---
Task ID: sec-a
Agent: Explore 子代理
Task: 本地仓库"敌意输入与执行边界"维度客观事实提取（vendor 补丁全量/资源限额族/JS 沙箱/crypt 能力面/fuzz 纪律），只读调查

Work Log:
- 读 worklog.md 前情（pdfcraft-analysis-1/2/3、gap-a、fin-a/b/c），确认 HEAD 7c35809 基线，本轮专注安全边界五子任务
- 子任务1：全文读 vendor/README.md（5 crate 行×编号补丁共 34 条：hayro-interpret 17/hayro 6/hayro-jbig2 3/lopdf 3/hayro-syntax 5）；Grep "PdfCraft patch:" 实测 27 文件 72 处标记（每补丁多标记），逐条核对 34 条补丁在源码中的常量/函数全部存在（MAX_PAINT_NESTING/MAX_NESTED_PAINTS/MAX_CID/image_size_ok/MAX_IMAGE_PIXELS/MAX_DASHES_PER_STROKE/tiling_cell_scale/MAX_PIXELS/EXTRA_EMPTY_STEPS/ccitt_size_ok/MAX_EI_CANDIDATES/page.rs 环守卫等）；5 个 crate 均保留 Cargo.toml.orig；发现 README 表格 34 条之外还有 1 条未编号尾注补丁（OutlineGlyph::is_vertical 垂直文本提取 #126，font/mod.rs:168 有标记）；对 34 条逐一判定攻击类别并统计（内存炸弹 11/挂起 6/递归失控 3/整数溢出 1/除零 NaN 1/规范歧义 3/外观退化 4/功能补全 5，其中 hs3 整数溢出→分配炸弹双属性）
- 子任务2：crates/ 全域 Grep MAX_*/budget/timeout/TooLarge/limit 族，共盘点 15 个 crate 约 60+ 个限额；精读关键强制点：cos MAX_DECODED=1GiB（Stream::decoded_within/raw_within，FilterError::LimitExceeded 结构化拒绝）、cos parser MAX_DEPTH=128、cos MAX_XREF_OBJECTS=8388607（xref 流炸弹→重建）、render MAX_SIDE=8192/MAX_PIXELS=64MP（effective_scale 钳制）+STUCK_AFTER=20s 看门狗、filters 中间缓冲 max_output、js Limits/两套 run_*_within 线程隔离、xfa 五组 MAX_*（TooLarge 结构化错误）、engine xfa.rs 七限额+双预算、engine links.rs DOCUMENT_SCHEMES 白名单+MAX_DOCUMENT_URL=2048+BlockedLink 七类（含 mailto attach/RFC2047 编码字/多层 percent-decode 检测）；区分硬拒绝/钳制降级/弃置线程三类行为
- 子任务3：读 crates/js 全部三源文件头注释+关键段：boa_engine 0.22.0（crates.io 注册表来源，未 vendor）；宿主面=event/document(this)/Field/app/util/console/display/color 完整清单（lib.rs:1-18+900-950）；显式沙箱承诺 lib.rs:20-22 原话；Limits 默认 1M 循环/256 递归（boa runtime_limits API 强制）；栈安全节 lib.rs:984-1003（boa 解析器无自限→refuse() 预扫描 4 限额+1GiB 专用线程栈）；FormCalc 六限额+六错误串；XFA 限额组+XfaOutcome.abandoned（线程不可中断→弃置）；engine 耦合：js.rs JsRunner 实现 forms::Scripts trait 注入 recalculate_with；xfa.rs limits_for/timeout_for 分级（click 1M/3s vs 自动事件 100k/64/1s）+ click_only 白名单+OPEN/EVENT_BUDGET_MS；ui-egui js_ui.rs → request_document_url（用户确认+同源唯一 pending）；parity JS 域统计（6 shipped/5 partial/7 planned 及逐条 notes）
- 子任务4：读 crates/crypt/src/lib.rs 全貌：L0 standalone、R2-R6 全版本支持（open r=2..=4 走 owner-first+Algorithm 7 恢复用户密码、r=5|6 走 SASLprep、其余 Unsupported）、仅 /Filter /Standard、create() 建 /Encrypt（Rc4_40/Rc4_128/Aes128/Aes256×Algorithm 3/8/9/10、调用方供 32 字节种子免 OS 访问）、StreamKind 三类（含 /EFF 附件专用与命名过滤器）、Permissions 10 位、SASLprep=RFC 4013 stringprep（lib.rs:308-310）；tests 含 RC4 参考向量+NIST SP800-38A；未实现面来自 parity（attachment-only partial/证书加密 planned）
- 子任务5：读 xtask/src/fuzz.rs 全文（M1.10 稳定 Rust 变异 fuzzer、check-one --edit 全链路子进程、crash/hang 判定、最小化、ulimit -v 4GiB 子进程地址空间帽、hang 3 倍超时独占复验、退出码语义）、crates/filters/fuzz（libfuzzer decode target、nightly、脱钩 workspace）、ci.yml（nightly cron 17 3 * * * 15 分钟 fuzz job+artifact 上传，push/PR 不跑 fuzz）、gates.rs check 基线对比（baselines/pdfjs.json=20 条：18 open-error+2 timeout）、ROADMAP 模糊会计原话（诚实评估 L46、量化 L71、风险 L123、10-05 双日志、10-07 L232≈15 万变异文件）、CLAUDE.md:32/40+AGENTS.md §4:115 崩溃修复配回归测试条款原文、catch_unwind 实测 6 处（render raster/inspect×3、engine guard/open、cos corpus 测试、sign 测试）
- 全程 Read/Grep/Glob+只读 Python 解析 TOML/JSON，未修改仓库任何文件，未运行构建/测试

Stage Summary:
- vendor 债务全量：34 编号+1 尾注补丁，全部有源码标记与配对测试名（1 条例外=nightly fuzz 发现本身）；攻击类别高度集中于"fuzz 级敌意输入"（内存/挂起/递归占 20/34），其余为规范歧义与外观保真修复——补丁本质是给上游渲染栈补敌意输入免疫
- 限额族画像：L0（filters/cos 解压炸弹+嵌套深度）→L2（render 尺寸钳制+看门狗+逐页 catch_unwind）→L3（js/xfa 全限额+线程弃置）→L4（engine 事件预算+URL 白名单七类拒绝）→UI 层（用户确认），纵深防御链完整；两类行为明确可分（结构化错误拒绝 vs 钳制降级带告知），弃置型（abandoned/stuck）是第三类诚实设计
- JS 沙箱：boa 0.22.0 无宿主访问（测试 the_sandbox_has_no_host_access 断言 5 个全局均 undefined）；launchURL 仅 3 scheme+用户确认+防弹窗洪泛；XFA 脚本 print/save/link/focus 仅 click 可请求；文档自认缺口="脚本内存无上限、嵌套调用循环仅靠弃线程停"
- crypt：标准安全处理器全版本读写+建字典，SASLprep 仅 R5/R6，L0 不知对象图；证书加密/附件专用加密为 planned
- fuzz 仪式：stable 变异 fuzzer（夜间 15min 进 CI schedule）+libfuzzer 局部+corpus 基线对比+150k 变异文件 7 bug 全配回归测试的诚实会计；治理条款（CLAUDE/AGENTS "Prove it"）与机器执行（ci.yml fuzz job）对齐

---
Task ID: sec-b
Agent: general-purpose 子代理
Task: 调研开源 PDF 引擎敌意输入安全行业坐标（解析器威胁模型 CVE / JS 沙箱策略对照 / PDF 加密现状 / 资源限额工程 / 2025-2026 近况），供主代理撰写 PDF 引擎"敌意输入与执行边界"安全架构分析

Work Log:
- 读 worklog.md 前情：pdfcraft 净室分析系列已完成架构/生态/治理三篇（pdfcraft-analysis 1-3 + gap-a/b/c/d + fin-a/b/c），本任务补安全维度的行业事实坐标
- z-ai web_search 共 15 次查询（num=5、查询间 sleep 7s 串行避限流、零 429 重试），关键词依次：①PDF decompression bomb CVE MuPDF pdf.js memory bomb ②recursive XObject infinite recursion stack exhaustion ③poppler CVE infinite loop hang ④pdf.js enableScripting AcroJS XFA policy ⑤GHSA-hq66-cqwq-w95j details ⑥pdf.js XFA forms Firefox 2024 ⑦PDFium JavaScript V8 sandbox ⑧MuPDF remove JavaScript interpreter 1.14 ⑨qpdf JavaScript encryption design philosophy ⑩Acrobat Protected Mode JavaScript number one attack vector ⑪PDF encryption RC4 AES-256 no integrity MAC ⑫ElcomSoft PDF password GPU R6 ⑬encrypted PDF malleability incremental update paper ⑭MuPDF fz_context memory limit fz_store ⑮Acrobat 2025 2026 sandbox hardening announcement
- 额外约 16 次免配额 curl 一手源核验：MuPDF context.h（FZ_STORE_UNLIMITED/FZ_STORE_DEFAULT 限额注释+分配失败先驱逐 store 再重试+分配失败抛 fz 异常）、MuPDF releases/history（1.25.6 "clip/layer nesting >1000 levels" 加固、1.27 C23 式 checked arithmetic fz_ckd_mul_int）、MuPDF docs 1.28.2（mutool run 内建 ES5 解释器、C API Javascript Library）、MuPDF 文档 PDF 版（-J Disable JavaScript in PDF forms）、mujs.com（轻量 ECMA-262 解释器、为 MuPDF 而生、ISC）、PDFium README.md 原文（"may be built either with or without JavaScript support…Both of these features are enabled by default. the XFA feature requires JavaScript"、pdf_enable_v8/pdf_enable_xfa GN 参数）、PDFium testing/fuzzers Gitiles JSON（51 个 fuzzer：pdf_cjs_util/pdf_formcalc/pdf_fm2js/pdf_psengine/pdf_xfa×3/全 codec）、pdf.js api.js（enableXfa、maxImageSize 参数）与 display_utils.js（limitCanvas maxPixels/maxDim/capAreaFactor+OutputScale.capPixels）、OSV API GHSA-hq66 全文（pdfjs-dist ≥5.6.83 <6.2.108、2026-08-06 发布、enableScripting 默认 true、无 CSP=宿主域任意 JS、workaround enableScripting=false/CSP）、pdf.js FAQ wiki（"PDF.js runs with the same permissions as any other JavaScript code"）、qpdf encryption.html（RC4 obsolete/40bit 密钥 5 字节可暴破/V5-R5 never fully specified/R6≈gpg/P 位域建议性/--allow-insecure）与 design.html（user/owner 密码等价对待不强制权限）、Adobe APSB26-43 公告全文（CVE-2026-34621 CWE-1321 原型污染→任意代码执行、CVSS 8.6、in-the-wild、Haifei Li EXPMON 报告、2026-04-11）、Adobe 社区 2026-09-18 沙箱强制化公告全文（macOS 2026-10 / Windows 2026-12 起不可关闭、第三方插件必须兼容、macOS App Sandbox 2024-09 引入、Windows Protected Mode 一直默认）、web-in-security PDFex 全文（RUB+Münster 2019：CBC 无完整性→密文可塑性、部分加密密文明文混排、Direct Exfiltration 与 CBC Gadget 两类攻击）、ElcomSoft APDFPR（40/128bit RC4+256bit AES 均可解、Key Search Attack 保证恢复 40bit、GPU 加速数天级）、hashcat wiki（10400=PDF 1.1-1.3/10500=1.4-1.6/10600=1.7 L3/10700=1.7 L8，$pdf$ 标准哈希格式）、OSS-Fuzz projects（mupdf/qpdf/poppler/ghostscript 均 HTTP 200 收录、pdfium 与 pdf.js 404）、pdf.js GHSA 与 CVE-2026-82864 详情（pdfme 的 pdf-lib <5.5.10、DecodeStream.ensureBuffer 无界增长、50KB→500MB）
- 产出 5 节结构化中文报告（每条事实附 1 URL、事实与推断严格分离标注、15 条查询关键词全记录），直接随响应交付主代理，未产出独立交付文件（遵任务约束）

Stage Summary:
- 威胁模型四类坐标：解压/内存炸弹=CVE-2026-82864（pdfme pdf-lib，FlateDecode 无界 buffer，50KB→500MB 崩溃 Node）；递归结构（对象环/自引用）=CVE-2024-7866（xpdf pattern 资源对象环→栈溢出）+GHSA-7wmm-6g5p-mv4p（pdfcpu ParseObjectContext/parseArray 无嵌套上限）+CVE-2025-50420（poppler pdfseparate）；整数溢出=VU#951662/CVE-2026-3308（MuPDF ≤1.27.0 OOB 堆写）+CVE-2026-10118（poppler Splash 整数溢出→RCE）；无限循环挂起=CVE-2009-0146（poppler JBIG2 挂起）+CVE-2019-13288（xpdf getObj）
- JS 沙箱格局（重点）：pdf.js=AcroJS 自 Firefox 90、库默认 enableScripting=true、无 CSP 即宿主域任意 JS（GHSA-hq66，2026-08 仍爆雷）、XFA 靠 enableXfa 渲染（交互性有限）、"沙箱"实为浏览器同源+CSP；PDFium=完整 V8 全引擎、编译开关 pdf_enable_v8、XFA 强依赖 JS、安全由 Chromium renderer 沙箱兜底（脱离 Chromium 体系自担风险）；MuPDF=自研 MuJS（ES5 无 JIT、API 面极小）同进程无隔离、-J 一键关闭表单 JS；Acrobat=Protected Mode（Win，2010 Reader X 起）+App Sandbox（macOS，2024-09 起）且 2026-10/12 起强制不可关，JS 攻击面延续至今（CVE-2026-34621 原型污染 ITW + 2010 年 CISA 官方"禁 JS"缓解史）；qpdf=从不实现 JS。推断结论（为何多数引擎不实现）：JS 引擎漏洞直接通向 RCE、独立沙箱成本极高（Acrobat 花了 16 年才强制化）、阅读场景 JS 收益趋零、AcroJS/XFA API 面巨大且部分仅 Acrobat 兼容——完整实现 JS 的引擎无一例外有平台级沙箱兜底
- PDF 加密现状：RC4（R2-R4）行业定性 obsolete，40bit 密钥仅 5 字节必被穷举（qpdf 官方+ElcomSoft"保证恢复"+hashcat 10400-10700 四个标准模式覆盖全部 handler 证明 GPU 攻击工业化）；R5 从未完整规范（Acrobat 短暂存在）；R6 AES-256+强密码≈gpg 同级，但弱口令仍为 GPU 靶子；结构性缺陷=加密无 MAC：PDFex（RUB 2019）证明 CBC 无完整性→密文可塑性+部分加密混排+CBC Gadget 可在无密钥下构造自外渗密文；权限位 P 仅建议性（qpdf 不强制、空 owner 密码即失效）
- 资源限额工程惯例：MuPDF fz_context 最系统（store 字节限额为构造参数+FZ_STORE_UNLIMITED/DEFAULT 枚举+分配失败先驱逐再重试+fz_try/fz_catch 结构化异常"Never panic"+1.25.6 千层嵌套加固+1.27 C23 式溢出检查函数）；pdf.js maxImageSize/limitCanvas(maxPixels,maxDim,capAreaFactor) 像素封顶降 scale；qpdf 拒绝默认创建不安全加密（--allow-insecure 显式豁免）；行业共识=持续 fuzz（OSS-Fuzz 收录 mupdf/qpdf/poppler/ghostscript，PDFium 自有 51 fuzzer 覆盖 codec/XFA/FormCalc/CJS/PS 引擎）+限额+结构化错误三板斧
- 2025-2026 趋势：①解析器内存安全漏洞年复一年（MuPDF/poppler/PDFium/pdf-lib 2026 均有）；②Acrobat 沙箱从可选推向强制（2026-10 macOS/2026-12 Windows，含第三方插件）——OS 级隔离成 PDF 引擎标配信号；③JS 类攻击未熄火（原型污染 ITW+pdf.js enableScripting 公告）；④开源引擎最常见类是 DoS（递归/挂起/炸弹），RCE 集中在带渲染后端引擎；⑤弃养库 fork（pdf-lib→pdfme）继续爆雷
---
Task ID: pdfcraft-analysis-4
Agent: Super Z (main agent)
Task: 用户质询"没有别的事项了吗？pdf分析上"——对"无剩余事项"结论执行残项审计，补齐发现的散落维度

Work Log:
- 交叉核对三份已交付文档（架构剖析/生态缺口/收官报告）全文与 17 维分析清单：确认 16 维专题覆盖成立，发现 1 维"敌意输入与执行边界"散落未专题化（资源限额 6 行/补丁摘要 8 行/JS 与 crypt 各附录一行/fuzz 一笔带过）
- 并行派发 2 个子代理：sec-a（Explore：vendor 35 处补丁逐条定位与分类、60+ 限额常量逐 crate 盘点、js 沙箱承诺原文与拒绝面断言、crypt 能力面、fuzz 基础设施与战果会计、治理条款原文定位）；sec-b（general-purpose：15 次检索取 CVE/厂商公告一手源——pdf.js GHSA-hq66-cqwq-w95j、Acrobat APSB26-43 原型污染 ITW、2026 沙箱强制化时间表、PDFex 加密可塑性、MuPDF fz_store 限额、PDFium 51 fuzzer、hashcat PDF 模式）
- 关键新事实：补丁实际 35 处（34 编号+1 尾注 is_vertical）；jbig2 符号字典条为唯一无命名测试补丁；限额五层纵深（L0 结构拒绝/L2 钳制+看门狗/L3 数量+时间预算/L4 事件总量/UI 输入面）与三种超限行为学（硬拒绝/钳制降级/弃置）；js 无宿主访问+副作用请求枚举+LaunchUrl 七层防线+未请自跑脚本降配额；boa 解析器栈炸弹预扫描+1GiB 专用栈；文档自认残留"script memory is not capped"；fuzz 为自研语法感知变异器+子进程 ulimit -v 4GiB 帽+夜跑 15 分钟，150k 变异文件、7 OOM+11 挂起全修复、"full run now finds 0 crashes"（10-05 起）；xtask check 与 fuzz 均不进 GitHub CI；lopdf vendored 0.45 自带上游递归防护（修正②系列 RUSTSEC 旧版本印象）
- 三段式撰写（scripts/sec_doc_part1/2/3.md）合并为 download/敌意输入与执行边界剖析（净室开发分析残项补充篇）.md（344 行，40KB）
- 编辑收官报告 §6.3 追加 2026-10-10 补记：残项审计闭环说明、矩阵增至 17 维、三则口径修正、无剩余事项结论正式成立

Stage Summary:
- 交付物：/home/z/my-project/download/敌意输入与执行边界剖析（净室开发分析残项补充篇）.md——结构：0 残项审计方法与结果/1 vendor 35 处补丁全量分类学（内存炸弹 11·挂起 6·递归 3 等 8 类统计+完整表）/2 资源边界五层纵深与三行为学/3 JS 执行边界（沙箱四件套+行业六引擎对照矩阵）/4 加密能力面与 PDFex 结构性边界/5 fuzz 战役设计与崩溃免疫仪式/6 威胁类×CVE×防御对照+自研最小安全骨架 12 条/7 收官更新（矩阵 v2+最终结论）
- 系列终态：四份文档闭环（架构→缺口→信任→安全骨架）；残项审计过程可复核（质询→17 维核对→1 残项→补齐→结论重申）
- 最终判定：净室开发分析工作无剩余事项，结论现建立在审计过程而非声明之上；后续仅增量跟踪（parity diff/vendor 补丁数/fuzz 战果三线）/施工规划/按需深读

---
Task ID: plan-oc-1
Agent: Super Z (main agent)
Task: 用户指令——PDF 分析线确认收官；基于 AI 工作台引擎组成面（dsh 框架已定 / MD 引擎开发中 / 剩余办公全家桶 + 画布，动画暂缓），规划下一批净室分析对象（全部开源/源可用，不涉闭源）

Work Log:
- 读 worklog.md 前情（pdfcraft-analysis-1/2/3/4、gap-a/b/c/d、fin-a/b/c、sec-a/sec-b 全系），确认 PDF 线四文档收官状态，无实质补充事项
- 撰写并执行 scripts/nextbatch_search.sh：11 组 z-ai web_search（num=5、查询间 sleep 7s、零失败），日志存 scripts/research_logs/nextbatch_20261009_170858.log
- 关键核实：dsh = DeepSeek Harness（DeepSeek AI 开源 agent harness，everything-is-a-plugin 架构，2026-08 developer preview）；Univer 定位 "The Office Harness for AI Agents"、workspace 仓 Apache-2.0；tldraw source-available（水印+商业许可）；drawio 源码 Apache-2.0；maxGraph = mxGraph 2020 归档后的 TS 继任；Penpot MPL-2.0；ComfyUI Nodes 2.0 已从 LiteGraph Canvas 迁往 Vue 架构（2025 末）；InvokeAI Apache-2.0；HyperFormula v3.4.0 GPLv3+商用双许可；AFFiNE doc/edgeless 双模式（具体许可待核）；ONLYOFFICE AGPL-3.0 且 2026-06 起 100% 开源
- 三段式撰写（scripts/oc_plan_part1/2/3.md）合并为 download/AI工作台引擎——办公与画布线净室分析规划.md（388 行，35KB）

Stage Summary:
- 交付物：download/AI工作台引擎——办公与画布线净室分析规划.md——结构：0 定位与净室边界/1 组成面盘点（含 dsh 身份确认）/2 17维→11维方法论裁剪/3 16 候选总览表（许可已核✅/待核⚠️标注）/4 办公线四层（Univer·OnlyOffice·HyperFormula·读写管线）/5 画布线五层（Excalidraw·tldraw·drawio-maxGraph·渲染基座·ComfyUI·InvokeAI·动画P3暂缓·节点图可选）/6 跨切面（Yjs CRDT vs OT、AFFiNE 双态、dsh 插件+MCP 双口径调用面）/7 批次表（P0=Univer深读+Excalidraw深读+ComfyUI协议pass；Task ID 已分配 office-a/canvas-a/canvas-d 起）/8 壁垒评估（"无 PDF 式壁垒"判断成立+两条 caveat：办公保真难度非零、红海差异化靠 AI-native）/9 排除清单/10 事实登记（11 条已核带源+待核清单+二手数据声明）
- 方法论资产迁移：限额五层纵深+三行为学+沙箱四件套作为新域敌意输入固定章节；净室合规 18 条清单复用；dsh 插件面+MCP 工具面双口径登记能力矩阵
- 下一批分析已可按批次表直接开工（P0 三项），无需再行规划
---
Task ID: status-query-1
Agent: Super Z (main agent)
Task: 用户询问云仓库上传情况（"成果没上传云仓库？看到三个？"）

Work Log:
- git ls-remote 验证两个我方推送仓库均存活且与本地 HEAD 一致：mmqz/agents-md-audit=89cfcae、mmqz/acrobat-pro-audit=246fc0f（含 R6 AGS 取证全部提交）
- GitHub API 被限流，改抓 repositories 页面 HTML：用户账号 mmqz 名下 6 个公开仓库=cpa-multi-plugins(10-10更新)/acrobat-pro-audit/agents-md-audit/realtime-cpu-vc/kokoa-browser/qoder-ui
- 对照 worklog 全文检索：我方创建并推送的仅 2 个（agents-md-audit、acrobat-pro-audit）；cpa-multi-plugins 今日更新但无我方工作记录，非我创建
- 主项目 /home/z/my-project/.git 无远程（本地自动提交）；download/ 的 PDF 线 4 文档+办公画布规划文档未上传任何云仓库
- 结论：审计期成果已上云（2 仓）；PDF 线及后续成果未上云——前次 token 按纪律单次经环境变量使用、未落盘，新推送需用户提供新 token 或自行推送

Stage Summary:
- 云仓库真实盘点：我建 2 个（均已验证同步）；第 3 个近期活跃仓库 cpa-multi-plugins 非我方产物；download/ 新成果待上云，推送方案=复用 push_to_github.sh 的内存 Basic 头机制（零 token 落盘）
---
Task ID: push-cleanroom-1
Agent: Super Z (main agent)
Task: 用户提供新 token，要求全部成果推送进单一云仓库

Work Log:
- 组建 /home/z/my-project/cleanroom-analysis/：5 份成果编号复制（01 架构剖析 359 行/02 生态缺口 430 行/03 敌意输入边界 344 行/04 收官报告 244 行/05 办公画布规划 388 行，共 1765 行）+ README 索引（阅读顺序/方法论资产/结论速览/关联仓库）+ tools/push_to_github.sh（复用零落盘机制）
- 本地 git init -b main 提交 7 文件（HEAD=b691773）
- API 创建仓库 mmqz/cleanroom-analysis 返回 201；git 推送经内存计算 Basic 头成功，main 新分支建立
- 验证：git ls-remote 远端 HEAD=b691773 与本地完全一致；网页端确认仓库公开可见（标题含中文描述）
- 零落盘核查：ghp_[脱敏] 在全部 .sh/.md/.py/.json/.txt/.log/.git/config 中零命中；token 仅在单次命令的环境变量赋值中出现，未写入任何文件

Stage Summary:
- 第三座云仓库落地：https://github.com/mmqz/cleanroom-analysis （public，5 份成果+README，与 download/ 本地副本一致）
- 用户账号现共 3 座我方仓库：agents-md-audit / acrobat-pro-audit / cleanroom-analysis（用户所称"三个"自此成立）
- 后续新成果（办公/画布线 R1 起）可直接提交进本仓库，推送机制 tools/push_to_github.sh 就绪（需新 token 时同法单次传递）
---
Task ID: merge-cleanroom-2
Agent: Super Z (main agent)
Task: 用户指示——全部工作成果合并进单一仓库，供新 AI 只看一个仓库接手

Work Log:
- 盘点两审计仓体量：agents-audit 20 文件 248K 全量并入；acrobat-audit 并入 analysis/evidence/tools/哈希登记与配置工件，排除 Adobe 闭源二进制原件（AcroPro.msi 13M、setup.exe 0.7M、AGS 的 EXE/MSI 载荷 8M，留原仓 mmqz/acrobat-pro-audit）
- 重组结构：pdf-line/（01~05）、agents-md-audit/、acrobat-audit/、tools/；新增 HANDOFF.md（新 AI 接手指南：四阶段状态/必读优先级/5 条纪律/R1 批次/环境备忘含网关 Basic 机制与 API 限流旁路）；重写 README 为总索引；worklog.md 副本入库
- 提交 100 文件（5.4MB），推送 913b2e7 成功，ls-remote 与本地一致，网页端五个结构关键词全部可见
- 自伤修正：前一条 worklog 记录误写 token 前 7 位（ghp_[已脱敏]）且随副本公开——仅 7/36 字符不可复原，但违反零落盘纪律；已对本地与仓内两份 worklog 双重脱敏并重推；教训=核查用语不得引用 token 任何片段
- 复核：完整 token 在全部文件零命中（本次核查模式改为正则全串匹配，避免再次引用片段）

Stage Summary:
- 单一成果仓库终态：https://github.com/mmqz/cleanroom-analysis = L0+L1+L2+L3 规划+worklog+HANDOFF，100 文件 5.4MB，新 AI 只需看此一仓
- 新 AI 接手话术（给用户）：让新 AI 读 cleanroom-analysis 仓库的 HANDOFF.md 即可

---
Task ID: merge-md-1
Agent: MD 引擎研发会话（main agent）
Task: MD 线成果同步进合并仓 + HANDOFF 两线登记（用户指示：新代理看两仓接手）

Work Log:
- 核查：本仓此前无 MD 线任何记录（R23/R24/main 首推/mdcore 全部 0 命中）；
  MD 线成果原存于 mmqz/typora-re-analysis（主档 §31/§32 + plans/ + gates +
  compliance_rounds/）与 mmqz/mdcore（main=66d1ea6 三提交链三分支）。
- 新增 md-line/STATUS.md：MD 引擎线结论层同步（架构/合规双门终态/仓库地图/
  R24 编排摘录/PDF 计划衔接/遗留定性/可开工清单）。分区铁律保持：审计底稿
  不出 typora-re-analysis，本目录只存结论与计划。
- 新增 md-line/pdf_carrier_plan_2026-10-10.md：PDF 载体工程计划副本（与
  pdf-line/01~04 互补——L2 分析别人怎么做，本计划是我们怎么做；L2 敌意输入
  边界篇为 PDF-a 安全策略输入）。
- HANDOFF.md 增补：§2 阶段表加 L-MD 行 + 关系注（组成面「MD 引擎开发中」
  实态 = 合规双门闭合/R23 收口/main 就位/R24+PDF 待开工）；§6 下一步改两线
  并行（线 A L3 R1 + 线 B R24/PDF），优先级裁决权在用户。

Stage Summary:
- 合并仓现为全工作台唯一接手入口：新代理读 HANDOFF + worklog + md-line/STATUS
  即可同时接手生态审计线（L3 R1）与 MD 引擎线（R24/PDF），产品操作落
  mmqz/mdcore，底稿按需 fetch mmqz/typora-re-analysis（不出仓）。
---
Task ID: md-r24a-1
Agent: Super Z (main agent, 双线会话)
Task: 用户裁决顺序（MD 检查→R24→PDF→画布）；执行 MD 实现质量审计 + 整备两提交 + R24-a 设置面核侧全轮落地

Work Log:
- 审计（Task md-audit-1，本地 worklog 详录）：六门禁全绿复现——scan.sh 0 命中 /
  cargo 148/148 / wasm32 check clean / web 构建 OK / web smoke 15/15 / 契约门
  13/13 / IME CDP 28/28；代码审查零 unsafe、零 TODO、unwrap 全在安全路径。
  结论：MD 实现无问题。非阻塞发现：Slint 文档漂移 + clippy ~60 风格警告。
- 整备提交 A（44ac3f9）：活文档 ADR-H1 对齐（tao+wry+GTK3；gates.md 历史段
  按追加纪律不动）。
- 整备提交 B（a7630c3）：clippy 清零（~60→2，余 2 条为 rustc 1.99 对
  wasm_bindgen 属性参数的误报，已定性登记；含 2 处带理由的显式 allow）。
- R24-a 主提交（2de27a2）：mdshell::settings 模块（四层 schema / TOML
  sidecar 原子写 / 默认合并 / 未知键+注释往返保留 / v0→v1 迁移 / 三层降级
  [文件级可修复、键级回默认、未来版本拒写]）；语义锁 14 条（含跨 crate
  常量锁 undo_coalesce_ms==COALESCE_MS）；CONTRACT v1→v1.1 minor 升版
  三处同步（__settings/__setSetting/mdc-settings-changed；契约门 13→16）。
- 合规：新增 toml_edit 族 8 依赖（全 MIT/Apache 白名单族，crates.io 版本级
  API 按锁定版本取证）；NOTICES 草案 v1.1 增补于 compliance-baseline 分支
  （8c26272；34→42 项，UNKNOWN 0）。
- 门禁终态：cargo 164/164（148+16）/ wasm clean / scan 0 / 契约门 16/16 /
  smoke 15/15 / IME 28/28。推送 main+compliance-baseline 双分支，ls-remote
  验证 local=remote；token 零残留（四仓 rg 全串零命中）。

Stage Summary:
- R24-a 收口完成：设置面核侧全轮交付（主档 §31 R24-a 四条机制合同全落地），
  产品仓 main=2de27a2（四提交链：66d1ea6→44ac3f9→a7630c3→2de27a2）。
- 关键决策：schema_version 为 set 面保留键；未来 schema load 全默认+save
  拒写（防降写）；值域表双侧同源（CONTRACT §4 为共同锚）。
- 待办提醒：定名商标检索是 R24-c 前置阻塞项（TRADEMARK_RULES §4）。
- 下一步：R24-b 多窗口核侧（SessionManager / 窗口↔document_id 映射 /
  资源上限 / 会话持久化；CONTRACT v1.1 增 session 面）。

---
Task ID: md-audit-2
Agent: main-agent (Super Z, 会话 web-52b2ba37)
Task: 用户裁决开工序第一项——mdcore 实现质量检查（"先检查md的实现是否有问题"，Rust 性能纪律确认，至少 4 轮持续工作）

Work Log:
- 环境重建：净工作区克隆 mmqz/mdcore（main=2de27a2，三分支就位）+ 本仓；
  Rust 1.99.0 + wasm32 target 安装；git 身份 mdcore-dev 本仓级配置。
- 基线双绿复现：scan.sh 0 命中 + cargo test --workspace 164/164（与 R24-a
  收口一致）+ wasm32 check 过。
- 深度勘察（超越上轮 md-audit-1 的门禁级审计——行为探针法）：lib.rs
  事务管线 / tree.rs arena / op.rs 求逆 / history.rs 合并窗 / settings.rs
  R24-a 新模块 / external.rs / fingerprint.rs 确定性 / wasm 桥 / mdshell。
- Bug#1 抓出并修复（R24-a1，P1 静默丢设置）：set() 直写
  doc[section][key] 在节缺失时 toml_edit IndexMut 自动升格 InlineTable
  （index.rs Item::None 分支实证），读侧 as_table() 不认 → 值落盘但下次
  load 回默认。修复：ensure_section（写侧规范化：缺失→标准表 / inline→
  值保留升格 / 异形→空表）+ item_of 改 as_table_like（读侧容错）。
  探针 3 条全红复现 → 修复后全绿 → 升级为 r24_settings.rs §10 语义锁。
- Bug#2 抓出并修复（R24-a2，undo 破坏精确恢复不变量）：invert_op 对
  SetMark 无条件翻转 add，no-op 场景（冗余 add / 移除不存在 span）的
  undo 会删 mark / 凭空造 mark。修复：逆方向由 state-before 的 span
  存在性决定（no-op→同向幂等）。附带加固 SetMark 区间校验
  （start>end / end>len → Reject）。合并窗并条对照组语义固化。
  锁 ×4 新套件 r24_quality_fixes.rs。
- 门禁终态：cargo 171/171（164+7）/ scan 0 / wasm32 clean / clippy
  零新增（2 条既有 wasm-bindgen 宏告警，已登记基线）。
- 提交推送：mdcore main 2de27a2→db305ab，ls-remote 验证 local=remote；
  token 磁盘零残留（全仓 rg 零命中）。gates.md 增「R24 质量审计轮」节，
  README 状态快照同步。

Stage Summary:
- 质量检查结论：mdcore 实现**总体过硬**（架构/ADR 纪律/事务语义扎实），
  但深挖出 2 个真 bug（均为上轮门禁级审计不可见的边界缺陷）并已修复
  +7 语义锁背书。用户"先检查"的决策被证明有效。
- R24-a1：settings 写读形态不对称（toml_edit 自动 inline 表）——静默丢设置。
- R24-a2：SetMark 求逆不对称——no-op 的 undo 破坏「精确恢复前态」。
- 下一步：R24-b 多窗口核侧（SessionManager / 窗口↔document_id 映射 /
  资源上限 / 会话持久化），随后 R24-c 打包基建（前置：定名商标检索）、
  PDF-a（Acrobat 成果转化 + pdfium/lopdf 探针 + ADR-P1）。

---
Task ID: md-r24b-1
Agent: main-agent (Super Z, 会话 web-52b2ba37)
Task: R24-b 多窗口核侧（用户裁决开工序第二项：MD 收尾）

Work Log:
- Core 脏标记：is_dirty/mark_saved 落 lib.rs（单一变异入口挂钩——
  View/Ai/Undo/Redo 置位、System 装载不置位、adopt_external 成功清位、
  粘性安全向假阳性）；mem_stats 增 dirty 字段；external.rs adopt 钩子。
- mdshell::session 新模块：SessionManager（BTreeMap 映射 + MRU 焦点序 +
  DEFAULT_MAX_SESSIONS=16）/ 一窗一档（DocumentAlreadyOpen 携带持有窗口）/
  超限显式 Err 不静默驱逐 / SessionRecord 持久化记录（schema v1，未来
  版本拒绝→宿主冷启动）/ WindowRecord（path 宿主域透传）。
- DocumentSession 增 path 字段 + new().with_path() 构造器。
- CONTRACT.md 新增 §5 会话面（纯宿主面 Rust↔Rust；window 面零新增，
  __contractVersion 保持 1.1——不升版裁决显式在档防两向漂移）；§1 增
  脏标记行；contract_freeze.rs 增会话面词面锁 + 编译期常量双锚。
- 语义锁 10 条（r24_session.rs）+ session.rs 单元 1 条；实现期修正 2 处
  测试构造问题（Core 无 Clone/Debug → 去 derive；session_with_doc 预设
  id 与防换牌拒绝冲突 → 改无身份核显式设同 id）。
- 门禁终态：cargo 183/183（171+12）/ scan 0 / wasm32 clean / clippy
  零新增。零新依赖（SBOM/NOTICES 无需再生成）。
- 推送：mdcore main db305ab→8f08368，ls-remote 验证 local=remote。

Stage Summary:
- R24-b 收口完成：多窗口核侧全轮交付（SessionManager/映射/上限/持久化
  + Core 脏标记权威源）。plan_reload(local_dirty) 自此有权威取值。
- 关键决策：一窗一档 v0；会话记录=可再生缓存（冷启动降级，不设拒写）；
  脏标记粘性；§5 不升 __contractVersion（宿主面非 window 面）。
- 下一步：R24-c 打包基建（定名商标检索为用户决策前置阻塞项——计划以
  参数化占位名落地基建，名称变量化不阻塞），随后 PDF-a。

---
Task ID: md-r24c-1
Agent: main-agent (Super Z, 会话 web-52b2ba37)
Task: R24-c 打包分发基建（用户裁决开工序第二项：MD 收尾的最后一轮核侧工作）

Work Log:
- CARGO_PKG_VERSION 单源：根 Cargo.toml [workspace.package].version（唯一
  手写点）+ 四 crate version.workspace 继承 + mdcore::VERSION 运行时
  唯一出口；Cargo.lock 零 diff（继承不改变解析结果）。
- packaging/ 基建：README（定名阻塞纪律 + 变量表 + 版本单源图）/
  checklist.md（A-F 六节核对单，A4 定名门 + B2 版本单源为硬门）/
  四平台模板（desktop 入口含 .md mime + mdcore:// scheme 注册 +
  SingleMainWindow 单实例语义；AppImage AppRun + 构建脚本；deb
  control/postinst/build（手工 dpkg-deb 布局，不引入 cargo-deb）；
  NSIS 安装器（注册表面 + WebView2 Evergreen 引导 + 卸载器）；
  macOS Info.plist（文档类型 + URL scheme + 无网络面声明））。
- 定名阻塞处理：正式名 = 用户决策 + TRADEMARK_RULES §4 商标检索强制门
  未决——{{APP_NAME}} 参数化占位（基建不阻塞），可执行名/包名/scheme
  用 mdcore 自研命名域先行；定名提交 = 占位符替换 + checklist A4 勾选
  + README 变量表登记三处一体（锁死在 r24_packaging.rs §4）。
- 合规门禁扩展：scan.sh PRODUCT_PATHS 纳入 packaging/（发布面=产品面）。
- 语义锁 4 条（r24_packaging.rs）：版本单源 / 工件在位与命名域 / 构建
  脚本注入纪律 / 定名门在档。实现期修正 2 处（const 字符串比较非稳定
  → 运行时断言；核对单反引号子串不匹配 → 断言词修正）。
- 门禁终态：cargo 187/187（183+4）/ scan 0（scope 扩展后）/ wasm32
  clean / clippy 零新增。零新依赖。
- 推送：mdcore main 8f08368→587246f，ls-remote 验证 local=remote。

Stage Summary:
- R24-c 收口完成：打包基建全轮交付（版本单源 + 四平台配置 + 合规
  核对单）。MD 收尾三连（质量审计 + R24-b + R24-c）全部完成。
- MD 线剩余 = R24-d 壳实体（开发环境轮，本环境不可做）+ R24-e 收口
  （依赖 d）+ 人工项（真机 IME / 定名商标检索）。
- 下一步：PDF-a（用户裁决序第三项）——Acrobat-audit R5/R6 成果 →
  安全策略表 + pdfium/lopdf 三探针 spike + ADR-P1 选型定谳。
