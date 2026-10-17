# D1-a 适配层加固轮报告（yrs-d）

> **性质**：用户加固指令执行轮（d1d-t1）——「Continue the existing
> collab-adapter prototype without changing mdcore public APIs」口径下
> 的六项交付：可复现测试矩阵 / 敌意输入与不变量测试 / skip_gc 全编队
> 不变量强制面 / 重复同步撤销循环的 RSS+wire 测量 / 有界工作与无界
> 增长登记 / XmlFragment 载体选型级深验。**mdcore 全程零接触**（分析
> 仓 spike workspace 内）。**不宣称上游 UAF 已修复；不实现假撤销栈
> 上限**（B3.4 定谳维持）。
> **执行链**：yrs-a（评估）→ yrs-b（边界）→ d1c-t1..t4（原型四轮，
> yrs-c）→ 本轮（加固：测试资产化 + 硬化 + 深验）。
> **净室纪律**：全部数字本机实测可复跑（`cargo test -p collab-adapter`
> / `cargo run --release --bin p12..p14` / nightly ASan 口径在档）。

## 0. 交付总表

| 交付项（用户口径） | 形态 | 结果 |
|---|---|---|
| 可复现测试矩阵（8 类） | `adapter/tests/matrix.rs` 25 测试 | ✅ 全绿（含 ASan 口径 25/25 零报告） |
| 敌意输入 + 无变量测试 | `adapter/tests/hostile.rs` 13 测试 | ✅ 全绿（含 ASan 口径 13/13 零报告） |
| skip_gc 不变量强制面 | 构造期强制（B1.1 维持）+ **client_id 53 位 fail-closed 新增** + p14 盲区实证 + §2 强制面文档 | ✅ 可强制面全执行，不可强制面显式登记 |
| RSS + wire（重复同步/撤销循环） | `p12_sync_undo_cycles`（SCALE/CYCLES 可调） | ✅ 20 轮全收敛，RSS 恒定无增长 |
| 有界工作 + 无界增长登记 | 小限额会话边界测试 + §4 无界增长清单 | ✅ |
| XmlFragment 选型级深验 | `p13_xml_carrier_deep` 七面 | ✅ 全过（含 ASan） |

**适配层硬化变更（lib.rs，两处，均为加性/收紧面）**：
1. `open()` 构造期拒绝超 53 位 client_id（yrs `ClientID::new` 仅
   `debug_assert`——release 构建下超界值静默破坏 ID 编码；适配层补
   运行期显式拒绝，`InvalidInput` fail-closed）；
2. `import_anchor_sidecar()` 尾部冗余字节拒绝（此前静默忽略——敌意
   载荷可借尾部夹带；侧车为适配层自有格式，无兼容负担，直接收紧）。

公开 API 仍零 `yrs::` 类型（既有 22 fn + 8 字段面未扩——两处硬化
均为既有签名行为收紧，非新面）。

## 1. 测试矩阵（matrix.rs，25 测试）

| 章节 | 用户口径类 | 测试 | 钉定行为 |
|---|---|---|---|
| §1 | bulk loading | 4 | 三路径装载 + UTF-8 切分字符边界 + 限额 ==过/+1拒 边界 |
| §2 | incremental updates | 2 | edit_stream 四变体全序 + 不存在目标 InvalidInput + 失败后状态零变化 |
| §3 | cross-peer sync | 2 | 单向收敛（块+文本双面）+ 幂等复交换 |
| §4 | concurrent edits | 2 | 双端不相交并发收敛 + 同位置冲突插入收敛（双块在场、双端同序） |
| §5 | deletion | 2 | 删除+同名重加（新 CRDT ID，BlockId 撞号= R4.4 本地面登记）+ 删除撤销复原 |
| §6 | snapshot restoration | 2 | 快照→新会话逐块复原 + 维护窗 gc 后快照仍复原（B1.3×D1③ 同点） |
| §7 | undo/redo | 6 | origin 分栈独立 + 捕获窗内合并（三笔一栈项）/窗外分离（显式 sleep 1.2s 确定性）+ 一笔装载单栈项 + 效果全局（undo 差量传播对端收敛于回退态）+ redo |
| §8 | maintenance GC | 1 | 墓碑治理收缩 + 内容保真 + 双栈清零 + 二次幂等 |
| §9 | anchors & sidecar | 6 | 结构性编辑 ID 稳定（下标 5→7 复原）+ **删除悬空显式化 + undo 复活新 ID 语义** + 跨端解析 + 侧车字节稳定/导入/解析 |

## 2. 敌意输入与 skip_gc 不变量（hostile.rs，13 测试 + p14 实证）

### 2.1 敌意面测试结果

- **malformed**：空串/全零/伪随机/高字节 64B——Decode 拒或合法空
  更新 no-op（全零 = lib0 空更新形状，**已钉定为合法 no-op**，p9 c4
  口径一致）；真实 update **全前缀截断**（每前缀一试）两类安全处理
  之一、零 panic 零挂起、引擎未毒化（后续合法 apply 正常）；尾字节
  翻转同面。
- **oversized**：apply_update / load_bulk_text / AppendText /
  InsertBlock / load_blocks 五面小限额会话（256B）全拒 `Limit`；
  ==limit 过门。
- **invalid anchors**：bind 不存在块/越界下标/空文档 → InvalidInput；
  resolve 伪造 client/clock、真 client 越界 clock → AnchorDangling。
- **deleted-anchor**：删除后 resolve 永不静默返回邻居（R4.3 反例
  钉定）；邻居块独立绑定解析不受影响。
- **sidecar**：坏 magic/坏版本/逐长度截断/未知 tag/count 虚增/
  **尾部冗余（新收紧面）** 全拒；敌意面后 good 侧车仍可导入。

### 2.2 skip_gc 全编队不变量——强制面与盲区（p14 实证）

**四源同形状对照**（100 笔小串 + 删 [50,105) 区间）：

| 源 | wire | 公开 API 观测（insertions 双面/SV/DS/extends） |
|---|---|---|
| A skip_gc=true 干净源 | 538B | 基线 |
| B skip_gc=false 默认 GC | 483B | **与 A 完全一致** |
| C skip_gc=false 显式 gc | 483B | **与 A 完全一致** |
| D skip_gc=true 显式 gc（=适配层 maintenance_gc 引擎面） | 483B | **与 A 完全一致** |

三项定谳（全部一手实测，公开 API = insertions(true/false)、
state_vector、state_vector_lower、delete_set、extends）：

1. **wire 级 GC 证据存在**：GC 源的墓碑内容被替换为
   `ItemContent::Deleted` 标记（编码 BLOCK_ITEM_DELETED_REF_NUMBER）
   ——wire 恰缩小 55B = 被删内容字节数；干净源携带全内容墓碑。
2. **公开 API 不可见**：`Update.blocks` 为 `pub(crate)`；Deleted
   内容项解码期不带 deleted 标志 → `insertions(false)` 同样计入 →
   四源观测面全同。**上游 API 请求候选**：暴露 update 的内容类
   枚举或 `has_compact_deleted_content()` 辅助面。
3. **不可区分性（比盲区更强）**：case D 证明**适配层自身的
   maintenance_gc（B1.3 受批准维护窗）与未授权逐笔 GC 产出同族
   wire**——即使上游暴露内容类，也无法从 wire 区分两者。强制面
   因此只能是：**构造期（B1.1 无条件，已强制）+ 部署策略（全编队
   = collab-adapter）+ 快照门（P7b3：!skip_gc 文档不可产快照）**。
   ASan 门禁（B1.4）为内存安全兜底。

**中继传播（实证）**：skip_gc 接收端应用 GC 源 update 后自身再编码
= 502B（GC 证据族，非 557B 干净族）——**GC 证据经中继传播**；第三
端无法从链路中段区分原始 GC 源。敌意输入含义：消毒五面前置 +
产物级断言（r-o2-t3 发现⑤同口径）是唯一可靠检出层。

**GC 型对端内容级互通（hostile 测试）**：GC 源 update 应用 → 内容
收敛（存活/已删内容断言）+ 本端后续编辑/维护 gc/快照/快照恢复全部
健康——「incompatible skip_gc configurations」的可执行面钉定。

### 2.3 新发现：client_id 53 位域

yrs `ClientID::new` 的 53 位上界仅 `debug_assert`（block.rs:92）——
**release 构建下超界值静默进入 ID 编码**（`value | MASK` 位运算污染
高 11 位）。适配层现在构造期 fail-closed（§0 硬化 1）。上游改进
候选登记：debug_assert → release 安全校验。

## 3. 重复同步/撤销循环测量（p12，release，SCALE=2000/CYCLES=20）

每轮固定序列：5 笔编辑（Human）→ 同步 → undo → 同步 → redo → 同步。

- **RSS**：循环段恒定 7984→8244 kB（步进与分配器粒度一致），**20 轮
  零增长**；峰值 8244 kB @ 2000 块双端编队（进程级口径）。
- **wire**：全量 148,772B → 154,821B（+302B/轮 ≈ 净增 1 块 + 墓碑；
  编辑差量 197→258B 线性微增 = 递增计数串长，良性）。
- **维护窗**：B 端治理 2.0%（154,821→151,741B ≈ 20 个 -0 块墓碑）；
  **A 端（作者端）治理 0.0%——观察登记**：undo/redo 链下作者端
  墓碑被钉住（机制假设：redo 重插项的 origin 引用，未验证不宣称），
  对端可释放而作者端不可——实施轮墓碑治理预期需区分作者/对端。
- **撤销栈**：循环末 Human 栈深度 20（每轮 undo+redo 后 **1 项返回
  undo 栈**）——B3.4 无界面的具体形态：redo 不释放栈项，历史越长
  栈越深（即使全部已消耗过）。
- ASan 口径（nightly + `-Zsanitizer=address`）：全收敛复现，峰值
  65,392 kB（插桩 ~8×，预算内），零报告。

## 4. 无界增长清单（登记，全部为已知裁决面）

| 面 | 现状 | 治理路径 |
|---|---|---|
| 撤销栈（B3.4） | 无逐出 API（yrs 0.28 observer_* 私有）；**redo 返回栈项加剧**（§3） | 上游 API 请求 / 粗粒度 clear_all 阈值 / 无限维持——三选项呈 B3.4×R-3 协同裁决 |
| skip_gc 墓碑 | 维护窗前线性累积（作者端 undo/redo 链钉住面 §3） | maintenance_gc 维护窗（B1.3）；作者端钉住面实施轮细化 |
| anchor_table | 随语义锚绑定数无界增长（侧车 u32 count 域） | 调用方治理；实施轮可加绑定上限（B1.6 定参轮） |
| text 根内容 | 产品模型固有（文档大小） | 产品面 |

有界工作测试：小限额会话（256B）五路径边界 + apply_update 全前缀
截断零挂起 + 维护 gc 二次幂等（§1/§2）。

## 5. XmlFragment 选型级深验（p13，七面全过，含 ASan）

承 p11 六维对照（建议倾向 XmlFragment 呈裁决）→ 本轮单载体深验：

| 面 | 结果 | 锚点 |
|---|---|---|
| V1 serialization | 50 混合段全量 6,739B；快照→新文档 apply 结构**逐节点相等** | 确定性遍历签名 |
| V2 mixed content | [T,E,T] 三段交织读取；文本节点**中段插入分裂**（hea\|·MID·\|d-0）；元素与文本间插入新元素——序全保持 | element_sig |
| V3 stable IDs | 前插 3 前删 2 → 下标 10→11 **同 ID**；目标删除 → 悬空（**位置语义观测**：get_offset 仍解算塌缩位置，身份级悬空 = ID 等值检查——与适配层 resolve 两步口径一致） | sticky_index |
| V4 deletion | 父子树整删 → 子锚点悬空（结构缺席判定；get_offset=Some(0) 位置语义观测登记） | structure() |
| V5 restoration | 子树复原 = **结构等价重插，新 ID 族**（与适配层 matrix 发现同构：undo/重插不复活墓碑）；新旧子锚点 ID 比对不等 | 新旧 ID 比对 |
| V6 sync | 双端并发混合内容编辑 → 收敛 + 结构相等 | 两轮双向交换 |
| V7 wire | 深嵌套（ul>li>p>[t,em,t]×100）14769B ≈ 148B/li vs 浅形 135B/段——嵌套开销 ~10%（结构共享部分抵消） | — |

**结论（呈裁决，零终裁）**：XmlFragment 在本轮七面上**全部满足**
mdcore BlockTree 的协同载体需求（含行内混排 V2、ID 稳定性 V3、
删除/复原语义 V4/V5 与适配层已钉定语义同构）。D1 载体裁决的
BlockTree→载体完整映射矩阵仍属实施轮设计文档（yrs-c §6 遗留维持）。

## 6. 使用纪律复登（三实例，全部为本轮探针代码 bug 修复过程）

yrs-a 登记的 async_lock 非重入死锁纪律，本轮以三个实例复验强化：
**`commit()` 不释放事务 guard，只有 Drop 释放**——
1. 读事务 shadowing 仍活着的写事务（V2）；
2. `t2.commit()` 后未出作用域即开 `d2.transact()` 读（V6）；
3. 复合表达式跨文档链式事务临时值（V6 变体）。
正确形态 = **一事务一作用域**（独立块）。适配层 API 面天然满足（每
方法自开自释，调用方不可见事务）——隔离设计的又一实证。

## 6.5 e-review 独立审查与三缺陷修复（d1d-t2，同会话）

独立审查子代理（五项清单对抗审查，只读零改动）裁决 FIX-REQUIRED——
三 SHOULD-FIX 已全部修复并回归锁：

| # | 发现（审查复现证据） | 修复 | 回归锁 |
|---|---|---|---|
| 1 | `import_anchor_sidecar` 敌意 count 预分配放大：13B 侧车声明 u32::MAX 条目 → `HashMap::with_capacity` 试图分配 489GB → **SIGABRT（exit 134）**——fail-open，证伪本轮初稿"敌意面全拒"表述；且侧车路径完全无 B1.6 限额 | ①入口补 `Limit` 检查；②count 物理上界钳制（`count > (len-9)/17` 先拒——条目最小 17B） | hostile ⑤b（极端 count → SidecarFormat 不 abort）+ ⑤c（小限额会话超限侧车 → Limit） |
| 2 | `StreamEdit::InsertBlockBefore` **绕过 input_byte_limit**（六写入路径五查一漏：InsertBlock 有检查、Before 变体无）——256B 限额会话 257B 文本 Before 变体 Ok 放行 | Before 变体补 Limit 检查（before 查找键计入总量） | hostile §2 InsertBlockBefore 超限 → Limit |
| 3 | （office-g 文档面，anchor-t3 修）`DocxError::InvalidInput` 变体不存在（实际仅 Container/Parse/Io/Limit）——Phase 2 错误面设计失实 | office-g §3/§11 勘误：Phase 2 **新增** `DocxError::InvalidInput` 加性变体（公开 API 新增面显式登记） | 文档修正（无代码） |

修复后复验：38/38 全绿（含三新回归锁）+ ASan 38/38 零报告 + p9 复跑
全过。审查同时确认：公开面恰 22 fn + 8 字段零 yrs 类型（静态枚举）、
Cargo 零新依赖、mdcore 零接触、零进程泄漏、yrs-d 全部数字断言逐位
复现（RSS ±1% 环境方差）。NOTE 采纳四条：coalesce 测试理论 flake
（假钟注入 = 实施轮项登记）/ hostile 死代码已清 / regress.py 冷暖
页数一致性升级硬门（本轮同批收严）/ 环境方差口径维持。

## 7. 已核带源 / 待核清单

**已核带源（一手）**：
- 测试与探针全部源码：`adapter/tests/{matrix,hostile}.rs`、
  `src/bin/{p12,p13,p14}_*.rs`（SCALE/CYCLES 环境变量可复现）；
- yrs 0.28 API 面：`Update.blocks` pub(crate)（update.rs:164-168）/
  insertions 双面语义 / `ClientID::new` debug_assert（block.rs:88-96）/
  `ItemPtr::redo` redone 指针重插（block.rs:445+）/ async_lock
  RwLock 非重入（store.rs:19 + transact.rs 文档）；
- ASan 口径：nightly-2026-10-15 + `-Zsanitizer=address`，38 测试 +
  三探针零报告（detect_leaks=0——泄漏面不在 B1.4 验收口径）。

**待核（登记）**：
- 作者端墓碑钉住机制（§3 假设未验证）；
- 捕获窗测试的假钟注入（e-review NOTE——实现全确定性，实施轮）；
- 侧车敌意 count 上界公式 (len-9)/17 的边界用例（空串条目 0 长度
  已由 ⑤b/⑤c 覆盖，极端组合面维持登记）；
- GC 证据上游 API 请求的正式提交（§2.2-2 候选）；
- redone 链映射的锚点跟随（undo 复活新 ID 语义的适配层对策——
  事件面遗留项联动）；
- D1 实施轮三裁决（载体选型/B3.2 知情确认/B3.4×R-3）维持呈报。
