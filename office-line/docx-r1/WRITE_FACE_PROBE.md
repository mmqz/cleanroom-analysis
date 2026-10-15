# docx-r1 写面探针报告（t20-a，2026-10-15）

> 主题：**docx-rs 0.4.22 写面四件 API 一手核验**（t18 四件降级项的
> 库能力边界裁决——t20「扩写面消降级」增量的前置可行性证据）。
> 性质：分析仓 spike 探针（`office-line/spike/`），**只读产品知识、
> 不碰 mdcore**（环境重置，docx_export.rs 原码不可访问——以 worklog
> t18 节 + REPORT §8 + crate 源码一手验证为据）。
> 探针：`spike/src/bin/docx_write_probe.rs`（新 bin，16 断言全过）。

## 1. 目的与范围

t18（DOCX 导出切片）登记四件降级项，根因统一为 office-io
`DocxWriter` 包装面未暴露对应能力（库能力待验）：

| # | 降级项 | t18 现状（REPORT §8.3） | t20 目标表达 |
|---|---|---|---|
| 1 | list（ListItem 块） | marker/depth（列表结构）丢弃，item 富段平铺 | OOXML numbering（numbering.xml + 段落 numPr/numId） |
| 2 | monospace（行内 code mark） | 降级纯文本（RunSpec 无等宽面） | run 层 rFonts 等宽字体 |
| 3 | hr（ThematicBreak 块） | 空段落（无水平线面） | 段落下边框 pBdr（OOXML 无原生 hr 的惯用表达） |
| 4 | quote（Blockquote 块） | 子块平铺（引用容器/缩进丢失） | 段落缩进 ind + 左边框 pBdr |

附加项（t19-review 遗留顺带确认）：**块级 code 等宽段落**
（CodeBlock 现降级为每行纯文本段——等宽/字体面全丢）。

范围 = 写面（docx-rs 0.4.22 源码 API 签名 + 探针实证 XML 结构面）；
不含 office-io 包装面改码（t20 主体）与 web 接线。版本与产品
office-io 锚定一致（spike Cargo.toml 新增 `docx-rs = "0.4.22"`，
未动既有四依赖；docx-rs 自身依赖 zip 8.6.0 与 spike 既有 zip 同
版本，零第二 zip）。

## 2. 核验表（五件：API 路径 / 源码锚点 / 可表达性 / XML 证据）

源码根：`~/.cargo/registry/src/index.crates.io-…/docx-rs-0.4.22/src/`
（下称 `src/`）。

### 2.1 numbering（列表块）——**可行（含多级）**

| 面 | API 路径（类型 + 方法签名） | 源码锚点 |
|---|---|---|
| 抽象定义注册 | `Docx::add_abstract_numbering(mut self, num: AbstractNumbering) -> Docx` | documents/mod.rs:662 |
| 具体 num 注册 | `Docx::add_numbering(mut self, num: Numbering) -> Docx` | documents/mod.rs:667 |
| 抽象编号构造 | `AbstractNumbering::new(id: usize)` → `.add_level(level: Level)`（多级 = 多次 add_level；可选 `.num_style_link/.style_link`） | documents/elements/abstract_numbering.rs:19-32 |
| 级定义 | `Level::new(level: usize, start: Start, format: NumberFormat, text: LevelText, jc: LevelJc)` + `.indent(left, special, end, start_chars)` / `.suffix(LevelSuffixType)` / `.level_restart(u32)` / `.is_lgl()` / `.paragraph_style(id)` | documents/elements/level.rs:26-59 |
| 有序 vs 无序 | `NumberFormat::new("decimal")`（或 lowerLetter 等 9 种）vs `NumberFormat::new("bullet")`；`LevelText::new("%1.")`（编号模板）vs `LevelText::new("•")`（符号字形） | elements/number_format.rs:13（impl Into<String> 自由值） |
| numId 桥 | `Numbering::new(id: usize, abstract_num_id: usize)` → `<w:num w:numId=…><w:abstractNumId …/></w:num>` | elements/numbering.rs:17 |
| 段落引用 | `Paragraph::numbering(id: NumberingId, level: IndentLevel) -> Self`——`NumberingId::new(numId)` + `IndentLevel::new(ilvl)` → `<w:numPr><w:numId/><w:ilvl/></w:numPr>`；**同置 `has_numbering=true`** | elements/paragraph.rs:327-331；elements/numbering_property.rs:30 |

**关键机制发现**（t20 实施要点）：

- `word/numbering.xml` **无条件写入**每个 docx（zipper/mod.rs:72/188；
  [Content_Types].xml override 恒在，content_types.rs:71）——t18 现有
  输出已含该 part（内建默认 abstractNum id=1：decimal 9 级 + numId=1
  桥，numberings.rs:49-176）；条件项是 `word/_rels/document.xml.rels`
  的 numbering 关系（`has_numbering` 链：Paragraph::numbering →
  document.rs:110 → document_rels.rs:83-89）。**t20 必须走
  `Paragraph::numbering` 方法**——`ParagraphProperty::numbering_property`
  不置 has_numbering → rels 缺关系 → 悬空 numPr。
- 多级列表：abstractNum 内多 `add_level`（Level.level = ilvl 0…n），
  段落 `IndentLevel::new(ilvl)` 选级；内建默认抽象即 9 级（ilvl 0-8）。
- 官方范例同构：examples/numbering.rs（abstractNum 2 + num 2,2 +
  Paragraph::numbering(NumberingId::new(2), IndentLevel::new(0))）。

**XML 证据**（探针实测，§3）：numbering.xml 含
`<w:abstractNum w:abstractNumId="100">`（decimal `%1.` + 二级
lowerLetter `%1.%2.`）与 `"101"`（bullet `•`）+ `<w:num w:numId="100">
<w:abstractNumId w:val="100" />`×2 桥；document.xml 段落 pPr 含
`<w:numPr><w:numId w:val="100" /><w:ilvl w:val="0" /></w:numPr>`×2、
ilvl=1×1、numId=101×2；document.xml.rels rId7→numbering.xml。

### 2.2 monospace（行内 code mark → run 层 rFonts）——**可行**

| 面 | API 路径 | 源码锚点 |
|---|---|---|
| 字体构造 | `RunFonts::new()` → `.ascii(f)` / `.hi_ansi(f)` / `.cs(f)` / `.east_asia(f)` / `.hint(f)`（四槽 + 主题变体，各 Option） | elements/run_fonts.rs:39-87 |
| run 挂接 | `Run::fonts(mut self, f: RunFonts) -> Run` | elements/run.rs:356 |
| 属性层 | `RunProperty::fonts(mut self, font: RunFonts)`（rPr 子 `w:rFonts`） | elements/run_property.rs:190 |

等宽表达 = `RunFonts::new().ascii("Consolas").hi_ansi("Consolas").cs("Consolas")`
→ `<w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas" />`
（run_fonts.rs 测试锚同形；槽值经 `escape_owned` 转义）。
`w:ascii`（首 128 码位）+ `w:hAnsi`（高 ANSI）双槽即覆盖行内 code
的全部常规字符；`w:cs` 补复杂文种槽。

**XML 证据**：document.xml run 层 `<w:rPr><w:rFonts w:ascii="Consolas"
w:hAnsi="Consolas" w:cs="Consolas" /></w:rPr>`（探针 1 处精确断言过）。

### 2.3 hr（ThematicBreak → 段落下边框）——**可行**

| 面 | API 路径 | 源码锚点 |
|---|---|---|
| 单边框 | `ParagraphBorder::new(position: ParagraphBorderPosition)` → `.val(BorderType)`（默认 Single）/ `.size(usize)`（1/8 pt）/ `.space(usize)` / `.color(impl Into<String>)`（默认 "auto"） | elements/paragraph_borders.rs:24-55 |
| 边框组 | `ParagraphBorders::with_empty()`（**六边全空**）→ `.set(border)` 按位挂；`.clear(position)` = nil 边框显式清除 | elements/paragraph_borders.rs:135-156 |
| 段落挂接 | `ParagraphProperty::set_border(border)` / `set_borders(borders)`；`Paragraph.property` 为 **pub 字段**（无便捷方法）：`p.property = p.property.set_border(...)` | elements/paragraph_property.rs:201-209；paragraph.rs:15 |

**陷阱登记**：`ParagraphBorders::new()`/`Default` = **四边全 Single**
（paragraph_borders.rs:117-127）——hr 若用 default 会画整框而非水平
线；必须 `with_empty().set(bottom)`。hr 惯用参数：bottom 单边
（探针 size=6 = 0.75pt、space=1）挂**空段落**（无 run）。

**XML 证据**：`<w:pPr><w:rPr /><w:pBdr><w:bottom w:val="single"
w:space="1" w:sz="6" w:color="auto" /></w:pBdr></w:pPr>`（空段）。

### 2.4 quote（Blockquote → ind 缩进 + 左边框）——**可行**

| 面 | API 路径 | 源码锚点 |
|---|---|---|
| 缩进 | `Paragraph::indent(left: Option<i32>, special_indent: Option<SpecialIndentType>, end: Option<i32>, start_chars: Option<i32>)`；`SpecialIndentType::{FirstLine(i32), Hanging(i32)}` | elements/paragraph.rs:306-315；types/special_indent_type.rs:10-13 |
| 左边框 | 同 2.3（`ParagraphBorders::with_empty().set(ParagraphBorder::new(Left)…)`） | 同 2.3 |

序列化锚：`Indent::new(Some(20), None, None, None)` → `<w:ind
w:left="20" w:right="0" />`（indent.rs:96-99 测试锚）——**end=None
恒出 `w:right="0"`**（`end.unwrap_or_default()`，indent.rs:63），
无语义害但 t20 断言须按此形。探针参数：left=720（0.5 英寸）+
左边框 size=12（1.5pt）space=4。

**XML 证据**：`<w:ind w:left="720" w:right="0" />` + `<w:pBdr><w:left
w:val="single" w:space="4" w:sz="12" w:color="auto" /></w:pBdr>`
（同段双面）。

**替代面登记**：无内建 "Quote" 段落样式可引（preset_styles 仅
toc——preset_styles/mod.rs；`Paragraph::style("Quote")` 只产悬空
pStyle 引用）。t20 表达 = 直接格式（ind + pBdr），非样式引用——
与 t18 REPORT §8 降级表预判一致。

### 2.5 块级 code（CodeBlock 整段等宽）——**可行（同 API 双通道）**

- **段落级**：`Paragraph::fonts(f: RunFonts)`（elements/paragraph.rs:353
  → ParagraphProperty.run_property.fonts）→ `<w:pPr><w:rPr><w:rFonts …/>
  </w:rPr></w:pPr>`（段落标记 run 属性 + 段内缺省面）；
- **run 级**：段内每 `Run::fonts(f)`（同 2.2）。

**XML 证据**（探针 Courier New 双通道）：
`<w:pPr><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"
w:cs="Courier New" /></w:rPr></w:pPr>` + 每 run 同形 rPr（2 run 断言过）。
工程建议 t20 **双通道并设**（run 级保真 + 段落级兜段落标记），成本
为零（同一 RunFonts 值复用）。lang/fence 结构仍不可表达（库无语法
高亮面）——维持降级登记，不在 t20 范围。

### 2.6 顺带发现（t18 降级表外的同族项）——库层面同样可达

- **行内 del（删除线）**：`Run::strike()` / `Run::dstrike()`（单/双
  删除线，run.rs:326-334）；
- **行内 u（下画线）**：`Run::underline(line_type: impl Into<String>)`
  （如 "single"，run.rs:346-349）。

t18 把 code·del·u 三 mark 一并降级为纯文本（RunSpec 无面）——库
层面三件全可达；t20 若一并扩 RunSpec（mono + strike + underline 三
布尔/枚举面），行内降级面可整批清零（链接 mark 除外——模型无
URL 载荷，mdcore MD 域上游缺口维持）。

## 3. 探针实证记录

命令（spike 工程根，前台执行）：

```console
$ cargo run --bin docx_write_probe
generated: /tmp/t20a_docx_write_probe.docx
PASS numbering.xml part exists in zip
PASS document.xml.rels has numbering relationship (has_numbering wiring)
PASS numbering.xml: ordered abstractNum (decimal fmt + lvlText %1.)
PASS numbering.xml: bullet abstractNum (bullet fmt + • glyph)
PASS numbering.xml: concrete num bridges (numId 100/101 -> abstract)
PASS numbering.xml: multi-level (ilvl=1 level with lowerLetter fmt)
PASS document.xml: ordered numPr (numId=100 + ilvl=0, exactly two items)
PASS document.xml: nested numPr (numId=100 + ilvl=1, exactly one)
PASS document.xml: bullet numPr (numId=101 + ilvl=0, exactly two items)
PASS document.xml: inline mono run rFonts (Consolas)
PASS document.xml: hr paragraph border (pBdr + w:bottom)
PASS document.xml: quote indent (w:ind w:left="720")
PASS document.xml: quote left border (pBdr + w:left)
PASS document.xml: block-code paragraph-level rFonts (pPr>rPr>rFonts Courier New)
PASS document.xml: block-code run-level rFonts (Courier New, two runs)
PASS document.xml: list item texts survived (5 items + inline + quote + code)
16/16 checks passed        # 退出码 0
```

首跑 15/16：唯一 FAIL 为**探针断言自身错**（numId=100 计数期望 2，
实际 3 = 2 顶层 + 1 嵌套——XML 正确），修正为 numId+ilvl 精确配对
断言后全过。库行为零异常。

独立第二通道复证（进程外，unzip + python zipfile）：

```console
$ unzip -l /tmp/t20a_docx_write_probe.docx        # word/numbering.xml 3194B 在包（18 条目）
$ unzip -p … word/document.xml | grep -o '<w:numPr>…'   # 2×(100,ilvl0) 1×(100,ilvl1) 2×(101,ilvl0)
$ unzip -p … word/document.xml | grep -o '<w:rFonts…'   # Consolas×1 + Courier New×3（1 段落级+2 run 级）
$ unzip -p … word/document.xml | grep -o '<w:pBdr>…'    # w:bottom（hr）+ w:left（quote）
$ unzip -p … word/document.xml | grep -o '<w:ind …'     # w:left="720" w:right="0"
$ unzip -p … word/_rels/document.xml.rels               # rId7 …numbering… numbering.xml
$ python3 -c "zipfile…testzip"                          # OK（none bad）
```

numbering.xml 骨架（探针自定义 + 库内建默认并存）：
abstractNum **1**（内建 decimal 9 级，恒在）→ numId=1；abstractNum
**100**（decimal `%1.` + lowerLetter `%1.%2.`）→ numId=100；abstractNum
**101**（bullet `•`）→ numId=101。

## 4. t20 实施映射草案（office-io DocxWriter 扩面清单）

依据：t18 REPORT §8.3 映射表（降级列）+ 本文 §2 API 核验。mdcore
仓本轮不可访问，以下为**改法要点草案**（非码级 diff，t20 主代理核对
DocxWriter 现行签名后落码）。

### 4.1 office-io `DocxWriter` 包装面新增（约 5 面）

1. **numbering 定义面**（writer 级状态）：构造期（或首个列表项时惰性）
   注册两个 abstract numbering——ordered（decimal，9 级，Level.indent
   悬挂缩进 720/360 步进）+ bullet（`•`，同构）+ 对应 `Numbering` 桥；
   numId 分配器（如 100/101 固定或递增，避开内建 numId=1）。
   `Docx::add_abstract_numbering/add_numbering` 均为 consuming builder
   ——包装层在 build 前注入即可（t18 走 `Docx::new().add_paragraph…`
   链，无冲突）。
2. **列表段落面**：`list_item(runs, num_id, ilvl)`（或 item 富段 + 属性
   重载）→ `Paragraph::numbering(NumberingId, IndentLevel)`（**必须走
   此方法**以接通 has_numbering→rels 链，§2.1）。
3. **等宽 run 面**：`RunSpec` 增 mono 位（或 font 面）→ `Run::fonts(
   RunFonts::new().ascii(MONO).hi_ansi(MONO).cs(MONO))`；同批可加
   strike/underline 位（§2.6，行内 del·u 一并消降级）。
4. **hr 面**：`horizontal_rule()` → 空段 + `ParagraphBorders::with_empty
   ().set(ParagraphBorder::new(Bottom).size(6).space(1))`（pub 字段
   `p.property = p.property.set_border(...)` 挂接——库无 Paragraph 级
   便捷法，包装层自封装）。
5. **quote 面**：段落属性选项（indent left + 左边框）→
   `Paragraph::indent(Some(720×depth 钳), None, None, None)` +
   `with_empty().set(Left…)`；嵌套 Blockquote 递归加深（left 步进，
   上限钳制防溢出版心）。

### 4.2 mdcore `docx_export.rs` 映射改法要点

| 块/mark | 现状（t18） | t20 改法 | 预期 |
|---|---|---|---|
| ListItem | 富段平铺，marker/depth 丢 | marker 判序（数字型→ordered numId，`-`/`*`/`+` →bullet numId）+ depth→ilvl（**钳 0-8**，abstractNum 9 级）；子块递归保持 | 降级消除 |
| 行内 code mark | 纯文本 run | runs_from 分段加 mono 位（RunSpec 扩面）；邻接合并键加 mono | 降级消除 |
| 行内 del/u mark | 纯文本 run | 同上加 strike/underline 位（§2.6 顺带消解） | 降级消除（建议纳入 t20 或登记紧后增量） |
| ThematicBreak | 空段落 | horizontal_rule() | 降级消除 |
| Blockquote | 子块平铺 | quote 段落属性（ind+pBdr left，嵌套步进）；子块递归保持 | 降级消除 |
| CodeBlock | 每行纯文本段 | 每行段落双通道等宽（段落级+run 级 fonts）；lang/fence 仍丢 | 等宽降级消除，lang 登记维持 |

### 4.3 测试与门禁要点（t20 预置）

- 原生：写→`read_docx_from_bytes`→模型断言（t18 模式）——注意读面
  对 numPr/pBdr/ind/rFonts 的模型承载（t16 读面若不产列表块，全环
  断言降级子集 + XML 结构面直断言〔本探针 16 断言可直接移植为测试
  模板〕）；
- e2e：`unzip -p document.xml | grep` 四结构面（探针 §3 命令即脚本
  骨架）+ 状态行 simplified 计数变化（四件清零后 `0 simplified` 或
  删计数）；
- wasm 包体：新增面均在既有 docx-rs 依赖图内（t18 已付 +50.3%），
  预期增量微小（包装层胶水）。

## 5. 结论

| 项 | 裁决 | 依据 |
|---|---|---|
| list（有序/无序/多级） | **可行，直接消解** | §2.1：三 API 面 + rels 自动接线 + 探针 9 断言 |
| monospace（行内 code） | **可行，直接消解** | §2.2：Run::fonts + 探针断言 |
| hr（ThematicBreak） | **可行，直接消解** | §2.3：pBdr bottom 空段 + 探针断言（with_empty 陷阱已登记） |
| quote（Blockquote） | **可行，直接消解** | §2.4：ind + pBdr left + 探针断言（无内建 Quote 样式，直接格式） |
| 块级 code 等宽 | **可行，直接消解** | §2.5：Paragraph::fonts + Run::fonts 双通道 + 探针 2 断言 |
| 行内 del/u（顺带） | **可行，建议同批** | §2.6：Run::strike/dstrike/underline 在库 |
| 链接 mark | **不可行（非库层面）** | 模型无 URL 载荷（MarkSpan 上游缺口，t18-review 登记）——mdcore MD 域裁决前置 |
| code lang/fence 结构 | **不可行（库层面）** | docx-rs 无语法高亮/元数据面——维持降级登记 |

**总结论：四件降级 + 块级 code 等宽全部库层面可达，t20 扩 office-io
包装面即可整批消解；无「库也不可行」项**（仅链接 URL〔模型上游〕与
code lang〔库面〕两残项维持登记）。

已知怪癖（如实登记，非阻塞）：docx-rs 序列化 pPr 子序非 ECMA-376
典范序（rPr 恒居首、pBdr 居 ind 后——paragraph_property.rs:227-254
build 链；典范序 rPr 应居末、pBdr 先于 ind）。t18 既有输出同形（空
`<w:rPr />` 恒出），docx-rs 自家 reader 与 python zipfile 均接受；
真 Word/Word Online 兼容性沙箱不可验——**与 t18 基线同边界**（t18
e2e 全环亦经 docx-rs 读面自证），t20 不新增此风险面。

---

*恢复注记：本报告及配套探针于 2026-10-15 第 10 次环境回滚后按
transcript 逐字重写恢复（原 588891b 提交被平台回滚删除）；探针
16/16 断言在恢复环境中重跑复证（release 构建），XML 结构面证据
逐位一致。*
