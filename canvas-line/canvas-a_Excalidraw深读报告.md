# canvas-a：Excalidraw 深读报告（11 维全量）

> **净室分析报告**（线 A 画布 P0 第一篇）。日期：2026-10-10。
> 对象：excalidraw/excalidraw @ `4c00f31`（master 浅克隆），npm 包
> `@excalidraw/excalidraw` 0.18.0，MIT。方法：四路并行勘察代理（元素模型
> 与编辑内核 / 渲染与性能 / 持久化与协同 / 插件·AI·安全·质量·合规）+
> 主代理交叉综合；所有结论标注证据文件路径，【推断】= 分析员判断。
> 纪律：只读不复制；路径约定 `E/` = packages/element/src、`X/` =
> packages/excalidraw、`APP/` = excalidraw-app。
>
> 一句话定性：**React 单机优先的白板引擎，元素级 LWW + fractional
> indexing 的"薄协同"，渲染走双 canvas + per-element 位图缓存的全量
> 重绘路线**——工程纪律（不可变元素 + 集中 mutation + 版本三元组）是其
> 最值得移植的资产。

## 0. 架构速览

```
monorepo: packages/{excalidraw, element, common, math, fractional-indexing,
                    utils, laser-pointer} + excalidraw-app（宿主）
npm 包（packages/excalidraw）＝ 编辑器内核 + React 组件 + 数据进出
  ├─ E/ 元素模型（types/mutateElement/newElement/Scene/store/delta）
  ├─ X/actions/ 命令系统（94 静态注册 + 2 工厂） X/history.ts 撤销栈
  ├─ X/renderer/ + X/scene/Renderer 双 canvas 渲染 X/data/ 序列化与恢复
  └─ 无协同代码（导出 reconcileElements 等原语，协同全在 excalidraw-app）
excalidraw-app ＝ excalidraw.com 宿主：collab/（socket.io 盲转 + E2E 加密）
  + firebase（Firestore 单文档密文场景 + Storage 图片）
```

## 1. 数据模型与持久化完整性

**元素类型体系**（E/types.ts:223-234）：`_ExcalidrawElementBase` 只读基座
（~30 字段）派生 12 成员判别联合（rectangle/diamond/ellipse/stickynote/
text/line/arrow/freedraw/image/frame/magicframe/iframe/embeddable）；
之上叠**语义子集组合**（BindableElement/TextContainer/Rectanguloid 等，
纯类型代数非继承）与**品牌类型**（`Ordered<T>` / `NonDeleted<T>` / 4 种
SceneElementsMap——编译期筛选零运行时开销，E/types.ts:241-251,448-478）。
守卫层 E/typeChecks.ts ~45 个谓词。

**关键不变量**（E/types.ts:40-87 注释完备）：
- `id` nanoid 永不更新；`seed` 跨渲染/跨客户端稳定的手绘噪声种子；
- **版本三元组**：`version`（每次变更 +1，协同 LWW 比较）+
  `versionNonce`（每次变更随机重生，版本撞车时**nonce 小者胜**的确定性
  平局裁决，X/data/reconcile.ts:23-44）+ `updated`；
- `created` 排除在 ElementUpdate 之外（复制即新 lifetime，
  E/mutateElement.ts:24-30）；`isDeleted` 软删除 tombstone 留数组保
  协同完整性（X/data/restore.ts:583-589）；
- `index` = fractional index 字符串（rocicorp vendored，CC0），
  **数组序是缓存、index 是真源**（E/fractionalIndex.ts:27-42 设计注释）。

**修改纪律**（双通道非纯函数式）：不可变通道 `newElementWith`（浅比较
无变化返原引用，有变化 bump 版本）用于结构性变更；就地通道
`mutateElement`（cast Mutable 直接改 + 自动版本提升 + w/h/points/fileId
变更时失效 ShapeCache）用于高频拖拽（E/mutateElement.ts:37-181）；
`ElementUpdate` 类型层面锁死 id/updated/created 不可写。所有变更汇到
单一提交边界 `store.commit()`（X/components/App.tsx:4483）。

**持久化形态**：`{type:"excalidraw", version:2, source, elements[],
appState, files?}`（X/data/json.ts:52-75）；elements 数组序即 z 序缓存；
导出剥离只被已删元素引用的 files；**元素纯 JSON 无派生缓存无本地
状态**（E/types.ts:218-222 注释即规范）。本地 localStorage 持久化剥离
tombstone（APP/data/LocalData.ts:90-93），协同 tombstone 保留 24h
（APP/data/index.ts:46-56）。

## 2. 渲染管线

**画布分层**（2 常驻 + 1 条件 + 2 非 canvas）：静态层（元素/网格/frame
裁剪）+ 交互层（选框/手柄/远程光标/snap 线）+ NewElementCanvas（创建中
元素隔离）+ SVG 轨迹层（laser/lasso/eraser 笔迹动画）+ DOM 覆盖层
（contenteditable 文本编辑 + embeddable iframe 懒挂载）。

**单帧流程**：mutation → sceneNonce（随机数专职失效）→ React setState →
`Renderer.getRenderableElements()`（按 nonce+视口 memoize）→ **线性扫描
全量元素做视口 AABB culling** → React.memo 门控（canvasNonce 无变化
整层跳过）→ `throttleRAF` 每帧至多一次绘制 → bootstrapCanvas（DPR
transform）→ 逐元素 `renderElement`。**无脏区局部重绘，一律全画布重绘**
（X/renderer/staticScene.ts:526-545；X/components/canvases/*.tsx memo）。

**粗略渲染与排版**：ShapeCache = WeakMap<element,{shape,theme}> 单例
RoughGenerator，seed 驱动可复现（E/shape.ts:83-193）；freedraw 不走
roughjs（perfect-freehand + Path2D 缓存）；**文本自制排版**（
textWrapping.ts 模拟浏览器软换行含 CJK/emoji 规则 + 离屏 canvas 逐字符
宽度缓存）；编辑态 contenteditable 覆盖层与画布变换对齐。

**缓存体系**（全 WeakMap 无容量上限靠 GC）：ShapeCache（几何量变更
失效）/ per-element 离屏位图（padding 按元素类型差异化；zoom 手势期
复用旧位图 + 300ms debounce 重建）/ imageCache / Renderer memoize /
选区缓存。**平移不动缓存**（拖拽 60fps 根基），仅几何量变更才重生成
（E/mutateElement.ts:133-140）。

**像素工程**：snapScrollToDevicePixels + 1:1 blit（关 smoothing）+
SNAP_TIE_BIAS 半像素平局（X/renderer/helpers.ts:36-63）。

## 3. 编辑内核

**Action 系统**（X/actions/types.ts:163-223）：接口 = name + perform +
keyTest（函数式快捷键谓词，可读 appState/平台/焦点）+ predicate +
trackEvent + PanelComponent（**UI 即 action**）；运行时 96 个（94 静态
注册 + undo/redo 工厂）；模块顶层副作用注册（X/actions/register.ts）；
keyboard/UI/API/命令面板/上下文菜单五入口统一 `executeAction`；多键位
匹配静默取消告警（X/actions/manager.tsx:98-119）。

**撤销栈**（X/history.ts + E/store.ts + E/delta.ts）：**快照 diff →
可逆 delta 混合式**（非命令模式非全量快照）。Store 单基线快照 + **三态
捕获标签**（ActionResult.captureUpdate）：`IMMEDIATELY`（入栈）/
`NEVER`（远端更新/初始化不入栈）/**`EVENTUALLY`（快照不前移——拖拽
中间态自动聚合进下一次捕获，天然手势合并）**。粒度 = 元素级字段 patch
（added/removed/updated）+ appState 白名单；入栈即反转
（inverse-delta）；无容量上限；纯 appState 变更不清 redo 栈；
undo/redo 每次视为新变更重新 bump 版本（协同端正确接收撤销，
history.ts:29-34）；手势进行中拒绝 undo/redo。

**选区模型**：双轨制——`appState.selectedElementIds` id 真值表 +
`selectedGroupIds`；框选矩形是**不入 scene 的影子元素**（复用几何代码
但制造边界特判）；组展开/帧互斥/绑定文本过滤（E/selection.ts,
E/groups.ts）；Scene.getSelectedElements 带 options-hash 备忘。Lasso
（路径 simplify 降采样 + 包围/相交双判定 + 视口缓存失效重建）与
Eraser（AnimatedTrail 基类共享；**拖动只算预览集、pointer-up 才提交
软删**的两段式交互）。

## 4. 格式进出保真

**读侧**（X/data/restore.ts，1416 行）：**恢复时规范化而非 schemaVersion
迁移链**（全库无 schemaVersion 字段）——任意旧形状就地升级（font 字符串
拆分 / strokeSharpness→roundness / 绑定 schema v1→v2 几何重投影 /
AppState legacy 迁移表）；未知字段 spread 保留（前向兼容，
restore.ts:500-508）；**单元素 restore 失败只丢该元素不崩场景**
（restore.ts:972-985）；缺字段逐项默认值回填；重复 id 重发；断裂绑定
二次遍历修复；超大线元素（>75000px）重置防渲染冻结（issue #11497）；
导入与本地版本冲突 `bumpElementVersions`（issue #3795）。

**写侧**：导出面 PNG/SVG/剪贴板；**场景回嵌**——PNG tEXt chunk（iEND
前插入）与 SVG metadata 注释包裹 base64，产物即 `.excalidraw.png/.svg`
可再编辑；SVG 内联 @font-face（内置 harfbuzz+woff2 wasm 子集化 worker，
packages/excalidraw/subset/）；坐标精度 2 位小数；导出走矢量直绘绕过
位图缓存。

**加密文件**：AES-GCM 128 + 12B IV + JWK 密钥；分帧容器
[元数据 JSON][iv][GCM(deflate(内容))]（X/data/encode.ts:322-354）；
密钥放 URL hash **永不发给服务器**（APP/data/index.ts:283-285）。

## 5. 协同协议

**定性：非 OT 非 CRDT——「加密快照广播 + 元素级 LWW + 确定性平局 +
周期全量重同步」**：
- socket.io 盲转中继（服务器看不到明文），事件族 SCENE_INIT/
  SCENE_UPDATE/MOUSE_LOCATION/IDLE_STATUS/USER_VISIBLE_SCENE_BOUNDS；
- 增量 = **元素版本差集**（只发 version > 已广播水位的元素）而非操作
  日志；每 20s 全量重播补丢消息（SYNC_FULL_SCENE_INTERVAL_MS）+
  Firestore 单文档落库（**存储事务内重跑同一 reconcile 规则兜持久化
  竞态**，APP/data/firebase.ts:187-247）；
- 收敛规则三行（reconcile.ts:23-44）：version 高者胜；同版本 nonce 小
  者胜；所有端同规则 → 收敛。**字段级不合并整元素覆盖**；本地正在编辑
  的元素无条件本地胜（UX 优先于严格收敛的显式取舍）；
- 排序收敛：`orderByFractionalIndex` + id 破平 + `syncInvalidIndices`
  修复；
- 删除并发：tombstone 参与同一 version 竞争；
- 房间权限：**能力链接即全部权限**（roomId 10 字节 hex + 128 位密钥
  base64url），无账号无 ACL；Firestore 规则公开读写、机密性全靠
  URL 密钥；`allow list: false` 禁枚举；
- 可替换性：npm 包对服务端无知，协同全在 app 层，接口收敛为
  「广播函数 + KV 读写」。

**离线语义**：无离线队列；断线编辑留本地靠版本水位 + 重连后差集 +
20s 全量重播；三重初始化兜底（socket 失败拉库 / first-in-room 拉库 /
5s 超时拉库）；协同期间暂停 localStorage 保存防旧态覆盖。

## 6. 插件与扩展边界

**无正式插件系统**（对比 tldraw SDK），三层替代：
1. **组件组合**：children 同名组件 tunnel + `withInternalFallback` HOC
   （宿主覆盖内建 MainMenu/Footer/Sidebar，自动让位）；
2. **命令式 API**：`ExcalidrawImperativeAPI`（updateScene/applyDeltas/
   mutateElement/registerAction/setActiveTool/setElementRenderOverrides/
   addFiles + onChange/onIncrement/onPointerDown/onUserFollow 订阅），
   X/types.ts:1360-1448；
3. **无头插件组件**：DiagramToCodePlugin 范式（空渲染组件注入回调）。
行为拦截面：onPaste/onDuplicate/onLinkOpen/onExport/onLibraryChange +
UIOptions.canvasActions 逐项开关 + render props（TopLeft/TopRight/
CustomStats/Embeddable/CustomUI）。

## 7. AI 调用面

**100% 无头化**（核心包零 AI 后端依赖）：
- Mermaid→图：转换器动态 import 懒加载（首屏控制）+ mermaidAutoFix
  BFS 重试 + 括号配平预校验；
- Text-to-Diagram 聊天：契约 `onTextSubmit`（宿主自接 LLM），输出
  mermaid 复用转换链；
- Diagram-to-Code 魔法框：流式 postMessage 喂沙箱 iframe
  document.write 渐进解析（**边生成边渲染**范式）；
- SSE 工具导出（TTDStreamFetch/parseSSEStream，处理限流头）；
  OpenAI 类型手工副本（零 SDK 依赖）；
- 官方接线示范在 APP/components/AI.tsx：后端 URL 构建期注入
  （VITE_APP_AI_BACKEND）。
- embed 面可整体替换（renderEmbeddable）。

## 8. 性能与资源边界

**限额面**（对照 pdf-line/03 五层模型）：
- L0 结构层：无元素数量硬上限（Stats 仅展示）；超大线元素 75000px
  上限（防冻结）；
- L2 渲染层：单元素位图栅格上限 32767px 边 / 16.7M px² 面积（超限
  降 scale）；IMAGE_RENDER_TIMEOUT 500ms；TEXT_MAX_WRAP_WIDTH 800px；
- L3 解释层：MAX_ZOOM=30；
- 帧预算：无显式常量，"每帧至多一次"策略（throttleRAF + 单一
  AnimationController 共享 rAF，空闲自动停表）；
- 【推断】舒适区千级元素；**无空间索引、无 WebGL、无脏区矩形 = 万级
  元素的结构性上限**（frame 命中测试已自标 "huge bottleneck"，
  E/frame.ts:815；snap gap 上限 99999 形同虚设，X/snapping.ts:44-45）。

## 9. 敌意输入与执行边界

六输入面三防线：
- **restore 归一化链**（默认值回填 + link 经 @braintree/sanitize-url +
  escapeDoubleQuotes）——宁可局部丢数据不让场景打不开（工程红线）；
- **SVG 导入无主动清洗**（normalizeSVG 仅 DOMParser 补 xmlns）——安全
  完全押注 `<img src=dataURL>` 加载隔离（浏览器不执行 img 内脚本）；
  【推断】若宿主 renderEmbeddable 自渲染原始 SVG 即绕过防线——设计
  取舍非疏漏；
- **嵌入 iframe 双闸门**：域名白名单 16 项（youtube/vimeo/figma/gist…
  E/embeddable.ts:133-150）+ **差异化 sandbox**（allow-same-origin 只
  授白名单域；**AI 生成 HTML 一律不透明源 iframe**）；
- 库 URL 装载白名单（excalidraw.com + 官方 libraries 仓）+ idToken
  防跨实例注入；
- 剪贴板 HTML 仅抽纯文本与图片 URL（不注入节点）；
- 超链接 noopener noreferrer。
- **缺口：无 CSP 头**（public/_headers 仅 CORS *）。

## 10. 质量体系

- 单测：vitest + jsdom，**~123 测试文件 / ~2000 用例**（element 数据
  面 26 文件 + editor 60 + math/common/utils ~16 + app 3）+ 18 快照；
  restore/reconcile 测试参数化写法是容错语义的直接背书；
- **e2e：无**（本仓库不含 playwright/cypress）；
- 覆盖率硬门禁：lines 60 / branches 70 / functions 63 / statements 60
  （vitest.config.mts）+ PR 机器人报告；
- CI：prettier + eslint --max-warnings=0 + tsc + 全量测试 + **size-limit
  体积门禁**（npm 包尺寸回归即红）+ locales 覆盖 + semantic PR title +
  husky 预提交。

## 11. 净室合规

- **MIT 确认**（根 LICENSE + package.json 显式声明）——架构参考与
  重实现依据充分；净室纪律照旧不复制表达；
- 依赖面抽查（yarn.lock 无 license 字段，registry 元数据）：roughjs
  【推断 ISC】/ react 19 / jotai / pako / nanoid / perfect-freehand
  【推断 MIT】/ firebase 11【推断 Apache-2.0】；**未发现 GPL 污染**；
  字体资产 OFL/MIT 系可嵌入分发；fractional-indexing vendored（CC0）。
- 【推断】正式引用前以 `yarn licenses list` 机器复核。

## 12. 能力矩阵（dsh 插件面 + MCP 工具面双口径，per 规划 §6.1）

| 能力 | dsh 插件面（进程内/SDK） | MCP 工具面（跨进程） |
|---|---|---|
| 场景读取 | `ExcalidrawAPI.getSceneElements()`（无头，React 外可用性待核）【推断 headless 模式有限】 | `get_scene` → elements JSON（直接映射） |
| 场景写入 | `updateScene` / `applyDeltas` / `mutateElement`（增量） | `apply_deltas`（直接映射，带 LWW 语义） |
| 元素构造 | `convertToExcalidrawElements`（markdown/mermaid→元素） | `create_elements(spec)`（薄封装） |
| 冲突消解 | `reconcileElements`（导出原语，可直接做服务端仲裁） | MCP 侧多客户端写 → 需要权威 reconcile 点 |
| 导出 | `exportToCanvas/Blob/Svg/Clipboard`（含场景回嵌） | `export(format)` |
| AI 生成 | onTextSubmit 契约（无头）+ mermaid 链（动态 import） | text_to_diagram（LLM→mermaid→元素管线可整体工具化） |
| 选区/视口 | `setActiveTool` / scrollToContent / onUserFollow | 视口跟随类操作（UX 级，MCP 优先级低） |

**关键判断**：Excalidraw 的命令式 API + 导出原语（reconcile/restore/
serialize）使它对"AI agent 画布"的 MCP 化路径**天然友好**——
`applyDeltas` + `reconcileElements` 几乎就是现成的 MCP 工具签名；但
React 组件依赖使 headless 嵌入（无 DOM）不可行，服务端侧只能复用
数据原语不能复用编辑器本体。

## 13. 对工作台的可移植结论（跨维度综合，按优先级）

1. **P0 移植：元素版本三元组（version/versionNonce/updated）+ 软删除 +
   纯 JSON 无本地状态**——单机编辑器第一天就获得 LWW 协同能力，成本
   极低迁移极贵（mdcore 的 document_id/meta sidecar 同哲学，可直接
   复用到画布元素 schema）。
2. **P0 移植：类型 Readonly + 双修改通道 + 单一提交边界**——不可变
   通道管结构性变更、就地通道管高频拖拽（自动版本提升 + 属性级缓存
   失效），全部汇到 store.commit 一个口（对应 mdcore apply_ops 单一
   变异入口纪律，两侧架构哲学同源）。
3. **P1 参考：快照 diff + 三态捕获的 history**——EVENTUALLY 态天然
   手势合并免命令类；mdcore 已是命令式 inverse-delta，两者可互鉴
   （mdcore 的事务求逆更精确；Excalidraw 的手势聚合更省心）。
4. **P1 参考：双 canvas + nonce 门控 + per-element 位图缓存 + 平移不
   动缓存**——画布渲染的最小可行分层；工作台画布线可直接采用该骨架
   （含 32767px/16.7M px² 位图钳制值）。
5. **P1 移植：Action 注册表**（perform + keyTest + predicate +
   PanelComponent 四合一）——与 mdcore 无命令系统（op-first）不同，
   画布线的用户操作面适合该模式；captureUpdate 语义标签把 history
   语义收进 action 契约是好设计。
6. **P2 决策：协同协议选型**——Excalidraw 的 LWW+nonce 适用于"元素小、
   同元素并发概率低、可接受整元素覆盖"的白板场景（三行规则即收敛）；
   文本级并发不可用（那需 CRDT/OT）。工作台画布线若与 mdcore 共享
   协同基建，LWW 是复杂度下限方案。
7. **P2 移植：敌意输入三层防线**（restore 归一化宁可丢元素不开不了
   场景 / embed 白名单 + 差异化 sandbox / AI 生成内容不透明源隔离）+
   补 CSP 头（Excalidraw 自己的缺口）。
8. **P2 参考：fractional indexing**（CC0 可 vendor）——多端 z 序收敛
   正解；纯单机可缓议。
9. **登记：万级元素是全量重绘路线的天花板**——frame 命中/snap 已是
   实证瓶颈；工作台若目标大画布，须预留空间索引与脏区重绘架构位
   （这是相对 Excalidraw 的差异化机会，也是 AI 批量生成场景的现实
   需求）。
10. **登记：无 CSP 头 + SVG 押注 img 隔离**是它的安全短板；工作台
    canvas 线安全策略表（对照 pdf-a SECURITY_POLICY）应把 CSP 与 SVG
    清洗列为硬门。

## 14. 与工作台架构的对照（一页）

| 面 | Excalidraw | 工作台目标（mdcore 系） | 差异定性 |
|---|---|---|---|
| 变异入口 | 双通道 + store.commit 汇聚 | apply_ops 单一入口（op-first） | 同哲学不同粒度：mdcore op 可序列化重放，Excalidraw delta 面向广播 |
| 撤销 | 快照 diff inverse-delta + 三态 | 命令式逐 op 求逆（ADR-B2/B5） | mdcore 精确性优；Excal 手势聚合优 |
| 身份 | nanoid + 版本三元组 | BlockId 单调分配 + document_id | Excal 面向无中心协同；mdcore 面向单机权威 + 重放 |
| 渲染 | 双 canvas 位图缓存全量重绘 | 画布线待建 | 骨架可移植；天花板已知 |
| 持久化 | scene JSON + PNG/SVG 回嵌 + 加密容器 | md+meta sidecar + 原子写 | sidecar 纪律更严；回嵌与加密可借鉴 |
| AI | 无头契约 + mermaid 管线 + 流式沙箱渲染 | dsh 插件 + MCP 双口径 | applyDeltas/reconcile 原语可直接映射 MCP 工具 |

## 附：证据锚点密度

四路勘察共引用证据文件 ~90 处（路径:行号），覆盖 11 维全部章节；
本报告所有未标注【推断】的结论均有直接代码证据。原始四路勘察输出
存档于本会话（agent-bf18b779 / c6a44fc3 / e8563cac / 1ab7bf8f）。
