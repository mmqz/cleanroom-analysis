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

---
Task ID: pdf-a-1
Agent: main-agent (Super Z, 会话 web-52b2ba37)
Task: PDF-a 侦察收口 + 三探针 spike + ADR-P1 选型定谳（用户裁决开工序第三项）

Work Log:
- 输入资产整合：acrobat-audit R1-R6（六轮结论）+ pdf-line/03（限额五层
  纵深 L0/L2/L3/L4/L7 + 三行为学 + JS 沙箱四件套 + 行业 JS 决策坐标）
  + pdf_carrier_plan §2 选型预判。
- pdfium 取件：fetch-pdfium.sh（版本 pin chromium/8086 + SHA256 首取
  回填复验闭环 + non-V8 构建断言）；args.gn 实证 pdf_enable_v8=false +
  pdf_enable_xfa=false（安全策略从构建配置层坐实）；licenses/ 全套传递
  声明已入包（NOTICES 素材，PDF-e 义务）。
- 三探针 spike（md-line/pdf-a/spike/，独立工程不入产品 workspace）：
  ①打开：3p/500p load 0ms + 文本抽取 19ms→0ms；②渲染：1200px 单页
  4ms / 连渲 5 页 17ms；③标注写：lopdf IncrementalDocument 增量写
  Highlight，原字节前缀逐位保持（1026B→1497B +471B），lopdf 重载 ✓，
  pdfium 交叉读回 type=Highlight ✓（双引擎互通坐实——PDF-d Acrobat
  互通验证的引擎层预证）。
- ADR-P1 定谳：pdfium + lopdf 双引擎分工（读写分离各取所长）；排除
  mupdf（AGPL）/ pdfium-v8（JS 引擎不接）/ 造轮子解析器；版本 pin
  chromium/8086 + pdfium-render 0.9.4 + lopdf 0.45.0；升级纪律三件一体
  （fetch 复验 + 三探针 + NOTICES）。
- 安全策略表 v1（SECURITY_POLICY.md）：S1 引擎能力面 / S2 危险对象族
  打开时检测（只读告警不执行）/ S3 限额五层（数值框架对齐 pdfcraft
  实测值）/ S4 沙箱进程边界（渲染工人池 + panic 隔离 + 增量写/渲染
  快照隔离）/ S5 加密签名面 / S6 诚实缺口登记（AcroForm formfill
  在 non-V8 构建中的存在性 = PDF-b 补探针）。
- 产品仓 gates.md 增 PDF-a 节（spike 数据入 gates = plan 门禁要求）；
  mdcore main 587246f→0d6ac2f 推送验证 local=remote。
- 本仓交付：md-line/pdf-a/{ADR-P1, SECURITY_POLICY, spike/}。

Stage Summary:
- PDF-a 收口完成：选型定谳 + 实测背书 + 安全规格基线全交付。三探针
  全过且数据强正向（500 页 0ms 打开 / 3.4ms/页渲染 / 增量互通）。
- 关键决策：双引擎分工（pdfium 读 + lopdf 写）；non-V8 构建为唯一
  取件形态；spike 属分析域不入产品 SBOM。
- 下一步（依用户裁决序）：画布线（线 A）P0——canvas-a Excalidraw
  深读 / canvas-d ComfyUI 协议 pass / office-a Univer 深读，按 11 维
  矩阵输出进本仓；PDF-b（mdc-pdf crate 骨架）待画布 P0 后依次。

---
Task ID: canvas-a-1
Agent: main-agent (Super Z, 会话 web-52b2ba37) + 四路 Explore 勘察代理
Task: 线 A 画布 P0 第一篇——Excalidraw 深读（11 维全量，用户裁决开工序第四项「画布依次」之首）

Work Log:
- 浅克隆 excalidraw/excalidraw @4c00f31（MIT 复核 ✓）；monorepo 结构勘察
  （packages/{excalidraw,element,common,math,fractional-indexing} +
  excalidraw-app 宿主）。
- 四路并行勘察代理（元素模型与编辑内核 / 渲染与性能 / 持久化与协同 /
  插件·AI·安全·质量·合规），~90 处证据锚点（路径:行号）。
- 综合 11 维深读报告：canvas-line/canvas-a_Excalidraw深读报告.md
  （含 dsh 插件面 + MCP 工具面双口径能力矩阵 + 10 条可移植结论 +
  与 mdcore 系工作台架构对照一页）。
- 核心发现摘要：
  ① 元素版本三元组（version/versionNonce/updated）+ 软删除 + 纯 JSON
    无本地状态 = 低成本 LWW 协同根基（与 mdcore document_id 哲学同源）；
  ② 双修改通道（不可变 newElementWith + 就地 mutateElement 属性级缓存
    失效）+ 单一提交边界 store.commit（与 apply_ops 单一变异入口同哲学）；
  ③ 撤销 = 快照 diff inverse-delta + 三态捕获（EVENTUALLY 手势聚合）；
  ④ 渲染 = 双 canvas + nonce 门控 + per-element 位图缓存 + 平移不动缓存，
    全量重绘路线万级元素天花板（frame.ts:815 自标瓶颈）= 差异化机会；
  ⑤ 协同非 OT 非 CRDF：加密快照广播 + 元素级 LWW + nonce 平局 + 20s
    全量重同步；能力链接即全部权限（零信任取舍）；
  ⑥ 敌意输入三层防线（restore 宁可丢元素 / embed 白名单 + 差异化
    sandbox / AI 生成不透明源隔离）+ 缺口（无 CSP、SVG 押注 img 隔离）；
  ⑦ AI 面 100% 无头化（applyDeltas/reconcileElements 原语天然 MCP 友好）。
- HANDOFF.md 更新：L3 行（canvas-a 完成）+ L-MD 行（R24-a/b/c + PDF-a
  收口状态同步）。

Stage Summary:
- canvas-a 收口完成：P0 三项之第一项交付。净室合规：MIT 确认、
  未发现 GPL 污染（依赖面抽查 + 【推断】机器复核建议）。
- 对工作台决策的直接输入：版本三元组/双修改通道/双 canvas 骨架列
  P0 移植；LWW 适用边界与万级元素天花板登记为架构决策点。
- 下一步：canvas-d ComfyUI 协议 pass（四问题：graph 模型/执行/通信/
  敌意输入）→ office-a Univer 深读；MD 线 R24-d 壳实体待开发环境。

---
Task ID: docking-verify-1
Agent: main-agent (Super Z, 会话 web-52b2ba37 续)
Task: 新会话入坞复核——两仓同步状态澄清 + mdcore HEAD 双绿门禁复验

Work Log:
- 澄清"ahead 5"假象：cleanroom-analysis 与 mdcore 上会话末均已实际推送完成，本地 tracking ref 失真导致误报；ls-remote 实证 cleanroom main=876ef9a / mdcore main=0d6ac2f，两仓 local=remote。
- mdcore HEAD 双绿门禁复验：scan.sh 0 命中 PASS；cargo test --workspace 187/187 全绿（与 gates.md 登记一致）。
- 凭证纪律延续：磁盘 ghp_ 全量扫描仅命中脱敏占位符（ghp_xxx/[已脱敏]），remote URL 干净，零真实凭证残留；本会话凭证仅以单命令环境变量形式经进程内存传递。
- 环境注意项登记：本沙箱 Bash 各次调用间 export 不持久，凭证传递沿用 HANDOFF 既有单命令模式。

Stage Summary:
- 两仓确认同步、门禁复验双绿，无代码变更；断点=canvas-d ComfyUI 协议 pass（线 A P0 第二项）→ office-a Univer 深读 → PDF-b。

---
Task ID: canvas-d-1
Agent: main-agent (Super Z) + 勘察代理 canvas-d-1a/1b + 复审代理 canvas-d-1r
Task: 线 A 画布 P0 第二篇——ComfyUI 协议 pass（graph 模型/执行/通信/敌意输入四问题）

Work Log:
- 浅克隆 comfyanonymous/ComfyUI @0df64eb（GPL-3.0 实证：LICENSE 674 行标准全文，
  规划期"待核"就此定谳→进程外协议集成为唯一路线）。
- 两路并行勘察代理：A（图模型+执行引擎）/ B（通信协议+敌意输入+安全治理）。
- 综合成文 canvas-line/canvas-d_ComfyUI协议pass.md（四问题 + 许可证定谳 +
  协议最小面六端点 + 安全姿势五条 + 引擎可移植资产六条）。
- 核心发现：①编辑图/执行图彻底分离（后端零解析 links/widgets_values，转换
  全在前端）；②执行=全局单队列串行 + prompt 内 async 并行 + merkle 式祖先
  签名缓存（上游变更自动失效下游）+ 增量拓扑溶解调度（非预计算拓扑序）；
  ③通信=REST 全路由双前缀 + WS 文本/二进制双通道（4 字节大端事件号）+
  /object_info 动态 UI 单一数据源 + 零认证（clientId 可踢旧连接）；
  ④敌意输入=custom node exec_module 任意代码（SECURITY.md 显式信任外包）、
  三目录路径沙箱（GHSA-779p 修复结晶）、模型无完整性校验、密钥走 prompt
  extra_data（仅队列脱敏）。
- 独立复审代理（canvas-d-1r，~50 锚点抽查全 7 节）：有条件通过→3 处修订
  全部落地（execution.py 实际在仓库根非 comfy_execution/、front 取负锚点
  移 server.py:1148-1150、workflow 透传措辞精确化为"仅提取 workflow_id"）。
- HANDOFF.md 更新：L3 行 + §6 线 A（canvas-d 完成）/ 线 B（R24+PDF-a 完成态同步）。

Stage Summary:
- canvas-d 收口完成：P0 三项之二交付。四问题全部实证回答，零代码复制净室合规。
- 对工作台直接输入：协议最小面（/prompt、/object_info、/history、/interrupt、
  /view、/ws）+ 集成安全五条（网关持证、沙箱隔离、路径沙箱复刻、模型哈希门禁、
  secrets 独立通道）+ 自研节点图内核六条可移植资产（merkle 缓存键/拓扑溶解/
  中断语义/PENDING 三态等）。
- 下一步：office-a Univer 深读（P0 最后一项）→ PDF-b mdc-pdf 骨架。

---
Task ID: office-a-1
Agent: main-agent (Super Z) + 勘察代理 office-a-1a/1b/1c/1d + 复审代理 office-a-1r
Task: 线 A 办公 P0——Univer 深读（11 维全量，P0 三项收官篇）

Work Log:
- 浅克隆 dream-num/univer @7166c69（v1.0.4；Apache-2.0 实证 176 行标准文本，
  无 NOTICE 无额外条款；内嵌 numfmt MIT）。
- 四路并行勘察：1a 架构+数据模型+编辑内核 / 1b 渲染+公式+性能 / 1c 协同+
  持久化+格式 / 1d 插件·AI·安全·质量·合规，~110 证据锚点。
- 综合 11 维报告 office-line/office-a_Univer深读报告.md（新建 office-line/），
  含 dsh/MCP 双口径能力矩阵 + 10 条可移植结论 + 与 mdcore 架构对照页。
- 核心发现：①快照即模型（Workbook 直持 JSON 原地改，undo/协同全外挂）；
  ②三层命令 COMMAND/MUTATION/OPERATION（视图态不进 op log）+ trigger 因果链；
  ③渲染=视口级双缓冲+条带 diff+前缀和二分虚拟滚动（与 Excalidraw
  per-element 路线分治）；④公式引擎=AST 解释器+RTree 依赖图增量重算+
  时间片让出+多层 LRU（504 函数）；⑤协同=docs 域 ot-json1 直嵌、sheets
  域 rebase 在服务端（Pro）；(sid,reqId) 幂等+checkpoint+断线补拉骨架完整；
  ⑥xlsx 导入导出全 Pro——开源办公引擎 OOXML 进出为真空=差异化蓝海；
  ⑦"harness"编排层全在生态仓（univer-cli/dsh-univer-office/univer-mcp），
  本仓只交付内核+Facade；⑧安全面：公式零远程/URL 白名单/权限三层 OSS
  完备，但 SECURITY.md 滞后、静态门禁弱于声明。
- 独立复审（office-a-1r，60+ 锚点抽查覆盖全 14 节）：0 阻断 0 重要，
  9 处轻微（锚点偏移/计数口径）全部修订落地。
- HANDOFF.md 更新：L3 行 P0 三项全部完成、§6 线 A 收官 + 主线索转 PDF-b。

Stage Summary:
- office-a 收口完成：线 A R1 P0 三项（canvas-a/canvas-d/office-a）全部收官，
  均经独立复审代理审定。净室合规：Apache-2.0 确认、零代码复制。
- 对工作台决策的直接输入：三层命令分类/公式引擎架构/协同版本骨架/
  agent 同一 op 通道四项列 P0 移植；OOXML 开源进出登记为差异化机会；
  op-first 与快照即模型的取舍再获实证。
- 下一步：主线索转线 B PDF-b（mdc-pdf crate 骨架，pdf_carrier_plan §5）。

---
Task ID: pdf-b-1
Agent: main-agent (Super Z) + 实现代理 pdf-b-1i + 独立审查代理 pdf-b-1r
Task: 线 B PDF-b——mdc-pdf crate 骨架（第二 Document Model，pdf_carrier_plan §5 第 2 轮）

Work Log:
- 实现代理交付 crates/pdf（mdc-pdf）：页/大纲/标注三层模型（Rotation 归一化/
  PageSpan 闭区间/OutlineTree 深度优先迭代/四 kind 标注 + Origin 溯源）+
  六命令边界（ExtractText/Search/Outline/AnnotateAdd/Edit/List，均带 Source）+
  typed 错误面五类 + 安全策略编码（S2 五危险族只读告警/S3 限额三行为学/
  S5 加密态写拒且先于存在性检查——反探测面）+ mdcore://pdf 协议草案
  （mdc-pdf-cmd/event 事件族）+ DocumentEngine object-safe 集成点（PDF-c
  绑 pdfium non-V8）+ CONTRACT.md 升版 v1.2 增 §6（__contractVersion 保持
  1.1——window 面零变化，仿 R24-b 先例）。
- 关键决策（gates 在档十条）：零引擎依赖延至 PDF-c（骨架轮不耦合原生库，
  workspace 无 pdfium 二进制可全绿；serde 系复用既有版本，Cargo.lock 仅
  +1 条目，NOTICES 义务后移）；Source 镜像而非复用（serde 名 + mdcore
  源面文件级读锁见证，零 path 依赖）；值时钟注入；搜索 v0 真实可执行
  （宿主注入文本面，PDF-c 换引擎载荷形状不变）。
- 独立审查代理（pdf-b-1r）：门禁四项亲测复跑 + 声明 10 条逐条验证 ✓；
  发现 1 阻断 + 1 重要 + 3 轻微：①PageSpan 派生 Deserialize 绕过构造
  校验——宿主畸形信封可致 debug panic/release 不可捕获 abort（实测复现）；
  ②Rect 同类绕过——反序矩形可经 patch 静默落库，PDF-d 将产出非法 /Rect；
  ③搜索两模式重叠语义不一致；④Source 镜像锁缺 rename_all 前置断言；
  ⑤enforce_snippet_chars 循环内重复求值。
- 主代理修复全部五项：PageSpan/Rect 字段私有 + 自定义 Deserialize（不变量
  回类型层）+ resolve_span 改走 new()；find_matches 统一非重叠语义（并修
  while 重写引入的 continue 标签死循环隐患）；镜像锁增严；循环不变量提出。
  +3 审计回归锁（反转 span JSON 拒收/反序 rect JSON 全链路封死/两模式
  计数一致）。
- 收口门禁：cargo --workspace 219/219（187 基线 + 29 骨架锁 + 3 审计回归
  锁）；clippy 零新增（mdcore-wasm 2 条既有基线）；scan.sh 0 命中；fmt
  过（crate 范围）；wasm32 check 过。mdcore main 0d6ac2f→408b109 推送
  验证 local=remote。

Stage Summary:
- PDF-b 收口完成：三层模型 + 命令边界 + 安全策略 + 协议草案 + CONTRACT
  v1.2 全交付，双绿门禁 219/219。审计-修复-回归锁闭环（用户要求的
  "子代理实现→新子代理审查"纪律首轮走通，阻断级缺陷被拦在合入前）。
- 遗留登记：S6 AcroForm 探针（spike 侧，PDF-c 前完成）；全仓 rustfmt
  格式化归 PDF-e 或独立提交；标注载荷健全性校验留 PDF-d 引擎写前。
- 下一步：PDF-c 阅读面（pdfium 接入 + 异步光栅化 + LRU 页缓存 + 文本层
  + 1000+ 页性能基线，性能门入 gates）。

---
Task ID: pdf-c-1
Agent: main-agent (Super Z) + 实现代理 pdf-c-1i + 独立审查代理 pdf-c-1r
Task: 线 B PDF-c——阅读面（pdfium 引擎接入 + LRU 页缓存 + 渲染工人 + 文本层 + 性能门）

Work Log:
- 实现代理交付（crates/pdf +2476 行）：cache.rs（PageCache 字节预算 LRU，
  HashMap+arena 双向链表 O(1)，零 unsafe 零依赖）/ render.rs（Rasterizer
  trait + BGRA 产物）/ worker.rs（专用渲染线程 + mpsc 双通道 + 取消/
  停机令牌 + catch_unwind panic 隔离=S4.2 落地）/ engine_pdfium.rs
  （feature 门：=0.9.4 pin、零 image crate 走低层 PdfBitmap 调用方缓冲、
  库定位序 env→候选→系统、OnceLock 单例失败可重试）/ document.rs
  attach_engine（命令面零破坏，search 复用 find_matches 单一出口）。
- 性能门实测（500 页 large.pdf）：打开 TTI 0.18ms / 首页渲染 8.2ms /
  顺序 50 页 p95 1.1ms / 翻页命中 5µs vs 未命中 2.1ms（~400×，二次
  取页 rasterize=0）/ 文本抽取 16k 页/s。阈值=实测×3 入 gates。
- 独立审查（pdf-c-1r）：**通过**（0 阻断 0 重要 8 轻微），声明 9 条逐条
  验证 ✓，亮点三项（工人排空设计/错误面工程/性能门诚实度）。
- 审计轻微项处置：修复四项（幽灵取消令牌防御=absorb_cancel 增
  job_watermark 双侧剪枝/attach_engine 页数一致性调用方合同入档/
  perf_random_flip 正名 perf_sequential_flip/Engine 512 钳制与
  put-更新超大自逐两回归锁）；在档不修两项（命中侧计时口径已注释
  明示；绑定失败重试路径无直接锁=进程单例语义限制）。
- 收口门禁：默认面 236/236（219+17 常开，裸环境零引擎依赖全绿）；
  feature 面 62/62（32+17+13）+ 性能门 6/6；clippy 零新增；scan 0；
  fmt 净；wasm32 check 过。mdcore main 408b109→7de5a34 推送验证
  local=remote。
- Cargo.lock +42 解析条目（默认构建零编译）——NOTICES 增量义务登记
  PDF-e SBOM 收口。

Stage Summary:
- PDF-c 收口完成：阅读面五件套（引擎/缓存/工人/文本/性能门）全交付，
  性能数字强正向（打开亚毫秒/翻页缓存 400×/文本 16k 页/s）——对标
  pdfcraft 实测值（PDF-a spike 对照）与竞品水位不落下风。
- 遗留登记：C 侧段错误进程隔离（PDF-e 强化面）；会话级限额接线与渲染
  看门狗留 PDF-d+/视图层；S6 AcroForm 探针（spike 侧）未做——下轮补。
- 下一步：PDF-d 标注面（lopdf 增量写 + v0 四标注往返语义锁 + Acrobat
  互通验证的引擎层预证）。

---
Task ID: s6-probe-1
Agent: main-agent (Super Z)
Task: S6 AcroForm formfill 探针（PDF-b 登记、PDF-c 轮补做的欠账闭环）

Work Log:
- spike 工程改造：src/lib.rs（fixtures 提升为公共面）+ src/bin/formfill_probe.rs
  （新增专项探针 bin）；fixtures.rs 增 build_form_pdf（1 页 + /AcroForm +
  /Tx 文本域 widget，lopdf 现场生成）。
- 探测两层：能力级 PdfDocument::form() Some/None + form_type 判别；行为级
  field_values 实枚举；对照组 = 无表单文档（small.pdf）。
- 结论（实测）：**formfill 模块存在于 non-V8 构建**——form() = Some、
  form_type = Acrobat（非 XFA）、/Tx 字段名值可读（"probe_text" =
  "hello form"）；对照组无表单文档 form() = None（判别干净）。
- SECURITY_POLICY.md S6 节定谳更新：表单字段 = 被动可读面非执行面（JS
  引擎已由构建层排除，字段 /A 与 format/calculate 脚本无解释器可跑）；
  处置纪律 = 按 S2 同款（打开时枚举进 SecurityReport 并告警，永不初始
  化交互式 formfill 环境执行动作，只读 field_values）。
- spike 双 bin 编译验证过（main 探针 + formfill_probe）。

Stage Summary:
- S6 已知缺口 #1 闭环。安全策略表 v1 全部条目要么已实证要么有明确排期。
- 下一步：PDF-d 标注面（lopdf 增量写 + 四标注往返 + Acrobat 互通引擎层
  预证）。

---
Task ID: pdf-d-1
Agent: main-agent (Super Z) + 实现代理 pdf-d-1i（两段叠加：首段超时遗留部分
改动 + 次段收尾定稿）+ 独立审查代理 pdf-d-1r
Task: 线 B PDF-d——标注面（lopdf 增量写 + 四标注往返 + Acrobat 互通引擎层预证）

Work Log:
- 交付（crates/pdf）：writer.rs（常开抽象：AnnotationWriter trait +
  IncrementalSave 统计 + MockWriter + /T origin codec + /M 纯函数日期串）
  / writer_lopdf.rs（feature engine-lopdf，=0.45.0 pin，照 spike 探针 3
  已验证模式：四 kind 标准 Annot 映射零私有扩展、间接 /Annots 三形态
  全覆盖、增量保存原字节前缀逐位保持）/ document.rs（attach_writer +
  flush_annotations：S5 先于写器 + dirty 集精确跟踪 + 幂等零追加）。
- 语义锁 24 条（pdfd_writer 15 + pdfd_session_flush 9）：四 kind 逐字段
  往返 / 前缀逐位保持（两轮传递）/ pdfium 交叉读（互通引擎层证据：
  subtype+/T+/Contents 全对上）/ Origin 往返含 UTF-16BE / /M 闰日与
  2100 世纪 / 加密双防线 / 幂等 / 编辑重脏 / 包络 Rect / 百条冒烟等。
- Acrobat 互通验证的诚实登记：引擎层证据（lopdf 写→pdfium 读）全对上；
  GUI Acrobat 人工目视 = 开发环境轮事项（沙箱无 GUI），不冒认。
- 独立审查（pdf-d-1r）：**有条件通过**（0 阻断 1 重要 5 轻微）——声明
  10 条全复跑实证，**叠加一致性确认**（两段代理工作无死代码残留）。
  修复：间接 /Annots 形态补回归锁 ⑯（审计员先仓外实证功能正确，补锁
  后三形态全覆盖）/ 曾加密空口令形态补锁 ⑰（was_encrypted 分支，S5
  双防线全形态闭环）/ 删零消费 pdfium_render 再导出（首段叠加遗留）/
  修测试 helper 结构性覆盖缺口（不解引用间接 /Annots）。
- 主代理修正过程中的自误三处（泛型 arity/Object API 形状/useless_asref）
  均由编译器与 clippy 门禁当场拦下——门禁纪律生效证据。
- 收口门禁四面：默认 245/245（236+9 常开）+ pdfium 71/71 + lopdf
  58/58 + 双引擎 88/88（含审计回归两锁）；clippy 双 feature 零警告；
  scan 0；fmt 净；wasm32 过；默认面零引擎依赖（cargo tree 实证）。
  mdcore main 7de5a34→eb466a2 推送验证 local=remote。
- NOTICES 增量登记：lopdf =0.45.0（MIT）入双引擎面，PDF-e SBOM 收口。

Stage Summary:
- PDF-d 收口完成：PDF 载体五轮计划的前四轮（a 侦察/b 骨架/c 阅读面/
  d 标注面）全部落地，双引擎读写闭环（pdfium 读 + lopdf 增量写），
  mdcore 语义锁总量 187→245（默认面）。
- 遗留：GUI Acrobat 人工互通验证（开发环境轮）；/DA 字体挂 /DR
  （GUI 轮）；渲染看门狗与会话级限额接线（PDF-e/视图层）。
- 下一步：PDF-e 阶段收口（门禁全复跑 + SBOM/NOTICES 增量 + gates
  PDF-e 段 + 双仓推送）或按用户裁决转线 A R2。

---
Task ID: t1-quality-audit-1
Agent: main-agent (Super Z) + 勘察代理 T1-survey + 独立复审代理 T1-review
  + 修复代理 T1-fix-a/b/c/d + 终审代理 T1-verify（六段写/审分离链）
Task: 轮1（用户开工序第一项）——MD 实现质量检查：全仓勘察 → 交叉复审
  → 修复 → 终审 → 双仓门禁全复跑

Work Log:
- 工具链入坞：Rust 1.99.0（wasm32/clippy/rustfmt）+ pdfium 取件
  （fetch-pdfium.sh，chromium/8086 non-V8，SHA256 验证过）+ spike
  fixtures 生成（small/large/small_annotated）+ 三探针复跑全过。
- T1-survey（全新视角勘察）：0 阻断 / 5 重要 / 7 轻微——F1 RenderWorker
  取消哨兵 off-by-one（id 0 排队取消被吞）、F2 ByAnchor 反转区间 panic
  （expect 分支切片违 typed-reject 纪律）、F3 S2 危险对象检测未实装 +
  族清单漂移（8 vs 5）、F4 看门狗/限额在档称启用实未接线、F5 fmt 门禁
  随工具链漂移（459 diff）；轻微 F6-F12。
- T1-review（交叉复审）：12 条零假阳性全确认；F5 计数修正 459/32；
  F12 升重要；新增漏报 O1（安全姿态漂移扩至 S1/S4/S6）/ O2（门禁口径
  失真 245 vs 实测）/ O3（attach 页数双权威）/ O4（snapshot 校验缺口）。
- 修复四代理：fix-a（mdcore 核：F2 AnchorError::Inverted 入口拒绝 +
  O4 SnapshotError 四变体落自定义 Deserialize + F10 失败全回滚含合并窗
  边界 + F9 文档；+5 锁 t1_quality_fixes）；fix-b（pdf crate：F1
  last_repaid→Option<u64> + O3 page_count trait + PageCountMismatch
  typed 拒绝；+2 锁）；fix-c（卫生：rust-toolchain.toml 钉 1.99.0 +
  fmt 492→0 清扫 + workspace.dependencies 统一（lock 零变化）+ r24 清单
  +pdf + 门禁口径文档 252/252 + feature 面 bootstrap 成文）；fix-d
  （S6 探针复原：**根因 = spike/.gitignore 裸模式 bin/ 吞掉 src/bin/**
  ——010b519 报文称入库实未入库；根锚定 /bin/ 修复 + 探针重建 +
  SECURITY_POLICY S6 复原注记；dual 面 89+1 → 90/0）。
- T1-verify（终审）：11 项全确认通过收口；4 条非阻断登记（基线勘误 /
  套件口径脚注 / 措辞精度 / trait 面增量）。
- 主代理权威门禁（本机全复跑）：默认面 252/252 + pdfium 73 + lopdf
  60 + dual 90（+6 ignored）+ 性能门 6/6 + scan 0 + fmt 0 diff +
  clippy 零新（wasm 2 条在档基线）+ wasm32 过 + 默认面零引擎依赖。

勘误登记（本仓 worklog 两条）：
- 0-bootstrap 条目所记 cleanroom HEAD=798f8bf 有误，正确基线 = b9ba950
  （798f8bf 为其第 15 代祖先；本条目按实修正）。
- T1-survey 记录的「223+ 测试 / 16 套件」口径失实（O2）：真实基线 =
  245 用例（修复后 252）；survey 工作记录原文保留不改，以本勘误为准。

Stage Summary:
- 轮1 完成：MD 实现质量检查闭环——2 个契约级代码缺陷（F1/F2）+ 2 个
  输入边界缺口（O3/O4）+ 门禁可复现性破洞（F5/F12）全部修复带锁；
  mdcore main eb466a2→3e450aa。
- 延后 PDF-e（轮2）：安全姿态对齐（O1：S1.1 CI 措辞 / S2 八族检测
  实装或措辞对齐 / S3 看门狗+L0/L7 接线或降级登记 / S4 单工人措辞 /
  S6 form 字段枚举进 SecurityReport）+ SBOM/NOTICES + gates PDF-e 段
  + 门禁全复跑 + 双仓推送。
- 下一步：PDF-e 阶段收口（即轮2）。

---
Task ID: t2-pdf-e-1
Agent: main-agent (Super Z) + 实现代理 T2-impl（E）+ 合规代理
  T2-notices（F）+ 审计代理 T2-review（G）+ 修复代理 T2-fix（H）
  + 复核代理 T2-verify2（五段写/审分离链）
Task: 轮2 PDF-e 收口——安全姿态对齐实装 + SBOM/NOTICES 增量 +
  gates PDF-e 段 + 门禁全复跑 + 双仓双分支推送

Work Log:
- T2-impl（E，含一次网关超时中断的遗留审计收口）：security_scan.rs
  344 行（engine-lopdf 面 8 族扫描 + S6 form_probe → OpenAssessment，
  产品面出口三打通）；RenderWorker 看门狗换代（Timeout 回执 + 毒化 +
  替代工人字节快照重开 + 在飞/未决令牌按提交序重放）；L0（×4 BGRA 实
  缓冲修正）/L4/L7 接线；object_depth 作扫描护栏；xref/解析深度 +
  seen-set 诚实登记为 lopdf 边界项；CONTRACT v1.2→v1.3；10 组短路验证
  （SC-A..J，含空转锁发现与重构）。
- T2-notices（F）：NOTICES v1.1→v1.2——cargo 30→90 + pdfium-bin 新类
  目 3（A-1 linux 实取件双 SHA 重算 + A-2/A-3 分发占位纪律）+ 总计
  42→105 零 UNKNOWN（118/118 lock 许可全解析，r-efi LGPL OR-uefi 透明
  注记）；四面 SBOM 29/45/76/90 并集=90 程序化实证；ASSET_CHECK 首个
  分发资产登记；R24-e 全量复跑预约兑现 + npm 面沿 R23 367（package-lock
  零 diff 实证）。
- T2-review（G）：通过收口（0 阻断/1 重要/3 轻微）——factory panic
  语义（注释与代码相反 + 静默死线程/看门狗泄漏/通道悬挂）+ licenses 18→
  17 + 畸形 /AcroForm 注释 + watchdog 零生产消费点登记。
- T2-fix（H）：五项终修——factory 移入渲染线程闭包 + catch_unwind +
  Retired 先置再 resume（断连不悬挂）；**实证勘误：T2-review 的现状
  方向描述反了（探针证明修复前工厂实在构造线程执行），处方目标态正确**
  ——诚实登记；新锁 22/23（断连判别 recv_timeout + /proc 探针 Linux
  cfg 门控）；README/COMPLIANCE 259 口径（在档=真值）。
- T2-verify2：通过——并发语义全写点枚举核证（Retired 无覆盖路径、锁
  先放再 resume、Fn 非 FnOnce 跨代独立 Arc）；259 口径四面静态复算自洽
  （259/78/80+6/108+6）；3 低 + 1 勘误均为文档级，登记 PDF-f+。
- 主代理权威门禁（本机全复跑）：默认 259/259 + lopdf 78/78 + pdfium
  80/80+6i + dual 108/108+6i + perf 6/6 + scan 0 + fmt 0 + clippy 零新
  （wasm 2 条在档基线）+ wasm32 过 + 默认面零引擎依赖。
- 分析仓权威文档对齐：SECURITY_POLICY（S1.1 无 CI 按实修正 / S2.4 实装
  / S4.1 单工人+换代 / S4.2 三机制 / S6 form_probe 实装 / 锚点 2、4 落
  地标记）；STATUS §5 进度 + §7 可开工项刷新；HANDOFF L-MD 行同步。

Stage Summary:
- 轮2 完成：PDF 载体五轮计划（a-e）全部收官。安全姿态「文档先于实现」
  漂移清零（O1 四面：S1/S2/S3/S4/S6 全部实装或按实修正）。
- mdcore main 3e450aa→03815b6；compliance-baseline 8c26272→1adced2；
  语义锁总量 252→259（默认面）+ dual 108。
- 遗留（PDF-f+ 宿主接线轮）：render_watchdog_ms 生产消费点、看门狗
  收报面登记、README 双引擎面口径补全、fuzz 战役排期；L0 引擎级端到端
  锁（冻结值下不可达，helper 面已锁）。
- 下一步：轮3 线 A R2-P1 批次（office-b ONLYOFFICE 深读 / office-c
  HyperFormula / canvas-b tldraw / collab-a Yjs pass）。

---
Task ID: t3-linea-p1
Agent: main-agent (Super Z) + 研究代理 I1-I4 + 复审代理 R3-R1..R3-R4
  + 修复代理 R3-fix（九段写/审分离链，4+4+1）
Task: 轮3 线 A R2-P1 批次——office-b / office-c / canvas-b / collab-a
  四项（深读×3 + 协议 pass×1）

Work Log:
- I1 office-b（ONLYOFFICE 11 维深读，六仓浅克隆）：一句话定性「内部二进制
  格式为真相源」的三段式（x2t C++/sdkjs JS/DocService Node）；服务器全序
  OT（change=可序列化 undo 项，undo/replay 替代 per-type transform）；
  保真=assemblyFormatAsOrigin+确定性重放装配；三段切分专项（四层 IPC/
  信任边界/无 OS 沙箱=结构性缺口）；693 行，130 锚实例。
- I2 office-c（HyperFormula 单件深读）：gridless 计算内核教科书实现——
  生产依赖仅 2（chevrotain+tiny-emitter）；**实测 423 函数**（369+53+
  OFFSET）；稀疏整数图+迭代 Tarjan+惰性 AST；v3.5.0 勘误（计划口径
  v3.4.0）+ org 勘误 handsontable；license 双门（Gate A #LIC!/Gate B
  粒度）空 key/gpl-v3 双路实测；深度炸弹 RangeError 逸出公有 API（缺口
  登记）；5 场景性能实测；AI 外置面 coming soon=窗口期判断（mdcore 抢跑
  机会）；616 行，131 锚。
- I3 canvas-b（tldraw 11 维深读）：shape util（record 纯 JSON+util 注册
  表）+全响应式信号 store；渲染=React/DOM+SVG 保留模式+自研 freehand+
  真 DOM 文本；协同=tlsync v8 git 式 push/rebase（**非 CRDT/OT**——与
  collab-a 对照关键事实）；AI 面=MCP exec「代码即操作」+focused proxy
  +流式执行（业界最前）；source-available 四层技术执行（ECDSA 验签/
  生产 5s 熔断/水印/遥测）；565 行，181 锚。
- I4 collab-a（Yjs 协议 pass，新建 collab-line/）：四问题全落（YATA
  工程化三件套 / subdoc 分片冷热 / 离线合并内生收敛三重随机锁 / provider
  生态三层硬边界）；**双印象核验：AFFiNE→Yjs 确认、Excalidraw 自托管
  协同→Yjs 证伪**（官方主线 socket.io+LWW+Firestore）；y/hub=AGPL 双
  许可例外；统一协同内核初答【推断】指向 CRDT 侧；664 行，~90 锚。
- 复审四链：R3-R1 office-b **需修补**（2 重要：六仓许可表述——sdkjs-
  plugins 实为 Apache-2.0；「零 MCP 引用」证伪——ROADMAP.md:22 已入
  MCP，窗口收窄信号漏登；锚点抽验 20/20）→ R3-fix 修补；R3-R2 office-c
  通过（实测复跑全一致：423 函数/license 双门/性能/深度炸弹；36 锚 33
  中）；R3-R3 canvas-b 通过（20/20 锚含 7 处修正锚复验；tlsync 定性逐
  行验证）；R3-R4 collab-a 通过（15/15 核心样本；证伪/确认证据链网络
  一手复抓全中；8 仓许可 9/9）。
- R3-fix：四报告 27 处处方统一修补（office-b 9 / office-c 7 / canvas-b
  6 / collab-a 3+2+回填——office-b 协同事实回填 collab-a §6.2/§6.4
  五处占位）；旧串 grep 零残留；新锚全部克隆核验。

Stage Summary:
- 轮3 完成：线 A R2-P1 四项全部落地并经独立复审+修补。报告总量 2538
  行、~532 锚实例；三线对照事实链成形（Univer 自研协议 / ONLYOFFICE
  服务器全序 OT / tldraw git 式 rebase / Yjs CRDT）。
- 关键裁决输入就绪：统一协同内核初答指向 CRDT（Yjs 侧），最终裁决留
  R2 收口综合（轮5）。
- 下一步：轮4 线 A R2-P2 前半（canvas-c draw.io/maxGraph / canvas-e
  渲染基座对照 / canvas-f InvokeAI / penpot-a / affine-a）。

---
Task ID: t4-linea-p2a
Agent: main-agent (Super Z) + 研究代理 J1/J1b、J2、J3/J3b、J4、J5/J5b
  + 复审代理 R4-R1..R4-R5 + 修复代理 R4-fix（含三次失败重试：
  J1/J3 网关超时、J5 超 200 轮上限——J1b 补救收口、J3b/J5b 收窄重试）
Task: 轮4 线 A R2-P2 前半——canvas-c / canvas-e / canvas-f / penpot-a /
  affine-a 五项（算法 pass / 对照 pass / UX pass / 架构 pass / 双态 pass）

Work Log:
- canvas-c（draw.io/maxGraph 算法 pass，792 行 150 锚）：布局族重写可行性
  评级表 13×5（简单族/FR/树 ★★★ 可按规格重写；层级 ★★；ELK/libavoid
  永远外置）；embed 25 命令协议；maxGraph JS→TS 重写与 Univer 互证。
  J1 网关超时遗留报告，J1b 审计补全（8 处计数修正 + worklog 补登）。
- canvas-e（渲染基座对照 pass，365 行 97 锚，14→15 张表）：Konva 色键
  命中区三重容错（snap 3/farbling 5 + Brave farbling 探测 + 10→256 环形
  搜索）；Fabric SVG 读侧 9 标签有损子集 + CVE-2026-44311；PixiJS 指令
  流批渲染断批三因 + 事件默认 passive；五路线光谱裁决表。
- canvas-f（InvokeAI Unified Canvas UX pass，550 行 ~130 锚）：**双强度
  灰度协议**（noiseLevel 前置加噪 + denoiseLimit 去噪上限，正交 0-1 双
  属性编码灰度 PNG）；三段协议（提交/接收/回流）+ 双重防陈旧；14 graph
  builder 自动构图（用户零图编辑，与 ComfyUI 手编辑图相反）。J3 超时
  后 J3b 沿 sparse-checkout 路线图重试成功。
- penpot-a（Penpot 架构 pass，426 行 121+30 锚）：三层混合渲染（SVG DOM
  形状层 + foreignObject 真文本 + **自研 CLJC flex/grid 2,564 行——
  浏览器只绘制不排版**）；帧级位图 LOD；**render-wasm 第二渲染线**（Rust/
  Skia/WebGL 512px 瓦片 + 三编译期 parity 守卫）；Rust 后端印象证伪
  （仅渲染层）；插件面 SES Compartment + MCP 在仓内；协同零 CRDT/OT
  （服务器行锁定序）。
- affine-a（AFFiNE 双态 pass，collab-line/，140 行 ~40 锚）：**「一份
  Yjs block 文档的两个视图投影」**——GfxCompatibleProps（xywh/index/
  lockedBySelf）mixin 挂载 + surface block Boxed(Y.Map) 收纯画布图形 +
  同 flavour 双根组件换绑 + undo 跨态保留；build-vs-buy 核验：官方博客
  in-repo 一手证实「拼装不可行→全量自研」+ vec.ts tldraw credit；精确
  时间线【推断：未能一手核实】诚实登记。J5 超 200 轮后 J5b 收窄（22 次
  调用）重试成功。
- 复审五链：R4-R1 canvas-c 通过（J1b 修正 8/8 成立；~80 锚抽验 98%
  精确）；R4-R2 canvas-e 通过（56 锚 98.2%；三大论断逐参数实证）；
  R4-R3 canvas-f 通过（65 锚 85% 严格/98% 内容；双强度协议端到端全链
  成立）；R4-R4 penpot-a 通过（68 锚 90%；六指定锚 6/6）；R4-R5
  affine-a 需修补轻量（turbo-renderer 三处定位方向修正——实为稳态位图
  优化层 opt-in 默认关闭）。
- R4-fix：五报告 37 处处方 + 9 项一致性联动全落地；旧串零活实例残留；
  技术论断零改动。

Stage Summary:
- 轮4 完成：线 A R2-P2 前半五项全部落地并经独立复审+修补。渲染光谱
  七路线成形（Excalidraw Canvas2D / tldraw DOM+SVG / Penpot 经典 DOM
  +排版自持 / Penpot wasm GPU 瓦片 / Konva 场景图 / Fabric 对象+SVG /
  PixiJS 指令流——AFFiNE 混合基座为第八点）。
- 双态决策输入就绪：AFFiNE「两视图投影」模式 + InvokeAI「画布态与生成
  管线三段协议」。
- 下一步：轮5 线 A R2-P2 后半（office-d 写侧管线 + 读侧对照）+ R2
  总收口综合（统一协同内核终裁 + 选型矩阵合成 + HANDOFF/STATUS 终态）。

---
Task ID: t5-linea-close
Agent: main-agent (Super Z) + 研究代理 K1r + 复审代理 R5-R1b + 修复代理
  R5-fixb + 综合代理 S1r + 终审代理 R5-R2b + 微修代理 R5-fix2b
  （含事故恢复：平台 08:50 重置工作区，前次轮5 全部中间产物丢失——
  此为全新重做轮，不引用丢失产物内容，一切重新一手验证）
Task: 轮5 线 A R2-P2 后半（office-d）+ R2 总收口综合（06 报告）
  + 线 A 阶段终态提交

Work Log:
- 事故与恢复：工作区重置（R5-fix2 网关超时为表征）；推送面零损失
  （三分支 ls-remote 复验：mdcore main=03815b6 / compliance-baseline=
  1adced2 / cleanroom-analysis=2086a20）；轮5 全套重做（K1r→R5-R1b→
  R5-fixb→S1r→R5-R2b→R5-fix2b 六段链）。
- office-d（写侧管线 + 读侧对照 pass，272 行）：docx 9.10 全内存+模板面
  最强（四 PatchType + estimatePageNumbers）；exceljs 唯一真流式
  （WorkbookWriter Archiver 管道+行级释放）；PptxGenJS stream() 与 README
  不符（整包 Buffer）；SheetJS CE Apache-2.0 壳仓化（npm 冻结 0.18.5）。
  **核心安全结论：CSV 公式首字符消毒四库零实现**（sheetjs 反而主动
  `'='+val.f` 输出）；**XML 控制字符三档**：损坏档（docx/pptxgenjs，
  拒开级）/ 剥除档（exceljs——合法 XML 静默丢字符，R5-R1b 依 PR #286
  证据链纠正）/ 保字符档（sheetjs `_xHHHH_`）；docx SimpleField 域指令
  直通 → **dsh 管线层消毒三闸**（字符过滤/CSV 首字符/域白名单七白三黑）。
  读侧目标线：docx-preview 级结构忠实+三处超额+数据骨架兜底。
  复审 R5-R1b 抓出 F1 高严重度失真（exceljs 档位）并修补；25 项回源
  65 锚 95% 命中。
- 06 综合报告（R2 总收口，458 行八节+两附录）：①协同内核五方链→
  三问建议（统一 CRDT 侧/Yjs 协议子集+yrs 起步+自研逃生舱/中心化
  权威快照+客户端离线自愈双层），反对因素三条诚实登记；②渲染八路线
  光谱+四口径选型矩阵+吞吐三招；③AFFiNE 两视图投影六条移植+风险
  R1-R6；④AI 调用面五方水位+差异化窗口（roadmap 已列、交付未发生，
  时效季度级【推断】）；⑤生成态 A 主（四件套直拼+消毒三闸）B 辅
  （触发条件）；⑥安全面三闸+八层汇总；⑦许可决策表 14 对象三档
  （ADR-C1 前提）；⑧D1/D2/D3 终裁 + yrs 评估 pass 最优先 + 七项触发
  条件表。终审 R5-R2b：25/25 忠实零失真、已知失真点（exceljs 三档）
  零残留、【推断】18→19/【建议】19 机械复点、「留用户裁决」12 处、
  零上游锚点；F1-F3 低/微级经 R5-fix2b 微修（条数口径 + §0.2 推断
  转写总则）。

Stage Summary:
- 轮5 完成：线 A 全阶段（R1 P0 三项 + R2-P1 四项 + R2-P2 六项 + 综合）
  收官。总计 14 篇报告 + 1 篇综合（06），全部经独立复审+修补。
- 待用户裁决（06 报告 §8.1）：
  D1 协同内核三问（统一性/实现层排序/中心化-离线取舍）；
  D2 双态立项（文档↔画布融合是否立项及范围）；
  D3 生成态主线（路线 A 四件套直拼为主确认）。
- 后续轮排队（06 §8.2/§8.3）：yrs 评估 pass（D1 裁决后第一动作）、
  Yjs 协议子集采纳设计、MCP 工具面窗口期执行、P3 动画线等七项触发条件。
- mdcore 两分支与产品状态不变（03815b6/1adced2）；PDF-f+ 宿主接线
  遗留项见 md-line/STATUS.md §7。

---
Task ID: t6-md-boundary-plan
Agent: main-agent (Super Z, 会话 web-52b2ba37 续) + 独立复审代理 t6-review
Task: 用户架构裁决后的《MD 引擎边界调查计划》起草（只读审计计划，先提交
审核后执行；本轮零代码改动、零凭据使用）

Work Log:
- 环境重建：平台工作区再次重置（mdcore/cleanroom 两克隆与全部本地状态
  丢失）；净克隆 cleanroom-analysis（HEAD=d9466fe，与远端一致实证）。
  mdcore 为私有仓且凭据未随会话保留（零落盘纪律），本轮不可访问。
- 状态对齐：读 HANDOFF/STATUS/06 综合/worklog 尾部 8 Task——前序会话
  已完成 R24-a/b/c + md-audit-2 + T1 + PDF-a~e 全链 + 线 A 全阶段
  （含 06 综合与 D1/D2/D3 终裁问题登记）。当前无未收口工程任务，
  待裁决项 = D1/D2/D3（06 §8.1）。
- 计划起草：md-line/boundary-audit/PLAN.md（253 行）——P0 基线校准 +
  P1-P6 对应用户七任务中的六项调查 + P7 架构决策输入汇总；每阶段含
  任务/依赖/验收/产物/风险五要素；证据分级五级（代码确认/测试验证/
  文档记录/推断/未知，文档记录级 = 本轮无凭据的诚实标注 + P0 强制
  落级）；基线表 B1-B14 全部标注文档级来源。
- 独立复审（t6-review）：有条件通过——1 重要（B11 门禁数字 pdfium/lopdf
  互换，正确 = lopdf 78/78 + pdfium 80/80+6i，出处 t2 权威复跑）+
  6 轻微（B3/B4/B6/B8/B10/B12 来源精度与署期核查）+ 2 建议（审核点
  增设证据分级裁决项 / B3 来源清理）——9 处修订全部落地；覆盖度核查
  7/7 任务映射完整；纪律合规五条全过（零选型/零凭据/零改动/五裁决
  忠实）。
- 日期核查：06 报告头部自署 2026-10-12 与其提交时间戳（d9466fe =
  2026-10-11 09:46 UTC）矛盾，判定为源报告署期笔误，PLAN 头部登记
  差异不改写。

Stage Summary:
- PLAN v1 草案完成并经独立复审修订，待用户审核（PLAN §8 六个审核点：
  阶段划分/文件落点/执行节奏/凭据前置/D1D2D3 联动/证据分级扩展）。
- 执行前置条件：用户重新提供推送凭据（环境变量单次传递，零落盘）——
  P0 需克隆私有仓 mdcore（预期 main=03815b6）+ 推送本仓。
- 本轮本地 commit（推送待凭据）；mdcore 仓零接触。

---
Task ID: t6-md-boundary-plan-r1
Agent: main-agent (Super Z, 会话 web-52b2ba37 续)
Task: 用户五点修订意见落地（PLAN v1 → v1.1）+ 48d0536 远端状态核实 +
凭据可用性验证（本轮零 commit、零推送、零克隆——修订以工作区改动落地，
待用户确认）

Work Log:
- 用户反馈"计划方向基本通过"+ 五点修订意见（mdc-pdf 参考案例化 / P0
  证据五级与阻塞记录 / P7 决策包不替裁决 / 只读负面清单 / 48d0536 核实）。
- 48d0536 核实（修订点 5）：本地 main 领先 origin/main 1 个提交（48d0536
  = PLAN.md 262 行 + STATUS §7 登记 + worklog t6）；匿名 ls-remote 实证
  origin/main=d9466fe → **48d0536 未推送确认**；未做任何重置/覆盖/重复
  提交，v1.1 修订以工作区改动落地（PLAN.md / STATUS.md / worklog.md），
  commit+push 方式待用户确认（不重写历史，v1.1 作为新提交追加）。
- 凭据验证：用户本轮提供 PAT（仅命令行单次传递，零落盘零复述）——
  ls-remote 实测 mdcore 三分支可读：main=03815b6 / compliance-baseline=
  1adced2（HEAD 指向）/ mother/r23-snapshot=24268ca，与 PLAN P0 预期
  完全一致；cleanroom-analysis 匿名可读（公开仓实证）。用毕零残留验证：
  本轮实际凭据值全仓零命中；仓内既有 12 处 "ghp_" 字样经逐条甄别均为
  检查机制的模式字面量（push 脚本用法示例 / 扫描正则）与历史脱敏记录，
  无一为真实凭据值。
- PLAN v1.1 修订五点全落地：① mdc-pdf 降格参考案例（§1.1 O4 / §1.2 /
  P1⑦ / P5⑥ / P6④ / P7② 六处，"不得直接推导画布应采用相同接入方式"
  入文）；② §2 纪律新增④阻塞记录规则（历史报告不得替代当前验证）+
  §3 B11 历史数字注记 + P0 任务/验收/风险联动；③ §0 末段"裁决输入
  而非裁决本身" + P7 任务重构 D1/D2/D3 四要素决策包（事实依据/约束/
  备选方案/验证缺口）+ P7 验收增"零终裁性表述"复核项；④ §1.2 只读
  边界负面清单五条；⑤ §8 审核点状态更新（点 4 凭据已解决 / 点 6 分级
  已确认 / 点 3 节奏仍待裁决）+ §9 版本表 v1.1 行 + STATUS §7 第 7 项
  一行同步。
- 遵"不要开始实施"：mdcore 零接触（未克隆），未执行任何 P0-P7 审计
  任务。

Stage Summary:
- PLAN v1.1 修订稿完成（工作区态、未 commit），待用户复审；剩余审核项
  = 执行节奏（§8 点 3）与修订稿整体认可。
- 执行就绪度：凭据已验证可用、mdcore 三分支 SHA 与预期一致；开工即
  P0（克隆 + 门禁复现 + B1-B14 基线校准）。
- 凭据阻塞项：无（本轮实测）；执行期失效预案 = §2 纪律④。

---
Task ID: t7-md-boundary-p0
Agent: main-agent (Super Z, 会话 web-52b2ba37 续)
Task: 用户 v1.2 范围收缩裁决落地（PLAN v1.1→v1.2）+ P0 启动——阻塞登记
（凭据未随会话保留）；本轮零 commit 零推送、mdcore 零接触、零代码级结论

Work Log:
- 用户裁决接收（2026-10-12）：v1.1 五项修订通过；主目标收缩为"MD 现有
  模块复用审计"（非统一引擎/协同架构研究）；P0/P1 保留原案；P2-P6 收敛
  六题（模块依赖/状态与序列化/编辑事务撤销/扩展接口/性能耦合/PDF 复用
  风险）；协同仅调查现状和约束、不做完整 CRDT 选型；P7 改复用方案报告
  （逐项三分类 × 四要素：代码证据/适配成本/对 PDF 质量的影响/验证标准）；
  D1/D2/D3 仅记录相关发现、不要求本轮架构终裁；本轮零 commit 零推送；
  mdcore 结论必须来自本轮实际访问与核验。
- PLAN v1.2 落地（309 行重写）：§0 三必答改写为复用三问（三分类/成本与
  验证标准/对 PDF 质量的影响）+ 协同降级为附属登记；§1.1 O4 双重角色
  （复用先例 + PDF 质量基线对象）；§1.2 追加两条非目标（不做统一引擎与
  协同架构研究；本轮零 commit 零推送）；§2 追加纪律⑤（本轮实际核验，
  历史文档不代答）；P1 增⑧模块依赖方向图；P2 改"模块依赖与状态序列化"
  （增①依赖闭包清单）；P3 增④复用先决条件清单；P4 改"协同现状与约束
  登记"+零选型验收；P5 六改"外部消费者五能力四态判定表"（直接作 P7
  中间产物）；P6 改"性能耦合与 PDF 复用风险"+新增④三项（依赖面图谱/
  波及面/质量敏感点清单）；P7 重构为复用方案报告（三分类 × 四要素 +
  D1/D2/D3 仅发现 + 终审零终裁表述机械复核）；§5.2 本轮零 commit 纪律 +
  R7 取舍已知悉登记；§7 R1 升"本轮已发生一次"；§8 审核点五点已裁决、
  凭据待重新提供；§9 v1.2 行。
- P0 启动尝试与环境实测（2026-10-11 16:07 UTC）：GITHUB_TOKEN 未设置
  （env 检查 NO）；匿名 git ls-remote mdcore → fatal: could not read
  Username（私有仓要求认证）；匿名 api.github.com/repos/mmqz/mdcore →
  HTTP 403；本地无 mdcore 克隆（文件系统检查）；~/.git-credentials、
  ~/.netrc 不存在（零落盘纪律合规）；cleanroom-analysis main=48d0536
  （ahead 1，origin/main=d9466fe）+ PLAN/STATUS/worklog 三文件 v1.1
  工作区改动在档（与 t6-r1 登记一致）。
- 阻塞登记（§2 纪律④执行）：新建 baseline.md——§0 BL-1~BL-3（克隆/门禁
  复现/B1-B14 对账三项阻塞，原因 = 凭据未随会话保留，影响面 = P0 全部 +
  P1-P6 连带，解除路径 = 用户单次传递 PAT）；§1 环境实测七项在档；§2
  P0 执行清单八步（含克隆后 remote set-url 清除凭据 + 全仓零残留验证
  程序）；§3 B1-B14 对账表骨架 14 行全部"待执行"。
- 遵用户指令核对：本轮【零】mdcore 代码级结论（PLAN §3 全部维持
  【文档记录】级）；零 commit 零推送（两仓 git 写操作零执行）；工作区
  改动 = PLAN.md（v1.2）+ baseline.md（新建）+ STATUS.md §7 第 7 项一行
  + 本 worklog 追加——全部在 PLAN §5.1 允许写面内。

Stage Summary:
- PLAN v1.2 完成（执行基线就绪）：范围收缩六点全落地；执行节奏已裁决
  （P0-P1 先行回报，方向确认后继续）。
- P0 阻塞在档：凭据未随会话保留（零落盘纪律预期后果，非失效），
  BL-1~BL-3 登记；执行清单已备（baseline.md §2 八步），PAT 到位即跑。
- 下一步（凭据到位后）：P0 八步（ls-remote 验证→浅克隆+URL 清除→零
  残留验证→工具链对齐→门禁四面复现→结构盘点→B1-B14 对账落级→只读
  红线验证）→ P1 结构勘察（入口/调用链/DocumentEngine 形状/模块依赖
  方向图）→ 回报调查方向供用户确认。

---
Task ID: t7-md-boundary-exec（P0-P7 全阶段执行，2026-10-12 用户开工指令后连续完成）
Agent: main-agent (Super Z) + 勘察代理 t7-p2p3-survey / t7-p4-survey /
  t7-p5-survey / t7-p6-survey + 复审代理 t7-p2p3-review / t7-p4-review /
  t7-p5-review + 终审代理 t7-p7-final-review（P6 复审合并覆盖）
Task: MD 现有模块复用审计 P0-P7 全阶段（PLAN v1.2 执行基线；全程只读，
  mdcore 零改动；连续执行不停工——用户 2026-10-12 指令）

Work Log:
- 环境：用户消息内提供 PAT（环境变量未实际注入，Bash 每调用为新 shell
  ——逐命令内联传递，零落盘零残留实测）；本仓 v1.2 改动 commit bbbc6af
  推送（远端=本地实证，48d0536 历史保留）；mdcore 浅克隆 main=03815b6
  （extraheader 认证，.git/config 干净 + 全仓 token grep 零命中）。
- P0（主代理）：三分支 SHA 验证一致（main=03815b6/compliance-baseline=
  1adced2/mother=24268ca）；工具链 1.99.0 重建（rustup minimal + wasm32
  + rustfmt/clippy）；pdfium 取件（chromium/8086 non-V8 SHA256 验证）+
  fixtures 现场生成（spike 双探针 + formfill_probe）；门禁九项全绿复现：
  默认 259/259 + lopdf 78/78 + pdfium 80/80+6i + dual 108/108+6i + perf
  6/6（打开中位 0.17ms/首页渲染 8.21ms/p95 0.97ms/命中 5.41µs/文本抽取
  26.48ms@500 页）+ scan 0 + wasm32 过 + clippy 2 基线零新 + fmt 0——
  B11 零漂移；workspace 结构盘点（5 crate 实测，B1 口径修正）；B1-B14
  对账：11 确认 + 1 证伪校准（B12：sync_resync=本地重同步非协同体系）+
  3 处差异登记（B1 crate 数/B13 补 ENTROPY_SEQ/B14 表述微调）→
  baseline.md（113 行）。
- P1（主代理亲读 ~9K 行核心源码）：模块地图（O1-O5）/ crate+模块双层
  依赖方向图（无环；核心五件闭包极小；mdc-pdf 零 path 依赖）/ 四条调用
  链全锚点（编辑→事务→op→树→历史；加载→adopt→指纹→会话；设置→
  sidecar→事件；桥→核→回推）/ DocumentEngine 四方法 object-safe 形状 /
  核侧零渲染器确认 → p1-structure.md（162 行）。
- P2+P3（勘察代理 + 复审全量核验）：依赖闭包表（含 fingerprint/external/
  document 三文件 impl Core 反向依赖——P1 勘误补录）；Block ID 三路保真
  分层（snapshot/replay 全保真；save→open 条件保真——先序 1..n 递增时
  才一致）；增量保存三态=无（宿主面）；op 求逆矩阵全成员；typed reject
  7 类 23 变体；五项不变量代码+测试双背书；**settings.undo_coalesce_ms
  死设置**（存储链齐备、should_coalesce 直读常量、全仓零消费者）；**
  coalesced 事务不 bump revision**（同号多记录→revision 非全局唯一）；
  复用先决条件四项；测试锁缺口 6 项。复审：213 锚点 100% 核验零误报，
  2 轻微（html.rs serde 单元格/r17_utf16 计数）已回写。
- P4（勘察 + 复审）：六项协同能力全缺失（多端同步/OT/CRDT/广播/离线/
  自动冲突——grep 搜索记录在档）；三边界判定成立（take_events 非广播/
  adopt_external 非离线合并/OfferChoice 非自动冲突）；B12 独立确认证伪
  校准；约束事实 C1-C10 + K1-K8（单写者模型/门禁语义/revision 三路径
  持久化）；零选型机械扫描通过。复审：83 锚点核验、六判定独立复核全
  一致，3 轻微（K4 锚点区间/五 crate 口径/broadcast 关键词注记）已回写。
- P5（勘察 + 复审）：五能力四态矩阵（命令调用=可直接/变更订阅=需适配
  [推送多订阅缺失]/按 ID 取块=需适配/块级监听=需适配[单块订阅缺失]/
  序列化访问=可直接）；契约三面对照 4 项漂移（F-1 __wasmMemory/
  __heapTrace 非恒在且门禁不可见——实质漂移；F-2 CONTRACT §3:62 版本
  号残留 1；F-3/F-4 弱漂移）；P1 勘误（wasm 导出 19→23 实为 22 方法+
  构造器；undo/redo 行号 :79/:85）。复审：150 锚点全量核验五格全成立，
  2 轻微（p1:156 残留/启动面计数 9 项）已回写。
- P6（勘察；复审由终审合并）：全局可变状态全仓恰 3 处（ENTROPY_SEQ
  原子/PDFIUM+BIND_LOCK OnceLock，多 Core 实例无静态障碍）；性能纪律
  合规（MD 面仅 1 条软冒烟断言 + r14 打点，B-1..B-10 待基准清单）；
  PDF 复用风险三项：依赖面=编译级零依赖 + 3 文件级耦合点（Source 语义
  镜像锁**单向**[mdcore 新增第六值静默漂移]/scheme 命名/版本单源）；
  波及面=抽取 op/history/tree → pdfb_skeleton 镜像锁读 mdcore 路径会破
  （测试面，非 lib 面）；敏感点 Q1-Q8（增量写前缀逐位/交叉读互通/四
  kind 往返/flush 事务/perf 门 6/6/安全八族/看门狗/装配对账）守护测试
  全部落在 mdc-pdf 自有代码内——**复用 mdcore 模块不触碰 PDF 质量守护
  面**；PDF 面无 undo 系统（Source::Undo/Redo 零消费）→
  p6-perf-coupling-pdf-risk.md（214 行）。
- P7（主代理汇总 + 终审）：REPORT.md（160 行 ≤300）——三必答逐条锚点
  背书；三分类 12 项（R1-R5 可直接复用/A1-A5 需最小适配/X1-X2 跨域
  不可复用[域内 R3]）× 四要素（代码证据/适配成本/对 PDF 质量的影响
  [12/12 显式作答]/验证标准）；协同附属登记；D1/D2/D3 仅发现（零终裁
  表述）；内容引用一行登记；质量缺口 5 组 + 遗留未知 U-1..U-5。终审
  （P6 复审 + P7 八项验收合并）：P6 通过（4 轻微计数已回写：9 锁/6 处
  str/十二字段/set 位点）；P7 八项全过（三必答/四要素完整性/零终裁/
  零选型/零性能宣称/引用忠实性 20+ 抽查/行数/无 P6 污染），4 轻微
  （复审范围限定表述/节号×3/U-4 推断级别）已回写。
- 只读红线：mdcore 全程 git status --porcelain 零改动（每阶段复核）；
  两仓 commit/push 仅本仓（用户指令允许按既有纪律保存文档成果，推送
  前远端核实无覆盖）。

Stage Summary:
- P0-P7 全阶段完成：8 份产物（PLAN 已在档 + baseline 113 行 + p1 162 +
  p2 150 + p3 150 + p4 262 + p5 155 + p6 214 + REPORT 160 行）全部经
  独立复审或终审核验，锚点抽查合计 500+ 零事实性错锚。
- 核心结论：MD 引擎 12 项复用单元三分类就绪；mdc-pdf 与 mdcore 零编译
  依赖，PDF 质量（布局/分页/保真）守护面全部在 mdc-pdf 自有代码内，
  复用 MD 模块对 PDF 质量无波及（唯三耦合点已锚点登记，Source 镜像锁
  单向为最重要缺口）；协同现状 = 零实现 + C/K 约束清单在档。
- 待用户裁决事项（REPORT §4/§6，本轮不动手）：D1/D2/D3 三裁决的事实
  输入就绪；5 组质量缺口（Source 镜像锁单向/undo_coalesce_ms 死设置/
  P3 六项测试锁缺口/P5 四项契约漂移/MD 面基准空位）全部登记留裁决。
- mdcore 两分支 SHA 不变（03815b6/1adced2，零接触零改动）。

---
Task ID: t7-u1-webgate-recheck（P7 收尾补验，隶属 t7-md-boundary-exec）
Agent: 主代理
Task: 消除 REPORT.md U-1（web 三门禁本轮未复验）——环境具备后复跑
契约门 / smoke / IME CDP 三门禁

Work Log:
- 前置恢复：子仓 26 文件模式位 755→644（环境侧 chmod 副作用，零内容
  改动，git status 归零）；确认 dfd41ae 已在 origin/main，无未推送提交；
  mdcore 工作区干净、main=03815b6 不变。
- 环境探测：node v24.21.0 + npm 11.19.0 + Python playwright + chromium
  缓存（1200/1248）齐备；wasm-bindgen CLI 补装 0.2.129（与 Cargo.lock
  精确一致，cargo install --locked）。
- 构建链（产物全在 .gitignore 覆盖面：pkg/dist/node_modules/target）：
  cargo build wasm32 --release 20.44s + wasm-bindgen → pkg/ + esbuild →
  dist/（main.js 531KB + wasm 704KB）。
- 三门禁复跑（web/tests/，2026-10-12，全部 stdout 全绿在档）：
  * contract_gate.py → 16/16 全绿（boot surface / bridge inventory /
    inject 族 6 检 / chunk progressive / undo key / unit constitution /
    settings 三检）；
  * web_smoke.py → 15/15 全绿（1MB/20448 块实测：typing p95 12.6ms /
    fallback sync 282ms / undo 重渲染 136.1ms）；
  * ime_cdp.py → 28/28 全绿（IC1-IC6 组合流全量 + CDP 点击不强制提交
    的路径限制登记与基线口径一致）。
- 只读红线：tests/results/*.json 为运行产物（本克隆不入库），提取
  汇总数字后整目录删除；mdcore git status --porcelain 归零复核。
- 回写：REPORT.md 头注补验说明 + §6 守护段三门禁升级【测试验证】+
  §7 U-1 行标记已消除（表名同步微调）；STATUS.md §7.7 补一行；本段
  worklog 追加。

Stage Summary:
- U-1 消除：web 三门禁从【文档记录：本轮未复验】升级【测试验证】
  （16/16 + 15/15 + 28/28）；U-2..U-5 维持原状（分别需新增往返测试/
  建基准/桥侧校验/开发环境轮，均超出本环境或本轮范围）。
- mdcore 全程零改动零提交零推送（构建/测试产物均在 .gitignore 覆盖
  面或已清理）；REPORT.md 补验后 164 行仍 ≤300。

---
Task ID: t8-plan（复用地基实施轮：启动 + 平台回滚恢复）
Agent: 主代理
Task: 按用户 2026-10-12 六点指令（审计收口进入开发）提交实施任务清单，
随后连续执行；先行恢复平台工作区回滚事故

Work Log:
- 事故与恢复：平台将 /home/z/my-project 回滚至 10-11 快照——本仓 .git
  退回 48d0536（bbbc6af/dfd41ae/eb18074 三提交从本地消失，p1-p6/REPORT
  磁盘丢失）；mdcore 目录清空；~/.cargo 工具链全丢（playwright 缓存与
  node 幸存）。恢复：fetch 核验远端 eb18074 完好 → 磁盘四文件备份对比
  （全部为提交前旧稿，远端版更完整=零内容损失）→ reset --hard
  origin/main → 八件套/ worklog/STATUS 全部恢复；mdcore 完整重克隆
  （默认分支为 compliance-baseline，checkout main=03815b6 核验；凭据
  extraheader 单次传递，.git/config 实测零残留）；rustup 1.99.0 +
  wasm32 + rustfmt/clippy 前台重装成功。
- 事实核对（计划输入）：重读 p1/p2/p3/p5/p6/baseline 全文 + 代码落点
  快查——settings.rs 对 mdcore 引用仅 :102 一处常量默认值（零 Core
  依赖，抽取缝极小）；main.ts 全部 apply_ops 调用点仅 'view' 两处
  （:846/:1027，T2c 校验化零破坏面）；镜像锁原文 pdfb_skeleton.rs:
  1040-1079 确认为"在场检查"（无多余变体检查=单向缺口实证）；r24_
  settings.rs 经 mdshell::settings:: 导入（re-export 可保路径稳定）、
  :365 defaults_align_with_core_constants 可继续背书抽取后单源。
- 计划：md-line/reuse-impl/PLAN.md（v1）——分诊表 7 项（4 先修 3 登记）
  + 任务 T0/T2a-d/T3/T4/T5（每项模块/复用依据/风险/验收）+ 六条全程
  不变量（PDF 质量零波及/契约面稳定/零新依赖/合规/Git 纪律/证据分级）。
  范围裁决转录：不做通用引擎/统一 CRDT/双向投影/D1-D3/A1-A3/session
  泛化/Office 引擎/增量保存接线/undo_coalesce_ms 接线（后两项登记留
  用户裁决）。

Stage Summary:
- 平台回滚事故零损失恢复（远端为唯一真源，磁盘旧稿未覆盖任何新内容）。
- 任务清单已提交（用户指令 6 前半），随即连续执行 T0→T5。
- mdcore 起点 main=03815b6（与审计基线一致），本轮起进入开发态写面。

---
Task ID: t8-exec（复用地基实施轮：T0-T5 连续执行收口）
Agent: 主代理
Task: 按 reuse-impl PLAN v1 连续完成 T0-T5（用户指令"先交清单后连续
执行"），mdcore 进入开发态写面

Work Log:
- T0：改前 12 门禁全绿认证（九 Rust 门禁 259/78/80+6i/108+6i/perf6/
  scan0/wasm32/clippy2基线/fmt + web 16/15/28）；pdfium 取件
  （fetch-pdfium.sh chromium/8086）+ fixtures 现场生成（spike 双探针
  全过）+ wasm-bindgen 0.2.129 重装。
- T2（提交 26aa4af）：镜像锁双向化（新增 source_mirror_bidirectional_
  exact_count：mdcore 侧恰五变体计数锁 + pdf 侧编译期穷尽锁；红绿演示
  = 注入 Ghost 第六变体精确报警后撤销）；t3_reuse_locks.rs 四条现状锁
  （G1 清 redo / G4a ID 重编号 [1,4,2,3]→[1,2,3,4] / G4b 撞号
  [1,2,3,3,4] 实测成立 / G6 coalesced 共享 revision）；wasm source
  typed reject（parse_source_str 纯函数 + 2 单测，main.ts 全调用点
  'view' 零破坏）；契约卫生（CONTRACT §3 勘误渐进专属 + 版本残留
  1→1.1；契约门 16→18 检含 F-1 双侧现状锁 + F-3 __view）。
- T3（提交 f08e376）：crates/settings（mdc-settings）独立 crate——
  settings.rs 整体迁移（rename 血缘 96%）；:102 唯一 mdcore 引用改本地
  常量（r24_settings:365 跨 crate 锁继续背书单源）；shell pub use
  re-export 路径零破坏（r24_settings/contract_freeze 原样过）；闭包锁
  测试（行首匹配——首版整文件 contains 被自身描述误伤，红绿校准）；
  r24_packaging 版本锁同步六 crate；toml_edit 随迁零新依赖。
- T4（提交 15efc28）：REUSE.md 91 行（12 单元转录 + 快速指引 + 九门禁
  基线 + 六条消费契约注记；零上游品牌词 scan 0）。
- T5：终跑全绿——G1 267/267（259+7 增量）+ G2 79 + G3 81+6i + G4
  109+6i + G5 perf 6/6 + scan 0 + wasm32 + clippy 2 基线 + fmt 0 +
  契约 18 + smoke 15 + IME 28；mdcore 工作区归零；运行产物清理。
- 实测新发现三条：G4b 撞号机制成立（P2-U2 推断→测试验证，撞点 p2/p3
  与预测 p3/p4 不同机制一致，修复留裁决 R-2）；G4a 漂移具体形状（ID
  语义转移）；P1"settings 用 Core"校准为仅常量取源（抽取缝极小的证据）。
- Git 纪律：三提交均门禁绿后提交；推送前 fetch 核实远端（03815b6 无
  位移）；PAT extraheader 单次传递，每次推送后 .git/ grep 零残留。

Stage Summary:
- 用户六点指令全部落地：阻断缺口先处理（4 项）+ 唯一结构抽取
  （settings 域无关化，三域可直接消费）+ 复用工具包（Office 后续按需
  复用的入口文档）+ 连续执行零停工 + 每阶段汇报三要素齐备。
- mdcore main：03815b6 → 15efc28（三工作流提交）；PDF 四面 + perf
  改前改后全绿 = 不降质证明；零新第三方依赖（NOTICES/SBOM 零增量）。
- 遗留登记 R-1..R-6（死设置接线/撞号修复/栈上限/F-4/基准/真机壳）
  全部留用户裁决，无阻塞项。未知项：U-2/U-4 本轮消除，U-3/U-5 维持。

---
Task ID: t9-a（继续推进轮启动：平台回滚恢复 + 基线复认证 + clippy 口径修复）
Agent: 主代理
Task: 用户指令"继续推进"（2026-10-12，凭据环境变量传递）——恢复再次被
平台回滚的工作区，12 门禁基线复认证，处置复认证中发现的 t8 漏检项

Work Log:
- 平台回滚恢复（第二次，同 t8 事故模式）：本仓 .git 退回 48d0536（五提交
  从本地消失），boundary-audit 八件套磁盘丢失；mdcore 目录被清空。
  恢复：fetch 核验远端 dc8c6a9 完好 → 本地未提交改动核实为提交前旧稿
  （远端版更完整）→ reset --hard origin/main → 八件套全部恢复；mdcore
  重克隆（checkout main=15efc28 核验，.git/config 零残留复核）。
- 环境重建（~/.cargo 全丢）：rustup 1.99.0 + wasm32 target + rustfmt/
  clippy 前台重装；pdfium 取件（fetch-pdfium.sh chromium/8086，
  SHA256 验证过）；fixtures 现场生成（pdf_a_spike 三探针 + formfill_probe
  S6 探针全过，form_probe.pdf 首次缺失致 G3/G4 首跑 13+1 失败后补齐）；
  wasm-bindgen-cli 0.2.129（与 Cargo.lock 精确一致）；web npm ci +
  esbuild dist 构建。
- 12 门禁基线复认证（mdcore 15efc28 零改动状态下）：G1 267/267 +
  G2 79 + G3 81+6i + G4 109+6i + G5 perf 6/6 + G6 scan 0 + G7 wasm32
  过 + G9 fmt 0 + 契约 18/18 + smoke 15/15 + IME 28/28 = 与 t8 收口
  口径逐项一致；**G8 clippy 偏差**：四面 --all-targets 口径实测 3 条
  （2 wasm 在档基线 + 1 条 26aa4af 引入的 pdfb_skeleton:1107
  manual_pattern_char_comparison）——t8 REPORT"2 基线零新"系口径收窄
  （--workspace 不含测试目标）所致漏检。
- 修复（mdcore d3123ba）：.split 闭包改字符数组模式（语义零变化，红绿
  = dual 109+6i 复验不变）+ REUSE.md §4 clippy 命令恢复 gates.md
  PDF-e 起四面 --all-targets 口径；修复后四面实测仅 2 条 wasm 基线
  零新增；REPORT §7 勘误段回写（诚实登记纪律）。
- Git 纪律：门禁绿后提交；推送前 ls-remote 核实远端 15efc28 无位移；
  PAT extraheader 单次传递，推送后 .git 零残留复核。

Stage Summary:
- 工作区与环境第二次零损失恢复；12 门禁基线全数复现。
- t8 唯一漏检项（clippy 口径回退）已修复并在两仓登记勘误。
- mdcore main：15efc28 → d3123ba（一卫生提交）。
- 下一步（本会话连续执行）：t9-b office-impl PLAN v1 任务清单提交——
  依据 = 用户六点指令第 3 条（Office 按自身需求开发）+ t8 PLAN §0
  "Office 引擎（按需另轮）"的另轮启动 + REUSE.md 复用入口就绪。

---
Task ID: t9-b（office 开发启动轮：任务清单提交）
Agent: 主代理
Task: 按用户六点指令第 3 条 + t8 PLAN §0 "Office 引擎（按需另轮）"登记，
提交 Office 首开发轮任务清单（指令 6 纪律：模块/复用依据/风险/验收）

Work Log:
- 依据链整理：用户指令第 3 条（Office 按自身需求开发/优先复用/不预设计
  跨格式抽象）→ t8 PLAN §0 明示"Office 引擎（按需另轮）"→ REUSE.md
  T4 即"Office 后续按需复用入口"→ 本轮启动。MD 线 R-1..R-6 全部留
  裁决、L3 后续轮 D1 门控——Office 启动轮是当前唯一无裁决依赖的
  可推进开发面。
- 规格输入精读：office-d §5 敌意输出面全节（五面职责表 + CSV 四库
  逐一裁决 + XML 三档一手核验 + 域白名单建议）+ office-b §3.5（
  x2t 敌意输入五件套限额：4GiB/zip 50MB·300MB/下载 100MB/2m/3 次/
  路径逃逸/私网守卫）+ office-a §9（isSafeUrl/custom 验证器无 eval/
  剪贴板剥离）。
- 分诊（指令 5）：唯一安全正确性级缺口 = sheetjs _xHHHH_ 字面量歧义
  （审计【推断】登记未展开）→ T1 升格处理（消歧规则 + 往返测试）。
  其余输入一手代码级已闭环或参照齐备，无阻塞。
- 任务清单：office-line/office-impl/PLAN.md（T0 已毕 + T1 消毒五面
  crate + T2 Rust 生态侦察 ADR-O1/安全策略 v1 + T3 spike 三探针 +
  T4 收口含内核轮裁决问题整理；六条全程不变量）。

Stage Summary:
- Office 开发启动轮任务清单已提交（两翼 = 安全基座直进 mdcore +
  侦察选型循 pdf-a 先例零依赖落地）。
- 范围红线：零内核代码、零 D1/D2/D3 预裁决、零跨格式抽象、
  选型库零进 mdcore。
- 随即连续执行 T1 → T2 → T3 → T4。

---
Task ID: t9-c（office-impl T1：mdc-office 消毒五面落地）
Agent: 主代理
Task: 按 office-impl PLAN v1 T1 实施 mdc-office crate（mdcore workspace
第七成员）+ 敌意输出面消毒五面 + 全门禁验收

Work Log:
- crate 落地：crates/office（mdc-office，纯 std 零三方依赖，对 mdcore
  零编译依赖）；workspace members/default-members 接线 + r24_packaging
  版本锁六→七 crate 同步 + 根清单注释增补。
- 五面实现（office-d §5 逐面）：①csv_formula_guard——=+-@ 触发集三
  模式（前置 '/前置空格/typed reject），xlsx 路线豁免注记（§5.1 裁决
  2）；②xml_control——保字符档 _xHHHH_ 转义（转义集 = 0x00-0x08/
  0x0B/0x0C/0x0E-0x1F/FFFE/FFFF，\t\n\r 与 0x7F 放行），**字面量消歧
  含逃逸边界闭合扩展**（闭合位 ∈ {_, 转义集字符}，多字节成员
  FFFE/FFFF 经字符级判定——设计推导中发现朴素"仅原位 _ 闭合"会漏
  跨逃逸边界假形态，例 _x00AB+U+000B 编码后 _x00AB_ 可解码）；③
  xml_entities——文本最小集/属性五实体 + 控制字符一体化兜底形态；
  ④html_escape——href 注入面五实体兜底；⑤field_whitelist——默认
  七白（DATE/PAGE/NUMPAGES/TOC/PAGEREF/MERGEFIELD/SEQ）六黑
  （INCLUDETEXT/INCLUDEPICTURE/DDE/DDEAUTO/LINK/RD），全 token 前缀
  扫描防嵌套注入（含引号粘连形态）。
- 测试：18 集成 + 5 文档 = 23 项——码位边界全扫（0..0x2000 +
  星面字符）、三档对照断言（损坏档 = 产物零转义集残留即合法 XML；
  剥除档 = 往返零丢失）、字面量消歧回归组 11 例（设计推导链全部
  手工用例）、4000 例 LCG 往返性质测试（敌意字母表，确定性零三方
  依赖）、审计锚定载荷族（DDE 命令/超链接/算式改写 + href 注入 +
  域嵌套注入）、零依赖自锁（清单级 [dependencies] 缺席 + mdcore
  依赖键缺席双保险）。
- 过程修正三处：&str 迭代（.chars()）；零依赖自锁的 mdcore 文本
  匹配误伤 description（改行首依赖键判定）；web 测试运行产物
  results/*.json 误入首提交（历史从未跟踪——amend 移除 + web/
  .gitignore 补 tests/results/，t8"产物提取数字后清理"纪律固化）。
- 门禁验收：G1 290/290（267+18+5，全对账）+ dual 109+6i（PDF 零
  波及）+ clippy 四面 2 wasm 基线零新（office 面 0）+ fmt 0 + scan 0
  + wasm32 过 + web 18/15/28；Cargo.lock 增量 = 包注册块 4 行（零
  三方依赖的机器验证，NOTICES 零增量）。
- Git：门禁绿后提交；推送前 ls-remote 核实 d3123ba 无位移；amend
  发生在推送前（产物修正，无历史改写风险）；PAT extraheader 单次
  传递零残留。

Stage Summary:
- T1 完成：Office 域第一块产品代码落地 mdcore（main d3123ba→2a4cb6c），
  三域纪律全维持（零编译依赖 + 零三方依赖 + PDF 零波及）。
- _xHHHH_ 字面量歧义（审计【推断】登记项）升级为已解决：消歧规则
  + 性质测试锁死，超参照库行为差异在模块文档显式标注。
- 下一步：t9-d（T2 Rust OOXML 生态侦察 + ADR-O1 + 安全策略 v1）。

---
Task ID: t9-d（office-impl T2：Rust OOXML 生态侦察 + ADR-O1 + 安全策略 v1）
Agent: 主代理
Task: 按 office-impl PLAN v1 T2 执行 Rust 生态一手核验选型 pass +
ADR-O1 + 敌意输入安全策略 v1 草案

Work Log:
- 一手核验：五仓浅克隆（office-line/_upstream/ 本地不入 git，.
  gitignore 增条目）——calamine 0.36.1 / rust_xlsxwriter 0.99.1 /
  umya-spreadsheet 3.1.1 / docx-rs docx-core 0.4.22 / zip(zip2 线)
  9.0.2；LICENSE 文件逐件直读（4×MIT + 1×MIT/Apache 双许可，净室
  零义务）；HEAD 提交日期全录（全部 2026-09~10 活跃）。
- 供应链陷阱一手发现：zip-rs/zip 仓 master 停滞 0.6.x（2024-05），
  活跃线在 zip-rs/zip2（crates.io 名 zip 9.x）——克隆陷阱登记
  ADR-O1 O1-5 禁令。
- docx 读面发现：docx-rs 存在 reader（read_docx/read_docx_with_
  options 进 writer 同一 Docx 模型，reader 78 子模块文件）——
  05 §8.1"蓝海"读侧假设校准：模型级读写已存在，蓝海在渲染保真级
  读侧（office-d §6.3 目标线），ADR-O1 开放项 O1-6 留裁决。
- zip 限额面一手证据：types.rs uncompressed_size pub 暴露（预扫
  求和可行）+ crc32.rs with_size_limit 按声明值截断（反谎言头
  炸弹两层闭环）——安全策略 §2.1 的工程依据。
- 过程自纠：rg -r 误用（replace 语义）导致两次输出被替换 chasing
  幽灵符号（with_ln 实为 with_limit）——干净复查后定谳，锚点已按
  正确 API 名写入报告。
- 三件交付：office-e 选型 pass（对象锚定/能力面/防御面/选型矩阵/
  锚点统计，零【推断】标注）+ ADR-O1（六面决策 + 风险缓解 + 落地
  前置 + 两开放项留裁决）+ SECURITY_POLICY v1 草案（威胁模型六项/
  四层防线/限额表六维建议值——三项一手参照采用 + 三项新设标注
  待 T3 校准）。

Stage Summary:
- T2 完成：Rust OOXML 选型证据链闭环，ADR-O1 六面建议稿在档
  （推荐 calamine/rust_xlsxwriter/docx-rs/zip2 线，umya 按需，
  渲染级读侧留裁决）。
- 安全策略 v1 草案就绪：输入防线四层 + 限额表待 T3 实测回填。
- 下一步：t9-e（T3 spike 三探针：xlsx 读 + xlsx 写 + 敌意输入
  限额行为）。

---
Task ID: t9-e（office-impl T3：spike 三探针）
Agent: 主代理
Task: 按 office-impl PLAN v1 T3 在 office-line/spike（分析仓 cargo
工作区，零进 mdcore）执行三探针：xlsx 读 + xlsx 写流式 + 敌意输入

Work Log:
- spike 基建：office-line/spike（Cargo.toml 四依赖 = office-e 推荐件
  calamine 0.36 / rust_xlsxwriter 0.99 constant_memory / zip 9 +
  flate2 原始流构造件；Cargo.lock 提交钉版本；.gitignore target/
  fixtures）。
- 探针 1 读面：rust_xlsxwriter 生成 fixtures（跨件互证设计）→
  calamine 全量提取 + 抽点断言（字符串/数字/布尔三型）+ 全量对账
  （156,000 格零丢失）+ 大表量级（500,000 格）。
- 探针 2 写面：constant_memory 100 万格写（971.50ms，产物 3.33MB）
  + calamine 重开往返（末格浮点断言 + 行数对账）——ADR-O1 O1-1+O1-2
  推荐件组合互证成立。
- 探针 3 敌意输入三件：3a 声明预扫（by_index 逐条 size() 求和 +
  enclosed_name 逃逸识别，0.01ms 零数据读取）；3b 谎言头（手构 zip
  单条目 deflate 声明 16B 实际膨胀 1MB——本地头/中央目录/EOCD 手工
  字节装配 + 自含 CRC-32 查表）→ zip 9.x 内建拒绝（"File is larger
  than its declared uncompressed size"）；3c 巨量 dimension（手构
  最小 xlsx 四部件，声明 A1:ZZZ9999999 ≈1.7e12 格）→ calamine 无害
  （Range 按实际单元格定尺寸 2×2，0.05ms）。
- 过程修正六处编译错（write_number 实参序 / add_worksheet_with_
  constant_memory 0.99 无 Result / get usize 元组 / open_workbook
  turbofish 序 / matches! 借用）——API 校准记录在案。
- 回填：SECURITY_POLICY §2.2.1（T3 降级为登记观察）+ §3 限额表
  （谎言头实测拒绝 / dimension 实测无害 / 预扫机制实证 / 路径逃逸
  实测识别）+ 头注状态升级；office-e 增 §8 探针实测表。

Stage Summary:
- T3 完成：三探针全过，数字在档；两条敌意输入关键实证（zip 内建
  反谎言头 + calamine 不按声明预分配）直接校准安全策略。
- 推荐件组合（calamine + rust_xlsxwriter + zip 9）读写往返互证
  成立——ADR-O1 证据链闭环。
- 下一步：t9-f（T4 收口报告 + 内核轮裁决问题整理 + 双仓收尾）。

---
Task ID: t9-f（office-impl T4：收口报告 + 裁决问题整理 + 全线同步）
Agent: 主代理
Task: 按 office-impl PLAN v1 T4 收口——REPORT 三要素 + 内核轮裁决
问题整理（证据先行）+ HANDOFF/STATUS 同步 + 双仓收尾

Work Log:
- REPORT.md（office-line/office-impl/）：T0-T4 完成项对账表 + 门禁
  改前改后全表（290/79/81+6i/109+6i/perf6/scan0/wasm32/clippy四面/
  fmt + 18/15/28 + Cargo.lock 4 行零依赖证明）+ 零阻塞声明 + §4 内核
  轮裁决问题四项（R-O1 格式优先序 / R-O2 docx 渲染级读侧 / R-O3 IO
  依赖落地批准 / R-O4 编辑往返，各带证据基础与建议非裁决）+ 遗留
  登记（R-1..R-6 维持 + docx-rs 观察项 + 条目数上限待实测）+ 执行链
  与 Git 纪律复现段。
- HANDOFF.md 同步：L3 行增补 office-impl 开发轮全记录与 R-O1..R-O4
  裁决入口；L-MD 行 mdcore 版本链更新（03815b6→15efc28→d3123ba→
  2a4cb6c，含各阶段一句定性）。
- 会话总账（t9-a..f 六轮，超"至少 5 轮"要求）：两次平台回滚恢复 +
  t8 漏检修复 + office 开发启动轮全链（任务清单→消毒五面产品代码→
  选型侦察→探针实证→收口）；mdcore 两提交 / 分析仓六提交全部推送
  零残留。

Stage Summary:
- office-impl 开发启动轮完整收口：安全基座（产品代码）+ 侦察选型
  （ADR/安全策略）+ 探针实证三线全部交付在档。
- 下一阶段入口四项裁决（R-O1..R-O4）已整理完毕，等待用户；MD 线
  R-1..R-6 维持留裁决。无阻塞，无悬空工作。

---
Task ID: t10（xlsx-r1 阶段 A：XLSX 内核 Round 1 分析仓全量）
Agent: 主代理
Task: 按 xlsx-r1 PLAN v1 执行 t10-a..g 阶段 A 全部任务（用户六项
子要求，凭据无关项先行；阶段 B mdcore 落地待解锁）

Work Log:
- 环境第 4 次重建：rust 1.99.0（rustup minimal + clippy/rustfmt）；
  filemode 噪声 45 件清理（core.fileMode false，零内容变化）。
- t10-b 补完：corpus.rs 17 件语料生成 + manifest 校验全过；mal6c
  手构 zip64 **双缺陷修正**（①EOCD size-of-record 是 u64 首版误写
  u32 致整条记录错位；②CDFH extralen 回填位算错 z.len()-30 打进
  name 末两字节且 extralen 留 0 → zip 按裸 0xFFFFFFFF seek 出界）
  ——两处均由 t10-d harness 失败回溯定位，字节级 Python 复核定谳。
- t10-c 补完：fidelity.rs 七处 API 校准修复后首跑（write_formula
  String→&str / i64 as f64 / merge_range u16 / zip 9 name() Result
  形态 / mut wb）→ 五档丢失矩阵：值 37/37、公式文本 6/6、合并 3/3、
  表序 5/5、修改标记 5/5 保；**样式结构性丢**（calamine 无可观测面
  + styles.xml 重写）、**关系引用结构性丢**（dim5 实测 .rels 条目
  整体消失）→ FIDELITY.md 定谳"组合语义=重建而非修改"。
- t10-d 新建：read_proto.rs（限额常量 + 预扫门四检查 typed reject +
  calamine 四 API 面 + 三分统一错误 + 17 件全语料 harness）→
  **17/17 全过零 panic**；mal7c/mal7e 实证钉定为解析器容忍度发现
  （0x00 入值/垃圾部件静默降级空表）带锚点闭包自证。
- t10-e 新建：SANITIZE_TESTS.md——61 用例攻击载荷表（csv 10/xml
  字符 10/xml 实体 7/html 6/域白名单 10/url 15 + 三模式对照），
  每用例四元组精确期望；**URL 策略缺口发现**（指令 4 五上下文 vs
  t9-c 五面：scheme 级白名单无承载面，H-03/H-04 用例实证字符转义
  对 scheme 攻击无效）→ 补面设计 + 管线顺序契约。
- t10-f 新建：DEPS.md——51 外部 crate 闭包全量许可证核验
  （Cargo.lock BFS + 本地 src 直读 + crates.io API 补 11 件），
  零 copyleft；zip 8.6/9.0 并存登记（传递面均为 8.6，直依赖 9）
  + 阶段 B 对齐 8.6 建议（API 面等价已核）+ 谎言头重测项；
  encoding_rs BSD-3-Clause / unicode-ident Unicode-3.0 两 AND 条款
  入 NOTICES 注意。
- t10-g 新建：CAPABILITIES.md——C1/C2 按实证宣称（边界清单），
  C3 唯一口径=不宣称（替代路径丢失矩阵引用），容忍度发现随宣称
  发布，DOCX 独立模块边界四条声明，宣称变更纪律。
- 产品代码预置（超 PLAN 最小集）：spike lib 形态——io.rs（预扫门
  + calamine 包装，8 测试含敌意件内联自构造）+ url_policy.rs（消毒
  第六面 16 测试 = t10-e §6 全表落地）→ **cargo test --lib 24/24**；
  新发现：缺根闭合 </worksheet> 截断被容忍（与 mal7a 标签内截断
  报错构成容忍度分界，双测钉定）。
- 阶段 B 架构决策登记：T1 mdc-office 零三方依赖自锁测试与 IO 依赖
  冲突 → 双 crate 方案 A（新 mdc-office-io 第八成员 + url_policy
  落现有 sanitize 家族）；runbook 七步写入 REPORT.md §3。
- 收口：REPORT.md（六项对账 + 数字汇总 + 遗留 5 项）+ HANDOFF L3
  行同步 + 本 worklog；提交 1fd729c（阶段 A 五件 + 三 bin）+ 收口
  二次提交；**推送与阶段 B 均凭据阻塞**（GITHUB_TOKEN 空/凭据文件
  零存在/两仓匿名不可达——实测登记，解锁即机械执行）。

Stage Summary:
- 阶段 A 全量完成：语料/保真/读骨架/测试设计/依赖锁定/宣称边界
  六线交付 + 产品代码 24/24 验证态；三轮实证发现（mal6c 构造缺陷
  ×2、解析器容忍度 ×3、URL 策略缺口 ×1）全部钉定在档。
- 阻塞面唯一：凭据。解锁后按 xlsx-r1/REPORT.md §3 runbook 七步
  （双 crate 落地 → 测试 → NOTICES → 门禁 → 提交推送）。

---
Task ID: t10-sup（xlsx-r1 阶段 A 补充轮：zip 8.6 对齐 + 谎言头安全发现闭环）
Agent: 主代理
Task: 凭据解锁前的最后零凭据推进面——DEPS §1 对齐建议提前实测验证

Work Log:
- zip 9→8.6 对齐切换：Cargo.toml 改版本 + API 差异三处校准
  （name() 9.x Result<Cow<str>> → 8.6 &str，io.rs/read_proto/
  fidelity.rs）→ 全量回归 lib 24/24 + harness 17/17 + corpus +
  fidelity 全绿，闭包收敛（zip 9 支线移出，51→50 外部 crate）。
- lie86 探针（手构谎言头：deflate 声明 16B 实际 1MB + 正确 CRC）
  → **定谳安全发现：zip 8.6 读界无声明值强制**——read_to_end 全量
  读回 1MB 零错误（仅 EOF CRC 校验，正确 CRC 即绕过）；T3 3b 记录
  的"库内建拒绝"为 zip 9.x 独有；calamine 0.36.1 内部依赖 8.6 →
  **SECURITY_POLICY §2.1 第 2 项的读界防线在实际读路径不存在**。
- 过程自纠两处：lie86 探针 CDFH 偏移字段笔误（写 cd_off 应写 0）+
  sed 无 g 误伤 EOCD 同文案行（字节级 Python 复核定位）；rg -rln
  再犯 t9-d 登记的 -r 替换语义坑（-rln = 替换为 "ln"）——两坑均
  worklog 在案，本次即时识别。
- 闭环实现：io.rs 新增 verify_actual_sizes（门 2）——逐条目 64KB
  有界读取计数，实际 > 声明即 typed reject（计数在 EOF CRC 校验前
  触发，CRC 正确性无关）；read_book 接线双门（门 1 声明四检查 +
  门 2 尺寸诚实性）；read_proto harness 委托 lib（原型→产品收束，
  本地 ReadError/prescan/read_book 副本删除）。
- 双层验证：lib 单测 lie_header_rejected_at_gate2（内联手构）+ 
  harness mal6a 升级 REJECT（正确 CRC 形态）→ lib 25/25 + 
  17/17 零 panic；mal6d 条目数门同步 SECURITY_POLICY §3 实测校准。
- 文档同步：DEPS.md §1（对齐定谳 + 安全发现 + 闭包 50）§4/§5
  （双 crate 落点）；SECURITY_POLICY §2.1 第 2 项改写（库内建→
  策略层）+ §3 两行校准 + §5 复核纪律（zip 8.6→9 迁移敏感点 +
  校准历史）；REPORT.md §5 数字表（安全发现置顶行）+ §6 遗留
  3 项闭环 + 新增 6 项（门 2 双读成本优化登记）。

Stage Summary:
- 本轮最重要产出：谎言头读界防线缺口发现 + 策略层闭环（门 2），
  SECURITY_POLICY 三处校准——安全策略从"引用库行为"校准为
  "实测版本行为 + 策略层自持防线"。
- 零凭据推进面至此耗尽：阶段 B（mdcore 落地）与双仓推送仍凭据
  阻塞；解锁后按 REPORT.md §3 runbook 执行（步骤 2 已简化：直移
  含门 2 的 io.rs）。

---
Task ID: t10-sup2（C2 样式写入 XML 级验证闭项）
Agent: 主代理
Task: 零凭据面收尾——REPORT §6 遗留 #2 闭项（C2 样式写入逐属性断言）

Work Log:
- style_probe.rs 探针：直查 dim2_styles.xlsx 生成档的 xl/styles.xml
  + sheet1.xml（zip 8.6 读条目 + 字符串级断言，零新依赖）——九项
  逐属性：font bold、字色 FFFF0000、填充 FFFFFF00、thin 边框、
  自定义 numFmt 区（numFmtId 164+）、¥ 货币格式码、百分比格式码
  （0.0%——首版断言误写 0.00%，与 corpus 构造对齐修正）、cellXfs
  xf 条目 ≥2、sheet 单元格 s= 引用 → **9/9 全过**。
- CAPABILITIES.md C2 边界升级：基础级（仅值面）→ XML 级验证
  （探针 9/9 + 完整样式模型[条件格式/主题色/富文本]仍不宣称）；
  REPORT.md §5 增行 + §6 遗留 #2 划线闭项。

Stage Summary:
- C2 宣称边界从"样式写入未断言"升级为"XML 级九属性断言通过"，
  读侧无样式面（calamine 限制）通过直查产物部件绕开——验证方法
  登记为后续样式面扩展的参照。
- 零凭据推进面彻底收尾：遗留项仅剩凭据门控（阶段 B + 推送）与
  四项登记裁决（R-O2/mal7e 严格模式/双 crate 确认/门 2 成本）。
---
Task ID: t10-B（xlsx-r1 阶段 B：mdcore 落地——runbook §3 七步全执行）
Agent: 主代理
Task: 凭据解锁后按 xlsx-r1/REPORT.md §3 runbook 执行阶段 B——双 crate
落地 + 测试全量 + NOTICES 增量 + 门禁全绿 + 双仓推送 + 分析仓收口

Work Log:
- 凭据解锁（2026-10-13 同日）：用户注入凭据；环境变量单次传递纪律
  维持（克隆走内联 credential helper / 推送走 extraheader，token 磁盘
  零残留——env 跨调用不持久的发现与两种单次传递形态均在案）。
  前置核实：mdcore main = 2a4cb6c 无位移；分析仓四提交先行推送
  （dd7fb1a..91db58a）。
- 环境第 5 次重建：rust 1.99.0 工具链在位（rustup 目录幸存）；
  wasm-bindgen-cli 0.2.129 补装（版本对齐 lock）；md-line pdf-a spike
  取件 pdfium chromium/8086（SHA256 pin 验证过）+ 现场生成 fixtures
  （small/large/form_probe，lopdf 构造）。
- 步 2 mdc-office-io 落地（workspace 第八成员）：spike io.rs 直移
  （门 1 prescan 四检查 + 门 2 verify_actual_sizes 谎言头闭环 +
  calamine 四 API 面 + 统一错误三分 + 内联 9 单测）；依赖按 DEPS §1
  （calamine 0.36 + zip 8.6 deflate；dev：rust_xlsxwriter 语料构造 +
  flate2 谎言头流构造）；workspace members/default-members 接线 +
  r24_packaging 版本锁七→八；crate 文档含 Capabilities 节
  （CAPABILITIES.md 口径引用）+ 容忍度发现三件。
- 步 3 mdc-office 第六面：url_policy.rs 直移 sanitize 家族（16 单测）
  + mod/lib 文档接线（面表五→六 + 消费契约五条：xlsx 豁免/_xHHHH_
  幂等契约/保守误杀/实体非幂等/URL 管线顺序）——纯 std 不破 T1
  自锁（清单级 [dependencies] 缺席断言恒绿）。
- 步 4 测试三层：spike 25 随直移 + sanitize_security.rs 44 用例
  （t10-e §1-§5 逐条精确断言）+ corpus_harness.rs 17 件三分断言
  （corpus.rs 构造函数内联·临时目录）。**四处设计-实现出入按「以
  mdcore 实际为准」钉定**：X-03 期望串拼接笔误（实测
  _x005F_x005F_x005F_x0000_，decode(encode(x))==x 恒成立）/ F-06
  引号包裹 token 实测放行（Word 域语法字面量语义，花括号嵌套裸
  token 拦截对测钉定）/ F-10 前缀超集实测拦截（startsWith 设计代价，
  F-09 同类保守误杀）/ F-08 双门分立（Blacklist 策略隔离边界）——
  SANITIZE_TESTS.md 四处回写。
- 步 5 NOTICES v1.3（compliance-baseline 分支）：四面复跑（default
  50/pdfium 64/lopdf 88/dual 101，host-target + default-members +
  mdcore-wasm 口径经 3e450aa 校准复现 v1.2 基准 29 后定谳）；正文
  90→101（新入 9 + rustversion/zlib-rs 账提升——atoi_simd proc-macro
  硬边/zip→flate2 zlib-rs 后端边）+ 分类账 28→33（rust_xlsxwriter
  dev 面 + tempfile 族 5 + windows-sys win 面）；三链核验 16/16
  （sparse 索引 etag 包装 raw_decode 解析 + 短名三层布局探测）；
  **两处许可修正**：zip 8.6.0 = MIT 单许可（首版记双许可系口径
  混淆）、rust_xlsxwriter 0.99.1 = MIT OR Apache-2.0（首版记 MIT）
  ——DEPS.md 回写；ASSET_CHECK v1.3 对账（office 线零二进制资产）；
  自研单列补齐 5→8。
- 步 6 门禁全绿：**G1 361/361**（290 基线只增不改 +71：office 83
  [url_policy 16+gates 18+security 44+文档 5] + office-io 11 [io 9+
  harness 2]）+ lopdf 79/79 + pdfium 81/81+6i + dual 109/109+6i（PDF
  四面零波及逐位复验）+ perf 6/6 + clippy 四面 --all-targets 零新
  （仅 wasm 2 条在档基线）+ fmt 0（spike 直移源清扫）+ wasm32 过 +
  scan 产品面/--all 双 0 + web 18/15/28（contract/smoke/ime_cdp）。
- 步 7 提交推送：mdcore **main 2a4cb6c→880864e**（README 架构图
  Office 域行 + 门禁计数 259→361 同步 + COMPLIANCE 计数同步）；
  **compliance-baseline 1adced2→ddc51c1**（NOTICES v1.3 + ASSET_CHECK）；
  SECURITY_POLICY **v1.0 生效**（§5 纪律兑现：草案→生效随实装）；
  CAPABILITIES crate 接线注记；REPORT §7 执行记录 + §1/§4 闭项。
- 过程修正五件：crc.to_le_bytes() 缺引用（直移编译错）；MultiEdit
  连续批次在长文件上的部分应用行为（逐条重发收敛）；sanitize_security
  首版两断言期望侧笔误（X-03 往返断言写错原像、F-08 未区分双门）——
  均首轮测试红后即改；cargo tree 计数管道（(\*) 后缀与 target 过滤）
  校准。

Stage Summary:
- xlsx-r1 阶段 B 全步完成：Office 域首批产品代码 + 三方依赖进 mdcore
  （mdc-office-io 读路径 + mdc-office 消毒六面 + 60 个针对性/语料
  测试函数），双分支推送零残留；NOTICES v1.3 闭环（第三方总数
  105→116，零 UNKNOWN，GPL/AGPL 零命中）。
- 安全面：谎言头双门 + 靶向安全测试全绿；四处设计-实现出入与两处
  许可修正全部钉定回写（设计文档与实现对齐的完整审计链在档）。
- 能力宣称边界不变：C1 可宣称 / C2 写面验证态（产品面下轮）/ C3
  不宣称。遗留裁决项维持：mal7e 严格模式 / 门 2 双读成本优化 /
  R-O2 docx 渲染级读侧。

---
Task ID: t10-verify（xlsx-r1 收口后：第 5 次平台重置恢复 + 12 门禁复认证）
Agent: 主代理（Super Z）
Task: 用户注入凭据后继续执行——盘点第 5 次平台重置损失面，恢复工作区，
对 xlsx-r1 双仓推送态（mdcore main 880864e / compliance-baseline
ddc51c1）做全量门禁复认证，确认远端真源可复现性

Work Log:
- 重置盘点：/tmp/my-project 工作区被第 5 次平台重置清空（前会话
  cleanroom/mdcore 克隆丢失，worklog 会话级快照幸存）；/home/z/
  my-project 从 Oct 12 15:33 快照恢复（HEAD=dd7fb1a t9-f 态 +
  未跟踪旧稿：xlsx-r1/PLAN.md + spike/src/bin 前会话半成品）。
- 关键转机（远端为真源纪律）：凭据注入后 ls-remote 实测——
  cleanroom-analysis main = e23efa9（dd7fb1a 之后 5 提交）、mdcore
  main = 880864e / compliance-baseline = ddc51c1。前会话中期 worklog
  （"两提交本地在案待凭据"）为推送前快照：最终凭据同日解锁，阶段 B
  七步全执行并双仓推送（1fd729c/4fa49ec/0fe8ce4/91db58a/e23efa9
  五提交 = 阶段 A 收口 + 安全发现轮 + C2 样式闭项 + 阶段 B 收口）。
  本地零未推送状态确认（HANDOFF e23efa9 注记与实测一致）。
- 零损失恢复：未跟踪旧稿移盘备份对比（xlsx-r1/PLAN.md 逐字节一致；
  corpus.rs 28556B/ fidelity.rs 15710B = 修复前旧版，远端 29438B/
  15871B 为修复版）→ core.filemode false（45 文件 mode 噪声消除，
  t7-u1 已知平台副作用）→ reset --hard origin/main → 工作区归零。
- 环境重建（第 5 次，~/.cargo 全丢）：rustup-init.bin 复用安装
  1.99.0 minimal + rustfmt/clippy + wasm32 target；pdfium
  chromium/8086 取件（fetch-pdfium.sh SHA256 验证过）；fixtures
  现场生成三件（small 1026B / large 124524B / form_probe 731B——
  与登记字节数逐位一致）+ pdf_a_spike 三探针 + S6 formfill_probe
  全过（数字与 gates.md 登记一致：3p load 0ms / 渲染 3ms /
  lopdf→pdfium 交叉读互通 OK / form()=Some）；wasm-bindgen-cli
  0.2.129 cargo install --locked（与 Cargo.lock 精确一致）+ web
  npm ci + esbuild（dist/main.js 531.2kb 与在档 531KB 一致）。
- mdcore 克隆：内联 credential helper（token 零落盘，.git/ grep
  ghp_ = 0 复核）；main checkout 880864e + compliance-baseline
  ddc51c1 双 SHA 核验。
- **12 门禁复认证全绿零漂移**（逐项对照 t10-B 登记）：
  G1 默认 361/361（290+71 只增不改）| G2 lopdf 79/79 | G3 pdfium
  81+6i | G4 dual 109+6i（raw 行含 pdfc_perf "6 ignored" 逐位确认）|
  G5 perf --ignored 6/6 | G6 scan 0（product scope, 20 terms）|
  G7 wasm32 check 过 | G8 clippy 四面（default/lopdf/pdfium/dual
  --all-targets）零新增——唯一 2 条 = mdcore-wasm lib.rs:188/:197
  wasm_bindgen(static_method_of) 宏属性告警 = 在档基线精确复现 |
  G9 fmt 0 diff | web 三门禁 contract 18/18 + smoke 15/15 + IME
  CDP 28/28。
- compliance-baseline 复核：NOTICES v1.3（草案头 + 增量明细 + 116
  总数 = cargo 101 + npm 12 + pdfium 3 + 分类账 28→33 + ASSET_CHECK
  结构）在档核验；16 新 crate 三链核验脚本重跑（xlsxr1_notices_
  verify.py 自 /tmp 幸存）——首跑 15/16（windows-sys ③ 链 .crate
  工件缓存缺失 = win 目标件 Linux 不下载的环境产物，①==② 关键链
  过；原会话同口径 15/16 + API 直验）；补齐 static.crates.io 工件
  后 16/16 全绿（超原会话口径）。
- 运行产物清理：web/tests/results/ 删除（t7-u1 纪律）；mdcore
  git status 归零复核（target/pkg/dist/node_modules 全在 .gitignore
  覆盖面）。

Stage Summary:
- 第 5 次平台重置零损失恢复：xlsx-r1 全部成果（阶段 A + B）确认
  已在双仓远端在案，本地无需重建任何工作。
- 复认证定谳：12 门禁 + NOTICES v1.3 三链全绿零漂移——mdcore
  main 880864e 在全新环境完整可复现，推送态稳定。
- 决策点五项维持待裁决（HANDOFF §6）：xlsx-r2 C2 写面产品化 /
  mal7e 严格模式 / 门 2 双读成本优化 / R-O2 docx 渲染级读侧 /
  L3 遗留 D1-D3（yrs 评估 pass 最优先）。Office 域无阻塞可推进项，
  下一动作 = 用户裁决。

---
Task ID: t10-v2（xlsx-r1 v2 勘误增量 + 分叉合并恢复，本会话）
Agent: 主代理
Task: 从第五次重置的 stale 快照继续；v1→v2 三处勘误闭环；双仓推送
分叉发现与合并（远端为真源）

Work Log:
- 起点（误框）：本地 = dd7fb1a + 未跟踪早期 t10-b/c 代码（corpus.rs
  3 链接 v1 + fidelity.rs 7 编译错 + PLAN.md）——按"上一会话未竟"
  推进：fidelity 修复运行 → 三处 v2 勘误链发现并闭环：
  ①超链接可观测性（calamine hyperlinks_by_sheet_name 存在——
  v1 manifest「无公开 API」作废；dim5 语料扩五链接形态；fidelity
  补提取/回放[值面驱动形态判定]/对比——5/5 零字段差异，rels 字节
  面单转义与原档一致）②calamine 读侧实体怪癖（rels Target 未做
  XML 实体反转义，utils.rs:117 只处理 _x00HH_；& URL 以 &amp;
  字面量读出 → 重放双转义实证 → xml_unescape5 补偿）③mal6c
  zip64 装配三缺陷（CD extralen 回填位 / EOCD size u64+56B 定长 /
  Locator 补齐；python+zip9 双解析验证）。
- t10-d/f/g 完成（stale 框下）：read_proto 独立原型 + DEPS +
  CAPABILITIES + FIDELITY/REPORT/SANITIZE_TESTS 各文档——mdcore
  阶段 B v2（18d5bb2：office-io 第五面超链接 + unescape_xml_
  entities5 + harness dim5 五形态；G1 362 全门禁绿）先行推送成功。
- 分叉发现：分析仓推送被拒——远端超前 6 提交（0fe8ce4/91db58a/
  e23efa9/998a7f4）：上一会话在 stale 快照之后还完成了门 2 发现轮
  （lie86）+ C2 样式 XML 级（style_probe 9/9）+ 阶段 B 收口六文档
  + 第 5 次重置 12 门禁复认证——本地未跟踪件为更早期快照（v1
  认知）；SANITIZE_TESTS/io.rs/url_policy 首版**并未丢失**（远端
  在档），本会话早段"复原注记"系误框，已在合并层修正。
- 合并（远端为真源）：reset --hard 998a7f4 → v2 层叠加：corpus/
  fidelity 取 v2 版（fidelity 校准 zip 8.6 name() &str）；spike
  io.rs 补第五面 + 补偿 + 26 测试；read_proto 委托版 dim5 锚升级
  （17/17 重跑零 panic）；CAPABILITIES 合并（C1 五面 + C3 风险面
  收窄为样式场景）；DEPS zip 8.6.0 MIT 单许可修正；FIDELITY 置
  v2 + 版本链注；REPORT 增 §8 v2 节 + §6 第 7 行；SANITIZE_TESTS
  维持（六族 + X-03 修正已在档）；HANDOFF L3/L-MD 合并（18d5bb2）。
- 全链重验：spike lib 26/26 + corpus v2 重生成 + fidelity v2 重跑
  （dim5 超链接 5/5）+ read_proto 17/17 + style_probe 9/9 +
  lie86 探针在役；mdcore 18d5bb2 门禁全绿（362/79/81+6i/109+6i/
  perf6/scan0/wasm/clippy 2 基线/fmt0/web 18/15/28；NOTICES
  v1.3 恒零新依赖）。

Stage Summary:
- xlsx-r1 v2 勘误层双仓闭环：超链接保真（C3 宣称修正）+ 实体
  补偿 + zip64 修复；mdcore main = 18d5bb2。
- 决策点维持（含上会话 t10-verify 登记）：xlsx-r2 C2 写面产品化 /
  mal7e 严格模式 / 门 2 双读成本 / R-O2 docx 渲染级读侧 / D1-D3
  （yrs 评估 pass 最优先）。
- 教训登记：跨重置会话续作必须先 fetch 远端再定框（本地未跟踪
  件可能滞后于远端多个提交轮）——本会话早段在误框下重复了部分
  已推送工作（read_proto 独立版被委托版取代、SANITIZE_TESTS 复原
  误注）；合并层已全部修正，零内容损失。

---

Task ID: t11
Agent: 主代理（Super Z 会话续作，第 6 次平台重置恢复）
Task: xlsx-r2 三阶段指令——Phase 1 XLSX 写面产品化（本轮完成）；Phase 2 mal7e 严格模式；Phase 3 DOCX 切片（连续执行）

Work Log:
- 环境恢复（第 6 次平台重置，比以往更彻底）：GITHUB_TOKEN 重置入环境变量（不落盘不回显）；cleanroom-analysis 本地 dd7fb1a → 快进 80499b4（未跟踪残留三件经逐位比对确认陈旧子集后清除，备份 .recovery-quarantine）；mdcore 目录整个被删 → 匿名克隆恢复（默认分支 compliance-baseline，切 main = 18d5bb2）；worklog.md 被删 → 本仓 worklog 为真源续写
- 工具链恢复：rustup + Rust 1.99.0（钉住 channel + wasm32 + rustfmt/clippy）；cargo fetch；wasm-bindgen-cli 0.2.129 按锁安装；web npm install + dist 构建；pdfium 载荷按 T1-F12 bootstrap 重建（fetch-pdfium.sh SHA256 验证 + fixtures 现场生成 + spike 三探针复测全过）
- 基线核实（改动前）：G1 362/362 全绿 = 18d5bb2 推送态逐位一致
- Phase 1 实现：Cargo.toml rust_xlsxwriter dev→产品（constant_memory）；src/write.rs 新模块（引擎类型零泄漏 + 输入验证前移 typed + 流式行序纪律 typed 拦截[引擎静默丢弃一手核验 worksheet.rs:15840] + 公式前缀规范化 + 缓存值三型）；lib.rs C2/C3 宣称同步
- Phase 1 测试：tests/write_roundtrip.rs 6 全环测试（经产品读路径含容器双门）；tests/range_anchor_pinned.rs 1 钉定测试；开发期捕获并修复 calamine 范围锚定双语义的相对坐标自洽假通过（修复=绝对坐标配对，fidelity abs 键同构）
- Phase 1 门禁全绿：G1 379/379 + PDF 四面零波及（79/81+6i/109+6i）+ clippy 四面 2 wasm 基线零新 + fmt 0 + scan 0 + wasm32 + web 全过
- Phase 1 合规：NOTICES v1.4（compliance-baseline 048f1bb）——纯面重分类零 lock diff；正文 101→106（账提升 5）+ 账 33→28 + 四面 50/64/88/101→58/71/93/106（SBOM 四面复跑 + 逐项并集实证 + 与 v1.3 对账无出入）；ASSET_CHECK 零新资产；中途捕获两处编辑失误（r-efi/find-msvc-tools 误删）并恢复，最终 28 账项逐项核验
- 分析仓文档：xlsx-r1 CAPABILITIES（C2 产品面 + mal7e 指向 Phase 2）/ DEPS（表行 + §5 增量）/ FIDELITY（§6 产品级重放注记）+ xlsx-r2/REPORT.md 新建

Stage Summary:
- mdcore main：18d5bb2 → e8c3a13（Phase 1 已推送）；compliance-baseline：ddc51c1 → 048f1bb（NOTICES v1.4 已推送）
- 五项待决裁决全部由用户指令落定：①写面产品化批准（完成）②mal7e 严格模式批准（Phase 2 进行中）③门 2 跳过否决 ④docx 改道模型级切片 ⑤D1-D3 延后
- 开发发现四件在档：calamine 范围锚定双语义（自洽假通过陷阱）/ rust_xlsxwriter 控制字符 _xHHHH_ 转义一手核验（写面 XML 级免疫）/ constant_memory 静默丢弃（包装层 typed 拦截）/ 字符串落 sharedStrings（断言须全部件扫描）
- 下一步：Phase 2（mal7e 默认严格 + 兼容模式）→ Phase 3（DOCX 切片），连续执行不阻塞

---

Task ID: t11-b
Agent: 主代理（xlsx-r2 Phase 2 续作）
Task: Phase 2 读面强健化——mal7e 类静默降级闭环（默认严格 typed 错误 + 显式兼容模式结构化警告）

Work Log:
- mdcore 实现（main e8c3a13 → 042a5e4 已推送）：门 3 部件良构性验证——.xml/.rels 部件流式结构校验（quick-xml 0.41 升直依赖，原 calamine 传递闭包原位零增量零 lock 变化），融合在门 2 既有解压趟内（CountingReader：尺寸计数+控制字节扫描[仅 XML 域]）零额外解压成本；校验面 = 标签配对/根在位唯一/EOF 未闭合[截断]/深度上限 1000[SECURITY_POLICY §2.2.3 挂账闭项]
- API 面：ReadError::InvalidPart{part,detail}（默认严格）；ReadOptions{compat} + CompatibilityWarning 随 Book.warnings（兼容模式）；门 1/门 2/安全上限两模式恒等；门 2 受信源跳过按裁决不设（verify_actual_sizes 文档回写）；verify_actual_sizes 公开独立面契约不变（独立测试钉定）
- 行为收敛（语料三分归类变化钉定）：mal7e OK→REJECT（兼容 Ok+警告+空表）；mal7a ERR→REJECT（兼容仍引擎 ParseError）；缺根闭合截断容忍→REJECT（兼容保持 xlsx-r1 分界）；mal7c 值面容忍不变+恒登记 ControlChars 警告
- 测试：lib 25（garbage 严格拒重写 + 多根/空部件/深度边界对测/非 XML 域/独立面 五新）+ harness 3（兼容模式全语料复扫新测试 + 容忍度锚点重钉）
- 门禁全绿：G1 385/385 + PDF 四面 79/81+6i/109+6i 零波及 + clippy 四面 2 wasm 基线零新 + fmt 0 + scan 0 + wasm32 过
- 文档：SECURITY_POLICY v1.1（门 3 + §2.2.3 闭项 + §3 两行）；CAPABILITIES（C1 宣称语 + §3 Phase 2 后口径）；DEPS（quick-xml 行）；xlsx-r2 REPORT §5；NOTICES 零增量（quick-xml 原在正文）

Stage Summary:
- mdcore main：e8c3a13 → 042a5e4（Phase 2 已推送）；分析仓 SECURITY_POLICY v1.0→v1.1
- mal7e 类问题闭环：畸形/无效 XML 部件（垃圾/截断/多根/空/超深）不再静默变空/部分数据——严格 typed reject + 兼容模式结构化警告双模式钉定
- 开发教训两件在档：深度计数含 root（边界对测首版 off-by-one）；控制字节扫描须限定 XML 部件域（二进制部件如图片本就是任意字节）
- 下一步：Phase 3 DOCX 小而可用读/写切片（模型级，复用 ADR-O1 docx-rs 结论）

---

Task ID: t11-c
Agent: 主代理（xlsx-r2 Phase 3 续作——三阶段收口）
Task: Phase 3 DOCX 小而可用读/写切片（模型级）+ 三阶段全部收口

Work Log:
- 引擎接入（复用 ADR-O1 O1-3 既有研究，零新生态调研）：docx-rs 0.4.22（MIT；default-features=false 关 image 巨树）+ base64 0.22.1（MIT OR Apache-2.0）——闭包仅 +2，lock 142→144；三链核验 lock==.crate 2/2 + cargo 传递 index cksum；fork 线供应链观察项纪律常驻化（docx_roundtrip.rs 升级首跑拦截）
- mdcore 实现（main 042a5e4 → 460f302 已推送）：src/docx.rs 独立模块（CAPABILITIES §2 边界裁决：自有模型 + 自有错误面 DocxError + docx-rs 类型零泄漏；共享面仅容器门基础设施[io::verify_gates crate 内复用]与消毒链）；读 = 三门复用 + 模型提取（块序/样式透传/run 级/超链接内文本[目标丢弃]/表格拼接/Tab·Break·CR 保真）；写 = DocxWriter 仅新建语义（Heading1-9 样式定义 outline 级 + 富文本段落 + 文本表格）；字节入口同三门；不支持面与数据损失逐项登记（模块文档登记表）
- 实测行为差异钉定：docx-rs 读引擎对原始控制字符自身硬拒（Parse）——较 calamine/XLSX 值面容忍更严；门 3 警告在该错误路径不可达；两模式同拒
- 测试：tests/docx_roundtrip.rs 5 测试（写读全环 6 块序逐面 + 容器门复用钉定[截断/垃圾/路径逃逸 × 严格/兼容双模式] + 字节入口同门 + save_to_vec + 控制字符行为）+ docx.rs 单测 2
- 门禁全绿：G1 392/392 + PDF 四面 79/81+6i/109+6i 零波及 + clippy 四面 2 wasm 基线零新 + fmt 0 + scan 0 + wasm32 过；四面闭包 58/71/93/106 → 62/75/95/108
- 合规：NOTICES v1.5（compliance-baseline 048f1bb → 5e144cc 已推送）：正文 106→108（docx-rs[MIT 族 +1]/base64[MIT OR Apache-2.0 族 +1]）+ 分类账 28 恒 + default +4[含 thiserror/thiserror-impl 原仅 lopdf 面] + ASSET_CHECK 零新资产
- 分析仓：xlsx-r2/REPORT §6 Phase 3 收口 + §7 三阶段总表 + 遗留登记四项；DEPS docx-rs 行；CAPABILITIES §2 切片落地 + DOCX 宣称口径

Stage Summary:
- 三阶段全部收口：Phase 1 写面产品化（e8c3a13/NOTICES v1.4）→ Phase 2 门 3 严格+兼容（042a5e4/SECURITY_POLICY v1.1）→ Phase 3 DOCX 切片（460f302/NOTICES v1.5）
- G1 362→392（+30）；PDF 四面零波及逐位一致；三仓版本链：mdcore main 18d5bb2→460f302、compliance-baseline ddc51c1→5e144cc、cleanroom-analysis 80499b4→本提交链
- xlsx-r2 五项遗留裁决全部执行完毕；新遗留四项登记（DOCX 增量面按需/XLSX 样式保真编辑另议/门 2 跳过维持不设/docx-rs fork 线观察）
- 开发期教训：git stash 与未跟踪新模块文件并行时 pop 可能失败（改用 worktree 或避免）；docx-rs Table/TableRow 构造器为 Vec 参数非 builder 链

---
Task ID: t12-a
Agent: subagent (env recovery + gate re-verification)
Task: 7th platform reset toolchain recovery + independent re-verification of xlsx-r2 three-phase gate claims at mdcore 460f302

Work Log:
- Read worklog t11/t11-b/t11-c sections (lines 2048-2107) as recovery precedent
- Repo state verified before any action: mdcore main = 460f302, compliance-baseline = 5e144cc, cleanroom-analysis = 6d0fd1e; both working trees clean; rust-toolchain.toml pins 1.99.0 + wasm32 + rustfmt/clippy
- Toolchain rebuilt from zero (~/.cargo and ~/.rustup absent, 7th reset confirmed): rustup 1.29.1 installed via rustup-init with --default-toolchain 1.99.0 → rustc/cargo 1.99.0, rustfmt 1.10.0, clippy, wasm32-unknown-unknown std (auto-added via rust-toolchain.toml on first cargo run in repo)
- cargo fetch: OK, Cargo.lock = 144 entries (matches 142→144 Phase 3 claim: docx-rs + base64); zero network failures
- G1 cargo test --workspace (plain run, ignored stay ignored): 392 passed / 0 failed / 1 ignored (mdshell_native BLUEPRINT_NOTE doc-test baseline) across 49 summary lines; claim 392/392 = PASS
- PDF feature faces (payloads bootstrapped per T1-F12): fetch-pdfium.sh OK (chromium/8086, SHA256 verified, libpdfium.so 7942208B at spike bin/lib/); pdf_a_spike --release: fixtures small.pdf 1026B / large.pdf 124524B / small_annotated.pdf 1497B + 3 probes pass (load 0ms, render 3ms, lopdf→pdfium cross-read OK); formfill_probe --release: form_probe.pdf 731B + S6 pass; payload discovery found spike bin/lib candidate path automatically (no env vars needed)
- Feature faces re-verified with per-binary accounting (awk pairing of Running↔result lines): lopdf 79/0/0 (36 skeleton + 23 cache_worker + 9 session_flush + 11 security); pdfium 81/0/6i (36+23+13 engine+9, pdfc_perf 6 ignored); dual 109/0/6i (36+23+13+9+17 writer+11 security) — all three match claims exactly
- Stretch perf face: pdfc_perf --ignored = 6/6 ok (0.11s)
- cargo fmt --all -- --check: 0 diff, exit 0
- compliance/scan.sh: product scope 0 hits exit 0; --all scope 0 hits exit 0 (both conventions run)
- clippy 4 faces: default --workspace --all-targets = exactly 2 warnings (mdcore-wasm lib.rs:188/:197 static_method_of = known baseline, zero new); lopdf / pdfium / dual -p mdc-pdf faces = 0 warnings each, all exit 0
- cargo check --workspace --target wasm32-unknown-unknown: pass, exit 0
- Purity check at end: mdcore git status clean (zero modifications — pure verification, no commit/push); cleanroom-analysis tree clean (0 lines)
- Progress notes appended incrementally to /home/z/my-project/scratch-t12a-progress.md (9 milestones)

Stage Summary:
- Verified claims table (claim vs actual):
  | Gate | Claim | Actual | Verdict |
  | G1 test --workspace | 392/392 | 392 passed / 0 failed / 1i doc-test baseline | PASS |
  | PDF lopdf face | 79/79 | 79 passed / 0 failed / 0 ignored | PASS |
  | PDF pdfium face | 81/81 + 6i | 81 passed / 0 failed / 6 ignored | PASS |
  | PDF dual face | 109/109 + 6i | 109 passed / 0 failed / 6 ignored | PASS |
  | clippy 4 faces | 2 wasm baseline, zero new | default: exactly 2 (lib.rs:188/:197); 3 feature faces: 0 | PASS |
  | fmt | 0 diff | 0 diff, exit 0 | PASS |
  | scan product | 0 hits | 0 hits (20 terms) | PASS |
  | scan --all | 0 hits | 0 hits | PASS |
  | wasm32 check | pass | pass, exit 0 | PASS |
  | perf stretch (non-claimed) | 6/6 (optional) | 6/6 | PASS |
- Blockers found: none. Zero deviations from t11 recovery procedure were necessary (wasm32 target auto-installed via rust-toolchain.toml instead of manual add — same as t11 precedent note; `cargo run --release` in spike needed explicit `--bin pdf_a_spike` due to two binaries, no default-run key — cosmetic only, not a procedure deviation in substance)
- All xlsx-r2 three-phase gate claims at mdcore 460f302 reproduce exactly in a from-zero environment: independent re-verification confirms t11-c stage summary with zero drift
- Toolchain state for successors: rustup 1.29.1 / rustc 1.99.0 (b940084d7) / cargo 1.99.0 (5f94df478) / rustfmt 1.10.0-stable / clippy installed / targets: x86_64-unknown-linux-gnu + wasm32-unknown-unknown; pdfium payload at cleanroom-analysis/md-line/pdf-a/spike/bin/lib/libpdfium.so (chromium/8086); spike fixtures at .../spike/fixtures/ (4 files); both repos clean at 460f302 / 6d0fd1e

---
Task ID: t12
Agent: 主代理（第 7 次平台重置恢复 + 三阶段复验编排）
Task: 平台重置恢复（双仓对齐 + 凭据重置）+ t12-a 复验编排 + 总计划进度汇报

Work Log:
- 第 7 次平台重置恢复：GITHUB_TOKEN 重置入 scripts/token.env（600 权限，不回显）；cleanroom-analysis dd7fb1a → 快进 6d0fd1e（未跟踪冲突三件备份至 scripts/backup-drift/ 后清除；stash 存平台漂移残留）；mdcore 目录被删 → token 克隆恢复（main = 460f302，compliance-baseline = 5e144cc）
- 权威状态核实：三阶段已全部收口于远端（Phase 1 e8c3a13 / Phase 2 042a5e4 / Phase 3 460f302；xlsx-r2 REPORT §7 三阶段总表 + 遗留四项登记）
- 派发 t12-a（子代理）：工具链从零重建（rustup 1.29.1 + rustc/cargo 1.99.0 + wasm32 + fmt/clippy）+ T1-F12 载荷 bootstrap（fetch-pdfium SHA256 验证 + fixtures 字节一致）+ 全门禁独立复验
- t12-a 审查通过：G1 392/392 + PDF 四面 79/81+6i/109+6i + clippy 四面仅 2 wasm 在档基线 + fmt 0 + scan 0+0 + wasm32 过 + perf 6/6（stretch）——全部声明复现零漂移，零阻塞
- 总计划进度汇报发送会话（三阶段收口+复验表 + 遗留四项定位 + 继续推进路径）

Stage Summary:
- 三阶段（xlsx-r2）完成态经从零环境独立复验确认；环境恢复完成度 = 核心 + feature 面 + perf；web 面待复验（t12-b）
- 遗留定位：①DOCX 增量面 = 唯一无阻塞开发项；②③为用户裁决阻塞项；④常驻纪律
- 下一步：t12-b web 冒烟基线复验 → t13 DOCX 增量第一轮（子代理执行后审查模式持续）
---
Task ID: t12-b
Agent: subagent (web environment recovery + web smoke baseline re-verification)
Task: 7th reset web env recovery (wasm-bindgen 0.2.129 + npm + dist) + re-verify web smoke claims (18/15/28) at mdcore 460f302

Work Log:
- Context read first: worklog t12-a/t12 (L2110-2162), web/README.md (build + serve + test conventions), build-wasm.sh (two-step: cargo wasm32 release → wasm-bindgen 0.2.129 exact-per-lock → web/pkg/), package.json (build = node build.mjs; serve dist), contract_gate.py (18 checks, port 8931, playwright chromium), web_smoke.py (15 tests W1-W13+G1/G2, ephemeral port, chromium launch with --js-flags=--expose-gc --enable-precise-memory-info), ime_cdp.py (28 checks IC1-IC6 via CDP Input.imeSetComposition/insertText, DIFF_baseline evidence-only), REUSE.md L70 claim, gates.md L406 (15/15 + 28/28) — check counts independently pre-verified by static count (contract_gate 18; ime_cdp 6+4+5+4+5+4=28)
- Environment state before recovery: node v24.21.0 / npm 11.19.0 / python 3.12.15 (venv, playwright 1.57.0 module present), ms-playwright cache survived reset (chromium-1200 + chromium_headless_shell-1200 + ffmpeg-1011, dated 2026-10-13 = t11-era install, matching playwright 1.57.0 browsers.json expectation); NO system chromium (not needed — tests use playwright bundled); wasm-bindgen absent from ~/.cargo/bin; web/ had no node_modules/pkg/dist (7th reset confirmed)
- npm install in web/: OK — added 15 packages, 0 vulnerabilities (package-lock respected, ~1s)
- wasm-bindgen-cli 0.2.129 install: `cargo install` background attempts (nohup &, setsid nohup &) were killed by the sandbox when tool calls returned (log died mid-compile after 3 crates, no error output); sources were already in the registry cache — workaround: extracted wasm-bindgen-cli-0.2.129 from ~/.cargo/registry (shipped Cargo.lock, 254 pkgs) to /tmp/wbg-src, then `cargo build --release --locked --bin wasm-bindgen` (resumable fingerprints, 2m33s on 2 cores) → copied to ~/.cargo/bin/wasm-bindgen; `wasm-bindgen --version` = wasm-bindgen 0.2.129 (exact lock match, zero version drift)
- bash web/build-wasm.sh: OK — cargo build -p mdcore-wasm --target wasm32-unknown-unknown --release finished 18.74s with only the 2 documented baseline warnings (mdcore-wasm static_method_of, matches t12-a clippy baseline); wasm-bindgen --target web → web/pkg/ 4 files (mdcore_wasm.js 20608B, mdcore_wasm_bg.wasm 704325B, d.ts ×2)
- npm run build: OK — esbuild via node build.mjs, dist/main.js 531.2kb + index.html + mdcore_wasm_bg.wasm (self-contained dist/)
- SUITE 1 `python3 tests/contract_gate.py`: 18/18 PASS, exit 0 (all checks verbatim PASS: ready/contract_version v1.1/boot surface/bridge inventory/inject+chunk rebuild+event family/undo key contract/unit constitution/settings v1.1 read+write+rejects/progressive-only members both sides)
- SUITE 2 `python3 tests/web_smoke.py`: 15/15 PASS (total 15, failed 0, pass true), exit 0; W4 1MB evidence: load 254.2ms, typing p95 17.3ms, 20448 blocks; G1 4/4 gate_verdict "PASS — cursor p95 0.9ms < 16ms（R15 装饰增量窗口化后）"; G2 6/6 gate_verdict PASS（10MB JS heap 58MB 达标 per R19 split; wasm linear 425MB transparent; undo no-op + undo_entries==0 per ADR-B4）
- SUITE 3 `python3 tests/ime_cdp.py`: 28/28 PASS (total 28, failed 0, pass true), exit 0 — IC1 6 + IC2 4 + IC3 5 + IC4 4 + IC5 5 + IC6 4, all verbatim PASS; chromium-1200 CDP accepted the imeSetComposition {text} param shape (protocol_note in script references chromium-1243 measurements — current build behaves identically, no BLOCKER); DIFF_baseline_synthetic = evidence-only (no checks key, correctly excluded from count)
- Purity at end: mdcore git status clean @ 460f302 (web/pkg, dist/, node_modules/, tests/results/ all covered by web/.gitignore — zero tracked-file modifications, no commit, no push); no lingering http.server/chromium processes after suites; evidence files at web/tests/results/{results_contract.json, results.json, results_ime.json}
- Progress milestones appended incrementally to /home/z/my-project/scratch-t12b-progress.md (5 entries)

Stage Summary:
- Claims vs actual (all at mdcore main 460f302, from-zero 7th-reset environment):
  | Suite | Claim | Actual | Verdict |
  | contract_gate.py | 18 tests | 18/18 PASS, exit 0 | PASS |
  | web_smoke.py (W1-W13 + G1/G2) | 15 tests, 0 failed | 15/15 PASS, 0 failed, exit 0 | PASS |
  | ime_cdp.py | 28 tests | 28/28 PASS, 0 failed, exit 0 | PASS |
  | TOTAL | 61 | 61/61 | PASS |
- Blockers: none. One environment workaround required: background `cargo install` is killed by sandbox between tool calls — replaced with foreground resumable `cargo build --release --locked` from the registry-cached .crate (same pinned 0.2.129 source + shipped Cargo.lock); binary installed to ~/.cargo/bin, version verified. Browser = playwright 1.57.0 bundled chromium-1200 (survived reset in ~/.cache/ms-playwright; no system chromium needed; IME CDP text-param shape works on this build)
- Web environment recovered to full working state for successors: node 24.21.0 / npm 11.19.0 / playwright 1.57.0 + chromium-1200 / wasm-bindgen 0.2.129 @ ~/.cargo/bin / web node_modules + pkg + dist built; combined with t12-a core gates, ALL gates at 460f302 now re-verified from zero post-7th-reset

---
Task ID: t13
Agent: subagent (DOCX 增量第一轮)
Task: docx-rs 0.4.22 能力一手核验 + OPC 关系解析层 + 超链接目标/页眉页脚模型级暴露 + 全门禁 + 双仓推送

Work Log:
- 前置读取：worklog t11-c/t12/t12-a/t12-b（L2090-2189）；基线 mdcore main @ 460f302 clean；docx-rs 0.4.22 源一手逐文件核验（~/.cargo/registry/src/*/docx-rs-0.4.22/）
- Phase A 裁决（CAPABILITY_NOTES.md 九节 + 暴露/登记总表，源码锚点在档）：①超链接读=引擎部分支持（rid 透传、无联接、内部目标丢——reader/hyperlink.rs:13-26 + read_docx.rs:386-398）→ 自有 rels 层联接后暴露；②页眉页脚读=支持（六槽位挂接；引用悬空=引擎静默跳过 read_docx.rs:15-24）→ 暴露（静默→严格拒/兼容警告）；③页眉页脚写=支持（Docx::header 六 API + 计数递增怪癖[重复调用自产悬空引用]）→ 暴露 default 槽位 + 包装层原位替换防御；④超链接写=支持（"Now support external only"）→ 暴露；⑤脚注读=引擎受限（从不读 footnotes.xml）→ 登记；⑥⑦图片/域=字节/字符串面在 → 登记不暴露
- 实证勘误（重要）：§8 初判「rels 未知元素 → 引擎 unwrap panic」被探针推翻——XMLElement::from_str catch-all Ok(Unsupported)（xml_element.rs:447）永不 Err；引擎静默容忍不崩溃（整包直调返回 Ok，探针测试在档）；自有层语义=严格拒[OPC ECMA-376 Part 2 词汇表]/兼容警告+跳过（可见化）
- mdcore 实现（main 460f302 → 6f74b71 已推送）：src/docx.rs 自有 OPC 关系层（quick-xml 解析全部 .rels 部件：document 主表 + per-part 表[页眉自身 rels] + 条目名集；门 3 同族纪律：LIMIT_RELATIONSHIPS=10_000 两模式恒拒/词汇表·schema 严格拒兼容降级/悬空引用[无条目·部件缺失·路径逃逸../../越根词法检出]严格拒兼容 BrokenReference 警告；属性实体补偿 &amp;→&；降级去重）+ 读面（RunData.hyperlink 四变体 External/Internal/Anchor/Unresolved；DocxDocument.headers/footers 六槽位变体透传+块序；页眉内超链接 per-part rels 联接）+ 写面（RunSpec.link 外链 run；DocxWriter::header/footer 重复调用=原位替换防御）；io.rs WarningKind::BrokenReference 增量变体（XLSX 面零使用点）；零新依赖（quick-xml 直依赖第二使用面；Cargo.lock 零变化 → NOTICES v1.5 不动）
- 测试 +19（G1 392→411）：docx.rs 单测 2→12 + docx_roundtrip 5→14（外链全环含 &-实体链[写侧 escape_owned 单次转义+attr 原样写=读回单次反转义复原]/锚点读/悬空×双模式/页眉页脚全环+替换语义/页眉引用敌意三面/整包重复 Id[rId 动态定位]/整包词汇表/引擎容忍探针/页眉内链接净室自构造——写器恒发射空页眉 rels 部件原位改写）
- 门禁全绿：G1 411/0/1i；PDF 四面 79/81+6i/109+6i 零波及；clippy 四面仅 2 wasm 在档基线（修复自有 2 处：闭包去 mut + match→if let）；fmt 0；scan 0+0；wasm32 过（office-io 不进 wasm 闭包，web 资产未重建）；web 18/15/28
- 分析仓：docx-r1/ 新建（CAPABILITY_NOTES.md[一手证据底稿+勘误链] + REPORT.md[裁决表/落地面/测试/门禁表/开发期发现 4 项]）；xlsx-r1/CAPABILITIES §2.5 增量；FIDELITY §7 新增（超链接目标闭项+仍登记面+新可见性）；DEPS quick-xml 角色注记（零新增）

Stage Summary:
- 遗留①（DOCX 增量面按需）第一轮收口：OPC 关系层落地 + 解锁三面（超链接目标读/外链写/页眉页脚读写）全暴露且全实测；脚注/图片/域/first-even 写面按纪律登记不暴露（第二轮候选，引擎能力锚点在 CAPABILITY_NOTES）
- G1 392→411（+19）；PDF 四面零波及；mdcore main 460f302→6f74b71（已推送）；cleanroom-analysis 本提交（含 t12-a/t12-b 未入档 worklog 补录 + docx-r1 目录 + 三文档增量）；NOTICES 零增量（v1.5 维持，无合规基线提交）
- 开发期教训：①源读初判须实证复核——unwrap 面看似 panic 实则 catch-all 永不 Err（探针先行再定语义）；②docx-rs 写器恒发射空页眉 rels 部件（zipper/mod.rs:84 无条件）——净室自构造测试注入 rels 须原位改写而非追加；③引擎 Docx::header 计数递增怪癖=包装层防御先例（同 XLSX 流式纪律）

---
Task ID: t14
Agent: subagent (DOCX 增量第二轮——脚注/图片/域, 遗留① 收口轮)
Task: docx-r2 四面收口（脚注/图片/域/first-even）+ 全门禁 + 双仓推送

Work Log:
- 前置读取：worklog t13/t12-b/t12-a；docx-r1 CAPABILITY_NOTES（§5/§6/§7
  二轮候选锚点）+ REPORT；mdcore main @ 6f74b71 clean；基线 G1 411
- Phase A 一手核验（CAPABILITY_NOTES §10 增补 + 勘误链）：①脚注写=引擎
  支持（add_footnote_reference run.rs:363-369 重置 marker 样式 + 打包
  自动收集 mod.rs:914-938 + zipper 恒写 footnotes.xml）→ 暴露；读=引擎
  受限（无部件读取 + reader/run.rs:63-146 无引用解析臂）→ 登记锚点
  精确化；②图片写=**r1 勘误**：Pic::new_with_dimensions 不受 image
  特性门控（pic.rs:66；仅 Pic::new 自动转码面被门控 pic.rs:53）→
  PNG-only 零依赖暴露；image 特性代价=/tmp cargo tree 对照 37→60 crate
  **+23 crate**（image v0.25.10 树逐项在档）→ 自动面登记不启用；
  ③图片读=引擎三静默怪癖（非 PNG 默认路径丢弃 mod.rs:291-327/悬空关系
  跳过 read_docx.rs:404-423/仅 document rels）→ 自有容器级清单绕开；
  ④域：fldChar/instrText 元素族在（add_tc 三连先例 run.rs:202-208）→
  PAGE/NUMPAGES 暴露；HYPERLINK=todo!() 崩溃面（instr_text.rs:32）绝不
  暴露；TOC/PAGEREF 登记；⑤first/even 写=激活语义完整（first 自动
  titlePg section_property.rs:95/even 自动 evenAndOddHeaders mod.rs:613）
  → 暴露；⑥zip 8.6 重名条目=by_name 最后胜（zip_archive.rs:49-51）+
  ZipWriter 拒重名（write.rs:1380-1386）——语义钉定
- 开发期发现 r1 潜在缺陷：replace_default_header 无条件 rIdHeader1 在
  多槽组合下踩踏（r1 暴露面不可达；暴露 first/even 后可达）→ r2 改
  保持原槽 rid + 踩踏序列集成测试
- mdcore 实现（main 6f74b71 → cbe85c0 已推送）：读面 = [Content_Types].xml
  自有解析器（词汇表/缺属性/重复条目同 rels 层纪律 + Override>Default
  查表）+ 媒体清单 ImageAsset 五字段（bytes/content_type/references
  [全 .rels 表 image 型联接]/body_uses[正文+六槽位 a:blip 经属主表计数]）
  + 敌意纪律（悬空关系/越根/悬空 embed=严格拒/兼容 BrokenReference；
  无内容类型声明=严格拒/兼容警告+None；External 链接图与非 media 目标
  跳过登记；孤儿媒体保留）；写面 = RunSpec.footnote/field_page/
  field_num_pages + DocxWriter::image（PNG 签名+正尺寸 typed 验证）+
  first/even 四方法（原位替换保 rid）；零新依赖（Cargo.lock 零 diff）
- 测试 +15（G1 411→426）：docx.rs 单测 12→19 + docx_roundtrip 14→22
  （脚注 XML 级[id 配对动态定位]/简单域/图片模型级全环/悬空 rel/
  未声明+孤儿媒体/悬空 embed/重名 zip 净室手造[测试内 CRC32+手写本地
  头/CD/EOCD；python zipfile 交叉验证 21 条目重名 2 testzip None 同
  last-wins]/first-even 全环含踩踏序列）
- 门禁全绿：G1 426/0/1i；PDF 三面 79/81+6i/109+6i 零漂移；clippy 恰
  2 wasm 基线（自有 5 新警修净：2 doc_lazy_continuation+2 map_entry+
  1 type_complexity）；fmt 0；scan 0+0；wasm32 过；web 18/15/28（资产
  未重建——wasm 面零波及）；NOTICES 零增量（v1.5 不动，无合规基线提交）
- 分析仓：docx-r1 CAPABILITY_NOTES §5/§6/§7 r2 交叉引用 + 新 §10
  （10.1-10.5 含 r2 总表）；REPORT 新 §6 Round 2；xlsx-r1 CAPABILITIES
  §2.5 docx-r2 增量行；FIDELITY 新 §7.1（闭项 2[图片媒体面+first/even
  写面]+新暴露 2[脚注/简单域写]+仍登记更新+新可见性）；DEPS 不动

Stage Summary:
- 遗留①（DOCX 增量面）四面全部收口：脚注（写暴露/读登记）、图片
  （读清单暴露/写 PNG 暴露[r1 勘误]/自动转码登记）、域（PAGE/NUMPAGES
  写暴露/读登记/HYPERLINK 崩溃面绝不暴露）、first/even 写面（暴露）——
  **无剩余无裁决面**
- mdcore main：6f74b71 → cbe85c0（已推送）；Cargo.lock 零 diff；G1
  411→426（+15）；PDF/web/wasm 零波及
- 开发期教训三件：①r1「写面需 image 特性」初判被一手源推翻——特性
  门控面要逐 API 粒度核验（构造器族可能部分门控）；②包装层无条件
  rid 假设在暴露面扩大后失效——防御逻辑的假设面要随暴露面扩展重验
  （踩踏序列测试钉定）；③zip 8.6 写拒重名读 last-wins——重名敌意
  档案须净室手造（测试内 CRC32 + 手写 zip 头），python zipfile 交叉
  验证独立确证

---
Task ID: t15
Agent: 主代理（t13/t14 审查 + 阻塞边界收口）
Task: DOCX 增量两轮主代理审查 + HANDOFF 版本链同步 + 阻塞边界呈报（遗留②③待用户裁决）

Work Log:
- t13 审查通过：6f74b71 仅 office-io 三文件（MD/PDF 零波及）；公开面零 docx-rs 类型泄漏（grep 实证）；亲跑 office-io 61 测试全绿与 G1 411 对账一致；供应链初判勘误（rels panic→引擎容忍）以探针实证入档
- t14 审查通过：cbe85c0 仅 office-io 两文件 +1801/−33；亲跑 office-io 76 测试（44+3+22+1+6）全绿与 G1 426 对账一致；r1 缺陷修复（replace_default_header 槽位踩踏）带序列测试；图片特性门控勘误（new_with_dimensions 无门控——PNG-only 写面零新依赖暴露，自动转码 +23 crate 登记不启用）；HYPERLINK 域 todo!() 崩溃面绝不暴露
- 遗留①穷尽确认：四面（页眉页脚/脚注/图片/域）+ first/even 槽位全部裁决——暴露面均有全环实测，engine-limited/崩溃面/dep-代价面均有源码锚点登记
- HANDOFF.md L3/L-MD 双行版本链同步（mdcore 18d5bb2→cbe85c0 全链 + NOTICES v1.3→v1.5 + G1 362→426 + 第 7 次重置复验注记）；本提交推送

Stage Summary:
- 子代理执行后审查模式运行 4 任务（t12-a/t12-b/t13/t14）全部审查通过，零返工
- 开发线阻塞边界到达：遗留①穷尽；②XLSX 样式保真编辑（umya vs 自研 vs 维持读-改-重建）= 需用户裁决；③门 2 受信源跳过 = 维持否决（无基准证据）；④fork 线观察 = 常驻纪律
- 最终态：mdcore main cbe85c0（G1 426/0/1i + PDF 四面零漂移 + clippy 2 基线 + fmt/scan 0 + wasm32 + web 61/61）；cleanroom-analysis c8fc486→0b21f04→本提交；compliance-baseline 5e144cc（NOTICES v1.5 恒，docx 两轮零新依赖）

---
Task ID: t16-pre
Agent: 主代理（t16 裁决记录前置）
Task: 用户三项裁决落档——①XLSX 维持 C3 读—改—重建（不引入 umya，宏/图表保真限制显式登记）②门 2 维持双读不设跳过 ③t16 立项（DOCX→mdc-wasm→web 端到端切片）

Work Log:
- 远端核实先行：mdcore main = cbe85c0（与远端一致，工作区干净）；cleanroom-analysis = 0b5b366（一致，干净）——不基于历史报告宣称
- 裁决①落档（FIDELITY.md）：丢失矩阵补两行——宏（vbaProject.bin）【推断】丢 + 图表（chart 部件）【推断】丢，锚点一手核验：calamine 0.36.1 xlsx/mod.rs:2577 `vba_project()` 为引擎潜在能力、office-io 读路径未调用（API 面不观测）；图表无任何数据 API（仅 lib.rs:254 chartsheet 类型枚举）；§4.3 必然丢失集同步 + §4.6 选项登记改裁决落定
- 裁决②落档（xlsx-r2 REPORT §7 遗留表）：②③状态更新——维持读—改—重建边界 + 门 2 维持双读验证不增跳过开关
- 裁决③立项 t16：核实 mdc-wasm 现状（262 行，仅依赖 mdcore/serde_json/wasm-bindgen，纯 MD 边界）+ web 结构（src/main.ts + 三 python 测试件）——office-io 接入为全新消费路径，派发子代理执行
- 本提交推送

Stage Summary:
- 三项裁决全部落档推送；t16 派发条件就绪（远端状态核实 + 依赖边界核实 + 消费面空白确认）

---
Task ID: t16
Agent: subagent (DOCX→mdc-wasm→web 端到端切片)
Task: office-io 首个消费边落地——.docx 导入 web 编辑器（真调用 + 错误处理 + 测试 + 文档）

Work Log:
- 前置读取：worklog t14/t15/t16-pre + t12-b；office-io docx.rs 模块文档
  与模型（docx.rs:1282-1469 读面五类型 + 读 API + DocxError）；
  mdcore-wasm lib.rs（262 行 + parse_source_str 纯函数先例）；
  web/main.ts（boot/窗口面/__inject→installNewCore 重建契约）；
  contract_gate.py（harness 范式 + boot_surface 加性安全面）；
  REUSE.md（§0「经 wasm 桥」消费路径在册 + §5 无禁边条款——
  三域纪律新边仅 mdcore-wasm→office-io，裁决兼容）
- Phase A 裁决：切片 (A) 导入（复用硬化读面 + 既有 ingestion，弃 (B)
  导出）；markdown.rs:152-169 证实 GFM 表受支持（表映射不降级）；
  可行性探针先行——wasm32 check 一次过（calamine/rust_xlsxwriter/
  docx-rs/zip/quick-xml 全 wasm32 可编译）+ Cargo.lock 行为实证
- mdcore 实现（main cbe85c0 → ddeea4a）：docx_import.rs 新模块
  （自由函数 import_docx：兼容模式读 + 纯映射内核 7 函数 + typed
  ImportError 六错误码 + stats 摘要；build_sample_docx 夹具产线
  单一真源）+ examples/gen_docx_fixture.rs + 提交夹具 26625B +
  web 工具条 UI + main.ts __importDocx（成功经 __inject 重建核，
  敌意/失败 typed 码入状态行编辑器零触碰）
- 测试：原生 +9（mdcore-wasm 2→11：全环精确等值/标题钳位/run 合并/
  链接四变体/表清洗补齐/跳过计数/垃圾×3+截断 typed 拒/夹具漂移
  守卫）；新 web 第四套 docx_import.py 9 检查（真实 file input 路径
  精确 markdown/块型/stats+状态行/活编辑器编辑块数+1/undo 契约/
  垃圾+截断零触碰/契约完整 0 pageerror）
- 门禁全绿：G1 426→435/0/1i；PDF 三面 79/81+6i/109+6i 零漂移；
  clippy 恰 2 wasm 基线；fmt 0；scan 0+0；wasm32 过；web
  18/15/28/9；tsc --noEmit 清洁
- Cargo.lock 恰一行 diff（mdcore-wasm 条目 +mdc-office-io——工作区
  内部 path 边机械登记；零新三方包/零版本变动）→ NOTICES v1.5
  不动（无合规基线提交）；任务书「零 diff」字面预期对任何新 path
  依赖不可达，最小必然 diff 如实登记
- wasm 包体：704,325→1,829,489 B（+159.8%）；dist/main.js
  +3,490 B——诚实记录（无门禁线）
- 分析仓：docx-r1/REPORT.md 新 §7（Round 3：裁决/落地面/映射规则
  与不导入面/测试证据/门禁表/开发期发现四件）；xlsx-r1/
  CAPABILITIES §2.5 消费面宣称（实测口径校准 + 不宣称面）

Stage Summary:
- 用户故事达成：web 编辑器「import .docx」按钮 → .docx 真实导入为
  可编辑 markdown 文档（标题/强调/链接/GFM 表），敌意输入 typed
  拒绝且编辑器零触碰——office 首个端到端消费切片全绿收口
- mdcore main cbe85c0→ddeea4a（9 文件 +960/−1：mdcore-wasm 4 新/改
  + web 4 新/改 + Cargo.lock 1 行）；MD/PDF/office-io 源零触碰
- 遗留：无阻塞。登记边界（非缺陷）：显示保真不宣称；Title/自定义
  样式→段落；图片/页眉页脚/脚注不导入（计数显式化）；MD 结构字符
  开头段落的行级解析解释（结构映射非显示保真）

---
Task ID: t16-review
Agent: 主代理（t16 独立审查）
Task: t16 DOCX→mdc-wasm→web 端到端切片——代码与测试证据独立审查

Work Log:
- 提交核实：mdcore ddeea4a 已推送与远端一致零漂移（9 文件 +960/−1，仅 mdc-wasm+web）；Cargo.lock 单行 = mdc-office-io 路径依赖注册（零三方包变化，NOTICES v1.5 恒成立）
- 边界核实：wasm 导出面零 docx-rs/calamine/zip 类型泄漏（grep 实证）；错误面 6 typed 码（门 1/2/3 拒绝与引擎解析错误分立，模块文档映射表在档）；WasmCore 既有面零改动（增量边）
- 测试亲跑复验：mdc-wasm 原生 11/11（映射逻辑原生可测 + bindgen 壳分离结构）；web docx_import.py 9/9（含敌意垃圾/截断 zip typed 错误面 + 编辑器前后字节一致断言）；web_smoke.py 15/15 pass=true（MD 零回归独立复证）
- 包体核实：mdcore_wasm_bg.wasm = 1,829,489 B（704,325 起 +159.8%，诚实登记，无尺寸门禁违反）
- 文档核实：REPORT §7 Round 3 + CAPABILITIES §2.5 消费面宣称 + worklog t16 均在档（分析仓 698976c 已推送）

Stage Summary:
- t16 审查通过零返工：office-io 首个 wasm/web 消费边落地，端到端真实可用（UI 导入→容器三门→模型→markdown 映射→编辑器核心重建，可编辑可撤销）
- 全链证据：G1 435/0/1i（+9）· PDF 四面零漂移 · clippy 2 基线 · fmt/scan 0 · wasm32 过 · web 四套件 18/15/28/9 全过
- 剩余缺口登记在档（图片/页眉页脚/脚注内容不导入仅计数、非 Heading 样式降段、MD 结构字符语义解释、表格内链接目标丢、h7-9 钳 h6）——切片边界诚实，无范围蔓延

---
Task ID: t17-pre
Agent: 主代理（第 8 次平台重置恢复 + t17 派发前置）
Task: 第 8 次重置恢复（token 文件/工具链 PATH/双仓对齐）+ t17 立项（XLSX 读面 web 消费切片，t16 模式对称展开）

Work Log:
- 重置影响面：scripts/token.env 被删（会话既有 PAT 重建，有效性经克隆实证）；mdcore 目录被删（克隆恢复 main = ddeea4a）；分析仓本地退回 dd7fb1a（45 文件权限位漂移丢弃 + 陈旧未跟踪子集备份 scripts/backup-drift-8/ 后快进 9e06bf9）；rustup/cargo 被清（重装 1.99.0 钉住 channel，wasm32/fmt/clippy 组件实为在位仅 PATH 断）；web 面全失（wasm-bindgen CLI/node_modules/dist/pkg——t17 子任务恢复项，t12-b runbook 在案：registry .crate 前台可续构建）
- cargo fetch 全量成功 + office-io check 过——依赖面完好
- t17 立项：XLSX 读面 web 消费（import .xlsx → 容器门 → calamine 读 → sheet→GFM 表 markdown → 编辑器），复用 t16 全部模式（typed 错误码族/__importDocx 页面契约/e2e 套件模式/包体诚实登记）

Stage Summary:
- 第 8 次重置恢复完成度：双仓对齐零漂移 + 工具链在位 + 依赖缓存完好；web 面恢复归入 t17 执行（非独立阻塞）
- 派发 t17 子代理执行，主代理审查跟进

---
Task ID: t17
Agent: 子代理执行（超时切断于 Phase D 前，A/B/C 全部完成）+ 主代理接管收口
Task: XLSX 读面 web 消费切片——import .xlsx → 字节入口 → calamine 五面 → sheet→GFM 表 markdown → 编辑器（t16 模式对称）

Work Log:
- [子代理] 第 8 次重置 web 环境恢复：npm install + wasm-bindgen 0.2.129（crate 缓存亦失，static.crates.io 下载 + /tmp 前台可续构建 2m29s）+ build-wasm + dist 构建；基线复证 contract 18/18 + smoke 15/15
- [子代理] Phase A 决策（锚点在档 scratch-t17）：office-io 字节入口增量（wasm32 无 fs，docx from_bytes 先例）；错误码族与 t16 同构（xlsx_parse 命名 + 单层 ParseError 差异登记）；值面=缓存值 Display 语义（公式文本→formulas_flattened 统计，FIDELITY §1 双歧义依据）；上限 16/50/32 + dropped 计数
- [子代理] Phase B：io.rs 字节入口 + extract_sheets 共享核（+2 测试）；xlsx_import.rs ~660 行（纯核+壳+夹具单源+10 测试）；web 按钮/__importXlsx 契约；e2e xlsx_import.py 9 检查
- [子代理] Phase C 全绿：G1 447/0/1i（435+12）+ PDF 三面零漂移 + clippy 仅 2 基线（1 自产新警已修）+ fmt/scan 0 + wasm32 + 五套件 18/15/28/9/9；Cargo.lock 零 diff；wasm 包体实测 +12.95%（死码剥离面转活——t16 docx 路径从不调 calamine xlsx 读机构，诚实归因在档）
- [主代理] 超时切断后接管：读 scratch 进度核全程证据 → io.rs diff 审查（三门恒等+单一真源）→ 亲跑复验（office-io 78/mdc-wasm 21/四 web 套件 18+15+9+9 全过）→ 泄漏检查（仅模块文档注释提及 calamine，代码面零引用）→ mdcore 提交推送 de2ecf7 → 本 worklog + 文档收口

Stage Summary:
- mdcore main：ddeea4a → de2ecf7（t17 已推送）；G1 447/0/1i；wasm 2,066,405 B（+12.95% 实测）；dist/main.js +3,662 B
- C1 消费面宣称落地（CAPABILITIES §1 t17 增量）：import .xlsx 端到端真实可用，不宣称面显式登记（numFmt/公式求值/超链接目标/样式图表图片）
- Office 消费面双子全齐：DOCX 导入（t16）+ XLSX 导入（t17）——容器门/wasm 桥/页面契约/e2e 模式三件套完全同构，维护成本边际递减
- 剩余缺口：DOCX 导出方向（编辑器→.docx 下载）未做；XLSX 写面 web 消费未做（C2 新建工作簿下载——决策面：是否暴露）；两者均为后续增量候选非阻塞

---
Task ID: t18
Agent: 子代理（DOCX 导出切片）
Task: 编辑器当前文档 → .docx 下载端到端——BlockTree 原生遍历 → office-io DocxWriter 写面 → wasm 字节出口 → web Blob 下载，与 t16 合流成 DOCX 往返闭环

Work Log:
- Phase A（锚点 scratch-t18-progress.md）：mdcore tree.rs 全 pub 原生遍历（零 JSON hop）；marks = 自由 kind 字符串 + char 偏移（strong/em 规范 + bold 拼写），**模型无链接 URL 载荷**（html.rs 无 `<a>` 面）；表 = 扁平管道行文本；office-io 写面核对：无字体/列表/水平线面 → 降级表逐项登记
- Phase B：docx_export.rs 新增 ~570 行（纯内核 7 函数 + WasmCore::export_docx 增量方法 Result<Vec<u8>,JsValue> → Uint8Array + 错误码单族 docx_build——映射层全函数降级永不报错）；web export .docx 动作钮 + __exportDocx 契约面 + Blob/a.download + 状态行（simplified 降级计数取 __blocks 现场面）；修 2 处测试期望错（mapper 本身正确）→ 原生 10 测试全过
- Phase C 门禁全绿：G1 **457/0/1i**（447+10）；PDF default/lopdf/pdfium/dual = 68/79/81+6i/109+6i 零漂移；clippy 恰 2 基线（PDF 三面 0）；fmt 0（新增代码已 fmt）；scan 0 hit-paths；wasm32 过；Cargo.lock 零 diff；**六套件 18/15/28/9/9/8 全过**（t16/t17 零回归实证）
- wasm 包体诚实登记：2,066,405 → 3,105,817 B（+50.3%）——docx-rs 写面机构（OOXML 序列化+打包）首次从导出函数可达（t16/t17 夹具构建器只被 #[cfg(test)]/examples 引用，cdylib 导出图死码剥离）；**t17「pub fixture builder 保活」归因勘误**；dist/main.js +3,133 B
- 全环实证（e2e）：真实按钮点击 → 合法 zip（PK/[Content_Types].xml/document.xml/testzip）；导出前后 __md() 逐字不变（只读性）；export → t16 __importDocx → markdown 逐字还原（非降级子集）
- 文档：docx-r1/REPORT.md §8 Round 4（映射表+降级+测试证据+门禁表+包体归因）+ xlsx-r1/CAPABILITIES.md §2 消费面 t18 增量（导出宣称口径 + 不宣称面）

Stage Summary:
- mdcore main：de2ecf7 → **9ee7f24**（t18 已推送）；分析仓本轮提交待推
- DOCX 往返闭环完成：t16 导入（.docx→编辑器）+ t18 导出（编辑器→.docx），双方向真实可用（UI + wasm 面 + e2e 六套件门禁）
- 降级面诚实登记于 docx_export.rs 模块文档（单一真源随码走）：code/quote/list/divider/行内 code·del·u/link；模型级无源映射（链接 URL）非降级
- 剩余缺口（后续增量候选，非阻塞）：降级块型的 docx 表达面（需 office-io 写面扩列表面/等宽字体面——t14 登记的逆方向）；页眉页脚/脚注/域/PNG 写面接线（DocxWriter 能力在库、未暴露给本切片）；XLSX 写面 web 消费（t17 登记缺口维持）

---
Task ID: t18-review
Agent: 主代理（t18 独立审查）
Task: t18 DOCX 导出切片——代码与测试证据独立审查

Work Log:
- 提交核实：mdcore 9ee7f24 已推送零漂移（5 文件 +965/−1，仅 mdc-wasm+web；Cargo.lock 零 diff 独立确认——git show 空 diff）
- 边界核实：docx_export.rs 零引擎类型泄漏（grep 实证，仅模块文档）；export_docx 为 WasmCore 增量方法（既有面零改动）；映射为全函数（降级不报错）+ docx_build 单错误码族
- 测试亲跑复验：mdc-wasm 31/31（21+10）；docx_export 8/8（含真实按钮点击 Blob 下载路径 + python zipfile 验证 + 全圆环 export→import→markdown 字节精确）；docx_import 9/9 + xlsx_import 9/9 零回归亲证
- 包体核实：wasm 3,105,817 B（+50.3% 与报告一致）；归因诚实（docx-rs 写面机构首次可达 + t17 夹具保活归因勘误入档）
- 文档核实：REPORT §8 Round 4 + CAPABILITIES t18 增量 + worklog t18 均在档（分析仓 fc8bc4f 已推送）

Stage Summary:
- t18 审查通过零返工：DOCX 往返闭环完成（t16 导入 + t18 导出，全圆环 e2e 字节精确互证）
- 消费面三部曲齐：DOCX 导入/导出 + XLSX 导入（G1 457/0/1i，六 web 套件 18/15/28/9/9/8 全绿）
- 上游缺口定位三件（均非本层可解）：MarkSpan 无 URL payload（链接导出被模型卡死）/ office-io 写面缺 list·monospace·hr·quote 面（降级根因）/ XLSX 写面 web 消费未接线

---
Task ID: t19
Agent: 子代理（XLSX 导出切片）
Task: 编辑器当前文档 → .xlsx 下载端到端——BlockTree 原生遍历 → office-io WorkbookWriter 写面 → wasm 字节出口 → web Blob 下载，与 t17 合流成 XLSX 往返闭环（office 消费三部曲收口）

Work Log:
- Phase A（锚点 scratch-t19-progress.md）：write.rs 精确 API + typed 验证面（表名全规则/MAX_ROW/MAX_COL）；t18 docx_export.rs 结构模板 + GFM 管道语义三函数；t17 import_xlsx_core（原生全环对面）；main.ts/index.html t18 接线面。映射决策：Table→每表一独立 sheet（全 Text 类型不推断——前导零/日期式字符串不失真 + t17 Display 逐字等值）；非表线性块→前置 Document 表（heading 加粗/level 丢/marks 降级）；空文档→单空表 Sheet1；表名 = 表头 A1 清洗派生（非法字符剥除/截 31/History 回退/冲突后缀去重）
- Phase B：xlsx_export.rs 新增 ~590L（纯内核 8 函数 + WasmCore::export_xlsx 增量方法 + 错误码单族 xlsx_build——映射全函数降级永不报错）；docx_export.rs 两函数 pub(crate) 共享（GFM 单一真源）；web export .xlsx 钮 + __exportXlsx/__lastXlsxExport（t18 约定镜像）+ 状态行 N sheets；原生 +12 首跑 11/12（wide_row 测试数据自身错修正后 43/43）
- **wasm32 运行时崩溃面两项一手发现**（e2e 首跑 wasm trap "unreachable"——wasm32 check 编译期不可见，t16 可行性探针盲区勘误）：①SystemTime::now（rust_xlsxwriter DocProperties 默认时间戳链）→ 引擎 wasm 特性（js_sys::Date；原生 cfg 恒 SystemTime 零变化）；②constant_memory 使 Worksheet::new 对每表（含非流式）急切 tempfile_in(temp_dir()) → "no filesystem" → 依赖边目标分裂 + office-io add_sheet_streaming #[cfg(not(wasm32))] 条件化（流式面 = 引擎 tempfile 结构性质；诊断 = 临时 panic hook 读真实消息后移除）
- Phase C 门禁全绿：G1 **469/0/1i**（457+12）；PDF 四面 68/79/81+6i/109+6i 零漂移；clippy 恰 2 基线（PDF 三面 0）；fmt 0；scan 0+0；wasm32 双面过；Cargo.lock 恰 +2 行（rust_xlsxwriter deps +js-sys +wasm-bindgen，两包均已在 lock——零新包零版本变动，wasm32 崩溃修复无 API 旁路的最小必然 diff，t16 先例登记）；**八套件 18/15/28/9/9/8/9 全绿**（t16/t17/t18 零回归亲证，终版 dist 复跑）
- 全环实证（双证）：原生 export_xlsx_core → t17 import_xlsx_core → markdown 逐字等值（表 + Document 表两形态）；e2e 真实按钮 → 合法 zip → __importXlsx → 规范形 markdown 逐字（三部曲合拢）
- 包体与归因勘误：wasm 3,105,817 → 3,950,549 B（+844,732，+27.2%）——rust_xlsxwriter 写面机构首次从导出函数可达，**t17「pub fixture builder 保活 rust_xlsxwriter」归因勘误成立**（非 wasm 导出面 → cdylib 死码剥离；t18 docx-rs 同款勘误的 XLSX 对偶）；dist/main.js +2,991 B
- 文档：xlsx-r2/REPORT.md §9 Round 3（映射表/wasm32 崩溃面/Cargo.lock 边界/测试/门禁/归因勘误）+ xlsx-r1/CAPABILITIES.md 消费面 t19 增量（宣称口径 + 不宣称面）

Stage Summary:
- mdcore main：9ee7f24 → **7705b44**（t19 已推送，9 文件 +1038/−4）；G1 469/0/1i；八 web 套件全绿
- office 消费四切片全部收口：DOCX 导入（t16）/ XLSX 导入（t17）/ DOCX 导出（t18）/ XLSX 导出（t19）——双格式双方向端到端真实可用，XLSX/DOCX 各自成往返闭环
- 开发期教训两件：①「wasm32 check 编译过」不证运行——运行时崩溃面（SystemTime/tempfile）只有真实 e2e 可暴露，可行性探针须如实分级；②引擎 feature 与目标双轴 cfg（feature"wasm" AND target"wasm32"）使特性启用对原生零行为影响——依赖边目标分裂是最小诚实修复
- 剩余缺口（后续增量候选，非阻塞）：XLSX 数值/布尔类型面（全 Text 决策的逆方向——需单元格类型推断策略裁决）；单元格富文本（rust_xlsxwriter 富文本 API 不在 office-io 包装面）；t17 导入规模上限（16/50/32）与 t19 导出无上限的不对称（导入面边界）

---
Task ID: t19-review
Agent: 主代理（t19 独立审查）
Task: t19 XLSX 导出切片——代码与测试证据独立审查（重点：wasm32 运行时崩溃修复 + Cargo.lock +2 行）

Work Log:
- 提交核实：mdcore 7705b44 已推送零漂移（9 文件 +1038/−4）；Cargo.lock 恰 +2 行（rust_xlsxwriter deps 数组 +js-sys +wasm-bindgen——两包均已在 lock，零新包，最小必然 diff 如实登记）
- wasm32 修复审查（两项一手发现，源码锚点在 Cargo.toml 注记）：①SystemTime::now() 运行时 panic → 引擎 wasm 特性切 js_sys::Date::now()（原生 cfg 恒真行为零变化）；②constant_memory tempfile 结构性质 → target-split 依赖（非 wasm32 保留 constant_memory，wasm32 仅 wasm 特性）+ add_sheet_streaming wasm32 条件化缺席（API 缺席非降级，模块文档登记）；write_roundtrip 6 测试亲跑确认 native 流式面在位
- 测试亲跑复验：office-io 78（46+3+22+1+6）+ mdcore-wasm 43/43（31+12）；xlsx_export 9/9（含全圆环 export→import→canonical markdown 逐字互证）+ xlsx_import 9/9 + docx_export 8/8 零回归亲证；wasm 3,950,549 B 与报告一致
- 归因勘误核验：t17「fixture builder 保活 rust_xlsxwriter」归因错误（非 wasm 导出面→cdylib 死码剥离）——t19 实测修正，诚实登记

Stage Summary:
- t19 审查通过零返工：XLSX 往返闭环完成；office 消费面四部曲齐（t16-t19：DOCX 导入/导出 + XLSX 导入/导出，双格式全圆环 e2e 字节/逐字互证）
- 供应链纪律执行到位：wasm32 运行时崩溃是 t16 以来首个「check 过但运行崩」案例——两项发现入档教训（可行性探针只证编译不证运行；引擎结构性质面须 target-split）
- G1 469/0/1i；八 web 套件 18/15/28/9/9/8/9 全绿；PDF 四面零漂移
- 消费面收口后剩余缺口：t18 降级根因四件（list/monospace/hr/quote——office-io 写面未暴露，docx-rs 能力待一手核验）= 下一增量 t20 候选；MarkSpan URL payload = mdcore MD 域上游缺口（触碰域边界，需单独裁决）

---
Task ID: t20-a
Agent: 子代理（docx-rs 写面四件 API 一手核验）
Task: docx-rs 0.4.22 写面能力边界一手核验（t18 四件降级项 + 块级 code 等宽）——源码 API 签名 + spike 探针实证，t20 扩写面消降级的前置可行性证据

Work Log:
- 2026-10-15 第 10 次环境回滚后重写恢复：原 588891b 提交（工作区文件 + git 对象 + /tmp 探针产物）被平台回滚删除，本节与四产物按 transcript 逐字重建，探针 16/16 在恢复环境重跑复证（release 构建）
- 读 worklog t18/t18-review/t19/t19-review 四节 + docx-r1/REPORT.md §8 Round 4——四件降级语义核对（list=marker/depth 丢、行内 code=无等宽面、hr=无水平线面、quote=缩进丢失；另 CodeBlock 等宽全丢 = t19-review 遗留顺带项）
- spike Cargo.toml 仅新增 docx-rs = "0.4.22"（与产品 office-io 同版；不动既有四依赖）；Cargo.lock +282/-2 = 30 新包（docx-rs default features=["image"] 拉 png/gif/tiff 解码链；其 zip=8.6.0 与 spike 既有同版零分裂）——spike 钉版提交设计内
- 一手源码核验（crate 源 ~/.cargo/registry/src/.../docx-rs-0.4.22/src/）：numbering 三层 API（Docx::add_abstract_numbering/add_numbering + AbstractNumbering::add_level(Level::new(ilvl,Start,NumberFormat,LevelText,LevelJc)) + Paragraph::numbering(NumberingId,IndentLevel)）；Run::fonts(RunFonts) / Paragraph::fonts；ParagraphBorder/ParagraphBorders + ParagraphProperty::set_border（pub 字段挂接）；Paragraph::indent(SpecialIndentType)——全锚点入 WRITE_FACE_PROBE.md §2
- 关键机制发现：word/numbering.xml 无条件写入（t18 现有输出已含）+ 内建 abstractNum id=1/numId=1；条件项 = document.xml.rels 关系（has_numbering 链）——t20 必须走 Paragraph::numbering 方法（numbering_property 路径不置位→悬空 numPr）；ParagraphBorders::new()=四边全 Single 陷阱（hr/quote 须 with_empty().set(单边)）
- 探针 src/bin/docx_write_probe.rs（新 bin）：五类表达面全构造 → /tmp docx → 进程内 zip 读回 16 断言；首跑 15/16（唯一 FAIL=探针断言自身计数错，XML 正确）修正后 **16/16 PASS exit=0**；独立第二通道复证（unzip -l/-p grep 四结构面 + python zipfile testzip OK + rels rId7→numbering.xml）
- 顺带发现：Run::strike/dstrike/underline 在库——t18 行内 del·u 降级库层面同可达（t20 RunSpec 扩面可同批消解）；链接 mark 维持模型上游缺口（MarkSpan 无 URL 载荷）非库层面
- 文档：docx-r1/WRITE_FACE_PROBE.md（§1 目的/§2 五件核验表/§3 探针实证/§4 t20 实施映射草案〔office-io 五面新增 + docx_export.rs 映射改法 + 测试门禁预置〕/§5 结论）

Stage Summary:
- **四件降级 + 块级 code 等宽全部库层面可达（可行），无「库也不可行」项**——t20 扩 office-io 包装面即可整批消解；残项仅链接 URL（模型上游）与 code lang/fence（库无语法高亮面）维持登记
- 探针 16/16 全过（双通道实证：进程内断言 + unzip/python 复证）；探针断言集可直接移植为 t20 原生/e2e 测试模板
- 已知怪癖如实登记：docx-rs pPr 子序非 ECMA-376 典范序（rPr 恒居首）——t18 既有输出同形，与 t18 基线同边界，t20 不新增风险面
- 产物：spike 新 bin + Cargo.toml/lock 增量 + WRITE_FACE_PROBE.md（分析仓本地提交，无 push 权限）；mdcore 仓本轮不可访问未触碰
---
Task ID: t20-a-review
Agent: 主代理（t20-a 独立审查 + 第 10 次回滚恢复 + PAT 解锁）
Task: PAT 恢复接入 + 第 10 次回滚事故处置（产物重建）+ t20-a 四维独立审查

Work Log:
- PAT 解锁：新凭据写入 scripts/token.env（600 权限）实证有效——mdcore 私有仓 ls-remote 通（远端 main = 7705b44 与 t19 记录一致）+ clone 恢复（工作区 clean 零漂移）
- 第 10 次回滚事故定性：回滚窗口精确命中 t20-a 原提交（588891b/0677e35）之后、token.env 写入之前——分析仓工作区与 .git 双双回退至 dd7fb1a 旧快照（原两提交对象删除、WRITE_FACE_PROBE.md/spike 探针/scratch 全失），token.env 与 mdcore clone 幸存；~/.cargo 与 ~/.rustup 亦被清（第二次工具链丢失，重装 1.99.0+wasm32+clippy+rustfmt 同链）
- 恢复路径：主代理 reset --hard origin/main（2b0268b）→ resume t20-a 子代理按 transcript 逐字重写五产物 → 新提交 2b04b11（5 文件 +927/−2，与原 588891b 零逻辑差异，仅两处恢复注记行）→ 探针 release 重跑 16/16 PASS exit=0 亲验 + unzip/python 第二通道计数一致；诚实勘误随恢复入档：原「30 新包」为首轮 head -30 截断计数，全集 31 包（漏计 zune-jpeg），包集合与锁形态 +282/−2 两轮完全一致
- t20-a 审查（对 2b04b11）：提交范围核实（docx-r1 文档 + spike Cargo.toml/lock/bin + worklog，既有四依赖零改动）；亲跑复验 16/16（numbering 三层/有序无序多级/行内与块级 rFonts 双通道/pBdr hr/quote ind+左框全 XML 结构面）；WRITE_FACE_PROBE.md 五节全（§2 源码锚点 + §4 t20 实施映射草案——office-io 五面新增/docx_export 六行映射表/测试预置）
- 关键机制复核采信：numbering.xml 无条件写 + rels 条件接线（必须走 Paragraph::numbering 否则悬空 numPr）+ ParagraphBorders::new() 四边全画陷阱（须 with_empty().set 单边）——两项一手发现纳入 t20 实施要点
- 防再丢措施：本轮起恢复完成即即时推送（远端为持久真源），不再积压本地提交

Stage Summary:
- t20-a 审查通过零返工：t18 四件降级（list/monospace/hr/quote）+ 块级 code 等宽 + 顺带 del·u 全部库层面可行（docx-rs 0.4.22 一手核验 + 探针 16/16 双通道实证）；残项仅链接 URL（模型上游缺口维持）与 code lang（库无此面登记维持）
- 第 10 次回滚恢复完成：分析仓 2b04b11 即时推送；mdcore clone 在位（7705b44）；工具链在位；PAT 在位
- t20 主体实施路径就绪：WRITE_FACE_PROBE.md §4（office-io 五面扩 + docx_export.rs 六行映射改法 + 测试门禁预置），即刻派发
---
Task ID: t20
Agent: 子代理（docx 导出消降级切片·b 轮接力）
Task: docx 导出消降级——office-io DocxWriter 写面五扩 + docx_export.rs 六行映射改法，整批消解 t18 登记的降级项（list/monospace/hr/quote + 块级 code 等宽 + 行内 del·u）；底稿 = WRITE_FACE_PROBE.md（t20-a 一手核验 + 探针 16/16）

Work Log:
- 局面勘察：mdcore 本地 HEAD = 5028ec4（ahead 1 未推送）——**上轮 t20 子代理已完成代码并本地提交**（5 文件 +1029/−56，提交信息与本任务描述逐字吻合）但被中断于分析仓文档前（无 t20 worklog 节/REPORT Round 5/CAPABILITIES 增量）；本轮 = 遗留提交审查 → 亲跑全门禁 → 补文档收口（代码零改动采纳）
- 遗留提交审查（对照 WRITE_FACE_PROBE §2/§4 逐面）：office-io 五面全在且两项陷阱规避到位（list_item 恒走 Paragraph::numbering 接通 has_numbering→rels；hr/quote 恒 with_empty().set 单边——四边全画陷阱反证注记在模块文档）；RunSpec 三位（mono/strike/underline）既有构造器恒 false 零漂移；numbering 惰性注册 + save 基态重置；docx_export 六行映射（rich_lines 提取单一真源、list_shape marker 判序+depth 钳 0-8、quote_depth 步进、code 双通道）；三域纪律（office-io 零 mdcore 依赖）与 Cargo.lock 零 diff 核实
- 门禁全绿（亲跑）：G1 **478/0/1i**（基线 469+9 只增：office-io 内联 46→48 + roundtrip 22→24〔探针 16 断言移植 + 惰性注册钉定〕、mdc-wasm 43→48〔5 新测试 + 1 语义翻转；XML 面直断言新通道 zip_stored_part——打包恒 Stored 零解压直切，zip 依赖不进 mdc-wasm〕）；PDF 四面 68/79/81+6i/109+6i 零漂移 + perf --ignored 6/6；clippy 恰 2 wasm 基线 + PDF 三面 0；fmt 0；scan 0+0；wasm32 过；web 七套件 18/15/28/9/9/9/**12** 全绿（docx_export 8→12：+4 t20 XML 结构面断言〔numbering/rels/pBdr/ind/rFonts〕，degraded 计数 5→1 翻转；六既有套件零回归亲证）
- 包体：wasm 3,950,549 → **3,969,144 B（+18,595 = +0.47%）**——WRITE_FACE_PROBE §4.3「docx-rs 写面机构 t18 已付、本轮纯映射胶水近零增长」兑现；dist/main.js 557,256 → 557,211 B（−45：注释剥除 + Set 字面量缩短）
- 诚实边界登记：numPr/pBdr/ind/rFonts 的读面往返承载未核（t20-a 遗留④）——e2e 只做导出 XML 面 + 文本面回归，不宣称 list/quote/hr/等宽往返等值；code lang/fence 与 link URL 维持降级/无源登记
- 文档：docx-r1/REPORT.md §9 Round 5（五面表/六行翻转表/陷阱两项实录/测试证据/门禁表/残余全景）+ xlsx-r1/CAPABILITIES.md 消费面 t20 增量（可表达面翻转 + 不宣称清单）+ 本节

Stage Summary:
- mdcore 本地 5028ec4（未推送——主代理审查后推）；分析仓本轮提交待推（REPORT §9 + CAPABILITIES t20 + worklog t20）
- t18 登记降级整批消解：list（numbering 三层 + rels 接线）/行内 code（rFonts）/行内 del·u（strike·underline——§2.6 顺带）/hr（pBdr bottom）/quote（ind+pBdr left 嵌套步进）/块级 code（双通道等宽）——web 状态行降级计数 5→1（仅 code_block lang/fence 库层不可达维持）
- 探针→产品测试移植闭环：t20-a spike 16 断言 = office-io `t20_write_faces_xml_level` 产品 API 移植 + mdc-wasm/e2e XML 面直断言三通道
- 剩余缺口（后续增量候选，非阻塞）：读面 numPr/pBdr/ind/rFonts 模型承载（往返等值前提——需 office-io 读模型扩字段）；code lang/fence（库层）；link URL（MarkSpan 上游——mdcore MD 域裁决前置）；多列族混排/自定义编号格式（引擎在、包装面未暴露）
---
Task ID: t20-review
Agent: 主代理（t20 独立审查）
Task: t20 DOCX 导出消降级切片——代码与测试证据独立审查（重点：遗留提交 5028ec4 采信 + 全门禁亲跑）

Work Log:
- 局面核实：首次 t20 派发超时（context deadline）但服务端实际执行至代码提交（5028ec4）后被切断——b 轮子代理发现遗留提交、对照 WRITE_FACE_PROBE §2/§4 逐面审查（两陷阱规避/六行映射/三域纪律/锁零 diff）后零改动采纳 + 补全门禁亲跑与文档；主代理对遗留提交独立四维复核
- 提交核实：mdcore 5028ec4（5 文件 +1029/−56：docx.rs +374/docx_export.rs +417/docx_roundtrip.rs +244/web main.ts +9/docx_export.py +41——范围恰为 docx 导出面）；Cargo.lock 零 diff 独立实证（git show 空 diff）；分析仓 9781371（REPORT §9 Round 5 + CAPABILITIES t20 + worklog t20，3 文件 +164）
- 边界核实：docx_export.rs 增量零 docx-rs/calamine 类型泄漏（grep 空匹配）；RunSpec 扩位既有构造器恒 false 零漂移；三域纪律在位
- 测试亲跑复验：office-io 82（48+3+24+1+6，+4）+ mdc-wasm 48（+5）+ G1 **478/0/1i** 全量亲跑；clippy native 恰 2 wasm_bindgen 基线 + PDF 三面 0；fmt 0；scan 0+0；wasm32 check 过；e2e docx_export **12/12**（四个 t20 XML 结构面新断言 + full_circle_reimport 维持 + contract 无 pageerror）——全部与汇报零漂移
- 额外发现（登记观察项非回退）：`cargo clippy --target wasm32-unknown-unknown --all-targets` 下 write_roundtrip 编译失败（add_sheet_streaming 在 wasm32 cfg 缺席 + 测试 244 行未同步 cfg）——**t19 target-split 既存结构性边界**（write.rs 最后改动即 t19 提交，t20 未触碰；门禁口径 wasm32 仅 check 不含 --all-targets，t19-review「双面过」与口径自洽）；修复选项（测试面 cfg 化）留微增量裁决，「既有测试只增不改」红线不自行扩动
- 文档核实：REPORT §9 Round 5 五扩表（含幂等续写钉定/常量面/两陷阱实录）+ 映射翻转表 + 包体 +0.47% 胶水兑现登记；CAPABILITIES t20 可表达面翻转 + 往返承载不宣称口径在档

Stage Summary:
- t20 审查通过零返工：t18 登记降级整批消解（list/monospace/hr/quote/块级 code 等宽/行内 del·u），G1 469→478，e2e docx_export 8→12，wasm 包体 +0.47% 预判兑现
- 残余降级四件维持登记：code lang/fence（库层）、link URL（MarkSpan 上游 mdcore MD 域裁决前置）、多行 item 续行缩进（映射登记）、读面往返承载（读模型扩字段裁决）
- 消费面四部曲后首个增量化收口：office 消费面全宣称与真实能力对齐（可表达面全表达，不可表达面显式登记）
- mdcore 7705b44 → 5028ec4 推送；观察项（wasm32 --all-targets 测试编译）登记待微增量

---
Task ID: t21
Agent: 子代理（wasm32 测试面修复微增量）
Task: 修复 wasm32 `--all-targets` 测试面编译失败（t20-review 登记观察项）+ REUSE.md §4 门禁口径收严

Work Log:
- 影响面全查（`cargo check --workspace --target wasm32-unknown-unknown --all-targets`）：全 workspace 测试面恰 **1 个**失败点 = E0599 `crates/office-io/tests/write_roundtrip.rs:244`（`w.add_sheet_streaming("大表")`——t19 target-split 条件化缺席 API 的测试侧对偶面）；其余 crate（mdshell/mdc-settings/mdc-pdf/mdcore/shell-native/mdc-office/mdcore-wasm）wasm32 测试面全部编译通过——定性：结构性缺席 API 引用，非真问题，t19 内联单测已 cfg 化（write.rs 622/640/718 行），仅集成测试漏同步
- 修复（断言语义零改动，仅编译门控 + 注释）：①`streaming_sheet_full_circle` 测试加 `#[cfg(not(target_arch = "wasm32"))]` + 注释锚点「t21：wasm32 结构性缺席，native 全跑」（锚到 write.rs 模块文档 t19 注记）；②`NonEmpty` trait + impl（该测试唯一私有辅助）随动同 cfg——防 wasm32 下 dead_code 警告（不随动则 trait/impl 无消费者告警，污染警告面）；与 write.rs t19 既有内联 cfg 模式（#[test] 后置 #[cfg]）同构
- 验证全绿：wasm32 `--all-targets` check 过 + clippy 过（警告面 = mdcore-wasm lib 恰 2 wasm_bindgen `static_method_of` 在档基线〔212/221 行，lib test duplicate 同源〕，修复零新增——NonEmpty 随动门控生效实证）；native 零回归：office-io **82**（48+3+24+1+6，write_roundtrip 6/6 含 streaming native 全跑）+ G1 **478/0/1i** 逐数对齐；native clippy --all-targets 恰 2 基线；fmt 0；scan 0 hit-paths；**Cargo.lock 零 diff 实证**（git diff 空——纯 cfg 属性 + 注释，无依赖面变化）
- 门禁收严：REUSE.md §4 wasm32 check 行后追加 `cargo check --workspace --target wasm32-unknown-unknown --all-targets` + `cargo clippy --workspace --target wasm32-unknown-unknown --all-targets` 两行 + 四行注释（t21 起收严 + 观察项修复锚点 + 此后 wasm32 结构性缺席面 API 与测试 cfg 须同步）
- web 面不跑理由：本轮零 dist/wasm 产物变更（无 lib 面改动、Cargo.lock 零 diff、无 web 文件触碰）→ wasm 产物位元不变，web 套件（G2）被测对象零漂移，无复跑必要

Stage Summary:
- mdcore 本地 **e967cf2**（2 文件 +12/−0：write_roundtrip.rs +6 / REUSE.md +6；不 push——主代理审查后推）；分析仓本轮本地提交（worklog t21 节；不 push）
- t20-review 观察项闭环：wasm32 门禁口径从「仅 lib check」收严为「lib + --all-targets check/clippy 双面」——t19 target-split 的 API/测试 cfg 同步纪律自此进成文门禁
- 修复面极小且诚实：单测试 + 单辅助 trait 纯编译门控，native 82/G1 478 零回归，Cargo.lock 零 diff，无任何断言语义/依赖/产物变化
---
Task ID: t21-review
Agent: 主代理（t21 独立审查）
Task: t21 wasm32 测试面修复 + 门禁收严——微增量审查

Work Log:
- 提交核实：mdcore e967cf2（2 文件 +12/−0——write_roundtrip.rs 两处 cfg 门控 + 注释锚点、REUSE.md §4 六行收严注记；断言语义零改动零删除实证，diff 全文 52 行逐行可读）；分析仓 375b487（worklog t21 节）；Cargo.lock 零 diff
- 亲跑复验：wasm32 --all-targets check Finished 过（观察项闭合）+ native office-io 82/0（write_roundtrip 6/6 streaming native 全跑）+ G1 全量 478/0/1i 零回归逐数对齐
- 修法合规性核验：恰 1 失败点定性正确（t19 target-split 对偶面，write.rs:303 cfg 与测试 244 行漏同步）；NonEmpty/SheetData 随动门控防 dead_code 污染——处置得当；其余 crate wasm32 测试面全过无遗漏
- 门禁收严核验：REUSE.md §4 wasm32 --all-targets check/clippy 两行入册 + 「结构性缺席面 API 与测试 cfg 须同步」纪律注记——口径外盲区转正式门禁

Stage Summary:
- t21 审查通过零返工：t20-review 观察项闭合，wasm32 测试面进门禁（此后 check 与 clippy 双 --all-targets 全绿为基线）
- mdcore 5028ec4 → e967cf2 推送；G1 478/0/1i 维持
- Office 线无需裁决的机械增量至此清空：剩余缺口全部带裁决前置（XLSX 类型面策略/导入导出上限对称/富文本消费方/MarKSpan URL 域边界/读面往返承载模型扩列）
---
Task ID: t22
Agent: 子代理（多行列表项续行缩进微增量）
Task: DOCX 导出多行列表项续行缩进对齐——t20 登记的映射层残余项（最后一件无需裁决的机械增量），office-io 新面 + docx_export 映射接线 + 测试三面

Work Log:
- 现状确认：ListItem 多行文本（op 面可构造——ReplaceText 无换行校验 op.rs:127-139；markdown 解析恒单行）当前导出 = 首行编号段 + 余行普通段（每行独立 body 级段落，无 numPr 无 ind——续行视觉顶格 = 不对齐根因，亦是项结构散失）；e2e 既有 12 断言零踩踏（DEGRADED_MD 全单行 item）
- 设计选择（映射层内部决策，任务优先项落地）：**单段落 + w:br + 段落级 hanging indent**——一手源码核验无 API 障碍（`Run::add_break(BreakType::TextWrapping)` run.rs:247、`Paragraph::indent` 四参 + `SpecialIndentType::Hanging`——退回形态未启用）；理由：①一个列表项 = 一个 OOXML 段落（numPr 单点）语义最正确，块内 `\n` = 项内软换行正是 w:br 语义；②对齐几何 = 编号抽象层级既有悬挂缩进（marker 列 = 720×ilvl+360、首行文字列 = left = 720×(ilvl+1)），br 续行起于段落 left → 与首行文字列精确同列；③新方法同值显式直挂段落级 `w:ind`（Word 自产文档惯例形态；document.xml 自含几何 + 渲染器对 numbering 层级 pPr 应用不全时更鲁棒）；④读面对称（office-io 读面 br→`\n` docx.rs:1633 t16 约定——多行项读回单段含 `\n`，文本层多行形态保持；列表 kind 往返等值仍不宣称，读模型无 numPr 登记维持）
- /tmp 独立探针（docx-rs 0.4.22 raw build）先证序列化形：`<w:pPr>…<w:numPr>…</w:numPr><w:ind w:left="720" w:right="0" w:hanging="360" /></w:pPr>` + `<w:r><w:rPr /><w:br w:type="textWrapping" /></w:r>`（邻接串 = 同段性断言基础）
- office-io 扩面（只增不改）：新 `DocxWriter::list_item_lines(&[Vec<RunSpec>], ListKind, ilvl)`（ilvl typed 验证 + numbering 三层接线同 list_item；空行 = 空 run 组仅 br 分隔）+ 新 pub const `LIST_HANGING_INDENT=360`（t20 抽象层原字面 360 改引此常量——零行为变化，单一真源）；内部 `specs_to_paragraph` per-spec 构造抽 `add_spec_to_paragraph`（纯代码搬移，既有 specs_expand_* 测试锁零漂移）+ 新 `spec_lines_to_paragraph`（行组间插 br run）；既有 `list_item` 签名与行为零漂移
- 映射改法（docx_export.rs ListItem 分支）：rich_lines 行切后单行（≤1）走既有 `list_item` 路径**不动**（无显式 ind，单行项导出字节级零变化——翻转测试内单行对照断言锚定）；多行走 `list_item_lines`（每行 run 序 = rich_lines 平移行内 marks 后 runs_from）；模块文档 t22 节 + 映射表 ListItem 行 + 全局文本行规则 ListItem 例外注记
- 测试三面：office-io 内联 `list_item_lines_model_level`（ilvl 拒 + 模型级 numPr/ind 双挂 + br run 行边界子序）+ docx_roundtrip `t22_list_item_multiline_xml_level`（numPr ×3 单点 + br ×3 空行也 br + ilvl0/1 邻接 ind 步进 720/1440 hanging 360 + 单行项 pPr 止于 numPr 零漂移对照 + 读回 br→\n 单段零损）；mdc-wasm t20 平铺断言**语义翻转改写**（`list_multiline_item_first_numbered_rest_plain` → `list_multiline_item_single_paragraph_br_hanging`：单段 numPr+br+ind 断言 + CRLF 单段 + 单行零漂移对照）+ 新增 `list_multiline_item_nested_depth_and_marks`（depth 2 → ind 1440、三行含空行 br ×2、strong 跨行 char 平移）；e2e docx_export.py **+1 断言** `t22_multiline_item_br_hanging_xml`（markdown 注入单行 item → op 面 replace_text 插 `\n` 构造多行 → 导出 XML：单段 numPr + br + hanging ind 邻接串 + 文本在场——既有 12 断言零改动）
- 门禁全绿（亲跑）：G1 **481/0/1i**（478 基线 +3 只增：office-io 82→84〔内联 +1 + roundtrip +1〕、mdc-wasm 48→49〔翻转净 0 + 嵌套新测 +1〕）；PDF 四面 68/79/81+6i/109+6i 零漂移 + perf --ignored 6/6；clippy native --all-targets 恰 2 基线（1 条新 doc_lazy_continuation 已修——续行不以 `+` 开头）+ PDF 三面 0；fmt 0；scan 0 hit-paths；wasm32 check + clippy 双 --all-targets 过（恰 2 同基线，t21 收严口径）；Cargo.lock 零 diff；web 七套件 **18/15/28/9/9/9/13** 全绿（contract/smoke/ime/docx_import/xlsx_import/xlsx_export 零回归 + docx_export 13/13 = 12 零改动 + 1 新）
- 包体：wasm 3,969,144 → **3,970,827 B（+1,683 = +0.042%）**——纯胶水近零增长兑现；dist/main.js 557,211 B 零变化（web 面无代码改动，仅测试文件 +1 断言）

Stage Summary:
- mdcore 本地 **ef74282**（4 文件 +443/−66：office-io/src/docx.rs、office-io/tests/docx_roundtrip.rs、mdc-wasm/docx_export.rs、web/tests/docx_export.py；不 push——主代理审查后推）；分析仓本轮本地提交（worklog t22 + REPORT §9 Round 5 追记 + CAPABILITIES t20 增量处补注；不 push）
- t20 残余登记项「多行 item 续行缩进」翻转为已表达：一个列表项 = 一个 OOXML 段落（w:br 软换行 + 段落级 hanging ind 与编号抽象层级同值）——续行与首行文字列对齐；单行项字节级零变化
- Office 线机械增量彻底清空后的最后一件收口：残余降级全部带裁决前置（code lang/fence 库层、link URL 域边界、读面往返承载模型扩列、多列族/自定义编号包装面）——下一增量必须先过主代理裁决
---
Task ID: t22-review
Agent: 主代理（t22 独立审查）
Task: t22 DOCX 导出多行列表项续行缩进——微增量审查

Work Log:
- 提交核实：mdcore ef74282（4 文件 +443/−66——office-io/docx.rs 新面 list_item_lines + LIST_HANGING_INDENT 常量 + add_spec_to_paragraph 纯搬移；docx_export.rs 多行接线；docx_roundtrip.rs +103 测试；docx_export.py +1 断言）；Cargo.lock 零 diff 实证；分析仓 2d6c8ff（REPORT §9.6 追记 + CAPABILITIES 补注 + worklog）
- 边界核实：docx_export.rs 增量零 docx-rs 类型泄漏（grep 0）；既有 list_item 签名与单行导出路径零漂移（字节级回归锚定在测试）；设计选择合规（单段 w:br + 段落级 hanging ind——OOXML 语义正确形态，源码一手核验 + /tmp 探针双证）
- 测试亲跑复验：G1 全量 481/0/1i（478+3）+ e2e docx_export 13/13（12 既有零改动 + t22_list_item_multiline 新断言 + full_circle 维持）——与汇报零漂移
- 包体核实：wasm 3,970,827 B（+0.042% 纯胶水兑现）；dist/main.js 零变化

Stage Summary:
- t22 审查通过零返工：t20 登记残余「多行 item 续行缩进」翻转为已表达（单段 w:br + 悬挂几何与编号抽象层级同值对齐）
- **Office 线无需用户裁决的机械增量至此彻底清空**：全部剩余缺口带裁决前置——①code lang/fence（库层无语法高亮面）②link URL（MarkSpan 域边界，mdcore MD 域上游）③numPr/pBdr/ind/rFonts 读面往返承载（读模型扩字段）④多列族/自定义编号格式（包装面暴露按需）⑤XLSX 数值/布尔类型推断策略 ⑥导入导出上限对称性 ⑦富文本（模型无 marks 载荷=无消费方）
- mdcore e967cf2 → ef74282 推送；G1 481/0/1i 新基线
---
Task ID: t23
Agent: 主代理（HANDOFF 版本链同步，循 t15 先例）
Task: HANDOFF.md L3/L-MD 双行同步——t16-t22 七轮（消费面四部曲 + 三增量）版本链与门禁基线落档

Work Log:
- 滞后核实：L-MD 行停在 cbe85c0（t15 同步后 t16-t22 七个 mdcore 提交未入链）；L3 行停在 docx-r2 遗留②③裁决（消费面阶段未记）；产物清单 docx-r1 两件（WRITE_FACE_PROBE 未入）
- L-MD 行四处更新：main 哈希 cbe85c0→ef74282；演进链追加七节点（ddeea4a t16[G1 435]/de2ecf7 t17[447]/9ee7f24 t18[457]/7705b44 t19[469]/5028ec4 t20[478]/e967cf2 t21[门禁收严]/ef74282 t22[481]）；九门禁基线 426→481 + web 18/15/28→七套件 18/15/28/9/9/9/13 + 零新三方依赖[NOTICES v1.5 恒] + wasm32 --all-targets 收严注记；重置记录补第 9/10 次[t20-a-review]
- L3 行更新：追加消费面阶段段（Round 3，t16-t22 七轮全收口：三项裁决落档 + 四部曲 + 三增量 + G1 426→481 + 机械增量清空与裁决前置清单）；产物清单 docx-r1 两件→三件[+WRITE_FACE_PROBE]
- 同步口径：与 worklog 各 Stage Summary 逐节核对（t16-pre/t16/t16-review/t17-pre/t17/t18/t18-review/t19/t19-review/t20-a/t20-a-review/t20/t20-review/t21/t21-review/t22/t22-review 全链）；NOTICES 维持 v1.5 恒（t16-t22 零新三方依赖）无需动

Stage Summary:
- HANDOFF 双行对齐实况：分析仓与 mdcore 版本链/门禁基线/裁决状态全部到位，无欠账
- Office 线 Round 3（消费面）完整入档：机械增量清空呈报，剩余缺口七项全部带用户裁决前置
- 分析仓 9d2b563 → 本节提交推送
---
Task ID: r4-t0
Agent: 主代理（Round 4 执行计划 T0）
Task: 环境与基线确认——双仓状态/工具链/门禁基线复跑/凭据检查

Work Log:
- 双仓核实：mdcore ef74282 / 分析仓 b9d1b33，本地=远端零漂移，双工作区 clean 零未提交改动（上一轮 t23 收口态延续，无回滚发生）
- 工具链核实：cargo 1.99.0 + wasm32 + clippy/rustfmt 在位；node v24.21.0；web 环境（node_modules/dist/wasm pkg）在位
- 门禁基线复跑（全部亲跑）：G1 481/0/1i；PDF 四面 68/79/81+6i/109+6i 零漂移；clippy native 2 + wasm32 --all-targets 2（均恰 wasm_bindgen 基线）；fmt 0；scan 0 hit-paths；wasm32 check+--all-targets 过（t21 收严口径）
- 凭据核实：token.env 600 权限在仓外 /home/z/my-project/scripts/，不在任何 git 追踪面，日志不回显全文

Stage Summary:
- T0 通过：环境与基线与 t22/t23 收口态完全一致，零异常零回归，Round 4 起点干净
- 立即启动 T1（XLSX 类型推断与转换规则）派发子代理执行
---
Task ID: r4-t1
Agent: 子代理（Round 4 T1：XLSX 类型推断与转换规则——用户裁决落地轮）
Task: office-io 读面类型化单元格值模型（CellData/CellGrid/formula_at/replay_cell）+ 探针与 kind 往返实测 + GFM 映射显示面兼容切换 + 文档三面（CAPABILITIES C1/FIDELITY §8/xlsx-r2 REPORT §10）

Work Log:
- 协议阅读：worklog 尾三节（t22-review 裁决前置清单⑤即本轮主题/r4-t0 基线 481）+ FIDELITY（t17 缓存值 Display 语义/C3 丢失矩阵）+ CAPABILITIES（C1/C2/C3）+ 产品四文件（io.rs/write.rs/xlsx_import.rs/xlsx_export.rs）
- calamine 0.36.1 一手核验（registry 源码，不凭记忆）：**Data::DateTime 判定 = numFmt 证据非「Float+猜测」**（cells_reader.rs read_v：数字经 format_excel_f64_ref[n，cell_format，is_1904]；cell_format = s 属性→cellXfs→numFmtId→builtin 14-22/45/47 表或自定义格式码 d/m/h/y/s token 检测[formats.rs，openpyxl 移植]）；t="d"→DateTimeIso 原串；t="e"→Error（8 变体码文本）；t="b"→Bool；xlsx 数值恒 Float（Int 仅 pivot 路径 mod.rs:3621）；DurationIso xlsx 不可达；ExcelDateTime Display = 裸序列数（datatype.rs:986）；to_ymd_hms_milli 无 chrono 门控；has_1904_epoch 构造期 eager（new 内 read_workbook）——与用户裁决方向逐条对齐（日期依据已内建于引擎判定机制，模型层缺承载）
- T1.1 探针（tests/cell_types.rs 前四测试，raw XML zip 自构造 14 格：数字/前导零文本/布尔/日期双 numFmt 面/纯序列数/ISO/公式/空格/错误/2^53+1/0.1+0.2/时长/t="str" + 1904 纪元变体）：Data 面 + Display 面逐格实测钉定——前导零 `00123` 原样、纯序列数 44927 = Number（不认日期）、日期格 Display = 裸序列数（与纯数值同串不可分→类型面必要性实证）、2^53+1 落盘即 9007199254740992、1904 同 serial = 2027-01-02（+1462 天）
- 缺口证明（scratch）：读面模型无自有类型层（values 直暴露 calamine::Data 引擎类型）；读-改-重建对 DateTime/DateTimeIso/DurationIso/Error 四类静默丢格（write_roundtrip.rs None 分支——「不支持类型给可诊断结果」不满足）；序列数→ISO 无公开转换；公式坐标联接易错（锚定语义踩坑在档）
- 实现（office-io 只增不改，既有调用方零破坏）：**CellData 九变体**（Text/Number/Boolean/DateTime{serial,is_1904}/DateTimeIso/Duration{serial,is_1904}/DurationIso/Error(String)/Empty——三分纪律：原始值/推断类型/显式转换；kind 依据表 = 变体文档）+ display()（t17 Display 语义同源）+ as_iso_datetime()/as_iso_duration()（显式转换——无据 → None，纯 Number 序列数不转换）+ to_write_parts()（kind 分派 → (write::CellValue, Option<CellStyle>) 成对返回防样式漏配；DateTime/Duration → Number+DATE_REPLAY_FORMAT/DURATION_REPLAY_FORMAT；1904→1900 纪元换算 epoch_to_1900[+1462，写面引擎恒 1900 纪元]；ISO/Error → Text 原样[写面无对应面——值保持+kind 降级登记]）；**CellGrid**（与 values 同一包围盒/坐标系单点派生；get 相对+get_abs 绝对双坐标面）+ SheetData.typed 字段 + **formula_at**（公式文本独立通道，锚定换算内化）+ **replay_cell**（读-改-重建单调用点：公式联接[缓存 kind 映射——DateTime→序列数缓存+日期样式/Error→码文本缓存/Empty→None 两义性在档]+kind 分派）；extract_sheets 附建（is_1904 = has_1904_epoch 一次查询）
- GFM 映射（xlsx_import）：渲染切至 typed.get().display()（显示面字节零变化——模块自此零 calamine 面接触）；公式计数/规模上限/stats 零变化；新 wasm 测试 typed_display_face_kinds_preserve_text_semantics（前导零逐字+日期格=序列数+数值/布尔 t17 同源）
- wasm 导出（xlsx_export）：**零代码变化**——BlockTree 文本模型无单元格类型载荷，「数字文本不擅自转数字」裁决使从文本推断类型不可行（前导零会失真）→ 全 Text 维持 = 裁决正确语义（任务书最小版路径：读面类型模型+导出按模型 kind 分派[office-io 层 replay_cell 已实测]）；模块文档 lang 语义登记（模型扩类型载荷 = mdcore 域上游裁决未立项不宣称）
- 测试（T1.3 全类覆盖）：cell_types.rs 9 测试（探针 4：Data 七类/Display 钉定/公式两通道/日期组件+1904；类型面 5：typed 七类 kind+双坐标越界/display 全格等值[typed.display == Data.to_string 逐字——t17 切换安全依据]/显式转换证据规则[含 E1 纯序列数→None]/formula_at+replay_cell 联接/kind 保持全链往返[探针档 replay→重读 kind+值双断言，含 1904 历法保持链]）；wasm +1；既有测试零改动（write_roundtrip/corpus_harness/range_anchor_pinned 的 Data 面断言原样全绿=既有消费方零破坏实证）
- 门禁全绿（亲跑）：G1 **491/0/1i**（481 基线+10：office-io 84→93、mdcore-wasm 49→50）；一次全量跑 r23_fingerprint 出现 14+1（负载敏感 flake——mdcore 域时间型测试与本轮零代码路径交集；隔离 8/8 绿+后续两次全量 491/0/1i 绿，如实登记）；PDF 四面 68/79/81+6i/109+6i 零漂移+perf --ignored 双面 6/6；clippy native --all-targets 恰 2 基线+PDF 三面 0+wasm32 --all-targets 2 同基线；fmt 0；scan 0；wasm32 check+--all-targets 过；Cargo.lock 零 diff；web 七套件 **18/15/28/9/9/9/13 全绿零断言改动**（显示面兼容 e2e 亲证）
- 包体：wasm 3,970,827 → **3,969,548 B（−1,279 B / −0.032%**——CellData::display 替代 wasm 面 calamine Data Display 路径净缩）；dist/main.js 557,211 B 零变化
- 文档三面：CAPABILITIES.md C1 r4-t1 类型语义增量（宣称口径+不宣称面）+ FIDELITY.md §8 类型规则（R1-R13 依据表[每条规则一行：规则/依据/测试锚点]+§8.2 不支持面与降级登记+§8.3 模型面）+ xlsx-r2/REPORT.md §10（工程收口：落地面/探针与往返/门禁/遗留）

Stage Summary:
- mdcore 本地 **c62d19e**（4 文件 +1,095/−15：office-io/src/io.rs[+427 模型与文档]、office-io/tests/cell_types.rs[新 606L 九测试]、mdcore-wasm/src/xlsx_import.rs[渲染切换+测试+文档]、mdcore-wasm/src/xlsx_export.rs[文档登记]；不 push——主代理审查后推）；分析仓本轮本地提交（worklog r4-t1 + FIDELITY §8 + CAPABILITIES C1 增量 + xlsx-r2 REPORT §10；不 push）
- 用户裁决四方向全部落地并实测：①默认保留（前导零/数字文本 Text 原样——探针+往返双证）②日期必须依据（numFmt/t="d" 证据进模型，纯序列数=Number；显式转换无据→None）③公式两通道区分（values/typed=缓存值，formulas/formula_at=文本，replay_cell 联接）④不支持类型可诊断（Error 码文本保留；ISO/Error 写面降级登记；DurationIso 防御性承载）
- t22-review 裁决前置项⑤「XLSX 数值/布尔类型推断策略」收口：读面类型模型 + kind 保持重放实测（DateTime/Duration kind 往返经 numFmt 证据链；Text/Number/Boolean 保持；1904 历法保持）；wasm 导出面全 Text 维持为裁决正确语义（BlockTree 类型载荷 = mdcore 域上游裁决登记）
- 登记缺口：BlockTree 扩类型载荷（若立项 wasm 导出可升级按 kind 分派——replay 通道已就绪）；numFmt 显示形态维持不宣称；公式求值引擎未立项
---
Task ID: r4-t1-review
Agent: 主代理（r4-t1 独立审查）
Task: T1 XLSX 类型推断与转换规则——四维独立审查

Work Log:
- 派发事故说明：T1 子代理派发两次均网关超时（context deadline），但服务端实际完整执行——scratch 212 行全程锚点 + mdcore 1dd5cb2 + 分析仓 e9745e4 双本地提交在位（未 push 合规）；主代理循 t20 先例对遗留产物独立审查
- 提交核实：mdcore 1dd5cb2（4 文件 +1095/−15：io.rs +427 CellData/CellGrid/typed/formula_at/replay_cell、cell_types.rs +606 九测试、xlsx_import.rs 渲染切换、xlsx_export.rs 文档登记）；Cargo.lock 零 diff 实证（git show 空 diff）
- 边界核实：mdcore-wasm 增量 5 处 calamine 匹配逐条精查全为注释（模块文档+行内），代码面零泄漏；CellData 独立 pub enum（office-io 自有类型层）；既有 values: Range<Data> 面不变（write_roundtrip/corpus_harness/range_anchor_pinned/docx 面零改动——既有消费方零破坏实证）；三域纪律在位
- 测试亲跑复验：office-io 93/0（84+9）+ G1 全量 491/0/1i + web xlsx_import 9/9 + xlsx_export 9/9（零断言改动全过 = 显示面兼容 e2e 亲证，R12 规则兑现）
- 文档核实：FIDELITY §8.1 R1-R13 类型规则依据表（每条：规则/一手源码依据/测试锚点——R4 日期 numFmt 证据/R7 公式两通道/R10 显式转换有据/R11 1904 纪元/R13 类型保持重放）+ §8.2 降级登记 + §8.3 模型面 API 清单 + CAPABILITIES C1 增量 + xlsx-r2 REPORT §10
- 裁决落地核对：用户四条裁决方向逐条兑现——默认保留（R1）/日期明确依据（R4-R6 numFmt 或 t="d" 证据，纯 Float 永不升级）/公式两通道区分（R7 formula_at 安全联接）/可诊断结果（R9 错误格码文本 + §8.2 降级登记非静默）
- flake 登记采信：r23_fingerprint 负载敏感一次失败（mdcore 域时间型测试，与 office 改动零代码路径交集；隔离 8/8 + 后续两次全量绿）——如实登记不改码，合规

Stage Summary:
- r4-t1 审查通过零返工：XLSX 类型语义层落地（CellData 九变体三分——原始值/推断类型/显式转换），G1 481→491，t22-review 裁决前置项⑤「XLSX 数值/布尔类型推断策略」正式闭环
- mdcore ef74282 → 1dd5cb2 推送；分析仓 e9745e4 → 本节提交推送
- 显示面兼容策略兑现：import markdown 逐字不变（t17 既有断言即钉定），类型信息进模型供 replay/导出/未来消费方
- T2（导入/导出资源上限与安全边界）即刻派发
---
Task ID: r4-t2
Agent: 子代理 a 轮（T2.2 实现，基准阶段被切断）+ 子代理 b 轮（审查校准 + 测试 + 基准 + 门禁 + 文档，commit 前被切断）+ 主代理接管（复验 + 提交）
Task: Round 4 T2 导入/导出资源上限与安全边界——限制清单逐项核实 + 缺口修补 + 边界测试 + 基准

Work Log:
- a 轮（超时派发服务端执行）：T2.1 四段清单（XLSX 读 R1-R13/DOCX 读 D1-D8/XLSX 写 W1-W8/DOCX 写 W9-W12 + 错误三分）+ T2.2 实现 6 文件 +673/−48；切断于基准阶段
- b 轮接力（超时派发服务端执行）：对 a 轮逐项审查修正四处（W3 表数 512→256——引擎 constant_memory 每表急切 tempfile、双簿并发 1024 fd EMFILE 实测证伪；W4 URL 限 32767→2080——引擎 url.rs MAX_URL_LEN 真值镜像、a 轮检查在 2081-32767 区间永不触发实测证伪；W4 补 merge 左上格面；补 wasm 渲染窗 16/50/32 与读面 1000/4M 两层关系登记）+ 测试 21 个 + 基准六场景 + 门禁全绿亲跑 + 文档四件（SECURITY_POLICY v1.2/CAPABILITIES×2/REPORT）+ spike sparse_bomb.rs 探针；切断于 commit 前
- 主代理接管：G1 全量 512/0/1i（491+21）+ sparse_grid_bomb_rejected_both_modes + sheet_count 三测试（读限/写限/fd 面）+ xlsx e2e 9/9 亲跑复验全过 → mdcore 6da3d10 提交
- **重大安全发现（稀疏包围盒炸弹）**：calamine 0.36.1 Range::from_sparse 按包围盒稠密化——1588 字节 xlsx（A1+XFD1048576 两格）→ ~550GB 分配 abort（ulimit 双形态同结果）；本轮以门 3 流式坐标镜像扫描修复（先于 calamine，strict/compat 两模式恒拒）——读面语义层无限制的实证闭环
- 补限制清单终态：读面 R1/R2（输入字节 302MB/52MB 双族——fs 元数据前置，先于任何读）、R10（表数 1000——空表名洪水 CPU 面）、R11（稠密包围盒 4M——稀疏炸弹面）+ DOCX D1/D2/D3（50MB 族对齐——修原 xlsx 300MB 常量误用）+ 写面 W2（全簿 1M 格预算）、W3（256 表 fd 校准）、W4/W4'/W5（文本 32767/URL 2080/公式 8192）+ DOCX W9/W10（段落/run 计数）——每项四件套（常量/执行点/typed 错误/测试）齐
- 错误分类三分对齐：读面新拒 → PrescanRejected（安全类，wasm 码 container_prescan 兼容不变）；写面 W2/W3 → WriteError::Limit（新变体）、W4/W5 → InvalidInput；DOCX W9/W10 → DocxError::Limit；wasm 新码 xlsx_limit/docx_limit
- 基准记录（examples/bench_limits.rs，VmHWM 口径，不设门禁阈值）：读 500k 格 517ms/38.5MiB；读 4M 格（恰=限）4410ms/278.9MiB；写 1M 格非流式 1139ms/143.4MiB vs 流式 1221ms/5.1MiB（RSS 28× 差）；DOCX 100k 段 801ms/390MiB（≈3.9KB/段）；DOCX 1M 段沙箱 OOM kill 诚实登记（限值=防御上界，实际包络受宿主内存约束——未实测不宣称可达）
- 登记不设面（有理由在档）：R6 压缩后单条目（≤输入自身非膨胀面）、R12 dimension 巨量声明（T3 实测无害维持）、D6 读文本长（族总量界内子集）、D8 媒体（族界联合）、W6/W12 输出大小（代理界 = 计数×长度）、W11 DOCX 写文本（Word 无规格限+引擎不检登记）、W7 临时文件（联合界 W3+W2）

Stage Summary:
- r4-t2 完成度：限制清单 33 项全部有执行点或明确登记理由（无「仅文档宣称」残留——SECURITY_POLICY v1.2 全量校准）；G1 491→512；七套件零断言改动全绿；Cargo.lock 零 diff；包体 −32,374 B（死码剥离面变化）
- 稀疏包围盒炸弹修复 = 本轮最高价值安全闭环（550GB 分配面 → 门 3 前置 typed 拒）
- mdcore 1dd5cb2 → 6da3d10；分析仓文档四件 + 本 worklog 节提交推送（主代理）
---
Task ID: r4-t3
Agent: 子代理（矩阵+方案+实施，三连切断于门禁/文档阶段）+ 主代理接管（门禁亲跑+文档+提交）
Task: Round 4 T3 DOCX 读写保真与内容承载——内容损失矩阵 + 最小保真方案 + numPr/ind 读面承载实施

Work Log:
- T3.1 矩阵：24 项五态判定（以产品代码与测试一手判定，不据引擎理论能力）——完整承载 8→10（+numPr/+ind 本轮）、部分承载 4、写入降级 2（可诊断）、读取丢失 8（3 项引擎私有性结构性）、尚未验证 0（逐行锚点）；落档新建 docx-r1/FIDELITY.md（DOCX 保真单一真源，循 xlsx-r1/FIDELITY §8 先例）
- 引擎面一手核验（CAPABILITY_NOTES §12）：numbering 链全公开（numId→kind 解析读面可达）；Bold.val/Strike.val/Underline.val/RunFonts/ParagraphBorders 六槽私有——presence-only 承载对显式关闭形态（w:val="none"/"false"）会误报，宁缺毋错登记不承载；Indent 全公开可承载；HyperlinkType::Anchor 写面在但 RunSpec 扩字段另行裁决
- T3.2 方案论证：A 原始 XML 保留（模型侵入大+触红线）否决 / C 拒绝不安全重写（零增益——numPr/ind 是 pPr 结构性字段）否决 / **B 最小版选定**（模型 +2 Option 字段只增不改，循 r4-t1 CellData 先例）
- T3.3 实施（子代理完成代码，主代理验证）：ParagraphNumbering{num_id, ilvl, kind, format} + NumberingIndex 解析链 + 悬空/numId=0/numStyleLink 边界三分处置 + ParagraphIndent{left,right,special} + docx_roundtrip 三新测试（读承载逐字段 / 悬空+自定义 lowerLetter numFmt 注入严格拒 vs 兼容承载 / 读→改→写→读全环 ZIP 部件+rels 对比——预期丢失集恰={footnotes.xml, media/*} 非意外删除）
- 子代理切断事故：三次超时派发（矩阵阶段死→b 轮接力做矩阵+方案+实施→门禁阶段死）——主代理接管亲跑全门禁 + 补写分析仓三文档（FIDELITY.md 新建/CAPABILITY_NOTES §12/CAPABILITIES 补注）
- 门禁（主代理亲跑全绿）：G1 **517/0/1i**（512+5：docx_roundtrip 26→28 + 内联 +2）；PDF 四面 68/79/81+6i/109+6i 零漂移；clippy native 2 + wasm32 --all-targets 2；fmt 0；scan 0；wasm32 check+--all-targets 过；Cargo.lock 零 diff；web contract 18 + smoke 15 + docx_import 9 + docx_export 13 全绿

Stage Summary:
- r4-t3 完成：numPr/ind 读面完整承载（写回降级显式化可诊断），保真矩阵 24 项全部有锚点判定，读→改→写全环验证非目标内容不意外删除；全量 OOXML 无损编辑不宣称（验收红线遵守）
- t20 登记的「读面往返承载（需读模型扩字段裁决）」缺口正式闭环（numPr/ind 两面）
- mdcore 6da3d10 → 本轮提交；分析仓文档四件（FIDELITY.md 单一真源建立）+ 本 worklog 节推送
---
Task ID: r4-t4
Agent: 主代理（Round 4 收口：全门禁快照 + 终审 + HANDOFF 同步）
Task: Round 4 T4 端到端集成与收口——全量门禁 + 终审五面 + 文档链同步 + 双库终态

Work Log:
- 全门禁快照（主代理亲跑，一次完整快照）：G1 **517/0/1i**；perf 6/6（pdfc_perf --ignored 正确口径）；PDF 四面 68/79/81+6i/109+6i 零漂移；clippy native 2 + PDF 三面 0 + wasm32 --all-targets 2（恰 wasm_bindgen 基线）；fmt 0；scan 0+0（含 --all 全仓模式）；wasm32 check+--all-targets 过；Cargo.lock 零 diff；web 七套件 **18/15/28/9/9/9/13 全绿**（smoke/ime 经 results JSON summary 字段核验 failed=0 pass=true——套件输出行格式差异不掩盖状态）
- 链路验证核对：XLSX 导入→类型→导出链（t1 replay_cell 类型保持 + t2 限额 + xlsx_import/export e2e 9+9）；DOCX 读取→修改→写出→重开链（r4t3_roundtrip_read_modify_write ZIP+rels 对比）；mdc-wasm/web 消费接口零变化（r4 三轮全部只增不改——contract 18 断言零改动全过即互证），无新增端到端测试需求
- 终审五面（主代理逐项）：①代码差异——三轮 15 文件全量复核（t1: io.rs CellData/CellGrid/replay_cell；t2: io.rs 限额 + write.rs 计数器 + docx.rs 限额；t3: docx.rs numPr/ind 承载），全部只增不改、既有消费方零破坏；②错误处理——三分对齐（PrescanRejected 安全类/WriteError::Limit·DocxError::Limit 资源类/InvalidInput 格式类），wasm 码兼容（container_prescan 不变）+ 新码 xlsx_limit/docx_limit；③测试质量——边界三件套（恰=限过/限+1 拒/极端值）+ 双模式同限 + 敌意载荷复用 + flake 如实登记（r23_fingerprint 负载敏感）；④依赖变化——零新三方依赖（Cargo.lock 三轮零 diff 实证），NOTICES v1.5 恒；⑤文档声明——FIDELITY §8（xlsx 类型规则 R1-R13）+ FIDELITY.md（docx 保真矩阵 24 项）+ SECURITY_POLICY v1.2（限额表 33 项）+ CAPABILITIES/CAPABILITY_NOTES 增量全部与代码对齐，「未实测不宣称」零违例
- HANDOFF 同步：L-MD 行版本链 ef74282→7055595（+r4 三节点 + 基线 517）+ L3 行 Round 4 段（四轮全记录）+ 产物清单 docx-r1 四件
- Round 4 执行纪律核对：每轮代码或可验证工程交付 ✓；主代理独立复核（T1 四维/T2 接管复验/T3 接管复验）✓；每阶段即时推送 ✓；复用既有探针门禁 ✓；平台切断四次的恢复（t1 服务端完整执行 / t2 a+b 轮接力 + 主代理接管 / t3 三连切断接管）✓；无裁决请求（无真实架构阻塞）✓

Stage Summary:
- Round 4 四轮全部收口：XLSX 类型语义（CellData 三分模型）+ 资源安全（33 项限额全执行 + 稀疏炸弹闭环）+ DOCX 保真（numPr/ind 承载 + 矩阵单一真源）+ 端到端终态（G1 481→517 零回归）
- 暂缓事项遵守：富文本/MarkSpan URL/自定义编号格式/code lang/统一 CRDT 未触碰（登记维持）
- 双仓终态：mdcore 7055595 / 分析仓本节提交——推送后核对远端 HEAD 一致、工作区干净
---
Task ID: r5-t0
Agent: 主代理（Round 5 执行计划 T0）
Task: 环境与基线确认——双仓状态/工具链/门禁基线复跑/凭据检查

Work Log:
- 双仓核实：mdcore 7055595（r4-t3）= 远端 HEAD 零漂移；分析仓 a5c0e4a（r4-t4）= 远端 HEAD 零漂移；双工作区 clean 零未提交改动（Round 4 收口态延续，无回滚发生；初始「ahead 6/16」为陈旧远端跟踪引用，fetch 后归零）
- 工具链核实：rustc/cargo 1.99.0 + wasm32-unknown-unknown + clippy/rustfmt 在位；node v24.21.0；web node_modules/dist（main.js + mdcore_wasm_bg.wasm）在位；wasm-bindgen CLI 在位
- 凭据核实：token.env 600 权限在仓外 /home/z/my-project/scripts/，不在任何 git 追踪面
- 门禁基线复跑（全部亲跑，零历史引用）：G1 **517/0/1i**；PDF 四面 68/79/81+6i/109+6i 零漂移 + perf --ignored 6/6；clippy native --all-targets 恰 2（wasm_bindgen static_method_of lib.rs:212/221 基线；grep 计数 4 = 2 实体 + 2 汇总行，实体即基线）+ PDF 三面 0 + wasm32 --all-targets 2 同基线；fmt 0；scan 0 hit-paths；wasm32 check+--all-targets 过；web 七套件 **18/15/28/9/9/9/13 全绿**（smoke/ime 经 results JSON summary failed=0 核验）
- T1 现状锚点核实（为派发准备）：docx_import.rs `docx_to_markdown` Block::Paragraph 分支完全忽略 `p.numbering`（列表降级为普通段落——FIDELITY.md L67 登记的「web 消费链 list 映射」缺口即本轮主题）；office-io 读面 ParagraphNumbering{num_id,ilvl,kind,format}（r4-t3）+ NumberingIndex 解析链已在位待消费；mdcore list 解析语义锚点 markdown.rs `list_item_of`（缩进 2 空格/级、`- `/`* `/`+ ` 无序、`N. `/`N) ` 有序）；夹具链 build_sample_docx→gen_docx_fixture.rs→web/tests/fixtures/import_sample.docx→docx_import.py 9 checks

Stage Summary:
- T0 通过：环境与基线与 r4-t4 收口态完全一致，零异常零回归，Round 5 起点干净
- 立即启动 T1（DOCX 列表导入映射）派发子代理执行

---
Task ID: r5-t1
Agent: 子代理 r5-t1（DOCX 列表导入映射，全阶段一次通过）
Task: Round 5 T1 DOCX 列表导入映射——numbering 读面承载（r4-t3）→ docx_import 消费链落地（列表段不再降级普通段落）

Work Log:
- 协议执行：worklog 尾三节（r5-t0/r4-t4/r4-t3）+ FIDELITY/CAPABILITIES + docx_import.rs/docx.rs/markdown.rs/docx_export.rs/main.ts/docx_import.py 全链一手读；scratch 全程在档（/home/z/my-project/scratch-r5t1-progress.md）
- T1.1 盘点（一手，锚点在档）：读面 ParagraphNumbering{num_id, ilvl, kind, format}（docx.rs:1413）+ 解析链 carry_numbering（docx.rs:1804——悬空严格拒/兼容警告+kind=None、numId=0→None、ilvl 不钳位）；不可达面（w:start/lvlText/isLgl/suff）不扩模型；mdcore list_item_of（markdown.rs:283——depth=indent/2+1、marker 存 attrs、`- `/`1. ` 尾空格项解析为空 ListItem、空行跳过使 `\n\n` 连接与紧凑列表同构树——测试实测验证）；写面对称面 docx_export list_shape（marker 馞字符数字→Ordered + ilvl 钳 0-8）
- T1.2 映射规则（scratch 定案后实现）：Bullet→`- `；Ordered→`{n}. `（计数器 key=(numId,ilvl)，同 numId 父项推进子级重起，不同 numId 独立，**任何非列表映射块中断清零重启**——Word 续号降级登记）；嵌套 2×min(ilvl,8) 空格钳 8（写面九级域对称）；kind=None（悬空/numStyleLink/超定义域）与 numFmt "none"（Word 该级不显示 marker——不发明数字宁缺毋错）→ 普通段落降级；自定义 numFmt→decimal 标准化；标题+numPr→标题赢编号丢弃；空项→marker+尾空格行（mdcore 空 ListItem 结构保真）；多行列表项续行脱离列表（既有软换行族）；stats additive 五键（list_items/list_items_degraded/list_numbering_dropped/list_depth_clamped/list_markers_normalized）
- T1.3 实现（docx_import.rs 单文件 +666 行；office-io/mdcore 零改动）：模块文档扩（映射规则+列表降级已知边界）+ IMPORT_MAX_ILVL/ListCounters（HashMap retain 清同 numId 深级）/ListMapping/list_marker_line + docx_to_markdown 列表分支（标题优先→列表分支→共享段落尾——降级段走段落尾自然中断 run）；build_sample_docx +列表节（ordered×2+嵌套 ilvl1+bullet×2——读回 100/101 全 decimal/bullet → e2e 无降级路径）；夹具再生 26,625→31,733 B
- T1.4 原生测试（docx_import 10→18，+8）：映射矩阵（多级计数/中断重启/独立 numId/无序混合 exact 断言）+ mdcore 解析闭环（markdown_to_tree 顶级序/嵌套父子/marker/depth——docx→markdown→BlockTree 消费链原生证明）+ 降级面（dangling kind=None + numFmt none + 空文本降级 + 警告透传 + 降级段中断重起）+ 自定义 numFmt 标准化 + ilvl 钳位（恰 8 不计/9/99 钳 16 空格 depth 9）+ 标题优先（中断重起）+ 空项 + 夹具全链（fixture_markdown_parses_to_list_tree）；既有 writer_roundtrip（exact md+stats 扩展不降强度）与漂移守卫强化（+list_items==5）；严格模式悬空 = office-io r4t3 zip 锚点（导入面恒 compat——strict 在导入不可达，零漂移）
- T1.5 web e2e：main.ts DocxImportStats +list_items + 状态行条件段 "N list items"（>0 显示——同 images/slots 惯例）；docx_import.py 9→11 checks（EXPECTED_MD 扩列表行 exact、EXPECTED_BLOCKS 嵌套 depth 1、import_list_nesting_depth 先序深度、import_list_markers_and_md_depth marker/深度逐项、stats 四降级键==0 + "5 list items" 状态行、live editor blockCount 9+列表存活、undo 回到 EXPECTED_MD）；wasm+dist 重建后 11/11 全绿
- T1.6 门禁（全部亲跑最终源码态）：G1 改动前 517/0/1i（亲跑基线）→ **525/0/1i**（+8 零回归）；PDF 四面 68/79/81+6i/109+6i 零漂移 + perf 6/6；clippy native 恰 2（static_method_of 基线）+ mdc-pdf 三组合 0 + wasm32 --all-targets 2；fmt 0；scan 0；wasm32 check+--all-targets 过；Cargo.lock 零 diff；web 七套件 **18/15/28/11/13/9/9**（docx_import 9→11）全绿；三域纪律 office-io 零改动实证（grep mdcore 仅 description 声明+文档注释）；包体信息面 wasm 3,969,548→3,953,796 B（−15,752）+ dist/main.js +149 B
- 中途修正三处（如实登记）：HashMap::retain 闭包双参签名；clippy doc_lazy_continuation 新告警（doc 行首 `+ ` 误判嵌套 marker → 措辞改「加空内容行」）——修后回恰 2 基线；两测试期望修正（mdcore 先序语义：中断段落落根层文档序 + 夹具嵌套项为子项非顶级——一手 trace 修正非放宽）
- 文档三面：FIDELITY.md 矩阵行 12/13/14 消费面注 + §3 缺口闭项（~~web 消费链 list 映射~~）+ 新 §5（映射语义/降级面/stats/证据/不宣称/门禁）；CAPABILITIES.md t16 宣称口径 +列表 + 新「消费面增量（r5-t1）」条 + t20/t22 不宣称口径按现状收窄留痕

Stage Summary:
- r5-t1 完成：docx 列表导入映射落地（`p.numbering` 完全忽略 → 全语义消费），r4-t3 读面承载首次接入 web 消费链；FIDELITY 登记缺口「web 消费链 list 映射」闭项
- mdcore 4 文件 +732/−30（docx_import.rs +666、main.ts +11、docx_import.py +85−24、夹具 Bin 26,625→31,733）；office-io/mdcore 域零改动；Cargo.lock 零 diff；不 push（主代理审查后提交）
- 降级面全登记可诊断（stats 五键 + 兼容警告透传 + 模块文档/§5）；不宣称：列表渲染保真（lvlText 拼号/w:start）、editor→export→import 单 e2e 复合往返（三段各自实测）
- 待派发：T2（按 Round 5 计划）
---
Task ID: r5-t1-review
Agent: 主代理（r5-t1 独立审查）
Task: Round 5 T1 DOCX 列表导入映射——四维独立审查

Work Log:
- 提交核实：mdcore 恰 4 文件（docx_import.rs +666[映射+模块文档+夹具+8 测试]/main.ts +11[类型+状态行 additive]/docx_import.py +85−24[9→11 checks]/夹具二进制再生成 26,625→31,733 B）；office-io 与 mdcore crate 零改动（三域纪律在位）；Cargo.lock 零 diff 实证
- 边界核实：ListCounters key=(num_id,ilvl) retain 清深级语义精查正确（父项推进子级重起=Word 多级；不同 numId 独立）；降级路径三分（kind=None/format="none"→普通段+计数；标题带 numPr→标题赢+list_numbering_dropped；ilvl>8 钳 8+计数）全部走显式登记非静默；stats 五键 additive 既有 key 零变化；既有断言全部强化非弱化（undo 从「不含尾串」升级为 exact==EXPECTED_MD；块类型 exact 8 序；stats 加严）
- 测试亲跑复验：G1 全量 **525/0/1i**（517+8，八新测试逐一过含 mdcore 解析闭环/降级/标准化/钳位/标题优先/空项/夹具树/漂移守卫）；PDF 四面 79/81+6i/109+6i + perf 6/6 零漂移；clippy native 恰 2 基线 + PDF 三面 0 + wasm32 --all-targets 2；fmt 0；scan 0；wasm32 check 双面过；web 七套件 **18/15/28/11/13/9/9 全绿**（docx_import 9→11：import_list_nesting_depth 先序嵌套深度 + import_list_markers_and_md_depth marker/深度逐项——列表确达 BlockTree 终端消费面）；ignored=1 疑点排查为我方 awk 列位口径（$6=failed 列），BLUEPRINT_NOTE ignored 测试在位无漂移
- 产物链核实：dist 再生（main.js 557,360 B +149；wasm 3,953,796 B −15,752/−0.40%）；e2e 11/11 过 = 新 wasm+main.js 实际生效亲证；夹具漂移守卫过 = 夹具与 build_sample_docx 同源
- 文档核实：FIDELITY.md L67 缺口闭项（删除线+§5 新建：映射语义/降级面/验证证据/不宣称面全量落档——w:start 起始值/非连续续号/单 e2e 复合往返均明确不宣称）；矩阵 12-14 行消费面增量补注；CAPABILITIES.md（xlsx-r1 中央文件——消费面宣称唯一真源所在地）r5-t1 增量条 + t20/t22 不宣称面历史口径精确化（list kind/深度三段各自实测）

Stage Summary:
- r5-t1 审查通过零返工：FIDELITY 登记的「web 消费链 list 映射」缺口正式闭环（t16 列表降级→结构化映射），G1 517→525，docx_import e2e 9→11
- mdcore 提交推送（本轮新基线 525/0/1i + web 11）；分析仓 FIDELITY/CAPABILITIES/worklog 提交推送
- T2（超链接锚点写回评估）即刻派发
---
Task ID: r5-t2
Agent: 子代理 r5-t2（超链接锚点写回评估——矩阵+探针+方案+裁决点，零实施红线遵守）
Task: Round 5 T2 超链接锚点写回评估——DOCX 读模型/写接口/RunSpec 三面承载能力盘点 + 四类目标区分探针 + 最小接口变更方案 + 裁决点（r4-t3 登记「anchor 写回另行裁决」的评估收口；公开模型变更零实施）

Work Log:
- 协议执行：worklog 尾三节（r5-t1-review/r5-t1/r5-t0）+ FIDELITY/CAPABILITY_NOTES + docx.rs/docx_import.rs/docx_export.rs/markdown.rs/tree.rs + 引擎注册源（docx-rs-0.4.22 一手）全读；scratch 全程在档（/home/z/my-project/scratch-r5t2-progress.md）
- T2.1 盘点（一手锚点全档）：RunSpec 链接面 = External URL only（link 字段 + 单点发射 docx.rs:2819-2822 HyperlinkType::External；无锚点字段）；DocxWriter 19 方法零锚点/零书签面；HyperlinkTarget 读面四变体全可达（自有 rels 层联接——引擎自身不联接且只收带 TargetMode 关系，本层补面）；引擎写面 Anchor+书签**全公开**（hyperlink.rs:42-44 w:anchor 属性形无 r:id；update_dependencies 只收 External 进 rels → 锚点零 rels 面；paragraph.rs:224-234 书签对；无书签存在性校验）；书签定义读面 = 引擎解析进模型但本层 extract_paragraph 静默丢弃（零警告零字段）；HYPERLINK 域 todo!() 红线维持（w:anchor 属性形与域形不相交）；全链往返 = DOCX 外链→`[t](url)` GFM 完整 / MD→树 = 字面文本（行级解析无 marks）/ 树→导出 = 零 w:hyperlink（MarkSpan 无 URL 上游缺口）
- T2.2 探针（5 新增全绿，只增不改）：①Internal 整包面（无 TargetMode 关系→Internal，严格过零警告——引擎自身丢此面/自有层补面的整包实证）②锚点优先序（w:anchor + 可解析 r:id 双属性→Anchor，引擎 reader/hyperlink.rs:14-26 行为镜像）③书签定义行为（静默丢弃零警告+被包裹 run 文本零损+锚点目标存在性读面不可辨——存在/不存在同形 Anchor 透传）④链接形态文本导出面（mdcore-wasm：markdown `[t](url)` 入树字面文本→导出零 w:hyperlink+字面逐字保留+重导入文本零损——「文本层往返成立/超链接层不成立」钉定）⑤**引擎直构 spike**（docx_rs 直调不经 DocxWriter：w:anchor 属性形+rels 超链接条目恰 1（外链对照——锚点零 rels 无懒注册陷阱）+书签 Start/End 对发射+引擎写→自有读全环 Anchor 透传零警告——Option A 实施面一手去风险）
- T2.3 方案（CAPABILITY_NOTES §13 落档，13.1 承载矩阵 15 行全锚点 + 13.2 缺口 + 13.3 选项）：**Option 0** 现状维持（成本 0 收益 0）/ **Option A 最小加性扩**（RunSpec +anchor: Option<String> 字段 + anchor_link() 构造器 + 发射点锚点分支 + 可并入 bookmark_paragraph 书签写面〔id 计数器包装层唯一化〕——加性非破坏循 r4-t1/r4-t3 Option 字段先例；struct-literal 位点 = office-io 内部 4 处机械面；外部消费者全构造器面零改动 grep 全证）/ **Option B 类型化枚举**（LinkSpec{External,Anchor}——类型安全但公开字段类型变更破坏性 + 能力增量 0，不推荐）/ **Option C writer 方法 only**（段落级包裹 ≠ run 级粒度、段落族面爆炸，不推荐）；保真边界诚实登记：web 端到端 TODAY 不可达（MarkSpan URL + 行内链接解析 = mdcore MD 域两处暂缓项上游）；Internal 写回引擎结构性不可达（rels 发射面 external-only）登记维持
- T2.4 裁决点（STOP 如约）：**零接口变更实施**（红线 #1 遵守——公开模型面只在方案中呈现 API 形状）；T2.4 逃生条款核查：外链经既有 API 已全环（hyperlink_roundtrip_external 既有在档）无需补测试，本轮交付 = 探针 only；裁决问题三项（A vs 0 时机 / 书签写面同批否 / Internal+红线陈述项）落档 §13.4
- T2.5 门禁（全部亲跑最终源码态）：G1 **530/0/1i**（525+5 探针：docx_roundtrip 28→32 + mdc-wasm 61→62）；PDF 四面 68/79/81+6i/109+6i 零漂移 + perf pdfc_perf --ignored 6/6；clippy native 恰 2（static_method_of 基线）+ PDF 三面 0 + wasm32 --all-targets 2；fmt 0；scan 0 hit-paths；wasm32 check+--all-targets 过；Cargo.lock 零 diff；web 七套件 **18/15/28/11/13/9/9** 全绿（dist 未重建亦无需——本轮改动全在 #[cfg(test)] 面，wasm 产物位元不变）；途中事故如实登记：根盘 100% 满（target/debug/incremental 1.7G）致 PDF clippy 建指纹失败——清 incremental 缓存（纯缓存零源影响）后全门禁复验绿
- 文档三面：CAPABILITY_NOTES §13（矩阵/缺口/选项/推荐/裁决点/探针清单/门禁——评估单一真源）+ FIDELITY §3 anchor 登记行闭项留痕（指向 §13，矩阵口径不变零实施）+ FIDELITY 矩阵行 6 证据列补 r5-t2 探针 + 本节

Stage Summary:
- r5-t2 完成：锚点写回评估收口——读面四变体/引擎写面/书签名面全部一手实证（5 探针），方案 Option A/B/C/0 落档 CAPABILITY_NOTES §13，**公开模型零实施**（r4-t3「另行裁决」正式呈报主代理转用户）
- mdcore 2 文件 +333/-0（office-io tests/docx_roundtrip.rs +4 探针 / mdcore-wasm docx_export.rs #[cfg(test)] +1 探针）——tests-only 零接口变更；不 push（主代理审查后提交）
- 裁决问题三项待用户：①office-io 锚点写面现在扩（A）vs 待 MarkSpan URL 裁决后一并做（0）②若 A：bookmark_paragraph 书签写面同批/分离/不做 ③Internal 写回引擎结构性不可达 + HYPERLINK 域红线 + mdcore MD 域暂缓项（陈述性维持）
- 关键去风险证据：探针⑤ 引擎直构整包实证（锚点零 rels 面/书签对/引擎写→自有读全环）——若裁决选 A，实施面 = 纯包装层胶水（无 t20 numbering 式接线陷阱、无 t14 域崩溃面）
---
Task ID: r5-t2-review
Agent: 主代理（r5-t2 独立审查）
Task: Round 5 T2 超链接锚点写回评估——四维独立审查

Work Log:
- 提交核实：mdcore 恰 2 文件全 tests-only（office-io/tests/docx_roundtrip.rs +295[四探针]、mdcore-wasm/docx_export.rs +38[cfg(test) 探针④——mod tests 内确认]）；产品代码零改动 = 公开模型未扩（红线遵守实证）；Cargo.lock 零 diff
- 边界核实：探针⑤ 引擎直构 spike 断言集精查（w:anchor 属性形 + rels 自含性[锚点零 rels 面，外链对照恰 1] + 书签 Start/End 对 + 引擎写→自有读全环）= Option A 真实去风险证据非纸面宣称；HYPERLINK 域 todo!() 红线未触（w:anchor 属性形路径不相交——核实）；MarkSpan URL/富文本暂缓项零触碰
- 测试亲跑复验：G1 全量 **530/0/1i**（525+5，五探针逐一过）；clippy native 恰 2 + wasm32 --all-targets 2 + PDF 三面 0；fmt 0；scan 0；lock 零 diff；web docx_import 11/11 + docx_export 13/13（dist 未动合理——改动全 cfg(test)）
- 文档核实：CAPABILITY_NOTES §13 承载矩阵 14 行全带锚点（RunSpec External-only/DocxWriter 19 方法零锚点书签/读面四变体/引擎写面 Anchor+书签在/HYPERLINK 域红线/全链各跳现状）+ §13.2 缺口分析（读 4/写 1 对称缺口 + 书签落点正交性 + web 端到端 TODAY 不可达诚实边界）+ §13.3 选项 0/A/B/C（A 加性非破坏循 r4-t1/r4-t3 先例、grep 全消费者核实、B 破坏性否决、C 粒度错配否决）+ §13.4 裁决三问；FIDELITY §3 anchor 行闭项留痕（评估落档指针）+ 矩阵行 6 补注
- 纪律核对：T2.4 STOP 点遵守——RunSpec 扩字段未实施，裁决问题成文呈用户（本轮汇报转呈）

Stage Summary:
- r5-t2 审查通过零返工：评估交付 = 5 只增探针（tests-only）+ §13 方案与裁决点；公开模型面零变更
- mdcore 提交推送（新基线 530/0/1i）；分析仓 CAPABILITY_NOTES §13 + FIDELITY + worklog 提交推送
- 裁决问题待用户：①office-io 锚点写面 A[最小加性扩] vs 0[维持登记待 MarkSpan URL 一并做] ②若 A 书签写面同批/分离/不做 ③Internal 引擎不可达/HYPERLINK 域红线维持（陈述性）
- T3（集成与收口）即刻执行
---
Task ID: r5-t3
Agent: 主代理（Round 5 T3 集成与收口）
Task: 端到端收口——文档链同步（能力矩阵/保真/夹具说明/HANDOFF）+ 全门禁快照 + 终审 + 双仓终态

Work Log:
- 文档链同步：REPORT.md r5 追记（§9 历史表 numPr/ind 行翻转留痕指向 FIDELITY 单一真源 + r5-t1 列表映射增量 + r5-t2 评估指针 + 夹具 26,625→31,733 B 说明）；HANDOFF L-MD 行（main 7055595→00ad88b、链尾 +a5c314b[G1 525]/00ad88b[G1 530] 两节点、九门禁 517→530、web 七套件 docx_import 9→11）+ L3 行 Round 5 段（t1 列表映射全语义/t2 评估与裁决点/t3 收口，两轮 lock 零 diff）
- 全门禁快照（主代理亲跑，当前 HEAD 一次完整快照）：G1 **530/0/1i**；PDF 四面 68/79/81+6i/109+6i 零漂移 + perf 6/6；clippy native 恰 2（wasm_bindgen 基线）+ PDF 三面 0 + wasm32 --all-targets 2；fmt 0；scan 0 hit-paths；wasm32 check+--all-targets 过；Cargo.lock 零 diff；web 七套件 **18/15/28/11/13/9/9 全绿**（本轮补跑 contract/smoke/ime/xlsx×2 五套件与 t2 审查时点 docx 两套件合并成完整快照；smoke/ime 经 results JSON failed=0 核验）
- 终审核对：①代码差异——Round 5 两轮 mdcore 恰 6 文件（t1 产品面 4：docx_import.rs+main.ts+docx_import.py+夹具；t2 tests-only 2），office-io 产品代码与 mdcore crate 零改动；②依赖——Cargo.lock 两轮零 diff 实证，NOTICES v1.5 恒（零新三方依赖）；③文档声明——FIDELITY §5/CAPABILITY_NOTES §13/CAPABILITIES/REPORT 追记/HANDOFF 全部与代码对齐，未实测不宣称零违例（t2 web 端到端不可达边界、t1 非连续续号降级、复合往返不宣称均显式登记）；④裁决纪律——RunSpec 公开模型扩面未实施（t2 STOP 点遵守），裁决三问成文
- Round 5 执行纪律核对：先 T1 后 T2 未重复派发 ✓；两子代理均一次通过零返工（审查通过即推送：a5c314b/00ad88b + dda71e1/9854e29）✓；暂缓项零触碰（富文本/MarkSpan URL/CRDT/融合——t2 评估引用为上游上下文未实施）✓；无裁决外等待（连续执行 T0→T3）✓

Stage Summary:
- Round 5 三轮全部收口：DOCX 列表导入映射落地（FIDELITY §3 登记缺口闭项——列表段从降级普通段升级结构化映射，e2e 证列表达 BlockTree 终端消费面）+ 超链接锚点写回评估交付（tests-only 五探针 + §13 方案，公开模型零变更，裁决三问呈用户）
- 新基线：G1 **530/0/1i**、web 七套件 18/15/28/**11**/13/9/9、PDF 四面与 clippy/fmt/scan/wasm32 全基线恒、NOTICES v1.5 恒
- 待用户裁决：①office-io 锚点写面 Option A[RunSpec+anchor 加性扩] vs 0[维持登记待 MarkSpan URL 一并做] ②若 A 书签写面 bookmark_paragraph 同批/分离/不做 ③（陈述性）Internal 写回引擎不可达/HYPERLINK 域红线/MarkSpan URL 暂缓——维持登记
- 双仓终态：mdcore 00ad88b = 远端零漂移工作区 clean；分析仓本节提交推送后核对
---
Task ID: yrs-t0..t4（yrs 评估 pass 全轮，L3 线后续轮——06 §8.2 建议 1 执行）
Agent: 主代理
Task: 按 pdf-line/06 §8.2 建议 1 + HANDOFF §6 ⑤（yrs 评估 pass 最优先）+ 用户"继续"
指令（2026-10-16，凭据随消息传递），执行 D1② 证据轮：上游一手核验 / 协议与 API
深读 / spike 六探针 / 闭包许可 / 评估报告。证据先行零终裁，mdcore 零接触。

Work Log:
- **yrs-t0 环境与基线**：第 11 次平台重置恢复——分析仓本地 dd7fb1a 陈旧 + 45 文件
  纯模式位变更，fetch 核验远端 1c6066a（r5-t3 收口态，含此前多会话 office 线
  ~40 提交）为唯一真源 → reset --hard 零内容损失；mdcore 重克隆 main=00ad88b
  （depth 30，PAT extraheader 单次传递，.git/config 零残留复核）；**核心 crate
  输入有效性钉定**：git log 03815b6..HEAD -- crates/mdcore/src/ 空输出（EXIT=0）
  ——boundary-audit p1-p5 op 模型锚点对当前 main 完全有效；rustup 1.99.0 前台
  重装（后台两次被平台清理）；node v24.21.0 幸存；凭据双仓 ls-remote 验证通过。
- **yrs-t1 上游一手核验**：yjs/y-crdt 克隆（真实仓 = y-crdt/y-crdt，API 重定向
  定谳；全量历史 1237 提交意外获得）；LICENSE 逐字直读 = 标准 MIT（Copyright
  2020 Kevin Jahns【Yjs 原作者】+ Bartosz Sypytkowski；GitHub API license 字段
  NOASSERTION = 分类失败怪癖，登记）；crates.io：yrs 0.28.0（2026-09-17，82 版
  线）总下载 4,131,530 / 近期 2,122,625；提交节奏 2026 年 138（近 90 天 43）；
  **AppFlowy-Cloud Cargo.lock 实证 yrs 0.23.5 生产采用**（raw 直读）；语言绑定
  家族 7 件；**lib0 已吸收进树内**（yrs/src/encoding，README 组件表述陈旧——
  勘误登记）；**开放问题面 API 逐条核验**：#676（0.28.0 现行版 heap use-after-
  free：并发 undo+map remove+默认 GC，2026-09-29 新档，直接命中 undo/encode_diff
  组合面）/ #373（堆损坏，2024 陈化）/ #445（内存泄漏，疑似 §发现2 已知面）/
  #458/#518。crates.io API 为本网关受限镜像（无 license 字段）——闭包许可转
  cargo metadata 路线。
- **yrs-t2 协议与 API 深读**：sync 模块（feature 门控）消息常量与 y-protocols
  精确一致（MSG_SYNC=0/AWARENESS=1/AUTH=2/QUERY_AWARENESS=3 +
  SyncStep1/2/Update=0/1/2，protocol.rs:347-433）；awareness LWW+clock+观察者；
  update v1/v2 双格式 + merge/merge_updates（服务端零实例合并）+ encode_diff
  （SV 差量）；TransactionMut：origin 事务溯源 + before/after_state + insert/
  delete_set + encode_update_v1/v2 + apply_update；UndoManager：expand_scope/
  include_origin/exclude_origin/undo_blocking/redo_blocking + capture_timeout_
  millis 捕获窗；StickyIndex 稳定位置锚；DocPrelim 不存在（0.28 API 演进为
  In::Doc）；四类型根方法均为 trait 方法（Text/Map/Array/XmlFragment/GetString/
  ReadTxn/WriteTxn/Encode/Decode/ToJson——探针开发期逐项校准）。
- **yrs-t3 spike 六探针**（collab-line/spike，分析仓工作区零进 mdcore；依赖仅
  yrs=0.28 + npm yjs 13.6.33）：
  * P1 基本面过：四类型根 + v1/v2 往返 + SV 差量（空=全量/自≈空）+ 子文档 guid 占位；
  * P2 双端收敛过 + **发现 1**：并发冲突交换后状态收敛等值 + 幂等复交换 + 三方
    SV 差量同步；但全量 update 编码字节非确定（121B vs 117B——块布局随整合历史
    异形）→ 支持权威快照单点编码（D1③ 方向）；
  * P3 互操作过（杀手级证据）：yrs 0.28 ↔ yjs 13.6.33 双向 wire 互证——Rust
    全量态 JS 精确重建 → JS 编辑回写 → Rust 精确重建（逐字节匹配）+ 反向 SV 差量；
  * P4 origin 分栈过：human/ai 事务分离追踪，undo 仅回退 human 面 ×3、ai 隔离
    不可 undo、redo 复原（= mdcore Source 溯源分栈的 yrs 对应物实证）；
  * P5 装载模式对比过 + **发现 2（重量级）**：单事务批量小串追加 commit 期内存
    二次放大（10k×8B=80KB 文本 → +415MB；20k → +1,581MB，比率 3.81≈4×；100k
    外推 >4GB 两次实测 OOM kill）；机制锚点 commit 第 7 步 squash_left
    （block_store.rs:222）→ try_squash（block.rs:2053），算术核验 8B×10k²/2=
    400MB≈415MB；array 批量同受影响更重（10k → 全程峰值 1.49GB）；**缓解面
    实测双零成本**：8KB 大块单事务（0.2ms/+0kB）与分事务（118ms/+0kB）；
  * P6 官方编辑轨迹基准 5/5 过（release 本机沙箱口径）：friendsforever 30ms /
    sveltecomponent 320ms / rustcode 1633ms / seph-blog1 21,277ms（137k 事务）/
    automerge-paper 25,814ms（260k 事务，逐键粒度 ≈99μs/键）；
  * **API 使用纪律发现**：async-lock 非重入——活跃写事务内 get_or_insert_* 或
    同语句读写双锁 → 死锁挂起（探针开发期两处实测定谳；纪律 = 引用先取/事务后
    开/读写分语句）；
  * 闭包许可（cargo metadata 45 包逐件）：32× MIT OR Apache-2.0 + 6× Apache
    OR MIT + 4× MIT + Unlicense OR MIT + unicode-ident（AND Unicode-3.0，office
    线 NOTICES 同例）+ r-efi（OR 集内 LGPL 可避）——**零 GPL/AGPL 零强制
    copyleft**。
- **yrs-t4 收口**：yrs-a_yrs评估pass.md 落档（对象锚定/成熟度与开放问题面/协议
  兼容实证/六探针含两发现/闭包许可/op 映射八行矩阵——四向核心面映射可行且溯源
  与事件面净收益、revision 与语义序列化需适配层、结构载体映射留实施轮/对 D1②
  证据输入双向清单/遗留五项/已核带源清单/执行纪律对账含全部过程修正）；
  HANDOFF L3 行 + §6 速览 + README L3 段（原"待启动"严重滞后，收口级同步）+
  根 .gitignore（collab-line/_upstream/）+ spike/.gitignore（target/out/
  node_modules）。

Stage Summary:
- yrs 评估 pass 完整交付：D1② 裁决的证据输入就绪——(a) 路线实证支撑（双语言
  wire 互证/MIT+闭包零 copyleft/溯源分栈映射/事件面净收益/活跃维护）与风险清单
  （批量装载内存放大【有零成本缓解】/#676 现行版 UAF/编码非确定/死锁纪律）成对
  在档；(b) 自研路线获得规格参照基线。零终裁（D1①②③ 全留用户）。
- 两大原始发现：编码字节非确定性（支持权威快照设计）；单事务批量装载 commit 期
  二次内存放大（机制锚点+算术核验+双零成本缓解面——若走 (a) 须立装载纪律门禁）。
- mdcore 零改动零提交（全程只读）；分析仓本节提交（yrs-a 报告 + spike 六探针
  源码 + Cargo.lock/package-lock 钉版 + worklog + HANDOFF/README 同步）。
- 会话五轮（yrs-t0..t4）满足"至少 5 轮"；此后所有剩余缺口带用户裁决前置
  （D1/D2/D3 + R-O2 + 锚点三问 + R-1..R-6 + 裁决前置清单①②④⑦）——阻塞呈报。

---

Task ID: d1b-t1
Agent: 主代理（cleanroom 线）
Task: D1-a 集成边界证据轮——#676 状态复查 + GC/undo/快照 API 面锚点 + P7 探针
（ASan 确定性复现与缓解矩阵 + skip_gc 代价定量）。D1 用户裁决 a（yrs 候选内核
+ 隔离适配层 + 可撤回选型）后的第一执行轮；四项验证边界文档（d1b-t2）的证据前置。

Work Log:
- 环境恢复（第 11 次平台重置）：双仓本地 = 远程（de186f7 / 00ad88b main）；PAT
  经 credential store 配置（本网关 Basic 口径）；rustup 1.29.1 + Rust 1.99.0 钉住
  channel 重装（与历次恢复同链）+ nightly 1.101.0（2026-10-15 构建，ASan 探针用）。
- #676 状态复查（GitHub API 一手）：**仍 open、零评论、零修复**（2026-09-29 报告，
  更新时间 = 创建时间）；上游自评估基线 ac33960（2026-09-30）后仅 2 提交（均
  ywasm 面，ac33960 本身即其一）——**UAF 在当前 HEAD 仍在**。issue 全文落
  tool-results/issue676_full.txt：复现形状逐字可得 + 上报者口径「skip_gc: true
  makes it clean」+ ASan 栈（slice.rs:210 ItemSlice::encode ← store.rs:244/212）。
- 上游源码锚点补强（_upstream/yrs 本地克隆，零新克隆）：
  * skip_gc = Doc Options 字段（doc.rs:492，默认 false = GC 开）；commit 期 GC
    执行点 = transaction.rs:1111（step 5 `if !self.store.skip_gc { GCCollector::collect }`）；
  * **显式压缩 API**：TransactionMut::gc(Option<&IdSet>)（transaction.rs:1357，
    「even if a document was created with skip_gc」——受控压缩点存在性钉定）；
  * **快照编码硬门**：Store::encode_state_from_snapshot 在 !skip_gc 时返回
    Err(Error::Gc)（store.rs:152-157）——**skip_gc 与 D1③ 权威快照同向**；
  * 上游自有测试 nested_undo（yjs#317 移植）与 undo_block_bug（yjs#343 移植）
    **均设 skip_gc: true**（undo.rs:1457/1665）——上游测试面隐性承认 undo+嵌套
    map+GC 是危险组合；
  * StickyIndex API 面（sticky_index.rs）：from_id/get_offset/Encode/Decode/
    Serialize/Deserialize——锚点可 wire 化可持久化，「不受文档变更影响」语义。
- spike P7 探针（src/bin/p7_gc_undo_safety.rs，新增）：#676 形状逐字移植
  （peer/insert/set_color/remove/sync 五函数对齐上报者代码；捕获窗 0 + 恒 0
  timestamp 关闭时间合并面）。**Cargo.toml 开启 features=["sync"] 对齐 #676
  上报口径**；Cargo.lock 零 diff（sync 只门控同 crate 内代码，闭包仍 45 包——
  许可结论不受影响）；p1-p5 + p3_interop_read 全部复跑 PASS（feature 开启
  零回归）。
- **P7a（ASan 确定性复现）**：nightly + `-Zsanitizer=address`，MODE=asan 默认
  GC 单发 → **heap-use-after-free 首发命中**（READ of size 8，216B region，
  EXIT=1）——与上游报告逐点一致，UAF 在 0.28.0 本机定谳。
- **P7b 缓解矩阵（ASan + release 双口径）**：
  * b1 双端 skip_gc：ASan 干净 + 收敛（EXIT=0）——缓解实证；
  * b2 混合模式（本轮重大发现）：**release 收敛但 ASan 口径 MIXED=1
    （a=skip_gc,b=GC）仍确定性 UAF**——「缓解可单端部署」被证伪；MIXED=2
    （a=GC,b=skip_gc）本形状干净（触发条件 = 接收远端删除 + 本地 undo 栈 +
    GC 的组合，任何 peer 都可能进入 b 角色）→ **skip_gc 须全编队一致
    （fleet-wide invariant），适配层单点构造强制**；
  * b3 快照编码门：skip_gc → Ok / 默认 GC → Err(Error::Gc) 双断言钉定；
  * b4 显式压缩点：clear_all 后 txn.gc(None) 可用不 panic，压缩后全量同步
    收敛（55B）。
- **方法论发现登记**：release 口径的收敛断言对 #676 类内存安全**零证实力**
  （MIXED=1 release 收敛而 ASan 崩溃）——边界文档须把 ASan 门禁定为该类
  面的验收口径。
- **P7c skip_gc 代价定量**（CYCLES=20000 轮嵌套 map 插入+删除）：
  * 默认 GC：终态 RSS +7,376 kB / 全量 update 348,903 B；
  * skip_gc：终态 RSS +26,536 kB（**3.6×**）/ 全量 update 817,893 B（**2.34×**，
    未 GC 的删除块保留内容并被编码）；
  * 显式 gc(None) 后：update 尺寸 348,903 B（**精确回到 GC 基线**——压缩对
    wire 尺寸是硬恢复）；RSS +27,680 kB（分配器保持策略，无 OS 侧回落——
    诚实登记：RSS 降幅不可作为压缩收益指标，update 尺寸可以）。

Stage Summary:
- #676 定谳链闭环：仍 open 零修复 → 本机 ASan 确定性复现 → 双端 skip_gc
  干净 → 混合部署被证伪（全编队不变量）→ 显式 gc() 压缩点存在且有效。
- 四项验证边界的证据面就绪：①GC/UAF 隔离（skip_gc 全编队 + 显式压缩点 +
  ASan 门禁口径）②事务粒度（yrs-a §4.5 已有）③撤销栈归属（clear_all 前置 +
  P4 分栈）④稳定锚点（StickyIndex wire 化）——d1b-t2 边界文档可直接引用。
- spike 面零污染：Cargo.lock 零 diff、p1-p5 零回归、新探针独立成文件；
  mdcore 零接触零提交。

---

Task ID: d1b-t2
Agent: 主代理（cleanroom 线）
Task: D1-a 集成验证边界文档（四项：并发 GC/UAF 隔离、事务粒度、撤销栈归属、
稳定锚点跨模型映射）——用户裁决 D1② 走 a 后的主交付物。

Work Log:
- 落档 `collab-line/yrs-b_D1a集成验证边界.md`（四项边界 + 适配层契约草案 +
  实施轮启动条件 + 撤回触发），证据全部引 yrs-a（评估 pass）与 d1b-t1
  （P7 探针 + 上游锚点）在档事实，零新宣称。
- 边界一（GC/UAF 隔离）：B1.1 skip_gc 全编队不变量（单点构造强制，依据 =
  P7b2 混合部署 ASan 证伪）/ B1.2 构造唯一入口（lint disallowed_method）/
  B1.3 显式压缩点两类受控位（clear_all 后 + 权威快照维护窗）/ B1.4 ASan
  门禁为该类面唯一验收口径（release 收敛零证实力——方法论发现升格为规则）/
  B1.5 上游修复跟踪节点 / B1.6 panic 隔离 + 限额外裹（mdcore 消毒层既有
  模式复用）；代价面诚实登记（RSS 3.6×/update 2.34× + 压缩后 update 精确
  回基线，RSS 降幅不可作指标）。
- 边界二（事务粒度）：B2.1 装载纪律双入口（load_bulk 大块化 / edit_stream
  分事务——A2/A3 双零成本缓解为依据，禁裸长事务）/ B2.2 AI 生成态直连
  load_bulk / B2.3 事务边界 = op 语义边界（装载纪律 × 撤销粒度 × origin
  归类三点同决）/ B2.4 死锁纪律适配层内消化 / B2.5 P5 三模式入回归（A1
  形状必须被拒或自动切分）。
- 边界三（撤销栈归属）：归属规则建议 = 适配层会话对象（每会话 × 每 origin
  分栈），mdcore History 协同域退役为兼容投影（本地单机维持现状——两模式
  并存的可撤回设计）；B3.1 栈永不序列化进 update / B3.2 Source::Human/Ai ↔
  Origin 字节串一一映射（未归类保守入 Human）+ 捕获窗对齐 1s / B3.3 压缩
  前置 clear_all + 客户端不压缩（权威端单点）/ B3.4 栈上限预留参数（R-3
  产品裁决联动）/ B3.5 P4+P7b4 入回归。撤销语义变更面（meta 复原差异）显式
  登记为实施轮用户知情确认项。
- 边界四（稳定锚点跨模型映射）：R4.1 本位币 = CRDT ID/StickyIndex（三模型
  语义锚互不可解析 + ID 唯一天然稳定可 wire 化 + 可撤回性三依据）/ R4.2
  入向映射 bind()（语义锚 + ID 成对入适配层自有侧车，不进三方格式）/
  R4.3 出向解析 resolve()（get_offset → 结构位置 → 各线语义锚重建；失败面
  显式化三线统一）/ R4.4 mdcore 特例（BlockId 跨会话保真由映射表承接，
  G4b 撞号面协同域消解、本地域维持 R-2 锁）/ R4.5 Office 书签落点依赖 r5-t2
  裁决三问结果（不预设）。
- 适配层契约草案六面（构造/装载/同步/撤销/锚点/快照+事件）+ 隔离不变量
  三条（公开 API 零 yrs 类型 / mdcore 公开模型零改动 / B 规则全部
  fail-fast 不依赖 review）。
- §7 实施轮启动条件四项（边界用户确认 / #676 启动日复查[已做：仍 open] /
  结构载体选型裁决 / B3.2 知情确认）+ 撤回触发三条（上游 6 个月不修且同类
  涌现 / 破坏性变更不可吸收 / 正式基准低于需求无缓解）。
- HANDOFF 双处同步（L3 行 d1b 两轮 + §6 速览重写：D1 实施轮待边界确认后
  启动，D2/D3 可裁决）。

Stage Summary:
- D1-a 四项验证边界闭环：每项 = 事实链（一手锚点）→ 边界规则（B 编号，
  可检收）→ 验收门禁（可执行）；适配层 = 唯一 yrs 接触点 + 可撤回边界的
  工程化定义。
- 两处与在档裁决的联动显式化：R4.5（锚点映射依赖 r5-t2 三问结果）与
  B3.4（栈上限与 R-3 联动）——跨线冲突面提前登记不预设。
- mdcore 零接触零提交维持；呈用户确认边界文档后开实施轮（启动条件四项）。

---

Task ID: r-o2-t1
Agent: 主代理（office 线）
Task: R-O2 docx 渲染级读侧独立评估（ADR-O1 O1-6 开放项）——生态盘点
一手核验 + LO 转换路线 spike 实证 + 评估文档落档。用户指令：独立评估，
不混入协同内核改造（D1 线零交互遵守）。

Work Log:
- 需求锚定：office-d §6.3 目标线三要素操作化（结构忠实 = 在场可辨不静默
  丢弃；显式分页 = 显式标记真实页边界可断言、lastRenderedPageBreak 不
  采信；按需渲染 = 首屏不随总页数线性劣化）；消费场景三分类登记。
- 生态盘点（全一手，2026-10-17）：
  * docx-preview：Apache-2.0（repo+npm 双确认）/2121★/活跃（0.4.1
    2026-09-21 发版）——能力上限转引 office-d §6.1 在档锚点（容器
    裁剪分页/wrapSquare 静默丢弃/全量 DOM 无虚拟化）；
  * mammoth：BSD-2-Clause/1.13.0/2026-09-26——语义 HTML 无分页，
    生态位被自有 t16 链覆盖，排除；
  * LibreOffice：上游稳定 26.8.1（2026-09-21，官网双页交叉）；仓 =
    三许可并存（根 COPYING GPL-3.0 + COPYING.LGPL + COPYING.MPL 逐件
    直读——GitHub API 检 GPL-3.0 的原因钉定）；**许可判定 = 子进程
    形态走 ComfyUI 进程隔离先例（canvas-d），嵌入/UNO 形态才入 MPL
    义务面**；
  * Univer OSS：Apache-2.0/22.6k★/极活跃，docs 引擎族 11 包在
    （engine-render 含 shaping/linebreaking = 自有排版栈）但 **docx
    导入器 0 命中**（importer/docxImporter/importDocx 三搜索式 + 30
    处 docx 提及全为内部布局代码）——导入 = Pro 面；判定 = 采用其
    渲染需自建导入桥 = 变相自研；其排版栈源码可作 B 路线算法参照；
  * SuperDoc（AGPL-3.0 npm 一手）/OnlyOffice x2t（office-b §3 在档）/
    云查看器（local-first 冲突）/pandoc（GPL+生态位覆盖双理由）——
    四件排除登记；
  * crates.io "docx" 搜索 10 件全列：**全模型级/模板级，零渲染级
    crate**——Rust 供给侧空位实证。
- **C 路线 spike**（office-line/spike-ro2/lo_convert_spike.py，python
  stdlib 手搓 OOXML 零三方依赖 + soffice 子进程 + poppler 校验）：
  * A 结构样本（Heading1/2 + 粗斜混排 + 2×3 表 + 1 分页符）：8 关键串
    全在场 + 页数恰 2 ——全过；
  * B 分页样本（5 显式分页符）：**页数恰 6 硬断言过**——显式分页保真
    定谳（要素②）；
  * C 真实夹具（mdcore t16 import_sample.docx 只读复制）：转换过
    （1 页/166 字符抽取）；
  * 延迟：冷启动 1,024 ms / 热态中位 304 ms（小文档口径登记，大文档
    待实施轮）。
- 评估文档落档 `office-line/office-f_docx渲染级读侧评估pass.md`：候选
  矩阵（三要素 × 许可 × 集成形态）+ 三路线总对照（C：LO→PDF→mdc-pdf
  零新 crate 三要素即时满足 / A'：docx-preview 过渡件 / B：Rust 自研
  生态空位）+ 裁决问题三组（Q1 立项与否 / Q2 路线与时机含组合序
  1→3、2 备援 / Q3 C 实施形态三分）+ 与 D3/锚点三问/PDF 线联动登记 +
  遗留五项 + 已核带源清单。
- 与 O1-6 原问的差异显式化：原问只有「JS 参照件过渡 vs 直接 Rust
  自研」两肢，本 pass 新增 C 路线选项（依据 = mdc-pdf 阅读面在产 +
  spike 实证）——差异在文档附录声明。

Stage Summary:
- O1-6 的证据面从「空位登记」推进到「三路线可裁决」：开源生态无单点
  全满足件定谳；C 路线（转换 + 资产复用）spike 全过 = 三要素即时
  满足的起步线候选（推荐序登记，终裁留用户）。
- mdcore 零接触零提交（只读复制一件二进制夹具）；D1/collab 线零交互。
- spike 可复跑（脚本 + .gitignore 排除 out/ 产物）；LO 版本差
  （本机 25.2.3.2 vs 上游 26.8.1）与敌意面/大文档口径登记为实施轮项。

---

Task ID: md-r1r6-t1
Agent: 主代理（md 线）
Task: MD 线 R-1..R-6 逐项处理（用户指令「遇产品语义或真机依赖即停」）
——逐项判定 + 本会话跨线新证据并入 + 收敛裁决问题呈报。mdcore 零接触。

Work Log:
- 逐项现状锚点亲核（mdcore main=00ad88b 只读）：R-1 settings 六锚点
  （lib.rs:45/76/83/302/378/463）+ history 硬编码面（:20/:96，History
  无窗字段）+ 消费面为零（COALESCE_MS 全仓引用仅自身+文档+常量锁）+
  **REUSE.md §R5 行显式裁决门原文钉定（「接线属产品裁决」）**；R-2
  G4b 现状锁（t3_reuse_locks.rs:135）+ 注册表覆盖无守卫锚；R-3
  Vec<Entry> 无容量面（history.rs:56-57/102-106）。
- 逐项判定：R-1 停-产品语义（REUSE.md 裁决门优先于本会话推进意愿）/
  R-2 停-产品语义（三选项行为面各异）/ R-3 停-产品语义（显式产品
  决策）/ R-4 无事可做（维持登记即目标态）/ R-5 维持延后（前置未变）/
  R-6 停-真机依赖（既有排期）——**六项零可机械推进项，「机械增量
  清空」状态维持**。
- 本会话新证据并入（yrs-b 跨线联动）：R-2 裁决范围收窄（**协同域撞号
  面被 yrs-b R4.4 消解**——CRDT ID 永不重编，守卫/持久化只剩本地域
  价值，若本地维持现状可整体关闭）；R-3 应与 yrs-b B3.4 协同裁决
  （同一栈上限产品语义在本地/协同两域的镜像参数，单一裁决点双域
  生效）；R-1 若接线应预留 B3.2 捕获窗同源镜像（同一设置键驱动
  两域）。
- 落档 `md-line/reuse-impl/R1-R6逐项处理.md`：处理结果总表 + 逐项
  记录（现状锚点/处理/裁决输入补充）+ 收敛三组裁决问题（Q-R1 接线
  A/B【推断推荐 A：schema plumbing 完整即原设计意图 + 默认行为零变化】/
  Q-R2 a/b/c【推荐 c：协同域已覆盖】/ Q-R3 与 B3.4 协同【推荐维持
  无限至产品信号】）+ 跨线联动登记 + 已核带源清单。
- 接线形状预研（若 Q-R1 裁 A 的实施细节，登记不实施）：History 加
  coalesce_ms 字段（serde default=COALESCE_MS + skip_serializing =
  持久化格式零变化）+ Core::set_undo_coalesce_ms + wasm 导出 +
  main.ts 三核心创建位（:1331/:1365/:1408）应用 + mdc-settings-changed
  活应用 + shell 常量锁扩展——本轮已定位全部锚点，实施轮直用。

Stage Summary:
- R-1..R-6「待逐项处理」状态闭环：三组裁决问题收敛呈报（含跨线协同
  裁决建议），R-4/R-5/R-6 维持既有状态——MD 线当前无未处理的登记项。
- 与 yrs-b 的三处联动（B3.2/B3.4/R4.4）双向互引闭环——用户单次裁决
  可双线生效。
- mdcore 零接触零提交；全部锚点本轮亲核在档。

---

Task ID: sess5-closeout
Agent: 主代理（cleanroom 线）
Task: 本会话（用户指令"继续做不要随便停下至少做五轮"）收口轮——HANDOFF/
README 全量同步 + 双仓状态核实 + 最终推送 + 阻塞呈报更新。

Work Log:
- 会话五轮执行：d1b-t1（#676 定谳链 + P7 探针）→ d1b-t2（D1-a 集成验证
  边界文档）→ r-o2-t1（docx 渲染级读侧评估 + LO spike）→ md-r1r6-t1
  （R-1..R-6 逐项处理）→ 本收口轮——满足"至少五轮"。
- HANDOFF 三处同步：L3 行追记（D1-a 两轮 + R-O2 评估 + MD 线逐项处理，
  office-line 成果清单 4→5 研究报告 + spike×2，报告计数 14→15）；
  §6 速览重写（④ R-O2 完成、⑤ MD 线处理完成、其余门控项维持）；README
  L3 段同步（yrs-b + office-f 两行新增）。
- 双仓状态核实：cleanroom-analysis 本地五提交（baaceba d1b-t1 →
  d1b-t2 → r-o2-t1 → md-r1r6-t1 → 本轮）待推送后与远程一致；mdcore
  main=00ad88b 全程零接触零提交（D1-a 可撤回选型纪律 + REUSE.md 裁决门
  纪律双遵守）。
- 环境工具链恢复登记（第 11 次平台重置口径）：rustup 1.29.1 + Rust
  1.99.0（钉住 channel，与历次同链）+ nightly 1.101.0（P7 ASan 专用）；
  spike p1-p5 复跑全过 = 恢复验证闭环。

Stage Summary:
- 本会话净交付四件：yrs-b 边界文档（D1-a 四项验证边界 + 适配层契约
  草案 + 启动条件/撤回触发）、P7 探针（#676 ASan 定谳链 + skip_gc
  全编队不变量 + 代价定量）、office-f 评估（O1-6 证据闭环 + C 路线
  spike 全过）、R1-R6 逐项处理（三组裁决问题收敛）。
- 阻塞呈报（全部带用户裁决前置，零无门控可推进项）：①D1 实施轮
  （边界文档确认 + 结构载体选型 + B3.2 知情确认）②D2/D3（06 §8.1）
  ③锚点写面三问（r5-t2 §13.4）④R-O2 三问（office-f §4.2 Q1/Q2/Q3）
  ⑤MD 线三问（Q-R1/R2/R3）⑥t22-review 裁决前置清单。
- mdcore 零接触；全部成果推送后双仓零未推送状态。

---

Task ID: d1c-t1
Agent: 主代理（collab 线）
Task: D1-a 适配层原型启动轮（用户"有条件确认边界"口径落地：允许可撤回
适配层原型、不批准全面集成；验证面 = 内存上限/撤销语义/跨模型锚点映射）。

Work Log:
- 前置核验（用户指令的必要检查）：远端 HEAD 复核 = 7746d96（sess5-closeout，
  本地落后两提交已拉齐）；yrs-b 边界文档四项边界与用户口径逐条对齐确认
  （B1 skip_gc 全编队 ↔ "skip_gc 必须全编队一致"；B2/B3/B4 ↔ 三验证面）；
  office-f C 路线 spike 与用户 R-O2 裁决对齐；#676 复查（yrs-b §7 启动
  条件②实施轮启动日复查）：仍 open/0 评论/0 修复，上游最新提交仍
  ac33960（ywasm 面）——B1.1 维持强制性。
- 环境恢复（第 12 次平台重置）：rustup + Rust 1.99.0 钉住（与历次同链）；
  spike p1/p2/p4/p5/p7（release 口径）复跑全过 = 恢复验证闭环；LO
  25.2.3.2/pdftotext/pdfinfo/node v24.21.0 在位（R-O2 轮备用）。
- 本地漂移定性：双仓大量 "M" 全部为文件模式位漂移（644→755，0
  insertions/0 deletions）——工作区恢复痕迹非内容漂移，双仓
  core.fileMode=false 静默后状态归零。
- **collab-adapter 原型 crate 落地**（spike workspace 化：根包 + adapter/
  成员；crate 边界 = 可撤回边界，yrs-b §6 隔离不变量 (a) 公开 API 零
  yrs 类型）：构造面 open()（B1.1 skip_gc 无条件 true + B1.2 单点构造，
  SessionOptions 刻意无 skip_gc 字段）；装载面 load_bulk_text（B2.1(a)
  8KiB 大块单事务）/load_blocks（B2.1(b) 512/批分事务）/edit_stream
  （逐笔事务 + origin 标注，B2.3 三同点入参面）——无裸长事务入口；
  同步面 state_vector/encode_diff/apply_update（B1.6 限额 + catch_unwind
  panic 隔离）；撤销面 undo/redo per-origin（B3.2 origin 常量表
  mdc:human/mdc:ai + 捕获窗 1000ms = COALESCE_MS 语义连续性；B3.4 栈
  上限参数预留）。interim 载体 = text 根 + blocks Array-of-Map 根（载体
  选型证据轮 d1c-t3 前的承载面，与装载/撤销/同步面正交）。
- **P8 探针全过**（src/bin/p8_adapter_load.rs，SCALE 可调默认 20k）：
  a1 skip_gc 外部可观测 true；a2 快照门 Ok（skip_gc 与权威快照同向）；
  b1 edit_stream 20k×8B 逐事务 ΔRSS +1828kB/476ms；b2 load_bulk_text
  160KB 大块 ΔRSS +768kB/2ms；b3 load_blocks 20k 块 ΔRSS +24092kB/
  5439ms（三路径全部远低断言上限——对照直接 yrs A1 形状 10k 笔即
  +415MB 在档）；b4 禁止形状 API 面不可表达；b5 输入限额 52MiB 生效；
  c1 origin 分栈（捕获窗合并 human×3 为单栈项——undo×1 全回退，ai 栈
  不被消耗，redo 复原）；d1 双适配器双向差量收敛（text 逐字 + blocks
  20000 项）；d2 敌意 update 解码层拒绝 + 引擎未毒化 + 超限拒绝。
- 门禁：cargo build 零警告；p1/p2/p4 旧探针复跑零回归；Cargo.lock +8
  行（仅 collab-adapter 自身条目，yrs 0.28.0 锁定不动）；mdcore 零接触。

Stage Summary:
- D1 用户三验证面之二（内存上限装载面 + 撤销语义冒烟）在适配层 API 上
  实证闭环；第三面（跨模型锚点映射）留 d1c-t3。
- 原型 crate 结构性满足隔离不变量 (a)：公开 API 零 yrs 类型（EditOrigin/
  BlockSpec/SessionOptions/AdapterError 全部本地类型）；B1.1/B1.2/B2.1
  全部构造期/API 面强制，不依赖调用方纪律。
- 遗留：undo_stack_limit 仅参数预留（逐出依赖 observe API 面验证 = yrs-b
  §9 待核项）；快照面完整语义 + 锚点面 + 载体选型证据在 d1c-t2/t3；
  敌意 update 的 panic 面（非解码层拒绝形状）需 ASan/深验补（d1c-t2）。

---

Task ID: d1c-t3
Agent: 主代理（collab 线）
Task: D1-a 适配层原型第三轮：锚点面 + 快照面（用户验证面之三
"跨模型锚点映射"闭环 + D1③ 权威快照）。

Work Log:
- 适配器 API 演进：StreamEdit 增 InsertBlockBefore（中段插入——锚点
  保持验证的结构性编辑面）；公开类型增 CrdtAnchor（R4.1 本位币）/
  SemanticAnchor（MdBlock·OfficeBookmark·PdfAnnotRef 三模型入向）/
  ResolvedAnchor + AdapterError 增 AnchorDangling/SidecarFormat 变体；
  锚点面方法 bind/resolve/anchor_of + 侧车 export/import（MDCA magic +
  版本 1 + 确定性排序字节稳定）；快照面 authoritative_snapshot（D1③
  单点编码）；探针面 __probe_sticky_wire_roundtrip。占位
  __probe_snapshot_gate 移除（被 authoritative_snapshot 取代，p8 调用点
  同步更新）。
- **P10 探针全过**：a1 MD 锚点保持——bind blk-00042（下标 42）→ 前插
  5 + 前删 3 → resolve 下标 44 且 block_id 复原（ID 稳定下标漂移——
  对照 mdcore BlockId 条件保真的跨会话漂移面被 CRDT ID 消解）；a2
  StickyIndex wire 往返（Encode→Decode→get_offset == 直接解算，yrs-b
  §5.3 验收第一项）；a3 目标删除面显式化（Err(AnchorDangling) 不静默
  返回邻居——R4.3 反例封堵，墓碑塌缩检测 = 位置块首 ID 与锚点比对）；
  b1 Office 书签弱绑定（bind 位 50 → 漂移后解析到当前位 50 的
  blk-00048——R4.5 维持现状公开 API 的位置语义，身份漂移诚实展示）；
  b2 PDF 标注登记面；c1 侧车往返（123B/4 条全复原/字节稳定）+ c2 敌意
  三面全拒（坏 magic/坏版本/截断）；d1 跨端锚点（A bind → 全量同步 →
  B resolve 同一 block_id——CRDT ID 跨进程稳定）；e1 权威快照恢复
  （9226B → 新会话 101 块逐块一致）；e2 快照→维护窗压缩→再快照
  （45780B→25476B 内容保真——B1.3 与 D1③ 同点）；e3 收敛双端全量编码
  字节一致（本形状未触发发现①——历史依赖面，诚实登记不外推）。
- 门禁：cargo build 零警告；p8/p9 回归全过；mdcore 零接触。

Stage Summary:
- 用户三验证面全部闭环：内存上限（d1c-t1 装载三路径 + 限额）、撤销
  语义（d1c-t2 栈本地/捕获窗/一笔装载/压缩前置）、跨模型锚点映射
  （本轮 ID 稳定/删除显式化/三模型入向/侧车/跨端）。
- yrs-b 四项边界全部有适配层 API 面与探针证据对应；D1③ 权威快照
  单点编码面落地。
- 遗留至 d1c-t4：结构载体选型证据（XmlFragment vs Array-of-Map——
  yrs-b §7 启动条件③）、隔离不变量终验（公开 API 零 yrs 符号面核验）、
  ASan 门禁（nightly 恢复 + P7/适配层形状）、原型报告 yrs-c。

---

Task ID: d1c-t4
Agent: 主代理（collab 线）
Task: D1-a 适配层原型第四轮（收口）：载体选型证据 + 隔离终验 + ASan
门禁 + 原型报告 yrs-c。

Work Log:
- **P11 载体对照探针**（XmlFragment vs Array-of-Map，同内容 1000 块同
  skip_gc 六维）：wire 128,927B vs 120,573B（XML +7% 扁平）；装载
  4.1ms/ΔRSS+1,580kB vs 3.8ms/+2,904kB；全量遍历 0.20ms vs 2.35ms
  （**XML 快 ~12×** children 迭代器 vs 索引 get 链）；中段插删一致；
  StickyIndex 双载体同保持（500→497）；树性表达 XML 原生（嵌套
  [li-1,li-2] + 行内混排 "文本<bold>粗体</bold>尾" 交织）vs AoM 扁平
  depth 字段单文本字段；嵌套 wire XML 反而更小（227B vs 289B）。
- **隔离不变量终验**：签名级静态检查——公开 fn 22 个 + pub 字段 8 个
  零 yrs 类型（解析器双检；探针面 5 个 __probe_* doc(hidden) 分离）。
- **ASan 门禁**（nightly 1.101.0 + -Zsanitizer=address）：P7 MODE=asan
  默认 GC **heap-use-after-free 复现**（#676 威胁模型维持）/ 
  asan_mitigated skip_gc 干净；**适配层 p8/p9/p10 三探针 ASan 构建全过
  零报告**（B1.4 验收满足）；p8 预算断言 ASan 口径加 MEM_BUDGET_MULT
  仪器乘数（插桩膨胀 ~36×@b1 方法论登记——预算断言 release 校准，
  安全检测=零报告不受影响）；release 回归恒（乘数默认 1 全过）。
- **原型报告 yrs-c** 落档：三验证面证据明细 + yrs-b 规则→API→探针
  对账表（22 条规则全对应）+ 载体六维证据与建议倾向（XmlFragment，
  理由 = BlockTree 近同构 + 行内混排原生表达 + 遍历快 12× + 嵌套
  wire 更小；代价 = 扁平 +7%——待完整映射矩阵随裁决呈报）+ 实施轮
  前置三裁决（载体选型/B3.2 知情确认/B3.4×R-3 协同）+ 遗留登记
  （含两项新增：OfficeBookmark 强绑定依赖锚点写面裁决联动、ASan
  常跑项常态化）。
- 门禁：p1/p2/p4/p8/p9/p10/p11 全绿；cargo build 零警告；mdcore
  零接触（四轮维持）。

Stage Summary:
- D1-a 适配层原型四轮全收口：用户三验证面（内存上限/撤销语义/跨模型
  锚点映射）+ 隔离不变量 + ASan 门禁全部实证闭环；yrs-b 22 条边界
  规则全部有 API 面与探针对应。
- B3.4（yrs-b §9 待核项）闭环：栈上限不可经公开 API 实现——呈
  B3.4×R-3 协同裁决。
- 实施轮输入就绪：三裁决呈用户（载体选型[建议倾向 XmlFragment]/
  B3.2 撤销语义变更知情确认/栈上限三选项）+ 设计文档工作面清单。
- 原型可撤回性维持：crate 边界即撤回边界，mdcore 四轮零提交。

---

Task ID: r-o2-t2
Agent: 主代理（office 线）
Task: R-O2 C 路线原型第二轮：复杂文档夹具扩展（用户裁决"补充复杂
文档夹具，验证分页、跨页表格、页眉页脚、分节、缺失字体和图片"执行轮）。

Work Log:
- 新探针 `spike-ro2/lo_convert_spike2.py`（stdlib 手搓 OOXML 六样本
  D-I + zlib/struct 生成最小 PNG + pdfinfo/pdftotext/pdfimages/
  pdffonts 四工具断言；承接第一轮 A/B/C）。
- **六样本全过**：D 跨页表格（80 行 2 页，首行页 1/末行页 2 跨页
  实证 + tblHeader 重复表头）；E 页眉页脚（header1/footer1 部件 +
  sectPr 引用，页眉页脚**逐页**在场断言）；F 分节（段内 sectPr +
  body 尾 sectPr 双处，逐页尺寸 595×842 → 842×595pt 钉定 + 文本
  落位）；G 缺失字体（TotallyMissingFont123 → 转换不失败 + 文本
  不丢 + pdffonts 无幻影嵌入，替换字体 NotoSerifSC/Carlito 在场）；
  H 图片（8×8 PNG media+rels+ct 接线，pdfimages 计数 1）；I 组合
  夹具（全要素 4 页，六要素关键串 + 图片 + 页眉逐页全过）。
- 三个工程发现登记：①**w:drawing 必须 run 级**（直接置于 w:p 下
  = 非法 OOXML，LO 静默丢弃不报错——C 路线敌意输入的消毒联动关注
  点 + 实施轮要素计数断言策略）；②pdftotext CJK/拉丁边界插空格
  （断言需归一化——消费侧文本比对口径）；③真件对照判别法（mdcore
  真实夹具图片存活 → 区分系统性丢弃 vs 手搓缺陷——夹具调试标准动作）。
- 保真边界诚实登记：要素级保真（六类不失败 + 文本层 + 结构断言），
  非视觉级（字体替换视觉近似/表格度量/色彩管理/first-even 槽位不
  主张）；系统性视觉校验维持实施轮必做。
- office-f 报告 §8 追记（结果矩阵 + 工程发现 + 保真边界 + Q3 细化
  输入两条）；第一轮 spike 回归全过（A/B/C 热中位 296ms 口径续档）。
- mdcore 零接触（只读复制其 web/tests/fixtures 真件一件用于对照）。

Stage Summary:
- 用户指定的六类复杂夹具全部要素级验证过——C 路线从"链路可行"升级
  为"要素级保真证据"；视觉级保真维持实施轮登记（FIDELITY 矩阵法）。
- Q3（C 实施形态）新增两条细化输入：静默丢弃面检出策略 + CJK 归一化
  口径。

---

Task ID: anchor-t1
Agent: 主代理（office 线）
Task: Office 锚点写面最小扩展方案与兼容性分析（用户裁决③"暂维持现有
公开 API，方案先行，不提前扩大 RunSpec"执行轮——文档面零代码变更）。

Work Log:
- 兼容性一手核实（mdcore @ 00ad88b，grep 全量）：RunSpec 全消费面
  清单——struct 字面位点仅 4 处（docx.rs:2688 code_line 用 ..base 语法
  免疫 + 2996/3011/3026 run_spec_constructors 测试断言机械补 anchor:
  None 三处）；外部消费者（mdcore-wasm docx_export/docx_import + 集成
  测试）全构造器面零改动；RunSpec 无 serde derive（序列化漂移面不
  存在）；发射点 add_spec_to_paragraph（docx.rs:2777）+ 读分发
  （docx.rs:1708）+ runs_to_specs 回放（tests:2074-2083 External-only
  映射 Anchor 静默丢）逐点亲读。
- **office-g 方案文档** 落档（office-line/office-g_锚点写面最小扩展
  方案.md）：三阶段（Phase 0 现状维持=本轮生效 / Phase 1 RunSpec.anchor
  实施就绪规格[字段/anchor_link 构造器/发射单分支/优先序 footnote>
  field>anchor>link 读写镜像] / Phase 2 bookmark_paragraph 正交件
  [id 计数唯一化 + _Toc 保留名拒绝]）+ 兼容性全量清单表 + 加性证明
  （CellData/ParagraphNumbering Option-字段先例对齐）+ 测试计划
  T1-T7（预期 G1 +7~9）+ yrs-b R4.5 联动升级路径表（弱绑定→强绑定
  = 适配层单点改动，office-io 不感知协同——非耦合面维持）+ 不做面
  登记（Internal 结构性不可达/HYPERLINK 域红线/MarkSpan 前置）。
- mdcore 零接触零提交（纯分析仓文档 + grep/读源一手）。

Stage Summary:
- 用户口径③完整落地：公开 API 零变化（Phase 0 生效），Phase 1/2
  实施就绪规格 + 兼容性全量清单在档——用户任何时点裁决即可开工，
  无前置调查欠账。
- 与 D1 线联动闭环：yrs-c 原型弱绑定形态 ↔ office-g Phase 1+2 强绑定
  升级路径互引。

---

Task ID: sess6-closeout
Agent: 主代理（cleanroom 线）
Task: 本会话（用户指令"持续推进6轮"）收口轮——HANDOFF/README 全量
同步 + 双仓状态核实 + 最终推送 + 阻塞呈报更新。

Work Log:
- 会话六轮执行：d1c-t1（构造/装载 + 前置核验三件套 + 第 12 次平台
  重置恢复）→ d1c-t2（同步/撤销深验 + 压缩点 + B3.4 定谳）→ d1c-t3
  （锚点/快照）→ d1c-t4（载体证据 + 隔离终验 + ASan 门禁 + yrs-c
  报告）→ r-o2-t2（复杂夹具六类 + 三工程发现）→ anchor-t1（office-g
  方案）→ 本收口轮——满足"持续推进 6 轮"。
- HANDOFF §6 速览重写（六轮成果 + 剩余门控全量）；README collab/office
  两行同步（yrs-c + office-f §8 + office-g）。
- 双仓状态核实：cleanroom-analysis 本会话六提交全部推送远端；mdcore
  main=00ad88b 全程零接触零提交（D1 可撤回纪律 + 用户口径④[D2/D3/
  MD 线/t22-review 不放行]双遵守）。

Stage Summary:
- 本会话净交付六件：collab-adapter 原型 crate（P8-P10 探针）+ yrs-c
  原型报告（三验证面闭环 + 22 条规则对账）+ P11 载体对照证据 +
  ASan 门禁（适配层零报告）+ r-o2-t2 复杂夹具六类全过（office-f §8）+
  office-g 锚点写面方案（兼容性全量清单）。
- 阻塞呈报（全部带用户裁决前置）：D1 实施轮三裁决（载体选型[建议
  XmlFragment]/B3.2 知情确认/B3.4×R-3 栈上限）+ D2/D3 + 锚点写面
  Phase 1/2 触发 + R-O2 Q3 实施形态 + MD 线三问 + t22-review 清单。
- mdcore 零接触；全部成果双仓推送后零未推送状态。

---
Task ID: r-o2-t3
Agent: 子代理（R-O2 回归套件）+ 主代理亲验集成
Task: R-O2 渲染管线回归套件固化：spike 证据（A-I 样本）→ 确定性
CI 可跑 14 用例回归套件（9 有效硬断言 + 5 畸形行为钉定）。

Work Log:
- 新建 spike-ro2/regress.py（套件主体：用例注册表 + 冷/暖双次转换
  [隔离 -env:UserInstallation profile] + 进程组超时杀除 + /proc 残留
  检测 + 逐断言 expected/actual 记录 + JSON 全量报告 + CLI
  --filter/--timeout/--json-out/--list）；ro2_fixtures.py（确定性夹具
  构建器：固定 zip 时间戳 + sha256 计数器流垃圾字节，9 有效 + 5 畸形）；
  rss_wrapper.py（每转换独立子进程峰值 RSS——/usr/bin/time 不存在，
  getrusage(RUSAGE_CHILDREN) 替代口径登记）；fixtures/import_sample.docx
  真件副本（sha256 与 mdcore 原件一致）；REGRESSION.md 报告。
- 全量运行：python3 regress.py → 14 用例 = 9 pass + 5 pinned + 0 fail，
  131 断言全绿，exit 0，~16s；连续 3 次全量绿 + 跨运行页数/图片数/
  尺寸零漂移 + 夹具 sha256 台账 5+ 次运行字节级一致。
- 超时治理双验证（--timeout 0.3 与 --timeout 1 各一）：kill=killed_pg、
  soffice 残留 PID []（零孤儿）、套件非零退出、无挂起。
- 畸形输入两条新发现登记：①垃圾字节 → LO 文本导入兜底（rc=0 + 乱码
  PDF 产出，"转换成功"≠合法 DOCX，消毒五面前置更必要）；②加载失败
  rc 仍为 0（truncated/bad-xml/missing-content-types：失败只在 stderr
  "source file could not be loaded" + PDF 缺席）——退出码不是失败信号，
  检出策略 = PDF 产出核验 + 输出扫描 + 转换后要素计数断言（发现①的
  教训推广为通用口径）。勘误登记：spike2「A5 横」实为 A4 旋转尺寸
  （842×595pt），套件按实测 pt 断言。
- 保真边界维持：要素级非视觉级；poppler 仅检视面；mdc-pdf 消费腿
  Q3-gated 未覆盖；LO 25.2.3.2 vs 26.8.1 版本差维持登记（钉定破裂 =
  版本差信号，复核后更新）。
- 两前序 spike 脚本零改动复跑全过（SPIKE/SPIKE2 PASS）；mdcore 零接触
  （只读复制真件一件）。
- 主代理亲验集成：全量重跑 14/14（9+5+0）复现 + 超时路径亲跑
  （REAL_EXIT=1 无孤儿）+ REGRESSION.md 通读 + 文件范围核查（仅
  spike-ro2/ 内）。

Stage Summary:
- C 路线证据形态升级：一次性 spike → 可重复回归资产（一条命令 16s，
  CI 可跑；JSON 报告 + 确定性门 + 超时治理内置）。
- 敌意输入面证据补强：静默丢弃钉定（w:p 级 drawing → pdfimages=0）
  + 两条新发现（垃圾字节兜底产出 / 加载失败 rc=0）→ 实施轮
  SECURITY_POLICY 的 LO 子进程治理与检出策略输入就绪。
- Q3 输入更新：mdc-pdf 消费腿口径（CJK 归一化 + pdfium 差异）维持
  实施轮必做；上游 26.8.1 复跑套件为版本差核对前置动作。

---
Task ID: d1d-t1
Agent: 主代理（collab 线，加固轮）
Task: D1-a 适配层加固（用户六项交付口径）：测试矩阵资产化 + 敌意
输入/不变量测试 + skip_gc 强制面 + RSS/wire 循环测量 + 无界增长登记
+ XmlFragment 选型级深验。mdcore 零接触。

Work Log:
- adapter/tests/matrix.rs 25 测试（用户八类一一对应 + 锚点/侧车面）：
  装载三路径/UTF-8 边界/限额 ==+1；edit_stream 四变体+错误面；跨端
  单向收敛+幂等复交换；双端不相交并发+同位置冲突收敛；删除/重加/
  撤销复原；快照恢复+gc 后快照；origin 分栈/捕获窗合并与分离（显式
  sleep 确定性）/一笔装载/效果全局/redo；维护 gc 治理+幂等；锚点
  ID 稳定/悬空显式化/跨端/侧车往返——全绿（含 ASan 25/25 零报告）。
- adapter/tests/hostile.rs 13 测试：malformed（空/全零=合法空更新
  钉定/伪随机/高字节/全前缀截断/尾翻转——零 panic 零挂起引擎未
  毒化）；oversized 五路径小限额全拒；invalid bind/resolve 全面；
  删除锚点永不返回邻居；sidecar 六敌意面（含尾部冗余新收紧）；
  skip_gc 构造强制+GC 型对端内容级互通+盲区活文档——全绿（含
  ASan 13/13）。
- lib.rs 两处硬化：client_id 53 位构造期 fail-closed（yrs 仅
  debug_assert，release 静默污染 ID 编码——新发现）；侧车尾部
  冗余拒绝（原静默忽略）。
- p14_gc_evidence 四源对照实证：GC 源 wire 恰缩 55B=被删内容
  （ItemContent::Deleted 标记）；四源公开 API 观测全同（blocks
  pub(crate) 盲区）；**case D 证明 maintenance_gc 与未授权 GC 产出
  同族 wire——不可区分性强于盲区**；中继传播 502B 实证；上游 API
  请求候选两项登记。
- p12_sync_undo_cycles（SCALE/CYCLES 可调）：20 轮编辑→同步→
  undo→同步→redo→同步全收敛；RSS 恒定 8244kB 零增长；全量 wire
  +302B/轮；维护窗 B 治理 2.0% 而 A（作者端）0.0%——undo/redo 链
  钉住作者端墓碑（观察登记，机制假设未验证）；撤销栈每轮净留 1 项
  （redo 返回栈项——B3.4 无界面具体形态）；ASan 口径复现零报告。
- p13_xml_carrier_deep 七面深验全过（含 ASan）：序列化往返逐节点
  相等；混排三段交织+文本中段分裂插入+元素间插入序保持；ID 稳定
  （下标 10→11 同 ID）+删除悬空（位置语义 vs 身份级两步口径与适配
  层 resolve 一致）；父子树删子锚点悬空；复原=结构等价重插新 ID 族
  （与适配层发现同构）；双端并发混排收敛；深嵌套 wire 148B/li vs
  浅形 135B/段（嵌套开销 ~10%）。
- 三处探针代码死锁 bug 定性与修复（async_lock 非重入纪律三实例
  复登：commit() 不释放 guard 只有 Drop 释放——shadowing/未出
  作用域开读/跨文档链式临时值三形态；正确形态=一事务一作用域；
  适配层 API 面天然满足=隔离设计再实证）。
- 报告落档 collab-line/yrs-d_D1a适配层加固轮.md（交付总表+强制面
  文档+无界增长清单+深验结论+待核清单）。
- 回归：p8/p9/p10/p11 复跑全过（lib.rs 硬化零破坏）。

Stage Summary:
- 适配层原型从「探针证据」升级为「可重复测试资产」（cargo test
  一条命令 38 测试 + 三新探针，release + ASan 双口径全绿）。
- 三项新发现定谳：①undo/重插复原=新 ID 族（锚点跟随需 redone 链
  映射——上游 API 候选+事件面联动登记）；②GC 证据 wire 存在但
  公开 API 不可见且与 maintenance_gc 不可区分（强制面=构造+部署+
  快照门三件套定谳）；③client_id 53 位域 release 静默污染面（已
  fail-closed 补上）。
- XmlFragment 七面深验全过——D1 载体裁决证据链补全（完整映射
  矩阵仍属实施轮设计文档）；B3.4 无界面新增 redo 返回栈项形态。
- mdcore 零接触零提交；全部成果在分析仓 spike workspace。

---
Task ID: anchor-t2
Agent: 主代理（office 线，方案补全轮）
Task: office-g 锚点写面方案补全为完整可评审设计（用户加固指令 C 组：
六个维度全覆盖），维持纯方案零代码变更。

Work Log:
- §7 调用示例：Phase 1（anchor_link 消费侧 + 与既有 link 构造器
  并存）+ Phase 2（bookmark_paragraph 生产侧 + 完整闭环）+ 发射
  OOXML 精确形状（探针⑤ 基准）；styled_full 签名/DocxWriter::new/
  crate 名逐一对照 mdcore@00ad88b 实际 API 校正。
- §8 锚点身份与解析模型：三层身份表（书签名=跨会话稳定公开身份
  [与 MDCA 侧车同源]；引擎 id=单导出会话内[不可作身份——与 BlockId
  条件保真同因]；链接侧=按名直引）；读面现状（Anchor 变体已在/
  书签定义读面不暴露=登记面维持）；重名语义设计决定（拒绝，呈评审）。
- §9 生命周期语义：复制/删除/移动/陈旧引用/文档版本五行——陈旧
  引用=合法形（T3 钉定）；移动=书签随段落、删+重插不自动迁移；
  版本语义=不引入（模型现状锚定）。
- §10 迁移与向后兼容：三阶段零破坏链（含新产物在旧 mdcore 读兼容
  已在的论证 + 第三方引擎原生面 + LO 回归联动项）。
- §11 校验与错误矩阵 V1-V8（书签名 Word 规则全量 + anchor/link
  同 Some 优先序=非错误）；全部经既有 InvalidInput 变体；V4 ASCII
  域=显式保守取舍（跨引擎保真约束传导）呈后续裁决点。
- §12 备选方案与最小性论证：Option 0/A/B/C 四选项对照承接 r5-t2
  §13；Option A 最小性四约束论证（零引擎类型/零调用方漂移/读写
  对称/可撤回）；扩面止损线声明。
- §3 bookmark_paragraph 签名修正：补 Result 返回（V1-V6 校验面
  需要——原规格签名与校验矩阵不一致，本轮修正为实施就绪形态）。
- §13 实施批触发与门禁预期增补（T1-T7 + V1-V8 测试 +4~6 + e2e
  锚点断言 + LO 回归联动）。

Stage Summary:
- office-g 从「实施就绪规格」升级为「完整可评审设计」——用户 C 组
  六维度（API diff+调用示例/身份与解析/生命周期/兼容迁移/校验错误/
  备选与最小性）全覆盖；仍为零代码纯方案，Phase 1/2 触发维持用户
  裁决前置。
- mdcore 零接触（只读 API 对照核实三处）；一处规格自洽性修正
  （bookmark_paragraph 返回 Result）。

---
Task ID: md-deci-t1
Agent: 主代理（MD 线 + 跨线，裁决包轮）
Task: 用户 D 组指令执行——全线剩余项三分类（安全实现/完整提案/显式
阻塞）+ 类别 1 实施。mdcore 零接触（只读核对）。

Work Log:
- 全线枚举（MD/office/collab/PDF/规划五线）：HANDOFF §6 门控全量 +
  t22-review 前置清单余项（①②④⑦）+ R-1..R-6 + D1-D3 + R-O2 Q3 +
  锚点触发 + fuzz/真机/上游跟踪——20 项入表（DECISIONS.md）。
- 类别 3 十九项：每项精确决策问题 + 三组「单次裁决双线生效」联动
  复登（R-3×B3.4 含 p12 新输入——redo 返回栈项使无界增长比此前
  认知更激进；R-1×B3.2 同源镜像；R-2×R4.4 协同域消解）。
- 类别 2 两件（本轮已完备，不做再调查）：锚点写面 office-g §7-§13
  + D1 载体证据链 yrs-d §5/p13——既有 Q-R1/R2/R3 提案完备度维持。
- 类别 1 恰一项并已实施：STATUS.md §7-6 陈旧性修正——三项「遗留」
  实为已完成（render_watchdog_ms 生产消费 worker.rs:16-17 / 看门狗
  收报面 worker.rs:24-293 三形态死亡可观测 / README 双引擎 :17-18/
  46-48，全部 mdcore@00ad88b 亲核）；真遗留仅 fuzz 战役排期（转
  DECISIONS #17）。
- 其余类别 1 候选逐一排除并登记核查过程（ASan 常跑项本轮已做/
  office-g 待核属实施批门内/R-O2 mdc-pdf 腿 Q3 门内/收报面与
  README 已证完成）。

Stage Summary:
- 裁决形态升级：门控项从散落各报告收敛为单一决策登记表
  （DECISIONS.md）——每项带分类/状态/证据锚点/精确决策问题，
  用户可按表逐项裁决。
- 类别 1 清空（一项已实施：文档与现实一致性修正）；类别 2 两件
  完备待裁；类别 3 十九项决策就绪零发明。
- mdcore 零接触零提交（只读核对四处）。

---
Task ID: e-review
Agent: 子代理（A-D 独立审查）
Task: 对 d1d-t1 / r-o2-t3 / anchor-t2 / md-deci-t1 四件产物做只读对抗
审查（五项清单逐项核验，零代码变更零 git 写操作）。

Work Log:
- A：跑 cargo test -p collab-adapter（25+13 全绿）+ nightly ASan 复跑
  零报告 + p8-p14 七探针 release 复跑全过；静态核对公开面 22 fn +
  8 字段零 yrs 类型、Cargo 零新依赖、mdcore 零接触；yrs-d 全部 wire
  数字逐位复现（148772→154821B/+302B 每轮/B 端 2.0%/A 端 0%/栈深
  20/6739B/14769B/538-483B/中继 502vs557）+ yrs 源码三锚点亲核。
- A 敌意复核（/tmp 独立探针仓，不入库）：①13 字节伪造侧车
  count=0xFFFFFFFF → HashMap::with_capacity 预分配 489GB → SIGABRT
  （exit 134）——fail-open，证伪初稿"敌意面全拒"表述，且侧车路径
  完全无 input_byte_limit；②256B 限额会话 InsertBlockBefore 257B
  文本 → Ok 放行（InsertBlock 同输入 Err(Limit)）——B1.6 六写入
  路径五查一漏。
- B：全量 regress.py 复跑 14=9+5+0/131 断言/exit 0/冷暖页数 14-14/
  确定性门过；超时治理实测（kill=killed_pg/残留 []/exit 1/事后
  /proc 零 soffice）；真件 sha256 与 mdcore 原件逐字节一致；双前序
  spike 复跑 PASS；RSS 冷段 186-191MB vs 文档 ~220MB（garbage 255MB
  精确）记环境方差。
- C：office-g 22 项代码锚点逐一亲核全准；发现两处失实：DocxError
  无 InvalidInput 变体（§3/§11"既有变体"不成立）+"10 字段"实为 9。
- D：DECISIONS 20 行文件自洽；§1.1 四子项对 mdcore@00ad88b 亲核
  全准；笔记：#20 行内引"§2"应为"§1.1"。

Stage Summary:
- 裁决 FIX-REQUIRED：三 SHOULD-FIX（侧车 count 预分配 abort/
  InsertBlockBefore 限额绕过/office-g 错误变体失实）+ 六 NOTE；
  其余全 OK——数字断言全部精确复现或方差内、公开 API/依赖纪律/
  最小性维持、mdcore 零接触、无进程资源泄漏。

---
Task ID: d1d-t2 + anchor-t3
Agent: 主代理（审查修复轮）
Task: e-review 三 SHOULD-FIX 修复 + NOTE 采纳四条 + 回归锁。

Work Log:
- 适配层修复①（e-review 发现 1）：import_anchor_sidecar 入口补 B1.6
  Limit 检查 + 敌意 count 物理上界钳制（count > (len-9)/17 先拒——
  条目最小 17B；13B 载荷声明 40 亿条目不再预分配 abort）。
- 适配层修复②（发现 2）：InsertBlockBefore 补 B1.6 限额检查（before
  查找键计入总量——六写入路径六查）。
- 回归锁三面：hostile ⑤b（极端 count → SidecarFormat 不 abort）+
  ⑤c（小限额会话超限侧车 → Limit）+ §2 InsertBlockBefore 超限 →
  Limit；死代码 a2 清理。
- 修复后复验：38/38 全绿 + ASan 38/38 零报告 + p9 复跑全过。
- office-g 勘误（发现 3 + NOTE）：§3/§11 改为 Phase 2 **新增**
  DocxError::InvalidInput 加性变体（公开 API 新增面显式登记——初稿
  "既有变体无新变体"失实，DocxError 实仅四变体）；§1.1"10 字段"
  勘误为 9。
- DECISIONS #20 引用勘误（§2 → §1.1；md-deci-t1 提交报文"类别 3
  十九项"实为 17——文件本身自洽，报文计数误差在此留痕）。
- regress.py 冷暖页数一致性升级硬门（NOTE 6 收严：非确定性转换 =
  用例 FAIL 而非警告打印）；复跑 14=9+5+0 全绿。
- yrs-d 增补 §6.5（审查与修复对账）+ 待核清单补两条（假钟注入
  实施轮项/count 上界边界组合面登记）。

Stage Summary:
- 三 SHOULD-FIX 全闭环（修复+回归锁+文档勘误）；"加固"宣称经对抗
  审查修复后成立；适配层敌意面收口至六写入路径六查 + 侧车双门
  （限额+物理上界）。

---
Task ID: e-gates
Agent: 主代理（集成与质量门禁轮）
Task: 环境恢复（第 13 次重置后 web 面 + pdfium fixtures）+ mdcore
全门禁亲跑复验 + collab spike 全探针回归 + 文档集成（HANDOFF/README/
worklog）。

Work Log:
- 环境恢复（第 13 次平台重置）：rustup 1.29.1 + Rust 1.99.0 钉住 +
  nightly-2026-10-15（ASan）+ wasm32 目标；pdfium chromium/8086
  SHA256 验证取件 + spike fixtures 四件重建（pdf_a_spike 主探针 +
  formfill_probe——lopdf 现场生成确定性产物，三探针/S6 全过）；
  wasm-bindgen-cli 0.2.129 钉版安装 + npm ci + build-wasm + bundle +
  typecheck 全过。
- mdcore 全门禁亲跑（main=00ad88b 零改动，基线逐项复验）：
  G1 cargo test --workspace **530/0/1i**；PDF 四面 **68/79/81+6i/
  109+6i**（默认/pdfium/lopdf/双引擎——修复 fixtures 缺失后精确
  复现）；perf 门 **6/6**；clippy native + wasm32 --all-targets 各
  **恰 2 wasm_bindgen 基线**；fmt **0**；scan **0 hit-paths PASS**；
  wasm32 check + --all-targets 过；web 七套件 **18/15/28/11/13/9/9**
  （contract_gate/web_smoke/ime_cdp/docx_import/docx_export/
  xlsx_import/xlsx_export）；Cargo.lock 零 diff（git status clean）。
- collab spike 全探针回归：p1-p14 全 PASS + adapter tests 38/38
  （含 ASan 口径）。
- spike-ro2 回归套件最终复跑：14=9 pass+5 pinned+0 fail（含新硬门）。
- 文档集成：HANDOFF §6 速览重写（本轮五件交付 + 全门禁复验 + 剩余
  门控收敛于 DECISIONS.md）；README 三处同步（yrs-d/R-O2 套件/
  anchor-t2/t3 + DECISIONS.md 入仓库结构表）；worklog 六节追加。

Stage Summary:
- 用户停机条件逐项核验：全部非阻塞工作流有实现或可评审交付物 ✓、
  全部可用相关测试已跑 ✓、独立审查发现已修复或登记 ✓、文档与实际
  代码一致（含 STATUS 陈旧修正）✓、全部成果已提交推送 ✓。
- mdcore 全门禁零漂移复现（第 13 次重置后首次全量复验）——office
  线基线可信度维持。

---
Task ID: sprint2-t0
Agent: 主代理（冲刺恢复轮）
Task: 环境恢复（第 14 次平台重置）+ 双仓远端核对。

Work Log:
- 远端核对：cleanroom-analysis 本地 de186f7 落后远端 main=ec9a7df
  （d1c/d1d/anchor/md-deci/e-review/e-gates 六轮已推送态）→ ff-only
  拉齐；mdcore 本地 = 00ad88b = 缓存 origin/main（fetch 无凭据 401，
  以缓存 ref + 用户指令口径一致为准）。两仓工作树仅 mode-bit 伪漂移
  （core.fileMode false 定性归零，零内容 diff）。
- 环境恢复：rustup 1.29.1 + Rust 1.99.0 钉住 + nightly-2026-10-15
  （rust-src，ASan 口径）重装；LibreOffice 25.2.3.2 在场（= 套件钉定
  版本，26.8.1 仍为上游复跑目标）；collab-adapter 基线 38/38 复现；
  R-O2 回归套件 14=9+5+0 SUITE PASS 复现；mdcore cargo check
  --workspace 干净（2 wasm_bindgen 警告 = 既有基线）。
- 推送凭据：GITHUB_TOKEN 环境变量缺席（脚本纪律 = 不落盘）——本轮
  全部提交先本地落账，推送在会话尾统一尝试，若 401 呈用户。

Stage Summary:
- 恢复一次完成，基线全部复现；无重复调查。

---
Task ID: sprint2-t1
Agent: 主代理（决策映射轮）
Task: DECISIONS.md 十七项类别 3 裁决 → 五依赖组映射（用户指令
Task 1+2：每组推荐默认/备选/具体后果/不可安全推断的精确决策，
转引在档证据零重写）。

Work Log:
- 落档 DECISION_GROUPS.md（仓库根，与 DECISIONS.md 并列）：17 行
  → G1 载体与适配层边界（#4①/#5/#6/#19）/ G2 撤销与逻辑锚点身份
  （#4②/#1/#2）/ G3 撤销栈资源上限（#3=4③/#15/#16）/ G4 Office
  锚点写 API 含保真载荷族（#8/#9/#10/#11/#12）/ G5 DOCX 渲染器
  部署治理含安全回归基建（#7/#18/#17）。
- 每组四件套齐全（默认/备选/后果/不可推断点）+ 组间依赖图 + 最小
  裁决批次建议（批次 1 D1 解锁包 = 载体+B3.2+栈上限三件同批；批次
  2 锚点 Option A；批次 3 Q3 预装+子进程；批次 4 低优先合批）。
- 证据锚点全部转引（yrs-c §4/§6、yrs-d §2.2/§3/§4/§5、office-f
  §4.2/§5.2/§8.4、office-g §12/§13、06 §3/§5.2/§8、md-r1r6-t1、
  r-o2-t3 REGRESSION）——零重调查零再推导。

Stage Summary:
- 十七项裁决从平表升级为依赖组视图：用户四批批注即可全线解锁；
  「不可安全推断」逐项显式化（载体终裁/D2D3 立项/B3.2 知情确认/
  REUSE §R5 裁决门/栈上限值/Phase 触发授权/Q3 分发授权等）。


---
Task ID: sprint2-t2
Agent: 主代理（适配层回归补强轮——撤销/重做 × 锚点身份）
Task: 用户指令 Task 3a——deleted-anchor undo/redo 与逻辑 ID 映射
回归覆盖（不发明 yrs 未支持行为：全部断言先观测后钉定）。

Work Log:
- 观测探针（临时 bin，跑完即删）：①重做被撤插入 = 块回归但锚点
  恒悬空 + 重绑定新 ID（clock 3→6）；②窗外分离项深度记账全链
  [A]1/0 [B]2/0 [C]1/1 [D]0/2 [E]1/1 [F]2/0——redo 弹 1 压 1
  （返回栈项对称守恒）；③maintenance_gc 后已删锚点仍显式悬空；
  ④跨端复活收敛（对端旧锚点同悬空 + 对端重绑定 = 作者端新锚点
  完全一致）。
- 新测试文件 adapter/tests/anchor_undo_redo.rs（6 测试）：重做
  新 ID 族钉定 / 深度记账对称性 / 多循环复活互异 ID + anchor_of
  映射表替换 / 跨端复活收敛与逻辑映射一致 / 侧车陈旧映射导入后
  显式悬空（权威面在 resolve）/ GC 后悬空显式。
- API 加性扩一件：redo_depth()（undo_depth 镜像，yrs redo_stack()
  公开 API 直读——p12「redo 返回栈项」B3.4 观测面的确定性形状）。
- 门禁：cargo test -p collab-adapter 44/44（25 matrix + 13 hostile
  + 6 新增）全绿；nightly-2026-10-15 ASan 口径 44/44 零报告；
  p8/p9/p10 前序探针 release 复跑全过；一处测试设计自纠（插入须
  Human origin 独立于 Ai 装载——撤销面才可独立触达）。

Stage Summary:
- 撤销/重做 × 锚点身份的行为面从「单一删除面测试」扩展为六面
  回归锁；复活=新 ID 族语义的跨端收敛性 + 逻辑映射表替换语义
  首次钉定；redo 侧深度观测面入 API（B3.4 证据链补强）。

---
Task ID: d1e-t2（含 mdcore r5-t3）
Agent: 主代理（资源限全覆盖轮——用户 Task 3b）
Task: 「Harden resource-limit coverage across every write entry
point」——适配层全入向入口边界矩阵 + mdcore office-io 写面补检。

Work Log:
- 适配层入口清点（B1.6 全量）：7 检查位在册（load_bulk_text/
  load_blocks/AppendText/InsertBlock/InsertBlockBefore/apply_update/
  import_anchor_sidecar）+ 3 缺口补检——①RemoveBlock 查找键（与
  e-review 先例 before 键同纪律「查找键同为入向输入」）；②bind
  语义锚字符串（写入 anchor_table 并经侧车导出——单笔入向账目
  统一；绑定**条数**上限维持 yrs-d §4 开放裁决面不预设）；③
  encode_diff remote_sv（读面但解码入向输入——纪律统一，合法 SV
  恒远低于限额不构成正常路径约束）。
- 新测试文件 adapter/tests/limit_boundaries.rs（10 测试）：全部
  入向入口「== limit 恰过门 + limit+1 恰拒」两钉（size/limit 字段
  精确）；算术口径路径（id+text+64[+before]）按精确账目构造、
  wire 路径用真实工件钉（差量 57B/全零 SV 256B/侧车 32B——先
  观测尺寸再定限额）；RemoveBlock == limit 过门到达搜索面
  （InvalidInput not found ≠ Limit 证明检查先于搜索）。
- mdcore 写面全入口核查（office-io）：XLSX 五限 + DOCX 段落/run
  双限边界在册完备（r4-t2/r4-t2b）——两真缺口补检：①
  write_hyperlink tooltip 无限长直写 XML 属性（引擎
  MAX_PARAMETER_LEN=255 真值镜像 typed 前移——url 2080/显示文本
  32767 在检而第三字符串字段漏检；MAX_TOOLTIP_TEXT 常量 + 恰 255
  过/256 拒）；②write_row 列域边界（恰 16384 列过/16385 拒）+
  发现登记：u16 宽度检查为不可达防御（u16 域 65535 > 列域
  16383，check_coord 恒先触发）。
- 门禁：适配层 54/54（25 matrix + 13 hostile + 6 anchor_undo_redo
  + 10 limit_boundaries）全绿 + ASan 口径 54/54 零报告 + 零编译
  警告（cargo fix 收敛）；mdcore G1 530→531 零失败 + clippy
  native/wasm32 恰既有 2 wasm_bindgen 基线 + fmt 0 + scan 0 +
  Cargo.lock 零 diff + web 面零 tooltip 消费核实不受影响。
- 文档登记：xlsx-r1/CAPABILITIES.md 写面限增量（tooltip 255 镜像
  + write_row 列域边界 + 测试锚 4→5）；mdcore 提交 b15c0ab。

Stage Summary:
- 「every write entry point」两面收口：适配层 10 入口全检全边界
  钉定（3 补检 + 7 复验）；mdcore 写面 2 真缺口修复（tooltip 255
  引擎镜像 + write_row 列域）——全部为既有纪律补齐，零新限额
  发明、零公开 API 签名变更。

---
Task ID: r-o2-t4
Agent: 主代理（R-O2 回归扩展轮——用户 Task 3c）
Task: 「Extend R-O2 regression tests for conversion failures,
malformed inputs, and deterministic fixture verification」——
14 → 21 用例，全部新形状先探针观测后钉定。

Work Log:
- 探针观测八面（临时脚本，跑完即删，LO 25.2.3.2 实测）：不存在
  源文件→rc=0 无 PDF；0 字节→writer_pdf_Export 装载为空 Writer
  文档→1 页零文本 PDF；PDF 字节改 .docx 扩展→**内容嗅探改道
  draw_pdf_Export（Draw 导入）**；外来 zip→load 失败；缺
  word/document.xml→load 失败；根元素错（html）→**rc=0 + 1 页
  零文本静默空产物**；声明 1GB/实际 32B zip 炸弹→0.55s 快速
  load 失败（无分配放大）；跨进程双冷转→同尺寸 34224B 但字节
  不同一、归一化文本同一。
- 转换失败面 3 用例（conv-fail-pinned 新类）：nonexistent-source
  （rc=0 族极端成员——产物断言唯一可靠检出）/ empty-source-file
  （writer 装载空文档 + 1 页零文本）/ pdf-renamed-docx（Draw 改道
  ——治理含义：文件类型纪律必须调用 LO 之前由我方执行）。
- 畸形补充面 4 用例（malformed-pinned）：foreign-zip /
  missing-document-part（与 missing-content-types 互补分别钉定）/
  wrong-root-element（静默空产物族第三成员——『转换成功≠内容
  合法』）/ declared-size-bomb（快速拒绝资源面锚，15s 宽松门）。
- 跨进程产物确定性门（套件级新门，8 断言）：字节级不同一（钉定
  ——PDF 元数据面差异）+ 内容级同一（页数/归一化文本/字体清单
  硬断言）——mdc-pdf 腿指纹/缓存/去重策略必须走内容级摘要。
- 台账语义精化：新增夹具键 = 良性扩展（首跑基线）；变更/移除 =
  非确定性回归（维持硬门）——此前语义套件自身扩展会误触门。
- 断言面工程自纠一处：滤镜名（writer_pdf_Export/draw_pdf_Export）
  在长仓路径下超出 240B stdout 摘要窗 → 改位置稳健文档类型短语
  （as a Writer/Draw document），滤镜名登记不依赖。
- 门禁：全量 21 用例 = 9 pass + 12 pinned + 0 fail（188 断言，
  SUITE PASS）双次复跑稳定（台账非首次零漂移 + 跨进程门恒过）；
  nonexistent-source 夹具 absent-by-design 守卫（台账/报告/尺寸
  三处）。
- 文档：REGRESSION.md §9（扩展轮四节：转换失败面/畸形补充面/
  跨进程确定性门/台账语义精化 + 断言面工程注记）；DECISION_GROUPS
  G5 证据增量（r-o2-t4 治理输入——Q3 的文件类型纪律输入侧对偶）。

Stage Summary:
- R-O2 回归资产从「畸形行为钉定」扩展为三族全覆盖：转换失败
  （rc=0 族全景）/ 畸形输入（9 钉定）/ 产物确定性（内容级同一
  硬门）——三项新治理发现（内容嗅探改道/静默空产物第三成员/
  字节级非确定）直接服务 Q3 裁决输入。

---
Task ID: d1e-t3
Agent: 主代理（实施就绪补丁轮——用户 Task 3d）
Task: 「Prepare implementation-ready patches or small isolated
branches where semantics are already settled」——锚点写面 Phase 1+2
（office-g §7-§13 六维度完备设计 + DECISIONS #8 门控）以隔离分支
形态实施，main 零接触。

Work Log:
- mdcore 隔离分支 office-g-phase1-anchor（b15c0ab → c4856f7）：
  Phase 1（RunSpec.anchor 加性字段 + anchor_link 构造器[w:anchor
  属性形零 rels——引擎 HyperlinkType::Anchor 直用] + V8 优先序
  发射分支[anchor 赢 link 忽略——读写镜像]）+ Phase 2
  （bookmark_paragraph[bookmarkStart/End 对 + 写器计数器 id 唯一
  化]）+ DocxError::InvalidInput 加性变体（office-g 唯一枚举新增
  面登记兑现）+ V1-V6 校验矩阵 + V7 空锚点粘性校验 + wasm
  docx_invalid_input 错误码完备性映射（当前 wasm 面不可达——
  BlockTree 无锚点载荷，②⑦ 门控诚实登记）。
- 测试组 T1-T7 全量（7 新测试 + T6 回放分支）：写读全环（w:anchor
  XML 形 + 零 rels 断言 + Anchor 变体复原）/ 优先序对抗位 / 悬空
  合法形 / 书签 XML 面 / V1-V6 矩阵（含 CJK 双面）/ V7 粘性拒绝
  零产物 / 构造器恒等。
- 实施发现两条（超出设计文档的实施层决策，均已在代码注释与分支
  提交留痕）：①**V5 校验序**——office-g §11 矩阵为校验面清单而非
  求值序；'_' 非 ASCII 字母，若 V3 先行则 _Toc 保留名检查恒不可达
  ——实现取 specific-before-general（V5 前移 V3 前）；②**V7 粘性
  错误语义**——不 take 不重置（错误路径与 Limit 路径同语义；无效
  run 已发射进引擎文档，后续 save 恒拒，唯一解 = 去除违规规格重建
  ——首版 take+重置实现被自测证伪后修正）。
- 分支门禁全绿：G1 531→538 零失败 / clippy native+wasm32 恰既有
  2 wasm_bindgen 基线 / fmt 0 / scan PASS / Cargo.lock 零 diff /
  wasm32 check 过。
- main 零接触回退：checkout main（b15c0ab 状态与分支前完全一致，
  office-io lib 70/70 复验）；评审件落档 office-line/0001-office-g-
  Phase-1-2-*.patch（format-patch 产物）；DECISION_GROUPS 批次 2
  更新为「裁决 = 合入 main 一句话，撤回 = 弃分支零残留」。

Stage Summary:
- DECISIONS #8 的裁决成本从「读设计文档想象实现」降为「看已测试
  补丁批注合入」——Phase 1+2 全量代码 + 门禁证据 + 两条实施发现
  就绪；gated API 纪律维持（main 干净，分支隔离）。

---
Task ID: sprint2-final
Agent: 主代理（冲刺收口轮——推送 + 收尾验证 + 决策单）
Task: 用户指令收尾件——「Commit and push each independently
reviewed increment; end with a concise decision sheet (decisions
needed, recommended defaults, what each unlocks, work completed
without decisions)」+ 停机条件兑现（全部可执行任务完成 +
测试/评审通过 + 剩余事项确实被门控 → 最终报告）。

Work Log:
- 环境核对（恢复一次纪律）：工作区持续未重置（双仓本地提交在
  场）；远端核对 cleanroom-analysis main=ec9a7df / mdcore
  main=00ad88b（= 指令基线），本地领先 = 上会话冲刺五件已提交
  未推送；工具链仅 PATH 新鲜（~/.cargo/env 重源即复：1.99.0 +
  nightly-2026-10-15；LO 25.2.3.2 在场）。
- 推送积压增量（用户提供凭据）：cleanroom-analysis main
  ec9a7df→141c482（五件：decis-g1/d1e-t1/d1e-t2+r5-t3/r-o2-t4/
  d1e-t3+HANDOFF）；mdcore main 00ad88b→b15c0ab（r5-t3）+ 新
  分支 office-g-phase1-anchor（c4856f7——锚点 Phase 1+2 就绪
  补丁呈批件）。
- token 零落盘纪律自纠：推送时曾以 PAT 嵌 remote URL（.git/config
  落盘违反 HANDOFF §5-2 铁律）→ 推送毕立即双仓 set-url 还原 +
  grep 验证零残留（exit 1 = clean）。
- 收尾验证三件（本会话亲跑，推送态精确复现）：适配层 cargo test
  -p collab-adapter = 54/54（matrix 25 + hostile 13 +
  anchor_undo_redo 6 + limit_boundaries 10）零失败；mdcore
  cargo test --workspace = 531/531 零失败（= b15c0ab 口径）；
  R-O2 回归套件 = 21 用例 9 pass + 12 pinned + 0 fail + 夹具
  确定性门 pass + 跨进程确定性门 pass，SUITE PASS。
- 决策单落档 DECISION_SHEET.md（仓库根，与 DECISIONS/
  DECISION_GROUPS 并列）：四批批注版（批次 1 D1 解锁包三件/
  批次 2 锚点合入一句话/批次 3 Q3 预装+子进程/批次 4 低优先
  合批）+ 每批解锁面 + 本轮零裁决已完成五件表 + 收尾验证证据 +
  剩余风险五条（#676/栈无界/LO 版本差/字节级非确定/沙箱真机
  不可比）——全部转引 DECISION_GROUPS 对应组节零重复。
- HANDOFF §6 速览更新：推送态（141c482/b15c0ab/c4856f7）+
  DECISION_SHEET 用户回答入口 + 收尾验证三大件全绿口径。

Stage Summary:
- 停机条件达成：五件冲刺交付全部推送在档（含 mdcore 隔离分支
  呈批件）；三大件测试本会话亲跑全绿零漂移；剩余事项全部为
  类别 2/3 门控项（DECISIONS §1 类别 1 已清空）——用户四批
  批注（DECISION_SHEET.md）即全线解锁；无未推送提交、无未登记
  工作、无环境缺口。

---
Task ID: sprint2-b2
Agent: 主代理（批次 2 裁决执行轮——office-g Phase 1+2 合入 main）
Task: 用户裁决 DECISION_SHEET 批次 2：检查 office-g-phase1-anchor
与 mdcore main 的提交关系/共同祖先/文件差异，满足条件（分支目标
正确、不覆盖 main 独有提交、无未解决冲突、架构兼容）则合入 +
跑门禁 + 推送；合并后更新 HANDOFF 记录最终提交/验证结果/已知
风险/待决策事项；不扩大范围。

Work Log:
- 环境恢复（第 15 次重置，恢复一次纪律）：双仓本地回旧态
  （cleanroom de186f7 / mdcore 00ad88b，本地分支丢失）→ 瞬态
  URL fetch 恢复远端 refs（mdcore main=b15c0ab + origin/office-g-
  phase1-anchor=c4856f7；cleanroom main=ae41cea）→ 双仓 ff-only
  拉齐 + 本地分支重建 + core.fileMode false 归零 mode-bit 漂移；
  工具链随重置丢失 → 重装钉住口径（rustup 1.29.1 + 1.99.0
  minimal + wasm32 target + clippy/rustfmt + nightly-2026-10-15
  [rust-src]）。
- 合并前置检查四件全过：①merge-base = main HEAD（b15c0ab）→
  main 为分支祖先，ff 可合零冲突可能；②main 独有提交 = 0（零
  覆盖）；③分支独有提交恰 1（c4856f7——office-g Phase 1+2 实施
  批）；④差异面 4 文件 +398/-3（office-io/src/docx.rs +366 /
  tests/docx_roundtrip.rs +17 / mdcore-wasm/docx_export.rs +12 /
  docx_import.rs +6）——零公开模型（BlockTree/mdcore 核心 crate）
  触碰，与 CAPABILITY_NOTES §13 Option A 加性扩承诺一致。
- ff 合入：main b15c0ab→c4856f7（线性历史维持，无 merge commit）。
- 全门禁本会话亲跑（合并后 main 上）：G1 cargo test --workspace
  = **538/538 零失败**（531 既有零回归 + 7 新测试全过，= 分支声明
  口径精确复现）；clippy native + wasm32 双面恰 2 wasm_bindgen
  基线（lib.rs:212/221 static_method_of 属性误判——既有基线，非
  分支触碰文件）；cargo fmt --all --check = 0；compliance/scan.sh
  product = 0 hit-paths PASS；Cargo.lock 相对 b15c0ab 零 diff；
  wasm32 check --workspace clean。
- 推送：mdcore main b15c0ab→c4856f7（瞬态 URL，.git/config 零
  token 残留 grep 验证）。
- 文档收口（cleanroom-analysis）：CAPABILITY_NOTES §13 头部裁决
  落定注记（Option A Phase 1+2 全批 + 合并证据 + §13.4 问题 1/2
  闭环、问题 3 陈述维持）；DECISION_GROUPS 批次 2 翻转「已裁决已
  执行」；DECISION_SHEET 批次 2 节改为结果表（合入方式/差异面/
  门禁复验/解锁面/已知风险）；HANDOFF L-MD 行（main=c4856f7 +
  版本链追加 b15c0ab/c4856f7 两环 + 九门禁 538）与 §6 速览
  （批次 2 执行后态 + 剩余门控批次 1/3/4 + 第 15 次重置恢复记录）。

Stage Summary:
- DECISIONS #8（G4 锚点写面 Phase 1/2 触发）闭环：Option A 裁决
  → 前置检查四件全过 → ff 合入 → 全门禁绿复验 → 推送。锚点
  写读对称达成（读 4/写 4：External/Internal/Anchor/Unresolved
  读面 vs External/anchor/anchor_link/bookmark_paragraph 写面）；
  R4.5 强绑定升级路径开通（OfficeBookmark 弱→强绑定）。
- 已知风险登记：悬空锚点=合法形（Phase 2 书签独立可用，V 边界
  已校验）；wasm 面 docx_invalid_input 当前不可达（BlockTree 无
  锚点载荷，②⑦ 门控维持 = DECISIONS #10/#12）。
- 剩余门控：DECISION_SHEET 批次 1（D1 解锁包三件——最优先）/
  批次 3（Q3 渲染管线）/批次 4（低优先合批）。

---
Task ID: r-o2-t5
Agent: 主代理（锚点存活轮——批次 2 联动项执行）
Task: 用户「继续」指令 → 批次 2（已裁决）解锁的联动项：office-g §13
「LO 侧回归套件可同批扩锚点存活用例」——mdcore c4856f7 锚点写面
发射形状经 LO → PDF 的链接存活行为钉定。批次 1/3/4 为不可代签产品
裁决，不在本轮范围。

Work Log:
- 环境恢复（第 16 次重置，会话中途发生）：双仓本地回旧态
  （cleanroom de186f7 / mdcore 00ad88b）→ 瞬态 URL fetch + ff 拉齐
  （cleanroom b3a6faf / mdcore c4856f7）+ 工具链重装（1.99.0 +
  wasm32 + clippy/rustfmt + nightly-2026-10-15）；全部成果已在远端
  零损失。
- 探针观测（临时脚本跑完即删）：mdcore 发射形状（w:hyperlink
  w:anchor 属性形零 rels + bookmarkStart/End 对 + 孤立书签 + 外链
  对照）→ LO 25.2.3.2 → PDF：①w:anchor 内链 = GOTO Link 注记
  **存活**，目标页 2 = 书签所在页，直接 XYZ 目的地（72,769.889,
  0——坐标为观察值）；②书签**不导出命名目的地**（PyMuPDF
  resolve_names() 空——LO 转换期消解为直接目的地）；③孤立书签零
  可观测面（无注记/无目的地/无 outline 差异）；④外链 URI 注记
  类型可区分；⑤Heading1 → PDF outline 条目（观察值）。
- 断言面工程选型：链接检视走 **poppler pdftohtml -xml**（内链 href
  = {stem}.html#{目标页} fragment 承载 + <a> 包裹文本关联）——套件
  **零三方依赖性质维持**（PyMuPDF 仅探针一次性观测用，不入依赖）；
  <a> 包裹为文本块粒度（实测吸收前置普通 run 不吸收后置句号）→
  链接文本断言用包含语义不用相等。
- 实施三件：ro2_fixtures.py build_anchor_survival（夹具精确镜像
  mdcore T1/T5 发射形状——链接段页 1 / 书签目标段页 2 显式分页分离
  + 孤立书签 + 外链对照 rels）+ regress.py pdf_links_by_page 检视器
  + checks_anchor_survival 十六断言（通用 5 + 文本 5 + 链接 6：
  内链恰一/目标页精确/外链恰一 URL 精确/页 2 零注记/链接文本关联）
  + CASES/build_all_fixtures 注册。
- 门禁：全量 22 用例 = 10 pass + 12 pinned + 0 fail（204 断言）
  SUITE PASS **双次复跑稳定**；夹具确定性门 + 跨进程确定性门恒过；
  新夹具键 anchor-survival = 良性首跑基线（§9.4 语义）。
- 文档：REGRESSION.md §1（poppler 行加 pdftohtml + stdlib only
  维持注记）+ §10 新增（10.1 断言表/10.2 行为发现五条/10.3 门禁与
  台账）；DECISION_GROUPS 批次 2 联动执行注记 + G5 证据增量；
  DECISION_SHEET 批次 2 解锁面更新；HANDOFF §6 速览同步。

Stage Summary:
- 产品 API 形状 → LO → PDF 链接存活**全链闭合**：mdcore 侧 T1/T5
  钉 XML 形状，本套件钉 LO 行为——w:anchor 内链 GOTO 存活且指向
  书签页（C 路线保留内部导航定谳），office-g §13 联动项闭环。
- 治理输入两条：书签不导出命名目的地（mdc-pdf 消费腿链接到达
  形态 = GOTO 显式目的地——命名树依赖策略不可行）；孤立书签对
  渲染产物零影响。
- 批次 2 全部后续（合入 + 门禁 + 联动用例）收口；剩余门控维持
  批次 1/3/4（DECISION_SHEET）。

---
Task ID: r-o2-t5-doc（一致性收尾）
Agent: 主代理（文档-现实一致性翻转）
Task: 批次 2 合入后保真矩阵单一真源同步（类别 1 纪律：文档与实际
代码一致）。

Work Log:
- FIDELITY.md #6 行翻转：超链接行从「External-only 写回 + RunSpec
  无锚点字段」翻转为「External + Anchor 内链双变体写回（c4856f7
  批次 2 裁决合入）」+ 测试锚点列补 T1-T7 锚点组与 R-O2
  anchor-survival（LO 存活钉定）；§2 anchor 论证行补闭环注记。
- CAPABILITY_NOTES §13.1 头部加时点注记（「现状」列 = r5-t2 评估
  时点，其中两行已被 c4856f7 翻转——防陈旧读取，证据行原样保留）。
- mdcore CONTRACT.md 核查：契约面辖设置/会话（宿主↔桥↔核），不辖
  office-io 公开 API——锚点合入无契约遗漏。

Stage Summary:
- 保真矩阵/能力注记/裁决文档三处与 mdcore main c4856f7 现实一致；
  零新调查，纯一致性翻转。
---
Task ID: sprint2-c（推送核实与停机复验轮）
Agent: 主代理（上下文恢复 + 双仓推送态核实）
Task: 「继续」指令——上轮 r-o2-t5/r-o2-t5-doc 收口后推送态核实
与停机条件复验。

Work Log:
- 环境核查：双仓工作区干净零漂移；cleanroom main=ae4aa4d（含
  r-o2-t5 锚点存活轮 + r-o2-t5-doc 一致性翻转两提交）；mdcore
  main=c4856f7。工具链未重置（第 16 次恢复后环境保持）。
- 推送态核实（关键发现）：本地 status 显示 cleanroom ahead 2，
  但瞬态 URL push 返回 up-to-date → fetch 证实远端 main 实际
  = ae4aa4d = 本地 HEAD——**上轮已完成推送，仅本地 origin/main
  tracking ref 陈旧**（瞬态推送不更新 tracking ref 所致假象）；
  update-ref 更正后 ahead 2 消失。
- owner 勘误：首次瞬态推送误猜 owner=cleanroom-universe（404
  Repository not found）→ remote -v 核实正确 owner=mmqz；token
  有效性经 GitHub API /user 探针确认（200, login=mmqz）。
- mdcore 私有仓同步验证：匿名 fetch 不可达（私有仓需凭证），
  瞬态 URL fetch 证实远端 main = c4856f7 = 本地 HEAD ✅。
- token 零落盘 grep 复验：仓内 ghp_ 命中 3 处（HANDOFF.md
  §5-2 纪律说明 / acrobat-audit WORKLOG 历史记述 / push 脚本
  用法注释）——**均为 ghp_xxx 占位符文档，非真实 token**，零
  落盘纪律维持。
- HANDOFF 版本链核查：L-MD 行 c4856f7 ✅；§6 行 73「b3a6faf
  基础上」为回溯性时点描述（冲刺五件推送时点基准）语义正确，
  非漂移——零编辑。

Stage Summary:
- 双仓与远端精确同步（cleanroom ae4aa4d / mdcore c4856f7），
  零积压零未推送提交。
- 停机条件维持：类别 1（可执行决策无关任务）清空；上轮门禁
  全绿在档（R-O2 22 用例双次复跑 / mdcore G1 538）；剩余全部
  门控 = DECISION_SHEET 批次 1（D1 解锁包，最优先）/批次 3
  （Q3 渲染管线）/批次 4（低优先合批）——等待用户批注。
---
Task ID: d1p-t1（十轮计划 R1）
Agent: 主代理（D1 就绪系列）
Task: BlockTree→XmlFragment 载体完整映射矩阵就绪件（批次 1 G1①
门控实施轮设计文档的预置件，d1e-t3 先例模式）。

Work Log:
- 环境第 17 次重置恢复：rustup 全家桶重装（1.99.0 minimal +
  clippy/rustfmt + wasm32 target + nightly-2026-10-15[rust-src]）
  + 双仓 fetch 同步复验（cleanroom a688e70 / mdcore c4856f7 零
  漂移）+ core.fileMode false 双仓。
- 模型面盘点（mdcore c4856f7 一手核验）：9 BlockKind 三分类
  （虚根 1 + 文本叶 5[Paragraph/Heading/CodeBlock/ThematicBreak/
  Table——Table 为 GFM 原文文本叶非行列容器] + 容器 2
  [Blockquote/ListItem 交织]）+ attrs 键面 3（level/lang/marker，
  类型开放 BTreeMap）+ MarkSpan 行内区间 + 寻址双轨
  （NodePath fragile 事务内 / BlockId 稳定注册表）+ Op 七操作
  全签名（InsertBlock.id 物化语义单列）。
- 就绪件落档 collab-line/yrs-e：§0 结论速览（全模型面 9/9 kinds +
  7/7 ops + 双轨零死角映射，两条语义差异 D-1/D-2 呈确认，零硬
  阻塞）+ §2 结构主表（元素名/属性开放编码/文本承载/子结构 +
  证据锚点列）+ §3 行内面（mark 包裹元素模型 + 重叠拆分最小区间
  等价覆盖 + 嵌套序确定性规约 + 段内分裂读回合并）+ §4 身份面
  （双轨解耦：BlockId 侧车 ↔ StickyIndex 两步解析 + InsertBlock.id
  物化与载体新 ID 族分轴不污染）+ §5 ops→载体事务七行表 + §6
  差异两条（D-1 SetBlockType 元素重命名子树截断[删旧建新搬子树，
  BlockId 连续载体换族]/D-2 marks 重叠→非重叠拆分[读回语义等价
  非字面]）+ §7 证据锚点与缺口闭环表（已有证据 vs p15/p16/p17
  就绪探针指向）+ §8 裁决联动（XmlFragment 升格路径/Array-of-Map
  降格对照参考 + 方法论迁移面）+ §9 遗留三条。

Stage Summary:
- 批次 1 G1① 裁决的就绪基础件完成：裁决落定即可开实施轮，
  设计文档写作量从「从零」降为「回填验证 + 细化签名」；
- 两条语义差异 D-1/D-2 首次显式成表（此前散在 p13 观察注记），
  进 DECISION_SHEET 批次 1 联动注记候选（R5 汇总轮处理）；
- p15/p16/p17 就绪探针需求清单定稿（§7 第三列），驱动 R2-R4。
---
Task ID: d1p-t2（十轮计划 R2）
Agent: 主代理（D1 就绪系列）
Task: p15 探针——映射矩阵结构实证（yrs-e §2 主表九 kind 形状全量）。

Work Log:
- 新增 src/bin/p15_mapping_structure.rs（注册 Cargo.toml）：
  p15-A1 九 kind 精确形状装载（p/h[level]/pre[lang] 多行原样/
  hr 空/tbl GFM 文本叶/blockquote 自有文本+子块交织/li[marker]
  首行文本+嵌套子项+项内段落）→ 确定性读回签名全等断言；
  p15-A2 wire 快照→新文档 apply 结构相等；p15-A3 属性开放面
  透传（未知键/转义敏感值 a<b>&"'/非 ASCII 零丢失——映射 §2.1
  开放编码不发明白名单实证）；p15-B 全顶层块 StickyIndex 锚定
  → 前插 3 前删 2（删新插者）→ 5 锚点 ID 全等值 + 序保持
  （p13-V3 判定形态泛化到全块）；p15-C 交织容器子块前插/删除
  后序保持（自有文本首位恒定）。
- API 形态对齐 p13 先例（sticky_index 位置锚定 + id() 等值比较 /
  remove_range(start,len) / encode_diff_v1）；修复构建三错
  （括号/usize→u32/字符串类型统一）。
- 门禁：p15 release 跑 ALL PASS（5 断言组）+ 适配层 54/54 零
  回归（matrix25+hostile13+anchor6+limit10）+ 全 15 探针构建
  零错误。

Stage Summary:
- 映射矩阵 §2 结构主表 + §2.1 属性编码 + §4.2 身份基础的实证
  闭环：九 kind 形状全保真（含 wire 往返）、开放键面零丢失、
  全块锚点邻居编辑稳定、交织容器序保持——结构面零缺口。
- yrs-e §7 证据列 p15-A/B/C 行可回填（R5 汇总轮统一处理）。
---
Task ID: d1p-t3（十轮计划 R3）
Agent: 主代理（D1 就绪系列）
Task: p16 探针——操作保真环（7 ops 翻译器微缩 + marks/attrs 编码
+ 对端收敛），含偏移域重大实测发现。

Work Log:
- 新增 p16_mapping_ops（七断言组）：A SetAttrs 终态替换（清旧写新
  非增量合并 + block_id 身份域解耦不触）；B 重叠 marks 拆分读回
  语义相等（em[0,10)∩strong[5,15) → 三段最小区间等价覆盖 → 并集
  还原）；C mark 边界推移（首插+尾插后覆盖随文本扩张）；D
  SetBlockType D-1 形态（删旧建新搬子树 + block_id 连续 + 载体身份
  换新族 StickyIndex ID 不等钉定）；E 七 op 翻译器微缩全环
  （InsertBlock×4 含嵌套 own-text 偏移路径翻译/ReplaceText/SetAttrs
  /SetMark/MoveBlock/RemoveBlock → 结构断言 → wire diff 对端 apply
  收敛 → 增量二轮 SV diff 再收敛）。
- 【重大实测发现】yrs 0.28 XmlText 索引/长度 = **UTF-8 字节域**
  （sem_probe 一次性验证："第一段".len()=15；insert(3) 落在第/一
  字节边界间；部分覆盖删除不整块误删）——非 Text(YText) 的
  UTF-16 域非 char 域。yrs-e §3.1 原「char 域直传」判断**错误
  已修正**：适配层边界 char↔byte 无损换算（ropey 原生支持），
  登记为 D-3 工程项（非语义差异）。临时探针跑毕即删（纪律维持）。
- 修复链：括号/usize→u32/Arc<str> 比较/Out::to_string(txn) 签名/
  XmlFragment 是 trait 非 type（方法来源导入）/split_at 字节域
  panic → c2b 字符→字节换算。
- 门禁：p16 release ALL PASS（7 断言组）+ 适配层 54/54 零回归 +
  全探针（15 个）构建零错误。

Stage Summary:
- 映射矩阵 §5 操作表 + §3 行内面 + §6 差异 D-1/D-2 全部实证闭环；
  D-3 新增（偏移域换算——实施轮边界工程项，零语义损失）。
- 七 op 在 XmlFragment 载体全环成立：结构断言 + 对端收敛 + 增量
  再收敛三层验证；own-text 偏移路径翻译规则（blockquote/li 子块
  +1）钉定。
- yrs-e 文档随实测同步修正（§3.1 重写 + §6 D-3 行 + §7 证据表
  两行已验翻转）。
---
Task ID: d1p-t4（十轮计划 R4）
Agent: 主代理（D1 就绪系列）
Task: p17 探针——撤销栈 B3.2 形态在 XmlFragment 载体复验 + redo
新 ID 族 + 物化解耦两轴钉定。

Work Log:
- 新增 p17_mapping_undo（六断言组）：A1-A4 per-origin 双栈互不干扰
  （human undo 仅撤 human 面 ai 保留 / ai 独立可撤 / 双 redo 跨栈
  序保持全量复原）；B/C redo 复原三合一断言（结构等价 + block_id
  属性保持[BlockTree 身份轴连续] + 载体 StickyIndex ID 换新族
  [协同轴独立]——§4.3 双轴解耦钉定，InsertBlock.id=Some 物化路径
  模拟）；D 混合序列跨栈交叉 LIFO（H/A/H 交替 → human 栈两撤
  LIFO / ai 栈独立一撤 / 三 redo 全量复原）。
- 适配层形态移植：双 UndoManager[human/ai] + include_origin +
  expand_scope(XmlFragmentRef) + capture_timeout=0（每事务一栈项
  ——p4 对照面；1000ms 合并窗语义已由 p4/适配层锁定不重复）。
- 构建修正：XmlFragment 是 trait 非 type → 参数类型用
  XmlFragmentRef。
- 门禁：p17 release ALL PASS + 适配层 54/54 零回归 + 全探针构建
  零错误。

Stage Summary:
- B3.2 撤销栈语义（per-origin 分栈/LIFO/跨栈独立/redo 复原）在
  XmlFragment 载体上与适配层既有 Array-of-Map 载体行为**完全
  同构**——载体更换不动撤销面（正交性实证，批次 1 G2① 裁决的
  载体侧风险清零）。
- redo 新 ID 族经 UndoManager 路径复验（p13-V5 同构）；双轴身份
  解耦（block_id 连续 + 载体换族）钉定——D1 就绪系列证据链齐。
---
Task ID: d1p-t5（十轮计划 R5）
Agent: 主代理（D1 就绪系列收口）
Task: D1 就绪包汇总呈批件 + DECISION_SHEET 批次 1 就绪注记 +
启动条件核验。

Work Log:
- 上游复查（启动条件②）：#676 GitHub API 复查仍 open 零修复零
  评论（updated 2026-09-29）——skip_gc 全编队不变量维持；yrs
  crates.io max_stable = 0.28.0 = 当前钉版（无升级压力）。
- yrs-e §7 证据表回填：8 行全部翻转为已验（p15/p16/p17 结果
  就位）；头部注记补就绪探针完成状态。
- yrs-f_D1就绪包汇总呈批件 落档：§1 四就绪件清单（含实测发现
  三条：D-3 字节域/own-text 偏移规则/B3.2 载体正交性）+ §2
  启动条件四条核验（①边界文档确认=批次1批注即视为确认 ②#676
  已复查 ③④=批次 1 批注——两条闭环两条待批）+ §3 裁决后执行
  预告（载体切换→双向转换器→集成边界，3-5 轮）+ §4 批注影响面
  + §5 遗留三条（不阻塞）。
- DECISION_SHEET 批次 1 节头插就绪注记（四件落档全过 + D-3
  发现 + 批注即开轮）；HANDOFF §6 速览更新（新态前置 + 历史
  保留）。

Stage Summary:
- D1 就绪系列（R1-R5）全部收口：批次 1 裁决的就绪成本降到
  「批注一行」——证据链（映射矩阵 + 三探针 + 上游复查 + 启动
  条件核验）全部在档。
- 实施轮首三步已预告（yrs-f §3），裁决后机械可执行。
---
Task ID: ro2-t6（十轮计划 R6）
Agent: 主代理（渲染线就绪系列）
Task: LO 26.8.1 获取 + R-O2 跨版本钉定复跑（#18 证据前置，批次 3
版本基线裁决证据增量）。

Work Log:
- LO 26.8.1.1 获取：TDF 官方 deb tarball（219,791,834B）下载解包
  独立目录（tool-results/lo26/，不入仓），dpkg-deb -x 全解包，
  --version 验证可运行；与系统 25.2.3.2 零接触（PATH 前置方式）。
- 全套件复跑：22 用例 = 21 维持（10 pass + 11 pinned）+ 1 钉定
  破裂 + 夹具确定性门 pass + 跨进程确定性门 pass。
- 【版本差信号命中】garbage-bytes 钉定破裂（双次复跑稳定）：
  25.2.3.2 = rc=0 + 乱码 PDF 产出（r-o2-t4 发现的静默兜底）；
  26.8.1.1 = rc=1 + 零产物 = **fail-closed 语义改进**（良性方向）。
- anchor-survival 在 26.8.1 同 PASS（内链 GOTO 存活指向书签页
  跨版本稳定——批次 2 联动结论的版本泛化证据）。
- 落档：REGRESSION §10.5（版本差信号命中与定性表 + 处置[基线
  钉定维持 25.2 口径，更新跟随批次 3 裁决] + 复现命令）；
  DECISION_SHEET 批次 3 节插版本差证据增量注记（升 26.x 复跑
  证据已就位：唯一钉定更新面 = 单用例四断言方向利好）。

Stage Summary:
- #18「上游复跑证据先行」的证据**已就位**：升 26.x 不再有证据
  前置缺口，批次 3 版本基线裁决从「需先复跑」变为「直接可裁」；
- 唯一版本差 = garbage-bytes fail-closed 改进（安全面利好非
  风险）；anchor 存活跨版本稳定。
---
Task ID: ro2-t7（十轮计划 R7）
Agent: 主代理（渲染线就绪系列）
Task: 渲染守护器就绪原型——批次 3 实施轮机械项（单实例并发锁/
RSS 看门狗/超时杀组）决策无关预置。

Work Log:
- 新增 render_guard.py（渲染守护器三合一，stdlib only）：G1 单实例
  并发锁（flock 排他 + 阻塞串行化 + 锁超时 fail-closed——内存
  上界 = 单转换 footprint）；G2 RSS 看门狗（/proc 进程树 VmRSS
  后代展开轮询——软限告警不杀/硬限 SIGKILL 整进程组）；G3 超时
  杀组（r-o2-t3 语义守护器化收编，start_new_session 独立进程组）。
- 新增 render_guard_test.py 四断言组：T1 串行化（3 并发真实
  soffice 转换全成功 + 锁等待 [0.704,1.024,0.0]s + 总墙钟 1.39s
  串行证据）；T2 RSS 正常面（软限告警不误杀，峰值 187MB 树 2
  进程）；T3 硬限 fail-closed（合成吞噬者 500MB + 限 200MB →
  峰值 243MB 触杀 rc=-9）；T4 超时杀组（sleep30 + 限时 2s →
  wall 2.015s rc=-9）。ALL PASS。
- REGRESSION §10.6 落档（含产品化映射：套件内联超时/杀组收敛
  守护器单点；单实例锁与 RSS 硬限 = 部署参数挂 SECURITY_POLICY
  限额表；看门狗在飞防护与 wrapper 事后计量互补）。

Stage Summary:
- 批次 3 裁决后的实施轮机械项全部预置完毕——裁决落地时零设计
  缺口（守护器已验证，参数化即产品）；
- 套件文件零触碰（纯新增两文件），R-O2 22 用例面不受影响。
---
Task ID: yrs-up（十轮计划 R8）
Agent: 主代理（上游跟踪轮）
Task: G1 #19 上游跟踪台账 + API 请求候选定稿（呈递时机维持治理
决策，不自行提交）。

Work Log:
- 上游状态快照落档（2026-10-18 API 复查，d1p-t5 同日数据）：
  #676 仍 open（updated 2026-09-29 零评论零标签）/ crates.io
  0.28.0 = 当前钉版无升级压力 / 撤回触发三条零触发（含窗口
  推演：#676 首报 + 6M 窗已过但无同类新增 → 触发①不成立）。
- API 请求候选**三项**定稿（英文呈递稿就绪）：①GC 内容类可见性
  （Update.blocks pub(crate) → 请求 content_kinds() 枚举或
  has_compact_deleted_content() 辅助面——快照门 wire 级复核需求）
  ②撤销栈公开逐出 API（请求 undo/redo_stack_len() 只读观测 +
  pop_oldest(n)/stack_limit 逐出面——B3.4×R-3 未来细粒度上限
  选项）③XML 元素原位改名（p16 D-1 新发现——请求 rename 原语
  或 rename-aware 最小编码，附贡献 PoC 意向）。
- 呈递时机三选项 + 默认建议 a（实施轮启动时合批呈递——单次交互
  成本最低且 ①② 正是实施轮验收门禁辅助面需求）。
- 跟踪纪律三条维持项落档（重置恢复后/启动日复查、新版影响面
  核对、性能触发待基准建成）。

Stage Summary:
- yrs-g 呈递件落档：三项候选英文稿就绪，裁决后（或用户指示）
  即可提交上游；
- DECISION_GROUPS G1 #19 的「择机呈上游」具体化为三选项 +
  默认建议——治理决策入口清晰化。
---
Task ID: docs-audit（十轮计划 R9）
Agent: 主代理（双仓一致性审计）
Task: 单一真源对账 + 版本引用核对 + 跟踪卫生八项审计。

Work Log:
- ① 探针注册对账：Cargo.toml bin 注册 vs src/bin 文件 = 18/18
  一致（p1-p14 + p15/p16/p17 全注册）。
- ② 版本引用核对：cleanroom 文档引用提交号 31 个——28 个 mdcore
  跨仓引用全部存在；3 个（1adced2/5e144cc/ddc51c1）经全分支
  fetch 定位为 **compliance-baseline 远端分支**提交（NOTICES 演进
  链）——**零真悬空**。审计用 origin-mirror 临时 refs 已清理。
- ③ 交叉引用：yrs-e/f/g 三件 + 证据链文件全部在位。
- ④ REGRESSION 章节序：§0-§10 + §10.5（ro2-t6）/§10.6（ro2-t7）
  连贯无断号。
- ⑤ 跟踪卫生：git ls-files 零构建产物（out/target/node_modules）。
- ⑥ worklog：139 个 Task ID 节完整，R1-R8 全登记。
- ⑦ DECISION_SHEET 三批次批注行完好未被注记插坏。
- ⑧ mdcore 终态：工作区干净，main=c4856f7=远端。

Stage Summary:
- 八项审计全过零漂移：双仓文档与现实一致、零悬空引用、零跟踪
  污染、批注入口完好——批次 1/3/4 呈批态有效。
- 审计副产物：compliance-baseline 为远端独立分支（本地单分支
  clone 不含）——跨分支引用核对方法已验证可用。
---
Task ID: final（十轮计划 R10）
Agent: 主代理（十轮收口）
Task: 三大件门禁全量亲跑复验 + 十轮台账 + 终态推送。

Work Log:
- 门禁亲跑（本会话收口态）：mdcore workspace **538/538** 零失败；
  适配层 **54/54**（matrix25+hostile13+anchor6+limit10）+ p15/
  p16/p17 三探针 release 复跑 ALL PASS；R-O2 **22 用例 SUITE
  PASS**（10 pass + 12 pinned + 0 fail，系统 LO 25.2.3.2 基线，
  双确定性门恒过）。
- HANDOFF §6 速览更新至十轮终态（新态前置 + 历史保留）。
- 十轮台账对账：R1 d1p-t1 映射矩阵（2578f2f）/ R2 d1p-t2 p15
  （9e1f8e0）/ R3 d1p-t3 p16+D-3（1fe2f01）/ R4 d1p-t4 p17
  （d525730）/ R5 d1p-t5 就绪包（73f3d44）/ R6 ro2-t6 LO 跨版本
  （e87c44a）/ R7 ro2-t7 守护器（b99b9a8）/ R8 yrs-up 上游跟踪
  （90065a8）/ R9 docs-audit 审计（1d8d0d1）/ R10 final 收口
  （本轮）——十轮十提交全推送。

Stage Summary:
- 十轮计划全收口：批次 1 就绪包（证据链完整批注即开实施轮）+
  批次 3 证据就位（26.8.1 复跑完成 + 机械项守护器预置）+
  上游候选定稿 + 双仓零漂移 + 门禁全绿。
- mdcore 零接触维持（c4856f7）；全部工作在分析仓 spike/文档面
  （d1e-t3 先例纪律）。

---
Task ID: recover-18（第 18 次重置恢复与停机复验轮）
Agent: 主代理（新会话「继续」指令恢复轮）

Task: 环境第 18 次重置后双仓恢复 + 工具链重建 + 推送终态全门禁
复验 + 停机条件核验 + 落档推送。

Work Log:
- 发现与恢复：本地 log 顶 = ae4aa4d（回滚态）且 status 假 ahead 2
  ——瞬态 URL 推送陷阱特征。fetch 核实远端真实态：origin/main =
  **defc055（十轮收口）**，即「至少10轮」任务上一会话已完整完成
  （R1-R10 十提交链：2578f2f/9e1f8e0/1fe2f01/d525730/73f3d44/
  e87c44a/b99b9a8/90065a8/1d8d0d1/defc055 全推送）。ff 恢复本地
  main→defc055，树净。mdcore=c4856f7 同步树净零接触维持。
- 工具链重建（重置抹除 cargo/rustc）：rustup 1.99.0 minimal +
  clippy/rustfmt + wasm32-unknown-unknown target +
  nightly-2026-10-15[rust-src] 全装；双仓 core.fileMode false；
  LO 25.2.3.2 系统基线存活（与钉定基线一致零重装）。
- 全门禁恢复复验（重建环境亲跑）：适配层 **54/54**（matrix25 +
  hostile13 + anchor_undo_redo6 + limit_boundaries10）；mdcore
  workspace **G1 538/538 零失败**；clippy 双面恰 2 wasm_bindgen
  基线（lib.rs:212/221 static_method_of——grep 4 行命中甄别 =
  2 真警告基线 + 2 摘要行，零增量）；fmt 0；p15/p16/p17 release
  复跑 ALL PASS；R-O2 **22 用例 SUITE PASS**（10 pass + 12
  pinned + 0 fail，夹具确定性门 + 跨进程确定性门双过，系统 LO
  25.2.3.2 基线）；scan.sh 双口径（product / --all）0 hit-paths
  PASS。双仓树净 = Cargo.lock 零 diff 蕴含。
- #676 重置恢复复查（跟踪纪律三条之「重置恢复后复查」）：匿名
  API 限流 403 → 瞬态 token 认证复查（零落盘）——state=open /
  updated=2026-09-29（与 yrs-g 快照零漂移）/ comments=0；
  crates.io yrs max_version=0.28.0=钉版维持。撤回触发三条零触发
  （skip_gc 不变量维持）。
- 文档落档：HANDOFF §6 新态速览前置（第 18 次重置恢复 + 停机
  复验；历史速览全保留）；本 worklog 条目。

Stage Summary:
- 第 18 次重置恢复完成：重建环境完整复现推送终态（defc055），
  全门禁亲跑全绿零漂移；十轮计划收口态维持有效。
- 停机条件维持：类别 1（决策无关可执行面）清空——批次 1 就绪包
  完整、批次 3 证据就位、上游候选定稿、审计八项全过；剩余全部
  = DECISION_SHEET.md 批次 1/3/4 用户裁决门控，按纪律如实报告
  不伪造工作。
- 批次 1 批注即开实施轮（载体切换→双向转换器→集成边界，预告
  3-5 轮）；批次 3 批注即开渲染管线轮（26.8.1 复跑证据 +
  render_guard.py 守护器已预置）。

---
Task ID: a5-r1（五轮自主计划 R1：里程碑标签抗重置轮）
Agent: 主代理（用户「继续至少5轮低成本自决策别问我」指令）

Task: 双仓已验证态打注释标签推送——抗重置可恢复性强化。

Work Log:
- 环境核查：无重置（双仓同步树净，工具链/LO 存活）——直接执行。
- cleanroom：tag `checkpoint/2026-10-19-batch1-ready` @ 2690bec（十轮
  收口 defc055 + recover-18 复验态；标签信息含全门禁绿在档与批次
  1/3/4 待批注状态）。
- mdcore：tag `checkpoint/2026-10-19-pre-batch1` @ c4856f7（批次 1
  实施前基线；标签信息含 G1 538/538 与全静态门禁态）。
- 双标签瞬态 URL 推送 + ls-remote 核实：注释对象（eec135db /
  0bb09dcb）与解引用提交（2690bec / c4856f7）均确认在远端。

Stage Summary:
- 抗重置强化：环境再重置时，两仓已验证态可经标签名一步定位
  （不必依赖 log 检索）；标签自带门禁证据摘要。
- 零代码接触：纯 ref 面操作，双仓树净维持，无门禁影响。

---
Task ID: a5-r2（五轮自主计划 R2：p18 D-3 换算完备 PoC）
Agent: 主代理

Task: 把 p16 的单向临时 c2b 升格为完备双向偏移域换算模块 PoC
（批次 1 已登记工程项 D-3 的实施前置）。

Work Log:
- 新探针 `p18_char_byte_conv.rs`（bin 注册第 19 号）：c2b（char→
  字节，越界显式 None + 末哨兵 == len）/ b2c（字节→char，三态
  Err：byte-out-of-range / mid-codepoint / Ok）双函数。
- 五面 12 断言组全部 PASS：A1 六语料精确对表（空/ASCII/BMP/
  非 BMP/组合符/混合——含手算真值全表）· B1-B4 活体 XmlText
  一致性（len=13 byte ≠ 6 char ≠ 7 UTF-16 三域分立复证 + 读回
  零畸变 + c2b 偏移 insert 落点 + remove_range 字节域口径）·
  C1-C3 全位置穷举（往返恒等 + 末哨兵 + 非边界字节全数拒止零
  误拒）· D ReplaceText 真实路径四案（char 区间→字节区间删除
  插入与 Rust 侧 char 域真值逐字相等——含区间内多字节/边界恰
  多字节/全清空/ASCII 退化）· E1-E3 错误面三态。
- 【实测发现 p18-F】yrs 0.28 `TransactionMut::commit()` 不释放
  写锁——锁随 Drop 释放；同作用域保活已 commit 写事务再开任何
  事务 = 死锁挂起（首版 B2 踩中，分步标记定位，独立块作用域
  重构后全绿）。实施轮硬约束登记。
- 向量表一处真值笔误自纠（nonbmp 3 字符误写 4 行 → c2b(4)
  期望越界 None 实测揭穿后修正）。
- yrs-e 增补：§7 D-3 证据行回填 p18（完备模块 PoC 已验）+
  §9 增补 4/5（p18 函数形态采纳建议 + p18-F 事务锁纪律）。
- 门禁：p18 release ALL PASS 零警告；p18 文件 rustfmt 过
  （spike 仓无 fmt 门禁史——adapter/lib.rs 既有格式差异非本轮
  触碰未动）；适配层 54/54 零回归。

Stage Summary:
- D-3 工程项从「p16 两断言佐证」升格为「完备模块 PoC + 实施轮
  直接采纳形态」——批次 1 就绪包证据面再加固。
- p18-F 事务锁死锁发现为批次 1 实施轮新增硬约束（写事务作用域
  纪律），已登记 yrs-e §9-5。

---
Task ID: a5-r3（五轮自主计划 R3：p19 撤销栈内存量化）
Agent: 主代理

Task: 已知风险 #2（撤销栈无界增长）从定性升格定量——G3 栈上限
裁决证据前置。

Work Log:
- 新探针 `p19_undo_stack_mem.rs`（bin 注册第 20 号）：计数分配器
  （GlobalAlloc 包装 System，原子计数存活字节）。
- 量纲设计：同一 N=30k 操作序列（XmlFragment 载体 + 双 origin
  交替 + 每操作独立事务 = 每事务一栈项最坏粒度），带双
  UndoManager 追踪 vs 无追踪的存活字节差。
- 六断言组 ALL PASS：A1 追踪严格大于基线（Δ=13,137,152 B）·
  A2 per-op 合理带 · B1 3 万操作后 undo 20 次计数精确
  （30000→29980）· B2 redo 5 复原（→29985）· C1 undo 200 项
  后存活字节 ratio=1.000（栈保留 + tombstone 不释放）· C2 首
  块文本零畸变。
- **核心数字：撤销栈成本 437.9 B/操作（最坏粒度）→ 6M 操作窗口
  投影 ≈ 2.4 GiB/端/文档**（不含文档本体）。捕获窗合并显著
  降低该量级（真实会话同 origin 1s 窗合并——投影为上界）。
- DECISION_GROUPS G3 追加 p19 定量证据增量（仅证据面，推荐
  结论不动——G3 仍待用户批注）。
- 门禁：p19 release ALL PASS 零警告 + rustfmt 过 + 适配层
  54/54 零回归。

Stage Summary:
- 风险 #2 定量在档：维持无限选项的内存上界公式有量级（文档
  大小 + 437.9 B × 追踪操作数[上界]）；G3 三选项裁决的证据面
  完备化（上游 API 请求候选 #2 的 stack_len/pop_oldest 语义
  需求亦有数字支撑）。
- 栈保留性格实证（undo 不释放内存）确认 B3.4 知情面的「已消耗
  仍在栈」认知。

---
Task ID: a5-r4（五轮自主计划 R4：soak 收敛轮——猎获 yrs 0.28 新缺陷）
Agent: 主代理

Task: 适配层随机化长跑收敛测试（确定性种子 PRNG，零三方依赖）。

Work Log:
- 新测试文件 `adapter/tests/soak_convergence.rs`（xorshift64* 种子
  PRNG）+ 新探针 `p20_undo_window_repro.rs`（bin 第 21 号）。
- soak-1 双端随机并发（60 轮 × 每端 8 随机操作：四变体流编辑 ×
  双 origin × 穿插随机 undo/redo，每轮双向同步后三面全等断言 +
  每 20 轮幂等复验）PASS。
- 【重大发现】soak-2 初版（全量复原不变量）失败 → 最小观测 →
  **yrs 0.28 新缺陷定谳（p20 四配置矩阵）**：双 UndoManager 各自
  track 不同 origin + capture_timeout>0 时，同窗内同 origin 编辑
  被其他 origin 隔断形成**非连续合并项** → ① undo 仅应用首子区间
  ② 后续 redo 在 Store::materialize panic（unwrap None，经
  pop_blocking）。干净对照：双 mgr+窗0 / 单mgr+窗1000 / 单mgr+窗0
  全部正确。**适配层现产配置（1000ms 窗）公开 API 今日可达**（交错
  origin 同窗编辑 + undo + redo = 进程 panic）。
- soak-2 重设计为长跑收敛稳定（3000 操作无撤销面：追踪面活跃 +
  双端收敛 + 幂等 + gc 存活）PASS；soak-3 = 缺陷钉定测试
  （catch_unwind 钉 panic + 终态断言钉部分撤销——修复/改配置后
  必失败强制解钉）PASS。
- 过程自纠两件：①p20 首版死锁挂起——get_or_insert_text 在事务内
  调用（p18-F 同族：先取根后开事务纪律）；②soak-1 150 轮 205s
  过慢 → 60 轮 30s（门禁可常跑）。
- 文档：yrs-g §5 新增缺陷候选 #4（英文呈递稿 + 最小复现 + 处置
  三选项 A[推荐]窗改0/B 维持+禁面/C 知情维持）；DECISION_GROUPS
  G2 增补 p20 呈批输入（撤销粒度语义 = 产品决策，与 #4② 同批或
  先行批注）。
- 门禁：全套件 **57/57**（54 既有 + 3 soak）零失败；soak 文件 +
  p20 rustfmt 过。

Stage Summary:
- 缺陷猎获入档：#676 之外第二个 yrs 0.28 缺陷（undo 捕获窗 × 多
  manager），适配层现产配置可达、复现矩阵干净、上游呈递稿就绪、
  钉定测试每跑必红。
- 决策面新增呈批输入：G2/B3.2 捕获窗处置三选项（推荐 A 窗改 0
  ——cfg-2 实证干净；p19 粒度成本 437.9 B/操作转常态的取舍入注）。
- 测试面 +3：随机并发收敛 / 长跑稳定 / 缺陷钉定。

---
Task ID: a5-r5（五轮自主计划 R5：上游生态快照 + 五轮收口）
Agent: 主代理

Task: 上游生态增量快照 + p20 缺陷上游比对 + 五轮终收口。

Work Log:
- 上游生态（瞬态 token，零落盘）：#676 仍 open / updated
  2026-09-29 零漂移；0.28.0 后新提交 2 项（#681 ywasm 面不影响
  本仓 + aaa1f50 fuzzy 死锁修复已在钉版内）；crates.io 0.28.0
  维持无新版。
- p20 缺陷上游比对：issues 搜索**未发现同缺陷报告**——最近邻
  #674（redone 项邻居合并 follow_redone 过删，yjs#806 对应）
  机制不同（p20 无 redo 链参与，首撤销即部分应用）；#665 另族。
  **p20 = 独立新缺陷确认**。UndoManager 缺陷家族图谱：#665/
  #666/#667/#674/#676/p20。
- yrs-g 更新：§1 快照 2026-10-19 增量 + §5 上游比对结论与家族
  地图。
- 收口门禁亲跑全绿：适配层全套件 **57/57**（matrix25+hostile13
  +anchor6+limit10+soak3）+ p15/p16/p17/p18/p19 ALL PASS +
  p20 观测矩阵完成（cfg-1 缺陷表现稳定复现）。
- HANDOFF §6 五轮终态速览前置（历史保留）。

Stage Summary:
- 五轮自主计划全收口：R1 标签 / R2 p18 D-3 完备化 + p18-F 事务
  锁纪律 / R3 p19 撤销栈定量（437.9 B/op · 6M≈2.4 GiB）/
  R4 soak 三测试 + yrs 0.28 新缺陷定谳钉定呈批 / R5 生态快照
  与上游比对（p20 独立新缺陷确认）。
- 呈批面更新：G2 新增 p20 捕获窗处置三选项（推荐 A 窗改 0）；
  批次 1/3/4 维持待批注。全部工作在分析仓（mdcore c4856f7 零
  接触维持）。

---
Task ID: s0-t2/s0-t3（修订实施计划 S0：协作缺陷缓解 + 资源清单）
Agent: 主代理

Task: 用户 2026-10-21 修订实施计划（S0-S5）S0 阶段——工程收口与风险
止损：仓库状态核实、p20 复现与缓解（捕获窗→0）、交错回归补齐、
粒度代价记录、撤销栈量化与低成本限制验证、一页资源限制与缺口清单。
附加用户澄清：标书证据系统只是需求之一，优先通用基础功能（S1 重排
为通用数据契约/sidecar/薄命令层先行，证据闭环为首个消费者）。

Work Log:
- 环境第 19 次重置恢复：rustup 全失→重装 1.99.0（clippy/rustfmt）
  + nightly-2026-10-15 后台；cleanroom 本地 main 假领先（重置回滚
  fetch）→ 认证 fetch + ff 恢复 7ac6b9e；mdcore c4856f7 零接触核对。
- 修复前复现（S0-任务2）：钉定测试 debug 口径确定性失败——新观测
  第三副面孔：undo 阶段 Item::splice `clock+offset-1` u32 下溢当场
  panic（block.rs:652）；release 口径复现部分撤销+redo panic（上会话
  57/57 绿为 release 口径——口径差揭穿：debug 抓下溢/release 抓静默
  回绕）。机制统一：合并非连续栈项 undo 应用时 offset=0/clock=0 下溢。
- 修复（S0-任务3）：adapter lib.rs capture_timeout_millis 1000→0
  （硬编码非配置面）；仅协作路径——mdcore COALESCE_MS 零接触。
- 测试改造：soak-3 解钉→dual_manager_interleaved_undo_redo_full_cycle
  回归锁（逐事务粒度/LIFO/origin 隔离/redo 全复原/双 origin 交叉
  全环）；soak-4 新增 blocks 载体交错回归（首版断言 LIFO 序算错
  自纠：撤完删除后次项=插 h2 而非 h1）；matrix 窗合并测试反转锁
  逐事务粒度 + boundary 测试并入（窗语义消亡）；anchor 三处 1100ms
  sleep 移除（断言零改动全过）。p20 探针复跑留证：cfg-1 原始 yrs
  层仍 panic=true（上游未修，适配层免疫）。
- 门禁：57/57（matrix24+hostile13+anchor6+limit10+soak4）debug+
  release 双口径零失败；clippy 增量零新（存量 16-18 登记）；fmt
  全仓漂移为历史存量（spike 从未 fmt 门禁——探针代码=归档证据，
  清收留产品化轮）；token 零落盘（env 单次传递，grep 验证）。
- 一页清单（S0-任务5/6）：collab-line/S0-资源限制与缺口清单.md——
  已有六限制/撤销栈定量 437.9B/op·6M≈2.4GiB/低成本限制验证结论
  （无合规逐出面，监控就绪，上游 API 呈递稿在档）/粒度代价与五条
  未覆盖路径（诚实边界：修复已知触发条件≠全面安全）/缺口六项按
  风险排序。

Stage Summary:
- S0 六项全收口：仓库状态✓（双仓同步零未提交）/复现✓（三副面孔
  机制统一）/缓解✓（窗0+回归锁双口径）/代价记录✓/栈定量与限制
  验证✓（维持无限+监控，不虚报不清空）/一页清单✓。
- 决策面变化：G2 p20 选项 A 已裁已执行（用户 S0 指令原文）；批次 1
  解锁（XmlFragment/B3.2/栈上限）被修订计划 S3 后置取代——协作
  产品化等真实多端需求，不再阻塞主线；批次 3 渲染产品化后置 S4。
- 下一阶段：S1 通用基础优先——DocumentRef/Locator/Command/
  AuditEvent 最小契约 + 本地 sidecar 存储 + 薄命令层（幂等键/预期
  版本/路径允许列表），证据系统作为首个消费者落闭环；不预设 UI。

---
Task ID: s1-r1（修订计划 S1 第一轮：mdc-evidence 契约层）
Agent: 主代理

Task: S1 通用基础第一轮——最小稳定数据契约 + sidecar v1（用户澄清
落位：通用基础优先，证据系统为首个消费者，不为标书特化）。

Work Log:
- 落位决策：产品主线已裁（修订计划 governing）→ 产品仓 mdcore 新
  crate crates/evidence（workspace 第九成员，循 office 系三域纪律：
  对其他 crate 零 path 依赖；纯 serde 依赖零新三方——NOTICES 零增量
  是选型约束：指纹走 trait + FNV-1a 64 默认而非引 sha2，算法前缀
  编码 + 跨算法恒不等保证升级路干净）。
- 契约面：refs（DocumentRef 四元组 + 算法前缀强制）/fingerprint
  （trait + 公开向量钉死）/locator（五变体 + A1 域校验镜像 office-io
  读面域）/status（三态正交——单字段 verified 类型级排除 + Valid
  才可复算级联门槛）/calc（i128 定点十进制 checked_* 显式错误算术，
  三舍入模式，除法双路径含 e′<0 安全降标论证）/evidence（Evidence/
  Calculation 双记录 + 构造不变量 + degrade 单向）/command（信封 +
  幂等键域 + 预期版本三态 + 五结果）/audit（追加式序号）/sidecar
  v1（schema 门/原子写/失效登记即检测/显式重验唯一恢复通道/幂等
  重放表）。
- 自纠三件：div 首版移位代数 e′<0 分支错误 + 双重舍入（重写双路径
  + 论证）；Ceiling 舍入负数方向歧义（砍掉该模式只留三个无歧义项）；
  mark_source_unavailable 的 debug_assert 在测试故意传非法态时崩溃
  （改状态机单向语义保证，不靠断言）。
- 门禁（mdcore 全套亲跑）：G1 538→**564**（+26 恰为契约层单测）
  零失败；clippy 四面恰 2 wasm_bindgen 在档基线零新增；fmt 全仓 0；
  scan 0 hit-paths PASS；wasm32 check --all-targets 过；Cargo.lock
  +8 行新包块；NOTICES 零变化；web 面零触碰。

Stage Summary:
- S1 通用基础第一轮收口：数据契约 + 状态语义 + sidecar 存储全落地，
  §四.5 铁律（指纹不匹配只降不升/显式重验唯一恢复/绝不静默重绑）
  从文档语义变为类型与测试钉死的代码语义。
- mdcore main c4856f7→8cd97cf 已推送。
- 下一轮（s1-r2）：表达式白名单求值器（数量×单价/求和/比例/单位
  换算四式 + CalcStatus 复算闭环）→ s1-r3 合成样例端到端八步闭环。

---
Task ID: s1-r2（修订计划 S1 第二轮：表达式白名单求值器 + 复算闭环）
Agent: 主代理

Task: S1 §四.4 受限确定性公式子集——白名单求值器（四式覆盖）
+ CalcStatus 复算闭环 + sidecar 级联执行点。

Work Log:
- mdcore crates/evidence 新模块 expr.rs（~570 行含测试）：
  白名单文法（NUMBER/IDENT/+-*/括号/单层一元负号；函数/幂/字符串/
  比较/科学记法/非 ASCII 标识符全部显式拒绝——白名单=文法本身）。
- 四式覆盖测试：数量×单价（qty*price=4210.00 + 轨迹两行）、求和
  链（左结合两步轨迹）、比例（part/total*100——除法按记录级精度，
  precision 2 vs 6 对照钉定确定性随精度走）、单位换算（因子字面量
  ×10000；因子正确性归人工审核——单位代数不发明）。
- 资源四限 fail-closed：表达式 256B/原子 32/括号深度 12/输入 16
  （恰过恰拒边界测试）。
- 单位面诚实边界：加减 operands 已声明且不同 → UnitMismatch 硬拒；
  乘除复合串（件*万元/万元）仅报告 computed_unit 不判定；结果单位
  与声明单位对照不机器验证（无单位本体）。
- recalc 四步链：InputUnavailable（含长度不齐）/InputMismatch
  （跨标度数值相等不算不符）/EvaluationError/PrecisionMismatch
  （篡改登记值或精度规则被改均捕获）；final_value=None 不可复算
  通过（先 evaluate+apply_evaluation 落值）；intermediates 刻意
  不比对（呈现层可演化，权威锚=值；threat model 本地可信存储）。
- evidence.rs 加性扩：claimed_value 字段（#[serde(default)] +
  legacy v1 JSON 加载测试）+ Calculated 型必填（新错误码
  CalculatedMissingValue）。sidecar.rs：apply_evaluation（拒覆写
  AlreadyEvaluated）+ recalculate_calc（失败同因级联/通过对照
  claimed_value → ResultMismatch 第五步）+ 审计两事件。
- 自纠三件：①ratio 测试期望值算错（0.807984×100=80.798400）
  ②深度边界差一（>= 改 >，12 层恰过 13 层拒）③sed 转义 & 损坏
  测试行（Edit 修复）。doctest 误捕文法块 → ```text 标记。
- 门禁（亲跑）：G1 564→**588**（evidence 26→50，+24 恰为求值器
  与复算测试）零失败；clippy native+wasm32 双面恰 2 wasm_bindgen
  在档基线零新增（useless_vec 一处即改）；fmt 0；scan 0 hit-paths
  PASS；Cargo.lock 零 diff 零新三方（NOTICES 恒）。

Stage Summary:
- §四.4 "计算保留输入公式精度可复算" 从契约语义变为可执行语义：
  evaluate（确定性）+ recalc（四步）+ sidecar 级联（五步含正文
  主张值对照）全链落地。
- mdcore main 8cd97cf→c529b49 已推送。
- 下一轮 s1-r3：工作台编排层（mdc-workbench 新 crate，依赖倒置
  注入 office-io 读写）+ 合成样例八步端到端闭环（验收门禁 12 条
  显式定义并逐条映射测试）。

---
Task ID: s1-r3（修订计划 S1 第三轮：工作台编排层 + 八步闭环）
Agent: 主代理

Task: S1 收口轮——mdc-workbench 编排层（新 crate）+ 合成样例八步
端到端闭环 + 验收门禁 12 条显式定义与测试映射。

Work Log:
- mdcore 新 crate crates/workbench（第十成员，2041 行）：ingest
  （XLSX 语义指纹 = 非空格 sheet|A1|值表示序——样式噪声不失效；
  DOCX 段落 trim 规范化指纹；仅段落计号对齐 DocxParagraph 口径）
  + pipeline（Workbench 会话：start/open/ingest_source/
  propose_binding/propose_calculation/recalc_current/refresh_body/
  reverify_source/reverify_claim/review/export_clean）+ report
  （三态分列 Markdown，刻意无汇总列）。
- 三条结构性纪律：正文零写入（干净导出=先三验证[可解析+零标记
  +字节同一]后产出）；AI 提议/程序验证（数值/指纹/摘录全部程序
  从真实文件提取，fail-closed 三前置：正文/来源指纹 fresh + 目标
  存在 + claimed_value 必须在场段落文本）；无任意路径/SQL/Shell。
- evidence crate 增量：CalcInput.source_doc_id 加性字段（v1 数据
  serde(default) 加载不破）+ sidecar 四方法（review_evidence 迁移
  矩阵含复位拒绝 / refresh_document claim 锚失效 / reverify_claim
  对称恢复通道 / push_audit_public）+ recalculate_calc 补 linkage
  门槛（非 Valid 级联 InputUnavailable——来源不实不许复算通过）
  + 五项再导出。
- tests/eight_step_loop.rs 四集成测试全绿：主闭环（4210.000 目标
  标度 3 含 rescale 轨迹 + 来源改写 Stale 级联 + 复位拒绝 + 删除
  sidecar 正文照常）+ 全恢复路径（重验→InputMismatch→旧证拒→
  新计算 5000.00→正文更正 claim 锚断→新证→通过→过审；审计五
  事件 + 序号单调）+ fail-closed 八拒绝路径 + 导出标记检测
  （污染被抓 + 失败零产出）。
- 自纠三件：①export_clean 先写后验违反"失败不产出"→ 重排为先
  验证后落产物；②C7 读回 336.8（scale 1）非 336.80——主循环精度
  改 target_scale 3 以覆盖 rescale 轨迹 + 跨标度相等闭环；③
  WorkbookWriter::new 非 Result（测试夹具修正）。
- 报告落档：workbench-line/S1-证据系统MVP验收报告.md——验收
  门禁 12 条逐条映射测试 + 三项选型决策显式登记（①sidecar JSON
  非 SQLite：零新三纪律+一文档一文件可携带+MVP 规模够用，升级
  路径 Store 抽象——**待用户追认**；②PDF 只读契约就位接入后置
  S4；③FNV-1a 非密码学，威胁模型=意外失效）。
- 门禁（亲跑）：G1 588→**595**（evidence 50→53 + workbench
  0→4）零失败；clippy native+wasm32 双面恰 2 基线；fmt 0；scan
  0 hit-paths；wasm32 check --all-targets 过；Cargo.lock +9 行纯
  新成员块零新三方（NOTICES 恒）。

Stage Summary:
- S1 三轮全收口（r1 契约/r2 求值器/r3 编排+闭环）：八步闭环端到
  端可运行，§四.3/§四.5 全语义（双侧失效/双侧重验/三态正交/
  AI 提议程序验证/人工归宿）从计划文本变为类型与测试钉定的代码
  语义。
- mdcore main c529b49→40e65a7 已推送。
- 待用户面：sidecar JSON vs SQLite 选型追认（报告 §3.1）。
- 下一轮 s2-r1：S2 写入安全（路径允许列表/命令信封分发/发布
  单点纪律/规模限制），随后 S3/S4/S5 决策记录轮 + 终收口。

---
Task ID: s2-r1 + s3s4s5（修订计划 S2 实施 + S3/S4/S5 决策记录 + 终收口）
Agent: 主代理

Task: S2 写入安全四件（路径允许列表/幂等/临时区发布/规模限制）
+ S3/S4/S5 阶段决策记录（计划原义 = 决策轮非实施轮）+ 停止条件
核对。

Work Log:
- security.rs：PathPolicy（canonicalize 符号链接逃逸拒；check/
  check_dest 读写两面）+ 规模门（证据 1000/计算 500/16MiB 字节门
  ——超限拒发布旧文件不动）+ publish 发布单点（同目录临时文件 +
  rename POSIX 原子）。
- command.rs：WorkbenchCommand 受控命令集（八步枚举，无任意路径/
  SQL/Shell）+ execute 分发——幂等重放返首果 + 审计；Exactly 预期
  版本乐观并发；记录规则（Succeeded/Failed 入表，Conflict/Rejected
  不入不毒化重试）；错误映射（路径/规模→Rejected）。
- pipeline 接线：start/open 强制 policy 五路径面全检（含
  fresh_source/recalc_current 注册路径复查——符号链接改指根外
  fail-closed 级联 Unavailable）；条数门前置；save 走发布单点；
  reverify_claim 工作台方法补齐（fail-closed + 程序提取指纹）。
- tests/command_layer.rs 五测试：根外四路径全拒（含拒绝零产出）/
  符号链接逃逸拒 / 幂等-版本语义全矩阵（重放零副作用、冲突不毒化
  重试、MustBeNew、失败重放同果）/ 八步全走命令分发面 / 规模四门
  恰过恰拒。既有四测试升级受限策略构造（安全面即默认面）。
- 时间面决策：结构性有界（输入字节门+表达式资源门+条数门三链），
  不设墙钟计时器（非确定性 + wasm 不可移植）。
- S3/S4/S5 决策记录（workbench-line/S2-S5收口与阶段决策记录.md）：
  S3 协作产品化等真实多端需求（S0 缓解已执行 + 重启条件两条）；
  S4 渲染不阻塞不捆绑（R-O2 资产在档 + C 路线重启条件）；S5 Canvas
  后置（Locator 加性扩展位）。均无代码——计划原义。
- 停止条件核对：12 门禁全过（S1 报告 §2 + S2 门禁 11 升级为分发面
  钉定）+ 提交推送（mdcore f6aa9e7）+ 交接完整（HANDOFF/worklog/
  双报告）→ 满足计划 §十停止条件。
- 门禁（亲跑）：G1 595→**600**（workbench 4→9）零失败；clippy
  native+wasm32 双面恰 2 基线；fmt 0；scan 0 hit-paths PASS；
  Cargo.lock 零 diff 零新三方。

Stage Summary:
- 修订实施计划 S0-S5 全部阶段收口：S0 止损 / S1 三轮 MVP /
  S2 写入安全 / S3-S5 决策记录。产品线 = mdcore 十 crate（新增
  evidence + workbench，零新三方依赖），G1 538→600。
- 呈用户三项待裁决：①sidecar JSON vs SQLite（S1 报告 §3.1）
  ②S3/S4 重启条件 ③前端选型与 wasm 接入节奏。
- 按计划停止条件停。后续会话从 HANDOFF §6 顶部速览恢复。

---
Task ID: wrap-t1（收尾轮：JSON sidecar 持久化与异常恢复验证）
Agent: 主代理

Task: 用户新指令第一步——验证 JSON sidecar 的持久化与异常恢复，
运行 mdc-workbench 八步闭环及必要门禁；环境第 20 次重置恢复
（双仓 ff：cleanroom ae4aa4d→63c5528 / mdcore c4856f7→f6aa9e7；
工具链重装 1.99.0 + clippy/rustfmt + wasm32 + nightly-2026-10-15）。

Work Log:
- 验证实抓缺陷：`Workbench::open()` 恢复保真三缺口（此前零测试
  覆盖——open 路径从未被任何测试调用）——body_doc_id 置空（重开
  后 refresh_body 会把空 doc_id 写进 sidecar.document）/ body_version
  硬编码 1（重开后版本回退与历史登记碰撞）/ SourceRegistration
  .version 硬编码 1（来源重登记版本碰撞）。修复 = 全部从 sidecar
  单一真源恢复（document.doc_id / document.version / source_docs
  按匹配恢复，未登记默认 1）。
- tests/persistence_recovery.rs 六测试（反向验证成立：git stash
  还原旧代码恰使版本单调与 doc_id 两测试失败）：①会话重开全量
  状态保真（证据/计算/审核/审计序号/幂等表）+ 重开后继续操作
  （复算 Passed / refresh_body 版本 2→3 单调续增 / 干净导出字节
  同一）②跨会话幂等重放（Replayed 返首果 + 零副作用 + 
  command_replayed 审计在档）③来源版本跨会话单调（v2 重开后再
  登记恰 v3；新来源首登记 v1 不受影响）④截断/损坏/未来版本/缺
  文件四异常 fail-closed（显式 Sidecar 错误，不静默新会话——那
  等于丢证据库；完好文件恢复加载正常）⑤中断发布 .publish.tmp
  残留不破加载，下次发布原子覆盖零残留 ⑥会话间正文改写 → 重开
  提议 BodyStale（指纹持久化生效），refresh_body 后恢复可提议。
- 门禁口径修正（存量发现登记，非本轮引入）：f6aa9e7 上 wasm32
  --all-targets 实存编译错误（command_layer symlink 测试 
  std::os::unix）与 drop_non_drop×2 漂移——上轮"wasm32 check
  过"口径不含 --all-targets 全测试面。本轮修复：symlink 用例
  #[cfg(all(test, target_family = "unix"))] 门控（非 Unix 编译面
  排除，宿主行为面不变）+ 三测试夹具 drop(sh) 改作用域块
  （SheetWriter 借用语义等价）。
- 八步闭环复跑：eight_step_loop 四集成测试全绿（主闭环 4210.000
  + 来源改写 Stale 级联 + 复位拒绝 + fail-closed 八拒绝路径 +
  导出标记检测）。
- 门禁（亲跑）：G1 600→**606** 零失败 / clippy native + wasm32
  --all-targets 双面恰 2 static_method_of 基线（mdcore-wasm 既有）
  零错误 / fmt 0 / scan 0 hit-paths PASS / wasm32 check
  --all-targets 零错误 / Cargo.lock 零 diff。

Stage Summary:
- mdcore main f6aa9e7→**1db4a6c** 已推送（wrap-t1）。
- sidecar JSON 选型（S1 报告 §3.1 待追认项①）现在有完整持久化与
  异常恢复证据链：原子发布/版本门/损坏拒/跨会话幂等/恢复保真
  全部测试钉定。
- 下一步（用户新指令）：协作引擎正式实施（适配层产品化进
  mdcore——undo/redo 已知缺陷处置 + 内存边界 + 命令接口 + 并发
  + 对象身份 + 故障恢复 + 端到端原型），随后 Canvas 引擎。

---
Task ID: collab-r1（协作引擎正式实施第一轮：mdc-collab 产品化落地）
Agent: 主代理

Task: 用户新指令②——协作引擎实施：在现有 yrs 适配器基础上继续，
先处理 undo/redo 已知缺陷和内存边界，再接入实际文档操作与共享状态；
人类和 AI 共用命令接口、并发修改处理、对象身份、撤销/重做及故障
恢复；可自动验证的端到端原型（不停留在设计文档和探针阶段）。

Work Log:
- mdcore 新 crate crates/collab（第十一成员，产品面唯一 yrs 接触
  点——公开 API 零 yrs 类型）：spike 适配器（997 行研究代码）全部
  证据链产品化 + yrs-e 映射矩阵形状落地（XmlFragment 载体、九
  kind 元素承载、开放属性 _xHHHH_ 转义、mark 包裹元素 D-2、
  char↔byte 换算 D-3 单点）。
- CollabCommand 九命令（LoadBlocks/InsertBlock[id 物化面]/
  RemoveBlock/ReplaceText/SetBlockType[D-1]/SetAttrs[终态钦定]/
  SetMark[归一化重建]/MoveBlock[环回拒]）——人类/AI 同一入口，
  每命令一事务一 origin。
- Actor 双撤销栈 + 三处上游观察面的产品收口：①每栈只跟自己
  origin + capture_transaction 拒 None-origin（远端应用不入本地
  栈——撤销域=本地；spike len==1 回退致 p12"对端墓碑 2%"）②
  撤销栈软上限 undo_stack_limit（B3.4 粗粒度 clear_all 显式回执
  undo_cleared——清理时置标志，事后重算不可见）③**撤销耦合防护
  （实施新发现）**：文本承载块出生恒带空文本节点 + SetMark 重建
  尾随空节点——否则首笔编辑才建节点时 undo 首笔=删整个节点含
  后续并发内容（探针实证 'hahahaha' 撤 4 笔得 '' 而非 'aaaa'，
  修复后恰 'aaaa'）。
- 事务纪律修复：set_block_type 读事务遮蔽死锁（yrs 读写事务同锁，
  commit 不消费借用随 Drop 释放——全部 op 事务显式作用域化）。
- 身份双轨：block_id 属性承载（远端块免侧车寻址）× CrdtAnchor
  （sticky 首项 ID；邻居编辑稳定；删除 AnchorDangling 显式悬空；
  undo 复活=新 ID 族重绑恢复；D-1/MoveBlock 重建后锚点表自动
  重指）+ MDCC v1 侧车（敌意 count 物理上界 + 尾部零冗余）。
- 同步/恢复：state_vector/encode_diff/apply_update（限额+panic
  隔离）/authoritative_snapshot/restore（撤销栈不随快照走）/
  maintenance_gc（清栈+显式 gc）。
- 23 测试五套件：matrix 7 / concurrency 4（双端全量+增量收敛、
  并发删+并发插收敛、三方链式、撤销跨端只能撤自己）/ undo_redo 4
  （七操作全环、p20 交错回归锁——上游回归或配置回退必失败强制
  解钉、软上限、删除撤销复活身份）/ recovery 4 / hostile 4。
- 门禁（亲跑）：G1 606→**629** 零失败 / clippy 双面恰 2 基线 /
  fmt 0 / scan PASS / Cargo.lock +15（yrs 0.28 闭包 14 外部，全
  MIT/Apache 零 copyleft）。
- 合规：NOTICES v1.6 落 compliance-baseline（53b8438）——yrs 闭包
  入账 + v1.5 后未随轮复跑的面漂移补账（serde 族经 evidence/
  workbench + futures/js-sys 六项账→正文经 rust_xlsxwriter 特性
  面）+ 四面复跑 75/88/108/121 + 纪律恢复（lock/feature 变化轮次
  NOTICES 增量随批）。

Stage Summary:
- mdcore main 1db4a6c→**b3ec4da** 已推送（collab-r1）；
  compliance-baseline 5e144cc→**53b8438**（NOTICES v1.6）。
- 协作引擎核心形态达成：可自动验证（23 测试）的共享文档状态机，
  含并发收敛/身份/撤销/故障恢复全链路——用户指令②的实现主体完成。
- 下一步：指令③ Canvas 引擎（独立领域模型 + 同一命令接口模式）
  → 指令④端到端验证（含证据系统关联画布对象）。

---
Task ID: canvas-r1 + e2e-r1（Canvas 引擎 + 端到端验证：正式交付目标③④）
Agent: 主代理

Task: 用户新指令③④——Canvas 引擎实施（独立领域模型：节点/连线/
位置/尺寸/分组；同一命令接口）+ 端到端验证（人类操作/AI 命令/
并发修改/状态同步/撤销重做/删除复原/证据系统对画布对象的关联
能力——可运行演示而非独立单测）。

Work Log:
- mdcore 新 crate crates/canvas（第十二成员）：画布 = 图形状独立
  领域模型（Node/Edge/Group × 几何 × 端点 × 成员 × 开放属性），
  与文档域并列各自 Doc——不建跨引擎统一 CRDT；根 Map 载体（对象
  ID = 键 → 删除撤销复原 = 同 ID 回归——画布域身份特性，测试钉定
  指纹同一）；键级 LWW（并发同键写双端收敛钉定）。
- CanvasCommand 八命令 + Actor 分栈（与 mdc-collab 同构模式）；
  不变量集同源同构（skip_gc/窗 0/53 位域/限额+panic 隔离/
  capture_transaction 拒 None-origin/软上限回执）。实施自纠：
  new_undo_manager 抄漏 actor 参数化（双栈都跟 mdc:human）+
  set_props 读事务遮蔽死锁（p18 纪律读侧扩展）。
- mdc-evidence 加性扩展：EngineKind::Canvas + Locator::CanvasObject
  （v1 数据 serde 兼容）；对象语义指纹（位置漂移即语义变更 = 画布
  域证据粒度决策）。
- **yrs 0.28 map-undo 交错语义定谳（实施新发现，探针→钉定）**：
  同会话内同键交错写（Human/Ai 轮流 MoveNode）撤销 = 逐笔回退
  （撤第 k 笔恢复该历史点值 3→2→1）+ pop-until-productive（undo
  弹到有产出为止——无产出项[内容已被对栈撤销的 redo 链复刻的
  原插入项]被丢弃，一次 undo 可消耗多项 depth 4→1）；跨会话编辑
  撤销不受影响（evidence_assoc：AI 独立会话编辑撤销干净复原）。
  上游询问候选，测试钉定观测行为防静默漂移。
- 13 测试四套件：matrix 4 / undo_redo 4 / concurrency 4 /
  evidence_assoc 1（画布受证全链：登记 Valid→拖动 Stale→重验
  Valid→AI 端并发改写再失效→AI 撤销→重验复原→三态正交）。
- e2e_demo（examples/e2e_demo.rs，mdc-collab 为演示专用
  dev-dep）：五幕 12 检查点断言内嵌——文档协作（AI 骨架+人类
  修订+并发插入双向收敛+origin 分域撤销跨端传播 redo 复原）×
  画布协作（人类搭图+AI 并发补充读模型一致）× 证据关联（文档
  主张↔画布节点跨引擎 Locator→拖动失效→重验恢复）× 删除与复原
  （文档块 block_id 连续+载体身份换新族重绑；画布同 ID+指纹同一；
  复原跨端传播）× 权威快照存档恢复。
- 门禁（亲跑）：G1 629→**642** 零失败 / clippy 双面恰 2 基线 /
  fmt 0 / scan PASS / Cargo.lock 零三方增量。

Stage Summary:
- mdcore main b3ec4da→3289d41（canvas-r1）→**f91b6be**（e2e-r1）
  已推送。
- 用户指令①-④全链完成：协作引擎（23 测试）+ Canvas 引擎（13
  测试）+ 证据契约扩展 + 可运行演示（12 检查点）——已实现/已验证/
  仍待集成/实际风险四分报告落档 collab-line/协作与Canvas正式交付
  报告.md。
- 仍待集成六项（mdcore op 对接/workbench 组合/事件面/awareness/
  前端/中继）+ 风险四面（yrs 缺陷族全缓解/写入方约定/重建语义/
  资源默认值）——详见报告 §3/§4。

---
Task ID: integ-r1（补账登记——前会话已实施推送，本仓当时未留记录）
Agent: 主代理

Task: 交付报告 §3.1 集成项①——mdcore（MD 引擎）↔ mdc-collab op 层
对接（应用层组合点）：BlockTree 七操作过桥、身份映射、真相源纪律、
publish 面。用户指令「继续持续做」背景下的前会话实施轮。

Work Log:
- mdcore 第 13 crate crates/mdbridge（1789493）：身份映射三面定谳
  ——family（本端 shadow u64 ↔ md:{client_id}:{n} 确定性编解码，
  client_id 域多端零撞号）/ foreign（外端 id 持久双向映射表，跨重
  物化稳定）/ 孤儿（空 block_id 或撞号重复 id = 分配不入表，结构
  保真身份不稳定——敌意面登记）。
- 核心执行序：plan（预态路径解析 + 协作域约束镜像预检，零副作用
  失败）→ shadow 增量（mdcore apply_op 原子应用 + id 回写）→
  collab 命令（失败 → shadow 自愈重物化 + 显式错误）；真相源纪律
  = CollabSession 唯一，shadow 纯投影。
- 翻译语义钉定：MoveBlock to_index 两侧同为摘除后坐标直传不改写
  （dual_run 双跑锁——同 op 序列过桥 vs 裸树覆盖等价断言）；
  SetBlockType D-1 载体换新族 block_id 连续 shadow u64 稳定；
  SetMark D-2 归一化字面形状允许差。
- collab 加性读面：BlockInfo.marks 展平区间（mdbridge materialize
  消费面）。
- publish 身份语义 = 重映射（import）：源树与 family 两分配器空间
  都从 1 起 → 每块 fresh family u64 + 回执 top_ids 即映射表；adopt
  （保 u64 活会话交接）无消费方不预制——登记留裁决。
- 测试 25 新增（bridge_ops 9 + bridge_publish 4 + bridge_sync 6 +
  collab marks_readback 4 + codec 2）；G1 642→667。

Stage Summary:
- mdcore main 1789493（integ-r1）已推送；集成项① 落地，交付报告
  §3 待集成清单剩五项（②③④⑤⑥）。
- 本条为补账：实施发生在前会话，本轮（integ-r1-verify）复验确认
  后补记——详见下一节复验记录。

---
Task ID: integ-r1-verify
Agent: 主代理

Task: 环境第 17 次重置恢复 + integ-r1 推送态复验 + cleanroom 记录
补账 + 门禁卫生补丁（用户「继续持续做」——按依赖序恢复执行）。

Work Log:
- 环境恢复：GH_TOKEN 环境变量重建（零落盘纪律维持）→ 双仓瞬态
  header fetch + ff 拉齐（cleanroom 07a9681 / mdcore 1789493——
  远端较本地 stale ref 多 27/9 提交，纯落后零分叉）→ 工具链重装
  （rustup 1.29.2 + 1.99.0 + nightly-2026-10-15[rust-src] + wasm32）
  → pdfium chromium/8086 脚本取件（SHA256 验证过）+ fixtures 三件
  再生成（spike 双 bin 跑通三探针 + S6 表单探针全过）。
- 推送态复验（本会话亲跑，mdcore 1789493）：G1 cargo test
  --workspace = **667/667 零失败**（= integ-r1 声明口径精确复现）；
  fmt 0；scan PASS（0 hit-paths）；wasm32 check --workspace
  --all-targets 零错误；**clippy --workspace --all-targets 发现
  integ-r1 漂移**——mdbridge 测试面 7 警告（bridge_ops 6：
  family_id/markdown unused import + 4 不必要 mut；bridge_publish
  1：family_id unused import），根因 = integ-r1 提交时 clippy 口径
  未含 --all-targets 测试面。
- 门禁卫生补丁（mdcore d3bbb12）：机械修复 7 警告（3 unused
  import 删除 + 4 mut 删除；实施自纠：replace_all 误伤 81/376 两处
  真实可变借用[apply_op/materialize 消费面]——编译错误即捕获即
  还原）；修复后 clippy 双面恰 2 wasm_bindgen 基线恒 + mdbridge
  21/21 零失败 + fmt 0；已推送（瞬态 header，token 零残留验证）。
- cleanroom 记录补账：integ-r1 worklog 补账条 + 交付报告 §3.1
  翻转（已落地 + 复验口径 + 剩余五项）+ HANDOFF §6 新速览块。

Stage Summary:
- 集成项① 全链收口确认（实施 1789493 + 卫生 d3bbb12 + 双仓文档
  同步）；推送态可复现性验证通过（漂移仅 clippy 测试面口径差）。
- 剩余待集成五项依赖序定谳：③事件/订阅面 → ④awareness →
  ②workbench↔协同组合 → ⑥服务端中继 → ⑤前端（用户裁决项维持
  门控）。

---
Task ID: integ-r2
Agent: 主代理

Task: 集成项③——事件/订阅面（交付报告 §3.3：前端宿主推式通知；
此前拉取式读模型，yrs 观察者未包装暴露）。双域（文档/画布）同轮
落地。

Work Log:
- mdc-collab events 模块：CollabEvent{source,updates}——来源五态
  （Command{actor}/Undo{actor}/Redo{actor}/Remote/Maintenance）与
  公共变更入口一一对应（方法上下文标注，不依赖 origin 推断）；
  updates=逐事务 wire 字节（每条可独立 apply_update 转发）；一公共
  入口恰一事件（LoadBlocks 512/txn 小批多事务合并为单事件 updates
  序——600 块=2 批=1 事件钉定）；失败命令不产事件（错误显式返回，
  双通道不重复）；touched 面刻意不提供（XML 载体 block 级提取需
  属性级事件解释——诚实登记延后）。
- mdc-canvas events 模块（同构镜像 + 画布域增量）：
  CanvasEvent{source,updates,touched}——touched=受触对象 ID 字典序
  去重；提取双路径（根 Map 自身事件 path 空 → MapEvent::keys 提取
  [对象增删]；嵌套事件 path 首段 [几何/属性/端点/组成员深变更]）。
- 派发纪律（结构性安全，全部文档化）：延迟派发（观察者只在事务
  提交期收字节；订阅者派发在公共方法返回前事务已释放后——p18
  读写同锁纪律的观察者面扩展）；回调不可重入会话（只收 &Event，
  借用检查器结构性阻止）；回调 panic 正常传播（宿主代码非引擎
  代码）；Drop 退订即时（派发中至多一次尾随调用）；锁序无自杀锁
  （注册表锁短暂持有→逐订阅者自身锁）。
- 维护窗事件恒发（updates 可能为空——GC 无新 CRDT 操作，观测
  钉定）；restore 新会话无历史事件（注册表随实例新生——恢复后
  首个变更正常派发，钉定）。
- 实施自纠两处：①open() 构造误用 UpdateCollector::default() 新
  实例而非已 attach 的局部收集器（观察者写入被丢弃实例——11
  测试全红定位后修复）；②调试示例踩 p18 变量遮蔽死锁（txn2 遮蔽
  txn1 自锁——即删，与产品代码无关，纪律再验证）。
- 测试 21 新增：collab events 11 + canvas events 10（含远端深观察
  touched 提取/转发收敛/删除撤销同 ID 回归触及/分组=分组键）。
- 门禁（亲跑）：G1 667→**688** 零失败 / clippy --workspace
  --all-targets 恰 2 wasm_bindgen 基线 / fmt 0 / scan PASS /
  wasm32 check --all-targets 0 错误 / Cargo.lock 零 diff。

Stage Summary:
- mdcore main d3bbb12→**02067ad**（integ-r2）已推送。
- 集成项③ 收口：拉取式读模型 + 推式事件双面在位；前端集成的
  通知面前置解除。剩余待集成 = ②workbench↔协同组合、④awareness、
  ⑥服务端中继、⑤前端（用户裁决项）。

---
Task ID: integ-r3
Agent: 主代理

Task: 集成项④——awareness/在场层（交付报告 §3.4：在线状态/光标；
yrs sync feature 在位未暴露 API——协同编辑完整体验需要，前端集成
前置件）。

Work Log:
- mdc-collab presence 模块：Presence 类型（yrs Awareness 受控包装，
  公开面零 yrs 类型）。归属定谳：在场层 = 会话层协同基础设施（非
  领域逻辑）→ 单点落位 mdc-collab；画布宿主经 Presence::standalone
  (client_id) 独立构造消费（私有载体 Doc 仅承载 client_id——
  awareness 状态在 DashMap 不写 doc；宿主保证 doc 流与 presence 流
  client_id 一致——身份关联纪律文档化）——领域独立纪律的反向应用
  （基础设施单点，不镜像）。
- 公开面：set_local_state（JSON fail-closed 校验）/local_state/
  clear_local_state（null+clock+1 离线语义）/states（client_id
  升序确定性读模型）/encode_update（v1 全量含 null 条目）/
  encode_update_for（定向，未知客户端显式错）/apply_update（入向
  限额默认 1MiB + panic 隔离，B1.6 同构）/subscribe（added/updated/
  removed 三分类）。
- **上游行为差异定谳（实施新发现）**：yrs Awareness::update() 刻意
  排除已清除（null）条目（"doesn't include states that have been
  removed"）→ 离线广播无法经 update() 传播；本 crate encode_update
  改走 update_with_clients 全键集（null+clock 随全量走——晚到对端
  退休旧态的唯一通道）。接收端三面钉定：json="null"+clock≥已知 →
  退休+removed 事件；clock 陈旧零回退；远端不可删本地态（时钟反增
  保护——yjs 安全语义）。
- wasm32 兼容：Awareness::new（SystemClock）在 wasm32-unknown 被上
  游 cfg 排除 → with_clock + SystemTime 回退 0 时钟（与撤销管理器
  timestamp 同款模式——排序由 clock 字段保证）。
- SessionOptions 增 presence_byte_limit（默认 1MiB——在场 JSON 小
  载荷与文档 52MiB 族分立）；CollabSession::presence_mut()；
  serde_json 显式声明（7 crate 既有直接依赖，零新三方）。
- 测试 11 新增：本地生命周期三事件/非法 JSON 拒收/双端 wire 收敛/
  null 离线传播/陈旧 clock 零回退/限额+垃圾隔离/standalone 域
  （53 位恰过恰拒）/会话 client_id 对齐/三客户端全量互发收敛/
  订阅生命周期/定向编码。
- 门禁（亲跑）：G1 688→**699** 零失败 / clippy --workspace
  --all-targets 恰 2 wasm_bindgen 基线 / fmt 0 / scan PASS /
  wasm32 check --all-targets 0 错误 / Cargo.lock 纯成员行。

Stage Summary:
- mdcore main 02067ad→**e2dfc7c**（integ-r3）已推送。
- 集成项④ 收口：在线状态/光标的完整 API 面暴露（含离线语义与
  安全边界钉定）；前端集成第二前置件解除。剩余待集成 = ②workbench
  ↔协同组合、⑥服务端中继、⑤前端（用户裁决项）。

---
Task ID: integ-r4
Agent: 主代理

Task: 集成项②——workbench 八步闭环 ↔ 协同会话组合（交付报告
§3.2：协同文档作为受证正文；mdbridge 落地后技术前置已齐备）。

Work Log:
- 组合架构定谳：**会话是唯一编辑面（真相源），文件是持久化投影**
  ——检查点落盘模式（materialize 会话读模型 → DocxWriter 写回
  body_path），workbench 的失效/重绑/导出机件对协同正文照常运转
  ——零 workbench crate 代码改动（组合纯在应用层，§六架构约束
  践行；workbench 只增 dev-deps：mdbridge/mdcore/collab 纯路径
  依赖）。
- 组合测试 3 新增（crates/workbench/tests/collab_composition.rs）：
  ①AI 润色受证段（数字保持纯润色）→ 落盘 → refresh_body 恰该
  主张 Stale + 相邻主张 Valid（选择性失效——指纹面钉定）→ 人工
  审核 + reverify 重绑恢复（§四.5 润色后允许失效）→ AI 撤销润色
  回退 → 再失效再重绑（指纹双向）→ export_clean 协同终态零标记；
  ②双端并发（A 端 AI 润色 + B 端人类补记来源段）→ 双向收敛 →
  单点落盘 → 选择性失效 → 重绑；③组合②×③——裸对端订阅 Remote
  事件 + updates 可转发。
- 可运行演示（examples/collab_body_demo.rs）：五幕 12 检查点断言
  内嵌（证据闭环[双绑定+程序求值 4210.000]/协同正文上线[5 块发布
  读模型段落序一致]/并发编辑收敛+事件观察/选择性失效/人工闭环
  [重绑+撤销回退+干净导出]）——cargo run -p mdc-workbench
  --example collab_body_demo 亲跑全过。
- 实施自纠两处：双向同步初版只单向（B→A）→ 双向 diff 收敛修正；
  人类补记插入位 char 差一（14→15 句号后）。
- 门禁（亲跑）：G1 699→**702** 零失败 / clippy --workspace
  --all-targets 恰 2 wasm_bindgen 基线 / fmt 0 / scan PASS /
  wasm32 check --all-targets 0 错误 / Cargo.lock 纯成员行。

Stage Summary:
- mdcore main e2dfc7c→**80e097a**（integ-r4）已推送。
- 集成项② 收口：协同正文 × 证据闭环全链闭合（选择失效/重绑/撤销
  回退/并发/导出）。剩余待集成 = ⑥服务端中继、⑤前端（用户裁决
  项维持门控）。

---
Task ID: integ-r5
Agent: 主代理

Task: 集成项⑥——服务端中继（交付报告 §3.6：多端常驻协作需要
权威端/中继形态；快照单点编码语义已就位）。

Work Log:
- mdcore 第 14 crate crates/relay（61f1ea8）：**传输无关中继状态机**
  ——入向 RelayMessage（Join/Leave/DocUpdate/PresenceUpdate/
  SyncRequest）→ 出向 RelayDelivery 表（DocUpdate/PresenceUpdate
  广播 + SyncResponse 定向）；零网络/线程/进程面（Headless 纪律
  延伸——宿主桥接真实传输；from 字段 = 传输层已认证身份，中继不
  重签，文档化边界）。
- **领域无关接入**：SyncReplica 门面 trait（state_vector/
  encode_diff/apply_update/snapshot 四操作——文档域 CollabSession
  与画布域 CanvasSession 同步 API 同构）；门面类型属宿主侧（孤儿
  规则），本 crate 零域引擎依赖；测试双域 harness 均接入并收敛。
- 语义面：DocUpdate = 权威副本先应用（fail-closed 失败零扇出）+
  CRDT 字节原样扇出（幂等/交换律——不重编码）；SyncRequest =
  差量追平；PresenceUpdate = 中继在场层聚合（组合⑥×④——integ-r3
  的 Presence 单点归属直接复用）；快照检查点（手动 + 间隔自动
  latest_snapshot）；崩溃恢复 = 新副本 + 快照经 DocUpdate 应用
  （update 即状态迁移）；限额/隔离继承域会话面（B1.6——无第二套）。
- 测试 9 新增（事件驱动 harness——组合⑥×③：客户端订阅自己的
  命令事件产 update 送中继，Remote 不回发无回声环）：三端扇出
  收敛/晚到端差量追平/在场扇出/画布域同构/快照检查点+新中继恢复/
  间隔自动检查点/离开停投/失败零扇出/加入幂等+中枢在场限额。
- 环境事件：本轮磁盘满（target/debug/incremental 1.6G + /tmp LO
  解压残留 3G）→ clippy/wasm32 报 No space left（非代码错误）→
  清理增量目录/spike target/LO 临时件 → 全门禁复验绿。
- 实施自纠：harness 初版 thread_local+mem::forget 过度复杂重写为
  Net 持有订阅句柄与共享队列；测试限额断言初版误设独立 Presence
  而非中枢构造参数。
- 门禁（亲跑）：G1 702→**711** 零失败 / clippy --workspace
  --all-targets 恰 2 wasm_bindgen 基线 / fmt 0 / scan PASS /
  wasm32 check --all-targets 0 错误 / Cargo.lock 纯成员行。

Stage Summary:
- mdcore main 80e097a→**61f1ea8**（integ-r5）已推送。
- 集成项⑥ 收口：P2P 直连形态升级为可常驻中枢形态（权威副本/
  扇出/差量/在场/检查点/恢复全链钉定）。
- **交付报告 §3 集成清单工程侧全部收口**：①②③④⑥ 已落地（G1
  642→711，四轮 integ-r2..r5 + 复验轮），仅剩 ⑤ 前端 = 用户裁决
  项（未选型不得指定框架——维持门控）。

---
Task ID: harden-r1
Agent: 主代理

Task: 风险面加固轮（交付报告 §4 工程侧收口——用户指令「前端⑤裁决期
并行推进非前端项」）+ 新 governing 计划落档（用户同消息下达《文档
工作台后续完整实施计划》）。

Work Log:
- **mdc-collab audit 模块**（§4.2 撤销耦合防护收口）：audit_update
  （隔离暂存 Doc 应用前审计——B1.1 skip_gc 同构 + panic 隔离 +
  限额同门 B1.6）/ audit_self（在位验证）/ repair_text_nodes
  （机械修复——origin=mdc:maintenance 不进 Human/Ai 撤销栈；产物
  经 Maintenance 事件 updates 可转发 = 对端同步获得防护；幂等）。
  违规分级：Error 5 类（MissingTextNode/MissingBlockId/
  DuplicateBlockId/ForeignRootType/WalkFailure）+ Warning 9 类
  （UnknownTag/RootLevelText/FragmentChild/ThematicBreakWithText/
  MarkMissingKind/MarkForeignChild/NonCanonicalAttrKey/
  NestedUnderNonContainer/TextAfterChildren/ForeignRootData——
  全部静默丢弃或强转面逐项登记）。
- **根类型判别探针定谳**（实施新发现）：update 应用后的根为
  UndefinedRef（yrs 根类型 = 本地约定，wire 不携带）——按
  XmlFragment 视图走查 + 内容判别面（MapRef::len 键值条目 /
  frag.len 与 children() 走查数差额 = 数组序列条目——两者对
  children() 均不可见）。
- **set_block_type 形状外泄修复**（audit 口径反噬发现）：重建原
  无条件复制子节点——p带文本→hr / 容器带子块→非容器 / hr→p
  转换均可产出引擎书写约定外形状（后者 = 撤销耦合隐患：无出生
  文本节点的文本承载块）。修复 = 承载面前置校验（KindIncompatible
  拒——镜像 insert 守卫）+ 重建按目标种类过滤 + 文本承载目标恒补
  出生文本节点（build_prelim 同约定）。
- **p18 死锁新面**（实施自纠）：get_or_insert_xml_fragment 内部开
  写事务——持读事务调用 = 死锁（repair 首版即踩，测试挂起定位）；
  修复 = 先 get_or_insert 后开事务（session.rs document() 同序）。
- **撤销耦合防护端到端证明**（tests/audit.rs）：对照组（外部脏块
  不修复——undo 首笔删整节点，B 端并发内容随删除传播全编队丢失）
  vs 修复组（repair + updates 转发——undo 恒只撤字符项，B 端并发
  内容存活「 world」）。10 新测试全绿。
- **relay_demo 中枢可运行演示**（13 检查点断言内嵌，cargo run -p
  mdc-relay --example relay_demo）：三端上线在场聚合/Join 幂等/
  并发扇出三端一致/权威态 SyncResponse 互证/字节直转发逐字节对照
  （位置插入 update 携带邻居依赖——独立应用到空端待决 = CRDT 正确
  语义，差量才是追平通道——检查点⑤初版误设「独立应用应有产出」
  自纠为逐字节对照）/晚到端 Join 零投递 + 差量追平/B 撤销跨端
  四端回退 + 分栈不受波及/检查点崩溃恢复（**快照 = 检查点时刻态
  诚实语义**——快照后的撤销不在内；间隔与丢失窗 = 部署取舍面）/
  恢复后继续扇出收敛/全编队 audit 姿态（组合⑥×audit）。
- **clippy 卫生**（integ-r4/r5 漂移 6 警告 + 本轮新码 3 警告清零）：
  is_multiple_of/Default impl ×2/clone_on_copy ×2/to_path_buf/
  field_reassign ×2/useless vec!——复归恰 2 wasm_bindgen 基线。
- **上游询问候选落档**（collab-line/upstream/）：p20 捕获窗缺陷
  bug 报告草案 + map-undo 交错语义澄清草案（英文、最小复现件内嵌、
  触发条件矩阵）；独立验证 crate（scripts/upstream_repro_verify/
  仓外工件）release profile 亲跑：p20 两症状逐字复现（部分撤销
  "A1H2A2" + store.rs:340 unwrap None panic；附加发现 debug profile
  更早触发 block.rs:652 subtract overflow——同路径溢出面，草案
  已注明）；map-undo 3→2→1 逐笔回退 + 单次 undo 消耗 3 栈项且键
  整体移除（<none>——行为 2 最强形态）。ready-to-file，**呈用户
  择机提交**（提交前复验：0.28.0 仍最新 / 上游无同题 issue）。
- **新 governing 计划落档**：workbench-line/文档工作台完整实施
  计划.md（用户原文逐字 + 实施侧对照批注——前端⑤从「悬置裁决」
  升级为「阶段 1 基准 + 双原型对照后定谳」）。
- 门禁（亲跑）：G1 711→**721** 零失败（+10 audit 测试）/
  clippy --workspace --all-targets 恰 2 基线 / fmt 0 / scan
  PASS / wasm32 check --all-targets 0 错 / Cargo.lock 零 diff /
  三演示全绿（e2e 13 + collab_body 12 + relay 13 检查点）。

Stage Summary:
- mdcore main 61f1ea8→**ef830dd**（harden-r1）已推送。
- 交付报告 §4：风险②（撤销耦合防护）工程侧收口（audit/repair +
  set_block_type 修复 + 端到端证明）；风险①（yrs 缺陷族）上游
  草案两份备稿 ready-to-file。
- 前端⑤之外的全部工程面收口。新计划（六阶段）已接管后续路线：
  下轮 = 阶段 0 核实 + 阶段 1 四类性能基准 + 宿主选型前置盘点。

---
Task ID: p0-verify + p1-bench
Agent: 主代理

Task: 新计划（文档工作台完整实施计划）阶段 0 收口核实 + 阶段 1 前半
（四类性能基准 headless 面 + 前端能力盘点）。

Work Log:
- **阶段 0（复用为主，亲跑补证）**：harden-r1 收口推送（mdcore
  ef830dd / cleanroom 7f73dba，PAT 环境变量单次传递 + 用毕 unset）；
  workbench 四测试面亲跑复现（八步闭环 4 + sidecar 持久化/异常恢复
  6 + 命令层 5 + 协同组合 3）；Office 锚点合并状态核实（office-g
  Phase 1+2 已在 main——c4856f7 历史链）；yrs 三缺陷处置结论复核
  （p20 窗 0 + 回归锁 + 上游草案 ready；#676 skip_gc；map-undo 钉定
  + 草案）；协作/Canvas 定性 = 产品实现（14 crate / G1 721 / 三演示
  可运行）；HANDOFF §6 五分类现状表（已实现/已验证/部分集成/延期/
  未实施）+ 新计划落档（workbench-line/文档工作台完整实施计划.md，
  原文逐字 + 实施侧对照索引）。
- **阶段 1 基准①（bench_long_doc，新 example 六场景）**：ingest
  10k=44ms/51MB、100k=471ms/456MB 线性；bindings 54.8ms/笔@10k×
  1000、494.5ms/笔@100k×100——**发现：每笔绑定 = 正文重摄取
  O(n)（fail-closed fresh-read 纪律代价）**；refresh_body 533ms
  含重摄取+全指纹对照、选择性失效恰 1 条（外部改写 1 段 → 恰 1
  主张 Stale）；reverify 8.3ms/条；persistence/report 亚毫秒。
  实施自纠三处：heading 占段索引 0 的锚段偏移；gen_body 被改写体
  同名自删（ParentUnavailable 定位）；主张值须嵌入锚段文本
  （ClaimedValueNotInParagraph 校验）。
- **阶段 1 基准②（bench_limits 扩展两场景）**：slice500k 可视区
  窗口（50×20）2µs/窗（typed 网格内存访问）；edit500k C3 单笔
  编辑提交全链 读536+重放64+写492 ≈ 1.1s@500k 格——**C3 读改重建
  的提交下界，阶段 5 局部更新/延迟提交的量化论据**。
- **阶段 1 基准③（PDF perf 门复跑）**：6/6 过（TTI 0.16ms / 首屏
  9.38ms / 顺序渲染 p95 0.82ms / 抽取 56k 页/s / 翻页命中 0.51µs）；
  冷进程首跑 43.7ms = 页缓存方差（三次复跑 12.9/8.4/9.4ms 过阈值）
  ——环境特性登记非回归。
- **阶段 1 基准④（bench_canvas，新 example 七场景双档）**：装载
  6µs/op；MoveNode p50 3-5µs；find ~1µs；删除+撤销 p50 11µs（模型
  面无瓶颈）；**objects() 全量快照 p50 1.0ms@1k / 13.3ms@10k——
  渲染每帧消费面 = 增量读论据（事件 touched 面已在位）**；全画布
  指纹 2.1/24.3ms；同步全量差量 1k=149KB/5ms、10k=1.5MB/61ms。
- **前端能力盘点（选型前置，不预判）**：web/ CM6 桥在位（MD 面）；
  shell-native 蓝图态；候选 A = web+wry（存量复用最高）B = 原生
  UI（Slint 不因 Kokoa 默认）；对照操作集 = 四类基准负载。
- **基准报告落档**：workbench-line/阶段1性能基线报告.md（数字+命令
  +环境口径+三项结构性行动项）。
- 门禁（亲跑）：基准代码 clippy 零新增（恰 2 wasm_bindgen 基线）/
  fmt 0；受影响 crate 测试 41/41；mdcore main ef830dd→**82884cc**
  已推送。

Stage Summary:
- 阶段 0 验收达成：成果可复现（四测试面亲跑）、缺陷处置结论明确
  （yrs 三缺陷+set_block_type 修复）、HANDOFF 与仓库一致（五分类表）。
- 阶段 1 前半验收达成：四类基线可复现在档（命令+数字）；PDF 6/6；
  三项结构性行动项进入阶段 5 议程（表格提交策略/Canvas 增量读/
  证据批量通道）。
- 下一步：阶段 1 后半双宿主原型对照（A web+wry / B 原生）→ 选型
  记录 → 阶段 2 证据复验。

---
Task ID: p1-host
Agent: 主代理

Task: 六阶段计划阶段 1 后半——双宿主原型对照 → 选型记录 → 定主宿主
（用户指令「继续持续的做不做完或者有重大阻塞不停息」；计划 §四.5
裁决方式 = 基准 + 双原型对照后定谳，前端不再悬置）。

Work Log:
- **环境第 ~20 次重置恢复**：rustup 1.29.2 + Rust 1.99.0 钉住
  （clippy/rustfmt/wasm32 经 rust-toolchain.toml 自动补装）+ cargo
  fetch + pdfium chromium/8086 SHA256 验证重取（spike bin/lib 第 2
  寻径序位）+ G1 cargo test --workspace = **721/721 零失败**（= 82884cc
  推送态精确复现）+ wasm-bindgen-cli 0.2.129 按锁安装 + web npm
  install/build-wasm/build + web_smoke **15/15**（W1-W11+perf 面）。
  GH_TOKEN 环境变量重建（零落盘纪律维持）。
- **可行性硬检查**：webkit2gtk-4.1/GTK3 MISSING（无 root 不可装）→
  wry 桌面壳沙箱不可编译（如实登记为 A 侧未测量面）；Vulkan ICD 空
  → wgpu/iced 排除；X11 全链在位（libX11/Xcursor/Xrandr/Xrender/
  xkbcommon）→ winit 路可行；Xvfb + Playwright Chromium 缓存幸存。
- **共享对照数据集**（单一真源，确定性 LCG）：10k 块中文正文 2952KB
  + 100 证据主张 + 500k 格表格 5441KB + 1k 节点/1.5k 边画布 105KB
  （proto-native --gen 生成，双侧字节同源）。
- **原型 B（原生 egui 候选）**：`workbench-line/proto-native/` = egui
  0.37 + glow + winit/X11（Xvfb 软渲染）+ mdcore path 依赖直连 +
  x11rb XTEST 真实输入注入（X 服务器→winit→egui→GL 全链，与 CDP
  对称）+ CJK 字体运行时装载（NotoSansSC 5.4MB 真实启动成本计入）。
  沙箱缺件自愈：apt download 无 root 提取 libxkbcommon-x11+libxcb-xkb
  → LD_LIBRARY_PATH（真实桌面标配，非产品约束）。
- **原型 A 基准页**：`mdcore/web/bench.html + src/bench.ts`（零新
  npm 依赖；CM6 = core.to_markdown() 投影镜像与产品面同纪律；虚拟
  网格表格；SVG 画布 + 应用级 O(n) 命中 + 拖拽交互含连线跟随）+
  build.mjs 双入口接线 + tsc 零错误。驱动器 bench_web.py（Playwright
  CDP 真实鼠标）+ bench_native.py（Xvfb+XTEST）。
- **13 场景双侧对称实测（双轮稳定）**：帧预算——web 四类负载全落
  60Hz vsync（16.6ms）；原生软渲染表格滚动帧 31.6ms 超预算、画布
  21.0ms、fps 20.6ms。内存 web 进程树 673MB vs 原生单进程 188MB。
  启动 255-277ms vs 119-141ms。命中 0.74µs vs 32-53µs（43-70×）。
  核 op 回写双侧验证过（rev 0→1 + 文本含编辑符）；命中点列 LCG
  互证（23=23）；拖拽位移 (37,21) 双侧精确。
- **实施发现三则**：①JS LCG 无 u64 回绕掩码 → 与 Rust 序列分叉
  （hits=0 定位即此，BigInt & 0xFFFF...FFFF 修复后序列逐点对齐）；
  ②String::insert 字节索引 vs 核 op 字符偏移——CJK 行直接 insert(5)
  panic（'## ' 后首字符横跨字节 3-5），字符边界换算修复；③egui
  scroll_to_rect 取屏幕坐标（= origin+内容坐标），初版误传内容坐标
  致画布滚动过冲、XTEST 按压落屏外。
- **IME 证据复跑**：web 面 ime_cdp.py = **28/28 全过**（CDP 真实
  输入法通道：组合流逐预编辑落核 op/1s 合并窗单 undo 条/跨行兜底
  同步/点击不清组合/提交后 undo 单条回退）——选型 IME 维度的
  决定性证据（egui/winit IME 在位未验证，如实登记）。
- **选型记录落档**：`workbench-line/宿主选型记录.md`——结论先行
  （主宿主 = A：web 面 + wry/tao 壳，ADR-H1 方向确认；B 保留为降级
  路径与参考实现）+ 方法/复现命令 + 双轮数据表 + 七维评估（A 四维
  B 两维一维混合）+ 未测量面五条（wry 壳真机验证 = 阶段 5 前置）+
  后续验证项清单。
- 门禁（亲跑）：mdcore Rust 面零触碰（仅 web/ 三文件：bench.html/
  bench.ts 新增 + build.mjs 接线）→ web 面 typecheck 0 错 +
  web_smoke 15/15 + ime_cdp 28/28 + build 双入口绿；proto-native
  cargo check/clippy 零警告；Cargo.lock（mdcore）零 diff。

Stage Summary:
- 阶段 1 全部收口：四类性能基线（前半，82884cc）+ 双宿主对照与
  定谳（后半，本轮）——计划 §四.5 验收达成（可复现基线 + 候选
  比较结果 + 明确宿主选型及理由）。
- 主宿主定谳 = A（web+wry）；选型记录 + 双轮实测 + 七维评估 +
  未测量面诚实边界全在档。B 降级触发条件明确（webview 矩阵不可
  接受/W7 单列目标）。
- mdcore main = 82884cc + web 基准页（本提交）；cleanroom main =
  本提交。下一步 = 阶段 2 证据复验（七项清单）。

---
Task ID: p2-r1
Agent: 主代理

Task: 六阶段计划阶段 2——稳固证据与命令基础（计划 §五 七项验证：
完整关联链/失效识别/不自动恢复/三态分离/导出独立/sidecar 异常处理/
命令统一入口）。

Work Log:
- **七项覆盖面 gap 分析**：八步闭环（①-⑧ 全链）+ persistence_recovery
  （六测）+ command_layer（五测）+ collab_composition（三测）对照计划
  §五 逐项核对——六项既有直接测试证据；唯一实质缺口 = 末段「不同
  业务场景可配置的审核门槛」（review() 无门槛面——任何操作者可翻
  任何状态）。
- **ReviewPolicy 落地**（mdc-workbench security.rs + pipeline.rs）：
  三维 fail-closed 门禁——①操作者前缀允许列表（默认 `human:`——
  AI 不可改审核态，架构原则「AI 只提关联与推导」的门面化）；
  ② `allow_approval_of_broken_linkage`（默认 false——Stale/Missing/
  Ambiguous 关联不得直接过审，过审前必须显式 reverify 恢复 Valid
  ——计划 §五.3「不得自动恢复为已核验」的执行面）；③
  `calc_required_at_or_above`（主张值达门槛的计算型证据须 calc
  Passed——业务场景差异面）。会话级注入（set_review_policy）不入
  sidecar——sidecar 只承载事实与审计，策略属调用方信任面。
  Workbench::review 单点接门（拒绝 = 零状态变化 + sidecar 零落盘）；
  命令层 map_error 映射 ReviewGate → Rejected（校验拒绝——与
  路径/规模门同语义）。
- **6 新测试**（crates/workbench/tests/review_gate.rs）：AI 操作者
  双方向拒 + 零状态变化 / 失效过审拒（拒绝原因指向关联态）+ 拒绝
  方向不受辖 / 值门槛（Passed 放行 → 来源修正后双门齐拒 Rejected
  →Approved 翻案同样吃门）/ 宽松档显式放行（allow_broken_linkage）
  / 自定义前缀（auditor:）/ 命令面同门（execute → Rejected + 不入
  幂等表 + 重试独立校验）。
- **测试适配 4 处**（门禁语义正确的既有测试翻新）：collab_composition
  ×2 先审后验旧序 → 先验后审（人工重绑批次内次序本非测试语义本体
  ——门禁守护的正是这次序）；command_layer full_loop 与
  persistence_recovery idempotent 的 Review 命令操作者 ai:glm →
  人工信封（统一 ai 操作者是便利性选择非语义）。
- **实施自纠**：write 坐标 0 基（write(6,1)=B7——首版 write(7,2)=C8
  即 CellNotFound）；tick 闭包可变借用冲突（快照时序前移）；Stale
  级联诚实性（InputUnavailable 非 InputMismatch——复算不经失效关联，
  测试期望对齐）。
- **门禁漂移修复**：bench_canvas 缺 wasm32 占位 main（t21 收严
  --all-targets 口径的漏网——bench_limits 同款既有模式[带文档注释]）
  → 补齐后 wasm32 check --workspace --all-targets 0 错误复绿；
  bench_long_doc 同型补齐。
- **磁盘满事件第 2 次**（环境既定复发）：LO 解压残留 903M +
  /tmp tool-results 1.2G + debug targets 5.7G（mdcore 4.0G +
  proto-native 1.7G debug + 1.4G release）→ 按既定程序清理（含
  proto-native 全 target——复跑原生基准需 rebuild）→ 7.1G 空闲。
  本轮全量测试两轮异常失败（sidecar 三测/presence 一测）均为磁盘
  满所致非代码缺陷——清理后 727/727 复绿验证。
- 门禁（亲跑，磁盘恢复后）：G1 cargo test --workspace = **727/727
  零失败**（721 既有 + 6 新门禁测试）/ clippy --workspace
  --all-targets 恰 2 wasm_bindgen 基线 / fmt 0 / wasm32 check
  --all-targets 0 错误 / Cargo.lock 零 diff。

Stage Summary:
- 阶段 2 验收达成：证据闭环可复现（727 全绿）/ UI 接入无需第二份
  权威数据（web 桥 CM6 doc = 核投影 + collab 会话 = 唯一编辑面在
  案）/ 失效与失败状态明确表达（三态正交 + 门禁拒绝零状态变化 +
  Rejected 语义分类）。
- mdcore main 2043bdb→**3d2e578**（p2-r1）已推送。七项中六项既有
  证据复验 + 一项实质缺口（可配置审核门槛）落地。
- 下一步 = 阶段 3 协作引擎完成实施核对（增量面：撤销粒度聚合[宿主
  侧输入聚合待阶段 5——既有限制在档]/内存边界[软上限 clear_all
  显式回执在档]/真实编辑器集成测试[collab_body_demo 既有]）。

---
Task ID: p34-verify
Agent: 主代理

Task: 六阶段计划阶段 3/4 核对——协作引擎与 Canvas 完成实施验收
（大部已在前序集成冲刺落地，本轮 = 计划条目逐项对照 + 增量验证面）。

Work Log:
- **§六.1 撤销粒度实证**（唯一增量验证面）：新测试
  typing_granularity_pinned_per_command——逐字符 5 命令 = 5 栈项
  （undo_depth 钉定：捕获窗 0 = p20 缓解的既定代价）；逐项撤销
  全链回退正确（语义完整性不因粒度细而破）；宿主聚合形态（单命令
  多字符）恰 1 栈项——聚合责任在命令构造侧而非捕获窗（p20 窗>0 +
  交错 origin = 缺陷路径，不可回退）。阶段 5 表面实施项（宿主侧
  输入聚合）既有限制登记维持。
- **§六.2 内存边界复验**：R3 定量（437.9 B/op · 6M≈2.4GiB）+ 软上限
  clear_all 显式回执（B3.4 非静默）+ S0 一页清单在档——「不得伪造
  上限/不得静默清空」两条纪律既有满足，零改动。
- **§六.3 七项对照**：稳定逻辑 ID 与 CRDT ID 分离（mdbridge
  family_id 重映射）/同命令契约（Actor×CollabCommand）/事件通知
  （integ-r2 双域事件层）/并发冲突过期命令（matrix+hostile+并发
  套件）/保存断连恢复删除复原（relay 检查点 + recovery 套件）/
  会话本地撤销（origin 分栈）/真实编辑器组合测试（collab_body_demo
  五幕 12 检查点）——全部既有产物，零增量。
- **§七 Canvas**：headless 引擎（mdc-canvas 领域模型/命令接口/证据
  关联）+ e2e_demo 全链复跑 PASS；「真实交互」= 阶段 5 表面项
  （宿主定谳 A：web+wry——选型记录已列）。
- **演示件适配**：collab_body_demo 幕五先审后验旧序 → 先重验恢复
  Valid 再过审（阶段 2 门禁次序守护——与 collab_composition 测试
  同型翻新，第 5 处适配点）。
- 复验亲跑：collab 套件 **61/61**（60 既有 + 1 新粒度测试）+
  collab_body_demo **五幕 12 检查点全过** + mdc-canvas e2e_demo
  **全链 PASS**；G1 **728/728** 零失败 / fmt 0 / clippy 恰 2
  wasm_bindgen 基线。

Stage Summary:
- 阶段 3/4 验收达成：撤销粒度实证钉定 + 内存边界纪律在档 + 全部
  条目既有产物对照 + 三演示/套件亲跑复现。
- mdcore main 3d2e578→**fc25f06**（p34-verify）已推送。
- 下一步 = 阶段 5 渲染表面（宿主 A = web 面扩展；wry 壳真机编译 =
  选型记录既列前置项）。

---
Task ID: p5-t1
Agent: 主代理

Task: 六阶段计划阶段 5 启动——第一个前置件：表格表面数据面 wasm
出口（§八.3「切片、虚拟滚动」的数据源；宿主 A = web 面已定谳）。

Work Log:
- **xlsx_grid 模块**（crates/mdcore-wasm/src/xlsx_grid.rs）：WasmBook
  一次解析常驻句柄（Book 生命周期挂 JS 侧——与 WasmCore 同模式）；
  sheets_json（表名/使用区行列）；slice_json 可视窗切片（typed
  kind/原值/display 三件套；窗外钳制 = empty 填充 = 虚拟滚动空白区
  语义）；cell_display/cell_kind 单格面（编辑定位/证据高亮钩子）。
- **单一真值纪律**：typed 值与 display 均出自 mdc-office-io 读面
  （read_book_from_bytes——同一套安全门：字节 52MB/表数/包围盒
  三前置检查）；宿主不建第二份表格数据（§二.1 架构约束的执行面）。
- **安全**：MAX_SLICE_CELLS=100k 单窗硬上限（读面总量门同纪律）；
  typed 错误码三枚（grid_read_failed/grid_sheet_index/
  grid_slice_too_large——宿主 JSON.parse 可结构化）。
- **测试**：纯内核 pub 化（sheets_core/slice_core JsValue-free）+
  2 原生测试（合成簿往返：数值原值/前导零文本原样[007 不变 number]/
  部分窗/百行外空窗钳制恰 4 empty/表索引错误/窗规模错误/上限常量）。
- 门禁（亲跑）：G1 **730/730** 零失败（+2）/ fmt 0 / clippy 恰 2
  wasm_bindgen 基线 / wasm32 check -p mdcore-wasm --all-targets
  0 错误 / web pkg 重建（build-wasm.sh）。

Stage Summary:
- 阶段 5 启动：表格表面数据面就绪（表格 UI 产品化的数据源）。
- mdcore main fc25f06→**e8bbc7f**（p5-t1）已推送。
- 阶段 5 后续路线（§八 依序）：表格 UI 产品化 → 证据界面 wasm 化
  （workbench 字节入口 + 内存 sidecar 两前置件）→ PDF 表面渲染
  路径设计决策 → Canvas 表面 → wry 壳真机集成（选型记录既列）。

---
Task ID: p5-t2
Agent: 主代理 + web 实施子代理 + 对抗审查子代理

Task: 阶段 5 §八.3 表格 UI 产品化——切片/虚拟滚动/局部更新/冻结
区域/单元格编辑/复制粘贴/键盘导航/计算结果/证据高亮全链 + 读-改
重建导出闭环（§八.3 全条目）。

Work Log:
- 环境第 19 次重置恢复：rustup 重装 1.99.0+wasm32+clippy+rustfmt
  （wasm-bindgen CLI 0.2.129 子代理侧补装）；双仓 ff 拉齐
  （cleanroom 7d539ff / mdcore e8bbc7f——前会话阶段 0-4 全收口 +
  p5-t1 表格数据面已在远端，本地仅旧克隆）。
- 子代理并行实施（用户指令「用子代理加速」）：web 实施子代理完成
  xlsx_grid.ts 1162 行 + index.html + main.ts 接线 + 夹具生成器 +
  e2e 20 用例初版（19/19 过 + 1 SKIP 等待 wasm 扩展）；主代理并行
  实施 Rust 侧（office-io rebuild.rs 重放器 + wasm export_with_edits
  + 测试 13 个）。
- 汇合修正：wasm 方法调用面统一 snake_case（全仓惯例——子代理用了
  camelCase；exportWithEdits → export_with_edits）；SKIP 项转全断言。
- 对抗审查子代理（用户指令「审核代码」）：2 MUST-FIX 实证复现——
  M1 unique_sheet_name 死循环（基底≥30 chars 时后缀拼接恒超 31 →
  release 下 wasm 调用挂死；良性输入可触发——两张表前 31 字符相同
  即碰撞）+ M2 sanitize 截断后缺重 trim（第 31 位引号残留 →
  add_sheet 拒绝 → 整簿导出失败）；6 SHOULD-FIX（S1 extent 契约/
  S2 防线文档错位/S3 merge 非 Text 左上扁平化未登记/S4 前端写面域
  零感知/S5 非有限数失真/S6 测试盲区六项）。
- MUST-FIX + SHOULD-FIX 全修复：表名防线重写（基底重截恒可终止 +
  重 trim）+ 3 审查回归锚测试（quote31 通过性/三表去重时限断言/
  扁平化登记）+ sheets_core 双暴露 ext_rows/ext_cols + 前端写面域
  钳制（MAX_GRID_ROW/COL 镜像）+ 账本 100k 上限 + parseCellInput
  非有限回退 + copy 截断提示 + __gridSetSelection 探针 + undo 栈
  语义用例（同格两版本逐次回退）。
- e2e 调试自纠 3 处：测试序列错误（Enter 确认后下移——同格多编辑
  须显式回格）/序贯状态契约破坏（清场位回 (2,1)）/repaintSelection
  方法名（refreshSelectionClasses）。

Stage Summary:
- §八.3 全条目达成：切片（slice_json 窗口）/虚拟滚动（三层 + DOM
  池化局部更新）/冻结区域（首行首列四层双轴钉扎）/单元格编辑（命令
  账本 + 撤销栈语义）/复制粘贴（TSV 双向 + 截断提示）/键盘导航
  （完整键位）/计算结果（display 面 + 公式缓存值）/证据高亮（钩子
  面就绪——证据链阶段接入）+ 读-改重建导出闭环（C3 替代路径产品化
  + 保真例外显式登记）。
- 门禁（亲跑）：G1 743/743 零失败（+13）/ fmt 0 / clippy 双面恰 2
  基线 / scan PASS / lock 零 diff / wasm32 check 0 错 / web 全回归
  （smoke 15 + 既有四套 42 + contract 18）+ 新 e2e 20/20 全 PASS。
- mdcore main e8bbc7f→823414a（p5-t2）已推送。
- 阶段 5 剩余（§八 依序）：§八.1 壳与证据界面 wasm 化（workbench
  字节入口 + 内存 sidecar 两前置件）→ §八.4 PDF 表面 → §八.5
  Canvas 表面 → wry 壳真机集成。

---
Task ID: p5-t3
Agent: 主代理 + web 实施子代理 + 对抗审查子代理

Task: 阶段 5 §八.1 壳与证据界面——workbench 字节入口 + 内存
sidecar 两前置件 + web 证据侧栏全链。

Work Log:
- 子代理并行：web 实施子代理完成 evidence_panel.ts 950 行 +
  index.html 双栏布局 + 夹具生成器 + e2e 16 用例初版（14 SKIP 等
  wasm）；主代理并行实施 Rust 侧（Io 双后端内存会话 + WasmWorkbench
  wasm 出口 + 测试 8 个）。
- 汇合修正：BigInt 边界（wasm-bindgen u64 参数须 BigInt——面板
  7 处 atMs 传参 + Ctor 签名）；restore 两步式对齐（restore +
  attach_source——子代理初版是平行数组单调用）。
- 对抗审查子代理：3 MUST-FIX——M1 export_clean 次序漂移（重构把
  check_dest 挤到审计后——dest 拒绝/写失败路径留下幽灵 clean_export
  审计 + sidecar 落盘，违反「拒绝=零状态变化」纪律）/M2 面板
  reverify 按定位器类型分派（source_locator 恒 xlsx_cell → claim
  侧失效恒走 source 通道静默恢复——绕过 claim 锚唯一恢复通道纪律，
  e2e 把漏洞锁成预期）/M3 target_scale as u8 截断（256→0 静默改值）；
  5 SHOULD-FIX（S1 claimed_value 静默吞跳过验证门/S2 Files 守卫
  不对称/S3 policy 畸形 fail-open/S4 generation_batch 伪造默认/
  S5 restore 后 sourceSeq 撞车）。
- 全修复 + 回归锁：M1 三段拆分（verified 零副作用验证核 + 审计后置
  + check_dest 前置）/M2 失效侧分派（claim 锚指纹对照 → 断则
  reverify_claim 重绑；e2e 锚重绑断言 anchor_before != anchor_after）/
  M3 try_from + S1-S5 + N1/N2；e2e 16/16 全 PASS 含新锁。
- 磁盘满第 3 次事件：target/debug/incremental 4.1G 清理（9.3G/100%
  → 4.7G/51% 空闲），collab_composition 3 测试 StorageFull 假失败
  排除（清理后全绿）。

Stage Summary:
- §八.1 两前置件达成：workbench 字节入口（ingest_*_from_bytes/
  start_memory/open_memory——验证核心单一，Io 仅分派 IO 边界）+
  内存 sidecar（sidecar_json 单点投影与磁盘字节同构，持久化由宿主
  决定——wasm 零 std::fs 纪律）。
- web 证据侧栏全链：三态徽标卡片 + 审核操作（review gate human:
  前缀）+ 失效侧 reverify + sidecar 导出/恢复 + 干净正文导出 +
  claim 跳转钩子（evidence-jump 事件——正文视图滚动接入留后续）。
- 门禁（亲跑）：G1 751/751（+8）/ fmt 0 / clippy 恰 3 同型基线 /
  scan PASS / lock +2 内部边 / wasm32 0 错 / web 八套全绿（evidence
  16 + xlsx_grid 20 + 既有六套）。
- mdcore main 823414a→3130b8a（p5-t3）已推送。
- 阶段 5 剩余（§八 依序）：§八.4 PDF 表面（页面缓存/缩放滚动/选区/
  批注叠加——复用 mdc-pdf 既有安全边界）→ §八.5 Canvas 表面 →
  wry 壳真机集成。

---
Task ID: p5-t4
Agent: 主代理 + 调研子代理 + web 实施子代理 + 对抗审查子代理

Task: 阶段 5 §八.4 PDF 表面——页面缓存/缩放滚动/文字选区/批注及
证据区域叠加；渲染路径设计决策定谳。

Work Log:
- 调研子代理（含两项实测 + 三项网络一手核验）：路线定谳 B = pdf.js
  渲染 + mdc-pdf lopdf wasm 写回（渲染分离、真相不分离）。A pdfium-
  wasm 否决（bblanchon 堆不可增长 = 多页档 OOM + wasm 无线程 S4 塌缩
  + 上游四轮断裂史）；C 壳侧渲染 = wry 落地后 native 升级位
  （protocol.rs 信封使渲染器可替换）。
- 子代理并行：web 实施子代理 pdf_view.ts 1991 行 + 夹具生成器 +
  e2e 14 用例 + **契约 mock 干跑全过**（wasm 汇合前全路径验证）；
  主代理 Rust 侧：mdc-pdf 便携化（PortableInstant——修 wasm trap
  'time not implemented'）+ trait product_bytes 产物取回面 + 新
  crate mdc-pdf-wasm（5 测试）。
- 汇合修复：snake_case 调用面（pdfOpenScan→pdf_open_scan——p5-t2
  同型教训）+ wasm trap panic hook 定位 + Instant 三调用点便携化 +
  glue 名（--out-name pdf_wasm）。
- 对抗审查：1 MUST-FIX（COMPLIANCE 事实错误——零传递依赖为假
  [@napi-rs/canvas 13 包 optional 传递面] + 首个三方位 false）+
  3 SHOULD-FIX（S-1 flush 文档被插页夺走/S-2 显示面 L2 像素钳制
  缺失[400% 缩放 + 3x 屏单页 279MB canvas 分配]+evict 单条滞留/
  S-3 双脏修订链无锁）全修 + NOTE 顺手（build 死分支 + hook 接线
  ——时序 bug 自纠[初始化前调用致 13 SKIP，移到 default() 后恢复]）。
- e2e 自纠：b64 前缀 != 字节前缀（3 字节块对齐——双修订链断言改
  字节级解码比对）。
- 磁盘满第 4 次事件（ld Bus error + tmp dir 拒绝——大测试二进制
  清理 5.3G 恢复）。

Stage Summary:
- §八.4 四要件达成：页面缓存（64MB ImageBitmap LRU + L2 钳制镜像）/
  缩放滚动（scale 控件 + IntersectionObserver 懒渲染 + 页码跟踪）/
  文字选区（pdf.js textLayer——Firefox 级原生）+ 批注（四 kind 全链
  + lopdf 增量写：前缀逐位/幂等/加密写拒语义锁 wasm 链路全数继承）
  + 证据区域叠加钩子（evidence-jump 事件——真证据链接后续）。
- 渲染路径决策落档：B 定谳（渲染分离、真相不分离——会话是单一
  真相，pdf.js 纯显示面）；C = 壳实体后升级位。
- 门禁（亲跑）：G1 767/767（+16）/ fmt 0 / clippy 恰 3 同型 +
  wasm32 0 / scan PASS / e2e pdf_view 15/15（1 SKIP 登记）+ web
  回归七套全绿 + typecheck 0。
- mdcore main 3130b8a→37399eb（p5-t4）已推送。
- 阶段 5 剩余：§八.5 Canvas 表面（实际节点编辑与协同操作——
  mdc-canvas headless 已备，web 面消费）→ wry 壳真机集成。

---
Task ID: p5-t5
Agent: 主代理 + 调研子代理 + web 实施子代理

Task: 阶段 5 §八.5 Canvas 表面——实际节点编辑与协同操作（四表面
收口轮）。

Work Log:
- 调研子代理：mdc-canvas 全 API 面（八命令/双栈撤销/指纹/事件/
  协同四方法）+ 唯一 wasm32 障碍定位（UndoManager SystemTime trap
  ——SessionOptions 无时钟注入口）+ 序列化面空白（无 serde——
  wasm 层新造 flat JSON）+ WasmCanvas 方法面定稿。
- 子代理并行：web canvas_view.ts 1585 行（四层结构 + SVG 保留模式
  + 双实例协同演示层）+ e2e 15 用例 + **契约 mock 干跑 15/15**
  （含双实例收敛/远端不入栈/CDP 拖拽）；主代理 Rust 侧：mdc-canvas
  时钟注入面（SessionOptions.clock_ms + HostClock 适配器——
  canvas 23 测 + e2e_demo 零回归）+ WasmCanvas 模块 + 测试 7。
- 汇合修复：BigInt 四处（client_id u64——p5-t3/t4 同型教训第三
  次：u64 参数恒 BigInt）；object_json "null" 串 vs JS null
  （Option<String> 契约直配）；NaN 断言分层（JSON 层 cv_bad_json
  先拒——serde 对非法 JSON 字面量的第一道防线，cv_non_finite 是
  Rust 直构 API 的第二道——诚实分层登记）；测试自纠 2 处（raw
  string "#" 冲突 / 双向同步 SV 快照顺序 / base64 期望值手算错）。
- 磁盘满第 5 次事件。

Stage Summary:
- §八.5 达成：实际节点编辑（八命令全链 + 拖拽 LWW 提交粒度 +
  撤销重做双栈 + 分组 + 属性编辑）与协同操作（双实例直连：
  事件 updates base64 转发 + encode_diff 追赶 + 收敛断言 + 远端
  不入本地栈）。
- 阶段 5 四表面全部落地（正文[CM6 既有]/表格/证据侧栏/PDF/
  Canvas）——「四类表面均有可重复的交互测试及性能数据；不存在
  UI 第二权威状态」验收面成立（wasm 会话为唯一权威 + 宿主零第
  二数据源，全表面同纪律）。
- 门禁（亲跑）：G1 769/769 / fmt 0 / clippy native+wasm32 恰 4
  同型基线 / scan PASS / lock +1 内部边 / web 十套全绿（canvas
  15/15 + 既有九套）。
- mdcore main 37399eb→2a67e77（p5-t5）已推送。
- 阶段 5 剩余：wry 壳真机集成（选型记录既列——开发环境轮 R24-d
  性质，桌面壳实体）；随后阶段 6 端到端质量验收（§九 十项清单）。

---
Task ID: p6-t1 + p6-t2
Agent: 主代理

Task: 阶段 6 启动件——四表面集成旅程 e2e（§九 实际使用路径）+
wry 壳编译面验证。

Work Log:
- integration_journey.py（端口 8941）：单页用户旅程串联四表面——
  文档编辑往返/表格开滚编导/证据八步骨架[计算 4210.000 + 审核 +
  sidecar schema]/PDF 高亮增量保存[前缀=原字节]/Canvas 双实例协同
  收敛/四 tab 轮转共存 + 零 pageerror。全走既有探针面（零产品代码
  增量）。调试自纠 3 处（服务 root/懒加载 ready 等待时机/grid
  viewport focus + 探针名）——**7/7 全 PASS**。
- wry 壳编译面（p6-t2）：webkit2gtk-4.1/gtk3 沙箱缺席——真机
  集成结构性受阻（需桌面环境 + 系统 webview 库），登记为开发环境
  轮边界项（选型记录既列；非产品代码阻塞）。

Stage Summary:
- 阶段 6 骨架启动：§九.1-4 的实际使用路径验证面就位（四表面真实
  互操作 + 跨表面共存）——这是端到端验收的组合测试底座，后续
  §九.5-8 项（并发/异常输入/退出恢复的 web 面组合）在其上扩展。
- mdcore main 2a67e77→6daeace（p6-t1）已推送。
- 剩余（阶段 6 完整清单）：§九.5-8 web 面组合[协同双端 web 化/
  sidecar 异常路径 web 面/超大输入组合] + §九.9 打包真机实测
  [环境阻塞——同 wry 壳] + §九.10 可选宿主适配[独立接口验证，
  无核心依赖]。

---
Task ID: p6-t2 + p6-t3..t6（欠账补记）
Agent: 前会话实施 / 本会话补记（main，Super Z）

Task: 补记 mdcore 06ba701 与 b4e6a12 两提交的工作记录（前会话已推送
代码但文档未落档——本会话 Gate 0 拉齐时发现文档欠账）。

Work Log:
- 06ba701（p6-t2）：协同双端 web 化基础设施（§九.5/§九.8 前置件）——
  mdc-collab SessionOptions.clock_ms 加性注入面（wasm32 无 std 时间）+
  mdcore-wasm 第 15 模块 collab.rs（WasmCollabSession 全方法 + cl_*
  12 错误码 + 纯内核 parse/receipt/event/block）+ 原生测试 8 件 +
  web wasm_boot.ts 单飞 init 门（六调用点 WeakMap 序列化，双实例化
  SEGV 根治）+ collab_channel.ts 464 行（BroadcastChannel 对等网格
  七消息 + 13 探针 + 断连重连 catch-up）+ canvas_view ?cc 槽位化；
  G1 774→782。
- b4e6a12（p6-t3..t6）：§九.5-8 web 面组合套件四件——
  concurrent_web.py（§九.5，16 检查：并发/过期命令/撤销重做/删除
  复活/双页零 pageerror）+ sidecar_faults_web.py（§九.6，15 检查：
  损坏六面/丢失/版本不匹配/异常退出恢复）+ oversized_web.py（§九.7，
  18 检查：10MB 渐进/切片超限/敌意 22 连发 no-hang）+ lifecycle_web.py
  （§九.8，15 检查：退出重启/房间状态存活/快照恢复/断连重连/晚加入
  catch-up）+ mdc-canvas 撞号跳段修复（next_id 环内 O(1) 占位检查）
  + collab 晚加入 catch-up 方向修复（joiner 显式 sync_req）+ 渲染层
  自愈面；G1 782→783；web 15 套共 202 检查点亲跑全绿。

Stage Summary:
- 阶段 6 §九.1-8 全部落地（1-4 = p6-t1 旅程 7/7；5-8 = 本两条）。
  §九 剩余：§九.9 打包真机[本会话 s5b 推进] + §九.10 可选宿主适配。
- 文档欠账根因：前会话推送代码后未及落档即耗尽；补记内容以 git log
  提交说明为单一事实源。

---
Task ID: s5a-env
Agent: 子代理（general-purpose）执行 + 主代理复验

Task: webkit2gtk-4.1/GTK3 dev 面无 root 自愈（wry/tao 壳前置——推翻
「真机集成结构性受阻」登记）。

Work Log:
- 闭包解析 scripts/webkit_closure.py（libwebkit2gtk-4.1-dev 出发递归，
  92 包清单）→ apt download 98 deb（66M pool）→ dpkg -x 提取树
  optroot/root（209M）→ env.sh（PKG_CONFIG_PATH/LD_LIBRARY_PATH/
  GI_TYPELIB_PATH/XDG_DATA_DIRS；.pc 重写 prefix；glib-compile-schemas）
- rustup 1.99.0+wasm32+clippy+fmt 装于 optroot（沙箱 cargo 重建）
- 三路验证史：bwrap（Permission denied）/proot（seccomp SIGSYS 31）
  不通 → LD_PRELOAD PLT 层 exec 路径 shim（libexecpath_shim.c）：
  WebKitNetworkProcess/WebKitWebProcess posix_spawn + injected-bundle
  dlopen 三映射（原路径存在则零行为——实机部署天然退化）
- mdshell-native 依赖实体化（wry 0.46/tao 0.30/gtk 0.18 循 ADR-H1
  注释原值）+ native 模块（WryHost/ShellConfig/run_smoke）+
  examples/xvfb_smoke
- 主代理复验（不采信未亲跑结果）：pkg-config webkit2gtk-4.1=2.54.0/
  gtk+-3.0=3.24.49/soup 3.6.5 三链通；cargo check 过；Xvfb 冒烟
  EXIT=0（tao loop + webview responsive）

Stage Summary:
- 「webkit2gtk/gtk3 沙箱缺席=真机集成结构性受阻」（p6-t1 时点 HANDOFF
  登记）被推翻：是可自愈缺件而非结构性阻塞。39bd74c 先例（apt
  download 无 root 提取）放大到 webkit 全闭包成立。
- 沙箱自愈工件存 /home/z/my-project/{optroot,scripts}/（仓外不入 git；
  optroot/env.sh + scripts/xvfb_shell_run.sh 为入口）。

---
Task ID: s5b-wry-deb
Agent: main (Super Z)

Task: wry 壳实机深度验证（四表面探针）+ mdapp:// 产品级 bundle 加载
+ §九.9 deb 打包面 + SBOM 合规 + 全门禁复验 + 文档收口。

Work Log:
- 完整 web 工作台重建：build-wasm.sh（1m32s，wasm-bindgen 0.2.129
  cargo install 精确锁版）+ build-pdf-wasm.sh + npm ci + build.mjs
  → dist/（main.js 1.9M + 双 wasm pkg + pdf worker）
- 实机三级递进（Xvfb 真进程，全部亲跑）：①about:blank 存活响应；
  ②file:// 加载 dist——title 出现但探针全空轮询 12 次不初始化；
  ③http:// 对照——全绿（blockCount:9/cmEditor/grid/ev/pdf/canvas/
  collab）→ WebKit file:// wasm/ESM 限制实证（dist 自包含声明在
  file:// 不成立——build.mjs 注释假设修正）
- 【工程发现 1】wry 0.46.3 IPC 缺陷：with_ipc_handler 回调内
  Request::builder().uri(webview.uri().unwrap().to_string())——
  file:///（空 authority）在 http::Uri 下 Err(InvalidUri(InvalidFormat))
  → 宿主 panic（隔离实验二分定位：about:blank 合法/file:// 必炸/
  消息内容无关）；绕开=导航拦截通道（with_navigation_handler
  Fn(String)->bool + location.assign("probe://r/"+encodeURIComponent)
  + percent_decode 宿主侧解码；probe:// 截获返回 false 取消导航）；
  缺陷注记在档未修上游
- 【工程发现 2→产品修法】mdapp:// custom protocol（with_custom_
  protocol）：serve_dist（穿越防护拒绝 .. 段含编码形/ / → index.html/
  mime 表 wasm=application/wasm/404；无缓存每请求读盘）——桌面壳
  加载本地 bundle 的产品级形态；探针轮询版 + flush 窗参数化
  （ShellConfig.flush_millis，看门狗两段式：探针→700ms~8s 回传窗
  →Exit）；mdapp:// 下四表面探针全绿 EXIT=0（wasm 核心+CM6+grid/
  ev/pdf/canvas/collab 在 WebKit 壳内真实初始化）
- shell-native 增强：probe_js/flush_millis/dist_root 三字段 +
  --probe-js/--flush-ms/--dist 参数 + 新 bin mdcore（--version 单源
  dist 解析序四层 --dist>MDCORE_DIST>/usr/share/mdcore/dist>./web/dist）
  + 单元测试 2→4（percent_decode 四形/serve_dist 默认页+mime+穿越
  四形+404）
- 【真缺陷修复】mdc-pdf pdfb_skeleton Instant::now→portable_now ×2
  （p5-t4 PortableInstant 化后 mdc-pdf 测试在 wasm32 --all-targets
  下无法编译——口径统一为最强形态）；wasm32 门禁口径更新：
  --workspace --all-targets --exclude mdshell-native（原生 GUI crate
  不属 wasm 目标=平台事实注记）
- 【§九.9 deb】build-deb.sh 两处修（# 注释行 grep -v 进 control
  [dpkg-deb 严格解析]；mv 路径 dpkg-deb 输出=<STAGE>.deb[从未实跑的
  假设]）+ dist 捆绑（usr/share/mdcore/dist 无 bundle 硬失败）→
  mdcore_0.1.0_amd64.deb（2.4M/Installed-Size 17832）→ dpkg -x
  提取形布局验证 + 包内二进制 Xvfb 首启冒烟全绿（mdapp:// 四表面
  探针 + EXIT=0 + --version 一致）；边界诚实登记：dpkg -i 安装面
  无 root 未做（提取形模拟）；AppImage 脚本在位未实跑（外部工具链
  linuxdeploy/appimagetool 未引入）
- 【合规】COMPLIANCE.md s5a-env 增量段 + SBOM 全量复跑 366 三方包
  （scripts/sbom_check.py，cargo metadata --locked 逐包 license：
  0 UNKNOWN/0 GPL/0 白名单外；r-efi LGPL 字样=MIT OR 组合可选路径；
  webkit2gtk LGPL 系统库动态链接政策复引）；NOTICES 再生成边界登记
  （compliance-baseline 分支不在 main）
- 【门禁全亲跑】G1 786/0（783+2 新测+pdfb 36/36 语义不变）/fmt 0/
  clippy mdshell-native 0（4 条修净：冗余 cast×3+any→contains）/
  wasm32 --workspace --all-targets --exclude mdshell-native 0 错误/
  scan PASS/lock +2813-308（wry/tao/gtk 闭包预期）/web 15 套全 PASS
  /磁盘满 2 次事件清理恢复（9.3G→4.3G；pool/incremental/examples/
  大测试二进制——清理模式沿用固化纪律）

Stage Summary:
- mdcore main c33bcab 已推送（b4e6a12→c33bcab，ls-remote 校验一致）。
- wry 壳实机集成达成：宿主 A 定谳（Web 前端 + wry/tao 壳）的实机
  兑现——四表面在真实 tao+wry+WebKit 进程内初始化并探针断言。
- §九 进度：1-8 全绿（既有）+ 9 deb 面实体过（构建/布局/首启冒烟；
  dpkg -i 与 AppImage 登记边界）；§九.10 可选宿主适配待评估。
- 三项工程发现入档（wry IPC 缺陷/WebKit file:// 限制/mdapp:// 形态）。
- 环境注记：磁盘满第 6/7 次事件（清理恢复在档）；~21 次重置恢复
  沿用固化程序。

---
Task ID: s5c-host10
Agent: main (Super Z)

Task: §九.10 可选宿主适配评估（治理计划收官项——「Kokoa/Slint 的
可选适配通过独立接口验证，不要求核心依赖该宿主」）。

Work Log:
- cargo metadata 依赖方向实证：mdcore（3 正常依赖）/mdshell（4 项）
  零 GUI 框架引用；mdshell-native 单向依赖 mdcore/mdshell+wry/tao/
  gtk（唯一 host 侧 crate，无反向）
- 壳替换点 = mdshell::Host trait（crates/shell/src/lib.rs:38 契约层）
  + 统一命令接口；本会话 WryHost（native 壳）与 web 壳（浏览器形态）
  即同一契约的两个独立实现——「独立接口验证」成立
- ADR-H1 维持：Slint GPL 排除不引入；「可选适配」的证明方式 =
  可替换性已被双壳实证，无需为「验证」引入新宿主实体

Stage Summary:
- §九.10 判定：**成立（零核心依赖 + 独立接口验证）**，以依赖方向
  与双壳同契约实证收口；无新实体引入。
- 治理计划 §九 清单终态：1-8 全绿 / 9 deb 面实体过（残项边界登记：
  dpkg -i、AppImage、NSIS、.app 需真机或外部工具链）/ 10 成立。

---
Task ID: maint-r1
Agent: 主代理（第 18 次重置事故恢复 + 丢失轮重做 + 推送收口）
Task: 用户 token 续能「继续做」→ push 时发现远端领先 53 提交
（Stage 5 workbench 会话线 10-19~10-24 推进），本地仓库回退到
ae4aa4d——同日早间的 r-o2-t6/maint-t1/maint-t2 三提交在环境重置中
本地全丢（reflog/fsck 零踪迹）。本轮：双仓 ff 拉齐（cleanroom
f9efe19 / mdcore c33bcab，后者 +28 提交）→ 甄别覆盖面（26.8.1
复跑证据已被并行线 REGRESSION §10.5 等价覆盖，无需重做）→ 重做
三件独有面 → 提交推送。

Work Log:
- 事故定谳：push rejected → fetch 后 HEAD..origin/main = 53 提交
  （p0-p6/s5b/s5c 系列，全部 workbench 线）且 origin/main..HEAD
  = 0；本地 HEAD = ae4aa4d（10-18 r-o2-t5-doc）；丢失提交
  ae6bec5/717628c/871a6fe 对象不存在（git cat-file 逐试探）+
  reflog 只到 ae4aa4d + fsck 零 dangling——第 18 次重置把仓库
  快照回退到 10-18 拍摄点（早间轮的三提交在快照之后产生，随重置
  抹灭）。cargo/rustup、.lo26/.lo262 前缀、scripts/ 同批丢失。
- 并行线甄别：远端 53 提交含 REGRESSION §10.5（LO 26.8.1 复跑
  22 用例 + garbage-bytes 破裂 + anchor-survival 跨版本稳定——
  结论与丢失的 r-o2-t6 完全一致）+ §10.6（render_guard 守护器
  原型）——26.8.1 证据腿已被覆盖；mdcore 远端 c33bcab（+28：
  p6-t3..t6 web 面组 + s5a/s5b wry/tao 壳实机集成 + deb 打包，
  G1 786/web 15 套全绿，s5b 速览在档）。未覆盖面 = yrs 上游跟踪
  / TDF 25.2 EOL / 26.2.6 复跑三件。
- 双仓 ff 拉齐：cleanroom ae4aa4d→f9efe19、mdcore c4856f7→
  c33bcab（token fetch，零冲突——本地无独有提交）。
- 重做①yrs 上游跟踪（三锚点重验，结论与丢失轮一致）：旧路径
  github.com/y-c-rs/yrs HTTP 404；crates.io repository 字段 =
  github.com/y-crdt/y-crdt/（monorepo 转移）；#676 新家保号在场
  仍 OPEN（页面嵌入 JSON state 三处一致）+ 标题即 UAF 主题；修复
  PR #682 在途（OPEN 未合并，作者 jonliu03，「GC: collect a
  collected type's children even when undo keeps them」——正中
  d1b-t1 ASan 定位根因面）；crates.io 最新 0.28.0（2026-09-17，
  = spike 钉版无新 release）。
- 重做②TDF 25.2 EOL：25.2.7 = final maintenance release（TDF
  Community Blog 检索 datetime 2025-10-30 直证，明示建议升
  25.8.x）；当前 still = 26.2.6（公告 2026-09-04）/ fresh =
  26.8.1。本机系统基线 25.2.3.2 所在分支上游停维一年。
- 重做③26.2.6.3 still 线复跑：TDF 官方 tarball 217,285,735 B
  （gzip -t 完整性验证，断点续传两段）用户前缀解包 42 deb
  （scripts/unpack_lo2626.sh 重写——原文件随重置丢失）+ shim；
  版本串 26.2.6.3 8221e31b 实测；R-O2 套件零改动双次复跑 = 22
  = 10 pass + 12 pinned + 0 fail SUITE PASS 双次稳定逐用例同判
  ——与基线完全同判零破裂（garbage-bytes 仍 PINNED：rc=0 + 页 1
  产出冷 1.34s/RSS 274MB；10 有效夹具零漂移含 anchor-survival 16
  断言三钉定；双确定性门双次 pass）。rc=0 兜底面收窄时间窗定谳：
  25.2 开放 → 26.2 仍开放 → 26.8 收窄（收窄变更在 26.2→26.8
  之间）。
- 文档登记：REGRESSION §10.7（接并行线 §10.5/§10.6 编号）+
  DECISIONS #19 跟踪增量 + DECISION_SHEET 批次 3 三选齐备/批次 4
  #19 行/剩余风险 #1 #3 + DECISION_GROUPS G5 TDF 证据 + HANDOFF
  新速览（并行线速览降为历史保留）。
- 推送：cleanroom f9efe19→maint-r1 提交（token 单次环境变量，
  push 脚本内存 Basic 头，.git/config 零 token）；mdcore 零新
  提交（ff 拉齐态即推送态）。
- 磁盘治理：.lo262dl 下载目录（415MB）用毕即删（.lo26dl 先例）。

Stage Summary:
- 第 18 次重置事故收口：早间三轮工作丢失面全部重做或确认被并行
  线覆盖，零未恢复项；双仓与远端一致推送态。
- 批次 3 版本基线三选证据腿齐备（25.2.x EOL 存量 / 26.2.x 零翻动
  最平滑 / 26.8.x 一良性破裂需翻钉一项）——裁决输入完备。
- yrs 升级窗口维持关闭（PR #682 在途未合并 + 0.28.0 无新
  release）；skip_gc 全编队不变量维持。
- mdcore 零接触零提交（ff 拉齐推送态，门禁口径以 s5b 速览为准）；
  剩余门控 = DECISION_SHEET 批次 1/3/4（用户批注即全线解锁）。

---
Task ID: maint-t3-r1
Agent: 主代理（环境自愈轮）
Task: 用户「继续至少10轮次」指令第 1 轮——第 19 次环境重置
（cargo/.lo26/scripts 大部丢失）后环境完整性核验与自愈。

Work Log:
- 核验发现：cargo/rustup 全缺、~/.lo26（26.8.1.1 前缀）缺失、
  ~/.lo262（26.2.6.3）在场、scripts/ 仅剩 2 件
  （unpack_lo2626.sh / yrs_dep_license_scan.py）、系统 LO 25.2.3.2 在场。
- rustup 重装（用户前缀，钉 1.99.0 + minimal + rustfmt/clippy +
  wasm32）：首次 300s 超时被杀但工具链已落盘；rustup 自动恢复
  "partially installed toolchain" 补齐 7 组件。
- 验证齐备：cargo 1.99.0 (5f94df478) / rustc 1.99.0 (b940084d7) /
  rustfmt 1.10.0（= T1 审计 F5 钉版口径）/ clippy 0.1.99 /
  wasm32-unknown-unknown target——与 mdcore rust-toolchain.toml
  声明完全一致。

Stage Summary:
- 环境自愈完成，可支撑全量门禁亲跑（轮 2）。
- .lo26（26.8.1.1）缺失登记为欠账——如需 26.8.x 复跑需重装
  （415MB tarball 下载，批次 3 裁决后按需执行）。

---
Task ID: maint-t3-r2
Agent: 主代理（门禁独立复验轮）
Task: 用户「继续至少10轮次」指令第 2 轮——mdcore c33bcab
全量门禁亲跑（不采信并行线自报口径，独立复验）。

Work Log:
- fmt 门禁：**发现 c33bcab fmt 破裂**（crates/pdf/tests/
  pdfb_skeleton.rs:1768 use 语句排序不符 rustfmt 1.10.0 口径，
  并行线提交信息声称"全门禁复验"与实际不符——独立复验抓到）。
- clippy --workspace --all-targets：首跑 rc=101 挂 gdk-sys 0.18.2
  （mdshell-native 的 GTK 绑定，第 19 次重置抹掉 webkit/gtk dev 面
  + sudo 不可用）→ 排除 mdshell-native 后核心面 rc=0。
- clippy warning 口径：5 条 wasm_bindgen static_method_of 误报 =
  在档基线 2 条（lib.rs:212/221 → 行号漂移至 216/225）+ p6-t2
  并行线新增 3 条（canvas.rs:377 / collab.rs:504 /
  workbench.rs:352，同性质同根因，非 error 不挡门禁）。
- 全量测试（排除 mdshell-native）：**782/782 零失败**（并行线 786
  口径差 4 = GUI 壳 crate 测试，环境受阻欠账非代码缺陷）。
- wasm32 check（mdcore-wasm）：rc=0。
- fmt 修复：cargo fmt 单文件 1 行（+1/-1）语义零变化 → fmt
  rc=0 PASS + mdc-pdf 68/68 复验不变 → mdcore 提交 3e1f616。

Stage Summary:
- c33bcab 独立复验结论：核心面全绿（782/782 + clippy rc=0 +
  wasm32 rc=0），唯一实质破裂 = fmt 单行（已修复 3e1f616）。
- mdshell-native（wry/tao 壳）在本环境构建受阻：gtk3 dev 缺 +
  sudo 需密码——s5a-env 无 root 自愈工件（optroot 209M 仓外）
  已随重置丢失，重建 = 欠账登记（66M pool 下载 + shim 三映射）。
- 并行线"全门禁复验"声称与 fmt 实际矛盾已登记——后续采信其
  自报口径时需保持独立复验纪律。

---
Task ID: maint-t3-r3
Agent: 主代理（+28 提交面审轮）
Task: 用户「继续至少10轮次」指令第 3 轮——mdcore c4856f7→
c33bcab（+28 并行线提交）差异结构面审（44,320 行插入，
135 文件）。

Work Log:
- 三大不变量核查全过：①yrs = 0.28 钉版维持（canvas/collab
  Cargo.toml 双钉 + sync feature，零升级）；②skip_gc B1.1 全编队
  不变量维持（collab/src/session.rs:193 `opts.skip_gc = true;
  // B1.1：无条件，无配置面` 单点构造在档，与 d1b-t1 纪律原文
  一致）；③**mdcore 核心 crate（crates/mdcore/）零触碰**——
  批次 2「零公开模型触碰」纪律在并行线被遵守。
- 新增面全貌 = 纯增量：canvas 测试 5 件（concurrency/events/
  evidence_assoc/matrix/undo_redo）+ collab 测试 9 件（audit/
  concurrency/events/hostile/marks_readback/matrix/presence/
  recovery/undo_redo）+ mdbridge bridge_ops + web/ python 套件
  15 件（lifecycle_web 901 行/oversized_web 846 行/sidecar_
  faults_web 517 行等）+ shell-native（wry 0.46/tao 0.30/GTK3，
  ADR-H1 无 Slint）。
- office-io 唯一 src 触碰 = 新增 rebuild 模块（390 行 + 测试
  441 行 + bench_limits 例程）：读-改-重建 C3 口径，复用 r4-t1
  单一安全调用点 replay_cell，非静默降级纪律（RebuildStats::
  skipped 显式登记），U+FFFE/U+FFFF typed 拒——纪律文档与既有
  write 模块同水准。lib.rs 仅 +1 行 `pub mod rebuild;`，既有
  公开面零修改。
- mdshell-native Cargo.toml description 自记 s5a-env 自愈方法
  （source optroot/env.sh 后 check）——optroot 已随重置丢失，
  与轮 2 受阻结论互证。

Stage Summary:
- +28 提交面审结论：纪律保持（三不变量全过 + 纯增量 + 非静默
  降级文档），无结构风险；并行线质量口径与主线一致。
- web/ python 套件（15 件）依赖 playwright/chrome 面待轮 8
  独立验证。

---
Task ID: maint-t3-r4
Agent: 主代理（yrs 上游跟踪增量轮）
Task: 用户「继续至少10轮次」指令第 4 轮——yrs 上游三锚点
复验（对 maint-r1 2026-10-25 早间结论的增量检查）。

Work Log:
- 锚点① PR #682（GC: collect a collected type's children
  even when undo keeps them）：仍 OPEN 未合并，最后更新
  2026-10-03（jonliu03 评论），目标分支 y-crdt:main——与
  maint-r1 时点一致零变化。
- 锚点② #676（heap UAF，undo+并发 GC）：仍 open，无修复 PR
  合并，最后活动 2026-09-29——零变化。
- 锚点③ crates.io：最新仍 0.28.0（2026-09-17）= spike 钉版，
  近三版 0.28.0/0.27.4/0.27.3——无新 release。

Stage Summary:
- 上游跟踪增量 = 零（三锚点与 maint-r1 全一致）；yrs 升级
  窗口维持关闭（PR #682 在途未合并 + 0.28.0 无后继）；
  skip_gc 全编队不变量维持（轮 3 已实证 session.rs:193）。
- 下次跟踪检查点：PR #682 状态翻转（merge/close）或
  0.28.x/0.29 发版。

---
Task ID: maint-t3-r5
Agent: 主代理（R-O2 套件复验轮）
Task: 用户「继续至少10轮次」指令第 5 轮——R-O2 回归套件在
第 19 次重置后环境复验跑（fixtures git 内 + 系统 LO 25.2.3.2
基线口径）。

Work Log:
- python3 regress.py 全量 22 用例：**22 = 10 pass + 12 pinned
  + 0 fail，SUITE PASS**（夹具确定性门 pass + 跨进程确定性门
  pass 8 断言）。
- 与基线（r-o2-t5/REGRESSION §10）完全同判零漂移：garbage-bytes
  仍 PINNED rc=0、bad-xml/missing-content-types 等 12 钉定面
  全维持、10 有效夹具（含 anchor-survival 16 断言三钉定）零漂移。
- 环境（第 19 次重置后）套件自愈确认：fixtures/ 在 git 内零丢失、
  regress.py/render_guard.py/rss_wrapper.py 全在档——套件工件
  未受 scripts/ 丢失影响（套件本体在 cleanroom 仓库内）。

Stage Summary:
- R-O2 套件新环境复验 = 基线同判零漂移，渲染管线证据腿在当前
  环境完整在场可用（maint-t3 口径）。
- 套件工件与 scripts/ 丢失面零交集（套件在仓内，丢失的是仓外
  辅助脚本——轮 7 盘点）。

---
Task ID: maint-t3-r6
Agent: 主代理（TDF 版本线全景补腿轮）
Task: 用户「继续至少10轮次」指令第 6 轮——批次 3 版本基线
三选的证据增量：25.8.x 第四选可行性 + 各分支 EOL 时间表。

Work Log:
- 第四选（25.8.x）定谳 = 死路：25.8.7 = final maintenance
  release（TDF 公告 2026-05-12）；25.8 分支 EOL = 2026-06-12
  （TDF Blog 原文 "reached end of life on 12 June 2026 and has
  received no security updates since that date. Users of 25.8.x
  should update to 26.2.5"）。25.2 EOL 公告建议的升级目标自身
  已死 4.5 个月——该建议已过时，官方现引导 25.8.x 存量 → 26.2.5。
- **26.2 分支 EOL = 2026-11-30（一手 TDF 26.8 发布新闻稿原文
  "The LibreOffice 26.2 branch reaches end of life on 30 November
  2026" + linuxiac 印证）——距当前 2026-10-25 仅剩约 5 周**。
- 26.8 分支 EOL = 2027-06-13（endoflife.date，fresh 线，
  26.8.1 发布 2026-10-08）。
- 四分支全景：25.2 EOL 2025-11-30（死 11 月）/ 25.8 EOL
  2026-06-12（死 4.5 月）/ 26.2 EOL 2026-11-30（**剩 ~5 周**，
  still 线 26.2.6 活跃）/ 26.8 EOL 2027-06-13（剩 ~7.5 月，
  fresh 线）。
- 口径史注记：官方生产环境推荐线曾迁移（2026-03 报道引官方
  推荐 25.8.x → 2026-09 26.8 公告推荐 26.2.5）——TDF 惯例
  still = fresh 的前一线。

Stage Summary:
- **批次 3 裁决输入重大时效增量**：26.2.x「零翻动最平滑」的
  维护窗口仅剩 ~5 周——钉它 = 2026-11-30 后重蹈 25.2 覆辙
  （变 EOL 存量再迁一次）；26.8.x 的「一良性破裂」代价对应
  最长维护窗口（至 2027-06，官方 fresh 线）。第四选 25.8.x
  排除（死路）。
- 三选证据格局更新为：25.2.x（死 11 月）/ 26.2.x（零翻动
  但 5 周短窗）/ 26.8.x（一良性翻钉 + 7.5 月长窗）——推荐
  默认的裁决权仍归用户，DECISION_SHEET 证据面登记本轮增量。

---
Task ID: maint-t3-r7
Agent: 主代理（scripts 欠账盘点与 SBOM 工具重建轮）
Task: 用户「继续至少10轮次」指令第 7 轮——scripts/ 丢失面盘点
+ 关键工具重建 + 对 mdcore 当前 HEAD 实跑 SBOM 合规门禁。

Work Log:
- 丢失面盘点（git 双仓历史检索定谳）：agents_structure_extract/
  agents_forensic_scan **未丢**（在 cleanroom agents-md-audit/
  tools/ 仓内零漂移在场，worklog 的 scripts/ 路径是运行时引用）；
  真丢 5 件 = verify_authenticode.py（Acrobat 审计一次性工具，
  结论已入档）/ webkit_closure.py + xvfb_shell_run.sh（optroot
  重建前置件）/ sbom_check.py（合规门禁常驻）/ nextbatch_search.sh
  （非关键）。
- **sbom_check.py 重建完成**（循 worklog s5b 功能描述 + COMPLIANCE
  §3 门禁 3 规格）：cargo metadata --locked 逐包 license，OR/AND
  组合语义（OR 任一全白分支过——r-efi 先例；AND 全白才过）、
  斜杠旧式双许可等价展开（MIT/Apache-2.0 → OR）、括号分配律
  展开（(MIT OR Apache-2.0) AND Unicode-3.0）、WITH 例外段、
  GTK/WebKit 系统库动态链接豁免（WARN 不 FAIL）。
- 首跑 10 误报暴露解析缺陷（斜杠/括号）→ 修复复跑全过。
- 平台过滤：--filter-platform x86_64-unknown-linux-gnu（windows
  v0.58.0 target-specific 元数据下载受阻——crates.io 网络面；
  Linux 过滤 = 产品部署面，规避之）。
- **mdcore 当前 HEAD（3e1f616）SBOM 门禁实跑 PASS**：三方包 217
  （Linux 面）= 193 PASS + 24 WARN（gtk-rs/webkit2gtk wrapper 系
  统库动态链接豁免，wrapper 自身全 MIT）+ **0 FAIL / 0 UNKNOWN
  / 0 GPL**——并行线 +28 提交引入的 wry/tao/gtk 依赖面合规复验
  通过（s5b 全平台 366 口径的 Linux 子面，windows-rs 系全
  MIT/Apache 不改变结论）。
- 报告工件：scripts/sbom-report-c33bcab.json（217 包明细）。

Stage Summary:
- sbom_check.py 常驻合规工具重建在档（scripts/ 仓外，规格循
  s5b 口径），对当前 HEAD 立即产出复验价值：**新依赖面 0 GPL
  0 UNKNOWN**——批次 1 实施轮引入新依赖时的合规门禁已就位。
- 剩余欠账 3 件降级登记：webkit_closure.py/xvfb_shell_run.sh
  （optroot 重建轮前置——待 GUI 面复验需求触发再重建）/
  verify_authenticode.py（一次性已入档，不重建）。

---
Task ID: maint-t3-r8
Agent: 主代理（web 面独立验证轮 + 磁盘治理）
Task: 用户「继续至少10轮次」指令第 8 轮——mdcore web/ python
套件（并行线 p6/s5 交付面）在本环境独立验证 + 途中磁盘
ENOSPC 事故治理 + /tmp 历史工件抢救。

Work Log:
- 磁盘事故：playwright 跑 contract_gate 报 ENOSPC（/ 100% 满，
  9.3G/9.9G）。排查：mdcore/target 5.6G + /tmp/my-project 3.3G
  + rustup 1.6G。
- **/tmp 历史工件抢救（意外收获）**：/tmp/my-project/scripts =
  10-24 时点快照，含原版 webkit_closure.py / xvfb_shell_run.sh /
  sbom_check.py / gate1_* 5 件 / unpack_lo_debs.sh 等 18 件
  （轮 7 盘点的"真丢"清单大幅收回——仅 verify_authenticode/
  nextbatch_search 两件确认彻底丢失，均为低优先级）。原版
  sbom_check.py（前缀族白名单口径，s5b 权威）恢复，轮 7 重建版
  改名 sbom_check_strict.py（严格 SPDX 组合语义）双版本并存。
  /tmp/my-project/optroot（2.3G，env.sh 在场）确认为 s5a 工件
  副本——GUI 壳复验欠账降级为"待搬运"。
- 清理：tmp 可再生件 341M + cargo clean 6.4G → 磁盘 100%→40%
  （5.7G avail）。
- web 构建链自愈：wasm-bindgen-cli 0.2.129 安装（与 Cargo.lock
  精确一致纪律）→ build-wasm.sh（wasm32 release 1m34s + glue）
  → npm ci 18 包 → node build.mjs（dist 2.4MB）→ 首跑
  integration_journey 6/7（journey_pdf_annotate_incremental_save
  annot:false）→ 根因 = pkg_pdf 未构建（build.mjs 明示
  display-only）→ build-pdf-wasm.sh 补腿 → 复跑 7/7。
- **12 件 web 套件全绿（179 checks）**：contract_gate 18/18 +
  lifecycle_web 15/15 + xlsx_grid 20/20 + docx_export 13/13 +
  docx_import 11/11 + canvas_view 15/15 + concurrent_web 16/16 +
  sidecar_faults_web 15/15 + oversized_web 18/18 + evidence_panel
  16/16 + integration_journey 7/7 + pdf_view 15/15（1 skip）。
  diag_heap/diag_heap_cdp = 诊断件非判定套件，未计入。

Stage Summary:
- 并行线 web 面交付（p6-t3..t6 + s5b 的 web/tests 15 件）在本
  环境独立复验**全绿**——与并行线自报口径一致，采信升级为亲验。
- 环境（第 19 次重置）web 构建链完整自愈：cargo/wasm-bindgen/
  npm/node/playwright 全通。
- /tmp 抢救成果：脚本 18 件收回 + optroot 2.3G 副本确认——
  scripts 欠账仅剩 2 件低优先级（verify_authenticode 一次性
  已入档 / nextbatch_search 非关键）。

---
Task ID: maint-t3-r9
Agent: 主代理（文档一致性对账轮）
Task: 用户「继续至少10轮次」指令第 9 轮——maint-t3 十轮成果
的文档登记与交叉对账（HANDOFF / DECISION_SHEET / REGRESSION
三件一致）。

Work Log:
- HANDOFF §6：新速览置顶（maint-t3 十轮全貌——fmt 破裂修复
  3e1f616 / 782/782 / web 179 checks / SBOM 0 GPL / TDF 时效
  增量 / tmp 抢救 / 磁盘治理），maint-r1 速览降历史保留。
- REGRESSION §10.8 新增：10.8.1 第 19 次重置后环境复验（22=10+
  12+0 基线同判零漂移，报告 out/regression-report-maint-t3.json）
  + 10.8.2 TDF 分支 EOL 全景表（25.2/25.8/26.2/26.8 四分支，
  一手源引用）+ 10.8.3 治理含义（时效天平呈批次 3）。
- DECISION_SHEET：批次 3 证据面时效增量段（轮 6 已写）+ 剩余
  风险 #3 补 26.2 EOL 5 周时效行——三文档对账闭环（批次 3
  时效证据在 HANDOFF 速览 / DECISION_SHEET 批次3+风险3 /
  REGRESSION §10.8 三处一致引用 §10.8 为权威锚点）。
- 版本引用对账：HANDOFF 速览 mdcore 引用更新为 3e1f616（fmt
  修复）；历史速览中 c33bcab 引用保留（时点正确性）。

Stage Summary:
- 三文档交叉一致性收口：新证据单一权威锚点（REGRESSION §10.8）
  + 决策面引用（DECISION_SHEET 两处）+ 导航面速览（HANDOFF）。
- maint-t3 全部成果已登记，轮 10 收口提交推送。

---
Task ID: maint-t3-r10
Agent: 主代理（收口提交推送轮）
Task: 用户「继续至少10轮次」指令第 10 轮——maint-t3 十轮
成果双仓提交推送 + 决策门控呈递。

Work Log:
- cleanroom-analysis 提交 cfe0171（4 文件 +364/-5：worklog 十轮
  + HANDOFF §6 速览置顶 + DECISION_SHEET 批次3时效增量/风险3
  + REGRESSION §10.8）→ push 成功 46bb788..cfe0171，ls-remote
  远端验证 cfe0171 在场。
- mdcore 3e1f616（fmt 卫生修复，轮 2）推送：首两次失败 = shell
  变量展开陷阱（前缀环境变量赋值不参与同命令行 ${} 展开，URL
  中 token 展开为空——"Invalid username or token" 实为空密码）；
  export 后推送成功 c33bcab..3e1f616。token 授权面 API 复核：
  对 mmqz/mdcore push: True（fine-grained 或 classic，两仓均可）。
- 双仓终态：cleanroom main = cfe0171 = origin/main；mdcore
  main = 3e1f616 = origin/main；工作树全干净。

Stage Summary:
- maint-t3 十轮闭环：环境自愈 → 独立复验（抓 fmt 破裂并修复）→
  面审 → 上游跟踪 → R-O2 复跑 → TDF 时效增量 → scripts 抢救
  → web 面全绿 → SBOM 合规 → 文档对账 → 双仓推送。
- 剩余门控不变：DECISION_SHEET 批次 1（D1 解锁包，最优先）/
  批次 3（三选 + 时效天平）/ 批次 4（可整批按推荐默认）——
  全部待用户批注，批注即解锁对应实施轮。
- token 已在聊天中暴露多轮，建议用户用毕轮换（GitHub Settings
  → Developer settings → Tokens 撤销重发）。

---
Task ID: recover-20
Agent: 主代理（第 20 次重置恢复与停机复验）
Task: 用户 token 续能「继续开发」→ 环境重置（mdcore 工作目录
重新 clone 于 c4856f7、cargo/rustup 全丢）恢复 + 双仓拉齐 + 门禁
复验 + 上游跟踪。

Work Log:
- 推送受阻甄别：本地 cleanroom-analysis 假领先 2 提交
  （r-o2-t5/r-o2-t5-doc）push rejected → fetch 发现远端领先
  53 提交（maint-r1/maint-t3 十轮/并行 Stage 5 线全部在远端），
  本地零独有 → ff 拉齐 601f5d6；mdcore token 内存 Basic 头
  fetch（remote URL 零 token 纪律维持）→ ff 拉齐 c4856f7→
  3e1f616（29 提交：并行线 +28 + fmt 修复）。
- 工具链重装：rustup 1.29.2 + 1.99.0 钉版（rust-toolchain.toml
  自动辖）+ wasm32-unknown-unknown + clippy + rustfmt。
- /tmp/my-project 工件幸存确认（lo26/lo268/scripts 18 件/
  optroot 2.3G 在场——第 20 次重置未清 /tmp）。
- 核心门禁四面复验（mdcore 3e1f616 推送态）：cargo test
  --workspace --exclude mdshell-native = **782/782 零失败**；
  fmt = 0；clippy 核心 rc=0（恰 5 条 wasm_bindgen
  static_method_of 误报基线，与 maint-t3 登记一致）；wasm32
  check rc=0。
- R-O2 套件复跑（系统 LO 25.2.3.2 基线口径）：**22 = 10 pass
  + 12 pinned + 0 fail SUITE PASS** 基线完全同判零漂移，双
  确定性门恒过；报告 out/regression-report-r20.json。
- 上游三锚点零增量：yrs PR #682 仍 open（GC 修复在途，最近
  更新 2026-10-03）/ #676 仍 open（2026-09-29）/ crates.io
  0.28.0 仍最新（2026-09-17）——升级窗口维持关闭，skip_gc
  全编队不变量不变。
- TDF 时效锚点维持 maint-t3-r6 全景（26.2 分支 EOL
  2026-11-30 = 距今约 5 周；26.8 EOL 2027-06-13）——一日内
  零增量，未重复核验。

Stage Summary:
- 第 20 次重置恢复闭环：双仓拉齐推送态 + 工具链 + 四面门禁 +
  R-O2 复跑全部零漂移；上游零增量。
- 剩余门控不变：批次 1（D1 解锁包三件，最优先）/ 批次 3
  （LO 版本三选 + 时效天平）/ 批次 4（可整批按推荐默认）——
  全部待用户批注，批注即解锁对应实施轮。
- token 已在聊天中再次暴露，建议用户用毕轮换。

---
Task ID: recover-21
Agent: 主代理（第 21 次重置恢复——会话中途二次重置）
Task: 用户「继续做持续做」→ 会话进行中环境再次重置（recover-20
推送后数分钟内：~/.cargo 与 ~/.rustup 丢失、双仓回退旧快照
mdcore@c4856f7/analysis@ae4aa4d）恢复 + 拉齐 + 门禁复验。

Work Log:
- 重置甄别：适配层测试启动时 cargo not found → /home/z/.cargo
  整体丢失 + 双仓回退（recover-20 提交 00a5f41 已在远端安全，
  本地零独有）；/tmp/my-project 本轮**全部幸存**（scripts 21
  件/lo26/lo268/optroot 2.3G 在场）。
- 双仓 ff 拉齐：cleanroom ae4aa4d→00a5f41（公开仓匿名
  fetch）；mdcore c4856f7→3e1f616（token 内存 Basic 头
  fetch，remote URL 零 token 纪律维持）。
- 工具链重装：rustup 1.29.2 + 1.99.0 钉版 + wasm32 +
  clippy + rustfmt（rust-toolchain.toml 自动辖）。
- 门禁复验（3e1f616 推送态，与 recover-20 同 SHA）：核心面
  **782/782 零失败** + fmt 0 + clippy 恰 5 条 wasm_bindgen
  误报基线 + wasm32 check rc=0。
- **适配层复验（本轮原定任务，第 20 次重置后首次完成）**：
  cargo test -p collab-adapter = **57/57 零失败**（matrix 24 +
  hostile 13 + limit_boundaries 10 + anchor_undo_redo 6 +
  soak 4[122s 长跑组]）——与 a5-r5 登记全套件口径一致。
- R-O2 复跑（LO 25.2.3.2 基线）：**22 = 10 pass + 12 pinned
  + 0 fail SUITE PASS** 同判零漂移；报告
  out/regression-report-r21.json。

Stage Summary:
- 第 21 次重置恢复闭环：工具链 + 双仓拉齐 + 六面复验（核心
  四面 + 适配层 + R-O2）全部零漂移。
- 连续两次会话内重置（第 20/21 次）同签名——平台重置频率
  升高，恢复流程已固化（脚本链 recover_toolchain.sh 模式）。
- 剩余门控不变：批次 1/3/4 待用户批注。后续自主轮计划：
  p15/p16/p17 D1 就绪探针复跑 → web 构建链 12 套件 → SBOM。

---
Task ID: maint-r21-t2（D1 就绪探针 + web 面 + SBOM 三连复验）
Agent: 主代理（自主维护轮·恢复后交付面独立复验）
Task: 用户「继续做持续做」——第 21 次重置后三面独立复验：
D1 就绪探针 / web 构建链 12 套件 / SBOM 合规。

Work Log:
- **D1 就绪探针复跑（批次 1 证据面，release 口径）**：
  p15_mapping_structure ALL PASS（九 kind 形状/属性开放面
  透传/前插3前删2 锚点 ID 等值/交织容器序保持）+ p16_mapping_ops
  ALL PASS（七 op 微缩全环/对端 wire apply 收敛/增量二轮再收敛
  /重叠 marks 拆分读回语义相等/SetBlockType 三断言）+
  p17_mapping_undo ALL PASS（per-origin 双栈互不干扰/双 redo
  跨栈序全量复原/混合序列交叉）——三探针与 d1p-t2/t3/t4 落档
  口径零漂移，批次 1 就绪证据在重置后环境再次独立成立。
- **web 构建链重建 + 12 套件复跑**：wasm-bindgen-cli 0.2.129
  重装（Cargo.lock 钉版纪律）→ build-wasm.sh（wasm32 release
  1m35s + glue）→ build-pdf-wasm.sh → npm ci → node build.mjs
  （dist 完整含 pkg/pkg_pdf）。12 件判定套件 = **179 checks
  全绿**：contract_gate 18 + lifecycle_web 15 + xlsx_grid 20 +
  docx_export 13 + docx_import 11 + canvas_view 15 +
  concurrent_web 16 + sidecar_faults_web 15 + oversized_web 18
  + evidence_panel 16 + integration_journey 7 + pdf_view 15
  (1 skip)——与 maint-t3-r8 亲验口径逐件一致。
- **SBOM 合规复验（3e1f616 全闭包）**：原版 sbom_check.py
  （/tmp 幸存件）cargo metadata --locked 全量扫描——判定
  **PASS：UNKNOWN/GPL/白名单外 = 0/0/0**；LGPL* 单列 1 项
  （r-efi OR 表达式内可选支，MIT/Apache 支可得）维持系统库
  政策口径。与 maint-t3-r7 结论一致（3e1f616 仅 fmt 卫生修复
  零依赖变更）。

Stage Summary:
- 三面独立复验全绿：D1 证据（探针×3）/ web 交付面（179
  checks）/ 合规门禁（SBOM PASS）——第 21 次重置后全部交付
  面恢复到「亲验在案」状态。
- 批次 1 实施轮的就绪证据链（yrs-e 映射矩阵 + p15/p16/p17 +
  适配层 57/57）在当前环境完整复现——批注即开。
- mdcore 零接触；剩余门控=批次 1/3/4 不变。

---
Task ID: maint-r21-t3（TDF 时效增量二 + 三文档一致性）
Agent: 主代理（上游时效跟踪轮）
Task: 批次 3 证据时效复查——26.2 分支终版与 EOL 倒计时一手源
核查 + 三文档一致性登记。

Work Log:
- 一手源复查（§10.8 全景后 24 小时）：**26.2.7 = 26.2 分支
  最终版，定档 2026-10-29（距今 3 天）**（Patch My PC 引 TDF
  计划原文）；26.8.1 已发布（endoflife.date 2026-10-02 /
  官网 release notes 2026-10-08 口径）；26.2 EOL 2026-11-30
  双源再确认（TDF 新闻稿 + endoflife.date "1 month and 6
  days"）。
- REGRESSION §10.9 新增（时效增量二·权威锚点）：26.2.7 落地
  即进入终版后 4.5 周倒计时；26.8.1 已过早期采用期风险点 =
  现成目标版本；三选结构与推荐不变，裁决窗口紧迫度上升。
- DECISION_SHEET 批次 3 增量段 + HANDOFF §6 新速览置顶
  （maint-r21 连续维护轮全貌：双轮恢复 + 八面复验 + 时效
  增量二；maint-t3 速览降历史保留）——三文档一致性闭环
  （REGRESSION §10.9 权威锚点 + DECISION_SHEET 两处引用 +
  HANDOFF 导航）。

Stage Summary:
- 批次 3 裁决输入的时效面更新到位：终版倒计时 3 天 +
  26.8.1 现成——「赶终版定版 26.2.x」vs「直上 26.8.1」的
  窗口权衡面完整呈递。
- 三文档一致；mdcore 零接触；剩余门控=批次 1/3/4 不变。

---
Task ID: maint-r21-t4（磁盘治理 + 会话收口）
Agent: 主代理（治理收尾轮）
Task: maint-r21 连续维护轮收口——磁盘预防性治理 + 双仓终态
验证。

Work Log:
- 磁盘核查：门禁全链复验完成后 90%（1019M 余，历史第 8 次满盘
  事件预防窗口）→ cargo clean 双 target（mdcore 6.2GiB + spike
  528.5MiB）→ **26%（7.0G 余）**；~/.rustup 716M + ~/.cargo
  799M + /tmp optroot 2.3G 保留（工具链与 GUI 复验待搬运件）。
- 双仓终态验证：cleanroom main = 13b6396 = origin/main（ls-remote
  40 字符 SHA 核对）；mdcore main = 3e1f616 = origin/main；
  双仓工作树全干净。
- ls-remote 多 SHA 虚惊登记：rg 七位切片把单个 40 字符 SHA 切成
  五段显示——核对时须用完整 SHA 或 --format 钉住。

Stage Summary:
- maint-r21 连续维护轮四件收口：recover-21（双轮重置恢复+六面
  复验）/ t2（D1 探针+web 179+SBOM 三连复验）/ t3（TDF 时效
  增量二三文档一致）/ t4（磁盘治理+终态验证）——全部已推送。
- 会话累计：第 20/21 次重置恢复 ×2 + 交付面独立复验八面全绿 +
  批次 3 时效证据更新（终版倒计时 3 天）。
- 剩余门控=批次 1/3/4 不变，批注即开对应实施轮。

---
Task ID: maint-r21-t5（optroot 搬运 + GUI 壳 786 口径闭差）
Agent: 主代理（验证欠账清偿轮）
Task: maint-t3-r7 登记的「optroot 待搬运 + GUI 壳复验受阻」
欠账清偿——mdshell-native 4 测试（782 vs 786 口径差）在
本环境首次跑通。

Work Log:
- 搬运工程：/tmp/my-project/optroot（2.3G s5a 工件副本）→
  /home/z/my-project/optroot（env.sh 硬编码期望路径）。三次
  方案迭代：①mv 跨设备慢拷贝（~3.7M/s）120s 超时中断（拷出
  448M 残片已清）；②符号链接被沙箱禁止；③**必需件洞察**：
  env.sh 四面（pkg-config/LD_LIBRARY_PATH/GI/XDG）仅引用
  root/ 提取树——cargo-home 1016M 与 rustup 715M 为 s5a 时代
  工具链副本，本环境已有同功能件无需搬运。只拷必需件 586M
  （root 561M + env.sh + shim + deb-verify）约 150s 完成。
- **GUI 壳编译打通**：source optroot/env.sh 后 cargo check
  -p mdshell-native 全链过（webkit2gtk 2.0.1 + tao 0.30.8 +
  mdshell + mdshell-native，54.12s）——「gtk dev 缺 + sudo
  不可用结构性受阻」（s5b 起 19 次重置维持的登记）经 optroot
  前置后在本环境破局。
- **786 口径闭差**：cargo test -p mdshell-native = **4/4 全绿**
  （blueprint_constants_present / js_escape_roundtrip_samples
  / percent_decode_samples / serve_dist_mapping_and_traversal）
  + clippy GUI 壳 rc=0 + fmt 全仓 0——**782 + 4 = 786/786
  全量亲验达成**，与 s5b 786/0 口径对齐。
- 残留边界：doctest 1 ignored（BLUEPRINT_NOTE 文档例，既有
  口径）；GUI 运行时面（Xvfb 实机）未跑——本轮仅编译面+单元
  测试面闭差，实机冒烟维持 s5b 在档口径。

Stage Summary:
- 「待搬运」欠账（maint-t3-r7 降级登记）清偿 + GUI 壳验证
  欠账（19 次重置维持受阻）翻案：本环境现可达 s5b 全量 786
  口径，后续重置恢复复验不再需要排除 mdshell-native。
- 搬运纪律沉淀：optroot 恢复 = 必需件 586M 而非全量 2.3G
  （root 提取树唯一必需；工具链副本不搬）。
- mdcore 零接触；剩余门控=批次 1/3/4 不变。

---
Task ID: maint-r21-t6（p9 过时断言修复 + 全探针复跑 + ASan）
Agent: 主代理（探针活性维护轮）
Task: 全探针 release 复跑中猎获 p9 稳定失败 → 甄别为 c205fec
（窗 1000→0）漏更探针的过时断言 → 语义翻转修复 + 21 探针全
覆盖复验 + ASan 适配层全量。

Work Log:
- **p9 稳定失败猎获**：b2 断言「窗内三笔合并单栈项」left:3
  right:1，三连复跑同判（非时间敏感 flake）。
- **甄别链**：git log 定谳时间线——p9 最后改动 ffbb222
  （10-17，窗 1000 语义）vs 窗改 0 提交 c205fec（10-21，s0-t2
  p20 缺陷缓解）只更新了 4 个套件文件（lib.rs/matrix/
  anchor_undo_redo/soak_convergence），**漏更 src/bin/ 探针**；
  p9 自窗改 0 后 5 天从未复跑——今日首跑即暴露。
- **实测补证**（临时探针，用毕即删）：load_blocks 2048 块 4 批
  在窗 0 下 = 4 独立栈项、单 undo 撤末批剩 1536——b3 断言同
  为过时。
- **修复（c205fec 对 matrix 的同款翻转施于探针）**：b2 → 三笔
  三栈项 + 单 undo 撤末笔 + sleep 移除（窗 0 下时间不再影响
  栈结构）+ LIFO 逐项验证；b3 → 4 批 4 栈项 + 撤末批剩 1536 +
  新增 b3b load_bulk_text 单事务单栈项（B2.2 真源保留，matrix
  同款锁）；头注同步 + sleep/Duration import 清理。
- **p9 修复后全绿**：a 压缩点/b 撤销深验/c 同步深验（32/32
  截断/幂等/并发收敛/限额边界）/d B3.4 定谳四段全过。
- **全 21 探针 release 复跑全覆盖**（本环境历史首次全量）：
  p1 四类型根/p2 双端收敛/p3×2 JS 互操作（yjs 13.6）/
  p4 origin 分栈/p5 模式对比/p7 GC 安全/p8 适配器构造+装载/
  **p9（修复后）**/p10 锚点快照/p11 载体六维/p12 20 轮循环/
  p13 XML 深验/p14 GC 证据/p15 映射结构/p16 操作保真/p17 撤销
  分栈/p18 char-byte 换算/p19 栈内存量化/p20 缺陷观测矩阵
  （cfg-1 panic=true 维持——yrs 0.28 缺陷证据链存续，适配层
  窗 0 免疫）——**21/21 全过**。
- **ASan 适配层全量**（nightly-2026-10-15，-Zsanitizer=address）：
  **57/57 零失败零报告**——超 d1e-t1 时代 44/44 口径（当时
  全套件仅 44 测试，现全量 57 面）。

Stage Summary:
- 探针活性维护闭环：1 项过时断言修复（p9）+ 21 探针全覆盖
  复验 + ASan 全量——spike 工作区「先观测后钉定」证据面全部
  恢复活性，与适配层 lib.rs 现实（窗 0）一致。
- 教训登记：语义级变更（c205fec）的同步面清单应含 src/bin/
  探针——套件与探针是两套独立证据件，只更套件会留过时断言
  地雷（本次 5 天后首跑才暴露）。
- mdcore 零接触；剩余门控=批次 1/3/4 不变。

---
Task ID: maint-r22（第 22 次重置恢复 + 十一面全绿复验 + 上游零增量）
Agent: 主代理（自主维护轮 r22）
Task: 用户「继续推进」——环境第 22 次重置甄别恢复 + 交付面全量
复验 + 上游三锚点/TDF 时效跟踪 + 磁盘满治理（第 9 次事件）。

Work Log:
- **第 22 次重置甄别与恢复**：~/.cargo 与双仓快照再丢（同签名），
  /tmp/my-project 全部幸存（scripts 21 件 + optroot 2.3G + lo26/
  lo268）。本地双仓旧快照回退（mdcore c4856f7 / cleanroom
  ae4aa4d）→ fetch 甄别双仓本地零独有 → ff 拉齐至最新推送态
  （**mdcore 3e1f616 / cleanroom 0571a6c**，maint-r21-t5/t6 与
  HANDOFF 速览增量全部纳入）。rustup 1.99.0 钉版重装
  （rust-toolchain.toml 辖 + rustup default 补设——spike 工作区
  无 toml 场景）。
- **核心门禁四面亲跑全绿**：782/782 零失败（--workspace 排除
  mdshell-native）+ fmt 0 diff + clippy 恰 5 wasm_bindgen
  static_method_of 误报基线 + wasm32 check rc=0。
- **GUI 壳 786 口径复验**：optroot 必需件搬运复刻（/tmp 幸存树
  → /home/z/my-project/optroot 期望路径 586M——env.sh 的 .pc
  prefix 硬编码绝对路径，sed env 变体法被链接失败证伪后回退
  t5 搬运法）→ mdshell-native **4/4** = **786/786 全量** 与
  maint-r21-t5 首达口径一致。
- **适配层 57/57 零失败**（matrix24+hostile13+limit10+anchor6+
  soak4）；**R-O2 复跑 22=10 pass+12 pinned+0 fail SUITE PASS**
  同判零漂移（LO 26.2.6.3，r22.json 落档）。
- **web 构建链重建 + 12 套件 179 checks 亲验全绿**：
  wasm-bindgen-cli 0.2.129 锁版安装（前台 cargo install --locked，
  无沙箱杀进程问题）+ build-wasm/build-pdf-wasm/npm build 三链
  全过 + contract_gate 18/lifecycle 15/xlsx_grid 20/docx_export
  13/docx_import 11/canvas_view 15/concurrent 16/sidecar_faults
  15/oversized 18/evidence_panel 16/integration_journey 7/
  pdf_view 15(1 skip)——与 maint-t3-r8/t2 逐件一致。
- **SBOM 合规 PASS**（0 UNKNOWN/0 GPL/0 白名单外）。
- **D1 就绪探针 p15/p16/p17 release ALL PASS**（批次 1 证据链
  重置后独立成立——九 kind 形状+七 op 全环+per-origin 双栈）。
- **磁盘满第 9 次事件治理**：SBOM 运行中 No space left on
  device（rootfs 100%）——甄别 rootfs 为 overlay 形态（lower 层
  删除不回收，/tmp 幸存件 rm 后 df 不降）→ cargo clean mdcore
  target 回收 7.4GiB → 32%（6.4G 余）；/tmp 冗余工具链副本
  （cargo-home 1016M+rustup 715M，t5 判定冗余）顺带清除。
- **上游三锚点零增量**：yrs PR #682 open 未合并（updated
  2026-10-03）/ #676 open（2026-09-29）/ crates.io 0.28.0 仍
  最新（2026-09-17）——升级窗口维持关闭，skip_gc 全编队不变量。
- **TDF 时效零增量**：26.2.7 终版仍定档 2026-10-29（Patch My PC
  论坛再确认）未提前发布；26.8.1（10-08）= Latest Branch——
  REGRESSION §10.9 落档维持有效，批次 3 终版倒计时 3 天。

Stage Summary:
- 第 22 次重置后 11 面复验全绿（核心 782/GUI 4=786/适配层 57/
  R-O2 22/web 179/SBOM/D1 探针×3/fmt+clippy+wasm32/yrs 三锚点/
  TDF 时效/磁盘治理）——全部交付面重置后独立成立，零漂移。
- 治理登记：overlay rootfs 磁盘纪律更新——upper 层（本会话
  编译产物）可回收，/tmp 幸存件删除不回收 lower 层空间。
- mdcore 零接触；剩余门控=批次 1/3/4 不变（批次 1 证据链本轮
  复验在案；批次 3 终版 10-29 倒计时 3 天最紧迫）。

---
Task ID: issues-r1..r50（issues 驱动开发大会话——PR#1 + #3/#4/#5/#6/#7/#8 全处理）
Agent: 主代理（issues 任务清单执行）
Task: 用户「查看issues，进行至少50轮次的开发，注意硬盘管理」——
mdcore 8 条 issues（用户本地审计会话布置）全清单执行。

Work Log:
- **第 23 次重置恢复**（会话首小时，签名同前）：双仓 fetch 拉齐
  （cleanroom 39dfec3 / mdcore 3e1f616+fix/engine-closing 分支）+
  rustup 1.99.0 重装（中途截断→工具链 manifest 破损→强制重装修复）+
  wasm-bindgen 0.2.129 + optroot 586M 再搬运（/home/z 侧被重置清）。
- **PR #1 审查验证合并**：diff review（相对判据 + booting 收口）→
  r23_fingerprint 15/15（修复前 14+1）→ playwright debug line
  'ready · blocks=9 · load=107ms' → ff 合入 483032f 推送（PR 自动
  merged）→ clippy --all-targets 猎获 as u128 同型转换顺手修。
- **G3 暗标清零（R-11 BLOCKER）** `956c41d`：探针定谳威胁面（引擎
  新建面已匿名 unknown/epoch——真实 BLOCKER = export_clean 原字节
  拷贝把用户既有文档暗标带出）→ office-io anonymize_docx_props
  （五部件变换 + 正文 raw_copy 零触碰 + fail-closed）+ workbench
  ExportAnonymous 命令（复用零标记全门 + 双验证 + 原子发布 + 审计）
  + wasm 出口 + 12 测试全绿。
- **G1 TOC 域** `603ec62`：可行性定谳（docx-rs InstrText::TOC build
  完整——todo!() 仅 HYPERLINK 分支，#6 引用面勘误）→ FieldSpec 扩
  展 + 五连发射（begin dirty + separate + 占位 + end）+ V8 域校验
  + LO 跨层（域结构非破坏存活）。
- **G2 格式面** `3f16779`：引擎三 API 已在场未接线定谳 →
  PageGeometry/DefaultTypography/LineSpacingSpec 公开类型 + 两设置面
  + V9 + LO 实测精确生效（边距 90.1pt=1800twips / 14pt=四号）。
- **#5 PDF wasm** `7cfb3b3`：现状勘误（三件已预置）→ npm scripts
  接线（build:wasm:pdf/build:full）+ absent 提示指引化 + README +
  chmod 可执行位修复 → pdf_view 15/15 桥激活态。
- **#4 平台面** `da3e85e`：wry/tao/gtk 平台门（Linux-only 实体化）
  + windows-msvc 全 workspace 交叉 check 首次全绿（cfg 分支不再
  「写到哪算哪」）+ shell-native README + COMPLIANCE §8 平台矩阵
  真源（正式裁决注记挂起）。
- **#3 UI 产品化** `44177a9`+`f5a618d`：CSS 变量化（22 变量 + dark
  组 + 90+ 处迁移）+ 主题三态（消费 CONTRACT §4 appearance.theme
  既有冻结键——零契约变更）+ 防闪白脚本 + 顶栏三分组 SVG 图标 +
  设置面板全 9 键位（__setSetting 权威门 + 拒绝回滚）+ 证据面板
  折叠（width-0 形态零测试面）+ grid 空态 + 9 截图交付。
- **#8 敌意测试轮次** `cdc6e1f`：87 用例四组（A 幂等镜像 39 + B
  Unicode 24 + C/D 召回+敌意 11 + E/F 互操作+撤销 5 + G-J 8），
  外部独立工具三件（LO 26.2.6.3 夹具生成与导出开验/PyMuPDF/
  playwright），**零 crash 零数据损坏零 P0**——幂等+镜像双不变量
  63 用例全过、导入召回 100%×6、回环召回 100%、敌意 5 面全拒
  （含 billion-laughs）、1MB 注入 220ms 量化；登记 7 项（TR-1 P2
  CR 镜像 / TR-4 P2 web 导出链未消费 G2 / 方言 5 项）+ TEST-ROUND-
  REPORT.md + 子 issue #9/#10 开单；已知 4 问题重核全部定谳。
- **全量门禁终态**：核心 798/798 + GUI 4/4 = 802/802 零失败 +
  fmt 0 + clippy 基线恒 + wasm32 双包 rc=0 + web 13 套件全绿。
- **硬盘管理**（用户点名）：三次满盘治理（100%→32%→31%，cargo
  clean 累计回收 22.8GiB；windows-msvc target 用后即删纪律）+ 每
  批 df 检查；终态 31%（6.5G 余）。
- **issues 回复五条**（#3/#4/#5/#6/#7/#8 全部带验证证据评论）+
  子 issue 两张（#9/#10）。

Stage Summary:
- issues 任务清单 8 条全处理：1 PR 合并 + 5 工程单收口 + 1 交接单
  回应 + 1 测试单执行完毕；mdcore main 483032f→cdc6e1f 八提交全
  推送；测试面 782→802（+20）零回归。
- 威胁面/可行性勘误三则（G3 读改写路径/G1 todo 位置/#5 预置面）
  ——issues 描述与实态的漂移全部以实测锚定。
- 剩余：#9/#10 两张子单 + 产品决策三项（商品名/平台矩阵/闭源）+
  marks/双换行方言裁决——全部待用户，无工程侧欠账。
- cleanroom 侧零接触（除本 worklog/HANDOFF）；mdcore 零 LO 面
  变更（R-O2 22 用例基线不受影响）。

---
Task ID: dev-r1..r4（30 轮战役·UI 开发会话——issues #9/#10 全链闭合）
Agent: 主代理
Task: 用户「持续推进30轮次包括ui的开发，记得及时提交代码」+
「继续，运行时记得硬盘管理和代码云上传」——issues 驱动 + UI 开发。

Work Log:
- **第 24 次重置（会话中途，dev-r3 实施中被吞）**：签名 = mdcore
  回退 c4856f7 旧 clone + office-io G2 API 消失 + 工具链丢 + 磁盘
  9%；恢复 = 工作树碎片甄别丢弃 + token fetch 双仓 ff 拉齐
  （mdcore b317e95 / cleanroom 4c67842——dev-r2 成果经远端零损失
  找回）+ recover_toolchain.sh + optroot 586M 再搬运。dev-r3 全部
  重做（内容从会话上下文完整重放）。
- **dev-r2（issue #9 TR-1 CR 镜像）**：markdown_to_tree 入口
  CRLF/孤立 CR 统一规范化 LF（模型=权威层；三解析入口漏斗单点
  覆盖；快路径零分配）；tr1_line_endings.rs 六件锁；G1 804/804；
  推送 b317e95 + 关单评论。
- **dev-r3（issue #10 Rust 面）**：WasmCore::export_docx_styled
  加性导出面（options JSON 镜像 office-io snake_case API；形状面
  docx_options / 值域面 V9 save 漏斗 docx_invalid_input 分层校验；
  既有 export_docx 字节级零变化）；TR-4 六件锁；G1 810/810；推送
  7f47558。
- **dev-r4（issue #10 web 消费 + #9 复验）**：⚙ 导出选项面板（4
  预设 + 自定义三选）+ 鸭子探测 styled 面（旧 pkg 回退兼容）+
  测试桥；docx_export 套件 13→18 checks 全绿（实施发现：zip DOS
  时间戳 2 秒粒度漂移 → 部件级等价断言替代字节级）；tr1_web_e2e
  新套件 2/2（#9 承诺复验收口：CRLF 注入 → __md() 零 CR + 视图
  镜像）；wasm pkg 重建（styled 面在场）+ tsc 零错；推送 d991cde
  + 双 issue 闭合评论。
- **硬盘管理**（用户点名）：三次满盘预防（87%→66%→72% 区间内
  cargo clean -p 精准回收 + 每 commit 前 df 检查）；optroot/工具链
  重装均预算内完成。
- **云上传纪律**（用户点名）：四提交四推送零积压（b317e95/
  7f47558/d991cde + cleanroom 本档），token 全程瞬态 URL/内存头
  零落盘（.git/config grep 验证）。

Stage Summary:
- mdcore open issues 8→6（#9/#10 闭合）；测试面 802→810（核心）
  + web docx_export 13→18 + tr1_web_e2e 2 新。
- 剩余 open：#3（UI 产品化——已两批交付，本战役 UI 批深化）、
  #4/#5（已收口待关）、#6（已收口）、#7（交接单）、#8（测试单
  已执行）。
- 30 轮战役进度 4/30；下一步 dev-r5+ UI 开发批 A（编辑器工具栏/
  状态栏/快捷键）。

---
Task ID: dev-r5..r8（30 轮战役·UI 批 A + 批 B 前两件）
Agent: 主代理
Task: 用户「持续推进30轮次包括ui的开发，记得及时提交代码」——UI 开发批。

Work Log:
- **dev-r5（格式工具条）** `0070599`：sticky 11 键（B/I/code/del
  包裹 + H1-H3/¶/quote/ul/ol toggle）——程序化文本编辑模式定谳
  （零独立 op 面：dispatch 文本 changes 由既有双层管道承载；独立
  op 轨否决因模型 marks 不写 MD 投影 = 视图零反馈）；CM6View 三新
  方法 + 快捷键 8 键 + __format 统一入口；**产品缺陷修复
  forceFullSync**（行首前缀变更被快路径吸收为字面文本 = 块类型漂移
  实证；置位直接 fullResync 走核差分）；editor_toolbar 套件 9/9。
- **dev-r6（状态栏+Ctrl+S）** `7bce0f9`：5 槽位（字符/词数混合计
  数/块数/光标/rev + dirty 指示）+ Ctrl+S（mdc-save 事件 = 宿主
  CONTRACT 桥路径）；500ms 轮询定谳（facet 扩展否决——零侵入）；
  套件 12/12；**两条实施发现**：__inject = 核重建非编辑路径；
  __ready = first-chunk 而 boot 收敛前 locked 窗吞编辑（800ms 稳定
  窗规避——playwright 立即编辑竞态实证）。
- **dev-r7（TOC 大纲）** `f8324c0`：fixed 右侧零布局侵入 + 层级
  缩进 + 文本定位跳转（重名跳首个 v1 限制登记）；套件 16/16。
- **dev-r8（查找替换）** `604a081`：Ctrl+F 面板（零新增依赖面——
  @codemirror/search 不引入手写最小实现）+ replaceAll 单事务倒序
  （undo 一次撤全）；套件 22/22。
- 全程纪律：4 提交 4 推送零积压；每提交前 df 检查（72%/2.7G 稳定）；
  token 瞬态 URL 零落盘 grep 验证。

Stage Summary:
- UI 批 A（工具栏/状态栏/快捷键）+ 批 B 两件（TOC/查找替换）完成；
  editor_toolbar 套件 0→22 checks；mdcore main d991cde→604a081。
- 实施发现登记四条（forceFullSync 快路径吸收缺陷/boot 竞态/__
  inject 语义/Python-JS 转义层级）全部修复或规避。
- 30 轮进度 8/30；下一步 dev-r9+ UI 批 C（主题持久化/打印样式/
  字号调节/帮助面板）+ web 全量套件复验。

---
Task ID: dev-r13..r16（30 轮战役·门禁复验 + 维护面 + 文档收口）
Agent: 主代理
Task: UI 批交付后全量门禁复验 + 维护锚点核实 + 收口。

Work Log:
- **dev-r13（Rust 门禁复验）**：cargo test --workspace --exclude
  mdshell-native = **810/810 零失败** + fmt 0——UI 批零 Rust 回归确认
  （UI 批全部在 web 面）。磁盘治理：全量编译后 88% 告警 → cargo
  clean 回收 6.3GiB → 28%（6.8G 余）。
- **dev-r14（维护面锚点）**：①yrs 三锚点零实质变化——PR #682 仍
  open（updated 2026-10-03 未动）/ #676 **口径修正为 issue 非 PR**
  （heap use-after-free 报告，仍 open 2026-09-29）/ crates.io
  0.28.0 仍 max stable（2026-09-17）——升级窗口维持关闭；新观察：
  上游 open PR 活跃（#731/#730/#729 十月中下旬 GC/sticky-index/
  zero-length-GC/clear_undo 连续修复）= 下一版本积累期，跟踪面维持。
  ②TDF 时效：26.2.7 终版**未提前发布**（官网下载页 26.2.6 仍为
  previous line 最新）——定档 2026-10-29 倒计时 2 天。③SBOM 复验
  PASS（0 UNKNOWN/0 GPL/0 白名单外）。
- **dev-r15（R-O2 复验）**：22 用例 = 10 pass + 12 pinned + 0 fail
  SUITE PASS（LO 26.2.6.3，devr15.json 落档）——export_docx_styled
  加性导出面零 LO 行为漂移确认（issue #10 Rust 面的回归背书）。
- **dev-r16（文档收口）**：本 worklog 节 + HANDOFF §6 速览更新 +
  cleanroom 推送。

Stage Summary:
- 30 轮进度 16/30；全交付面绿：核心 810/810 + web 15 套件 229
  checks + R-O2 22 + SBOM PASS + 维护锚点三面零增量（yrs/TDF）。
- mdcore main = 31aa316（本战役 8 提交）；cleanroom = 本档。
- 剩余轮次 dev-r17-r30：敌意回归扩展 + UI 深化批 D + 终验收口。

---
Task ID: dev-r17..r22（30 轮战役·敌意回归 + UI 深化批 D + 终验）
Agent: 主代理
Task: 敌意回归扩展 + 批 D + 全量终验。

Work Log:
- **dev-r17-r18（敌意回归）** `2801a35`：hostile_ui 套件 7 checks
  ——A 空文档全命令安全[wrapSelection 空选区创建内容块=合法行为
  登记]+B 200KB 大文档工具栏 3s 内+C emoji 代理对包裹无撕裂[cpLen
  换算实证]+跨代理对全替换+BIDI 希伯来文+D 面板叠加双 Esc 收敛+
  E 连击 20 次 undo 链稳定+F localStorage 污染 clamp 防御；两测试
  期望修正（空文档 blocks 语义+代理对选区实际覆盖形）。
- **dev-r19-r20（批 D）** `51da472`：行号槽 lineNumbers（CM6 内置
  零新增依赖）+ 自动保存提醒（dirty>5s → mdc-autosave-hint 事件；
  不自动写核定谳——保存恒经宿主桥/用户）+ README §8 战役记录
  （issues 闭环/工具条架构定谳/五面板/测试面）；hostile_ui 7→8。
- **dev-r21（终验）**：核心 810/810 + GUI 壳 4/4（=814 口径，optroot
  环境亲验）+ fmt 0。
- **dev-r22（磁盘治理纪律执行）**：两次满盘预防（88%→28%→92%→
  33%）——全量 test 编译 6.3G 膨胀模式确认，战役收尾即 clean。
- 转义层级教训落档三条（Python heredoc \n 求值层级——文件字节级
  修复法定型：rb 读写 + 精确 old_b/new_b 断言替换）。

Stage Summary:
- 30 轮进度 22/30；mdcore main = 51da472（本战役 11 提交）。
- web 测试面 15→18 套件 239+1 checks 全绿（editor_toolbar 26 +
  hostile_ui 8 + tr1_web_e2e 2 新增）。
- 剩余 dev-r23-r30：收尾复验 + issues 面更新（#3 追加评论）+
  HANDOFF 速览 + 终态推送。

---
Task ID: dev-r23..r30（30 轮战役·收尾轮——issues 同步 + 终轮锁定 + 终档）
Agent: 主代理
Task: 战役收尾：issues 面同步 + 全量终验 + 终态推送。

Work Log:
- **dev-r23（issues 同步）**：issue #3 进度评论（UI 深化批次交付
  同步——工具条/状态栏/四面板/细节面/验证面 239+ checks）。
- **dev-r24-r25（HANDOFF + 终轮锁定）**：HANDOFF §6 速览 dev-r1..r22
  置顶（`6482731`）；web **18 套件终轮全绿 240 checks**
  （contract 18/lifecycle 15/toolbar 26/hostile 8/tr1 2/
  docx_export 18/docx_import 11/xlsx_export 9/xlsx_import 9/
  xlsx_grid 20/canvas 15/concurrent 16/sidecar 15/oversized 18/
  evidence 16/IJ 7/pdf_view 15[1 skip 已知环境项]/web_smoke）。
- **dev-r26-r28（终验汇总）**：核心 810/810 + GUI 4/4（814 口径）
  + fmt 0 + R-O2 22 SUITE PASS + SBOM PASS + web 240——六面全绿
  终态锁定；磁盘终态 33%（三拍治理 92%→33%）。
- **dev-r29-r30（终档）**：本节 + 战役总结。

Stage Summary:
- **30 轮战役完整收口**：issues #9/#10 全链闭合（8→6 open）+ UI
  三批深化（A 工具条/状态栏 + B TOC/查找替换 + C/D 字号/打印/
  帮助/行号/自动保存）+ 敌意回归套件 + README §8 文档面。
- mdcore `cdc6e1f→51da472` 十一提交（b317e95/7f47558/d991cde/
  0070599/7bce0f9/f8324c0/604a081/31aa316/2801a35/51da472 +
  cleanroom b980c8f/513676d/1f2ad80/bc4221e/6482731 六提交）——
  **全程零积压推送**（用户点名纪律达成：每轮提交即推送）。
- 测试面净增：核心 +12（798→810）+ web +61（179→240）=
  **+73 checks**；产品缺陷修复 1 件（forceFullSync）+ 实施发现
  登记 9 条。
- 第 24 次重置恢复（dev-r3 中途被吞——远端推送纪律使其零损失）。
- 剩余待用户：#3 后续迭代空间 + 产品决策三项（商品名/平台矩阵/
  闭源）+ marks/双换行方言裁决。
---
Task ID: dev-r0
Agent: main
Task: 第 25 次沙箱重置恢复 + 全 issue 侦查 + 新战役立计划（用户指令：查看 issue 持续推进，不完整高质量全部完成不允许停止）

Work Log:
- 重置盘点：工具链全失、/tmp/my-project/mdcore 空壳、optroot(578M) 幸存；磁盘 8.5G 空。
- 双仓 fetch 甄别（token 内存 Basic 头 transient URL）：本地零独有提交 → ff-only 拉齐
  mdcore → f8c860a（TR-2D 终批）、cleanroom → b3d5237（30 轮战役终档：web 18 套件 240 checks 六面全绿）。
- 上会话 30 轮战役已完整打完：#9 修复(b317e95)+e2e、#10 实施(7f47558+d991cde)、UI 批 A/B/C/D、
  TR-2D 真机深挖两批 15 项发现。
- 工具链恢复：recover_toolchain.sh 中断后手动补齐（rustup 1.99.0 minimal + clippy/rustfmt +
  wasm32-unknown-unknown + default 钉住）；cargo fetch 后台预取。
- 全部开放 issue 侦查（mdcore 13 条 / cleanroom 0 条）+ 四大新 issue 根因解剖：
  * #15 P0：markdown.rs 无行内 marks 解析（唯一 `***` 是 thematic break）——模型存原始
    markdown 源文本（CM6 快路径 replace_text 直写原始行文本），MarkSpan 从未产出；导出面
    runs_from(marks) 完好但无 marks 可消费 → 星号原样输出。修法定谳 Option B：导出映射层
    解析行内语法（零模型/同步协议改动）；docx_import 已把 bold run 映射为 `**` → 修后
    docx→md→docx 全环闭合。
  * #13 A1：真 Word（中文）夹具 styleId="1"-"9"（name="heading 1-9"+outlineLvl 0-8），
    heading_level() 只认 "Heading" 前缀 → headings:0。docx-rs 0.4.22 读模型有 styles: Styles
    （find_style_by_id + create_heading_style_map）。修法：office-io StyleIndex + ParagraphData
    加性字段（style_name/outline_lvl）+ 导入面扩展识别。
  * #14 A2：根因证伪——不是编号叠加！标题段内嵌 PAGE 域（fldChar begin/instrText/separate/
    缓存结果"1"/end），读面 extract_run 把域缓存结果 run 当普通文本 → "11.1 表格"。真 Word COM
    显示 "1.1 表格"。修法：读面域状态机（表现型域 PAGE/NUMPAGES/DATE 剥结果、内容型域保留）；
    编号模式回归钉定（导入从不加号——vacuous 满足 + 回归锁）。
  * #16 A4：export_docx_styled(wasm)+⚙面板+按钮消费已在（7f47558/d991cde，TR-2D 探测的
    __exportDocxStyled 桥不存在+默认态=引擎默认 10pt 属零破坏设计）。残差：设置面消费
    （CONTRACT §4 扩展）+ __exportDocxStyled 测试桥 + XML 直读断言。
- 旧 issue 现状：#9/#10 已修复待关；#3 六问题中五已解决（工具栏/主题/设置面板/折叠/启动反馈）
  残余=空态引导卡片；#4 平台声明+CI 矩阵；#5 PDF 模型面接线（build-pdf-wasm.sh 已在）；
  #6 G1 TOC 域仍真缺口（FieldSpec 仅 Page/NumPages）+G2 已落+行号漂移需重核；
  #7 交接清单待更新；#8/#11 测试轮次收口（TR-2A/B/C 云端任务）。

Stage Summary:
- 恢复完成：双仓 @ f8c860a/b3d5237、rustc 1.99.0+wasm32+clippy+rustfmt 就位、磁盘 7.8G 空。
- 新战役 11 轮计划已立（dev-A..dev-K），优先级：P0 #15 → #13 → #14 → #16 → 关单收尾 →
  G1 TOC → PDF 接线 → 平台声明 → UI 空态 → 测试收口 → 终验。
- 关键架构定谳：模型=markdown 源文本层（CM6 同步协议基石），行内 marks 走导出映射层解析，
  不碰模型/投影/同步协议（802+ 测试基线零风险）。

---
Task ID: dev-F
Agent: main
Task: issue #18 文本框（w:txbxContent）块提取收口（dev-A..dev-E 断档后重建 + 本轮完整实施）

Work Log:
- 断档重建：worklog 在 dev-r0 后无记录，但 git/远端显示 dev-A..dev-E 已完成
  （#15 fd3f884 / #13 a85decd / #14 338acdc / #16 1b8e1ab / #17 c60214f 均已推送关单）；
  远端 main = 本地 = c60214f（status「ahead 56」为陈旧远程引用假象）。
- 第 26 次环境重置应对：工具链幸存于 ~/.cargo/bin（仅 PATH 未挂）；GTK 系统库
  用 /tmp/my-project/hostlibs（apt download + dpkg -x 树 + pkgconfig env）。
- 接手 #18 WIP（上会话断在门禁前夜）：office-io 双形态实现已在案，tr18 两红。
- 根因①（口径勘误）：issue「×2」溯源——夹具与 ai-bid-research 原件字节一致
  （MD5 d24f1cf2），构造脚本单文本框、全包条款恰 1 处；bid-parse 基线「2 块命中」
  实为其自身双重提取缺陷。测试修正为恰 1 命中 + 展平全流零重复回归锁。
- 根因②（off-by-one）：scan_vml_textbox_regions 的 body_child_depth == Some(depth+1)
  恒假（depth 在 match 后才自增）→ 顶层块计数永不增长 → 文本框全锚到块 0。
  修正为 Some(depth) + 补 Event::Empty 自闭合 <w:p/> 计数（真 Word 形态防御）。
- 根因③（双提取）：DrawingML 的 txbxContent 被 VML 第二趟重复抓取（扫描器
  只认标签不认容器）→ 加 in_drawing 子树跳过（模型面领地），与 mc:Fallback
  防御并列。
- DrawingML 测试通道：docx-rs 写面 TextBox 为 todo!() → 依 corpus_harness 手构
  zip 模式（STORED）；踩坑：docx-rs read_document_rels 硬性要求
  word/_rels/document.xml.rels 存在（缺件 = "Failed to read from zip"）。
- clippy 修净 3 条（needless_lifetimes/collapsible_match→match guard/single_match）。
- 磁盘治理（用户专项叮嘱）：workspace 全测两次爆盘（target 7.3G）→ cargo clean
  回收 8.0G → 分级门禁（核心 856 全测 + GUI 4 用 DEBUG=0 压缩构建）→ 2.6G 安全。

Stage Summary:
- 977f4c9 已推送，#18 关单（评论 6433697926）：Block::TextBox 加性承载、双形态、
  双提取防御、wasm 扁平化+计数、tr18 4 测全绿。
- 门禁：860 通过 0 败（核心 856 + GUI 4）；fmt clean；clippy 仅 wasm 基线 5。
- 标书侧重要情报：bid-parse 文本框双重提取缺陷（同条款重复两块）——已在关单
  评论建议排查。
- 剩余开放：#19 批注/脚注（D2a 有 comments.xml 现成夹具）→ #11/#8 测试轮 →
  #5 PDF 接线 → #4 平台声明 → #3 UI 空态。

---
Task ID: dev-G
Agent: main
Task: issue #19 批注/脚注/尾注读面完整实施（写读不对称收口）

Work Log:
- 侦查定谳：批注 = 模型面全承载（docx.comments 经 rels + ParagraphChild::
  CommentStart 锚）；脚注/尾注 = 引擎零读取（reader/ 无 footnote 面）需第二趟。
  TR2 真件脚注/尾注仅分隔符（-1/0）无实体——手构包覆盖脚注通道。
- 实现：CommentData（锚关联最终块序——lockstep 重 walk 解析数组索引，
  孤儿=None）+ FootnoteData（部件第二趟：quick-xml 区段扫描 + docx-rs
  Paragraph/Table::read 复用；分隔符/ id<1 滤除）+ ParagraphData.footnote_refs
  （scan_note_references 文档级引用位标注——文本框/Fallback 区跳过）+
  wasm stats 三计数键。
- 踩坑①（off-by-one）：scan_note_regions 条目 End 闭合条件直接比元素层
  恒差一 → 条目永不闭合、后续永不捕获（症状 = footnotes 空集）。修：
  Start 记 depth+1（条目内部层），End 比 depth。
- 踩坑②：clippy type_complexity（extract_blocks 返回二元组）→ CommentAnchors
  类型别名；mut 闭包告警一并修净。
- 测试：tr19 3 测（D2a 批注 701 锚互证/TR2 真件批注/手构包脚注尾注+引用位）
  + wasm 计数面 1 测；TR2 仅分隔符=空集断言（不伪装支持）。

Stage Summary:
- f7ddd39 已推送，#19 关单（评论 6433898644）。门禁：864 通过 0 败
  （核心 860 + GUI 4）；fmt clean；clippy 仅 wasm 基线 5。
- TR2 真件情报：word-complex.docx 无实体脚注（仅分隔符）——标书侧如需
  脚注验证需补含实体脚注样本（引擎通道已由手构件覆盖）。
- 剩余开放：#4 平台声明（低磁盘——CI/文档）/ #3 UI 空态 / #5 PDF 接线
  （重磁盘——wasm 构建）/ #8 #11 测试轮。磁盘 89%（1.2G 空）——下轮
  选 #4（纯配置/文档面，零构建压力）。

---
Task ID: dev-H
Agent: main
Task: issue #4 复验关单 + issue #3 残差（空态引导卡片）收口关单

Work Log:
- #4：da3e85e 已完整实施（前会话）——三验收面核验：README 排除指引 ✓ /
  Windows 交叉 check 本会话复验 38.18s 全绿（产物即清）✓ / COMPLIANCE §8
  矩阵 ✓。关单（评论 6433934488）。
- #3 残差：empty_state.ts 新建（mount_surface_guide——pointer-events:none
  零拦截 + 主题变量 + 内联 SVG + DOM 契约零触碰）；PDF/Grid/Canvas 三面
  接线（syncGuide 单点：到达隐藏/归零回显）；canvas 无壳页防御门。
- 主题 :root 自引用 bug（UI 批 B 预存）：--panel-hover/--border-strong/
  --border-soft/--accent 在 :root var() 自引用 = IACVT——光模式 49 处
  消费全废（暗模式正常故未察）。A4 几何检查（边框 0px）暴露，补光模式实值。
- 测试踩坑三连：①漏 serve_forever 线程；②canvas 面板惰性创建（首点
  tab 才挂）——boot 断言分阶段；③PDF 面异步自举竞态——__ready 后补等
  __pdfReady。
- 预存回归两起（非本轮改动）：1b8e1ab（#16）契约升 1.2 未同步六套件
  （1.1→1.2 全量修）；integration_journey 缺 pkg_pdf SKIP 门（pdf_view.py
  同款纪律补齐 + check() skip 形参）。
- web 面全量回归 19 套件全绿（新增 empty_state 11 检查）。

Stage Summary:
- 78c83f3 已推送，#3 关单（评论 6434092587）——六问题全部收口。
- 本会话累计关单：#18/#19/#4/#3 四单。剩余开放：#5 PDF 模型面接线
  （pkg_pdf 构建 + pdf_view 模型面激活——重磁盘）、#8/#11 测试轮。
- 磁盘 86%（1.4G 空）——#5 的 wasm32 构建估 300-600M，可行（构建后清
  target/wasm32 保留 pkg_pdf）。

---
Task ID: dev-I
Agent: main
Task: issue #5 PDF 模型面接线收口关单（pkg_pdf 构建 + 模型面激活 + UI 级验收）

Work Log:
- 根因定谳：#5 非源码缺口——脚本/package.json（build:wasm:pdf +
  build:full）/README 依赖文档/pdf_view.ts 模型面分支全部前会话已备；
  真实缺口 = build-pdf-wasm.sh 从未在存活环境跑成（环境重置历史中断）。
- build-pdf-wasm.sh 首次完整跑通：wasm32 release 1m01s → wasm-bindgen
  → web/pkg_pdf/（101M target 产物，磁盘可控）→ npm run build 接入
  （absent 提示消失）。
- 探针面转正：pdf_view.py 15/15（此前 2-12 SKIP 的模型面检查全部实跑
  ——text_selection/search/extract_text/annot_highlight/annot_list_filter/
  flush/incremental_prefix/double_revision_chain/scan_warnings）；仅加密
  档合法 SKIP。
- UI 级验收（acceptance_pdf5_ui.py 入库）：打开前 save 禁用 → 打开后
  启用（bridge 接线）→ 注释 460B 追加 → 按钮点击真实下载 1881B %PDF
  （字节 = prefix + appended 逐位一致）。
- 跨面回归：integration_journey 7/7（注释检查转正实跑）；empty_state
  11/11 无回归。

Stage Summary:
- 0a622a9 已推送，#5 关单（评论 6434134375）。本会话累计五单：
  #18/#19/#4/#3/#5。
- 剩余开放：#8 敌意测试轮 / #11 TR-2 覆盖率补强——纯测试轮 issue。
- pkg_pdf 为 gitignored 构建件——部署侧 npm run build:full 重建。

---
Task ID: dev-J（本轮终档）
Agent: main
Task: issue #8 K1-K6 重核闭环 + issue #11 TR-2A/B 收口——issue 面全清

Work Log:
- #8：K4/K5/K6 当前构建实测补遗（K4 设计内形态清单/K5 证伪——add×20
  undoDepth=20 全撤回无 8 步截断/K6 证伪——3×4 表导出零空段），
  K1/K2/K3 已由 #15/#10/#16 修复——六问题全闭环（5acd31e），重核脚本
  入库（web/k45_recheck.py + k5_recheck.py）。
- #11 TR-2A：5 轮独立 chromium OS 进程（冷 wasm）× 8 语料 × 2 面
  = 80 组 digest 零失配 → DETERMINISTIC（tr2a_cross_process.py +
  cross-process-determinism.md）。
- #11 TR-2B：外部语料 1317 用例三层（L1 CommonMark 655 + markdown-it
  652 注入 100% 幂等零崩溃/99.2% 保留；L2 pandoc 四件导出→LO 开验
  4/4 召回 92-100%；L3 POI 六件含 Word 崩溃夹具 6/6 零崩溃 + 2 项
  登记）——EXTERNAL-TEST-REPORT.md + provenance.json（五源四证，
  GPL 仅输入数据纪律）。TR-2C 已由 #16 先行收口。
- 踩坑：GitHub raw 直连 404 → API contents base64 通道；markdown-it
  夹具分隔线实长 42 波浪线（非 40）；commonmark spec.json 为生成物
  → 直取 spec.txt 解析 655 example 块。

Stage Summary:
- baf47d1 已推送，#11 关单（评论 6434311086）。**issue 面全清：开放 0**。
- 本会话七单：#18/#19/#4/#3/#5/#8/#11（另含 dev-A..E 断档重建确认）。
- 交付物链：引擎双仓 9 提交推送（977f4c9→baf47d1），web 19 套件全绿
  （新增 empty_state 11 检查），外部语料 1317 用例 + 三报告 + 六驱动
  脚本全部入库可复现。
- 战役状态：上一代 11 轮计划（dev-A..K）中 A(#15)/B(#13)/C(#14)/
  D(#16)/E(#17) 前会话完成 + F(#18)/G(#19) 本会话完成 + H(#4 复验)/
  I(#5)/J(#8/#11) 本会话收口——**全部 issue 高质量完成，无遗留**。

---
Task ID: m1（第 26 次环境重置恢复 + 全门禁复验 + 两缺陷修复轮）
Agent: main
Task: 用户「继续推进（记得磁盘管理）」——issue 面全清态下的复验战役：环境恢复、六面门禁、上游跟踪、缺陷修复关单、文档同步。

Work Log:
- 第 26 次环境重置恢复：双仓 fetch 甄别（本地 mdcore 快照陈旧 c4856f7
  [10-17 office-g 态] → ff 至 baf47d1；cleanroom → b3d5237）+ rustup
  1.99.0 重装（minimal + clippy/rustfmt + wasm32）+ cargo fetch 732M。
- issue 面侦查：双仓开放 0/0（dev-A..J 战役已于今日 02:40-05:10 全清
  12 单——上会话 worklog 终档 05:11 后即被重置吞）。
- 核心门禁复验（DEBUG=0 压缩构建纪律）：
  * 默认面 818/818（--exclude mdshell/mdshell-native——包名非目录名，
    踩坑登记：exclude 须用 package name）；
  * 发现①（#20）：mdc-collab presence `multi_client_states_sorted_and_merge`
    失败——跨视图 assert_eq 全量相等含 last_updated_ms（接收方本地活性
    时钟，yrs Awareness with_clock 语义），毫秒边界竞态=确定性 flaky；
    修：语义面投影三元组（tests-only a800c73）；
  * GUI 面 46/0（optroot hostlibs 711M 幸存：mdshell 42 + mdshell-native 4
    ——818+42+4=864 与 dev-G 口径精确互证）；
  * 发现②（#21）：`--features engine-pdfium,engine-lopdf` 编译断裂——
    p5-t4（37399eb 10-23）给 AnnotationWriter trait 加 product_bytes 漏改
    pdfd_writer.rs 测试侧 SharedLopdf impl；10-23→10-27 四天 30 测未实跑
    （dev-A..J「核心 860」口径不含特性面=覆盖回归）；修：同源透传
    （tests-only c57b6ce）；
  * 特性面环境按纪律重建：fetch-pdfium.sh（chromium/8086 SHA 校验）+
    spike 双二进制现场生成 fixture（pdf_a_spike + formfill_probe——
    fixtures 为 gitignore 面，重置即失）；特性门 109 过 0 败 + 6 忽略；
  * fmt 0（fmt 抓己改动一处——修后 clean）；clippy native+wasm32 恰 5
    wasm_bindgen 基线；wasm32 check（mdcore-wasm + mdc-pdf-wasm）过。
- 上游三锚点零增量：yrs #682 open 未合 / #676 open 0 评论 / crates.io
  仍 0.28.0 / TDF 26.2.7 未发（定档 10-29 倒计时 2 天；stable 目录
  25.8.7/26.2.6/26.8.1 维持）。
- 磁盘纪律全程：峰值 90%（1.0G）三次治理——spike target 即用即清
  （245M）+ cargo clean 2.6G → 终态 71%→64% 区间；DEBUG=0 全程。
- 文档同步：HANDOFF §6 速览刷新（dev-A..J + m1 终态置顶，30 轮战役
  速览降历史）。

Stage Summary:
- mdcore main baf47d1→c57b6ce 两提交推送；#20/#21 开单即关（评论
  6435427061/6435429005）；issue 面 0/0 维持。
- 门禁六面全绿（m1 口径）：默认 864 + 特性 30 复活 = 894 过 + 9 忽略；
  fmt 0 / clippy 恰 5 基线 / wasm32 过。
- 关键口径勘误：dev-era「核心 860」= 默认 818 + mdshell 42（GUI 4 另
  计）——特性面从未在其内；环境重置复验基线自本轮起含特性门+fixture
  重建（防再发 #21 同类静默断裂）。
- 剩余：DECISIONS 类别 3 用户裁决 19 项 + 产品决策三项——工程侧无欠
  账；web 19 套件复验待 npm 链重建（下轮候选）。

---
Task ID: m2（web 链重建 + 19 套件全量复验轮）
Agent: main
Task: web 测试面环境重建（npm/wasm-bindgen/pkg/pkg_pdf 四件全失）与全量复验。

Work Log:
- 幸存面：node v24.21.0/npm 11.19.0（系统装）+ playwright chromium
  613M（~/.cache）+ python playwright 模块；node_modules/pkg/pkg_pdf/
  wasm-bindgen CLI 全失。
- 重建链：npm install（18 包 2s）→ wasm-bindgen-cli 0.2.129 --locked
  精确对齐 Cargo.lock（前台直装——后台 nohup 会被会话收割，踩坑登记：
  cargo install 后台化两连死）→ build-wasm.sh（wasm32 release 1m38s）→
  build.mjs → build-pdf-wasm.sh（mdc-pdf-wasm 57s）→ dist 全量。
- 19 套件全量复验：**279 checks 0 败**（web_smoke 15/docx_export 25/
  docx_import 14/editor_toolbar 26/hostile 8/canvas 15/concurrent 16/
  contract 19/empty_state 11/evidence 16/IJ 7/lifecycle 15/oversized 18/
  sidecar 15/tr1 2/xlsx_export 9/xlsx_grid 20/xlsx_import 9/
  pdf_view 15[1 skip=加密档已知环境项]）——较 dev-J 口径 +30 checks
  （dev-H/I/J 新增面首次全量实跑）。
- 战役面互证：TR-4 选项桥/TR-16 设置键消费 XML（docx_export 25 含
  #16 面）/ 标题样式+域剥离+批注（docx_import 14 含 #13/#14/#19 面）/
  pdf_view 模型面 pkg_pdf 激活 15/15（dev-I 面复现）。

Stage Summary:
- 六面复验完整收官：核心 818 + GUI 46 + 特性 109 + web 279 + fmt/clippy/
  wasm32 基线 + 上游零增量——**当前 main（c57b6ce）全绿无死角**。
- 磁盘终态 82%（1.8G 空）——wasm32/pdf 双 target 即用保留（后续轮
  可增量复用），必要时 clean 回收 ~3.4G。
- web 链恢复 SOP 固化：npm install → wasm-bindgen-cli 0.2.129 前台
  装 → build-wasm → build.mjs → build-pdf-wasm → build.mjs（pkg_pdf
  接入需二次 build）→ serve 8123 → python3 tests/*.py。
