# Office 开发启动轮计划（office-impl PLAN v1）

> 定位：**开发轮**（Office 域首阶段）。依据 = 用户六点指令第 3 条
> （2026-10-12：Office 按自身需求开发，优先复用已验证模块与接口，不
> 预设计完整跨格式抽象）+ t8 PLAN §0 "Office 引擎（按需另轮）"的另轮
> 启动 + L3 审计链四件（office-a/b/c/d）+ 06 综合 + mdcore REUSE.md
> 复用入口（T4 明示"Office 后续按需复用入口"）。
> 本轮两翼：**安全基座**（消毒五面——office-d §5 核心裁决"消毒是管线
> 层责任"的产品侧落地，规格一手代码级、零选型依赖，直接进 mdcore）
> + **侦察选型**（Rust OOXML 生态一手核验 ADR + 敌意输入安全策略 v1 +
> spike 三探针，循 pdf-a 先例——选型未定谳前零依赖进 mdcore）。
> 代码写面 = mdcore（mdc-office 新 crate，循 mdc-pdf 并列引擎先例）；
> 侦察/spike 写面 = office-line/（office-e 报告 + ADR-O1 +
> SECURITY_POLICY v1 + spike/）。

## 0. 范围裁决转录（不再讨论）

- **做**：①消毒五面 toolkit（office-d §5.4 职责表逐面，mdc-office
  首模块）；②Rust OOXML 生态一手核验（xlsx 读/写、docx 读/写、zip
  容器防御五对象）+ ADR-O1；③office 敌意输入安全策略 v1（office-b
  §3.5 限额参照 + office-a §9 边界 + L2/03 方法论）；④spike 三探针
  （office-line/spike，循 pdf-a/spike）；⑤收口报告（含内核轮裁决
  问题整理——证据先行，零擅自定谳）。
- **不做**（用户指令 2/3/4 + 审计登记项）：office 编辑内核（op 集/
  求逆——boundary-audit X1 模式移植属内核轮，本轮零内核代码）/
  公式引擎（office-c 借鉴面，独立后置）/ D1/D2/D3 任何预裁决 /
  通用引擎或跨格式抽象 / JS 侧 dsh 管线代码（dsh 是宿主线，本轮只
  提供引擎侧 toolkit；JS 四件套消费面留宿主线）/ 选型库进 mdcore
  （ADR 定谳 + 用户过目证据后，另轮落地 + NOTICES 增量）。

## 1. 输入与分诊（指令 5：开发前质量缺口检查）

| # | 输入/缺口 | 判级 | 处置 |
|---|----------|------|------|
| 1 | office-d §5 敌意输出面规格 | **一手代码级已闭环**（四库逐库锚点 + 三档裁决 + 白名单建议）→ 无阻塞 | T1 直接实施 |
| 2 | sheetjs `_xHHHH_` 字面量歧义 = 审计【推断】登记未展开（office-d §5.2 裁决 3） | **安全正确性级**（解码往返失真是数据正确性问题）→ T1 内解决：定义消歧规则（字面量 `_x` 模式前置转义下划线）+ 往返性质测试锁死 + 文档显式标注与参照库行为差异 | **T1 升格处理** |
| 3 | Rust OOXML 生态现状 = 05 §8.1 留白（审计只覆盖 JS 生态） | 侦察缺口 → 本轮 T2 补（这正是本轮存在的理由之一） | T2 |
| 4 | 敌意输入限额参照 = office-b §3.5 一手数字（4GiB / zip 50MB·300MB / 下载 100MB/2m/3 次 / 路径逃逸 / 私网守卫） | 参照齐备，无阻塞 | T2 策略表 |
| 5 | R-1..R-6（MD 线留裁决项） | 与本轮无依赖（Office 域独立） | 维持登记 |
| 6 | D1/D2/D3（L3 留裁决项） | 本轮零触碰（消毒与选型不依赖任一裁决） | 维持登记 |

## 2. 任务清单（指令 6：模块 / 复用依据 / 风险 / 验收）

### T0 环境与改前基线 —— ✅ 已完成（t9-a，2026-10-12）
12 门禁全绿 @ mdcore d3123ba：267 / 79 / 81+6i / 109+6i / perf 6/6 /
scan 0 / wasm32 / clippy 四面 2 wasm 基线零新 / fmt 0 + 契约 18 +
smoke 15 + IME 28。环境重建记录见 worklog t9-a。

### T1 mdc-office crate + 消毒五面（mdcore 写面）
- **模块**：`crates/office`（mdc-office，包名循 mdc-pdf 先例）新 crate
  （workspace 第七成员）；`src/sanitize/` 五模块 + `src/lib.rs`。纯
  std 零三方依赖，对 mdcore **零编译依赖**（cargo tree 断言）。
- **五面**（office-d §5.4 职责表逐面对应，编号一致）：
  1. `csv_formula_guard`——CSV 出口首字符消毒：触发集 `=` `+` `-`
     `@`（§5.1）；模式可配 = 前置 `'`（默认，业界惯例）/ 前置空格 /
     typed reject；**xlsx 路线豁免**显式注明（Excel 对 .xlsx 内
     t="s" 字符串不重解释为公式——§5.1 裁决 2，防误用）。
  2. `xml_control_chars`——保字符档转义（§5.2 唯一达标档）：字符集
     = 0x00-0x08 / 0x0B / 0x0C / 0x0E-0x1F / U+FFFE-U+FFFF（非法与
     非字符集，\t\n\r 与 0x7F 放行——XML 1.0 合法）；`_xHHHH_` 约定
     + **字面量消歧**（分诊 #2：正文含 `_xHHHH_` 形态时前置转义其
     下划线，保往返零失真）+ 对称解码（decode 含消歧逆规则）。
  3. `xml_entities`——5 实体（& < > " '）+ 控制字符一体化文本/属性
     转义（pptx 属性位兜底职责，§5.2 表 docx 行调用面例外项的对策面）。
  4. `html_escape`——HTML 出口转义（§5.1 裁决 3：链接 href 未转义
     面的兜底——属性与文本位全转义，防属性注入）。
  5. `field_whitelist`——wordprocessing 域指令白名单（§5.3）：默认
     白名单 DATE/PAGE/NUMPAGES/TOC/PAGEREF/MERGEFIELD/SEQ；黑名单
     首列 INCLUDETEXT/INCLUDEPICTURE（外部资源面）/DDE（执行面）；
     策略可配（白名单制/黑名单制/全拒），typed reject 带原因。
- **复用依据**：office-d §5.1/§5.2/§5.3/§5.4（锚点已在审计报告在档）；
  REUSE.md §0 行 2（"接入第二文档模型（并列引擎）"先例 = crates/pdf）；
  门禁/合规/版本单源基建复用（r24_packaging 版本锁 + scan.sh + fmt/
  clippy 纪律）。mdc-settings 暂不消费（纯函数 + 配置结构体即可，
  消费接线留宿主/内核轮——"不预设计"）。
- **风险**：①`_xHHHH_` 消歧规则是**新设计**（审计只登记未展开）→
  以编码-解码往返性质测试锁死（任意 Unicode 输入 roundtrip 零失真）
  + 文档显式标注与参照库的差异面；②CSV 模式默认值 = 产品语义微决策
  → 默认前置 `'` 有 §5.1 业界惯例背书，其余模式全部可配不删除选项；
  ③净室纪律：产品源零上游品牌词（scan 面只描述行为与标准名
  ECMA-376，库对照证据只存分析仓）；④新 crate 接线面 = workspace
  members/default-members + r24_packaging 七 crate 锁同步 + NOTICES
  零增量（纯 std）验证。
- **验收**：五面各带正反测试（含审计证据锚定的行为用例：剥除档
  "静默丢字符"对照断言我们的保字符零丢失、损坏档"非法 XML"对照
  断言我们的输出合法、`'` 前缀消毒后 Excel 语义安全）；往返性质
  测试（proptest 级任意输入 roundtrip）；九门禁 + web 三门禁全绿；
  scan 0；`cargo tree -p mdc-office` 零 mdcore 依赖断言测试。

### T2 Rust OOXML 生态侦察 + ADR-O1 + 安全策略 v1（分析仓写面）
- **模块**：`office-line/office-e_Rust生态选型pass.md`（侦察报告）+
  `office-line/ADR-O1-office-io-selection.md` + `office-line/
  SECURITY_POLICY.md`（v1 草案）。
- **对象**（净室纪律：浅克隆一手源 + LICENSE 文件核验 + crates.io
  元数据 + 活跃度；全部结论带锚点）：xlsx 读 = calamine；xlsx 写 =
  rust_xlsxwriter；xlsx 读写 = umya-spreadsheet（对照件）；docx 写 =
  docx-rs；docx 读 = Rust 面现状盘点（docx-preview 是 JS 参照——
  Rust 读侧空位与否本身就是发现，直接印证/证伪 05 §8.1 "蓝海"判断）；
  zip 容器防御 = zip crate 限额面。
- **ADR-O1 内容**：六对象选型矩阵（能力面/许可/活跃度/敌意输入姿态/
  IO 完整性/流式能力）+ 推荐 + 未决项登记 + 与 office-d JS 四件套的
  分工定位（路线 A 出口的引擎侧职责边界）。
- **安全策略 v1 输入**：office-b §3.5 五件套限额（内存 4GiB / zip
  解压总量 docx 50MB·xlsx 300MB / 下载 100MB/2m/3 次 / 路径逃逸 /
  私网守卫）+ office-a §9（isSafeUrl 协议白名单 http/https/mailto、
  custom 验证器走公式求值无 eval、剪贴板剥离 img/svg）+ L2/03 敌意
  输入方法论（五层纵深/三行为学/沙箱四件套）+ office-d §5 输出面
  （消毒五面即 T1，策略表引用不重复）。
- **复用依据**：pdf-a 先例（ADR-P1 + SECURITY_POLICY v1 + 探针的
  结构与纪律）；office-d §8 供应链注意（sheetjs 冻结 0.18.5 / npm
  壳仓 / exceljs 低维护——JS 面登记项延续）。
- **风险**：①Rust 读侧可能空缺（发现即结论，蓝海印证写入报告）；
  ②GitHub API 限流（旁路已备：浅克隆 + 网页抓取 + git ls-remote）；
  ③许可证核验遇非标准 LICENSE 文本 → 逐字引用 + 分级标注，不猜。
- **验收**：每对象 LICENSE 一手核验在档；ADR-O1 有明确推荐 + 依据
  锚点；安全策略 v1 含威胁模型 + 输入限额表 + 输出面引用；报告经
  独立复审（锚点抽查）。

### T3 spike 三探针（office-line/spike/ 写面）
- **模块**：`office-line/spike/`（分析仓 cargo 工作区，循 pdf-a/spike
  先例；**不进 mdcore**——选型定谳前零依赖落地产品仓）。
- **探针**：①xlsx 读探针（calamine：打开 + 取值 + 大表量级时间——
  fixtures 现场生成）；②xlsx 写探针（rust_xlsxwriter：N 行写 + 产物
  zip 结构核验 + 体积量级）；③敌意输入探针（zip 解压总量限额行为 +
  超界 dimension 声明行为 + 路径逃逸条目行为——对照安全策略 v1 限额
  表，为策略表提供 Rust 侧实测依据）。
- **复用依据**：pdf-a 三探针结构（探针即基线复现 + 数字在档）；
  office-b §3.5 限额数字 = 探针③的对照组。
- **风险**：探针库版本漂移 → Cargo.lock 钉住 + 报告记录精确版本；
  敌意输入构造样本自建（zip 结构手工构造，不引外部恶意样本——净室）。
- **验收**：三探针全过 + 数字在档（时间量级 + 限额行为实测）；探针
  代码与依赖零进 mdcore。

### T4 收口报告 + 内核轮裁决问题整理
- **模块**：`office-line/office-impl/REPORT.md`（≤300 行）。
- **内容**：用户指令三要素（实际完成项/测试结果/剩余阻塞）+ **内核
  轮裁决问题整理**（证据先行）：①格式优先序建议（xlsx/docx 证据权
  重）；②内核 op 集设计输入（boundary-audit X1 模式映射 + office-a
  §13 三层命令分类的移植面）；③IO 面落地方案（ADR-O1 结果 + NOTICES
  增量预估）；④与 D1/D2/D3 的交界（零预裁决，只标注影响面）。
- **验收**：worklog + STATUS 同步；双仓推送（凭据纪律不变）。

## 3. 全程不变量（六条）

1. **PDF 质量零波及**：mdc-pdf lib 面零改动；PDF 四门禁 + perf 恒绿。
2. **mdcore 既有锁恒绿**：267 基线只增不改（本轮新增测试全部正向）。
3. **三域纪律**：mdc-office 零编译依赖 mdcore；不建通用引擎/统一
   CRDT/双向投影/跨格式抽象。
4. **零新三方依赖进 mdcore**：T1 纯 std；T3 依赖只进分析仓 spike。
5. **合规**：每提交 scan 0；产品源零上游品牌词；NOTICES 零增量
   （T1）。
6. **证据分级**：新结论带锚点或测试名；【推断】显式标注；性能仅引
   实测数字。
