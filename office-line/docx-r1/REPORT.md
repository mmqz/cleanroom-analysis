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
