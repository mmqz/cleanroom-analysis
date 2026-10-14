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

**裁决（r1）：读面引擎受限（登记），写面引擎支持（不暴露）**
**→ r2 收口：写面已暴露（RunSpec::footnote）；读面锚点精确化——见 §10.1。**

- 读：`read_docx` **从不读 word/footnotes.xml**（read_docx.rs 全文无
  footnote 部件读取）；正文 run 级 `w:footnoteReference` 亦不解析
  （reader/run.rs 无 FootnoteReference 分支；RunChild::FootnoteReference
  为写侧变体，documents/elements/run.rs:45）。
- 写：`Run::add_footnote_reference(Footnote)`（documents/elements/run.rs:366）
  → footnotes.xml + has_footnotes content type（documents/mod.rs:927-932）。

## 6. 图片（第二轮候选——快评）

**裁决（r1）：读面包级字节面在（模型级正文提取面外，登记），写面引擎支持（不暴露）**
**→ r2 勘误+收口：`Pic::new_with_dimensions` 不受 image 特性门控——PNG-only 写面零依赖暴露 + 容器级媒体清单读面暴露——见 §10.2。**

- 读：media 关系逐条读原始字节进 `Docx.images: Vec<(rId, path, Image, Png)>`
  （read_docx.rs:382-384, 403-423；documents/mod.rs:167-173）；PNG 直通
  预览，非 PNG 无 `image` 特性时预览为空（我们 default-features=false，
  documents/mod.rs:291-327）。正文 Drawing 入模型（reader/run.rs:100-102）
  但模型级提取面外。
- 写：`Run::add_image(Pic)`（documents/elements/run.rs:230）。

## 7. 域（第二轮候选——快评）

**裁决（r1）：读面指令文本可入模型字符串（提取面外，登记），写面引擎支持（不暴露）**
**→ r2 收口：简单域 PAGE/NUMPAGES 写面暴露；其余域登记（含引擎 todo!() 崩溃面）——见 §10.3。**

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

## 10. docx-r2 增量核验（脚注/图片/域/first-even 收口轮，2026-10-16；t14）

一手源同 §0（锚点均为 docx-rs-0.4.22/src 内 file:line）。落地面 =
mdcore `crates/office-io/src/docx.rs`（docx-r2 增量）+ tests/
docx_roundtrip.rs。零新依赖（Cargo.lock 零 diff）。

### 10.1 脚注

- **写面（暴露）**：`Run::add_footnote_reference(Footnote)` push
  `RunChild::FootnoteReference`（documents/elements/run.rs:363-369）且
  **重置 run_property 为 FootnoteReference 字符样式**——marker run 的
  bold/italic 被引擎丢弃（包装层诚实登记：RunSpec.footnote 的样式
  域值忽略）；`Footnote::new()` id 来自**进程级原子计数器**（起始 1，
  documents/footnote_id.rs:5-13——同进程多档 id 连续，包内自洽即可）。
  打包**自动收集**：prepare_package → collect_document_part →
  `self.footnotes.add(...)` + content type add_footnotes +
  document_rels.has_footnotes（documents/mod.rs:914-938）→ zipper 恒写
  word/footnotes.xml（zipper/mod.rs:195）+ rels rId8（document_rels.rs:
  87-92）。往返验证 = XML 级断言（脚注内容 + marker 引用 id 配对 +
  rels + content types；读面受限——不依赖读面）。
- **读面（引擎受限，登记——r1 §5 锚点精确化）**：read_docx.rs 全
  流程只读 [Content_Types].xml / _rels/.rels / 主文档 / headers /
  footers / themes / settings / webSettings / images / hyperlinks——
  **无 footnotes 部件读取**；body `w:footnoteReference` 无解析臂
  （reader/run.rs:63-146 match 无该分支 → XMLElement::Unsupported
  静默忽略）。产品面：读回文本不含脚注内容（模型级钉定）。

### 10.2 图片

- **写面（勘误 → 暴露 PNG-only）**：r1 §6 初判「写面需 image 特性」
  **不成立**——仅 `Pic::new(buf)`（自动转 PNG + 尺寸推断）被
  `#[cfg(feature = "image")]` 门控（documents/elements/pic.rs:53）；
  **`Pic::new_with_dimensions(buf, w_px, h_px)` 无门控**（pic.rs:66）。
  媒体部件恒 `media/{rid}.png` 命名（image_collector.rs:137 +
  zipper/mod.rs:118 注释「For now only png supported」）+
  [Content_Types].xml 恒含 png/jpeg/jpg/bmp/gif Default（content_types.rs:
  160-168）→ **PNG-only 面零新依赖暴露**（DocxWriter::image：PNG
  签名 + 正尺寸 typed 验证；EMU 换算引擎内建 from_px = px×9525）。
- **image 特性代价证据（register-only 的依据）**：/tmp 对照
  cargo tree（docx-rs default vs default-features=false）闭包
  37 → 60 crate，**+23 crate** = image v0.25.10 树：adler2/autocfg/
  bitflags/bytemuck/byteorder-lite/color_quant/fax/fdeflate/gif/half/
  image/miniz_oxide/moxcms/num-traits/png/pxfm/quick-error/tiff/weezl/
  zerocopy/zerocopy-derive/zune-core/zune-jpeg。`Pic::new` 自动转码
  面登记不启用。
- **读面（容器级清单，暴露——自有实现非引擎面）**：`DocxDocument
  .images: Vec<ImageAsset{part_name, content_type, bytes, references,
  body_uses}>` = word/media/* 枚举 + [Content_Types].xml 解析（自有
  解析器，词汇表 Types/Default/Override——严格拒/兼容警告，同 rels
  层纪律）+ 全部 .rels 表 image 型关系联接 + run 级 `a:blip r:embed`
  计数（正文 + 六槽位；引擎读侧 Pic.id 透传 rid——reader/pic.rs:
  24-28）。**引擎面三个静默怪癖在档（清单不受影响——直读 zip）**：
  ① 无 image 特性时非 PNG/非 EMF 媒体在 previews=true 默认路径
  **静默丢弃**（documents/mod.rs:291-327 fall-through 不 push）；
  ② 悬空 image 关系 read_zip Err 静默跳过（read_docx.rs:404-423）；
  ③ 引擎 `Docx.images` 仅收 document rels 的 IMAGE_TYPE（页眉图片
  字节不入）。
- **敌意面纪律（与 r1 同族）**：悬空 image 关系（目标部件缺失/越根）
  = 严格拒/兼容 BrokenReference；媒体部件无内容类型声明 = 严格拒/
  兼容 MalformedXml 警告 + content_type=None 保留（OPC：每部件必须有
  内容类型）；悬空 r:embed = 严格拒/兼容 BrokenReference 不计数；
  TargetMode=External 型 image 关系（链接图）不入清单不警告（登记）；
  非 word/media/* 目标的 image 关系不联接（登记观察）；孤儿媒体
  （无关系引用）保留不警告。**zip 8.6 重复条目名**：by_name 最后
  条目胜（read/zip_archive.rs:49-51 IndexMap::insert 后胜；门 2 按
  by_index 验全部副本 io.rs:308）——与引擎读路径同语义；ZipWriter
  拒绝重名（write.rs:1380-1386）→ 重复名档案只能净室手造（测试内
  stored + 手写本地头/CD/EOCD，python zipfile 交叉验证一致）。

### 10.3 域

- **写面（暴露：简单域 PAGE/NUMPAGES）**：`Run::add_field_char(
  FieldCharType, dirty)` + `Run::add_instr_text(InstrText)`（run.rs:
  194-215；`add_tc` 即 begin/instr/end 三连先例 run.rs:202-208）。
  发射 `<w:fldChar w:fldCharType="begin" w:dirty="false"/>`（closed!
  宏双属性恒发 xml_builder/elements.rs:176）+ `<w:instrText>PAGE
  </w:instrText>`（open! 宏无 xml:space——简单域无空白面）。
  RunSpec::field_page/field_num_pages（域 run 无缓存结果文本——域值
  是渲染期计算面，读回文本不含域值，模型级钉定）。
- **登记不暴露**：`InstrText::HYPERLINK` = `todo!()` **崩溃面**
  （documents/elements/instr_text.rs:32）——绝不暴露；TOC（文档级
  构造 + 展开）与 PAGEREF（书签面）域族在但未实测不暴露。
- **读面（维持登记）**：引擎区分结构（instrText → InstrTextString
  reader/run.rs:110-114；fldChar → FieldChar :105-108）但提取面外
  ——域结果文本（缓存值）以普通 w:t 呈现，域语义不入模型。

### 10.4 first/even 页眉页脚（写面）

- **引擎支持且激活语义完整**：`Docx::first_header` 计数分配 rId 且
  **自动置 titlePg**（section_property.rs:92-97 title_pg = true）；
  `Docx::even_header` 自动置 settings evenAndOddHeaders（mod.rs:
  613-616 → `<w:evenAndOddHeaders/>` xml_builder/elements.rs:424）；
  六 API 共享 header_count/footer_count 计数器（mod.rs:576-616——
  同槽重复调用计数递增 = 自产悬空引用，与 default 槽同怪癖）。
- **包装层原位替换（暴露）**：first_header/even_header/first_footer/
  even_footer 四方法；同槽重复调用 = 原位替换**保持原槽 rid**
  （rels 计数不变 + 无跨槽碰撞）。
- **r1 修复（开发期发现）**：r1 的 replace_default_header 无条件
  `create_header_rid(1)`——在 r1 暴露面（仅 default 槽）不可达；暴露
  first/even 后可达（先 first 后 default 时 default 槽实为 rIdHeader2，
  无条件替换会踩踏 rIdHeader1 错指 first 内容）→ r2 改为**保持原槽
  rid**（集成测试钉定该序列）。
- **部件序语义（登记观察）**：headers/footers 部件按 rid **字典序**
  排序后编号（mod.rs:798-800 `sort_by`）——>9 个头才乱序（本面单节
  3 槽上限不可达）。

### 10.5 r2 暴露/登记总表

| 面 | 引擎裁决 | 暴露/登记 | 依据 |
|---|---|---|---|
| 脚注（写） | 支持（marker 样式重置 + 自动收集） | **暴露**（RunSpec::footnote；样式面忽略登记） | §10.1 |
| 脚注（读） | 引擎受限（无部件读取 + 无引用解析臂） | **登记**（读回文本不含脚注内容） | §10.1 |
| 图片（写·PNG 显式尺寸） | 支持（new_with_dimensions 无特性门控——**r1 勘误**） | **暴露**（DocxWriter::image；PNG 签名/正尺寸 typed 验证） | §10.2 |
| 图片（写·自动转码） | 需 image 特性（+23 crate） | **登记**（代价证据在档） | §10.2 |
| 图片（读·容器级清单） | 引擎面三静默怪癖（清单绕开） | **暴露**（DocxDocument.images 五字段；块序内图片位置仍登记） | §10.2 |
| 域（写·PAGE/NUMPAGES） | 支持（fldChar/instrText 元素族） | **暴露**（RunSpec::field_page/field_num_pages） | §10.3 |
| 域（写·TOC/PAGEREF） | 支持 | **登记**（未实测不暴露） | §10.3 |
| 域（写·HYPERLINK） | `todo!()` 崩溃面 | **登记绝不暴露** | §10.3 |
| 域（读） | 结构面在、提取面外 | **登记**（域值 = 渲染期计算） | §10.3 |
| first/even 页眉页脚（写） | 支持（激活语义完整） | **暴露**（四方法 + 原位替换保 rid——r1 踩踏修复） | §10.4 |
| [Content_Types].xml 解析 | 引擎 FromXML 宽容（depth==1 全收） | 自有解析器词汇表纪律（严格拒/兼容警告） | §10.2 |
