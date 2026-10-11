# canvas-f：InvokeAI Unified Canvas UX pass（四问题定向）

> **净室分析报告**（线 A 画布 P2 批次；本篇为重试轮——前次网关超时无产出）。
> 日期：2026-10-12。对象：invoke-ai/InvokeAI @ `aa46003`（master 浅克隆，
> 2026-10-10，`invokeai/version/invokeai_version.py:1` 自报 7.0.0-alpha.2-post
> （R4-fix 锚修正：该文件仅 1 行，原记 ：4）。
> 许可证：**Apache-2.0**（LICENSE 为 Apache-2.0 正文全（9 节，176 行；无
> END OF TERMS AND CONDITIONS 行与 APPENDIX 附文，R4-fix 措辞修正：原记
> 「标准全文」不确）；NOTICE 在根；
> 与规划预判一致，允许阅读学习与协议级借鉴，本 pass 零代码复制）。
> 方法：定向浏览——现役前端 `invokeai/frontend/webv1/src/features/controlLayers/`
> 为 Unified Canvas 实现主体（README 自述「The canvas is a fairly complex
> feature」，目录名 controlLayers 为 v6 改名后的现役 Canvas 系统【推断】，
> 早期版本的 unifiedCanvas 目录在本 HEAD 已不存在）；后端只读调用边界
> （图提交 / 图像服务 / 蒙版节点），不深入 diffusers 管线内部。
> **环境说明**：本克隆由前次超时代理配置了 `git sparse-checkout`
> （54 条 patterns，恰覆盖 controlLayers / graph builder / queue / services
> / webv2 canvas-engine 等本 pass 相关路径）；少量后端文件（sd/ 子目录等）
> 未检出，经 `git show HEAD:<path>` 只读读取，行号以 git 对象为准，报告中
> 注明「（git show）」。结论标注 `路径:行号` 证据锚点；【推断】= 分析员判断。
> 未 git commit。
>
> 一句话定性：**「画布即图层化蒙太奇 + 一次性前端合成」的 AI 绘画画布——
> 持久态全在 Redux（zod schema + `_version` 迁移），渲染为原生 Konva 命令式
> 模块层级；生成时前端把四种 entity 合成为「单张底图 + 两张灰度强度蒙版 +
> 逐层 control/regional 图」（stableHash 缓存免重合成），由 14 个按模型分发
> 的 graph builder（覆盖 16 个模型基座值）自动组装后端图（用户不编辑图），paste-back 在图内完成；
> 结果经 destination 会话寻址回流 staging 暂存区，accept 后落为新图层**。

## 0. 架构速览与断代背景

```
InvokeAI @ aa46003（7.0.0-alpha.2-post，前端重写期双轨）
  ├─ invokeai/frontend/webv1/   现役前端（React + Redux + RTK Query）
  │    └─ src/features/
  │         ├─ controlLayers/   ★ Unified Canvas（本 pass 主体）
  │         │    ├─ konva/      原生 Konva 类层级（CanvasManager 顶层模块）
  │         │    ├─ store/      canvasSlice（undoable）+ params + settings + staging
  │         │    ├─ components/ StagingArea / InpaintMask / CanvasEntityList / 工具面板
  │         │    └─ util/       canvasProjectFile（.invk 工程）/ bbox 缩放
  │         ├─ nodes/util/graph/generation/   14 个 build<BASE>Graph + addInpaint/
  │         │    addOutpaint/addImageToImage/addTextToImage（canvas 生成图组装）
  │         └─ queue/           useEnqueueCanvas / queueApi（enqueue_batch）
  ├─ invokeai/frontend/webv2/   重写中（ARCHITECTURE.md 172 行模块宪法）
  │    └─ src/workbench/canvas-engine/（309 个 TS，自研 Canvas2D，无 Konva）
  │       + src/features/generation/（webv2 目前唯一 feature——canvas 是
  │         workbench 内部模块而非 top-level feature，见 webv2/ARCHITECTURE.md
  │         「Workbench ownership destinations」表）
  └─ invokeai/app/              Python 后端（本 pass 只读调用边界）
       ├─ invocations/canvas.py         canvas_output 节点（staging 回流入口）
       ├─ invocations/image.py（git show）  expand_mask_with_fade / img_noise /
       │                                 apply_mask_to_image / canvas_paste_back
       ├─ invocations/composition-nodes.py（git show）  invokeai_img_blend（vendored，
       │                                 源自 dwringer/composition-nodes，:1 自述）
       ├─ invocations/infill.py         5 种 infill（patchmatch/lama/cv2/tile/color）
       ├─ invocations/sd/create_gradient_mask.py（git show）  渐变去噪蒙版内核
       └─ services/session_queue/       队列会话与 destination 寻址
```

- [事实] 仓库处于 v7 alpha 断代期：`webv1`（src/features 三目录
  controlLayers/nodes/queue）与 `webv2`（src/features 仅 generation + workbench）
  并存；webv2 的 ARCHITECTURE.md 明言 canvas 未来是 workbench 深层内部模块
  （`canvas-engine/api.ts` 单一入口），不再是独立 feature（webv2/ARCHITECTURE.md
  「Workbench ownership destinations」表，`Current Workbench path` → `Target owner: Workbench` 行）。
- [事实] webv2 canvas-engine 已达 309 个 TS 文件，文档化契约
  `CanvasDocumentContractV3` 四 stack 森林 + 10 层组嵌套 + 10,000 节点上限
  （webv2/src/workbench/canvas-engine/ARCHITECTURE.md「Document contract
  and hierarchy」段）；其测试面为「real-Canvas2D suite」（同文件：209）——
  【推断】webv2 渲染层弃 Konva 改自研 Canvas2D（文档全文零 Konva 字样）。
- [事实] 本 pass 以 webv1 为 UX 对象（现役用户可见行为），webv2 仅登记
  方向性事实，不展开。

## 1. Q1 数据模型：图层 / 遮罩 / 再生区域的表达

### 1.1 四种 canvas entity（判别联合）

- [事实] entity 根 schema：`zCanvasEntityBase = { id, name, isEnabled,
  isLocked }`（store/types.ts:460-465），四种类型构成判别联合
  `_zCanvasEntityState`（store/types.ts:725-731）：

| entity 类型 | 专有字段 | 语义 |
|---|---|---|
| `raster_layer` | position / opacity / objects / adjustments（simple+curves LUT）/ globalCompositeOperation / isTransparencyLocked（store/types.ts:693-704） | 传统像素图层，Photoshop 式 |
| `control_layer` | raster_layer 全字段 + withTransparencyEffect + controlAdapter（controlnet/t2i_adapter/control_lora/z_image_control/anima_lllite 五判别，store/types.ts:707-717） | 内部是栅格层，语义上是 ControlNet 数据载体 |
| `regional_guidance` | fill / positivePrompt / negativePrompt / referenceImages / autoNegative（store/types.ts:529-539） | 蒙版形状 + 提示词/参考图条件，「条件只作用于蒙版区域」 |
| `inpaint_mask` | fill / opacity / noiseLevel / denoiseLimit（0-1 可选，store/types.ts:542-551） | 再生区域蒙版 + 两个逐层强度属性 |

- [事实] README 对后三者的定位描述：regional guidance「mask-like layer
  (arbitrary shapes but no color or texture) plus conditioning data」；inpaint
  mask「indicate regions to inpaint/regenerate」（features/controlLayers/
  README.md「Layers (Entities) and Adapters modules」段）。
- [事实] entity 分组存于 canvas 状态根：`zCanvasState = { _version: 3,
  selectedEntityIdentifier, bookmarkedEntityIdentifier, inpaintMasks /
  rasterLayers / controlLayers / regionalGuidance（各 `{ isHidden, entities: [] }`）,
  bbox }`（store/types.ts:1137-1147）。四组各自的 `isHidden` 是「整类一键
  隐藏」全局开关。
- [事实] **生成图像的元数据里携带完整画布状态**：`zCanvasMetadata` 直接
  收纳四种 entity 数组（store/types.ts:1182-1189）——「从结果图像恢复画布」
  （recall）路径的数据基础【推断：recall 交互按钮在 UI 中存在，此 schema
  为其服务】。

### 1.2 对象层：十种对象判别联合 + 蒙合成的矢量表达

- [事实] `zCanvasObjectState` 联合（store/types.ts:358-368）：`image` /
  `brush_line` / `brush_line_with_pressure`（点列 `[x,y,pressure,...]`，
  store/types.ts:222-236）/ `eraser_line(+pressure)` / `rect` / `oval` /
  `polygon` / `lasso`（闭合轮廓，store/types.ts:277-287）/ `gradient`
  （linear/radial 判别，store/types.ts:313-348）。R4-fix 口径修正：type
  literal 判别值实测 **10 席**（brush_line/brush_line_with_pressure/
  eraser_line/eraser_line_with_pressure/rect/oval/polygon/lasso/gradient/
  image，store/types.ts:210/224/240/252/267/291/301/279/315/352）——原记
  「九种」系把 eraser_line(+pressure) 两席并作一席计数，本清单本身已含
  全部成员。
- [事实] 图像对象双形态引用：`image: z.union([zImageWithDims,
  zImageWithDimsDataURL])`（store/types.ts:350-356）——后端图像名引用
  `{ image_name, width, height }`（store/types.ts:41-46）或前端 dataURL；
  ref 图像另支持裁剪盒（`zCroppableImageWithDims`，zod preprocess 做旧数据
  迁移，store/types.ts:54-84）。
- [事实] 擦除不是对象类型而是**合成模式**：`globalCompositeOperation` 字段
  挂在 brush_line 上（可选，store/types.ts:218/234），rect/lasso 的
  `compositeOperation` 枚举含 `destination-out`（store/types.ts:263/275）；
  README 专节「Erasing is hard」：Canvas 2D 无局部合成模式，靠
  destination-out「cut a hole」+ 离屏读回计算视觉边界，重活进 web worker
  （README「Erasing is hard」段；worker 入口 konva/CanvasWorkerModule.ts，
  README 同段引用）。
- [事实] 「保持矢量、需要时才光栅化」是显式设计决策：README「Rasterizing
  layers」段——矢量对象留在内存，仅在应用后端滤镜/画布变换时才光栅化
  （性能理由原文「We'd need to write the images to an offscreen canvas, …
  This would be too slow and block the main thread」）。

### 1.3 生成区域：bbox（generation bounding box）

- [事实] `zBboxState = { rect { x,y,width,height }, aspectRatio
  { id, value, isLocked }, scaledSize, scaleMethod(none/auto/manual), modelBase }`
  （store/types.ts:806-820）；`GenerationMode = 'txt2img' | 'img2img' |
  'inpaint' | 'outpaint'`（store/types.ts:1239）。
- [事实] bbox 尺寸必须对齐模型基座的网格：`getGridSize` 按 base 返回
  8（sd-1/sd-2/sdxl/anima 及默认）/ 16（flux/flux2/sd-3/qwen-image/wan/
  z-image/ernie-image/krea-2/ideogram-4）/ 32（cogview4/minimax-h3/ltx-2，
  其中 minimax-h3 注释明言「16x VAE spatial compression x patch size 2」）
  （features/parameters/util/optimalDimension.ts:110-137，git show）。
  README 同义解释：模型宽高约束「related to the internal "latents"
  representation」（README「The generation bounding box」段）。
- [事实] bbox 工具的交互对齐：位置取整到 `positionGridSize`
  （konva/CanvasTool/CanvasBboxToolModule.ts:305-310）；resize 时
  `gridSize = ctrl || meta ? getBboxGridSize() : 64`（默认粗对齐 64，
  按住 Ctrl/Meta 才细对齐模型网格；再按 **Alt** 网格加倍（居中缩放
  $altKey，CanvasBboxToolModule.ts:347-348；R4-fix 键位修正：原记
  「Shift ×2」不确——Shift 为宽高比锁定 ：322，Alt 为居中缩放 ：321，
  docstring 锚 CanvasBboxToolModule.ts:320-323）。
- [事实] 缩放档位：`scaleMethod` 三值（store/types.ts:720）；`auto` 下
  bbox 尺寸变化时重算 `scaledSize`（canvasSlice.ts `syncScaledSize`，
  store/canvasSlice.ts:2024-2037）——生成时可用比原尺寸更优的
  `scaledSize` 走管线，再缩回原尺寸（见 §2.1 图内 resize 双向链）。

### 1.4 前端状态树与持久化

- [事实] canvas slice 注册于根 store 且**用 redux-undo 包装**：
  `[canvasSliceConfig.slice.reducerPath]: undoable(...)`（app/store/store.ts:118-121，
  git show）；undo 配置：`limit: 64` + 只认本 slice action + 快照恢复/
  工程载入不入历史 + 同型动作节流（`canvasUndoableConfig`，store/canvasSlice.ts:2041-2061）；
  `clearHistoryType: canvasClearHistory.type`（store/canvasSlice.ts:2045；
  R4-fix 锚修正：原记 ：2046）。
- [事实] 账号切换/登出时强制清空 undo 历史（跨账号 ctrl+Z 不回放上一账号
  画布）：`accountAwareRootReducer` 追加 `canvasClearHistory()` 二次派发
  （app/store/store.ts:152-170，git show，注释原文「the stacks keep the
  previous account's states, and ctrl+Z would walk the next account straight
  back into them」）。
- [事实] 持久化用 **redux-remember**（非 redux-persist）+ per-slice
  `persistConfig`；反序列化时 `zCanvasState.parse(state)` 做 zod 校验迁移
  （app/store/store.ts:184-201，migrate 调用在 ：191，git show；
  store/canvasSlice.ts:2067-2069
  `persistConfig.migrate`；R4-fix 锚修正：原记 store.ts:179-190 漏含
  :191 migrate 调用行、canvasSlice 原记 :2072-2076 实为 :2067-2069）。
- [事实] 画布工程文件 `.invk`：`CANVAS_PROJECT_VERSION = 1` + manifest
  （version/appVersion/createdAt/name）+ CanvasProjectState（四类 entity +
  bbox + 选中/书签标识）+ 图像名收集清单（util/canvasProjectFile.ts:16-17/
  33-43 及「Image Name Collection」region）——工程文件只存 `image_name`
  引用，像素仍在服务端。

### 1.5 后端张量边界：图像名引用 + 灰度 PNG + latent 域蒙版

- [事实] 前后端图像交换**全部走「上传 → image_name 引用」**：合成器把
  canvas 元素转 blob 后 `uploadImage({ file, image_category, is_intermediate:
  true })` 上传，后端返回 ImageDTO（konva/CanvasCompositorModule.ts:308-321）；
  蒙版同样以 PNG 灰度图上传（`canvas-grayscale-composite.png`，
  CanvasCompositorModule.ts:611-617）。张量只存在于后端图执行内部
  （如 `image_resized_to_grid_as_tensor`，invokeai/app/invocations/sd/
  create_denoise_mask.py:14，git show；R4-fix 锚修正：原记 ：12，
  import 行实在 :14）。
- [事实] **双灰度蒙版编码**：合成器 `getGrayscaleMaskCompositeImageDTO`
  白底 + `darken` 合成模式叠画各 mask 层（CanvasCompositorModule.ts:526-538）；
  alpha>127 判为蒙版覆盖区，灰度值 = `255 - 255 * attributeValue`
  （attributeValue 为该层的 noiseLevel 或 denoiseLimit，
  CanvasCompositorModule.ts:562-578）——即**逐像素强度蒙版**：纯白=保留、
  纯黑=全强度重生成、中间灰=部分强度。多次重叠时 darken 取更强者。
- [事实] 后端 `create_gradient_mask` 节点把灰度蒙版降采到 latent 域
  （`//8 LATENT_SCALE_FACTOR`，「Makes the filter 64x faster」，sd/
  create_gradient_mask.py:85-90，git show）做边缘扩张（Gaussian/Box Blur/
  Staged 三模式 kernel + `minimum_denoise` 阈值下限，同文件 :96-145），
  输出双产物：`denoise_mask`（「0.0 = fully denoised, 1.0 = preserved」，
  同文件 :29-33）+ `expanded_mask_area`（二值化回贴蒙版，同文件 :148-156）。
- [事实] 语义注释双证：前端 addInpaint 对 Anima LLLite 分支注释「canvas
  denoise-limit composite is white = keep / black = inpaint」（features/
  nodes/util/graph/generation/addInpaint.ts:208-209）。

## 2. Q2 画布状态 ⇄ 生成管线三段协议

### 2.1 提交段：模式推导 → 合成上传 → 自动构图 → 队列

- [事实] 入口 `enqueueCanvas`（features/queue/hooks/useEnqueueCanvas.ts:41）：
  ① 取 `destination = selectCanvasDestination(state)`（:46）；②
  `generationMode = await canvasManager.compositor.getGenerationMode()`（:57）；
  ③ 按 `model.base` 分发到 14 个 graph builder（覆盖 16 个 base 值）——sdxl/sd-1/sd-2/sd-3/flux/
  flux2/cogview4/qwen-image/z-image/ernie-image/krea-2/ideogram-4/external/
  anima/wan/minimax-h3（:60-93）；④ `prepareLinearUIBatch({ origin: 'canvas',
  destination })`（:119-131）；⑤ `queueApi.endpoints.enqueueBatch.initiate
  (batchConfig)`（:140-145）。用户在画布上**从不编辑图**——图是按模型与
  模式自动组装的。
- [事实] REST 边界：`enqueueBatch` mutation POST
  `/api/v1/queue/{queue_id}/enqueue_batch`（services/api/endpoints/queue.ts:30-38）。
- [事实] destination 语义：默认 = `selectCanvasSessionId(state)`（画布
  会话 ID），`saveAllImagesToGallery` 设置开启时改用字面量 `'canvas'` 绕过
  staging 直入画廊（features/nodes/util/graph/graphBuilderUtils.ts
  `selectCanvasDestination`，:68-77；注释原文「the canvas will stage
  images that have its session ID as the destination」）。
- [事实] canvas 会话 ID：`canvasSessionId: getPrefixedId('canvas')`（store/
  canvasStagingAreaSlice.ts:23），sessionReset 时换新 ID（:41-52）——一次
  staging 会话 = 一个 destination 值，丢弃/接受后即弃用。
- [事实] **生成模式由合成透明度推导而非用户选择**（UX 核心）：
  `getGenerationMode` 注释（konva/CanvasCompositorModule.ts:660-672）：
  合成 raster 层全透明 → txt2img；部分透明 → outpaint；不透明且 inpaint
  蒙版全透明 → img2img；不透明且蒙版有内容 → inpaint。透明度经
  `getImageDataTransparency`（:636-658）计算且按 hash 缓存
  （`generationModeCache`，:687-693）。
- [事实] 合成的分工（README「Compositing layers to prepare for generation」
  段 + 代码）：raster 层**合并为单张底图**；inpaint 蒙版**合并为单张灰度
  蒙版**；regional guidance 与 control 层**不合并，逐层单独上传**——
  composite 的对象集合即「同类型可见 adapters 序」（getVisibleAdaptersOfType，
  CanvasCompositorModule.ts:144-175，过滤 disabled 与空层）。
- [事实] 合成结果三缓存链：`getCompositeHash(adapters, {rect})` =
  各 adapter `getHashableState()` + rect 的 `stableHash`（:177-190）→
  canvasElementCache（HTMLCanvasElement，:211-217）→ imageNameCache
  （已上传 image_name，:273-285）→ 失效仅随状态 hash 变化。即**未改动的
  层组合在重复生成时零重复合成、零重复上传**。
- [事实] inpaint 模式图内管线（features/nodes/util/graph/generation/
  addInpaint.ts:47-351）完整链条：
  1. 底图：`getCompositeImageDTO(rasterAdapters, rect, { is_intermediate:
     true })`（:89-93）；
  2. 噪声蒙版：有 noiseLevel 属性的 mask 层 → `getGrayscaleMaskComposite
     ImageDTO(..., 'noiseLevel', preserveMask)`（:95-113）；
  3. 去噪限制蒙版：全部 mask 层 → 同函数 `'denoiseLimit'`（:116-125，
     未设置的层默认 1.0 全强度）；
  4. `scaledSize ≠ originalSize` 时图内先 `img_resize` 放大底图与蒙版再
     处理、生成后缩回（:127-233，resize 节点四连）；
  5. 噪声蒙版前置加噪：`img_noise`（gaussian + `noise_color: true` +
     「amount: 1.0 // the mask controls the actual intensity」，
     :149-160）——蒙版灰度直接控制噪声注入强度；
  6. `create_gradient_mask`（coherence_mode / minimum_denoise / edge_radius
     参数来自 params，:181-188）→ `denoise_mask` 输入 denoise 节点；
  7. 回贴准备：`expanded_mask_area` → `expand_mask_with_fade`
     （`fade_size_px = params.maskBlur`，:189-193/322-326）；
  8. **paste-back**：`invokeai_img_blend`，`layer_base` = 原底图、
     `layer_upper` = 生成图、`mask` = 扩展蒙版（:236-243）。R4-fix
     provenance 注：`invokeai_img_blend` 为 **vendored 节点**——所在文件
     invokeai/app/invocations/composition-nodes.py:1 自述「All nodes in
     this file are originally pulled from https://github.com/dwringer/
     composition-nodes」，:406 docstring「Originally created by @dwringer」；
     上游仓许可未随附，但本仓整体 Apache-2.0 覆盖，无许可风险，引用时
     按 vendored 资产登记上游出处；
  9. `outputOnlyMaskedRegions` 开启时改为 `apply_mask_to_image`（invert，
     :246-253）。
- [事实] outpaint 模式差异（addOutpaint.ts:39-372）：多三件套——
  `getInfill`（infill 方法五选一，见 §3.1）预处理底图透明区（:119/152）；
  `tomask` 把底图 alpha 转蒙版（:125-129）+ `mask_combine` 与 inpaint 蒙版
  合并（:130-135）——**outpaint 的再生区域 = inpaint 蒙版 ∪ 底图透明区**。
- [事实] canvas 输出节点字段：`selectCanvasOutputFields` 设
  `is_intermediate`（默认画布结果不进画廊，仅在 accept 后转正）+
  `use_cache: false` + id 前缀 `CANVAS_OUTPUT_PREFIX`（graphBuilderUtils.ts
  `selectCanvasOutputFields` :47-63 + :226 `isCanvasOutputNodeId` 前缀判定）。

### 2.2 接收段：destination 过滤 + 三通道事件 + 防陈旧

- [事实] staging 状态核为独立类 `StagingAreaApi`（nanostores，非 redux）：
  `StagingAreaAppApi` 接口定义三通道（components/StagingArea/state.ts:15-30）：
  `onItemsChanged`（队列列表查询订阅）、`onQueueItemStatusChanged`（WS）、
  `onInvocationProgress`（WS 进度，含中间预览图）。
- [事实] 通道接线（components/StagingArea/context.tsx）：进度/状态经
  `$socket` 的 `invocation_progress` / `queue_item_status_changed` 事件
  （:38-49）；items 经 `store.subscribe` + `buildSelectCanvasQueueItems
  (sessionId)`（= `queueApi.endpoints.listAllQueueItems.select({ destination:
  sessionId })` 过滤 discarded，:33/:50-58 + store/canvasStagingAreaSlice.ts:93-95）。
- [事实] 条目模型：每个 queue item 的每张 canvas_output 图像是独立 entry
  （多输出 item 展开，`$entries` computed，state.ts:194-215）。
- [事实] 输出图提取：`getOutputImageNames` 遍历 `item.session.
  source_prepared_mapping`，按 `isCanvasOutputNodeId` 前缀匹配源节点 →
  取 prepared 节点 result 的 ImageField/ImageFieldCollection（components/
  StagingArea/shared.ts:19-44，R4-fix 锚修正：原记 ：27-55 越出
  文件尾 11 行，实际为 getOutputImageNames 函数全块 ：19-44）——
  **前缀约定即输出契约**。
- [事实] 防陈旧双机制：① `_itemsEventGeneration` 计数器丢弃过期异步写
  （state.ts:103-104/:460-463/:549-553）；② `_seenItemStatusRanks` +
  `_seenItemStatusSequences` 单调记录（pending/in_progress → terminal 三级
  rank；state.ts:67-153）——防「终端状态被旧查询快照回退成 pending」；
  消失 item 的记录被 `_pruneSeenItemOrdering` 逐出防 Map 无界增长
  （:155-170）。
- [事实] 完成态图获取：`onItemsChangedEvent` 对 completed item 逐图
  `getImageDTO(imageName)`（经 `getImageDTOSafe`），并在并发写竞争时取
  「不少于已见数量」的一方（:528-565）；进度事件直接携带
  `progressImage`（`setProgress`，:642-654）。
- [事实] 自动切换两档（`switch_on_start` / `switch_on_finish`）：完成后
  不直接切选（socket 事件无完整 item），而是记 `$lastCompletedItemId`，
  待图加载完成 `onImageLoaded` 再切换（:439-453 的注释块 + :600-616）。

### 2.3 合成回流段：staging 暂存 → accept 落层

- [事实] staging 交互面：缩略图横条（React 渲染非 Konva，README「Staging
  generations」段）+ 选中图在画布上预览（konva/CanvasStagingAreaModule.ts，
  README 同段）；「生成开始后画布大部分锁定，直到 finalize」——锁定是
  显式 UX 取舍（README「Do not lock down canvas as much during staging」
  未来项自述「It's more locked-down than it needs to be」）。
- [事实] 逐条目操作：acceptSelected / discardSelected / discardAll / 上下
  张循环导航（state.ts:288-413）；onDiscard 取消未完成 item
  （`cancelQueueItem`），全空则 sessionReset（context.tsx:60-68）；
  onDiscardAll = sessionReset + `cancelQueueItemsByDestination`（:69-74）。
- [事实] **accept 回流落层逻辑**（context.tsx:75-98）：读取当前 bbox rect →
  生成图**等比缩放适配 bbox 内接**（scale = min(bbox.w/img.w, bbox.h/img.h)
  居中，:79-85）→ `rasterLayerAdded({ overrides: { position, objects:
  [imageObject] } })` 落为新栅格层（:88-92）→ `canvasSessionReset()` 换新
  会话 ID + 取消同 destination 剩余 item（:94-97）。另有「add as new layer /
  save to gallery」等变体（StagingAreaToolbarMenuNewLayerFromImage /
  StagingAreaToolbarSaveSelectedToGalleryButton，组件目录实存）。
- [事实] 后端 `canvas_output` 节点本体极薄：仅 load PIL → save 图像服务
  → ImageOutput（invokeai/app/invocations/canvas.py:10-24）；它的存在意义
  是给「workflow integration」提供把任意 workflow 结果送进 staging 的
  通道（docstring「Use this node in workflows intended for canvas workflow
  integration」，:16-19）——配合 `canvasWorkflowIntegrationSlice`（选中
  entity → 选 workflow → 填参数 → 以选中 ImageField 为源跑 workflow，
  store/canvasWorkflowIntegrationSlice.ts:11-19 的 state 字段）。

### 2.4 协议总览

```
[提交]  bbox + layers(redux) ──getGenerationMode(透明度推导)──▶ mode
         compositor: raster→1 底图 | masks→2 灰度蒙版 | control/rg→逐层图
         （stableHash 三级缓存；uploadImage is_intermediate）
         build<BASE>Graph(mode) ─▶ invokeai_img_blend(paste-back 在图内)
         POST /api/v1/queue/{id}/enqueue_batch  {origin:'canvas', destination:sessionId}
[接收]  WS invocation_progress(中间图) + queue_item_status_changed
         + listAllQueueItems({destination:sessionId}) 订阅
         （防陈旧：generation 计数器 + status rank/sequence 单调）
         getOutputImageNames：source_prepared_mapping 前缀匹配 canvas_output
[回流]  staging 条目（每图一条）→ 选中预览 → accept：
         bbox 内接等比缩放 → rasterLayerAdded(新层) → sessionReset
         （新 destination）→ cancelQueueItemsByDestination(清场)
```

- [事实] 失败路径：图构建失败 toast（含 `UnsupportedGenerationModeError`
  专项文案，useEnqueueCanvas.ts:96-115）；canceled/failed item 清空其进度
  图（state.ts:509-517）；合成器全程 `$isCompositing/$isProcessing/
  $isUploading` 三 atom 合成 `$isBusy`（CanvasCompositorModule.ts:68-76）。

## 3. Q3 UX 细节：操作语义

### 3.1 inpaint / outpaint 语义：模式即状态，不是按钮

- [事实] 用户没有「切到 inpaint 模式」的开关：**inpaint = 在 inpaint_mask
  层上画蒙版；outpaint = 把 bbox 框到已有内容之外**（透明底自动成为再生
  区域）。模式推导规则（§2.1）与 README 的 bbox 约束说明共同构成操作
  语义：bbox 部分盖住透明区 → outpaint。
- [事实] outpaint 透明区预处理（infill）五法：patchmatch（默认，可用性
  探测 `PatchMatch.patchmatch_available()`）/ lama / cv2 / tile / color
  （前端枚举 store/types.ts:841 `zInfillMethod`；后端
  invokeai/app/invocations/infill.py:22-30 + 69-147 各类实现；patchmatch
  支持 downscale 加速，:104-108）。【推断】infill 作用于「为模型提供透明
  区的假想底色」而非最终输出——输出仍由 paste-back 用原底图+蒙版混合
  （addOutpaint 的 blend `layer_base` 为 infill 前的原底图，addOutpaint.ts:248-255）。
- [事实] 逐 mask 层的再生强度双属性：`noiseLevel`（生成前对底图注入噪声
  的强度蒙版）与 `denoiseLimit`（去噪上限蒙版）均为可选 0-1
  （store/types.ts:548-549；UI 为 InpaintMaskNoiseSlider /
  InpaintMaskDenoiseLimitSlider，components/InpaintMask/ 目录实存）——
  两个正交旋钮分别控制「噪声前置注入」与「去噪步强度」，同一画布可并存
  不同强度的多张蒙版（合成时 darken 叠合，§1.5）。
- [事实] canvas coherence 三参数全局化：coherence_mode（Gaussian/Box/
  Staged）/ minimum_denoise / edge_size（后端 create_gradient_mask 的
  输入，前端 addInpaint.ts:184-186）——即「蒙版边缘过渡带」的老 inpaint
  pain point 在此以参数形式暴露而非自动。
- [事实] `preserveMask` 设置（默认 false）：作为 `invertMask` 传入灰度
  蒙版合成（CanvasCompositorModule.ts:505 + addInpaint.ts:107/120）——
  「保留蒙版外、重生成蒙版内」与反相之间的切换开关。

### 3.2 遮罩工具与蒙版呈现

- [事实] 工具枚举：`brush / eraser / move / rect / lasso / gradient /
  view / bbox / colorPicker / text`（store/types.ts:111）——蒙版层与栅格层
  共用同一工具集（README「A singleton module that handles tool-specific
  interactions」；CanvasBrushToolModule 等 per-tool 模块）。
- [事实] 蒙版层的可视化：`fill: { style: solid|grid|crosshatch|diagonal|
  horizontal|vertical, color }`（store/types.ts:510-513）；非 solid 图案由
  SVG pattern dataURL 染色实现（konva/patterns/getPatternSVG.ts:13-29，
  5 张内置 svg 换 stroke 颜色）+ 透明棋盘格底纹（transparency-checkerboard-
  pattern.ts）。
- [事实] mask 相关专用操作：蒙版反相按钮（EntityListSelectedEntityAction
  BarInvertMaskButton，组件实存）、蒙版区域提取（InpaintMaskMenuItems
  ExtractMaskedArea）、mask 层与 regional guidance 层互转（inpaintMask
  ConvertedToRegionalGuidance / rgConvertedToInpaintMask，store/canvasSlice.ts
  :1999/:2014）。
- [事实] SAM 辅助选择：点标签三值（background=-1 / neutral=0 /
  foreground=1，store/types.ts:154-158）+ SAM1/SAM2 双模型枚举
  （:166）；README「Selecting objects from the pixel data of a layer
  (aka segmentation tasks)」（konva/CanvasSegmentAnythingModule.ts）。
- [事实] 压感两条路径：`pressureAffectsWidth`（默认 true）与
  `pressureAffectsOpacity`（默认 false，store/types.ts:231-232）。

### 3.3 图层管理

- [事实] 实体列表 = 四栈合一列表 + 每栈头部（CanvasEntityList 目录：
  CanvasEntityGroupList / CanvasEntityList / 拖拽 useCanvasEntityListDnd）；
  选中实体操作栏：透明度 / 混合模式 / 复制 / 填充 / 滤镜 / 反相蒙版 /
  变换 / 保存到资产 / 对象选择（EntityListSelectedEntityActionBar* 10 件；
  R4-fix 计数修正：原记 14 件，实测容器 + 9 按钮 = 10 件）。
- [事实] 混合模式：全部 Canvas 2D 复合运算 26 种在后端 schema 允许
  （COMPOSITE_OPERATIONS，store/types.ts:643-670），UI 收窄为双兼容
  （Konva+CSS）子集 15 种（COLOR_BLEND_MODES，:675-691）——「live
  rendering 用 CSS blend」的注释（:641-642）。
- [事实] 栅格层调色：simple 六旋钮 + curves 四通道 LUT（store/types.ts:
  603-635），curves 以 LUT 存于 Konva 属性由滤镜应用（:621-625 注释）。
- [事实] `isTransparencyLocked`：笔迹只画在既有非透明像素上（preserve
  alpha，store/types.ts:702-703）。
- [事实] 实体状态三标志（isEnabled/isLocked/isHidden-by-group）+ 书签
  （bookmarkedEntityIdentifier，store/types.ts:1140）。
- [事实] 合并操作三式：merge visible（含类型过滤 getVisibleRectOfType，
  CanvasCompositorModule.ts:96-112）/ merge down / merge selected
  （getRectOfAdapters 含禁用层的注释差异，:114-133；mergeByEntity
  Identifiers 上传合成图并新建实体，:336-386）。

## 4. Q4 前端架构定位：渲染路线 + 状态管理

### 4.1 渲染路线：原生 Konva 命令式模块层级

- [事实] 渲染器为**原生 Konva（无 react-konva 绑定）**：「uses "native"
  KonvaJS (i.e. not the Konva react bindings)」（README 开篇）；上层是
  类层级：`CanvasModuleBase`（id/parent/manager/path/log/repr/destroy 契约）
  → `CanvasManager` 顶层 → per-entity adapter（CanvasEntityAdapterRasterLayer
  等）+ renderer / buffer renderer / filterer / transformer 子模块（README
  「Modules」「Layers (Entities) and Adapters modules」两段）。
- [事实] 绘制流（README「Canvas -> State」段）：笔迹画在 buffer renderer
  （进行中对象），完成后转入主 renderer 并 dispatch redux action——
  「We create/update/delete objects in Konva… serialize the object to a
  plain JS object and dispatch a redux action」。
- [事实] 每实体一个 Konva.Layer = 每实体一个 HTML canvas 元素；README
  「Future enhancements」承认这是性能天花板并给出三层压缩（活动层前后
  各拍平一层）与 Konva-in-worker 两个未做方案。
- [事实] 合成器对 canvas 2D 的 `imageSmoothingEnabled = false`
  （CanvasCompositorModule.ts:229）——合成路径零插值，保像素精确。
- [事实] 对照 canvas-e Konva 深读的框架级事实：InvokeAI 的用法落在
  Konva「保留模式场景图 + 每层双 canvas」的常规路径上（HitCanvas 命中
  由 Konva 内建），其增量在**应用层模块化**（adapter/buffer/cache 分层）
  而非改渲染内核。

### 4.2 三轨状态管理

- [事实] README「State mgmt」节的三轨分工（原文归纳）：**Redux = 持久
  真相源**（layers/order/properties，订阅无粒度）；**Konva = 单向推送
  目标**（「we almost exclusively push data to Konva, we never "read"
  from it」）；**nanostores = 模块内瞬态 + React 双向绑定**（工具态、
  compositor 忙闲 atom、staging 全部状态）。
- [事实] redux 桥接集中在 `CanvasStateApiModule`：提供
  `createReduxSubscription` 订阅工具 + 把 canvas 类模块的意图转成 slice
  action（konva/CanvasStateApiModule.ts:17-50 的 action import 面：
  entityBrushLineAdded / entityRasterized / bboxChangedFromCanvas 等）。
- [事实] canvas 五 slice 分工：canvasSlice（结构，undoable）/ paramsSlice
  （生成参数）/ canvasSettingsSlice（工具设置）/ canvasStagingAreaSlice
  （会话 ID + 丢弃集合）/ canvasTextSlice + canvasWorkflowIntegrationSlice
  （app/store/store.ts:39-43 注册面）。

### 4.3 画布渲染/状态光谱对照（与 canvas-a/b/e/penpot-a 并列）

| 维度 | InvokeAI webv1 | Excalidraw（canvas-a） | tldraw（canvas-b） | Konva/Fabric/Pixi（canvas-e） | Penpot（penpot-a） |
|---|---|---|---|---|---|
| 渲染 | 原生 Konva，每实体一 canvas 元素 | Canvas2D + rough.js 手绘风 | DOM/SVG 保留模式 | Konva/Fabric=Canvas2D 场景图；Pixi=指令流批渲染三后端 | SVG DOM + foreignObject + 自研 CLJC 排版 |
| 命中 | Konva color-key 像素命中（内建） | 几何命中（frame 瓶颈自认） | DOM 事件 + R-tree | Konva color-key / Fabric OBB / Pixi 几何递归 | DOM 事件 |
| 持久真相源 | Redux（zod schema + `_version` 迁移）+ redux-undo 64 步 | 自研 excalidraw 元素数组 + 快照式 | tldrecord store（响应式）+ 迁移链 | 库内模型（toJSON/SVG） | 9 类型扁平 record + changes |
| 瞬态 | nanostores 三轨之一 | 无（React state） | signals | 库外自理 | ratom |
| AI 耦合 | **一等公民**：合成器/构图器/staging 为核心模块 | 无 | 无（MCP exec 为 agent 面） | 无（渲染基座） | 无（插件/MCP 面） |
| 协同 | 无 | E2E+socket.io+LWW | 服务器定序+rebase | N/A | 服务器悲观锁+xlog |

- 说明：对照列仅引用已落盘报告结论（canvas-a §2/§14、canvas-b §2、
  canvas-e §4.1 五路线主表、penpot-a §1.4/§3；R4-fix 措辞修正：原记
  「已复审」，penpot-a 复审在档 r4-review-p），无新增未验证断言。

## 5. 生成管线侧对照（对 canvas-d ComfyUI）

- [事实] 两个系统同为「前端提交节点图、后端执行图」，但图的生产方式
  相反：ComfyUI 用户手编辑 workflow 图（canvas-d §1.1「编辑图与执行图
  彻底分离」）；InvokeAI 画布用户**零图编辑**，14 个 build<BASE>Graph 按
  模型+模式自动组装（useEnqueueCanvas.ts:60-93），图仅是传输格式。
- [事实] 节点命名域对照：InvokeAI 后端 invocations 与 ComfyUI nodes 同为
  `@invocation("type_id")` 注册式节点；InvokeAI 的 canvas 专用节点族
  （canvas_output / create_gradient_mask / expand_mask_with_fade /
  img_noise / invokeai_img_blend / infill_* / tomask / mask_combine）构成
  一条**可复用的 inpaint/outpaint 节点词汇表**——对 dsh 外置引擎的图
  词汇设计是最直接的参照物。
- [事实] 队列语义对照：ComfyUI `/prompt` + client_id 会话（canvas-d §6.1
  六端点）；InvokeAI `POST /api/v1/queue/{queue_id}/enqueue_batch` +
  batch 的 `origin/destination` 字段做画布会话寻址
  （queue.ts:30-38 + graphBuilderUtils.ts selectCanvasDestination）——
  destination 是**比 client_id 更细的结果路由**（同一客户端多画布会话
  可分址），cancelQueueItemsByDestination 提供按会话清场。
- [事实] 输出契约对照：ComfyUI 靠 OUTPUT_NODE 产物 + history；InvokeAI
  canvas 走「节点 id 前缀约定」（CANVAS_OUTPUT_PREFIX）+ session.
  source_prepared_mapping 前缀匹配提取（shared.ts:19-44）。
- [事实] 进度流对照：两者都有 WS 中间图（ComfyUI WS 二进制帧；InvokeAI
  `invocation_progress` 事件携带 progressImage，state.ts:642-654）。

## 6. 对 mdcore / dsh 工作台的集成输入

1. **三段协议可直接移植为 dsh 外置 AI 绘画引擎的画布侧协议**：提交段
   （画布状态 → 合成底图/蒙版 → 构请求）｜接收段（会话寻址 + 进度事件 +
   防陈旧单调性）｜回流段（staging 暂存 → accept 落层 + 会话重置 + 按
   destination 清场）。InvokeAI 的实现证明该协议可以完全架在「图像名
   引用 + REST + WS」上，不依赖任何 CRDT/OT。
2. **灰度蒙版 = 逐像素强度协议**：把「重生成强度」编码为蒙版灰度
   （255=保留 / 0=全强度），noise 与 denoise-limit 两张正交强度图分别
   在「前置加噪」与「去噪步」生效（addInpaint.ts:95-125 + create_
   gradient_mask.py:29-33）。这比二值蒙版表达力高一档，且对后端只是
   一张 L 模式 PNG——dsh 画布侧应一开始就按灰度蒙版设计蒙版数据结构。
3. **模式推导 vs 显式模式**：InvokeAI 把 txt2img/img2img/inpaint/outpaint
   从 UI 按钮降级为**状态推导**（合成透明度四象限，CanvasCompositor
   Module.ts:660-672）。收益是「少一个开关」，代价是模式不可见、用户
   需理解隐含规则（README 亦未在 UI 层提示）。dsh 建议折中：默认推导 +
   HUD 显式标注当前推导结果（InvokeAI 的 HUD 组件目录存在
   components/HUD/，但其内容未深读【推断：HUD 展示推导模式】）。
4. **合成缓存三链**：stableHash(可哈希层状态+rect) → canvas 元素缓存 →
   image_name 缓存（CanvasCompositorModule.ts:177-190/211-217/273-285），
   使重复生成零重复合成/上传。dsh 的「提交段」应内置同构缓存（Rust 侧
   即内容哈希 → blob 存储 name）。
5. **paste-back 放后端图内**（invokeai_img_blend：base/upper/mask 三入参
   + fit_to_width/height），并单独用 `expand_mask_with_fade`（fade_size_px）
   做回贴蒙版软化——保证「生成尺寸 ≠ 画布尺寸」时缩放往返由同一引擎
   完成。【推断】如此设计避免了前端二次采样的色彩/对齐漂移。
6. **staging 暂存区是「生成暂存 UI」成年样本**：每图一条目、多批次并存、
   逐条目 accept/discard、自动切换两档（on_start/on_finish）、防陈旧双
   计数（generation 计数器 + status rank/sequence 单调）。dsh 若做「AI
   生成结果不入文档直到用户裁决」，此为完整参照。
7. **画布元数据随图落盘**：zCanvasMetadata 把全部 entity 状态写进结果
   图元数据（store/types.ts:1182-1189）——任何结果图可重建其生成时的
   画布上下文。dsh 的图像资产应预留同构「生成上下文」字段。
8. **许可**：Apache-2.0 与 cleanroom 规则兼容；协议/交互语义可自由借鉴，
   代码仍零复制。

## 7. 四问题结论一览

| 问题 | 一句话结论 | 关键锚点 |
|---|---|---|
| 数据模型 | 四 entity 判别联合（raster/control/regional/inpaint-mask）+ 十种对象联合（literal 判别值 10，§1.2 口径注）+ bbox；蒙版 = 带 noiseLevel/denoiseLimit 双强度属性的矢量对象集合；后端边界 = image_name 引用 + 两张灰度 PNG + latent 域蒙版节点 | `store/types.ts:725-731`、`store/types.ts:542-551`、`CanvasCompositorModule.ts:562-578`、`sd/create_gradient_mask.py:29-33`（git show） |
| 三段协议 | 提交 = 透明度推导模式 + 前端合成（stableHash 三级缓存）+ 14 builder 自动构图 + enqueue_batch(destination)；接收 = destination 过滤 + WS 进度/状态 + 双重防陈旧；回流 = staging 每图一条目 → accept 等比内接落新层 + 会话重置清场；paste-back 在图内 | `useEnqueueCanvas.ts:41-147`、`CanvasCompositorModule.ts:660-737`、`StagingArea/state.ts:460-598`、`context.tsx:75-98` |
| UX 细节 | 模式即状态（无 inpaint/outpaint 按钮）；infill 五法仅喂模型不进输出；蒙版工具与笔刷同集 + 图案化呈现 + SAM 三值点选；图层 = 四栈列表 + 15 混合模式 + curves LUT + 类型互转；staging 期画布锁定为显式取舍 | `optimalDimension.ts:110-137`（git show）、`infill.py:22-30`、`getPatternSVG.ts:13-29`、README「Do not lock down…」段 |
| 架构定位 | 原生 Konva 命令式类层级（每实体一 canvas 元素）+ Redux(undoable 64)/Konva/nanostores 三轨；光谱上 = 白板产品中最重的「AI 管线耦合型」画布，渲染内核常规、应用层模块化深；webv2 重写弃 Konva 转自研 Canvas2D【推断】 | README「State mgmt」「Broad Strokes」段、`store.ts:118-121`（git show）、webv2 `canvas-engine/ARCHITECTURE.md`（309 文件） |

## 附：锚点与标注纪律

- 锚点形态：`路径:行号`；webv1/webv2 路径省略公共前缀
  `invokeai/frontend/`（如 `store/types.ts` =
  `invokeai/frontend/webv1/src/features/controlLayers/store/types.ts`）；
  后端路径以 `invokeai/app/...` 起；标「（git show）」者为 sparse-checkout
  未检出、经 git 对象只读读取的文件（行号与 HEAD 一致）。
- 锚点清单（本 pass 实例）：`路径:行号` 形态实例约 130 处（R4-fix 计数
  修正：原记「约 139/反引号完整锚 21」均偏高；实测全式 file:line 100 +
  裸续行锚约 30；含反引号完整锚 13——§7 结论表 12 + 头部 1；分布：§0 ~8、
  §1 ~33、§2 ~36、§3 ~18、§4 ~10、§5 ~6、附表 4 + 散布引用若干）。
- 【推断】标注：7 处（webv2 弃 Konva ×2——正文+结论表；zCanvasMetadata 服务于 recall 交互；
  controlLayers 为 v6 更名史；infill 不进最终输出的解释；paste-back 防前端
  二次采样漂移的动机；HUD 展示推导模式）。
- 净室合规：只读（磁盘文件读取 + `git show` 对象读取，零工作树修改、
  零 commit）；零代码抄录（全部为行为/结构描述与单行字段引用）；许可
  Apache-2.0 核验（LICENSE 正文 9 节全（176 行）+ NOTICE + pyproject）。
- 复审状态：已复审（r4-review-f 通过：两核心结论端到端实证；11 项低/信息级
  发现（锚点邻域漂移/计数/措辞口径）已经 r4-fix 本轮统一修补，均不触及
  技术论断）。
