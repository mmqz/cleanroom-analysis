# AFFiNE / BlockSuite 双态（doc ↔ edgeless）定向净室 Pass

> Task ID: r4-affine-a · Agent: 研究代理 J5b · 日期: 2026-10-12 · 净室只读，未 commit
> 对象仓：`toeverything/affine` 主仓（浅克隆 @ 9be1a82，本地路径 `analyses-workdir/affine/affine/`，下称 **AFF**）+ `toeverything/blocksuite` monorepo（随主仓 `AFF/blocksuite/` 目录就位的工作区快照，无 .git；下称 **BS**）
> 方法：grep 定向 → 只读命中文件；不做仓级漫游；git 历史未查（Q3 核验限定 CHANGELOG/README + 官方博客/web 各一次）。

## 0. 结论速览

1. **双态映射本质 = 单一 block 树 + 双视图层换装**：同一份 Yjs 支撑的 block 数据，画布几何（`xywh`/`index` z 序/`rotate`）与画布可见性（`displayMode`）都是 block model 的普通 props；doc/edgeless 两态各注册一套 view 扩展，一键切换 = 整体换装编辑器组件，数据层与 undo 历史原样保留。
2. **渲染层 = DOM(Lit) block 内容 与 Canvas 图形混合**：block 内容（笔记/富文本）始终 DOM 渲染、由 CSS transform 定位缩放；形状/连线等纯图形走 2D Canvas 层；另有 turbo-renderer 稳态位图优化层（OffscreenCanvas worker 位图；R4-fix 定位修正：原记「缩放平移时位图兜底」方向倒置——该层 enableBitmapRendering 默认 false 为 opt-in 能力，本快照 AFF app 配置未开启；启用时服务于稳态/平移期吞吐，交互与高倍缩放反而回退 DOM/占位符）。与 tldraw 全 DOM、Excalidraw 主画布方案形成三档光谱。
3. **build-vs-buy 核验**：一手证据（官方博客 in-repo 副本 + 代码 credit）证实"拼装 React 系开源编辑器（含 tldraw 类白板）不可行 → 全量自研"的决策叙事；但"早期版本曾嵌入 tldraw 后替换"的精确时间线未获一手核实，登记为【推断】。
4. **store**：Yjs 事实直接引注 collab-a（本 pass 不重复分析）；双态支撑面一句话——几何 props 与 surface 元素都在 store 里，两态共享同一数据层，画布编辑天然走协同合并。
5. **可移植结论**见 §5（核心：把"画布属性进数据模型"当作文档-画布融合的第一杠杆）。

---

## 1. 双态映射核心：block 模型如何挂载画布属性 + 切换状态保持

### 1.1 数据层：画布属性是 block 的普通 props（不是视图私有状态）

**Gfx 兼容 props 契约**（BS `framework/std/src/gfx/model/gfx-block-model.ts`）：
- `:32-36` `GfxCompatibleProps = { xywh: SerializedXYWH; index: string; lockedBySelf?: boolean }` —— 任何想上画布的 block 只需带这三个 props；
- `:42-45` `GfxCommonBlockProps` 在其上加 `rotate: number; scale: number`；
- `:52-56` `GfxBlockElementModel extends BlockModel<Props> implements GfxCompatibleInterface`；`:58-76` xywh 序列化字符串存 props、反序列化结果带缓存（`_cacheDeserXYWH`）；
- 序列化格式：BS `framework/global/src/gfx/xywh.ts:9` `SerializedXYWH = `[${number},${number},${number},${number}]``（模板字符串字面量类型）。

**挂载方式 = 模型 mixin**（doc block 定义处直接混入 Gfx 能力），以 note 块为例（BS `affine/model/src/blocks/note/note-model.ts`）：
- `:55-88` `defineBlockSchema({ flavour: 'affine:note', props: () => ({ xywh: '[0,0,W,H]', index: 'a0', displayMode: NoteDisplayMode.DocAndEdgeless, edgeless: { style: {...} }, ... }) })`；
- `:119-121` `class NoteBlockModel extends GfxCompatible<NoteProps>(BlockModel) implements GfxElementGeometry` —— mixin 把 containsBound/includesPoint/intersectsBound 等命中测试能力注入 doc block（`:127-142`）；
- **双态可见性进模型**：`displayMode` 枚举 `DocAndEdgeless='both' / DocOnly='doc' / EdgelessOnly='edgeless'`（BS `affine/model/src/consts/note.ts:38-42`）；note-model `:96-104` 留有 `hidden` → `displayMode` 的迁移注释（旧布尔 → 三态枚举）；
- **edgeless 专属外观进 props**：`:29-53` `NoteZodSchema` 的 `edgeless.style`（圆角/边框/阴影）+ `:107-117` `collapse/collapsedHeight/scale` 折叠形态——即 doc 模式不消费的字段就命名空间化挂在 `edgeless` 子对象下。

**纯画布图形（形状/连线/画笔）不占 block 节点**，收进一个专用 surface block（BS `affine/blocks/surface/src/surface-model.ts`）：
- `:15-36` `flavour: 'affine:surface'`，`props: elements: internalPrimitives.Boxed(new Y.Map())`，`role: 'hub'`、parent `@root`——画布元素是 root 下一个普通子 block 里装箱的 Y.Map 条目；
- root 块本身零画布状态：BS `affine/model/src/blocks/root/root-block-model.ts:8-10,53-63`（`affine:page` 只有 `title: Text`）。

**z 序 = 分数索引字符串**：`index: 'a0'` 由 `fractional-indexing` 生成（BS `framework/std/src/gfx/layer.ts:4` import `generateKeyBetween`），LayerManager 统一算 zIndex（BS `framework/std/src/view/element/gfx-block-component.ts:207-209`）。

**结论**：doc ↔ edgeless 是"同一数据两种投影"——投影所需的一切状态（几何、z 序、旋转、可见性、画布外观）都在模型 props 上，视图层零拷贝可重建。

### 1.2 视图层：同 flavour 双套 view 扩展，edgeless 组件复用 doc 渲染

**入口分叉在 view 扩展注册**（BS `affine/blocks/root/src/view.ts`）：
- `:34,50-54` `setup()` 按 `context.scope` 分流（`isEdgeless(scope)` → `_setupEdgeless`，否则 `_setupPage`）；
- `:65-68` page 分支注册 `BlockViewExtension('affine:page', literal`affine-page-root`)`；edgeless 分支（R4-fix 拆锚）：`:71-75` 注册 `EdgelessRootService` + `ViewportElementExtension`，`:86-91` 注册 `BlockViewExtension('affine:page', literal`affine-edgeless-root`)`（BlockViewExtension 调用在 :87-90；原记把 EdgelessRootService 并置在 `:86-91` 区间不确）—— 同一 `affine:page` flavour 按编辑器 scope 换绑不同自定义元素。

**edgeless 侧根组件**（BS `affine/blocks/root/src/edgeless/edgeless-root-block.ts`）：
- `:477-515` `renderBlock()`：`<gfx-viewport>` 容器内并排渲染 block 树子节点与 surface block 子节点（`:502-503` `renderChildren(this.model)` + `renderChildren(this.surfaceBlockModel)`）；
- 空间虚拟化：`:487-499` `maxConcurrentRenders=6` + `getModelsInViewport` 用 grid 索引只取视口 overscan 内的 block。

**edgeless 笔记组件直接继承 doc 组件**（BS `affine/blocks/note/src/note-edgeless-block.ts:32-34`）：
`class EdgelessNoteBlockComponent extends toGfxBlockComponent(NoteBlockComponent)`。
工厂 `toGfxBlockComponent`（BS `framework/std/src/view/element/gfx-block-component.ts:216-334`）关键语义：
- `:301-307` `renderGfxBlock() { return this.renderPageContent(); }` 而 `renderPageContent() { return super.renderBlock(); }` —— **edgeless 模式的笔记内容就是 doc 模式那棵 DOM，原样复用**；
- `:289-299` `renderBlock()` 把 `xywh` 解析成 `style.left/top/width/height/zIndex` 绝对定位；`:145-156` `getCSSTransform()` 由 viewport（translateX/Y + zoom/viewScale）合成 CSS `translate(...) scale(...)`——块内容不动，只改 CSS 变换；
- `:41-58` 三档虚拟化态：`transformState$ = 'active' | 'survival' | 'idle'`（idle = visibility:hidden 占位、survival = 可见禁交互）；
- 拖拽写回模型：`:236-246` `onDragMove` 直接 `model.xywh = ...serialize()`；`onDragStart/onDragEnd` `model.stash('xywh')/pop('xywh')`（实现在 store，BS `framework/store/src/reactive/proxy.ts:215,347`）——拖拽全程是一次可撤销事务而非流式提交。

**doc 侧对应过滤**（BS `affine/blocks/root/src/page/page-root-block.ts`）：
- `:407-415` `renderChildren` 过滤 `displayMode === EdgelessOnly` 的 note；`:389-397` 订阅 `propsUpdated(displayMode)` 驱动重渲染——EdgelessOnly 笔记是"画布原生、doc 隐身"的内容。

### 1.3 切换的状态保持语义（四层，全部一手锚点）

**① 模式值的两级语义**（BS `affine/shared/src/services/doc-mode-service.ts`）：
- `:11-57` `DocModeProvider` 接口显式区分 `setPrimaryMode`（持久化偏好，不影响当前编辑器）与 `setEditorMode`（当前编辑器换装）；`:9` 默认 `'page'`；
- `:95-100` `togglePrimaryMode` = `'page' ↔ 'edgeless'`；`DocMode` 类型定义 BS `affine/model/src/consts/doc.ts:4`。

**② app 层换装 = 整组件替换，数据零迁移**（AFF `packages/frontend/core/src/blocksuite/block-suite-editor/blocksuite-editor.tsx`）：
- `:259-275` `mode === 'page' ? <BlocksuiteDocEditor/> : <BlocksuiteEdgelessEditor/>`，两者绑定同一个 `page: Store`；
- `:38-48,126-150` `AffineEditorContainer` Proxy 统一暴露 page/host/std，把"哪个编辑器在跑"对上层透明。

**③ 视口/滚动按态分别持久化**：
- edgeless：BS `edgeless-root-block.ts:288-329` 挂载时从 `EditPropsStore` 读 `viewport`（zoom/centerX/centerY，兼容旧 xywh+padding），卸载时写回（`:322-328`）——**切走再切回，画布停在原地**；
- doc：AFF `packages/frontend/core/src/modules/editor/entities/editor.ts:45-55` scrollPosition 结构体 `page: number | null` 与 `edgeless: {centerX,centerY,zoom} | null` 分槽保存，`:125-130` workbench view 恢复时按当前态取对应槽。

**④ undo 历史跨态保留**（官方自述）：BS `docs-site/blog/document-centric.md:162` "switching content between document editors and whiteboard editors…all operation history recorded on the doc can be preserved, rather than reset"——因为历史记在 doc（数据层）而非编辑器实例上（同文 `:51` "document…should be maintained completely independent of the editor"）。

**补充**：`primaryMode` 落在 doc 元数据（AFF `packages/frontend/core/src/blocksuite/view-extensions/editor-view/doc-mode-service.ts:16-49` 转接 `docService/doc.setPrimaryMode`），链接文档嵌入块会订阅它决定展示形态（R4-fix 锚修正：订阅锚为 BS `affine/blocks/embed-doc/src/embed-linked-doc-block/embed-linked-doc-block.ts:534,536-540`——`getPrimaryMode` 读当前值 + `onPrimaryModeChange` 订阅变化驱动 `_linkedDocMode`；原记 `render-linked-doc.ts:94` 实为拖拽落画布时的 `setPrimaryMode('edgeless', ...)` 写调用，非订阅）。

---

## 2. edgeless 渲染层技术选型：DOM block + Canvas 图形混合（非单选）

三层结构，全部自研：

1. **block 内容层 = DOM（Lit Web Components）**：见 §1.2——笔记/富文本在 edgeless 里仍是 DOM，靠 CSS transform + 绝对定位投影到画布坐标系；`gfx-viewport` 元素做视口剔除与渲染节流（BS `framework/std/src/gfx/viewport-element.ts:165` 类定义；`:943-996` `scheduleUpdateChildren` 并发渲染调度，`maxConcurrentRenders` 默认 2）。
2. **纯图形层 = 2D Canvas**：surface block 的 `CanvasRenderer`（BS `affine/blocks/surface/src/renderer/canvas-renderer.ts:302`）持主 canvas（`:361-363` `canvas: HTMLCanvasElement; ctx: CanvasRenderingContext2D`）+ **stacking canvases 分层位图**（`:315-324` 栈式 canvas 池，按 dirty layer 局部重绘，`:392,:519` 动态 `document.createElement('canvas')`）。官方博客描述该设计为"图形元素自动合并进尽可能少的 canvas"（BS `docs-site/blog/document-centric.md:174-176`）。
3. **稳态位图优化层 = turbo-renderer（OffscreenCanvas worker 位图；R4-fix 定位修正）**：位图渲染是**稳态/平移期的性能优化**而非交互兜底——BS `affine/gfx/turbo-renderer/src/turbo-renderer.ts:73-74` 类注释原文 "rendering block content onto a canvas bitmap, **falling back to standard DOM rendering during interactions**"（交互期回退 DOM 的是位图层）；状态机 `refresh()`（`:295-339`）：稳态/平移 `drawCachedBitmap`，缩放手势走占位符（iOS 低缩放缓存位图例外），`zoom > zoomThreshold`（`:303-304` "falling back to DOM rendering"）显式回退 DOM。且 `defaultOptions.enableBitmapRendering: false`（`:64`）——**该能力 opt-in 默认关闭，本快照内「已布线未启用」**（AFF app 配置 `packages/frontend/core/src/blocksuite/view-extensions/turbo-renderer/turbo-renderer.ts:23-28` 只设 zoomThreshold/debounceTime，未开启 enableBitmapRendering）；`:103` 主线程持展示 canvas；绘制在 worker：BS `affine/gfx/turbo-renderer/src/painter/painter.worker.ts:32-33` `new OffscreenCanvas(0,0)` + `OffscreenCanvasRenderingContext2D`（Canvas 2D，非 WebGL）。

**与 canvas 线光谱对照（一句）**：BlockSuite edgeless 取"DOM 富文本 + Canvas 图形"混合位（tldraw 为全 DOM 渲染基座、Excalidraw 为主画布 + DOM 覆盖层——见 `cleanroom-analysis/canvas-line/canvas-b_tldraw深读报告.md` / `canvas-a_Excalidraw深读报告.md` / `canvas-e_渲染基座对照pass.md`），其自述动机正是 tldraw 类 DOM 白板在 ~2000 图元时平移缩放掉帧（BS `docs-site/blog/document-centric.md:178`）。

---

## 3. build-vs-buy 案例：tldraw → 自研 edgeless

**核验点 ①（仓内 CHANGELOG/README/代码）**：
- AFF 主仓与 BS 的 README/CHANGELOG 对 "tldraw" **零命中**（两仓全树 grep，仅 4 处命中均在 BS 内部文件，见下）；
- **代码级 credit（一手）**：BS `framework/global/src/gfx/model/vec.ts:1-2` 头注释 "Inlined from https://raw.githubusercontent.com/tldraw/tldraw/…/packages/vec/src/index.ts / Credits to tldraw"——向量数学模块直接从 tldraw 内联，佐证"参考其实现但未作为依赖引入"（BS 依赖表无 tldraw 包）；
- **官方博客 in-repo 副本（一手，最强证据）**：BS `docs-site/blog/document-centric.md`（"Building Document-Centric, CRDT-Native Editors"，Yifeng Wang/ewind1994，2024-01-10）：
  - `:24` "the front-end community offers many great rich text editors…and whiteboard editors (like tldraw)…Bundling various React-compatible editors together **seemed convenient - but proved impractical**"；
  - `:26` "The frustration encountered in directly integrating various open-source editors led us to…**decided to rebuild all necessary infrastructure** for our editors"（选 CRDT/Yjs 重建）；
  - `:174-178` 自研 canvas renderer vs "tldraw, the DOM-based open-source whiteboard" 在 2000 图元下的性能对照（见 §2）。

**核验点 ②（官方博客/web，一次即止）**：
- web 搜索 "blocksuite edgeless tldraw replace"：结果未直接命中"替换"叙事文章；blocksuite.io 官方站仅确认 "the BlockSuite framework allows runtime compatibility between the page editor and the edgeless editor, beyond mere static file format compatibility"（blocksuite.io Doc Editor 页，snippet 一手引用）。

**判定**：
- **可证（一手）**：拼装 React 系开源编辑器（tldraw 为白板代表）不可行 → 全量自研渲染/数据层；tldraw 的 vec 数学被内联致谢；自研 canvas 层以 tldraw DOM 方案为性能对照面。
- **【推断：未能一手核实】**"AFFiNE 早期版本曾实际嵌入 tldraw 作为 edgeless 引擎、后被自研替换"的精确时间线：两仓 CHANGELOG/README 零提及，本 pass 的 web 单次搜索亦未命中直接史料（blocksuite 本地副本无 .git，未做历史考古——按任务限定不越界）。登记为待核：若需定谳，需查 AFFiNE 2022 年 pre-BlockSuite 版本的依赖清单或官方公告。

---

## 4. BlockSuite store 对双态的支撑面（引注 collab-a，不重复分析）

BlockSuite store 与 Yjs 的绑定（`@blocksuite/store` → `yjs`/`y-protocols` 依赖、v13 线）**已由 collab-a 复审确认，直接引注**：`cleanroom-analysis/collab-line/collab-a_Yjs协议pass.md:503-506`（worklog 佐证 `worklog.md:328,334,425`）。本 pass 本地复核一致：BS `framework/store/package.json:23-24`（`y-protocols ^1.0.6`、`yjs ^13.6.27`）。

**双态支撑面（一句话级）**：store 把"画布属性"当作普通响应式 props（xywh/index/displayMode 走 props→Yjs 同步路径），surface 图形收在 `Boxed(Y.Map)`（§1.1），因此**双态不是两份数据而是一份 Yjs 文档的两个投影——几何拖拽、画布增删天然进协同合并与 undo 栈**，这正是切换时历史不重置（§1.3-④）的数据层前提。

---

## 5. 对工作台的可移植结论（按优先级）

1. **P0｜"文档↔画布融合"的正确杠杆是数据模型，不是视图**：把画布几何（xywh/rotate）、z 序（分数索引字符串）、双态可见性（三态 displayMode）做成 block 的普通 props，视图层两态各自投影。工作台的"卡片在文档流与白板间互投"若在 mdcore 数据层加 `xywh + displayMode` 等价物，可获得零迁移切换 + 协同/undo 免费继承（对照 §1.1/§1.3-④）。**这是本 pass 对融合决策的最大输入**。
2. **P0｜edgeless 视图复用 doc 渲染（继承/包装而非重写）**：`toGfxBlockComponent(DocComponent)` 模式 + CSS transform 定位（§1.2），使画布上的"文档卡片"与文档模式像素级同源、编辑器行为同源。工作台白板侧节点应直接复用文档渲染组件，避免双实现漂移。
3. **P1｜纯图形走独立 Canvas 通道 + 分层位图**：DOM 管富文本/交互控件、Canvas 管高频重绘图形，stacking canvas 脏层重绘 + turbo-renderer 稳态位图优化（§2；R4-fix 限定修正：turbo 层为 opt-in 默认关闭，本快照未启用，原记「交互期 turbo 位图兜底」为「已验证路径」不确——应为「已布线路径」，启用需显式开启 enableBitmapRendering）——工作台画布若需 2000+ 图元级性能，stacking canvas 分层为已验证路径、turbo 位图为 opt-in 可选增强；但若量级在百级，tldraw/Excalidraw 的单层方案（canvas-a/b/e）仍可候选，BlockSuite 方案的代价是自研渲染层 + 双态视图两套维护面。
4. **P1｜视口与滚动按态分槽持久化**：`scrollPosition.page` 与 `viewport{center,zoom}` 分开存、切换即恢复（§1.3-③）——小成本高感知，工作台双态切换应原样照搬。
5. **P2｜模式值拆两级**：`primaryMode`（文档级偏好，影响嵌入口径）与 `editorMode`（当前会话态）分离（§1.3-①），链接/嵌入展示可订阅前者。工作台若做"文档可被白板内嵌"，此分层直接可用。
6. **P2｜拖拽事务化**：拖拽几何用 `stash/pop`（临时遮蔽 props，结束才提交事务，§1.2）——避免高频 xywh 更新刷爆协同/undo 栈，Yjs 侧工作台同理。

---

## 附：双仓许可核验（逐件）

| 对象 | 证据 | 判定 |
|---|---|---|
| AFFiNE 主仓整体 | AFF `LICENSE`（"Copyright (c) 2022-present TOEVERYTHING PTE. LTD."，含 carve-out：`packages/backend` 与 `packages/common/native` 走各自 LICENSE，其余按 `LICENSE-MIT` 即 MIT）；AFF `package.json` `"license": "MIT"` | 双层许可：主体 MIT + 后端 carve-out【一手】 |
| blocksuite 各包 | BS 本地快照 `affine/*/package.json` + `affine/*/*/package.json` + `framework/*/package.json` 全量扫描：70/70 声明 `"license": "MIT"` | 包级 MIT【一手】 |
| blocksuite 仓级 | 本地快照无根 `LICENSE`/`package.json`/`.git`（工作区目录快照），仓级 LICENSE 文件未在手 | 【推断：仓级应为 MIT（70 包声明 + 官方站口径），未逐字核验根 LICENSE 文本】 |

（净室纪律：全程只读，未复制任何上游代码文本入本仓；引用均为行级锚点与短摘要；未 git commit。）
