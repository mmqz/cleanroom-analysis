# canvas-b：tldraw 深读报告（11 维全量）

> **净室分析报告**（线 A 画布 P1 第二篇）。日期：2026-10-11。
> 对象：tldraw/tldraw @ `7937be3`（main 浅克隆，HEAD 日期 2026-10-09），npm 包
> `tldraw` 5.5.2（monorepo 全包 5.5.2，packages/editor/src/version.ts
> 为生成物含 publishDates）。
> 许可：**source-available（tldraw license，非 OSI）**——Development Environment
> 免费可用；Production Environment 需 License Key，未授权生产环境编辑器 5 秒
> 后熔断；含 "Made with tldraw" 水印义务与许可密钥校验技术措施（§11 详证）。
> 方法：单代理 11 维顺序深读（约 60 处定向文件读 + grep 交叉），对照
> canvas-a（Excalidraw P0 先例）做同位对比；【推断】= 分析员判断。
> 纪律：只读不复制；路径约定 `E/` = packages/editor/src/lib、`T/` =
> packages/tldraw/src/lib、`S/` = packages/tlschema/src、`ST/` =
> packages/store/src/lib、`Y/` = packages/sync-core/src/lib、`M/` =
> apps/mcp-app/src；其余相对仓根。
>
> 一句话定性：**shape util（数据 record 纯 JSON + 行为 util 注册表）+ 全响应式
> 信号 store 的 SDK 化画布引擎——渲染走 React/DOM+SVG 保留模式（非位图
> canvas），文本用真 DOM，笔迹是自研 perfect-freehand 重写（SoA 管线 +
> svgInk 单遍出路径）；协同是服务器定序的 git 式 push/rebase（非 CRDT 非
> OT）；AI 面以 MCP exec「代码即操作」+ focused proxy 走在业界最前；商业
> 模式由 ECDSA 密钥校验 + 水印 + 生产熔断 + 遥测四层技术执行**。

## 0. 架构速览

```
monorepo（pnpm + turbo，21 packages + 9 apps）
packages/editor    = 编辑器内核（Editor 11764 行 + 16 managers（目录 17 项含
                     1 测试文件）+ StateNode 工具机 +
                     ShapeUtil/BindingUtil/AssetUtil/OverlayUtil 抽象 + License 执行）无默认形状无 UI
packages/tldraw    = SDK 壳：默认 13 种 shape util + Select/Draw/Eraser/Hand/Laser/Zoom 工具树 +
                     全套 UI 组件 + .tldr 文件与外部内容处理 + SVG 清洗器
packages/store     = 响应式记录数据库（AtomMap + RecordsDiff + squash + sideEffects + 迁移）
packages/tlschema  = TLRecord 判别联合 + 每记录 validator + 三段式迁移链（root/props/store）
packages/state     = 自研 signals（atom/computed/reactor）；utils = 依赖工具箱（含 CC0 fractionalIndexing）
packages/sync-core = 协同协议 v8（TLSyncClient/TLSocketRoom）；sync = useSync 薄钩子
apps/mcp-app       = MCP 应用（Cloudflare Worker + DO）：search/exec/双向 checkpoint + 单文件 widget
                     （focused editor proxy + 沙箱 exec + updateModelContext 回推）
apps/dotcom        = tldraw.com 宿主（sync-worker/zero-cache/asset workers/评论路由）
```

关键分层：**editor 包零默认形状零 UI**（AGENTS.md:22 自述），tldraw 包只是
editor 的一组"官方插件"（defaultShapeUtils/defaultTools/defaultSideEffects），
custom shape 与内建 shape 走完全同一注册路径——这是它的 SDK 化卖点。

## 1. 数据模型与持久化

**shape util 架构（数据与行为分离的完整形态）**：
- 数据侧：`TLBaseShape<Type, Props>`（S/shapes/TLBaseShape.ts:63-75）＝
  id/typeName/type/x/y/rotation/**index**（fractional index）/parentId/
  isLocked/opacity/**props**（判别联合按 type 分派）/**meta**；shape 是纯
  record，无任何行为与方法。13 种默认 shape（S/records/TLShape.ts:47-60：
  arrow/bookmark/draw/embed/frame/geo/group/image/line/note/text/video/highlight）
  + 自定义 shape 经模块增强 `TLGlobalShapePropsMap` 并入同一联合
  （S/records/TLShape.ts:84-115——禁用/覆盖内建类型也在类型代数层完成）。
- 行为侧：每种图形一个 `ShapeUtil`（E/editor/shapes/ShapeUtil.ts:109）：
  static type/props(validator)/migrations 三件套注册（:31-37），抽象面 =
  getDefaultProps(:189) / **getGeometry→Geometry2d**(:210) / **component**(:218
  React 渲染) / toSvg(:585)；能力协商面 = canBind/canResize/canResizeChildren/
  canCrop/canScroll(:285-348)；交互钩子 = onResizeStart/onResize/onResizeEnd/
  onResizeCancel(:781-813)、onDoubleClick 系列(:954-983)、getInterpolatedProps
  (:532 动画插值)。`ShapeUtil.configure()` 静态方法支持对内建 util 做选项级
  定制（:111-120）。
- 持久化编码亮点：draw 笔迹以 **base64 + Float16/Float32 增量编码** 存储
  （S/misc/b64Vecs.ts:1-18：首点 Float32 绝对值、后续 Float16 增量、2D 模式
  丢弃压力位 3 模式保留），draw shape props 里是字符串而非点数组。

**schema 演进（与 Excalidraw「restore 归一化」路线相反的显式迁移链）**：
- 三层迁移：root shape 迁移（S/records/TLShape.ts:288-293：AddIsLocked/
  HoistOpacity/AddMeta/AddWhite，up/down 成对 :304-356）+ 每 shape props
  迁移（如 geo 12 步链 S/shapes/TLGeoShape.ts:193-206：AddUrlProp→…→
  AddRichText→AddFlipProps）+ store 级迁移（S/store-migrations.ts:16-24 五步：
  删除 code/icon 形状类型、删除 user/presence 记录、**FixIndexKeys 修复
  fractional index 尾零** :103-137——索引修复带随机重发而非确定性重排）。
- 迁移 ID 规范 `com.tldraw.shape.{type}/{version}`（S/records/TLShape.ts:541-543）；
  序列化时 schema 一并写入文件（ST/Store.ts:841-871），读取侧 `getMigrationsSince`
  计算 delta（ST/StoreSchema.ts:440）。

**store（响应式记录数据库）**：记录存于 `AtomMap`（ST/Store.ts:350-352），每记录
即一个信号；`history` atom（historyLength 1000，:361-364）+ HistoryAccumulator
**按相邻同源 squash 后帧级 flush**（:400-410）；`atomic()` 事务内收集
pendingAfterEvents，副作用更新深度 >100 直接抛错（:1246-1249），副作用来源标记
user/remote（:1296）；`mergeRemoteChanges` 包裹远端合并（:1058-1067）；写路径
put/remove 前经 `schema.validateRecord`（:460-476 initialData 路径）；
`ensureStoreIsUsable` 完整性自检（:1209-1213，undo/redo 后强制执行）。**scope 三
分：document/session/presence**（S/records/TLRecord.ts:96-101）——序列化、协同、
本地持久化都按 scope 过滤，相机/选中态是 session 记录但可被 .tldr 完整携带。

**撤销（mark + diff 双栈）**：HistoryManager（E/editor/managers/HistoryManager/
HistoryManager.ts:37-63）三态录音（Recording/RecordingPreserveRedoStack/Paused）
拦截 store 历史帧；栈条目 = `stop`（mark）或 `diff`（squash 后的 RecordsDiff）；
undo 弹到上一个 mark **原子应用逆 diff**（:204-254）；`bail()`= 不入 redo 栈的
撤销（手势中途作废）、`bailToMark(id)`、`squashToMark(id)` 把 mark 间的 diff
压成一条（:254-303）——比 Excalidraw 的三态捕获标签更显式：**手势边界由调用方
显式 mark**，聚合语义相同。

**本地持久化**：`TLLocalSyncClient`（E/utils/sync/TLLocalSyncClient.ts）=
IndexedDB（350ms 节流 :17）+ BroadcastChannel 跨标签页 diff 广播（:123-133）+
schema 版本协商（他端 schema 更新则整页刷新 :185-196 getMigrationsSince 判定）+
pagehide/visibilitychange 兜底 flush（:139-153）；session 态以
TLSessionStateSnapshot 信号单独抽取。

## 2. 渲染管线（本篇核心维）

**定性：React/DOM+SVG 保留模式渲染，非 canvas 位图**——与 Excalidraw 全 canvas
路线相对照的另一极。分层（E/components/default-components/DefaultCanvas.tsx:133-165）：

1. `tl-svg-context`（全局 SVG defs：fill patterns 等按 shape util 注册的
   getCanvasSvgDefs 去重合并 :105-120）；
2. 背景 div + Grid；
3. **`tl-html-layer`（shape 层）**：相机移动时只更新该层一条 CSS transform
   （DefaultCanvas.tsx:95-101 + E/utils/getHtmlLayerTransform.ts:14-22——
   `scale(z) translate(x,y)`，缩放期间零 React 重渲染，GPU 合成）；
4. CanvasOverlays（**canvas 2D**，仅覆盖层：选区框/句柄/snap 线/笔刷，见
   E/editor/overlays/OverlayUtil.ts:36-45「Render into a canvas 2D context」与
   T/overlays/ 14 个 OverlayUtil）。

**每 shape 双容器**（E/components/Shape.tsx:27-161）：定位走**命令式 CSS**——
quick reactor 里逐项比较后 setStyleProperty 写 transform（页面变换矩阵
`Mat.toCssString`）/宽高/clip-path（:65-107），opacity 与 z-index 在 layout
effect 中设置（:110-121）；**内容才走 React**——InnerShape memo 且比较函数为
`areShapesContentEqual`（:248-260），位置/层级/裁剪变化永不触发组件重渲染。
z 序方案：DOM 按 **shape id 排序**保持节点稳定（避免 iframe 重挂载），
z-index 用整数 index 表达层级（E/editor/Editor.ts:4816-4829 注释「Steve,
this is the past you telling the present you not to change this」；frame 背景
层用 `maxShapesPerPage` 偏移实现"底色永远在子元素之下"，:4690-4747）。

**视口裁剪**：`getCulledShapes`（Editor.ts:5777-5825）= notVisibleShapes 增量
派生（E/editor/derivations/notVisibleShapes.ts:12-72：R-tree 查视口内 id，类型
不可变假设下 `unsafeGetWithoutCapture` 免订阅 peek，canCull 默认路径零逐形
订阅）∪ 排除选中/编辑中；**裁剪只改 display 不卸载组件**，由单一
CullingController reactor O(1) 广播（DefaultCanvas.tsx:243-257，注释明确
「replaces per-shape subscriptions for O(1) instead of O(N)」）。相机移动期间
用**第二个更大的渲染包围盒**，停稳后才重算可见集（Editor.ts:4762-4768 注释）。
排序列表自带**置换缓存**（>100 形状时复用上次 id→position 映射 O(N) 代替
sort O(N log N)，Editor.ts:343, 4830-4862）。

**笔迹（自研 perfect-freehand 重写）**：`T/shapes/shared/freehand/` 为
vendored 重写（svg.ts:10 文档仍引用 perfect-freehand 名字；**仓内无版权头**，
见 §11 净室观察）。核心是 **struct-of-arrays 类型化管线**（core.ts:1-17 注释：
模块级复用 Float64Array 缓冲、热循环局部绑定、容量预置零拷贝增长）+
`svgInk`（svgInk.ts:30-51）**单遍从原始输入点直接产出 SVG path 字符串**
（fill 轮廓 + 圆帽），带**肘部切分**（partitionAtElbows :61-80：方向点积
< -0.8 判急性肘部、近邻距离归一化判硬肘，锐肘用原始输入点保"甩笔感"）。
DrawShapeUtil 渲染时按 LOD 降级：zoom<0.5 且 <1.5/线宽强制实线
（DrawShapeUtil.tsx:310-317），zoom<0.2 时虚线间距从 0.1 提到 9（Chrome 不渲染
过小虚线，:319-328）；单点笔迹仍加 `rng(shape.id)` 确定性噪声（:336，同
Excalidraw 的 seed 思路）。画笔上限 maxPointsPerShape 600 自动分段
（DrawShapeUtil.tsx:53-56,66）。

**文本（激进路线 = 真 DOM + 隐藏测量池）**：text shape 组件是
`RichTextLabel`（真 DOM 文本，编辑态切 contenteditable RichTextArea）
（T/shapes/text/TextShapeUtil.tsx:160-200）；测量不模拟排版而**用隐藏
DOM 元素实测**：TextManager 维护测量元素池、`measureHtmlBatch` 批量测量、
逐 grapheme 遍历（E/editor/managers/TextManager/TextManager.ts:107-210, 281-313）；
行高取整像素 `resolveLineHeightPx`（:11-25，跨引擎一致性，issue #8970 引证）；
导出走 foreignObject（RichTextSVG）+ 字体子集嵌入（E/exports/FontEmbedder.ts）。
shape 组件挂载时按需请求 webfont（Shape.tsx:49-54），`maxFontsToLoadBeforeRender`
门控渲染等待（E/options.ts:339 默认 Infinity）。

**overlay 画布**：EffectScheduler 驱动，renderInputs 合并 computed（dpr/尺寸/
相机）相等性短路，单次 setTransform 进页面空间后按 zIndex 顺序渲染全部
active overlay（E/components/default-components/CanvasOverlays.tsx:27-104）。

## 3. 编辑内核

**工具状态机（层级 StateNode）**：StateNode = root/branch/leaf 三型
（E/editor/tools/StateNode.ts:32-77），静态声明 children + initial；事件处理
是**可选方法**（Partial<TLEventHandlers>：onPointerDown/onWheel/onTick/
onInterrupt/onComplete/onCancel…），`_path` computed 暴露激活路径
（`select.idle` 这类字符串，供 isIn 断言与调试）。SelectTool 子状态树 18 个
（T/tools/SelectTool/SelectTool.ts:19-46：Idle/PointingCanvas/PointingShape/
Translating/Brushing/ScribbleBrushing/Resizing/Rotating/Cropping/EditingShape/
DraggingHandle…）——**每个手势阶段是一个叶子状态**，转换即状态切换，行为内聚
在叶子里。`useCoalescedEvents`（StateNode.ts:27）支持指针事件合并流。

**命中与选区**：命中入口 `getShapeAtPoint({hitInside, hitLabels, margin,
renderingOnly})` + hitTestMargin 3px（粗指针 4px）（T/tools/selection-logic/
getHitShapeOnCanvasPointerDown.ts:4-22 + E/options.ts:316-317）；命中即
geometry 的 `hitTestPoint/hitTestLineSegment`（Geometry2d 多态），选区句柄/边/
背景由 SelectionForegroundOverlayUtil 在 canvas 2D 层绘制（T/overlays/）。
选中集合存在 instance_page_state 记录（session scope），编辑器提供
getSelectedShapeIds/onlySelectedShapeId 等派生。

**对齐/分布/堆叠的计算模型**：一等编辑器方法 `alignShapes/distributeShapes/
stackShapes/flipShapes/rotateShapes`（E/editor/Editor.ts:7824 起；mcp exec 文档
直接以 `editor.distributeShapes(...)` 为例，M/tools/exec.ts:56）。关键设计：
**先做 cluster 聚类**（`getShapeClusters`——组/绑定形状聚为一簇），再按簇
pageBounds 对公共包围盒计算平移 delta，最后 `getChangesToTranslateShapeByPageDelta`
把页面空间 delta **逆变换回各形状父坐标系**（Editor.ts:7845-7899）——对齐语义
天然兼容嵌套组与旋转父级。

**snap**：SnapManager = BoundsSnaps（边缘对齐 + gap 等距）+ HandleSnaps 两个
子管理器（E/editor/managers/SnapManager/SnapManager.ts:43-47）；阈值
`snapThreshold / zoomLevel`（:55-60 屏幕像素恒定）；候选集 = 视口内 +
`canSnap` 未禁用 + 排除选中（:62-90），父 frame 也参与（frameLike 显式加入）。
snap 指示线是 overlay（SnapIndicatorOverlayUtil）。

**相机**：camera 是 store 里的 session 记录（stopFollowingUser 里
`store.put([camera])` 可证，Editor.ts:4656-4668）；cameraState idle/moving 驱动
MovingCameraHitTestBlocker（移动中挡指针事件）与裁剪重算节奏；
`getEfficientZoomLevel` = 页面形状 >500 时用**相机开始移动时冻结的 zoom**
（Editor.ts:3221-3266 + E/options.ts:341-342 debouncedZoomThreshold 500）——
拖动缩放期间文本测量/LOD 不抖。

## 4. 格式进出

**.tldr 容器**（T/utils/tldr/file.ts）：`{tldrawFileFormatVersion:1, schema:
SerializedSchema, records:[]}`，MIME `application/vnd.tldraw+json`（:31-45）。
schema 双版本（v1 按 recordVersions、v2 按 migration sequences，:47-77）。
读侧五段防线：总形校验（records 只要求 id/typeName、**其余字段 allowUnknown**）
→ 版本过新拒绝（:128-133）→ `migrateStoreSnapshot` 按 schema 迁移 → 迁移中任何
抛错归为 invalidRecords（:138-168）→ createTLStore 再校验。错误全部类型化
Result（v1File/notATldrawFile/fileFormatVersionTooNew/migrationFailed/
invalidRecords，:92-97），上层转 toast 不崩（:236-294）；**v1 老文件
（2021 tldraw.com document）单独识别并 buildFromV1Document 升级**（:80-89,
279-283）。写侧：资产内联——非 data: 的 image/video 资产 fetch 后转 base64
dataUrl 写入（:182-221），未引用资产双向剪除（pruneUnusedAssets :171-179）；
打开时 data: 资产重新走资产管线注册（extractAssets :318-365）并 zoomToBounds。

**导入面**（T/defaultExternalContentHandlers.ts:92 起）：files（mime 白名单 +
资产 util 扩展点 + 10MB/5000px 上限，:46-51, 890-955）、svg 文本（**先
sanitizeSvg 再入资产**，:314-354）、url（embed 定义匹配或 bookmark）、tldraw
content（:663）、**Excalidraw 场景直接导入**（:717——竞争者格式兼容）。

**导出面**：SVG 优先——`editor.getSvgString/getSvgJsx`（E/exports/）内嵌字体
子集（FontEmbedder）+ CSS 规则内联（StyleEmbedder/parseCss）+ 延迟导出上限
maxExportDelayMs 5000（E/options.ts:332）；PNG 经 getSvgAsImage；剪贴板 SVG/
PNG；`TldrawImage`（T/TldrawImage.tsx:27-77）以快照静态渲染 SVG/PNG 而不挂
完整编辑器（只读展示位）。

## 5. 协同协议

**定性：自研 tlsync——服务器定序 + 客户端 git 式 push/pull/rebase，非 OT 非
CRDT、非 Yjs**：
- 传输：纯 WebSocket JSON；协议版本 8 握手（Y/protocol.ts:3）；
- 消息面（client→server）：`connect{lastServerClock, schema, protocolVersion}` /
  `push{clientClock, diff?, presence?}` / `ping`（Y/protocol.ts:186-278）；
  （server→client）：`connect{hydrationType: wipe_all|wipe_presence, diff,
  serverClock, isReadonly, objectAccess?}` / `patch{diff, serverClock}` /
  `push_result{action: discard|commit|rebaseWithDiff}` / `custom`（:111-135）——
  服务器可以**打回并附修正 diff**（rebaseWithDiff）；
- 冲突模型：客户端本地乐观应用（store 即真源），变更帧级 squash 成 diff 推
  送；**服务器单线程按消息序定序**（TLSocketRoom 会话互斥，Y/TLSocketRoom.ts:223
  + RoomSession；【推断】服务器内部按到达序串行合并，无字段级合并），收敛由
  服务器串行化 + 客户端 rebase 保证，而非版本竞争规则；
- **rebase 全过程可审计**（Y/TLSyncClient.ts:315-330 自述 git-like 模型）：
  重连/被拒时：flush 历史 → 冻结本地 speculative 变更 → **逆应用本地变更 →
  wipe presence（必要时 wipe document）→ 应用服务器 diff → 重放本地变更 →
  重推**（:700-830）；全程 `mergeRemoteChanges` 包裹使历史/副作用正确归类；
- presence 独立于 document diff（put/patch 单记录，协议级支持"只改光标不碰
  文档"）；
- 对象级 ACL：objectAccess read/write 与文档 isReadonly 分离（"可评论不可
  编辑"，Y/protocol.ts:30-36）；
- 持久化：服务器侧 SQLite（Cloudflare DO / Node 双实现，Y/SQLiteSyncStorage.ts
  等）+ 快照历史（dotcom getRoomHistory）；**sync 是 core SDK 的免费 feature
  （非 premium）**，而评论/协同伞（packages/collaboration + commenting）是
  license flag 门控的 premium 模块（两包 package.json `tldraw_product.premium:
  true` + `licenseFlag: FEAT_COLLABORATION/FEAT_COMMENTING`）——同步引擎开放、
  增值层收费的商业切分。
- 【推断】离线语义：本地编辑保留（store 即缓存），重连靠 wipe_all 水化 +
  speculative 重放，无操作日志回放——比 Excalidraw 的 20s 全量重播更精细。

## 6. 插件与扩展边界

**SDK 主边界 = 注册表组合，非事件插件**：
1. **custom shape util**：`<Tldraw shapeUtils=[MyUtil] bindingUtils=[...]/>`
   （T/Tldraw.tsx:156-159）；util = schema（props validator + 迁移）+ 几何 +
   渲染 + 能力协商 + 钩子的完整行为包（§1）；类型层模块增强获得精确类型；
2. **custom record**：TLGlobalRecordPropsMap 增加任意记录类型入 store
   （S/records/TLRecord.ts:13-46 文档示例即"评论记录"）；
3. **组件覆盖**：components prop 同名替换 25+ 内建组件（含置 null 移除，
   T/Tldraw.tsx:53-86）；
4. **工具覆盖**：StateNode 子类注册新工具（editor.setCurrentTool）；内建工具
   也可整体用 shapeTools 约定挂接（T/lib/defaultTools.ts）；
5. **副作用边界**：store.sideEffects register before/after create/change/
   delete（ST/StoreSideEffects.ts 684 行）；内建业务规则亦走此处
   （T/defaultSideEffects.ts:10-36：embed 宽高解析只在 source==='user' 时做，
   远端/加载跳过——副作用协议自带来源语义）；
6. **overlay util / asset util / binding util** 同构扩展点
   （E/editor/overlays/OverlayUtil.ts:56-77 configure + zIndex 插槽）；
7. **app-owned element**：shape util 可声明 `getAppOwnedElement`，画布承诺
   **永不卸载/重建/移动该元素**（id 稳定 DOM 序的意义所在），销毁前回调
   onReleaseAppOwnedElement 供宿主用 Node.moveBefore 迁走保状态
   （E/components/Shape.tsx:164-232——为 iframe/编辑器类 shape 保活设计的
   罕见硬承诺）；
8. 生态件：create-tldraw 模板、vscode 扩展、mermaid 包、namespaced-tldraw
   （遗留）。

对照 Excalidraw 的"组件 tunnel + 命令式 API"两件套：tldraw 把扩展点做进
**数据 schema 与行为注册表本身**，类型安全与运行时校验同源（validator 从
props 定义自动生成）——SDK 边界更工程化。

## 7. AI 调用面（本篇核心维）

**仓内实体 = apps/mcp-app**（MCP app 规范 io.github.tldraw/tldraw，server.json
双 remote：streamable-http + sse）。tldraw 公司的 "computer" 产品在独立仓库
（外部知识，仓内 md/mdx 全检索零命中）；仓内唯一痕迹是 v3.9.0 release
notes 的 API 名 `createComputerCache`（apps/docs/content/releases/v3.9.0.mdx:22）
——【推断】该名实为对现存 API `createComputedCache`（ST/Store.ts:1184）的
**笔误**：后者今日仍在源中，其第三参 options 的 `areRecordsEqual`/
`areResultsEqual`（ST/Store.ts:171-173）与 mdx 描述逐字吻合，「v3.9 后
更名/移除」的解释力弱——**mcp-app 才是对 dsh 直接对位
的官方 agent 画布**。架构（M/README.md:1-24）：Cloudflare Worker + Durable
Object（SQLite checkpoint）+ 单文件 widget iframe。

**操作原语设计 = 「代码即操作」而非枚举工具**：
- `exec` 工具（M/tools/exec.ts:40-79）：输入 `{code, canvasId?}`——模型写
  JavaScript，在 widget 内对**活的 editor**执行；工具描述内嵌 API 速成示例
  （createShape/select/zoomToSelection/distributeShapes/boxShapes，:45-58）；
- **focused editor proxy**（M/widget/focused/focused-editor-proxy.ts:1-8）：ES
  Proxy 包住 editor，**agent 全程用简化格式**——裸字符串 id（自动补 `shape:`
  前缀）、扁平面屑 `_type` 字段；createShape/updateShape/animateShape 特判处理
  **箭头绑定自动建链**（:312-341 + exec-helpers.ts:24-76 createArrowBetween
  Shapes：一条 createShape + 两条 binding）；通用方法按生成的方法规格做参数
  **容错转换**（单 id→数组、数组→变参展平 spread-ids、返回值 shape/ids 统一
  回转 focused 格式，:344-404）——「模型输出不可靠」被系统性吸收在 proxy 层；
- **helper 函数层**（M/widget/exec-helpers.ts:128-145）：createShapeId/Box/Vec/
  Mat/fitFrameToContent + createArrowBetweenShapes/boxShapes（组标签框 + 自动
  group，:78-126）；
- 执行机制：代码包成 ES module 经 Blob URL 动态 import（:208-236）；**轻量
  沙箱 = 执行期间禁用 fetch/XHR/timers**（深度计数可重入，:149-198）+ 10s
  超时竞速（EXEC_TIMEOUT_MS :149）；
- **会话可靠性**：execKey = hash(code+canvasId)（:98）；结果双通道竞速（同
  session 回调 vs 跨 session rendezvous DO），canvasId 不符即拒收（防串台，
  :104-161）；超时不报错——widget 仍会执行并自行回推状态，返回「still
  executing」+ canvasId 让下一轮续作（:166-188）；失败显式声明「code was NOT
  applied」（:190-204）；
- **流式执行**：`app.ontoolinput/ontoolinputpartial`——**模型还在生成代码时
  就 debounced(500/1000ms) 执行**（M/widget/mcp-app.tsx:588-606），画布随生成
  渐进呈现（与 Excalidraw TTD 的边生成边渲染同哲学，作用面移到工具层）；
- **状态回推**：每次 exec 成功后 `pushCanvasContext` →
  `app.updateModelContext({structuredContent:{shapes:[focused 格式]}})`
  （M/widget/persistence.ts:241-292）——**画布状态以结构化数据常驻模型上下文**，
  这是"让 agent 可靠操作"的闭环另一半；附带 checkpoint（DO SQLite +
  localStorage 双落）与 zoomToFitRequestShapes 自动追视（M/widget/snapshot.ts:52-63）；
- **`search` 工具**（M/tools/search.ts）：对**从 editor 包 TypeScript 声明自动
  抽取的 API 规格**（scripts/extract-editor-api.ts → editor-api.json/
  method-map.json）做查询，在隔离动态 Worker 中跑 5s 超时——agent 自助发现
  API 而非靠长 prompt；
- 挂载面：canvas resource 返回单文件 widget HTML + CSP 域白名单
  （cdn.tldraw.com + 字体 + blob:）+ clipboardWrite 权限声明
  （M/register-tools.ts:327-375）；Claude/ChatGPT 有专属域名路由（:45-61）。

**对 dsh 的直接对位**：dsh 插件面 ≈ focused proxy + helpers（进程内、宽容
输入转换、能力即 editor 方法全集）；MCP 工具面 ≈ exec/search/checkpoint 三
件套（跨进程、代码序列化、状态回推）。tldraw 用「一个万能 exec + 规格查询」
取代「N 个细粒度工具」——token 效率与表达力优先，可靠性靠 proxy 容错 +
超时语义 + checkpoint 兜底。

## 8. 性能与资源边界

- **限额面**（E/options.ts:291-347 对照默认值）：maxShapesPerPage 4000 / maxPages
  40 / maxFilesAtOnce 100 / 资产 10MB + 图片边 5000px（T/defaultExternalContent
  Handlers.ts:46-51）/ draw 单形状 600 点 / hitTestMargin 3px / maxExportDelayMs
  5000 / textShadowLod 0.35 / debouncedZoomThreshold 500 / cameraMovingTimeoutMs 64；
- **空间索引**：RBush R-tree + **结构化脏集扩散**（形状变更沿父子/绑定关系扩
  散重查，O(affected) 而非 O(page)；自述已知缺口：几何读非亲缘形状的自定义
  shape 不会被重查，E/editor/managers/SpatialIndexManager/SpatialIndexManager.ts:9-32）；
- **帧预算策略**：无显式帧预算常量——以「响应式依赖即调度」代替 rAF 节流
  （signals 按需重算 + 历史 squash 帧级 flush，ST/Store.ts:400-410）；
  overlay 画布只在 renderInputs 变化时重绘（CanvasOverlays.tsx:30-55）；
- **观测量化内建**：PerformanceManager 惰性挂 LoAF 观察器与 frame 统计
  （avg/median/p95/p99），零订阅零开销（E/editor/managers/PerformanceManager/
  PerformanceManager.ts:59-80）；
- e2e 性能护栏：playwright-perf 工作流 + 基线目录 + PostHog 历史曲线
  （apps/examples/e2e/perf/ + .github/workflows/playwright-perf.yml）；
- 【推断】舒适区：DOM 路线的天花板是**节点数与合成器**（每形状 2 容器 + 内容
  节点），万级形状的 DOM 内存与布局压力高于 canvas 位图路线；但文本/富文本
  保真与可访问性（真 DOM 文本可选中、可读屏）是 canvas 路线拿不到的——
  tldraw 用 R-tree 裁剪 + 稳定序 + 命令式 CSS 把 DOM 成本压到可接受，这是
  与 Excalidraw 相反的取舍而非劣势。

## 9. 敌意输入与执行边界

- **SVG 导入有真清洗器**（Excalidraw 缺口在此补齐）：T/lib/utils/svg/
  sanitizeSvg.ts（692 行 + 866 行测试）——DOMPurify 衍生的标签/属性白名单
  （:1-8 明示署名 MIT 来源）；script 等阻断（BLOCKED_HTML_TAGS :122-142）；
  URI 只放行安全协议与限定的 data: MIME（:372-401 正则族）；`<image>` 的
  svg data: URI **递归再清洗**（:463 深度防递归）；style 元素与 CSS url() 净化
  （:413）；危险动画（运行时改 href）直接删（:600-658）；解析错误返回空串
  → 上层丢弃（file.ts 语义）；
- **场景文件防线**（§4 已列）：类型化错误 + 迁移抛错归一 + 校验失败拒开 +
  schema 过新拒开——宁可不开不执行未知迁移；
- **资产门禁**：mime 白名单（+ asset util 扩展点）、10MB 上限、无 mime 拒收
  （defaultExternalContentHandlers.ts:890-955）；图片维度 5000px 上限；
- **embed 白名单**：主机名匹配定义（tldraw/figma/canva/google/youtube/
  codesandbox/codepen/scratch/val.town…，T/defaultEmbedDefinitions.ts），URL
  → embed URL 转换函数双向校验；youtube 例配 overridePermissions
  'allow-popups-to-escape-sandbox'（:273-279）——权限按站点差异化；
- **mcp widget 双向防护**：widget CSP 资源/连接域白名单（register-tools.ts:
  356-367）；exec 沙箱禁网禁定时器 + 10s 超时；search 隔离 Worker + 5s 超时；
  checkpoint 注入 JSON 数组强校验（parseArrayJson register-tools.ts:35-43）；
  大载荷冒烟测试在档（M/large-payload-smoke.test.ts）；
- **deep link**：URL 参数仅承载编码后的 shape/page/viewport id，解析走
  RecordType.parseId 校验（E/utils/deepLinks.ts:26-50）；
- **缺口**：主仓未发现 CSP 响应头配置（部署在 dotcom/cloudflare 层，【推断】
  CSP 由 widget 声明式 _meta 承担、宿主站点头未见仓内证据）；localStorage
  持久化无容量自检（配额溢出走 alerts 提示，E/utils/sync/alerts.ts）。

## 10. 质量体系

- **单测**：packages 下 **463 个 .test.ts(x)**（vitest + jsdom；store/tlschema/
  editor/tldraw/sync-core 全覆盖，mcp-app 亦有 exec-helpers/idle-expiry/大载荷
  等单测）；
- **e2e**：playwright 多套件——examples 全功能（34 spec：相机/剪贴板/
  富文本/**licensing**/a11y/overlay 快照）、dotcom、perf（基线 + PostHog
  历史）、mermaid、device、staging；快照测试成目录（-snapshots）；
- **CI 门禁**（.github/workflows/checks.yml:33-92）：pnpm dedupe --check /
  check-packages / **check-circular-deps** / typecheck / **check-bundle-size**
  （体积回归红）/ PR 模板检查 / **api-check**（api-extractor 生成 api-report.
  api.md，API 面变更显式化，packages/tldraw/api-report.api.md 在档）/ lint /
  i18n 产物 diff --exit-code / vitest 全量 / build 全量；16 核 ARM runner +
  15 分钟超时 + 并发取消 + 缓存治理（vitest 模块缓存 54MB 显式管理 :84-89）；
- **版本纪律**：turbo 任务图 + bump-versions/publish/get-changelog/update-
  release-notes 工作流（35 个 workflow 的发布编排链）；CHANGELOG 由
  extract-draft-changelog 抽取；version.ts 由脚本生成并携带 publishDates
  （供永久许可版本门控消费，§11）；
- **许可自动化**：license-report 工作流在 PR 改依赖时自动重生成许可报告并
  回推分支（.github/workflows/license-report.yml）；
- **AI 辅助开发制度化**：AGENTS.md/CLAUDE.md 仓根在档（代理行为守则 + 包
  结构导览）——AI-native 开发流程本身已是其质量体系一部分。

## 11. 净室合规（特色维：source-available 许可的技术执行）

**许可条款核验**（LICENSE.md）：source-available 属实且比传闻更严——
- 定义（:9-13）：Production Environment = 任何向最终用户提供功能的部署（含
  web 应用），**排除内部开发**；License Key = 程序生成密钥；
- 义务（:24-34）：默认许可**禁止生产使用**、禁止干扰 License Key 执行、禁止
  移除版权声明、禁止独立分发、须附许可副本全文；
- 试用/商业（:36-42）：trial key 限期生产权；商业协议 key 才有生产权；
- **技术执行条款明示**（:48-50）：软件含技术措施校验密钥、检测部署环境、
  强制水印，且**可能收集并回传使用数据用于许可合规**。

**技术执行四层实现**（E/license/）：
1. **密钥校验**：key 格式 `tldraw-{b64(JSON)}.{b64(sig)}`，WebCrypto **ECDSA
   P-256/SHA-256** 对内嵌公钥验签（LicenseManager.ts:121-122 公钥常量,
   :274-323）；载荷 = [id, hosts, flags, expiry]；flags 位掩码含
   ANNUAL/PERPETUAL/INTERNAL/**WITH_WATERMARK**/EVALUATION/NATIVE +
   FEAT_COLLABORATION/FEAT_COMMENTING（:9-32）；零宽字符清洗防复制损坏
   （:351-352）；域校验支持精确/www/通配/native 协议正则/vscode-webview
   extensionId（:408-454）；
2. **环境检测**：isDevelopment = http / loopback（localhost、127/8、::1）/
   NODE_ENV 非生产（:187-215，blob: origin 归属处理在 :202-209）；生产无 key
   或验签失败 → `unlicensed-production`（:611-635，控制台红字引导购买）；
3. **熔断**：expired 或 unlicensed-production 状态下**编辑器渲染 5 秒后整体
   隐藏**（LicenseProvider.tsx:34-36, 67-82，LICENSE_TIMEOUT=5000，留
   `tl-license-expired` 隐藏节点供测试断言 :88-90）；e2e 实测在档
   （apps/examples/e2e/tests/test-licensing.spec.ts:15-32 断言 canvas 不渲染）；
4. **水印 + 遥测**：unlicensed 与 licensed-with-watermark 两态显示水印
   （Watermark.tsx:26）；水印 SVG 为生成资产内联（E/watermarks.ts:4-8，桌面
   3001×1000 "made with tldraw" 图形）；未授权态升级为「Get a license for
   production」按钮链到 pricing（含 100 天 trial 文案，Watermark.tsx:51-77）；
   CSS 注释即合同（「It is shown for unlicensed users. By using this library,
   you agree to preserve the watermark's behavior」, :137-145）；**遥测回传**：
   未授权生产/水印/评估态向 CDN 发 watermark-track.svg 请求，携带 version/
   license_type/license_id/sku/**部署 URL**/environment（LicenseManager.ts:244-272）
   ——与 LICENSE.md:50「collect and transmit usage data」一一对应。

**产品分层**：core SDK（free, watermark）→ sync（免费 feature）→
collaboration/commenting（premium, licenseFlag 门控，两包 package.json
tldraw_product 声明）→ perpetual key 以 publishDates 门控大小版本
（LicenseManager.ts:489-497）。**口径勘误**：「hobby 免费须水印」的通俗说法
与 LICENSE.md 严格条款有出入——严格读法是**生产使用一律需 key**（无 key =
5 秒熔断 + 未授权水印 + 遥测），namespaced-tldraw README:15 的宽口径
（"so long as you preserve the watermark"）是官方对外沟通口径，【推断】实际
商业执行对小型水印用户从宽（技术上有遥测与熔断手段）。

**净室结论**（与规划 §8.2 呼应）：
- **只读分析无碍**：不复制表达、只提取思想（shape util 模式、rebase 语义、
  exec 设计）在 CA v. Altai 抽象-过滤-比较下安全；
- **未来直接依赖需商业决策**：技术执行是真实且带遥测的（域名/版本/URL 回传），
  生产部署无灰色地带；工作台若仅借鉴架构可安全，若引入 npm 包则触发许可链；
- **第三方件观察**：vendored freehand（T/lib/shapes/shared/freehand/）无上游
  版权头（原 perfect-freehand 为 MIT），fractionalIndexing 显式声明 CC0
  vendored（packages/utils/src/lib/fractionalIndexing.ts:1-10——**这个有规范
  署名**，二者形成对照）；sanitizeSvg 保留 DOMPurify MIT 署名（:1-8）。
  【推断】freehand 重写已到「换表达」程度，但对依赖它的净室方而言不可复制。

## 12. 能力矩阵（dsh 插件面 + MCP 工具面双口径，per 规划 §6.1）

| 能力 | dsh 插件面（进程内/SDK） | MCP 工具面（跨进程） |
|---|---|---|
| 场景读取 | `editor.getCurrentPageShapes()/getShape()` + `store.query`（响应式，订阅增量） | `read_checkpoint` / `_get_canvas_state`（DO SQLite）+ widget `updateModelContext` 自动回推 focused 格式 shapes |
| 场景写入 | `createShape(s)/updateShape(s)/deleteShapes/animateShape(s)`（Editor 一等方法） | `exec{code, canvasId}`（代码即批量操作，focused proxy 容错转换） |
| 元素构造 | shape util + `createShape` partial（含箭头绑定自动建链 helper） | exec 内 `createArrowBetweenShapes`/`boxShapes` helpers + `_type` 扁平格式 |
| 冲突消解 | `store.mergeRemoteChanges` + TLSyncClient rebase（服务器定序） | 服务器 DO 串行化；checkpoint 按 canvasId 寻址续作 |
| 导出 | `getSvgString/getSvgJsx`（字体内嵌）/`TldrawImage`/`serializeTldrawJson` | widget `exportTldr`（host downloadFile resource） |
| API 发现 | TS 类型 + api-report.api.md | `search` 工具（抽取的 API 规格 + 隔离 worker 查询，5s 超时） |
| 选区/视口 | `select/zoomToSelection/zoomToFit` + 相机动画 | exec 直调同 API；widget 自动 zoomToFit 新形状 |
| 流式生成 | —（进程内可自建） | `ontoolinput/partial` debounced 500/1000ms 流式执行 |

**关键判断**：tldraw 给出了「**单 exec 工具 + API 规格 search + 结构化状态回推**
」的极简 MCP 形态——与「一操作一工具」的朴素 MCP 化互为对照样本；对 dsh 的
启示是**进程内插件面应做宽容输入转换层**（focused proxy 模式），跨进程面应
**把画布状态常驻模型上下文**（updateModelContext 模式）而非每次读回。React/DOM
依赖使 headless 服务端渲染不可行（同 Excalidraw 结论），但 editor/renderer 分包
更干净：@tldraw/editor 可无 UI 运行（TldrawEditor 无默认形状），数据原语
（store/schema/migrate/sync-core）可独立于 React 消费。

## 13. 对工作台的可移植结论（跨维度综合，按优先级）

1. **P0 移植（设计模式）：shape util 注册表**——数据 record 纯 JSON + 行为
   util（validator/迁移/几何/渲染/能力协商/钩子）+ 类型层模块增强。工作台画布
   线的 shape 系统蓝本：数据可序列化可迁移，行为可替换可扩展，13 内建与
   自定义走同一路径（比 Excalidraw 的 12 类型硬编码联合更工程化）。
2. **P0 移植：store 的 scope 三分 + RecordsDiff 协议**——document/session/
   presence 在记录层区分，序列化/协同/持久化全按 scope 过滤；diff + squash +
   mergeRemoteChanges 的变更语言可整体平移到 mdcore 系画布（与 apply_ops
   单一入口哲学兼容：diff 即 op 序列的声明式压缩）。
3. **P0 移植（AI 面，本篇最重要资产）：agent 画布四件套**——①宽容输入
   proxy（裸 id/扁平格式/数组变参容错）；②代码即操作 + 轻量沙箱（禁网禁
   定时器 + 超时竞速）；③结构化状态回推（画布状态常驻模型上下文）；④
   checkpoint 寻址（canvasId 贯穿会话）。dsh 插件面可直接按此模板实现，
   MCP 面同构。
4. **P1 参考：mark + diff 撤销**——显式 mark 边界 + bailToMark/squashToMark，
   手势聚合语义与 Excalidraw EVENTUALLY 等价但更可控；两案对照后工作台可取
   「mark 显式化 + diff 自动 squash」折中。
5. **P1 参考：渲染分层取舍**——tldraw 证明 DOM/SVG 保留模式 + 命令式 CSS +
   裁剪隐藏 + R-tree 索引 + 相机单 transform 可行且换来真 DOM 文本（可选/
   可读屏/富文本）与 SVG 导出零转换；工作台画布若富文本优先可采此路线，
   若追求十万级对象则 PixiJS 批渲染（P2 对照对象）才是位图正解。
6. **P1 移植：svgInk/SoA 笔迹管线思想**——单遍输入→路径字符串、SoA 复用
   缓冲、肘部切分（原始输入点保甩笔感）——自研笔迹的性能基准设计（注意
   净室：不可复制其代码，tldraw license）。
7. **P1 移植：SVG 白名单清洗器思路**——DOMPurify 式标签/属性/URI 三白名单
   + 递归 data:URI 清洗 + 危险动画删除；补齐 canvas-a 发现的 Excalidraw
   「SVG 无清洗」短板，工作台 SVG 导入硬门。
8. **P2 决策：协同选型**——服务器定序 + 客户端 rebase 的复杂度低于 CRDT、
   高于 Excalidraw 广播 LWW；若工作台画布协同复用 mdcore 权威服务器，tlsync
   模型（push_result 可 rebaseWithDiff 打回）与服务端 SQLite 快照是现成参照；
   文本级并发仍不可用（同 canvas-a 结论）。
9. **P2 移植：性能工程细节集**——debouncedZoom 冻结（>500 形状缩放期间不重
   排）、渲染排序列表置换缓存（O(N) 代替 O(N log N)）、相机第二包围盒
   （移动中不重算裁剪）、O(1) culling 广播器、整像素行高（跨引擎文本一致）、
   R-tree 结构化脏集扩散。
10. **登记：source-available 许可依赖边界**——架构参考安全，npm 依赖即触发
    商业链（含遥测与熔断）；工作台画布线若自研内核，本报告 1/2/3 条为设计
    输入，代码零复制（净室双保险）。
11. **登记：DOM 路线天花板**——每形状 2 容器 + 内容节点，万级形状内存/布局
    压力高于 canvas；与 Excalidraw 全量重绘天花板互补：**两者共同指向
    「空间索引 + 差量渲染 + LOD」是中间正解**（tldraw 已做前两者）。

## 14. 与工作台架构的对照（一页，与 canvas-a §14 并列）

| 面 | tldraw | Excalidraw（canvas-a §14） | 工作台目标（mdcore 系） |
|---|---|---|---|
| 变异入口 | store.put/applyDiff + sideEffects（来源 user/remote） | 双通道 + store.commit | apply_ops 单一入口（op-first）——三者中 mdcore 与 tldraw 的 diff 协议最近 |
| 撤销 | mark + squash diff 双栈（bail/squashToMark） | 快照 diff + 三态捕获 | 命令式逐 op 求逆；可借鉴 mark 显式手势边界 |
| 身份 | uniqueId + 前缀品牌 id + scope 三分记录 | nanoid + 版本三元组 | BlockId 单调分配；scope 思想可移植（session 态不入库） |
| 渲染 | React/DOM+SVG 保留 + 命令式 CSS + canvas2D overlay | 双 canvas 位图缓存全量重绘 | 画布线待建：文本保真→DOM 混合层；吞吐→WebGPU 批渲染 |
| 持久化 | .tldr（schema 随行 + 资产 base64 内联）+ IndexedDB 节流 + schema 协商刷新 | scene JSON + PNG/SVG 回嵌 + 加密容器 | md+meta sidecar + 原子写；schema 随行文件头值得吸收 |
| 协同 | tlsync：服务器定序 + git 式 rebase + presence 独立通道 | 盲转快照广播 + LWW + nonce 平局 | mdcore 单机权威 + 重放；若画布协同，tlsync 模型对位最近 |
| AI | MCP exec + focused proxy + updateModelContext + checkpoint | 无头契约 + mermaid 管线 + 流式沙箱渲染 | dsh 插件 + MCP 双口径——tldraw 四件套是直接模板 |
| 许可 | source-available + 技术执行（ECDSA/熔断/水印/遥测） | MIT | 自研内核零许可负担；借鉴设计不复制代码 |

## 附：证据锚点密度

本报告共引用证据锚点约 **130 处**（路径:行号），覆盖 11 维 + 矩阵/对照/
结论全部章节，密度与 canvas-a（~90 处）同量级以上；所有未标注【推断】的
结论均有直接代码证据，【推断】标注共 7 处（§5 服务器串行化、§5 离线语义、
§7 computer 史笔误说、§8 DOM 天花板、§9 CSP 缺口、§11 商业执行宽严、
§11 freehand「换表达」）。深读
工作副本：/home/z/my-project/analyses-workdir/tldraw（浅克隆 @7937be3，未做
任何修改、未 git commit）。
