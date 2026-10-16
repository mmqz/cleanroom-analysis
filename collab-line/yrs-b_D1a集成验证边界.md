# D1-a 集成验证边界（yrs-b）

> **性质**：D1 裁决 a（yrs 为候选内核 + 隔离适配层 + 可撤回选型）后的边界
> 定义轮。用户已裁 D1② 走 (a)；本文档把 yrs-a 评估证据 + d1b-t1 探针证据
> 收敛为**四项集成验证边界**——即隔离适配层必须强制的不变量、禁止泄漏的
> 面、以及实施轮的验收门禁。**本文档不启动任何产品代码**（mdcore 零接触
> 维持）；结构载体选型（XmlFragment vs Array-of-Map）、协议部署面、D2/D3
> 均不在本文范围。
> **执行链**：yrs-a（评估 pass，`yrs-a_yrs评估pass.md`）→ d1b-t1（#676
> 定谳链闭环 + P7 探针，worklog 同 ID）→ 本轮（d1b-t2 边界文档）。
> **净室纪律**：只基于公开一手源（上游仓源码/issue API、spike 本机实测）；
> 全部数字与锚点可复核；推断与建议零漏标。

## 0. 执行摘要（四项边界一览）

| # | 边界 | 一句话规则 | 证据锚点 |
|---|------|-----------|----------|
| 一 | 并发 GC/UAF 风险隔离 | **skip_gc: true 全编队不变量**（单点构造强制）+ 显式压缩点纪律 + ASan 门禁验收 | d1b-t1 P7（#676 ASan 首发复现/混合证伪）+ 上游 issue #676（仍 open） |
| 二 | 事务粒度约束 | **批量装载禁止单事务小串流**（大块化或分事务，双零成本缓解实测）+ 死锁使用纪律（引用先取/事务后开/读写分语句） | yrs-a §4.5 发现 2（10k×8B→+415MB，比率 3.81）+ §4.8 |
| 三 | 撤销栈归属 | **撤销栈归适配层会话对象**（每会话 × 每 origin 一栈，栈本地不跨线）；压缩前必须 clear_all | yrs-a §4.4 P4 + d1b-t1 P7b4 + boundary-audit p3 |
| 四 | 稳定锚点跨模型映射 | **锚点本位币 = CRDT ID(client,clock)**；三模型语义锚（MD BlockId / Office 书签 / PDF Annot 引用）经适配层映射到 ID 空间，禁止跨模型直传语义锚 | yrs-a §6 行 4 + sticky_index.rs + boundary-audit p2 + r5-t2 §13 |

**总原则**（贯穿四项）：适配层是唯一接触 yrs 的地方——所有不变量在适配层
**构造期**强制（fail-fast），不依赖调用方纪律；可撤回性 = mdcore 公开模型
零 CRDT 类型渗透（撤除 yrs 时只拆适配层）。

## 1. 前提与范围

### 1.1 D1-a 裁决的内容与含义

用户裁决（2026-10-16 会话）：D1② 实现排序走 **(a) yrs 起步**，且限定为
**可撤回的技术选型**——yrs 为候选内核 + 隔离适配层，不立即升级为整个产品
的基础设施。工程含义三条：

1. **隔离适配层是裁决的组成部分**：不是"顺手包一层"，而是可撤回性的
   载体——适配层边界外的代码（mdcore 公开模型、三线消费方）对 yrs 的
   类型、错误、语义**零可见**。
2. **候选期验证义务**：进入实施轮前，已知风险面（#676 现行版 UAF、批量
   装载内存放大）必须有确定性验证手段与规避不变量——本文档四项边界
   即此义务的落实。
3. **撤回触发条件应显式化**（§7）：上游长期不修 #676 类内存安全面、
   协议面出现不可绕过的破坏性变更、或 (b) 净室自研成熟——任一成立时
   撤回评估重启。

### 1.2 本文档定义/不定义

**定义**：适配层必须强制的构造不变量；使用面纪律（进入编码规范与 lint）；
实施轮验收门禁（可执行、可复核）；撤回触发条件。

**不定义**：结构载体选型（XmlFragment vs Array-of-Map——实施轮首要设计题，
yrs-a §6 已登记两可行性）；本地 revision 序号语义再造方案；语义序列化
（blocks_json 类）映射方案；服务端组件选型（collab-a §4.3 JS 生态结论
未变）；D2 双态 / D3 生成态（等本边界确认后用户另行裁决）。

## 2. 边界一：并发 GC/UAF 风险隔离

### 2.1 事实链（全部一手）

| 步 | 事实 | 锚点 |
|----|------|------|
| 1 | #676 上报（2026-09-29）：双 peer 各持 UndoManager 并发 undo 嵌套 map 后同步，GC 在 commit 期释放 map，encode_diff 读已释放内存；**yrs 0.28.0 现行版**，feature sync + 默认 GC | GitHub issue #676（d1b-t1 复查：**仍 open、零评论、零修复**；ac33960 后仅 2 提交均 ywasm 面） |
| 2 | 本机 ASan 确定性复现：P7 探针逐字移植上报形状，nightly + `-Zsanitizer=address` 首发命中 heap-use-after-free（READ 8B/216B region） | d1b-t1 P7a |
| 3 | 缓解实证：双端 `skip_gc: true` 同形状 ASan 干净 + 收敛 | P7b1 |
| 4 | **混合部署证伪**：a=skip_gc/b=GC 方向 ASan 仍确定性 UAF（release 口径"收敛"是假阳性安全信号） | P7b2（MIXED=1 崩 / MIXED=2 本形状不崩——触发角色 = 接收远端删除+本地 undo 栈+GC 的 peer，**任何 peer 都可能进入该角色**） |
| 5 | 显式压缩 API 存在：`TransactionMut::gc(Option<&IdSet>)`——skip_gc 文档上仍可在受控点调用 | transaction.rs:1357 |
| 6 | 快照编码硬门：`encode_state_from_snapshot` 在 !skip_gc 时返回 `Err(Error::Gc)`——**skip_gc 与 D1③ 权威快照同向**，不是纯代价 | store.rs:152-157 |
| 7 | 上游自有测试（yjs#317/#343 移植）在 undo+嵌套 map 场景**自设 skip_gc: true** | undo.rs:1457/1665 |

### 2.2 边界规则

- **B1.1（全编队不变量）**：适配层构造 Doc 时**无条件** `skip_gc: true`
  （`DocOptions` 单点设置，调用方不可覆盖）。依据：事实 4——任何编队内
  默认 GC peer 都可能进入 #676 触发角色；"单端部署缓解"已被 ASan 证伪。
- **B1.2（单点构造）**：Doc 构造只经适配层唯一入口（构造函数/工厂），
  禁止业务代码直接 `Doc::new()`/`Doc::with_options()`——不变量强制点
  唯一化是隔离层的存在意义（lint 面：`disallowed_method` 补 `yrs::Doc::*`）。
- **B1.3（显式压缩点纪律）**：`txn.gc()` 只允许在两类受控点调用：
  (a) 会话撤销栈 `clear_all()` 之后（P7b4 实证：clear_all → gc(None) →
  编码/同步收敛）；(b) 权威端快照/checkpoint 边界（快照产出后的维护窗，
  与 D1③ 权威快照节奏合一）。**禁止**在收到远端 update 的同一事务内、
  或任何 UndoManager 存活期间调用。
- **B1.4（验收口径）**：#676 类内存安全面的验收**必须**走 ASan 门禁
  （nightly + `-Zsanitizer=address`，P7 探针形状入回归套件）——release
  收敛断言对该类面**零证实力**（P7b2 方法论发现：release 收敛而 ASan
  崩溃同形状成立）。
- **B1.5（上游修复跟踪）**：实施轮启动前 + 每次升级 yrs 版本前复查 #676
  状态（一手 API）；若上游修复并发布，B1.3 压缩点纪律仍维持（防御纵深，
  修复不改变 skip_gc 与快照面的同向收益），但可在实施轮重新评估 B1.1
  是否放宽为可选。
- **B1.6（隔离层外裹）**：yrs 调用点全部经适配层包装：panic 边界
  （`catch_unwind` + 错误三分，mdcore 消毒层既有模式）+ 输入限额
  （update 字节上限——对齐 SECURITY_POLICY v1.2 读面限额纪律）。#373/
  #445/#458/#518 陈年开放问题面（yrs-a §2）由此外裹兜底，不逐项对策。

### 2.3 代价登记（诚实面）

skip_gc 的删除块保留代价（P7c，20k 轮嵌套 map 插入+删除）：终态 RSS
**3.6×**（+26,536 vs +7,376 kB）、全量 update **2.34×**（817,893 vs
348,903 B）。缓解面：B1.3 受控压缩后 update 尺寸**精确回到 GC 基线**
（348,903 == 348,903 B）——wire 尺寸可完全治理；RSS 受分配器保持策略
影响不回落（+27,680 kB 口径登记），**RSS 降幅不可作为压缩收益指标，
update 尺寸可以**。结论：skip_gc + 周期压缩 = 快照能力与 wire 治理兼得，
RSS 峰值靠会话生命周期管理（关闭文档即释放）。

## 3. 边界二：事务粒度约束

### 3.1 事实链

yrs-a §4.5（发现 2，重量级）：单事务 N 笔 8B 小串装载，commit 期内存
二次放大——10k 笔 +415MB、20k 笔 +1.58GB（2× 规模 → 3.81× 内存 ≈ 4×
理论值）；机制锚点 = commit 第 7 步 block 合并路径（block_store.rs:222
squash_left 循环 → block.rs:2053 try_squash；String 臂 push_str 重组、
Any 臂 append(v2.clone())），算术核验 8B×10k²/2 = 400MB ≈ 实测 415MB；
100k 笔外推 >4GB（本机两次 OOM kill 实证）；上游 #445（2024-06 开放）
疑似同面。**双零成本缓解实测**：A2 单事务 N/1000 笔 8KB 大块（0.2ms/+0kB）
与 A3 分事务 N 笔小串（118ms/+0kB）。

另（yrs-a §4.8）：`Doc` 内部 async-lock RwLock 非重入——两类死锁形态
实测定谳（活跃写事务内 `get_or_insert_*`；同语句读写双锁）。

### 3.2 边界规则

- **B2.1（装载纪律）**：向文档批量写入 >1 笔内容时，**禁止**单事务逐笔
  小串流。两条合法路径：(a) **大块化**——内容拼装为大块（≥KB 级）后单笔
  入事务（AI 生成态典型形态，A2 实测零放大）；(b) **分事务**——每笔/每
  小批独立事务（交互编辑流，A3 实测零放大）。适配层提供两个装载 API
  （`load_bulk` 强制大块路径 / `edit_stream` 逐事务路径），**不暴露**裸
  长事务入口。
- **B2.2（AI 生成态直连含义）**：D3 生成态的批量写入（整节生成/整表生成）
  是发现 2 的直接命中面——生成态装载一律走 `load_bulk`（适配层内部完成
  分块与事务切分，调用方只见"一笔装载"语义）。
- **B2.3（事务边界 = op 语义边界）**：一个事务 = 一个 undo 栈项候选
  （P4 实证捕获窗 0 时逐事务成栈）+ 一个 origin 溯源单位。适配层事务
  切分规则必须同时满足：装载纪律（B2.1）、撤销粒度产品语义（B3.2）、
  origin 归类（Human/Ai）——三者同点决策，实施轮在适配层单点实现。
- **B2.4（死锁使用纪律）**：引用先取（独立语句）、事务后开、读写分语句
  ——适配层封装后业务代码不直接接触事务对象，纪律在适配层内部消化；
  适配层自身代码入编码规范 review 清单（yrs-a §4.8 两形态为反例库）。
- **B2.5（门禁形态）**：实施轮建装载模式回归（P5 三模式探针入套件：
  A1 形状在限额内**必须**被适配层 API 拒绝或自动切分，不允许透传）。

## 4. 边界三：撤销栈归属

### 4.1 事实链

- yrs 侧（yrs-a §4.4 P4）：`transact_mut_with(Origin)` 事务级溯源 +
  `UndoManager::include_origin` 分栈——human/ai 混合编辑后 undo 仅回退
  human 面、ai 面隔离；`capture_timeout_millis` 捕获窗与 mdcore 合并窗
  （1s）语义同构（默认值会合并连续事务，P4 置 0 逐事务成栈）。
- yrs 侧（d1b-t1 P7b4）：`clear_all()` 后显式 `gc(None)` 安全——撤销栈
  是 GC 安全的**引用持有方**之一。
- mdcore 侧（boundary-audit p3）：`history.rs` invert_op 显式求逆 + 1s
  合并窗（COALESCE_MS）+ 栈经 meta 持久化复原；**replay 不重建 undo 栈**
  （p2 已核）；X1/X2 结论：跨域自有 op 集不可移植（mdcore 自有 op 求逆
  与 yrs 内建 undo 的域不同）。
- 归属冲突面：若 mdcore History 与 yrs UndoManager 并存，双栈双求逆 =
  语义双轨（同一编辑两个"真相"）——必须单一归属。

### 4.2 归属规则（建议，终裁在实施轮设计文档）

**撤销栈归适配层会话对象**：每会话 × 每文档一组 UndoManager（Human/Ai
各一栈，include_origin 分栈），生命周期 = 会话（不持久化跨会话复原——
与 mdcore 现状 meta 复原面的差异显式登记为语义变更，实施轮呈用户确认）。
mdcore History 层角色 = **退役为兼容投影**（在协同文档上不再自持求逆栈，
本地单机文档维持现状——两模式并存的可撤回设计）。

依据：(a) P4 实证 origin 分栈 = mdcore Source::Human/Ai 溯源的直接对应物
（yrs-a §6 行 2 净收益面）；(b) undo 产生新 op 随 update 同步（栈本地、
效果全局）——与 mdcore 既有 sync_resync/revision 体系叠加无冲突
（collab-a §6.3 判据 2）；(c) X1/X2 域不可移植 → 协同域求逆必须用 yrs
内建（其 op 域即 CRDT 域），mdcore invert_op 的域是自有 op 集——两域
各自配套，**禁止交叉求逆**。

### 4.3 边界规则

- **B3.1（栈本地性）**：撤销栈是会话本地状态，**永不序列化进 update**
  （yrs 语义原生如此——适配层禁止任何"栈同步"发明）。远端看到的只是
  undo 产生的新 op。
- **B3.2（origin 语义映射）**：mdcore `Source::Human/Ai` ↔ yrs Origin
  字节串一一映射（适配层常量表）；未归类 origin 一律 Human（保守面：
  AI 误标 Human 的代价 = 可被 undo，反向的代价 = 用户编辑不可 undo——
  前者轻）。捕获窗对齐 mdcore 1s 合并窗语义（产品语义连续性）。
- **B3.3（压缩前置）**：任何 `txn.gc()` 调用点必须先 `clear_all()` 本
  会话全部 UndoManager（B1.3 的撤销侧面）；多会话共享文档时，压缩只在
  权威端维护窗执行（与 D1③ 单点整合合一——客户端不压缩）。
- **B3.4（栈上限）**：mdcore R-3（undo 栈无上限，产品决策挂起）在协同
  域同样成立——适配层预留栈容量上限参数（默认无限维持现状，产品裁决
  后一处启用）；yrs 侧 `StackItem` 无内建逐出，上限实现 = 适配层在
  observe_item_popped/栈长度上的包装（实施轮验证 API 面充分性）。
- **B3.5（门禁形态）**：P4 探针（分栈/隔离/redo）+ P7b4（clear_all →
  gc → 同步收敛）入实施轮回归套件。

## 5. 边界四：稳定锚点跨模型映射

### 5.1 事实链（三模型锚点系统现状）

| 线 | 锚点系统 | 稳定性事实 | 锚点 |
|----|---------|-----------|------|
| MD（mdcore） | `Anchor=(BlockId, offset)` 双轨寻址（ADR-B1：mutation 走 NodePath、意图/锚点走 BlockId） | **条件保真**：跨 save→open 仅在先序 1..n 无洞递增时一致；结构性编辑（中段删/move/中段插）后漂移；跨 snapshot/replay 可靠；document_id（UUIDv7）与 BlockId 两层身份互不派生 | boundary-audit p2（tree.rs:202-223/154-156 等） |
| Office（docx） | 书签对（`w:bookmarkStart/End`）+ 超链接锚（`w:anchor` 属性形）；读面书签定义静默丢弃、锚点存在性不可辨；写面引擎双能力在（探针⑤）但 DocxWriter 无包装（裁决三问挂起） | 书签名 = 文档内字符串命名空间，Word 语义层稳定；与内容位置无因果绑定（书签可指向被删内容 → 悬空） | r5-t2 CAPABILITY_NOTES §13（承载矩阵 14 行） |
| Office（xlsx） | 单元格坐标（Sheet!A1）+ 表名 | 坐标随行列插入漂移（引擎无引用重写面——xlsx-r1 C3 矩阵登记） | xlsx-r1 FIDELITY |
| PDF（mdc-pdf） | 内建 Annot 对象（页号 + 对象引用 + rect），**不自研锚定层**（PDF-c/d 裁决） | PDF 对象引用在增量写内稳定；页内位置 = rect 坐标（内容不动即稳） | md-line PDF-b/c/d（pdf_carrier_plan §标注模型） |
| CRDT（yrs） | `ID(client, clock)` 因果身份 + `StickyIndex` 位置锚（from_id/get_offset；Encode/Decode + Serde —— **可 wire 化可持久化**） | **ID 天然跨会话稳定（永不重编）**，强于 mdcore BlockId 条件保真；StickyIndex「不受文档变更影响」（前后关联语义） | yrs-a §6 行 4 + sticky_index.rs:45-155 |

### 5.2 映射规则

**R4.1（本位币规则）**：跨模型/跨线锚点的**唯一交换格式 = CRDT ID
(client, clock) 或其 StickyIndex 包装**。理由：(a) 三模型语义锚互相
不可解析（BlockId 不进 docx、书签名不进 mdcore、PDF 对象引用不进两者）
——直传语义锚 = 硬耦合，违反"禁止高耦合"项目铁律；(b) ID 是唯一在
协同域天然稳定且已可 wire 化的身份；(c) 可撤回性——撤 yrs 时映射表
随适配层拆除，语义锚系统各自无损。

**R4.2（入向映射：语义锚 → ID）**：各线在**自己模型内**产生语义锚
（用户意图层：MD 锚点点击、Office 书签跳转、PDF 标注定位），适配层
提供 `bind(semantic_anchor) -> crdt_anchor` 登记面：语义锚 + 当时解析
到的 ID/StickyIndex 成对入映射表（持久化为适配层自有侧车，不进三方
文档格式）。映射表是**会话/文档级资产**，与 D1③ 权威快照同点产出。

**R4.3（出向解析：ID → 语义锚）**：解析时 ID → StickyIndex.get_offset
→ 当前结构位置 → 各线语义锚重建（MD：结构位置 → BlockId 查注册表；
Office：书签定义存在性检查【r5-t2 读面"不可辨"缺口的消费侧对策——
悬空即显式警告不静默】；PDF：页/rect 直接可用）。**解析失败面显式化**
（目标块被并发删除 → 返回可诊断错误，三线统一语义）。

**R4.4（mdcore 特例登记）**：mdcore BlockId 在协同文档上的跨会话保真
**由映射表承接**（save→open 先序条件不再依赖——CRDT ID 是持久身份，
映射表把 BlockId 钉到 ID 上）。本地单机文档维持现状双轨（可撤回设计：
协同能力不渗透本地模式）。**G4b 撞号面**（reuse-impl R-2：重开 undo
物化 ID 与重解析块撞号）在协同域被本规则消解（ID 永不重编无撞号面），
本地域维持 R-2 现状锁。

**R4.5（Office 锚点写面依赖声明）**：R4.2/R4.3 的 Office 书签落点
依赖 r5-t2 裁决三问的结果（Option A RunSpec+anchor 扩面 vs 0 维持）
——若维持 0，Office 线入向映射退化为"读面书签名 + 位置快照"（弱绑定，
书签定义静默丢弃面继续由 C3 矩阵登记），出向解析仍可用（悬空警告面）。
**本边界不预设该裁决**。

### 5.3 验收门禁

- 探针面：StickyIndex wire 往返（Encode/Decode + 跨进程解析）+ 并发
  编辑下锚点保持（插入/删除邻居后 get_offset 语义断言）——实施轮入
  回归套件；
- 映射表面：三线各一入向/出向闭环样例（MD 锚点 / Office 书签 / PDF
  标注），含目标删除的可诊断失败面；
- 隔离面：映射表序列化格式版本化（适配层自有，不进三方格式）。

## 6. 适配层接口契约草案（隔离面定义，实施轮细化）

> 形状级草案——签名细节、错误类型、模块划分为实施轮设计文档的工作。

```
CollabAdapter（唯一 yrs 接触点；crate 边界 = 可撤回边界）
├── 构造面    open_document(options -> Doc)         // B1.1 skip_gc 强制、B1.2 单点
├── 装载面    load_bulk(items)                       // B2.1 大块路径（AI 生成态）
│             edit_stream(op)                        // B2.1 分事务路径（交互流）
│             // 两入口内部完成事务切分 + origin 标注（B2.3/B3.2）
├── 同步面    encode_diff(remote_sv) / apply_update(bytes)
│             // panic 隔离 + 输入限额（B1.6）
├── 撤销面    session_stacks(origin)                 // B3.1-B3.4 分栈/上限/压缩前置
├── 锚点面    bind(semantic) / resolve(crdt_anchor)  // R4.2/R4.3 映射表
├── 快照面    authoritative_snapshot()               // D1③ 单点编码 + B1.3 维护窗压缩
└── 事件面    subscribe(kind, handler)               // yrs-a §6 行 6 净收益面（A1/A3 补齐）
```

隔离不变量（验收可检）：(a) 适配层 crate 对外**零** `yrs::` 类型出现在
公开 API；(b) mdcore 公开模型零改动（撤回 = 删适配层 crate + 三线消费点
回归本地模式）；(c) 全部 B 规则的违反在适配层**构造期或调用期 fail-fast**
（不依赖 review 纪律）。

## 7. 实施轮启动条件与撤回触发

**启动条件**（全部满足才开实施轮）：
1. 本边界文档经用户确认（或按用户指示修订后确认）；
2. #676 修复状态复查（B1.5——已做：仍 open；实施轮启动日再复查一次）；
3. 结构载体选型设计题裁决（XmlFragment vs Array-of-Map，yrs-a §6 遗留）；
4. B3.2 撤销语义变更（meta 复原面差异）用户知情确认。

**撤回触发**（任一成立即启动撤回评估，(b) 净室自研重上桌面）：
1. #676 类内存安全面上游 6 个月无修复进展且新同类 issue 涌现（供应链
   健康度恶化——跟踪面：上游 issue 流）；
2. yrs 破坏性版本变更命中适配层无法吸收的面（协议常量/事务语义变更）；
3. 性能面：正式基准套件（yrs-a §8.1 遗留）实测低于产品需求且无缓解路径。

## 8. 遗留与边界（登记）

1. 本地 revision 序号语义再造（yrs-a §6 行 3）——SV 因果向量替代
   revision 角色的适配方案未设计（实施轮）；本边界只钉定"revision 不进
   跨端协议面"。
2. 语义序列化（blocks_json 类）映射（yrs-a §6 行 7）——同上，实施轮。
3. `ywasm`/web 桥面（yrs-a §8.3）——若 web 端需要直连 yrs 另立设计轮；
   当前消费面（web 三门禁）不依赖。
4. Rust 侧服务端组件未评估（yrs-a §8.4 维持）——D1③ 权威端实现选型
   （自建 Rust / Hocuspocus JS / 其他）是实施轮前置调查项。
5. P7c RSS 口径受分配器保持策略影响（d1b-t1 已登记）——正式基准套件
   应加 allocator 统计面（如 jemalloc stats 或 custom GlobalAlloc 计数）。

## 9. 已核带源 / 待核清单

**已核带源（一手）**：
- #676 状态与全文（GitHub API 2026-10-17 复查 + tool-results/issue676_full.txt）
- 上游提交流（commits API：ac33960 后 2 提交均 ywasm 面）
- skip_gc/gc()/encode_state_from_snapshot/上游测试自设 skip_gc/StickyIndex
  API（_upstream/yrs 本地克隆源码行号，见 §2.1/§5.1 表）
- P7 全部数字（本机 nightly ASan + release 双口径实测，探针源码
  `collab-line/spike/src/bin/p7_gc_undo_safety.rs`，MODE/MIXED/SCALE/CYCLES
  环境变量可复现）
- yrs-a §4.4/§4.5/§4.8/§6 与 boundary-audit p2/p3、r5-t2 §13、xlsx-r1
  FIDELITY、PDF-b/c/d 计划（本仓在档文档锚点）

**待核（登记）**：
- B3.4 栈上限包装的 API 面充分性（observe 事件族是否覆盖逐出实现所需——
  实施轮验证）
- §8 全部遗留项（实施轮/设计轮）
- B1.6 限额具体数值（对齐 SECURITY_POLICY v1.2 限额表口径——实施轮
  定参）
