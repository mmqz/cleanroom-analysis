# docx-rs 0.4.22 能力一手核验（docx-r1 / t13，2026-10-15）

一手源：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/docx-rs-0.4.22/src/`
（下文锚点均为该目录内 file:line）。纪律：未实测不暴露——本文件为
Phase A 证据底稿，exposed/registered 裁决以此为界。

## 1. 超链接目标（读）

**裁决：引擎部分支持（rid 透传 + 仅 External 关系表），无 rid→target 联接 → 由自有 OPC 关系层补面后暴露。**

- 读侧 `w:hyperlink` 元素 → 共享模型 `Hyperlink.link`：
  - `w:anchor` 优先 → `HyperlinkData::Anchor { anchor }`（reader/hyperlink.rs:14-20）；
  - 否则 r:id → `HyperlinkData::External { rid, path: "" }`——**path 置空
    "not used"**，即引擎不做关系联接（reader/hyperlink.rs:21-26）；
  - 两者皆缺 → `rid: unwrap_or_default()` = 空串 External（畸形面）。
- 包级关系表：`Docx.hyperlinks: Vec<(rId, target, mode)>`
  （documents/mod.rs:175），由 read_docx.rs:386-398 从
  `rels.target_paths(HYPERLINK_TYPE)` 填充——**仅 TargetMode 属性存在的
  关系入表**（`if let Some(mode) = mode`，read_docx.rs:390）：无
  TargetMode 的（包内相对目标）超链接关系被静默丢弃。
- 引擎自身 rels 解析怪癖：hyperlink 型 Target 以 `Path::new("").join(t)`
  存储（reader/rels.rs:93-97），非 hyperlink 型以部件目录 join
  （reader/rels.rs:94）。

## 2. 页眉/页脚（读）

**裁决：引擎支持——读面经 docx-rs 模型暴露（本轮暴露）。**

- `read_docx` 读 `word/header*.xml`/`word/footer*.xml` 部件
  （read_docx.rs:7-49，经 document rels `target_paths(HEADER_TYPE/FOOTER_TYPE)`），
  `Header::from_xml`/`Footer::from_xml` 解析段落/表/StructuredDataTag
  （reader/header.rs:10-40 / reader/footer.rs:10-40）。
- 挂接：`section_property` 六槽位（default/first/even × header/footer），
  `header_reference: Option<HeaderReference>`（rId+type，reader/
  section_property.rs:111-126）与 `header: Option<(rid, Header)>`
  （documents/elements/section_property.rs:23-42）分离——**引用在而内容
  挂接失败（rid 无关系/部件缺失/Header 解析失败）时静默不挂**
  （read_docx.rs:15-24 `filter_map`）。
- 表格单元格内超链接：单元格 children 仅 Paragraph/Sdt/Property/嵌套
  Table（reader/table_cell.rs:16-40），但**单元格段落内**的
  `w:hyperlink` 经 reader/paragraph.rs 正常解析（Phase 3 提取已证）。
- 页眉部件自身的 rels（页眉内超链接/图片）经
  `read_header_or_footer_rels`（reader/header_or_footer_rels.rs:27-46）
  → 与 1 相同的 `read_rels_xml` 路径。

## 3. 页眉/页脚（写）

**裁决：引擎支持——写面暴露（default 槽位；first/even 槽位登记不暴露）。**

- `Docx::header/first_header/even_header/footer/first_footer/even_footer`
  （documents/mod.rs:576-601）：rId 分配 `rIdHeader{n}`/`rIdFooter{n}`
  （documents/header_id.rs / footer_id.rs `create_header_rid/footer_rid`）、
  content type add_header/add_footer、部件发射（prepare_package：
  documents/mod.rs:920-935 `collect_header_images`/`collect_footer_images`）。
- 页眉/页脚模型：`Header/Footer::new().add_paragraph(Paragraph)/add_table(Table)`
  （documents/header.rs:19-36 / footer.rs:19-36）。

## 4. 超链接（写）

**裁决：引擎支持外部目标（"Now support external only"）——写面暴露。**

- `Hyperlink::new(url, HyperlinkType::External)` → `HyperlinkData::
  External { rid: rIdHyperlink{n}, path }`（documents/elements/hyperlink.rs:21-45）；
  `Paragraph::add_hyperlink`（documents/elements/paragraph.rs:193）。
- 打包时收集进 document rels：`hyperlink_map` → `document_rels.hyperlinks
  .push((id, path, "External"))`（documents/mod.rs:1209-1214，注释原话
  "Now support external only"）；rels 发射带 TargetMode
  （documents/document_rels.rs:124-130 `relationship_with_mode`）。

## 5. 脚注（第二轮候选——快评）

**裁决：读面引擎受限（登记），写面引擎支持（不暴露）。**

- 读：`read_docx` **从不读 word/footnotes.xml**（read_docx.rs 全文无
  footnote 部件读取）；正文 run 级 `w:footnoteReference` 亦不解析
  （reader/run.rs 无 FootnoteReference 分支；RunChild::FootnoteReference
  为写侧变体，documents/elements/run.rs:45）。
- 写：`Run::add_footnote_reference(Footnote)`（documents/elements/run.rs:366）
  → footnotes.xml + has_footnotes content type（documents/mod.rs:927-932）。

## 6. 图片（第二轮候选——快评）

**裁决：读面包级字节面在（模型级正文提取面外，登记），写面引擎支持（不暴露）。**

- 读：media 关系逐条读原始字节进 `Docx.images: Vec<(rId, path, Image, Png)>`
  （read_docx.rs:382-384, 403-423；documents/mod.rs:167-173）；PNG 直通
  预览，非 PNG 无 `image` 特性时预览为空（我们 default-features=false，
  documents/mod.rs:291-327）。正文 Drawing 入模型（reader/run.rs:100-102）
  但模型级提取面外。
- 写：`Run::add_image(Pic)`（documents/elements/run.rs:230）。

## 7. 域（第二轮候选——快评）

**裁决：读面指令文本可入模型字符串（提取面外，登记），写面引擎支持（不暴露）。**

- 读：`instrText` → `RunChild::InstrTextString(String)`（reader/run.rs:110-114）；
  `fldChar` → `FieldChar`（reader/run.rs:19-38）。本切片 run 提取跳过
  （Phase 3 登记不变）。
- 写：fld_char/instr_text/instr_hyperlink/instr_page/instr_num_pages 等
  元素族在（documents/elements/）。

## 8. 引擎敌意面观察（rels 词汇表——实证勘误）

- **初判（源读）**：`read_rels_xml` 对每个 StartElement 做
  `XMLElement::from_str(&name.local_name).unwrap()`（reader/rels.rs:73）
  ——表面 `type Err = ()` 使 unwrap 疑似崩溃面。
- **实证勘误（探针测试 `engine_rels_unknown_element_tolerated`，
  tests/docx_roundtrip.rs）**：`XMLElement::from_str` 的 match 有
  catch-all 兜底臂 `_ => Ok(XMLElement::Unsupported)`
  （reader/xml_element.rs:447）——**永不返回 Err**，unwrap 恒安全；
  未知元素（如 `<zzz/>`）被映射为 Unsupported 后被
  `if let XMLElement::Relationship` 静默忽略——**引擎容忍、不崩溃，
  合法关系照常收集**（整包直调 `docx_rs::read_docx` 返回 Ok）。
- 处置（可见性提升，非崩溃面闭环）：自有关系层对同一面给显式信号
  ——严格拒（OPC 关系部件模式 ECMA-376 Part 2 词汇表：Relationships/
  Relationship only）/ 兼容警告 + 跳过（与引擎同等容忍 + MalformedXml
  警告可见化）。

## 9. 本轮暴露/登记总表

| 面 | 引擎裁决 | 暴露/登记 | 依据 |
|---|---|---|---|
| 超链接目标（读·正文） | 部分（rid 透传、无联接、内部目标丢） | **暴露**（自有 rels 层联接：External/Internal/Anchor/Unresolved 四变体） | §1 |
| 超链接（写·外部 URL） | 支持 | **暴露**（RunSpec.link + 写读全环） | §4 |
| 页眉/页脚（读·六槽位） | 支持 | **暴露**（default/first/even × header/footer，块序提取；first/even 激活设置不解释——变体透传） | §2 |
| 页眉/页脚（写·default 槽位） | 支持 | **暴露**（header/footer 单段 run 序） | §3 |
| 页眉/页脚（写·first/even 槽位） | 支持 | **登记不暴露**（未实测面） | §3 |
| 页眉内超链接目标（读） | 支持（页眉自身 rels） | **暴露**（per-part rels 联接） | §2 |
| 表格单元格内超链接目标（读） | 引擎解析、本模型扁平 | **登记**（TableData 扁平文本面，目标丢弃） | §2 |
| 脚注（读/写） | 读受限/写支持 | **登记不暴露** | §5 |
| 图片（读模型提取/写） | 字节面在/写支持 | **登记不暴露** | §6 |
| 域（读提取/写） | 字符串面在/写支持 | **登记不暴露** | §7 |
| .rels 词汇表敌意面 | 静默容忍（实证勘误——非 panic） | **严格拒 / 兼容警告+跳过**（自有层可见化） | §8 |
