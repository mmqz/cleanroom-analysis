# MD 复用地基实施轮计划（reuse-impl PLAN v1）

> 定位：**开发轮**（非研究轮）。依据 = boundary-audit 八件套（2026-10-12，
> REPORT.md 163 行）+ 用户六点指令（2026-10-12：审计收口进入开发；不重复
> 统一架构讨论；三域独立；只抽取明确共性基础能力；Office/跨引擎统一后置；
> 阻断缺口先处理；先交任务清单后连续执行）。
> 本轮代码写面 = mdcore 工作区（进入开发态，循其 R 轮纪律：门禁双绿 +
> CONTRACT 升版纪律 + 每提交 scan）；文档写面 = 本目录。
> 执行链 t8；环境重建（平台回滚事故恢复）记录见 worklog。

## 0. 范围裁决转录（不再讨论）

- **做**：阻断缺口修复 + settings 基础设施独立 crate 化 + 复用工具包文档 +
  全门禁复跑。全部动作有审计锚点背书，零新架构研究。
- **不做**（用户指令 2/3/4 + 审计留裁决项）：通用引擎 / 统一 CRDT / 双向
  投影 / D1-D3 立项 / A1-A3 消费者侧适配（无消费者）/ session 泛化
  （类型 MD 绑定，审计 A4 原文）/ Office 引擎（按需另轮）/ 增量保存接线
  （格式层就绪零消费者，P2 §5）/ undo_coalesce_ms 行为接线（产品语义留裁决）。

## 1. 质量缺口分诊表（指令 5：开发前检查）

输入：REPORT §6 五组 + §7 U-2..U-5。判级标准 = 是否阻塞本轮开发 / 影响
安全、数据正确性、PDF 质量。

| # | 缺口（审计锚点） | 判级 | 处置 |
|---|-----------------|------|------|
| 1 | Source 镜像锁**单向**（P6 §4.1-D2：mdcore 加第六值静默漂移，pdf 侧反序列化失败不可见） | **数据正确性级**（溯源面跨域静默漂移）→ 先修 | **T2a 双向化** |
| 2 | wasm source 静默兜底 `_ => View`（P3 G3/U2；mdcore-wasm lib.rs:49） | **数据正确性级**（非法串错标为人手 View = 溯源污染；实测 web/main.ts 仅 :846/:1027 两调用点全 'view'，零破坏面）→ 先修 | **T2c 校验化** |
| 3 | P3 测试锁缺口 G1/G4/G6（G2=设计事实非缺陷；G5=资源策略留产品裁决） | 复用地基回归网（test-face only，不阻断但必须随包出厂）→ 先修 | **T2b 补四条** |
| 4 | P5 契约漂移 F-1/F-2（文档宽于实现且门禁不可见；§3:62 版本残留） | 复用包出厂卫生（后续消费者第一个读的就是契约面）→ 先修；F-3 顺手；F-4 登记（锁"无多余成员"脆性大） | **T2d** |
| 5 | undo_coalesce_ms 死设置（P3 §4②：存储链齐备零消费者，should_coalesce 直读常量） | **不阻断**（行为一致且被测试锁；接线 = History 配置入口 = 核行为变更，属产品语义决策） | **登记**：选项 A 接线（History 配置入口 + shell 装配 + 用例）/ 选项 B schema 声明保留键；成本与语义影响留用户裁决 |
| 6 | MD 性能基准空位 B-1..B-10（P6 §5） | 不阻断（零性能宣称原则下无消费者要求） | **登记**（基准化需统一 profile 口径，另轮） |
| 7 | U-5 真机壳实体（R24-d 开发环境轮） | 本环境无实体 | **登记**（既有排期不变） |

## 2. 任务清单（指令 6：模块 / 复用依据 / 风险 / 验收）

### T0 环境与改前基线
- 模块：无代码改动。内容：工具链重建（已毕：rustc 1.99.0 + wasm32 +
  rustfmt/clippy）；pdfium 取件（cleanroom `md-line/pdf-a/spike/
  fetch-pdfium.sh`，chromium/8086 non-V8）+ fixtures 现场生成（spike 双
  探针）；web 依赖（npm install + wasm-bindgen 0.2.129 精确版 + 构建）。
- 验收：九门禁（G1-G9，命令照 baseline.md §1.2）+ web 三门禁（契约门/
  smoke/IME CDP）**改前全绿**，数字与 B11/U-1 补验口径一致（259/78/
  80+6i/108+6i/perf6/scan0/wasm32/clippy2基线/fmt0 + 16/15/28）。

### T2a Source 镜像锁双向化（分诊 #1）
- 模块：`crates/pdf/tests/pdfb_skeleton.rs` 新增兄弟测试（既有
  source_mirrors_mdcore_five_values :1040-1079 原样保留）。
- 内容：①mdcore 侧**恰五变体**计数锁（解析枚举体变体声明行数 == 5，
  抓"新增第六值"——既有锁只查在场不查多余）；②pdf 侧编译期穷尽 match
  锁（无通配 match 全五臂——pdf 加第六变体即编译失败）。
- 复用依据：P6 §4.1-D1/D2 + REPORT §6.1。
- 风险：文件文本解析脆性 → 红绿验证（临时注入第六变体证红，撤销证绿）。
- 验收：双向红绿演示在档；lopdf/dual/默认面全绿；mdc-pdf lib 面零改动。

### T2b P3 测试锁四条（分诊 #3）
- 模块：`crates/mdcore/tests/` 新增测试文件（命名循 r 前缀惯例）。
- 内容：G1 push_entry 清 redo 专属用例；G4a 结构性编辑后 save→open ID
  漂移 characterization（P2 §3 路 A【推断】→【测试验证】）；G4b 重开
  redo 物化 ID 撞号边界（P2 §3 连带边界）；G6 coalesced 事件 revision=
  组首消费契约锁（P3 §5）。
- 风险：characterization 固化的是漂移现状 → 测试名/注释明示"现状锁非
  语义承诺"，防未来误当规格。
- 验收：默认面全绿（259+N）；四条断言与审计锚点一一对应。

### T2c wasm source 输入校验（分诊 #2）
- 模块：`crates/mdcore-wasm/src/lib.rs:44-50`。
- 内容：五合法值显式映射（补 "view" 显式臂）+ 非法值 typed reject
  （Err，不再兜底 View）。
- 复用依据：P3 G3/U2 + REPORT §6.3；main.ts 调用面已核（仅 'view'）。
- 风险：外部未知消费者非法串从静默变显式拒绝 = 修复目的本身；三门禁
  复跑兜底；CONTRACT §3 合法值集合不变（无契约变更）。
- 验收：新增单测（非法串拒绝 + 五合法值映射）+ wasm32 check 过 +
  web 三门禁全绿。

### T2d 契约面卫生 F-1/F-2/F-3（分诊 #4）
- 模块：`web/tests/contract_gate.py`（+3 检查：__wasmMemory/__heapTrace
  渐进在场+默认不在场现状锁、__view 启动面类型断言）；`CONTRACT.md`
  §3:56-62 文本修正（二成员改"渐进路径安装"、:62 版本残留 1→1.1）。
- 复用依据：P5 §7 F-1/F-2/F-3。
- 风险：门禁计数 16→19 需提交信息登记（循 gates 数字纪律）。
- 验收：契约门 19/19 全绿；CONTRACT 与实现零矛盾（三面复对齐）。

### T3 settings 基础设施独立 crate 化（指令 1/2：唯一结构抽取）
- 模块：新 `crates/settings`（包名 mdc-settings，循 mdc-pdf 命名先例）；
  `crates/shell`（re-export）；根 Cargo.toml（members + default-members）。
- 复用依据：REPORT R5（域无关四层 schema + TOML sidecar 格式保持 + 六态
  降级 + FutureSchema 拒写，r24_settings 全套背书）+ 本轮实测：settings.rs
  对 mdcore 引用**仅 :102 一处常量默认值**（`undo_coalesce_ms:
  mdcore::history::COALESCE_MS`），零 Core 依赖——抽取缝极小。
- 内容：settings.rs 整体迁移；:102 默认值改 crate 本地常量（跨 crate
  对齐由既有锁 r24_settings.rs:365 `defaults_align_with_core_constants`
  原样背书——单源纪律不断）；shell `pub use mdc_settings as settings;`
  保 `mdshell::settings::*` 路径零破坏；toml_edit 依赖随迁（非新增）；
  r24_settings/contract_freeze 测试留 shell（经 re-export 路径验证）。
- 风险：workspace 波及各 feature 构建 → 全门禁复跑兜底；r24_packaging
  版本一致性锁若枚举 crate 清单需同步（执行时核对）；新 crate 需
  version.workspace=true；回滚 = revert 单提交。
- 验收：默认面全绿且测试数不变+新增；shell::settings 公开路径零破坏；
  **新 crate 对 mdcore 零依赖**（新增 grep 式依赖闭包锁测试固化）；
  pdf 四面 + perf 全绿；scan 0。

### T4 复用工具包文档（指令 3 的落地件）
- 模块：mdcore 根新增 `REUSE.md`。
- 内容：12 复用单元逐项转录（接口路径 / API / 消费模式 / 必跑测试 /
  坑位 / 对 PDF 质量的影响）+ 后续消费者快速指引（settings 基础设施 =
  本轮抽取件；治理面模式；DocumentEngine 集成先例）。全部来自 REPORT
  §2-§7，零新研究；**零上游品牌词**（scan denylist 纪律）。
- 验收：scan 0 hits；与 REPORT 零矛盾；每单元含可执行验证命令。

### T5 全门禁复跑 + 收口
- 内容：九门禁 + web 三门禁**改后全绿**（PDF 四面 + perf 6/6 全在清单 =
  指令 1"不降 PDF 质量"的量化证明）；mdcore 分工作流提交（T2/T3/T4）
  + extraheader 推送（推送前远端核实）+ 凭据零残留验证；本目录
  REPORT.md（完成项 / 测试结果 / 剩余阻塞）+ STATUS §7 更新 + worklog
  t8 收口 + cleanroom 提交推送。
- 验收：用户指令"每阶段汇报"三要素齐备；mdcore 工作区收尾干净。

## 3. 全程不变量（守护红线）

1. **PDF 质量零波及**：mdc-pdf lib 面零改动（唯一触碰 = 测试面加锁）；
   Q1-Q8 守护测试（P6 §4.3）全部不在改动面；四面门禁 + perf 复跑作证。
2. **契约面稳定**：JS 面成员清单与合法值集合不变（T2c 仅收紧非法输入、
   T2d 修文档不修行为）；wasm 导出清单不变。
3. **零新第三方依赖**（NOTICES/SBOM 零增量；toml_edit 随迁非新增）。
4. **合规**：每提交前 scan.sh；产品文档零上游品牌词；版本单源
   version.workspace 纪律延伸至新 crate。
5. **Git 纪律**：mdcore 循既有 R 轮 main 直接提交惯例（门禁双绿 →
   commit → push 前远端核实零覆盖）；PAT 经 extraheader 单次传递零落盘；
   cleanroom 追加式 worklog。
6. **证据分级**：本轮全部新结论带锚点；characterization 测试显式标注
   现状锁身份。
