# DOCX 读写保真矩阵（r4-t3，2026-10-15）

> 依据：产品代码与测试一手判定（office-io/src/docx.rs 3591 行 + tests/
> docx_roundtrip.rs 28 测试 + docx-rs 0.4.22 引擎面一手核验），**不根据
> 引擎理论能力认定产品支持**。五态：完整承载 / 部分承载 / 写入降级 /
> 读取丢失 / 尚未验证。本文件为 DOCX 读写保真单一真源，随码走更新。

## 1. 矩阵（24 项）

| # | 内容 | 状态 | 读面锚点 | 模型锚点 | 写面锚点 | 测试锚点 |
|---|------|------|----------|----------|----------|----------|
| 1 | 段落与文本 Run | 完整承载 | extract_paragraph | ParagraphData/runs | add_paragraph 族 | 既有 26 测试 |
| 2 | 标题级别（Heading） | 部分承载 | heading 激活语义透传 | level | heading() | docx_roundtrip 既有 |
| 3 | bold/italic 行内 | 部分承载 | **读取丢失**（Bold.val 私有——w:val="false" 显式关闭形态不可辨，presence 会误报 → 不做 presence 承载，登记）；写面完整 | RunSpec.bold/italic | bold/italic | t18 既有 |
| 4 | strike/underline 行内 | 写入降级（写面 t20 已达；读面同 #3 理由丢失：Strike.val/Underline.val 私有） | — | RunSpec.strike/underline | strike/underline | t20 |
| 5 | 等宽 rFonts | 写入降级（写面 t20 双通道；读面 RunFonts 私有不可承载） | — | RunSpec.mono | fonts() | t20 |
| 6 | 超链接（External + Anchor 内链） | 部分承载（**锚点写面写读对称已达成**：External/Anchor 双变体写回[mdcore c4856f7 批次 2 裁决合入]；Internal 变体写面引擎结构性不可达登记维持；书签定义读面静默丢——存在性不可辨 r5-t2 探针③） | t13 四变体读面 | HyperlinkData | hyperlink（External）+ anchor_link/RunSpec.anchor（w:anchor 属性形零 rels）+ bookmark_paragraph（书签目标段）——裁决落定注记见 CAPABILITY_NOTES §13 头部 | t13 既有 + r5-t2 探针①②⑤ + T1-T7 锚点组（c4856f7）+ R-O2 anchor-survival（LO 链接存活钉定） |
| 7 | 页眉/页脚六槽位 | 部分承载（first/even/default 读写两面在；default 占位语义 replace 踩踏 t14 已修） | headers/footers | HeaderFooterData | add_header/add_footer 族 | t13/t14 |
| 8 | 脚注 | 读取丢失（引擎不读 footnotes.xml 部件——t14 登记）；写面可达 | — | — | add_footnote | t14 |
| 9 | 图片（PNG）与媒体关系 | 读取丢失（位置/尺寸不读；媒体清单读 t14）；写面可达 | media 清单 | — | add_image | t14 |
| 10 | PAGE/NUMPAGES 域 | 部分承载（读面缓存文本保留；域结构不承载——渲染期计算语义，登记） | 文本透传 | — | add_page_number 族 | t14 |
| 11 | HYPERLINK 域 | **绝不暴露**（引擎 todo!() 崩溃面——t14 登记，红线维持） | — | — | — | — |
| 12 | 列表编号 numPr 结构 | **完整承载（r4-t3 本轮）** | carry_numbering（NumberingIndex：numId→num 桥→abstractNum→ilvl 级 numFmt 解析） | ParagraphNumbering{num_id, ilvl, kind, format} | list_item/list_item_lines（自有抽象 100/101） | r4t3_numbering_and_indent_read_carry + **web 消费链映射 r5-t1**（docx_import 8 测试 + e2e，见 §5） |
| 13 | numPr 语义解析边界 | 完整承载（numId=0 = OOXML 去编号惯例→非列表；num 桥悬空严格拒/兼容 BrokenReference 警告+结构承载 kind=None；numStyleLink/ilvl 超定义域软警告+结构承载） | NumberingIndex::build | 同上 | — | r4t3_numbering_dangling_custom_format_injected_ind；**消费面（r5-t1）：kind=None / numFmt `none` → 普通段落降级（非崩溃非静默列表，`list_items_degraded` 计数）** |
| 14 | 自定义编号格式（lowerLetter 等原生 numFmt） | 写入降级（format 字段承载原生 numFmt 使降级**可诊断**——写回经自有抽象 100/101 decimal/bullet，原生格式不回写）；**导入消费面（r5-t1）同步降级：有序序号标准化 decimal（`list_markers_normalized` 计数）** | format: Option<String> | 同上 | ListKind 两态 | 同上（自定义 numFmt 注入测试）+ docx_import list_custom_numfmt_normalizes_to_decimal |
| 15 | 段落缩进 ind | **完整承载（r4-t3 本轮）** | carry_indent（reader 全属性：left/start/right/end/hanging/firstLine） | ParagraphIndent{left,right,special: FirstLine/Hanging} | quote_paragraph/list_item（固定步进 720/360） | r4t3_numbering_and_indent_read_carry |
| 16 | ind Chars 单位变体 | 部分承载（hanging_chars 等字段读面可达但单位换算语义不承载——登记） | reader | — | — | — |
| 17 | 段落边框 pBdr | 写入降级（写面 t20 hr/quote 左框；读面 ParagraphBorders 六槽字段私有不可承载） | — | — | horizontal_rule/quote | t20 |
| 18 | 换行 w:br | 完整承载（读面 t16 br→\n 约定；写面 t22 w:br textWrapping） | t16 | 行内 \n | list_item_lines/break | t22 |
| 19 | 表格 | 部分承载（扁平管道行文本——格内结构与合并目标不承载，t18 登记） | 表文本透传 | — | 表写面 | t18 |
| 20 | 样式定义（styles.xml） | 部分承载（读透传 preset_styles；自定义样式定义不可回写） | 引擎 preset | — | — | — |
| 21 | 主题/设置/编号原始定义（abstractNum 全量） | 读取丢失（部件级不读或部分读——重建时按自有抽象生成） | — | — | ensure_numberings | t20 |
| 22 | 修订/批注/SDT | 读取丢失（结构性，登记） | — | — | — | — |
| 23 | 多节 section | 读取丢失（单节模型，登记） | — | — | — | — |
| 24 | ZIP 容器/关系完整性 | 完整承载（三门 + rels 读写 + 读→改→写全环非目标内容不意外删除） | 三门 | — | 引擎打包 | r4t3_roundtrip_read_modify_write（预期丢失集恰={footnotes.xml, media/*}——引擎不写部件，结构性登记） |

### 计数（本轮前 → 本轮后）

- 完整承载 8 → **10**（+numPr 结构、+ind 读面）
- 部分承载 4（不变）
- 写入降级 2（numPr/ind 写回经自有抽象——降级可诊断）
- 读取丢失 8（不变——其中 3 项为引擎值私有结构性：rFonts/strike/underline、pBdr）
- 尚未验证 **0**（逐行给锚点）

## 2. 方案论证（T3.2 结论：B 最小版）

- **A 原始 XML/部件保留**：保真最高但模型侵入大（需原始 XML+回放机制），
  与「模型级处理与渲染保真分离」的模块定位冲突；引入准全量 OOXML
  表面积，触碰「不引入通用 OOXML 抽象」红线——否决。
- **C 拒绝不安全重写**：最保守但零增益——numPr/ind 是 pPr 结构性字段，
  承载不引入渲染语义——否决。
- **B 显式扩展模型（最小版，选定）**：读面解析进模型字段
  （ParagraphData +2 Option 字段默认 None，既有消费方零破坏——循
  r4-t1 CellData 先例）+ 写面既有 API 回放（list_item/list_item_lines/
  quote_paragraph 全部在位）+ 降级显式化（format 字段承载原生 numFmt，
  写回降级可诊断；悬空 numId 严格拒/兼容警告）。

## 3. 明确不做（登记，非本轮范围）

- rFonts/strike/underline 读面承载：引擎值私有（Bold.val/Strike.val/
  Underline.val/RunFonts 无 getter）——presence-only 承载对
  w:val="none"/"false" 显式关闭形态会误报，宁缺毋错。
- pBdr 读面承载：ParagraphBorders 六槽字段私有，同上理由。
- ~~anchor 超链接写回：引擎 HyperlinkType::Anchor 写面在，但 RunSpec
  扩锚点字段触碰公开模型语义——另行裁决。~~ **r5-t2 已评估**（方案
  与裁决点落档 CAPABILITY_NOTES §13；本文件矩阵口径不变——本轮
  零实施，只增 5 探针测试：tests/docx_roundtrip.rs 28→32〔Internal
  整包面/锚点优先序/书签静默丢弃+存在性不可辨/引擎直构 spike〕+
  mdcore-wasm docx_export.rs +1〔链接形态文本导出面〕）。**后续
  闭环（2026-10-17）：批次 2 裁决 Option A Phase 1+2 全批合入 mdcore
  main c4856f7**（RunSpec.anchor/anchor_link/bookmark_paragraph/
  V1-V7 校验/T1-T7 测试——矩阵 #6 行已同步翻转，本论证行闭环）。
- 脚注/图片读面：引擎结构性不读部件（footnotes.xml / 图片位置）。
- ~~web 消费链 list 映射（docx_import 列表→markdown）：下一轮候选~~
  **r5-t1 已落地闭项**（映射语义与降级登记见 §5）。
- 全量 OOXML 无损编辑：**不宣称**（验收红线）。

## 4. 往返验证证据（T3.3）

- `r4t3_numbering_and_indent_read_carry`：写（list_item×2 级 +
  quote_paragraph + list_item_lines 多行）→ 读 → 断言 numId/ilvl/kind/
  ind 结构逐字段承载。
- `r4t3_numbering_dangling_custom_format_injected_ind`：手构 XML 注入
  自定义 numFmt（lowerLetter）+ 悬空 numId + firstLine ind——严格模式
  typed 拒 / 兼容模式警告 + 结构承载（kind=None + format 原样）。
- `r4t3_roundtrip_read_modify_write`：读 → 改（真实修改一个 run 文本）
  → 写 → 重读：目标特性保真 + ZIP 部件对比——**预期丢失集恰 =
  {footnotes.xml, media/*}**（引擎不写部件，结构性登记，非意外删除）；
  rels 对比零意外。
- 门禁（主代理亲跑）：G1 **517/0/1i**（512+5）；PDF 四面
  68/79/81+6i/109+6i 零漂移；clippy native 2 + wasm32 --all-targets 2
  基线；fmt 0；scan 0；wasm32 check+--all-targets 过；Cargo.lock 零
  diff；web contract 18 + smoke + docx_import 9 + docx_export 13 全绿。

## 5. web 消费链列表映射（r5-t1，2026-10-17）

`crates/mdcore-wasm/src/docx_import.rs` `docx_to_markdown` 消费
`ParagraphData.numbering`（此前 `p.numbering` 被完全忽略——列表段降级
普通段落，即本节闭项的登记缺口）。映射语义（模块文档单一真源随码走）：

- **映射面**：`kind: Some(Bullet)` → `- `；`kind: Some(Ordered)` →
  `{n}. `（n = 连续列表 run 内计数器）；嵌套缩进 = 2×min(ilvl, 8) 空格
  （mdcore `list_item_of` depth = indent/2+1——ilvl 0 = 顶级；钳 8 对称
  写面九级抽象域，`list_depth_clamped` 计数）；计数器 key =
  (num_id, ilvl)——不同 numId = 独立实例（Word 语义），同 numId 更浅级
  推进 → 深级清零（Word 多级：父项推进子级重起）。
- **降级面（全部 stats 计数可诊断 + 模块文档登记）**：
  - `kind=None`（悬空 numId / numStyleLink / ilvl 超定义域；兼容模式
    读面 BrokenReference 警告随 stats.warnings）→ 普通段落
    （`list_items_degraded`）；
  - numFmt `"none"`（Word 该级不显示 marker）→ 普通段落——不发明数字
    （宁缺毋错；`list_items_degraded`）；
  - 有序起始值假定 1（读模型不承载 `w:start`）；同 numId 非连续段
    **中断重启**（markdown 无持久列表身份；Word 对同 numId 非连续段
    续号——降级登记）——任何非列表映射块（普通段/标题/表/降级段）
    清零计数器；
  - 自定义 numFmt（lowerLetter/upperRoman/…）→ 序号标准化 decimal
    （`list_markers_normalized`）；
  - 标题段带 numPr → 标题赢（Word 编号标题仍显示为标题），编号丢弃
    （`list_numbering_dropped`）且该标题中断列表 run；
  - ilvl > 8 → 钳 8（`list_depth_clamped`）；空列表项 → marker + 尾
    空格行（mdcore 解析为空 ListItem——结构保真）；多行列表项
    （`w:br` 软换行）首行带 marker、续行脱离列表项（既有软换行族降级）。
- **stats 新增（additive）**：`list_items` / `list_items_degraded` /
  `list_numbering_dropped` / `list_depth_clamped` /
  `list_markers_normalized`——既有 key 零变化。
- **验证证据**（全部亲跑）：原生 8 新测试（映射矩阵+mdcore 解析闭环
  `list_numbering_maps_to_markdown_lines`/`list_markdown_parses_back_
  in_mdcore`、降级 `list_degraded_cases_count_and_warn`、标准化
  `list_custom_numfmt_normalizes_to_decimal`、钳位
  `list_depth_clamp_at_ilvl8`、标题优先
  `list_heading_precedence_drops_numbering`、空项
  `list_empty_item_emits_marker_only`、夹具全链
  `fixture_markdown_parses_to_list_tree`）+ 夹具扩展（build_sample_docx
  有序×2 + 嵌套 ilvl1 + 无序×2）+ e2e `docx_import.py` 9→11 checks
  （`import_list_nesting_depth` 先序深度 + `import_list_markers_and_
  md_depth` marker/深度逐项；既有 exact 断言同步扩展不降强度——
  EXPECTED_MD/块类型/计数/undo 回到导入态）。导入→编辑器消费闭环
  （docx → markdown → BlockTree 嵌套父子/marker）经
  `mdcore::markdown::markdown_to_tree` 直测 + e2e 双面背书。
- **不宣称**：列表渲染保真（marker 字形/编号起点/多级拼号形态——
  `%1.%2.` outline 与 `w:lvlText` 读模型不承载）；editor 列表 → 导出 →
  重导入单测复合往返（t18/t20 写面、r4-t3 读面、本节消费面三段各自
  实测，未做单 e2e 复合断言）；`w:start` 起始值 / `w:suff` / isLgl
  （读模型无字段——扩字段另行裁决）。
- **门禁（r5-t1 全量亲跑）**：G1 **525/0/1i**（517+8）；PDF 四面
  68/79/81+6i/109+6i 零漂移 + perf 6/6；clippy native 2（基线）+
  PDF 三面 0 + wasm32 --all-targets 2；fmt 0；scan 0；wasm32
  check+--all-targets 过；Cargo.lock 零 diff；web 七套件
  **18/15/28/11/13/9/9** 全绿（docx_import 9→11）。
