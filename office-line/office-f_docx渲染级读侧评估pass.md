# docx 渲染级读侧评估 pass（office-f / R-O2）

> **性质**：ADR-O1 开放项 O1-6（docx 渲染级读侧：结构忠实 + 显式分页 +
> 按需渲染——立项与否、时机）的独立评估轮。用户指令：**独立评估，不混入
> 协同内核改造**（D1 线与本 pass 零交互）。证据先行、零终裁——产出选项
> 与裁决问题，立项决定留用户。
> **评估基线**：2026-10-17；mdcore main=00ad88b（模型级读侧现状 =
> docx-rs → 自有模型 → markdown，t16 链在产）；mdc-pdf 阅读面在产
> （PDF-c：pdfium + O(1) LRU + panic 隔离 + 性能门）。
> **净室纪律**：许可/活跃度全部一手直读（GitHub API + npm registry +
> crates.io + 官网）；转换实证 = 本机 spike（`office-line/spike-ro2/`，
> python stdlib 手搓 OOXML + soffice 子进程，零第三方依赖）；office-d
> §6.1/§6.2 已复审事实转引并标注。

## 0. 执行摘要

| 候选 | 许可 | 显式分页 | 结构忠实 | 按需渲染 | 集成形态 | 一句话判定 |
|------|------|---------|---------|---------|---------|-----------|
| **C 路线：LO headless → PDF → 既有 mdc-pdf 阅读面** | MPL-2.0（三许可仓，进程隔离先例在） | **真排版引擎**（spike 硬断言过） | 完整布局栈（字体度量/DrawingML/表格） | **继承 PDF 面**（pdfium 逐页 + LRU + 已有性能门） | 子进程 + 既有 crate 零新依赖 | **评估新增选项，spike 全过** |
| docx-preview（JS DOM） | Apache-2.0 ✓ | 容器裁剪非真排版（office-d §6.1） | 三面代差（分页/浮动/字体） | ✗ 全量 DOM 无虚拟化 | web 侧 npm | O1-6 原问的「JS 参照件」= 校准锚与过渡件 |
| mammoth（JS 语义） | BSD-2-Clause ✓ | ✗（语义 HTML 无页概念） | 语义级非视觉 | ✗ | npm | 不满足目标线（登记排除依据） |
| Univer OSS | Apache-2.0 ✓ | 自有排版栈在（engine-render docs/） | — | — | **OSS 无 docx 导入器**（30 处 docx 提及全为内部布局代码；导入 = Pro 面） | 渲染引擎可参照，导入桥自研 = 变相自研 |
| SuperDoc | **AGPL-3.0** | — | — | — | — | 许可排除（GPL/AGPL 先例） |
| OnlyOffice x2t/viewer | **AGPL-3.0**（office-b §3 在档） | — | — | — | — | 许可排除（同上） |
| 云查看器（MS/Google/WPS） | 专有 SaaS | — | — | — | 网络依赖 | local-first 定位排除（登记） |
| Rust 自研 | — | 按目标线建 | 按目标线建 | 按目标线建 | mdc-office-io 内 | **生态空位实证**（crates.io 全模型级/模板级，零渲染级）——O1-6 原问的「直接 Rust 自研」 |

**核心发现**：目标线三要素（结构忠实 + 显式分页 + 按需渲染）在开源生态
**无单点全满足件**——docx-preview 是结构近似的 DOM 渲染（分页容器化、
无虚拟化），真排版引擎全在编辑器级方案（AGPL/专有）或 LibreOffice。
**C 路线**（本 pass 新增论证）通过「转换 + 既有资产复用」同时满足三要素：
真分页（spike 硬断言）+ 完整布局栈 + 按需渲染零新建（mdc-pdf 在产），
代价 = 子进程运维 + 转换时延（热态中位 304ms/小文档）+ 保真上限受
LO 对 docx 的解释差异约束。

## 1. 需求锚定（评估基准）

**目标线**（office-d §6.3，已复审在档）："docx-preview 级结构忠实 +
三处超额（显式分页符全支持、wrapSquare 至少降级近似、嵌入字体还原）+
SheetJS 式数据骨架兜底，按需渲染解决大文档性能——**不做内容驱动重排版**
（分页只信文档显式标记）"。

三要素的操作化定义（本 pass 供验收用）：

1. **结构忠实**：标题层级/run 格式/表格/列表/图片在场且语义可辨（允许
   视觉近似，不允许静默丢弃——docx-preview 的 wrapSquare 静默丢弃即
   反例）；
2. **显式分页**：`<w:br w:type="page"/>` 与分节符必须产生真实页边界
   （页数可断言）；`lastRenderedPageBreak` 编辑器残留**不采信**
   （docx-preview 同款默认，office-d §6.1）；
3. **按需渲染**：首屏不随文档总页数线性劣化（逐页/视口级渲染）。

**消费场景登记**（决定三要素权重）：(a) 工作台内嵌只读预览（mdcore
生态的 docx 消费面——t16 导入链的渲染级补充）；(b) AI 生成态产物的
视觉校验（D3 面向，本轮零交互仅登记）；(c) 高保真打印/交付（PDF 出口
天然对齐）。**非目标**：docx 的渲染级**编辑**（那是编辑器级方案的成本
域，office-b §4 在档）。

## 2. 现状基线（mdcore 侧）

- **模型级读侧在产**：t16 链（.docx → 三门 → docx-rs 模型 → 自有模型
  → markdown → 编辑器），语义保真面由 docx-r1/r2 FIDELITY 矩阵治理
  （24 项五态）。该链回答"内容是什么"，**不回答"看起来什么样"**——
  分页/字体/浮动对象/精确布局全不在面。
- **mdc-pdf 阅读面在产**（PDF-c/d）：pdfium 渲染 + O(1) LRU 页缓存 +
  panic 隔离工人 + 性能门（打开 0.18ms/翻页缓存 400×）+ 标注面（lopdf
  增量写，Acrobat 互通预证）。**该面对任意合法 PDF 通用**——是 C 路线
  的既有资产。
- **R-O2 的空位**即两者之间：docx 的"渲染级视图"。ADR-O1 O1-6 当时
  空位登记的理由（"先模型级满足生成态校验往返，渲染级立项单议"）在
  模型级链稳定后已到评估时点。

## 3. 候选盘点（一手核验，2026-10-17）

### 3.1 C 路线：LibreOffice headless 转换 + mdc-pdf 阅读面

**对象锚定**：
- LibreOffice 上游稳定版 **26.8.1**（2026-09-21 发布，官网 whatsnew +
  release-notes 交叉；版本制 year.month）；本机 spike 用 Debian 13 打包
  **25.2.3.2**（版本差登记——实施轮应按 26.x 校准复测）。
- 仓 `LibreOffice/core`：活跃（pushed 2026-10-16）；**三许可仓**——
  根目录 COPYING（GPL-3.0）/COPYING.LGPL/COPYING.MPL 并存（GitHub API
  license 字段检为 GPL-3.0 即因此）；README 自述 open source。逐文件
  许可混合（MPL-2.0 为主，历史 Sun 代码三许可）。

**许可判定**（进程隔离先例）：**子进程调用**（`soffice --headless
--convert-to pdf`）不构成代码链接、不产生衍生作品义务——与 ComfyUI
（GPL-3.0，进程外协议集成唯一路线，canvas-d 定谳先例）同一法理。**若
走嵌入/链接（UNO API 进程内）则进入 MPL-2.0 义务面**（文件级 notices
与修改披露）——本评估只主张子进程形态。分发包再分发（若随产品分发
LibreOffice 本体）另涉 MPL 源码义务——**实施形态应优先"宿主环境预装
或容器镜像"而非随产品再分发**（登记为实施轮供应链决策点）。

**spike 实证**（`office-line/spike-ro2/lo_convert_spike.py`，本机
release 口径，python stdlib 手搓 OOXML 三样本 + mdcore t16 真实夹具）：

| 样本 | 断言面 | 结果 |
|------|--------|------|
| A 结构（Heading1/2 + 粗斜混排 + 2×3 表 + 1 显式分页符） | 8 关键串全在场（标题/run 文本/表格单元格/分页后段落）+ 页数恰 2 | **全过**（34,224 B PDF） |
| B 分页（5 显式分页符） | **页数恰 6（硬断言）** + 6 页文本各在场 | **全过**——显式分页保真定谳（目标线要素②） |
| C 真实夹具（import_sample.docx） | 转换成功 + 文本抽取 | 过（1 页，166 字符） |
| 延迟 | 冷启动 1,024 ms；热态 3 次中位 **304 ms**/小文档 | 登记（大文档/并发口径待实施轮基准） |

**保真上限（诚实登记）**：LO 对 docx 的解释 ≠ Word 原生渲染（布局引擎
差异——字体替换/域计算/复杂表格度量等场景存在已知偏差）；本 spike 证明
的是**结构/显式分页/文本层**忠实，视觉级逐像素对齐不主张。**系统性
保真校验 = 实施轮必做**（用 docx-r1/r2 语料 + FIDELITY 矩阵法：逐面
比对 + 降级显式化，方法已在 xlsx-r1 C3 矩阵验证过一轮）。

**安全面**：输入先过 mdc-office 消毒五面（在产）→ LO 转换（子进程，
崩溃隔离天然）→ 产物 PDF 过 mdc-pdf 阅读面安全策略（在产）。敌意
docx 在 LO 面的崩溃/资源面 = 子进程重启成本，不波及引擎进程——与
ComfyUI 同构。**LO 子进程自身的资源限额**（内存/超时/并发数）=
实施轮 SECURITY_POLICY 增量项。

### 3.2 docx-preview（O1-6 原问的「JS 参照件」）

一手核验：仓 `VolodymyrBaydalka/docxjs`，**Apache-2.0**（GitHub API +
npm 双确认），2,121 stars，活跃（2026-09-21 push + 同日发版 0.4.1；
npm 包 `docx-preview` 创建 2018-03）。

能力上限（office-d §6.1 已复审事实转引，锚点在档）：三种分页源（手动
分页符/分节变化/`lastRenderedPageBreak` 默认忽略）；**内容驱动自动分页
官方明确不做**（README:101 性能理由原文在档）；页容器 `overflow:hidden`
= 超页内容裁剪；wrapSquare/wrapTight/wrapThrough **无 case 静默丢弃**；
图表/形状/SmartArt 不渲染；嵌入字体可还原（odttf 反混淆）；**全量 DOM
无虚拟化**（大文档线性劣化）。

**判定**：满足"结构近似 + 显式分页符"，不满足"按需渲染"（要素③），
"结构忠实"三面代差（office-d 与 OnlyOffice 对照定级在档）。作为
**过渡件/校准锚**价值在（office-d §6.3 已用它定义目标线），作为终态
渲染级读侧不满足。

### 3.3 mammoth（语义 HTML）

一手核验：BSD-2-Clause（repo + npm 双确认），活跃（2026-09-26 发布
1.13.0）。docx→语义 HTML（无样式无分页）——设计目标即语义转换非视觉
渲染。**不满足目标线要素①②**（分页无概念、视觉结构不保），且该生态位
已被自有 t16 链覆盖（docx→markdown 语义转换在产）。登记排除。

### 3.4 Univer OSS（渲染引擎在、导入桥不在）

一手核验：Apache-2.0，22,642 stars，极活跃（pushed 2026-10-16）。
README 自述 "Spreadsheets, Docs, Slides, Canvas, Relational Tables,
and PDF in one runtime"——**docs 产品面存在**：packages/ 下 docs 引擎
族 11 包（docs/docs-ui/docs-toc/docs-hyper-link/engine-render docs
布局[shaping/linebreaking/table] 等）= 自有排版栈。

**关键缺口（code search 一手）**：全仓 "docx" 30 处提及**全部是内部
布局/数据模型代码**（update-document-section/text-x utils 等），**无
docx 导入器**（importer/docxImporter/importDocx 三个搜索式均 0 命中）。
Univer 的 Office 格式导入导出在其商业面（Pro/API 层，office-a §5/§7
在档：Univer Pro 的协同/导入服务）。**判定**：采用 Univer 渲染 docx =
自建 docx→Univer 模型导入桥 = 变相自研解析面 + 引入整套运行时（60 包
monorepo）——相较 C 路线（复用在产 mdc-pdf）与 Rust 自研（可控面），
该组合无净收益。其 docs 排版栈（shaping/linebreaking 源码 Apache-2.0）
**可作为 Rust 自研路线的算法参照**（许可宽松可读可借鉴）——登记为
B 路线资产。

### 3.5 许可排除面与云服务面

- **SuperDoc**（npm 2.24.0）：AGPL-3.0（npm license 字段一手）——排除
  （GPL/AGPL 净室先例，collab-a/canvas-d 同口径）。
- **OnlyOffice x2t/viewer**：AGPL-3.0（office-b §3 在档）——排除。
- **云查看器**（Microsoft Office Online viewer / Google Docs viewer /
  WPS WebOffice）：SaaS 网络依赖 + 专有许可——与工作台 local-first
  定位（06 §1 判据 2）冲突，登记排除。
- **pandoc**：GPL-2.0+（且为 docx→HTML 语义面，与 mammoth 同生态位
  被自有链覆盖）——双理由排除，登记不展开。

### 3.6 Rust 生态空位实证（O1-6 原问「直接 Rust 自研」的供给侧事实）

crates.io 搜索 "docx"（2026-10-17，一手 API）：Top 结果全为模型级/
模板级——`docx` 1.1.2（解析+生成）、`zavora-docx`（读写转换）、
`docxtplrs`（模板）、`ferrodoc-docx`（pandoc AST 读）、MCP 族四件——
**零渲染级 crate**（渲染级 = 分页布局 + 字体度量 + 绘制）。与 office-e
（Rust 生态选型 pass）结论一致：Rust 侧渲染级读侧**必须自研**（或走
C 路线绕开）。

## 4. 路线对照与裁决问题

### 4.1 三路线总对照

| 维度 | **C：LO→PDF→mdc-pdf** | A'：docx-preview 过渡 | B：Rust 自研（目标线） |
|------|----------------------|----------------------|------------------------|
| 三要素满足 | ✓✓✓（spike 证①②，③继承 PDF 面） | ~✓✗✗（结构近似/容器分页/无虚拟化） | 按投入定（目标线即规格） |
| 新增代码面 | 转换编排 + LO 进程治理（最小） | web 侧集成（零 Rust） | mdc-office-io 渲染层（最大） |
| 新增依赖 | **零 crate**（soffice 为环境依赖） | npm 1 件（Apache-2.0） | 零三方（或字体度量件——实施轮评估） |
| 保真上限 | LO 布局栈（高但非 Word 原生） | DOM/CSS 近似（三面代差） | 目标线定义（不含内容重排） |
| 运维面 | **子进程生命周期/限额/沙箱** | 无 | 无 |
| 时延 | 冷 1s/热 ~300ms（小文档）+ PDF 面打开 0.18ms | 首帧全量解析 | 实施定 |
| 文本语义 | PDF 文本层（可抽取，非 DOM 语义树） | DOM（可交互/CSS 定制） | 自有模型（最强） |
| 可撤回性 | 高（拆转换编排即可） | 高 | 结构性（资产沉淀） |
| 与标注面互通 | **直接**（PDF 标注面在产） | 需自建 | 需自建 |

### 4.2 裁决问题（呈用户，零终裁）

**Q1 立项与否**：渲染级读侧是否立项（O1-6 原问）——若工作台近期消费
场景 (a)（内嵌只读预览）有真实需求则立项；(b)(c) 场景单独立项口径。

**Q2 路线与时机**（若立项）：
- **选项 1（C 先行）**：LO→PDF→mdc-pdf 起步——三要素即时满足、零新
  crate、标注面直接互通；接受子进程运维 + LO 解释差异（系统性保真
  校验实施轮做）。**本 pass 证据推荐此为起步线**。
- **选项 2（A' 过渡）**：web 侧 docx-preview 内嵌（零 Rust 面即时可用）
  ——接受三面代差与无虚拟化；作为 C/B 未就绪期的过渡。
- **选项 3（B 自研）**：Rust 渲染层按目标线建（结构忠实 + 显式分页 +
  数据骨架兜底 + 逐页渲染）——投资最大、文本语义最强、无运维面；
  Univer docs 排版栈源码（Apache-2.0）可作算法参照。
- **组合序**（登记供参考）：1 → 视保真校验结果决定是否追加 3（C 作
  兜底、B 逐面替代）；2 仅在 1 的子进程形态受阻时启用。

**Q3 C 路线实施形态**（若选 1）：LO 分发形态（宿主预装 vs 容器镜像 vs
随产品再分发[MPL 义务面]）；LO 版本基线（26.x 校准 vs 发行版打包）；
子进程治理参数（并发/内存/超时——SECURITY_POLICY 增量轮）。

### 4.3 与在档裁决的联动（登记）

- **D3 生成态**（06 §8.1，零交互仅登记）：生成态产物的视觉校验出口
  与 C 路线天然同构（docx→PDF 已是生成态校验链候选）——D3 裁决时
  本 pass 结论可直引。
- **锚点写面三问**（r5-t2 §13.4）：C 路线 PDF 面的书签/链接面与 docx
  书签的映射属 yrs-b R4.5 联动面——三线裁决互不阻塞，登记即可。
- **PDF 线资产**：mdc-pdf 阅读面的性能门与安全策略直接复用，PDF-e
  收口的 SBOM/NOTICES 面无新增（零新 crate）。

## 5. 遗留与边界（登记）

1. spike 为小文档口径（<40KB docx）——大文档/并发转换的延迟与资源
   面待实施轮基准（对齐 bench_limits 模式）；
2. LO 25.2.3.2（本机）与上游 26.8.1 的版本差——实施轮按 26.x 复测
   spike 断言集（脚本可复跑，环境变量化口径已具备）；
3. LO 转换的敌意输入面（畸形 docx 触发 LO 崩溃/挂起）——消毒五面前置
   后残余面 + 子进程超时治理 = 实施轮安全增量；
4. PDF 文本层的中日韩抽取质量（pdftotext 口径 vs pdfium 口径差异）
   ——影响"文本语义"维度评价，实施轮以 mdc-pdf 实测；
5. B 路线（Rust 自研）的工作量估算未做（O1-6 若裁 B 需先立估算轮——
   office-d §6.3 目标线即规格起点）。

## 6. 已核带源 / 待核清单

**已核带源（一手）**：
- docx-preview：GitHub API（Apache-2.0/2121★/pushed 2026-09-21）+
  npm registry（0.4.1 同日发布）；能力面转引 office-d §6.1（锚点在档）
- mammoth：GitHub API + npm（BSD-2-Clause/1.13.0/2026-09-26）
- LibreOffice：仓 API（三 COPYING 文件直读：GPL-3.0+LGPL+MPL 并存）+
  官网 whatsnew/release-notes（26.8.1/2026-09-21）+ 本机 soffice
  25.2.3.2 实测
- Univer：仓 API + contents/code search（docs 11 包在、docx 导入器
  0 命中——两轮搜索式）
- SuperDoc/OnlyOffice/pandoc/云查看器：npm license 字段 / 在档
  office-b §3 / 官网（一手或已复审转引，逐件标注）
- crates.io "docx" 搜索（2026-10-17 API，10 件全列）
- spike 全部数字：`office-line/spike-ro2/`（脚本 + out/ 产物可复跑复验）

**待核（登记）**：§5 五项遗留；LO 逐文件许可构成（若未来走嵌入/UNO
形态才需逐文件口径——子进程形态无需）。

## 附录：执行纪律对账

- mdcore 零接触零提交（本 pass 只读 web/tests/fixtures 一件二进制夹具，
  复制到分析仓 spike 目录使用）；
- spike 零第三方依赖（python stdlib zipfile 手搓 OOXML；soffice/
  pdftotext/pdfinfo 为环境工具）；
- D1/collab 线零交互（用户指令遵守：独立评估）；
- 与 O1-6 原问的差异显式化：新增 C 路线选项（原问只有「JS 参照件过渡
  vs 直接 Rust 自研」两肢）——依据 = mdc-pdf 阅读面在产资产 + spike
  实证。
