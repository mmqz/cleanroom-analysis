# 裁决包三分类（DECISIONS）——全线剩余门控项的决策就绪视图

> **性质**：用户加固指令 D 组执行轮（2026-10-17）——「Convert each
> remaining item into one of: (1) Safe implementation that does not
> change product semantics. (2) Concrete proposal with trade-offs and
> tests. (3) Explicit blocker with the exact decision needed. Implement
> category 1 items.」本文 = 全线（MD/office/collab/PDF/规划）剩余项的
> **单一裁决登记表**：每项给出分类、状态、证据锚点与**精确决策问题**。
> 纪律：不为类别 3 发明决策；类别 2 只收**已完成提案**（本轮新增两件）
> 不做再调查；类别 1 本轮实施并在档。
> **输入**：HANDOFF §6 + R1-R6逐项处理.md + t22-review 前置清单 +
> yrs-c/yrs-d + office-f/office-g + r-o2-t3 REGRESSION + 06 §8.1。

## 0. 分类总表

| # | 项 | 线 | 类别 | 精确决策问题（类别 3）/ 提案锚点（类别 2） |
|---|---|---|---|---|
| 1 | R-1 undo_coalesce_ms 接线 | MD | **3** | A 接线（默认零变化使能）/ B 正式保留键声明——REUSE.md §R5 显式裁决门 |
| 2 | R-2 本地域撞号面对策 | MD | **3** | a register 守卫 / b ID 持久化进 meta / c 维持现状（协同域已由 R4.4 消解） |
| 3 | R-3 undo 栈上限（×B3.4 协同） | MD+collab | **3** | 上限值+逐出策略 / 维持无限（**新输入**：p12 实测 redo 返回栈项——历史越长栈越深即使已消耗） |
| 4 | D1 实施轮三裁决 | collab | **3** | ①载体选型（p13 七面深验全过，证据链完整）②B3.2 撤销栈会话本地语义知情确认 ③B3.4×R-3 三选项 |
| 5 | D2 双态立项 | 规划 | **3** | 立项与否及范围（06 §3） |
| 6 | D3 生成态主线 | 规划 | **3** | A 主 + B 辅触发条件（06 §5.2） |
| 7 | R-O2 Q3 实施形态 | office | **3** | LO 分发/版本基线/子进程治理参数（**新输入**：r-o2-t3 检出策略通则 + rc=0 两条新发现） |
| 8 | 锚点写面 Phase 1/2 触发 | office | **3** | 现在扩 / 与 MarkSpan URL 裁决合并 / Phase 2 三选一（悬空合法形）——**类别 2 提案已完备**（#13） |
| 9 | ① code lang/fence | office | **3** | 接受永久降级（docx-rs 无语法高亮/元数据面——库层不可达）/ 立项带外携带方案（发明表示 = 产品语义裁决） |
| 10 | ② link URL（MarkSpan） | MD | **3** | MarkSpan 是否增 URL/锚点载荷（mdcore 公开模型变更；office-io 侧就绪度见 office-g §12 Option B） |
| 11 | ④ 自定义编号格式暴露 | office | **3** | 消费方出现时触发增量轮（引擎能力在、包装面按需——既有口径维持，无新决策需求） |
| 12 | ⑦ 富文本 marks | MD | **3** | BlockTree 是否增 marks 载荷（**新证据**：p13 V2 证明 XmlFragment 原生行内混排——模型裁决落地日载体即绪） |
| 13 | 锚点写面完整设计 | office | **2** | **office-g §7-§13**（本轮 anchor-t2 补全：调用示例/身份模型/生命周期/迁移/校验 V1-V8/备选与最小性——可直接评审） |
| 14 | D1 载体证据链 | collab | **2** | **yrs-d §5**（本轮 p13 七面深验——序列化/混排/稳定 ID/删除/复原/同步/深嵌 wire 全过；完整映射矩阵仍属实施轮设计文档） |
| 15 | R-5 MD 基准 B-1..B-10 | MD | **3** | 建基准轮时机（前置：统一 profile 口径 + 消费者性能要求——两者均未变） |
| 16 | R-6/R24-d 真机壳实体 | MD | **3** | 开发环境轮排期（本沙箱非目标环境） |
| 17 | fuzz 战役排期 | PDF | **3** | SECURITY_POLICY S6 表战役启动时机 |
| 18 | LO 26.8.1 上游复跑 | office | **3** | 上游版本环境就绪时复跑 r-o2-t3 套件（版本差核对前置——钉定破裂=版本差信号） |
| 19 | 上游协同（yrs） | collab | **3** | #676 UAF 修复跟踪（open/0 修复维持——不升生产默认内核的约束维持）+ **新增两项 API 请求候选**（yrs-d §2.2：Update 内容类枚举 / ClientID release 校验）。**跟踪增量（2026-10-25 maint-r1）**：上游仓库转移 y-c-rs/yrs → **y-crdt/y-crdt**（旧路径 HTTP 404 实测，crates.io repository 字段直证；#676 在新家保号在场仍 OPEN）；**修复 PR #682 在途**（OPEN 未合并，作者 jonliu03，「GC: collect a collected type's children even when undo keeps them」——正是 d1b-t1 ASan 定位根因面）；crates.io 最新仍 0.28.0（= spike 钉版，无新 release）——skip_gc 全编队不变量维持 |
| 20 | STATUS §7-6 陈旧清单 | MD | **1** | **本轮已实施**（§1.1——e-review 勘误：初稿误引 §2） |

## 1. 类别 1 清单与实施结果（安全实现——零产品语义变化）

**全量扫描结论**：跨五线枚举后，类别 1 恰一项——文档与现实的一致性
修正（本项即用户停机条件「文档与实际代码一致」的直接兑现）：

### 1.1 STATUS.md §7-6 PDF-f+ 遗留项陈旧修正（已实施）

| 子项 | STATUS 旧文 | 实际状态（本轮亲核证据） | 处置 |
|---|---|---|---|
| render_watchdog_ms 生产消费点 | 列为遗留 | **已消费**：`crates/pdf/src/worker.rs:16-17`（watchdog 工厂 + `render_watchdog_ms` 超时监控线程）+ `security.rs` 在场 | 旧文删除 |
| 看门狗收报面登记（T2-verify2） | 列为遗留 | **已登记**：worker.rs:24/197/255/269/293（三收报形态 + 死亡可观测不悬挂语义，T2-fix 闭环） | 旧文删除 |
| README 双引擎面口径补全 | 列为遗留 | **已补全**：README:17-18（双引擎描述）+ :46-48（feature 门禁命令） | 旧文删除 |
| fuzz 战役排期（S6） | 列为遗留 | **真遗留**（排期决策，类别 3 #17） | 保留并转 DECISIONS #17 |

其余候选逐一排除（核查过程在档）：yrs-c「ASan 常跑项」遗留 → 本轮
d1d-t1 已做（yrs-d §0）；office-g「待核（实施批）」两件 → 属 Phase 1/2
实施批（触发前不可做）；R-O2 mdc-pdf 腿 → Q3 门内；收报面/README →
上表已证完成。**类别 1 清空**。

## 2. 类别 2 已完成提案（本轮新增/补全，不做再调查）

| 提案 | 完备性 | 等待 |
|---|---|---|
| **锚点写面 Phase 1/2**（office-g §7-§13） | 六维度全覆盖：API diff + 调用示例（对照 mdcore 实际 API 校正）/ 三层身份模型 / 生命周期五行 / 迁移零破坏链 / 校验 V1-V8 / 四备选最小性论证 + 测试计划 T1-T7+V 面 + 门禁预期 | 用户裁决 Phase 触发（#8） |
| **D1 载体证据链**（yrs-d §5 + p13） | 七面深验全过（含 ASan）；XmlFragment 满足 mdcore BlockTree 协同载体全部受测面；建议倾向维持 | D1 裁决①（#4）；完整映射矩阵 = 实施轮设计文档 |

既有完整提案维持（不再重述）：Q-R1 接线形状（锚点全定位，实施轮直用）
/ Q-R2 三选项 / Q-R3×B3.4 三选项（yrs-c §1.2 + p12 新输入）——
均达「裁决即可实施」完备度。

## 3. 类别 3 精确决策包（不发明决策，只呈问题）

每项的最小决策输入已在总表；三组「单次裁决双线生效」联动复登：

1. **R-3 × B3.4**（#3/#4③）：同一栈上限产品语义的本地/协同镜像——
   单一裁决点双域生效（p12 新输入：redo 返回栈项使无界增长比此前
   认知更激进）。
2. **R-1 × B3.2**（#1/#4②）：接线若裁 A，适配层捕获窗应同源镜像
   （同一设置键驱动两域）。
3. **R-2 × R4.4**（#2）：协同域撞号已消解，R-2 只剩本地域价值。

## 4. 已核带源清单

- STATUS §7-6 陈旧性：worker.rs:16-17/24/197/255-293 + security.rs
  + README:17-18/46-48（mdcore main=00ad88b 亲核，2026-10-17）；
- ①④⑦⑨ 精确口径：docx-r1 REPORT.md:559/564/652/657 + FIDELITY.md:14
  （降级/未暴露登记原文）；
- 全部门控项来源：HANDOFF §6（2026-10-17 六轮后速览）+ R1-R6逐项
  处理.md §0/§2 + t22-review 前置清单（worklog 2684 行）+ 06 §8.1；
- 本轮新增证据：yrs-d 全卷（p12/p13/p14）+ r-o2-t3 REGRESSION +
  office-g §7-§13。
