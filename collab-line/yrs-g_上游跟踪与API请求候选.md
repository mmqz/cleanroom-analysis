# 上游跟踪与 API 请求候选（yrs-g）

> **性质**：G1 #19「上游协同维持跟踪 + 两项 API 请求候选择机呈上游」
> 的跟踪台账 + 候选定稿件（呈递时机 = 治理决策，本件不自行提交——
> DECISION_GROUPS G1 组「上游交互时机」明示节奏为治理决策）。
> **基线**：2026-10-18 API 复查（d1p-t5 同日）。

## 1. 上游状态快照（2026-10-19 增量复查 · 基线 2026-10-18）

- **2026-10-19 增量**（a5-r5）：0.28.0 后新提交 2 项——#681 ywasm
  applyUpdate 返回变更标志 + 暴露事务插入（ywasm 面，不影响本仓
  Rust 消费）+ aaa1f50 fuzzy 测试死锁修复（0.28.0 发布前夜，已在
  钉版内）。**无新版本发布**（crates.io max=0.28.0 维持）。#676
  仍 open（updated 2026-09-29 零漂移）。
- 基线（2026-10-18）：#676 open / 0.28.0 = 钉版 / 撤回触发零触发。

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


## 5. 缺陷候选 #4（2026-10-19 p20 发现，未呈递——本仓复现在档）

**标题（英文呈递稿）**：Undo of a capture-window-merged stack item
applies only its first sub-range and subsequent redo panics in
`Store::materialize` (multiple UndoManagers with disjoint origins)

**触发面（p20 四配置矩阵定谳，bin p20_undo_window_repro）**：
- 复现：**两个及以上 UndoManager 各自 track 不同 origin** +
  `capture_timeout_millis > 0` + 同窗内同 origin 多次编辑被其他
  origin 编辑隔断（非连续区间合并项）
- 表现①：undo 对合并项仅应用首个子区间（H1..H2 合并项被 A1 隔断
  → 仅 H1 移除）
- 表现②：后续 redo 于 `yrs::store::Store::materialize`
  `Option::unwrap()` on None panic（经 `UndoManager::pop_blocking`）
- 干净对照：单 manager（全窗连续合并）/ 双 manager + timeout=0 /
  单 manager + timeout=0 三配置全部正确

**最小复现（raw yrs 0.28，Text 根）**：
```rust
// 双 manager（include_origin 各 "human"/"ai"，expand_scope 同根，
// capture_timeout_millis = 1000）
// 同窗交错追加："H1"(human) "A1"(ai) "H2"(human) "A2"(ai)
// → undo(human) 得 "A1H2A2"（期望 "A1A2"——仅首段应用）
// → redo(human) panic: store.rs:340 Store::materialize unwrap None
```

**影响**：适配层现产配置（双 manager + 1000ms 窗——B2.1(b) 口径）
下公开 API undo/redo 可达：交错 origin 同窗编辑 + undo + redo =
进程 panic。与 #676（encode_diff heap UAF）独立。

**处置（呈批，非本仓自决）**：
- 选项 A（推荐）：适配层捕获窗改 0（cfg-2 实证干净）——撤销粒度
  变逐事务（产品语义变更 → DECISION_GROUPS G2 批注项；p19 实测
  粒度成本 437.9 B/操作从上界转常态）；
- 选项 B：维持 1000ms 窗 + 临时禁用公开 undo/redo 面（等上游修
  复——呈递本候选 + 跟踪）；
- 选项 C：维持现状知情（钉定测试 soak-3 在档——每跑必红提醒）。
- 上游呈递与候选 ①②③ 合批（§3 时机选项默认 a 维持）。
**上游比对（2026-10-19 a5-r5 复查）**：经 issues 搜索**未发现同
缺陷报告**——最近邻 #674（redone 项与邻居合并 → follow_redone
过删，yjs#806 对应）**机制不同**（p20 无 redo 链参与，首撤销即
部分应用）；#665（事务中 drop manager panic）另族。p20 = 独立
新缺陷确认（同 UndoManager 子系统家族图谱：#665/#666/#667/#674/
#676/p20——呈递稿可附家族地图助上游定位）。
