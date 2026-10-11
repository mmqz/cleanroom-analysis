# collab-a：Yjs 协议 pass（四问题定向）

> **净室分析报告**（线 A 跨线 P1 批次——协同内核是 MD/办公/画布三线共享的
> 结构决策）。日期：2026-10-11。
> 对象（双轨）：yjs/yjs **v13.6.33 @ `9ea3a4f`**（tag 浅取，生态稳定基线，
> 分析主线）与 **master v14.0.0-rc.28 @ `d01eefc`**（`@y/y` 重写线，方向
> 分析）。周边仓：y-protocols（master `73b2ff7` rc 线 + **v1.0.7 @ `85f5f89`**
> v13 兼容线）、y-websocket（`f786ae9`，main=@y/websocket 4.0.0-rc.4）、
> y-websocket-server、y-leveldb（`7207eb9`）、ueberdosis/hocuspocus（v4）。
> 许可：yjs / y-protocols / y-websocket / y-websocket-server / y-leveldb /
> Hocuspocus / y-webrtc / y-sweet = **MIT**（逐仓 LICENSE 已核；规划期
> "Yjs MIT 待核"定谳）；**y/hub = AGPL-3.0 OR 专有双许可**（一手 README，
> §4.3）。
> 方法：主代理单 pass 直读（v13 全量 src ~11k 行通读 + v14 结构对照 +
> tests 抽读 + provider 仓定向读 + 官方文档 docs.yjs.dev 两页一手抓取）。
> 锚点约定：`src/...` 相对各自仓根；v13 锚点记于 `yjs-v13/`（= tag
> v13.6.33 worktree），v14 锚点记于 `yjs/`；y-protocols v13 兼容线锚点记于
> `y-protocols-v1007/`，rc 线记于 `y-protocols/`。【推断】= 分析员判断。
> 定位：协议 pass 而非 11 维深读——回答规划设定的四问题（YATA 工程化 /
> 分片与冷热 / 离线合并 / provider 生态），为工作台「三条线统一协同内核」
> 决策提供 CRDT 侧一手输入。
>
> 一句话定性：**协议冻结、内核重写中的生态型 CRDT——update 是可交换可
> 幂等的自描述字节流（structs+deleteSet 两段式），awareness 是带 30s
> 过期 + 15s 自续约的每客户端 LWW 广播，subdoc 是「父文档只存 guid 引用」的分片单位；
> 同步协议三消息（SV→diff→update）十年未变，而内核正从 v13 的
> Item/类型树走向 v14 的统一 YNode+schema+delta 重写。**

## 0. 双轨现状：分析对象选择说明（重要生态事实）

- [事实] yjs master 已是 **`@y/y` v14.0.0-rc.28**：包更名（`package.json:2-3`），
  共享类型不再按 YMap/YArray/YText/YXml 分类，统一为 **YNode + lib0 schema**
  （`src/ynode.js:1-40`、`src/utils/schemas.js:3`），新增 delta 渲染与
  attribution（`src/utils/Renderer.js`、`src/utils/delta-helpers.js`）。
- [事实] y-websocket README 明言："This README primarily documents the
  currently stable `y-websocket` release and its usage with `yjs` v13. The
  `main` branch … is the development branch for the unstable `@y/websocket`
  release… Most users should continue to use the stable `y-websocket` package
  with Yjs v13 for now"（`yjs/y-websocket README.md:20-24`）。Hocuspocus/
  BlockSuite/AFFiNE 等生态全部运行在 v13 线（§4.5）。
- [事实] 同步协议本体跨代冻结：y-protocols v1.0.7（imports `yjs`，dep
  `yjs: ^13.5.6`）与 master rc（imports `@y/y`）的 `sync.js` **逐字节仅差
  import 行**（本 pass diff 实证）；新增 PROTOCOL.md 规范化文档
  （`y-protocols/PROTOCOL.md`，RFC2119 口径）。
- [事实] v14 仍承诺 v13 update 的解码兼容：compatibility tests 以
  "The v1 documents were generated with Yjs v13.2.0" 的冻结 base64 语料
  回归（`yjs/tests/compatibility.tests.js:5,23-46`）。
- 【推断】选型视角的真实风险不是"Yjs 消失"而是"生态长期停在 v13 稳定线、
  v14 rc 缓慢收口"——协议面（本 pass 对象）与生态绑定最深、最稳；内核
  实现面（若直接依赖库本体）则处于两代之间。对 mdcore 的可移植结论：
  **抄协议与语义（稳定），不绑实现（换代中）**。
- 本 pass 以下正文以 **v13.6.33** 为主证据（生态基线），v14 差异单列登记。

## 1. Q1 YATA 的工程化：update / awareness / subdoc 三件套

### 1.1 分层全景：内核不碰网络，协议三件套各自独立

- [事实] 内核零网络代码：yjs src 内无任何网络代码（网络 API 引用零命中；
  JSDoc 中的 `https://` 为纯文档链接非网络调用）；
  README 明言 "Network functionality and two-way-bindings are implemented
  in separate modules"（`yjs-v13/README.md:151-153`）。内核只暴露**三个
  可交换的数据面**：update（持久化与同步的单位，`Uint8Array`）、
  state vector（状态摘要，`Uint8Array`）、事件流（`update` 事件增量广播）。
- [事实] **update**：`Y.applyUpdate(doc, update, origin)` /
  `Y.encodeStateAsUpdate(doc, targetSV?)` / `Y.encodeStateVector(doc)` /
  `Y.mergeUpdates([...])` / `Y.diffUpdate(update, sv)`
  （`yjs-v13/src/utils/encoding.js:476-555`、`src/utils/updates.js:186-507`）。
- [事实] **awareness** 不在内核：实现在 y-protocols（`y-protocols-v1007/
  awareness.js`），与 doc 生命周期解耦但借用 `doc.clientID` 标识身份
  （`awareness.js:44-54`）。
- [事实] **subdoc**：内核内建（`Doc._item` + `ContentDoc`），但**内容永不
  随父文档传输**——父文档 update 里只有一个 guid 字符串 + opts
  （§2.1）。

### 1.2 update 的编码格式：structs + deleteSet 两段式

- [事实] 一个 update = **struct 列表 + deleteSet** 两段（`writeStateAsUpdate`
  `encoding.js:504-507`）。struct 段按 client 分组：`[组数][每组: struct 数,
  client, 起始 clock, structs...]`（`writeStructs`/`writeClientsStructs`
  `encoding.js:57-102`）；组按 **client id 降序**写（"Write items with
  higher client ids first. This heavily improves the conflict algorithm"
  `encoding.js:97-99`——编码顺序参与整合性能的工程取舍）。
- [事实] deleteSet 段 = 每 client 的 `[clock,len]` RLE 区间序列，写序确定
  （client 降序，`src/utils/DeleteSet.js:222-244`）；**删除不占 clock、
  不进 struct 段**（INTERNALS.md "the clientID's clock is not incremented
  by deletes"，`yjs-v13/INTERNALS.md:104-111`）。
- [事实] 基元编码：lib0 `varUint`（LEB128 变长，53 位无符号）、
  `varBuffer`、`varString`（PROTOCOL.md §2 原语表，
  `y-protocols/PROTOCOL.md:18-35`）。
- [事实] **V1/V2 双格式**：V1 = 单一 varUint 流（`UpdateEncoderV1`
  `src/utils/UpdateEncoder.js:36-125`）；V2 = **列式 9 缓冲 + rest**：
  keyClock(IntDiffOptRle)/client(UintOptRle)/leftClock/rightClock(IntDiffOptRle)/
  info(Rle)/string/parentInfo(Rle)/typeRef(UintOptRle)/len(UintOptRle)
  （`UpdateEncoder.js:162-202`），头部留 feature flag 字节
  （`UpdateEncoder.js:189`）；两者可互转（`convertUpdateFormatV1ToV2/V2ToV1`
  `updates.js:717-722`），默认事件与 provider 仍用 V1（README "Using V2
  update format"，`yjs-v13/README.md:1026-1035`）。
- [事实] Item 的 info 字节：**低 5 位 = content 类型**（0=GC，1=Deleted，
  2=JSON，3=Binary，4=String，5=Embed，6=Format，7=Type——`encoding.js:5-15`
  头注 + `readClientsStructRefs` switch `encoding.js:130-193`；解码端
  contentRefs 表 1-9 加 9=Doc，`src/structs/Item.js:712-724`）；
  **BIT8=有 origin（左锚）、BIT7=有 rightOrigin（右锚）、BIT6=有
  parentSub**（Item.write `Item.js:659-662`）。即**并发上下文内联在
  struct 头里**，单条 struct 自描述、无需外部状态即可在其他端重放。
- [事实] key 字典串缓存（writeKey Map + keyClock）为**半成品**：功能
  关闭，注释自陈 "this feature here is basically useless as it is not
  being used"（`UpdateEncoder.js:297-319`）——诚实登记的编码优化残留。

### 1.3 结构系统：Item / GC / Skip 三 struct + 九种 content

- [事实] 每个插入物 = 一个 `Item`，字段：`id(client,clock)`、`origin`/
  `rightOrigin`（**插入时刻**的左右邻居 ID——YATA 冲突证据）、`left`/
  `right`（**当前**邻居，随整合漂移）、`parent`/`parentSub`、`content`
  （`src/structs/Item.js:250-313`）；info 位字段：keep/countable/deleted/
  marker（`Item.js:305-363`）。
- [事实] 存储双结构（INTERNALS.md "Item Storage"）：**文档序**双链表树
  （parent/_start/_map）+ **插入序** StructStore（`Map<client, structs[]>`
  按时钟连续数组，二分查找；`src/utils/StructStore.js:10-25`）。
- [事实] `GC` = 仅存 id+length 的墓碑（`src/structs/GC.js:12`）；删除
  item 的 content 被替换为 `ContentDeleted(len)`（`Item.gc` `Item.js:634-644`）
  ——**内容擦除但 ID 序保留**；`Skip` = 传输流中的"此处缺一段"占位
  （structSkipRefNumber=10，`src/structs/Skip.js:5-12`）。
- [事实] clientID：v13 = `random.uint32`（`src/utils/Doc.js:23`）；
  v14 升为 `random.uint53`（`yjs/src/utils/Transaction.js:18`）。冲突
  防御：若远端事务携带自己的 clientID，**本地立即换新 ID**
  （`src/utils/Transaction.js:360-362`，日志 "Changed the client-id
  because another client seems to be using it"）。

### 1.4 YATA 整合算法本体（Item.integrate）

- [事实] 并发消解在 `Item.integrate` 的 while 循环（`Item.js:419-488`），
  从冲突候选 `o` 出发：**case 1**——同 origin（`compareIDs(this.origin,
  o.origin)`）：若 `o.id.client < this.id.client` 则 `left = o` 继续
  右移（**clientID 小者排左**），否则若 rightOrigin 也相同则按 ID 定序
  break（`Item.js:466-475`）；**case 2**——`o` 的 origin 在
  itemsBeforeOrigin 中（传递冲突）：`left = o`（`Item.js:476-481`）；
  其余 break。整个冲突消解 = **"同锚比 clientID、传递锚继续右移"两条
  规则 + 链表重接**（`Item.js:489-516`）。
- [事实] 整合前置检查 `getMissing`：origin/rightOrigin/parent 三引用
  若指向缺失 client 状态 → 返回缺失 client（由 integrateStructs 栈调度
  延后，`Item.js:372-413`）；**因果缺失 ≠ 失败，而是待处理**
  （§2.3）。
- [事实] 远端 struct 流整合 `integrateStructs`：栈式调度——遇缺失依赖
  就把该 client 的 reader 压栈先处理，环不可能（因果偏序），最大栈深
  = reader 数（`encoding.js:200-359` docstring + 实现）。
- [事实] YATA 论文外的工程增强：`rightOrigin`（"The most notable is that
  items have an `originRight` as well as an `origin` property, which
  improves performance when many concurrent inserts happen after the same
  character"，`yjs-v13/INTERNALS.md:10-13`）——**右锚是 Yjs 对 YATA 的
  私有改进**，把并发插入的回退扫描从 O(冲突数) 收敛到同锚即断。
- [事实] v14 README 新增**形式化验证**条目：lean-yjs 用 Lean 证明 YATA
  的保持性与交换性，且"论文伪代码含错误"（`yjs/README.md:1439-1446`）。

### 1.5 结构共享类型的实现边界：类型是 Item 链的薄视图

- [事实] `AbstractType` 本体只有 `_map`（Map<key,Item>）/`_start`（链头）/
  `_length` + 事件处理器（`src/types/AbstractType.js:261-291`）——**类型
  不存数据，只是对子 Item 链的命名视图**；嵌套类型 = ContentType 里的
  Item（`Item.js:712-724` ref 7）。
- [事实] **一切皆列表**（INTERNALS.md "At its heart, Yjs is a list CRDT"）：
  Array=任意 content 列表、Text=字符+Format 标记+Embed 列表、**Map=
  键的条目链，"最后插入的条目生效，其余标记删除"**（`yjs-v13/INTERNALS.md:
  15-25`）。
- [事实] `YMap.set` 每次**追加新 Item 到该 key 的链尾**（left=旧值 item，
  origin=left.lastId），值类型分派：标量→ContentAny、Uint8Array→
  ContentBinary、Doc→ContentDoc、AbstractType→ContentType、**其余抛
  "Unexpected content type"**（`typeMapSet` `AbstractType.js:846-879`）——
  值域边界硬编码于 switch，无法扩展自定义 content。
- [事实] Map 读取 = 取 key 链**右端未删除 item**（`typeMapGet`
  `AbstractType.js:889-893`；integrate 时新值入链并顺手 `left.delete`
  `Item.js:509-516`）——**YMap 的 LWW 是"链尾即真相"的 CRDT 化实现，
  而非时间戳比较**。
- [事实] `YText`：insert 先扫描当前属性上下文（缺省继承光标处格式，
  `src/types/YText.js:1124-1142`），否定属性以 `ContentFormat(key,null)`
  记录（`insertNegatedAttributes` `YText.js:150-175`）；格式是**不可
  计数 item**（不占 index，`AbstractContent.isCountable` 契约
  `Item.js:739-755`）；对外 API 直接吃 Quill delta（`applyDelta`
  `YText.js:971-995`）。未挂 doc 的类型操作进 `_pending` 队列，挂载后
  重放（`YText.js:1139-1141`）。
- [事实] 位置索引性能：链表 + **≤80 个 search marker 缓存**（LRU 式
  时间戳淘汰，`AbstractType.js:27,49-80`，INTERNALS.md 称 "skip list /
  fast search marker" `yjs-v13/INTERNALS.md:87-96`）——**没有真正的
  跳表/平衡树**，索引访问是 marker + 局部线性扫描的工程折中。
- [事实] v14 边界重画：YMap/YArray/YText/YXml 四类**合并为 YNode 单类**
  + `lib0/schema` 描述结构（`yjs/src/ynode.js:1-40`、`schemas.js:3`），
  输出走统一 delta（`delta-helpers.js`）+ 归因渲染（`Renderer.js`）——
  从"多类型类族"收敛为"单节点模型 + schema 约束"。

### 1.6 CRDT 语义的工程取舍（诚实登记）

- [事实] **插入 op-based、删除 state-based** 的混血：删除只翻
  deleted 位 + 记入 transaction.deleteSet（`Item.delete` `Item.js:616-628`），
  不记"谁删/何时删"（INTERNALS.md "No data is kept on *when* an item
  was deleted, or which user deleted it" `yjs-v13/INTERNALS.md:104-111`）。
- [事实] 体积只增的对策三件：相邻 struct 合并（`Item.mergeWith`
  `Item.js:571-609`）、删除内容剥壳（ContentDeleted）、父删则子 GC
  （`tryGcDeleteSet` 事务尾部执行，受 `doc.gcFilter` 过滤
  `src/utils/Transaction.js:199,328`）；`doc.gc=false` 可全关（快照/恢复
  需要墓碑时，`yjs-v13/README.md:859-864`）。
- [事实] 事务是原子与增量单位：变更收集→观察者→update 事件单条发出
  （`Transaction.js:370-376`）；**嵌套事务合并为一条 update**（README
  "You should *bundle* changes into a single transaction"，`README.md:
  865-874`）。
- [推断] 取舍画像：Yjs 用"**元数据极简**"（删除零元数据、ID 双字段、
  无版本号）换"协议最小可交换"，代价是历史信息（谁删的、何时删）天然
  缺席——需要审计/归因的产品层只能在外部记账（v14 的 attribution 体系
  正是在补此课，`yjs/src/ynode.js:44-80`）。

**净室问题小结（Q1）**：YATA 在 Yjs 的工程化 = **自描述 struct 字节流
（V1 流式/V2 列式）+ 链表整合（两规则+clientID 平局）+ 薄类型视图 +
state-based 删除 + 元数据极简**。三件套分层清晰：update 是内核唯一的
可交换载荷；awareness 完全外包给 y-protocols（LWW 广播语义）；subdoc
只是"guid 引用 + 独立 Doc"的占位协议。**可移植结论**：①update 编码
（structs+DS 两段、列式压缩、ID 内联锚点）可直接作为 mdcore 协同内核
op 编码的对照蓝本；②"类型=Item 链视图"的薄类型层与 Univer"快照即
模型"是两个极端，mdcore 的 op-first 树结构更靠近 Yjs 侧；③值域 switch
（1.5）说明扩展 content 类型需要动内核——Rust 侧应以 trait/enum 显式
留位。

## 2. Q2 大文档分片与冷热数据

### 2.1 subdoc 分片机制：引用即分片

- [事实] 父文档中一个 subdoc = 一个普通 Item，content 为 `ContentDoc`，
  **线上的全部载荷 = guid 字符串 + opts 对象（gc/autoLoad/meta）**：
  `write: writeString(guid); writeAny(opts)`（`src/structs/ContentDoc.js:
  121-124`）——子文档体**永不内嵌**，是天然的分片边界。
- [事实] 远端解码 = 新建一个**冷 Doc**：`createDocFromOpts(guid, opts)`，
  `shouldLoad = opts.shouldLoad || opts.autoLoad || false`（`ContentDoc.js:
  9-11,140`）；即远端拿到的是"占位壳"，内容需另走一次以 guid 为房名
  的同步。
- [事实] 集成时向事务登记：`transaction.subdocsAdded.add(doc)`，若
  shouldLoad 则同时 `subdocsLoaded`（`ContentDoc.js:92-99`）；删除时
  移入 `subdocsRemoved`（`ContentDoc.js:104-110`）。
- [事实] 事务收尾统一发 **`subdocs` 事件 `{added, removed, loaded}`**
  并级联销毁 removed（`src/utils/Transaction.js:380-391`）——这是
  provider 感知分片的唯一钩子。
- [事实] 官方文档口径（docs.yjs.dev/api/subdocuments，2026-10-11 抓取）：
  "An obvious use-case is to manage documents in a folder structure. Each
  document (potentially containing large amounts of rich-text content) could
  be represented as a subdocument that is **lazily loaded to memory when
  needed**"；"**By default, subdocuments are empty until they are explicitly
  loaded**"。

### 2.2 懒加载语义与 provider 契约

- [事实] `DocOpts`：`gc/gcFilter/guid/collectionid/meta/autoLoad/
  shouldLoad`（`src/utils/Doc.js:26-34`）；`load()` = 通知父文档事务
  `subdocsLoaded` + 本地置 shouldLoad=true，幂等（`Doc.js:146-161`）；
  `destroy()` = 递归销毁子级 + 把 ContentDoc 里的 doc **换成新的冷实例**
  （`new Doc({guid, ...opts, shouldLoad:false})`，`Doc.js:325-341`）——
  **销毁即卸载内存、再访问即重建壳**。
- [事实] provider 生命周期契约（内核侧文档注释）："Note that not all
  providers implement this feature. Provider authors are encouraged to fire
  the `load` event when the doc content is loaded from the database"（`Doc.js:92-98`）；
  配套 `isLoaded/isSynced` 标志 + `whenLoaded/whenSynced` Promise（断线
  时 whenSynced 重建，`Doc.js:100-143`）。
- [事实] 行为锁（tests）：远程应用 update 后 subdoc 的 shouldLoad 恒为
  false（冷壳）——本地 shouldLoad **不序列化**（`tests/doc.tests.js:
  199-209`）；autoLoad 才随 opts 传播（`tests/doc.tests.js:220-229` 的
  autoload 边界用例）；同 guid 两实例自动汇合（`tests/doc.tests.js:
  133-136`）。
- [事实] 官方文档对实现的诚实表述："It is up to the providers to sync
  subdocuments… **But all official Yjs providers currently think of
  sub-documents as separate entities**"；懒加载标准姿势 = 监听 `subdocs`
  事件、对 loaded 集合按 `doc.guid` 开 provider 实例（docs.yjs.dev，
  同上抓取）。
- 【推断】Yjs 的分片是**协议级占位 + provider 级实现**：内核保证"父
  文档可先同步、子文档引用可见而内容不可见"，但**不提供分片内部分页/
  区间加载**——单个 subdoc 依然整体进内存（无"只加载某页"语义）。
  对超大单文档，Yjs 的答案是"你应该拆 subdoc"，而非流式加载。

### 2.3 乱序缓冲：pendingStructs / pendingDs / Skip（分片的传输前提）

- [事实] `readUpdateV2`：无法整合的 struct 流打包成
  `{missing: Map<client,clock>, update: Uint8Array}` 存入
  `store.pendingStructs`，与既有 pending 用 `mergeUpdatesV2` 合并；新
  update 到达且 missing 满足即整体重放（`encoding.js:395-417,444-448`）。
- [事实] deleteSet 乱序同理存 `store.pendingDs`，`encodeStateAsUpdate`
  会把 pending 一并编码进全量导出（`encoding.js:420-438,522-541`）——
  **pending 态不会导致导出丢数据**。
- [事实] `Skip` struct 出现在 update 流 = "本区间内容缺失"的显式占位
  （`encoding.js:137-142` 读取；`updates.js:52-55` lazy reader）；
  mergeUpdates 以 Skip 填洞并可延展合并（`updates.js:405-418`）。
- [推断] 这是离线-first 的核心机制：任意顺序、任意缺口的 update 都能
  先收下后整合（与 OT 需要 baseRev 对齐形成对照，§3.4）。

### 2.4 大文档策略：SV diff / mergeUpdates 压实 / 无 Doc 服务端合并

- [事实] 同步按需差量：`encodeStateAsUpdate(doc, targetSV)` 只写对端
  缺的 struct（`encoding.js:504-541`；官方口径 "Sync two clients by
  computing the differences"，`README.md:962-976`）。
- [事实] **不加载 Y.Doc 的服务端合并**：`encodeStateVectorFromUpdate`/
  `diffUpdate`/`mergeUpdates` 全部直接吃 update 字节（README "Syncing
  clients without loading the Y.Doc" 完整示例 `README.md:978-1000`）——
  relay 服务器可以零实例合并增量。
- [事实] 持久化压实范式（y-leveldb）：update 追加日志 + **>500 条自动
  压实**（`PREFERRED_TRIM_SIZE=500` `y-leveldb/src/y-leveldb.js:11`；
  `getYDoc` 重放全部后在 updates>500 时 flush，`y-leveldb.js:485-499`）；
  flush = 追加全量快照 + 写 SV + 清区间（`y-leveldb.js:402-413`）。
- [事实] 快照 = SV + DS 的可恢复时间点（Snapshot，`src/utils/Snapshot.js`；
  INTERNALS.md "Snapshots" `yjs-v13/INTERNALS.md:168-179`——不回滚文档，
  而是按 SV 截断可见性重算旧态）。
- [事实] v14 新增 BlockSet/ID 索引等内核级结构（`yjs/src/utils/BlockSet.js`、
  `src/utils/ids.js` 的 IdSet/IdMap 泛化）——【推断】为大文档查询/归因
  铺路的重写件，尚未定稿。

### 2.5 provider 侧房间冷热（经典后端样本）

- [事实] y-websocket-server：房间 = URL 路径名，`getYDoc` 惰性建内存
  Doc（`src/utils.js:139-147`）；**最后一个连接断开 → writeState 落库 →
  doc.destroy() → 从 docs Map 删除**（`src/utils.js:196-202`）——内存
  房间完全卸载，下次连接重走 bindState 冷启动。这是"热=内存 Doc、
  冷=持久层"的最简完整样本。
- [事实] y/hub（新一代后端）宣称"Memory efficient: The server doesn't
  maintain a Y.Doc in-memory. It streams updates through redis. The Yjs
  document is only loaded to memory for the initial sync"（yjs/yhub
  README，2026-10-11 抓取）——**服务端常态零 Doc、仅初始同步时加载**。

**净室问题小结（Q2）**：分片单位 = subdoc（guid 引用 + 独立 Doc + 独立
同步房间）；冷热语义 = load/destroy 双向开关 + provider 契约（load 事件/
subdocs 事件/按 guid 开房）；大文档 = SV 差量 + mergeUpdates 压实 +
（y-leveldb 式）日志→快照双形态 + 服务端可零实例转发。**可移植结论**：
①「父文档只存 guid 引用」是**协议内建的分片原语**，mdcore 的文档树
（MD/画布/表格页）可直接映射：树骨架在根 Doc，重内容各为一页 subdoc，
懒加载语义现成；②持久层应抄 y-leveldb 的"update 日志 + 阈值压实为快照
+ SV 伴随"三件套（与 mdcore 既有 snapshot/revision 体系天然同构）；
③单 subdoc 无分页加载是**边界事实**——单页超大（如十万行表格）时
Yjs 生态本身也要靠 subdoc 拆分，mdcore 表格引擎若要页内流式则属超越
项（对照 Univer 的 SaveSheetBlock 行分块，office-a §4）。

## 3. Q3 离线合并语义：原生收敛的实证

### 3.1 官方承诺（口径）

- [事实] README 定义（原文）："Document updates are **commutative** and
  **idempotent**. This means that they can be applied **in any order and
  multiple times**"（`yjs-v13/README.md:924-926`）——可交换、幂等、任意
  顺序重复应用三性即官方离线合并承诺。
- [事实] README CRDT 章：无中心服务器立场原文——"OT approaches that
  support shared editing without a central source of truth (a central
  server) require too much bookkeeping to be viable in practice. CRDTs are
  better suited for distributed systems… and do not require a central
  source of truth"（`README.md:1317-1327`）。

### 3.2 收敛的机械保证（代码）

- [事实] 整合顺序无关的机制链：任意 update 进来先解析成带 origin 锚的
  struct 列表（`readClientsStructRefs` `encoding.js:112-198`）→
  `integrateStructs` 栈式调度按**因果依赖**而非到达顺序整合（`encoding.js:
  200-359`）→ 冲突消解只看 ID 与锚（`Item.integrate` §1.4）→ 乱序缺口
  进 pending 缓冲（§2.3）。**没有任何"版本号对齐/回滚"步骤**。
- [事实] 删除合并是集合并运算：DS 按 client 分区间 merge（`mergeDeleteSets`
  `src/utils/DeleteSet.js:144`）；远端 DS 应用时对已删区间幂等
  （`readAndApplyDeleteSet` `DeleteSet.js:278`）。
- [事实] 重复应用幂等：applyUpdate 对已存在的 struct 区间直接跳过
  （integrateStructs 中 `offset>=0` 分支与 `readAndApplyDeleteSet` 的
  已删判断；测试锁 `compare` 断言 `pendingDs===null && pendingStructs
  ===null` 收尾，`tests/testHelper.js:324-327`）。

### 3.3 测试实证（收敛性的三重锁）

- [事实] **随机收敛引擎**：`applyRandomTests` 以 5 实例 + 随机修改器 +
  随机断连/消息乱序（2% 断连、1% 全量 flush、50% 单消息乱序投递）
  跑 N 轮后 `compare` 全量对比（`tests/testHelper.js` applyRandomTests
  主体）；npm test 以 `--repetition-time 50` 跑随机时长、extensive 模式
  10000ms（`yjs-v13/package.json` test 脚本）。
- [事实] **compare 是结构级对比**：比较 array/map/xml/text 四视图 JSON +
  **逐 struct 对比 origin/rightOrigin/left/right/parentSub/content**
  （`compareStructStores` `tests/testHelper.js:376-399`）+ SV + DS +
  snapshot 相等（`testHelper.js:343-361`）；且每个用户的全部 update 经
  `Y.mergeUpdates` 重放到新 Doc 后**并入对比集**（`testHelper.js:312-319`）
  ——同时锁"乱序应用收敛"与"mergeUpdates 与逐条应用等价"两个命题。
- [事实] **合并排列锁**：`checkUpdateCases` 造 5 种合并形态（全量、重叠
  切片、跨序切片、含 Skip 的分离段、含重复的全叠）逐一应用后与原 Doc
  数组比对相等（`tests/updates.tests.js:140-200,223-258`）。
- [事实] **乱序补投锁**：`testMergePendingUpdates` 按 0→1→3→2→4 的洞序
  投递，最终断言文本收敛为 'nenor'（`tests/updates.tests.js:263-306`）。

### 3.4 与 OT 路线的语义差异（Yjs 侧事实登记；对照表留综合节）

- [事实] Yjs 侧无版本号/无 rebase/无仲裁：update 自描述（内联 origin
  锚），SV 仅用于**差量计算**而"不用于跟踪因果"（README State Vector
  节原文 "we use state vectors only to describe the state of the local
  document… We do not use it to track causality"，`README.md:1373-1384`）。
- [事实] 意图保持的近似性：并发插入按 clientID 定序（§1.4 case 1），
  **双方都保留、顺序确定**——不是 OT 的"位置变换保意图"，而是"全保留 +
  确定性定序"；YMap 并发写 = 链尾 LWW（§1.5），后者**明确丢失**败方
  意图（对 OT 阵营"map 并发也是 last-writer-wins"的常见批评，Yjs 以
  结构事实接受之）。
- 【推断】与 Univer changeset 协议（office-a §5 事实：baseRev/revision
  双版本号、`(sid,reqId)` 幂等会话、`FetchMissingChangesets(from,to]`
  补拉、服务端 rebase）逐项对照：Yjs 把"对齐/补洞/rebase"全部下沉为
  结构内生的因果整合 + pending 缓冲，**协议面没有"拒绝"态**（无
  csRejEvent 等价物；任何合法 update 都被接受），代价是无中心仲裁时
  map 语义退化为 LWW、且无法按版本回放服务端编排的并发历史。
- 【推断】离线合并语义对比的实质：OT = **强意图保持 + 需要有人算
  transform**（中心化或去中心化记账）；Yjs = **零协调收敛 + 意图按
  clientID/链尾近似**。对"离线编辑后重连"场景 Yjs 是免协议的（收下即
  整合），OT 需要版本对齐协议（Univer/ONLYOFFICE 均有专门设计）。

**净室问题小结（Q3）**：原生收敛不是宣传语而是**结构内生**：自描述
struct（origin 锚内联）+ 因果栈整合 + pending 缓冲 + DS 集合并 + 随机
测试三重锁（乱序投递/合并排列/mergeUpdates 等价）。**可移植结论**：
①若 mdcore 三线统一走 CRDT，离线合并的语义免费获得，但需在产品层
补"意图审计"（Yjs 删除零元数据 + clientID 定序是明确的能力缺口）；
②收敛性的测试范式（随机修改器 + 消息乱序 + 结构级 compare）可直接
移植为 mdcore 协同内核的 property-based 测试骨架；③"SV 只算差量、
不追因果"是协议减负的关键决断，值得照抄。

## 4. Q4 provider 生态与持久化

### 4.1 三层架构分层（provider = 可替换件）

- [事实] 分层：**内核（yjs，零网络）→ 协议（y-protocols：sync/
  awareness/auth 三模块，纯消息编解码）→ provider（传输 + 房间 +
  持久化回调）**。内核只约定事件契约（update 事件/updateV2/subdocs/
  load/sync），README 列 19 连接 provider 与 7 持久化 provider
  （`<dt>` 条目合计 26，`yjs-v13/README.md:200-364`）。
- [事实] y-websocket 客户端在同一 WS 连接上多路复用四个信封消息：
  `messageSync=0, messageAwareness=1, messageAuth=2,
  messageQueryAwareness=3`（`yjs/y-websocket/src/y-websocket.js:20-23`）；
  sync 消息体即 y-protocols 三消息透传（`y-websocket.js:31-65`）；跨标签
  页再复用 BroadcastChannel（`bcChannel = serverUrl + '/' + roomname`
  `y-websocket.js:384`，README "Supports cross-tab communication"）。
- [事实] v4 rc 客户端新增**未确认 update 追踪**：用
  `Y.createContentIdsFromUpdate` 对收到的 Step2/Update 做 ID 级对账，
  维护 `unconfirmedUpdates` 并发同步状态事件（`y-websocket.js:52-64`）
  ——provider 侧对"服务器收到没有"从黑盒变可观测。

### 4.2 同步协议（wire 格式，v13/v14 双线一致）

- [事实] sync 三消息：`SyncStep1=0`（载荷 SV）、`SyncStep2=1`（载荷
  差量 update）、`Update=2`（载荷增量 update）；消息 = `[messageType:
  varUint, 定义...]`，**不含房间名——房间归上层**（`y-protocols-v1007/
  sync.js:30-40` docstring + `:38-40` 常量）。
- [事实] 客户端-服务器同步剧本（docstring 原文设计）："The client
  should initiate the connection with SyncStep1. When the server receives
  SyncStep1, it should reply with SyncStep2 immediately followed by
  SyncStep1… The server should only reply to requests, and not initiate
  them"（`sync.js:23-28`）——**服务器只应答不发起**，天然适配 HTTP/WS
  与无状态扩展。
- [事实] awareness wire = `[条数, (clientID, clock, JSON 状态)...]`
  （`y-protocols-v1007/awareness.js:194-212`）；LWW 时钟比较 `currClock <
  clock` 才应用，**null 状态 + 同 clock 才允许远端移除**（`awareness.js:
  241-288`），本地身份保护（远端删不掉我的状态，clock++ 反播，
  `awareness.js:261-268`）；30s 过期 + 15s 自续约（`outdatedTimeout=30000`
  `awareness.js:13,61-82`）。**awareness 不进 CRDT、不持久化**（docstring
  "non-persistent data like awareness information" `awareness.js:22-24`）。
- [事实] auth 消息 = 权限拒绝通知（`y-protocols/src/auth.js`，消息 2）；
  PROTOCOL.md 已把上述全部 wire 格式规范化（RFC2119 口径，
  `y-protocols/PROTOCOL.md` §3-§5）。
- [事实] Hocuspocus 对信封做了**向后兼容扩展**：0-3 与 y-websocket 对
  齐，新增 `SyncReply=4`（避免服务器回触 SyncStep1 循环）、`Stateless=5/
  BroadcastStateless=6`（应用自定义消息）、`CLOSE=7`、`SyncStatus=8`、
  `Ping/Pong=9/10`（`hocuspocus/packages/server/src/types.ts:61-74`）；
  v3↔v4 线协议兼容声明（`hocuspocus/RELEASE_NOTES_V4.md:6-17`）。

### 4.3 自托管后端矩阵（生态事实）

| 后端 | 许可（已核） | 架构事实 | 一手锚点 |
|---|---|---|---|
| y-websocket-server（`@y/websocket-server` 0.1.5，已迁 v14 线；即 y-websocket 历史内置服务器抽出的 forkable 仓） | MIT | 内存房间 + 可插拔 persistence（bindState/writeState），末连接即卸载（§2.5） | `src/utils.js:139-205`、`package.json`（yjs ^14.0.0-7） |
| **Hocuspocus v4** | MIT（LICENSE.md，Tiptap GmbH） | 扩展服务器：hooks（onAuthenticate/onChange…）+ 持久化/Redis/S3/SQLite/throttle/webhook 扩展件；v4 用 crossws 跨运行时（Node/Bun/Deno/CF Workers） | `RELEASE_NOTES_V4.md:1-10`、`packages/*` 目录 |
| y/hub | **AGPL-3.0 OR 专有** | Redis 流式转发、常态零内存 Doc、S3/Postgres 持久化、auth 钩子；自称 beta | yjs/yhub README（抓取） |
| y-sweet | MIT（Rust） | S3/对象存储后端 document store | repo README + LICENSE（抓取核验） |
| y-webrtc | MIT | P2P + 信令服务器（去中心化样本） | GitHub API license 字段 |
| （托管）Liveblocks/PartyKit 等 | 商业 | 全托管 Yjs 兼容 | README:229-257 |

- [事实] Hocuspocus 双包均钉在 v13 线：`@hocuspocus/provider` 与
  `@hocuspocus/server` 依赖 `yjs ^13.6.8` + `y-protocols ^1.0.6`
  （两 package.json dependencies）——与 §0 双轨判断互证（生态主体仍在
  v13，官方新件先迁 v14）。
- [事实] 官方 README 的 provider 推荐已把 Hocuspocus/y-sweet/@y/hub 与
  y-websocket 并列为替代后端（`yjs/y-websocket/README.md:47-53`）。
- 【推断】自托管协同的**第一梯队事实标准 = Hocuspocus（MIT、扩展钩子
  型、TS）**；官方新一代 y/hub 反而走双许可——**"Yjs 系自托管"在许可
  干净度上要逐后端核验**，这对 mdcore 的"不用别人后端、只对齐协议"是
  重要边界澄清。

### 4.4 持久化面

- [事实] Hocuspocus Database 扩展模型：`fetch → Y.applyUpdate` 读入、
  `store → Y.encodeStateAsUpdate(doc)` **全量快照 upsert**（非日志追加）
  （`hocuspocus/packages/extension-database/src/Database.ts:45-61`）；写
  触发经服务器级 debounce（默认 **2000ms**，`packages/server/src/
  Hocuspocus.ts:29`）；SQLite 扩展即 `documents(name, data blob)` 单表
  upsert（`packages/extension-sqlite/src/SQLite.ts:6-19,38-57`）。
- [事实] y-leveldb 是**日志型**（append update + 500 条阈值压实 + SV
  伴随记录），README 顶部标注 **deprecated**（`y-leveldb/README.md:1-6`）。
- [事实] 浏览器侧 y-indexeddb（"persists document updates… only diffs
  need to be synced"，`yjs-v13/README.md:327-332`）；服务侧另有
  y-mongodb/y-postgresql 社区件（`README.md:333-360`）。
- 【推断】持久化两种形态并存且可互换（因为 update 可交换幂等）：
  **快照型**（Hocuspocus：简单、丢历史、写放大固定）vs **日志型**
  （y-leveldb/y-indexeddb：保历史、可回放、需压实）——Yjs 的 update
  语义让两者只是"存什么"的策略差，不影响正确性。这对 mdcore 是关键
  解耦证据：**协同正确性与持久化形态无关**。

### 4.5 生态印象核验（规划 §6.1 两条待核，逐一定谳）

- **AFFiNE → Yjs 系：确认**。[事实] Yjs README "Who is using Yjs" 首位
  即 AFFiNE（`yjs-v13/README.md:51-52`）；AFFiNE 引擎 BlockSuite 的
  `@blocksuite/store` 依赖 `yjs@^13.6.18` + `y-protocols@^1.0.6`
  （toeverything/blocksuite master `packages/framework/store/package.json`
  raw 抓取 2026-10-11）；Yjs README 绑定表将 BlockSuite 列为 native
  binding（`README.md:165`）。
- **Excalidraw 自托管协同 → Yjs 系：证伪**。[事实] `@excalidraw/
  excalidraw@0.18.0` 的 dependencies/peerDependencies/optionalDependencies
  **零 yjs/y-protocols/hocuspocus 依赖**（excalidraw/excalidraw master
  `packages/excalidraw/package.json` raw 抓取 2026-10-11）；本仓 canvas-a
  深读已定谳其协同 = "socket.io 盲转中继 + E2E 加密 + 元素级 LWW +
  Firestore 单文档密文落库，非 OT 非 CRDT"（canvas-a §5；协同服务端
  闭源，npm 包对服务端无知）。规划印象仅对到"Excalidraw 生态存在
  第三方 yjs 集成"的弱形式——**官方主线不是 Yjs**。
- 【推断】修正后的生态画像：Yjs 的统治域是**块编辑器/富文本/知识库**
  （AFFiNE/Huly/GitBook/Evernote/Linear/JupyterLab/Typst/NextCloud，
  `README.md:49-131`）与**协同框架层**（tldraw 系除外）；Excalidraw
  代表"轻量白板自研 LWW"的反例路线——与 canvas-a 结论（元素小、
  LWW 够用）互证。

### 4.6 安全姿态（一手 THREAT_MODEL）

- [事实] 官方威胁模型三大立场（`yjs/THREAT_MODEL.md`）：①"**The CRDT
  protocol is designed for collaboration between trusted peers**… does
  not protect against a malicious peer intentionally writing destructive
  operations. Once a malicious update is applied… part of the document's
  permanent history"（:29-31）；②"**Server-side filtering of updates is a
  common antipattern**… selectively filtering them can easily be exploited
  and is not a reliable security boundary"（:33-35）；③建议 **"Suggestion-only
  (untrusted) users should work on a separate Yjs document fork"**，由
  可信端审核后选择性合并（:37）。
- [事实] 攻击面登记：恶意 peer 可发巨量/深嵌套 update 做 DoS、触发解码
  器越界（:39-43）；缓解 = 认证在前、消息尺寸限额、fuzz 解码器（:44-51）。
  认证/授权/TLS/存储访问控制**显式划出库外**（§5 Out of Scope，:115-122）。
- [事实] 供应链姿态：**运行时依赖只有维护者自家的 lib0/@y/protocols**
  （"This significantly reduces supply chain risk"，`THREAT_MODEL.md:53-57`；
  v13 `package.json` dependencies 仅 `lib0`）。

**净室问题小结（Q4）**：生态分层 = 内核/协议/provider 三层硬边界，
sync+awareness 协议十年冻结且被 Hocuspocus 等向后兼容扩展；自托管第一
梯队是 MIT 的 Hocuspocus（快照型持久化 + 2000ms debounce + hook 扩展），
官方新后端 y/hub 转双许可；持久化形态（快照/日志）与正确性解耦。
**可移植结论**：①mdcore 自研协同服务端只需实现 **三消息同步协议 +
awareness 编码**即可全量兼容 Yjs 客户端生态（y-websocket-server 参考实现
<300 行，`src/utils.js` 全长 280 行）；②信任模型必须按 THREAT_MODEL 采纳：**写权=受信、建议=fork+
审核合并**——这恰好是 agent 写入的理想通道（与 office-a 结论 8"agent
走同一 op 通道"互证：Yjs 的答案是把 untrusted agent 放 fork doc）；
③y/hub 的 AGPL 双许可提醒：**引用 Yjs 生态 ≠ 引用其全部许可面**，后端
选型要逐件核（本 pass 已核清单见 §5）。

## 5. 许可证与合规定谳

| 仓 | 许可 | 核验锚点 |
|---|---|---|
| yjs（v13.6.33 与 v14-rc 双线） | MIT | `yjs-v13/LICENSE`（23 行）、`yjs/LICENSE` |
| y-protocols（v1.0.7 与 master rc） | MIT | `y-protocols-v1007/LICENSE`、`y-protocols/LICENSE` |
| y-websocket / y-websocket-server | MIT | 两仓 LICENSE |
| y-leveldb | MIT | `y-leveldb/LICENSE`（README 标注 deprecated） |
| Hocuspocus（provider/server/extensions） | MIT | `hocuspocus/LICENSE.md`（Copyright Tiptap GmbH） |
| y-webrtc | MIT | GitHub API license 字段 |
| y-sweet | MIT | repo LICENSE（raw 抓取） |
| **y/hub** | **AGPL-3.0 OR 专有** | repo README "Licensing" 节（raw 抓取） |

- 定谳：规划"Yjs MIT 待核"成立；**但生态后端中 y/hub 为 AGPL 双许可**，
  规划期"生态 provider 成熟"的印象需加许可注脚。MIT 各件允许净室引用
  其协议/行为描述乃至未来直接依赖；本 pass 全部产出为事实与推断描述，
  零代码复制，净室合规。

## 6. 对照与决策输入（统一协同内核？）

### 6.1 与 Univer 自研协议的对照（OT 侧参照，office-a §5/§14 事实引用）

- [事实→引用] Univer：三层契约包（protocol/network/rpc）+ Changeset
  `{baseRev, revision, mutations[], sid, reqId}` + 18 类 WS 事件 + 服务端
  rebase（sheets 域客户端零 transform）+ 快照/oplog 混合 + `(from,to]`
  断线补拉（office-a 报告 §5）。Yjs 侧对应物：**无版本号、无 rebase、
  无事件类型族（只有 update）、补拉=SV 差量**（§3.4 对照）。
- [事实→引用] 两者持久化同构：Univer `GetUnitOnRev = 最近 snapshot + 待
  跟进 changesets`（office-a §5）≡ Yjs "快照 + pending 合并导出"
  （§2.4）+ y-leveldb 日志/快照双形态（§4.4）——**快照+oplog 混合是
  两阵营共同收敛点**，对 mdcore 是稳态设计。

### 6.2 ONLYOFFICE OT 事实（office-b 已落盘，本轮回填）

- [事实→引用] ONLYOFFICE：change = 可序列化 undo 项（CChangesBase 的
  Class/Undo/Redo/WriteToBinary 双向二进制）+ **服务器全序**（变更索引
  由服务器单调分配）+ 客户端「先撤自有→按序应用外来→重放自有」
  ——**不写 per-type transform，用 undo/replay 换变换**；断线重连 =
  auth 流程补拉 + authChanges 版本对齐；表格域 = 行列悲观锁 + 结构
  变更后服务器重算锁区间（office-b §3）。Yjs 侧对应物：无版本号、
  无 rebase、无事件类型族（只有 update）、补拉 = SV 差量（§3.4）。
- [事实→引用] 与格式保真的耦合：真相源 = DOCY/XLSY/PPTY 私有二进制
  （万物过二进制枢纽），保存 = 旧 bin + changes 在 DoctRenderer 确定性
  重放装配——「保真不被协议承诺，被装配=确定性重放承诺」，协同与
  保真共存靠「同代码双执行环境」隐性契约（浏览器/服务器同一 sdkjs）
  （office-b §4/§5）——OT 与私有二进制真相源深度绑定，此点为 §6.3
  判据 1 的 ONLYOFFICE 侧实证。

### 6.3 决策问题初答（【推断】，最终裁决留 R2 收口综合）

**三条线是否统一协同内核？——初步依据指向"统一"，且统一在 CRDT 侧，
但按线分层实现强度：**

1. **格式保真耦合度是第一判据**。MD 线（mdcore 自有树模型）与画布线
   （Excalidraw 式元素模型/自有图模型）的文档模型都是**自有 schema**，
   CRDT 的"文档模型自定、协议无关格式"（Yjs update 不含任何 schema）
   完全解耦；而 ONLYOFFICE 式 OT 与私有二进制真相源/OOXML 装配深度绑定
   （office-b §4 实证；另参规划 §6.1
   表），自研产品仓复刻该绑定的成本高且与 mdcore 的 op-first 树结构相斥
   （office-a 结论 3 已判"快照即模型是反面教材"）。
2. **离线/断线需求是第二判据**。工作台定位 local-first（PDF 系列与
   05 规划一致），Yjs 的免协议离线合并（§3）与 mdcore 既有
   sync_resync/revision 体系可叠加；OT 需要自建版本对齐协议（Univer
   花了 18 类事件 + sid/reqId 全套）。
3. **agent 写入通道是第三判据**。Yjs THREAT_MODEL 的 fork+审核合并
   模式（§4.6）与 office-a 结论 8（agent 走同一 op 通道 + DirectWrite
   返回 revision）语义兼容：CRDT 的 fork 天然是无冲突 worktree。
4. **反对/谨慎因素（诚实登记）**：①map/属性并发在 CRDT 侧退化为 LWW、
   删除零元数据——办公线若要"意图级审计/编辑历史"，CRDT 内生不提供
   （需产品层外挂，成本≈自建 changeset）；②Yjs 内核正处 v13/v14 断代
   （§0），**直接依赖库本体有代际风险**——缓解 = 只依赖其协议（冻结）
   或选 yrs（Rust 端口，AppFlowy/Multi 在用，`README.md:378`、
   AppFlowy `:105`、Multi.app `:106`）/
   自研内核对齐协议；③中心化部署形态下 OT 的服务端管控（权限/编排/
   收敛顺序可控）优于 CRDT 的"收到即整合"——若工作台需要强服务端
   仲裁（如企业审计），需在 CRDT 之上再叠仲裁层，复杂度未必更低。

**若统一，选型排序的初步依据**：协议层采纳 **Yjs 同步协议子集**
（SyncStep1/2/Update + SV + awareness 编码，全部 MIT、规范已文档化）
作为跨线通用协同语言；内核实现层三选一【推断，留 R2】：(a) 直接依赖
yrs（Rust、MIT、Yjs 协议兼容——mdcore 技术栈最短路径）；(b) 净室
自研 YATA 内核对齐协议（本 pass §1-§3 的结构/编码/收敛事实即为规格
输入）；(c) Univer 式自研 changeset（仅当办公线保真需求压倒一切）。
(a) 与 mdcore"引擎化 + 外置能力"路线（PDF 系列方法论）最契合。

### 6.4 CRDT vs OT 对照表（初稿；最终裁决 R2）

| 对照项 | Yjs（CRDT，本 pass 实证） | Univer（自研 OT，office-a 实证） | ONLYOFFICE（OT，office-b 实证，本轮回填） |
|---|---|---|---|
| 中心化要求 | 零（P2P 亦可；服务器只应答 §4.2） | 需中心（服务端 rebase） | 需中心（DocService 服务器全序仲裁 + pubsub 广播；office-b §3） |
| 与格式保真的耦合 | 解耦（update 无 schema §1.2） | 中（mutations=JSON params） | 深绑（真相源=DOCY 族私有二进制，万物过二进制枢纽+确定性重放装配；office-b §4） |
| 离线合并 | 原生收敛（§3，零协议） | (from,to] 补拉 + 幂等会话 | 版本对齐（断线重连 auth 补拉 + authChanges；变更索引服务器单调分配；office-b §3） |
| 意图保持 | 并发全保留+clientID 定序；map=LWW（§3.4） | transform 保位置 | 服务器全序 + undo/replay（非 per-type transform；office-b §3） |
| 生态可复用件 | 协议冻结 + provider 全家桶（§4.3） | 协议开源但协同服务器在 Pro | 无独立可复用件（协同栈与产品深绑） |
| 许可 | MIT（y/hub 例外 AGPL 双许可 §5） | Apache-2.0 | AGPL-3.0+Ascensio §7（五主仓；sdkjs-plugins 例外 Apache-2.0；office-b §11） |
| agent 写入 | fork+审核合并（官方建议 §4.6） | DirectWrite/ForkUnit（Pro） | REST /ai-proxy + builder 脚本，无 agent 专用协同通道（change=可序列化 undo 项；office-b §5/§7） |

## 7. 四问题结论一览

| 问题 | 一句话结论 | 关键锚点 |
|------|-----------|---------|
| Q1 YATA 工程化 | 三件套 = 自描述 struct 字节流（两段式/列式 V2）+ 外包给 y-protocols 的 LWW awareness + guid 占位 subdoc；类型是 Item 链薄视图，冲突消解仅两规则+clientID 平局 | `encoding.js:57-102`、`Item.js:419-533`、`UpdateEncoder.js:162-202` |
| Q2 分片与冷热 | subdoc = 父文档只存 guid 引用、内容另走以 guid 为房的同步；load/destroy 开关 + provider 契约；大文档 = SV 差量 + mergeUpdates 压实 + 服务端零实例合并，但单 subdoc 无分页 | `ContentDoc.js:121-140`、`Doc.js:146-161,325-341`、`README.md:978-1000` |
| Q3 离线合并 | 可交换+幂等是结构内生（因果栈整合+pending+DS 集合并），随机乱序测试三重锁实证；与 OT 的实质差 = 零协调收敛换 map-LWW/删除零元数据 | `README.md:924-926`、`encoding.js:395-448`、`testHelper.js:309-363`、`updates.tests.js:263-306` |
| Q4 生态与持久化 | 三层硬边界 + 协议十年冻结；自托管第一梯队 Hocuspocus（MIT，快照型+2000ms debounce）；AFFiNE→Yjs 确认、Excalidraw→Yjs 证伪；y/hub AGPL 双许可为生态例外 | `sync.js:19-40`、`Database.ts:45-61`、`THREAT_MODEL.md:29-37`、README:49-131 |

---

附：证据锚点密度——本 pass 合计 ~90 处 `路径:行号` 级锚点（v13 主线
~55、y-protocols/y-websocket-server/y-leveldb/hocuspocus ~20、tests ~10、
v14/文档口径 ~5），覆盖四问题全部小节；许可 8 仓逐件核验在档（§5）。
外部一手源（docs.yjs.dev 两页、blocksuite/excalidraw/yhub/y-sweet raw
抓取，2026-10-11）均已标注抓取性质。
