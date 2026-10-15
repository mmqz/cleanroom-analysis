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

## 8. Round 4（t18，2026-10-14——DOCX 导出切片：编辑器 → .docx 下载，与 t16 合流 = 往返闭环）

任务：web 编辑器**当前文档**（mdcore BlockTree——唯一语义真理，主档
§9）→ office-io `DocxWriter` 写面 → zip 字节 → `Blob + a.download`
下载 `document.docx`，端到端。与 t16（DOCX→编辑器导入）对称合流，
DOCX 往返故事闭环。

### 8.1 Phase A 裁决（映射面选型 + 证据锚点）

- **纯原生树遍历，零 JSON hop**：mdcore `tree.rs` 的 `BlockTree::
  top_level()/node()` 与 `Node{kind,text,attrs,marks,children}` 全
  `pub`——mdc-wasm 侧 Rust 直走（t16/t17 经 markdown 字符串边界的
  差异 = 方向性：导入终点是 MD ingestion 管道，导出起点是树本体）；
- **marks 面（模型事实）**：`MarkSpan{start,end,kind:String}` 自由
  kind 字符串 + **char 偏移**（op.rs `SetMark` L135 `len_chars` 契约
  ——不可字节切片）；规范 kind = `strong`/`em`/`code`/`del`/`u`
  （html.rs L20），spike 测试另有 `bold` 拼写——双拼写并收；
  **模型无链接表示**（MarkSpan 无 URL 载荷；html.rs 无 `<a>` 处理）
  → 导出无链接 run 可产（RunSpec.link 面 = 无源映射，非降级）；
- **表 = 扁平管道行文本**（markdown.rs L152-169 连续 `|` 行合并单
  Table 块；html.rs canonical_table_text 同形 + `\|` 转义）→ 导出需
  管道行解析（分隔行剔除 + 反转义）；
- 写面能力核对（docx.rs L1934-2130）：Heading1-9（样式 + outline
  双通道）/ 富段 RunSpec(text/bold/italic/link/footnote/field)/
  文本表 / 页眉页脚槽 / PNG——**无字体面、无列表面（t14 登记）、
  无水平线面** → 降级表逐项登记（8.3）。

### 8.2 落地面（mdcore）

- `crates/mdcore-wasm/src/docx_export.rs`（新增 ~570 行，模块文档 =
  错误码表/映射表/降级面/已知边界唯一真源随码走）：
  - 暴露面 = **WasmCore 增量方法** `export_docx(&self) -> Result<
    Vec<u8>, JsValue>`（t16/t17 自由函数模式的差异 = 导出读 `self`
    状态；`#[wasm_bindgen] impl` 分文件合法，既有方法零变化）；
    `Vec<u8>` → JS `Uint8Array`（wasm-bindgen 0.2.129 原生面，编译
    期证实）；失败抛 JSON `{"code":"docx_build","message"}`；
  - **错误码单族** `docx_build`（引擎写/打包——`DocxError::Io` 打包
    失败为唯一现实路径；`Parse` = heading 越界，映射层恒钳 1-9 故
    不可达仍完备映射；`Container` 只在读面产生——写侧不可达登记）；
    **映射层全函数：降级永不报错**；
  - 纯内核 `export_docx_core(&BlockTree)`（7 个纯函数：heading_level
    / mark_style / runs_from 边界集分段 + 邻接同式合并 / text_lines
    CRLF 剥离 / emit_rich_paragraphs 多行 char 偏移平移 / split_
    pipe_row + is_separator_row + table_rows / emit_block 递归）——
    零 JsValue 原生直测。
- `web/index.html`（工具条 `export .docx` 动作钮——.import-btn 共享
  样式 + `#docx-export-status`）；`web/src/main.ts`（t18 块镜像
  t16/t17 约定：`__exportDocx`/`__lastDocxExport` window 面 +
  not_ready 兜底 + JSON 错误面 + `downloadDocx` Blob/a.download +
  状态行 `exported: X KB · N blocks · M simplified`——降级计数取
  `__blocks()` 现场面，零新增 wasm 面）；`tsc --noEmit` 清洁。

### 8.3 映射表与降级登记（BlockKind → DocxWriter）

| mdcore 块 | DocxWriter | 降级（不可表达面，显式登记） |
|---|---|---|
| Heading（attrs.level 缺省/不可解析→1，钳 1-9） | heading()（Heading1-9 + outline） | 模型 level 任意 usize >9 钳 9；标题 marks 丢弃（heading 纯文本面） |
| Paragraph | paragraph() run 序 | strong/bold→加粗、em/italic→斜体；**code/del/u/未知 → 纯文本**（RunSpec 无等宽/删除线/下画线面）；link kind → 纯文本（模型无 URL 载荷） |
| CodeBlock | 每行一个纯文本段 | **等宽/字体面 + lang + fence 结构全丢**（文本逐字保留） |
| Table | table() | 分隔行剔除；`\|` 反转义；单元格 `\n`/`\r`→空格（GFM 单行面，t17 同向）；孤立 `\` 透传（无内联转义层——t16 同边界）；0 行表跳过（空 w:tbl 非法形） |
| Blockquote | 子块平铺递归 | 引用容器/缩进丢失 |
| ListItem | item 富段 + 子块递归 | **marker/depth（列表结构）丢弃**（t14 登记列表写面未暴露；不注入 `- ` 伪 markdown）；次序保持 |
| ThematicBreak | 空段落 | 无水平线面；空段 = 分隔提示 |

全局文本规则：`\n` 分行 + 行尾 `\r` 剥离（CRLF 容忍），多行文本 →
每行一段（mdcore 行级哲学，t16 w:br 同向）；mark 偏移按行区间裁剪
平移（多行不丢格式）；相邻同 (bold,italic) run 合并（docx run 边界
不承载语义——t16 导入侧合并登记的反向）。

### 8.4 测试证据（原生 10 + web e2e 8）

原生（mdcore-wasm 21→31；写→`read_docx_from_bytes`→模型断言全环）：
1. `headings_map_to_styles_with_levels`——H1/2/6 样式 + 无样式正文；
2. `heading_level_default_and_clamp`——level "42"→H9 / "abc"/缺失→H1；
3. `bold_italic_marks_roundtrip`——char 偏移四段 run（plain/bold/
   间隔空格/italic——首跑抓出我方错误预期，mapper 正确）；
4. `mark_adjacent_merge_and_overlap`——邻接同式并 1 run；交叠四段；
5. `unsupported_inline_marks_degrade_to_plain`——code/del/u/link →
   单纯文本 run（文本零损）；
6. `table_roundtrip_escaped_pipe_and_separator_drop`——分隔行剔除 +
   `\|` 反转义（`b \| c` → `b | c`）；
7. `code_block_degrades_to_plain_paragraphs`——两行代码 → 两无样式段；
8. `quote_list_divider_flatten_in_order`——quote/li×2 嵌套/hr →
   有序平铺 + 末尾空段；
9. `multiline_paragraph_splits_with_marks_and_crlf`——多行 marks 平移
   + `\r\n` 剥离；
10. `empty_tree_yields_valid_minimal_docx`——空树 → PK 魔数 + 读回
    0 块。

web e2e（`web/tests/docx_export.py`，第六套，**8/8 PASS**，端口 8935）：
1. `export_ui_present`——按钮/状态行/`__exportDocx` 面 + 契约 1.1；
2. `happy_export_valid_zip`——**真实按钮点击**（page.click → Blob
   下载路径）→ python `zipfile` 验证 PK 魔数/[Content_Types].xml/
   word/document.xml/testzip + 状态行 `exported:`；
3. `document_xml_mapping_strings`——document.xml 含标题文本 +
   `Heading1` + `<w:outlineLvl`（style_probe 先例）；
4. `table_export_xml`——`<w:tbl>` + 四单元格文本；
5. `degraded_blocks_survive_export`——code/quote/list×2/divider 全
   存活 + 状态行 `5 simplified` + **导出前后 `__md()` 逐字不变**
   （只读性实证）；
6. `empty_document_minimal_docx`——空文档 → 最小合法 docx + `0 blocks`；
7. `full_circle_reimport_content`——**导出→`__importDocx`→`__md()`
   逐字还原**（非降级子集 heading/paragraph/table——降级块型不可
   全环属映射表登记的诚实边界，测试显式取可表达子集）；
8. `contract_intact_no_pageerror`——契约面 + 全程 0 pageerror。

### 8.5 门禁表（全部实测，mdcore main de2ecf7 + 本轮工作树）

| 门禁 | 基线 | 实测 | 结论 |
|---|---|---|---|
| G1 `cargo test --workspace` | 447/0/1i | **457 / 0 / 1i**（+10 原生） | PASS |
| PDF default/lopdf/pdfium/dual | 68/79/81+6i/109+6i | 68/79/81+6i/109+6i（零漂移） | PASS |
| clippy default（--all-targets） | 仅 2 wasm 基线 | 恰 2（static_method_of） | PASS |
| clippy 3 PDF 面 | 0 | 0/0/0 | PASS |
| `cargo fmt --all --check` | 0 | 0（新增代码已 fmt） | PASS |
| scan.sh product | 0 hit-paths | 0 hit-paths | PASS |
| wasm32 check | pass（2 基线警） | pass（2 基线警） | PASS |
| web 六套件 contract/smoke/ime/docx_import/xlsx_import/**docx_export** | 18/15/28/9/9/— | 18/15/28/9/9/**8**（全过） | PASS |

依赖与 Cargo.lock：**零 diff**（无新包、无版本变动、无 path 边变化
——office-io 依赖自 t16 在位）。

wasm 包体（诚实记录，无门禁线）：`web/pkg/mdcore_wasm_bg.wasm`
2,066,405 → **3,105,817 B**（+1,039,412 = **+50.3%**）；`dist/main.js`
551,132 → 554,265 B（+3,133）。归因：docx-rs **写面机构**（OOXML
序列化 + 打包）首次从导出函数可达——t16/t17 的 DocxWriter 调用点
（夹具构建器）只被 `#[cfg(test)]`/examples 引用，cdylib 导出图内
死码剥离；**t17「pub fixture builder 保活 rust_xlsxwriter」的归因
解释据此勘误**（pub 性不改变 wasm-ld gc-sections 可达性判定，t17
的 +236,916 B 应全部为 calamine 读面）。dist 侧 +3,133 B = t18
TS 包装面。

### 8.6 开发期发现登记

1. **Vec<u8> wasm 返回面**：wasm-bindgen 0.2.129 原生支持
   `Result<Vec<u8>, JsValue>`（→ Uint8Array）——与 t16/t17 String/JSON
   面并列的第三种出入口形态，build-wasm.sh 编译即证；
2. **blob 构造的 TS 5.9 泛型面**：`new Blob([bytes])` 受
   Uint8Array<ArrayBufferLike> 泛型化影响——`new Blob([new
   Uint8Array(bytes)])`（构造器产 Uint8Array<ArrayBuffer>）静态过，
   拷贝量微（编辑器文档级）；
3. **Playwright evaluate 传参**：非函数表达式体内无 `arguments`
   ——`(s) => window.__inject(s)` 箭头形（docx_import.py 的 IMPORT_JS
   先例即箭头形，直接 evaluate 字符串面新踩）；
4. **e2e 期望锚**：`__blockCount()` = 先序**全节点**计数（含 quote
   容器的段落子块）——DEGRADED_MD 6 块 ≠ 顶层块数 5；降级计数同源
   （`5 simplified` 不计 quote 的段落子块——按块型过滤）。

## 9. Round 5（t20，2026-10-15——写面消降级切片：office-io DocxWriter 五面扩 + docx_export 六行映射改法，t18 登记降级整批消解）

任务：依 WRITE_FACE_PROBE.md（t20-a 一手核验 + 探针 16/16）把
t18 登记的四件降级（list/monospace/hr/quote）+ 块级 code 等宽 +
行内 del·u（§2.6 顺带）**整批消解**——office-io `DocxWriter` 写面
五面新增 + `docx_export.rs` 六行映射改法 + 探针断言集移植为原生/
e2e 测试。mdcore main 7705b44 → **5028ec4**（5 文件 +1029/−56：
office-io/src/docx.rs +374、mdc-wasm/docx_export.rs +417、
office-io/tests/docx_roundtrip.rs +244、web main.ts/docx_export.py）。

### 9.1 office-io 写面五扩（DocxWriter 公共契约面）

| 面 | 新增 API | 语义要点 |
|---|---|---|
| numbering 定义面 | `ListKind`（Ordered/Bullet）+ 常量 `ORDERED_NUM_ID=100`/`BULLET_NUM_ID=101`/`MAX_LIST_LEVEL=8` + 私有 `ensure_numberings`（**惰性注册**：首个 `list_item` 时注入） | ordered = decimal outline 式 9 级（lvlText 逐级拼号 `%1.`…`%1.…%9.`，left=720×(i+1) 悬挂 360）；bullet = `•` 9 级同构；numId=abstractNumId=100/101 **避开引擎内建 numId=1**；`save_to_vec` 基态重置含注册位（幂等续写钉定） |
| 列表段面 | `list_item(&[RunSpec], ListKind, ilvl)`（ilvl typed 验证 0-8） | **必走 `Paragraph::numbering`**（同置 has_numbering → 打包时 document.xml.rels 发射 numbering 关系） |
| 等宽 run 面 | `RunSpec` 扩 `mono`/`strike`/`underline` 三位 + `styled_full` 构造器 + 常量 `MONO_FONT="Consolas"`/`CODE_FONT="Courier New"` | mono → `Run::fonts(rFonts 三槽 ascii/hAnsi/cs)`；strike → `w:strike`；underline → `w:u single`；既有构造器三位恒 false（零漂移） |
| hr 面 | `horizontal_rule()` | 空段 + **`ParagraphBorders::with_empty().set(单边 Bottom)`**（size=6/space=1——§2.3 探针参数） |
| quote 面 | `quote_paragraph(&[RunSpec], depth)` + 常量 `QUOTE_INDENT_STEP=720`/`MAX_QUOTE_DEPTH=8` | `indent(left=720×depth 钳 8)` + `with_empty().set(Left)`（size=12/space=4）；无内建 Quote 样式 → 直接格式 |

块级 code 面：`code_line(text)` = **双通道等宽**（段落级
`Paragraph::fonts` → `pPr>rPr>rFonts` 兜段落标记 + run 级 rFonts
保真——同一 `RunFonts` 值复用，零成本双设）。

### 9.2 docx_export.rs 六行映射改法（t18 降级 → t20 表达翻转表）

| 块/mark | t18（Round 4 §8.3） | t20 实测 | 残余 |
|---|---|---|---|
| ListItem | marker/depth 丢弃，富段平铺 | `list_shape`：marker 首字符 ASCII 数字 → Ordered（`1.`/`1)` 两形态），`-`/`*`/`+`/缺失 → Bullet；depth（1 基）→ ilvl=depth−1 **钳 0-8**（九级抽象域，越界防御全函数）；首行 = 编号段（marks 保持），余行（op 面多行防御）= 普通段；子块递归保持 | 多行 item 续行无同级缩进对齐（登记）；多列族混排/自定义编号格式（office-io 未暴露，引擎能力在） |
| 行内 code mark | 纯文本 | `mark_style` 五位；rFonts 三槽 Consolas；runs_from 合并键扩五位 | — |
| 行内 del/u mark | 纯文本 | strike/`w:u single` 位（§2.6 顺带消解） | — |
| ThematicBreak | 空段落（无水平线面） | `horizontal_rule()` 空段下边框 | — |
| Blockquote | 子块平铺（容器/缩进丢） | `quote_paragraph`：ind 720×depth 嵌套步进 + 左边框；段落/嵌套引用承载引用面 | 引用上下文中的 heading/table/code/list/hr 子块不挂 ind/pBdr（markdown 解析器不产此形态，防御递归保持 t18 行为，登记） |
| CodeBlock | 每行纯文本段（等宽全丢） | 每行 `code_line` 双通道 Courier New | **lang/fence 仍丢**（docx-rs 无语法高亮/元数据面——库层不可达，登记维持） |
| link mark | 纯文本（模型无 URL 载荷） | 不变 | **上游缺口维持**（MarkSpan 无 URL 载荷——mdcore MD 域裁决前置，非本层降级） |

工程重构：`rich_lines`（行切分 + mark char 偏移平移）提取为单一
真源（ListItem 首行/余行分道与引用段落共用）；`emit_block` 增
`quote_depth` 参数（Blockquote 子块递归传 depth+1）。

### 9.3 陷阱两项实录（WRITE_FACE_PROBE §2 核验 → 产品落地验证）

1. **悬空 numPr**：`ParagraphProperty::numbering_property` 直挂路径
   不置 `has_numbering` → 打包时 document.xml.rels 缺 numbering
   关系 → 合法包但 Word 无列表渲染。产品面恒走
   `Paragraph::numbering(NumberingId, IndentLevel)`；e2e 断言
   `relationships/numbering` 在 rels（反证在位：docx_roundtrip
   `t20_write_faces_xml_level` ① + web `t20_list_numbering_xml`）。
2. **`ParagraphBorders::new()` 四边全 Single**（Default impl）——hr
   若用 default 画整框而非水平线。产品面恒
   `with_empty().set(单边)`；e2e 断言 pBdr 内**恰单边**子元素
   （`<w:pBdr><w:bottom …/></w:pBdr>` 精确串匹配 = 反证）。

### 9.4 测试证据（原生 +9 office-io +4·mdc-wasm +5；e2e 8→12）

原生 office-io（46→48 内联 + 22→24 roundtrip）：
- `t20_write_faces_xml_level`——探针 16 断言集的**产品 API 移植**
  （zip 结构 numbering part + rels 接线 / numbering.xml 双抽象 +
  num 桥 + 9 级 + 多级 lvlText / document.xml numPr 精确配对
  〔100×2 ilvl0 + 100×1 ilvl1 + 101×2 ilvl0〕/ 行内 rFonts /
  hr pBdr / quote ind 720+2160 步进 + pBdr left / code 双通道 /
  写读全环文本零损 11 段 + warnings 空）；
- `t20_numbering_lazy_registration_and_reset`——无列表档
  numbering.xml 保持内建形态（100/101 不在，内建 abstractNum 1
  恒在）+ save 后续写重注册（基态重置钉定）；
- 内联 `list_item_ilvl_validation`（0-8 过 / 9 拒）+
  `specs_expand_mono_strike_underline`（三位引擎接线 + plain run
  零新位）。

原生 mdc-wasm（43→48；**XML 面直断言新通道** `zip_stored_part`
——docx-rs 打包恒 Stored 无压缩 → 本地头遍历零解压直切，zip
依赖不进 mdc-wasm〔Cargo.lock 零 diff 纪律〕）：
`inline_mono_del_u_expressed_link_plain`（t18 降级断言语义翻转
为 XML 表达 + 文本零损双面）、`code_block_mono_dual_channel_xml`、
`list_mapping_numbering_xml`、`list_marker_forms_and_depth_clamp`
（`1)` 形 / depth 99 钳 8 / 缺失防御）、
`list_multiline_item_first_numbered_rest_plain`、
`quote_and_hr_mapping_xml`。

web e2e（docx_export.py 8→**12/12**）：+`t20_list_numbering_xml`
（numPr 配对 + bullet 抽象 + **rels 接线**）/`t20_quote_indent_
border_xml`/`t20_hr_bottom_border_xml`/`t20_code_block_mono_
rfonts_xml`（双通道）；`degraded_blocks_survive_export` 期望
`5 simplified` → **`1 simplified`**（web 状态行降级计数缩为
code_block——lang/fence 登记维持）。**既有断言零回归**：六套件
contract 18 / smoke 15 / ime 28 / xlsx_import 9 / docx_import 9 /
xlsx_export 9 全绿（亲跑复验）。

**诚实边界（t20-a 遗留④，维持登记不宣称）**：导出→t16 重导入
往返中 numPr/pBdr/ind/rFonts 的读面模型承载未核（office-io 读
模型无对应字段）——e2e 断言只做导出 XML 面 + 既有文本面回归，
**不宣称 list/quote/hr/等宽的往返等值**；`full_circle_reimport_
content` 维持可表达子集（heading/paragraph/table）。

### 9.5 门禁表（全部亲跑实测，mdcore 5028ec4）

| 门禁 | 基线（t19） | 实测 | 结论 |
|---|---|---|---|
| G1 `cargo test --workspace` | 469/0/1i | **478 / 0 / 1i**（+9 只增） | PASS |
| PDF default/lopdf/pdfium/dual | 68/79/81+6i/109+6i | 同值零漂移（+perf --ignored 6/6） | PASS |
| clippy default（--all-targets） | 仅 2 wasm 基线 | 恰 2（static_method_of） | PASS |
| clippy 3 PDF 面 | 0 | 0/0/0 | PASS |
| `cargo fmt --all --check` | 0 | 0 | PASS |
| scan.sh product | 0 hit-paths | 0 hit-paths | PASS |
| wasm32 check | pass（2 基线警） | pass（2 基线警） | PASS |
| web 七套件 | 18/15/28/9/9/9/8 | 18/15/28/9/9/9/**12**（docx_export 扩展） | PASS |

Cargo.lock：**零 diff**（docx-rs 0.4.22 自 t16 在 office-io 依赖
面，五面扩全部在既有 API 面——无新包无版本变动）。

wasm 包体（诚实记录）：`web/pkg/mdcore_wasm_bg.wasm`
3,950,549 → **3,969,144 B（+18,595 = +0.47%）**；`dist/main.js`
557,256 → **557,211 B（−45）**。归因：docx-rs 写面机构 t18 已付
（+50.3%），本轮增量 = 纯映射胶水（numbering 构造 + 三 run 位 +
两段落属性面）——WRITE_FACE_PROBE §4.3「预期近乎零增长」兑现；
dist −45 B = TS 面注释剥除 + degradedBlockCount Set 字面量缩短。

### 9.6 残余降级登记（t20 后全景）

| 项 | 状态 | 根因层 |
|---|---|---|
| code lang/fence 结构 | 降级维持 | 库层（docx-rs 无语法高亮/元数据面） |
| link mark URL | 无源映射维持 | 模型层（MarkSpan 无 URL 载荷——mdcore MD 域上游裁决前置） |
| 多行 list item 续行缩进对齐 | 登记（op 面可构造形态） | 映射层哲学（续行普通段不重复编号） |
| 引用上下文非段落子块不挂 ind/pBdr | 登记（防御形态） | markdown 解析器不产此形态（attach 恒 root） |
| numPr/pBdr/ind/rFonts 读面往返 | 未核不宣称 | 读模型无字段（t20-a 遗留④——读面扩字段需单独裁决） |
| 多列族混排/自定义编号格式 | 未暴露 | office-io 包装面（引擎能力在，按需增量） |

**t22 追记（2026-10-16）**：上表「多行 list item 续行缩进对齐」已
从登记残余**翻转为已表达**（office-io `DocxWriter::list_item_lines`：
一个列表项 = 一个 OOXML 段落——行间 `w:br w:type="textWrapping"`
软换行 + 段落级 hanging `w:ind`〔left = 720×(ilvl+1)、hanging 360，
与编号抽象层级同值〕→ 续行与首行文字列对齐；单行项字节级零变化），
mdcore ef74282，详见 worklog t22 节。

**r5 追记（2026-10-17，Round 5 DOCX 消费链补全）**：上表「numPr/
pBdr/ind/rFonts 读面往返」行已被 r4-t3 翻转（numPr/ind 读面完整承载
——保真判定自 r4-t3 起**单一真源 = docx-r1/FIDELITY.md**，本表为
t20 时点历史快照）。Round 5 两增量：① r5-t1 **DOCX 列表导入映射**
（`docx_import.rs` 消费 r4-t3 numbering 读面——列表段此前降级普通
段落，现 Bullet→`- `/Ordered→`{n}.`/嵌套 2×min(ilvl,8) 空格/降级面
五类 stats 计数可诊断；映射语义与降级登记 = FIDELITY §5；夹具
`import_sample.docx` 扩列表节 26,625→31,733 B，web e2e 9→11 checks）
；② r5-t2 **超链接锚点写回评估**（tests-only 五探针 + 方案与裁决点
= CAPABILITY_NOTES §13——RunSpec 公开模型扩面**未实施**，呈用户
裁决）。mdcore a5c314b/00ad88b，G1 517→530 零回归。
