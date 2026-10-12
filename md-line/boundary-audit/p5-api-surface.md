# P5 扩展接口审计（p5-api-surface.md）

> 状态：完成（2026-10-12 执行轮，Task t7-p5-survey）。对应计划：PLAN.md v1.2
> §4 P5。证据基线：mdcore main=03815b6 本轮克隆亲读（工作区零改动复核在档）；
> 全部锚点为本轮实测。证据级别标注按 PLAN §2。消费口径：**外部消费者 =
> 未来任何想复用 MD 引擎能力的宿主代码**（不预设具体形态）；对照面 =
> mdc-pdf 六命令边界与设置面（已实现扩展接口先例，仅形状对照）。

## 1. ① 公开 API 清单（消费视角分组）

### 1.1 mdcore 核（crates/mdcore，全部【代码确认】）

| 消费场景 | API | 锚点 |
|---|---|---|
| 构造/复原 | `Core::new / with_clock / from_markdown / from_markdown_with_clock`；`Core::restore(Snapshot)`；`Core::replay_jsonl(&str)` | lib.rs:149/153/167/171；lib.rs:955；lib.rs:966 |
| 命令（十原语+） | `insert_block / delete_block / set_block_type / set_attrs / move_block / split_block / merge_block`（结构类，base_revision 硬门禁 :378）；`replace_text / set_mark`（文本类，无 base 门禁）；`commit_source` | lib.rs:387/410/418/431/444/464/528；lib.rs:564/569；lib.rs:622 |
| 事务 | `apply_ops(Vec<Op>, ApplyMeta)`（唯一变异入口）；`commit_transaction / commit_transaction_as(base, ops, meta)`（gate + 带源提交） | lib.rs:253；lib.rs:600/610 |
| 历史 | `undo / redo`；`history_json / set_history_json`；`History` 栈面（pop_undo/pop_redo/undo_depth…全 pub） | lib.rs:875/907；lib.rs:939/934；history.rs:136-246 |
| 事件 | `take_events() -> Vec<Event>`（drain 语义）；`set_selection / selection` | lib.rs:237-239；lib.rs:229/233 |
| 检视 | `revision`；`mem_stats`；`is_dirty / mark_saved`；`now_ms` | lib.rs:194/201/216/221/225 |
| 序列化 | `markdown::tree_to_markdown / markdown_to_tree`；`debug::blocks_json / blocks_json_range / blocks_count / inspect / dump_tree`；`html::html_to_tree / import_html / tree_to_html`；`Snapshot::capture/to_json/from_json/validate`；`RevisionLog::to_jsonl / from_jsonl` | markdown.rs:32/103；debug.rs:39/69/49/146/114；html.rs:108/113/132；snapshot.rs:131/174/178/148；revision.rs:60/69 |
| 装载 | `sync_from_markdown`（全量差分兜底）；`extend_from_markdown`（渐进追加）；`document::save_document / open_document`（md+meta 双产物）；外部重载三件 `match_external_markdown / plan_reload / adopt_external` | lib.rs:765/849；document.rs:220/234；fingerprint.rs:834；external.rs:61/101 |
| 身份 | `document_id / set_document_id`；`BlockId(pub u64)`；`DocumentId::generate` | document.rs:183/192；id.rs:12；document.rs:47 |
| 调试/内存 | `debug_tree`（经 wasm）；`BlockTree::mem_stats / TreeMemStats` | debug.rs:114；tree.rs:345 |
| 树直访 | `Core` 字段 **pub tree / pub history / pub log**（公开可变字段，消费方可直访树与历史——绕过 apply_ops 管线的旁路在类型层敞开，纪律仅靠约定）；`BlockTree::lookup_id / node / node_mut / resolve / path_of / collect_subtree` | lib.rs:130-132；tree.rs:150/130/134/202/211/252 |

注：`Core` 无 Clone/Debug derive（lib.rs:129，B13 口径一致）；`Event/EventMeta/Reject` 仅 derive Serialize（lib.rs:70/78/91）——出核单向。

### 1.2 mdcore-wasm（crates/mdcore-wasm，单文件 217 行）

**WasmCore 实际导出 23 个**（1 构造器 + 20 实例方法 + 2 静态；`pub fn` 全量计数【代码确认】，P1"19 导出"为勘误，见 §8）：

| 组 | 导出 | 锚点 |
|---|---|---|
| 构造/复原 | `new(Option<md>)`；静态 `restore(snapshot_json)`；静态 `replay(jsonl)` | lib.rs:26/195/204 |
| 命令 | `apply_ops(ops_json, source, command)`（source 五值映射 ai/undo/redo/system/else→view）；`replace_text_at_anchor(block_id,start,end,insert,expect)`；`undo`；`redo` | lib.rs:36/44-49/59/79/85 |
| 事件 | `take_events() -> String`（JSON 数组，取走清空） | lib.rs:92 |
| 检视/序列化 | `to_markdown`；`blocks_json`；`blocks_count`；`blocks_json_range(from,count)`；`snapshot`；`history_json / set_history_json` | lib.rs:97/104/110/126/190/180/184 |
| 装载 | `sync_from_markdown(md)`；`extend_from_markdown(chunk)` | lib.rs:133/143 |
| 身份 | `document_id`；`set_document_id`；`revision` | lib.rs:151/157/210 |
| 时钟/调试/内存 | `set_clock_ms`（宿主时钟注入）；`debug_tree`；`core_mem_stats` | lib.rs:172/176/118 |

协议纪律：入参/出参全 JSON 字符串、u64→BigInt、拒绝走 `serialize_reject`（lib.rs:215）。

### 1.3 mdshell（crates/shell）

文档生命周期：`Host` trait（单方法 surface_load_markdown，lib.rs:36）；`DocumentSession{core,id,path}`（lib.rs:43）；`open_document / open_document_progressive / save_document`（lib.rs:69/86/124；OpenProgress 三态回调 lib.rs:23）。多窗会话：`SessionManager`（session.rs:68，open/close/focus/get/get_mut/dirty_windows/snapshot_record 等 14 方法）。设置：`SettingsStore`（settings.rs:476，load/save/set + 六态 LoadStatus settings.rs:157）。

### 1.4 crate 依赖方向（对照 P1 §2 复核，一致）

【代码确认：Cargo.toml 实测】mdcore-wasm → mdcore（mdcore-wasm/Cargo.toml:11）；mdshell → mdcore（shell/Cargo.toml:8）；**mdc-pdf → ∅**（pdf/Cargo.toml 无 mdcore path 依赖）；shell-native 为 workspace/default 成员（根 Cargo.toml:3-4）但依赖全注释态（shell-native/Cargo.toml:7-13，零依赖蓝图随门禁编译）。**方向无环**；mdcore 是唯一被依赖方。P1 §2.1 判词复核成立。

## 2. ② 变更订阅能力

**MD 文档面事件的确切形状**【代码确认】：
- 事件类型：`Event { ops: Vec<Op>, meta: EventMeta, revision: u64 }`；`EventMeta { source, coalesced, command }`（lib.rs:70-83）。事务管线内逐事务 push_event（lib.rs:365-371/292-339），undo/redo 亦发事件（lib.rs:895/922）。
- **拉取式**：唯一消费口 = `take_events()`（lib.rs:237-239，`self.events.drain(..)`）——取走**全部**并清空队列；事件不复制、无回放。积压量仅可观测（`mem_stats.events_pending`，lib.rs:206），无上限告警。
- **单消费者**：`&mut self` 独占调用；无订阅者注册、无回调、无监听器（全仓 grep subscribe/listener/on_event/watch 零命中，crates/ 范围【代码确认】）。
- wasm 面：`take_events` 出 JSON 字符串（mdcore-wasm:92）；web 侧由桥在每次变异后 drain 进 `window.__mdcoreEvents` 数组（main.ts:769-787 mutate/drainEvents）——**被动累积数组，宿主轮读**，同样无推送。

**对照面（已实现扩展接口先例，形状对照）**：
- 设置面：`mdc-settings-changed` = window CustomEvent（main.ts:1440）——**推送式、多订阅者**（addEventListener），载荷 {path, value}；读面 `__settings` 深拷贝快照（main.ts:1433）。
- PDF 面：`mdc-pdf-cmd`（命令信封 CommandEnvelope{id, command}）/`mdc-pdf-event`（回执 + 自发事件）事件族（protocol.rs:28-42）——**协议层事件族（草案）**，回执按信封 id 关联请求。

**关键缺口登记（PLAN P5② 预设判定项）**：MD 文档面**无推送式/多订阅者事件族**——文档变更只能拉（take_events / __mdcoreEvents 轮读），与设置面（推送多订阅）和 PDF 面（cmd/event 事件族）形成鲜明形状差。第二消费者（如未来并行视图）在现接口下会与 CM6 桥**争抢同一 drain 队列**（先取先得、取走即空）。此为 P7 分类的直接输入。

## 3. ③ 命令调用入口

### 3.1 wasm 桥协议族全量
见 §1.2（23 导出分组）。命令面实际形状 = **op JSON 批流**（apply_ops）而非具名命令枚举；仅 `replace_text_at_anchor / undo / redo` 三个具名快捷口。

### 3.2 window 面（CONTRACT §3）与 main.ts 实际对齐
CONTRACT.md:54-93 登记三块：启动面（:56-62，13 成员）；宿主注入面（:64-78，__inject/__inject_chunk → mdc-inject* 事件族）；__bridge 面（:80-88，10 成员）。main.ts 实装：启动面三路径安装（main.ts:1320-1328 渐进 / 1352-1360 一次性 / 1394-1408 重建）；注入面 main.ts:1410-1431；__bridge = CM6View 实例（main.ts:1322 等），声明面收窄为 10 成员（main.ts:1172-1190）。逐条对齐结果见 §7。

### 3.3 Rust 侧直调路径（不经 wasm）
Core 十原语 + apply_ops/commit_transaction_as + undo/redo + 装载/序列化全家（§1.1），全部 pub、可直接调用【代码确认】。examples/inspector.rs 即现成 Rust 消费样例（击键合并窗 → AI 命令 → stale 拒绝 → undo/redo → replay/snapshot，inspector.rs:17-115）。

### 3.4 __contractVersion 契约门成员清单
web/tests/contract_gate.py（Playwright 活页断言，**锁 8 组**）：
1. `__contractVersion === 1.1`（:58）；
2. 启动面类型 9 项：__inject/__inject_chunk/__md/__blocks/__blockCount/__memStats = function、__mdcoreEvents = Array、__perf/__core = object（:59-72；§7 全口径 11 项含 __ready/__contractVersion——复审修正计数）——**不含 __view/__wasmMemory/__heapTrace**；
3. __bridge 清单 10 成员（pushRender/undo/redo/computeDecorations/dumpDecorations/setSelections/debugMapVsFullRebuild/unit.utf16ToCp/unit.cpToUtf16 = function + locked = boolean，:75-88）；
4. 注入面：重建 + mdc-inject 计数 + 事件流清零/排空时序（:96-110）；
5. 渐进通道：chunk 追加 + mdc-inject-chunk 计数（:113-119）；
6. undo 键位契约恒 true（:122，R21 勘误口径）；
7. 单位宪法换算（:126-130）；
8. 设置面：读快照 + 写事件 + 五类拒绝（未知路径/保留键/类型/越界/非整数）不派事件（:132-166）。

crates/shell/tests/contract_freeze.rs（Rust 侧词面锁 4 测）：shell-native 蓝图 mdc-inject* 词面 + 禁 md-inject* 旧名（:19-36）；CONTRACT.md v1 关键词存在性（:40-62）；v1.1 设置面词面 + SCHEMA_VERSION==1 编译期常量（:67-84）；v1.1 会话面词面 + DEFAULT_MAX_SESSIONS==16 / SESSION_RECORD_VERSION==1 编译期常量（:90-110）。

## 4. ④ 按 ID 获取块

**判定：需适配（组合可达，无单步 API）**【代码确认】：
- **无 `get_block(BlockId)`**：全仓（crates/）grep get_block/block_by_id/by_id/find_block 零命中。
- Rust 组合路径（两步）：`tree.lookup_id(BlockId) -> Option<NodeId>`（tree.rs:150，BTreeMap 注册表索引）→ `tree.node(NodeId) -> &Node{id, kind, text: Rope, attrs, marks, parent, children}`（tree.rs:130）。全 pub，消费方薄封装一步即得结构化块数据。辅助：`path_of`（ID→位置，tree.rs:211）、`collect_subtree`（子树快照，tree.rs:252）。
- 人读单块面：`debug::inspect(tree, BlockId) -> Option<String>`（debug.rs:146，格式化字符串非结构化数据）。
- **wasm/JS 面：无按 ID 出口**。唯一携带 id 的批量面 = `blocks_json / blocks_json_range`（debug.rs:39/69），BlockJson 形状 `{id, type, depth, attrs, text}`（debug.rs:16-23，**不含 marks**）——先序平铺，消费方须全量/分片 dump 后自行按 id 过滤或维护 byId 索引。
- 核内 anchor 用法佐证：`resolve_anchor` = lookup_id → path_of → 越界检查（lib.rs:672-689）——核自己也是组合路径消费者，无更快通道。

## 5. ⑤ 块级变化监听

**事件粒度 = op 级（事务批）**，非块级、非纯文档级：每个 Event 携带一个事务的全部 op + revision（lib.rs:77-83）。**无单块订阅能力**（无过滤参数、无订阅面）。

**关键细节核实**【代码确认】——变更通知定位块的信息量：
- `InsertBlock` 物化后带 id：apply_op 回写 `*id = Some(block_id.0)`（op.rs:115-119），事件流中的 insert_block 携带具体新块 ID；CM6 桥正是靠它拿新块 ID 定位光标（main.ts:1036-1042：`op.t === 'insert_block' && typeof op.id === 'number'`）。
- **`ReplaceText` 只带 path**（op.rs:40-45：path/start/end/insert，无 BlockId）；`RemoveBlock/SetBlockType/SetAttrs/SetMark/MoveBlock` 同样只带位置（op.rs:37-65）。**path 是位置不是 ID**——任务提示核实成立。
- 后果：消费者从事件流定位"哪个块被改"必须做 **path→id 换算**：Rust 侧可即时 `tree.resolve(path)` 反查 Node.id（tree.rs:202）；JS 侧无 resolve 出口，须依赖 blocks_json 重建的索引。CM6 先例 = **BlockMap**：rebuild(docText, blocks_json) 全量重建 lines + byId Map（main.ts:158-271，byId.set :200/:307）+ 渐进 extend（main.ts:288 起）；op 构造侧同样吃 BlockMap 的 path（main.ts:844）。
- 事件源可区分人机：EventMeta.source ∈ view/undo/redo/ai/system（history.rs:24 serde snake_case；wasm 映射 mdcore-wasm:44-49）。

## 6. ⑥ 外部消费者五能力 × 四态判定表（P7 中间产物）

| 能力 | 判定 | 主锚点 | 适配内容（若"需适配"） |
|---|---|---|---|
| 命令调用 | **可直接调用** | lib.rs:253-618；mdcore-wasm:36-85 | — |
| 变更订阅 | **需适配**（推送式/多订阅者子面 = **缺失**） | lib.rs:237-239；mdcore-wasm:92；main.ts:779-787 | 消费方自建轮询泵（wasm 时钟下宿主需主动调 take_events）；多订阅者需自建分发层，且现单队列语义下不得与既有消费者（CM6 桥）共存争抢 |
| 按 ID 取块 | **需适配** | tree.rs:150+130；debug.rs:39/69 | Rust：lookup_id→node 两步组合或薄封装；JS：blocks_json(_range) dump 后按 id 过滤/自维护 byId 索引（且 dump 面不含 marks） |
| 块级监听 | **需适配**（单块订阅子面 = **缺失**） | lib.rs:77-83；op.rs:40-45/119；main.ts:1036-1042 | 消费方对 op 流做 path→id 换算（Rust 用 tree.resolve；JS 须自建索引）+ 按块过滤；InsertBlock 类可直接读 id，文本类 op 需换算 |
| 序列化访问 | **可直接调用** | markdown.rs:32；debug.rs:39；snapshot.rs:44；revision.rs:60；mdcore-wasm:97-204 | —（注：marks/ID 不写入 MD——markdown.rs:8，持久化走 meta/snapshot/history 面） |

**逐行论证**：

1. **命令调用 = 可直接调用**。Rust 直调面全量 pub（十原语 + apply_ops + 带源事务 + undo/redo，§1.1/§3.3），wasm 面 apply_ops + replace_text_at_anchor + undo/redo（§1.2）；命令拒绝 typed（Reject 四变体 lib.rs:92-101，AnchorError 四类 anchor.rs:48-67）。两种宿主形态（进程内 / WebView）均无需适配层。唯一形态注意：主命令口是 **op 流**而非具名命令枚举（对照 mdc-pdf 的 Command enum + PdfSession::execute 单一调度，pdf/command.rs:94-121 + pdf/document.rs:172——先例对照，非照抄依据）。
2. **变更订阅 = 需适配 + 关键缺口**。拉取式单消费者面存在且语义清晰（take_events drain，§2）；但推送式/多订阅者事件族**缺失**——与设置面（mdc-settings-changed 推送多订阅）和 PDF 面（mdc-pdf-cmd/event 信封族）的既有先例相比是形状空洞。任何第二消费者需要适配层（轮询泵 + 分发），且**单 drain 队列的争抢语义**是复用时必须处理的约束。
3. **按 ID 取块 = 需适配**。能力存在（lookup_id 注册表 + Node 全字段 pub），但无一步 API：Rust 两步组合、JS 无出口只能 dump 过滤。适配内容薄（Rust 薄封装即可；JS 需索引维护策略）。
4. **块级监听 = 需适配 + 子面缺失**。op 级流 + revision 序号可用（事件含 source/command/coalesced，可审计）；但（a）无单块订阅；（b）除 InsertBlock（物化 id，op.rs:119）外全部 op 只带位置 path——块级定位依赖消费方换算基础设施（CM6 BlockMap 即该基础设施的实现，main.ts:120-384：类型 + 类）。事件信息量本身够用（有 path + 树可查），缺的是 ID 直达与订阅过滤。
5. **序列化访问 = 可直接调用**。五面俱全：MD 全文（tree_to_markdown）、结构化块平铺（blocks_json/分片）、快照（Snapshot to/from_json + validate）、op 日志（RevisionLog jsonl）、历史栈（history_json）；wasm 面一一对应出口；window 面 __md/__blocks/__blockCount 兜底（main.ts:1323-1326）。往返语义边界在档：marks/ID 不入 MD（markdown.rs:8），ID 走 meta/snapshot/log 面。

## 7. 契约三面对照结果（CONTRACT.md ↔ main.ts ↔ contract_gate.py）

**对齐面（无漂移）**【代码确认】：启动面 11 项（__ready/__contractVersion=1.1/__inject/__inject_chunk/__md/__blocks/__blockCount/__memStats/__mdcoreEvents/__perf/__core）；__bridge 10 成员逐条；注入事件族（mdc-inject/mdc-inject-chunk 派发与计数）；undo 键位契约恒 true（CONTRACT.md:83-85 ↔ main.ts:967-971 ↔ gate:122）；单位宪法出口；设置面全矩阵（读快照/写事件/五类拒绝不派事件，CONTRACT §4:97-111 ↔ main.ts:1433-1442 ↔ gate:132-166）；shell-native 蓝图词面与 CONTRACT 词面锁（contract_freeze.rs:19-110）。**核心协议面三面一致。**

**漂移/瑕疵发现（4 项）**：
- **F-1【代码确认】`__wasmMemory`/`__heapTrace` 非恒在**：CONTRACT.md:56-62 将其列入「启动面（boot 后恒在）」（:61-62）；main.ts 仅在渐进路径安装（main.ts:1297/1314，`nChunks > 1` 分支内）；默认一次性启动（main.ts:1346-1377）与 __inject 重建（installNewCore，main.ts:1388-1409）**均不安装** → 默认 boot 后 undefined。contract_gate.py 启动面断言不含此二成员（gate:59-72）——**门禁不可见的文档-实现漂移**（文档宽于实现）。
- **F-2【代码确认】§3 文本版本号未随 v1.1 改写**：CONTRACT.md:62 启动面仍写 `__contractVersion: 1`（v1 原文），运行时与门禁均为 1.1（main.ts:1443；gate:58）；§4:110 与变更记录 :142 已载 1→1.1。文档内部前后不一致（§3 原文未同步），语义以 §4 为准——文档级瑕疵，非代码漂移。
- **F-3【代码确认】`__view` 锁面不完整**：CONTRACT.md:58 列为启动面、main.ts 三路径均安装（:1321/:1353/:1395），但 gate 启动面类型断言未含 __view（仅 :106 间接使用）——弱漂移（实现一致，锁不全）。
- **F-4【代码确认】__bridge 运行时面宽于声明面**：运行时对象 = CM6View 类实例，携带 TS 私有方法（onDocChanged/handleEnter/drainEvents 等，main.ts:650-1063）；契约声明面收窄为 10 成员（main.ts:1172-1190），gate 只断言"10 成员在位"不锁"无多余"。低风险登记。

## 8. P1 勘误与复核

- **E-1（数字勘误）**：P1 §1 O2 与 §6.2"**WasmCore 19 导出**"→ 实际 **23**（1 构造器 + 20 实例 + 2 静态；`pub fn` 全量计数 23，mdcore-wasm/src/lib.rs:26-210）。P1 自身分组枚举恰为 23 项——计数与枚举自相矛盾，本轮以枚举+实测为准。
- **E-2（锚点勘误）**：P1 O2"undo/redo（:79/83）"→ 实际 **:79/:85**（:83 为 undo 收尾行）。
- **复核一致项**：P1 §6.2 三项初步观察本轮独立核验**全部成立**（19 导出数字除外）：①按 ID 取块无直接 API（§4）；②事件流 op 粒度非块粒度（§5）；补充细节——InsertBlock 物化后事件携带 id、ReplaceText 只带 path（op.rs:119/40-45）。③P1 §2.1 依赖方向/无环/pdf 零依赖（§1.4 复核一致）；④P1 §6.5 Core pub tree/history/log（lib.rs:130-132 一致）。

## 9. P5 验收对照（PLAN §4 P5）

| 验收项 | 结果 |
|---|---|
| ① pub 面按消费场景分组 + 依赖方向无环复核 | ✅ §1（mdcore 11 组 / wasm 23 导出 7 组 / mdshell 三面；Cargo.toml 实测无环） |
| ② 变更订阅能力（对照设置面/PDF 面；缺口登记） | ✅ §2（拉取单消费者 + 推送/多订阅缺失 = 关键缺口显式登记） |
| ③ 命令调用入口（wasm 全量 + window 面对齐 + Rust 直调 + 契约门清单） | ✅ §1.2/§3（contract_gate.py 8 组断言 + contract_freeze.rs 4 测逐条在档） |
| ④ 按 ID 获取块 | ✅ §4（需适配：组合可达/无单步 API） |
| ⑤ 块级监听 | ✅ §5（op 粒度 + path≠ID 关键细节核实 + CM6 BlockMap 先例） |
| ⑥ 五能力 × 四态判定表全格锚点 | ✅ §6（2 可直接 / 3 需适配，子面缺失 2 处标注；每行论证段） |
| wasm 协议族与 CONTRACT.md 逐条对齐（漂移即发现） | ✅ §7（对齐清单 + F-1..F-4 四项发现） |
