# D1-a 适配层原型报告（yrs-c）

> **性质**：D1-a 适配层原型轮（d1c-t1..t4）收口报告。用户裁决口径
> （2026-10-17 会话）：**有条件确认边界**——允许可撤回的适配层原型，
> 不批准全面集成；验证三面 = 内存上限 / 撤销语义 / 跨模型锚点映射；
> 上游 UAF 未修复前 yrs 不升级生产默认内核。本报告把四轮证据收敛为
> 实施轮输入与待裁决项。**mdcore 全程零接触**（分析仓 spike workspace
> 内原型 crate，可撤回性 = 删 crate 即撤）。
> **执行链**：yrs-a（评估）→ d1b-t1/t2（#676 定谳 + 边界文档 yrs-b）
> → 用户有条件确认 → d1c-t1（构造/装载）→ d1c-t2（同步/撤销深验 +
> 压缩点）→ d1c-t3（锚点/快照）→ 本轮（载体证据 + 隔离终验 + ASan 门禁）。
> **净室纪律**：全部数字本机实测可复跑（探针源码 + 环境变量口径在档）。

## 0. 执行摘要

| 验证面 | 结论 | 证据锚点 |
|--------|------|---------|
| 内存上限 | **过**——适配层三装载路径全部远离放大区（release：+0.8/+1.8/+24MB @20k，对照直接 yrs A1 形状 10k 笔 +415MB）；限额 52MiB 双面生效 | p8 b1-b5（release + ASan 双口径） |
| 撤销语义 | **过**——origin 分栈（Human/Ai 互不消耗）+ 栈本地效果全局（undo 产生 op 随 update 同步收敛）+ 捕获窗 1s = COALESCE_MS 连续性（窗内合一项/窗外逐项）+ 一笔装载 = 单栈项 | p8 c / p9 a-b |
| 跨模型锚点映射 | **过**——CRDT ID 本位币：结构性编辑下 ID 稳定（下标漂移 42→44 语义复原）、删除面显式化（AnchorDangling 不静默返回邻居）、三模型入向（MD/Office 弱绑定/PDF 登记）、侧车版本化往返、跨端解析、StickyIndex wire 往返 | p10 全 |
| 隔离不变量 | **过**——公开 API 22 fn + 8 pub 字段零 yrs 类型（签名级静态检查）；crate 边界 = 可撤回边界 | d1c-t4 静态检查 |
| ASan 门禁（B1.4） | **过**——P7 复现链维持（默认 GC = UAF / skip_gc = 干净）；适配层 p8/p9/p10 三探针 ASan 构建全过**零报告** | d1c-t4 |
| 结构载体证据 | **已采集呈裁决**——XmlFragment 与 Array-of-Map 六维对照（见 §4） | p11 |

**适配层原型形态**：`collab-line/spike/adapter/`（crate `collab-adapter`，
workspace 成员，yrs 0.28 钉版与主 spike 共享 lock——lock 增量仅自身条目
8 行）。公开类型 9 个（AdapterError/EditOrigin/BlockSpec/SessionOptions/
StreamEdit/CrdtAnchor/SemanticAnchor/ResolvedAnchor/CollabAdapter）全部
本地类型；探针面 5 个 `__probe_*` 方法 `#[doc(hidden)]` 分离。

## 1. 用户三验证面证据明细

### 1.1 内存上限（d1c-t1 p8 + d1c-t4 ASan 复验）

| 路径 | 形状 | release ΔRSS | ASan ΔRSS（乘数 4 预算内） |
|------|------|-------------|--------------------------|
| edit_stream 20k×8B | A3 逐事务 | +1,828 kB / 476ms | +65,632 kB（插桩膨胀 ~36×，零 ASan 报告） |
| load_bulk_text 160KB | A2 8KiB 大块单事务 | +768 kB / 2ms | +3,404 kB |
| load_blocks 20k 块 | 512/批分事务 | +24,092 kB / 5.4s | +52,484 kB |

- 禁止形状（单事务 N 笔小串——yrs-a §4.5 发现 2 命中面，10k 笔
  +415MB / 100k 外推 >4GB）在适配层 **API 面不可表达**（无裸长事务
  入口——B2.1 编译期证明）；
- 限额（B1.6）：52MiB 占位口径（对齐 SECURITY_POLICY v1.2 声明总量
  族），load/apply 双面 `Err(Limit)`；`len == limit` 过门、`+1` 拒
  （p9 c4 边界钉定）；实施轮定参（yrs-b §8 遗留 6 维持）；
- ASan 口径方法论：预算断言按 release 校准 + `MEM_BUDGET_MULT` 仪器
  乘数；**B1.4 验收本体 = 零 ASan 报告**（内存安全），非预算面。

### 1.2 撤销语义（d1c-t1 p8c + d1c-t2 p9a/b）

- **origin 分栈**：mdc:human/mdc:ai 常量表一一映射（B3.2）；human
  undo 耗尽后 ai 栈深度不变、ai 面独立回退、redo 按面复原；
- **栈本地 + 效果全局**（B3.1）：peer1 undo 产生新 op，差量同步后
  peer2 收敛于回退态（栈不出进程，效果进 update）；
- **捕获窗 1000ms**：窗内三笔合并单栈项（一次 undo 全回退）；窗外
  （1.15s 间隔）逐项成栈（p9 b2 双面）——mdcore COALESCE_MS 语义
  连续性成立；
- **一笔装载**（B2.2）：load_blocks 2048 块（4 小批事务）= 1 栈项；
  单次 undo(Ai) 清空全部装载——AI 生成态"一笔装载"语义自然成立；
- **压缩点**（B1.3/B3.3）：`maintenance_gc()` 内置 clear_all 前置
  （不依赖调用方纪律）→ 2000 块删 1000 后全量 update 167,890B →
  93,976B（墓碑治理 44%，P7c 口径复现）+ 双栈清零 + 内容保真 +
  二次幂等；
- **B3.4 定谳（§9 待核项闭环）**：yrs 0.28 UndoManager 的
  observer_added/updated/popped/cleared 字段**全部私有**、无公开
  observe API、无栈逐出/容量 API；`undo_stack()/redo_stack()` getter
  公开（深度监控可行）。**栈上限不可经公开 API 实现逐出**——选项 =
  上游 API 请求 / 粗粒度 clear_all 阈值 / 无限维持现状，呈
  B3.4 × mdcore R-3 协同裁决（SessionOptions 刻意移除该参数，避免
  假承诺）。

### 1.3 跨模型锚点映射（d1c-t3 p10）

- **R4.1 本位币**：CrdtAnchor{client, clock}——三模型语义锚唯一交换
  格式；撤 yrs 时映射表随适配层拆除，语义锚系统各自无损；
- **R4.2 入向**：MdBlock（按 block_id 定位）/ OfficeBookmark（书签名
  + 位置快照——R4.5 维持现状公开 API 的弱绑定形态）/ PdfAnnotRef
  （页号+标签+位置——登记面）；
- **R4.3 出向 + 悬空显式化**：ID → StickyIndex.get_offset → 当前位置
  → 语义重建；目标删除 → `Err(AnchorDangling)`（墓碑塌缩检测 = 位置
  块首 ID 与锚点比对——不静默返回邻居）；
- **保持实证**：bind blk-00042@42 → 前插 5 + 前删 3 → resolve@44
  block_id 复原（对照 mdcore BlockId 跨 save→open 条件保真/结构性
  编辑漂移面——CRDT ID 消解该面，yrs-b R4.4 判断实证）；
- **弱绑定漂移诚实展示**：Office 书签 bind 位 50 → 漂移后解析到当前
  位 50 的 blk-00048（bind 时刻是 blk-00050）——身份漂移是 R4.5
  弱绑定的声明代价，Option A 扩面后升级强绑定；
- **侧车**：MDCA magic + 版本 1 + 确定性排序（字节稳定）+ 敌意三面
  全拒（坏 magic/坏版本/截断）；4 条往返全复原；
- **跨端**：A bind → 全量同步 → B resolve 同一 block_id（ID 跨进程
  稳定）；StickyIndex Encode→Decode→get_offset == 直接解算（yrs-b
  §5.3 验收第一项）。

### 1.4 快照面（D1③，d1c-t3 p10e）

- `authoritative_snapshot()`：skip_gc 与快照同向（P7b3 门）；快照 →
  新会话 apply → 101 块逐块一致；
- **B1.3 与 D1③ 同点**：快照 → maintenance_gc → 再快照（45,780B →
  25,476B）内容保真——维护窗压缩不打断快照产物线；
- **发现①（编码字节非确定）**：本形状未触发（收敛双端全量编码 40B
  == 40B 一致）——历史依赖面，**不外推为否定 yrs-a 发现**（该发现有
  独立证据在档）；单点编码设计不依赖本结果。

## 2. 边界规则 → API → 探针映射（yrs-b 对账表）

| yrs-b 规则 | 适配层 API 面 | 探针证据 |
|-----------|--------------|---------|
| B1.1 skip_gc 全编队 | open() 无条件 + SessionOptions 无该字段 | p8 a1/a2 |
| B1.2 单点构造 | open() 唯一 Doc 构造入口 | p8 a3（编译期事实） |
| B1.3 受控压缩点 | maintenance_gc() | p9 a1-a4 / p10 e2 |
| B1.4 ASan 验收 | （门禁口径非 API） | d1c-t4 三探针零报告 |
| B1.5 上游跟踪 | #676 复查仍 open/0 评论/0 修复（2026-10-17 API） | d1c-t1 前置核验 |
| B1.6 panic 隔离 + 限额 | apply_update（catch_unwind + 限额） | p8 d2 / p9 c1-c4 |
| B2.1 装载纪律 | load_bulk_text / load_blocks / edit_stream（无裸长事务） | p8 b1-b4 |
| B2.2 生成态大块路径 | load_*(origin, ...) 一笔装载 | p9 b3 |
| B2.3 事务 = op 语义边界 | origin 参数 + 捕获窗（三同点入参面） | p9 b2/b3 |
| B2.4 死锁纪律 | 适配层内部消化（引用先取/事务后开） | 全探针隐式（零死锁） |
| B2.5 装载模式回归 | 三路径探针入套件 | p8（SCALE 可调） |
| B3.1 栈本地 | 撤销面不序列化 | p9 b1 |
| B3.2 origin 映射 + 捕获窗 | EditOrigin 常量表 + 1000ms | p8 c / p9 b2 |
| B3.3 压缩前置 | maintenance_gc 内置 clear_all | p9 a2 |
| B3.4 栈上限 | **API 面定谳：不可公开实现**（参数已移除） | p9 d（源码核验） |
| B3.5 门禁形态 | P4/P7b4 形状经适配层复现 | p8 c / p9 a |
| R4.1 本位币 | CrdtAnchor | p10 全 |
| R4.2/R4.3 入向/出向 | bind/resolve/anchor_of | p10 a/b/d |
| R4.4 mdcore 特例 | MD 锚点跨会话保真由映射表承接 | p10 c1/d1 |
| R4.5 Office 依赖声明 | OfficeBookmark 弱绑定形态 | p10 b1 |
| §5.3 验收（wire 往返） | __probe_sticky_wire_roundtrip | p10 a2 |
| §6 契约草案 | 构造/装载/同步/撤销/锚点/快照六面落地（事件面未做） | — |

## 3. 隔离不变量终验（d1c-t4）

- **签名级静态检查**：公开 fn 22 个、pub 字段 8 个，**零** `yrs::`
  类型出现（解析器签名级 + 字段级双检）；探针面 5 个 `__probe_*`
  `#[doc(hidden)]` 与产品面分离；
- **crate 边界**：collab-adapter 是唯一 yrs 依赖方；主 spike crate 的
  d1c 系列探针仅经 collab_adapter API（p11 载体对照除外——评估探针
  先例，载体是适配层内部设计题）；
- **可撤回性**：撤 yrs = 删 adapter crate + 探针回归本地模式；mdcore
  零接触维持（四轮全部零提交）。

## 4. 结构载体证据（p11——yrs-b §7 启动条件③呈裁决）

同内容（1000 混合块）同 skip_gc 双载体六维对照：

| 维度 | Array-of-Map（interim） | XmlFragment | 差异 |
|------|------------------------|-------------|------|
| 全量 update | 120,573B | 128,927B | XML +7%（扁平内容） |
| 装载 1000 块 | 3.8ms / ΔRSS +2,904kB | 4.1ms / ΔRSS +1,580kB | 相当（XML RSS 略低） |
| 全量遍历 block_id | 2.35ms（索引 get 链） | **0.20ms（children 迭代器）** | **XML 快 ~12×** |
| 中段插入@500 / 删除 | 0.02ms / 一致 | 0.01ms / 一致 | 相当 |
| StickyIndex 锚点保持 | 500→497 复原 | 500→497 复原 | **双载体同支持** |
| 树性表达 | 扁平 depth 字段；行内混排退化为单文本字段（marks 需另设子结构） | **原生树**（children/parent/siblings API）+ **行内混排**（文本与元素子节点交织） | XML 表达力显著优 |
| 嵌套结构 wire（ul+2li） | 289B | **227B** | XML 反而更小（结构共享） |

**评估（呈用户裁决，零终裁）**：mdcore BlockTree 是树（块含子块 +
行内 runs）。XmlFragment 与之**近同构**（tag=kind、attributes=元数据、
children=子块+行内交织）且遍历快一个量级、嵌套 wire 更小；代价 = 扁平
内容 wire +7% 与 API 学习面。Array-of-Map 优点是形状直白、与 interim
原型连续。**建议倾向 XmlFragment**，理由是行内混排（marks/inline runs）
在 AoM 上无原生表达而 mdcore 公开模型有此面——但该建议待实施轮设计
文档做 BlockTree→载体完整映射矩阵后随裁决问题呈报。

## 5. ASan 门禁（B1.4——d1c-t4）

| 探针 | 构建 | 结果 |
|------|------|------|
| p7 MODE=asan（默认 GC） | nightly 1.101.0 + `-Zsanitizer=address` | **heap-use-after-free 复现**（exit 1）——#676 威胁模型维持 |
| p7 MODE=asan_mitigated（skip_gc） | 同上 | 干净（exit 0） |
| p8/p9/p10（适配层，B1.1 全 skip_gc） | 同上 | **三探针全过零 ASan 报告** |

结论：适配层路径在消毒构建下内存安全（B1.4 验收口径满足）；p8 预算
断言在 ASan 下以 `MEM_BUDGET_MULT=4` 运行（插桩膨胀口径登记，安全
检测不受影响）。

## 6. 实施轮建议与遗留

**已可进实施轮设计文档的面**：六面 API（构造/装载/同步/撤销/锚点/
快照）形状级契约经四轮探针校验；三验证面证据在档；B3.4 API 面定谳。

**呈用户裁决（实施轮前置）**：
1. **结构载体选型**（§4——建议倾向 XmlFragment，待完整映射矩阵）；
2. **B3.2 撤销语义变更知情确认**：适配层撤销栈 = 会话本地（不持久化
   跨会话复原）——与 mdcore 现状 meta 复原面的差异（yrs-b §4.2 登记
   语义变更）；
3. **B3.4 × R-3 协同**：栈上限三选项（上游请求/粗粒度 clear_all/无限
   维持）。

**遗留（实施轮工作面，yrs-b §8 维持 + 本轮新增）**：
- 本地 revision 序号语义再造（SV 因果向量替代方案）；
- 语义序列化（blocks_json 类）映射；
- 事件面（yrs-a §6 行 6——observe_changes API 未接入原型）；
- 服务端组件选型（D1③ 权威端：自建 Rust / Hocuspocus JS / 其他）；
- B1.6 限额定参（部署面口径）；
- **新增**：OfficeBookmark 强绑定依赖锚点写面裁决（r5-t2 §13 →
  office-g 最小扩展方案轮联动）；
- **新增**：正式基准套件应含 ASan 口径常跑项（B1.4 常态化）。

## 7. 已核带源 / 待核清单

**已核带源（一手）**：
- 全部探针数字：`collab-line/spike/src/bin/p8..p11`（SCALE/
  MEM_BUDGET_MULT/MODE 环境变量可复现；release + ASan 双口径）
- 适配层公开面清单与隔离检查：`adapter/src/lib.rs` + d1c-t4 静态检查
  输出（22 fn / 8 字段零 yrs）
- #676 状态：GitHub API 2026-10-17 复查（open/0 评论/0 修复；上游
  最新提交仍 ac33960 ywasm 面）
- UndoManager API 面：yrs 0.28.0 源码 undo.rs（observer_* 私有字段
  108-111 行 / undo_stack 公开 getter 486 行）
- StickyIndex/IndexedSequence/Xml API：yrs 0.28.0 源码 sticky_index.rs
  / xml.rs（行号在档）

**待核（登记）**：
- §6 全部遗留项；
- XmlFragment 建议的完整 BlockTree 映射矩阵（实施轮设计文档）；
- 大文档/并发基准（bench_limits 模式对齐——原型轮未做性能基准）；
- 服务端组件调查（yrs-a §8.4 维持）。
