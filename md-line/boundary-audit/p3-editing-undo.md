# P3 编辑事务/撤销审计（p3-editing-undo.md）

> Task：t7-p2p3-survey（P3 半区）。基线：mdcore main=03815b6 本轮克隆亲读，全部锚点本轮实测。
> 证据级别按 PLAN.md §2。默认面 259 测试已全绿（任务简报给定，本轮未重跑；【测试验证】= 亲读测试源码确认断言内容）。
> P1 勘误 2 项（wasm 导出计数、fingerprint 反向依赖 Core）见 p2-state-serialization.md §0，本文件不重复登记；P3 范围内未发现 P1 结构性错误。
> 与 P2 的分工：状态/序列化/ID/revision 细节归 p2-state-serialization.md；本文件专注事务与撤销链，交叉处以"p2 §n"引用。

## 1. 命令/事务管线

**commit_transaction / commit_transaction_as 签名与语义**【代码确认 lib.rs:600-618】：
- `commit_transaction(base: u64, ops: Vec<Op>)` = 委托 `commit_transaction_as(base, ops, ApplyMeta::view("transaction"))`（:600-602）。
- `commit_transaction_as(base: u64, ops: Vec<Op>, meta: ApplyMeta)` = `self.gate(base)?` → `self.apply_ops(ops, meta)`（:610-618）。R12 动机：此前带 gate 的命令面硬编码 Source::View，AI 二选一丢溯源或丢 stale gate；本方法让调用方同时声明两者，**执行仍走既有 gate→apply_ops 管道，无旁路**。
- `gate(base)`（:378-385）：`base != self.revision` → `Reject::Stale { current }`——结构性 stale **必 reject 不静默 rebase**（§11.2）。

**无旁路验证（十原语 + 三个装载通道）**【代码确认，逐条】：insert_block:397 / delete_block:411 / set_block_type:419 / set_attrs:432 / move_block:445 / split_block:524 / merge_block:560 / commit_source:654+667 / replace_text:566 / set_mark:596——内部全部收敛到 `apply_ops`；sync_from_markdown:774 与 extend_from_markdown:862 亦走同一管道（前者 Source::View、后者 Source::System）。**例外面**：replace_text/set_mark 与 apply_ops 直通道**无 base 门禁**（§11.3 text-local 设计：锚点/区间/expect 保护代替 stale gate）——复用时需知的既定边界，非缺陷。

**事务原子性边界（backup = tree.clone() 回滚，三条路径）**【代码确认】：
- apply_ops：`backup = self.tree.clone()`（:267）；逐 op 先 `invert_op(tree_before, op)` 后 `apply_op`（:269-283，任一失败 → `self.tree = backup` + Err）；求逆基于该 op 之前的现场（顺序求逆协议）。
- apply_raw_batch（undo/redo 专用，:349-358）：同款 backup，**不进历史不 bump**（调用方编排）。
- replay_jsonl 内联循环（:985-991）：每记录 backup，失败整体回滚 + Err。
- **T1-F10 路径**（undo/redo 应用失败全回滚）：条目压回原栈（push_back_undo/push_redo）+ 合并窗边界拨回（restore_boundary，history.rs:152-158）+ 树由 backup 恢复 + revision 不动（lib.rs:879-888, 911-917）。可达面：set_history_json / open_document 载入与当前树不一致的历史（inverses 走 BadPath/BadRange）。

**命令拒绝面 typed reject 清单（全枚举）**：

| 枚举 | 变体 | 触发条件 | 锚点 |
|---|---|---|---|
| Reject（wasm/命令公共面） | Stale{current} | gate：base ≠ revision（结构性命令/事务） | lib.rs:92-95 |
| | Anchor(AnchorError) | 文本命令锚点解析失败（下表四类） | lib.rs:96 |
| | Invalid(String) | OpError 转 String（From，:103-107）；split/merge/commit_source 自定义消息（坏路径/根/无前兄弟）；open_document meta 畸形 | lib.rs:97 |
| | IncompleteChunk(String) | 渐进打开 chunk 悬挂（未闭合 fence 收在 EOF）——**op 构造之前拒绝，树零污染** | lib.rs:98-100 |
| AnchorError | Missing(BlockId) | 目标块已删（lookup_id None，绝不静默猜测 §8.4） | anchor.rs:49 |
| | Inverted{block,start,end} | start > end（T1-F2：expect=Some 时修复前 ropey slice panic，wasm 公开可达；现入口前置拦截，两路径统一） | anchor.rs:55-59 |
| | OutOfRange{block,len} | end > 块文本 char 长度 | anchor.rs:60-63 |
| | ExpectMismatch{block} | expect 与现场内容不等（乐观并发保护） | anchor.rs:64-66 |
| OpError（op 层） | Tree(TreeError) | 结构原语失败（下表） | op.rs:69-70 |
| | BadRange | ReplaceText/SetMark：start>end 或 end>len（apply 与 invert 双侧同款校验） | op.rs:71 |
| | InvalidKind(&'static str) | InsertBlock/SetBlockType 携 Document kind（虚根保留） | op.rs:72,111-113,143-145 |
| TreeError（树原语） | BadPath / BadIndex / Cycle / RootForbidden | 路径不可解析 / index>len / 移动成环 / 根操作被拒 | tree.rs:79-84 |
| SnapshotError（反序列化边界） | DuplicateBlockId / NextIdNotAbove / MarkOutOfRange / ReservedKind | 恶意/损坏 snapshot 注入（T1-O4；from_json 即 validate） | snapshot.rs:57-80 |
| SessionError（shell 会话面） | LimitReached / DuplicateWindow / UnknownWindow / DocumentAlreadyOpen | 会话上限硬门 / 窗口重复 / 未注册 / 一窗一档 | shell session.rs:50-62 |
| DocumentIdError | (String) | DocumentId FromStr 失败（version≠7 / variant≠10 / 长度形状） | document.rs:107-115 |

wasm 面补充事实【代码确认 mdcore-wasm/src/lib.rs:44-49】：source 字符串映射含**静默兜底** `_ => Source::View`——非法 source 字符串被归为 View（无 reject）；复用时经 wasm 桥的溯源完整性依赖调用方自律。

**wasm 桥事务入口面（管线可达性，P5 详查此处仅登记管线相关三条）**【代码确认 mdcore-wasm/src/lib.rs:36-89】：`apply_ops(ops_json, source, command)`（source ∈ ai/undo/redo/system/view 五字符串）、`undo()`/`redo()`（Option，None 经 JsValue 通道不区分栈空/失败）、`replace_text_at_anchor`（ByAnchor+expect，T1-F2 修复的公开可达面）。结构原语（insert_block 等十原语）**不经 wasm 桥导出**——桥上仅有 op 直通道，结构命令由 web 层组装 op JSON 走 apply_ops（与 P1 链①一致）。

## 2. Undo/Redo 全链

**完整链路**【代码确认 lib.rs:875-932】：
- undo：`boundary()` 快照 → `pop_undo`（last_kind=None 破组 + last_at_ms=entry.at_ms，history.rs:136-141）→ inverses.clone() → `apply_raw_batch`（失败→T1-F10 回滚路径）；成功 → push_redo → bump_revision → `log.record(rev, inverses, Source::Undo, "undo")`（R11 审查修正：**undo 入日志**，否则重放与实时状态分叉，:891-894）→ push_event(coalesced:false) → **dirty=true**（undo 回到已存内容仍报脏 = 安全向假阳性，宁可多问存盘，lib.rs:139-145 注释）。
- redo 对称（:907-932，Source::Redo，redo 条目 ops 重放）。

**粒度控制点（全部）**【代码确认 + 测试验证】：
1. `COALESCE_MS = 1000`（history.rs:20 pub const）：合并窗 <1s（`now - last_at_ms < COALESCE_MS`，saturating_sub）。
2. `should_coalesce` 五条件合取（history.rs:91-98）：kind==Text ∧ last_kind==Text ∧ last_source==View ∧ source==View ∧ !undo.is_empty()。
3. **R12 双端 View**：合并要求条目两端都是 View——AI/System/undo 边界后 1s 内人手击键必开新条目（否则一次 undo 连人手字符一起吞掉）。锁：r12::ai_text_command_bumps_own_revision_never_coalesces（断言"合并窗两端都必须是 View"）、spike_d_ai_ops_are_first_class_history【测试验证】。
4. `redo_depth()==0` 附加条件（lib.rs:303-304）：有待重做 redo 时禁止合并（防 push_entry 清 redo 丢重做机会）。
5. kind 切换破组（文本↔结构天然新条目）；undo/redo 后 last_kind=None（pop_undo/pop_redo 破组）。
6. **System 边界破组（ADR-B4）**：System 事务不入 undo 栈（undo 栈=交互编辑史；行为锚点：打开文档不可撤销）但 note_boundary 推进边界（history.rs:108-116）——System 后第一个 View 击键因 last_source≠View 必开新条目。锁：lib.rs::system_apply_ops_filtered_from_undo_stack（同钟同窗断言）、document_progressive::extend_not_undoable_and_system_boundary_breaks_group（固定时钟 42_000 全程同窗，破组断言）、document_progressive::extend_pipeline_events_revision_and_log（revision/log/事件照常，仅历史栈通道跳过）【测试验证】。
7. coalesce_into_last 语义（history.rs:119-134）：ops **尾部追加**、inverses **前插**（保持"逆序对偶"不变量——undo 按数组序应用）、at_ms 更新（窗口续燃：后续同窗击键继续并入同条目）。

**redo 清空时机**：`push_entry`（新条目入栈即 `self.redo.clear()`，history.rs:100-106）——任何开新条目的交互/编辑都会废 redo；coalesce 分支不清（由控制点 4 保证无 redo 可清）。**测试锁缺口登记**：该行为在本轮测试源码通读中未见专属用例（r12:521 的 `!can_redo()` 断言上下文是 redo 栈耗尽而非清空语义），仅有代码级保证【代码确认 history.rs:104 + 缺口如实登记】。

**粒度实测形状（手势级 vs 字符级）**【代码确认 + 测试验证】：
- 实际粒度 = **时间窗级**（同 1s 窗内连续 View 文本 op 并为一条），非词边界、非输入法组合感知。history.rs:10 明示"词边界/输入类型破组、AI 组粒度 → ADR-B1/B2 留档，R9 实测调参"——即 v0 未实现更细的语义分组。
- 实测锚：spike_d_coalescing_window_and_semantic_break（t=0/100/200/300 四击键 → 单条目单 revision；t=400 结构命令破组；t=10_000 出窗新条目）；r12::ai_text_command（Human 三击键合并 → AI 独立 → Human 再独立，undo 链三次分撤）。
- 结构命令（kind=Structural）**永不合并**（should_coalesce 第一条件），任何结构 op 混入事务即整事务按 Structural 处理（lib.rs:259-263：`ops.iter().any(|o| !o.is_text_class())`）——混合事务的合并资格由"最重 op"决定。
- IME/组合输入在核侧无特殊通道：CM6 侧组好 op 再进 apply_ops（P1 链④；web 层职责，核不感知 IME）。

**clear_interaction 语义**（history.rs:180-186）【代码确认+测试验证 r23_external::adopt_take_external_drains_undo】：清空 undo+redo 双栈 + 重置合并窗三字段（last_at_ms=0/None/None）。调用点唯一：adopt_external 成功后（external.rs:160，ADR-B7——被放弃文档态的条目按路径/身份回放会 BadPath，保留即坏引用）；Noop 路径不调用（幂等，r23_external::adopt_noop_idempotent 锁"Noop 不清交互史"）。重载后首个 View 击键独立新条目，不回溯重载前状态。

**脏标记联动**（R24-b，lib.rs:139-145 注释）【代码确认】：粘性语义（只增不减，mark_saved 清位）；View/Ai/Undo/Redo 置位；System 装载不置位（缓冲==外部内容）；undo/redo 置位（安全向假阳性）；adopt 成功 mark_saved。

## 3. 不变量清单（逐条：代码确认 + 测试依据）

**① 精确恢复前态（undo 恢复内容/结构/身份）**
- 代码：invert_op 逐 op state-before 求逆（op.rs:243-344）+ 物化 ID（InsertBlock id=Some，op.rs:352-359）。
- 测试【测试验证】：spike_semantics::spike_b_identity_invariants_matrix（文本/attrs/类型/move/split/merge 全矩阵，6 命令各推进一 revision）；r11_stress::move_and_undo_keep_identity_at_scale（20k 块 50 次 MoveBlock 一次 undo 与初始**逐字节一致**）；sync_resync::undo_reverts_sync（结构+文本 sync 撤销 + 身份复原）；r17_id_boundaries 15 条中 5/7/11/12/15（undo 后身份复原断言遍布三域）；r17_utf16_mapping::property_mixed_scripts_edits_undo_redo_identity（混合文字随机编辑属性测试）；r12::ai_batch_undo_reverts_whole_batch_not_human_edit（AI 整批回退不吞 Human 编辑 + redo 复原 AI 追加块身份）。

**② no-op 幂等**
- 代码：SetMark 求逆方向由 state-before 存在性决定（op.rs:290-324，R24-a2）；空 ops 早退（lib.rs:254）；空差分 sync 不 bump。
- 测试【测试验证】：r24_quality_fixes::probe_redundant_add_mark_undo_must_be_noop（重复加粗 no-op 条目的 undo 必须 no-op）；::probe_remove_absent_mark_undo_must_be_noop；::probe_coalesced_adds_undo_reverts_whole_group（同窗连击并条，单 undo 撤整组——对照锁）；r23_external::adopt_noop_idempotent（Noop 零事务：revision/历史不动）；sync_resync::sync_advances_revision（幂等 sync 不推进 revision）。

**③ 失败全回滚（事务原子性）**
- 代码：backup clone 回滚三路径（§1）；T1-F10 失败路径（条目回栈+边界拨回）。
- 测试【测试验证】：r11_stress::bulk_transactions_are_atomic_and_undoable_at_scale（49 合法 + 1 BadPath → 全回滚，无 XXX 残留）；r12::ai_batch_is_atomic_on_bad_op（坏 op 拖垮整批：树 dump 一致 + revision 不动 + undo_depth=0 + 零事件四断言）；r12::ai_stale_transaction_rejected_then_retries_with_fresh_base（stale reject 零污染四断言：树/历史深度/revision/事件 + 换新 base 重试恰好生效一次）；spike_semantics::spike_e_structural_stale_must_reject（reject 不改状态/不产历史/不产事件三断言）；t1_quality_fixes::probe_undo_failure_keeps_entry_stack_and_tree + ::probe_redo_failure（History 与调用前**逐字节一致** + 树 dump 一致 + revision 不动 + last_kind 拨回）；document_progressive::extend_rejects_incomplete_fence_without_pollution（四类悬挂 chunk 拒绝且树/revision/事件/历史全不变）。

**④ 日志自含可重放（重放 == 实时状态）**
- 代码：undo/redo 入日志（R11 修正）；genesis 物化 ID；replay 逐记录赋 revision。
- 测试【测试验证】：r12::ai_inclusive_log_replays_to_identical_state（**含 AI/undo/redo 记录**的日志重放 = 实时状态逐字节）；r11::move_and_undo（渐进 genesis + 全部结构事务重放复现）；r14_gates::gate1_core_ingest_and_replay_at_1mb（1.2MB ingest+replay 打点，断言重放=实时）；spike_d_history_json_roundtrip_and_log_replay（重放后 dump_tree+revision 全等）；document_progressive::genesis_carries_document_id_and_replay_restores（genesis document_id + 旧格式兼容 + 冲突拒绝）。

**⑤ ID 复原（§8.2：undo 不改写历史身份）**
- 代码：逆携带物化 ID（op.rs:358）；create_node(Some)→observe 抬升水位（tree.rs:178-184）——undo 复原后新分配不与历史撞车。
- 测试【测试验证】：spike_d_snapshot_midstream_and_allocator_watermark（restore 后新分配 ≥ 水位）；r17::cross_block_paste_undo_redo_identity / ::paste_table_and_fence_tail_new_ids_undo_identity（redo 复原粘贴身份=inverses 物化 ID）；r12::ai_entries_survive_save_open_and_undo_whole_batch（snapshot+history_json 往返后 undo/redo 链跨档可用，AI 整批单条 redo）。

## 4. 复用先决条件清单（P7 三分类直接输入）

**① op 可逆性要求**
- 全部 7 op 需 state-before 求逆；求逆协议由 apply_ops 编排（逐 op **先求逆后应用**，求逆必须基于该 op 之前的现场，lib.rs:269-283）——复用方新增 op 必须同时实现 apply_op 分支 + invert_op 分支，且 invert 在 apply 之前可达【代码确认】。
- no-op 语义必须实现**同向幂等逆**（R24-a2 先例：漏掉即破坏不变量①）【代码确认+测试验证】。
- 逆以 `Vec<Op>` 表达（RemoveBlock 逆=序列）——调用方按数组序应用，存储为逆序（lib.rs:284 注释）。

**② 合并窗参数化程度——关键疑点实测结论**
- `COALESCE_MS` 为 pub const 编译期常量（history.rs:20）；`should_coalesce` 直读 const（history.rs:96）。**History/Core 无任何运行时配置接口**（grep 实测：mdcore src 无 coalesce setter；wasm 桥无对应导出）【代码确认】。
- **settings.undo_coalesce_ms 未接线**【代码确认，本轮 grep 全仓证据】：shell settings.rs:102 默认取 `mdcore::history::COALESCE_MS`；:285-290 读 TOML；:361-365 越界降级（bounds (1,10_000)，settings.rs:37）；:446 TOML 写回；:579-583 set() 路径——**存储/持久化/校验/写回全链齐备，但全仓零消费者把它传入 mdcore**（mdcore src grep "coalesce" 仅 const 与内部逻辑；shell src grep "undo_coalesce" 仅 settings.rs 自身 6 处；web main.ts 仅类型声明+默认值+范围校验 ：1226/1240/1256，无运行时消费）。即：**该设置当前对引擎合并窗行为完全无效**（死设置）；CONTRACT.md:118 的对齐登记仅存在于默认值层面（r24_settings.rs:365 断言默认==COALESCE_MS，锁的是"默认值同源"而非"行为接线"）。P7 分类时此为"需要最小适配"的直接证据：参数化需在 History 增设配置入口。

**③ Source 枚举封闭性**
- 5 值封闭（View/Undo/Redo/Ai/System，history.rs:24-32），serde snake_case；扩展=破坏 serde 兼容（旧数据遇未知变体反序列化失败）+ wasm 字符串映射需同步（且现有 `_ => View` 静默兜底会吞掉新值，mdcore-wasm/src/lib.rs:44-49）【代码确认】。

**④ 事务大小假设**
- 无上限：apply_ops 不检查 ops 数量（lib.rs:253 签名直收 Vec<Op>）；r11 实测 100-op 事务=单 revision 单条目【测试验证】。
- 每事务 `backup = tree.clone()` 全树深拷贝（lib.rs:267）；undo/redo 再一次（apply_raw_batch :350）；RemoveBlock 求逆 collect_subtree 全子树拷贝（op.rs:250）。**潜在瓶颈**【推断：机制代码确认，量级无本轮基准数据——r14_gates 有 println 打点但无断言，数字未本轮采集，不引用】：大树上频繁小事务的成本随节点数线性增长；缓解既有项=OpText Arc 三方共享（R22-a，lib.rs:1130-1185 测试锁 log/事件/克隆共享分配）。
- Entry 双持 ops+inverses（history.rs:42-52；mem_stats 注释 :196-198 明示"Entry 双持"）——内存双份为既定形状【代码确认】。

**⑤ 附加先决（复用方须知，P7 四要素输入）**
- 时钟注入必须：wasm32 无 SystemTime（lib.rs:119-123 返回 0；宿主经 with_clock/set_clock 注入，wasm set_clock_ms）；合并窗与 at_ms 全部依赖此时钟【代码确认】。
- revision 是核内计数**非全局唯一**（open/replay 直接覆写；coalesce 不 bump——详见 p2 §6）：以 revision 做跨核/跨档乐观锁的复用方只能取"单核会话内单调"语义【代码确认】。
- Entry.selection 记录但 **undo 不恢复选区**（history.rs:47 写入 lib.rs:324；undo 路径零读取——选区恢复属 View 域）【代码确认】。
- 重放确定性要求日志物化 ID——依赖"入核事务全部经 apply_ops"纪律（p2 §2）【代码确认】。
- undo/redo 返回 `Option<Receipt>`（None=栈空或失败），**不区分两种 None**（失败与栈空对调用方同形；t1 测试仅以状态一致性锁语义）【代码确认 lib.rs:875-905】。
- **事件面是管线的唯一推送语义**：每事务一条 `Event{ops, meta:{source, coalesced, command}, revision}`（lib.rs:70-83），`take_events` 轮询取走（:237-239，drain 语义——不取即积压，无背压/无订阅）；复用方必须自带轮询泵（wasm 桥即此形状，P1 链④）。coalesced 事务的事件 revision 与组首相同（不 bump，p2 §6）【代码确认】。

## 5. 测试锁覆盖缺口清单（本轮通读发现，供 P7 验证标准项）

| # | 行为 | 代码级保证 | 专属测试锁 | 处置 |
|---|------|-----------|-----------|------|
| G1 | 新条目清空 redo 栈（push_entry） | history.rs:104 | **未见专属用例**（§2） | 登记为待补验证项 |
| G2 | undo 恢复选区 | **无此行为**（Entry.selection 记而不读，§4⑤） | 不适用（行为本身缺席） | 登记为设计事实 |
| G3 | wasm source 非法字符串兜底为 View | mdcore-wasm/src/lib.rs:49 | 未见（契约门只锁五合法值） | P5 复核 |
| G4 | save→open 后结构编辑场景 ID 漂移 / redo 物化 ID 撞号 | p2 §3 路 A 机制 | **未覆盖**（现有 save→open 测试均为文本编辑/追加场景） | P2 U1/U2，待补验证项 |
| G5 | undo 栈无深度上限/无逐出策略（History 无 cap） | history.rs:55-66（undo/redo 纯 Vec，无容量限制） | 未见（mem_stats 仅观测不限制） | 登记为复用前提（长会话内存形状，量级归 P6 待基准项） |
| G6 | 事件流中 coalesced 事务与组首同 revision（消费方按 revision 去重/排序的假设不成立） | lib.rs:306-318（事件 push 时 self.revision 未变） | 未见专属断言（r12 事件序断言未校验 revision 重复） | 登记为事件面消费契约注记 |

## 6. P3 验收对照

| PLAN P3 验收项 | 结果 |
|---|---|
| ① 管线签名/原子性/typed reject 清单 | ✅ §1（Reject 4 + AnchorError 4 + OpError 3 + TreeError 4 + SnapshotError 4 + SessionError 4 + DocumentIdError 全枚举+触发条件；T1-F10 全路径） |
| ② Undo/Redo 全链 + 粒度控制点 | ✅ §2（7 控制点 + redo 清空时机 + clear_interaction） |
| ③ 不变量清单双背书 | ✅ §3（五不变量逐条代码+测试锚；仅"性能量级"保持未知不冒充） |
| ④ 复用先决条件四项 | ✅ §4（四项全落级 + 附加先决 5 条；**undo_coalesce_ms 未接线**关键疑点已实测闭环） |

## 7. 遗留未知/待查项

| # | 项 | 级别 | 待查路径 |
|---|---|---|---|
| U1 | 合并窗/backup 的性能量级（大树频繁小事务的实际耗时分布） | 【未知】（机制【代码确认】，量级无基准） | 复用方按目标文档规模跑 r14_gates 同款打点（本审计零性能宣称）；P6 待基准项清单收录 |
| U2 | wasm source 静默兜底（`_ => View`）在真实前端的误用面 | 【推断】（机制代码确认，误用频率未量化） | 查 web/main.ts 调用点的 source 实参集合（P5 范畴）；复用方若需严格溯源需自行校验入参（登记，不展开方案） |
| U3 | Entry.selection 字段是否有历史消费者（序列化兼容面） | 【代码确认】（写 lib.rs:324，undo 零读取）；外部消费者未扫 | P5 pub 面扫描时复核（若零外部读者则为死字段，P7 登记） |
| U4 | r14_gates 打点的 MD 面耗时数字（gates.md 历史登记）本轮未运行采集 | 【文档记录：本轮未复验】 | 需要时跑 r14_gates 读 println 输出；本报告零性能宣称（与 p2 §8 U3 同源） |
