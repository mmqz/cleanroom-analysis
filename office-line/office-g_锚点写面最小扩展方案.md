# office-g 锚点写面最小扩展方案与兼容性分析（方案文档，不实施）

> **性质**：用户裁决③（2026-10-17 会话）执行轮——"Office 锚点写面：
> **暂维持现有公开 API**。先给出最小扩展方案和兼容性分析，**不提前
> 扩大 RunSpec**"。本文档 = 该口径的落档：把 r5-t2 CAPABILITY_NOTES
> §13 的 Option A 细化为**实施就绪规格**（字段/构造器/发射点/优先序/
> 测试/门禁逐项 exact），并补全一手核实的兼容性全量清单。**本轮零
> 代码变更**（mdcore main=00ad88b 零接触维持；全部 grep/读源一手亲核）。
> **决策链**：r5-t2 §13（评估 + 裁决三问）→ 用户裁③（维持 + 方案先行）
> → 本文档（Phase 0 生效，Phase 1/2 待裁决触发）→ yrs-b R4.5 联动
> （d1c 原型已按弱绑定形态实现，见 §6 升级路径）。
> **anchor-t2 增补（2026-10-17 加固会话）**：§7-§12 补全评审维度——
> 调用示例 / 锚点身份与解析模型 / 生命周期语义（复制·删除·移动·
> 陈旧引用·文档版本）/ 迁移与向后兼容 / 校验与错误矩阵 / 备选方案
> 与最小性论证。仍为**纯方案**（零代码变更，公开 API 待产品语义
> 裁决后触发）。

## 0. 方案总览（三阶段 + 触发条件）

| 阶段 | 内容 | 状态 | 触发条件 |
|------|------|------|---------|
| **Phase 0 现状维持** | 公开 API 零变化；读面四变体 + 写面 External-only + 回放 Anchor 静默丢 = FIDELITY §3 维持登记 | **本轮生效（用户裁决）** | — |
| **Phase 1 最小加性扩** | `RunSpec.anchor` + `anchor_link()` 构造器 + 发射点单分支 + 回放示范扩 Anchor | 规格 ready（本文 §1-§2），**未实施** | 用户裁决"现在扩"；或 MarkSpan URL 裁决落地后与 web 链一并做 |
| **Phase 2 书签定义写面**（正交件） | `DocxWriter::bookmark_paragraph` + id 计数唯一化 | 规格 ready（本文 §3），**未实施** | 与 Phase 1 同批 / 分离 / 不做（悬空锚点 = 合法形边界）三选一裁决 |

**不做的面（维持登记）**：Internal 变体写回（引擎 rels 发射面
external-only——documents/mod.rs:1210，结构性不可达）；HYPERLINK 域
（todo!() 崩溃面红线，w:anchor 属性形与域形不相交）；MarkSpan URL /
行内链接解析（mdcore MD 域暂缓项）。

## 1. Phase 1 精确规格：RunSpec 锚点字段

### 1.1 API 形（exact）

```rust
pub struct RunSpec {
    // …既有 9 字段零变化（text/bold/italic/mono/strike/underline/
    //    link/footnote/field——docx.rs:2205-2231；e-review 勘误：
    //    初稿「10 字段」计数失实，实为 9）…
    /// 文档内锚点链接（Some = 本 run 发射 `w:hyperlink w:anchor`
    /// 属性形——引擎 HyperlinkType::Anchor，零 rels 面，悬空合法）。
    /// 与 `link` 互斥语义：同 Some 时 anchor 优先（读写镜像，见 1.2）。
    pub anchor: Option<String>,
}

impl RunSpec {
    /// 锚点链接 run（Some(anchor) → Hyperlink::new(anchor, Anchor)）。
    pub fn anchor_link(text: impl Into<String>, anchor: impl Into<String>) -> Self;
}
```

### 1.2 发射点单分支扩（add_spec_to_paragraph，docx.rs:2777）

优先序（与读面探针② 镜像——`w:anchor` 优先于 r:id）：

```
footnote > field > anchor > link
```

- 既有 `if let Some(url) = &spec.link`（docx.rs:2819）前面插入
  `if let Some(a) = &spec.anchor` 分支：`Hyperlink::new(a, HyperlinkType::Anchor)`
  （探针⑤ 已整包实证：w:anchor 属性形 + 零 rels + 引擎写→自有读全环）；
- anchor 与 link 同 Some：anchor 赢、link 忽略——**文档登记语义**
  （读写两侧对称：读面同元素双属性 → Anchor 赢，探针② 钉定）。

### 1.3 文档面

`link` 字段 doc 注释更新（现"锚点写面不暴露"→ 指向本方案 + anchor
字段）；`RunSpec` 结构体文档补互斥语义一行。

## 2. 兼容性分析（一手 grep 全清单，2026-10-17 @ 00ad88b）

### 2.1 消费者全量清单（grep -rn "RunSpec" --include="*.rs"，target 外）

| 消费面 | 文件 | 用法 | Phase 1 影响 |
|--------|------|------|-------------|
| 定义 + 构造器 + 发射 | office-io/src/docx.rs（2205 定义 / 2233-2333 七构造器 / 2777 发射） | 自有 | 字段 +1、构造器 +1、发射分支 +1 |
| **struct 字面位点 ①** | docx.rs:2688 `code_line` | `..RunSpec::text("")` base 语法 | **免疫**（functional update 不列字段） |
| **struct 字面位点 ②③④** | docx.rs:2996/3011/3026 `run_spec_constructors` 测试 | 全字段等值断言 | **机械补 `anchor: None` 三处**（含 link 断言处顺延） |
| 字段读取面 | docx.rs:1708（读分发 match &h.link）/ 2819（写发射） | 自有 | 1708 读面**零变化**（读模型 RunData.hyperlink 不动）；2819 扩分支 |
| wasm 导出链 | mdcore-wasm/src/docx_export.rs（140 runs_from / 186 / 208 styled_full / 433 lines） | **全构造器面**（text/styled_full） | **零改动** |
| wasm 导入链 | mdcore-wasm/src/docx_import.rs（523-530 测试） | 构造器面 | **零改动** |
| 集成测试 | office-io/tests/docx_roundtrip.rs（37-1313 构造器散布 + 2074 runs_to_specs 回放） | 构造器面 | 零强制改动；**回放示范建议扩**（2.3） |
| 序列化面 | — | RunSpec **无 serde derive**（grep 核实零 serde 面） | **零序列化漂移** |

### 2.2 加性证明（先例对齐）

循 r4-t1 CellData / r4-t3 ParagraphNumbering 的 Option-字段加性先例：
全部既有构造器恒 `anchor: None`（构造器内部 `..Default` 式构造或显式
None），既有调用方行为零漂移；**唯一破坏面 = 全字段 struct 字面**，已
全量枚举 = 4 处（1 免疫 + 3 测试机械补）——公开 API 面无破坏（外部
零字面位点，§2.1 清单 grep 亲核）。

### 2.3 建议性顺带修（非强制，实施批并入）

`runs_to_specs` 回放示范（tests/docx_roundtrip.rs:2074-2083）现只映射
`HyperlinkTarget::External`——Anchor 变体静默丢弃（§13.1 末行登记面）。
Phase 1 实施时顺带扩 `HyperlinkTarget::Anchor { anchor } → s.anchor =
Some(...)` 分支，修对称缺口（读 4 → 写 2；Internal 维持不可达登记）。

### 2.4 不受影响面（显式登记）

- 读模型 `RunData.hyperlink: Option<HyperlinkTarget>`（四变体）**零
  变化**——Phase 1 只动写侧；
- web 导入链（`#anchor` GFM 渲染）零变化；
- web 导出链零变化（MarkSpan 无 URL/锚点载荷——mdcore MD 域暂缓项，
  web 端到端收益在 MarkSpan URL 裁决前诚实为 0，§13.4 维持）。

## 3. Phase 2 精确规格：书签定义写面（正交件）

```rust
impl DocxWriter {
    /// 书签包裹段：bookmarkStart(id, name) … runs … bookmarkEnd(id)。
    /// id 由写器内部计数器唯一化（引擎不校验唯一性——包装层保证；
    /// 与 TOC `_Toc` 族命名空间冲突面：**保留名前缀 `_Toc` 拒绝**
    /// （InvalidInput）——引擎 TOC 内部机制不被用户面踩踏）。
    /// 校验面 V1-V6（§11）→ `DocxError::InvalidInput`（**新增变体**，
    /// 见 §11 勘误——既有 DocxError 仅 Container/Parse/Io/Limit 四变体）。
    pub fn bookmark_paragraph(
        &mut self,
        runs: &[RunSpec],
        name: &str,
    ) -> Result<(), DocxError>;
}
```

- 引擎面：`Paragraph::add_bookmark_start/add_bookmark_end`（探针⑤
  整包实证在案）；
- 独立于 Phase 1 可分离裁决（无书签定义的锚点链 = 悬空合法形——
  Word 点击报"书签未定义"，边界见 §13.2-2）。

## 4. 测试计划（Phase 1+2 实施批，预期门禁面）

| # | 测试 | 钉定行为 |
|---|------|---------|
| T1 | anchor_link 写→读全环（docx_roundtrip） | w:anchor 属性形 + 零 rels + 自有读 Anchor 变体复原（探针⑤ 断言集移植产品面） |
| T2 | anchor 优先于 link（同 Some） | 发射 w:anchor 无 r:id；读回 Anchor 赢（读写镜像） |
| T3 | 悬空锚点写（合法形） | 无书签定义仍发射；引擎/自有读不拒 |
| T4 | bookmark_paragraph XML 面 | bookmarkStart/End 对 + id 唯一（两书签段） |
| T5 | `_Toc` 保留名拒绝 | InvalidInput 错误面 |
| T6 | 回放示范扩 Anchor | runs_to_specs Anchor 变体不再静默丢 |
| T7 | 构造器零漂移断言 | run_spec_constructors 扩 anchor: None 恒等断言 |

门禁预期：G1 +7~9（T1-T7）；PDF 四面/clippy/fmt/scan/wasm32/web 七套件
零漂移（纯 office-io 面变更，无 wasm/web 产物变更）；Cargo.lock 零 diff。

## 5. 与协同锚点面的联动（yrs-b R4.5 升级路径）

d1c 原型（yrs-c §1.3）已按**弱绑定形态**实现 Office 线入向
（`SemanticAnchor::OfficeBookmark { name, block_index }`——位置快照，
身份漂移诚实展示）。Phase 1+2 落地后升级路径：

| 组件 | 弱绑定（现状原型） | 强绑定（Phase 1+2 后） |
|------|------------------|----------------------|
| 适配层 bind 入向 | 书签名 + 位置快照 | 书签名 + **书签定义存在性检查**（读面 C3 悬空警告面消费侧对策，yrs-b R4.3） |
| 适配层 resolve 出向 | 位置语义（当前该位置的块） | 书签 ID → CRDT ID（块身份） |
| 悬空面 | 位置漂移不可辨 | 锚点删除 → `AnchorDangling` 显式化 |

**升级为适配层单点改动**（SemanticAnchor 变体字段 + bind/resolve 两
方法），office-io 公开 API 与协同适配层保持单向依赖（office-io 不感知
协同）——非耦合面维持。

## 6. 已核带源清单

- RunSpec 全消费面 grep（2026-10-17 @ 00ad88b）：字面 4 处 / 构造器
  面散布 / 字段读取 2 处 / wasm 两文件构造器面——逐一列于 §2.1；
- 发射点与优先序：add_spec_to_paragraph（docx.rs:2777）+ 读面分发
  （docx.rs:1708）+ 探针②⑤ 断言（§13.5）；
- 引擎面可用性：探针⑤ 整包实证（§13.3-3）；
- runs_to_specs 现状：tests/docx_roundtrip.rs:2074-2083（External-only
  映射，Anchor 静默丢——亲读源码核实）；
- RunSpec 无 serde：grep "derive.*Serializ" 核实零命中（序列化漂移面
  不存在）。

**待核（实施批）**：T1-T7 断言的精确 XML 形状（以探针⑤ 产物为基准）；
`_Toc` 保留名策略的用户确认（Phase 2 裁决点）。

## 7. 调用示例（Phase 1 + Phase 2 实施批形态，可直接评审）

### 7.1 Phase 1：锚点链接（消费侧——链接到书签）

```rust
use mdc_office_io::docx::{DocxWriter, RunSpec};

let mut w = DocxWriter::new();

// 目录/交叉引用形态：链接文本 + 目标书签名（w:hyperlink w:anchor
// 属性形——零 rels 面，悬空合法）
w.paragraph(&[
    RunSpec::text("参见 "),
    RunSpec::anchor_link("第 3 节 方法论", "sec-methods"),
    RunSpec::text("。"),
]);

// 与既有 link 构造器并存（External 面）：
w.paragraph(&[
    RunSpec::link("外部参考", "https://example.com"),
    RunSpec::text(" / "),
    RunSpec::anchor_link("内部跳转", "sec-methods"),
]);

// 消费侧（mdcore-wasm docx_export 路径）零改动——构造器面默认
// anchor: None，既有 text/styled_full 调用点行为零漂移（§2.2）。
```

### 7.2 Phase 2：书签定义（生产侧——被链接的目标）

```rust
// 书签包裹段：bookmarkStart(id, name) … runs … bookmarkEnd(id)
// id 由写器内部计数器唯一化；重名拒绝（§11）；`_Toc` 保留名拒绝。
w.bookmark_paragraph(
    &[RunSpec::styled_full("3. 方法论", true, false, false, false, false)],
    "sec-methods",
)?;
w.paragraph(&[RunSpec::text("正文……")]);

// 完整闭环（生产侧 + 消费侧同文档）：
w.bookmark_paragraph(&[RunSpec::text("附录 A")], "appendix-a")?;
w.paragraph(&[
    RunSpec::text("详见 "),
    RunSpec::anchor_link("附录 A", "appendix-a"),
]);
```

发射的 OOXML 形状（探针⑤ 实证基准，T1 断言集的精确期望）：

```xml
<w:p><w:bookmarkStart w:id="1" w:name="sec-methods"/><w:r>…</w:r>
<w:bookmarkEnd w:id="1"/></w:p>
…
<w:p><w:hyperlink w:anchor="sec-methods"><w:r><w:t>第 3 节 方法论</w:t>
</w:r></w:hyperlink></w:p>
```

## 8. 锚点身份与解析模型（稳定身份 = 书签名，非引擎 id）

| 层 | 身份 | 稳定性 | 说明 |
|---|---|---|---|
| **公开身份** | 书签名 `w:name`（String） | **跨会话/跨引擎稳定** | 用户可见、可持久化、与协同侧车（MDCA OfficeBookmark.name）同源——R4.5 强绑定升级的单点对齐面 |
| 引擎内部 id | `w:id`（u32 计数器） | **仅单次导出会话内稳定** | 写器实例唯一化；重开/重导出不保证复用——**不可作为身份**（与 mdcore BlockId 跨 save→open 条件保真面同因：引擎不持久化分配器状态） |
| 链接侧引用 | `RunSpec.anchor`（书签名） | 与公开身份同 | `w:hyperlink w:anchor` 属性形直接携带书签名（无 rels 间接层） |

**解析语义（读面，既有公开 API，Phase 1/2 零变化）**：读面
`RunData.hyperlink: Option<HyperlinkTarget>` 已有 `Anchor { anchor }`
变体（r5-t2 探针② 钉定 w:anchor 优先于 r:id）——链接侧解析已就绪；
**书签定义读面当前不暴露**（引擎读面丢弃 bookmarkStart/End——
r5-t2 登记面），即"锚点存在性不可辨"。Phase 1/2 是**写面**方案；
书签定义读模型 = 独立后续项（登记，不在本方案扩面——维持最小性）。

**重名语义（设计决定，呈评审）**：同文档内 `bookmark_paragraph`
重名 → `InvalidInput` 拒绝（宁缺毋错对齐——静默双书签在 Word 中
点击行为歧义）。Word 本身容忍重名但跳转不可预期；包装层显式拒绝。

## 9. 生命周期语义（复制 / 删除 / 移动 / 陈旧引用 / 文档版本）

| 操作 | 语义 | 依据 |
|---|---|---|
| **复制**（段落含锚点链接或书签定义被复制到别处） | 本方案域内无复制操作面（DocxWriter 是一次性导出流）；跨会话"复制"= 重新构造导出——书签定义是否重建由调用方决定。**链接与定义不自动跟随内容复制**（OOXML 语义：链接指向名字，不指向段落） | 引擎语义 + 一次性导出模型 |
| **删除**（书签目标段落删除） | 书签定义随段落消失 → 指向它的链接成为**陈旧引用**——这是**合法形**（T3 钉定：发射不拒、引擎/自有读不拒）；Word 打开时点击报"书签未定义"，文档本身完好 | Word 实测行为面（r5-t2 §13.2-2 登记边界） |
| **移动**（书签段落移动位置） | 书签定义**随段落移动**（bookmarkStart/End 是段落内元素，位置语义在段落内部）；链接侧不受影响（按名解析，与位置无关）。mdcore 当前无移动 op（BlockTree 结构性编辑 = 删+插）——移动后**同段重导出**书签保持；**删+重插**产生新段落则书签不自动迁移（调用方语义责任） | OOXML 结构语义 |
| **陈旧引用**（链接指向不存在书签） | 合法形（见删除行）；读面 `Anchor` 变体照常返回（不静默丢——与 C3 悬空警告面消费侧对策衔接：存在性检查是消费侧责任，因读面不暴露书签定义，§8） | §13.2-2 边界登记 |
| **文档版本** | 本方案**不引入版本语义**：锚点是内容级（书签名 ∈ 文档内容），无版本号/代际；mdcore 文档模型无版本概念（CONTRACT v1 面无此字段）。重导出同一逻辑文档 → 书签名不变（调用方责任）、引擎 id 变（§8 不作为身份）——稳定身份链 = 名字 | 模型现状锚定 |

## 10. 迁移与向后兼容（三阶段零破坏链）

| 迁移面 | Phase 0 → 1 | Phase 1 → 2 |
|---|---|---|
| 既有代码 | 零改动（构造器内部恒 `anchor: None`，§2.2 加性证明；唯一字面位点 4 处中 1 免疫 + 3 测试机械补） | 零改动（Phase 2 是新方法，非字段变更） |
| 既有文档（.docx） | 读面零变化（读模型 RunData.hyperlink 不动，§2.4）；写面旧行为零漂移 | 同左 |
| 新产物在旧 mdcore 打开 | **读兼容已在**：`Anchor` 变体是既有读模型面（t16 以来），旧版本读新文档的锚点链接照常返回 Anchor 变体 | 书签定义在旧读面照常静默丢（既有登记面，不新增破坏） |
| 新产物在第三方引擎 | 标准 OOXML（w:hyperlink w:anchor + bookmarkStart/End 均为 WordprocessingML 主流面）；Word/WPS/LibreOffice 均原生支持 | 同左（LO headless 侧 r-o2-t2 回归套件可扩锚点用例——实施批联动项） |
| 序列化 | RunSpec 无 serde（§6 已核）——零序列化漂移 | 同左 |
| 撤回 | 删字段+构造器+分支即回 Phase 0；测试 T1-T7 同批删 | 删方法即回；与 Phase 1 正交可独立撤 |

## 11. 校验与错误矩阵（fail-closed 面）

| # | 校验 | 错误 | 依据 |
|---|---|---|---|
| V1 | 书签名非空 | `InvalidInput` | 空名无语义 |
| V2 | 书签名 ≤ 40 字符 | `InvalidInput` | Word 上限（超出在 Word 中被截断/拒绝——包装层显式拒绝优于静默截断） |
| V3 | 书签名首字符为字母 | `InvalidInput` | Word 命名规则 |
| V4 | 书签名仅 `[A-Za-z0-9_]` | `InvalidInput` | Word 命名规则（无空格/无 CJK——**注**：Word 实际允许部分 Unicode 字母，本方案取保守 ASCII 域=跨引擎最稳交集；放宽为后续裁决点） |
| V5 | `_Toc` 前缀保留名拒绝 | `InvalidInput` | 引擎 TOC 内部机制不被踩踏（office-g §3 既有） |
| V6 | 同文档重名拒绝 | `InvalidInput` | §8 重名语义 |
| V7 | `anchor_link` 空 anchor | `InvalidInput` | 同 V1 |
| V8 | anchor 与 link 同 Some | **非错误**——anchor 赢、link 忽略（文档登记语义，读写镜像，T2） | §1.2 优先序 |

**e-review 勘误（anchor-t3）**：初稿「全部经既有 `DocxError::InvalidInput`
变体（无新变体）」失实——mdcore 现行 `DocxError` 仅
Container/Parse/Io/Limit 四变体（docx.rs:212-224）；`InvalidInput`
仅存在于 XLSX 侧 `WriteError`（write.rs:159）。修正后的错误面设计：
**Phase 2 新增 `DocxError::InvalidInput(String)` 变体**（加性枚举变体
——workspace 内 exhaustive match 机械补齐，office-io 外部零消费者
grep 在档 §2.1；这是本方案唯一的公开 API 新增面之一，已在 §1.1
字段扩之外显式登记）。V1-V6 校验全部经该新变体。
V4 的 ASCII 域取舍是**显式设计决定**（非能力缺失）——CJK 书签名需求
出现时升级为独立裁决点（Word 支持 ≠ 全引擎支持，跨引擎保真是本
管线约束——office-f 路线约束传导）。

## 12. 备选方案与最小性论证（承接 r5-t2 §13 四选项）

| 方案 | 内容 | 不选理由 |
|---|---|---|
| **Option 0 维持现状** | 读面四变体 + 写面 External-only + 回放静默丢 | 对称缺口永久化（读 4/写 1）；锚点写面能力为协同 R4.5 强绑定的前置依赖（yrs-b 联动） |
| **Option A（本方案）RunSpec.anchor 加性扩** | 单字段 + 单构造器 + 单发射分支 | ——（选定：4 字面位点全量枚举 = 1 免疫 + 3 机械补；零 serde；零 wasm；G1 +7~9 门禁预期） |
| Option B BlockTree MarkSpan URL 域扩 | 链接语义进 mdcore 公开模型（MarkSpan 载荷） | mdcore MD 域上游裁决未立项（t22-review ②登记面）；改动面跨仓跨域（最小性否决） |
| Option C 引擎直通 | 暴露 docx-rs 原生 Hyperlink/Paragraph 类型 | 引擎类型泄漏公开 API（净室纪律否决——office-io 公开面零 docx-rs 类型是既有不变量） |

**最小性论证**：Option A 是**唯一同时满足**（a）公开 API 零引擎类型、
（b）既有调用方零漂移（加性证明 §2.2）、（c）读写对称（探针②⑤ 已证
镜像可行）、（d）可撤回（删分支即回 Phase 0）四个约束的方案。扩面
止损线：**不扩 RunSpec 以外任何公开类型**；书签定义写面（Phase 2）
以独立方法形态与 Phase 1 可分离裁决（悬空链接 = 合法形使两相位
非依赖，§0 表）。

## 13. 实施批触发与门禁预期（维持 §4，增补锚点）

Phase 1+2 实施批（用户裁决触发后）：T1-T7 + V1-V8 校验面测试
（+4~6 预期）+ e2e docx_export 锚点断言；G1 预期 +11~15；PDF 四面/
clippy/fmt/scan/wasm32/web 七套件零漂移（纯 office-io 面）；LO 侧
回归套件可同批扩锚点存活用例（r-o2-t3 REGRESSION 联动项）。
