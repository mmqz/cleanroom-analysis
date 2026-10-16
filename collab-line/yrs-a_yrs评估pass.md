# yrs 评估 pass（yrs-a）

> **性质**：L3 线后续轮（`pdf-line/06` §8.2 建议 1「yrs 评估 pass（最优先）」的执行轮）。
> **立项依据**：HANDOFF §6 ⑤「L3 分析线遗留裁决……yrs 评估 pass 最优先」+ 用户"继续"
> 指令（2026-10-16）。**本 pass 证据先行、零终裁**：产出直接供用户裁决 D1 问②
> （实现排序：(a) yrs 起步 → (b) 净室自研 → (c) changeset 特例），不对 (a)/(b)/(c)
> 作任何选择；零产品代码（mdcore 零接触，全程只读）。
> **执行链**：yrs-t0（环境恢复与基线）→ t1（上游一手核验）→ t2（协议与 API 深读）
> → t3（spike 六探针 + 闭包许可）→ t4（本报告收口）。2026-10-16 单日完成。
> **净室纪律**：只基于公开一手源（上游仓 LICENSE/源码/测试/发布元数据、crates.io、
> GitHub API、npm registry、AppFlowy 公开仓）；二手数据零引用。

## 0. 执行摘要（对 D1② 的证据输入一览）

| 维度 | 结论一句话 | 证据锚点 |
|------|-----------|----------|
| 许可 | MIT（LICENSE 直读），闭包 45 包全宽松零 copyleft | §1/§5 |
| 协议兼容 | **双语言 wire 互证实证**：yrs 0.28 ↔ yjs 13.6.33 双向精确重建 | §3/P3 |
| 成熟度 | 活跃（2026 年 138 提交/月度发版/413 万下载），但**开放问题面不可忽视**（现行版 UAF #676 等） | §2 |
| mdcore 映射 | 命令面/溯源面/身份面/undo 面四向映射可行，事件面（A1/A3 缺口）恰好原生补齐 | §6 |
| 关键风险 1 | **单事务批量小串装载 → commit 期内存二次放大**（10k×8B → +415MB；20k → +1.58GB）——有双零成本缓解面 | §4.5 |
| 关键风险 2 | 全量 update 编码字节非确定（状态收敛 ≠ 编码同一）——需权威快照单点编码 | §4.2 |
| 使用纪律 | 同语句双事务锁死锁（async-lock 非重入）——引用先取/事务后开/读写分语句 | §4.8 |

## 1. 对象锚定与许可

- **仓库**：`y-crdt/y-crdt`（GitHub API `repositories/299648293` 重定向定谳；`yjs/y-crdt`
  与旧名 `y-c-rd` 均为重定向源——克隆 URL `github.com/yjs/y-crdt` 实测有效）。评估
  基线 = 本地克隆 HEAD `ac33960`（2026-09-30，"Merge pull request #681"）。
- **crate**：`yrs` 0.28.0（crates.io 最新版，2026-09-17 发布；82 个历史版本）。
  工作区三成员：`yrs`（核心）/ `yffi`（C FFI）/ `ywasm`（wasm-JS 绑定）。
- **作者**：Kevin Jahns（**Yjs 原作者**）+ Bartosz Sypytkowski——Rust 端口由 Yjs
  作者共同署名，协议兼容性主张的出处强度高（`yrs/Cargo.toml` authors 字段）。
- **许可**：根 `LICENSE` 逐字直读 = 标准 MIT 全文，Copyright (c) 2020 两作者。
  勘误登记：GitHub API `license` 字段返回 `NOASSERTION`（API 自动分类失败，非许可
  异常；以文件直读为准）。`yrs/Cargo.toml` `license = "MIT"` 与 crates.io 元数据一致。
- **README 陈旧表述勘误**：README 称 "lib0 is a serialization library" 为项目组件，
  实际 lib0 已吸收进树内（`yrs/src/encoding/{varint,read,write,serde}`，工作区无
  lib0 成员、`yrs/Cargo.toml` 无 lib0 依赖）——供应链面更小，对采纳是正面事实。

## 2. 成熟度与供应链

**提交与发布节奏**（全量历史克隆，1237 提交，2021-08 窗口起点）：

| 年 | 提交数 |
|----|--------|
| 2021 | 117 |
| 2022 | 384 |
| 2023 | 232 |
| 2024 | 246 |
| 2025 | 120 |
| 2026（至 9 月） | **138**（近 90 天 43 提交） |

发布线（GitHub releases + crates.io 交叉）：v0.27.3（2026-07-13）/ v0.27.4
（2026-08-22）/ v0.28.0（2026-09-17）——2026 年月度节奏，无断档。

**采用证据（一手）**：
- crates.io：总下载 **4,131,530**；近期下载 2,122,625（API `recent_downloads` 字段）。
- **AppFlowy 生产采用实证**：AppFlowy-Cloud 主分支 `Cargo.lock` 含
  `yrs 0.23.5 (registry crates.io)`（raw.githubusercontent.com 直读）——其 collab
  crate 家族的底层引擎。
- 语言绑定家族 7 件（README 列表）：pycrdt / yrb / yr / ydotnet / yswift / ykt /
  ypy（deprecated）。README 底部含 AppFlowy 赞助徽标。
- 仓库元数据：2,179 stars / 153 open issues / watchers 24。

**开放问题面（重量级登记，均一手 API 核验）**：

| # | 状态 | 日期 | 标题与要点 |
|---|------|------|-----------|
| #676 | open | 2026-09-29 | **Heap use-after-free in encode_diff after concurrent undo and map remove**——yrs **0.28.0 现行版**，feature `sync` + 默认 GC 开；两 peer 各持 UndoManager 并发 undo 嵌套 map 后同步，GC 已释放的 map 被引用。**直接命中本 pass P4 探针路径（UndoManager）与 P2/P5 路径（encode_diff）的组合面。** |
| #373 | open | 2024-01-02 | Memory safety issues（heap corruption / segfaults）——多 peer mesh 状态预测场景；2024-07 后无更新（陈化）。 |
| #445 | open | 2024-06-07 | Memory leak?——循环插入场景内存增长；**疑似本 pass §4.5 批量装载发现的已知面**。 |
| #458 / #518 | open | 2024 | 内存占用 / 内存泄漏类报告。 |

采纳姿态含义（登记非裁决）：若走 (a) 路线，#676 类问题的工程对策 = 已知组合面
规避（undo 本地化策略 / GC 策略或 `skip_gc` 选项 / 权威端单点整合）+ 上游修复
跟踪 + mdcore 既有隔离层（panic 隔离/限额）外裹；#676 修复状态应在实施轮前复查。

## 3. Yjs 协议兼容性（源码锚点 + 双语言实证）

**源码级锚点**（`yrs/src/sync/protocol.rs`，feature `sync` 门控，核心 crate 内建）：

- 消息常量与 y-protocols 精确一致：`MSG_SYNC=0 / MSG_AWARENESS=1 / MSG_AUTH=2 /
  MSG_QUERY_AWARENESS=3`（protocol.rs:347-353）；`MSG_SYNC_STEP_1=0 /
  MSG_SYNC_STEP_2=1 / MSG_SYNC_UPDATE=2`（protocol.rs:429-433）——即 collab-a
  §4.2 采纳范围（SyncStep1/2/Update + SV + awareness）的 Rust 原生实现。
- awareness（`sync/awareness.rs`）：LWW 状态 + clock + on_update/on_change 观察者
  + local_state 泛型序列化——y-protocols awareness 的同构面。
- update 双格式：V1/V2 编解码（`updates/decoder.rs` DecoderV1/V2）+ `merge` /
  `merge_updates`（update.rs:272/648，服务端零实例合并 = collab-a §2.4 三件套的
  合并件）+ `encode_diff_v1/v2`（按 SV 求差量 = SyncStep2 载荷）。

**互操作实证（P3 探针，双语言闭环）**：
yrs 0.28（Rust）全量态 143B → 写文件 → **yjs 13.6.33（npm 当前版，JS）**
`Y.applyUpdate` 精确重建（text/map/array 三面断言过）→ JS 侧追加编辑 +
`Y.encodeStateAsUpdate` 回写 → Rust `apply_update` 精确重建（合并态文本
`"[js prefix] hello from rust yrs 0.28"` 逐字节匹配）+ 反向 SV 差量（SyncStep2
语义）尺寸对照成立。**这是比 README 主张（"binary protocol compatibility with
Yjs"）更强的实证级证据。**
佐证：上游 `tests-wasm/` 依赖 npm `lib0 ^0.2.42`（Yjs 原版 JS 编码库）做解码
验证；`tests-wasm/editing-traces.tests.js` 移植 Yjs 编辑轨迹基准套。

## 4. spike 六探针（`collab-line/spike/`，分析仓工作区零进 mdcore）

> 探针纪律：断言全过 = PASS；发现如实登记不粉饰；release 构建；SCALE 环境变量可调。

### 4.1 P1 基本面 ✓
四类型根（text/map/array/xmlFragment+XmlText）操作 + update v1/v2 双格式跨 doc
往返等值 + SV 差量语义（空 SV 差量 = 全量；自 SV 差量 ≈ 空）+ 子文档 guid 占位
（父文档 update 仅含引用，collab-a §1 subdoc 语义实证）。

### 4.2 P2 双端合并收敛 ✓ + **发现 1**
两 client 并发冲突（同位置 text 插入 / 同 map 键 / 同 array 槽）双向交换后：
状态收敛等值（text/map/array 三面）✓；**幂等复交换**（互灌对方全量后状态不变）✓；
第三方迟到者经 SV 差量同步同态 ✓；空差量回传幂等 ✓。

**发现 1（编码字节非确定性）**：状态收敛的两端，全量 update 编码字节可不同
（实测 121B vs 117B）——d1 侧 "base" 块因本端删除被拆分（4B 块），d2 侧整合
把 "base-from-1" 合并为单块（11B 块）。块布局是整合历史的函数。
**设计含义**：快照/持久化不能依赖对端编码同一性，应由单一整合者（权威端）产出
规范快照——**该发现直接支持 D1③ 建议「中心化权威快照 + 客户端 CRDT 离线自愈」
的架构方向**。

### 4.3 P3 互操作（§3，双语言 wire 兼容闭环）✓

### 4.4 P4 UndoManager origin 分栈 ✓
`transact_mut_with(Origin)` 事务级溯源标记 + `UndoManager::include_origin` 分栈：
human 三笔/ai 两笔混合编辑后，undo 仅回退 human 面（×3 逐笔），ai 面隔离不可
undo（第 4 次 undo 返回 false），redo 复原。**= mdcore `commit_transaction_as
(Source::Human/Ai)` 溯源 + 分栈 undo 语义的 yrs 对应物，映射可行性实证。**
`capture_timeout_millis` 捕获窗与 mdcore undo 合并窗（1s）语义同构（默认值会
合并连续事务，探针置 0 后逐事务成栈项）。

### 4.5 P5 装载模式对比 ✓ + **发现 2（重量级）**

三装载模式对比（SCALE=10000 / 20000 双档，release，VmHWM 增量口径）：

| 模式 | 10k 耗时 | 10k 内存 | 20k 耗时 | 20k 内存 |
|------|---------|---------|---------|---------|
| A1 单事务 N 笔 8B 小串 | 510 ms | **+415 MB** | 2,406 ms | **+1,581 MB** |
| A2 单事务 N/1000 笔 8KB 大块 | 0.2 ms | **+0 kB** | 0.7 ms | **+0 kB** |
| A3 分事务 N 笔 8B 小串 | 118 ms | **+0 kB** | 461 ms | **+0 kB** |

- **发现 2**：A1 内存随 N **二次增长**（2× 规模 → 3.81× 内存 ≈ 4× 理论值）；
  外推 100k 笔 >4GB——**实测两次被沙箱 OOM kill（4GB cgroup 上限）**。
  机制锚点 = 事务 commit 第 7 步 block 合并路径（`block_store.rs:222
  squash_left` 循环 → `block.rs:2053 try_squash`；String 臂 `push_str` 重组、
  Any 臂 `append(v2.clone())`）；算术核验：10k×8B 的逐对重组复制量
  8B×10k²/2 = 400MB ≈ 实测 415MB。上游 #445（2024-06 开放）疑似同面已知问题。
- **缓解面（实测零成本）**：A2 大块追加（8KB 块 = AI 生成态典型装载形态）与
  A3 分事务均零放大——装载纪律可完全规避；**该风险对 mdcore AI 生成态（批量
  写入）有直接工程含义，实施轮须立装载纪律门禁**。
- C 面（array 批量 10k 笔单事务）同样受影响且更重：全程峰值 1.49GB（A1 累计
  之上 +1.07GB）。
- D/E 面（通信路径健康）：10k 笔规模的全量 update 编码 211,520B / 0.8ms；远端
  apply 1.7ms；增量差量交换（22B）+ 双向合并收敛 0.03ms 级。

### 4.6 P6 官方编辑轨迹基准（上游 `edit_traces_tests`，release，本机沙箱口径）
5/5 全过（终态内容精确断言）：

| 轨迹 | 事务数 | 耗时 |
|------|--------|------|
| friendsforever_flat | 1,523 | 30 ms |
| sveltecomponent | — | 320 ms |
| rustcode | — | 1,633 ms |
| seph-blog1 | 137,154 | 21,277 ms |
| automerge-paper | 259,778 | 25,814 ms（≈99μs/键，逐键粒度事务） |

口径注：本机为共享沙箱（不作跨机器/跨实现宣称）；轨迹为逐键单操作事务 =
交互模式（A3 类），不触发发现 2；数值供纵向参考与后续实施轮基准对照基线。

### 4.7 闭包许可（§5 详）45 包全宽松零 copyleft ✓

### 4.8 API 使用纪律发现（死锁面）
`Doc` 内部为 async-lock RwLock（**非重入**）。两类实测死锁形态（探针开发期踩坑
定谳，均已规避）：
1. 活跃写事务内调用 `get_or_insert_*`（其内部再取写事务）→ 写写重入挂起；
2. 同一语句混合两把事务锁（如 `text.insert(&mut doc.transact_mut(),
   text.len(&doc.transact()), ...)` 参数求值期读写双锁）→ 挂起。
使用纪律：**引用先取（独立语句）、事务后开、读写分语句**——实施轮应入编码
规范与 lint 面。

## 5. 依赖闭包许可（spike Cargo.lock 实测闭包，cargo metadata 读注册表缓存清单）

45 包（不含本 spike）：**零 GPL/AGPL、零强制 copyleft**。

| 许可 | 数量 |
|------|------|
| MIT OR Apache-2.0（双许可） | 32 |
| Apache-2.0 OR MIT | 6 |
| MIT | 4 |
| Unlicense OR MIT | 1 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 1（unicode-ident——AND 条款，office 线 NOTICES v1.3 已有同例登记） |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 1（r-efi，getrandom 传递；OR 集内可选 MIT/Apache，LGPL 可避） |

运行时直接依赖（`yrs/Cargo.toml`）10 件全为知名宽松件：thiserror/fastrand/
smallstr/smallvec/async-lock/async-trait/arc-swap/serde/serde_json/dashmap。

## 6. mdcore op 映射分析（映射可行性登记；方案设计属 D1 裁决后实施轮）

> mdcore 侧输入 = boundary-audit p1-p6 锚点文档。**输入有效性钉定**：mdcore
> 核心 crate（crates/mdcore/src）自审计基线 03815b6 至当前 main 00ad88b **零
> 提交改动**（`git log 03815b6..HEAD -- crates/mdcore/src/` 空输出，EXIT=0），
> p1-p5 锚点对当前版本完全有效。

| mdcore 概念（boundary-audit 锚点） | yrs 对应物（本 pass 一手锚点） | 映射判定 |
|----------------------------------|------------------------------|----------|
| apply_ops 十原语唯一变异入口（lib.rs:387-618） | TransactionMut 上的类型方法（text.insert/remove_range；array/map/xml 同构） | 命令面同构；"事务内多 op + 单点提交"范式一致 |
| commit_transaction_as(Source::Human/Ai)（lib.rs:610） | transact_mut_with(Origin) + UndoManager include_origin 分栈（P4 实证） | **溯源面可直接映射**；origin 任意字节串，跨端随 update 保留 |
| revision 本地单调计数（revision.rs；coalesced 不推进 = p2 U2） | before_state/after_state（StateVector，transaction.rs:579/592） | 语义不同：SV 是因果向量（多端合并语义），协同场景**替代** revision 角色；本地序号语义需适配层再造 |
| BlockId 稳定块身份（条件保真：save→open 先序 1..n 时一致——p2 §3） | ID(client, clock) 因果身份 + StickyIndex 稳定位置锚（sticky_index.rs） | yrs 身份**天然跨会话稳定**（ID 永不重编），强于 mdcore 条件保真；语义 ID（可读）需映射层（id↔block 位置索引） |
| undo/redo 闭环：invert_op 显式求逆 + 1s 合并窗（op.rs:243、history.rs:91） | UndoManager 内建求逆 + capture_timeout_millis 窗（undo.rs） | 求逆内建 vs 显式——mdcore X1/X2 不可移植结论（跨域自有 op 集）在 yrs 侧由其内建 undo 覆盖文本域 |
| 事件面：take_events 拉取式单消费者（A1/A3 缺口：推送多订阅/块级监听缺失） | observers：doc 级 + 类型级（path+delta 事件）+ deep events（transaction.rs:420/observer.rs） | **yrs 原生推送式多订阅，恰好补齐 boundary-audit 登记的 A1/A3 缺口** |
| 序列化五面（md/blocks_json/snapshot/history/revision） | update v1/v2 + snapshot（Snapshot/restore_from_snapshot，doc.rs:2175）+ SV | 快照语义不同（yrs 快照=时间点视图）；blocks_json 类语义序列化需映射层 |
| 结构载体：BlockTree（块树 + attrs + 文本） | 四类型根：XmlFragment（层级树）/ Array+Map（块=Map{attrs, text}）/ Text | **结构映射方案（XmlFragment vs Array-of-Map）属设计决策，本 pass 只登记两可行性（P1 已证四类型根可用）** |

映射结论（登记）：**四向核心面（命令/溯源/身份/undo）映射可行且其中溯源与事件
两面为净收益**；两处需适配层再造（本地 revision 序号语义、语义序列化面）；结构
载体映射是实施轮首要设计题。

## 7. 对 D1② 的证据输入（非裁决）

- **(a) yrs 起步路线的实证支撑**：协议兼容双语言互证（§3）；Rust 原生与 mdcore
  同栈（零 FFI/零子进程）；MIT + 闭包零 copyleft（§1/§5）；活跃维护与大规模
  采用（§2）；溯源分栈与事件面净收益（§6）。
- **(a) 路线的风险清单（实施轮须对策）**：发现 2 批量装载内存放大（装载纪律
  门禁：大块化/分事务）；#676 现行版 UAF（undo+并发 GC 组合面规避 + 修复跟踪）；
  发现 1 编码非确定（权威端单点快照编码）；死锁使用纪律（§4.8）；#373/#445/#458
  陈年开放问题面（隔离层外裹姿态）。
- **(b) 净室自研路线的对照输入**：本 pass 的协议常量锚点（§3）、API 面盘点与
  六探针行为基线（§4）可直接作为自研规格参照；发现 2 同时是自研实现须规避的
  反例工程教训。
- **终裁在用户**：本 pass 不做 (a)/(b) 选择；D1①（是否统一 CRDT 侧）与 D1③
  （仲裁/离线取舍）不属本 pass 范围（06 §8.1 原文）。

## 8. 遗留与边界（登记）

1. 性能面为 sanity 级（P5/P6）；若 D1 裁决 (a) 并进入实施，须建正式基准套件
   （对齐 mdcore perf 门模式：阈值 + 每提交复跑）。
2. #676 修复状态跟踪：实施轮启动前复查上游（一手 API 可复核）。
3. `ywasm`/`yffi` 面未深读（评估范围外；web 桥若走 yrs 另立设计轮）。
4. 服务端 provider 生态（y-websocket/Hocuspocus 等）依 collab-a §4.3 的 JS 生态
   结论，本 pass 未重复核验；Rust 侧服务端组件未评估（登记待查）。
5. spike 探针源码在 `collab-line/spike/`（分析仓，循 office-line/spike 先例，
   Cargo.lock 钉版 + npm package-lock 钉 yjs 13.6.33）；上游克隆在
   `collab-line/_upstream/`（本地不入 git）。

## 9. 已核带源 / 待核清单

**已核带源（一手）**：
- LICENSE（根文件逐字直读）；yrs/Cargo.toml（版本/许可/依赖/作者）
- 提交节奏（全量克隆 git log 聚合）；发布线（GitHub releases API + crates.io）
- 采用（crates.io API 下载数；AppFlowy-Cloud Cargo.lock raw 直读）
- 开放问题（GitHub issues API 逐条：#373/#445/#458/#518/#676）
- 协议常量（sync/protocol.rs 源码行号）；lib0 入树（工作区成员 + Cargo.toml 反证）
- 全部探针数字（本机 release 实测，断言在探针源码内）
- 闭包许可（cargo metadata + 注册表缓存清单，45 包逐件）
- mdcore 核心零改动（git log 范围空输出）

**待核（登记）**：§8 五项遗留；r-efi 的 LGPL-2.1 分支未被采用（OR 集选 MIT）的
工程确认（实施轮 NOTICES 流程内做）。

## 附录：执行纪律对账

- mdcore 全程只读零改动零提交（本 pass 无产品面）。
- spike 依赖仅 `yrs = "0.28"`（Cargo.lock 钉 0.28.0）+ npm `yjs`（package-lock
  钉 13.6.33）——评估对象本尊，零引导性依赖。
- 探针开发期过程修正如实登记：trait 导入面校准（Text/Map/Array/GetString/
  ReadTxn/Encode/Decode/ToJson）、encode_update 归属 TransactionMut（改用
  encode_diff_v1(&SV::default()) 全量语义）、Any::Number 载荷（Number::Int）、
  UndoManager 默认捕获窗（置 0）、两处死锁修正（§4.8）、P2 字节级断言改为幂等
  复交换（发现 1 的断言演化）、P5 定长块算术修正与模式重构（发现 2 的定位过程）。
