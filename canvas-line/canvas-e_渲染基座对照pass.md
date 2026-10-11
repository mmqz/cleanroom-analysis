# canvas-e：渲染基座对照 pass（Konva / Fabric.js / PixiJS 三路线）

> **净室分析报告**（线 A 画布 P2 批次，对照 pass）。日期：2026-10-11。
> 对象（三仓均 master/dev 默认分支浅克隆 --depth 1，只读零修改零 commit）：
> - konvajs/konva @ `de4fef4`（2026-10-07），npm konva **10.7.1**
> - fabricjs/fabric.js @ `c83d042`（2026-10-11），npm fabric **7.4.0**（monorepo，packages/core 为内核）
> - pixijs/pixijs @ `6555201`（dev 分支=默认，2026-10-10），npm pixi.js **8.22.0**
> 许可：**三仓逐件核验全 MIT**（见 §0.2）——规划"印象均 MIT 待核"就此定谳。
> 方法：单代理三路线定向勘察（对照 pass 体例，表格化比较为主文体）；
> 结论标注 `路径:行号` 证据锚点，【推断】= 分析员判断。路径前缀约定：
> `K/` = konva/src，`F/` = fabric.js/packages/core/src，`P/` = pixijs/src；
> 三仓根文件直接写（如 `konva/package.json:3`）。
> 定位：回答规划 §5.2 设定的四问题（Konva 场景图+命中 / Fabric SVG 保真 /
> PixiJS 批渲染 / 选型矩阵），并补全渲染路线光谱——与 canvas-a §2（Excalidraw
> Canvas 2D 即时模式）和 canvas-b §2（tldraw DOM/SVG 保留模式）构成五点对照。
>
> 一句话定性：**三条渲染基座路线互不替代——Konva 是"Canvas 2D 上的 DOM"
> （保留模式场景图 + color-key 像素命中），Fabric 是"带 SVG 进出的编辑器对象
> 模型"（几何命中 + 有损 9 标签子集往返），PixiJS v8 是"指令流批渲染器"
> （三后端 + 纹理槽批 + GC，与场景图编辑无涉）**；选型判据不是"谁更好"而是
> 命中精度、矢量保真、十万级吞吐三个正交轴各自向谁收敛。

## 0. 勘察基线与许可核验

### 0.1 三仓克隆基线

| 项 | Konva | Fabric.js | PixiJS v8 |
|---|---|---|---|
| 仓库 | konvajs/konva | fabricjs/fabric.js | pixijs/pixijs |
| HEAD（浅克隆） | de4fef4，2026-10-07 | c83d042，2026-10-11 | 6555201（dev），2026-10-10 |
| 版本 | 10.7.1（konva/package.json:3） | 7.4.0（fabric.js/package.json:5） | 8.22.0（pixijs/package.json:3） |
| 源码规模 | 根层 25 文件 14,235 行 TS + shapes/ 17 + filters/ 20（全 src 62 文件 24,126 行；R4-fix 口径修正：原记「18 文件+shapes 18+filters 20」分项误，14,235 为根层精确值） | packages/core/src 45,316 行 TS（不含 spec；monorepo 另有 browser/node/e2e 等 9 包） | src 122,535 行 TS（R4-fix 口径注：全部 .ts 排除 __tests__ 目录；另一口径排除 *.test.ts 但含 __tests__ 内 4 件辅助为 122,747，原记 122,470 偏差 <0.3%；rendering/scene/events/assets 等 30+ 模块） |
| 单测 | 43 spec + 2 辅助（test/unit 45 文件，mocha 浏览器+Node 双跑；R4-fix：原记 45 spec 系目录全量口径） | 99 spec/test 文件（vitest + snapshot） | 216 个 `*.test.ts`（src 内 215 + tests/visual 入口 visuals.test.ts 1；另有 4 件 __tests__ 非 test 辅助 .ts，__tests__ 下全部 .ts 为 219——原记 219 系后一口径；77 个 `__tests__` 目录）+ visual 像素快照（pixelmatch） |
| 服务端渲染 | node-canvas / skia-canvas 双后端（K/canvas-backend.ts:1-40、K/skia-backend.ts:1-30） | jsdom + node-canvas 桥（fabric.js/packages/node/src/env.ts:1-50） | 无（GPU 后端依赖 DOM；Canvas 后端亦为浏览器环境设计）【推断】 |

### 0.2 许可逐件核验（三仓各唯一一件 LICENSE）

| 仓库 | 文件 | 许可 | 版权链（行号） | 核验结论 |
|---|---|---|---|---|
| konva | konva/LICENSE（21 行） | MIT | 原作 KineticJS Eric Rowell 2011-2013 → Anton Lavrenov 2014-present（konva/LICENSE:3-5） | MIT 定谳；package.json license: MIT |
| fabric.js | fabric.js/LICENSE（22 行） | MIT | Printio（Juriy Zaytsev 等）2008-2015 → Andrea Bogazzi 等 2016-present（fabric.js/LICENSE:3-4） | MIT 定谳；package.json license: MIT |
| pixijs | pixijs/LICENSE（21 行） | MIT | Mathew Groves, Chad Engler 2013-2023（pixijs/LICENSE:3） | MIT 定谳；package.json license: MIT |

三仓 `find -name "LICENSE*"`（排除 node_modules）各恰 1 件，无子包许可
分叉；【推断】monorepo 子包（fabric packages/*、pixi 无独立包）以根 LICENSE
统一覆盖，npm 发布物沿用同许可。规划 §5.2"印象均 MIT 待核"→ 三仓全 MIT，
无 tldraw 式 source-available 附加条款、无 HyperFormula 式双许可分叉。

## 1. Q1 Konva：保留模式场景图 + color-key 命中检测

### 1.1 场景图骨架（Node/Container/Stage/Layer 四层）

Konva 的定位是"Canvas 2D 上的 DOM"：一颗显式场景树 + 每层两张位图
（场景画布 + 命中画布），把浏览器"保留模式 DOM + 事件冒泡"的模型搬到
canvas 上。

| 场景图构件 | 实现要点 | 锚点 |
|---|---|---|
| Node（3579 行） | 属性系统（Factory get/set）+ 变换缓存 `_clearSelfAndDescendantCache`（ABSOLUTE_TRANSFORM/OPACITY/VISIBLE/LISTENING/STAGE 五类）+ 级联深度计数（absoluteTransform 扇出时把 O(N) 监听折叠为 1） | K/Node.ts:372-392、K/Node.ts:1035-1039 |
| Container | 递归遍历 drawScene/drawHit + clip（rect 或自定义 clipFunc，先 apply 后 invert 回乘） | K/Container.ts:375-424 |
| Layer | **每层独立 SceneCanvas + HitCanvas（pixelRatio=1）**；非监听层命中画布按 0×0 释放（纯展示层不付 stage 级位图，issue #2009） | K/Layer.ts:57-60、K/Layer.ts:234-244 |
| Stage | 多 Layer 编排；指针事件状态机（pointer/mouse/touch 三系兼容配对 relatedPointer，坐标取整差异容忍）；getIntersection 自顶层向下逐层问询 | K/Stage.ts:596-664、K/Stage.ts:400-416 |

绘制调度：`batchDraw` = RAF 合并（每层 `_waitingForDraw` 闸门，同帧多次
请求只画一次；弹出窗口场景向 stage 所属 window 要帧）——K/Layer.ts:320-333、
K/Stage.ts:1109-1116。命中图在拖拽/Transformer 变换期间**整体跳过重绘**
（`shouldDrawHit` 检 DD._dragElements 与 Transformer._isLayerTransforming，
拖完一帧恢复）——K/Layer.ts:450-471。这是"命中图付得起"的关键节流阀。

节点级缓存 `cache()`：离屏场景位图 + 可选低分辨率命中位图
（`hitCanvasPixelRatio` 参数）——K/Node.ts:451-461；缓存命中图走
`drawHitFromCache`（alpha 阈值逐像素替换为 colorKey，默认阈值 0）——
K/Shape.ts:810-853。10.7.0 新增 `Group.isolated()`（SVG `<g opacity>` 式
隔离合成）与"perfect drawing"按形状尺寸缓冲（konva/CHANGELOG.md:13-15，
"isolated group draws its children into a transparent canvas … child blend
modes and erasing stay inside the group"）。

### 1.2 命中检测：color-key 命中区（color-math）

Konva 命中是**像素级**的：每 shape 分配唯一 RGB 色（colorKey），命中画布
用该纯色重绘全部形状，查询时读回 1 像素 → 反查形状注册表。这是本次 pass
最值得登记的机制（2026 年仍在持续硬化——一手佐证：konva/CHANGELOG.md:6
10.7.1@2026-10-05、:11 10.7.0@2026-09-23 perfect drawing/isolated、
:82 10.5.0 命中边缘改进）：

| 环节 | 实现 | 锚点 |
|---|---|---|
| 色键分配 | 随机 RGB → 冲突重试（上限 10000 次，超限告警"canvas farbling"，兜底随机色） | K/Shape.ts:196-219 |
| **色键网格化**（color-math 核心） | RGB 各通道 snap 到步长 3 网格（farbling 激活时步长 5）——预乘 alpha 边缘像素读回偏差 ≤1/通道、Brave farbling 加噪，网格化保证仍落回原形状键 | K/Util.ts:754-762 |
| farbling 探测 | 10×10 canvas 画 #282828 读回逐像素比对，任一偏差即判定 farbling（惰性一次缓存） | K/Util.ts:696-734 |
| 命中绘制 | HitContext 以 `willReadFrequently:true` 获取（K/Context.ts:976-981）；填充色即 colorKey（K/Context.ts:983-988）；stroke 命中宽度可用 `hitStrokeWidth` 覆盖（'auto'=strokeWidth） | K/Context.ts:989-1006 |
| 单点查询 | 读 1 像素 → alpha≥128 才反查（半透明边缘不算）→ `shapes[getHitColorKey(r,g,b)]` | K/Layer.ts:406-428、K/Layer.ts:33-37 |
| **环形搜索**（抗锯齿容错） | 边缘像素（opaque 但色为两形状混合）时按半径 10 起倍增至最大 256 的同心环扫描；alpha 不再上升即停（细线场景即停，避免沿线走到无关形状） | K/Layer.ts:23-28、K/Layer.ts:351-405 |
| 命中图释压 | 非监听层 0×0；拖拽/变换期跳过重绘；deprecated FastLayer → `Layer({listening:false})` | K/Layer.ts:234-244、K/Layer.ts:450-471、K/FastLayer.ts:6-9 |
| 全局开关 | `Konva.hitOnDragEnabled`（拖拽期仍要命中时开） | K/Global.ts:62-69 |

对照结论（Konva vs 浏览器原生）：Konva 用 color-key 换来了**任意
sceneFunc/滤镜/缓存路径下的像素级命中一致性**——命中面 = 渲染面（含阴影
stroke 偏移之外的区域由 hitStrokeWidth 显式声明）；代价是每监听层一张
stage 级命中位图 + 命中图重绘（节流阀见上）。Excalidraw（canvas-a §2）
没有独立命中图，命中走元素几何判定（canvas-a 报告 L208：frame 命中测试
自标 "huge bottleneck"），与 Konva 的"渲染即命中"是 Canvas 2D 路线内
两个正交选择。

### 1.3 Konva 路线小结

| 维度 | 判定 | 依据 |
|---|---|---|
| 渲染模式 | 保留模式场景图 → Canvas 2D 即时绘制，整层重绘（无脏区） | K/Layer.ts:429-449 |
| 命中精度 | 像素级（color-key + 网格 snap + 环形搜索三重容错） | §1.2 |
| 数量级 | 【推断】千级形状交互舒适层；无批渲染，逐 shape Canvas API 调用，静态巨量场景需 `listening:false` 层 + cache() | §1.1 |
| 编辑器配套 | Transformer（八向句柄组，本身是 Group）+ 20 个像素滤镜 + Tween/Animation | K/shapes/Transformer.ts:297、K/filters/（20 文件）、K/Animation.ts:32、K/Tween.ts:232 |
| 服务端 | node-canvas/skia-canvas 可跑（toDataURL/toImage 导出） | K/canvas-backend.ts:1-40 |
| 许可 | MIT | §0.2 |

## 2. Q2 Fabric：对象模型 + SVG 进出

### 2.1 对象模型与几何命中

Fabric 的模型是**编辑器对象**（FabricObject 家族 13 类：Rect/Circle/
Ellipse/Line/Polygon/Polyline/Path/Text/IText/Textbox/Image/Triangle/
Group + ActiveSelection——F/shapes/ 目录），每对象自带变换、样式、controls
（角/边句柄）与序列化。命中检测与 Konva 相反，**纯几何**：

| 环节 | 实现 | 锚点 |
|---|---|---|
| 查找 | `_searchPossibleTargets` 自顶向下（倒序索引）线性扫 `_objects`；Group 开 `subTargetCheck` 时递归入组收集 subTargets；`interactive` 组决定最深可选中目标 | F/canvas/SelectableCanvas.ts:934-962、:971-1005 |
| 判定 | visible+evented+**OBB 选择区**（getCoords 四角 + padding 按对象角度展开）→ 命中 | F/canvas/SelectableCanvas.ts:903-923、:861-881 |
| 可选像素精化 | `perPixelTargetFind` 开启后对 OBB 命中者再画到 1 像素专用画布读 alpha（isTargetTransparent；编辑中文本豁免） | F/canvas/SelectableCanvas.ts:910-917、:468-489 |
| 句柄优先 | 活动对象先 findControl（命中句柄即短路返回，含"角外圈"） | F/canvas/SelectableCanvas.ts:809-820 |

渲染：单画布全量重绘 + RAF 合并（requestRenderAll 闸门
F/canvas/StaticCanvas.ts:490-494；renderCanvas 全清重画
F/canvas/StaticCanvas.ts:534-552）。**对象位图缓存**（objectCaching）：
每对象离屏 canvas，尺寸上限 perfLimitSizeTotal=2Mpx / 单边
maxCacheSideLimit=4096 / minCacheSideLimit=256（F/config.ts:36-50；
F/shapes/Object/Object.ts:380-386、:397-432）；Group 递归缓存判定
shouldCache（F/shapes/Object/Object.ts:775-778）。文本走逐 grapheme 的
canvas measureText + 字宽缓存（F/shapes/Text/Text.ts:92-93、:823；F/cache.ts:8-43）。

### 2.2 SVG 解析管线（读侧 = 有损 9 标签子集）

| 环节 | 实现 | 锚点 |
|---|---|---|
| 入口 | `loadSVGFromString` → **DOMParser**（'text/xml'）→ parseSVGDocument（DOM 全量遍历，非流式） | F/parser/loadSVGFromString.ts:19-28、F/parser/parseSVGDocument.ts:35-57 |
| 有效标签 | **仅 9 个可绘制标签**：path/circle/polygon/polyline/ellipse/rect/line/image/text | F/parser/constants.ts:23-33 |
| 无效祖先 | pattern/defs/symbol/metadata/clipPath/mask/desc 内的元素跳过（defs 语义：不实例化为对象） | F/parser/constants.ts:35-43、F/parser/parseSVGDocument.ts:54-57 |
| `use` 展开 | 就地 cloneNode(true) 替换（属性合并、style 特殊合并、id 移除防重复）——**每次 use = 深拷贝，非实例引用** | F/parser/parseUseDirectives.ts:6-91 |
| 样式优先级 | style > class > attribute（CSS 规则经 getCSSRules 解析，含 `<style>` 与内联） | F/parser/parseSVGDocument.ts:76-83、F/parser/elements_parser.ts:60-62 |
| 渐变/裁剪 | url(#id) 引用解析：线性/径向渐变 → Gradient 对象；clipPath 递归物化为 Group（qrDecompose 分解变换，Set 防环，祖先同 id 链式裁剪） | F/parser/elements_parser.ts:127-145、:149-271 |
| 分派 | `classRegistry.getSVGClass(tag).fromElement(el, options, cssRules)`——每形状类自带 fromElement | F/parser/elements_parser.ts:23-24、:70-93 |
| viewBox | applyViewboxTransform 逐元素应用 | F/parser/parseSVGDocument.ts:48-57 |

**保真边界**（读侧）：SVG filter 效果（feGaussianBlur 等）零支持（parser
目录无 filter 处理）；`<mask>` 不支持（仅在无效祖先表——元素被丢弃而非
物化）；`<pattern>` 同为无效祖先（Pattern 对象只用于程序化填充）；`<text>`
按 fabric 文本模型重排（grapheme 级重排版，非 glyph 保真）。【推断】与
office-b §4 的"万物过二进制枢纽"同理，fabric 的 SVG 读入是**语义转译**：
文件 → 对象模型，一切不在 13 类对象表达力内的 SVG 特性必有损；无损往返
不是 fabric 的设计目标。

### 2.3 SVG 序列化（写侧）与敌意面

写侧为字符串拼装：StaticCanvas.toSVG（preamble + header/viewBox 由
viewportTransform 反推 + defs(fontfaces/渐变/clipPath) + 对象 markup +
背景/覆盖，F/canvas/StaticCanvas.ts:950-1037）；每对象 `_toSVG` 多态 +
`_createBaseSVGMarkup` 公共骨架（transform/clipPath/vector-effect，
F/shapes/Object/FabricObjectSVGExportMixin.ts:136-152、:195-244）。
**数值精度**：全局 NUM_FRACTION_DIGITS=4（F/config.ts:107-111，注释明言作用于
SVG/JSON 序列化 4 位小数，【推断】坐标往返存在 1e-4 量级量化损耗（可
配置收紧）。敌意输入面：7.4.0 修复 CVE-2026-44311（SVG 导出 CSS 注入，fabric.js/CHANGELOG.md:68），
`isSafeSvgStyleValue` 正则拦截控制字符/`;`/`<>`/注释/url(/expression(/
javascript:/data:/@import（F/util/internals/svgExportCheck.ts:1-8），全部
导出样式值过闸（F/shapes/Object/FabricObjectSVGExportMixin.ts:32-60）；
读侧信任 DOMParser 的 XML 解析错误行为，reviver 回调暴露原始元素供宿主
扩展检查（F/parser/parseSVGDocument.ts:24-27）。

### 2.4 Fabric 路线小结

| 维度 | 判定 | 依据 |
|---|---|---|
| 渲染模式 | 即时模式（单画布全量重绘 + 对象位图缓存） | §2.1 |
| 命中精度 | OBB 几何优先，opt-in 逐像素精化 | §2.1 |
| SVG 读 | 9 标签 + 渐变 + clipPath + CSS 三级样式 + use 展开；mask/filter/pattern 无 | §2.2 |
| SVG 写 | 全模型 → SVG 字符串，4 位小数量化 | §2.3 |
| 编辑器配套 | 最全：IText/Textbox 画布内文本编辑、PencilBrush 等 4 笔刷（F/brushes/）、Control 句柄系统（F/controls/Control.ts:31 + 13 变换 handler 文件）、LayoutManager 布局策略、对齐辅助线（packages/aligning-guidelines） | 目录盘点 |
| 服务端 | jsdom+node-canvas（loadSVGFromString/toSVG/toJSON 均可离线） | §0.1 |
| 许可 | MIT | §0.2 |

## 3. Q3 PixiJS v8：WebGL/WebGPU 批渲染与纹理管理

### 3.1 架构：场景图 → 指令流 → 三后端执行

v8 重构核心 = **渲染与场景解耦**：场景侧只产出 InstructionSet（指令流），
后端（WebGPU/WebGL/Canvas2D 三 adaptor）各自执行同一指令。

| 层 | 实现 | 锚点 |
|---|---|---|
| RenderGroup | 每个渲染根一颗 RenderGroup：持有独立 InstructionSet + 增量更新簿记（childrenToUpdate 按 flag 分层 / childrenRenderablesToUpdate / structureDidChange 结构性变化才全重建） | P/scene/container/RenderGroup.ts:52-118 |
| 渲染入口 | render() → 更新 cacheAsTexture 组 → 更新变换 → executeInstructions（按 renderPipeId 分发指令，如 'batch'/'renderGroup'） | P/scene/container/RenderGroupSystem.ts:44-93、P/scene/container/utils/executeInstructions.ts:10-21 |
| InstructionSet | 定长数组复用（instructionSize 计数不 resize）；Batch 可池化 | P/rendering/renderers/shared/instructions/InstructionSet.ts:17-79 |
| 后端探测 | autoDetectRenderer 默认序 **['webgl','webgpu','canvas']**——注释明言 WebGL 是"most tested safe API"、WebGPU 待稳定后将前移 | P/rendering/renderers/autoDetectRenderer.ts:38-44 |
| 扩展机制 | ExtensionType 全景（Batcher/ pipes adaptor/ system 各三后端注册位） | P/rendering/batcher/shared/DefaultBatcher.ts:68-73、P/rendering/batcher/gl/GlBatchAdaptor.ts:18-23 |
| 渲染顺序 | zIndex/sortableChildren 之外，RenderLayer 允许**渲染序与场景树解耦**（attach 不改父子；引入版本 v8.2+【推断：版本号为外部知识，仓内无 CHANGELOG 可证】） | P/scene/container/Container.ts:749-754、:1132-1134、P/scene/layers/RenderLayer.ts:1-7（R4-fix 锚修正：原 :1052 实为 sortableChildren 判定行，解耦证据在 parentRenderLayer 声明与摘挂处） |

### 3.2 批渲染（Batcher）

| 环节 | 实现 | 锚点 |
|---|---|---|
| 批元素契约 | BatchableElement：texture/blendMode/topology/packAsQuad + 属性/索引预算；quad=4 顶点 6 索引，mesh 自带 uvs/positions/indices | P/rendering/batcher/shared/Batcher.ts:115-272 |
| **批切分规则** | 断批仅三种：纹理数达 maxTextures / blendMode 变 / topology 变（`breakRequired`）；同纹理源复用槽位（`_batchTick` + `_textureBindLocation`） | P/rendering/batcher/shared/Batcher.ts:529-609（断批 :534、:579） |
| 顶点格式 | DefaultBatcher 每顶点 6×32bit：x,y,u,v,color(ARGB uint32),textureIdAndRound（`(textureId<<16)\|roundPixels` 位打包） | P/rendering/batcher/shared/DefaultBatcher.ts:80-81、:100-141 |
| 纹理槽数 | WebGL 由 MAX_TEXTURE_IMAGE_UNITS + **shader if 语句数上限**双重探测（`checkMaxIfStatementsInShader`，批着色器按槽数生成 if 链） | P/rendering/batcher/gl/utils/maxRecommendedTextures.ts:14-30 |
| 缓冲增长 | 属性/索引缓冲倍增扩容 + fastCopy 拷贝；索引 >65535 自动 Uint16→Uint32 | P/rendering/batcher/shared/Batcher.ts:707-764 |
| WebGL 执行 | 每 Batch 一次 drawElements：绑全部批纹理到槽 → draw(topology, size, start)；着色器 uniform 仅首绑同步（_didUploadHash） | P/rendering/batcher/gl/GlBatchAdaptor.ts:44-82 |
| WebGPU 执行 | 每 Batch 一次 drawIndexed：纹理批 → BindGroup（缓存于 batch.bindGroup）+ pipeline + `renderPassEncoder.drawIndexed(size,1,start)` | P/rendering/batcher/gpu/GpuBatchAdaptor.ts:66-103 |
| Canvas 执行 | CanvasBatchAdaptor 以 Canvas2D drawImage/pattern 语义执行同一批（含 FillPattern 矩阵） | P/rendering/batcher/canvas/CanvasBatchAdaptor.ts:1-23 |
| 场景接入 | SpritePipe：sprite → BatchableSprite（transform/texture/bounds/roundPixels）→ `_renderer.renderPipes.batch.addToBatch`；不动时 `checkAndUpdateTexture` 仅换纹理槽不重排 | P/scene/sprite/SpritePipe.ts:36-78 |
| 粒子路线 | ParticleContainer 独立管道：dynamic/static 属性拆分（静态属性一次上传，动态逐帧），绕过逐精灵对象开销 | P/scene/particle-container/shared/ParticleContainer.ts:200-224、ParticleContainerPipe.ts:29 |

对照结论：PixiJS 的"十万级"吞吐来自三个叠加——quad 位打包顶点（12+12
字节级开销）、纹理槽批（一次 draw 摊薄 N 精灵）、RenderGroup 增量指令流
（结构不变只更新变换/数据位）。这与 Excalidraw（canvas-a §2：全量线性扫描
+ per-element 位图缓存）、Konva/Fabric（逐对象 Canvas API）是数量级差异的
机制来源。【推断】"十万级"未在本仓内找到基准测试佐证（tests/visual 为像素
快照非性能），README 仅自称"fastest, most lightweight"（pixijs/README.md:31）；
数量级判断按机制外推，登记为推断。

### 3.3 纹理管理

| 环节 | 实现 | 锚点 |
|---|---|---|
| 模型 | Texture = TextureSource（GPU 资源 + resolution/antialias/mipmap 选项）+ frame/uv（图集区片）；spritesheet/compressed-textures（basis/dds/ktx/ktx2）为独立模块 | P/rendering/renderers/shared/texture/sources/TextureSource.ts:38-160、P/compressed-textures/ 目录 |
| 单元管理 | WebGL `_boundTextures[location]` 记账式绑定（已绑即跳过）；float/integer 采样冲突防御（unbindIntegerTextures 掩码清位） | P/rendering/renderers/gl/texture/GlTextureSystem.ts:145-187、:197-213 |
| GC | 旧 TextureGCSystem（帧计数：maxIdle 3600 帧收、每 600 帧扫，P/rendering/renderers/shared/texture/TextureGCSystem.ts:63-84）→ 8.15.0 起统一 **GCSystem（时间制：60s 未用即收、30s 扫一次，覆盖资源+renderable）** | P/rendering/renderers/shared/GCSystem.ts:96-117 |
| 池 | TexturePool（渲染目标复用）+ CanvasPool | P/rendering/renderers/shared/texture/TexturePool.ts、CanvasPool.ts |
| 资产 | Assets 异步加载器（resolver/detections/cache/parser 分层） | P/assets/ 目录 |

### 3.4 命中检测（几何递归，opt-in）

PixiJS 事件系统默认**关闭**（`eventMode='passive'` 起 opt-in），命中 =
EventBoundary 场景树递归：hitPruneFn（显式 hitArea 逆变换判界）→
hitTestFn（无 hitArea 时看 Renderable.containsPoint——Sprite 为四边形逆
变换、Graphics 为路径几何判定；两者皆无则**不命中**，连 bounds 都不用，
源码留 TODO）——P/events/EventBoundary.ts:253-269、:583-640（:637 TODO
注释）。与 Konva（始终像素级）形成第三种姿态：**编辑不是 PixiJS 的主业**，
事件是为交互体验（hover/点击）服务的低频路径。

### 3.5 PixiJS 路线小结

| 维度 | 判定 | 依据 |
|---|---|---|
| 渲染模式 | 保留模式场景图 → 增量指令流 → 三后端批执行 | §3.1-3.2 |
| 命中精度 | 几何递归 + per-view containsPoint；默认 passive | §3.4 |
| 矢量 | Graphics 自建 Path/ShapePath 体系（非 SVG），SVG 仅 path/stroke/gradient 子集进出（SVGParser.ts:37、SVGExporter.ts:21） | P/scene/graphics/shared/svg/ |
| 文本 | Canvas 2D 栅格化 → 纹理（CanvasTextGenerator/CanvasTextMetrics），位图文本走 MSDF | P/scene/text/Text.ts:215-228、P/scene/text/sdfShader/ |
| 动画 | 仅 Ticker（RAF）+ AnimatedSprite；无内建 tween/时间轴 | P/ticker/Ticker.ts:71、:284 |
| 服务端 | 无 | §0.1 |
| 许可 | MIT | §0.2 |

## 4. Q4 选型矩阵：五路线渲染光谱

### 4.1 主表（三基座 + canvas-a/b 两参照）

| 维度 | Konva 10.7 | Fabric 7.4 | PixiJS 8.22 | Excalidraw（canvas-a §2） | tldraw（canvas-b §2） |
|---|---|---|---|---|---|
| 渲染路线 | Canvas2D 保留模式场景图 | Canvas2D 对象模型（即时+对象缓存） | WebGL/WebGPU/Canvas2D 指令流批渲染 | Canvas2D 即时（双画布+全量重绘） | DOM+SVG 保留模式（html-layer CSS transform） |
| 真相源 | 场景树（Node 树） | `_objects` 数组 + Group 树 | Container 树（renderGroup 增量簿记） | elements 数组（不可变+nonce 失效） | record store（响应式信号） |
| 命中策略 | color-key 像素命中（网格 snap+环形搜索） | OBB 几何 + opt-in 像素精化 | 几何递归 containsPoint（默认关） | 视口 AABB 线性扫描 culling | DOM 原生命中 + hitTestMargin 3px |
| 层级/裁剪 | 层=独立 canvas；clipFunc/clip rect | Group/Layout + clipPath 对象 | RenderGroup/RenderLayer；effect/mask 管道 | frame+group（逻辑层） | DOM z-index（id 稳定序+置换缓存） |
| 文本 | 画布内自绘（measureText 缓存） | 画布内编辑（IText/Textbox）+ grapheme 测量 | 栅格化纹理/MSDF 位图 | 自制排版+contenteditable 覆盖 | 真 DOM 文本+隐藏测量池 |
| 矢量进出 | 无（Path 手写 SVG path 串解析） | **13 类对象 ↔ SVG 双向**（9 标签读+全模型写） | Graphics↔SVG path 子集（自建 Path 体系） | .excalidraw JSON 单向 + SVG/PNG 导出 | .tldr schema + SVG 导出（foreignObject） |
| 服务端渲染 | ✅ node-canvas/skia | ✅ jsdom+node-canvas | ❌ | ❌（仅浏览器） | ❌（SSR 占位）【推断，canvas-b §2 未证】 |
| 对象量级 | 千级（cache/listening 节流） | 千级（2Mpx 缓存上限） | **十万级**（批+槽+增量指令） | 千级（位图缓存+全量重绘） | 万级（R-tree culling+O(1) 广播） |
| 许可 | MIT | MIT | MIT | MIT | source-available（ECDSA 验签+水印） |
| 规模（核心 src） | 根层 14.2k / 全 src 24.1k | 45.3k | 122.5k | —（canvas-a 基线） | —（canvas-b 基线） |

### 4.2 分场景裁决表（工作台视角）

| 工作台场景 | 首选基座 | 判据 | 不可选的硬理由 |
|---|---|---|---|
| **交互编辑画布**（选中/变换/句柄/对齐） | Fabric > Konva | Fabric 自带 Control 句柄系统+IText 画布内编辑+对齐辅助线（编辑器完备度最高）；Konva Transformer 次之 | PixiJS 事件默认关、无句柄系统；Excalidraw/tldraw 引擎不可拆（canvas-a/b 定谳：内核与产品一体） |
| **矢量创作**（SVG 进出/图形保真） | Fabric（唯一双向）；严无损场景三者皆不足 | 13 类对象 ↔ SVG 双向 + 渐变/clipPath/CSS 三级样式 + CVE 修复中的导出消毒 | Konva 无 SVG 序列化；PixiJS 仅 path 子集；【推断】任何"对象模型↔SVG"往返都非无损（Fabric 9 标签读侧 + 4 位小数写侧量化）——无损需求应走 SVG 原文保存（同 office-b §4"未知 part 透传"结论的画布版） |
| **动画性能**（十万级/粒子/回放） | PixiJS（唯一解） | 三后端批渲染+纹理槽+增量指令流+GPU GC | Canvas 2D 路线（Konva/Fabric/Excalidraw）逐对象 draw call 无数量级余量 |
| **服务端出图**（缩略图/导出流水线） | Konva ≈ Fabric | 双双可 headless（Konva skia 后端更原生；Fabric jsdom 桥） | PixiJS 无服务端面 |
| **AI 工作台画布嵌入**（本线立场） | 三者皆非产品内核候选 | 三者都是**渲染基座**而非协同编辑内核（无 CRDT/OT/历史栈内建）——数据模型应工作台自持（同 canvas-a/b §13 结论：shape util/store 自研，渲染层可换） | — |

### 4.3 关键机制对照（命中/缓存/抗锯齿/节流）

| 机制 | Konva | Fabric | PixiJS |
|---|---|---|---|
| 命中成本 | 每监听层 1 张 stage 级位图（不监听=0×0）+ 查询读 1 像素 O(1) | O(N) 倒序线性扫 + OBB 逆变换 | O(树高×扇出) 递归 containsPoint |
| 边缘容错 | alpha≥128 阈值 + 色键网格 snap + 半径 10→256 环形搜索 | targetFindTolerance 像素容差（opt-in） | 无（几何即精确） |
| 浏览器怪癖防御 | Brave canvas farbling 探测（#282828 读回）+ 网格步长 3→5 | 无（node-canvas patternQuality 选项为服务端怪癖） | WebGL if 语句数上限探测；float/integer 采样单元冲突掩码 |
| 重绘节流 | RAF 合并 + 拖拽期命中图跳过 | RAF 合并（renderOnAddRemove 开关） | 增量指令流（结构不变不重建） |
| 缓存上限 | 无显式上限（GC 靠 releaseCanvas） | 2Mpx/4096 边硬上限（可配置） | GPU 侧时间制 GC（60s/30s） |

## 5. Q5 与动画线的关系：PixiJS 回放基座预留（数据模型级登记）

规划 §5.4 动画线暂缓，本 pass 仅做**数据模型级登记**（不做选型承诺）：

1. **PixiJS 是 P3 动画线回放引擎的唯一候选基座**（三基座中唯一具备
   GPU 批渲染 + 十万级吞吐 + 三后端降级），但**不是动画系统本身**：
   v8 仅提供 Ticker（RAF 时间源，P/ticker/Ticker.ts:71、:284）与
   AnimatedSprite（帧序列），无 tween/时间轴/关键帧——这些在生态层
   （GSAP/pixi-tween）。【推断】若动画线激活，形态应为
   "工作台动画数据模型（关键帧/时间轴）→ PixiJS 场景图驱动"，而非
   依赖 PixiJS 生态的动画表达。
2. **数据模型预留判据**（本次勘察登记，供 P3 起点直接取用）：PixiJS
   场景树的序列化面**没有官方 schema**——Container/Sprite/Graphics 均
   运行时对象，无 toJSON/toObject 等价物（对比 Fabric Object.toObject/toJSON、
   Konva Node.toJSON=toObject 的 JSON 串，K/Node.ts:1696-1698）。【推断】回放通道应把"动画状态"表达为
   关键帧 + 目标寻址（scene graph path），回放时由驱动层重建 PixiJS
   对象树；即：**动画数据模型须工作台自持，PixiJS 只承接渲染帧**——
   与 §4.2 "渲染层可换"立场一致。
3. **回放保真注意**：PixiJS 文本=栅格化纹理（分辨率绑定）、纹理
   antialias/mipmap 为 per-source 选项（P/rendering/renderers/shared/
   texture/sources/TextureSource.ts:38-160）——动画回放若含文本/缩放
   交互，需在数据模型层登记 resolution/scaleMode 决策位。

## 6. 对工作台的集成输入（可移植结论）

1. **渲染层与数据层解耦是三仓共同架构姿态**（PixiJS 指令流最彻底：
   场景侧产出 InstructionSet，后端执行器可换）。工作台画布应沿
   canvas-a/b 已立的"store 自持 + shape util 注册表"路线，渲染后端留
   可替换接口——本 pass 三基座即三类后端实现参照。
2. **命中检测三态选型**（§4.3）：像素命中（Konva color-key）适合"所见
   即所得"手绘类；几何 OBB（Fabric）适合编辑器性能敏感场景；几何
   递归（PixiJS）适合游戏化交互。工作台白板（draw.io 对位）建议几何
   命中为主 + 命中区显式声明（对齐 canvas-b shape util 的 hitTestMargin
   思路），Konva 的 color-key 网格 snap 方案登记为像素命中兜底技术
   （若未来做 freedraw 复杂笔迹命中）。
3. **SVG 往返保真判据**（Fabric 面的教训）：对象模型 ↔ SVG 的双向
   转译必有损（9 标签读侧 + 4 位小数写侧 + mask/filter 不支持）。
   工作台矢量面若重保真：原文（raw SVG）与编辑模型分层存
   ——编辑时按能力子集转译，导出时优先原文透传。【推断】与
   office-b §4 的 OOXML 保真结论同构。
4. **导出消毒面**：Fabric CVE-2026-44311 的修复形态（导出样式值过
   白名单正则闸，F/util/internals/svgExportCheck.ts:1-8）是工作台
   SVG/HTML 导出通道的最小参照实现。
5. **服务端出图通道**：Konva skia-canvas 后端证明 Canvas 2D 场景图可
   无头渲染出图——工作台缩略图/导出服务若选 Canvas 2D 路线，该形态
   （后端注入 createCanvasElement/createImageElement，K/skia-backend.ts:12-30、K/canvas-backend.ts:21-40）
   可直接照搬。
6. **GPU 资源治理参照**：PixiJS GCSystem 时间制（60s 未用收/30s 扫）
   + 绑定记账 + 帧计数旧制对比（§3.3），为工作台未来 GPU 面（如
   WebGL 滤镜/纹理图集）提供资源生命周期治理的成熟样本。

## 附：证据锚点密度

- `路径:行号` 锚点 97 处（去重口径；R4-fix 分项修正并注明口径：原记「实例
  97 处、K 32/F 30」——97 实为文件:行号去重计数，分项应为 **K 33 / F 29 /
  P 26 / 三仓根文件（package.json/LICENSE/CHANGELOG/README）9**；按出现
  实例字面口径为 104，含 7 处重复引用（K×4/F×1/P×1/根×1）；另目录级锚
  ~7 处不计入实例口径）；范围锚（如 K/Layer.ts:351-405）计 1 实例；路径
  前缀声明见头部。
- 正文对照表 15 张（§0.1/§0.2/§1.1/§1.2/§1.3/§2.1/§2.2/§2.4/§3.1/
  §3.2/§3.3/§3.5/§4.1/§4.2/§4.3；R4-fix 计数核对——worklog 登记时误记
  14 张，以本口径为准）。
- 【推断】显式标注 12 处（R4-fix 后）：§0.1 服务端【推断】、§0.2 子包许可覆盖
  【推断】、§1.3 数量级、§2.2 SVG 读侧语义转译、§2.3 量化损耗、§3.1
  RenderLayer 引入版本号（R4-fix 补标）、§3.2
  十万级外推、§4.1 tldraw SSR【推断】、§4.2 SVG 无损推论、§5.1 回放
  形态、§5.2 动画数据自持、§6.3 保真分层【推断】。
- 勘察范围：三仓全部为定向浅读（非 11 维深读）——Konva src 根层 25 文件
  中精读 Node/Shape/Layer/Stage/Context/Canvas/Util/Container/FastLayer
  9 件 + CHANGELOG；Fabric 精读 parser/ 全目录 + Object.ts/SelectableCanvas/
  StaticCanvas/config/cache/svgExportCheck/env.ts + CHANGELOG 版本史；
  PixiJS 精读 batcher/ 全目录 + RenderGroup 三件 + events/EventBoundary +
  GlTextureSystem/GCSystem/autoDetectRenderer + particle/text/sprite/
  svg 定位。未做实测复跑（对照 pass 体例，与 canvas-d 同级深度）。
- 复审建议：锚点行号均为单次读取快照（HEAD 见 §0.1），如后续三仓
  变更需按 HEAD 重核；本 pass 未 git commit，克隆工作树保留于
  analyses-workdir/render-bases/{konva,fabric.js,pixijs}/。
