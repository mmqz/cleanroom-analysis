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

## 11. r4-t2 写面资源上限核验（段/run 计数漏斗，2026-10-17；Round 4 T2 b 轮）

一手源同 §0。落地面 = mdcore `crates/office-io/src/docx.rs`
（`DocxError::Limit` + `MAX_PARAGRAPHS`/`MAX_RUNS` +
`check_document_scale`）+ docx.rs 内联 6 测试 + mdcore-wasm
`docx_limit` 码两处。零新依赖。

- **引擎无内建上限（一手核验——本轮资源限的存在依据）**：
  docx-rs 段落模型 = `Document.children: Vec<ParagraphChild>` 无界
  push（documents/mod.rs:158-160 add_paragraph 链式）；run 同构
  （run push 无计数/无校验）——写面规模唯一界 = 调用方自律。包装
  层补面：`DocxWriter` 段/run 计数（全部 14 个发射方法接线——
  heading/paragraph[含脚注内容段 +1段+1run]/table[每格]/六槽位
  页眉页脚/code_line/horizontal_rule[零 run]/quote/list_item/
  list_item_lines/image），`save_to_vec`/`save_to_path` 统一漏斗
  typed 拒（发射方法不可变签名 = 既有 API 契约，逐调用拒结构性
  不可用——漏斗 + `check_document_scale` 消费方预检共用单点比较）。
- **读面族分化（v1.2）**：docx 读路径此前误用 xlsx 族 300MB 声明
  常量——本轮 `prescan_docx` 对齐 50MB（office-b checkInputLimits
  格式族分档）；输入字节上限 52MB（=50MB+2MB 边距）四入口统一，
  path 入口元数据检查先于 `fs::read`（原无前置 = 真实内存面）；
  bytes 入口去 `to_vec` 复制（全借用化——大档双倍峰值消除）。
- **计数接线准确性锚**：`docx_paragraph_run_counting_wiring`
  （14 发射面 → 14 段/17 run 逐位断言）；边界锚 =
  `document_scale_limit_boundaries`（恰=限过/段+1/run+1 拒）；
  **超限导出零产物**锚 = `docx_save_funnel_rejects_over_scale_
  no_artifact`（save_to_path 错误路径零文件——fs::write 只在成功
  save 后执行）。
- **基准（examples/bench_limits.rs，沙箱 2CPU/3GB）**：100k 段 =
  801ms / 390.0MiB（≈3.9KB/段——引擎模型重量）；1M 段（恰=
  MAX_PARAGRAPHS）**沙箱 OOM kill（exit 137）**——线性外推
  ≈3.9GB：限值为防御上界，实际包络受宿主内存约束（未实测不宣称
  可达）。SECURITY_POLICY v1.2 §3.1。

## 12. r4-t3 增量：numPr/ind 读面承载 + 保真矩阵单一真源

- **numPr 读面**（完整承载）：`ParagraphNumbering{num_id, ilvl, kind,
  format}`——NumberingIndex 解析链（numId→num 桥→abstractNum→ilvl 级
  numFmt）一手核验引擎全公开字段面；numId=0 去编号惯例；悬空 numId
  严格拒/兼容 BrokenReference 警告 + 结构承载；numStyleLink/超定义域
  软警告。
- **ind 读面**（完整承载）：`ParagraphIndent{left, right, special:
  FirstLine/Hanging}`——reader 全属性（left/start/right/end/hanging/
  firstLine）；Chars 单位变体登记不承载。
- **降级可诊断**：format 字段承载原生 numFmt（lowerLetter 等）——写回
  经自有抽象（100/101 decimal/bullet）的降级显式化。
- 引擎面私有性一手核验（本轮证据）：Bold.val/Strike.val/Underline.val/
  RunFonts/ParagraphBorders 六槽均无私有 getter——presence-only 承载会
  对显式关闭形态（w:val="none"/"false"）误报，宁缺毋错，登记不承载。
- **保真矩阵单一真源**：`docx-r1/FIDELITY.md`（24 项五态 + 方案论证 +
  往返证据）——本轮起 DOCX 读写保真宣称以该文件为准。

## 13. r5-t2 增量：超链接锚点写回评估（承载矩阵 + 最小接口变更方案 + 裁决点）

> 任务边界：**评估止于方案与裁决点**——RunSpec/DocxWriter 为 office-io
> 公开模型面，r4-t3 起登记「另行裁决」（FIDELITY §3），本轮不实施任何
> 接口变更；交付 = 5 只增探针测试（tests/docx_roundtrip.rs 28→32 +
> mdcore-wasm/docx_export.rs +1）+ 本节。全部断言一手亲跑（未实测
> 不宣称）。
>
> **裁决落定（2026-10-17，批次 2）**：用户裁 **Option A，Phase 1+2
> 全批**（含书签写面正交件）——实施批 = mdcore `c4856f7`（原隔离
> 分支 office-g-phase1-anchor，ff 合入 main 全门禁绿：G1 538/538 /
> clippy 双面恰 2 wasm_bindgen 基线 / fmt 0 / scan PASS / lock 零
> diff / wasm32 check 过；合并前置检查：merge-base=main HEAD、main
> 独有提交 0、差异面 4 文件 +398/-3 全在 office-io+mdcore-wasm 零
> 公开模型触碰）。§13.4 裁决问题 1/2 就此闭环；问题 3 陈述维持
> （Internal 写回引擎不可达登记 / HYPERLINK 域红线 / MarkSpan URL
> 维持 mdcore MD 域门控 = DECISIONS #10）。

### 13.1 承载能力矩阵（证据锚点）

> 时点注记：下表「现状」列 = **r5-t2 评估时点**（2026-10-17）——
> 其中「RunSpec 链接表达 External-only」「DocxWriter 锚点/书签方法
> 无」两行已被批次 2 裁决翻转（c4856f7 合入，见 §13 头部裁决落定
> 注记）；其余行为面（读面四变体/书签静默丢/引擎私有面）维持。

| 面 | 现状 | 证据锚点 |
|---|---|---|
| RunSpec 链接表达 | **External URL only**（`link: Option<String>` → 引擎 `HyperlinkType::External` 单点发射） | docx.rs:2216-2219 字段 / 2287 link() / 2819-2822 发射点 |
| DocxWriter 锚点/书签方法 | **无**（19 公开方法全列无一面） | docx.rs:2368-2759 |
| 读面 External | 完整（自有 rels 联接 + `&` 实体补偿） | resolve_link docx.rs:1658-1698；hyperlink_roundtrip_external |
| 读面 Internal | 完整（自有 rels 层全量收条目；**引擎自身只收带 TargetMode 的关系**——本层补面） | 探针① r5t2_hyperlink_internal_target_read（整包面）+ hyperlink_resolution_dispatch（函数面） |
| 读面 Anchor | 完整（字符串透传；**目标存在性读面不可辨**——引擎无书签存在性校验） | hyperlink_anchor_read + 探针②③ |
| 读面 Unresolved | 兼容 only（严格拒 + 兼容 BrokenReference 警告） | hyperlink_dangling_strict_rejects_compat_unresolved |
| 读面书签定义 | **静默丢弃**（引擎解析进模型，本层 extract_paragraph 跳过——无字段、零警告、文本零损） | docx.rs:1912-1913 + 探针③ |
| 读面锚点优先序 | `w:anchor` 优先于 r:id（同元素双属性 → Anchor，可解析 r:id 被忽略） | 引擎 reader/hyperlink.rs:14-26 + 探针② |
| 引擎写面 Anchor | **在**（`Hyperlink::new(name, HyperlinkType::Anchor)` → `w:anchor` 属性形，无 r:id；**零 rels 面**——update_dependencies 只收 External 进 hyperlink_map；无书签存在性校验——悬空锚点合法形） | hyperlink.rs:42-44/107-118 + documents/mod.rs:1014-1017 + **探针⑤ 整包实证** |
| 引擎写面书签 | **在**（`Paragraph::add_bookmark_start(id,name)`/`add_bookmark_end(id)`；`Docx::add_bookmark_start`；XML = `<w:bookmarkStart w:id w:name/>` 对） | paragraph.rs:224-234 + mod.rs:556-563 + **探针⑤ 整包实证** |
| 引擎 HYPERLINK 域 | **todo!() 崩溃面——红线维持绝不暴露**（w:anchor 属性形与域形两条不相交路径，方案不触域形） | instr_text.rs:32；FIDELITY 矩阵行 11 |
| web 导入链（锚点→MD） | `#anchor` GFM 渲染在位 | docx_import.rs:175-183 + link_variants_render |
| web 导入链（MD→编辑器树） | **结构丢失、字面文本存活**（mdcore 行级解析不产行内 marks——`[t](#a)` 入树 = 段落字面文本） | markdown.rs:8 模块文档 + 探针④ |
| web 导出链 | 链接不可表达（MarkSpan 无 URL 载荷——mark "link" → 纯文本；字面文本路径同样零 `w:hyperlink`） | tree.rs:41-45 + docx_export.rs:124-135 + 探针④ |
| office-io 读→写回放示范 | **External only**（runs_to_specs 只映射 External——Anchor/Internal/Unresolved 静默丢弃） | tests/docx_roundtrip.rs:2071-2081（r4t3_roundtrip_read_modify_write 内） |

### 13.2 缺口分析（锚点写回需要什么）

1. **run 级锚点表达**：RunSpec 无锚点字段——读面四变体中 Anchor 在
   写侧无对应面（对称缺口：读 4 / 写 1〔External〕）。
2. **书签定义写面**（锚点的落点）：无 DocxWriter 方法。无书签定义的
   锚点链 = 悬空（合法 OOXML，Word 点击报「书签未定义」）——「锚点
   写回」若只做链接不做落点 = 半成品（但落点本身可独立裁决：TOC/
   heading 书签 `_Toc` 族是引擎 TOC 面的内部机制，非本面必需）。
3. 引擎面两者**全部公开可用**（探针⑤ 整包实证：锚点发射/零 rels/
   书签对/引擎写→自有读全环）——实施面 = 纯包装层胶水，**无**
   numbering 式懒注册/rels 接线陷阱（t20），**无** HYPERLINK 域
   todo!() 面（t14 红线，w:anchor 属性形不相交）。
4. 上游依赖链（诚实边界）：**web 端到端 TODAY 不可达**——即使
   RunSpec 扩锚点，docx_export 无源可映射（MarkSpan 无 URL/锚点载荷
   〔mdcore MD 域暂缓项〕+ markdown 行级解析不产链接 mark〔同域〕）；
   消费者 TODAY = office-io 直接用户（读→改→写回放示范可修对称缺口）
   与未来 web 链（MarkSpan URL 裁决落地后）。

### 13.3 选项（最小接口变更方案）

**Option 0 现状维持（零改动）**
- 锚点写回维持 FIDELITY §3 登记；读面四变体 + 写面 External-only
  不变。消费链无恶化（web 链今天本就无任何链接导出能力）。
- 成本 0；收益 0（对称缺口 + 回放静默丢面维持）。

**Option A 最小加性扩（推荐）**
- API 形（exact）：
  ```rust
  pub struct RunSpec {
      // …既有字段零变化…
      pub link: Option<String>,      // 外部 URL（既有）
      pub anchor: Option<String>,    // 新增：文档内锚点名（w:anchor）
      // footnote/field 既有
  }
  impl RunSpec {
      pub fn anchor_link(text: impl Into<String>, anchor: impl Into<String>) -> Self;
  }
  // 写面发射点单分支扩（add_spec_to_paragraph）：
  //   anchor.is_some() → Hyperlink::new(anchor, HyperlinkType::Anchor)
  //   优先序：footnote > field > anchor > link（与探针② 读面优先序
  //   镜像——anchor 与 link 同 Some 时 anchor 赢，文档登记）
  ```
- 可并入正交件（书签定义写面）：
  ```rust
  impl DocxWriter {
      /// 书签包裹段（bookmarkStart … runs … bookmarkEnd；id 由写器
      /// 计数器唯一化——引擎不校验，包装层保证）。
      pub fn bookmark_paragraph(&mut self, runs: &[RunSpec], name: &str);
  }
  ```
- 兼容性：**加性非破坏**（循 r4-t1 CellData / r4-t3 ParagraphNumbering
  Option-字段先例）——全部既有构造器恒 `anchor: None` 零漂移；struct
  literal 位点 = office-io 内部 4 处（code_line 用 `..base` 语法不动 +
  内联测试 3 处机械补）；外部消费者（docx_export.rs / docx_import.rs
  build_sample_docx / gen_docx_fixture.rs / docx_roundtrip.rs 测试）全
  构造器面零改动（grep 全量核实）。
- 测试范围：docx_roundtrip 锚点写读全环（探针⑤ 断言集移植产品面）+
  书签 XML 面 + 悬空锚点写（合法形断言）+ id 唯一性（两书签段）+
  `runs_to_specs` 回放示范扩 Anchor 变体（修 §13.1 末行静默丢面）+
  RunSpec 构造器零漂移断言（run_spec_constructors 扩）。
- 保真边界：office-io 层「锚点链接读→写回放等值」可达；**web 端到端
  不可达**（§13.2-4 诚实维持）；Internal 变体写回**引擎结构性不可达**
  （rels 发射面 external-only——documents/mod.rs:1210 "Now support
  external only"，登记维持）。

**Option B 类型化单字段（不推荐）**
- API 形：`pub enum LinkSpec { External(String), Anchor(String) }` +
  `RunSpec.link: Option<LinkSpec>`——类型安全（link/anchor 互斥编码
  进类型）。
- 兼容性：**破坏性**（`link: Some(String)` 字面构造全部失效——外部
  虽只有 office-io 内部测试用 struct literal，但公开字段类型变更 =
  公开 API 语义变更）；churn 大于 A，能力增量 = 0。

**Option C writer 方法面 only（不推荐）**
- API 形：`DocxWriter::anchor_paragraph(runs, anchor)`（不改 RunSpec）。
- 缺陷：段落级整段包裹语义 ≠ run 级粒度（同段普通 run + 锚点 run +
  外链 run 混排不可表达）；与读面 RunData.hyperlink run 级模型不对称；
  header/footer/list_item/quote 段落族需各复制一遍方法（面爆炸）。

### 13.4 推荐与裁决点

**推荐：Option A**（含正交件书签写面可同批或分离）——理由：加性
非破坏先例在位 / 引擎面探针⑤ 一手去风险 / 修读→写回放静默丢面 /
web 端到端收益诚实为 0（MarkSpan URL 裁决前无 web 消费者——若用户
优先级是 web 链接端到端，应先裁决 mdcore MD 域，office-io 扩面自然
排后到那时一并做）。

**裁决问题（T2.4 STOP 点——主代理呈用户）**：
1. office-io 锚点写面是否现在扩？（A：库级对称 + 回放保真，web 暂无
   消费者 / 0：维持登记，待 MarkSpan URL 裁决后与 web 链一并做）
2. 若 A：书签定义写面（`bookmark_paragraph`）同批 or 分离 or 不做
   （接受悬空锚点=合法形的半成品边界）？
3. （陈述性，无需决策）Internal 变体写回引擎结构性不可达，登记维持；
   HYPERLINK 域红线维持；MarkSpan URL / 行内链接解析 = mdcore MD 域
   既有暂缓项，本评估不触碰。

### 13.5 探针清单（本轮交付证据基座）

| # | 测试（文件） | 钉定行为 | 结果 |
|---|---|---|---|
| ① | r5t2_hyperlink_internal_target_read（office-io） | 无 TargetMode 关系 → Internal（整包面；引擎自身丢此面、自有 rels 层补面） | PASS |
| ② | r5t2_hyperlink_anchor_wins_over_rid（office-io） | w:anchor 优先于可解析 r:id（双属性形零警告） | PASS |
| ③ | r5t2_bookmark_definitions_dropped_anchor_carried（office-io） | 书签定义静默丢弃（零警告+文本零损）；锚点目标存在性读面不可辨（同形 Anchor 透传） | PASS |
| ④ | r5t2_markdown_link_literal_text_not_hyperlink（mdcore-wasm/docx_export） | 链接形态文本导出：零 w:hyperlink + 字面逐字保留 + 重导入文本零损（文本层往返成立/超链接层不成立） | PASS |
| ⑤ | r5t2_engine_anchor_and_bookmark_write_spike（office-io） | 引擎直构：w:anchor 属性形 + 零 rels 面 + 书签对发射 + 引擎写→自有读全环（Option A 去风险） | PASS |

门禁（r5-t2 全量亲跑）：G1 525→**530/0/1i**（+5 探针）；PDF 四面
68/79/81+6i/109+6i 零漂移 + perf 6/6；clippy native 恰 2 基线 + PDF
三面 0 + wasm32 --all-targets 2；fmt 0；scan 0；wasm32 check+
--all-targets 过；Cargo.lock 零 diff；web 七套件 18/15/28/11/13/9/9
全绿（零漂移——本轮无 wasm/web 产物变更）。
