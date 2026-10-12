# P1 代码结构勘察（p1-structure.md）

> 状态：完成（2026-10-12 执行轮）。对应计划：PLAN.md v1.2 §4 P1（含 v1.2 新增
> ⑧模块依赖方向图）。证据基线：mdcore main=03815b6 本轮克隆亲读；全部锚点
> 为本轮实测。证据级别标注按 PLAN §2。

## 1. 模块地图（O1-O5 全覆盖）

### O1 mdcore（块核，crates/mdcore，13 模块 12,504 行含测试）

| 模块 | 行数 | 职责 | 关键符号【代码确认】 |
|------|------|------|---------------------|
| lib.rs | 1187 | Core 聚合 + apply_ops 管线 + §10.2 十原语 + sync/extend + undo/redo + snapshot/replay | `Core`（lib.rs:129，字段 tree/history/log/revision/document_id/selection/events/clock/dirty 全 pub 或私有混布）；`apply_ops`（:253）；`Reject`（:92，Stale/Anchor/Invalid/IncompleteChunk 四变体）；`Event`/`EventMeta`（:71-83）；`Receipt`（:86） |
| op.rs | 371 | 7 原子 op + apply_op + invert_op + mark 区间映射 | `Op`（:24，InsertBlock/RemoveBlock/ReplaceText/SetBlockType/SetAttrs/SetMark/MoveBlock，serde tag="t" snake_case）；`OpText = Arc<str>`（:20，R22-a 三方共享）；`invert_op`（:243）；`adjust_marks`（:196，四情形） |
| tree.rs | 388 | arena 槽位树 + 双轨寻址 + 结构原语 | `BlockTree`（:98，slots: Vec<Option<Node>> + id_index: BTreeMap<BlockId,NodeId> + pub alloc: IdAllocator）；`NodePath = Vec<usize>`（:62）；`BlockKind`（:21，8 成员）；`remove_subtree`（:269，槽位置 None 不压缩）；`move_subtree`（:290，环检查 + 摘除后语义） |
| id.rs | 70 | 稳定块身份 | `BlockId(pub u64)`（:12）；`IdAllocator`（:23，单调永不回收 + `observe` 水位抬升 :38） |
| history.rs | 301 | undo/redo 栈 + 合并窗 | `COALESCE_MS=1000`（:20）；`Source`（:24，View/Undo/Redo/Ai/System）；`Entry`（:42，ops+inverses 逆序对偶）；`should_coalesce`（:91，五条件合取）；`clear_interaction`（:180，ADR-B7 重载作废）；`BoundarySnapshot`（:84，T1-F10） |
| anchor.rs | 68 | (BlockId, offset) 寻址 | `Anchor`/`Selection`/`TextTarget`（ByPath 快路径 / ByAnchor+expect）；`AnchorError`（:48，Missing/Inverted/OutOfRange/ExpectMismatch 四类 typed reject） |
| revision.rs | 147 | op 流持久化 | `RevisionLog`（:30，append-only Vec<LogRecord>）；`LogRecord`（:17，revision/ops/source/command/document_id——document_id 仅 genesis 携带）；`to_jsonl/from_jsonl`（:60/69） |
| document.rs | 285 | 文件层：DocumentId + 开档/存档往返 | `DocumentId`（:41，UUIDv7 RFC 9562 零依赖，熵源 = AtomicU64+RandomState+栈 ASLR :25-37）；`save_document`（:220，md + meta JSON 含整栈历史）；`open_document`（:234，畸形 meta 显式 Reject） |
| external.rs | 209 | 外部重载：探针/决策/采纳 | `FileStamp`（:30，纯数据无 fs）；`plan_reload`（:61，Noop/SilentAdopt/OfferChoice 三态纯函数——v0 明确不做区域合并）；`adopt_external`（:101，take-external 身份保持全量重建） |
| fingerprint.rs | 880 | 块指纹 + 结构匹配 | FNV-1a 64 手写（:31）；`normalize_text`（:69，空白折叠）；`TreeFingerprint`（:95，下标 memo 无 HashMap）；`match_trees`（五类清单 preserved/moved/edited/added/removed，三级策略：manifest 快路径/结构锚走查/中段兜底） |
| snapshot.rs | 243 | 文档态快照 | `Snapshot`（:44，revision+next_id+递归 SnapNode，**含分配器水位**）；T1-O4 `validate`（:148，四不变量：DuplicateBlockId/NextIdNotAbove/MarkOutOfRange/ReservedKind，反序列化边界拒收） |
| markdown.rs | 317 | MD↔树 解析/序列化 | `markdown_to_tree`/`tree_to_markdown`（:32）；`is_complete_chunk`（:20，fence 开合扫描）；**marks 不写入 MD、ID 不写入 MD**（:8） |
| html.rs | 1682 | HTML 读侧（round-trip） | r13_html_roundtrip 19 测试背书（本轮 G1 通过） |
| debug.rs | 160 | 检视面 | `blocks_json`（:39，先序平铺含 depth）；`blocks_count`（:49）；`blocks_json_range`（:69，R16 分片）；`dump_tree`/`inspect` |

### O2 mdcore-wasm（CM6 桥，crates/mdcore-wasm，217 行）

单文件 lib.rs，`WasmCore` 23 导出（22 方法 + 构造器；P5 勘误后计数，P1 初稿误记 19）【代码确认，逐条在档】：
构造/身份：`new(markdown)`（:26）、`document_id`/`set_document_id`（:151/157）、`revision`（:210）
命令面：`apply_ops(ops_json, source, command)`（:36，source ∈ ai/undo/redo/system/view 五值映射 :44-49）、`replace_text_at_anchor`（:59，ByAnchor+expect）
事件面：`take_events`（:92，轮询取走 JSON 数组）
撤销面：`undo`/`redo`（:79/:85）
序列化面：`to_markdown`（:97）、`blocks_json`（:104）、`blocks_count`（:110）、`blocks_json_range`（:126）、`snapshot`（:190）、静态 `restore`（:195）、静态 `replay(jsonl)`（:204）、`history_json`/`set_history_json`（:180/184）
装载面：`sync_from_markdown`（:133，全量重同步兜底）、`extend_from_markdown`（:143，渐进打开）
杂项：`set_clock_ms`（:172，宿主时钟注入——wasm32 无 SystemTime）、`debug_tree`（:176）、`core_mem_stats`（:118，R20 内存审计）
协议纪律：所有入参/出参 JSON 字符串、u64→BigInt、拒绝走 `serialize_reject`（:215）。

### O3 mdshell（宿主契约层，crates/shell，4 文件 2,348 行）

| 文件 | 职责 | 关键符号 |
|------|------|----------|
| lib.rs | 文档生命周期 | `Host` trait（:36，surface_load_markdown 单方法）；`DocumentSession`（:43，core+id+path）；`open_document`（:69）/`open_document_progressive`（:86，Host 回调 OpenProgress 三态：FirstChunkInteractive/ChunkAppended/Completed）；`save_document`（:124） |
| session.rs | 多窗口核侧 | `SessionManager`（:68）；`SessionRecord`/`WindowRecord`（:212/220，schema v1） |
| settings.rs | 设置面 | `Settings` 四层（:54）；`SCHEMA_VERSION=1`（:32）；`bounds` 值域表（:35，undo_coalesce_ms (1,10000) 等）；`LoadStatus` 六态（:157） |

### O4 mdc-pdf（第二 Document Model，crates/pdf，15 模块 10,755 行）

模块清单（lib.rs:45-62）：annotation（标注模型四 kind + Origin Human/Ai + patch）/ cache（PageCache LRU 字节预算）/ command（六命令 + typed 错误七变体）/ document（PdfSession 单形态状态机）/ engine（**DocumentEngine trait**）/ engine_pdfium（feature 门，OnceLock 单例）/ outline / page（PageSpan 构造即校验 + /Rotate 归一化）/ protocol（mdcore://pdf + mdc-pdf-cmd/event）/ render（Rasterizer 抽象）/ security（S2 八族 + S3 限额 + S5 加密写拒 + S6 表单探针）/ security_scan（feature engine-lopdf）/ worker（渲染工人 + panic 隔离 + 看门狗换代）/ writer（AnnotationWriter trait + MockWriter + /T 编码 + /M 日期串）/ writer_lopdf（feature 门，增量写四 kind）。

**与 mdcore 关系【代码确认】**：crate 间**零 path 依赖**（pdf/Cargo.toml 无 mdcore 项）；Source 五值为语义镜像（command.rs:4-6"语义镜像而非类型复用"；serde 词面逐字一致，语义锁背书）。

### O5 治理面

CONTRACT.md v1.3（§1 核 compile 冻结面 / §2 宿主壳 / §3 web 桥 window 面 v1 / §4 设置面 v1.1 / §5 会话面 v1.1 / §6 PDF 面 v1.2→v1.3）；compliance/scan.sh（denylist 20 词，本轮 PASS 0 hits）；gates.md（102KB 全轮次登记）；contract_freeze.rs（4 测试 + 编译期常量锁）。

## 2. 依赖方向图（v1.2 P1⑧）

### 2.1 crate 级（无环验证【代码确认：Cargo.toml 实测】）

```
mdcore-wasm ──→ mdcore ←── mdshell（lib/session/settings）
                    ↑                    │
                    │（无）               ↓
                mdc-pdf（零依赖 mdcore——并列模型）
shell-native（蓝图，依赖全注释态，不参与编译面）
```

- mdcore-wasm → mdcore（唯一下游消费者之一）
- mdshell → mdcore（path 依赖，session/settings 均用 Core）
- **mdc-pdf → ∅**（对 mdcore 零 path 依赖——并列而非叠加）
- workspace 无环；serde/serde_json/ropey/toml_edit/pdfium-render/lopdf/wasm-bindgen 均叶子依赖
- **方向判词**：mdcore 是被依赖方（wasm 桥与 shell 消费它）；mdc-pdf 完全独立——"复用 mdcore 模块"的抽取面不涉 mdc-pdf 编译依赖

### 2.2 mdcore crate 内模块依赖（关键闭包，P2 依赖闭包的底座）

- op.rs → tree.rs（NodePath/BlockKind/Attrs/MarkSpan）+ id.rs（InsertBlock 物化）
- history.rs → op.rs（Entry 持 Op）+ anchor.rs（Selection）
- revision.rs → op.rs + history.rs（Source）+ document.rs（LogRecord.document_id）
- lib.rs（Core）→ 全部模块聚合
- external.rs → fingerprint.rs（match_trees）+ markdown.rs + op/tree/history
- snapshot.rs → tree.rs + id.rs（独立于 op/history/revision——快照面最小闭包 = tree+id）
- markdown.rs → tree.rs（仅此；解析/序列化不依赖 op/history）
- html.rs → tree.rs（读侧独立面）
- debug.rs → tree.rs + id.rs
- fingerprint.rs → tree/id/markdown **+ Core 反向依赖（fingerprint.rs:25 `use crate::Core`；:815 `impl Core` 段——match_external_markdown 为 Core 方法。P2 勘误补录）**；external.rs / document.rs 同样含 `impl Core` 段（external.rs:22 / document.rs:179）——抽取这三文件需连带 Core 聚合或剥离 impl 段
- document.rs → markdown.rs（save_document 用 tree_to_markdown）

**最小闭包观察**【代码确认】：①op 求逆/应用闭包 = tree+id（op.rs:15 仅 use tree/id）；②history 闭包 = op+anchor（history.rs:17-18）；③snapshot 闭包 = tree+id（snapshot.rs:19-20）；④markdown 闭包 = tree（markdown.rs:11）。核心五件（tree/id/op/history/anchor）不依赖 serde 以外的任何模块——**serde 仅在序列化边界**（lib.rs:27-28 注释实证"tree/op 本身可脱离 serde 编译"）。

## 3. 四条关键调用链（全部【代码确认】）

### 链 ① 编辑命令 → 事务 → op → 树 → 历史

`CM6 updateListener`（main.ts:650）→ JS 构造 op JSON → `WasmCore.apply_ops`（mdcore-wasm/src/lib.rs:36）→ source 字符串映射（:44-49）→ `Core.apply_ops`（lib.rs:253）：
1. 空 ops 早退（:254）；kind 判定 Text/Structural（:259）
2. `backup = tree.clone()`（:267）
3. 逐 op `invert_op(tree, op)` + `apply_op(tree, op)`（:269-283，任一失败恢复 backup 整体回滚）
4. System 源 → 边界推进不入栈（:286-301，ADR-B4）；否则 `should_coalesce` 五条件（history.rs:91）→ coalesce_into_last（ops 追加/inverses 前插）或 push_entry（清 redo）
5. `bump_revision` + `RevisionLog.record` + `push_event`（:329-339）
6. 交互事务置 dirty（:342）
结构命令（insert_block 等十原语，:387-618）= `gate(base_revision)` 硬门禁（:378，不等即 Stale reject 不 rebase）→ 内部转 apply_ops。文本命令（replace_text/set_mark，:564-597）无 base 门禁，靠锚点/expect 保护。

### 链 ② 加载 → adopt_external → 指纹 → 会话

宿主 FileStamp 比对（external.rs:30-41）→ `match_external_markdown` 预览（fingerprint match_trees）→ `plan_reload`（:61，纯函数三态）→ 用户 take-external → `Core.adopt_external`（:101）：
1. 解析新 md → `match_trees`（五类清单 + carry 映射 :121-135）
2. op 合成 = RemoveBlock[0]×n + 先序重建 InsertBlock（携带旧 id/manifest 可信 id/现场分配，:139-150）
3. `apply_ops(Source::System, "external_reload")`（:152-158）
4. `history.clear_interaction()`（:160，ADR-B7 交互史作废）+ `mark_saved()`（:163，脏清位）
Noop 路径零事务零 revision（:109-117）。会话层：mdshell `open_document`（lib.rs:69）→ `SessionManager.open`（session.rs:149，三重硬门）。

### 链 ③ 设置变更 → sidecar → 事件广播

window 面 `__setSetting`（CONTRACT §4）→ mdshell::settings `set()`（写侧规范化 ensure_section，R24-a1 修复 :22-25）→ TOML sidecar 原子写（tmp+rename）→ `mdc-settings-changed` 事件族（CONTRACT §4，contract_freeze.rs:73 词面锁）。Rust 侧无事件总线——设置事件在 window 面（web 层），核侧仅状态机 + LoadStatus。

### 链 ④ wasm 桥 → 核 → 回推

`take_events` 轮询驱动（main.ts:781 `JSON.parse(core.take_events())`）：核事件（Event{ops,meta,revision}）→ JS BlockMap 增量建图（blocks_json_range :881-892 渐进专用）→ CM6 `view.dispatch`（:950/1058）。快路径：击键 ReplaceText 直发 `apply_ops`（:846）；兜底：`sync_from_markdown`（:865，核内按位置朴素差分→最小 op 序列→同一 apply_ops 管道，command="sync_from_markdown"）。时钟：`set_clock_ms`（JS Date.now 注入，mdcore-wasm:172）。

## 4. DocumentEngine trait 完整形状（O4 复用先例核心事实）

【代码确认 engine.rs:17-37】：

```rust
pub trait DocumentEngine {          // object-safe（&dyn 可命名，S4 前瞻）
    fn page_count(&self) -> u32;    // 引擎侧权威；attach 时对账 meta.page_count
    fn page_geometry(&self, idx: PageIndex) -> Result<PageGeometry, CommandError>;
    fn outline(&self) -> Result<OutlineTree, CommandError>;   // 无大纲 → 空树非 Err
    fn extract_text(&self, span: PageSpan) -> Result<Vec<(PageIndex, String)>, CommandError>;
}
```

- **只读数据面**：标注写入不在本 trait（写路径 = AnnotationWriter，文档级会话面 writer.rs:39-48，`write_annotation` 幂等合同 + `save_incremental` 前缀逐位保持）
- 生命周期假设：Box 所有权传递（`attach_engine`/`detach_engine`，document.rs:233-252；detach 供渲染工人接管 Rasterizer）
- 装配期校验：页数双权威对账（AttachEngineError::PageCountMismatch，T1-O3 收口 :81-95）
- **集成分层**【代码确认 document.rs:98-122】：PdfSession 持 `engine: Option<Box<dyn DocumentEngine>>` + `annot_writer: Option<Box<dyn AnnotationWriter>>`——引擎与写器**独立装配**（阅读与落盘互不为前提）

## 5. 渲染路径定位（P1④）

**核侧零渲染器确认**【代码确认】：mdcore/mdshell/mdcore-wasm 三 crate 无任何渲染依赖（Cargo.toml 无 GUI/绘图项；lib.rs 模块清单无 render 模块）——渲染全在 CM6/web 层（main.ts EditorView + Decoration）与 mdc-pdf worker（PDF 面独立渲染线，与 MD 面无共享）。**若复用 mdcore 核心五件套，零渲染耦合**。

## 6. 复用视角初步观察（供 P2-P6 展开，非结论）

1. 核心五件（tree/id/op/history/anchor）依赖闭包极小（§2.2）且 serde 仅在边界——**抽取面干净**。
2. wasm 桥 23 导出 = 现成"命令调用 + 变更订阅（轮询）+ 序列化访问"协议面；**按 ID 取块/块级订阅无直接 API**（blocks_json 是全量/分片 dump，无 get_block(id)；事件流是 op 粒度非块粒度订阅）——P5 四态表已正式登记（p5-api-surface.md §6）。
3. mdc-pdf 与 mdcore 零编译依赖——任何 mdcore 模块抽取**不直接波及 mdc-pdf 编译面**（P6 波及面分析的事实基础）。
4. 全局状态仅 3 处（ENTROPY_SEQ 原子 / PDFIUM+BIND_LOCK OnceLock）——P6 将全量登记。
5. Core 字段 `pub tree / pub history / pub log`（lib.rs:130-132）——公开可变字段（非封装 getter），消费方可直访树/历史/日志。

## 7. P1 验收对照

| PLAN P1 验收项 | 结果 |
|----------------|------|
| ① 入口定位（三 crate 公开面） | ✅ §1 O1/O2/O3（wasm 23 导出逐条，勘误口径同 §1 O2） |
| ② 核心状态模型四态定位 | ✅ 块树（tree.rs）/ 文档会话（document.rs+shell DocumentSession）/ 设置（settings.rs）/ 会话（session.rs） |
| ③ 编辑器定位（命令入口） | ✅ wasm 桥命令面 + Core 十原语（§3 链①） |
| ④ 渲染路径定位 | ✅ §5 核侧零渲染器（重大"干净"确认，非异常发现） |
| ⑤ 序列化与持久化模块定位 | ✅ markdown.rs/snapshot.rs/revision.rs/history to_json/document save_meta/settings TOML/session record |
| ⑥ 四条调用链【代码确认】 | ✅ §3 全部主路径锚点在档 |
| ⑦ DocumentEngine 完整形状 | ✅ §4 四方法 + object-safe + 生命周期 + 独立装配 |
| ⑧ 模块依赖方向图（v1.2） | ✅ §2 crate 级无环 + 模块级闭包 |
