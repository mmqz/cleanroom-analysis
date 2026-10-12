# P4 协同现状与约束登记（p4-collab-status.md）

> 状态：完成（2026-10-12 执行轮）。对应计划：PLAN.md v1.2 §4 P4。
> 执行者：勘察代理 t7-p4-survey。证据基线：mdcore main=03815b6 本轮克隆亲读
> （strict 只读，git status 恒空）。证据级别按 PLAN §2 五级；锚点 `路径:行号`。
> 定位：**P7 的附属登记输入**（v1.2 已降级协同为附属项）。
> **零选型纪律**：本文只登记现状与约束事实；下文出现的 Yjs/yrs/OT/CRDT 等词
> 仅作为**搜索关键词记录**或**用户裁决回放**，不含任何选型、比较或推荐表述。

## 1. B12 核查（独立复核，第一优先）

### 1.1 sync_resync 的真实性质

【代码确认】`crates/mdcore/tests/sync_resync.rs:1-285` = **R9 轮 sync_from_markdown
（CM6 桥全量重同步兜底）的本地语义测试**，共 10 用例：revision 前进
（sync_advances_revision:45）、undo 撤 sync（undo_reverts_sync:59）、diff 最小性
（diff_minimality_inplace_text_only:86）、kind/attrs 原位改（:107）、尾部增删
（:133）、嵌套子块 diff（:168）、**管道一致性**（sync_goes_through_pipeline:194
——source=View、command="sync_from_markdown"、与后续击键同 1s 窗合并、一次 undo
同时撤掉打字+sync）、commit_source 重建 ID 冲突回归（:232）、fence 吞行（:261）、
空文档边界（:274）。

它测的对象 `Core::sync_from_markdown`（lib.rs:765-781）= 把 CM6 当前 doc 文本交核，
核内做**按位置朴素对照差分**（diff_forest lib.rs:784-829，"允许非最优，不追求树
编辑距离" lib.rs:756）→ 最小 op 序列 → 既有 apply_ops 管线（Source::View）。
wasm 侧消费点 main.ts:815/862-872（快路径失败后的 fullResync 兜底）。

**独立结论**：主代理初判**成立**——sync_resync 是**单进程内 view↔核的文档级对账
机制**（CM6 View 因 IME/粘贴/多 change 无法走快路径时，把整篇文本回灌核内差分），
零网络、零多端、零并发语义。文件名中的 "sync" 指本地重同步，非协同同步。

### 1.2 revision 体系的真实性质

【代码确认】revision = Core 上的**本地文档级 u64 计数器**（lib.rs:133，私有字段，
bump_revision lib.rs:360-363 仅 `+=1`）+ append-only JSONL 日志（revision.rs:30
`RevisionLog{records: Vec<LogRecord>}`；LogRecord=revision/ops/source/command/
document_id :17-27）。**无分布式版本语义**：无 lamport/vector clock（grep 零命中，
见 §1.3）、无 peer/站点标识、无并发分支。详见 §4 约束事实登记。

### 1.3 全仓搜索记录（本轮独立重验，mdcore 五 crate + web/ + 全部 md）

| 关键词（rg -i） | 命中数 | 命中性质 |
|---|---|---|
| crdt / yjs / yrs / websocket / lamport / vector clock / operational transform | **1** | 全部为 `web/package-lock.json:128` sha512 完整性哈希串内 "Yrs" 偶然子串——**假阳性**；代码/文档 0 真命中 |
| collab / multi-peer / multi-client / multi-writer / 多端 / 多写 / 协同 | **0** | 无命中（含 CONTRACT.md / gates.md / README.md / web/README.md） |
| broadcast / 订阅 / subscri / publish / fan-out | **3** | lib.rs:77 注释"View 与 AI 订阅的**同一** op 流事件（轮询式取走）"；fingerprint.rs:273/282 = "pub subtree_size" 匹配 pub.?sub 的假阳性 |
| resync / offline / 离线 / reconnect / 重连 | **5** | 全部为本地 fullResync/sync_from_markdown 面（r17_id_boundaries.rs:10、web_smoke.py:258-297、main.ts:815/862） |
| conflict / 冲突 / merge / 三方 | 多 | 全部本地域：merge_block 编辑原语（lib.rs:528）、合并窗（history.rs）、document_id 重放冲突拒绝（lib.rs:970-980）、外部重载 OfferChoice（external.rs:54）、settings 部分合并、gates.md git rebase 历史注记（:106） |
| transform / xform / position map / rebase | **5** | 全部为"**绝不静默 rebase**"设计立场表述（lib.rs:90/:377、r12_ai_loop.rs:599、spike_semantics.rs:594）+ gates.md:106 git 注记；**无任何 op 变换函数** |
| Cargo.toml 全 workspace：tokio/axum/http/socket/grpc/reqwest/url | **0** | 依赖面零网络组件【代码确认：全 workspace 五 crate Cargo.toml 亲读（含 shell-native 蓝图态），mdcore 仅 ropey+serde+serde_json——复审修正口径】 |

### 1.4 B12 落级结论

**B12 证伪校准成立并落级【代码确认】**：collab-a §6.3 转述的"既有 sync_resync/
revision 体系（可与 CRDT 叠加）"系望文生义——sync_resync=本地兜底，revision=本地
单调计数，**均非协同资产**；"可与 CRDT 叠加"仅为源报告推断，本仓代码/文档零对应
实现。唯一需补的精度注记：baseline B12 行"全仓 grep 全部 0 命中"应加注
"yrs 在 package-lock 哈希串有 1 假阳性"（见 §5 勘误 2）。**现状 = 协同六能力中
的多端面全部缺失**（§2）。

## 2. 六项能力三态判定矩阵

> 判定口径：能力定义=**多端协同范畴**（网络/并发/多写者）。本地 view-resync
> （单进程 CM6↔核对账）与多端协同是两个范畴，逐项显式区分。

| # | 能力 | 判定 | 相邻既有事实（不算该能力） | 证据级别 |
|---|------|------|--------------------------|----------|
| 1 | 同步（多端状态同步） | **缺失** | sync_from_markdown=本地 view 对账；extend_from_markdown=渐进装载 | 【代码确认】§1.1+§1.3 |
| 2 | OT（操作变换） | **缺失** | 无 transform 函数；反方向立场="绝不静默 rebase" | 【代码确认】§1.3 |
| 3 | CRDT | **缺失** | 0 真命中；无任何冲突无关复制结构 | 【代码确认】§1.3 |
| 4 | 广播（多订阅者分发） | **缺失** | take_events=**单消费者排空队列**（详见 2.4） | 【代码确认】 |
| 5 | 离线（离线编辑+重连合并） | **缺失** | adopt_external=外部文件重载（语义边界详见 2.5）；autosave/会话恢复=持久化便利 | 【代码确认】 |
| 6 | 冲突（并发冲突自动检测/解决） | **缺失（自动层）** | plan_reload OfferChoice=**人机 UX 决策原语**；base_revision gate=单核进程内乐观检查 | 【代码确认】 |

### 2.1 同步 —— 缺失

- 网络层：产品唯一协议面 `mdcore://app` = "静态资源只读，**无网络面**"【文档记录→
  本轮亲读确认：CONTRACT.md §2:41-42】；mdc-pdf 的 `mdcore://pdf` 同为静态视图层
  （CONTRACT.md §6:264-265）。依赖面零网络 crate（§1.3 末行）。
- 协议层/数据层：无任何对端握手、版本交换、状态向量、快照差分协议。
- 区分登记：sync_from_markdown（lib.rs:765）与 adopt_external（external.rs:101）
  均为**单进程内**"文本→树"的收敛机制，消费方是本地 CM6 View 与本地文件 watcher。

### 2.2 OT —— 缺失

- transform/xform/位置映射协作版：0 命中（§1.3）。op.rs 内唯一位置映射
  `adjust_marks`（op.rs:196）是**单块内 mark 区间随 ReplaceText 的本地平移**
  （四情形），与并发操作变换无关。
- 现有并发处理模式（同进程双写者 View/AI）：base_revision 精确相等门禁（lib.rs:378-385，
  不等即 `Reject::Stale`，**不做 rebase** lib.rs:377）→ 拒绝后**重读重规划重试**
  （【测试验证】r12_ai_loop.rs:563 `ai_stale_transaction_rejected_then_retries_with_fresh_base`
  ：stale 提交→零污染→重读→确定性重规划→新 base 重试）。此为"拒绝+重试"模式，
  无操作变换成分。

### 2.3 CRDT —— 缺失

- 0 真命中（§1.3）。IdAllocator（id.rs:23-42）是**本地**单调分配器（永不回收 +
  observe 抬水位），无跨副本合并语义；BlockId 是本地身份非分布式版本。

### 2.4 广播 —— 缺失（take_events 不构成广播）

【代码确认】事件机制的真实形状：
- `Core.events: VecDeque<Event>`（lib.rs:137），`take_events` = `drain(..)` **整体
  排空**（lib.rs:237-238）；事件 push 于每次事务（lib.rs:365-371），队列**无容量
  上限**（无人排空则增长，mem_stats 以 events_pending 暴露 lib.rs:206）。
- 消费端：wasm 桥单出口（mdcore-wasm lib.rs:92-95）；web 层 main.ts **恰两处**
  排空点（:781 drainEvents、:892 渐进聚合护栏），**同属一个 Bridge 类**——先到者
  取走全部，后到者拿空。无订阅者注册表、无按订阅者复制、无 fan-out。
- `window.__mdcoreEvents`（main.ts:782/894）是**append-only 证据数组**（冒烟测试
  与"未来 AI 侧"共用 op 流证据，main.ts:778 / web/README.md:38——此为文档记录的
  意图注记，非订阅机制）。
- **判定**：take_events 是单消费者轮询队列（"View 与 AI 订阅的同一 op 流"lib.rs:77
  ——同一**条**流，非各持一条），不构成多订阅者分发。多端广播缺失。

### 2.5 离线 —— 缺失

- offline/reconnect/离线/重连：0 命中（§1.3）。
- adopt_external 语义边界【代码确认】：external.rs 模块头（:1-14）自述"外部文件
  重载（P1-8 核侧）"——宿主 watcher/窗口聚焦触发 → FileStamp（mtime+size，:30-41）
  比对 stale → 读盘采纳。这是**文件监视重载**场景，非"离线编辑后重连合并"：无离线
  编辑队列、无重连握手、无双方 op 合并。
- 相邻既有：storage.autosave_enabled（默认 false，CONTRACT.md §4:122）、
  session_restore（§5 会话恢复）均为持久化便利，与离线协同无关。

### 2.6 冲突 —— 缺失（自动检测/解决层）

- 现有"冲突"面的两个真实机制，语义边界如下：
  ① `plan_reload` OfferChoice（external.rs:61-69）= **人机 UX 决策**：本地脏 +
  外部有变化 → 宿主弹窗，用户二选一（keep-mine=不 adopt / take-external=adopt）；
  弹窗文案须注明 take 不可撤销【文档记录：gates.md:458-459】。**无任何自动合并**。
  ② base_revision gate（lib.rs:378-385）= 单 Core 进程内的精确相等检查（结构命令
  专用，文本命令靠锚点/expect 保护，lib.rs:564-597 面向 AI 路径）——对象是"过期
  提交"而非"并发写者"，且对 coalesce 透明的缺口见 §4-C4。
- 并发冲突的自动检测（如同块双写识别）/自动解决（合并策略）：**缺失**——v0 明确
  不做区域级合并推断（external.rs:58-60，见 §3.3），"merge 与区域级静默合并留 v1"
  为计划性文字【文档记录：gates.md:465/:506——**登记为计划，非实现**】。

## 3. adopt_external 的冲突语义（external.rs 全文亲读）

### 3.1 plan_reload 三态的确切触发条件

【代码确认】external.rs:61-69，纯函数，输入 = (BlockMatchReport, local_dirty)：

| 态 | 条件 | 语义 |
|---|---|---|
| Noop | `!report.has_changes()`（moved/edited/added/removed 四类全空；preserved 满额不算变化，fingerprint.rs:326-334） | 外部内容与现树逐块等价（mtime 变化多为自存回声/触碰）——忽略，**即使本地脏也忽略**【测试验证 r23_external.rs:87 plan_noop_when_unchanged】 |
| SilentAdopt | 有变化 ∧ `!local_dirty` | 本地无未存改动 → 静默重载不打扰【:101 plan_silent_adopt_when_clean】 |
| OfferChoice | 有变化 ∧ local_dirty | 冲突 UX（keep-mine/take-external）【:110 plan_offer_choice_when_dirty】 |

local_dirty 的权威取值 = `Core::is_dirty()`（粘性脏标记，lib.rs:139-145/:216-218；
View/Ai/Undo/Redo 置位，System 装载不置位，adopt 成功清位）。

### 3.2 外部变更 vs 本地未保存变更的关系

**既非三方合并，也非自动覆盖，而是"人机选择 + 全量身份保持重建"**：
- take-external 路径（external.rs:101-171）：按匹配报告携带身份全量重建——
  op 合成 = RemoveBlock[0]×n 清空旧顶层 + 先序 InsertBlock（preserved/moved/
  edited 携旧 id，manifest 可信 added 携声明 id，其余现场分配，:139-150）→
  `apply_ops(Source::System, "external_reload")`（:152-158，ADR-B4 不入 undo 栈）→
  `history.clear_interaction()`（:160，ADR-B7：被放弃文档态的 undo/redo 对重建后
  的树会 BadPath，保留即坏引用——**本地未保存交互史整体作废**）→ `mark_saved()`
  （:163，脏清位=缓冲==外部内容）。**本地未保存文本内容在 take-external 时被
  放弃且不可撤销**（Noop 早退路径不清脏，:161-162 注）。
- keep-mine 路径 = 不调用 adopt，树零触碰。
- 失败面：apply_ops 事务内失败 → 求逆回滚树零污染（:99-100）。

### 3.3 v0 明确不做什么（区域级合并推断为何被收缩）

【代码确认】external.rs:58-60 原文依据："只要有**任何**外部变化且本地脏，一律
OfferChoice——不做'变化区域不相交则安全合并'的区域级推断（**核无逐块本地脏
标记**，贸然合并等于替用户做决定；v0 明确收缩）"。收缩的技术根因 = 脏标记粒度
是**文档级 bool**（lib.rs:145），不存在块级/区间级本地改动记录，因此无区域相交
判定的事实输入。计划侧对应登记：merge 与区域级静默合并"留 v1"【文档记录：
gates.md:465——计划文字，非实现】。

### 3.4 manifest 快路径与"合作写手"场景

【代码确认】fingerprint.rs:373-383：`ManifestEntry{id, structural, content}` =
"新树先序第 p 位 = 该 id + 该双指纹"，由 **Conforming 写手（我方编辑器存档）写入
meta 侧车"；匹配端**全有全无**核验（:439-449）：条数与新树先序节点数一致 ∧ 每条
双指纹逐位相等，才允许直接对位；任何一位不符 = 清单过期/异源，整体弃用走结构锚/
指纹兜底（【测试验证】r23_fingerprint.rs:276 manifest_verification_failure_falls_back
：长度不符→弃用；单条指纹不符→整体弃用）。

"合作写手场景"（合作写手跨端场景，r23_external.rs:251-276）：另一端的我方
Conforming 编辑器存档 md + manifest 侧车 → 本端重载时，**added 类块若 manifest
声明了 id 则视为可信新身份直接携带**（external.rs:130-135），经 InsertBlock
id=Some 物化 + create_node observe 抬水位（op.rs:115-119；tree.rs:178-182），
跨端保持块身份。**性质登记**：这是文件级（md+侧车）的身份携带机制，非状态同步
协议——无网络、无增量传输、无冲突处理（本地脏时仍走 OfferChoice 人机决策）。

边界注记【代码确认】：manifest 快路径核验只覆盖"指纹 vs 新树"，**无清单内重复
id 校验**；id_index 注册面为 BTreeMap::insert 直接覆盖（tree.rs:154-156），无
重复 ID 拒绝——ID 唯一性靠"分配器单调 + 上游 ID 纪律"（id.rs:20-21、lib.rs:1000-1002
R9 修复注）维持。当前 v0 manifest 为宿主供给的可信输入，此边界仅登记。

## 4. revision 体系约束事实登记（事实基座，不选型）

### 4.1 基本事实

| # | 事实 | 锚点 | 级别 |
|---|------|------|------|
| C1 | revision 单核生命周期内单调不减；推进点恰三处：非合并前向事务（entry 级 bump）、System 装载事务、undo/redo | lib.rs:288/:321/:890/:919 | 【代码确认】 |
| C2 | **coalesced 事务不推进 revision**：合并分支以未 bump 的当前号记日志+发事件（`log.record(self.revision,…)`） | lib.rs:303-318；模块注释 revision.rs:3-4"coalesced 追加同 revision 号多记录，重放按序合并语义等价" | 【代码确认】 |
| C3 | **undo/redo 推进 revision 且入日志**（Source::Undo/Redo；R11 审查修正：不记会导致重放与实时分叉） | lib.rs:890-894/:919-921；revision.rs:5-8 | 【代码确认】 |
| C4 | base_revision 门禁 = 精确相等否则 Stale，**无 rebase 路径**；且 coalesce 不 bump ⇒ 同号下已发生的文本事务对 gate 不可见（门禁粒度=entry 级非事务级） | lib.rs:303-318/:377-385 | 【代码确认】 |
| C5 | 持久化：save_document meta = {document_id, revision, history 整栈 JSON}；open_document 三者全恢复；replay_jsonl 逐条 `core.revision = r.revision`（终值=末条记录号），undo 栈不重建（设计如此）；snapshot 携 revision | document.rs:220-230/:253-262；lib.rs:963-996；snapshot 面 lib.rs:947-961 | 【代码确认】 |
| C6 | 粒度 = **文档级**单计数器；无块级版本字段（BlockId=身份非版本） | lib.rs:133；id.rs:12 | 【代码确认】 |
| C7 | history Entry.revision = 条目创建（首事务）时的核号；coalesce_into_last 只追加 ops/inverses 不改 Entry.revision ⇒ 一个 undo 条目可跨多条同号日志记录 | history.rs:42-52/:119-134；lib.rs:320-321 | 【代码确认】 |
| C8 | 事件 Event{ops, meta, revision} 随事务发出；合并事务事件带 coalesced:true 且 revision 沿用旧号 | lib.rs:70-83/:311-318/:365-371 | 【代码确认】 |
| C9 | document_id 一次性落定：已持不同值拒绝替换；genesis 记录携带；重放遇冲突 id 拒绝；会话面一窗一档（同 id 不得双窗） | document.rs:192-216；lib.rs:966-980；CONTRACT.md §5:155-157 | 【代码确认】 |
| C10 | 合并窗依赖宿主注入毫秒时钟（wasm32 无 SystemTime；JS Date.now 每次变异前注入；COALESCE_MS=1000） | lib.rs:119-123；mdcore-wasm lib.rs:170-174；main.ts:768-770；history.rs:20 | 【代码确认】 |

### 4.2 对"未来任何并发版本体系"构成的事实约束（只登记，不下结论）

- **K1（唯一序号约束）**：同 revision 号可对应多条 LogRecord/多个 Event（C2）⇒
  revision 不能充当日志记录的唯一序号；外部观察者若需逐记录排序，现有体系内
  只能用记录下标，无现成单调记录号。
- **K2（内容≠号码约束）**：undo/redo 使号码前进而内容可回退（C3）⇒ 同一文档
  内容可出现在多个 revision 号上；revision 号不标识唯一内容状态。内容等价性
  判定现有面 = 指纹（TreeFingerprint）与逐块对照（match_trees），与 revision
  无关联字段。
- **K3（事件去重约束）**：合并事务事件与所属条目 bump 事件同号（C8）⇒ 事件流
  消费者不得按 revision 去重/对齐；coalesced 布尔是唯一区分位。
- **K4（门禁语义约束）**：结构命令的并发保护=精确相等+拒绝+重读重试（C4、
  §2.2），无变换/重放路径；且门禁对同号内 coalesce 事务不可见。文本命令
  （replace_text lib.rs:564 / set_mark :569）无 base 参数即无 base 门禁，靠
  锚点寻址（BlockId+offset）+expect 内容保护（resolve_text_target 的
  ByAnchor+expect 分支，lib.rs:698-746，expect 内容比对在 :732-737——复审修正锚点区间）【代码确认】。
- **K5（单写者模型约束）**：Core 变异面全部 `&mut self`（无内部锁/共享会话）；
  会话层一窗一档 + document_id 拒绝换牌（C9）⇒ 现有模型假设"每文档每进程至多
  一个活动编辑会话"，多写者入口不存在（View/AI 双写者是同进程同 Core 的两个
  source 标签，非并发主体）。
- **K6（事件通道约束）**：事件出口=单队列排空（§2.4），无重放/回补接口（重放
  面在 RevisionLog JSONL 导出，C5）；迟到的消费者无补偿机制。
- **K7（ID 空间约束）**：分配器单调永不回收 + observe 抬水位（id.rs:20-42；
  tree.rs:178-182）⇒ 外部声明 ID 可无撞车纳入活树（manifest 机制的事实基础，
  §3.4）；但 id_index 注册面无重复 ID 拒绝（tree.rs:154-156），唯一性纪律在上游。
- **K8（时钟约束）**：合并窗与 Entry.at_ms 依赖宿主时钟（C10）⇒ 跨端时钟不可
  比较的现状下，历史条目的时间字段仅本机语义。

## 5. P1 / baseline 勘误登记（本轮发现，不阻塞）

1. **P1 §1 O2 计数误差**：标题"WasmCore 19 导出"与该节逐条清单不符——逐条
   清单（构造/身份 4 + 命令面 4 + 事件面 1 + 序列化面 9 + 装载面 2 + 杂项 3）
   = **23 项**，与实测 `rg 'pub fn' mdcore-wasm/src/lib.rs | wc -l` = 23 一致
   （22 方法 + 构造器，含 2 静态方法 restore/replay）。以逐条清单为准。
2. **baseline B12 精度注记**："全仓 grep 全部 0 命中"应补注"yrs 在
   web/package-lock.json:128 的 sha512 哈希串内有 1 假阳性子串命中"（§1.3）。
3. 其余复核项：baseline B12 行对 sync_resync.rs 的判定与 10 用例枚举、revision.rs
   性质描述均与代码一致（本轮逐行亲读复核）；P1 §3 链②/链④ 的 adopt 与
   take_events 描述与代码一致。

## 6. P4 验收对照

| PLAN P4 验收项 | 结果 |
|----------------|------|
| ① B12 核查（存在性/位置/公开面/与 adopt_external 关系） | ✅ §1（独立复核成立 + 精度注记） |
| ② 六项能力三态判定 + 证据 | ✅ §2 矩阵 + 逐项证据（缺失项全部附 grep 搜索记录 §1.3） |
| ③ adopt_external 冲突语义 | ✅ §3（三态触发条件 / 非合并非覆盖=人机选择+全量身份保持重建 / v0 收缩根因 / manifest 场景） |
| ④ revision 约束事实登记 | ✅ §4（C1-C10 + K1-K8，含 coalesce 不推进、undo/redo 推进入日志、持久化三路径、文档级粒度） |
| 零选型纪律 | ✅ 全文无选型/比较/推荐表述；计划性文字（gates.md"留 v1"）均标注【文档记录】非实现 |
