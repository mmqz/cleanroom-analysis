# office-a：Univer 深读报告（11 维全量）

> **净室分析报告**（线 A 办公 P0 第一篇）。日期：2026-10-10。
> 对象：dream-num/univer @ `7166c69`（master 浅克隆，v1.0.4，Apache-2.0）。
> 方法：四路并行勘察代理（架构+数据模型+编辑内核 / 渲染+公式+性能 /
> 协同+持久化+格式 / 插件·AI·安全·质量·合规）+ 主代理交叉综合；结论标注
> `路径:行号` 证据锚点（相对 `packages/` 或仓根），【推断】= 分析员判断。
> 生态仓（univer-workspace / dsh-univer-office / univer-cli / univer-mcp）
> 不在本地，相关内容仅引 README 一手声明并逐处标注。
>
> 一句话定性：**「快照即模型」+ 三层命令系统的 Canvas 办公引擎——OSS
> 交付内核与 Facade（协同/导入导出/SSR 全在 Pro 闭源），"The Office
> Harness for AI Agents" 的编排层在生态仓；视口级双缓冲缓存 + RTree
> 依赖图增量重算是其工程精髓，而"无迁移器、约定性旁路、无 e2e"是其
> 债务面**。

## 0. 架构速览

```
pnpm monorepo（60 包 + 17 preset + examples）
core ＝ 内核：DI(redi)/命令/撤销/权限/插件/资源加载 + sheets 模型
  ├─ 快照即模型：Workbook 直持 IWorkbookData，ObjectMatrix 包引用原地改
  ├─ 三层命令：COMMAND(编排)/MUTATION(入快照,协同单元)/OPERATION(视图态)
  └─ 内置 OT：docs 域 TextX + JSONX(ot-json1 封装)；sheets 域无客户端 transform
engine-render ＝ Canvas 2D 引擎（Scene→Layer→Object + Viewport 缓存）
engine-formula ＝ 自研公式引擎（词法→AST→解释器 + RTree 依赖图）
sheets/docs/slides(+*-ui) ＝ 三大领域，逻辑/UI 分包；sheets 62 命令/46 mutation
protocol/network/rpc ＝ 协同契约(Pro 服务端)/HTTP+WS 原语/进程内 RPC
presets ＝ 预组装插件集（node preset = headless 一等公民）
Pro(闭源) ＝ 协同服务器/导入导出/SSR/编辑历史/图表 等全部在边界外
```

## 1. 数据模型与持久化完整性

**快照即模型（无独立内存模型）**：Workbook 直接持有
`_snapshot: IWorkbookData`（`core/src/sheets/workbook.ts:71`），`save()`
即 `deepClone(_snapshot)`（:141-143）；Worksheet 构造时
`new ObjectMatrix(cellData)` 包住 snapshot 内对象引用原地修改
（`core/src/sheets/worksheet.ts:114-120`）。即 mutation 直接改快照、
undo/协同全靠外挂机制（命令逆操作 + collab 监听），模型与持久化格式
零分离。

**稀疏双层对象**：`cellData = {行号:{列号:ICellData}}` 双层数字键对象
（`core/src/sheets/typedef.ts:148-151`）；空单元格直接删键
（`set-range-values.mutation.ts:184,190` 的 `realDeleteValue`）。行高列宽
由 RowManager/ColumnManager 管理（`worksheet.ts:41-42`）。

**ICellData 短键设计**（`typedef.ts:283-336`）：`p`(富文本完整内嵌
IDocumentData)/`s`(style id 或内联)/`v`(原始值)/`t`(类型)/`f`(公式串)/
`si`(公式 id)/`ft/fd`(动态数组)/`custom`。富文本为区间 run
`ITextRun{st,ed,ts}`（`types/interfaces/i-document-data.ts:452-462`）。

**样式表 arena + LRU intern**：workbook 级 `styles: Record<id, IStyleData>`
（`typedef.ts:64`），`Styles` 类用 LRU(10万) 按序列化串去重、随机 6 位 id
（`core/src/sheets/styles.ts:27-77`）——样式不复制进单元格，间接引用。

**版本兼容 = 默认值合并，无迁移器**：构造时
`commonExtend(DEFAULT_WORKBOOK, data)`（`workbook.ts:93-97`）+
`mergeWorksheetSnapshotWithDefault`（`sheet-snapshot-utils.ts:47-80`）；
`appVersion` 仅写入不读取（`empty-snapshot.ts:31`）。【推断】依赖"字段全
optional + 新版本补默认值"，长期演进有 schema 漂移风险——对 mdcore 是
反面教材（应配显式 version + 顺序迁移器）。

**插件状态 sidecar**：`resources: IResource[]{name, data}`（`typedef.ts:
80-82`），`ResourceLoaderService` 在 unit 加载/销毁时 load/unload
（`resource-loader.service.ts:102-135`）——核心快照不被插件字段膨胀。

## 2. 渲染管线

**纯 Canvas 2D，每 unit 一个 Engine+Scene**：`getContext('2d')`
（`engine-render/src/canvas.ts:88`）；`RenderManagerService.createRender`
为每个 unitId 建独立 Engine 与 Scene（`render-manager.service.ts:203,
253-256`）。Scene→Layer（zIndex 排序、可整层离屏缓存 `layer.ts:107-130`）
→BaseObject（`_dirty` 标志）。**表格视图模型 = SpreadsheetSkeleton**
（由 Worksheet 派生，`components/sheets/sheet.render-skeleton.ts:278,
313-330`），渲染组件经 `updateSkeleton` 换骨架
（`sheet-component.ts:37-40`）。

**网格线/边框/文字不是对象，是 Extension 绘制通道**：Spreadsheet 内置
Background/Border/Font 等 extension 按序 draw（`spreadsheet.ts:270-296,
364-416`）——**单元格是即时绘制的几何而非对象**，与 Excalidraw
per-element 对象模型根本不同（两条路线的根因）。

**绘制管线 = rAF 循环 + 分层脏标志 + 滚动条带脏矩形**：
`Scene.render()` 开头 `if (!isDirty()) return`（`scene.ts:1013-1016`）；
对象 `makeDirtyWithDebounce` 宏任务合并冒泡（`layer.ts:288-298`）；滚动时
计算新旧视口差集条带，`drawImage` 位图搬移 + clip 只重画暴露条带
（`layer.ts:47-94`；`scene.ts:912-948,1038-1056`）。拖动滚动条超阈值时
降级为只画滚动条预览（`scene.ts:950-1033`）。

**视口级双缓冲缓存画布**：每 Viewport 一块 cacheCanvas（视口+2×
bufferEdge×scale，`viewport.ts:1191-1193`）；滚动时旧缓存 copy 到 buffer
画布再交换、仅对 diffBounds 增量重绘（`spreadsheet.ts:517-573,626-645`；
diff 计算 `viewport.ts:1455-1542`），含合并单元格修补
`mergeRepairBounds`（:575-624）。

**虚拟滚动 = 前缀和 + 二分**：骨架维护行高/列宽前缀和数组，
`_getRangeByViewBounding` 二分映射 bound→行列区间
（`sheet.render-skeleton.ts:1256-1290`）；缓存区=视口外扩 100px、距缓存
边 <2px 触发整缓存重刷（`viewport.ts:1411-1452`）；文字溢出再外扩 20 列
（`constants.ts:24`）。

**文本排版**：浏览器 `measureText`（`context.ts:833-837`）+ 硬编码字体
回退链（`basics/const.ts:32`）；单元格富文本复用 docs 自研排版（断行
Unicode trie、连字符、kerning），Font extension 直接驱动
documentSkeleton 绘制（`extensions/font.ts:389-421,687-701`）；docs 有
worker 化布局会话，`step(generation, budgetMs=8)` 时间片推进
（`worker-layout.ts:118-120`）。

## 3. 编辑内核

**三层命令分类（架构精髓）**：`CommandType` 三枚举
（`command.service.ts:37-55`）——COMMAND=编排者（生成 mutation 不直接改
数据）；MUTATION=入快照的最小变更、**协同冲突解决单元**；OPERATION=
非快照数据变更（滚动/选区/UI 态），不参与冲突解决。注册表
`Map<id, ICommand>`（:273-310），id 命名 `<ns>.<type>.<name>`；
`IMultiCommand` 按 priority+preconditions 分发（:92-105,666-718）。

**执行管线**：before 监听（权限）→ handler（`injector.invoke`）→
executed 监听 + mutation 专属 `onMutationExecutedForCollab`（执行点
:403，注册口 :270）；
`IExecutionOptions{onlyLocal,fromCollab,syncOnly,fromChangeset}`（:186-199）为协同预留；
执行栈自动给 mutation 打 `trigger`（来源 COMMAND id，:576-605）——
**意图→数据变化的因果链**内建。

**单一变异入口 = 设计意图 + 约定性**：注释明言 "all data modifications
need to be executed through commands"（:58-63），facade 全走命令、公式
回写也走 mutation（`set-formula-calculation.mutation.ts`）；但 Worksheet
公开 setCellContent 等直写 API，无运行时强制拦截
（【推断】纪律靠约定，Rust 字段私有 + 单入口可天然消除）。

**撤销 = 命令逆操作（非快照非 OT 反演）**：`LocalUndoRedoService` 按
unit 双栈（`undoredo.service.ts:184-186`），上限 50、push 清 redo
（:112,227），支持 group 合并与 rollback（:229-241,343-353）。undo 信息
由 COMMAND 执行前生成：SetRangeValuesCommand 先经
`SetRangeValuesUndoMutationFactory` 读旧值构造 undo params 再执行 redo
mutation（`set-range-values.command.ts:105-160`；factory
`set-range-values.mutation.ts:108-118`）。拦截器可追加联动 undo
（:117-133）。

**选区**：sheets 每 sheet 存 `ISelectionWithStyle[]` 支持多选区 + primary
活动格（`selection-data-model.ts:29-46`；`basics/selection.ts:152-160`）；
选区变更走 `SetSelectionsOperation`=OPERATION 不入快照
（`selection.operation.ts:42`）。docs 光标=折叠 textRange
（`doc-selection-manager.service.ts:50-58`）。

**action-recorder**：独立 UI 插件录制用户意图命令（COMMAND 级）供回放，
显式禁止录 mutation（`action-recorder.service.ts:68-69`）。

## 4. 格式进出保真

**xlsx/csv 导入导出 = Pro（闭源）**：README 能力矩阵将 import/export 列
于 Pro 扩展（`README.md:292,308`）；OSS 包内无 xlsx/io 实现代码。【推断】
这是与"保真"目标最相关的战略缺口：开源世界尚无 Apache 级办公引擎交付
完整 OOXML 读写——mdcore 线的 PDF/OOXML 进出若做成 OSS 即为差异化。

**OSS 交换格式 = 自有 JSON**：快照（IWorkbookData/IDocumentData）+
resources sidecar + changeset JSON；无版本化 JSON schema（§1 appVersion
结论）。服务器建 unit 支持 `snapshot/templateID/initialSheets(行式)/
docContent/workbookDataJson` 多形态（`protocol/src/ts/universer/v1/
snapshot.ts:156-186`）。

**大表分块存储契约**：cell 数据从 meta 剥离、按行分块
`ISheetBlock{startRow,endRow,data:Uint8Array}` + Save/GetSheetBlock
（`ts/univer/workbook.ts:53-70,108-114`；`universer/v1/snapshot.ts:
289-312`）。

## 5. 协同协议

**三层分离**：`@univerjs/protocol` 纯契约包（`protocol/README.md:7`）——
`ts/univer`（核心类型）/`ts/universer/v1`（服务端契约：authz/comb/
comment/history/license/snapshot/user/file）/`ts/univerpro/v1`（Pro 契约
apply/ssc/ssr/helper）/`ts/univercloud`（遥测）。`network` 只给 HTTP/WS
传输原语（OSS 内无业务消费者）；`rpc` 是**进程内**跨线程 RPC（主线程→
worker，`data-sync-primary.controller.ts:41-45,165-181`），不是网络协同。
【推断】协同客户端/服务端在 Pro/独立仓，本仓是协同的"内核与契约面"。

**Changeset 协议**：`IChangeset{unitID,type,baseRev,revision,userID,
memberID,mutations:IMutation{id,data:string}[],sid,reqId,mutationSize}`
（`changeset.ts:20-59`）；WS 事件总线 `ICollaMsg` 18 类（join/leave/
newCsEvent/csAckEvent/csRejEvent/csShouldRetry/permissionRej/
updateCursor(光标)/liveShare*/commentUpdate/uniscriptRun 等，
`colla-msg.ts:20-55`）；房间广播服务 Comb HELLO/JOIN/INGEST/HEARTBEAT
命令集（`universer/v1/comb.ts:27-46,127-130`）。

**OT 内置但分域**：docs 域 mutation 参数**直接就是 ot-json1 op**
（`core-editing.mutation.ts:63-69`），undo 用 `JSONX.invertWithDoc`、选区
随 op 变换 transformPosition（:109-127,204）；TextX 提供 apply/compose/
transform/invert（`text-x.ts:109-388`），JSONX 注册其为 json1 子类型
（`json-x.ts:22,42-57`）。**sheets 域 mutation 是普通 JSON params，仓内
零客户端 op transform 代码**（grep 零命中）。【推断】sheets changeset
的 rebase 在服务端完成——证据：`ISaveChangesetRequest` 响应
`concurrent: IChangeset[]` 注释 "need to follow up"（`snapshot.ts:
188-200`）+ `IApplyRequest.fastForward`（`univerpro/v1/apply.ts:92-101`）。

**快照+oplog 混合同步**：`GetUnitOnRev` 返回最近 snapshot + 待跟进
changesets（`snapshot.ts:271-287`）；SaveSnapshot 幂等（:394-398）、后台
checkpoint（`helper.ts:32-43`）；断线补拉 `FetchMissingChangesets
(from,to]`（:214-228）+ `(sid,reqId)` 单调幂等会话序列
（`changeset.ts:44-51`）+ `GetLatestCsReqIdBySid` 重连查位
（`snapshot.ts:202-212`）。

**presence 与权限推送**：`IUpdateCursor{unitID,memberID,selection:string}`
（`colla-msg.ts:85-93`，同用户多开不同 memberID）；权限变更运行时推送
updatePermissionObjEvent/shouldCloseConn（:33-48,170-173）。

## 6. 插件与扩展边界

**生命周期**：`abstract Plugin` 四钩子
onStarting/onReady/onRendered/onSteady（`plugin.service.ts:42-74`，对应
`LifecycleStages` 四阶段 `lifecycle.ts:20-41`）；依赖 `@DependentOn`
装饰器 + 注册时 DFS 拓扑排序、缺失依赖自动补注册（:104-119,231-292）；
版本不匹配仅告警不阻断（:200-212）【推断→应改严：编译期锁定】。

**60 包拆包原则 = 同构三层**：逻辑插件（模型/命令/mutation）与 UI 插件
分离（sheets-filter vs sheets-filter-ui），Node-only 依赖再拆 server 插件；
Facade API 必须实现在逻辑插件供服务端复用（`docs/ISOMORPHIC.md`
"Separate Plugins"/"Separate Facade API"）。

**preset = 预组装插件集**：`IPreset={plugins,locales}`（`presets/src/
preset.ts:23-31`）；`createUniver` 去重合并注册（:48-104）；注意
`collaboration:true` 时强制 override **移除** `IUndoRedoService/
IAuthzIoService/IMentionIOService` 三个本地实现为 Pro 让位（:51-55）——
OSS 与 Pro 的接缝实证。

**UI 扩展点 = 声明式菜单 schema**：`IMenuManagerService.mergeMenu`
合并菜单树（`menu-manager.service.ts:79,322-333`），应用层按 id 覆盖
hidden/disabled/order（:343-347）；工具栏/右键同属该体系；快捷键
`IShortcutService.registerShortcut`；Canvas 层
`registerRenderModule`（`ui.controller.ts:75-95`）。

## 7. AI 调用面（harness 定位的真实水位）

**README 一手声明**："The Office Harness for AI Agents"（`README.md:5`）
+ agent 三支柱：Programmatic editing / Output verification（内容检查/
截图/布局诊断）/ Worktree isolation（`README.md:133-141`）。

**仓内 agent 面 = 同一套 Facade（无 agent 专用接口）**：
`FUniver.newAPI(univer)`（`f-univer.ts:86`）+ FRange
`getValues/setValue/setValues`（`f-range.ts:583,1351`）+ 快照读写
`FWorkbook.save()`（`f-workbook.ts:180`）+ 命令级审计
`onCommandExecuted`（`f-univer.ts:177-205`）。幂等设计直接服务 agent：
评论 API caller 自定 ID 注释明说 "useful for idempotent agent
workflows"（`thread-comment-api.service.ts:43,66`）。

**headless = 一等公民**：`preset-sheets-node-core` 打包无 UI 逻辑插件
（`preset.ts:24-52`）；`rpc-node` 用 `child_process.fork` 主/从进程数据
同步（`rpc-node/src/plugin.ts:22-40`）——OSS 提供进程隔离原语。

**Pro 契约明确面向 agent**：`DirectWrite`（agent 身份直写 mutations、
返回 revision，`snapshot.ts:122-132,411-412`）、`Apply+fastForward`
（`apply.ts:92-109`）、`ForkUnit`（worktree 语义，`snapshot.ts:102-107`；universer v1
契约亦有，非仅 Pro）、`GetSSR→
PNG base64`（`ssr.ts:20-33`，输出验证）。

**【推断】harness 编排层在生态仓**：univer-cli（agents 的本地命令行
工作区）/ dsh-univer-office（DeepSeek Harness 插件，connected content/
validation/isolated worktrees）/ univer-workspace（自托管工作台）/
univer-mcp 均为 README 一手声明（`README.md:62,77-78,336`）且本仓零代码
引用——**本仓角色是"内核 + Facade"，agent 编排在仓外**。这对 mdcore 的
启示：harness 价值在编排层与内核 API 的分工，而非在内核里做 agent 特权
通道。

## 8. 性能与资源边界

**公式引擎（自研，架构可抄）**：手写状态机词法（`lexer-tree-builder.ts:
99-131`，2562 行）→ 类型化 AST（`parser.ts:48-89`）→ 树遍历解释器（非
编译，`interpreter.ts:132-173`，支持异步函数节点）。**依赖图 = 依赖树 +
RTree 空间索引，增量重算**：dirtyRanges 查 RTree（注释标 O(logN+k)）筛
脏公式、生成器 DFS 排计算序（`formula-dependency.ts:145-203,485-589`）；
`forceCalculate` 才全量。循环引用=三色标记检环 + 迭代计算
（:205-256；`calculate-formula.service.ts:154-168`）。**分片执行**：每
500 公式让出主线程可中止（:65,298-356）+ 命令后 10ms 防抖批量触发
（`formula-calculation-trigger.service.ts:34,113-145`）。**多层 LRU**：
AST 5000/词法树 2000/引用对象 10000（`generate-ast-node.ts:31,58-91`）。
**函数库 ≈504 内置函数**（math 82/statistical 111/financial 55/
engineering 57…）；自定义函数 `FFormula.registerFunction`
（`f-formula.ts:202,335-349`）。

**性能边界（诚实登记）**：仓库**无 benchmark 目录、无百万行量化声明**
（tests/ 仅 formula-integration；md 全文检索无 million/百万）。能力上限
源码间接证据：行高前缀和整表 O(n) 构建（`sheet.render-skeleton.ts`
`_updateLayout`）、无分片/懒加载（lazyLoad/virtualList 仅命中 design/ui 的
UI 虚拟列表组件，数据路径零命中）。
【推断】OSS 单机瓶颈在几十万行量级，大表/流式加载在 Pro Server——
mdcore 若做分块加载 + 骨架增量更新即为超越点。

**资源边界**：dispose 体系完整（`render-manager.service.ts:336-356`）
+ 官方内存泄漏排查手册（`docs/FIX_MEMORY_LEAK.md`）；无通用对象池（仅
画布复用与公式侧 LRU）；DPR 适配 pixelRatio 乘物理尺寸 + setTransform
（`canvas.ts:141-160`）；PerformanceMonitor 滚动窗 FPS/帧方差内建
（`performance-monitor.ts:54-114`）。

## 9. 敌意输入与执行边界

**公式引擎零远程调用**：fetch/XHR/axios/websocket 在 engine-formula/core
源零命中；远程能力收口在显式通道（filter 的 `useRemoteFilterValuesGenerator`
走 `_rpcChannelService`、network 包 HTTPService 可 override）——"纯计算
内核 + 显式 IO 抽象"的隔离范式。

**超链接防护**：`isSafeUrl` 协议白名单 http/https/mailto
（`common/url.ts:371-382`）；外链 `window.open(url,'_blank','noopener
noreferrer')` 且可被宿主 urlHandler 接管（`resolver.service.ts:210-221`）；
`javascript:` 注入有专门测试（`url.spec.ts:45`）。

**DOM 攻击面小但集中在剪贴板**：正文 Canvas 渲染不进 DOM；粘贴 HTML 用
DOMParser 解析并主动剥离 img/svg（`docs-ui clipboard.service.ts:174-178,
1112`）。

**权限三层 OSS 完备**：权限点注册/组合 PermissionService
（`permission.service.ts:48-120`）+ 对象规则 ObjectPermissionRuleModel
（objectId→permissionId，经 resource-manager 持久化，反序列化结构校验）
+ 三层点位枚举（Workbook/Worksheet/Range）与角色模式
（`permission-types.ts:48,98,144,153-161`）；鉴权 IO 是接口
`IAuthzIoService`（`authz-io/type.ts:37-52`）——OSS 只带本地实现，服务端
鉴权留白（与 preset collaboration 让位互证）。

**数据验证**：枚举 11 值/实际验证器 9 类（custom/list/textLength/date/
checkbox…，
`data-validation-type.ts:21-37`）；custom 验证器**走公式求值而非任意 JS**
（`custom-validator.ts:25-60`）——无 eval 类执行边界。

**安全治理缺口（诚实登记）**：SECURITY.md 只支持 latest（:9-13）且仍写
"not reaching version 1.0" 与 HEAD v1.0.4 矛盾（文档滞后）；声称 CodeQL
集成（:54）但 workflows 仅有 security-semgrep.yml（push main +
workflow_dispatch 触发）——【推断】实际静态安全门禁弱于文档声明。

## 10. 质量体系

**测试规模**：vitest workspace（`vitest.workspace.ts:4-6`），spec 与源码
同目录；**spec/test 文件共 1819**（packages+presets 计数）；tests/ 顶层仅
formula-integration（3 spec）。**无 Playwright/e2e**——UI 验证靠 storybook
+ examples 工作台（`README.md:351-374`）。

**CI 门禁**：4 分片单测（15 分钟超时）+ 汇总 + ESLint + typecheck
（`ci-quality-checks.yml`）；codecov 项目级 **target 80% + if_ci_failed:
error**；TS strict:true（`common/shared/tsconfigs/base.json`）。

**治理文档**：API 稳定性四级（stable/experimental/internal/deprecated，
`docs/API_STABILITY.md`）+ 弃用流程 + 命名规范 + 内存泄漏手册；commitlint
+ PR 标题校验。CHANGELOG 95 版本，1.0.0→1.0.4 约 3.5 个月 ≥12 个
release，但 CHANGELOG 顶部停在 1.0.0-beta.2 未同步 stable 条目（release
notes 部分走 GitHub Releases，自动化反面教材）。

## 11. 净室合规

**许可证**：LICENSE = 标准 Apache-2.0（176 行），无 NOTICE、无额外条款；
商标按 Apache §6 默认不授予。内嵌第三方：numfmt 3.2.6（MIT，
borgar/numfmt，`core/src/shared/numfmt/LICENSE`）——借用需保留该声明。

**DREAMNUM.md**（仓库职责地图，写给自动化工具/agent）："does not own
Univer Pro's commercial collaboration, import/export, server, or
enterprise capabilities"——OSS/Pro 边界的机器可读声明。

**Pro 边界清单**（`README.md:305-312`）：Sheets Pro=协同/编辑历史/导入
导出/打印/图表/透视表/sparklines/outline/shapes/数据连接器；Docs Pro=
协同/导入导出/打印/增强表格/分栏/callout/代码块/形状；Slides Pro=模型
UI/导入导出/图表表格/形状；Server Pro=协同服务器/Node 协同客户端/SSR/
服务端计算/changeset replay。四条边界原则（:318-323）：OSS 自足、OSS bug
在 OSS 修、文档不得暗示 Pro 能力。

**对竞品内核的影响**：Apache-2.0 允许商用/修改/再发行（义务=保留版权
与许可声明）；净室路线（只读行为、不拷代码）无义务；接口级模仿不构成
侵权但建议差异化命名。【推断】"无恶心壁垒"判断在办公线同样成立，但
**保真难度不为零且开源实现密度低于画布线**（xlsx 进出全闭源）——
差异化空间在 AI-native 引擎 + 开源 OOXML 进出。

## 12. 能力矩阵（dsh 插件面 + MCP 工具面双口径）

| 能力 | OSS 仓内（实证） | Pro/生态仓（README 声明） |
|------|----------------|------------------------|
| 程序化读写 | Facade FRange/FWorkbook + 快照 save/createUnit | DirectWrite（agent 身份+返回 rev） |
| 无头运行 | preset-sheets-node-core + rpc-node fork | Server 计算委派/SSR |
| 输出验证 | save() 快照 + onCommandExecuted 审计 | GetSSR→PNG + 内容检查/布局诊断 |
| 隔离协作 | —— | ForkUnit（worktree）+ univer-workspace |
| MCP 工具面 | ——（univer-mcp 为外部仓） | dsh-univer-office（DeepSeek Harness 插件） |
| 协同 | protocol 契约 + docs 域 OT 原语 | 协同服务器 + sheets rebase |

**口径结论**：dsh 插件面与 MCP 工具面的实际交付都在生态仓/Pro——本仓
只提供"内核 + Facade"这层。【推断】mdcore 若把 MCP 工具面做进内核仓
（mdshell 既有方向），相对 Univer 是更彻底的 AI-native 一体化。

## 13. 对工作台的可移植结论（按优先级）

1. **三层命令分类直接照搬**：COMMAND(编排)/MUTATION(持久化+协同单元)/
   OPERATION(视图态不进 op log)——Rust 用 enum + 类型状态实现；这是
   "视图态与持久态分离"的成熟答案。
2. **undo=逆操作需执行前捕获旧值**：`UndoMutationFactory` 模式可行但
   脆弱；更优路径是 **apply_ops 直接返回逆 op**（docs text-x 的 delete
   带 body 便于 invert 已朝此演化，`action-types.ts:33-75`）。
3. **op-first 优于快照即模型**：Univer 的 ObjectMatrix 包引用原地改 +
   外挂 undo/协同是历史包袱；mdcore 坚持单一入口 + 字段私有（消除
   setCellContent 式约定性旁路）。
4. **trigger 因果链**：mutation 自动标注来源 COMMAND——Rust op 内建
   `cause` 字段，审计/录制免费获得。
5. **两条 canvas 缓存路线按内容类型分治**：表格类（规则网格）用视口级
   双缓冲 + 条带 diff + 前缀和二分虚拟滚动（Univer 路线）；自由绘制用
   per-element 位图缓存（Excalidraw 路线）——mdcore 双页型可并存。
6. **公式引擎架构整套可借鉴**：AST 解释器 + RTree 依赖图 + 三色检环 +
   时间片让出 + 多层 LRU——增量重算入口是 dirtyRanges 空间查询而非全图
   遍历，这是可扩展到十万公式的关键。
7. **协同版本骨架**：baseRev/revision + (sid,reqId) 幂等 + checkpoint
   幂等保存 + (from,to] 断线补拉——实现成本低于全序广播且支持历史回放。
8. **agent 走同一 op 通道**：DirectWrite/ForkUnit/SSR 验证范式证明
   "agent 与人共享同一 changeset 管线"可行——mdcore 的 AI 面不应做旁路
   改文件，而是同一 op 通道 + 幂等命令 + 审计日志。
9. **resources sidecar**：插件状态不入核心 schema（register/load/save
   hook）；配合显式 schema version + 迁移器（对冲 Univer 无迁移器的
   风险）。
10. **同构三层 → Cargo features**：逻辑/UI/server 拆包原则对应
    `ui`/`server` feature gates；headless 是一等公民而非事后适配。

## 14. 与工作台架构的对照（一页）

| 决策点 | Univer 选择 | mdcore 现状/规划 | 取舍建议 |
|--------|------------|----------------|---------|
| 模型 | 快照即模型（原地改） | op-first（document_id/apply_ops） | 坚持 op-first |
| 命令分层 | COMMAND/MUTATION/OPERATION | apply_ops 单入口 | 吸收三层分类为 op 类型状态 |
| undo | 逆操作+执行前捕获 | 树结构逆 op（已有） | apply_ops 返回逆 op 收口 |
| 渲染 | 视口缓存+条带 diff | 双 canvas（Excalidraw 式） | 表格页用视口路线 |
| 公式 | AST+RTree+增量重算 | 无（MD 线暂不需要） | 表格引擎立项时整套借鉴 |
| 协同 | 协同单元=MUTATION，服务端 rebase（Pro） | 单机 | 版本骨架照抄，客户端 rebase 开源化=超越点 |
| agent 面 | Facade+生态仓编排 | mdshell host-only 面 | MCP 工具面进内核仓=差异化 |
| 进出格式 | xlsx 全 Pro | PDF 双引擎已定谳 | 开源 OOXML 进出=蓝海 |

## 附：证据锚点密度

四路勘察合计 ~110 处 `路径:行号` 锚点，覆盖 11 维全部章节；本报告为
综合层，锚点已内嵌各节。复审记录：独立复审代理（office-a-1r）对 ~25
锚点抽查 + 语义核对后定稿（见 worklog office-a-1）。
