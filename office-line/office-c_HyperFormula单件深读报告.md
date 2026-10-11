# office-c：HyperFormula 单件深读报告（组件版 11+4 维）

> **净室分析报告**（线 A 办公 P1 批次第三篇）。日期：2026-10-11。
> 对象：handsontable/hyperformula @ `99a45ea`（master 浅克隆，v3.5.0，
> 双许可：GPLv3 或专有商业许可）。注意：计划文档所记 v3.4.0 为计划时点
> 版本，HEAD 已是 **v3.5.0**（2026-10-09 发布，CHANGELOG `CHANGELOG.md:10`），
> 本轮以实况为准；仓主 org 为 `handsontable`（`handsoncode` 为旧 org 名，
> ls-remote 已不可达，`package.json:6` 的 repository 字段为准）。
> 方法：单代理全维度深读（组件为界，无渲染/协同维——其缺位本身为登记项）
> + **本地实测**（`npm install` 仅装依赖 + `tsc` 出 commonjs 构建后直接驱动
> 引擎：函数计数、license 门控行为、性能探针、敌意输入探针，实测数据逐处
> 标注「实测」）；证据锚点 `路径:行号`（相对仓根），【推断】= 分析员判断。
> 生态件（`@hyperformula/mcp`、`hyperformula/ai`、`hyperformula/langchain`、
> 私有测试仓 `hyperformula-tests`）不在本地，仅引仓内文档一手声明并逐处标注。
>
> 一句话定性：**gridless 计算内核的教科书实现——纯数据进、纯结果出、
> 生产依赖仅 2 个（chevrotain + tiny-emitter），稀疏整数图 + 迭代 Tarjan +
> 惰性 AST 变换 + RangeVertex 聚合缓存把 Excel 语义做成了可完全外置的
> 确定性计算服务；但它的 AI 外置面（MCP/AI SDK）全部是"coming soon"
> 的 roadmap，且函数级行为锁全锁在私有测试仓——**引擎开放、质量底座
> 闭锁**，双许可的/gpl-v3 通道就是净室对照的取用方式**。

## 0. 组件速览

```
src/（全量 54,558 行 TS，单一 npm 包；根层 38 文件 13,854 行——
  13,854 仅是根层口径，子目录大头：interpreter 19,827【含 vendored
  3rdparty 1,504】、i18n 7,833、vendored license-key-parser 1,853）
HyperFormula.ts（5141 行）＝ 唯一门面：build* 工厂 + ~120 公有方法 + 事件
  ├─ BuildEngineFactory ＝ 装配线：Config→Registry→Graph→Parser→Evaluator
  ├─ Config（~40 参数）＝ 不可变配置 + license 解析结果（WeakMap 私有化）
  ├─ parser/ ＝ Chevrotain 词法/语法 → AST + hash 缓存 + Unparser 反解
  ├─ DependencyGraph/ ＝ 稀疏整数图 + 四类 Vertex + RangeMapping + TopSort
  ├─ interpreter/ ＝ 树遍历解释器 + 54 个函数插件（423 个可调用 id）
  │   └─ plugin/3rdparty ＝ 内嵌 bessel.js(Apache-2.0) + jStat(MIT)
  ├─ CrudOperations/Operations/UndoRedo/Clipboard ＝ 结构变更 + 逆操作栈
  ├─ LazilyTransformingAstService ＝ 行列增删的惰性 AST 变换 + 版本号
  ├─ license/ ＝ 双门许可：Gate A(#LIC!/抛错) + Gate B(按函数/特性授权)
  ├─ Evaluator ＝ 全量 run / 增量 partialRun（脏点种子 + Tarjan 子图）
  └─ i18n ＝ 18 个语言包（函数名+错误码+UI 串翻译）
生产依赖：chevrotain ^6.5.0 + tiny-emitter ^2.1.0（package.json:110-111）
生态（仓外）：@hyperformula/mcp / hyperformula/ai / hyperformula/langchain
  （全部"not available yet — coming soon"，docs/guide/mcp-server.md:15-16 等）
  + 私有测试仓 hyperformula-tests（test/README.md:7-9）
```

## 1. gridless 边界设计（核心维）

**定位声明是一手文本**："An open-source headless spreadsheet for business
web apps"（`README.md:9`）；README 用例清单第一条即 "Deterministic compute
layer for AI & LLMs"（`README.md:31`）——把"给 LLM 当确定性计算层"写进
README 用例，是办公全家桶里唯一一件。

**零 UI/零 IO 依赖的实证（三层证据）**：

1. **依赖面**：生产依赖恰 2 个——chevrotain（解析器）+ tiny-emitter（事件）
   （`package.json:110-111`）。grep 全 src 无 react/vue/dom/window/fetch/
   xhr/websocket 引用（实测 rg 零命中于 src/）；`getContext`/`canvas`/
   `requestAnimationFrame` 零命中。
2. **运行面**：同构到 Node 无需任何 polyfill——官方 server-side 文档仅要求
   Node 13+ 的 full-icu（`docs/guide/server-side-installation.md:9-14`），
   本地实测 `tsc` 直出 commonjs 后纯 Node 驱动即可 build/求值/读值
   （见 §6/§7 实测记录）。
3. **数据面**：进出全是普通 JS 值——进：`buildFromArray(sheet: Sheet)` /
   `buildFromSheets(sheets: Sheets)`（`src/HyperFormula.ts:283,331`），
   Sheet 就是二维数组 `RawCellContent = Date | string | number | boolean |
   null | undefined`（`src/CellContentParser.ts:25`）；出：`getCellValue →
   CellValue`（`src/HyperFormula.ts:866`）、`getSheetValues → CellValue[][]`
   （`src/HyperFormula.ts:991`）、`getAllSheetsSerialized → RawCellContent[][]`
   （`src/HyperFormula.ts:1183`）。**无任何渲染语义、坐标只有
   {sheet,col,row} 整数三元组**（SimpleCellAddress，`src/Cell.ts:192`）。

**「数据进结果出」的完整闭环形态**（这正是外置能力的理想形态样本）：

```
输入侧（三形态）:  JS 二维数组 / 多表对象 / named expressions 序列化数组
                  （buildFromArray:283 / buildFromSheets:331 / buildEmpty:360）
变更侧:            setCellContents / addRows...moveCells / clipboard / namedExpr
                  （全部返回 ExportedChange[] 变更回执，src/Exporter.ts:18）
读取侧:            getCellValue / getSheetValues / getSheetSerialized /
                  getCellDependents / getCellPrecedents（依赖可解释性！）
                  （src/HyperFormula.ts:3355,3393）
一次性求值:        calculateFormula(formula, sheetId) —— 不入图的 fire-and-forget
                  （src/HyperFormula.ts:4570-4589；runAndForget 源 src/Evaluator.ts:56-75）
事件侧:            8 个生命周期事件（sheetAdded/Removed/Renamed、
                  namedExpression*/valuesUpdated/evaluationSuspended/Resumed，
                  src/Emitter.ts:9-18）——事件载荷是变更数组而非视图请求
```

关键洞察：**HyperFormula 的"无头"不是"渲染被剥掉"，而是从未存在过渲染**。
单元格没有样式/边框/列宽概念（Config 里没有，ICellData 等价物 CellValue 里
也没有）——与 Univer 的"快照含样式 + 渲染旁挂"是两种不同的解耦深度。
代价：**单格无富文本、无格式化模型**（TEXT 函数做字符串化，自定义
`stringifyCurrency` 钩子，`docs/guide/currency-handling.md`）。
【推断】这正是 MCP 化/dsh 工具化的最低摩擦形态：工具输入输出可直接
映射 `RawCellContent[][]` ↔ `CellValue[][]`，官方 roadmap 的
`createSpreadsheetTools`（docs/guide/ai-sdk.md:44）API 形态也确实就是
"把 hf 实例包成 tools"。

## 2. 数据模型

**单元格 = 依赖图节点（Vertex 类型系统）**，五类（`src/DependencyGraph/`）：

- `ValueCellVertex`——存已解析值 + 原始串双份（RawAndParsedValue，
  `src/DependencyGraph/ValueCellVertex.ts:13,21`）；
- `ScalarFormulaVertex`——AST + 地址 + **版本号** + 缓存值
  （`src/DependencyGraph/FormulaVertex.ts:245-284`，cachedCellValue:246）；
- `ArrayFormulaVertex`——数组公式（占多格，CellArray 值矩阵，含
  NotComputedArray/ErroredArray 三态，:98-108）；
- `RangeVertex`——区间节点，**本身是聚合缓存容器**（见 §3）；
- `EmptyCellVertex`——空格占位：仅当它是（含间接）依赖时才存在
  （不变量声明于 `src/DependencyGraph/DependencyGraph.ts:72-74`）。
- 解析失败不是异常：`ParsingErrorVertex` 进图、读出来是 `#ERROR!`
  （`src/DependencyGraph/ParsingErrorVertex.ts`；实测 `=SUM(` → `#ERROR!`）。

**地址→节点 = AddressMapping，稠密/稀疏双策略**：按 sheet 建策略，
fill 率超阈值选 DenseStrategy（行数组）否则 SparseStrategy（Map），
策略可注入（AlwaysDense/AlwaysSparse/DenseSparseChooseBasedOnThreshold，
`src/DependencyGraph/AddressMapping/ChooseAddressMappingPolicy.ts:9-30`）；
默认 AlwaysDense（`src/Config.ts:50`）。**sheet 容量在建表时按数据边界
锁定**（findBoundaries + 尺寸硬检 + 策略选定，`src/BuildEngineFactory.ts:
104-111`），越界操作抛 `SheetSizeLimitExceededError`（§7）。

**区间表达 = AbsoluteCellRange（不可变矩形）**：547 行完整实现
（`src/AbsoluteCellRange.ts`），列区间/行区间/整列（A:A）为其子类；
`SimpleRangeValue` 区分"仅区间引用（lazy，经图取值）"与"已物化值矩阵"
（onlyValues/onlyRange，`src/SimpleRangeValue.ts:67,74`）——大区间在解释器
里是**引用传递而非拷贝**。

**序列化与快照**：无独立持久化格式——`getAllSheetsSerialized()` 即
"输入形态回放"（unparser 把 AST 反解回 A1 文本，`src/Serialization.ts:
50-58`）；`rebuildWithConfig`（updateConfig 的实现）就是"序列化全部表 +
named expressions → 用新 Config 整体重建引擎"（`src/HyperFormula.ts:
5068-5093`）——**引擎不可热改配置，改配置=重建**，这保证了 Config 与
图/parser/求值器生命周期的一致性（也是 license 换 key 的实现路径）。

**无版本化 schema、无迁移器**：序列化产物是普通 JS 值的二维数组 +
named expression 数组（`src/Serialization.ts:16-21`），不含引擎版本号。
【推断】与 Univer 同款"无迁移器"债，但因数据模型极简（无样式/合并格/
富文本），漂移面也极小——这是"内核瘦"带来的意外收益。

## 3. 依赖图工程（核心维）

**图结构 = 稀疏整数数组 CSR 风格**（`src/DependencyGraph/Graph.ts:28-63`）：
节点进图得自增 `idInGraph`（:141-155），`nodesSparseArray[id]` 存节点、
`edgesSparseArray[id]` 存邻接 id 数组；删节点是 delete 槽位（:187-207），
脏邻接先入 dirty 集再删（:194-197）。边去重（:177-179）。**节点 id 是
稳定小整数**——Tarjan 的 entranceTime/low/parent 全用整数数组按 id 索引
（`src/DependencyGraph/TopSort.ts:22-29`），对象只做载荷。源码里两处
自注性能债：邻接表 `Array.includes` 是 O(n) 使环检测整体 O(n²)
（`src/DependencyGraph/TopSort.ts:172-175`；`src/DependencyGraph/Graph.ts:25-27`）。

**拓扑序 + 环检测 = 迭代 Tarjan**（显式 DFS 栈，无递归，
`src/DependencyGraph/TopSort.ts:45-53,65-92`）：三色状态机
ON_STACK→PROCESSED→POPPED（:12-16），产出 {sorted, cycled} 两列
（:165-187）。环上的公式**直接置 `#CYCLE!` 错误值**（不是禁用迭代计算
或警告——与 Excel 的循环引用警告模型不同，HyperFormula 拒绝环，
`src/Evaluator.ts:102-112`；全量路径 :117-122 同款）。依赖收集发生在
**解析期**而非求值期（官方差异表：`docs/guide/list-of-differences.md:41`
"Dependencies are collected during the parsing phase"）——所以
`=IF(FALSE(), A1, 0)` 里 A1 自引用也会报 CYCLE（`docs/guide/
known-limitations.md:21-24` 一手登记），这是对齐 Excel 的**已知不兼容面**。

**RangeVertex = 区间即缓存（本维工程精髓）**。两个机制叠加：

1. **小区间前缀复用**：B5:D20 若已存在 B5:D19 节点，则新区间 = 旧区间
   节点 + 尾行三格（`RangeMapping.findSmallerRange`；`src/DependencyGraph/
   DependencyGraph.ts:202-213`；官方文档专章讲 n²→n 的动机：
   `docs/guide/dependency-graph.md:51-74`）。找不到更小区间则 bruteForce
   逐格建边（:212），一旦将来出现可复用前缀再逐格拆边（:205-210）。
2. **聚合缓存**：RangeVertex 内挂 `functionCache`（SUM/COUNT 等可结合
   聚合按函数名缓存）+ `criterionFunctionCache`（SUMIF/AVERAGEIF 按
   判别式缓存）+ `dependentCacheRanges`（嵌套区间级联失效）
   （`src/DependencyGraph/RangeVertex.ts:22-31,95-109`）；求值器对
   RangeVertex 的处理就是"清缓存"（`src/Evaluator.ts:91-93`）。配套
   ColumnIndex 值→地址倒排（含 version 号惰性变换，`src/Lookup/
   ColumnIndex.ts:39-53`）服务 VLOOKUP/MATCH。

**增量重算调度**：写操作只 `markNodeAsDirty`（`src/DependencyGraph/
DependencyGraph.ts:93`），volatile 函数（RAND/NOW…）进 volatile 集
（`src/DependencyGraph/Graph.ts:284-293`；解析期由函数元数据标注，
`src/parser/Cache.ts:28-29`），每次重算 dirty ∪ volatile 合并做种子
（:312-314）；结构敏感函数（OFFSET 等）挂 changingWithStructure 集，
增删行列时一并标脏（:327-347）。**每个公有变更方法尾部统一走
`recomputeIfDependencyGraphNeedsIt()`**（`src/HyperFormula.ts:5118-5140`）：
取脏种子 → Tarjan 子图拓扑（`Evaluator.partialRun`，`src/Evaluator.ts:
44-54`）→ 求值函数返回"值是否变化"决定是否继续向下游传播
（`TopSort.postprocess` 的 operatingFunction 谓词，`src/DependencyGraph/
TopSort.ts:180-183`）——**不变不传**，避免无谓下游遍历。

**批重算与惰性求值**：

- 批操作 = `batch(cb)`：挂起求值 + undo 合批 + 末尾一次 resume 收割
  全部变更（`src/HyperFormula.ts:3894-3907`）；resume 是唯一出口且
  **刻意不设 license 门**（防"许可变更把实例锁死在挂起态"，工程注释
  原文 :3982-3990——这个"释放阀不设门"的原则本身值得照抄）。
- **惰性 AST 变换是本维第二精髓**：增删行/列/移动不立刻改所有公式，
  而是压入变换队列；每个 FormulaVertex 记自己的版本号，被读到时才
  apply 增量变换（`src/DependencyGraph/FormulaVertex.ts:60-67`；
  `src/LazilyTransformingAstService.ts:14-46` 模块注释整段讲清问题/
  方案/压实）。变换队列长度到 `maxPendingLazyTransformations`（默认 50，
  `src/Config.ts:79`）就强制全量 apply + compact（`src/HyperFormula.ts:
  5100-5107`；compact 推进 versionOffset :135-138）——**内存有界性是
  配置化的**（3.3.0 专门为此加的选项，`CHANGELOG.md:59`）。
- 不可逆变换（删行删列）额外把 pre-image 存进 undo（`src/
  LazilyTransformingAstService.ts:94-97`），undo 机制与惰性变换通过
  版本号耦合（`src/UndoRedo.ts:489-551` 的 oldData 生命周期注释）。
- 数组公式**例外地走急切变换**（`src/DependencyGraph/FormulaVertex.ts:
  224-228` 显式 no-op）——数组占多格，地址不跟版本走。

【推断】对照 Univer 的 RTree 方案：HyperFormula 的"区间前缀复用"只对
**同起点嵌套区间**有效（SUM(A1:A1..A1:A100）场景），对任意相交区间的
脏区筛选仍是图遍历；两者是"聚合缓存优先"vs"空间索引优先"的两条路线，
对 mdcore 的启示是：**先问"聚合能否复用"，再决定要不要空间索引**。

## 4. 函数库规模与对齐

**实测统计（本轮直接驱动引擎）**：`HyperFormula.buildEmpty({licenseKey:
'gpl-v3'}).getAvailableFunctions().length = 423`。静态口径对账（脚本
扫 `src/interpreter/plugin/*.ts` 的 `implementedFunctions` 与 `aliases`）：
**369 个规范名 + 53 个别名 = 422，加保护函数 OFFSET = 423**，与运行时
实测逐位吻合。构成：54 个插件文件（`src/interpreter/plugin/`，含
FunctionPlugin 基类与 3rdparty 子目录），最大 5 类为 Statistical 46 /
Financial 29 / DateTime 27 / Complex 26 / Text 21。README 口径
"over 400 built-in functions"（`README.md:43`）与文档页模板变量
`{{ $page.functionsCount }}`（由同一 API 现算，`docs/.vuepress/config.js:
74-76`）——**文档口径与代码实测一致，无夸大**。

**别名机制**：`aliases: {VAR: 'VAR.S', ...}` 53 条，全为 Excel 遗留名
（VAR/STDEV/NORMDIST/TDIST…），别名共享目标的实现与元数据、保留自己的
可调用名（`src/interpreter/plugin/FunctionPlugin.ts:270`；`src/
HyperFormula.ts:708-716`）。两个**保护函数** VERSION/OFFSET 永不设许可
门（`src/interpreter/FunctionRegistry.ts:42-45`；OFFSET 在解析期被改写
为纯引用——官方 known-limitations 专节，`docs/guide/
known-limitations.md:64-96`）。

**对齐策略（工程面三支柱）**：

1. **标准锚点是 OpenDocument 公式规范**（ODF 1.3 part-4），Excel/Sheets
   作为行业实践参考（`docs/guide/compatibility-with-microsoft-excel.md:22`）。覆盖声明：**350/515 Excel 函数（68%），另有 Excel 之外的
   扩展（HF.* 运算符函数、SPLIT、MAXPOOL、INTERVAL…），合计 423**
   （`docs/guide/compatibility-with-microsoft-excel.md:30`，标注
   "as of 3.1.0 and Excel 2024"）。
2. **差异显式登记**：`docs/guide/list-of-differences.md` 整页三栏对照表
   （HyperFormula vs Google Sheets vs Excel），细到 TBILLEQ/GCD/COMBIN
   的具体数值差异（:64-71）；known-limitations 单列"函数实现细节"
   （SUBTOTAL 不忽略嵌套小计、CHISQ 系列大参数线性慢、INDEX 不支持
   整行/列返回、FILTER 维度限制、TEXT 不支持 "" 转义等，
   `docs/guide/known-limitations.md:39-62`）。
3. **可配置兼容开关**：caseSensitive/accentSensitive/caseFirst/
   ignorePunctuation/localeLang/leapYear1900/useWildcards/
   useRegularExpressions/matchWholeCell（`src/Config.ts:45-82`）——
   "对齐"不是硬编码，而是**默认贴 Excel + 配置项贴 ODF/locale**。

**行为对齐的锁在哪里？——不在开源仓**（§8）。CHANGELOG 的 Fixed 条目
（如 3.5.0 的 MOD 符号修正对齐 Excel :32、VAR/STDEV 精度重写 :28-31）
显示对齐是**持续回归修正**过程，且每条都带 PR
链接——修正发生在带私有测试套件的 PR 里。

**i18n**：18 个语言包（`src/i18n/languages/`，enGB/enUS/deDE/frFR/
plPL/ruRU/…/idID），包内 `functions` 字典把规范名映射为本地名（enGB
约 422 键，`src/i18n/languages/enGB.ts:20-443`）；**无翻译的函数在该
语言下不可调用**（getAvailableFunctions 过滤 isFunctionTranslated，
`src/HyperFormula.ts:766-769`）——语言包是函数可用性的一部分。
【推断】对 mdcore：函数目录（canonical id）与本地化目录分离 + "未翻译
即不可见"的强一致原则可移植。

## 5. API 面与 license key 机制的技术执行

**API 面**：单类 `HyperFormula` + 静态注册面（语言/插件/函数）。
构造仅三条工厂（buildFromArray/Sheets/Empty，:283,331,360）+ `destroy()`
（:4987-4989，走 objectDestroy 递归置空防 use-after-destroy）。方法分组：
Cells/Rows/Columns/Sheets/Clipboard/Range/NamedExpressions/Helpers/
Events/Config——每个写方法配 `isItPossibleTo*` 谓词（如 :1423,1567），
谓词 = "调用同名方法不会抛"的完整投影（含许可检查，:4046-4049）。
**所有写方法返回 `ExportedChange[]` 变更回执**（`src/Exporter.ts:18`）
——宿主拿回执驱动自己的视图/存储，引擎不反向调用宿主任何东西。

**license key 机制（v3.5.0 重写为双门架构，本轮实测验证）**：

- **解析路由**（`src/license/licenseResolution.ts:165`）：带方括号块的
  entitlement 新格式走 vendored 读钥器（`src/license/
  handsontable-license-key-parser/`，上游 `handsontable/license-key` 仓
  的逐字节拷贝 + CI 漂移检查 `.github/workflows/vendored-parser.yml:
  1-15`）；其余（`gpl-v3`、25 字符经典 key、空串）走经典校验器
  `checkLicenseKeyValidity`（`src/helpers/licenseKeyValidator.ts:225`）。
  **`gpl-v3` 字面量直接 VALID**（:230，另有 internal-use 与硬编码 trial
  key 两字面量同列）。经典 key 的校验是混淆过的 25 字符模式校验
  （`src/helpers/licenseKeyHelper.ts:17-28`，@ts-nocheck + 混淆标识符）
  + **key 内嵌过期日 vs 构建期内联的发布日期**（HT_RELEASE_DATE 由
  构建期写入，`ht.config.js:9` = '09/10/2026'；比较用 UTC 修复过时区
  bug，`CHANGELOG.md:24`）。
- **Gate A（阻断门）**：`blocksEvaluation=true`（missing/invalid/过期
  经典 key/试用超宽限）时，**函数求值返回 `#LIC!` 错误**（除 VERSION/
  OFFSET），**所有门控 API 抛 `LicenseCapabilityMissingError`**
  （`src/license/FunctionCallLicenseGate.ts:36-56`；`src/errors.ts:442`）。
  门控面 = 编辑 cells/rows/columns/sheets、copy/cut/paste、undo/redo、
  batch、suspendEvaluation、named expressions 全部（文档口径
  `docs/guide/license-key.md:81-93`；代码证据：setCellContents 首句
  `this.ensureCapability(FeatureId.Crud)` `src/HyperFormula.ts:1481`；
  `ensureFeatureAllowed` `src/license/ensureFeatureAllowed.ts:39`）。
  **读方法与清理方法刻意不门控**（getCellValue/clearClipboard/destroy/
  resumeEvaluation），工程注释整段论证"释放阀不设门"（:5002-5031）。
- **Gate B（授权粒度门）**：entitlement key 按 token 授予子集——
  `feat:crud/undo_redo/clipboard/named_expressions/batching` 5 个特性
  区（`src/license/featureCapabilities.ts:8-15`）+ 21 个函数组
  （`fun:math.a`…，`src/license/functionCapabilities.ts:16-64`）+
  单函数 token + `fun:all`/`feat:all`（:64-110）。未获授权的函数调用
  求值为 `#LIC!`，其余照算；未识别 token **fail-closed 且静默**
  （`src/license/LicenseEntitlement.ts:60-68` 注释原文）；自定义函数
  永不进 gate（`src/license/functionCapabilities.ts:70-75` 明确论证）。
  `getAvailableFunctions()` 只列授权函数——函数选择器不会推荐一个
  调用即失败的函数（`src/HyperFormula.ts:741-785`，:719-735 有整段
  "Gate B only, deliberately" 论证）。
- **离线执行**：文档明示 "HyperFormula doesn't use an internet
  connection to validate your proprietary license key"（`docs/guide/
  license-key.md:51`）——**无 phone home**（实测：无网络行为，
  全部本地）。生命周期宽限矩阵（usage/trial/release ×
  valid/notice/soft_stop/hard_stop）在 `LIFECYCLE_VERDICTS`
  （`src/license/licenseResolution.ts:52-66`），原则="过期的付费 key
  永不硬阻断客户"（trial_hard_stop 是唯一阻断项）。
- **防绕过工程细节**：capability 解析缓存于 Config（WeakMap 私有池
  `src/Config.ts:33-40,282-291`）；热改 key 走 rebuildWithConfig 全引擎
  重建（§2）；entitlement 载荷被当攻击面处理——key 内 125k token 的
  spread 攻击会打爆调用栈，已实测测量并改为 Set 构造
  （`src/license/licenseResolution.ts:114-121` 注释）。

**实测复验（本地直接驱动）**：空 key → console.warn "The license key
for HyperFormula is missing."，`=SUM(1,2)` → `#LIC!`（DetailedCellError
message: 'License key is missing.'），`=VERSION()` 照常求值，
`isItPossibleToAddRows → false`，`setCellContents` 抛
`LicenseCapabilityMissingError: License key is missing. Feature crud
is not available.`；`gpl-v3` → 423 函数全可用、无任何 console 输出。
**技术执行的定性：不是"拒绝实例化"的硬锁，而是"降级运行"的软锁**——
引擎照常构建、照常读值，产出物被污染（#LIC!）+ 写路径抛错。
【推断】这是"可审计的执行"（错误值可被宿主检测）而非"防破解"——
JS 仓的 license 门本质不可防篡改，Handsontable 的选择是把执行做成
**诚实降级 + 法律条款兜底**，这与 GPLv3 双许可的商业模式自洽。

## 6. 性能与内存

**官方性能基线在私有仓**（`test/hyperformula-tests/performance/
run-basic-benchmark.ts`，package.json scripts `benchmark:basic/cruds`；
CI 有专用 workflow 对比 base/head 两个 commit 的基准
`.github/workflows/performance.yml:30-45`），OSS 仓不可见具体数字。
公开口径仅 GitHub 徽章级（"High-speed parsing and evaluation"，
`README.md:41`，无量化声明——诚实）。

**本轮实测（Node 24，单线程，tsc 直出构建，gpl-v3）**：

| 场景 | 构建 | 单点变更重算 |
|---|---|---|
| 依赖链 1,000 格（=A_{n-1}+1） | 61 ms | 13 ms |
| 依赖链 10,000 格 | 137 ms | 41 ms |
| 前缀 SUM 1,000 行（A1:A1..A1:A1000，n² 依赖场景） | 89 ms | 7.4 ms |
| 前缀 SUM 5,000 行 | 151 ms | 54 ms |
| 50 万值矩阵（25k×20）+ SUM 全表 | 985 ms | 74.6 ms（heap 322 MB） |

- 伸缩曲线：链场景构建近线性、单点重算线性于链长（全下游传播，符合
  预期）；前缀 SUM 场景若无 RangeVertex 复用将是 O(n²) 边，实测 5,000
  行构建仅 151 ms，证明小区间前缀复用生效。
- 50 万值 + 重算 ~75ms 的量级【推断】对"LLM 工具单次调用"的延迟预算
  完全够用；322 MB heap 对浏览器 tab 偏重——AddressMapping 默认
  AlwaysDense 的取舍在超大稀疏表上会放大内存（可切 AlwaysSparse）。
- **结构性限额**：默认 maxRows 40,000 × maxColumns 18,278（Excel 列上限，
  `src/Config.ts:65-66`），建表与增行列都按此硬拒（实测 10 万行链与
  5 万行 addRows 均抛 `SheetSizeLimitExceededError`，抛点
  `src/BuildEngineFactory.ts:107-108`）——**默认上限即第一道资源护栏**。
- 统计内建：StatType 17 项逐段计时（BUILD_ENGINE_TOTAL/PARSER/
  COLLECT_DEPENDENCIES/TOP_SORT/EVALUATION/CRITERION_FUNCTION_*
  _CACHE_USED…，`src/statistics/StatType.ts:6-27`），`useStats:true` +
  `getStats()`（`src/HyperFormula.ts:1260`）——性能观测是一等公民；
  甚至缓存命中率都有计数（`statsCacheUsed`，`src/parser/
  ParserWithCaching.ts:45,93`）。
- 内存策略盘点：AST 池按 hash 全局缓存（**无淘汰**，`src/parser/
  Cache.ts:26` 无界 Map——与 Univer 的多层 LRU 相反，靠"公式去重后
  基数有限"假设）；RangeVertex 缓存按失效清空；lazy transformations
  按阈值压实（§3）；undo 栈默认 20 条（`src/Config.ts:78`）。**无大
  文档分片、无冷热数据分层**——单实例单工作簿（`docs/guide/
  known-limitations.md:19-20` 明示"Multiple workbooks are not
  supported"）。【推断】大工作簿=多实例进程外分片是宿主的责任，
  引擎自身不提供任何分片原语（无 worker 化——根目录 worker/ 是文档站
  的 Cloudflare Worker，`wrangler.jsonc:11-13`，非计算 worker）。

## 7. 敌意输入防护

**循环引用**：Tarjan SCC 检出即置 `#CYCLE!`，不迭代计算、无迭代设置
（§3；实测 A1=B1,B1=A1 及 =SUM(A:A) 自列引用均得 #CYCLE!——后者与
Excel 循环引用警告语义一致）。解析期收集依赖导致 `IF(FALSE(),A1,0)`
型假环也报错（官方登记为限制，`docs/guide/known-limitations.md:21-24`）。

**深度爆炸公式（实测，512 MB heap 约束）**：

- 括号嵌套：depth 100 正常求值；**depth ≥500 抛 RangeError**；
- SUM 嵌套：depth 100 正常；**≥1000 抛 RangeError**；
- 一元负号链：**≥1000 抛 RangeError**（无界 2GB heap 下更糟：深式在
  打满堆之前不抛任何错，直接 node OOM abort——RangeError 是 V8 栈深
  保护的副产品，**不是引擎级深度限额**；栈顶确认抛自 Chevrotain
  `subruleInternal` 递归）。
- 关键行为：`setCellContents` 塞入深度炸弹会**把未捕获 RangeError
  抛出公有 API**（宿主必须 catch），但**引擎存活**（实测后续 getCellValue
  正常）；普通语法错误则不抛、进格为 `#ERROR!`（ParsingErrorVertex）。
  **结论：无显式公式深度/长度限额**，防护 = sheet 尺寸限额 + V8 栈深
  的意外兜底。【推断】这是其敌意输入面最大的真实缺口（对比：Univer
  同样无显式深度限额，属行业常态；引擎化时应加 parser 深度上限→
  `#ERROR!` 化）。

**巨量区间引用**：整列 A:A 是一等公民（实测求值正确 55 = sum(1..10)），
RangeVertex 复用 + bruteForce 策略控制边数（§3）；ColumnIndex 倒排
加速查找。区间上限受 maxRows/maxColumns 硬界（40k×18,278 ≈ 7.3 亿格
理论上限——但地址迭代是惰性生成器 `range.addresses()`，实测 SUM 跨
25k×20 全表 985ms 完成，未见物化爆炸）。**许可门本身也被当攻击面**：
entitlement key 载荷无尺寸上限，spread 攻击已被显式防掉（§5）。

**数值与类型卫生**：解释器出口统一后处理——溢出→`#NUM!`、负零修正、
1×1 数组降标量（`src/interpreter/Interpreter.ts:69-84`）；smartRounding
+ precisionEpsilon/precisionRounding 双精度策略（`src/Config.ts:70-72`，
3.5.0 的 VAR/STDEV 大均值小方差精度重写即为该面的回归
`CHANGELOG.md:28-31`）。错误值统一为 CellError 传值（非异常），错误链
带 origin vertex（`src/Evaluator.ts:108`）。

**无 eval/无远程**：函数实现全部是静态 TS 插件方法，无 eval/new
Function；grep fetch/xhr/websocket 于 src/ 零命中（实测）；超链接是
纯数据（getCellHyperlink 只回 URL 字符串，`src/HyperFormula.ts:927`，
不打开、不渲染——攻击面归零）。CodeQL 双 workflow（codeql.yml +
codeql-advanced.yml）+ `npm audit --omit=dev` 门（package.json
`check:licenses` 同列）。

## 8. 质量体系

**测试面（最重要的结构性发现）**：OSS 仓**只带 1 个 106 行冒烟测试**
`test/smoke.spec.ts`；完整套件（单测/兼容性/性能）在**私有仓
hyperformula-tests**，"available on request"（`test/README.md:5-9`），
CI 用 deploy token 拉取（`.github/workflows/test.yml:34-41`），外部
贡献者的测试会被内部团队"moved to the private repository"
（`DEV_DOCS.md` Definition of Done 节）。**即：函数级行为锁（423 函数
× 语言包 × 兼容矩阵）全部不在开源面**——这与 Univer（1819 spec 文件
全开源）形成两极对照。可核验的公开质量证据退化为：CHANGELOG 的逐条
Fix 记录（带 PR 号）+ CI 徽章。

**CI**：test.yml（Node 22 矩阵 + jest + coverage，私有套件）、
build.yml（**三 OS 矩阵** ubuntu/windows/macos 打包 + `bundle-all` +
verify-bundles 校验四种产物 es/cjs/umd/umd-full，`.github/workflows/
build.yml:24,39`）、lint.yml（ESLint）、performance.yml（PR 基准对比
base..head：base checkout :30 + benchmark:compare-benchmarks :62）、codeql 双跑、audit.yml、publish.yml、vendored-parser.yml
（许可钥读器漂移日检 cron，`'17 6 * * 1-5'`）。codecov 集成
（codecov.yml）。构建产物五形态（es/commonjs/typings/dist umd/
languages umd）全 verify。

**版本纪律（从 CHANGELOG 重建）**：2019 起持续演进，1.0.0 于 2021-07-15
（`CHANGELOG.md:363`）、2.0.0 2022-04-14（:285）、3.0.0 2025-01-14
（`CHANGELOG.md:108`）、3.5.0 2026-10-09（:10）——近 12 个月（自
2025-10-14 起）6 个 release（3.1.0@2025-10-14、3.1.1@2025-12-18、
3.2.0@2026-02-19、3.3.0@2026-05-20、3.4.0@2026-08-10、3.5.0@
2026-10-09；3.0.1@2025-08-11 在窗口外不计），节奏约 6-10 周一版，
Keep-a-Changelog + SemVer，每条目带 PR/issue 链接，迁移指南随大版本
（migration-from-0.6-to-1.0.md / 1.x-to-2.0.md）。HEAD 提交即 3.5.0
release merge（git log："Merge branch 'release/3.5.0' into master"）。

**工程治理文档是仓内最强项之一**：DEV_DOCS.md（Definition of Done
清单含"测试必须无控制流语句"等苛律 + 性能条款"performance is a
feature, not an afterthought"）、AGENTS.md + CLAUDE.md（AI 代理守则，
含"敏感信息永不入 commit"与失败案例累积器）、DOCS_CONTENT_GUIDE.md、
CONTRIBUTING.md 含 CLA（:3）。文档站由同一仓 VuePress 生成且函数页
从代码自动生成（`script/generate-builtin-functions-doc.ts:10-16`，
"single source of truth... cannot drift"）。

## 9. 净室合规

**双许可核验（一手源）**：LICENSE.txt 33 行——"dual-licensed, giving
you the option to use it under either a proprietary license or the GNU
General Public License version 3 (GPLv3). The specific license under
which you use the software is determined by **the license key you
apply**"（`LICENSE.txt:8-11`）。 GPLv3 通道 = 配置 `licenseKey: 'gpl-v3'`
（`docs/guide/license-key.md:16-28`）；商业通道 = 付费 key（含子集授权，
§5）。**许可选择由技术执行（key）与法律文本（LICENSE.txt）双锚定**。

**对"只读分析"无碍**：本轮全程只读源码 + 本地运行（gpl-v3 通道即合法
运行），零复制代码到分析仓/产品仓。GPLv3 的 copyleft 义务仅在**发行
衍生作品**时触发；净室路线（读行为、产出结论、不拷代码）无义务。

**「未来依赖」决策依据（净室分析要产出的核心产出）**：

| 依赖路径 | 法律面 | 工程面 | 结论 |
|---|---|---|---|
| **A. 直接依赖**（npm 包 + gpl-v3） | 产品若闭源分发 → GPL 传染；产品若开源（GPLv3 兼容）→ 可行 | 423 函数、依赖 2 个、可嵌入 Node/Rust FFI/wasm 边界 | 仅当 mdcore 全线 GPLv3 或不分发（SaaS 服务端不分发即无传染——GPLv3 §10 网络服务豁免于"convey"）时可行 |
| **B. 直接依赖**（商业 key） | 按席位/授权付费，子集授权计价（fun:*/feat:* token） | 同 A + 按函数计价的授权面 | 商业成本 + 供应链闭锁（质量底座在私有仓）风险 |
| **C. 净室自研**（推荐） | 无义务 | 以本报告 §3/§4/§5 为规格参照 | **行为对齐目标（423 函数、#CYCLE! 语义、RangeVertex 聚合缓存、双门许可的反面教训）已经完整提取** |

**内嵌第三方**：`src/parser/LICENSE.txt`（Chevrotain Apache-2.0）；
`src/interpreter/plugin/3rdparty/bessel/bessel.ts:1-6`（SheetJS bessel.js，
Apache-2.0，501 行）；`src/interpreter/plugin/3rdparty/jstat/jstat.ts:1-8`
（jStat，MIT，1003 行）——若走路径 A 需连带保留三份声明；路径 C 无涉。
npm 依赖许可门禁 `check:licenses` 白名单 MIT/Apache/BSD/ISC/Unlicense
（package.json scripts）。

**商标/品牌**：README 与文档有 branding.md 指南（名称使用规范），
Handsontable 商标不在任何许可授予内。

## 10. 能力矩阵（dsh 插件面 + MCP 工具面双口径）

| 能力 | 引擎仓内（实证） | 生态/roadmap（文档声明，均未交付） |
|------|----------------|----------------------------|
| 程序化读写 | buildFromArray/Sheets + setCellContents + ExportedChange[] 回执 | `createSpreadsheetTools(hf)`（docs/guide/ai-sdk.md:37,49） |
| 无头运行 | 纯 Node 同构，零 UI 依赖（README.md:9） | —— |
| 一次性求值 | `calculateFormula` fire-and-forget（HyperFormula.ts:4570） | `evaluate` MCP 工具（docs/guide/mcp-server.md:41） |
| 依赖可解释 | getCellPrecedents/getCellDependents（:3355,3393）——agent 解释"数从哪来"的原语 | "Trace dependencies... explain how every value was derived"（mcp-server.md:25） |
| MCP 工具面 | **——（仓内零 MCP 代码）** | `npx -y @hyperformula/mcp`，"cannot be installed or used today"（mcp-server.md:15-16,33） |
| dsh/插件面 | —— | hyperformula/ai（Vercel AI SDK）+ hyperformula/langchain，同款 coming soon（ai-sdk.md:16-17；integration-with-langchain.md:14-15） |
| 自定义函数 | FunctionPlugin + registerFunction（静态 :493,547）——可给 agent 注册领域函数 | —— |
| 函数目录 API | getAvailableFunctions/getFunctionDetails 按许可过滤（:4761,4807）——LLM 函数选择器的直接数据源 | —— |

**口径结论**：与 Univer 相反的镜像——Univer 是"编排在外、内核开源、
Pro 闭锁"；HyperFormula 是"**内核完全开源、外置编排一件都还没交付**"。
README 已把 "Deterministic compute layer for AI & LLMs" 列为第一用例
（`README.md:31`），MCP/AI SDK/LangChain 三件全部挂 early-access 表单
（`docs/guide/mcp-server.md:18,59`）。【推断】外置化的**内核前提**（纯进出
数据、依赖可解释、函数目录 API、变更回执）HyperFormula 已 100% 具备，
缺的只是协议包装层——**mdcore 若把 MCP 工具面做进内核仓，即抢在
HyperFormula 官方交付之前完成它自己的 roadmap**；且 mdcore 的表格引擎
可外置为同构"确定性计算服务"，与 Handsontable 商业形态正面对位。

## 11. 可移植结论（按优先级）

1. **「数据进结果出」边界照抄**：引擎公有面只认 `RawCellContent[][]`
   进、`CellValue[][]` + `ExportedChange[]` 出 + {sheet,col,row} 坐标
   三元组——MCP 工具 schema 可以逐字段对映此三件套，零渲染语义泄漏。
   mdcore 表格计算内核立项的第一条架构约束应写明：**计算内核不知道
   任何渲染/样式/富文本概念**（HyperFormula 证明此约束可持续 7 年）。
2. **RangeVertex 聚合缓存 + 前缀复用优先于空间索引**：SUM(A1:A1..A100)
   类嵌套区间先复用小区间节点再补尾差（DependencyGraph.ts:202-213）；
   criterion cache 按判别式键缓存（RangeVertex.ts:24）——Rust 实现为
   `RangeId -> HashMap<FnKey, Value>` + 环形失效。对照 Univer 的 RTree：
   先做聚合复用，空间索引只给"任意相交区间的脏区筛选"补位。
3. **惰性变换 + 版本号 + 阈值压实**：结构变更不重写公式，公式自带
   version，读取时应用增量变换（FormulaVertex.ts:60-67），队列到
   maxPendingLazyTransformations（默认 50）强制全量 apply + compact +
   undo oldData 清扫（HyperFormula.ts:5100-5107）——这是行列操作的
   O(1) 摊还方案，mdcore 的 op-transform 通道可直接借用此"版本号 +
   惰性 + 压实"三件套。
4. **"值不变不传播"谓词拓扑**：Tarjan postprocess 的
   operatingFunction 返回"值是否变化"决定下游是否继续标脏
   （TopSort.ts:180-183）——增量重算的传播准线，比"全下游遍历"省
   一个数量级（实测链 10k 单点 41ms vs 构建 137ms）。
5. **环 = 错误值而非迭代**：`#CYCLE!` 直接落格（Evaluator.ts:102-112）
   + 解析期依赖收集（代价是假环也报错，官方显式登记）——对 mdcore：
   拒绝环是更 AI-friendly 的语义（确定性），但应把"假环"当作已知
   不兼容登记。
6. **双门许可的反面教材**：Gate A 降级运行（#LIC! + 抛错）+ Gate B
   粒度授权 + "释放阀不设门"原则（HyperFormula.ts:5002-5031 的整段
   论证值得逐句读）——若 mdcore 未来有商业版，**照抄"读不门控、写
   门控、清理不门控"三分类**，但公开仓应保持单许可（Apache/MIT）
   以避免"引擎开放、质量闭锁"的社区代价（见 8）。
7. **函数目录即 API**：canonical id ↔ 本地化名分离、"未翻译即不可
   调用"、getAvailableFunctions 按授权/翻译过滤——LLM 的函数选择器
   数据源与人类 UI 同源。mdcore 的 MCP 函数发现工具应复用此设计。
8. **isItPossibleTo* 谓词层**：每个写方法配"不会抛"谓词（含许可/
   边界/类型全检查）——agent 的 pre-flight check 原语，等价于 dry-run。
9. **变更回执 ExportedChange[]**：一切写操作返回结构化 diff——宿主/
   agent 的观测点统一在回执 + valuesUpdated 事件，引擎不回调宿主。
10. **已知缺口要前置**：公式深度无显式限额（RangeError 逃逸公有 API，
    §7 实测）——自研时应加 parser 深度上限并 `#ERROR!` 化；AST 缓存
    无淘汰（Cache.ts:26）——Rust 侧用 LRU 或世代回收。
11. **性能观测一等公民**：StatType 17 段计时内建 + getStats 出口
    （含缓存命中计数）——引擎化产品的可观测性应从第一天内建。

### 与工作台架构的对照（一页）

| 决策点 | HyperFormula 选择 | mdcore 现状/规划 | 取舍建议 |
|--------|-----------------|-----------------|---------|
| 定位 | gridless 计算内核（无头、无样式、无格式化） | mdshell 文档内核 + 表格引擎未立项 | 表格计算内核按此形态立项（轻内核+重外置） |
| 模型 | 图即模型（Vertex 挂缓存值） | op-first（apply_ops） | 计算内核可独立于 op 通道：op 生成引擎变更，回执生成逆 op |
| 增量重算 | 脏种子 + Tarjan 子图 + 值不变不传播 | 无（MD 线不需要） | 表格页立项时整套借鉴（§3） |
| 结构变更 | 惰性变换 + 版本号 + 压实 | history/revision | 版本号三件套可移植到 op transform |
| undo | 逆操作栈 + 不可逆变换存 pre-image | 树结构逆 op（已有） | "不可逆才存数据、可逆只存 op"的分级与 mdcore 现状同构 |
| 函数面 | 423 函数 + 18 语言 + 插件注册 | 无 | 若做：canonical id/本地化分离 + 目录 API 先行 |
| 许可 | GPLv3+商业双许可、key 技术执行 | 单许可（干净） | 保持单许可；双门设计仅作未来商业版参照 |
| AI/MCP 面 | 内核就绪、外置全 roadmap | mdshell host-only 面 | **窗口期**：MCP 计算内核面进 mdcore 内核仓 = 抢跑 |
| 质量 | 私有测试仓（行为锁闭源） | 259 语义锁全开源 | 坚持开源锁——对 AI-native 定位是信任资产 |

## +4 缺位登记（组件边界的另一面）

1. **渲染维：整维缺位**（非"未实现"而是"从未设计"）——无样式/边框/
   列宽/冻结/合并格模型，无任何渲染回调。配图仅文档用
   （examples/ 的 png 与 topshot.png 是架构说明图）。
2. **协同维：整维缺位**——无 OT/CRDT/多人概念，无网络层；变更回执
   + 事件是其为宿主协同系统预留的唯一接缝（8 事件里无任何 presence/
   冲突语义）。
3. **持久化/格式进出：缺位**——"HyperFormula has no built-in file
   import functionality"（`docs/guide/file-import.md:8`），xlsx/csv 解析
   交给 ExcelJS/SheetJS/PapaParse 三方、结果作为 JS 数组喂入（:20-25）；
   无自有文件格式（序列化=输入形态回放，§2）。保真责任完全在宿主。
4. **编辑内核：半缺位**——无文本编辑/光标/输入法概念（无富文本单元格），
   undo/redo 是结构级的（20 个 BaseUndoEntry 子类覆盖单元格内容/行列/表/
   命名表达式/移动，`src/UndoRedo.ts:38-80`），不是文本级的。剪贴板是引擎内抽象
   （copy/cut/paste 操作 ClipboardOperations），与系统剪贴板无关。

## 附：证据锚点密度

单代理深读合计 **130 处 `文件:行号` 锚点**（其中 93 处为完整
`路径:行号` 反引号引用、其余为紧邻上文文件名的裸 `:行号` 引用），
覆盖 72 个不同源文件/文档页，横跨 11+4 维全部章节。实测面共 5 类：
函数计数（423）、license 门控行为（空 key/gpl-v3 双路）、性能探针
（5 场景）、敌意输入探针（3 族）、RangeError 栈顶取证。**最小复现
命令**（worklog r3-office-c 仅记方法概要，复跑以下列命令为准）：
`npm install` → `npx tsc`（默认产物 → lib/，es2015）→
`BABEL_ENV=commonjs npx babel lib --out-dir <scratch>` → 以
`<scratch>/commonjs/index.js` 为入口、`NODE_PATH=<仓>/node_modules`
驱动引擎——**入口必须用 index.js**（直接 require HyperFormula.js 会因
Cell↔plugin 循环引用在 LookupPlugin 顶层求值崩溃；复审 r3-review-c
独立复跑时实证的关键细节）。锚点重镇：HyperFormula.ts
19、DependencyGraph 家族 21、license 9、Config 7、Evaluator 5、
parser 4、docs/guide 13、根文件 12、.github/workflows 6。
【推断】标注 9 处（计数法：正文显式【推断】标签逐处计）；实测标注
~23 处（计数法：「实测」字样的断言/数据点逐处计，全文「实测」字样
含方法叙述共 28 处）。
