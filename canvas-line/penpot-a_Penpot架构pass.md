# penpot-a：Penpot 架构定向 pass（线 A 画布 P2）

> **净室分析报告**（线 A 画布 P2 批次；开源 Figma 对位件）。日期：2026-10-11。
> 对象：penpot/penpot @ `10955f17e27943429a10e90360d0404e3f56e27c`（master 浅克隆
> --depth 1，2026-10-09 提交），根 package.json 自报版本 2.17.0（CHANGES.md:3 显示
> 2.19.0 Unreleased 在途）。方法：架构定向 pass——浅克隆后两面（frontend +
> backend）定向浏览，围绕五个净室问题取证；所有结论标注证据
> `路径:行号`，【推断】= 分析员判断。
> 纪律：只读不复制；路径约定 `F/` = frontend/src/app、`C/` =
> common/src/app/common、`B/` = backend/src/app、`E/` = exporter/src/app、
> `RW/` = render-wasm、`P/` = plugins/libs/plugins-runtime/src/lib、`M/` =
> mcp/packages。
>
> 一句话定性：**ClojureScript/Clojure 双面 + 共享 CLJC 数据层的"服务器定序
> changes-log"设计平台，经典渲染器是 SVG DOM + foreignObject 真 HTML 文本 +
> 自研 CLJC flex/grid 排版（浏览器当渲染引擎而非排版引擎），同时正在并行建设
> Skia/WebGL WASM 瓦片渲染器（用户可选、默认仍 SVG）**——"用 Web 标准表达设计"
> 是其路线主张，"DOM 性能天花板"是其正在用第二条渲染路线回应的问题。

## 0. 架构速览

```
penpot/penpot monorepo（9 顶层代码模块 + docker/docs/experiments/sample_media/
scripts 5 个非代码目录，单仓多运行时）
├─ common/      CLJC 共享数据层（175 文件 58,731 行）——形状/组件/changes 处理器
│               /flex+grid 排版/特性集/校验，前后端同一份代码
├─ frontend/    ClojureScript 工作台（697 文件 180,032 行，shadow-cljs→ESM）
│               ├─ 经典渲染器：SVG DOM（<svg id="render"> viewBox 相机）
│               │   + foreignObject 真 HTML 文本 + 帧位图缩略图 LOD
│               └─ wasm 渲染器（feature flag）：WebGL2 + Skia 瓦片
├─ backend/     Clojure JVM 服务（179 文件 50,255 行）——RPC 163 方法
│               PostgreSQL（xlog+快照）/ Redis msgbus / WebSocket 通知
├─ exporter/    Node ClojureScript 导出服务——Playwright 无头浏览器 或 wasm 无头渲染
├─ render-wasm/ Rust 渲染引擎（104 文件 46,516 行，Emscripten+rust-skia）
│               同源双 target：frontend（-O3）/ export（-Oz）
├─ plugins/     插件运行时+示例（TS）——SES Compartment 沙箱 + iframe UI
├─ mcp/         官方 MCP server（TS，stdio）+ MCP 插件（WS 连回 server）
├─ media-processor/  无状态 TS 服务：图片/字体处理
└─ library/     @penpot/library npm 包（CLJS）——无头构建 .penpot 文件
外部依赖服务：PostgreSQL / Redis(valkey) / S3 兼容存储(rustfs) / SMTP /
             LDAP / nitrate（企业版组织服务，仓外闭源）
```

## 1. CSS/DOM 渲染路线（本篇核心维）

### 1.1 定性核验：是"SVG DOM + 真 HTML 文本"，不是"CSS 盒模型画布"

任务书问题表述"用浏览器 CSS/DOM 渲染设计画布（非 Canvas 位图）"**方向正确但
需要精化**——三层事实：

1. **形状层是 SVG DOM，不是 CSS 布局的 div 树**。经典视口 `viewport-classic*`
   挂一个全屏 `<svg id="render" viewBox=...>`（F/main/ui/workspace/viewport.cljs:391-401，
   相机=viewBox，页面 z 序=SVG 文档序），由 `shapes/root-shape` 递归渲染形状树
   （viewport.cljs:431-436；F/main/ui/workspace/shapes.cljs:58-97）。帧=带
   clipPath/双 filter（blur 与 shadow 分层）的 `<g>` 容器（F/main/ui/shapes/frame.cljs:95-123）；
   9 种形状类型按 `case` 分派到 per-type 组件（shapes.cljs:112-121，类型表
   C/types/shape.cljc:70-79：frame/group/bool/rect/path/text/circle/svg-raw/image）。
2. **文本层是真 HTML/CSS（浏览器排版引擎）**。文本形状渲染为
   `<foreignObject>` 内的 `div.root > p.paragraph > span.text-node` 真实 DOM
   （F/main/ui/shapes/text/fo_text.cljs:158-187；auto-width 用 `width 100000`
   的测量技巧 :178-179；渐变/透明填充先映射为纯色再用 data-mapping 换回
   :63-156）。工作台还有一层全屏 `svg>foreignObject` 文本覆盖层
   （viewport.cljs:366-377，`stvh/viewport-texts`），承担编辑态合并/排版
   回流（F/main/ui/workspace/shapes/text/viewport_texts_html.cljs；R4-fix
   路径修正：原记漏 workspace 路径段）。文本编辑器 v1 是
   SVG+Draft.js、v2 是 DOM 编辑器、v3（wasm 配套）是 contenteditable 捕获面
   （F/main/ui/workspace/shapes/text/v3_editor.cljs:97,173,191）。
3. **flex/grid 布局是自研 CLJC 实现，不委托浏览器布局引擎**。"CSS Grid and
   Flex Layout"（README.md:49 的产品话术）= Penpot 用 ClojureScript 重写了
   flexbox/grid 语义：flex 的行断行/min-max/gap/wrap 算法在
   C/geom/shapes/flex_layout/layout_data.cljc:24-70（`init-layout-lines`
   手写 flexbox 行装配），产出 `layout-child-modifiers` 把子形状定位为绝对
   坐标再交给 SVG 渲染；flex+grid 两套合计 2,564 行 CLJC（实测 wc；
  R4-fix 口径注：2,564 = flex_layout/ 6 件 1,369 + grid_layout/ 5 件
  1,172（两目录 2,541）+ 根 flex_layout.cljc 23；同性质的根
  grid_layout.cljc 23 行未计入，全计 2,587——±0.9% 不影响本节结论）。
   即：**浏览器只做绘制，排版语义由 Penpot 自持**——这与"CSS 渲染"的直觉
   路线（把 div+CSS 交给浏览器排版）是关键区别，也是 inspect 面板能生成
   HTML/SVG markup + CSS 代码（F/util/code_gen.cljs:14-33）的同一根源。

结论句：Penpot 经典路线 = **"SVG 保留模式 DOM + 浏览器文本引擎 + 自研布局
算法"** 的混合体；CSS 的角色在文本排版与光标/混合模式等属性，而非画布结构。

### 1.2 性能特征（经典渲染器）

- **无 per-shape 位图缓存**（与 Excalidraw 的 ShapeCache 相反）：保留模式
  SVG 交给浏览器合成，缩放/平移=改 viewBox 属性。
- **视口裁剪只到根帧粒度**：root-shape 用 memoized 的帧-视口重叠过滤器裁掉
  视口外根帧（shapes.cljs:69-75）；帧内子树全量挂载。
- **LOD = 帧位图缩略图**：非 active 的帧渲染为后端缩略图 `<image>`
  （frame.cljs:125-175），策略写死在代码注释里：zoom<25% 显示缩略图（单选
  帧/子元素除外）、zoom≥25% hover 帧激活、zoom>130% 视口内全部激活、无
  hover 保留上一个 active、active 帧必须在视口内
  （F/main/ui/workspace/viewport/hooks.cljs:431-476）。缩略图本身由前端生成：
  序列化帧为 HTML 字符串 → 隐藏 rasterizer iframe 光栅化为 Blob → 上传
  （F/main/data/workspace/thumbnails.cljs:64-82；F/main/rasterizer.cljs:7-14,
  290-291 origin 校验，parent-origin 定义在 :30；R4-fix 锚修正：origin
  校验原锚 :33-49 实为 get-document-element/create-image，消息入口的
  origin 前缀校验在 :290-291）。这是 DOM 路线的关键吞吐手段：**离屏内容
  退化为位图**。
- **滤镜走 SVG filter**（blur/shadow 每 render-id 一组 defs，
  frame.cljs:68-72,100-102）——DOM 滤镜是公认的性能/一致性软肋，wasm 路线
  文档明确以 GPU surface 重新设计同一语义（RW/docs/rendering_architecture.md:35-42）。
- 【推断】该组合（SVG DOM + 帧缩略图 LOD + 帧粒度裁剪 + SVG 滤镜）的性能
  天花板是 DOM 节点数与浏览器 SVG 光栅化成本——这正是 render-wasm 存在的
  结构性动机；仓内文档未直说"因为 DOM 慢"，此因果为分析员从双渲染器并存 +
  瓦片缓存设计反推。

### 1.3 render-wasm：并行的 Skia/WebGL 第二渲染路线（状态核验）

- **存在且已产品化到用户可选**，但**默认关闭**：feature `render-wasm/v1` 不在
  default-features（C/features.cljc:64-73），属 frontend-only
  （:79-87）；启用优先级 = URL `?wasm=true|false` > 用户 profile 偏好
  （`render-switch` 全局开关下，默认 `:svg`）> 团队特性集
  （F/main/features.cljs:28-66）；部署级开关为 config.js 的
  `enable-feature-render-wasm`（RW/README.md:91-94）。视口组件按特性二选一
  （F/main/ui/workspace/viewport.cljs:778-786）。
- **技术栈**：Rust crate 编译到 `wasm32-unknown-emscripten`，Skia 经
  rust-skia 官方定制二进制（RW/README.md:7-17；Cargo.toml:3-8 "Wasm-based
  canvas renderer for Penpot"）。同源双 target：frontend（-O3 速度）与
  export（-Oz 体积），产物分别落到前端与 exporter（RW/README.md:37-45）。
- **渲染模型**：WebGL2 上下文（F/render_wasm/api/webgl.cljs:87-106）+ 512px
  逻辑瓦片（DPR/zoom 无关）、Tile↔Shape 双索引、TileTextureCache 缓存
  `skia::Image`、曼哈顿距离从视口中心向外优先渲染、interest_rect 预渲染
  （RW/docs/tile_rendering.md:21,39-44,77）。C ABI 导出
  `render(timestamp, flags)` 等（RW/src/main.rs:38,47,68）；交互变换的 modifier
  瓦片失效合并到每 rAF 一次（main.rs:71-76 注释"cost is paid once per rAF
  rather than once per pointer move"）。CLJS 侧形状对象在 wasm 模式下
  直接由 wasm 工厂创建共享（C/types/shape.cljc:40-61
  `wasm-enabled?`→`wasm-create-shape`），changes 应用后增量同步进 wasm
  ShapesPool（F/main/data/changes.cljs:134-145）。
- **文本脱离浏览器引擎**：Skia Paragraph 排版、字体数据显式传入 wasm、
  显式 Noto 回退链（不能像 SVG 路线那样依赖浏览器 fallback，
  RW/docs/texts.md:5,27-29）——这是"跨浏览器像素一致"的代价面。
- **双路径守卫（工程亮点）**：live/GPU 路径（多中间 surface 合成）与
  vector/PDF 路径（单 CPU Skia canvas 直接产出矢量 PDF）必须同图，靠
  `ShapeRenderer` trait 能力守卫 + 穷举 match 类型守卫 + 单一绘制顺序接缝
  `render_leaf_content` 三编译期守卫防漂移（RW/docs/rendering_architecture.md:5-12,92-113）。
- **导出双侧接线**：单件导出走客户端 wasm（含矢量 PDF），批量导出永远走
  服务端 `:export-shapes`（带 `:is-wasm` 提示，服务端用 exporter 内嵌的
  同一 wasm 引擎）（rendering_architecture.md:78-90；
  F/main/data/exports/assets.cljs:267-303）。exporter 另保留 Playwright
  无头浏览器路径（E/browser.cljs:11 playwright、:50-80 字体就绪等待），
  `is-wasm` 参数二选一（E/renderer.cljs:41-60）。
- 【推断】方向判断：render-wasm 是长期替代经典 SVG 渲染器的主线（双
  target、编译期 parity 守卫、视口/编辑器/rulers 全套 UI 已接），但当前
  仍以 opt-in 并行运行收敛正确性——"CSS/DOM 路线"是 Penpot 的现在时，
  wasm 是其自证的路线修正。

### 1.4 渲染光谱对照（与 canvas-a §2 / canvas-b §2 并列）

| 面 | Excalidraw（canvas-a） | tldraw（canvas-b） | **Penpot 经典** | **Penpot wasm** |
|---|---|---|---|---|
| 画布载体 | 双 canvas 2D（静态+交互） | DOM/SVG shape 层 + canvas2D 仅 overlay | **全屏 `<svg>` viewBox + foreignObject HTML 文本** | **WebGL2 canvas + Skia** |
| 相机实现 | canvas transform + 全量重绘 | html-layer 单 CSS transform，零 React 重渲染 | **改 svg viewBox 属性**（形状 DOM 不动） | wasm 内部矩阵 + 瓦片重算 |
| 文本排版 | 自制排版（离屏 canvas 逐字符） | 真 DOM 文本 + 隐藏测量池 | **浏览器排版（foreignObject 真 HTML）** | **Skia Paragraph（自带引擎+显式回退）** |
| 布局语义 | 无（手放） | 无（手放+reparent） | **自研 flex/grid（CLJC，绝对坐标输出）** | 同一 CLJC 算法，渲染端 Skia |
| 局部重绘 | 无（全画布重绘+元素位图缓存） | 无需（保留模式） | 无需（保留模式）+**帧级位图 LOD** | **脏瓦片 + 纹理缓存** |
| 裁剪 | 视口 AABB 线性扫描 | R-tree + display 开关（不卸载） | **根帧-视口重叠过滤（帧内不裁）** | 瓦片粒度（结构性） |
| 滤镜/模糊 | canvas 绘制时实现 | CSS/SVG | **SVG filter（每 render-id defs）** | GPU surface 分层合成 |
| z 序方案 | 数组序 | DOM 按 id 稳定排序 + z-index | **SVG 文档序** | 绘制调用序 |
| 导出渲染 | 同 canvas 代码路径 | foreignObject + 字体子集 | **Playwright 无头浏览器 / wasm 双路径** | wasm vector（真矢量 PDF） |

光谱定位：Excalidraw（即时模式位图，全部自绘）↔ tldraw（保留模式 DOM，排版
半委托）↔ **Penpot 经典（保留模式 DOM，排版全自持 + 文本委托浏览器）** ↔
**Penpot wasm（即时模式 GPU 瓦片，引擎全自持）**——Penpot 一家横跨了光谱的
中间两格并正在向右端迁移，是三件样本中唯一能同时观察"DOM 路线上限在哪"与
"迁离 DOM 的工程代价"的件。

### 1.5 MPL-2.0 文件级隔离含义（净室合规）

- 许可核验在档：根 LICENSE:1 = "Mozilla Public License Version 2.0"；根
  package.json:4 = `"license": "MPL-2.0"`（version 2.17.0 @ package.json:3；
  R4-fix 锚修正：license/version 原记 :5/:4 各偏一行）；
  render-wasm/Cargo.toml:6 `license-file = "../LICENSE"`；plugins/LICENSE:1 =
  MPL-2.0。逐文件头抽查：Clojure 源 1,085 文件中 1,053 带 MPL 头（32 个无头，
  如 C/thumbnails.cljc 直接以 ns 开头——属疏漏非例外声明）；Rust 104 文件
  （排除 build.rs；R4-fix 口径统一：原记 103，与 §0/§4 的 104/46,516 对齐）
  **全部无文件头**（仅 crate 级声明）；TS 包（mcp/plugins/media-processor）
  无逐文件头、仅 package.json 声明。
- **包级许可不一致（登记为事实）**：plugins/package.json:5 = `"MIT"` 而同目录
  LICENSE 文件为 MPL-2.0；mcp/package.json:5 = MPL-2.0 而子包
  mcp/packages/server/package.json:26 = `"MIT"`。【推断】推测是 npm 发布生态
  友好考虑（对插件作者/ MCP 客户端集成零传染）或有意的双声明，但仓内无
  说明文件裁决，取用时以 LICENSE 文件（MPL-2.0）为保守口径、以 package.json
  （MIT）为发布口径的两可状态登记。
- **文件级隔离（MPL §1/§3 语义）对工作台的含义**：MPL 是弱著佐权——①净室
  借鉴架构/算法思想（本报告方式）零义务；②若逐文件复制（如 flex 排版
  C/geom/shapes/flex_layout/），该文件及其修改永久保持 MPL-2.0 且需提供
  源码，但可与私有文件同仓组合（文件边界隔离，非 GPL 的整仓传染）；③依赖
  其 npm 包按各自声明另行评估（@penpot/library = MPL-2.0，
  library/package.json:4；plugins/mcp 子包的 MIT 声明见上条）。相比 tldraw
  （source-available + 技术执行，canvas-b §11）与
  Excalidraw（MIT），MPL-2.0 是三件中对"抄结构不抄代码"最友好的中间档。

## 2. 插件面：SES Compartment + iframe UI + 权限模型（核验：已产品化）

核验结论：插件系统**存在且默认启用**（`plugins/runtime` 在 default-features，
C/features.cljc:70），运行时为 `plugins/` 仓 TS 包（plugins/README.md:7-11）。

**执行模型（双层隔离）**：

1. **插件逻辑跑在宿主同 realm 的 SES Compartment**（非 iframe、非 Worker）：
   `ses.hardenIntrinsics()` 冻结内建对象 → `createCompartment(publicPluginApi)`
   白名单全局（Object/Math/typed arrays/JSON/Promise/Proxy/Intl/console/
   atob…，P/create-sandbox.ts:110-198）→ `penpot` API 经 Proxy 逐调用
   `ses.safeReturn` 深冻结（:60-84）。依赖 Agoric `ses` ^2.3.0
   （P/package.json:6，P/ses.ts:1-23 薄封装）。定时器登记可清理
   （:113-146）；`safeFetch` 强制 `credentials:'omit'` + 清空 Authorization
   头（:86-108）——插件拿不到 Penpot 会话凭证。scope="global" 的插件额外
   获得 `penpotMgmt`（:204-208）。
2. **插件 UI 跑在沙箱 iframe**：custom element `plugin-modal` 内
   `iframe.sandbox = allow-scripts, allow-forms, allow-modals, allow-popups,
   allow-popups-to-escape-sandbox, allow-storage-access-by-user-activation,
   allow-same-origin (+allow-downloads 按权限)`，clipboard 按权限进
   `allow`（P/modal/plugin-modal.ts:114-134）；宿主与 iframe 用 postMessage
   （`'*'` 目标）桥接（:162-175；P/load-plugin.ts:38-48 按 event.source
   路由到对应插件）。
3. **origin 防逃逸**：因 iframe 带 allow-same-origin，插件 UI URL 不得指向
   Penpot 自身 origin，否则抛错（P/validate-url.ts:33-44 注释明言"防止
   插件 iframe 逃出沙箱隔离"）。

**权限模型**：manifest 声明权限字符串——content/library/user/comment 各自
read/write + allow:downloads/localstorage/global + clipboard:read/write
（F/main/ui/workspace/plugins.cljs:270-352 渲染清单）；安装与权限变更均需
用户确认对话框（:354-479）。打开已装插件时**重新拉取 manifest 并逐字段比对**：
权限变化或 manifest 变化（防篡改/注入）都要再确认，拉取失败拒绝执行
（F/main/data/plugins.cljs:116-162，注释 :143-144 "Prevents execution of
tampered/injected plugins"）。`plugins-whitelist`（部署配置，默认空，
F/config.cljs:166）内的 host 免"第三方风险"免责声明（plugins.cljs:399-401）。
插件写路径与 UI 同源：`app.plugins.api`（744 行 CLJS）全部经
`commit-changes` 走统一 changes 管线（F/plugins/api.cljs:7-63）。

**安全面在档事实**（CHANGES.md 修复史即边界演化史）：曾修"postMessage 广播
到所有插件且无 origin 校验"（CHANGES.md:21 #10968）与"插件 API 在
tokens/shapes/variants/flows/layouts/用户身份上缺权限检查"（CHANGES.md:28
#11137）——说明权限模型是**事后逐步补硬**的，采纳其清单时按现状而非历史最优
理解。

**MCP 面（对 dsh/agent 语义直接相关）**：官方 MCP server 在仓内 mcp/（TS，
stdio + HTTP stream，mcp/package.json bin `penpot-mcp`）。架构：MCP server
←WebSocket→ 浏览器内的 **MCP 插件**（mcp/README.md:18-29）——即 MCP 的执行
环境就是上述 SES 插件运行时，LLM 写任意 JS 经 `execute_code` 工具注入执行，
可用全局 `penpot`/`penpotMgmt`/`penpotUtils`/`storage`（跨调用存续的对象
库）（M/server/src/tools/ExecuteCodeTool.ts:36-48 及工具描述）；另有
export_shape/import_image/import_penpot_file/high_level_overview/
penpot_api_info/read_taiga_issue 及三个 Clojure REPL 向工具（M/server/src/tools/
目录 10 文件）。连接状态含心跳与"浏览器冻结"检测（M/server/src/PenpotConnection.ts:1-22），
多用户会话经 RedisBridge（M/server/src/RedisBridge.ts）——**实时性依赖浏览器
tab 活着**，浏览器后台冻结曾致 MCP 挂起（CHANGES.md:157 #10323 修复）。前端
侧 MCP 插件为**内置 global-scope 插件**（硬编码 plugin-id，
F/plugins/register.cljs:23；default-manifest 全权限集
F/main/data/workspace/mcp.cljs:32-46），:mcp-enabled 是 profile 级字段
（B/rpc/commands/profile.clj:65），远程托管 MCP 端点默认 `/mcp/ws`
（F/config.cljs:185-187）。另有 @penpot/library（library/README.md:1-8）
支持纯 Node 无头构建 .penpot 文件——code-to-design 的非浏览器路径。

## 3. 数据模型与协同

### 3.1 形状/组件/实例

- **形状 = 扁平 record + 类型字段**（非判别联合）：`Shape [id name type x y
  width height rotation selrect points transform transform-inverse parent-id
  frame-id flip-x flip-y]`（C/types/shape.cljc:48-49；R4-fix 锚收紧：原记
  :44-53 区间偏松，defrecord 字段面实在 :48-49），9 类型
  （:70-79）；页面=UUID→shape 的 objects map，树由 parent-id/shape 列表表达。
  id 为**自研 UUIDv8**：48bit 毫秒时间戳（自定义纪元 2022-01-01）+ 单调
  clockseq + 16384 ids/ms 自旋锁（C/UUIDv8.java:10-17）——时间可排序身份。
- **组件/实例**：component = {id name path objects main-instance-id
  main-instance-page} + variant 扩展（C/types/component.cljc:21-31）；实例是
  树中挂 :component-id/:component-file/:component-root/:main-instance/
  :remote-synced/:shape-ref/:touched 的普通形状（C/types/shape.cljc:192-202）。
  **覆盖同步语义按属性组组织**：sync-attrs 把 ~20 个属性归入
  name/fill/geometry/visibility/text-font 等组，改主组件同组属性不会冲掉
  实例已 touched 的覆盖（component.cljc:39-60 注释即规范）——Figma 式
  override 模型的最小实现面。
- **持久化编码**：file.data 为 blob（zstd 压缩，B/deps.edn:9 zstd-jni）；
  `fdata/objects-map` 特性下 UUID→shape 用 ObjectsMap——Fressian 字节阵列
  **逐字段独立解码**的结构（C/types/objects_map.cljc:7-14），配合 pointer-map
  去重，支撑"只重编码变更形状"的增量持久化。数据格式演化走**特性集协商**：
  file/team/global 三级 feature 集合 + 打开/粘贴/导入时双向兼容检查
  （C/features.cljc:186-336），components/v2 已强制（:241-245）。

### 3.2 changes 协议（单一写路径）

一切修改 = changes 向量（op 集合：add-obj/mod-obj/del-obj/reg-objects/
mov-objects + 库类 color/typography/component/media + token 类，B/rpc/commands/
files_update.clj:89-119）。**前后端共享同一处理器** `cpc/process-changes`
（common CLJC；后端 files_update.clj:409 调用，前端 changes.cljs:128-131
调用）。前端 commit 同时携带**预计算的逆操作** undo-changes（changes.cljs:159-192，
undo=回放逆向量），source 区分 local/remote，remote 提交不入撤销栈
（F/main/data/workspace/notifications.cljs:270-276 `save-undo? false`；
  R4-fix 路径修正：原记 workspace/notifications.cljs 缺 data 路径段，
  ui/ 下同名文件非此实现）。这与 ONLYOFFICE
"change=可序列化 undo 项"（office-b）同族、与 tldraw "mark+squash diff 双栈"
异族。

### 3.3 协同：服务器定序 + xlog + 快照 + msgbus 推送（与 collab-a 对照）

**零 CRDT/零 OT**（核验：common/ 与 backend/ 全文 rg "crdt|operational
transform" 零命中；自研印象成立）。机制四件：

1. **写路径（update-file RPC）**：校验编辑权限 → **悲观行锁**
   `db/xact-lock! conn id` 串行化同文件并发（files_update.clj:148-149）→
   revn+1 应用 changes（:408-410）→ **file-change xlog 落行**（session/
   profile/revn/features/changes blob，:262-274）→ 持久化 file → msgbus
   广播 `{:type :file-change :revn :changes}` 到文件主题（:462-487）。
   冲突面：incoming revn > stored 抛 revn-conflict（:183-189）；vern 不匹配
   抛 vern-conflict（版本恢复竞态，:175-181）；**落后方在响应里带回 lagged
   changes**（xlog 中 revn > 客户端 revn 的全部行，:447-460）供本地追平——
   即"服务器定序 + 客户端追平重放"，与 tldraw tlsync 的 push/pull/rebase
   同光谱（服务器权威、无 transform），粒度是整个 change 批。
2. **读路径（ws 通知）**：前端开 `/ws/notifications` 长连接
   （F/main/data/websocket.cljs:23-33）；subscribe-file 经权限检查后订阅
   msgbus 文件主题，通道为 **dropping-buffer 64**（慢消费者丢消息后靠 revn
   追平机制自愈）、自己 session 的消息被过滤（B/http/websocket.clj:181-221，
   :95-112）；presence = join/leave/disconnect 事件转 `:presence` 消息
   （:198-211），前端渲染远端光标并按 5s 新鲜度过滤
   （F/main/ui/workspace/viewport/presence.cljs:59-74）。**无字段级锁/
   无编辑锁协议**——同形状并发编辑按提交顺序最后写入胜。
3. **快照**：`auto-file-snapshot` 下每 20 个 revn 或 1h 超时落全量快照
   （files_update.clj:434-445）——xlog+周期快照即经典 event-sourcing 组合，
   版本恢复（vern）另走文件级 version 链。
4. **前端持久化队列**：本地 commit 入队，逐个发 update-file，瞬时错误按
   退避重试（toast 提示），停滞检测 5min（F/main/data/persistence.cljs:38-40,
   280-314, 372-402）——离线容忍度 = 队列在内存，刷新即失（有专门修复史
   "read-only 阻塞持久化时丢未存变更"，CHANGES.md:280）。

与 collab-a（Yjs）对照登记：Penpot = 服务器全序 + 显式逆操作 + 追平重放，
**协作语义强度低于 CRDT**（无离线合并、无意图保持、整批 LWW），但**数据模型
与协议同构度高**（change 操作可序列化、前后端同处理器、xlog 可回放），接近
office-a（Univer IChangeset）与 tldraw 的中间形态；多分支/离线编辑是明确
非目标。

## 4. 全架构速览（技术栈事实，供工作台语言选型参照）

- **语言分布（实测 wc）**：ClojureScript frontend 697 文件/180,032 行 +
  common CLJC 175/58,731；Clojure backend 179/50,255；Rust render-wasm
  104/46,516；TS：plugins ~31,650 + mcp 8,629 + media-processor 1,336
  （R4-fix 计数修正：workflow 原记 26 实为 27、plugins TS 原记 ~31,416
  实测 ~31,650，均 -0.7% 内）。
  即**主体是 Clojure 系（~77%，289,018/376,915 行 = CLJS+CLJC+CL 三项
  合计；R4-fix 口径修正：原记 ~66% 系误用 frontend 单项当 Clojure 系）+
  渲染引擎 Rust + 扩展面 TS** 的三语言体。
- **前端栈**：shadow-cljs 编 ESM 多模块（main/main-viewer/main-workspace/
  main-dashboard/main-settings/render/rasterizer，frontend/shadow-cljs.edn:14-52）；
  React 经 rumext.v2 + potok（re-frame 式事件）+ okulary（lens）+ beicon
  （RxJS 风格流）；UI 组件库自研 @penpot/ui + react-aria-components
  （frontend/package.json dependencies 仅 2 项）；SCSS + storybook。
- **后端栈**：JVM Clojure（无 Rust——"自研 + Rust 后端"印象核验：**Rust
  仅渲染层，后端是 Clojure**）；PostgreSQL（next.jdbc + HikariCP，**169 个
  SQL 迁移文件（0001–0156 编号，另有 XXX 模板）**）+ Redis/lettuce
  （msgbus.clj:8）+ Jetty/yetti WebSocket +
  reitit + Prometheus；RPC 163 处 sv/defmethod 服务注册（29 命令文件）。
- **部署拓扑**（docker/devenv）：开发=单容器 tmux devenv + infra compose
  （postgres 18.6 / valkey 8.1 / rustfs 1.0.0 S3 兼容 / mailpit / openldap）
  （docker-compose.infra.yml:18,47,71,82,108）；运行时服务 = backend +
  frontend 静态 + exporter（Node+Playwright 池+wasm 池，E/wasm/pool.cljs）+
  media-processor + MCP server，企业能力经仓外 nitrate 服务（B/nitrate.clj:8）。
- **质量体系**：common/backend/frontend 测试文件 102/95/161；27 个 CI
  workflow（含 tests-e2e playwright、tests-mcp、tests-plugins、tests-wasm、
  tests-exporter）；render-wasm 另有像素回归测试基建
  （RW/docs/visual_regression_tests.md、preview-snapshots/）；clj-kondo +
  cljfmt + stylelint + prettier。2.19.0 Unreleased 在途（CHANGES.md:3），
  2.17.0→2.18.3 多版高频在档。
- 【推断】**对工作台语言选型的诚实参照**：有限——工作台是 Rust（mdcore 系），Penpot
  的 Clojure 系选型不可移植；可参照的是**分层方式**（共享数据层 CLJC 一份
  代码两端跑 ≈ mdcore 的 lib 双执行环境思想）、**Rust 渲染引擎的集成形态**
  （C ABI + 序列化层 + 宿主语言绑定，RW 与 wasm.cljs/serializers 的边界），
  以及"多语言按变更频率分层"（核心引擎 Rust/CLJC，扩展面 TS）的分工本身。

## 5. 对工作台的可移植结论（按优先级）

1. **P0|渲染路线的"第三样本"结论**：画布线现有两极样本（Excalidraw 即时
   位图 / tldraw 保留 DOM）。Penpot 证明**中间路线 = 保留模式 DOM + 布局
   语义自持 + 离屏内容位图化**可以撑到专业设计工具规模，其两个具体手段
   直接可借鉴：①**帧级位图 LOD 的显式策略**（zoom 阈值 + hover 激活 +
   视口内保持，hooks.cljs:431-476 把 LOD 规则写成可读注释——比"R-tree
   裁剪"更省的对象淘汰面）；②**光栅化服务隔离**（HTML 字符串 → 独立
   rasterizer iframe → Blob，origin 校验）。工作台画布若走 DOM 混合层
   （R2 规划方向），这两招即是吞吐保底的已知答案。
2. **P0|双渲染路径的 parity 守卫设计**（RW/docs/rendering_architecture.md:92-113）：
   能力守卫（trait 方法即特性清单，加特性不加实现=编译失败）+ 类型守卫
   （穷举 match 禁 `_`）+ 顺序接缝单点——**任何"同一数据两种渲染后端"的
   工作台架构（如 DOM 预览 + WebGPU 高性能模式）都应内建这三守卫**，这是
   Penpot 用 Rust 类型系统把"两路漂移"变成编译期错误的现成范式。
3. **P0|changes 单一写路径 + 预计算逆操作**：插件（SES）/MCP（execute_code）/
   UI/导入全部经 `commit-changes` 一个入口（F/plugins/api.cljs:65-82），
   redo/undo 双向可序列化。对 dsh 的 agent 写入面：**给 agent 的写 API 应该
   是宿主同一 changes 协议的薄包装**（【推断】Penpot MCP 的 execute_code 本质是
   "在宿主沙箱里跑用户代码操作宿主 API"），而非旁路 REST——审计、撤销、
   协同广播免费获得。
4. **P1|SES Compartment 插件沙箱**：对 dsh 插件面，Penpot 给出比 iframe/
   Worker 更细的第三选项（同 realm、冻结内建、Proxy safeReturn、凭证擦除
   fetch、白名单全局、UI 独立 iframe + origin 防逃逸 + manifest 防篡改重验）。
   采纳时注意其修复史（postMessage 无 origin 校验、API 权限缺失两条已修
   漏洞）说明**清单式权限要在 API 层逐方法执行而非仅在安装时声明**。
5. **P1|MCP 经插件运行时的架构**：MCP server 不直连文档数据库，而是经
   浏览器内常驻插件 WS 桥接（mcp/README.md:18-29）——好处是 LLM 操作天然
   走用户会话权限 + 实时 UI 反馈 + 复用插件 API 全集；代价是**实时性绑死
   浏览器 tab 存活**（冻结检测、心跳、重连策略都是为此补的）。工作台若
   桌面端（无 tab 冻结问题）此模式更稳；若 Web 端需评估后台节流场景【推断】。
6. **P2|数据格式演化的特性集协商**：file/team/global 三级 feature 集 +
   打开/粘贴/导入逐场景检查（C/features.cljc）——比 mdcore 的 revision/
   migration 更细粒度的**运行时能力协商**模式，适合工作台插件生态多版本
   共存场景。
7. **P2|自研 flex/grid 排版的位置输出协议**：布局算法产出"子形状绝对坐标
   modifiers"再交渲染（flex_layout.cljc:16-21 导出面；R4-fix 锚收紧：
   原记 :17-18，dm/export 面实在 :16-23，:16-21 覆盖主要导出）——
   布局与渲染解耦
   的干净接口，任何工作台画布若要支持 auto-layout，这是比"把子元素交给
   CSS 容器"更可控的模型（导出/打印/协作三端一致）。
8. **P3|可复用资产清单（许可口径见 §1.5）**：@penpot/library（无头 .penpot
   构建，MPL-2.0 声明）、@penpot/mcp（MCP 客户端模式参照）、UUIDv8 自研实现
   （MPL 文件级）。除思想借鉴外不建议代码级搬运。

## 6. 与工作台架构的对照（一页，与 canvas-a/b §14 并列）

| 面 | Penpot | tldraw（canvas-b） | Excalidraw（canvas-a） | 工作台目标（mdcore 系） |
|---|---|---|---|---|
| 渲染 | SVG DOM + foreignObject 文本 + 帧位图 LOD；wasm/Skia/瓦片并行 | DOM/SVG + 命令式 CSS + canvas2D overlay | 双 canvas 全量重绘 + 元素位图缓存 | 画布线待建：文本保真→DOM 混合层；吞吐→WebGPU |
| 布局 | 自研 flex/grid CLJC → 绝对坐标 | 无 | 无 | op 树上可选 auto-layout；位置输出协议可借鉴 |
| 变异入口 | changes 单一管线（UI/插件/MCP 同门） | store.put/applyDiff | 双通道+store.commit | apply_ops 单一入口——四者中 Penpot 证明该入口可承载插件+LLM |
| 撤销 | 预计算逆 changes 双向量 | mark+squash diff | 快照 diff+三态捕获 | 命令式逐 op 求逆（Penpot 同族最近） |
| 协同 | 服务器定序+xlog+lagged 追平+msgbus（dropping-buffer） | tlsync rebase | 盲转+LWW+nonce 平局 | 单机权威+重放；若画布协同，Penpot/tlsync 同族可选 |
| 插件 | SES Compartment+iframe UI+清单权限+manifest 防篡改 | 无第三方插件面（SDK 集成） | 无 | dsh 插件面：SES 模式是新样本 |
| AI/agent | 官方 MCP（execute_code 经插件运行时）+@penpot/library 无头路径 | MCP exec+focused proxy | 无 | dsh+MCP 双口径：Penpot 是"agent 经宿主 API"路线的成年样本 |
| 许可 | MPL-2.0（包级 MIT 不一致登记） | source-available+技术执行 | MIT | 自研零负担；借鉴设计不复制代码 |

## 附：证据锚点密度与推断清单

- 本报告共引用证据锚点约 **150 处**（`路径:行号` 形态实测 121 处 + 目录/
  文件级引用 ~30 处），覆盖 6 个章节 + 对照表；密度与 canvas-a（~90）/
  canvas-b（~130）同量级以上。所有未标注【推断】的结论均有直接代码/文档
  证据。
- 【推断】标注共 **6 处**：§1.2 DOM 性能天花板为 wasm 动机的反推；§1.3
  wasm 为长期主线替代的方向判断；§1.5 包级 MIT/MPL 不一致的双声明解释；
  §4 语言选型参照有限的定性；§5-3 Penpot MCP 本质描述的概括句；§5-5
  桌面端该模式更稳的条件判断。
- 核验在档：MPL-2.0（根 LICENSE + package.json + Cargo.toml license-file +
  1,053/1,085 Clojure 文件头；Rust 零文件头仅 crate 级；plugins/mcp 包级
  MIT 不一致登记）；零 CRDT/OT（rg 全仓核验）；"Rust 后端"印象证伪
  （Rust 仅渲染层，后端 JVM Clojure）；插件系统已产品化且默认启用。
- 深读工作副本：/home/z/my-project/analyses-workdir/penpot（浅克隆
  @10955f17，未做任何修改、未 git commit，磁盘紧张未跑任何构建）。
