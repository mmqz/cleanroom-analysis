# XLSX 三能力宣称边界（t10-g）

> 定位：xlsx-r1 PLAN §2 t10-g——用户指令 2/5 的文档交付：三能力
> （读取/新建/修改已有）的**对外宣称口径**（每条带证据锚点）+ DOCX
> 独立模块边界声明。纪律：**未实测不宣称**；宣称语 = 本文件口径的
> 逐字引用，产品文档/README/对外描述不得超出。
> 证据链：t10-b 语料（manifest.md）· t10-c FIDELITY.md · t10-d
> read_proto 17/17 · T3 探针（office-e §8）· DEPS.md。

## 1. 能力宣称表

### C1 读取单元格数据（已验证，可宣称）

**宣称语**：读取 XLSX 工作簿的表结构（表名与顺序）、单元格值矩阵
（字符串/数字/布尔/日期/错误值，含公式缓存值）、公式文本、合并单元
格范围、**超链接**（target/location/displayed_text/tooltip 四字段，
v2 增量）。读取前经容器预扫门（声明尺寸求和/单条目/条目数/路径逃逸
四检查）+ 尺寸诚实性门（谎言头闭环）+ **部件良构性门（门 3，
xlsx-r2 Phase 2：畸形/无效 XML 部件不再静默变空/部分数据——默认
严格 typed reject；显式兼容模式降级为结构化警告随返回值携带）**，
敌意输入 typed reject 零 panic。

**v2 增量注记（2026-10-13 后半）**：超链接面经独立勘误后并入——
calamine 0.36.1 hyperlinks_by_sheet_name 完整可观测（v1 曾误记
「无公开 API」，作废）；rel 属性值的 XML 实体未反转义怪癖由 IO
边界统一补偿（消费方拿逻辑 URL，如 `...?q=1&x=2` 的真实 `&`）。

**证据**：
- 7 维语料正向 5 维 + 对照件全过（read_proto 17/17，锚点断言含
  表序/非空格数/公式数/合并数逐档对账）；
- 量级：T3 探针 50 万格 139ms、156,000 格零丢失全量对账；
- 敌意 11 件三分归类钉定（**Phase 2 后口径**：拒绝 5[门 1/2 四件
  + 门 3 mal7e] / 解析错误 5[mal7a 兼容模式下引擎仍错；严格模式
  下归拒绝族] / 容忍 1[mal7c——恒登记 ControlChars 结构化警告]）；
  兼容模式全语料复扫归类在档（变化仅门 3 两项，门 1/门 2 恒不变）；
- 超链接五形态往返（lib/io v2 测试 + mdcore 同构）：外链纯 / 外链
  `&` 查询（实体补偿核心用例）/ internal / tooltip / file 子锚
  （target+location 双字段）——5/5 零字段差异。

**边界（不宣称）**：
- 样式/格式面（calamine 无可观测 API——FIDELITY §4）；
- 图表/透视表/图片部件（未测，不在四 API 面）；
- 公式**求值**（只读缓存值与文本，无计算引擎——T4 威胁面随公式
  引擎立项）；
- docx/xlsx 以外格式（xls/xlsm/ods 未在本轮语料，calamine 支持
  面未验证不宣称）。

### C2 新建 XLSX（已验证，可宣称；**产品面已落地**——xlsx-r2 Phase 1，2026-10-14）

**宣称语**：从零新建 XLSX 工作簿：多工作表、字符串/数字/布尔值、
公式（含跨表引用与缓存值）、合并单元格、超链接（外部 URL 与内部
表引用）。大表流式写（constant_memory）。产品 API = mdcore
`crates/office-io/src/write.rs`（mdc-office-io，main e8c3a13）：
rust_xlsxwriter 包装，引擎类型零泄漏（公开签名只含本 crate 类型）；
输入验证前移（表名全规则/坐标边界/合并次序 → typed 错误）；流式
表行序纪律 typed 强制（引擎对已刷写行的回退写静默丢弃——源级一手
核验 worksheet.rs insert_cell，包装层拒绝，绝不静默丢数据）。

**证据**：
- 写读全环测试（tests/write_roundtrip.rs，6 测试，经产品读路径含
  容器双门）：五面全环（表序/值四型/公式+缓存/合并/超链接五形态）
  + 流式 2000 行计数对账 + 样式 XML 级 9/9 + 控制字符 XML 安全
  （全部件无裸控制字节 + _xHHHH_ 转义 + 值往返零失真）+ 读-改-重建
  路径证据 + save_to_vec 重开；
- 语料生成侧 = rust_xlsxwriter（历史 spike 写读互证，manifest 逐件
  锚点）；量级：100 万格 972ms / 3.33MB（T3 探针）；
- calamine 范围锚定语义钉定（tests/range_anchor_pinned.rs：值范围
  锚首格/公式范围锚首个公式格——跨面配对须绝对坐标，开发期实测
  捕获相对坐标错位的自洽假通过）；
- 门禁：G1 379/379（基线 362 + 新增 17）+ PDF 四面零波及 + clippy
  零新 + fmt 0 + scan 0 + wasm32 + web 全过（mdcore e8c3a13）。

**边界（不宣称）**：
- 样式写入正确性 = **XML 级验证**（style_probe 探针 9/9 + 产品 API
  复现 9/9：bold/FF0000 字色/FFFF00 填充/thin 边框/¥ 与百分比自定义
  numFmt/居中对齐/cellXfs 应用链；2026-10-13 闭项）；完整样式模型
  （条件格式/主题色/富文本内联样式）未验证不宣称；
- 数值型合并区左上格（引擎 merge_range 仅字符串面——类型扁平化
  如实登记）；
- 公式缓存值仅数值/文本/布尔三型（日期缓存不在面）；
- 修改已有工作簿（见 C3）。

### C3 修改已有工作簿（**不宣称支持**——结构性限制在档）

**宣称语（唯一允许的口径）**：不提供已有工作簿的编辑往返能力。
已知替代路径为「读-改-重建」（calamine 提取 → rust_xlsxwriter
从零新建），该路径的实测保真边界：**保** = 表序/值（含缓存值）/
公式文本/合并范围、**超链接（v2 勘误后并入：五形态 5/5 零字段差异，
rels 字节面与原档一致——FIDELITY.md v2）**；**丢** = 单元格样式与
格式、元数据字节保真（FIDELITY.md §4 丢失矩阵 v2，五档实测）。

**定性**：该丢失为 calamine（无**样式**可观测面）+ rust_xlsxwriter
（从零写器）组合的**结构性限制**，非实现缺陷、非尽力不足（重建
路径已重放读侧全部可观测面 = 理论上限）。v2 勘误把超链接移入保真
集后，风险面收窄为**样式保真**场景：含样式且需保真的已有工作簿
修改 = **数据丢失风险面**（用户指令 6 暂停条件之一）——此场景明确
告知用户不支持，不静默降级；含超链接的修改在尽力而为重建路径下
已可保真（超链接自定义格式仍属样式面，随样式丢）。

## 2. DOCX 独立模块边界声明（用户指令 5；**切片已落地——xlsx-r2
   Phase 3，2026-10-14**）

1. DOCX 读写为**独立模块**，与 XLSX 模块**不共用领域模型、不建统一
   Office 接口抽象**。**已落地**：mdc-office-io `docx` 模块（自有
   模型 DocxDocument/Block/RunData/TableData + 自有错误面 DocxError
   + docx-rs 类型零泄漏；引擎 = docx-rs 0.4.22）。
2. 现有 mdc-office crate 内的共享面仅限**消毒六面**（上下文级纯
   函数：CSV/XML 字符/XML 实体/HTML/域白名单——上下文语义与格式
   无关，csv_formula_guard 的 xlsx 路线豁免注记为形态差异的唯一
   例外），非领域模型耦合。
3. 不为「统一接口」预设计：容器三门（门 1/2/3）为 zip+OPC 同构的
   **基础设施共享**（非领域抽象）；工作簿/文档对象模型、IO 管线、
   错误面均按格式独立演进。
4. ADR-O1 O1-6（docx 渲染级读侧）维持开放裁决——Phase 3 切片为
   **模型级**（布局/分页/字体度量面外），渲染级未触碰。
5. **DOCX 切片能力宣称**（唯一真源：xlsx-r2/REPORT.md §6
   [Phase 3 基线] + docx-r1/REPORT.md [增量第一轮]；宣称纪律同
   本文件 §4）：
   - **Phase 3 基线（模型级）**：读 = 块序/样式 ID 透传/run 级文本
     加粗斜体/超链接内文本/表格单元格拼接（含容器三门与严格/兼容
     双模式）；写 = Heading1-9（outline 级）/富文本段落/文本表格；
   - **docx-r1 增量（2026-10-15，遗留①）**：读 = **超链接目标**
     （run 级 `HyperlinkTarget` 四变体：External[URL 原样透传永不
     解引用]/Internal[包内目标]/Anchor[w:anchor]/Unresolved[仅
     兼容模式]）+ **页眉页脚六槽位**（default/first/even × 页眉/
     页脚，变体透传 + 块序提取；页眉部件自身 rels 联接页眉内
     超链接）；写 = **外部超链接 run**（RunSpec.link——引擎关系面
     TargetMode="External"）+ **默认页眉页脚**（单段 run 序，
     重复调用 = 原位替换）；支撑层 = **自有 OPC 关系解析层**
     （quick-xml 解析全部 .rels 部件，门 3 同族纪律：上限恒等两
     模式、词汇表/schema 违反严格拒兼容警告、悬空引用
     BrokenReference 类）；
   - **docx-r2 增量（2026-10-16，遗留① 收口轮）**：读 = **容器级
     媒体清单** `DocxDocument.images`（word/media/* 枚举 + 自有
     [Content_Types].xml 解析 + 全 .rels 表 image 型关系联接 + run 级
     a:blip 引用计数——字节/内容类型/引用关系/body_uses 五字段；
     悬空关系/越根/悬空 embed 严格拒兼容 BrokenReference，未声明
     内容类型严格拒兼容警告）+ **[Content_Types].xml 词汇表纪律**
     （Types/Default/Override）；写 = **脚注**（RunSpec.footnote
     ——footnotes.xml 自动收集，marker 样式面忽略登记）+ **简单域**
     （RunSpec::field_page/field_num_pages——PAGE/NUMPAGES；HYPERLINK
     域 = 引擎 todo!() 崩溃面绝不暴露）+ **PNG 图片**（DocxWriter::
     image——r1 勘误：`Pic::new_with_dimensions` 不受 image 特性
     门控，自动转码面 +23 crate 仍登记）+ **first/even 页眉页脚**
     （四方法；激活语义引擎完整，同槽重复调用 = 原位替换保 rid
     ——r1 无条件 rIdHeader1 踩踏修复）；
   - 不支持面与数据损失在 mdcore docx.rs 模块文档逐项登记
     （docx-r1 变化：超链接目标丢弃**闭项**[表格单元格内目标
     仍丢——TableData 扁平面]；新增 WarningKind::BrokenReference
     共享警告类别——XLSX 面无使用点；
     docx-r2 变化：图片媒体面**闭项**[块序内图片位置仍登记]、
     first/even 写面**闭项**；脚注/域读面与其余域/图片自动转码
     维持登记——全链见 docx-r1/REPORT.md §6 + FIDELITY §7.1）；
   - **消费面（t16，2026-10-17——office-io 首个 wasm/web 消费边）**：
     **web 编辑器 .docx 导入已落地**（mdcore `crates/mdcore-wasm/src/
     docx_import.rs` + web 工具条 UI + `web/tests/docx_import.py`）。
     实测宣称口径：.docx 字节 → 容器三门兼容模式读 → DocxDocument →
     markdown（标题 Heading1-9[7-9 钳 6] / 加粗 / 斜体 / 超链接四变体
     [Unresolved 降级] / GFM 管道表）→ 既有 `__inject` 重建核 →
     可编辑活文档（e2e 实证：导入后派发编辑事务块数 +1、undo 键位
     契约成立）；敌意/截断档案 typed 拒绝（`container_*` 错误码面）
     且编辑器现状零触碰（e2e 实证）。不导入面计数显式化（图片/
     页眉页脚/脚注 → stats）——映射规则与已知边界唯一真源 =
     docx_import.rs 模块文档（docx-r1/REPORT.md §7）。**不宣称**：
     显示保真（结构映射非渲染语义）、样式语义解释（Title/自定义
     样式 → 普通段落）、图片/脚注/页眉页脚内容导入。

## 3. 容忍度发现登记（t10-d 实证；**Phase 2 后口径——两项已闭环**）

1. **0x00 读侧容忍**（mal7c，**维持**）：XML 1.0 非法控制字符在
   文本位被解析器容忍，值含 NUL 落内存——**读侧不过滤**；再输出
   时消毒链（xml_control_chars）为必经路径（SANITIZE_TESTS.md §8
   联动）；Phase 2 起恒登记 ControlChars 结构化警告（两模式同，
   mdcore `Book.warnings`）。
2. **垃圾部件静默降级**（mal7e，**已闭环**——xlsx-r2 Phase 2
   门 3）：非 XML 二进制部件不再静默变空表——默认严格 typed
   reject（`ReadError::InvalidPart`，部件名+诊断在错误内）；
   兼容模式降级为结构化警告 + 数据照常提取（引擎容忍行为不变，
   但消费方对畸形件有了明确信号）。同族闭环：截断（缺根闭合
   与标签内两种形态严格均拒；兼容模式保持 xlsx-r1 分界[缺根闭
   合→警告+值读回；标签内→警告后引擎 ParseError]）/ 多根 /
   空部件 / 超深嵌套（LIMIT_XML_DEPTH=1000）。
3. **谎言头分层**（mal6a）：声明 16B 实际 1MB 的条目在 calamine
   语义层先报错（缺部件），未触达 zip 读界拒绝路径——分层防线
   中语义层先拦；zip 读界行为以 8.6 对齐重测为准（DEPS.md §1）。

## 4. 宣称变更纪律

本文件为能力宣称的**唯一真源**：新宣称 = 先实测（语料/探针/丢失
矩阵）后写入；每条宣称带证据锚点；删除宣称同样留痕（能力回撤 =
行为变化，须带原因与替代口径）。~~阶段 B（mdcore 落地）后本文件
随 crate 文档同步（crate lib 文档的 Capabilities 节引用本文件）~~
**已接线（2026-10-13）**：mdc-office-io crate 根文档（mdcore main
880864e）的 Capabilities 节 = 本文件 §1 三能力宣称口径的引用摘要
（C1 已验证可宣称 / C2 写面验证态·产品面本轮不在直移面 / C3 不
宣称支持 + 丢失矩阵引用），容忍度发现三件随 crate 文档同步发布；
对外宣称语仍以本文件为唯一真源，crate 文档为引用摘要不得超出。
