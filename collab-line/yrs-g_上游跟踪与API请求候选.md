# 上游跟踪与 API 请求候选（yrs-g）

> **性质**：G1 #19「上游协同维持跟踪 + 两项 API 请求候选择机呈上游」
> 的跟踪台账 + 候选定稿件（呈递时机 = 治理决策，本件不自行提交——
> DECISION_GROUPS G1 组「上游交互时机」明示节奏为治理决策）。
> **基线**：2026-10-18 API 复查（d1p-t5 同日）。

## 1. 上游状态快照（2026-10-18 GitHub API / crates.io）

| 面 | 状态 | 影响 |
|---|---|---|
| #676 heap UAF | **open**（updated 2026-09-29，0 评论，0 标签） | skip_gc 全编队不变量维持（B1.1）；快照门/显式压缩点断言在档 |
| crates.io 最新版 | **0.28.0**（2026-09-17 发布）= 当前钉版 | 无升级压力；无破坏性变更风险面 |
| 撤回触发三条 | **零触发** | ①内存安全面 6 个月无进展 + 同类涌现——#676 open 未满窗且无同类新增；②破坏性版本——无新版；③性能低于产品需求——正式基准未建（实施轮验收项） |

## 2. API 请求候选定稿（三项，英文呈递稿就绪）

### 候选 ① GC 内容类可见性（yrs-d §2.2-2 发现）

**背景**：`Update.blocks` 为 `pub(crate)`；Deleted 内容项解码期不带
标志 → `insertions(false)` 同样计入 → 干净源与压缩源 wire 观测面
全同（四源核验）。适配层快照门（P7b3）依赖构造期强制，wire 级
复核无辅助面。

**请求稿**（issue 草案）：

> **Title**: Expose a way to detect compacted/deleted content in a
> decoded `Update`
>
> **Body**: In yrs 0.28, `Update.blocks` is `pub(crate)` and decoded
> `Deleted` content entries carry no public flag, so
> `insertions(false)` counts them the same as live insertions. As a
> result, a diff produced from a GC'd document store is wire-
> indistinguishable from a clean-source diff at the API level.
>
> Would you consider exposing either:
> 1. a public content-kind enumeration over update blocks (e.g.
>    `Update::content_kinds()`), or
> 2. a minimal helper `Update::has_compact_deleted_content() -> bool`?
>
> Use case: snapshot gates in CRDT adapters that must verify a peer
> update was produced without garbage collection (we currently enforce
> `skip_gc` at construction and would like a wire-level cross-check).

### 候选 ② 撤销栈公开逐出 API（B3.4×R-3 协同裁决面）

**背景**：yrs 0.28 UndoManager 栈上限 API 缺失（`observer_*` 私有，
无逐出面）；redo 返回栈项（d1e-t1 钉定）加剧无界增长。G3① 推荐默认
= 维持无限 + 运维监控——若上游提供逐出面，未来可改细粒度上限。

**请求稿**（issue 草案）：

> **Title**: Public API for undo stack depth / eviction
> (UndoManager)
>
> **Body**: `UndoManager` in yrs 0.28 exposes no public API to read
> the undo stack depth or evict entries (the `observer_*` hooks are
> private). Long-lived sessions accumulate unbounded undo history
> (and redo re-pushes returned entries), so memory grows as a linear
> function of document + edit history.
>
> Would you consider exposing:
> 1. `UndoManager::undo_stack_len()` / `redo_stack_len()` (read-only
>    observability), and/or
> 2. an eviction method (e.g. `pop_oldest(n)` or an optional
>    `stack_limit` option, mirroring yjs `trackedOrigins` limits)?
>
> Use case: session-scoped undo with operational memory monitoring;
> today we can only measure stack depth indirectly.

### 候选 ③ XML 元素原位改名（p16 D-1 发现，新）

**背景**：载体无原位改名 API → SetBlockType 翻译 = 删旧建新搬子树
（收敛正确，字节面非最小 + 子树载体身份换族）。D-1 已裁决建议
「接受」；改名优化为上游增强候选非阻塞项。

**请求稿**（issue 草案，择机与 ①② 合批或单独）：

> **Title**: XmlElement: in-place rename (tag change) API
>
> **Body**: `XmlElementRef` has no way to rename a tag in place, so
> semantically renaming a node today requires delete + reinsert with
> a rebuilt subtree. This converges correctly but emits a
> delete+insert pair on the wire instead of a minimal rename, and
> rotates subtree identities (StickyIndex).
>
> Would an in-place rename primitive (or a rename-aware minimal
> encoding) be feasible? Happy to contribute a proof-of-concept if
> there's interest.

## 3. 呈递时机选项（治理决策，非本件自决）

| 选项 | 时机 | 权衡 |
|---|---|---|
| a | 实施轮启动时（D1 裁决后） | 与实施需求同步；若上游采纳，实施轮晚期即可受益 |
| b | 观察窗口后（#676 满 6 个月窗 = 2026-03 首报 + 6M ≈ 2026-09 已过 → 下一复查点即评估撤回触发①时） | 与供应链健康度复核合拍 |
| c | 用户指示即呈 | 最直接 |

**默认建议**：a（实施轮启动时合批呈递三项——单次交互成本最低，
且 ①② 正是实施轮验收门禁的辅助面需求）。

## 4. 跟踪纪律（维持项）

- 每次环境重置恢复后 + 实施轮启动日：#676 状态复查（B1.5）；
- yrs 新版本发布：映射矩阵（yrs-e）+ 适配层（B1.2 单点）影响面
  核对，破坏性变更命中适配层无法吸收面 → 撤回触发②评估；
- 撤回触发③（性能）：正式基准套件（yrs-a §8.1）建成后生效。
