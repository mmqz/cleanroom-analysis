# canvas-c：draw.io / maxGraph 算法 pass（六问题定向）

> **净室分析报告**（线 A 画布 P2 批次，算法定向 pass）。日期：2026-10-11。
> 对象一：jgraph/drawio @ `9034053`（dev 浅克隆，2026-10-08，VERSION 32.4.1，
> `drawio/VERSION:1`）。对象二：maxGraph/maxgraph @ `5bc8dc9`（main 浅克隆，
> 2026-10-11；org/仓名经 `git ls-remote` 实证——maxGraph 官方继任仓，非
> jgraph 下；npm 包 `@maxgraph/core` v0.25.0，`maxgraph/packages/core/package.json:3-6`）。
> 方法：单代理深读（模型/布局/路由/stencil/embed/重写策略六面），只读零修改
> 零 commit；结论标注 `路径:行号` 证据锚点，【推断】= 分析员判断。
> 路径前缀声明：`drawio/` = drawio 仓根；`w/` = drawio/src/main/webapp/；
> `m/` = maxgraph/packages/core/src/。文风与四问题框架对齐 canvas-d（ComfyUI
> 协议 pass），本篇按规划 §5.1 扩为六问题。
> 定位：**算法定向 pass**——draw.io 的核心资产不在渲染而在布局算法族 + 连线
> 路由 + stencil 形状库；这三样是纯算法，在 CA v. Altai 抽象-过滤-比较法下
> 是最安全的净室参考对象（思想不受版权保护）。本 pass 的核心产出是 §2.4
> 的**逐算法重写可行性评级表**。
>
> 一句话定性：**draw.io = 「mxGraph 瘦身内核 + 三座外置算法桥（ELK 布局 /
> libavoid 避障路由 / Bridge.NET 组织架构图）+ 数据化布局（childLayout 即
> 样式）」；maxGraph = mxGraph 的 TypeScript 再实现，以「XML 格式兼容 +
> mixin/插件拆解 God Object」为兼容锚点，仍处 0.x 破坏性演进期**。

## 0. 架构速览

```
draw.io（本 HEAD 的结构性事实：内嵌 mxGraph 已被裁剪为 7 包 115 文件*）
  ├─ w/mxgraph/src/      mxGraph 裁剪版：model/io/view/handler/shape/util
  │                      layout/ 仅存 5 文件（基类 mxGraphLayout + circle/
  │                      stack/parallel/composite 四个简单布局；无
  │                      Hierarchical/Tree/Organic）
  ├─ w/js/elk/           drawio-elk 桥（~923KB）：layered/mrtree/radial/force/
  │                      stress/disco/box —— 层级/树/有机布局全部外包给 ELK
  ├─ w/js/libavoid-js/   libavoid（Adaptagrams C++ 编译纯 JS，LGPL-2.1）：
  │                      避障正交路由（全局路由器）
  ├─ w/js/orgchart/      OrgChart.Layout（Bridge.NET 编译 C#→JS）+ 本地绑定
  ├─ w/js/grapheditor/   Graph.js（36,791 行）：stencil() 内联形状、childLayout
  │                      布局即样式、ELK 旧名映射
  └─ w/js/diagramly/     EditorUi.js（35,275 行）：embed postMessage 协议全在
                         此 + ElkLayout/LibavoidRouting 绑定层

maxGraph @maxgraph/core 0.25.0（Apache-2.0，零运行时依赖，ESM+CJS 双发布）
  ├─ m/view/             Graph(全家桶)/BaseGraph(零注册)/AbstractGraph(mixin 合成)
  │                      layout/ 保留 mxGraph 全部布局族（14 个布局 TS 类
  │                      文件 = 12 布局 + LayoutManager/SwimlaneManager
  │                      两管理器；另 types.ts + hierarchical/ 七件，
  │                      行数口径见 §2.1）
  │                      style/edge/ Orthogonal/Manhattan/EntityRelation/…
  │                      style/perimeter/ 5 个周界函数 + 注册表
  ├─ m/serialization/    Codec/ModelXmlSerializer：mxGraphModel XML 兼容
  │                      （__tests__/serialization/serialization.xml.mxGraph.test.ts 锁行为）
  └─ m/editor|gui/       mxEditor 遗存（Editor/KeyHandler/PopupMenu/Toolbar）
```

\* J1b 复核修正（2026-10-11）：内嵌 mxGraph 实测 115 个 .js 文件（7 包 +
根 mxClient.js；另有 index.txt 非 JS）——原文误记 138；Graph.js 36,791 行、
EditorUi.js 35,275 行（wc -l 实测，原文各多记 1）。

## 1. Q1 mxGraphModel XML 数据模型

### 1.1 cell/geometry/style 三件套

- [事实] `mxCell(value, geometry, style)` 构造即三槽位（`w/mxgraph/src/model/mxCell.js:67-77`）；
  id/value/geometry/style/vertex/edge/parent/source/target/children/edges 全部为
  可空字段（`mxCell.js:197-481`）。vertex/edge 布尔并存——**cell 是顶点或边由
  标志位决定，不是两个类型**。
- [事实] `mxGeometry` = (x,y,width,height) + `alternateBounds`（折叠态换算，
  `w/mxgraph/src/model/mxGeometry.js:159-178`）+ 边专属 `sourcePoint/targetPoint/
  points[]/offset` + `relative` 标志（顶点几何 absolute、端口几何 relative；
  `mxGeometry.js:73-244`）。`translate` 对 relative 几何跳过 xy 只动点列
  （`mxGeometry.js:283-291` docstring）。
- [事实] style 是**扁平字符串** `key=value;key=value`；解析在
  `mxStylesheet.getCellStyle`：分号切分、`key=none` 删除键、数值自动 parseFloat、
  裸名查命名样式表（`w/mxgraph/src/view/mxStylesheet.js:217-278`）。命名样式
  查表用 `hasOwnProperty` 防原型链成员解析为样式（`mxStylesheet.js` getCellStyle
  尾段，注释明言 "inherited member such as constructor must not resolve"）——
  与 embed 协议 remoteInvoke 的同类防御（§5.5）互证为 mxGraph 系一以贯之的
  输入卫生习惯。
- [事实] 命名样式经 `putCellStyle` 注册（默认 defaultVertex/defaultEdge 两套，
  `mxStylesheet.js:61-133`）；样式继承是**单层**：cell style 裸名 → 命名样式
  整体作为底座再叠加键值（`mxStylesheet.js:217+`）。

### 1.2 codec 层：对象图 ⇄ XML 的反射式序列化

- [事实] `mxCodec`/`mxCodecRegistry`/`mxObjectCodec` 是通用反射编解码器：按
  构造器名注册 codec，编码遍历对象可枚举属性写 XML 属性/子元素
  （`w/mxgraph/src/io/mxCodec.js`、`mxObjectCodec.js`，合计 2,146 行）。
  模型序列化入口 `mxModelCodec.encodeObject` 只做一件事——把 root cell 子树
  编进 `<root>` 节点（`w/mxgraph/src/io/mxModelCodec.js:36-40`）。
- [事实] `mxCellCodec.beforeDecode` 处理**倒置**：非 `mxCell` 命名的外层
  XML 节点整体作为 user object 挂到 `cell.value`（id 搬家到 cell 后从 value
  上删除，`mxCellCodec.js:167-197`）——这是「图形容器 + 任意业务负载」的
  兼容层。前向 id 引用（边引用尚未解码的顶点）由 `dec.getElementById(ref)`
  即时解码补齐，非 cell 引用报 console.error 并置空（`mxCellCodec.js:212-262`，
  含 "can be superlinear" 的性能注释）。
- [事实] 根结构约定：`createRoot` 造一个空 cell 再 insert 一个空 cell
  （`w/mxgraph/src/model/mxGraphModel.js:338-346`）——id "0"=root、"1"=默认层，
  **层（layer）= 普通 cell**，多页=多子 cell，无专门类型。`cells` 是 id→cell
  的字典缓存（`mxGraphModel.js:355-398`）。
- 【推断】模型层无版本字段：mxModelCodec 只写 `<root>`，五个 codec 文件均无
  version 属性写入（负断言，基于 §1.2 所列 codec 的通读）——版本兼容完全靠
  「字段可缺省 + 新读者容忍未知键」，与 office-a 对 Univer 的「无迁移器」
  观察（office-a §1）同构，是这一代 JS 图库的共同债。

### 1.3 drawio 文件层：mxfile/diagram 双层 + deflate

- [事实] drawio 在 mxGraphModel 外再包两层：`<mxfile><diagram>…</diagram>
  </mxfile>`；`Editor.parseXmlFileForXml` 识别 mxfile 取第 N 页（URL `page`
  参数，`w/js/diagramly/Editor.js:2499-2507`）；diagram 文本内容默认是
  **base64(raw-deflate(mxGraphModel XML))**，`Graph.decompress` 走 pako
  inflateRaw（`Editor.js:2393-2408`；`Editor.compressXml` 默认开，
  `Editor.js:277-279`）。
- [事实] 敌意输入防线实例：从 PDF 提取内嵌图时，"stream" 关键字只在对象
  字典头 2048 字节窗口内搜索、inflate 尝试上限 8 次、MIME 匹配
  `application#2Fvnd.jgraph.mxfile`（`Editor.js:2593-2686`，注释明言
  "Hostile files may contain many tokens without many streams to inflate"）。

### 1.4 表达能力与局限清单（对工作台）

**能表达**：任意深的组合层级（parent 指针 + children 数组）；边的中途
waypoint（geometry.points）；相对端口（relative 几何子 cell）；固定连接点
（边 style 上的 exitX/exitY/entryX/entryY，见 §3.3）；任意业务负载（user
object 倒置）；一图多页；命名样式单层继承；字符串/数值样式值。

**局限**：style 值域只有标量——无法表达数组/嵌套结构（对齐/分布/复杂
stroke 模板只能发明新键或编码进字符串）；无 z-index——叠放序 = parent 的
children 数组序，跨容器排序需 move 语义；无模型版本与迁移器（§1.2 推断）；
cell 类型仅 vertex/edge 二值，表/泳道/容器全靠 style 键 + 约定（drawio 用
`childLayout` 约定容器行为，见 §2.5）；样式解析是 O(键数) 字符串处理，
大图每次重渲染重复解析（mxCellState 缓存缓解）。

**小结**：mxGraphModel 是「薄 cell 图 + 扁平样式串 + 反射式 codec」的三层
极简设计——表达力上限低但 20 年兼容包袱小；drawio 的全部高级语义
（表格、泳道、树容器、ELK 布局参数）都是**寄生在 style 字符串上的约定**。

**可移植结论（Q1）**：mdcore 若做结构化图形页，应保留其三件事——
(1) cell 一元论（顶点/边同构、层即普通 cell）已被 20 年生态验证；
(2) 边的 waypoint/relative 几何与 exit/entry 键值对是端口语义的最小完备集；
(3) user object 倒置（图形层与业务负载解耦）。但 style 应改为**类型化
map + 字符串编解码边界层**（序列化兼容 mxGraph 风格，内存表示强类型），
且必须带 schema version + 迁移器（对冲 office-a §13-9 同款建议）。

## 2. Q2 布局算法族（核心产出节）

### 2.1 三代引擎并存的结构性事实

- [事实] drawio 内嵌 mxGraph 的 `w/mxgraph/src/layout/` 只剩 5 个布局文件：
  mxCircleLayout / mxCompositeLayout / mxGraphLayout / mxParallelEdgeLayout /
  mxStackLayout（目录实测；其中 mxGraphLayout 为基类，具体简单布局为
  circle/composite/parallel/stack 四个）——**Hierarchical/CompactTree/RadialTree/
  FastOrganic/Partition/EdgeLabel/Swimlane 全部被移除**。官方口径即注释：
  「The names of the removed mxGraph layouts run as their ELK replacements or
  are skipped」（`w/js/grapheditor/Graph.js:2018-2019`）。
- [事实] 旧名→ELK 映射保持 20 年旧文件可打开：`mxHierarchicalLayout` →
  elkLayered、`mxCompactTreeLayout` → elkTree、`mxRadialTreeLayout` →
  elkRadial、`mxFastOrganicLayout` → elkOrganic，旧参数逐项翻译
  （orientation→elk.direction、intraCellSpacing→elk.spacing.nodeNode、
  interRankCellSpacing→elk.layered.spacing.nodeNodeBetweenLayers、
  interHierarchySpacing→elk.spacing.componentComponent），
  mxPartitionLayout/mxEdgeLabelLayout 无替代直接跳过
  （`Graph.js:2096-2143`）。ELK 名单：layered/mrtree/radial/force/stress/
  disco/box 七算法（`Graph.js:2031-2039`）。
- [事实] ELK 桥为外置 bundle `@drawio/elk 0.1.0+commit.c6b8e34`（built
  2026-09-30，~923KB，`w/js/elk/drawio-elk.min.js:1` 文件头注释）；管线 =
  「mxGraph → ELK JSON → layout → mxGraph mutations」，**异步 prepare() +
  同步 apply() 闭包**（`w/js/diagramly/ElkLayout.js:26-27,102-107`），
  菜单预设与 drawio-mcp 共享单一来源 ElkLayout.MENU_PRESETS
  （`w/js/diagramly/Menus.js:1730-1739`；`ElkLayout.js:5-6` 明言 shared with
  drawio-mcp）。
- [事实] 组织架构图是第三套引擎：Bridge.NET（C#→JS 编译器，
  `w/js/orgchart/bridge.min.js:1-5` 头注释 "Bridge.NET 15.6.0"）编译的
  OrgChart.Layout（`OrgChart.Layout.min.js:1` `Bridge.assembly("OrgChart.
  Layout"…)`）+ 本地绑定 mxOrgChartLayout（版权头 2019-2020 JGraph，
  `w/js/orgchart/mxOrgChartLayout.js:2`），分支优化器枚举 LINEAR/HANGER2/
  HANGER4/FISHBONE1/FISHBONE2/1COLUMN_L/1COLUMN_R/SMART
  （`mxOrgChartLayout.js:28-53`）。
- [事实] mxGraph 原版布局族完整存活于 **maxGraph**（`m/view/layout/` 14 个
  TS 类文件（12 布局 + LayoutManager/SwimlaneManager 两管理器）+
  hierarchical/ 7 文件，共 9,679 行，`wc -l` 实测——J1b 复核：行数准确，
  「14 布局类」口径修正为「14 类文件含 2 管理器」；R4-fix 口径注：9,679 =
  根层 15 个 .ts（5,993 行，14 类文件 + types.ts 46 行）+ hierarchical/
  7 件（3,686 行），子目录 datatypes/ 4 件（533 行）+ util/ 2 件（138 行）
  共 671 行未计入，全树 28 件 .ts 实为 10,350 行）——本节深读以此
  为准（对 drawio 而言这些算法已是历史，对净室参考而言这是 Apache-2.0 的
  活规格）。

### 2.2 逐算法深读（maxGraph TS 版 = mxGraph 4.2.2 算法的直系移植）

**HierarchicalLayout（Sugiyama 族，四阶段流水线）**
- [事实] execute() → run() 中四阶段顺序调用：`cycleStage → layeringStage →
  crossingStage → placementStage`（`m/view/layout/HierarchicalLayout.ts:505-509`）；
  layeringStage 注释自称 "Implements first stage of a Sugiyama layout"
  （`HierarchicalLayout.ts:706-710`）。
- [事实] 阶段 1 去环 = `MinimumCycleRemover`：DFS 遇到「子是己之祖先」即
  invert 边并改挂 connectsAsSource/Target（`m/view/layout/hierarchical/
  MinimumCycleRemover.ts:63-93`）；DFS 漏掉的节点补第二遍随机起点 DFS
  （`MinimumCycleRemover.ts:103-135`）——**非最小反馈弧集，是 DFS 启发式**。
- [事实] 阶段 2 分层 = **最长路径分层**（注释原文 "Starting at the sinks is
  basically a longest path layering algorithm"，`hierarchical/
  GraphHierarchyModel.ts:255-257`）：worklist 算法，已扫描边打魔数戳
  5270620（`GraphHierarchyModel.ts:297,320`），结束后整体减 maxRank 归一化
  （`:356-361`）+ 根节点向源收紧（`:363-375`）；跨层边在 `fixRanks` 补
  **哑节点**（`GraphHierarchyModel.ts:383-389`）。
- [事实] 阶段 3 交叉最小化 = `MedianHybridCrossingReduction`：median 上/下
  扫 + transpose 启发式（docstring "Uses median down and up weightings as
  well heuristic"，`hierarchical/MedianHybridCrossingReduction.ts:25-30`），
  maxIterations=24（`:49`），主循环每轮 `transpose(i, model)`（`:92-97`），
  保留历史最优序（nestedBestRanks/currentBestCrossings，`:53-60`）。
- [事实] 阶段 4 坐标 = `CoordinateAssignment`（1,452 行，全家族最大件）：
  minNode/medianPos 两族启发式 + 基于「连线数+是否哑节点」的**优先级**
  排序（`hierarchical/CoordinateAssignment.ts:290,445,605+`），heuristic
  迭代 maxIterations=8（`:95`），最宽层居中再向上下扫
  （`rankCoordinates`，`:691-697`），参数含 prefHozEdgeSep=5/
  prefVertEdgeOff=2/minEdgeJetty=12/channelBuffer=4（`:100-116`）——**经典
  dot 优先级法的 JS 直译，价值全在调优常数**。
- [事实] 入口工程面：无入边者选根，否则取 fanOut−fanIn 最大者
  （`HierarchicalLayout.ts:264-308`）；不相交层级各自布局后以 initialX
  串接（`HierarchicalLayout.ts:485-510`）；端口=relative 几何子 cell
  （`HierarchicalLayout.ts:544-549`）；边样式默认 POLYLINE
  （`HierarchicalLayout.ts:162-165`）；边缓存三 Map（`:146-159`）。
- SwimlaneLayout（863 行）= 同一四阶段加泳道分组语义（`hierarchical/
  SwimlaneModel.ts`/`SwimlaneOrdering.ts`），复用同一 CrossingReduction/
  CoordinateAssignment。

**CompactTreeLayout（Moen 紧凑树）**
- [事实] 自述 "compact tree (Moen) algorithm"，只处理无环树、非连通点忽略
  （`m/view/layout/CompactTreeLayout.ts:63-68`）；核心是**轮廓链表**（contour
  upper/lower head/tail 的线段链，`CompactTreeLayout.ts:40-49` 类型），
  horizontal/vertical 双向递归 + 叶节点特化 layoutLeaf（`:553,561-660`），
  1,018 行。**Moen(1990) 轮廓合并 = 子树紧凑拼接，总复杂度 O(n) 摊还**
  【推断：复杂度为文献级结论，代码未见注释声明】。

**RadialTreeLayout**
- [事实] 继承 CompactTreeLayout，把树布局的 x 坐标**重解释为角度**：
  `theta = 2π × xProportion`（`m/view/layout/RadialTreeLayout.ts:192-193`），
  逐层半径由行极值梯度约束（`:171-183`），自半径开关 autoRadius（`:81`）——
  树布局的 200 行径向投影，复用轮廓算法（`RadialTreeLayout.ts:37-40`）。

**FastOrganicLayout（Fruchterman–Reingold 力导向）**
- [事实] 参数族 forceConstant=50 / minDistanceLimit=2 / maxDistanceLimit=500 /
  initialTemp=200（`m/view/layout/FastOrganicLayout.ts:58-91`）；迭代数未设时
  **20×√n**（`FastOrganicLayout.ts:273-275`）；主循环 = calcRepulsion（全对
  斥力）+ calcAttraction（沿边引力）+ 温度衰减限位移（`:280-295`）——教科书
  FR 算法直译，**O(iterations×(n²+e)) = O(n^2.5)**（全对斥力为复杂度主导，
  【推断】由 calcRepulsion 的双重循环结构得出）。

**简单族（drawio 仍内嵌其中四个）**（R4-fix 复核修正：原文误记「三个」，
  circle/stack/parallel/composite 四者均在 `w/mxgraph/src/layout/` 实测在位）
- [事实] mxCircleLayout（171 行，圆周排布+可选 moveCircle/x0/y0，drawio 用
  作 childLayout 'circleLayout'）、mxStackLayout（503 行，横/竖栈+fill/
  allowGaps 网格吸附）、mxParallelEdgeLayout（228 行，平行边扇形展开）、
  mxCompositeLayout（89 行，布局序列编排）、mxPartitionLayout（211 行）、
  mxEdgeLabelLayout（142 行）——后两者在 drawio 已删（`Graph.js:2105-2106`）。

### 2.3 复杂度与规模对照（maxGraph 实测行数）

| 算法 | 行数 | 复杂度（主项） | 备注 |
|---|---|---|---|
| HierarchicalLayout 主管 | 739 | — | 四阶段编排 |
| ├ MinimumCycleRemover | 138 | O(V+E) DFS | 启发式非最优 FAS |
| ├ GraphHierarchyModel | 642 | O(V+E) worklist | 最长路径分层+哑节点 |
| ├ MedianHybridCrossingRed. | 535 | O(24×(V+E)) | median+transpose |
| └ CoordinateAssignment | 1,452 | O(8×rank×n) | 优先级法+jetty/通道调优 |
| CompactTreeLayout | 1,018 | O(n)【推断】 | Moen 轮廓 |
| RadialTreeLayout | 303 | O(n) | CompactTree 径向投影 |
| FastOrganicLayout | 486 | O(n^2.5)【推断】 | FR 力导向 |
| ELK layered（drawio 外置） | — | EPL 工程（network simplex 等） | 未随仓源码 |

### 2.4 重写可行性评级表（本 pass 核心产出）

评级口径：★★★ = 可直接按公开规格净室重写（思想/参数/复杂度全部一手在档，
工程量 <1 人周/件）；★★ = 可重写但调优常数多（1–3 人周，成品质量依赖打磨）；
★ = 不建议按规格重写——应外置集成（进程/沙箱/WASM）或整体移植。
净室安全 = 该算法思想的公开文献基础（思想不受版权保护；实现表达不得复制）。

| 算法/家族 | 规格来源（一手） | 许可 | 文献基础 | 净室安全 | 重写评级 |
|---|---|---|---|---|---|
| 简单族（Stack/Circle/ParallelEdge/Composite） | maxGraph TS 全文 | Apache-2.0 | 初等几何 | 高 | ★★★ |
| FastOrganic（FR 力导向） | maxGraph TS 全文 | Apache-2.0 | Fruchterman–Reingold 1991 | 高 | ★★★ |
| ER/Elbow/SideToSide/TopToBottom/Segment/Loop 路由 | mxEdgeStyle.js 全文 | Apache-2.0 | 初等几何 | 高 | ★★★（§3） |
| 周界函数族（5 个） | mxPerimeter.js / maxGraph TS | Apache-2.0 | 解析几何 | 高 | ★★★（§3） |
| OrthConnector（局部正交） | mxEdgeStyle.js:1097-1400 | Apache-2.0 | channel/corner 启发式（自研风格） | 高 | ★★ |
| CompactTree（Moen 树） | maxGraph TS 全文 | Apache-2.0 | Moen 1990 | 高 | ★★（轮廓链表易错） |
| RadialTree | maxGraph TS（增量 200 行） | Apache-2.0 | 同上 | 高 | ★★ |
| HierarchicalLayout 四阶段 | maxGraph TS 全文（3,506 行） | Apache-2.0 | Sugiyama 1981 / dot（Gansner 1993） | 高（文献充分） | ★★（思想★★★；CoordinateAssignment 调优★★） |
| mxStencil 形状 VM | mxStencil.js 内嵌规格书 | Apache-2.0 | HTML5 canvas 语义 | 高 | ★★★（§4） |
| ELK 七算法（layered/mrtree/…） | 仅 bundle（min.js） | 上游 EPL-2.0【推断/外部：bundle 内仅 "Eclipse Layout Kernel" 字样（bundle 字符串实测）+ drawio README.md:17 「全部与 Apache-2.0 兼容」声明；elkjs 许可文本未随仓附】 | Kiel 大学 15 年工程 | **低（无仓内源码）** | ★（外置进程集成，见 §7） |
| libavoid 避障路由 | 仅 bundle + README | LGPL-2.1（`w/js/libavoid-js/LICENSE:1` 随仓附） | Wybrow/Marriott（Adaptagrams） | 低（同上） | ★（外置） |
| OrgChart.Layout | 仅 bundle | 仓内 Apache-2.0 覆盖（版权头 `mxOrgChartLayout.js:2`） | JGraph 自研启发式 | 低 | ★（org 专用，价值面窄） |

评级的三点脚注：
1. **「可按规格重写」的规格指 maxGraph TS 源码**——它本身就是 Apache-2.0，
   重写时允许读（净室纪律仍要求：读思想与参数，不逐行翻译表达）。
2. drawio 生产环境的真实选择（布局外包 ELK、路由外包 libavoid）是**重要的
   负面证据**：mxGraph 自家布局的成品质量被原作者自己放弃了——重写
   mxGraph 系布局的合理预期是「可用」而非「drawio 级观感」。
3. 唯一值得「先重写后外置」的例外是简单族与 ER/elbow 路由：它们是交互
   语义的一部分（拖拽时实时重路由），必须进程内低延迟。

### 2.5 布局即数据（childLayout / layout spec）

- [事实] 布局可挂在容器 style 上：`childLayout` 键驱动 `mxLayoutManager`
  （hasLayout=「style 有 childLayout」，`w/js/grapheditor/Graph.js:11441-11448`）；
  stackLayout/circleLayout/tableLayout/tree/flow 各自实例化并从 style 读参
  （`Graph.js:11469-11612`）；嵌套布局子先父后（`Graph.js:11454-11458` 注释）。
- [事实] 布局规格可序列化：custom-layout 数组 ↔ childLayout 字符串互转
  （`Graph.js:12706-12725` decode/encode）；`resolveLayoutList` 是**单一解析
  器**，同时服务桌面 `--layout` CLI、embed "layout" action、`#create` hash /
  load 选项、CSV 导入 `#` layout 行（`w/js/diagramly/EditorUi.js:26714-26733`
  docstring）——五种入口一种数据。

**小结**：布局资产分三代——mxGraph 原生族（maxGraph 为活规格，Apache-2.0，
全部可按规格重写，其中层级布局质量有限）、ELK 外置桥（EPL，工程级分层/
树/径向/力导向）、libavoid 外置桥（LGPL，避障路由）；「布局作为可序列化
样式/spec」是 drawio 最值得偷的设计。

**可移植结论（Q2）**：mdcore 图形页布局路线 = **自写★★★级（简单族+FR+树）
+ 层级布局按 Sugiyama 四阶段规格实现（对照 maxGraph 源码校验行为，预期
调优投入集中在坐标分配阶段）+ 重布局走「可序列化 layout spec」通道**
（Rust 侧 enum + serde，5 入口 1 数据的先例已验证）；ELK 级质量需求出现时
进程外集成而非重写。

## 3. Q3 连线路由

### 3.1 路由管线（mxGraphView 五步）

- [事实] 每次边状态更新：`updateEdgeState` → `updateFixedTerminalPoints`（约束
  定端点）→ `updatePoints`（调边样式函数算中段）→ `updateFloatingTerminalPoints`
  （周界求交）→ `removeDuplicatePoints` → `updateEdgeBounds/updateEdgeLabelOffset`
  （`w/mxgraph/src/view/mxGraphView.js:2547-2590`）。端点空/点数<2 的边直接从
  视图清除（`mxGraphView.js:2553-2571`）——**无效边不渲染的防线前置**。
- [事实] `updatePoints` 中边样式函数签名 `(edgeState, sourceState, targetState,
  points, result)`（`mxGraphView.js:2821-2882`）；端口间接层 `getTerminalPort`
  （`:3282`）；stencil fixed-aspect 端点路由前临时换 bounds、路由后恢复
  （`updateBoundsFromStencil`，`:2843-2863`、`:2788-2806`）。
- [事实] 边样式注册表：`mxEdgeStyle`（函数字典）+ `mxStyleRegistry` 供
  `edgeStyle=orthogonalEdgeStyle` 类字符串解析（`w/mxgraph/src/view/
  mxEdgeStyle.js:11-70` docstring）；`getEdgeStyle` 解析并取函数
  （`mxGraphView.js:2970`）。
- [事实] 去重策略按样式区分：OrthConnector 无控制点 / Elbow / SideToSide /
  TopToBottom / EntityRelation 才做 waypoint 去重（`mxGraphView.js:3091-3111`）。

### 3.2 边样式函数族（局部路由器）

- [事实] **OrthConnector** 自述 "a local orthogonal router between the given
  cells"（`mxEdgeStyle.js:1097-1112`）——非避障、非全局：入口取 jetty size
  （`getJettySize`，`:977`、`:1133-1134`），距离过短或有控制点时**回退
  SegmentConnector**（`mxEdgeStyle.js:1149-1164`），端口方向约束
  DIRECTION_MASK_ALL 起步（`:1166-1206`），旋转目标以 bounding box 换算
  （`:1180-1187`），之后进入 channel/corner 判定（自环 buffer 特判
  `:1138-1143`）。即：**正交路由 = 端点 jetty + 通道对齐启发式 + 分段回退**，
  不做障碍检测。
- [事实] **EntityRelation**（ER 图路由）：取 `STYLE_SEGMENT`（默认
  ENTITY_SEGMENT）定第一段长度，源/目标以相对几何 x≤0.5 判左右、y≤0.5 判
  上下，产出「水平-垂直-水平」三段（`mxEdgeStyle.js:98-150`）——几十行级
  的定向肘形路由。
- [事实] TopToBottom/SideToSide（垂直/水平肘形）与 ElbowConnector 互相代理
  （`mxEdgeStyle.js:391-395`——由 ElbowConnector 按源/目标方位代理；肘形两
  变体各为独立实现 `:402,438-539`）；SegmentConnector 消费既有
  控制点列（`:588-607`）。
- [事实] maxGraph 侧同族齐全且含 **Manhattan**（`m/view/style/edge/Manhattan.ts`）
  与注册表 EdgeStyleRegistry（`m/view/style/edge/index.ts`、`EdgeStyleRegistry.ts`）。

### 3.3 端口-周界模型（port/perimeter/constraint 三件套）

- [事实] **周界函数**签名 `(bounds, vertex, next, orthogonal)`：返回 next 点
  沿中心连线与周界的交点，orthogonal=true 时返回到边界的垂直投影
  （`w/mxgraph/src/view/mxPerimeter.js:68-84` docstring）；RectanglePerimeter
  以 atan2 角度分四边求交 + ortho 钳制（`mxPerimeter.js:84-152`）；族内还有
  Ellipse/Rhombus/Triangle/Hexagon（`mxPerimeter.js:156-229+`；maxGraph
  `m/view/style/perimeter/` 五文件+注册表）。周界求交入口
  `getPerimeterPoint`（`mxGraphView.js:3318`）。
- [事实] **连接约束** `mxConnectionConstraint = {point(相对 0-1), perimeter,
  name, dx, dy}`（`w/mxgraph/src/view/mxConnectionConstraint.js:19-29`）；
  stencil 声明的固定点直接成为该 cell 的约束集
  （`mxGraph.prototype.getAllConnectionConstraints`：`terminal.shape.stencil.
  constraints`，`w/mxgraph/src/view/mxGraph.js:7300-7306`）。
- [事实] **端点持久化在边 style 上**：exitX/exitY/exitPerimeter 与
  entryX/entryY/…（`getConnectionConstraint`，`mxGraph.js:7322+`）；约束→
  绝对坐标 `getConnectionPoint` 处理 direction 旋转/flipH/flipV/stencilFlip
  遗留键（`mxGraph.js:7435+`）。固定点间距后处理 `updateFixedPointSpacing`
  （STYLE_FIXED_POINT_SPACING，`mxGraphView.js:3113-3131`）。
- [事实] 端口语义三形态并存：(a) relative 几何子 cell（布局层 isPort 判定，
  §2.2）；(b) stencil constraints（形状库声明）；(c) 边 style 的 exit/entry
  键——mxGraph **没有一等 Port 实体**，端口是三种约定的叠加。

### 3.4 第三代路由：libavoid 避障正交（drawio 生产级答案）

- [事实] `w/js/libavoid-js/` 随附 **LGPL-2.1 LICENSE 全文**（`LICENSE:1-6`）；
  README 定位 "Obstacle-avoiding orthogonal edge routing for the Arrange >
  Layout > Orthogonal Routing menu item (and the `orthogonalEdge`
  childLayout)"，"libavoid never moves a vertex — it only computes edge paths
  that route around the vertices as obstacles"（`README.md:3-7`）。
- [事实] 工程链：Adaptagrams C++ 源码 + drawio 自写 embind 绑定 → Emscripten
  wasm2js 纯 JS（`-sWASM=0 -sDYNAMIC_EXECUTION=0 -sWASM_ASYNC_COMPILATION=0`）
  ——无 WASM、同步初始化、任意 CSP 兼容；C++ 异常被补丁改为显式控制流
  （编译期 `-DNDEBUG` + VPSC throw/catch 补丁，`README.md:9-38`）。
- [事实] 编辑器绑定两个入口：一次性菜单 run() + 常驻自动路由 installAuto-
  Routing（`libavoidRouting=1` 样式标志）；自动路由监听 GRAPH 事件
  （CELLS_ADDED/CELL_CONNECTED/CELLS_MOVED/CELLS_RESIZED）而**非模型 CHANGE**
  事件——「只对前向用户动作触发，undo/重放/远端变更不触发」，路由求解
  **停泊到 BEFORE_UNDO**（布局管理器跑完同步 childLayout 之后），结果并入
  同一可撤销编辑（`w/js/diagramly/LibavoidRouting.js:5-47` 头注释）；
  手工拖 waypoint 即清除标志让位用户（`LibavoidRouting.js:5-10`）。
  障碍间距默认 shapeBufferDistance=16 / idealNudgingDistance=14
  （`LibavoidRouting.js:47-49`）。
- [事实] 同一 routing core（libavoid-routing.js）被 **drawio-mcp 原样 vendor**
  用于其 app-server viewer 与 node tool server（`libavoid-routing.js:1-8` 头
  注释 + `LibavoidRouting.js:40`）——算法核心与 AI 调用面共享一份。

**小结**：路由资产三层——局部启发式（mxEdgeStyle 族，Apache-2.0，规格全、
规模小、无避障）、周界/端口数学（Apache-2.0，微件级可重写）、全局避障
（libavoid，LGPL-2.1，外置工程件）。drawio 的事件模型设计（GRAPH 事件 +
BEFORE_UNDO 停泊 + 单编辑合并）与 mxGraph 局部路由的「分段回退」策略都是
一手工程范本。

**可移植结论（Q3）**：(1) 五步路由管线（定端→算中段→浮端→去重→定界）
应整体采纳为 Rust 图视图的边更新骨架；(2) exit/entry 键值对 + 相对坐标 +
周界函数是端口模型的**最小完备集**，直接进 mdcore 图形页 schema；(3) 局部
正交路由自写（★★），避障需求出现时进程外集成 libavoid（LGPL 边界见 §7）
或等价 Rust 件（如未来出现）；(4) 「自动路由求解并入用户编辑的事务停泊」
模式可移植到 mdcore 的 op 通道（自动布局/自动路由产生的是普通 op，且与
触发它的用户 op 同事务撤销）。

## 4. Q4 stencil 形状库机制

### 4.1 XML 形状描述 = 一个微 VM 的完整规格

- [事实] `mxStencil` 的类 doc（102 行）就是完整规格书：`<shape name w h
  aspect strokewidth>`——w/h 定义**局部坐标系**（默认 100×100），aspect=
  variable|fixed（fixed 时 computeAspect 取 min 缩放并居中，`w/mxgraph/src/
  shape/mxStencil.js:10-25,494-533`）；`strokewidth=inherit` 语义 = 只随缩放
  不随 resize（`mxStencil.js:20-25`）。
- [事实] `<background>`（阴影源，仅一个 path/rect/roundrect/ellipse，无
  fill/stroke）与 `<foreground>`（真正绘制）分离（`mxStencil.js:44-57`）；
  绘制 = **HTML5 canvas 式状态机**：save/restore 栈、strokecolor/fillcolor/
  alpha/dashed/dashpattern/linejoin/linecap（`mxStencil.js:64-107`）。
- [事实] 路径原语 move/line/quad/curve/arc（arc 为 SVG arc 参数拷贝）/close
  + 复合原语 rect/roundrect/ellipse（必须跟 fill/stroke/fillstroke 之一）
  （`mxStencil.js:116-146`）；text 元素（str/x/y/align/valign/localized/
  vertical/rotation/align-shape/flip-shape，`allowEval` 时可内嵌求值函数）
  （`:148-174`）；image 元素（src 可 data URI）+ flip（`:176-189`）；
  **include-shape 引用子 stencil**（name+x/y/w/h，`:191-198`）——形状库可
  组合复用。
- [事实] `parseDescription`/`parseConstraints` 在构造期完成（`mxStencil.js:
  209-214`）；约束即端口：`<connections><constraint perimeter x y name/>`
  （`mxStencil.js:27-42` 规格节 + `:327-346` 解析），perimeter=1 时连接点
  沿中心-(x,y) 延长线投到周界（§3.3 已接线）。

### 4.2 渲染与导出的多后端

- [事实] 绘制入口 drawChildren → drawNode 逐元素分派到 **mxAbstractCanvas2D
  抽象画布**（begin/moveTo/lineTo/quadTo/curveTo/arcTo/rect/roundrect/ellipse/
  image/text/fill/stroke/fillAndStroke/setStrokeWidth/setDashed…，`mxStencil.js:
  463-479,643+` 各 canvas.* 调用实测）——同一形状描述可落到 SVG
  （mxSvgCanvas2D）、XML 导出（mxXmlCanvas2D）、图像导出，**形状定义与渲染
  后端正交**。
- [事实] 颜色占位：strokecolor=stroke / fillcolor=fill / font 引用当前 cell
  样式（`parseColor`，`mxStencil.js:540-566`）；`getStyleWithNone` 带未解析
  样式缓存（`:573-586`）——性能卫生细节。

### 4.3 交互行为的三条接线

- [事实] **端口**：stencil constraints → `getAllConnectionConstraints` →
  连接约束高亮/吸附（§3.3，`mxGraph.js:7300-7306`）。
- [事实] **属性面板**：形状可声明 `customProperties`（name/dispName/type/
  min/max/defVal），Format 面板据此生成编辑器（声明示例 `w/js/shapes-14-6-5.
  min.js:1` mxBpmnShape 头部 + `w/shapes/mxFlowchart.js:23-26`；消费端
  `w/js/grapheditor/Format.js:7126-7130,8060-8062`）——**形状自带属性
  schema**。
- [事实] **标签边界**：drawio 扩展 `<labelBounds if="…" x y w h>` 条件标签
  区（Shapes.js 重载 `mxShape.getLabelMargins` 解析，`w/js/grapheditor/
  Shapes.js:11-36`）；fixed-aspect stencil 的路由边界换算在 §3.1。

### 4.4 注册面三轨

- [事实] (a) **注册表**：`mxStencilRegistry.addStencil(name, stencil)` +
  `parseStencilSet`（`<shapes name="…">` 集、空格转下划线、全小写命名空间）
  （`w/mxgraph/src/shape/mxStencilRegistry.js` 全 56 行 + min bundle 内
  parseStencilSet 实现）；
  (b) **内联形状**：style 里 `shape=stencil(<base64-deflate-XML>)`，drawio
  覆写 `mxCellRenderer.createShape` 就地解压构建（`w/js/grapheditor/
  Graph.js:22182-22203`）——**形状可完全自包含于单条样式串**；
  (c) **程序化形状**：JS 类继承 mxShape + `mxCellRenderer.registerShape`
  （`Shapes.js:189,902,917,1054,1060` 等 100 处；`w/shapes/mx*.js` 库以
  mxBpmnShape 风格分发，含 customProperties——`w/shapes/mxFlowchart.js:1-31`）。
- [事实] 形状资产分发包：`w/js/stencils.min.js`（3.87MB）为
  `filename→base64` 字典（'alibaba_cloud.xml'→…，文件头实测），程序化库另
  打包 `w/js/shapes-14-6-5.min.js`（含 mxBpmnShape 等）；XML 形状库本体在
  `w/shapes/`（39 项：bpmn/er/ios7/mockup/pid2/rack + 33 个 mx*.js 库文件）
  与 `w/stencils/clipart`。〔J1b 复核修正：`w/shapes/` 39 项的准确构成 =
  bpmn/emoji/er/ios7/mockup/pid2/rack 七个子目录 + LICENSE + 31 个 mx*.js
  库文件（原文漏记 emoji 子目录、误记 33 个库文件）〕。【推断】资产以代码 bundle 而非独立 XML 目录分发，
  使「形状资产另行授权」的边界（§7）在源码树里更不显眼，引用时须自觉避开。

**小结**：stencil 机制 = 「局部坐标路径语言 + canvas 状态机语义 + 多后端
抽象画布 + 约束/属性/标签三线交互接线」的完整闭环，规格 100% 自文档
（类 doc 即规范），且经 drawio 二十年形状库规模验证。

**可移植结论（Q4）**：mdcore 结构化图形页的形状系统可直接按此规格设计
Rust 版：XML(或等价 DSL)→ 解析成「路径指令序列 + 状态栈程序」→ 求值到
SVG/canvas/导出多后端；端口=constraint 声明、属性=customProperties schema、
内联形状=压缩 payload 进样式——四个决策点各只需 <1 人周即可对齐业界事实
标准（drawio 形状生态的 XML 可作测试语料，但**资产本身另行授权，不可复制**
，见 §7）。

## 5. Q5 embed postMessage 协议

### 5.1 通道、握手与消息格式

- [事实] embed 模式 = iframe/新窗口宿主 ↔ 编辑器；编辑器就绪后**主动向宿主
  推 `init`**（proto=json 时 `{event:'init'}`，否则 ready 串），目标 `'*'`
  （`w/js/diagramly/EditorUi.js:29052-29054`）。
- [事实] 入站消息校验只做 **source 身份比对**（`evt.source != validSource`
  即丢，validSource = embedMessageSource || opener || parent，`EditorUi.js:
  27434-27441`）——无 origin 白名单；未知 action 回 `{error:'unknownMessage'}`
  （`EditorUi.js:28807-28812`）。【推断】信任模型 = 「能持有 opener/parent
  引用的窗口即宿主」；对高安全宿主（如 mdcore 嵌 AI 生成内容）应在外层
  iframe sandbox + 宿主侧 origin 过滤补强。
- [事实] `proto=json` 时消息体为 JSON 对象 `{action/event:…}`；非 JSON 时
  裸 XML 字符串也接受（`extractDiagramXml` 兼容 PNG data URL / SVG base64 /
  urlencoded / deflate 串，`EditorUi.js:27448-27483`）。

### 5.2 命令面（宿主→编辑器，25 个 action）

- [事实] 全集（按分发序）：`dialog`（27517）`layout`（27531）`prompt`
  (27548) `draft`(27573) `template`(27618) `textContent`(27694) `status`
  (27702) `spinner`(27724) `exit`(27739) `viewport`(27745) `invokeAction`
  (27761) `resetEditor`(27772) `fit`(27787) `viewbox`(27801)
  `fullscreenChanged`(27819) `snapshot`(27848) `export`(27854) `load`
  (28239) `merge`(28664) `patch`(28693) `getDiff`(28736) `resetDiff`(28755)
  `remoteInvokeReady`(28792) `remoteInvoke`(28797) `remoteInvokeResponse`
  (28802)（行号 = `EditorUi.js` dispatch 链实测；括号内数字为行号）。
- [事实] `load` 的选项面即「数据面+行为面」：xml/autosave/diffSync(+patchOnly)/
  exportProtocol/zoomEvents/modified/saveAndExit/toSketch/layout/sourceMetadata
  （`EditorUi.js:28239-28260`）——**layout 可以随 load 一次性执行**（preset
  名或 custom-layout JSON，与 §2.5 同一解析器）；plantuml/mermaid 源码直接
  可 load（本地转换器 drawio-plantuml，无外部服务器，`EditorUi.js:28571-
  28599`）。
- [事实] 加载即隔离：每个 embed 编辑器实例获得唯一 cell-ID 前缀
  （`model.prefix = Editor.guid()+'-'`），防同文件多实例 ID 冲突
  （`EditorUi.js:28835-28841` 注释：otherwise "a merge/diff treats
  different new cells as the same cell"）。
- [事实] `export` 支持 png/xmlpng 等（含把 xml 内联回 PNG 的往返，
  `EditorUi.js:27854-27900`）。

### 5.3 事件面（编辑器→宿主）

- [事实] 事件全集（postMessage `{event:…}`，grep 计数实测）：`init` `load`
  `autosave`（changeListener 里 `createLoadMessage('autosave')`，`EditorUi.js:
  28862-28872`）`save` `exit`（:26619）`export`（:26637）`template`（:27634,
  27667）`draft`（×4，:27583-27612）`patch` `merge` `resetDiff` `getDiff`
  `resize`（×3）`shortcut`（×2）`prompt`/`prompt-cancel` `textContent`
  `openLink`（:29065）`remoteInvoke`/`remoteInvokeResponse` `configure`
  （App.js:1744）；`snapshot` 动作对应 sendEmbeddedSvgExport（:27848-27852）。
- [事实] diffSync 模式：load 后克隆 shadow pages 为基线（`EditorUi.js:
  28857-28860`），宿主可 `getDiff`/`patch`/`merge`/`resetDiff`——**以页面
  快照 diff 为协同单元**，autosave 事件携差异而非全量（:28736-28790 分支）。

### 5.4 remoteInvoke：受限 RPC 白名单

- [事实] 白名单仅 4 函数：getDiagramTextContent（无域限）；getLocalStorageFile/
  getLocalStorageFileNames/setMigratedFlag（**allowedDomains=['app.diagrams.net']
  origin 精确匹配**）（`EditorUi.js:31656-31660`）；调用侧 own-property 查
  白名单防原型链冒名（`:31775-31777`）、函数参数须数组（`:31788+`）、异步
  函数以末两参 success/error 回调约定（`sendResponse`，`:31750-31767`）；
  callbackId 必须为真实数组索引（`__proto__` 防御注释，`:31678-31690`）。

### 5.5 对 dsh 调用面设计的参照（协议分层归纳）

- [事实] 三面分层清晰：**命令面**（24 action，含视图控制 fit/viewbox/
  viewport 与动作转发 invokeAction）、**事件面**（init/load/autosave/save/
  export/exit/template/draft/diff 族 + 快捷键回放 shortcut）、**数据面**
  （xml/png/svg/plantuml/mermaid 多形态输入 + export 多格式输出 + diff/patch
  增量）；配置面独立（configure action + {event:'configure'} 拉取，
  `w/js/diagramly/App.js:1688-1744`）。
- [事实] 同一编辑器内核服务三种宿主形态：iframe embed（本节）、桌面 CLI
  （--layout 等走同一 spec 解析器，§2.5）、drawio-mcp（算法核心 vendor 共享，
  §3.4/§2.1）——**「编辑器即服务」的多调用面先例**。

**小结**：embed 协议是「画布外置化」最成熟的业界范本：init 主动握手、
25 命令/约 20 事件/多格式数据、diff 增量通道、白名单 RPC、ID 前缀隔离、
load 期布局钩子；安全模型偏「宿主可信」。（J1b 复核：dispatch 链
`data.action ==` handler 实测 25 个，§5.2 枚举列表 25 项本身完整无误，
原文计数「24」系漏数一项；R4-fix 事件面口径注：EditorUi.js `event: '…'`
字面量 distinct 实测 18（含 remoteInvoke/remoteInvokeResponse 两 RPC 帧，
剔响应帧为 17），另有动态构造 createLoadMessage 七名（export/layout/fit/
viewbox/load/autosave/zoom）新增 5，App.js 另有 configure/ready 2（
App.js:1744/:6657），全集并入 ≈23-25——「约 20」量级结论维持。）

**可移植结论（Q5）**：mdcore/dsh 的画布外置面可直接套此分层：iframe 宿主
+ postMessage JSON 协议（命令/事件/数据三面 + 主动 init 握手 + unknownMessage
错误回执）；Rust 侧需补 drawio 没做的两件事——**origin 白名单**与
**消息 schema 校验**；autosave 的 diffSync（快照 shadow + getDiff/patch/
resetDiff 四操作）是「无 CRDT 协同」的最小可行方案，可与 collab-a 的 Yjs
子集路线二选一做降级通道。

## 6. Q6 maxGraph JS→TS 重写兼容策略

### 6.1 定位与兼容锚点的官方口径

- [事实] 自述：mxGraph 2020 归档后的活跃继任，「preserves mxGraph's
  comprehensive features and **XML compatibility** while modernizing with
  native TypeScript, modular architecture, and smaller bundle sizes」
  （`maxgraph/README.md:15-16`）；「XML import/export — Compatible with
  mxGraph format for data persistence」（README Features 节）。
- [事实] 兼容性主张是**代码行为锁**而非口号：`m/../__tests__/serialization/
  serialization.xml.mxGraph.test.ts:21+` 直接导入 mxGraphModel/mxCell/
  mxGeometry/mxPoint XML 断言行为；`__tests__/serialization/codec/mxgraph/`
  目录专项；发布包随附 `xsd/stencils.xsd`（package.json files 含 xsd）。
- [事实] API 层坦承不完全兼容：「The maxGraph APIs are not fully compatible
  with the mxGraph APIs, but … close」（`packages/website/docs/guides/
  migrate-from-mxgraph.md:31`）；主要断代 = IE/VML/旧 Edge 支持
  （`migrate-from-mxgraph.md:34`）；迁移指南**停在 0.18.0**、明示以
  CHANGELOG 为准（`migrate-from-mxgraph.md:13-21` danger 块）。
- [事实] 版本现状：v0.25.0（2026-10-06 发布，`CHANGELOG.md:12-14`），**未到
  1.0**，Unreleased 节持续登记 breaking changes；TS 下限 3.9（README.md:74）；
  ES2020 + ESM/CJS 双格式（README.md:73；`m/package.json` main/module/types
  双出口）；**零运行时依赖**（package.json dependencies 为空，实测）。

### 6.2 重写策略五支柱（ADR 在档）

- [事实] **支柱一：mixin 拆 God Object（保 API 兼容）**。动机：mxGraph.js
  v4.2.2 为 13,229 行 God Object；「The obvious approach, extracting
  collaborator classes and delegating … breaks every existing call site」而
  当时卖点是 drop-in successor——决策：mixins（实现文件 + `.type.ts` 声明合
  并对），调用点零改动（`docs/adr/0001-use-mixins-to-split-the-graph-class.md:
  20-40`）。决策时间 2021-08-01（commit bc400a3ae），ADR 为回溯文档
  （`0001:3-6`——J1b 复核修正：日期/commit 在第 6 行，原文误锚 :8-13，
  该区间实为 Analysis basis/Scope 节）。
- [事实] **支柱二：plugin 化（换 tree-shaking 与扩展点）**。mixin 四宗罪
  在案：No tree-shaking / No per-instance state / No extension point / A
  growing class again（`docs/adr/0002-use-plugins-for-optional-and-new-
  features.md:22-28`）；七个 handler（cellEditor/connectionHandler/
  graphHandler/panningHandler/popupMenuHandler/selectionCellsHandler/
  tooltipHandler）从构造器协作者转注册插件（`0002:29-33`）。0.25.0 继续
  加码：handler 工厂方法整体移出 AbstractGraph 入 SelectionCellsHandler，
  换 factory setter（CHANGELOG.md:21-45）。
- [事实] **支柱三：分层入口**。`Graph` = 自动注册全部内置（「Good for
  evaluation and prototyping, but not recommended for production」），
  `BaseGraph` = 零注册、按需 tree-shake（`m/view/Graph.ts:31-40`；
  `m/view/register-style-elements.ts:33-39` 明言 registering everything
  defeats tree-shaking）。
- [事实] **支柱四：格式兼容优先于 API 兼容**。XML codec 全保（§6.1 测试锁）
  + 序列化新 API `ModelXmlSerializer` 标 experimental（`m/serialization/
  ModelXmlSerializer.ts:37-58`）；CHANGELOG 反复说明 XML 元素顺序变化不破坏
  解码（decode 按 `as` 属性匹配、与顺序无关，CHANGELOG.md:52-54）。
- [事实] **支柱五：类型即文档**。TS 类型复用 typed-mxgraph（MIT）项目成果，
  NOTICE 逐条致谢（`maxgraph/NOTICE:1-20`）；0.25.0 把 CellStateStyle 等
  type alias 改 interface 以支持**module augmentation**（应用自声明样式键，
  CHANGELOG.md:55-65）。
- [事实] 许可演进史在档：initial contributions 在「modified Apache-2.0」下，
  后经贡献者逐人重授权回标准 Apache-2.0（issue #89，NOTICE 全文名单）——
  mxGraph 上游曾用带附加条款的 Apache-2.0，maxGraph 完成了去附加条款化。

### 6.3 债与未竟

- [事实] 遗存 `m/editor/`（Editor/EditorKeyHandler/EditorPopupMenu/
  EditorToolbar）与 `m/gui/`（MaxForm/MaxLog/MaxPopupMenu/MaxToolbar/
  MaxWindow）——mxEditor 生态仅翻译未重构【推断：无对应 ADR/CHANGELOG
  重构计划，属「翻译完成、现代化未做」段】。
- [事实] 测试面：63 个 `*.test.ts` 测试文件、130 个 describe 块（实测
  计数；J1b 复核修正：原文记 67/131；R4-fix 口径名修正：67 实为
  「`__tests__/` 目录下全部 .ts」口径（67 = 63 个 *.test.ts + 4 个非
  test 后缀辅助文件，即 `__tests__/utils.ts`、`__tests__/serialization/
  utils.ts`、`__tests__/serialization/codec/shared.ts`、`__tests__/view/
  no-global-state-for-mixin-properties.ts`），文件名含 test 的 .ts 实测亦
  为 63，J1b 原记「文件名含 test 的 .ts 全量」口径名不确）——与
  tldraw 463 spec、HyperFormula 私有套件相比为轻量级，行为锁集中在序列化
  与 codec。项目自述「a few adjustments still required to match the
  behavior of mxGraph」（README Project status 节）。
- 【推断】兼容优先级实际排序 = XML > 概念 > API 名。当 mixin→plugin 迁移
  与 API 兼容冲突时（handler 工厂搬迁即破坏子类覆写），项目选择现代化——
  「drop-in successor」承诺已实质让位于长期可维护性。

### 6.4 与 Univer 重写案例互证（交叉引注 office-a）

maxGraph（JS→TS 同构重写）与 Luckysheet→Univer（整体推倒重写）构成
「重写时保什么」的两极样本：

| 对照项 | maxGraph（mxGraph→TS） | Univer（Luckysheet→Univer） |
|---|---|---|
| 数据格式 | **保留**（XML 兼容为第一锚点 + 测试锁） | **推倒**（新快照 JSON，无迁移器，office-a §1） |
| API | 名近而不兼容（migrate 指南自认 0.18 后停更） | 全新 Facade（office-a §7） |
| 架构手段 | mixin 保调用点 → plugin 换扩展点（ADR 0001→0002 演进） | 三层命令/快照即模型（office-a §0/§1） |
| 兼容投入重心 | 序列化行为锁（serialization tests） | 无迁移投入，押注「字段全 optional + 默认值合并」（office-a §1，被其自判为债） |
| 共同教训 | —— | —— |

两案互证结论（【推断】，供 mdcore 借鉴）：(1) **「格式兼容」是重写中唯一
值得花大力气保的东西**——它决定存量生态资产（drawio 的亿级 XML 文件/
Univer 的用户文档）能否延续；maxGraph 把宝押对了，Univer 的无迁移器被
office-a §1 判为 schema 漂移风险；(2) **API 兼容可以放弃，但要在 ADR 与
CHANGELOG 里显性化**（maxGraph 的 mixin→plugin 三部曲 = 每步有文档）；
(3) 办公线对位：若 mdcore 表格引擎立项，「能读写 xlsx/mxGraphModel 级
存量格式」应列为 P0 而非「API 像 Univer」——这正是 office-a §13-9
「schema version + 迁移器」建议的镜像面。

**小结**：maxGraph 的重写策略 = 「XML 兼容为锚、mixin 过渡、plugin 终态、
类型即文档、分层入口买 tree-shaking」，代价是 5 年仍处 0.x 与指南停更；
其对 mdcore 的价值是**把「怎么体面地重写一个 13k 行 God Object」写成了
三份可抄的 ADR**。

**可移植结论（Q6）**：mdcore 任何「自研引擎兼容业界格式」立项（图形页
mxGraphModel、表格 xlsx）应照抄其兼容优先级排序 + 行为锁测试形态（用真
实存量文件做 golden tests）；mixin→plugin 的两步走（先保调用点拆类、再换
扩展点付破坏成本）适用于 mdcore 内部任何 God Object 拆分场景。

## 7. 许可核验定谳（净室合规范畴）

| 资产 | 许可 | 一手证据 | 净室取用边界 |
|---|---|---|---|
| drawio 源码 | Apache-2.0 | `drawio/LICENSE`（200 行标准全文）+ `README.md:9` | 可读可参考可（法务口径下）引用；本 pass 零复制 |
| drawio 图标集/stencil 库/模板 | **自定义条款（非 Apache）** | `README.md:11-15` + `w/shapes/LICENSE` + `w/stencils/LICENSE`（同文随资产分发） | 禁止作为软件资产复制进产品；「derivatives …including traced reproductions, substantially similar visual representations, or AI-generated images created using these icons as reference or training input」均受限；端用户导出物例外 |
| drawio 内第三方 JS | 仓方声明全兼容 Apache-2.0 | `README.md:17` | —— |
| ELK 桥（drawio-elk.min.js） | 上游 Eclipse Layout Kernel | bundle 内 "Eclipse Layout Kernel" 字样（字符串实测）+ `ElkLayout.js:26-27`（源在 jgraph/drawio-elk 仓） | 【推断/外部】elkjs 上游为 EPL-2.0（弱著佐权），仓内未随附其许可文本；算法思想可参考，bundle 与其衍生代码不可入 mdcore |
| libavoid | LGPL-2.1 | `w/js/libavoid-js/LICENSE:1-6`（全文随仓）+ README.md:31 | LGPL 边界：不可复制源码进闭源仓；进程外/动态链接口径需法务确认 |
| Bridge.NET / OrgChart.Layout | Bridge.NET 头注指向其 LICENSE.md；OrgChart.Layout 版权 2019-2020 JGraph | `bridge.min.js:1-5`、`mxOrgChartLayout.js:2` | 仓内 Apache-2.0 覆盖范围内【推断：Bridge.NET 运行时为 Apache-2.0 系（外部事实）】 |
| maxGraph | Apache-2.0（+NOTICE） | `maxgraph/LICENSE`（201 行）+ `NOTICE:1-20`（含 typed-mxgraph MIT 致谢 + mxGraph 衍生声明） | 同 drawio 源码口径 |

定谳：与规划预判一致——drawio/maxGraph 的**代码与算法层**是 Apache-2.0
净室友好区；**形状/图标/模板资产层另有现场约束条款**（Atlassian 排他 +
衍生视觉表示限制，条款之宽覆盖 AI 训练输入，须严格避开）；两个外置算法
桥（ELK【推断 EPL-2.0】/libavoid LGPL-2.1）是许可凹地，只能做协议级/进程
级集成参考。本 pass 全部产出为行为与规格描述，零代码复制，净室合规。

## 8. 对工作台的可移植结论汇总（按优先级）

1. **P0｜样式串边界层 + 类型化内存表示**（Q1）：mxGraphModel 的 cell 一元论
   /exit-entry 键/relative 几何三件套直接进 schema；style 内存态改强类型 map。
2. **P0｜布局重写路线表**（Q2.4）：简单族+FR+树自写（★★★），Sugiyama 四
   阶段按 maxGraph 规格重写（★★，调优集中在 CoordinateAssignment），ELK/
   libavoid 永远外置（★）。
3. **P0｜布局即数据**（Q2.5）：layout spec 可序列化、五入口一数据、childLayout
   挂容器样式——Rust enum + serde 直接建模。
4. **P0｜五步边更新管线 + 端口三件套**（Q3）：定端→算中段→浮端→去重→定界
   的视图骨架；perimeter 函数族 + connection constraint + exit/entry 键为
   端口语义最小完备集。
5. **P1｜stencil VM 规格移植**（Q4）：局部坐标路径语言 + canvas 状态机 +
   多后端抽象画布 + constraint/customProperties/labelBounds 三线接线；
   drawio 形状 XML 只作测试语料，资产不复制。
6. **P1｜embed 协议分层**（Q5）：命令/事件/数据三面 + init 握手 + diffSync
   四操作（无 CRDT 协同降级通道）+ 白名单 RPC + ID 前缀隔离；Rust 侧补
   origin 白名单与 schema 校验。
7. **P1｜自动路由事务停泊**（Q3.4）：GRAPH 事件收集 + BEFORE_UNDO 求解 +
   单编辑合并——op 通道内自动布局/路由与用户操作同事务撤销。
8. **P2｜格式兼容为重写第一锚点**（Q6）：golden tests 锁序列化行为；
   「mixin 保调用点 → plugin 换扩展点」两步走拆 God Object；与 office-a
   §13-9 的迁移器建议合并为 mdcore 重写纪律双条。
9. **P2｜drawio-mcp 生态信号**：jgraph 自家 MCP 服务器与主仓共享 ELK/libavoid/
  XML 往返核心（`ElkLayout.js:4-6`、`libavoid-routing.js:1-8`、`EditorUi.js:
  14290-14304`）——结构化图形的「AI 可调用面」已有厂商级先例，dsh 图形页
  插件设计应与其对位（外部仓事实，本轮未克隆深读，留 R2 综合或后续 pass）。

## 9. 六问题结论一览

| 问题 | 一句话结论 | 关键锚点 |
|---|---|---|
| Q1 数据模型 | 薄 cell 图 + 扁平样式串 + 反射式 codec：vertex/edge 二值 cell、层即普通 cell、高级语义全寄生在 style 约定上；无版本/无迁移器 | `w/mxgraph/src/model/mxCell.js:67-77`、`view/mxStylesheet.js:217-280`、`io/mxCellCodec.js:167-262` |
| Q2 布局族 | mxGraph 原生族在 drawio 已被 ELK 桥整体取代（旧名映射兼容），活规格存于 maxGraph（Apache-2.0）；简单族/FR/树可直接按规格重写，层级布局=dot 优先级法直译、调优重，ELK/libavoid/orgchart 外置 | `Graph.js:2018-2143`、`HierarchicalLayout.ts:505-509`、`GraphHierarchyModel.ts:255-257`、§2.4 评级表 |
| Q3 连线路由 | 五步视图管线 + 局部启发式边样式族（OrthConnector 无避障、有分段回退）+ 周界/约束/exit-entry 端口三件套；生产级避障外包 libavoid（LGPL，事件停泊 + 单事务合并） | `mxGraphView.js:2547-2590,2821-2882`、`mxEdgeStyle.js:1097-1206`、`mxPerimeter.js:68-152`、`LibavoidRouting.js:5-47` |
| Q4 stencil | XML 形状 = 局部坐标路径语言 + canvas 状态机 + 多后端抽象画布，规格全自文档；交互三线（constraint 端口/customProperties 属性面板/labelBounds）+ 注册三轨（registry/内联 stencil()/JS 类） | `mxStencil.js:10-214,463-533,643+`、`Graph.js:22182-22203`、`Format.js:7126-7130` |
| Q5 embed 协议 | 25 命令/约 20 事件/多格式数据的成熟三层协议：init 主动握手、diffSync 增量、白名单 RPC、ID 前缀隔离、load 期布局钩子；安全仅 source 校验 | `EditorUi.js:27434-27441,28239-28260,28835-28872,29052-29054,31656-31660` |
| Q6 maxGraph 重写 | 「XML 兼容为锚（测试锁）+ mixin 保调用点 + plugin 换 tree-shaking + 分层入口 + 类型即文档」，5 年仍 0.x；与 Univer 互证：重写值得保的只有格式兼容，API 兼容可弃但须 ADR 显性化 | `README.md:15-16`、ADR 0001/0002/0003、`migrate-from-mxgraph.md:13-34`、`serialization.xml.mxGraph.test.ts:21+` |

---

**附：锚点密度与纪律声明**。本报告证据锚点约 120 处（`路径:行号`，全部
为本次会话 grep/Read 实测行号；`w/`、`m/` 前缀见头部声明）——J1b 复核
正则实计（`文件名.扩展名:行号` 形态）原文为 148 处（J1b 补丁后全文 150
处），原记偏保守；【推断】实质
标注 14 行（含表内 5 行；R4-fix 口径统一）：§1.2×1、§2.2×2、
§2.3 表×2、§2.4 表×1、§4.4×1、§5.1×1、§6.3×2、§6.4×1、§7×3
（许可表 2 行 + 定谳段 1）——纪律完整无漏标（J1 原记「正文 10 处 + 表内
2 处」漏计 §2.3 表行与 §7 定谳段，J1b 原记「含变体共 17 行」误把附录
自引 3 行计入，均废弃，以本口径为准）；外部仓事实
（drawio-elk / drawio-libavoid / drawio-mcp / Bridge.NET 运行时许可）均标注
未克隆待核。净室纪律：只读不复制（唯一代码围栏为 §0 自绘 ASCII 架构图，
原创编排）；零 git commit；克隆工作树保留于 analyses-workdir/drawio 与
analyses-workdir/maxgraph 供复审抽验。

---

**附二：J1b 补救复核记（2026-10-11）**。前代理 J1 在网关超时前落盘本报告
但未及登记 worklog；J1b 接手做完成度审计与增量修补（只读克隆、零 commit、
不重写正文，修正处均以「J1b 复核」标记就地标注）。复核范围与结论：

1. **六问题规格对照**：Q1-Q6 + 许可核验（§7）+ 每问题小结/可移植结论 +
   【推断】纪律全部在位，无缺失章节；本 pass 核心产出（§2.4 重写可行性
   评级表，13 行算法/资产 × 规格/许可/文献/净室安全/评级五维）完整。
2. **锚点抽验**：直读验证约 60 处（Q1 codec/style/几何、Q2 四阶段/参数/
   行数、Q3 五步管线/OrthConnector/周界/约束/libavoid、Q4 stencil 规格/
   三轨注册、Q5 dispatch 链 25 个 action 行号逐一比对/remoteInvoke 白名单/
   ID 前缀、Q6 README/ADR/CHANGELOG/测试锁/NOTICE、§7 许可全文）——除
   下述计数项外**全部命中**，含 Graph.js:2018-2019/2031-2039/2096-2143
   （ELK 旧名映射）、EditorUi.js 25 个 action 行号全数精确、ElkLayout
   bundle 头注释逐字符一致、CHANGELOG 0.25.0 三处破坏性变更原文一致。
3. **计数修正 8 处**（详见就地标记）：mxGraph 内嵌 138→115 文件；Graph.js
   36,792→36,791 行；EditorUi.js 35,276→35,275 行；「14 布局类」→「14 类
   文件含 2 管理器」；`w/shapes/` 漏记 emoji 子目录、33→31 个 mx*.js；embed
   action 24→25；maxGraph 测试 67/131→63/130（口径注记）；ADR 0001 锚
   :8-13→:3-6。锚点总数原记「约 120」→ 正则实计 148。
4. **许可纪律**：§7 定谳复核无误——drawio/LICENSE 200 行 Apache-2.0 全文、
   maxgraph/LICENSE 201 行、libavoid-js/LICENSE LGPL-2.1 随仓、README.md:9-17
   资产条款（Atlassian 排他 + 衍生视觉表示限制）原文核对一致；ELK
   【推断 EPL-2.0】的外部标注纪律保持（bundle 内仅 "Eclipse Layout
   Kernel" 字样 + `@drawio/elk 0.1.0+commit.c6b8e34 (built 2026-09-30)`
   头注释实测，仓内确无其许可文本）。

J1b 完成度定谳：**六问题 + 许可 + 纪律 = 全数完整（6/6），无需补研究**；
本报告收口条件达成，登记 worklog（r4-canvas-c）。
