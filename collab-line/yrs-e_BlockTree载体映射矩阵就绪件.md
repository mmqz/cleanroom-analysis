# BlockTree → XmlFragment 载体完整映射矩阵（就绪件）

> **性质**：D1 实施轮设计文档的**就绪预置件**（d1e-t3 先例——隔离
> 就绪 + 呈批，零 mdcore 接触，批次 1 G1① 裁决「XmlFragment」落定
> 后本件升格为实施轮设计文档基础；裁决改选 Array-of-Map 则本件转
> 对照参考，映射方法论可迁移、承载表重写）。
> **证据基线**：mdcore main `c4856f7`（BlockTree 模型面）+ 适配层
> 加固轮全套（yrs-d：matrix 25 / hostile 13 / anchor_undo_redo 6 /
> limit_boundaries 10，ASan 零报告）+ p13 XmlFragment 七面深验全过。
> **本文档不启动任何产品代码**；缺口行全部指向就绪探针 p15/p16/p17
> （本文档同批落档），实施轮验收门禁以 yrs-b §7 为准。

## 0. 结论速览

BlockTree 全模型面（9 kinds / 3 attrs 键 / marks 行内区间 / 寻址
双轨 / 7 ops）在 XmlFragment 载体上**存在完整、无死角的映射方案**：
结构面 9/9 kinds 有承载形态（§2 主表），行内面 text+marks 映射为
mixed content（§3，p13-V2 已验同构），操作面 7/7 ops 有事务形状
（§5），身份面双轨各有落位（§4）。**两条已知语义差异**（SetBlockType
的元素重命名子树截断、marks 非重叠规约）呈实施轮确认（§6）；
**零硬阻塞**。

## 1. 模型面盘点（映射输入侧全量）

### 1.1 BlockKind 九值三分类（mdcore tree.rs:21）

| 分类 | kind | 文本载荷 | 子结构 | attrs 键 |
|---|---|---|---|---|
| 虚根 | Document | — | 顶层块序列（children 唯一合法面） | — |
| 叶块（文本承载） | Paragraph | ✔ rope | — | — |
| | Heading | ✔ | — | `level`（1-6） |
| | CodeBlock | ✔（含换行） | — | `lang` |
| | ThematicBreak | 空 | — | — |
| | Table | ✔（**GFM 原文整块**——表非结构化行/列模型） | — | — |
| 容器块 | Blockquote | ✔（自有文本段） | 子块任意 | — |
| | ListItem | ✔（首行文本） | 嵌套子项（缩进即深度，markdown.rs:286-307） | `marker`（`-`/`{n}.`） |

**要点**：①Table 是**文本叶**不是行/列容器——映射面按叶块处理，
GFM 表原文进文本节点（协同面 = 整块文本合并语义，与 Paragraph 同
形，不发明单元格级结构）；②Blockquote 既有自有文本又有子块 =
mixed content 天然形状（p13-V2 已验 [T,E,T] 交织序保持）；③attrs
类型 = 开放 `BTreeMap<String,String>`（键面上表为已观测集，编码
方案须容忍未知键——§2 属性编码取开放面）。

### 1.2 行内面

- `text: Rope`（per-block，char 域偏移）；
- `marks: Vec<MarkSpan{start, end, kind}>`——kind = 开放字符串
  （md/em/strong/…），§8.1 第一阶段不要求独立 Stable ID；
- **规约现状**：mdcore 侧 marks 允许重叠存储（html.rs 渲染层消解），
  载体侧映射取**非重叠规约**（§3.2，差异面 D-2）。

### 1.3 寻址双轨（ADR-B1 混合式）

- `NodePath = Vec<usize>`：子索引链，mutation 入口（重放安全，
  事务内解析即用即弃）；
- `BlockId(u64)`：注册表索引（`id_index: BTreeMap<BlockId, NodeId>`），
  意图/锚点入口——结构变化（移动/前插/前删）后仍可解析，槽位
  永不压缩（`Vec<Option<Node>>` arena，remove 留墓碑槽）。

### 1.4 Op 七操作（op.rs:24）

InsertBlock{parent,index,kind,attrs,text,id?} / RemoveBlock{path} /
ReplaceText{path,start,end,insert} / SetBlockType{path,kind} /
SetAttrs{path,attrs} / SetMark{path,start,end,kind,add} /
MoveBlock{from,to_parent,to_index}。注意 InsertBlock.id：
None=现场分配 / Some=物化恢复（undo/redo/replay 路径）——载体侧
身份语义见 §4.3。

## 2. 结构映射主表（kind → XmlFragment 承载形态）

| BlockKind | 载体形态 | 属性编码 | 文本承载 | 子结构承载 | 证据/缺口 |
|---|---|---|---|---|---|
| Document | `XmlFragment` 根本尊（不设包装元素——顶层块即 fragment 直接子节点） | — | — | children → fragment 子节点序列 | p13-V1 全量序列化已验同构 ✅ |
| Paragraph | `<p>` 元素 | 开放键值 → XML 属性（§2.1） | 单文本子节点 | — | p13 混合段已验 ✅ |
| Heading | `<h>` 元素 | `level` → 属性 `level` | 单文本子节点 | — | p15-A ✅（本文档同批探针，下同） |
| CodeBlock | `<pre>` 元素 | `lang` → 属性 | 单文本子节点（**含换行原样**——载体文本节点多行合法，yrs XmlText 无换行禁约） | — | p15-A |
| ThematicBreak | `<hr>` 空元素 | — | — | — | p15-A |
| Table | `<tbl>` 元素（**语义标注元素**，非真 HTML 表——GFM 原文整块进文本） | — | 单文本子节点（GFM 原文） | — | p15-A |
| Blockquote | `<blockquote>` 元素 | — | 自有文本 = **首位文本子节点**；子块 = **元素子节点**——二者交织即 mixed content | children → 元素子节点（序保持） | p13-V2 [T,E,T] 交织序已验 ✅ |
| ListItem | `<li>` 元素 | `marker` → 属性 | 首行文本 = 首位文本子节点 | 嵌套子项 → 元素子节点（交织同 Blockquote） | p13-V7 深嵌套 ul>li>p 已验 ✅ |

### 2.1 属性编码通则（开放键面）

- 编码 = `BTreeMap<String,String>` → XML 属性直映（键合法化：XML
  Name 禁约字符集外的键以 `_xHH_` 转义——与 BlockTree 开放键面
  兼容；值侧 XML 属性转义由引擎承担）；
- **attrs 全量替换语义**（mdcore SetAttrs = 整 map 替换）→ 载体侧
  = 先删现存属性再写目标集（单事务内两步，对端收钦定终态——
  不依赖增量属性 diff 语义）→ p16-B 验证；
- 未知键零丢（开放面透传）——映射不发明白名单。

### 2.2 元素名命名域

块元素名集合 {p, h, pre, hr, tbl, blockquote, li} + 行内 mark 元素
（§3）**共用 XmlFragment 子节点命名域**；块/行内同名冲突面 = 空
（mark kind 见 §3.2 白名单转义规则）。实施轮落 crate `collab-adapter`
内部常量，公开 API 零泄漏（隔离不变量 (a) 维持）。

## 3. 行内面映射（text + marks → mixed content）

### 3.1 无 mark 文本（【实测修正】偏移域边界换算）

单文本子节点，`ReplaceText{start,end,insert}` → 载体文本节点
`insert(pos,…)/remove(range,…)`。**偏移域实测定谳（p16
sem_probe 实证）：yrs 0.28 `XmlText` 索引/长度 = UTF-8 字节域**
（"第一段".len()=15；insert(3) 落在第/一字节边界间；部分覆盖
删除不整块误删——yrs 内部分块正确处理）——非 Text(YText) 的
UTF-16 域非 char 域。BlockTree ReplaceText/SetMark = char 域 →
**适配层边界 char↔byte 无损换算**（ropey char→byte 原生支持；
实施轮边界工程项 D-3，非语义差异）。

### 3.2 mark 承载（包裹元素模型）

- `MarkSpan{start,end,kind}` → 包裹元素 `<m kind="…">`（kind 白名单
  常见集 em/strong/code/… + 未知 kind 转义为属性原值透传——不发明
  白名单拒绝）；
- **非重叠规约（D-2 差异面）**：载体侧 mark 元素 = 树形嵌套，重叠
  marks（mdcore 允许）映射时**拆分为最小区间等价覆盖**（如 [0,10)em
  ∩ [5,15)strong → em[0,5) + em∩strong[5,10) + strong[10,15) 嵌套
  形）；读回方向**保留拆分形态**（不还原重叠表示——读回 BlockTree
  侧 marks 集语义等价但区间切分可能不同，往返 ≠ 字面相等 = **语义
  相等**）→ p16-C 钉定语义相等断言（非字面）；
- **嵌套序规约**：同类嵌套按区间起点排序，同起点按长度降序（外层
  长）——确定性编码（读回遍历序确定，配合 §5 事务映射使 wire
  确定性可控）；
- mark 编辑下文本稳定：mark 边界推移 = 包裹元素内文本插入/删除，
  邻文本节点不受 ID 影响（p13-V3 同构）。

### 3.3 段内分裂形态（p13-V2 已验同构）

文本中段插入元素 → 文本节点分裂 hea|MID|d-0 三段——**载体固有
形态，读回时相邻文本节点合并**（适配层读回器职责），BlockTree
侧单 rope 语义不变。

## 4. 身份面映射（寻址双轨 → 载体两轴）

### 4.1 NodePath（事务内即用即弃）

path → 载体遍历序（fragment 根起逐 index 走子节点）——**fragile
坐标，仅事务内解析**（与 mdcore 语义同构：path 也是事务内解析即用）。
事务内 path→节点解析后立即转为 §4.2 身份轴执行操作。**不跨事务
持久化 path**（双方规约一致，零差异）。

### 4.2 BlockId（跨事务身份轴 = 侧车映射表）

- **承载**：`BlockId ↔ 载体节点身份` 侧车映射表——适配层既有
  `id_registry` 侧车（d1c-t3 形态）扩列 XmlFragment 节点列；
- **载体节点身份锚**：`StickyIndex`（p13-V3 已验：前插 3 前删 2 →
  下标 10→11 同 ID；目标删除 → 悬空身份 = ID 等值检查 + 位置语义
  get_offset 观测——与适配层 resolve 两步口径一致）；
- **解析两步**（yrs-b §4.1 已定形态）：BlockId → 侧车查 StickyIndex
  → 引擎解算现位置；悬空 = `AnchorDangling` 显式错误（不静默邻居）；
- **墓碑/复活**：RemoveBlock → 子树整删（p13-V4）；undo 复原 =
  结构等价重插**新 ID 族**（p13-V5 + 适配层 matrix 六面已钉）——
  侧车映射**不复活旧载体身份**，新 BlockId 分配（InsertBlock.id=
  Some 物化路径与载体新 ID 族**解耦**，见 4.3）。

### 4.3 InsertBlock.id 物化语义（身份双轨解耦点）

mdcore 物化路径（undo/redo/replay 回写原 BlockId）与载体身份
（StickyIndex 新族）**分属两轴**：适配层装载事务内——先载体建
节点得新 StickyIndex，再侧车登记 `恢复的 BlockId ↔ 新 StickyIndex`。
**效果**：BlockId 语义（mdcore 本地撤销域）与载体身份（协同域）
各自连续，互不污染；跨端 BlockId 不进 wire（yrs-b §5 已钉
「revision/BlockId 不进跨端协议面」——本映射维持该钉定）。

## 5. 操作面映射（7 ops → 载体事务形状）

| Op | 载体事务（单事务 = 原子） | 备注 |
|---|---|---|
| InsertBlock{parent,index,kind,attrs,text,id?} | parent 路径解析 → 目标父节点 index 位置 insert 建元素（+属性集+文本子节点）→ id=Some 时侧车登记恢复映射 / None 时分配新 BlockId 登记 | 复合装载与 B2.1(b) 分事务路径兼容（适配层既有三装载面） |
| RemoveBlock{path} | path 解析 → remove 子树（引擎墓碑）→ 侧车映射**不删**（悬空态，undo 复原重指新族） | p13-V4/V5 语义 |
| ReplaceText{path,start,end,insert} | path 解析 → 文本子节点 remove(range) + insert(start, insert) | char 域直传；空 insert = 纯删；空 range = 纯插 |
| SetBlockType{path,kind} | path 解析 → **元素重命名**（删旧元素建新元素搬运子树）——⚠️ 载体无原位改名 API，形状见 §6 D-1 | 语义差异面 D-1 |
| SetAttrs{path,attrs} | path 解析 → 清现存属性 → 写目标集（§2.1） | 终态钦定非增量 |
| SetMark{path,start,end,kind,add} | path 解析 → 文本区间拆分（§3.2 规约）→ add=包裹元素嵌套 / remove=解包裹（文本节点合并回填） | 最小区间等价覆盖算法在适配层 |
| MoveBlock{from,to_parent,to_index} | from 解析 → 子树 detach → to_parent 解析 → to_index 插入（**同事务内两步**——中途态不进 wire） | 与 mdcore move_subtree 原子性同构 |

**装载模式对齐**：B2.1(a) 大块单事务（load_bulk_text）/ B2.1(b)
小批分事务（load_blocks / edit_stream）三面不变——ops 序列在适配
层逐 op 转载体事务（B2.1(b)）或聚合单事务（B2.1(a)），**事务粒度
= 内存放大缓解既证结论复用**（yrs-a spike：单事务批量 commit 期
二次放大 → 分事务零成本）。

## 6. 语义差异清单（实施轮呈用户确认：两条语义差异 + 一条工程项）

| # | 差异 | 形状 | 建议 |
|---|---|---|---|
| D-1 | **SetBlockType 元素重命名子树截断** | 载体无原位改名 → 实施形状 = 删旧建新搬子树。**后果**：①子树节点载体身份全部换新族（StickyIndex 新族——但**侧车 BlockId 映射随子树搬移保留**，BlockId 语义连续 ✅）；②对端 wire 收到的是 删除+插入 对而非改名（收敛正确、字节面非最小） | 接受（收敛正确性优先；改名字节优化留上游 API 请求候选） |
| D-2 | **marks 重叠 → 非重叠拆分** | mdcore 允许重叠 marks，载体树形嵌套不表重叠 → 最小区间等价覆盖拆分；读回**语义等价但字面区间切分可不同** | 接受（往返断言取语义相等；字面还原仅当输入本就非重叠） |
| D-3 | **偏移域边界换算**（p16 实测发现，工程项非语义差异） | 载体 XmlText = UTF-8 字节域（实测 len/insert 均字节）；BlockTree = char 域 → 适配层边界无损换算 | 接受（纯边界工程，零语义损失；适配层单点换算不暴露字节域） |

（其余面零差异：寻址双轨解耦 §4.3、attrs 终态替换 §2.1、move 原子
§5 均为同构映射非语义变更。）

## 7. 证据锚点与缺口闭环表

| 映射行 | 已有证据（零新增动作） | 就绪探针（本文档同批） |
|---|---|---|
| 结构 9 kinds | p13-V1 / V7 + **p15-A1/A2 已验**（九 kind 精确形状装载读回全等 + wire 快照往返结构相等） | p15-B：邻居编辑稳定（**已验**：5 锚点前插3前删2 ID 全等值） |
| mixed content（Blockquote/ListItem 交织） | p13-V2 + **p15-C 已验**（子块前插/删除后序保持，自有文本首位恒定） | — |
| 属性开放面 | **p15-A3 已验**（未知键/转义敏感值/非 ASCII 零丢失）+ **p16-A 已验**（SetAttrs 终态替换非合并 + 身份域解耦） | — |
| 行内 mark 拆分 | **p16-B/C 已验**（重叠拆分读回语义相等并集还原 + 边界推移覆盖随文本扩张） | — |
| 偏移域换算 D-3 | — | p16 实测：XmlText 字节域定谳 + c2b 换算全环（**已验**——E1 中文 op5/op7 断言） |
| SetBlockType D-1 | **p16-D 已验**（删旧建新搬子树：结构等价 + block_id 连续 + 载体身份换新族三断言） | — |
| ops 7 条 | 适配层 matrix 25 + **p16-E 已验**（七 op 翻译器微缩全环：结构断言 + 对端 wire apply 收敛 + 增量二轮再收敛，含 own-text 偏移路径翻译） | — |
| 身份双轨 §4.3 | p13-V3/V4/V5 + anchor_undo_redo 6 + **p17-B/C 已验**（redo 复原 block_id 连续 + 载体换新族双轴解耦） | — |
| 撤销 B3.2 分栈 | p4/p9 + d1e-t1 六面回归 + **p17 已验**（per-origin 双栈互不干扰/LIFO/跨栈交叉/双 redo 全量复原——XmlFragment 载体与既有载体完全同构，G2① 载体侧风险清零） | — |

## 8. 裁决联动与升格路径

- **批次 1 G1① 裁「XmlFragment」（推荐默认）**：本件升格为实施轮
  设计文档基础，p15/p16/p17 探针结论回填 §7 证据列，实施轮开写
  BlockTree↔载体双向转换器（适配层 crate 内，公开 API 零泄漏）；
- **裁「Array-of-Map」**：本件降格为对照参考——§1 模型盘点/
  §5 操作映射/§6 差异清单方法论整体可迁移，§2/§3 承载表按
  Map 键位重写（p11 六维对照已有 Array-of-Map 行锚点；
  Blockquote/ListItem 的 mixed content 面需重验——p11 已登
  「锚点双保持失败」为该载体否决主因之一）；
- **批次 1 G2①（B3.2）/ G3①（栈上限）**：与本件正交——p17 就绪
  探针覆盖其载体侧复验，裁决后直接进实施轮冻结。

## 9. 遗留登记

1. **Table 单元格级协同**不在本映射面（GFM 文本叶定谳）——若未来
   BlockTree 升级结构化表模型（无此路线图），映射表重开；
2. **mark kind 白名单**：载体侧透传开放（不拒绝未知 kind），但
   **读回方**对未知 kind 的 BlockTree 落位（保留 MarkSpan 原样）
   为实施轮转换器行为规约——语义零丢失、表示零发明；
3. **wasm 桥面**（yrs-a §8.3）维持遗留——本件不影响其正交性。
