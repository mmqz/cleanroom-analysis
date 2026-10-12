# P6 性能耦合与 PDF 复用风险审计（p6-perf-coupling-pdf-risk.md）

> 状态：完成（2026-10-12 执行轮，Task t7-p6-survey）。对应计划：PLAN.md v1.2 §4 P6。
> 证据基线：mdcore main=03815b6 本轮克隆亲读（strict 只读；收尾复核
> `git status --porcelain` 0 改动、HEAD=03815b6）。证据级别标注按 PLAN §2
> 五级；锚点 `路径:行号`。
> **性能纪律**：本文件零性能宣称。全部量级判断标【推断】；唯一引用的实测
> 数字 = P0 轮亲跑门禁（baseline.md §1.2）与测试源码在档断言阈值，均注明
> 出处、不外推。MD 面基准现状注脚见 §1.6 / §5。

## 0. 任务简报校准与 P1-P5 产物复核

| # | 项 | 结果 |
|---|----|------|
| C1 | 简报称"默认面 259 测试**全部功能测试**，无任何 MD 面 perf 门" | **需校准**【代码确认】：r23_fingerprint::perf_1mb_smoke（r23_fingerprint.rs:404-464）含两条**软时间断言**——match-only identical < 400ms / single-edit < 400ms（debug 构建，:456-463；自述门职责 = "抓 O(n²) 级退化而非绝对毫秒" :455）；r14_gates 三 gate 计时**全部 println 留痕无断言**（r14_gates.rs:5-6 明示"不做脆弱的时间硬断言；策略与 r11_stress 一致"）。精确表述：MD 面**无** pdfc_perf 形态（#[ignore] + 阈值常量 × 实测中位）的性能门，但有 1 条软冒烟断言 + 3 组打点留痕。§5 待基准清单不受影响（该断言只覆盖匹配算法面） |
| C2 | P2 产物复核 | 一致：增量保存=无（宿主面；本轮复核 save_document document.rs:220-230 全量重写）；RemoveBlock 逆 = collect_subtree 全拷贝（op.rs:248-255 本轮亲读一致）；B12 校准口径（sync_resync=本地机制）沿用 |
| C3 | P3 产物复核 | 一致：backup=tree.clone()（lib.rs:267 本轮亲读）；事务无大小上限（lib.rs:253 签名直收 Vec<Op>）；undo/redo 栈纯 Vec 无 cap（history.rs:56-57）；undo_coalesce_ms 死设置（本轮 grep 复核：mdcore src 零消费者，唯一 mdcore 命中 = settings.rs:102 默认值取源 `mdcore::history::COALESCE_MS`，运行时无接线） |
| C4 | baseline B13"全仓 3 处实际可变全局" | 复核成立（§2 本轮全量重扫，仍恰 3 处） |

P1-P5 文件级勘误：**本轮未发现新的 P1-P5 结构性错误**（P1 wasm 计数勘误已由 P2/P5 在档，不重复登记）。

## 1. ① 大文档潜在瓶颈静态分析（只登记，不宣称）

### 1.1 ropey 使用面（读写模式）

【代码确认】per-block Rope：`Node.text: Rope`（tree.rs:51；ropey 1.6 为 mdcore 三外部依赖之一，crates/mdcore/Cargo.toml:7-10——ropey/serde/serde_json，P2 §1 复核一致）。

| 面 | 模式 | 锚点 |
|----|------|------|
| 写路径（变异） | rope 原生 remove+insert（ReplaceText 的 apply）；区间校验前置（start>end 或 end>len → BadRange） | op.rs:135-139 |
| 求逆路径 | slice→String 物化**编辑区**（state-before 被删文本；O(编辑区) 长度上界为断言语义） | op.rs:268 |
| 读路径 | **每块全文 to_string 物化**后按 String 消费——rope 不进任何整文档出口 | 见下表逐点 |
| 长度查询 | len_chars / len_bytes 直读（零物化） | op.rs:135/265；lib.rs:476/550/637/681/728；tree.rs:355/357 |

读路径 to_string 全清单【代码确认，grep 实测】：
- tree.rs:257（collect_subtree——remove 求逆 / deep_clone / 快照三路共用）
- debug.rs:32/91/125/158（blocks_json / blocks_json_range / dump_tree / inspect 全家）
- markdown.rs:55/68/92（tree_to_markdown 逐 kind 出口）
- fingerprint.rs:151/571/585/692/715/787（内容指纹 / 结构锚走查 / 中段兜底对照 / manifest 快路径核验）
- lib.rs:480（split 尾块）/540（merge 前块）/733（expect 比对 slice）/814（diff insert）/1017（事件构造）

【推断】静态形状：per-block rope 把整篇文本分散到块级（单块变异以块长为界）；但所有**整文档出口**（序列化 / dump / 指纹 / diff / save）对每块至少一次 Rope→String 全展开 + 克隆，量级下限随"总文本量 + 节点数"线性。**无本轮基准数据**，此句仅为结构推断，不作任何毫秒/常数宣称。

### 1.2 arena 增长模式

【代码确认】`slots: Vec<Option<Node>>`（tree.rs:99）；slot() 只 push（tree.rs:142-146——NodeId = slots.len() 单调）；remove_subtree 置 None **不压缩不回收**（tree.rs:269-286：stack 走查 + unregister + slots[cur]=None）；tree.rs 全文无 shrink_to_fit / retain / swap_remove 调用点（本轮通读）。NodeId 与 slots.len() = 历史峰值单调量（P2 §2 一致）。id_index 侧随 unregister 收缩（tree.rs:158-160）；children.retain 收缩父链（:246/:283）。Vec 本体只增长——回滚路径 `self.tree = backup` 是整体替换（lib.rs:274/279）非收缩。

mem_stats 观测面【代码确认】：slots_capacity_bytes（含 None 空槽，tree.rs:371）、rope_est_bytes / rope_text_bytes（:355-357）、id_index_entries（:374）——R20"活数据近似值（非精确 RSS）"，**只观测不限制**（tree.rs:343-344 注释），经 wasm 出口 core_mem_stats（mdcore-wasm lib.rs:118）给宿主内存面板。

【推断】大文档反复增删的内存行为：活节点数下降时槽位数组容量不缩——内存高水位 = 历史峰值节点数 × sizeof(Option<Node>)，不随删除回落；id_index 与 children Vec 随删除相应缩小（这两项与 slots 不同步）。观测出口在位、无门禁——量级【推断】无数字。

### 1.3 全量遍历点清单（复杂度量级全部【推断】，无基准数据）

| 遍历点 | 访问面（亲读） | 锚点 |
|--------|----------------|------|
| blocks_json | 全树先序 + 每块 attrs 深拷贝 + 全文本 to_string + serde 序列化为单 String | debug.rs:39-45（collect :25-37） |
| blocks_count | 全树走查计数（专为计数遍历整树——R16 分片游标基准） | debug.rs:49-61（:47-48 注释"免去为取一个计数而全量 dump 20 万块"） |
| blocks_json_range | 先序游标走查至 from+count（前段跳过仍需逐节点走查，剩余计数短路） | debug.rs:69-112（remaining==0 短路 :79/:96-98） |
| tree_to_markdown | 每 kind 出口全文本物化 | markdown.rs:32 起（:55/68/92） |
| collect_subtree | 子树全拷贝：每节点 Rope→String + attrs/marks clone；remove 求逆（op.rs:248-255）/ deep_clone（tree.rs:315-329）/ 调试共用 | tree.rs:252-266 |
| TreeFingerprint | 下标 memo（NodeId.0 → Vec<Option<u64>>，每节点至多算一次，无 HashMap——迭代序排除设计 :93-94）；content_of 每块 to_string + Vec<char> 收集（:151-153） | fingerprint.rs:95-159 |
| match_trees | 三级策略：manifest 快路径（全有全无核验）→ 结构锚走查 → 中段兜底对照 | fingerprint.rs（P1 §1）；策略分支间量级差异大——整体【推断】 |
| sync_from_markdown diff | 先整档 parse 出**第二棵树**（markdown_to_tree），再逐位置对照：common 段每块 kind/attrs 比较 + Rope PartialEq（文本不等时再 to_string 物化差异块 :809-815）；尾部删/插 op 合成 | lib.rs:765-781（diff_forest :784-829；"允许非最优，不追求树编辑距离" :756） |
| snapshot to_json / from_json | 全树递归（to_snap 逐节点拷贝含 data.id） | snapshot.rs（P2 §3 路 B 锚点族） |
| replay_jsonl | 逐记录 apply（每记录内含 backup 深拷贝） | lib.rs:966-996 |

最接近基准的既有证据【测试验证】：r23_fingerprint::perf_1mb_smoke——1MB/6300 块（:412-413），断言 match-only identical / single-edit < 400ms（debug，:456-463）+ eprintln 记录 parse/match 分项（:452-454）。仅覆盖匹配算法面；不外推其他遍历点。

### 1.4 序列化全量重算点

【代码确认】save_document 每次全量：tree_to_markdown 全文 + `history.to_json()` **整栈**（含 undo/redo 双栈全部 ops/inverses）嵌入 meta（document.rs:220-230；P2 §5 三态判定 = 宿主面无增量，本轮复核一致）。history to_json / RevisionLog to_jsonl 均全量 dump（revision.rs:60；P2 §1 表）。mem_stats 聚合 = 全树走查 + 全栈走查（lib.rs:201-210：tree/log/history/events_pending 六组件）。
【推断】长会话 + 大文档下，保存成本随"文档体量 + undo 栈深度 × 条目 op 数"两维增长（Entry 双持 ops+inverses，P3 §4④）；undo 栈无上限（§1.5）→ 保存面无上界假设可依。零数字。

### 1.5 每事务固定成本（P3 产物消费 + 本轮复核）

【代码确认】三路径 backup=tree.clone() 深拷贝：apply_ops（lib.rs:267）/ apply_raw_batch（:350，undo/redo 专用）/ replay 每记录（:985-991）；RemoveBlock 求逆 collect_subtree（op.rs:248-255 → tree.rs:252-266）；Entry 双持 ops+inverses（history.rs:42-52，mem_stats 注释 :196-198"Entry 双持"）；事务大小无上限（lib.rs:253）；undo/redo 栈无 cap（history.rs:56-57，P3 G5）；事件 VecDeque push_back 无上限（lib.rs:365-371）。
既有缓解【代码确认】：OpText = Arc<str> 三方共享（R22-a——log/事件/克隆共享分配，lib.rs:1130-1185 测试锁）。
【推断】大树频繁小事务：每次事务含一次全树深拷贝（每节点 Rope clone + attrs BTreeMap clone + marks Vec clone + children Vec clone），量级随节点数线性；undo/redo 再各一次（apply_raw_batch）。**本轮无数字**——r14_gates gate2 有同款打点（100 单 op p50/p95 + 100-op 批量 + undo + 小文档归因探针 r14_gates.rs:158-234），println 留痕、无断言；gates.md 历史数字【文档记录：本轮未复验】，不引用。

### 1.6 MD 面基准现状（默认面 259）

- **无** pdfc_perf 形态性能门：mdcore tests 14 文件全部默认运行（无 #[ignore] 计时断言套件）【代码确认，ls + 通读】。

既有计时面盘点【代码确认】：

| 位置 | 形态 | 断言？ | 覆盖面 |
|------|------|--------|--------|
| r23_fingerprint::perf_1mb_smoke | 常规 #[test] + eprintln | **有**（match-only identical/edited < 400ms debug，:456-463） | 指纹匹配算法（1MB/6300 块）；自述职责="抓 O(n²) 级退化" :455 |
| r14_gates gate1（1.2MB ingest+replay） | #[test] + println | 无（:5-6 立场声明） | 重放面 |
| r14_gates gate2（20k 块单 op p50/p95 + 批量 + undo + 小文档归因探针） | #[test] + println | 无（唯一功能断言=块数稳定 :214） | 事务管线 |
| r14_gates gate3（1MB snapshot/restore） | #[test] + println | 无 | 快照面 |
| r11_stress（20k 块结构压测） | 纯功能 | 无（:3-6"全部功能性"） | 原子性/身份/重放 |

- gates.md 历史打点数字【文档记录：本轮未复验】（P2 U3 / P3 U4 同源，不引用）。
- **结论**：MD 面性能现状 = 打点留痕 + 1 条软冒烟断言（护 O(n²) 退化下限）；绝对毫秒级门为零；事务/序列化/桥面无任何断言。待基准项清单见 §5。

## 2. ② 全局可变状态清单（全仓枚举）

枚举方法【代码确认】：`static |OnceLock|lazy_static|thread_local` 四关键词，mdcore + mdcore-wasm + mdshell（crates/shell）+ mdc-pdf（crates/pdf）四 crate **src/** 全扫（不含 tests/，按任务口径）。命中 = 3 处实际可变全局；另 6 处 `&'static str` 类型注解（html.rs:75/525/1143、op.rs:72、pdf command.rs:161、shell settings.rs:138——字符串字面量类型标注，非全局状态，不计——终审修正计数）；lazy_static / thread_local **零命中**。shell-native（蓝图态 69 行）零命中。

### 2.1 三处全局逐一标注

| # | 位置 | 类型 | 线程安全证据 | 失败可重试性 | 多实例/多文档含义 |
|---|------|------|--------------|--------------|-------------------|
| G1 | mdcore document.rs:25 `ENTROPY_SEQ` | `static AtomicU64` | fetch_add(Relaxed)（document.rs:31）——原子操作；`std_entropy` 混入 RandomState OS 种子 + 单调计数器 + 栈地址 ASLR（:29-37，模块注释"非加密强度，按任务书豁免"） | 不适用（计数器无失败路径，只增） | 进程级共享熵源，仅用于 DocumentId::generate 随机位混合（:47-50）。多 Core 实例只共享此熵——**无任何文档状态耦合**；反而使跨实例 document_id 天然互异 |
| G2 | pdf engine_pdfium.rs:76 `PDFIUM` | `OnceLock<&'static Pdfium>` | OnceLock 幂等 set；set 仅出现在绑定成功路径（:348 shared_pdfium / :359 bind_explicit / :406 try_bind；系统分支的 set 经 shared_pdfium:348 调用链完成，:388 为 return 点非 set 点——终审修正位点）；`&'static Pdfium` 经 Box::leak（"Pdfium 结构体极小…泄漏成本可忽略" :13-15） | **可重试**【代码确认】：绑定失败不写 OnceLock（bind_by_resolution Err 直接返回 :395-398；try_bind Err 不 set :409-414）→ 下次调用重走定位序（env → 候选清单 → 系统，:369-399 错误消息含全部尝试路径——"不静默降级"） | 进程级单绑定：所有 PdfiumEngine 共享一个库实例；多文档 = 多个独立 pdfium 文档句柄，字节各自持有（模块文档 :17-20"load_pdf_from_byte_vec 全内存、路径无关…单进程多引擎 = 多个独立 pdfium 文档句柄"）。**单库版本 per 进程**：二次绑定请求报 PdfiumLibraryBindingsAlreadyInitialized（:410-412"外部绑定者不受支持——本 crate 单例未就位的竞争窗口"）——需要不同 pdfium 版本并存的消费者在现形状下不可行 |
| G3 | pdf engine_pdfium.rs:80 `BIND_LOCK` | `OnceLock<Mutex<()>>` | Mutex 串行化并发首个 open；中毒复位 into_inner（:343 `unwrap_or_else(|poisoned| poisoned.into_inner())`——"锁毒化不复位：绑定失败不持锁状态" :78-79）；双检等待窗口（:338-346"双检（等待期间他线程完成绑定）"） | 无失败缓存（锁无状态） | 竞争仲裁专用；不承载绑定结果，与 PDFIUM 分工 |

范围外注记【代码确认】：r14_gates.rs:19-24 测试内 `static LOCK: OnceLock<Mutex<()>>`（gate 计时串行化——"并行 build_big 会互相污染内存带宽" :17-18）——tests/ 范围，不属产品面，登记不计。

**判定**：MD 侧唯一可变全局 = ENTROPY_SEQ（熵源）；Core 本体零全局（§3-R1）→ **多 Core 实例并存无静态状态障碍**。PDF 侧两全局均为引擎绑定生命周期管理，与文档数据无关；PdfSession 本体零全局（document.rs:98-122 十二字段纯会话态——终审修正计数）。

## 3. ③ 复用限制面（单实例假设清单）

| # | 约束 | 事实与锚点 | 级别 |
|---|------|-----------|------|
| R1 | 单 Core 单文档假设 | Core = 纯数据聚合九字段（tree/history/log/revision/document_id/selection/events/clock/dirty，lib.rs:129-146），**零 static**。多 Core 实例并存：无共享状态、无互斥需求、无初始化序依赖（唯一进程级共享 = ENTROPY_SEQ 熵源，§2-G1）；revision/事件/树/时钟全为实例内（P2 §6、P4 C10） | 【代码确认】 |
| R2 | Core 并发共享形态 | 全部变异面 `&mut self`（无内部锁；P4 K5"每文档每进程至多一个活动编辑会话"单写者模型）；字段 pub tree / pub history / pub log（lib.rs:130-132）公开可变——跨线程共享需消费方自带同步；直访旁路在类型层敞开（P5 §1.1 同源） | 【代码确认】 |
| R3 | SessionManager 一窗一档 + 上限 16 | DEFAULT_MAX_SESSIONS=16（session.rs:39，"桌面编辑器惯例量级，每 session 一个 wry WebView + wasm 核，内存为主要约束" :37-38）；三重硬门 DuplicateWindow/DocumentAlreadyOpen/LimitReached（session.rs:149-167）；`sessions: BTreeMap<WindowId, DocumentSession>`（session.rs:69）——**类型层只装 MD 会话**：DocumentSession{core,id,path}（shell lib.rs:43-49），SessionRecord v1 只持久化 window→document_id+path（session.rs:212-227）。第三模型/多画布会话在现类型层无槽位、无窗口映射、无持久化 schema。仅登记约束，不设计 | 【代码确认】 |
| R4 | wasm 桥单实例假设 | window.__core / window.__bridge 单例，三条启动路径各装一次（main.ts:1320-1322 渐进 / 1352-1360 一次性 / 1394-1396 __inject 重建）；take_events = `drain(..)` 整体排空（lib.rs:237-239），桥内恰两排空点 :781（mutate 后 drainEvents）/ :892（渐进聚合护栏）**同属一个 Bridge 实例**（P4 §2.4 复核一致——先到者取走全部，后到者拿空；无订阅者注册表、无按订阅者复制、无 fan-out）。**第二消费者后果链**【推断，依据 P4 §2.4 + main.ts 结构】：外部代码直接调 window.__core.take_events() = 与 CM6 桥争抢同一 drain 队列（先取先得、取走即空）→ 桥丢失事件 → BlockMap 与核失步 → 依赖事件携带 id 的定位（main.ts:1036-1042 insert_block 消费）失效。无仲裁、无补偿（P4 K6） | 【代码确认】+ 后果【推断】 |
| R5 | 跨 crate 消息大小/频率假设 | blocks_json 全量 dump：每块 attrs 深拷贝 + 全文本进**单 JSON String**（debug.rs:25-45）；量级注记（注释级非实测）："10MB 文档每次 ~20 万块对象"（debug.rs:67，R16 设计注记）；缓解出口 = blocks_json_range 分片（debug.rs:69-112 + r16_blocks_range 功能锁：切片拼接 ≡ 全量、越界截断）；blocks_count 亦需全树走查（debug.rs:49-61）。事件积压：VecDeque 无界（lib.rs:137/365-371），events_pending 仅可观测（lib.rs:206），无上限告警无背压 | 【代码确认】（量级数字为注释记录） |
| R6 | wasm 出口频率假设 | 桥在每次变异后 drain（main.ts:774-783）；渐进聚合护栏 :892；window.__mdcoreEvents 被动累积数组（main.ts:782/894，append-only 证据面）——宿主轮读，无推送（P5 §2） | 【代码确认】 |

## 4. ④ PDF 复用风险（v1.2 新增重点；用户核心关切）

### 4.1 (a) mdc-pdf 对 MD 基础设施的实际依赖面图谱

**编译级**【代码确认】：pdf/Cargo.toml 依赖 = serde + serde_json + pdfium-render(optional, pin =0.9.4) + lopdf(optional, pin =0.45.0)（:20-27），default = [] 零引擎依赖（:29-30）；**对 mdcore 零 path 依赖**（P1 §2.1 / P5 §1.4 / 本轮三方复核一致）。

语义级共享点逐条（代码实测）：

| # | 共享点 | 机制（同步/纪律是什么） | 锚点 |
|---|--------|------------------------|------|
| D1 | Source 五值语义镜像 | pdf command.rs:23-31 ↔ mdcore history.rs:24-32（View/Undo/Redo/Ai/System；serde snake_case 词面逐字一致——command.rs:20-22 自述"语义镜像而非类型复用"）。**同步机制 = 语义锁测试** `pdfb_skeleton::source_mirrors_mdcore_five_values`（pdfb_skeleton.rs:1040-1079）：(a) 锁 pdf 侧五值 serde 名（:1043-1056）；(b) **直读 mdcore 源文件文本**（fs::read_to_string `crates/mdcore/src/history.rs` :1059-1060）锁三点——`pub enum Source` 在位（:1061）、rename_all="snake_case" 属性存在（枚举声明前 200 字符窗 :1065-1071，pdf-b-1r 增严：防镜像锁在无属性时假阳）、五变体逐个 contains（其后 400 字符窗 :1072-1077）。手法同 contract_freeze.rs 蓝图词面锁——零 path 依赖下的文件级证据 | 【代码确认】 |
| D2 | Source 镜像的**缺口** | 若 mdcore **新增**第六个 Source 值：镜像锁**不报警**（测试只断言既有五值在场 :1073-1077，不查多余变体）→ 单向漂移静默发生；pdf 侧 Source 封闭枚举遇第六值字符串 → 反序列化失败（serde 未知 tag 报错，P2 §5 演化约束同源）。改名/删除/改 rename_all → 测试失败（有守卫） | 机制【代码确认】+ 影响【推断】（依据链：:1073 常量名遍历五值） |
| D3 | 协议族 scheme 命名共享 | SCHEME="mdcore" 与宿主壳自定义协议 mdcore://app 同 scheme（protocol.rs:13-14"与 MD 面 mdcore://app 同 scheme——CONTRACT §2"；CONTRACT.md §2:41-42 静态资源只读无网络面 / §6:264-265 `mdcore://pdf/<document-slot>/…` 独立轻量视图层不经 CM6）；URL_PREFIX/EVENT_COMMAND/EVENT_EVENT 词面冻结（pdfb_skeleton.rs:1086-1108）；`<document-slot>` 语义显式"宿主域"（protocol.rs:21-23"多窗路由与槽位复用规则属宿主域"）；window 面零耦合（`__contractVersion` 不因 PDF 面变化——CONTRACT §6:236-239，v1.3 变更记录 :315 在档） | 【代码确认】 |
| D4 | 会话/设置面共享 | **零共享**：shell src grep pdf/Pdf 零命中（本轮实测）；SessionManager/DocumentSession 只装 Core（§3-R3）；设置面四层（editor/ai/storage/appearance）全 MD 域。**PDF 会话现在只活在 mdc-pdf crate 内**——测试直用 PdfSession（pdfd_session_flush/pdfd_writer/pdfc_* 六套件），无产品宿主接线（web main.ts grep pdf 零命中）；shell-native 蓝图对接点清单 5 项（窗口创建/文本注入/渐进进度/保存/身份匹配，shell-native lib.rs:26-34）全 MD 面，无 PDF 预留 | 【代码确认】 |
| D5 | 时钟注入纪律同构 | Core::set_clock(ClockFn = Arc<dyn Fn()→u64>)（lib.rs:186-190；wasm set_clock_ms mdcore-wasm lib.rs:172，JS Date.now 每次变异前注入 main.ts:768-770）vs PdfSession::set_clock_ms(u64 快照字段)（pdf document.rs:277-279，标注 created_ms/modified_ms 取会话时钟）。纪律同文（"核不取系统时间（确定性/可测性/wasm32 兼容）" pdf document.rs:34-35 ↔ mdcore lib.rs:186"wasm32 无 SystemTime"），**类型独立零依赖**（闭包 vs u64 字段） | 【代码确认】 |
| D6 | 版本单源耦合 | pdfb_skeleton::version_single_source_with_workspace_package 直读根 Cargo.toml [workspace.package] version 节 + 锁 version.workspace=true 继承、禁独立 version 声明（pdfb_skeleton.rs:1115-1148）——文件级耦合点之二（与 D1 同为"零 path 依赖下的文件级证据"手法） | 【代码确认】 |

### 4.2 (b) 未来模块抽取/适配对 mdc-pdf 的波及面（假设场景，仅风险分析，不设计）

| 场景 | mdc-pdf lib 编译面 | mdc-pdf 测试面 | 契约面 |
|------|-------------------|----------------|--------|
| A：op/history/tree 抽到独立 crate | **零影响**（无 path 依赖，4.1 编译级） | **破**：source_mirrors_mdcore_five_values 直读 `crates/mdcore/src/history.rs` 文件路径（pdfb_skeleton.rs:1059-1060）——文件移动/改名即 expect panic（"mdcore history.rs 在位" :1060）；镜像锁必须随抽取同改读入路径 | CONTRACT.md §6:244-245 镜像表述（"serde 名与 mdcore::history::Source 逐词面一致，语义锁背书"）成为陈旧文档。**注意**【代码确认】：contract_freeze.rs 词面锁只覆盖 v1/v1.1 面（contract_freeze.rs:40-62 v1 关键词 / :67-84 设置面 / :90-110 会话面），**§6 文本自身无 Rust 侧词面锁**——§6 的守护在 pdf 侧测试（protocol_constants_are_frozen 等），CONTRACT.md §6 改动需人工同步，无跨文件机械守卫 |
| B：mdcore 新增 Source 值 | 不破 | 不报警（4.1-D2 缺口——测试只查五值在场） | 无机械守卫；需人工双侧同步（pdf Source 枚举 + 镜像锁五值表 + CONTRACT §6 表述） |
| C：新 Op 变体 / 改 Event 形状 | **零影响**——pdf crate 无 Op/Event 镜像（pdf src 无对应类型引用，本轮 grep；PDF 面事件 = 自持 WorkerReply（worker.rs:78-91，Serialize-only）与 CommandEnvelope（protocol.rs:38-42，透传不 typed 到命令版本），形状独立） | 零影响 | 零影响（CONTRACT §6 不含 Op/Event 形状——命令面摘要只列六命令 :252-260） |

**mdc-pdf undo 现状（关键事实）**【代码确认】：六命令逐条（command.rs:94-121）= ExtractText / Search / Outline / AnnotateAdd / AnnotateEdit / AnnotateList——**无 undo/redo 命令、无 undo 栈、无求逆面、无合并窗**。Source::Undo/Redo 仅为镜像枚举值，pdf src 内零消费（grep `Source::Undo|Source::Redo` 0 命中——五值只在 serde 面存在）；Source 溯源在 execute 中被解构丢弃（document.rs:172-198 `source: _source` 占位——"先冻结在命令面防协议漂移" :170-171）。AnnotateAdd/Edit 不可撤销；AnnotNotFound 注释自证"v0 无删除命令"（command.rs:157）；undo 的最近似替代 = flush 幂等 + 写器同 id 重写原位替换（document.rs:29-30，pdfd_session_flush 锁 6）。**与 MD 面 undo 管线的关系 = 没有关系**（无编译依赖、无语义镜像之外的耦合、无共享测试）——mdcore undo 管线（invert_op/history/合并窗）的任何改动对 mdc-pdf 零波及。

### 4.3 (c) PDF 质量敏感点清单（P7"对 PDF 质量的影响"评估的事实基线）

| # | 敏感点 | 守护测试（文件::用例） | 断言内容（亲读） | 破坏条件（改动什么才会破坏它们） |
|---|--------|------------------------|------------------|----------------------------------|
| Q1 | 增量写原字节前缀逐位保持 | pdfd_writer::original_prefix_bitwise_preserved；::bulk_100_annotations_smoke；::mount_preserves_existing_page_annots；::edit_redirty_no_duplicate_reflow | 两轮修订后 product[..original.len()] == original 逐位（:515-519）；百条规模前缀仍保持（:1119）；form fixture 前缀保持 + 既有 widget /FT /Tx 原样（:1182）；编辑轮仍保持原字节前缀（:832） | writer_lopdf save_incremental 前缀语义 / 修订链只追加逻辑（writer_lopdf.rs:8-18 模块契约）变更 |
| Q2 | pdfium↔lopdf 交叉读互通 | pdfd_writer::pdfium_cross_read_subtypes_and_author | lopdf 写 → pdfium 读：页 0 三条标注四 subtype 全对上 + /T（creator 面）可读（:545-578）；Ink /T Ai 后缀可读（:578） | 写器产出对象形状（subtype 编码 / /T 编码约定 encode_author）改变 |
| Q3 | 四 kind 字段级往返保真 | pdfd_writer::four_kinds_roundtrip_field_exact；::origin_roundtrip_ai_suffix_reversible；::modification_date_pdf_string；::freetext_da_default_appearance | subtype/Rect/T/Contents/QuadPoints（2 quad × 8 数逐位）/InkList 笔划逐位 + 颜色 + 包络 Rect（:375-471）；/T Human 原文/Ai " [ai]" 后缀可逆往返（:594 + ORIGIN_SUFFIX_AI 词面冻结 :446）；/M 日期串（:650）；/DA 词面 "/Helv 12 Tf 0 g"（:1039-1062） | annotation.rs 模型↔writer 映射 / FREETEXT_DEFAULT_DA 改变 |
| Q4 | flush 保存事务语义 | pdfd_session_flush 9 锁（:4-29 锁清单——终审修正计数） | 脏集升序逐条写 + 成功才清脏 + 二次 flush 零追加（幂等）；S5 加密态拒**先于一切**（零脏也拒，:14-17）；no-op edit 不脏；保存失败重试安全（写器幂等合同）；无引擎会话可落盘（attach_writer 不要求 attach_engine） | pdf document.rs flush_annotations 语义序（:281-325：S5 → 零脏短路 → 逐条 → save）改变 |
| Q5 | 性能门 6/6 | pdfc_perf 六测试（#[ignore]，`--ignored --nocapture` 运行） | THRESHOLDS = 实测×3 向上对齐稳定数（pdfc_perf.rs:61-70）：open_ms=1 / first_render_ms=25 / seq_render_p95_ms=5 / hit_median_us=20 + hit_vs_miss_divisor=10 / text_all_ms=100；会话端到端冒烟无阈值（:425-484）。**P0 本轮亲跑 6/6**【测试验证，baseline.md §1.2 G5】：打开 TTI 500 页中位 0.17ms / 首页渲染 8.21ms / 顺序渲染 p95 0.97ms / LRU 命中 5.41µs / 会话冒烟 / 文本抽取 26.48ms@500 页 | cache.rs LRU 逐出策略 / worker.rs 渲染路径 / engine_pdfium 引擎调用面回归（阈值随实测更新纪律 :7-9——改 THRESHOLDS 本身也需同步 gates.md） |
| Q6 | 安全姿态实装面（PDF-e 收口） | pdfe_security 11 锁（:5-34 锁清单）；security_scan.rs 对象图全扫 | 八族全检（OpenAction/AA/Launch/URI/EmbeddedFiles/RichMedia/JavaScript/Names-JS 树，含**不可达**敌意对象——全扫非可达性走查，security_scan.rs:13-16）；同 dict 同族合并去重 + 确定性报告；S6 表单探针（字段枚举+层级全名+现值+计数，**非危险族**——warnings 恒空）；加密检测边界（仍加密=检测面不可用旗标仍置位 / 曾加密=照常检测）；L7 输入字节 + L4 打开预算双入口；深度护栏 + 字段树环防 | security_scan 检测面 / DangerousFamily 集合（serde 冻结锁 + 斜杠形拒绝，pdfb_skeleton.rs:1030 附近）改变；S6 form_probe 字段形状改变 |
| Q7 | 看门狗换代 | pdfc_cache_worker 换代重放语义锁（:851 起）；worker.rs 模块契约 | 超时作业 Timeout 回执（结果永不采用，迟到回执机械丢弃）+ 工人毒化 + 替代工人（字节快照重开引擎）+ 在飞/排队作业按提交序重放 + 旧代断连弃置不 join；回执合同"每 id 至多一条"（worker.rs:40-44） | worker.rs 换代/重放/回执语义改变 |
| Q8 | 引擎装配与双路径一致 | pdfc_engine（:319 BGRA 缓冲长契约）；pdfd_writer::writer_without_engine_session_end_to_end；attach_engine 页数对账 | 引擎与写器独立装配；注入/引擎两路径同锁（语义零变化）；页数双权威不一致在装配面收口 typed 拒绝（pdf document.rs:233-246 AttachEngineError::PageCountMismatch 双侧数值携带） | engine.rs trait 形状 / 装配校验改变 |

**P7 评估的直接推论基线**【推断，依据 §4.1-4.2 图谱】：mdcore 侧任何复用改动（模块抽取/适配/新消费者），只有触及 D1（Source 镜像文件路径/词面）、D3（scheme 命名）、D6（workspace 版本单源）三个文件级耦合点时才会波及 mdc-pdf；Q1-Q8 的破坏条件全部落在 mdc-pdf crate 自有代码内——**复用 mdcore 模块本身不触碰任何 PDF 质量守护测试的被测对象**（唯一例外 = 改动 mdcore history.rs 的 Source 区段会触发 D1 镜像锁报警，这正是该锁的设计意图——防两向漂移）。

### 4.4 两模型对称/非对称面汇总（P7 分类参考，全部已验证事实）

| 面 | MD 面（mdcore） | PDF 面（mdc-pdf） | 耦合关系 |
|----|-----------------|------------------|----------|
| 溯源 Source | history.rs:24-32 五值（驱动合并窗/入栈判定） | command.rs:23-31 五值（serde 载荷，执行中被丢弃 document.rs:172-198） | 镜像（D1/D2）——唯一语义共享点 |
| undo 系统 | invert_op + 合并窗 + 双栈（P3 全链） | **无**（4.2 关键事实） | 无关系 |
| 变更事件 | Event 单队列 drain（P4 K6） | WorkerReply（渲染回执合同）+ mdc-pdf-event 协议族（草案） | 类型独立 |
| 时钟 | ClockFn 闭包注入 | set_clock_ms u64 快照 | 纪律同构、类型独立（D5） |
| 保存 | save_document 全量（md+meta） | flush_annotations 增量写（前缀逐位保持 Q1） | 类型独立 |
| 会话 | SessionManager 一窗一档上限 16（只装 Core） | PdfSession 打开后单形态；**无宿主接线**（D4） | 类型独立 |
| 全局状态 | ENTROPY_SEQ（熵源） | PDFIUM/BIND_LOCK（引擎绑定生命周期） | 各自独立（§2） |

## 5. 待基准项清单（若未来需要 MD 面性能验证的直接输入）

> 现状口径（§1.6）：无绝对毫秒门。运行口径注记：r23/r14 均为 debug 测试 profile（perf_1mb_smoke 自注"release 实测 ≈1/15" r23_fingerprint.rs:455——注释级）；pdfc_perf 面注记"pdfium 原生侧不受 profile 影响"（pdfc_perf.rs:54-55）。基准化时需统一 profile 口径并声明构建形态。

| # | 待基准项 | 现有最接近证据（钩子） | 空位 |
|---|----------|------------------------|------|
| B-1 | 大树频繁小事务（backup=tree.clone() 每事务成本） | r14_gates gate2 打点（100 单 op p50/p95/max + 100-op 批量 + undo + 小文档归因探针 r14_gates.rs:158-234） | 无断言无阈值 |
| B-2 | 100-op 批量事务 + undo 耗时（同 B-1 打点内） | gate2（:184-206） | 同上 |
| B-3 | sync_from_markdown 全量差分（含第二棵解析树与逐位置对照） | 无（sync_resync.rs 10 用例全功能性） | 完全空位 |
| B-4 | blocks_json 全量 dump / 分片（消息面大小 + 耗时） | r16_blocks_range 功能锁（无计时）；debug.rs:67 注释量级（~20 万块/10MB） | 无计时；量级为注释级 |
| B-5 | save_document 全量重算（tree_to_markdown + 整栈 history JSON 嵌入） | 无 | 完全空位 |
| B-6 | fingerprint / match_trees | **perf_1mb_smoke 已有**（<400ms debug 断言 + eprintln 分项） | 已有软门，可升级为绝对阈值门 |
| B-7 | snapshot to_json / from_json | gate3 打点（r14_gates.rs:59-82 println） | 无断言 |
| B-8 | replay_jsonl | r12 / r14 gate1 打点 | 无断言 |
| B-9 | undo 栈 / 事件积压的长会话内存形状 | mem_stats 观测面（lib.rs:201-210；wasm core_mem_stats mdcore-wasm lib.rs:118）——观测在位无门 | 无阈值 |
| B-10 | arena 槽位不缩的内存行为（反复增删场景） | mem_stats slots_capacity_bytes（tree.rs:371） | 同上 |

## 6. P6 验收对照（PLAN §4 P6）

| 验收项 | 结果 |
|--------|------|
| ① 大文档潜在瓶颈静态分析（ropey / arena / 遍历 / 序列化 / backup；MD 基准现状注脚） | ✅ §1（全部"潜在"+【推断】+零宣称；基准现状 §1.6——含简报校准 C1） |
| ② 全局可变状态清单全仓枚举（线程安全/重试性/多实例标注） | ✅ §2（3 实全局 + 假阳性排除 + tests 范围外注记） |
| ③ 复用限制面（单 Core / SessionManager / wasm 桥 / 消息大小频率） | ✅ §3（R1-R6 约束事实，零设计零选型） |
| ④ PDF 复用风险三项 | ✅ §4：依赖面图谱 D1-D6（镜像锁机制 + D2 缺口）/ 波及面三场景 + mdc-pdf 无 undo 关键事实 / 质量敏感点 Q1-Q8（守护测试 + 断言内容 + 破坏条件全锚点） |
| 报告零未经验证的性能宣称 | ✅ 全文量级判断标【推断】；数字仅两类 = P0 亲跑门禁（注明 baseline 出处）与在档断言阈值（注明测试文件），均不外推 |
| 待基准项清单单独成节 | ✅ §5（B-1..B-10 + 口径注记） |

## 7. 遗留未知 / 待查项

| # | 项 | 级别 | 待查路径 |
|---|----|------|----------|
| U1 | ropey 1.6 内部操作（remove/insert/slice/clone）的精确复杂度与常数——本轮只登记调用模式，未查 ropey 库实现 | 【未知】（调用模式【代码确认】） | 需量化时：查 ropey 1.6 文档/源码后仍须实测（本审计零宣称） |
| U2 | debug vs release 的 MD 面 profile 差（"release ≈1/15"为 r23 注释级注记） | 【文档记录：注释级】 | 基准化时统一 profile 口径（§5 注记） |
| U3 | "~20 万块对象/10MB"量级（debug.rs:67 设计注释，非实测） | 【文档记录：注释级】 | B-4 基准化时以实测替换 |
| U4 | mdcore 新增 Source 值的实际演进概率（D2 缺口的现实暴露面） | 【推断】（机制已确认，演进未发生） | 未来 mdcore 改 Source 时，镜像锁（pdfb_skeleton.rs:1040）是唯一机械提醒点——缺口（新增不报警）需人工纪律补位 |
| U5 | 增量保存"格式层就绪零落盘消费者"（P2 §5）与 B-5 的关系——JSONL append 设计在档未接线 | 【代码确认】（P2 复核沿用） | 若复用方需增量保存，P2 §5 三态判定是直接输入（本审计不动手） |
