# P2 模块依赖与状态序列化审计（p2-state-serialization.md）

> Task：t7-p2p3-survey（P2 半区）。基线：mdcore main=03815b6 本轮克隆亲读，全部锚点本轮实测。
> 证据级别标注按 PLAN.md §2（五级）。纪律：只读审计，性能断言零容忍（无基准数据只写"潜在"+【推断】）。
> 默认面 259 测试已全绿（任务简报给定，本轮未重跑；测试结论均来自亲读测试源码，落级【测试验证】指"读源码确认断言内容"）。

## 0. P1 勘误（本轮发现，2 项）

| # | P1 原文 | 实测 | 证据 | 影响 |
|---|---------|------|------|------|
| E1 | p1-structure.md §1 O2 称 wasm 桥"**19 导出**" | 实际 **23** 个 `#[wasm_bindgen]` 方法（含 constructor 与 2 个 static）；P1 自己的枚举清单本身即列出 23 项符号，仅标题计数笔误 | crates/mdcore-wasm/src/lib.rs:25,36,59,79,83,92,97,104,110,118,126,133,143,151,157,172,176,180,184,190,194,203,210【代码确认】 | 轻微（枚举完整正确，计数失真） |
| E2 | p1-structure.md §2.2"fingerprint.rs → tree/id/markdown" | 漏记 `use crate::Core`（fingerprint.rs:25）——fingerprint.rs:815-842 含 `impl Core`（`fingerprint_manifest` / `match_external_markdown` 两个 Core 方法落在此文件），即 fingerprint 模块**反向依赖 lib.rs 的 Core 类型** | crates/mdcore/src/fingerprint.rs:25,815-842【代码确认】 | 中等：复用抽取 fingerprint.rs 必须连带 Core 聚合或剥离 impl Core 段，P1 闭包表按原样沿用会低估拖带面 |

注：P1 各模块行数（lib.rs 1187 / history.rs 301 / fingerprint.rs 880）与本轮 wc -l（1186/300/879）存在 ≤1 行差异，属行尾计数口径，不作勘误登记。其余 P1 事实（模块清单、依赖方向、四条链锚点、Reject 四变体、COALESCE_MS、should_coalesce:91、BoundarySnapshot:84、IdAllocator、snapshot validate:148、save/open:220/234、replay 不重建 undo 栈等）本轮逐条核验**全部属实**。

## 1. 模块依赖闭包表（复用单元口径）

外部依赖全表（mdcore crate）：`ropey 1.6` / `serde(derive,rc)` / `serde_json`，仅此三项（crates/mdcore/Cargo.toml:7-10【代码确认】）。核心五件（tree/id/op/history/anchor）不依赖 serde 以外的任何模块；serde 仅在 derive/序列化边界（根 Cargo.toml:27-28 注释"tree/op 本身可脱离 serde 编译"【文档记录→本轮结构核验：op.rs derive 之外无 serde 调用】）。

| 复用候选 | 模块级依赖（use 实测） | crate 内反向依赖（lib.rs/Core） | 外部依赖 |
|---|---|---|---|
| **op 求逆/应用**（op.rs） | tree.rs（op.rs:15 use Attrs/BlockKind/BlockTree/MarkSpan/NodePath/TreeError）；id.rs（op.rs:116 全限定 `crate::id::BlockId`，无 use 但编译依赖） | **无**（apply_op/invert_op 均为自由函数） | serde（仅 derive）、std::sync::Arc（OpText） |
| **undo 管线**（history.rs） | op.rs（history.rs:18）；anchor.rs::Selection（:17，Entry.selection 字段） | **无** | serde |
| **块树**（tree.rs+id.rs） | id.rs（tree.rs:15） | **无** | ropey、serde |
| **序列化·markdown**（markdown.rs） | tree.rs（markdown.rs:11，仅此） | **无** | **零外部**（无 serde——纯文本解析/序列化） |
| **序列化·snapshot**（snapshot.rs） | tree.rs + id.rs（snapshot.rs:19-20） | **无** | serde |
| **序列化·revision**（revision.rs） | op.rs + history.rs::Source（revision.rs:13-14）；document.rs::DocumentId（:26 全限定） | 无（但传递拖入 document.rs） | serde |
| **序列化·history JSON**（history.rs 内 to_json/from_json） | 同 history 闭包 | 无 | serde |
| **序列化·document/meta**（document.rs） | markdown.rs（document.rs:21）；Core 与 Reject（:179/:22 use） | **有**（impl Core：save/open/set_document_id 落在此文件） | serde |
| **指纹匹配**（fingerprint.rs） | id + markdown + tree（fingerprint.rs:22-24）；**Core（:25，P1 漏记，见 E2）** | **有**（impl Core 段 ：815-842） | serde |
| **外部采纳**（external.rs） | fingerprint + history::Source + op + tree（external.rs:18-21）；`crate::{ApplyMeta, Core, Reject}`（:22）；markdown（:106 全限定） | **有**（impl Core：adopt_external 落在此文件） | serde |
| 参考：html.rs | tree.rs（html.rs:57，仅此） | 无 | 零外部（仅 tree；html.rs 全文零 serde 引用——复审修正） |
| 参考：debug.rs | id + tree（debug.rs:9-10） | 无 | serde（仅 Serialize） |
| 聚合：lib.rs Core | 全部 13 模块（lib.rs:16-28） | — | serde/serde_json |

**闭包判定摘要**：
- 最小独立闭包 = **tree+id（+ropey）**；其上叠加 op = "求逆/应用单元"（仍无 Core 依赖）；再叠 history+anchor = "undo 数据单元"。核心五件无反向依赖【代码确认】。
- 三处 **impl Core 外置**（fingerprint/document/external 各自文件内含 `impl Core` 块）构成模块→lib.rs 反向耦合：单抽这三个文件必须连带 Core 聚合体或剥离方法段【代码确认】。
- crate 级：mdcore 被依赖方向不变（wasm/shell → mdcore；pdf 零依赖，复用 P1 §2.1 结论，本轮经 Cargo.toml 抽查一致）。

## 2. 状态模型形状判定

- **块树组织**：arena 槽位树 `slots: Vec<Option<Node>>` + 虚根（root=NodeId(0)，保留 BlockId(0)，**不经 create_node 构造故不入 id_index**——`lookup_id(BlockId(0))` 恒 None，对外不可寻址）+ `id_index: BTreeMap<BlockId,NodeId>` 注册表 + `pub alloc: IdAllocator`（tree.rs:98-122【代码确认】）。槽位**不压缩**：remove_subtree 置 None（tree.rs:269-286），slot 编号只增不复用（slot() 只 push，tree.rs:142-146）——NodeId 与 slots.len() 均为历史峰值单调量【代码确认】。
- **双轨寻址**（ADR-B1 混合式）：mutation/重放走 NodePath（子索引链），意图/锚点走 BlockId（注册表，结构变化后仍可解析）（tree.rs:202-223【代码确认】）。
- **op log ↔ 树**：**op log 可重放重建树**。`replay_jsonl` 语义（lib.rs:966-996【代码确认】）：空核逐记录 raw apply（每记录内 backup 失败即整体回滚 + Err）；每记录后 `core.revision = r.revision`（**赋值非 bump**——恢复日志记载的 revision 序列而非重新计数）；document_id 取首个非 None 携带值（genesis），多记录冲突 → Err 拒绝（:973-977）。
- **undo 栈为什么不重建**：重放只产生"前向等价终态"——undo/redo 已作为普通 op 序列入日志（R11 修正，lib.rs:891-894），重放结果 = 实时状态，但历史栈/合并窗状态/选区均不在日志中（revision.rs:8"精确状态复原走 snapshot"【代码确认】）。设计分工：**snapshot = 精确状态复原（树+revision+分配器水位，undo 栈不入，snapshot.rs:4-5）；op log = 自含审计轨迹/重放；history JSON = undo 栈跨会话**（save_document 把整栈嵌入 meta）。
- replay 确定性隐含约束：日志记录的 InsertBlock 必须已物化 ID。实测所有 `log.record` 调用点均在 apply_op 物化之后（lib.rs:290/310/330 记录的是物化后 ops；undo/redo 记录 entry 内物化 op）【代码确认】——"日志自含"依赖"入核事务全部经 apply_ops"这一纪律。

## 3. Block ID 稳定性

**生成机制**【代码确认】：IdAllocator `next` 从 1 起（id.rs:29），allocate 单调 +1（:32-36），`observe` 抬升水位（:38-42），**永不回收**；根保留 ID 0 不参与分配。测试锁：id.rs::monotonic_never_recycles【测试验证】；水位抬升入树路径 = create_node(Some)（tree.rs:178-184）。

**保存/重载后稳定性——按三路持久化路径分别判定**：

| 路径 | 树 ID 保真 | 机制 | 测试锁 |
|---|---|---|---|
| A. save_document（md+meta）→ open_document | **条件性**（见下） | md 不含 ID（markdown.rs:8 §8.3；id.rs:5"不写入 Markdown，持久身份走 workspace metadata 映射"）；meta 仅 document_id+revision+history（document.rs:224-228）；open = from_markdown **重解析现场按先序重新分配**（document.rs:253） | document_progressive::save_open_roundtrip_restores_all_state、::document_id_survives_save_open_and_replay（两测试的 preorder_fingerprint **含 id 字段**，document_progressive.rs:30-36，断言保存前后树指纹全等）【测试验证】 |
| B. snapshot JSON → restore | **全保真** | to_snap 逐节点拷贝 data.id（snapshot.rs:183-192）；validate 拒收撞号 | spike_semantics::spike_a_snapshot_roundtrip_preserves_state_and_ids、r14_gates::gate3_snapshot_restore_identity_at_1mb（1MB 级逐块恒等含身份）【测试验证】 |
| C. revision JSONL → replay | **全保真** | genesis 物化 ID（lib.rs:176-183 materialize_ids=true）+ 全记录物化 op | r11_stress::move_and_undo_keep_identity_at_scale、r12_ai_loop::ai_inclusive_log_replays_to_identical_state、r14_gates::gate1_core_ingest_and_replay_at_1mb【测试验证】 |

**路 A 的保真条件**【推断，依据链】：open 后树 ID = 重解析按文档先序的 1..n 连续序列；保存前活树 ID = 会话历史累积分配值。两者相等 ⟺ 活树先序 ID 序列恰为 1..n 无洞递增。现有两测试场景满足该条件（纯文本编辑不改结构不改 ID；System extend 只在尾部追加且新 ID 接续水位）。**结构性编辑（中段删除留洞 / move 重排 / 中段插入）后走路 A，树 ID 必然漂移**——该场景现有测试未覆盖，属边界缺口登记（复用方若依赖跨 save→open 的 BlockId，此为硬约束）。
**路 A 连带边界（机制【代码确认】+触发【推断】）**：history 经 meta 复原后，条目内物化 InsertBlock 的 id 在 undo→redo 重放时经 create_node→register（tree.rs:154-156，BTreeMap::insert **覆盖无守卫**）落树；若该 id 与重解析分配的存活块撞号，树内将出现重复 BlockId 且旧块从 id_index 失联。现有 save→open 测试均为文本编辑场景（无可重插条目），未触达此路径；属未锁边界，登记供 P7 评估。

**跨会话稳定性**：document_id（UUIDv7，document.rs:41-69，零依赖熵源 :25-37）持久化到 meta 与 genesis；**BlockId 本身不持久化**（除 history/op 中附带）。document_id 与 BlockId 是两层身份（文档级/块级），互不派生【代码确认】。

**外部引用可行性（仅登记事实）**：Anchor=(BlockId,offset) 可跨结构变化解析（ADR-B1）；跨 save→open 的块级引用在结构性编辑场景不可靠（路 A 漂移）；跨 snapshot/replay 可靠。块的 ID→内容持久映射侧车机制已存在形状：`fingerprint_manifest()`（fingerprint.rs:817，注释"R23-c 存进 meta 侧车的形状"）+ adopt_external 的 manifest 快路径（external.rs:104）——但 save_document **不**输出 manifest，该面当前无宿主消费者。

**r17_id_boundaries.rs 15 条测试锁定内容**【测试验证，逐条亲读】：
- 域1 表格（4 条）：表=单块，表内编辑/加行/改写均为块内 ReplaceText 或原位 op，ID 保持不产新块（::table_single_block_row_edit_keeps_identity 等 4 条）；
- 域2 嵌套列表（5 条）：split 尾块新 ID+marker/depth 继承且既有块（含祖先与子树）ID 不变；merge survivor=前兄弟、被并块失效不回收；sync 子块插入到正确父位且既有 ID 全保持；
- 域3 跨块 paste（6 条）：sync 按位置对齐规则——**对齐位保 ID（错位槽被原位改写：ID 跟位置不跟内容）**、净增尾部新 ID（永不复用旧号）、尾部删除高位→低位、**undo/redo 身份恒等（物化 ID 复原）**。

## 4. op 模型（7 原子 op + 可逆性矩阵）

Op 枚举 7 成员（op.rs:24-66，serde tag="t" snake_case）：InsertBlock/RemoveBlock/ReplaceText/SetBlockType/SetAttrs/SetMark/MoveBlock。

| op | invert_op 逆形状 | 逆成本 | 特殊语义/已知边界 |
|---|---|---|---|
| InsertBlock | `[RemoveBlock(join(parent,index))]` 长度 1 | O(1) 构造 | 逆不含子树（插入时块尚无子树；子块由后续 op 建）【代码确认 op.rs:245-247】 |
| RemoveBlock | **先序重建整棵子树的 InsertBlock 序列**，每节点 `id=Some(data.id)` 物化复原 | O(子树)——collect_subtree 全拷贝（文本 Rope→String）+ build_rebuild_ops | §8.2 身份复原的关键路径【代码确认 op.rs:248-256,346-364】 |
| ReplaceText | ReplaceText(同 path, `start..start+insert_len`, insert=state-before 被删文本) | O(编辑区)（slice 拷贝） | 需 state-before；`start>end \|\| end>len` → BadRange（:265-267） |
| SetBlockType | SetBlockType(旧 kind) | O(1) | Document kind 保留（apply 拒绝，:143-145） |
| SetAttrs | SetAttrs(旧 attrs 克隆) | O(attrs) | — |
| SetMark | SetMark(**方向由 state-before span 存在性决定**) | O(1) | **R24-a2**：no-op（add=true 且已在 / add=false 且不在）→ 逆=同向（幂等）；有效变更 → 反向。此前无条件翻转 add 违反精确恢复（op.rs:290-324）【测试验证 r24_quality_fixes::probe_redundant_add_mark_undo_must_be_noop / probe_remove_absent_mark_undo_must_be_noop】 |
| MoveBlock | MoveBlock(from=`to_parent+[to_index]`, to=原父+原下标) | O(1) | **摘除后语义**（to_index 为 detach 后下标，tree.rs:288-312 环检查+先摘后插）；**from 空路径 → Err BadPath**（R11 压测逮到的减法溢出 panic 边界，op.rs:330-333）【测试验证 r11_stress 50 次 MoveBlock 往返】 |

附带：`adjust_marks` 四情形区间映射（op.rs:196-239：前不动/后平移/包含吞入/部分重叠截断）【测试验证 spike_semantics::spike_e_marks_edit_semantics；跨区间删除截断=v0 有损语义，测试内注明登记 README】。

## 5. 加载/保存路径（五格式盘点 + 往返锁 + 增量保存判定）

| 格式 | 承载内容 | 消费者 |
|---|---|---|
| md 明文 | 块内容+结构（**marks 不写入、ID 不写入**，markdown.rs:8） | 宿主 fs、wasm to_markdown、CM6 sync 兜底 |
| meta JSON | document_id + revision + **整栈 history JSON 嵌入**（document.rs:224-228） | open_document（document.rs:234，畸形显式 Reject） |
| snapshot JSON | revision + next_id（分配器水位）+ 递归 SnapNode 全树 | Core::restore / wasm restore；from_json 反序列化即 validate（T1-O4 四不变量拒收） |
| history JSON | undo/redo 双栈 + 合并窗三字段（last_at_ms/last_kind/last_source，history.rs:54-66） | set_history_json / save→open；S6 往返 |
| revision JSONL | 每事务一条 LogRecord（revision/ops/source/command/document_id-仅 genesis）（revision.rs:16-27） | Core::replay_jsonl / wasm replay；**append-only 格式设计**（revision.rs:9"便于增量 append 与截断"） |

**往返保真语义锁清单**（哪种往返被哪些测试锁）【测试验证，均亲读断言】：
- md parse→serialize 幂等（canonical 方向）：spike_semantics::spike_a_markdown_roundtrip_idempotent；r13_html_roundtrip 19 条（HTML 读侧独立面）。
- snapshot 往返（含 ID+revision）：spike_a_snapshot_roundtrip_preserves_state_and_ids；r14_gates::gate3_snapshot_restore_identity_at_1mb（逐块恒等+blocks_json 逐字节+MD 恒等+revision）+ gate3_restore_then_editing_and_undo_continue（restore 后可继续编辑/undo 跨档）。
- history JSON 往返：spike_d_history_json_roundtrip_and_log_replay（S6）；document_progressive::history_json_roundtrip_with_system_boundary_state（**System 边界态无损**——last_source=system 随 JSON 落盘，往返逐字节一致）。
- save→open 往返：document_progressive::save_open_roundtrip_restores_all_state（树指纹含 ID + revision + history_json 三一致；meta 畸形三例显式失败）；::document_id_survives_save_open_and_replay（save/open/replay 三路一致）；shell lib.rs::full_open_save_roundtrip_keeps_identity。
- replay 往返：r11::move_and_undo_keep_identity_at_scale、r12::ai_inclusive_log_replays_to_identical_state、r14::gate1_core_ingest_and_replay_at_1mb、document_progressive::genesis_carries_document_id_and_replay_restores（含旧格式向后兼容 + 冲突 document_id 拒绝）。

**serde 演化纪律（schema evolution，序列化审计附属观察）**【代码确认】：
- Op：`attrs`/`text`/`id` 均带 `#[serde(default)]`（op.rs:29-35）——旧 JSON 缺字段可反序列化；`id: Option<u64>` 缺省 None=现场分配。
- LogRecord：`command` 与 `document_id` 带 default + `skip_serializing_if`（revision.rs:21-26）——旧 JSONL 兼容已被测试显式锁（document_progressive::genesis_carries_document_id_and_replay_restores 剥掉 document_id 字段重放 = 无身份档照常打开）【测试验证】。
- History：`last_at_ms`/`last_kind`/`last_source` 带 default + skip（history.rs:58-65）——旧栈 JSON（无边界字段）可读，边界语义按 None 处理（即不与新击键合并）。
- 演化约束面：**加字段=可（带 default）；加枚举值=破坏兼容**（Source/Kind/Op/Reject/BlockKind 全部封闭枚举，serde 遇未知 tag 报错）。

**增量保存能力三态判定**：
- **宿主实际保存面 = 无增量**【代码确认】：核内零 fs（mdcore src 全文无 std::fs 使用，唯一命中为 external.rs:5 注释声明"无 std::fs"）；宿主唯一保存面 save_document = md 全文 + meta（含整栈 history）**全量重写**（document.rs:220-230；shell 侧透传 lib.rs:124-126）；SessionRecord 持久化仅窗口→document_id+path 映射（shell session.rs:213-227），不含文档内容。
- **格式层 = 部分（条件：仅 JSONL 设计就绪、零落盘消费者）**【代码确认】：RevisionLog.record 仅内存 Vec push（revision.rs:50-58）；to_jsonl 为全量 dump；注释明示 append-only 设计意图（revision.rs:9），但 workspace 内无任何"增量 append 到持久化"的实现或调用点（grep 实测 replay/history_json 消费面均为一次性全量读入）。
- 判定词：**无（宿主面）**；潜在能力=revision JSONL 行式 append-only 格式（设计在档未接线）。

## 6. Revision / Anchor

**revision 单调性来源**：`Core.revision` 私有 u64，`bump_revision` +=1（lib.rs:360-363）。bump 点全表【代码确认】：
- 非合并事务（Entry 构造处，lib.rs:321）；
- System 事务（lib.rs:288）；
- undo（lib.rs:890）/ redo（lib.rs:919）。
**不 bump 的路径**：空 ops 早退（:254）；**coalesce 分支**（lib.rs:306-318——`log.record(self.revision, …)` 直接用当前值，合并击键与组首共享同一 revision，事件 meta.coalesced=true；日志同 revision 号多条记录，重放按序合并语义等价，revision.rs:3）；sync 空差分（经 apply_ops 早退）；adopt Noop（external.rs:109-117 零事务）。
即 **revision = "已封口事务 + undo/redo"的计数**，合并窗内击键不各自计数【测试验证 spike_d_coalescing_window（"合并击键共享一个 revision"断言）+ r12::ai_text_command_bumps_own_revision_never_coalesces】。
**revision 非全局唯一**：open_document 直接覆写 revision=meta 值（document.rs:257）；replay 逐记录赋值（lib.rs:992）——跨 Core 实例不可比较【代码确认】。

**Anchor 定位语义**（anchor.rs + lib.rs:672-696【代码确认】）：
- resolve_anchor：lookup_id → AnchorError::Missing（块删，**绝不静默猜测**）→ path_of → offset > len → OutOfRange → 返回 (path, offset)。
- ByAnchor 文本命令四类 typed reject：Missing / **Inverted{start>end}**（T1-F2：修复前 expect=Some 时 ropey slice 反转区间直接 panic，wasm replace_text_at_anchor 公开可达；现入口前置拦截，expect=None/Some 统一防线，lib.rs:712-720）/ OutOfRange / ExpectMismatch（乐观并发保护，expect 与现场内容比对，lib.rs:732-737）【测试验证 t1_quality_fixes::probe_byanchor_inverted_range_rejects_before_slice + spike_e_anchor_mapping_and_rejects】。
- ByPath 快路径（View 保证 path 新鲜）vs ByAnchor（AI/延迟命令，结构变化后仍可解析）；**偏移平移自动跟移属 R9 增强，v0 未实现**，以 expect 保护（anchor.rs:11）。
- **Anchor = (BlockId, offset) 二元组**（anchor.rs:17-21）；Selection = (anchor, focus) 双 Anchor。文本 op 单位 = char 码点（op.rs:10；r17_utf16_mapping 11 条锁非 BMP 安全含 1 条 fixture 自证，含 property_mixed_scripts_edits_undo_redo_identity 属性测试——复审修正计数）。

## 7. P2 验收对照

| PLAN P2 验收项 | 结果 |
|---|---|
| ① 复用单元依赖闭包清单 | ✅ §1（六候选全覆盖 + 反向耦合三处新发现，含 P1-E2 勘误） |
| ② 状态模型形状 + op log↔树 | ✅ §2（重放语义/限制/undo 不重建原因） |
| ③ Block ID 稳定性 | ✅ §3（三路判定 + 路 A 条件保真【推断】+ 未锁边界登记 + r17 15 条锁定内容） |
| ④ op 可逆性矩阵全成员 | ✅ §4（7/7 + 边界） |
| ⑤ 加载/保存 + 往返锁 + 增量三态 | ✅ §5（判定：宿主面无；格式层部分） |
| ⑥ Revision/Anchor | ✅ §6（coalesce 不 bump 关键事实 + Anchor 语义） |

## 8. 遗留未知/待查项

| # | 项 | 级别 | 待查路径 |
|---|---|---|---|
| U1 | 路 A（save→open）结构性编辑场景 ID 漂移的实际面（哪些真实编辑序列会破坏外部持有的 BlockId）——机制已明，**面的大小**未量化 | 【推断】（机制代码确认） | 构造 delete/move 后 save→open 的对照实验（复用方可自行跑）；或查 web/main.ts 是否持有跨档 BlockId（P5 范畴） |
| U2 | 路 A 重开 redo 物化 ID 撞号（§3 登记）在真实会话的可达频率 | 【推断】（触发条件未测） | 定向测试：结构编辑→save→open→undo→redo，断言 id_index 无重复；属"需新增验证项"候选（P7 输入） |
| U3 | r14_gates 打点的 MD 面耗时数字（gates.md 历史登记）本轮未运行采集 | 【文档记录：本轮未复验】 | 需要时跑 r14_gates 读 println 输出；本报告不引用任何数字 |
