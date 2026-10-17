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
    // …既有 10 字段零变化（text/bold/italic/mono/strike/underline/
    //    link/footnote/field——docx.rs:2205-2231）…
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
    pub fn bookmark_paragraph(&mut self, runs: &[RunSpec], name: &str);
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
