# 开源 PDF 生态缺口与差异项深度分析

> **文档性质**：净室分析续篇（分析员视角，不含任何实现代码）
> **前篇**：《PdfCraft架构深度剖析.md》——讲的是 pdfcraft **怎么做**（数据完整性 / 渲染管线 / 格式转换的内部机制）
> **本篇主题**：反向视角——**整个开源生态（包括 pdfcraft 自己）还没做出来的东西是什么、为什么没人做、差异项的价值在哪**，供"用 Rust 自研 PDF 引擎"做选型决策
> **基准日**：2026-10-09。调研方法：pdfcraft 本地克隆全量挖掘（ROADMAP / parity 清单 / 30 个 crate 自我声明 / vendor 补丁表）+ 三路生态网络调研（主流开源 PDF 项目能力边界、Rust crates 全景、格式转换格局），关键结论均附出处
> **阅读约定**：● 成熟可用 / ◐ 部分可用或进行中 / ○ 真空；"agent 小时"指 pdfcraft 口径下 AI 连续开发折算工时

---

## 0. 导读：怎样才算"缺失"

"开源没有 X"是一个容易被情绪左右的说法，本篇先钉死判定标准。一项能力缺失，必须在下面三层基准中**至少两层找不到可用开源实现**，才被本篇列为缺口：

1. **功能面基准（用户视角）**：以 Acrobat Pro 的离线功能清单为标尺（pdfcraft 的 parity 清单把它 TOML 化成 830 项，是目前最细的公开拆解）；
2. **规范面基准（格式视角）**：以 ISO 32000-2:2020 及其衍生标准（PDF/A、PDF/X、PDF/UA、PAdES）的规定能力为标尺——规范要求了而生态做不到的，是真缺口；
3. **工程面基准（生产视角）**：以"能否进入商业生产管线"为标尺——许可证传染（AGPL）、性能、保真度未验证，都算做不到。

用这三把尺子扫过整个生态之后，结论可以压成三句话，作为全文总论：

**第一，开源 PDF 生态的成熟区是"哑铃形"的。** 两端结实：结构变换端（qpdf）有 25 年积累，渲染端（MuPDF / PDFium / pdf.js）有巨头供养，生成端（typst / krilla）在 2025–2026 达到了含 tagged PDF 与 PDF/A 验证的成品级。中间塌陷：**编辑、修复、表单、XFA、脚本、色彩管理、签名信任链、标准符合性修正**这整条"语义中间层"，无论 C/C++、JavaScript 还是 Rust，都没有任何一个开源项目达到生产可用。

**第二，pdfcraft 是第一个正面进攻中间层的开源项目，但它自己也刚走到山脚。** effort 口径只完成 30–35%；五座主峰——自研渲染器（M2，15%）、既有文本编辑（M7，17%）、preflight 与标准转换（M11，18%）、XFA 与无障碍面板（M12，23%）、成品打磨（M14，5%）——全部停在四分之一以下。它证明了中间层"可以开工"，同时暴露了中间层的成本量级：距离完整平价还需 600–1100 agent 小时。

**第三，差异项的最大价值不在"比 pdfcraft 做得更好的地方"，而在"连 pdfcraft 都没打算做、或没做到的地方"。** 本篇第 5 章把这些地方整理成十大无人区并逐个深挖，第 6 章给出红海/蓝海决策框架。对自研者而言，pdfcraft 的价值是现成的分层骨架与验收口径；你的价值，只能来自它没覆盖的差异项。

---

## 1. 全景：开源 PDF 世界的能力分层图

把 2026-10 时点的全部主流开源 PDF 软件按"能干的事"铺开，会得到一张三层蛋糕图。这张图是本篇一切分析的底座，值得先在脑中建立：

```
┌─────────────────────────────────────────────────────────────────┐
│ 成熟层（生产可用，多项目竞争）                                    │
│  结构变换    qpdf / pikepdf     对象级变换·xref重建·线性化·加密    │
│  渲染        MuPDF(AGPL) / PDFium(BSD式) / pdf.js(浏览器)        │
│  命令行工具  Poppler 全家桶      pdftotext / pdftoppm / pdfsig    │
│  生成        typst / krilla / pdf-writer   含 tagged·PDF/A 验证   │
│  图像往返    pdftoppm / pdfimages / img2pdf / OCRmyPDF(拉丁)     │
│  PS 解释     Ghostscript(AGPL)   开源世界唯一完整的 PostScript    │
├─────────────────────────────────────────────────────────────────┤
│ 独苗层（存在但未成品，单一项目扛旗）                               │
│  纯Rust渲染   hayro(786★)        CPU-only·无knockout·无未嵌CID   │
│  纯Rust中间层 pdfcraft(5.5k★)    cos+engine·0.4版·渲染仍借用     │
│  纯Rust解析   pdf-rs / lopdf     读侧可用·编辑孱弱                │
│  纯Rust OCR   ocrs               拉丁字母·early preview          │
├─────────────────────────────────────────────────────────────────┤
│ 真空层（无任何可用开源实现，详见第3/5章）                           │
│  语义修复与preflight fixup · 对象级语义编辑器 · XFA动态表单        │
│  Acrobat JavaScript完整层 · 纯Rust ICC CMM · 结构树读取与编辑      │
│  PAdES LTV全链与AATL信任 · CJK OCR · JBIG2/JPX编码器 · GPU渲染    │
│  表格/多栏语义重建(→Office) · 语义级文档比较 · AI文档助手          │
└─────────────────────────────────────────────────────────────────┘
```

这张图回答了第一个直觉问题：**开源 PDF 工具看起来那么多，为什么还是"没有 Acrobat 的替代品"？** 因为生态里的项目几乎全部堆在成熟层互相重叠（渲染器至少五个、解析器至少八个、转换 CLI 不计其数），而真空层里每一格都对应 Acrobat 一个卖钱的功能域。项目数量与能力覆盖严重不成比例——这是选差异项时首先要看清的事实。

第二个问题是结构性的：**为什么中间层真空？** 调研证据指向四个互相强化的原因，而不是"没人想到"：

1. **知识壁垒**：preflight 需要印刷行业的规范知识库（油墨覆盖率、出血、透明度扁平化的工程判断），ICC 色彩管理需要色彩科学工程，LTV 签名需要 PKI 信任运营（AATL、吊销策略、时间戳治理）。这些是行业 know-how 而非公开算法，规范文本（ISO 19005 / 15930 / 32000-2）只定义"什么合规"，不教你"怎么把不合规修成合规"。
2. **收入结构**：认真做这一层的公司——callas pdfToolBox、Enfocus PitStop、Artifex 商业版——全部以此为生，商业定价（千美元级/年）本身就说明了该层的维护成本；开源世界没有任何一个项目有对应的持续资金。
3. **用户结构**：开源主力贡献者是开发者，要的是管线与 API（提取、转换、渲染），而中间层的终端用户是排版师、印前工程师、法务与政务人员——这群人向开源提需求的能力最弱。
4. **语料与回归护城河**：Acrobat 与 callas 各有二三十年真实坏文件数据库。qpdf 也花了 25 年才把"结构修复"做稳；pdfcraft 用 9 天夜跑 fuzz 就炸出 7 个 OOM 加 11 个挂起，可见这条护城河的宽度是时间本身。

理解这四条，就能理解后面所有具体缺口的成因，也能判断哪些缺口"补得上"、哪些"补上了也养不起"。

---

## 2. pdfcraft 自身缺口盘点：金字塔形的未完成

前篇已经讲清 pdfcraft 的机制设计；本节只盘点**它自己承认或被计量出来的缺口**——这是判断"跟随它"还是"错开它"的事实基础。全部数据来自本地克隆（HEAD `7c35809`，2026-10-09）的 `parity/acrobat-features.toml`、`ROADMAP.md` 与各 crate 自我声明，其中 parity 清单是唯一可信准绳（理由见 2.5）。

### 2.1 总量：按优先级呈金字塔塌陷

当前实测 parity 清单共 **830 项**（417 shipped / 74 partial / 316 planned / 23 不适用），比 ROADMAP 快照（806 项 / 411 shipped，2026-10-07）又推进了一小步——两三天加六个功能，是这个项目的日常节奏。按优先级分层：

| Tier | shipped | partial | planned | 总数 | 完成 % |
|---|---|---|---|---|---|
| P0（必备） | 224 | 24 | **3** | 251 | 89.2% |
| P1（重要） | 175 | 41 | 110 | 326 | 53.7% |
| P2（进阶） | 17 | 9 | 161 | 187 | 9.1% |
| P3（长尾） | 1 | 0 | 42 | 43 | 2.3% |

P0 只剩 3 个 planned：大文件懒加载（`core.lazy-loading`）、纯键盘操作（`a11y.keyboard-only`）、性能预算（`misc.performance-budgets`）——注意这三个恰好都不是"功能"而是"工程质量"，说明**功能层的 P0 已近收官，工程层的 P0 才刚开始**。更要紧的是 24 个 P0 partial：其中**七个渲染项（页面渲染、shading、透明、Type3、CID 字体、图像解码、深缩放）全部标注 "via hayro bootstrap"**——P0 的 89% 里有一块是用借来的渲染器撑起来的，这是金字塔底部最软的一块砖。

### 2.2 十四领域完成度：最低的五个域

| 领域 | shipped % | 备注 |
|---|---|---|
| I · OCR | **19%** | 5/27，模型只认无重音拉丁（实际语言表只有英语） |
| N · 杂项（偏好面板/本地化/安装器/用户文档） | **24%** | 50 项 planned，数量最多的未动区 |
| K · 优化 | **26%** | preflight 全家 14+ 项全部 planned |
| J · 创建/Office | 34% | xlsx/pptx/svg 导出、Office 输入全未动 |
| L · 打印 | 38% | Output Preview、分色、叠印预览全未动 |
| H · 签名 | 47% | B-B 可用；B-T/LTV 的网络侧空白 |
| C · 编辑 | 46% | 行级替换；段落回流、RTL/CJK 编辑未动 |
| G · 保护 | 52% | 公钥加密、脱敏代码表未动 |
| B · 查看 | 55% | 依赖渲染，7 个 P0 partial |
| D · 组织 | 57% | 较扎实 |
| M · 无障碍 | 62% | 31/32 规则；autotag 与三个面板全未动 |
| E · 注释 | 67% | — |
| F · 表单 | 70% | XFA 拖后腿 |
| A · 核心 | 75% | 线性化、Arlington 校验、PDF 2.0 三项未动 |

### 2.3 六大自认短板与五大风险

README "What's next" 原文的自我分类（这是全生态少见的诚实声明，值得整段采信）：

- **Good today**：查看与搜索、组织合并拆分、多数注释、表单填写与制作（含沙箱 JS）、密码/密文/净化、基础签名、打印、无障碍检查器、CLI；
- **Still borrowed**：页面绘制整个走 hayro；
- **Thin or missing（原文清单）**：既有文本的可靠编辑（**尤其 CJK**）、拉丁之外的 OCR、**Office 导入导出**、**签名时间戳与长期验证**、**PDF/A/X/UA preflight**、**XFA 表单**、本地化；
- **Hardening**：fuzz 仍会在敌意文件上炸出崩溃与挂起；**质量从未与 Acrobat 并排对比过**。

ROADMAP 风险清单（八条原文归并）则进一步给出了量化预期：文本编辑保真（字体/子集/回流）、XFA 动态布局、真实世界签名链与吊销、渲染长尾（Type3/坏字体/shading）、PDF/A 与 UA 转换正确性、Web 端 CPU 渲染性能、敌意输入（"每轮 fuzz 都有新发现"）、未测量的保真度。对照里程碑完成度——M2 渲染 15%、M7 编辑 17%、M11 preflight 18%、M12 XFA+无障碍 23%、M14 打磨 5%——**风险清单与低完成度里程碑完全重合**：他们最怕的正是他们最没做完的。

### 2.4 未完成项里的"高价值清单"

316 个 planned 项中，P1 的 110 项按主题归组后，能看到明确的"价值密度"分布。这对自研者是最实用的一张表——每一组都是一个潜在的差异项赛道：

| 主题组 | 代表性未完成项（编号摘自 parity TOML） |
|---|---|
| 签名链（7+ 项） | pkcs11 智能卡、field-mdp、签后锁定、**时间戳服务器配置**、系统根证书信任、签出版本比较、Web 签名 |
| OCR（8 项） | 可编辑文本输出、旋转检测、deskew、despeckle、背景去除、相机透视、**可插拔引擎**、Web OCR |
| Office/导出（5 项） | **xlsx 导出（含表格识别）**、pptx、svg、导出 OCR、截图创建 |
| Preflight/印前（14+ 项全未动） | preflight 全家、**PDF/X 验证与转换**、**PDF/UA 验证**、Output Preview、**分色预览**、油墨覆盖率、转色、叠印预览、打印机标记、hairline 修复、色彩管理 |
| 无障碍（8 项） | **autotag**、autotag-fields、阅读顺序工具、Tags/Order 面板、artifact 标记、Make Accessible 动作 |
| 核心扩展（7 项） | **公钥加密打开**、**线性化（Fast Web View）**、Arlington 校验、PDF 2.0 三项（UTF-8 字符串/关联文件/命名空间）、Web 恢复 |
| 编辑（7 项） | **RTL/CJK 编辑**、矢量编辑、对象排列、对齐分布、Edit Object 工具、列表、编辑保 tag |
| 保护（5 项） | 证书加密、收件人权限、脱敏代码表、词表脱敏、模式本地化 |
| 成品化（10+ 项） | 十个偏好面板、本地化、签名安装器、用户文档、并排比较视图、动作归档、自定义命令 |

### 2.5 质量会计：shipped ≠ Acrobat 质量

pdfcraft 的自我会计体系是本节最值得信任的部分，因为它主动暴露了三个口径问题——这也是所有"平价百分比"类宣传的共同陷阱：

1. **测试深度**：当前 417 个 shipped 功能里，**146 个只有一条测试**；引用外部 oracle 的证据仅 **4 条**（qpdf ×2、OpenSSL ×2）。换句话说，"已发货"的真实含义是"存在且有至少一条测试"，距离"与 Acrobat 同质量"有量级差距——这是 ROADMAP 原话承认的。
2. **数量口径漂移**：README 说 200+ 测试，ROADMAP 说约 600 个，实测自产代码 `#[test]` 有 1356 个（154 个文件）；行数上 ROADMAP 的"72k kept, tested"对照实测约 130.7k 自产 Rust 行——"保留下来的带测试代码"是一个过滤后口径，实际产出更大但含未计入部分。
3. **文档双向漂移**：cos 的 lib.rs 头注释还写着"加密与对象流输出未做"（实际早已完成）；forms 的 "Not yet" 清单落后于 ROADMAP；engine 工具目录的 34 个 `Planned("Mxx")` 条目中有两条与 parity 状态矛盾。**结论：评估这个项目（以及任何同类项目）只能信 parity TOML 这类机器计量文件，不能信 README 与代码注释**——这条经验对自研者的意义是：从第一天就把"状态"放进机器可读的单一事实源。

补充两个鲁棒性事实：主 CI 曾连续红 30+ 次（clippy/Windows 换行/性能 flaky）；三轮回合 15 分钟 fuzz 炸出 7 个 OOM 与 11 个挂起，其中 7 个在自家 cos 解析器。语料面是正面消息：pdf.js 983 文件语料 963 个干净打开渲染、958 个往返、0 崩溃。

### 2.6 借用债务：34 条补丁悬在头上

vendor 目录携带 4 个上游 crate（hayro-interpret 0.7.0、hayro 0.7.1、hayro-jbig2 0.3.0、hayro-syntax 0.7.2、lopdf 0.45.0）共 **34 条补丁**，全部是敌意输入加固，每条带回归测试。关键事实是：**除 lopdf 的 3 条与上游 PR #589 对齐外，其余 31 条上游均未合并修复**。pdfcraft 的纪律是"上游发布修复即移除 vendor"，但只要 hayro 上游不收，这些补丁就要永远自己背着——自研渲染器（M2，预算 150–250 agent 小时）完成前，渲染质量与债务清偿都被锁在别人手里。

### 2.7 本节结论

pdfcraft 的缺口分布呈金字塔形：底层（P0）近收官但靠借用撑着；中层（P1）半数未动且未动部分恰好是高价值差异项（签名链、preflight、autotag、Office、CJK）；顶层（P2/P3）几乎空白。**它的缺口清单约等于"开源公认难点清单"的机器可读版**——跟随它的骨架、错开它的主峰、深挖它 23% 以下的山谷，是第 6 章决策框架的基本思路。

---

## 3. Rust 生态空白地图：生成端成品级，中间层无人踏足

pdfcraft 之外，"用 Rust 写 PDF"这件事本身在 2026 年处于什么状态？这是自研者必须独立评估的环境变量——你的引擎可以不依赖 pdfcraft，但躲不开整个生态的地板高度。下表汇总 crates.io / GitHub 实测数据（2026-10-09，下载量为历史总量）：

| 环节 | 代表实现 | 状态 | 关键短板 |
|---|---|---|---|
| 解析 | lopdf（2.3k★，2335 万下载） | 维护中 | 全量内存加载；RUSTSEC-2026-0187 无界递归；增量保存存在损坏 /Prev 的已知 bug |
| 解析 | pdf-rs（1.7k★，62.5 万） | 维护慢 | 读侧质量好（容错/解密/图像解码），写侧自认"实验性"；渲染线因 pathfinder 归档而死 |
| 写入 | pdf-writer（515 万） | 成熟 | 低层流式 writer，typst 生产背书，只写不读 |
| 写入 | **krilla**（typst 团队出品，457★，260 万下载） | **成品级** | tagged PDF + PDF/A-1/2/3/4 + PDF/UA-1 导出经 Arlington+veraPDF+6 查看器矩阵验证；但明确不做加密/签名/复杂色彩/排版 |
| 渲染 | **hayro 家族**（786★，413 万） | WIP 独苗 | 自称最完整的纯 Rust 渲染器，1400+ PDFBox/pdf.js 语料；**CPU-only、无 knockout group、无未嵌入 CID 字体支持、性能未优先** |
| 渲染 | zpdf（Xero-Team） | 刚起步 | wgpu GPU 路线 2026-09 才开始，下载千级 |
| 过滤器 | flate2 / miniz_oxide / weezl | 极成熟 | deflate/LZW 完备；**JBIG2、JPEG2000、CCITT 只有 hayro 的只读解码器，编码器为零** |
| 字体 | harfrust（官方 Rust 移植）/ skrifa / ab_glyph / unicode-bidi | 活跃 | 通用层成熟；**PDF 特有层半真空**：预定义 CMap（GBK/UniJIS）、非 Identity 写入、竖排、未嵌 CID 替代渲染无人做全 |
| 加密 | 各解析器内置读侧 | 半真空 | RC4/AES-256 **读**侧多家可用；**写**侧（加密态保存、crypt filter、增量更新于密文）无成熟实现 |
| 签名 | underskrift / pdf-signer | **近真空** | 全部 2026 年新生物、下载 <1.1 万、与 Acrobat 互操作未验证；RustCrypto 原语成熟，缺的正是 PDF 协议层 |
| 绑定 | pdfium-render（714★，310 万） | 生产事实主力 | PDFium 宽松许可——纯 Rust 引擎至今打不过 C++ 绑定 |
| 绑定 | mupdf-rs | 受限 | MuPDF AGPL 传染，商业集成需 Artifex 授权 |
| OCR | ocrs + rten（1.9k★） | early preview | 自评"错误多于商业引擎"，**仅拉丁字母；纯 Rust CJK OCR 是真空** |

### 3.1 十六条真空/半真空清单

综合上述，Rust 侧按"真空（无实现）/ 半真空（仅有话术级或玩具级实现）"分级：

| # | 能力 | 级别 |
|---|---|---|
| 1 | qpdf 级对象保持重写 / 线性化 / 字节级变换 | **真空**（flpdf、pdfrs 等 2026 新生全部下载千级） |
| 2 | 损坏文件容错修复（qpdf --repair 等级） | **真空** |
| 3 | 完整对象级语义编辑器（内容流编辑+外观重建） | **真空** |
| 4 | XFA 表单（连读取都没有） | **真空** |
| 5 | Acrobat JavaScript 层（boa/rquickjs 存在但无 PDF API 绑定层） | **真空** |
| 6 | 纯 Rust ICC 色彩管理 CMM（全员绑 lcms2 C 库） | **真空** |
| 7 | 既有 PDF 的结构树 / Tagged 读取与编辑 | **真空**（krilla 只解决了生成侧） |
| 8 | CJK OCR | **真空** |
| 9 | JBIG2 / JPEG2000 / CCITT 编码器 | **真空**（只有 hayro 只读解码） |
| 10 | PDF/A 合规验证器（veraPDF 等价物） | **真空**（krilla 只在 CI 验证自家导出） |
| 11 | 增量保存与 xref/Prev 链完整性 | 半真空（lopdf 的 /Prev 损坏 bug 至今在修） |
| 12 | AcroForm 完整表单引擎 | 半真空 |
| 13 | 数字签名生态（PAdES/LTV/DSS） | 半真空（underskrift 等 <半岁） |
| 14 | 加密写出（AES-256+crypt filter） | 半真空 |
| 15 | GPU 渲染管线 | 半真空（zpdf wgpu 刚起步，hayro 明确 CPU-only） |
| 16 | 非 Identity CMap 与竖排的读写全链路 | 半真空 |

### 3.2 三个结构性观察

**新生军现象**。2026 年出现了 pdf_oxide（1.1k★，宣称"0.8 毫秒/3830 文件 100% 通过"）、zpdf、oxidize-pdf、flpdf、pdfrs 等一批不满半岁的 Rust PDF 项目，下载量全部在千到万级。共同特征是营销话术浓、全部挤在**解析**这一最容易演示的环节，没有一家触碰上表任何一条真空项。这说明"Rust PDF"风口确实起来了（对融资/获客有利），但风口的能量至今没有投向中间层——留给后来者的空位反而更大。

**生产仍靠绑定**。pdfium-render 以 310 万下载统治 Rust 生产环境，理由朴素：PDFium 的渲染保真度与性能是 Chrome 级的，而纯 Rust 引擎（hayro）连 knockout group 和未嵌入 CID 字体都还没有。任何自研渲染器在发布之前，都必须假设用户会拿它和 PDFium 逐页对比——**渲染是唯一"用户每天肉眼验收"的环节**，这决定了它是回报最高、也是翻车最快的赛道。

**生成与编辑的极端不对称**。krilla 已经用 Arlington 模型 + veraPDF + 六查看器视觉回归矩阵验证自己的**输出**，这是生成端的天花板做法；而"把别人的 PDF 读进来、改一改、写回去还不坏"这条链，Rust 侧没有任何人建立过等价的验证体系（pdfcraft 的 qpdf-oracle 与语料往返是最接近的尝试）。不对称本身就是机会：**谁能把 krilla 式的验证纪律搬到编辑链上，谁就重新定义了"编辑器可信"的含义**。

---

## 4. 全开源生态 vs Acrobat 差异矩阵

把视野从 Rust 扩大到全部语言，用第 0 章的三层基准逐域打分。这张矩阵是"差异项"的总目录，第 5 章从中挑出最深的价值带逐个挖掘。

| 能力域 | qpdf/pikepdf | pdf.js | MuPDF | PDFium | Poppler/GS | krilla/typst | OCRmyPDF | **pdfcraft** | 商业独占者 |
|---|---|---|---|---|---|---|---|---|---|
| 结构修复/xref 重建 | ● | ○ | ◐ | ○ | ◐ | — | — | ◐(M1 78%) | Acrobat |
| 增量保存/保签名字节 | ◐ | ○ | ○ | ◐ | ○ | — | — | ◐ | Acrobat |
| 渲染保真 | ○ | ◐ | ●(AGPL) | ● | ◐/◐ | — | — | ◐(借 hayro) | — |
| ICC 色彩/叠印/专色 | ○ | ○ | ◐ | ○ | ◐(分色设备) | ◐(写侧) | — | ○ | Acrobat/callas |
| Preflight/fixup | ○ | ○ | ○ | ○ | ◐(PDF/A 有坑) | ◐(生成侧) | — | ○(19→26%) | **callas/PitStop** |
| Tagged PDF 生成 | — | ○ | ◐ | ○ | ○ | **●** | — | ○ | — |
| Tagged 读取/编辑/autotag | ○ | ○ | ○ | ○ | ○ | ○ | — | ○(M12 23%) | Acrobat |
| AcroForm 表单 | ○ | ◐ | ◐ | ◐ | ○ | — | — | ◐(70%) | — |
| XFA（静态/动态） | ○ | ◐(静态部分) | ○ | ○ | ○ | — | — | ◐(静态+缺口) | Acrobat |
| Acrobat JS 沙箱 | ○ | ◐(受限+bug) | ○ | ◐ | ○ | — | — | ◐(forms 子集) | Acrobat |
| PAdES 签名/LTV/吊销 | ○ | ◐ | ◐ | ◐(只读枚举) | ◐(pdfsig 无吊销) | — | — | ◐(结构有/网络无) | Acrobat/欧签生态 |
| OCR | — | — | — | — | — | — | ●(拉丁归档) | ◐(19%,英语) | ABBYY/Acrobat |
| 文本编辑/语义重建 | ○ | ◐(注释级) | ◐ | ◐ | ○ | — | — | ◐(行级) | Acrobat |
| 文档比较 | ○ | ○ | ○ | ○ | ○ | — | — | ◐(文本 diff) | Acrobat |
| PDF→Word/Excel | ○ | ○ | ○ | ○ | ○ | — | — | ◐(启发式导出) | Acrobat |
| Office→PDF | — | — | — | — | ◐(LibreOffice 分页不一致) | ◐ | — | ○ | Aspose/Solid |
| PDF/A 转换+修复 | ○ | ○ | ○ | ○ | ◐(GS 生成需叠参) | ◐(生成侧) | ◐ | ◐(A-2b/3b 子集) | **callas/Acrobat** |
| PostScript 解释 | ○ | ○ | ○ | ○ | ●(唯一,AGPL) | — | — | ○ | — |
| AI 文档助手 | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○(明确排除云) | **Acrobat 2026** |

矩阵速读：**每个 ● 都有两巨头背景**（qpdf 25 年、MuPDF/Artifex 商业供养、typst 团队、Poppler 由 freedesktop 社区维护）；**每个 ○ 行都是 Acrobat 的收入来源**；pdfcraft 的 ◐ 数量最多但没有一个 ● ——它是最宽的，但还不是最深的。另有三行值得单独展开：

### 4.1 许可证差异项：AGPL 墙的位置

MuPDF 与 Ghostscript 都是 AGPL/商业双许可——官方页面直接写明"要嵌入产品请购买商业许可"。这堵墙直接塑造了开源格局：**所有需要商业分发的产品被迫二选一——买 Artifex 授权，或者用 PDFium（BSD 式）**。Chrome 选了后者，于是 PDFium 成了事实上最安全的商业渲染底座；而 Rust 侧 hayro/krilla 的 Apache/MIT 双许可，正是给"既不想付授权费、又不想传染 AGPL"的中间地带准备的。对自研者的含义：**MIT/Apache 本身就是一个差异项**——如果你的引擎能以宽松许可填补任何一条上表的 ○，商业集成者会主动找上门，这在 AGPL 项目上是结构性不可能的。

### 4.2 PDF.js 的边界：浏览器查看器 ≠ 文档平台

pdf.js 的定位决定了它的能力边界：内存模型为浏览器标签页设计，大文档/高分辨率渲染会崩标签页；XFA 只有静态部分场景，动态 XFA 与 Adobe 官方都确认不显示；内嵌 JS 以受限沙箱执行，表单联动存在已知 bug；注释编辑 2025 年才逐步引入。它是最广泛部署的开源渲染器，但**从来不是编辑器或文档平台**——把它当竞品的人其实看错了赛道，把它当依赖的人则终将撞到内存与 XFA 两堵墙。

### 4.3 Acrobat 2026 的零对标区：AI

2026 年 Acrobat 的主要增量全部在 AI 侧：10 月 AI Assistant 上线**联网搜索问答**（答案带可点击溯源引用）；9 月新增"把密集文档变成视觉与音频摘要"与"对话式润饰交付物"；Search 工具内置对话式任务入口。开源侧**零对标**——没有一个项目做文档内 RAG+联网溯源、文档转音频、对话式任务代理。pdfcraft 明确排除云功能（parity 清单 23 项 na），这是净室立场决定的；但对自研者，"本地文档+本地模型"的隐私版 AI 查询是一条 pdfcraft 结构上永远不占的赛道。

---

## 5. 十大无人区深挖：每一条都是 Acrobat 的收费理由

以下十个领域，经三层基准判定为"开源无生产可用实现、且 pdfcraft 完成度低于四分之一或明确未排期"。每项按统一模板拆解：现状 → 为什么没人做 → 技术难点 → 自研机会评级（★ 链：价值密度 ÷ 实现成本，满分三星）。

### 5.1 语义修复与 Preflight Fixup 引擎 —— 机会 ★★★

**现状**：开源最强者是 veraPDF，但它**只验证不修复**，且覆盖限于 PDF/A/UA 子集；Ghostscript 能生成 PDF/A，但典型输出会被 veraPDF 挑出"DeviceRGB 无 ICC"这类错，社区要靠 pdf2archive 式叠参数才能"多数情况"合规；把任意 PDF 自动纠正成合规（嵌入缺失字体、色彩转换、透明度扁平化、XMP 修复）的能力，全世界开源为零。印刷社区的原话是："callas pdfToolBox 和 PitStop 是唯一可信的 preflight 方案"。pdfcraft 的 preflight 仅覆盖"从对象图可判定"的 PDF/A-2b/3b 规则子集，修复仅限 XMP/OutputIntent/动作禁用，且明说"没嵌入的字体修不了"——全家族 14+ 项 parity 条目全部 planned，M11 完成度 18%。

**为什么没人做**：fixup 的难度不在判定而在**决策**——同一个"透明度未扁平化"警告，面向屏幕阅读与面向 PDF/X-4 印刷的处置完全相反；修复器必须内建印刷行业规范知识库，而这知识库正是 callas/Enfocus 的商品本体。开源无对应维护资金。

**技术难点**：规则引擎本身是中等工程；难在三类 fixup 的副作用控制——字体子集补嵌必须重算宽度和编码、色彩转换要走 ICC 设备链、透明度扁平化等价于一次受控重渲染。每个 fixup 都要"验证一遍无回归"，这与渲染保真共享同一套难点。

**对自研者的含义**：这是矩阵里"开源零竞品 + 商业定价最高 + pdfcraft 最薄"三项叠加的格子。起步路径可从 PDF/A-2b 一个 profile 的**只修不判**做起（判断交给 veraPDF，修复自己做）——把 veraPDF 当外部 oracle，恰好复用 pdfcraft 的 oracle 方法论而错开它的路线图峰值。验收口径现成：veraPDF 通过率。

### 5.2 打印级保真的自研渲染器 —— 机会 ★★

**现状**：MuPDF/PDFium 的保真度是生产级，但一个 AGPL 一个 C++（绑定）；纯 Rust 只有 hayro——自称最完整，实际无 knockout group、无未嵌入 CID 字体替代、性能未优先、1400 文件语料远小于 PDFium 的 Chrome 级实测。pdfcraft 的方案是 bootstrap+冻结 API（前篇 §3），自研预算 150–250 agent 小时，M2 完成 15%。GPU 路线（zpdf wgpu）刚起步；Lege 嵌入式渲染器实测单线程比 PDFium 快 50%、多线程 10 倍，但非通用库。

**为什么难**：渲染是"用户每天肉眼验收"的环节，长尾无底——pdfcraft 的 vendor 补丁表（自引用平铺、宽度炸弹、像素炸弹、路径爆炸、EI 歧义、裁剪语义）只是 9 天 fuzz 的首批战果。shading、Type3、透明组、半色调、overprint 每一项都有独立巨坑，且坑的分布只有真实语料能告诉你。

**对自研者的含义**：正面全线冲锋（复刻 PDFium）是资源黑洞；但**垂直裁剪**可行——"文本为主、无叠印、无专色"的办公文档渲染器，验收用 pdf.js 语料+PDFBox 语料，工程量收进单人可及范围。这条线的判据是：你的目标用户是否每天需要渲染广告打样稿。不需要，就借 hayro；需要，就只做垂直。

### 5.3 结构语义重建：表格/多栏/阅读顺序 → Office —— 机会 ★★★

**现状**：pdf2docx（开源事实标准）已被 Artifex 官方停止积极维护、改 MIT 重发布——**主力弃养且无接棒者**；其规则法在复杂表格（窄列内容并入邻列）、RTL（阿拉伯整串反转）、旋转文本上系统性失败。LibreOffice Draw 的"PDF 编辑"实为 Poppler 导入重建，复杂布局即位移破碎、矢量甚至被栅格化。Acrobat 导出 Word 公认最强（表格识别为表格、扫描件走 OCR）。学术侧有 OmniDocBench（CVPR 2025，1651 页、28 类属性标注）与 DocLayNet（IBM，80863 页人工标注），但**社区重心已转向 PDF→Markdown（Marker/Docling）而非可编辑 Office**。pdfcraft 的导出是"字号大于正文即标题"的显式启发式，xlsx/pptx 导出全部 planned。

**为什么没人做**：文本提取是已解决问题，语义重建是研究问题；开源受众（开发者）要 Markdown 喂 LLM，不要 docx 给排版师——付费用户与开源贡献者再次错位。

**技术难点**：表格结构识别（跨行列span、无线表格）、多栏阅读顺序、连字拆解（fi/fl）、基线聚类区分标题/正文/脚注，每项都有独立文献；难点在于把"识别准确率"换算成"可编辑文档的可用性"——错一个 span 整表报废。

**对自研者的含义**：与 5.1 并列的价值顶格项。判定"缺失"的三层基准在此全部满足；基准数据集现成（OmniDocBench/DocX-Bench 刚起步），可量化；且**开源侧没有 strong incumbent**（pdf2docx 已死）。若做，先建评测再写代码——这是与所有失败前辈最大的差别。

### 5.4 Tagged PDF 的另一半：autotag 与结构树编辑 —— 机会 ★★☆

**现状**：生成侧已被 krilla 做到成品级（typst 0.14+ 默认输出 tagged+PDF/A）；**读侧与编辑侧全生态真空**——无任何开源工具能编辑既有 PDF 的结构树；无障碍社区只有验证器（veraPDF/PAC，均只报错不修）。Acrobat 有 autotag + Tags/Order/Content 三面板完整工作流。pdfcraft 32 条 a11y 规则发了 31 条（对比度留给人工），autotag、三面板、阅读顺序工具全部 planned（M12 23%）。

**为什么没人做**：无障碍规范的受益者（视障用户）几乎没有开源开发预算；政府采购合规压力是需求主引擎，而采购方买的是 Acrobat 培训生态。法规（欧美的 EN 301 549、文档无障碍指令）正在把这个域变成刚需，但钱流向了合规咨询而非开源工具。

**技术难点**：autotag 本质是 5.3 的镜像问题——从内容流反推文档结构（标题层级、表格、列表、阅读顺序），然后写回 StructureTree 并重算 ParentTree 指针。树编辑的麻烦在于结构元素与内容流的 marked-content 标记必须双向一致，改一处动两处。

**对自研者的含义**：法规驱动的刚性需求 + 零竞品 + 生成侧参照系现成（krilla 证明了树写出的验证方法）。与 5.3 共享 80% 的技术底座（结构推断），可一并收割两个市场。

### 5.5 PAdES LTV 信任链：吊销、DSS、TSA —— 机会 ★★☆

**现状**：全链 = 签名 + 时间戳 + DSS/VRI 嵌入 + 吊销数据抓取（AIA/CRLDP）+ 验证时重建信任。开源碎片：pdfsig 能验签但**不做 CRL/OCSP 检查**，AATL 证书几乎不被系统信任库收录；MuPDF/PDFium 无完整 LTV；Rust 侧 underskrift 等新生物互操作未验证。在 pdfcraft 一侧，B-B 与 B-LTA 的**结构**（sign→DSS→timestamp）已经做到，但吊销抓取（网络层）、TSA 配置、PKCS#11 智能卡全部未做——crate 纪律是永不开 socket，网络决策被 owner 挂起待定。欧洲的严肃开源实现是 Java 系的 EU DSS 库，非 Rust。

**为什么没人做**：代码只占三成，七成是**信任运营**——AATL/EATL 信任列表的更新分发、TSA 服务选择、吊销策略。这些是运营成本，开源项目养不起；而法律效力要求让"差不多"的实现对法务用户毫无意义。

**对自研者的含义**：RustCrypto 原语成熟，协议层（CMS/PAdES）是清晰的规范工程；抓取层做成可插拔 trait（离线/在线/自带 CRL 文件），正好绕开 pdfcraft 卡住的"是否开网"决策。验收 oracle 用 pdfsig+OpenSSL+EU DSS 互测。法务/政务/归档垂直市场付费意愿最高。注意：这是十个无人区里**唯一"难在运营不在代码"的**——若你只做库不做服务，差异化有限；若做服务，就进入了 Adobe Sign 的射程。

### 5.6 XFA 表单 —— 机会 ★

**现状**：全生态接近真空——pdf.js 只支持静态部分场景；Adobe 自己已弃 XFA（Acrobat 新版默认降权）。pdfcraft 是唯一认真做的开源项目（静态 XFA 的模型/布局/脚本/数据包全部有骨架），但其 README 的"故意不做"清单很长：默认绑定 only、脚本对象缺 xfa.template/connectionSet、签名/图像/条码/密码字段留空、内存不设上限、保真度"肉眼对政府表格检查，未像素对比"。M12 完成度 23%，ROADMAP 风险清单把它列为四大吞时怪兽之一。

**为什么没人做**：它是**死格式**——规范不公开（ISO 19444 只是 XML 架构层）、动态布局引擎等价于再造一个 HTML 排版器、需求方只剩政府遗留表单。做了没有新用户，不做也没有新损失。

**对自研者的含义**：除非你的垂直市场就是某国政务遗留系统迁移，否则这是十个里唯一建议**明确跳过**的。pdfcraft 做它的理由是"Acrobat 有所以平价清单上有"，自研者没有这个义务。

### 5.7 纯 Rust ICC CMM 与印前色彩 —— 机会 ★★

**现状**：Rust 侧全员绑 lcms2 C 库（pdf_oxide 的 icc-lcms2 feature 是缩影），无纯 Rust CMM；叠印预览、专色叠印模拟（DeviceLink/SpotOverprint profile 是 ICC 的研究性扩展）、模拟纸色软打样在 Acrobat/InDesign 里是交互功能，在 Ghostscript 里只有 tiffsep/psdcmyk 分色输出设备这种批处理近似。pdfcraft 的 color crate 是 L2 预留槽位，色彩管理整条 planned。

**为什么没人做**：色彩科学工程门槛高（PCS 转换、感知渲染意图、黑点补偿各有论文），用户群小（印前工程师），且 lcms2 宽松许可免费好用——**重写收益对通用场景接近零**。

**对自研者的含义**：单做 CMM 性价比低；但**叠印预览+分色软打样**（不做 CMM，只做合成模拟）是印前用户每天要用的交互功能，开源真空、工程边界清晰（走 PDF 的 blend/overprint 语义而非完整 ICC 链）。若你的垂直是出版印刷，做模拟不做 CMM。

### 5.8 CJK 全链：CMap 预定义 / 竖排 / CJK OCR —— 机会 ★★★

**现状**：这是**全生态（含 Acrobat 之外的商用替代品）共同的最弱区**，且 pdfcraft 自己承认 CJK 是文本编辑的最大短板（"reliable editing of existing text, especially CJK"，`edit.rtl-cjk-editing` planned）。拆开看：预定义 CMap（GBK/UniJIS-UCS 等）读写——Rust 侧 hayro-cmap 0.1.0 刚起步、krilla 只覆盖 Identity-H；竖排（WMode 1）——pdfcraft 仅在文本提取侧支持（vendor 补丁 #126），写入与编辑为零；CJK OCR——纯 Rust 零实现（ocrs 仅拉丁，连带重音的都不支持）；CJK 字体子集化嵌入——krilla 强但那是生成侧。转换侧同样重灾区：连字/竖排/旋转文本是 pdf2docx 公认失败模式。中文世界的事实方案仍是绑定 PDFium/MuPDF。

**为什么没人做**：核心贡献者社区（欧美）对 CJK 的痛感为零；CJK 的规范面（Adobe CMap 资源、竖排排版习惯、CJK 字体结构）又是独立的知识体系。

**对自研者的含义**：**中文市场视角下性价比最高的差异项**。它不是一个大功能而是四个互相咬合的中型件（CMap 表、竖排布局、CJK 子集化、CJK OCR 可外接 PaddleOCR 等成熟模型做桥）；每件都有明确验收（与 PDFium/PDFTotext 对照）；且 pdfcraft 在该域只有 46% 的编辑完成度和一条 Type 3 曲线（前篇 §3.5），正面空位。若自研者以中文文档处理为垂直锚点，这一项应排第一优先级。

### 5.9 Acrobat JavaScript 完整层 —— 机会 ★

**现状**：pdf.js 受限沙箱执行（已知 bug：注释跨页联动等）；pdfcraft 用 boa 做了 **forms 所需子集**（event/document 12 个方法/Field 17 属性/app/util/console），显式无文件/网络/定时器、无 annotations/layers/dialogs。完整的 AcroJS（文档级动作、页面动作、批量控制台、app 对象全量）无任何开源实现。

**为什么难**：对象面极大（Acrobat JS API 数百个方法）、安全面极大（历史漏洞库就是攻击面清单——pdfcraft 的 ADR-0009 因 Marvin 漏洞直接禁了浏览器端 RSA）、收益集中在"老表单自动计算"这一个场景。pdfcraft 的选择（只做 forms 子集）证明了这个域的正确姿势是**按需裁剪**。

**对自研者的含义**：跳过或按需。除非做表单自动化垂直，不值得为完整层付安全税。

### 5.10 qpdf 等价层：对象保持重写 + 线性化 —— 机会 ★★☆

**现状**：qpdf 本身开源免费且质量天花板，所以这不是"能力缺失"而是**生态位缺失**：需要 Rust 原生（嵌入 wasm、无 C++ 工具链、单一二进制分发）的场合，Rust 侧只有话术级新生军（flpdf/pdfrs 下载千级）。pdfcraft 的 cos 是最接近的实体（增量/全量写、确定性序列化、COW），但**线性化（Fast Web View）没做**（P1 planned）、Arlington 校验没做、且它是应用内 crate 而非独立库。lopdf 则带着 /Prev 损坏 bug 与无界递归 RUSTSEC 咬牙维持。

**为什么值得**：管线刚需（每个文档服务都要"打开-变换-保存不坏"）、验收现成（**qpdf 本身就是 oracle**：你写出的文件 qpdf --check 必须绿）、工程边界清晰（无 UI、无渲染、无语义）。

**对自研者的含义**：这是十个里**工程风险最低、最快可验证**的。单人自研若要"三个月有一个能打的东西"，从这里起步；pdfcraft 的 cos 设计（前篇 §2）可以直接当规格书读。

### 5.11 小型真空杂项箱

三个不足独立成节但值得记录的真空：**GPU 渲染**（zpdf wgpu 刚起步，hayro 明确 CPU-only；谁先做对"瓦片+设备"的 GPU 化谁拿高性能标签）；**JBIG2/JPEG2000/CCITT 编码器**（Rust 零实现，扫描件压缩管线的最后一里；JBIG2 编码器还带专利历史包袱，Debian 至今省略 OCRmyPDF 的 JBIG2 编码即因专利）；**语义级文档比较**（开源只有 pdftoppm+pixelmatch 像素 DIY，Acrobat Compare 产出语义差异报告；可与 5.3 的结构推断共享底座）。

---

## 6. 差异化决策框架：红海、蓝海与错峰带

把第 2/3/5 章的证据收敛成一张决策地图。横轴是竞争密度，纵轴是 pdfcraft 的占位深度：

```
                 pdfcraft 占位浅（<25% 或未排期）
                        │
   【蓝海·直做】        │        【蓝海·垂直】
   qpdf 等价层(Rust)    │   5.7 印前叠印模拟
   5.3 语义重建→Office  │   5.8 CJK 全链(中文市场)
   5.4 autotag+结构树   │   5.2 裁剪版渲染器(办公文档)
   5.1 PDF/A fixup      │   AI 本地文档问答(隐私向)
   5.5 PAdES LTV(库形态)│
                        │
 ───────────────────────┼─────────────────────────────
                 pdfcraft 占位深（>40% 或主峰在爬）
                        │
   【红海·绕开】        │        【错峰·跟随骨架单项深挖】
   通用查看器           │   签名链(它47%但有网络空白→补抓取层)
   文本提取/F1 竞赛     │   表单 appearance(它70%→做到 Acrobat 级)
   PDF→Markdown         │   优化器(它26%→"有效分辨率"口径可续写)
   图像转换 CLI         │   组织/拆分合并(qpdf 也在此→差异化在语义保持)
   从零生成 PDF         │
                        │
```

四条决策规则，按优先级排列：

1. **先垂直后通用**。上表蓝海右列的四个垂直项（印前/中文/办公/隐私 AI）各自锚定一个付费画像明确的窄市场；通用型项目（左列）则会同时面对 pdfcraft 的 5.5k 星与 qpdf 的 25 年——赢得概率不取决于代码质量而取决于你能不能活得比它们久。
2. **让 pdfcraft 当你的规格书，不当你的对手**。它的分层表、parity TOML、oracle 网、vendor 加固表都是公开的工程决策记录；错峰带项目应直接继承其验收口径（如"qpdf --check 必须绿""F1 对 pdftotext ≥0.98"），只在它 <25% 的山谷里做单项 100%。
3. **组合拳优先于单点**。无人区之间的咬合关系比单件深度更值钱：增量保存+LTV（法务归档闭环）、CJK 四件套（中文办公闭环）、autotag+语义重建+比较（政务无障碍闭环）、qpdf 等价+线性化+JBIG2 编码（管线基建闭环）。每一组都能构成"别人必须整链购买"的壁垒，而单点功能会被上游（qpdf/krilla/hayro）顺手覆盖。
4. **保真度测量本身就是差异项**。全生态（含 pdfcraft）的诚实会计都承认"与 Acrobat 未并排对比"。你的项目若从第一天就把**可量化的保真度基准**当产品能力（OmniDocBench/veraPDF/PDFBox 语料/PDFium 渲染对照），就等于在"人人都在说 shipped"的市场里唯一拿数字说话——这是零工程量的信任差异。

最后一条风险提示贯穿全部无人区：**每个真空格的成因（第 1 章四条）不会因为你的进入而消失**。知识壁垒项（preflight/ICC）要求你先成为半业内人士；语料护城河项（渲染/修复）要求你把 fuzz 当宗教仪式；信任运营项（LTV）的真正成本在代码之外。选差异项的本质是选你愿意补哪一类结构性成本。

---

## 7. 结语

开源 PDF 生态二十年建成了两根柱子：qpdf 代表的字节层与 MuPDF/PDFium/krilla 代表的像素层。柱子之间的"语义中间层"——编辑、修复、结构、信任、印前、CJK——至今无人驻守；pdfcraft 是第一个搬进来的住户，用净室纪律、机器计量与 oracle 网把这块空地圈出了施工图，但它自己的账本（effort 30–35%、五座主峰全部 <23%、保真度未测量、渲染靠 31 条未上游化补丁的借用）说明圈地远大于建成。

对自研者的最终判断因此可以一句话给出：**跟着 pdfcraft 的骨架建，选它没圈的地活**。骨架（分层表、parity 计量、oracle 网、限额族、fail-closed）是它验证过的工程文化，照抄不构成任何竞争；地（本篇十大无人区）才是差异化的全部来源——其中以中文市场锚定的 CJK 全链、以法规锚定的 autotag 与语义重建、以归档合规锚定的 PDF/A fixup 与 LTV、以管线锚定的 qpdf 等价层，是四条最陡峭也最空旷的谷地。前篇回答"怎么建"，本篇回答"建什么才值钱"；两份合读，决策图即完整。

---

## 附录 A：证据索引

**本地仓库证据**（/home/z/my-project/scripts/pdfcraft_research/repo，HEAD 7c35809）：

| 结论 | 出处 |
|---|---|
| parity 830 项 / 417 shipped / 146 单测试 / 外部 oracle 仅 4 条 | parity/acrobat-features.toml（逐项统计） |
| 诚实评估六维 / 里程碑 M0–M14 完成度 / 八条风险 / 23 项云排除 | ROADMAP.md |
| "Thin or missing" 六项自认短板 / 963/983 语料 / "What's next" | README.md |
| 34 条 vendor 补丁与上游未合并状态 | vendor/README.md |
| XFA"故意不做"清单 / 肉眼保真声明 | crates/xfa/README.md |
| AcroJS forms 子集覆盖面 / 无网络声明 | crates/js/src/lib.rs |
| PAdES B-B/B-LTA 结构与网络空白 / Marvin 禁令 | crates/sign/README.md、src/keys.rs |
| preflight 仅 A-2b/3b 子集 / 字体修复边界 | crates/preflight/src/lib.rs |
| OCR 仅英语 / 行级文本替换边界 | crates/ocr/src/lib.rs、crates/edit/src/text.rs |
| 34 个 Planned("Mxx") 工具目录条目 | crates/engine/src/catalog.rs |

**网络证据**（按主题，均为调研日 2026-10-09 可访问）：

- qpdf 定位与边界：github.com/qpdf/qpdf；pikepdf 无 appearance 重建：stackoverflow.com/questions/59992748
- pdf.js XFA/JS/内存边界：github.com/mozilla/pdf.js/issues/19926、experienceleague.adobe.com（XFA 显示策略）
- 许可证：mupdf.com/releases、artifex.com/licensing、pdfium.googlesource.com/LICENSE、ghostscript.com/releases/gpcldnld.html
- 签名链缺口：poppler 邮件列表（pdfsig 无吊销）、pdfa.org（LTV 定义）、kdecherf.com/blog/2024/04/29（AATL 信任）
- preflight 商业格局：reddit.com/r/CommercialPrinting（"callas/PitStop 唯一可信"）、docs.verapdf.org、pdfix.net
- 转换格局：github.com/ArtifexSoftware/pdf2docx（停维护公告）、github.com/pdf2htmlEX、ocrmypdf.readthedocs.io、callassoftware.com、OmniDocBench（github.com/opendatalab/OmniDocBench）、DocLayNet（IBM Research）
- Rust 生态：crates.io API 实测（lopdf/pdf-rs/pdf-writer/krilla/hayro 系/underskrift/pdfium-render/mupdf-rs/ocrs 下载量与版本）、github.com 观测（stars/最近提交）、hayro issue #1358（Lege 性能实测）
- Acrobat 2026 AI：blog.adobe.com（2026-09-09 与 2026-10-07 两篇）、helpx.adobe.com（What's new）

## 附录 B：无人区 × pdfcraft 路线对齐表

| 无人区 | pdfcraft 槽位/里程碑 | 其完成度 | 自研机会 |
|---|---|---|---|
| 5.1 PDF/A fixup / preflight | preflight crate + M11 | 18% | ★★★ |
| 5.2 自研渲染器 | render + M2（ADR-0004） | 15% | ★★（垂直裁剪） |
| 5.3 语义重建→Office | export + M10 | 启发式、xlsx/pptx planned | ★★★ |
| 5.4 autotag/结构树编辑 | a11y + M12 | 23%（面板全 planned） | ★★☆ |
| 5.5 PAdES LTV | sign + M9 | 47%（网络层空白） | ★★☆ |
| 5.6 XFA | xfa + M12 | 23%（静态子集） | ★（建议跳过） |
| 5.7 ICC/叠印预览 | color*（L2 预留）+ M11 | 0% | ★★（做模拟不做 CMM） |
| 5.8 CJK 全链 | fonts/edit/ocr 多点 | 46%/未排/planned | ★★★ |
| 5.9 AcroJS 完整层 | js + M6 | forms 子集 | ★ |
| 5.10 qpdf 等价+线性化 | cos + core.linearization | M1 78%、线性化 planned | ★★☆ |
