# docx-r1 增量第一轮报告（t13，2026-10-15）

> 主题：**OPC 关系解析层 + 该层解锁的面**（遗留①：DOCX 切片按需
> 增量面——引擎能力在，未实测不暴露）。
> 前序：xlsx-r2 Phase 3（460f302，模型级切片）。
> 底稿：`office-line/docx-r1/CAPABILITY_NOTES.md`（一手源码锚点）。
> 纪律：未实测不暴露；引擎不支持面登记为 engine-limited 不暴露。

## 1. Phase A 能力核验裁决（docx-rs 0.4.22 一手源读）

| 面 | 引擎裁决 | 暴露/登记 | 关键锚点 |
|---|---|---|---|
| 超链接目标（读·正文） | **部分**：body 元素透传 r:id（`HyperlinkData::External{rid, path:""}`——path "not used"）；包级 `Docx.hyperlinks` 仅收 TargetMode 存在的关系（内部目标丢）；**无 r:id→target 联接** | **暴露**（自有 rels 层联接） | reader/hyperlink.rs:13-26；read_docx.rs:386-398；documents/mod.rs:175 |
| 超链接（写） | **支持**（"Now support external only"——rels 发射带 TargetMode="External"） | **暴露** | documents/elements/hyperlink.rs:21-45；documents/mod.rs:1209-1214；documents/document_rels.rs:124-130 |
| 页眉/页脚（读） | **支持**：header*/footer* 部件解析挂 section_property 六槽位；**引用在而挂接失败 = 静默跳过**（filter_map None） | **暴露**（静默跳过 → 严格拒/兼容警告） | read_docx.rs:7-49,140-141,232-319；reader/header.rs:10-40 |
| 页眉/页脚（写） | **支持**（`Docx::header` 六 API + rIdHeader{n}/rIdFooter{n} + content type + 部件发射；**重复调用计数递增 → 自产悬空引用**） | **暴露**（default 槽位；重复调用 = 包装层原位替换；first/even 写面登记不暴露） | documents/mod.rs:576-601；zipper/mod.rs:79-97 |
| 脚注 | 读**引擎受限**（从不读 footnotes.xml + body `w:footnoteReference` 不入模型）；写支持 | **登记不暴露** | read_docx.rs 全文；reader/run.rs（无 FootnoteReference）；documents/elements/run.rs:366 |
| 图片 | 读 = 包级字节面（`Docx.images`，PNG 直通[无 image 特性]）；写支持 | **登记不暴露** | read_docx.rs:382-384,403-423；documents/elements/run.rs:230 |
| 域 | 读 = `RunChild::InstrTextString(String)` 字符串面；写元素族全 | **登记不暴露** | reader/run.rs:110-114；documents/elements/fld_char.rs 等 |
| .rels 词汇表敌意面 | **静默容忍**（初判 unwrap panic 经实证**勘误**——`from_str` catch-all `Ok(Unsupported)` 永不 Err） | 严格拒 / 兼容警告+跳过（可见化） | §8（勘误链在档） |

## 2. 落地面（mdcore crates/office-io）

### 2.1 自有 OPC 关系解析层（src/docx.rs）

- **解析面**：全部 `.rels` 部件（zip 条目扫描 + 逐部件 quick-xml 拉取
  解析）；产出 = document 主关系表（rId → {Type, Target, TargetMode}）
  + 按属主部件路径索引的 per-part 表（页眉/页脚自身 rels）+ 全包条目
  名集（悬空目标判定）。
- **纪律（门 3 同族）**：
  - 上限 `LIMIT_RELATIONSHIPS = 10_000` **恒等两模式**；
  - 词汇表（ECMA-376 Part 2：Relationships/Relationship only）：
    严格拒 / 兼容警告+跳过（引擎同等容忍 + 可见化）；
  - schema 违反（缺 Id/Type/Target、重复 Id）：严格拒 / 兼容警告 +
    跳条目/首胜；
  - 悬空引用（超链接 r:id 无条目 / 页眉页脚引用无条目 / 目标部件
    缺失 / 目标路径越出包根[词法归一化检出]）：严格拒 / 兼容
    BrokenReference 警告（新增 `io::WarningKind::BrokenReference`
    共享类别——XLSX 面无使用点，纯枚举增量零行为波及）；
  - 属性值实体补偿：`decode_and_unescape_value`（`&amp;` → `&`——
    XLSX v2 IO 边界补偿同构）；
  - 降级去重：兼容模式下 .rels 解析失败不重复登记（门 3 已警）。
- **管线位置**：门 1/2/3 → rels 层（先于引擎）→ 引擎模型解析 →
  联接提取。字节入口同管线（零门绕过）。

### 2.2 读面增量

- `RunData.hyperlink: Option<HyperlinkTarget>`（四变体：External
  {target}/Internal {target}/Anchor {anchor}/Unresolved {rid}[仅兼容]）
  ——同一超链接内各 run 共享目标；正文与页眉/页脚段落同构提取；
- `DocxDocument.headers/footers: Vec<HeaderFooterData>`（六槽位中
  实际挂接的：variant[Default/First/Even 透传] + blocks[与文档体
  同构]）；页眉内超链接经页眉部件自身 rels 联接（净室自构造测试）。

### 2.3 写面增量

- `RunSpec.link: Option<String>`（+ `RunSpec::link(text, url)` 构造
  器）——外部 URL run（引擎关系面发射）；
- `DocxWriter::header/footer(&[RunSpec])`——default 槽位单段；
  **重复调用 = 原位替换**（包装层防御：引擎按调用计数递增 → 重复
  调用发射 rIdHeader2 关系而无部件 = 自产悬空引用；replace_default_
  header/footer 保持 rIdHeader1 + rels 计数不变——与 XLSX 流式纪律
  的包装层 typed 拦截同族）。

### 2.4 登记面变化（docx.rs 模块文档 = 唯一真源随码走）

- **闭项**：超链接目标丢弃（Phase 3 登记）→ 正文/页眉面已暴露；
- **仍登记**：表格单元格内超链接目标（TableData 扁平面）、脚注/
  尾注（读面引擎受限 + 写面未实测）、图片/域/修订/批注/编号语义、
  first/even 页眉页脚写面、页眉页脚内表格写面、锚点写面（引擎在，
  未实测不暴露）。

## 3. 测试（+19：docx.rs 单测 2→12，docx_roundtrip 5→14）

单测（docx.rs）：rels 基本解析（含 &-实体补偿）/ 词汇表严格拒+兼容
跳过 / 错根严格拒+兼容容忍 / 缺属性严格拒+兼容跳条目 / 重复 Id 严格
拒+兼容首胜 / 垃圾截断严格拒+兼容降级 / 垃圾怪名（quick-xml 宽名面）
/ 属主路径推导 / 包内路径归一化+越根检出 / 联接分派（四变体+悬空双
模式）/ heading 校验 / RunSpec 构造器。

集成（docx_roundtrip.rs 新 9）：
1. `hyperlink_roundtrip_external`——外链写读全环（含 `&`-URL 实体
   补偿链 + run 边界[链接后普通 run 无目标]）；
2. `hyperlink_anchor_read`——w:anchor 锚点读（净室自构造）；
3. `hyperlink_dangling_strict_rejects_compat_unresolved`——悬空
   r:id：严格 typed 拒 / 兼容 Unresolved+警告+文本保留；
4. `header_footer_roundtrip`——页眉页脚全环 + 重复调用替换语义 +
   变体透传 + 文档体不受影响；
5. `header_reference_hostile_strict_compat`——页眉引用敌意三面
   （r:id 无条目 / 目标部件缺失 / 路径逃逸 `../../`）× 严格/兼容；
6. `rels_duplicate_id_in_package_strict_compat`——整包重复 Id
   （rId 动态定位不硬编码）：严格拒 / 兼容首胜可观测；
7. `rels_unknown_element_in_package_strict_compat`——整包词汇表：
   严格拒 / 兼容警告+数据照常；
8. `engine_rels_unknown_element_tolerated`——引擎容忍面直探针
   （CAPABILITY_NOTES §8 勘误实证：整包直调返回 Ok，不崩溃）；
9. `hyperlink_inside_header_resolved_via_header_rels`——页眉内
   超链接经页眉部件自身 rels 联接（净室自构造：改写 header1.xml +
   原位改写 header1.xml.rels——写器恒发射空页眉 rels）。

## 4. 门禁表（全部实测，mdcore main 460f302 + 本轮工作树）

| 门禁 | 声明 | 实测 | 结论 |
|---|---|---|---|
| G1 `cargo test --workspace` | 392+19 | **411 passed / 0 failed / 1 ignored**（+19） | PASS |
| PDF lopdf 面 | 79 | 79/0/0 | PASS（零波及） |
| PDF pdfium 面 | 81+6i | 81/0/6i | PASS（零波及） |
| PDF dual 面 | 109+6i | 109/0/6i | PASS（零波及） |
| clippy default | 仅 2 wasm 基线 | 恰 2（mdcore-wasm lib.rs:188/:197 static_method_of） | PASS |
| clippy 3 PDF 面 | 0 | 0/0/0 | PASS |
| `cargo fmt --all -- --check` | 0 diff | 0 diff | PASS |
| scan product / --all | 0 hits | 0 / 0（20 terms） | PASS |
| wasm32 check | pass | pass（office-io 不进 wasm 闭包） | PASS |
| web contract_gate | 18 | 18/18 PASS exit 0 | PASS |
| web web_smoke | 15 | total 15 / failed 0 / pass true | PASS |
| web ime_cdp | 28 | total 28 / failed 0 | PASS |

依赖：**零新增**（quick-xml 0.41 直依赖第二使用面 + zip 8.6 原位；
Cargo.lock 零变化）→ NOTICES v1.5 不动（无合规基线提交）。

## 5. 开发期发现登记（后续轮候选）

1. **引擎写面怪癖**：`Docx::header` 重复调用计数递增 → 自产悬空
   引用（包装层已防御；若暴露 first/even 写面须同族防御）；
2. **写侧实体链**：`Hyperlink::new` escape_owned 单次转义 +
   XMLBuilder attr 原样写字节（xml/writer.rs:237-244 不转义）=
   文件内单次转义——读回单次反转义正确（若引擎升级改双转义，
   `&`-URL 全环测试首跑拦截）；
3. **兼容模式重复 Id 分歧**：本层首胜 vs 引擎 BTreeSet+HashMap
   后胜（仅影响重复 Id 的页眉目标归属——已警告面，登记不改）；
4. 第二轮候选（引擎能力在档）：脚注写面 / 图片面（读包级字节 +
   写面）/ 域面 / first/even 页眉页脚写面——各需独立实测轮。

## 6. Round 2（docx-r2，2026-10-16 / t14——遗留① 收口轮：脚注/图片/域/first-even）

> 主题：四面收口——写面脚注/简单域/PNG 图片/first-even 槽位 + 读面
> 容器级媒体清单。**零新依赖**（Cargo.lock 零 diff → NOTICES v1.5
> 不动）。底稿：CAPABILITY_NOTES §10（r2 一手核验 + 勘误链）。

### 6.1 Phase A 裁决（r2；含 r1 勘误）

| 面 | 引擎裁决 | 暴露/登记 | 关键锚点 |
|---|---|---|---|
| 脚注（写） | **支持**：`add_footnote_reference`（run.rs:363-369）+ 打包自动收集（mod.rs:914-938）+ zipper 恒写 footnotes.xml（zipper/mod.rs:195）；**marker run 样式被引擎重置**（FootnoteReference 字符样式） | **暴露**（RunSpec.footnote；样式域值忽略——诚实登记） | §10.1 |
| 脚注（读） | **引擎受限**：无 footnotes 部件读取 + 无引用解析臂（reader/run.rs:63-146） | **登记**（读回文本不含脚注内容） | §10.1 |
| 图片（写·PNG） | **支持——r1 勘误**：`Pic::new_with_dimensions` **不受** image 特性门控（pic.rs:66；仅 `Pic::new` 自动面被门控 pic.rs:53）；媒体恒 .png 命名 | **暴露**（DocxWriter::image：PNG 签名 + 正尺寸 typed 验证） | §10.2 |
| 图片（写·自动转码） | 需 image 特性 = **+23 crate**（image v0.25.10 树——cargo tree 对照证据在档） | **登记**（代价证据登记不启用） | §10.2 |
| 图片（读） | 引擎面三静默怪癖（非 PNG 默认路径丢弃 / 悬空关系跳过 / 仅 document rels——mod.rs:291-327 + read_docx.rs:404-423） | **暴露**（自有容器级清单绕开引擎面） | §10.2 |
| 域（写·简单域） | **支持**：fldChar/instrText（run.rs:194-215；add_tc 三连先例） | **暴露**（PAGE/NUMPAGES；域值 = 渲染期计算，读回无域文本） | §10.3 |
| 域（写·其余） | HYPERLINK = `todo!()` 崩溃面（instr_text.rs:32）；TOC/PAGEREF 在 | **登记**（崩溃面绝不暴露；其余未实测不暴露） | §10.3 |
| 域（读） | 结构面在（InstrTextString/FieldChar）、提取面外 | **登记** | §10.3 |
| first/even 页眉页脚（写） | **支持且激活完整**（first 自动 titlePg section_property.rs:95；even 自动 evenAndOddHeaders mod.rs:613-616） | **暴露**（四方法 + 原位替换保 rid） | §10.4 |
| zip 重复条目名 | zip 8.6 by_name 最后条目胜（zip_archive.rs:49-51）；ZipWriter 拒重名（write.rs:1380-1386） | **语义钉定**（净室手造档案测试 + python zipfile 交叉验证） | §10.2 |

### 6.2 落地面（mdcore crates/office-io）

- **读面**：[Content_Types].xml 自有解析器（词汇表 Types/Default/
  Override——严格拒/兼容警告跳过；缺属性/重复条目同 rels 层分派）
  + **媒体清单** `DocxDocument.images: Vec<ImageAsset>`（五字段：
  part_name / content_type[Override>Default，无声明严格拒/兼容警告
  +None 保留] / bytes / references[全部 .rels 表 image 型关系联接，
  (owner,rid) 排序去重] / body_uses[正文+六槽位 a:blip r:embed 经属主
  表解析计数]）；敌意面 = 悬空关系/越根/悬空 embed → 严格拒 /
  兼容 BrokenReference（同 r1 纪律）；External 链接图与非 media 目标
  跳过（登记）；孤儿媒体保留不警告。
- **写面**：`RunSpec::footnote(note)`（marker + footnotes.xml 单段
  纯文本）、`RunSpec::field_page()/field_num_pages()`（三连展开）、
  `DocxWriter::image(png, w, h)`（PNG-only + EMU 换算）、
  `first_header/even_header/first_footer/even_footer`（原位替换
  **保持原槽 rid**——r1 无条件 rIdHeader1 在多槽组合下踩踏，修复 +
  集成测试钉定）。
- **模块文档登记表更新**：图片媒体面闭项（块序内图片位置仍登记——
  Block 无图片块）；脚注读面/域读面/其余域面维持登记。

### 6.3 测试（+15：docx.rs 单测 12→19，docx_roundtrip 14→22）

单测（+7）：RunSpec 构造器（footnote/两域）+ specs_to_paragraph 展开
（脚注 marker 引擎模型级[内容在脚注内+无文本 run] + 简单域三连[Begin/
End 配对 + instr 串]）+ 图片写验证（非 PNG/零尺寸 typed 拒 + Pic 尺寸
EMU 换算）+ [Content_Types].xml 四项（查表优先级/词汇表/缺属性+重复/
垃圾截断）。

集成（+8）：
1. `footnote_write_xml_level_roundtrip`——footnotes.xml 在位 + 双脚注
   内容 + document.xml 引用 id 配对（动态定位不硬编码）+ rels +
   content types + 读回文本不含脚注内容（引擎受限面钉定）；
2. `simple_fields_write_xml_level_roundtrip`——footer1.xml instrText
   PAGE/NUMPAGES + fldChar begin/end 计数 + 读回「第  页 / 共  页」；
3. `image_write_read_roundtrip`——模型级全环（字节逐位/内容类型/
   引用关系/body_uses=1）+ XML 级 blip rid 可解析；
4. `image_inventory_dangling_rel_strict_compat`——悬空 image 关系：
   严格拒 / 兼容 BrokenReference + 资产不增；
5. `image_inventory_undeclared_and_orphan_media`——未声明扩展
   （word/media/evil.tiff）：严格拒 / 兼容警告 + content_type=None
   保留；孤儿媒体两模式保留不警告；
6. `image_inventory_dangling_embed_strict_compat`——悬空 r:embed：
   严格拒 / 兼容警告 + 计数不增 + rels 侧引用不受影响；
7. `image_inventory_duplicate_zip_entries_last_wins`——净室手造
   重名 zip（测试内 CRC32 + 手写本地头/CD/EOCD；zip 8.6 ZipWriter
   拒重名）→ 最后条目胜（python zipfile 交叉验证一致）；
8. `first_even_header_footer_roundtrip`——六槽位 + titlePg/
   evenAndOddHeaders 激活 XML 断言 + rels 恰 3+3（替换不增计数）+
   三变体透传 + **踩踏序列钉定**（先 first 后 default 再替换 default
   → default 内容正确）。

### 6.4 门禁表（全部实测，mdcore main 6f74b71 + 本轮工作树）

| 门禁 | 声明 | 实测 | 结论 |
|---|---|---|---|
| G1 `cargo test --workspace` | 411+15 | **426 passed / 0 failed / 1 ignored** | PASS |
| PDF lopdf / pdfium / dual 面 | 79 / 81+6i / 109+6i | 79 / 81+6i / 109+6i（零漂移） | PASS |
| clippy default | 仅 2 wasm 基线 | 恰 2（自有 5 新警已修：2 doc_lazy_continuation + 2 map_entry + 1 type_complexity） | PASS |
| clippy 3 PDF 面 | 0 | 0/0/0 | PASS |
| `cargo fmt --all -- --check` | 0 diff | 0 diff | PASS |
| scan product / --all | 0 / 0 | 0 / 0 | PASS |
| wasm32 check | pass | pass | PASS |
| web contract / smoke / ime | 18 / 15 / 28 | 18 / 15 / 28（全过 0 failed） | PASS |

依赖：**零新增**（Cargo.lock 零 diff）→ NOTICES v1.5 不动（无合规
基线提交）。

### 6.5 开发期发现登记

1. **r1 勘误（重要）**：CAPABILITY_NOTES §6「图片写面需 image 特性」
   不成立——`Pic::new_with_dimensions` 无门控，仅自动转码面被门控；
   本轮据此暴露 PNG-only 写面（约束的本意 = 零新依赖，维持满足）；
2. **r1 潜在缺陷修复**：replace_default_header 无条件 rIdHeader1 在
   多槽组合下可踩踏（r1 暴露面不可达；暴露 first/even 后可达）→
   改为保持原槽 rid + 踩踏序列集成测试；
3. zip 8.6 重复条目名语义：by_name 最后条目胜 + ZipWriter 拒重名
   （读宽容/写严格）——重名档案须净室手造，python zipfile 交叉
   验证同语义；
4. 引擎读面三个图片静默怪癖（非 PNG 丢弃/悬空跳过/仅 document
   rels）——自有清单面绕开，登记在档。

## 7. Round 3（t16，2026-10-17——web 端到端消费切片：office-io 首个 wasm/web 消费边）

任务：DOCX 能力 → mdcore-wasm 导出面 → web 编辑器端到端集成——最小
真实可用切片（真调用 + 错误处理 + 测试 + 文档），非接口包装。

### 7.1 Phase A 裁决（切片选型 + 证据锚点）

- **选型 (A) DOCX 导入进编辑器**（备选 (B) 编辑器→DOCX 导出否决）：
  复用已硬化的读路径（容器三门敌意输入防线 + 兼容/严格双模式）+ 既有
  ingestion（`WasmCore::from_markdown` / web 宿主注入契约
  `window.__inject`→`installNewCore` 全量重建，main.ts:1388-1431），
  真实价值/表面积比最优；写面（B）需 MD→结构解析面，表面积更大。
- 证据锚点：mdcore `markdown.rs:152-169`（`|` 起始连续行合并单 Table
  块——**GFM 表受支持**，故 DOCX 表可映射 GFM 管道表而非降级文本）；
  `html.rs:1381` canonical_table_text（表头行 + `---` 分隔行 + 体行
  ——与导入映射同形）；lib.rs 纯函数先例 `parse_source_str`
  （JsValue 非 wasm 目标构造即 panic → 映射内核须纯原生可测）；
  REUSE.md §0「在 MD 文档域内消费……Rust 直调或**经 wasm 桥**」+
  §5 消费契约注记——**无禁止 mdcore-wasm→office-io 条款，三域
  纪律新边仅此一条（消费方依赖）**，裁决 = 兼容。可行性探针先行：
  wasm32 check + Cargo.lock 行为（下）。

### 7.2 落地面（mdcore）

- `crates/mdcore-wasm/Cargo.toml`：+`mdc-office-io = { path = "../office-io" }`
  （注释注明三域纪律唯一新边）；
- `crates/mdcore-wasm/src/docx_import.rs`（新增，模块文档 = 面/错误
  码表/映射规则/不导入面/已知边界的唯一真源随码走）：
  - 暴露面 = **自由函数** `import_docx(bytes: &[u8]) -> Result<String,
    JsValue>`（不碰 WasmCore 状态——既有类面零变化，加性暴露）；
    成功 JSON `{"markdown", "stats"}`；失败抛 JSON
    `{"code", "message"}`（typed 错误码，宿主 `JSON.parse(e)`）；
  - 错误码表（容器门拒 ≠ 引擎解析错误）：`container_prescan` /
    `container_corrupt` / `container_parse` / `container_invalid_part`
    / `docx_parse` / `io`；
  - 读模式 = **兼容**（可恢复畸形部件降级警告随 stats 返回；门 1/2
    与全部安全上限两模式恒等——敌意输入仍 typed 拒绝）；
  - 映射内核纯原生可测（`import_docx_core` 等 7 个纯函数零 JsValue）；
  - `#[doc(hidden)] pub build_sample_docx()`：夹具产线单一真源
    （office-io DocxWriter，examples/gen_docx_fixture.rs 生成
    web/tests/fixtures/import_sample.docx——提交入库 + 原生漂移
    守卫测试锁期望 markdown）。
- `web/index.html`（工具条：`import .docx` 文件入口 + `#docx-status`
  状态行）；`web/src/main.ts`（`docxImport` 薄包装：JSON 出入口 +
  异常→typed 结果；`__importDocx`/`__lastDocxImport` window 面；
  成功经既有 `__inject` 契约重建核，失败 = 状态行 typed 码 + 编辑器
  现状零触碰；`tsc --noEmit` 清洁）。

### 7.3 映射规则与不导入面（诚实边界——完整版 = docx_import.rs 模块文档）

- 标题：`Heading1`..`Heading9` → `#`×级（**7-9 钳 6**，MD 上限 h6）；
  Title/Subtitle/自定义样式 → 普通段落（模型级样式语义边界）；
- run：加粗 `**` / 斜体 `*`（标记只裹非空白核）；超链接
  `[text](target)`（URL 空/空白/`)`/`<>` → `<...>` 角括号形；锚点
  `#anchor`；Unresolved → 降级纯文本（BrokenReference 警告随
  stats）；**相邻同格式 run 合并**（`*a*`+`*b*` 直连 = CommonMark
  歧义串 `*a**b*`）；
- 表 → GFM 管道表（表头 + `---` 分隔行 + 体行；单元格 `|`→`\|`、
  `\n`/`\r`→空格、短行补齐；空表跳过）；
- 不导入面（计数进 stats）：图片（body_uses>0 资产；孤儿媒体不计）、
  页眉/页脚（实际挂接槽位数）、脚注（引擎读面受限登记原样继承）；
- 已知边界：段落文本以 MD 结构字符开头会被行级解析解释为对应块型
  （导入 = 结构映射非显示保真）；软换行（w:br）段落原样多行（mdcore
  行级建块拆多块）；内联转义字符不转义（mdcore 无内联转义层）；
  表格单元格内超链接目标随扁平文本丢弃（TableData 平面）。

### 7.4 测试证据（原生 9 + web e2e 9）

原生（mdcore-wasm，`cargo test --workspace` 计入；crate 2→11 测试）：
1. `writer_roundtrip_import_maps_structure`——写→读→映射全环**精确
   markdown 等值**（含 GFM 分隔行）+ 全 stats 字段；
2. `heading_styles_map_with_clamp`——Heading1/3/9 + Title/Normal/无样式；
3. `adjacent_same_format_runs_merge`——合并 + 纯空白 run 无标记直出；
4. `link_variants_render`——External 角括号/Anchor `#`/Unresolved 降级
   + hyperlinks 计数；
5. `table_cell_sanitization_and_padding`——`\|` 转义/换行→空格/短行
   补齐/空表跳过；
6. `skip_counts_cover_headers_footers_images`——孤儿媒体排除 +
   broken_reference kind 映射；
7. `garbage_bytes_typed_container_error`——空/垃圾/非 zip 三载荷
   → `container_corrupt`；
8. `truncated_docx_typed_container_error`——真档案半截 →
   `container_corrupt`；
9. `web_e2e_fixture_imports_expected_markdown`——入库夹具漂移守卫
   （含再生成指引）。

web e2e（`web/tests/docx_import.py`，新增第四套，9/9 PASS，端口
8933，夹具 26625 B）：
1. `import_ui_present`——UI 在场 + 契约面不变（启动零回归）；
2. `happy_import_file_input_exact_markdown`——**真实 file input 路径**
   （Playwright set_input_files → 原生 change → arrayBuffer → wasm →
   `__inject` 重建）→ 编辑器 markdown **精确等值**（核心树规范形：
   空行 ingestion 时跳过，块间单 `\n`——首跑即抓住我方错误预期，
   修正测试后过）；
3. `import_block_types`——`[heading, paragraph, table]`（块型级）；
4. `import_stats_and_status_line`——typed 摘要 + 不导入面计数状态行
   （`1 table`/`image skipped`）；
5. `imported_doc_is_live_editor`——导入文档派发编辑事务 → 块数 3→4
   （**真活文档**，全桥 op/sync/事件管道，非静态 dump）；
6. `undo_works_after_import`——undo 键位契约在导入文档上成立；
7. `hostile_garbage_typed_error`——垃圾字节 → `container_corrupt`
   typed 码入状态行 + **编辑器现状零触碰**；
8. `truncated_zip_typed_error`——截断档案同 typed 面 + `__ready`
   仍真；
9. `contract_intact_no_pageerror`——契约版本/inject 面在 + 全程
   **0 pageerror**。

### 7.5 门禁表（全部实测，mdcore main cbe85c0 + 本轮工作树）

| 门禁 | 基线 | 实测 | 结论 |
|---|---|---|---|
| G1 `cargo test --workspace` | 426/0/1i | **435 / 0 / 1i**（+9 原生） | PASS |
| PDF lopdf / pdfium / dual | 79 / 81+6i / 109+6i | 79 / 81+6i / 109+6i（零漂移） | PASS |
| clippy default（--all-targets） | 仅 2 wasm 基线 | 恰 2（static_method_of，行号随 lib.rs 增补位移——计数口径不变） | PASS |
| clippy 3 PDF 面 | 0 | 0/0/0 | PASS |
| `cargo fmt --all -- --check` | 0 diff | 0 diff（新增代码已 fmt） | PASS |
| scan product / --all | 0 / 0 | 0 / 0 | PASS |
| wasm32 check（workspace） | pass | pass（calamine/rust_xlsxwriter/docx-rs/zip/quick-xml 全 wasm32 可编译——探针先行证实） | PASS |
| web contract / smoke / ime / **docx** | 18 / 15 / 28 / — | 18 / 15 / 28 / **9**（全过 0 failed） | PASS |

依赖与 Cargo.lock：**零三方增量**（无新包、零版本变动）→ NOTICES
v1.5 不动（无合规基线提交）。**诚实登记**：Cargo.lock 恰一行 diff——
mdcore-wasm 包条目 dependencies 增 `"mdc-office-io"`（工作区内部
path 边在 lock 的机械登记；任务书「零 diff」预期对任何新 path 依赖
不可达——这是最小必然 diff，非供应链变更）。

wasm 包体（诚实记录，无门禁线）：`web/pkg/mdcore_wasm_bg.wasm`
704,325 → **1,829,489 B**（+1,125,164 = **+159.8%**——docx-rs + zip
+ quick-xml 经 office-io 进 cdylib 图）；`dist/main.js` 543,980 →
547,470 B（+3,490——t16 TS 面增量）。

### 7.6 开发期发现登记

1. **mdcore 树规范形与导入分隔**：导入串 `\n\n` 块分隔只是宿主可读
   性面——核心 ingestion 跳过空行、`to_markdown()` 按行 join，编辑器
   canonical 形 = 块间单 `\n`。e2e 断言应锁**规范形**而非导入串原样
   （首跑错误预期即此发现）；
2. wasm32 图谱先验疑虑（calamine/rust_xlsxwriter 是否 wasm32 可编
   译）被探针一次证伪——**加依赖前先跑最小 check 探针**，免实现后
   返工；
3. 隐藏 `input[type=file]` + `Playwright set_input_files` 驱动真实
   change 路径（可见性非 set_input_files 前提）——e2e 走真 UI 通道
   而非仅 window 函数通道；
4. 夹具产线单一真源（`build_sample_docx` 一份代码：example 写盘 +
   原生测试在程 + 漂移守卫读盘三消费）——避免 example/测试/夹具
   三处漂移；漂移守卫报错内嵌再生成命令行。
