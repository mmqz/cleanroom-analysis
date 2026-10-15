# 已有工作簿修改最小技术验证（t10-c FIDELITY v2）

> xlsx-r1 PLAN §1 C3 设计执行结果。方法：calamine 全量提取（值/
> 公式文本/合并/超链接 = 全部可观测面）→ 首表 (8,8) 写 `R1-MODIFIED` →
> rust_xlsxwriter 从零新建重放 → 与原档双重对比（条目级 + 语义级）。
> 此路径（尽力而为重放）已是 calamine + rust_xlsxwriter 组合的
> 理论上限。运行产物：`spike/fixtures/fidelity_report.md`（本地，
> .gitignore）；证据 = fidelity.rs 源 + 本文件矩阵。
>
> **版本链**：v1（超链接误记为丢失维度，git 历史 e23efa9 在档）→
> 本 v2 勘误定谳版——超链接移入保真集（§0 勘误），样式/元数据
> 维持丢失面；v2 实证 = 本会话重跑（fidelity.rs v2 + 语料 dim5
> 五链接形态扩展）。

## 0. v1 → v2 勘误记录（证据链）

| 项 | v1 断言 | v2 实证 | 定谳依据 |
|----|---------|---------|----------|
| 超链接可观测性 | 「calamine 无公开 API」→ 登记为预期丢失维度 | **误判**：calamine 0.36.1 `hyperlinks_by_sheet_name` 完整可观测（target/location/displayed_text/tooltip/range 五字段） | xlsx/mod.rs:2190 API + Hyperlink 结构体（xlsx/mod.rs:2292）；上一会话 v1 manifest 断言作废 |
| dim5 rels 部件 | 条目级**丢失** | 条目级**变化**（部件恢复，字节级 rId 差异） | v2 重放后 rels Target 字节面与原档一致（单转义 `&amp;`，python 双向核验） |
| 语料 dim5 覆盖 | 3 种链接形态 | 5 种（+tooltip 面 + file 子锚外部工作簿引用——补齐 PLAN 规格） | corpus.rs dim5 v2 + 断言 |

v2 过程发现并补偿一个**calamine 0.36.1 读侧怪癖**（源级定谳）：
hyperlink rels `Target` 属性值**未做 XML 实体反转义**——
`unescape_xml`（utils.rs:117）只处理 Excel `_x00HH_` 约定（非
`_x00` 前缀原样返回），5 个 XML 实体不解码，`&` 以 `&amp;` 字面量
读出。补偿：回放前做 5 实体反转义（fidelity.rs `xml_unescape5`，
`&amp;` 最后替换序），重建字节面与原档一致。边界登记：真含字面
`&amp;` 文本的 URL 在此补偿下有双解歧义（不可分，实际概率≈0）。

## 1. 语义级结果（五维逐面，实测数字）

| 面 | dim1 | dim2 | dim3 | dim4 | dim5 | 判定 |
|----|------|------|------|------|------|------|
| 表序 | 保 | 保 | 保 | 保 | 保 | **全保** |
| 值面（非空格） | 9/9 | 6/6 | 4/4 | 13/13 | 7/7 | **全保（0 差异）** |
| 公式文本 | 4/4 | 无公式 | 无公式 | 2/2 | 无公式 | **全保（0 差异）** |
| 合并范围 | 0 | 0 | 3/3 | 0 | 0 | **全保** |
| 超链接（逐字段） | 0 | 0 | 0 | 0 | **5/5（0 字段差异）** | **全保（v2 修正）** |
| 修改格 (8,8) | 在位 | 在位 | 在位 | 在位 | 在位 | **全保** |

dim5 五链接形态全覆盖实证：外链纯形态（值面=URL）/ 外链自定义
文本（write_url_with_text 语义）/ 内部链接（`internal:` → location）/
tooltip 面 / file 子锚（target+location 双字段，`file#anchor` 还原）。

读侧行为登记（manifest 实测，calamine 0.36.1）：
- 公式文本不带 `=` 前缀；`CONCAT` 存储为 `_xlfn.CONCAT`（往返前缀
  一致，0 文本差异）。
- 无缓存值公式读回 `Float(0.0)`——与真值 0 不可区分；重放统一按
  `"0"` 缓存值落盘，两义性如实登记（读侧库行为边界，非缺陷）。
- 超链接值面：write_url 格值面=URL 本体；write_url_with_text 格
  值面=自定义文本。回放形态判定 = 值面驱动（值面≠URL → 文本形态）。

## 2. 条目级变化全解释（逐项机械归因，无未解释差异）

| 变化 | 出现档 | 机械解释 |
|------|--------|----------|
| `+xl/sharedStrings.xml`（新增） | dim1/dim4 | 修改标记 `R1-MODIFIED` 是字符串格；rust_xlsxwriter **零字符串格时整部件省略**，有字符串格时生成。原 dim1/dim4 无任何字符串格（纯数字+公式），重建引入 1 个字符串格 → 部件出现（内容恰为 `R1-MODIFIED` 单条）。**非策略漂移，是修改本身的副产物** |
| `docProps/core.xml`（变化） | 全部 | 创建时间戳不同（两档不同时刻生成） |
| `[Content_Types].xml`（变化） | dim1/dim4 | 新增 sharedStrings 部件的类型声明 |
| `xl/_rels/workbook.xml.rels`（变化） | dim1/dim4 | sharedStrings 部件的关系条目 |
| `xl/styles.xml`（变化） | dim2/dim3/dim5 | 原档样式不可观测 → 重建只有 rust_xlsxwriter 默认样式；dim1/dim4 无自定义样式故 styles 相同——**样式丢失的条目级实证** |
| `xl/sharedStrings.xml`（变化） | dim2/dim3/dim5 | 字符串表内容变化（`+R1-MODIFIED` 条目） |
| `xl/worksheets/sheetN.xml`（变化） | 全部 | dimension ref 扩展（修改格 (8,8) 引起）+ 新增格 + 超链接归属重排 |
| `xl/worksheets/_rels/sheet1.xml.rels`（变化，v1 曾为丢失） | dim5 | v2 超链接重放后 rels 部件恢复——Target 字节面与原档一致（单转义）；"变化"= rId 序/部件内条目排列的字节级差异，**语义级 0 丢失**（5 链接 0 字段差异） |

## 3. 丢失矩阵（C3 核心交付）

| 面 | calamine 可观测 | 尽力而为重建保留 | 判定 |
|----|----------------|------------------|------|
| 值（含公式缓存值） | 是 | 是（0 差异实测） | **保** |
| 公式文本 | 是 | 是（`_xlfn` 前缀往返一致） | **保** |
| 合并范围 | 是 | 是（3/3） | **保** |
| 超链接（target/location/文本/tooltip） | 是（v2 修正） | 是（5/5，0 字段差异；含实体补偿） | **保** |
| 表序/表名 | 是 | 是 | **保** |
| 修改写入 | — | 是（五档全在位） | **保** |
| 样式（bold/字色/填充/边框/数字格式/对齐；含超链接自定义格式） | 否 | 否（styles.xml 实证变化；超链接重建用默认链接样式非原自定义格式） | **丢** |
| 文档属性（创建/修改时间戳） | 否 | 否（core.xml 实证变化） | **丢** |
| 主题部件内容 | 否（部件字节透传但语义不可见） | 部分（未涉及时字节相同；涉及即丢） | 条件保 |
| 列宽/行高/冻结窗格/打印设置/命名区域 | 否 | 否 | **丢**（未单独构造语料；按机制推定，标注【推断】） |
| **宏（vbaProject.bin）**（2026-10-14 裁决补记） | 否（office-io 读路径未调用 calamine 潜在能力 `vba_project()`——calamine 0.36.1 xlsx/mod.rs:2577 引擎面存在，我们的 API 面不观测） | 否 | **丢**【推断】（不可观测即不可重放；二进制部件字节亦不透传） |
| **图表（chart 部件 + 图表 sheet 内容）**（2026-10-14 裁决补记） | 否（calamine 无图表数据 API——仅 sheet 类型枚举含 chartsheet（lib.rs:254），chartN.xml 部件不进模型） | 否 | **丢**【推断】（部件不可观测即不可重放；重建产物无 chart 部件） |

## 4. 定性结论

1. **组合语义 = 重建（rebuild），非修改（in-place edit）**：实证成立。
   「读 → 改一格 → 写」的真实形状是「读出全部可观测面 → 从零新建
   整档 → 全档重写落盘」。不存在任何"未触及字节保持"的修改路径。
2. **尽力而为上限**：值/公式文本/缓存值/合并/超链接五面在重放下
   全保（0 差异实测）——这已是不引入第三写侧件的前提下该组合的
   极限。**v2 把超链接从丢失面移入保留面**（v1 误判勘误 + 实体
   怪癖补偿后）。
3. **必然丢失集**：一切 calamine 不可观测面。语料已实证：样式、
   文档元数据；机制上推定同丢（未单独构造）：列宽/行高/冻结/
   打印/命名区域；**宏（vbaProject.bin）/图表（chart 部件）同理
   必丢（2026-10-14 裁决补记）**——calamine 对宏有引擎级潜在能力
   （`vba_project()`，未被我们的 API 面调用），对图表无任何数据面；
   两者在读—改—重建路径中既不可观测也不可重放，重建产物必无
   vbaProject.bin 与 chart 部件。丢失不是"可能"，是"必然"——
   不可观测即不可重放。
4. **C3 宣称边界（PLAN 指令 2 落地）**：
   - calamine 与 rust_xlsxwriter **不得被描述为具备已有工作簿可靠
     编辑往返能力**——本轮以丢失矩阵实证该结论（样式/元数据面必丢）。
   - 允许的宣称形态（见 CAPABILITIES.md C3 条目）：「已有工作簿
     修改 = 尽力而为重建路径：值/公式/合并/超链接可保，样式/元数据
     必丢；适用于可接受降维重写的场景，不适用于样式保真需求」。
5. **数据丢失风险**：对"编辑已有工作簿"需求为真实风险（修改即
   降维：样式/元数据）；对"读 + 从零新建"两能力（C1/C2）无此
   风险。本轮按指令如实登记，不构成暂停条件（验证目标恰是弄清
   此边界）。
6. **后续选项登记（2026-10-14 用户裁决落定）**：样式保真编辑需求
   **维持读—改—重建边界，不引入 umya-spreadsheet，不自研写侧**；
   样式/宏/图表等保真限制以本矩阵为唯一口径显式登记。若未来真实
   业务出现样式保真编辑需求，按 ADR 纪律重新裁决另议。

## 5. 复现

```
cd office-line/spike
cargo run --bin corpus     # 语料（17 件 + manifest；dim5 v2 五链接形态）
cargo run --bin fidelity   # 本验证 v2 → fixtures/fidelity_report.md
```

fidelity.rs 中修改标记位 `(8,8)`、回放策略（公式 `=` 前缀还原 /
`_xlfn` 前缀透传 / 无缓存值按 `"0"` 重放 / 超链接值面驱动形态判定 +
`xml_unescape5` 实体补偿）均以常量与注释在源内登记。

## 6. 产品级重放测试注记（xlsx-r2 Phase 1，2026-10-14）

本验证的回放策略已产品化复现：mdcore `crates/office-io/tests/
write_roundtrip.rs` 的 `read_modify_rebuild_path_evidence` 测试（经
产品写 API [`write`] + 产品读路径含容器双门）——五面保真对账（表序/
值/公式文本/合并/超链接逐字段）+ 修改格在位，与本文件 §1 矩阵一致。
开发期捕获并修复一处回放实现陷阱（登记）：**calamine 范围锚定语义**
（值范围锚首格 / 公式范围锚首个公式格——tests/range_anchor_pinned.rs
钉定）——跨面配对必须绝对坐标；相对坐标会产生整体错位的**自洽假
通过**（值面按位缓存保留 + 公式列均匀平移相对重锚，两面对账均不可
见）。spike fidelity.rs 的 abs 键策略（§4 复现源）从一开始就正确。

## 7. DOCX 切片数据损失登记变化（docx-r1 增量第一轮，2026-10-15）

本节登记 DOCX 切片（Phase 3 落地，xlsx-r2/REPORT §6）的数据损失
面变化；完整登记表 = mdcore docx.rs 模块文档（唯一真源随码走）。

**闭项（本轮）**：
- **超链接目标丢弃**（Phase 3 登记「超链接内文本保留、URL 丢弃」）
  → **闭项**：自有 OPC 关系解析层（word/_rels/document.xml.rels +
  per-part rels）承担 r:id → target 联接（引擎只透传 r:id——
  docx-r1/CAPABILITY_NOTES §1），run 级 `HyperlinkTarget` 四变体
  暴露（External/Internal/Anchor/Unresolved[仅兼容]）；写侧
  `RunSpec::link` 外部 URL 全环往返（含 `&` 实体补偿链：写侧引擎
  单次转义 + 读侧单次反转义 = 逻辑值复原，实测钉定）。

**仍登记（不变/更新）**：
- 表格单元格内超链接目标仍丢（TableData 扁平文本面——单元格文本
  保留；悬空引用仍按严格/兼容纪律处置）；
- 页眉/页脚内容丢失闭项（六槽位读暴露 + default 写暴露）；first/even
  槽位**写面**与页眉内表格写面仍登记（引擎在，未实测不暴露）；
- 脚注/尾注读面 = 引擎受限登记（docx-rs 从不读 footnotes.xml），
  写面引擎在未暴露；图片/域/修订/批注/编号语义维持 Phase 3 登记。

**新可见性（非损失，登记口径）**：悬空引用（超链接 r:id 无关系/
页眉页脚引用无条目/目标部件缺失/目标路径越出包根）严格模式 typed
拒、兼容模式 BrokenReference 结构化警告——Phase 3 的引擎静默跳过
（read_docx filter_map None）在两模式下均有了显式信号。

### 7.1 docx-r2 增量变化（2026-10-16 / t14——遗留① 收口轮）

**闭项（本轮）**：
- **图片媒体面丢失**（Phase 3 登记「图形/图片——模型级切片不含媒体面」）
  → **闭项（读侧）**：容器级媒体清单 `DocxDocument.images`（字节/
  内容类型/引用关系/run 级引用计数——body_uses；引擎面三个静默
  怪癖由自有实现绕开，docx-r1/CAPABILITY_NOTES §10.2）。**块序内
  图片位置仍登记**（Block 无图片块——模型级文本提取边界）；
- **first/even 页眉页脚写面**（r1 登记）→ **闭项**：四方法暴露
  （激活语义引擎完整：first 自动 titlePg / even 自动
  evenAndOddHeaders；同槽重复调用 = 原位替换保 rid——r1 踩踏修复）。

**新暴露（损失面收窄）**：
- 脚注写面（RunSpec.footnote）——脚注内容发射进 footnotes.xml；
  **读面维持引擎受限登记**（读回文本不含脚注内容，模型级钉定）；
- 简单域写面（PAGE/NUMPAGES）——域结果文本不入读回文本（域值 =
  渲染期计算面，诚实登记非损失）。

**仍登记（不变/更新）**：
- 表格单元格内超链接目标仍丢（TableData 扁平面）；
- 脚注/尾注**读面**仍引擎受限；图片自动转码（image 特性 +23 crate）
  仍登记；TOC/PAGEREF 域与 HYPERLINK 域（todo!() 崩溃面）仍登记；
- 图片**块序内位置**仍登记（清单面已闭）；
- 域读面（instrText/fldChar 提取）仍登记（结构面在，提取面外）。

**新可见性（非损失）**：媒体部件无内容类型声明（严格拒/兼容警告 +
content_type=None）；悬空 image 关系与悬空 r:embed（严格拒/兼容
BrokenReference）；zip 重复条目名 last-wins 语义钉定。

## 8. XLSX 类型规则（r4-t1，2026-10-17——Round 4 T1 用户裁决落地）

> 定位：XLSX 单元格**类型推断与转换规则**的依据表与边界登记。
> 用户裁决方向（2026-10-17）：默认保留标识符/前导零/数字文本不擅自
> 转数字；日期转换必须有明确依据；公式读取结果与公式文本区分；
> 不支持类型给可诊断结果。产品面 = mdcore office-io `io.rs`
> CellData/CellGrid/SheetData（r4-t1 提交）；实测锚点 =
> office-io tests/cell_types.rs（9 测试）；规则表镜像 =
> io.rs CellData 文档（单一真源随码走）。

### 8.1 类型规则依据表（每条规则一行：规则 / 依据 / 测试锚点）

| # | 规则 | 依据（一手核验 calamine 0.36.1 源码 + 探针实测） | 测试锚点（tests/cell_types.rs） |
|---|------|------|------|
| R1 | **前导零/数字文本/标识符 = Text 原样，绝不转数字** | 存储类型 = 字符串恒读 `Data::String`（cells_reader.rs read_v t="s"/t="str"/inlineStr/无 t 非数字回退）——存储面无转换发生，本层零新增推断 | `data_face_seven_kinds_pinned` B1；`kind_preserving_replay_roundtrip` B1 前导零往返 |
| R2 | **数值 = Number（f64）**，显示面 = Rust 格式化 | xlsx 数值存储面恒 IEEE 754 double（read_v 数字路径只产 `DataRef::Float`；`Data::Int` 仅 pivot 路径不可达，xlsx/mod.rs:3621） | A1（123→"123"）；K1（0.1+0.2 → "0.30000000000000004" 最短往返） |
| R3 | **>2^53 整数精度边界**：9007199254740993 落盘即 9007199254740992 | Excel 数值面 = double（Excel 自身 15 位有效数字同限——存储面结构性边界，非读侧缺陷） | J1 双面钉定 |
| R4 | **日期认定必须 numFmt 证据**：serial + cellXfs numFmtId（builtin 14-22/45/47 或自定义格式码 d/m/h/y/s 计 token）→ DateTime；**纯 Float 永不升级为日期** | formats.rs `format_excel_f64_ref`（numFmt 判定驱动，非「Float+猜测」）——判定机制本身可靠（builtin 表 + openpyxl 移植 token 检测双面）；保真边界 = numFmt 证据可被生产行为影响（如文本格配日期格式不触发——数值路径才判型） | D1（numFmt14）/L1（自定义 yyyy-mm-dd）vs E1（同 serial 无 numFmt = Number） |
| R5 | **ISO 日期 = DateTimeIso（t="d" 显式存储）**——第二类明确依据 | read_v t="d" → `Data::DateTimeIso(原串)`（文件生产者定型，无推断） | F1 |
| R6 | **时长 = Duration（TimeDelta numFmt 证据）**：builtin 46 `[h]:mm:ss` 或自定义 `[…]` 时长格式码 | formats.rs TimeDelta 判定分支 | M1（1.5 天 → Duration；as_iso_duration = PT36H0M0S） |
| R7 | **公式两通道区分**：值面 = 缓存计算结果（values/typed）；公式文本 = 独立通道（formulas/`formula_at`）——禁止缓存值误当公式文本 | calamine `worksheet_range`（含缓存值）与 `worksheet_formula`（文本，无 `=` 前缀）两 API 分立；安全联接 = `SheetData::formula_at`（锚定换算内化——range_anchor_pinned 锚定语义免坑） | `formula_text_channel_distinct_from_cached_value`；`formula_at_safe_join_and_replay_junction` |
| R8 | **无缓存值公式两义性**（读回 Float(0.0) 与真值 0 不可区分） | FIDELITY §1 既有登记（读侧库行为边界）；r4-t1 维持——replay_cell 对 Empty 缓存 → cached None（读回 Float(0.0)） | write_roundtrip.rs 既有断言（本轮未改） |
| R9 | **错误格 = Error（码文本保留）**：#DIV/0!/#N/A/#NAME?/#NULL!/#NUM!/#REF!/#VALUE!/#DATA! | t="e" → `Data::Error`（CellErrorType 8 变体，Display = 码文本）；可诊断不丢弃不 panic | I1；replay → Text(码文本)（降级登记） |
| R10 | **显式转换有据才发生**：`as_iso_datetime` 仅 DateTime/DateTimeIso；`as_iso_duration` 仅 Duration/DurationIso；纯 Number 序列数 → None | 裁决「日期转换必须有明确依据」——无据转换不发生（映射层零静默语义转换）；序列数→组件 = calamine `to_ymd_hms_milli`（无 chrono 门控；1900/1904 双纪元 + Excel 1900 闰年怪癖由引擎承载） | `typed_explicit_conversions_ruled_by_evidence`（E1 → None 钉定） |
| R11 | **1904 纪元正确解码**：workbookPr date1904 → is_1904 进模型；同 serial 不同历法（+1462 天） | `Xlsx::has_1904_epoch`（构造期 eager 解析——xlsx/mod.rs read_workbook 在 new 内）；写面联接 = epoch_to_1900（写面引擎恒 1900 纪元） | 1904 探针（44927 = 2027-01-02）；往返历法保持 |
| R12 | **显示面兼容**：CellData::display() 与 calamine Data Display 逐字等值（t17「缓存值 Display 语义」不变） | display 按 Data Display 语义逐变体实现；日期格 Display = 裸序列数（ExcelDateTime Display = value——datatype.rs:986） | `typed_display_equivalence_with_data_display` 全格等值 + web 七套件零断言改动 |
| R13 | **类型保持重放**（读-改-重建联接）：Text/Number/Boolean kind 保持；DateTime/Duration → Number(serial)+日期 numFmt 样式（写回经 numFmt 证据再判型）；1904→1900 纪元换算；DateTimeIso/DurationIso/Error → Text（引擎无对应写面——值保持 + kind 降级登记，可诊断） | `SheetData::replay_cell` 单调用点（公式联接 + kind 分派 + 样式配对成对返回防漏配）；t22-review 前置裁决项「⑤XLSX 数值/布尔类型推断策略」的落地答案 | `kind_preserving_replay_roundtrip`（14 格全链 + 1904 链） |

### 8.2 不支持面与降级登记（可诊断，非静默）

- **wasm BlockTree→xlsx 导出面维持全 Text**（t19 口径）：markdown
  表格单元格 = 文本模型（无类型载荷）；「数字文本不擅自转数字」裁决
  使从文本推断类型不可行（前导零会因推断失真）。类型保持写面在
  office-io 层（replay_cell）供有类型源消费；BlockTree 扩类型载荷
  = mdcore 域上游裁决，未立项不宣称（xlsx_export.rs 模块文档登记）。
- **写面无 t="d"（ISO 存储）面**：DateTimeIso/DurationIso 重放为
  Text 原串（值保持，kind 降级）。
- **写面无错误格**：Error 重放为 Text(码文本)（码可诊断，kind 降级）。
- **DurationIso 在 xlsx 读路径不可达**（t="d" 恒产 DateTimeIso；
  DurationIso 仅 ODS）——防御性承载保 kind 完整性。
- **无缓存值公式**（R8）与 **>2^53 精度**（R3）= 存储面/引擎面
  结构性边界，非缺陷。

### 8.3 模型面（新公开 API——office-io，只增不改）

`io::CellData`（九变体 + display/as_iso_datetime/as_iso_duration/
to_write_parts）、`io::CellGrid`（get/get_abs 双坐标系）、
`SheetData.typed` 字段、`SheetData::formula_at`、
`SheetData::replay_cell`、`io::DATE_REPLAY_FORMAT`/
`DURATION_REPLAY_FORMAT` 常量；既有 `values: Range<Data>` 面不变
（既有消费方零破坏——write_roundtrip/corpus_harness/
range_anchor_pinned/docx 面零改动验证）。
