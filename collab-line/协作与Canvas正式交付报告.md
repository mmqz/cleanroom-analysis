# 协作与 Canvas 正式交付报告（collab-r1 / canvas-r1 / e2e-r1）

> 用户指令（2026-10-21）：协作引擎与 Canvas 转正式交付目标，不再列为
> 等待需求触发的延期项目。本报告区分**已实现 / 已验证 / 仍待集成 /
> 实际风险**四类（指令终段要求），并登记全部实施发现。

## 0. 交付物清单

| 交付物 | 位置 | 提交 |
|---|---|---|
| 协作引擎 mdc-collab（文档域） | mdcore crates/collab（第 11 成员） | b3ec4da |
| Canvas 引擎 mdc-canvas（画布域） | mdcore crates/canvas（第 12 成员） | 3289d41 |
| 证据契约扩展（Canvas 引擎/定位器） | mdcore crates/evidence（加性） | 3289d41 |
| 端到端可运行演示 | mdcore crates/canvas/examples/e2e_demo.rs | f91b6be |
| 合规 NOTICES v1.6（yrs 闭包入账 + 补账） | mdcore compliance-baseline 分支 | 53b8438 |
| 收尾验证（sidecar 持久化/异常恢复） | mdcore crates/workbench | 1db4a6c |

产品线现况：mdcore 12 crate，G1 **642** 零失败（本会话 606→642：
collab 23 + canvas 13；收尾轮 +6），零新三方依赖超 yrs 闭包
（NOTICES v1.6 在账）。

## 1. 已实现（代码在产品仓 main）

### 1.1 mdc-collab —— 文档域协作引擎

- **隔离纪律**：产品面 yrs 接触点收敛于 crate 边界（公开 API 零
  yrs 类型；撤 yrs 只拆本 crate，12 成员中其余 11 零波及）。
- **载体**：XmlFragment 根（yrs-e 映射矩阵定谳形状）——非根 7
  kind 元素承载 + 开放属性编码（非法 XML 名 `_xHHHH_` 转义 + 字面
  形下划线前瞻转义，往返零丢测试钉定）+ mark 包裹元素（D-2 非重叠
  最小区间等价覆盖，归一化重建策略）+ char↔byte 单点换算（D-3，
  公开面恒 char 域）。
- **命令接口**（人类/AI 同一入口）：`CollabCommand` 九命令
  （装载/插入[id 物化面]/删除/文本替换/种类变更[D-1]/属性终态/
  mark/搬移[环回拒]）——每命令一事务一 origin。
- **撤销/重做**：双栈 origin 分栈 + 捕获窗 0（p20 缓解）+ 软上限
  粗粒度策略（clear_all 显式回执）。
- **同步/恢复**：state_vector / encode_diff / apply_update（限额
  + panic 隔离）/ authoritative_snapshot / restore / maintenance_gc。
- **身份双轨**：block_id（属性承载，远端免侧车寻址）× CrdtAnchor
  （sticky 首项 ID；邻居编辑稳定；删除显式悬空；D-1/MoveBlock 重建
  后锚点表自动重指；MDCC v1 侧车）。

### 1.2 mdc-canvas —— 画布域协作引擎

- **独立领域模型**：CanvasObject{Node/Edge/Group} × 几何 × 端点 ×
  成员 × 开放属性——图形状自有模型，与文档域并列各自 Doc（不建跨
  引擎统一 CRDT）。
- **载体**：根 Map（对象 ID = Map 键 → 删除撤销复原 = **同 ID 回归**，
  画布域身份特性）；属性写入 = 键级 LWW。
- **命令接口**（同构模式）：`CanvasCommand` 八命令 + Actor 分栈。
- **证据关联**：对象语义指纹（几何/属性/端点/成员——位置漂移即
  语义变更）+ 全画布指纹；依赖 mdc-evidence 契约层。

### 1.3 mdc-evidence 加性扩展

- `EngineKind::Canvas` + `Locator::CanvasObject{object_id,
  object_fingerprint}`（v1 sidecar 数据 serde 兼容——加性变体）。

### 1.4 端到端演示

- `cargo run -p mdc-canvas --example e2e_demo`：五幕 12 检查点，
  断言内嵌（失守非零退出 = 可自动化验证），分幕叙事输出（可演示）。

## 2. 已验证（测试钉定，全部亲跑）

| 面 | 验证内容 | 测试 |
|---|---|---|
| 文档操作矩阵 | 九 kind 装载读回/容器子块/char 域跨 mark/重叠 mark 归一/属性转义往返/D-1+搬移环回/锚点身份 | matrix 7 |
| 并发收敛 | 双端全量+增量/并发删+并发插/三方链式/撤销跨端只能撤自己 | concurrency 4 |
| 撤销/重做 | 七操作类型全环/p20 交错回归锁（上游回归或配置回退必失败强制解钉）/软上限回执/删除撤销复活身份 | undo_redo 4 |
| 故障恢复 | 快照恢复续编+互同步/快照差量互通/维护压缩/敌意 wire fail-closed 零污染 | recovery 4 |
| 敌意输入 | 构造域/区间块缺失保留键/字节限额四面/侧车敌意载荷 | hostile 4 |
| 画布操作矩阵 | 生命周期+读模型确定性序/边分组语义四面拒收/语义指纹对象级隔离/敌意输入 | matrix 4 |
| 画布撤销 | 全操作环/删除撤销同 ID+指纹同一/交错分栈+pop-until-productive 钉定/软上限 | undo_redo 4 |
| 画布并发 | 人类拖动+AI 新增双向收敛/并发同键 LWW 收敛/快照恢复/坏 wire+维护压缩 | concurrency 4 |
| 证据关联画布 | 登记→拖动 Stale→重验 Valid→AI 端并发改写再失效→AI 撤销→重验复原→三态正交 | evidence_assoc 1 |
| 端到端全链 | 五幕 12 检查点（见 §1.4） | e2e_demo |
| sidecar 持久化 | 重开保真/跨会话幂等重放/版本单调/四异常 fail-closed/中断发布残留/会话间改写 BodyStale | persistence_recovery 6（收尾轮） |

门禁：G1 642 零失败 / clippy native+wasm32 --all-targets 恰 2
static_method_of 基线 / fmt 0 / scan 0 hit-paths / wasm32 check
--all-targets 零错误。

## 3. 仍待集成（诚实边界——已实现但不在此轮闭环内）

> **状态更新（2026-10-22）**：项① 已于后续轮落地并推送
> （mdcore `1789493` integ-r1：第 13 crate **mdc-mdbridge**——
> 身份映射三面定谳[family/foreign/孤儿]、plan→shadow 增量→collab
> 命令核心执行序、真相源纪律[CollabSession 唯一、shadow 纯投影]、
> dual_run 双跑锁翻译语义钉定、25 新测试；补丁 `d3bbb12` 清测试面
> clippy 漂移）。**项③ 已落地**（mdcore `02067ad` integ-r2：双域
> 事件层——一公共入口一事件/updates 逐事务可转发/失败不产事件/
> canvas touched 对象 ID；21 新测试，G1 667→688）。**项④ 已落地**
> （mdcore `e2dfc7c` integ-r3：presence 模块——在线状态/光标
> JSON 在场层，独立 wire 通道；null 离线语义/陈旧 clock 取舍/
> 限额+隔离/standalone 画布宿主路径；11 新测试，G1 688→699）。**项②
> 已落地**（mdcore `80e097a` integ-r4：协同文档作为受证正文——应用层
> 组合[会话=唯一编辑面/文件=持久化投影]；3 组合测试+五幕 12 检查点
> 可运行演示，G1 699→702）。**项⑥ 已落地**（mdcore `61f1ea8`
> integ-r5：第 14 crate **mdc-relay**——传输无关中继状态机/
> SyncReplica 门面双域接入/权威副本扇出/差量追平/在场聚合/
> 快照检查点与恢复；9 测试，G1 702→711）。
> **剩余待集成 = 项⑤ 前端（用户裁决项——未选型不得指定框架，
> 维持门控）。工程侧集成清单全部收口。**

1. ~~**mdcore（MD 引擎）↔ mdc-collab 的 op 层对接**~~ **已落地**
   （integ-r1 `1789493` + 卫生补丁 `d3bbb12`，2026-10-22 复验
   G1 667 零失败 / clippy 恰 2 基线 / fmt 0 / scan PASS / wasm32
   check 零错误）。身份映射裁决项已定谳：本端 shadow u64 ↔
   `md:{client_id}:{n}` 确定性编解码 + 外端持久双向映射表 + 同族
   伪造水位纪律；publish = 重映射语义（fresh family u64）。
2. ~~**workbench 八步闭环 ↔ 协同会话组合**~~ **已落地**（integ-r4
   `80e097a`：协同文档作为受证正文——应用层组合零 workbench 改动；
   选择性失效[润色段 Stale/相邻 Valid]/重绑恢复/撤销回退/双端并发
   收敛/干净导出全链钉定；3 测试 + 五幕 12 检查点可运行演示
   `cargo run -p mdc-workbench --example collab_body_demo`）。
3. ~~**事件/订阅面**：前端宿主需要变更通知（推式）~~ **已落地**
   （integ-r2 `02067ad`：mdc-collab/mdc-canvas events 模块——
   延迟派发/一入口一事件/updates 逐事务可转发/失败不产事件；
   canvas 域 touched 对象 ID 双路径提取；21 新测试）。拉取式读
   模型与推式事件现已双面在位。
4. ~~**awareness（在线状态/光标）**~~ **已落地**（integ-r3
   `e2dfc7c`：mdc-collab presence 模块——基础设施单点归属[画布
   宿主经 Presence::standalone 消费不镜像]；上游发现：yrs
   update() 排除 null 条目 → 离线广播改走 update_with_clients
   全键集；11 新测试）。
5. **前端**：未选型（指令⑤纪律——Headless 优先，未选定不得指定
   框架；宿主经 wire 字节 + 命令接口接入）。
6. ~~**服务端中继**~~ **已落地**（integ-r5 `61f1ea8`：第 14 crate
   mdc-relay——传输无关中继状态机[宿主桥接传输]；SyncReplica
   门面使文档/画布双域同构接入；权威副本 fail-closed 扇出 +
   差量追平 + 在场聚合[组合⑥×④] + 快照检查点/崩溃恢复；
   事件驱动 harness[组合⑥×③] 9 测试钉定。当前 P2P 直连形态
   升级为可常驻中枢形态。

## 4. 实际风险（如实登记）

1. **yrs 0.28 上游缺陷族**（全部有缓解，钉版 0.28.0）：
   - #676 heap UAF（undo + 并发 GC 面）——skip_gc 全编队不变量
     （B1.1，上游未修复期间结构性规避；代价 = wire 墓碑不压缩直至
     维护窗）。
   - p20 捕获窗缺陷（双栈 + 窗>0 + 同窗非连续合并 → undo 部分应用
     + redo panic）——窗 0 硬编码缓解 + 交错回归锁强制解钉。
   - **map-undo 交错语义（本轮新发现）**：同会话同键交错写撤销 =
     逐笔回退（撤第 k 笔恢复该历史点值）+ pop-until-productive
     （undo 弹到有产出为止，一次 undo 可消耗多个栈项）——跨会话
     编辑撤销不受影响；探针钉定 + 测试锁 + 上游询问候选。
2. **撤销耦合防护依赖写入方约定**：文本承载块"出生即带空文本
   节点"（否则 undo 首笔 = 删整个节点含后续并发内容——探针实证）。
   本 crate 是唯一写入方时不变量成立；外部直写 wire 的旧数据无此
   保证（集成时需校验或迁移）。
3. **D-1/SetMark 重建的文本身份换新**：同块并发文本编辑与
   SetBlockType/SetMark 的叠加——收敛性恒成立（CRDT），但重叠区
   可能被归一化终态取代（字面叠加语义注册）；块级锚点不受影响。
4. **资源面**：撤销栈默认无界（p19 定量 437.9B/op 最坏粒度）——
   软上限策略在位但默认 None；产品部署建议显式设限。单事务批量
   装载的 commit 期内存二次放大（yrs-a 实测）已由 512/事务小批
   路径缓解。
5. **依赖面**：yrs 0.28 闭包 12 面 crate（全 MIT/Apache 双许可零
   copyleft，NOTICES v1.6 在账）；hashbrown 双版本并存（0.14.5/
   0.17.1）；供应链钉版 + 升级首跑回归测试纪律。
6. **合规补账缺口（已修复但登记成因）**：v1.5 后 t19-t22/r4/r5/
   S1/S2 期间 NOTICES 未随轮复跑——面漂移 13 项本轮吸收；纪律
   已恢复（lock/feature 变化轮次 NOTICES 增量随批）。

## 5. 实施发现汇总（本轮新知识）

1. **撤销耦合防护**（§4.2）——空文本节点出生约定，探针三态实证。
2. **yrs 读写事务同锁**：`commit(&mut self)` 不消费事务，借用随
   Drop 释放——持有读事务（含变量遮蔽）再开写事务 = 死锁。两 crate
   全部 op 事务显式作用域化（p18 纪律的读侧扩展）。
3. **UndoManager tracked_origins len==1 回退**：单 origin 集合会
   捕获 None-origin 事务（Yjs 编辑器约定）——远端 update 应用会进
   本地撤销栈（spike p12"对端墓碑 2%"观察的根因）；产品收口 =
   capture_transaction 拒 None-origin。
4. **map-undo 交错语义**（§4.1）——逐笔回退 + pop-until-productive。
5. **画布域身份优势**：Map 键身份稳定 → 删除撤销同 ID 回归 + 指纹
   同一（证据锚连续性优于文档域的新 ID 族 + 重绑通道）。

## 6. 结论

用户指令的执行序列 ①收尾 → ②协作引擎 → ③Canvas → ④端到端验证
全部完成：**已实现** = 双域协作引擎 + 证据契约扩展 + 可运行演示；
**已验证** = 42 项自动化测试 + 12 演示检查点 + 全门禁绿；**仍待
集成** = mdcore op 层对接 / workbench 组合 / 事件面 / awareness /
前端 / 中继（六项，全部有明确边界与前置条件）；**实际风险** =
yrs 上游缺陷族（三面全缓解）+ 写入方约定 + 重建语义 + 资源默认值
（四面，全部登记且有对策）。按指令⑦：每阶段测试/审查/提交/推送/
交接完整；未遇需停下事项——常规实现细节自主决定，未降级未砍范围。
