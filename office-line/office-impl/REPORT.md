# Office 开发启动轮报告（office-impl REPORT）

> 阶段：T0-T4 全部完成（PLAN v1 任务清单连续执行，2026-10-12）。
> 执行链 t9-a..t9-f；代码写面 = mdcore（main d3123ba→2a4cb6c，两
> 提交）；文档/spike 写面 = 分析仓（office-line/：office-e + ADR-O1 +
> SECURITY_POLICY + spike + 本报告）。本报告按用户指令三要素组织：
> 实际完成项 / 测试结果 / 剩余阻塞。

## 1. 实际完成项（对照任务清单逐项）

| 任务 | 状态 | 交付物 |
|------|------|--------|
| T0 环境 + 改前基线 | ✅（t9-a） | 第二次平台回滚零损失恢复（远端为真源）；12 门禁全绿复认证（267/79/81+6i/109+6i/perf6/scan0/wasm32/clippy四面/fmt + 18/15/28）+ **t8 唯一漏检项修复**（clippy 口径回退：pdfb_skeleton 测试面 1 条 + REUSE.md 命令恢复四面 --all-targets + REPORT §7 勘误） |
| T1 mdc-office + 消毒五面 | ✅（t9-c，mdcore 2a4cb6c） | `crates/office`（第七成员，纯 std 零三方依赖，零 mdcore 编译依赖）：csv_formula_guard（三模式）/ xml_control 保字符档（`_xHHHH_` + **字面量消歧含逃逸边界闭合扩展**——超参照库登记项在此解决）/ xml_entities（属性位兜底形态）/ html_escape（href 面）/ field_whitelist（七白六黑 + 全 token 前缀扫描防嵌套注入）；18 集成 + 5 文档测试 |
| T2 Rust 生态侦察 + ADR + 安全策略 | ✅（t9-d） | `office-e` 选型 pass（五仓一手核验：4×MIT+1×双许可；zip-rs/zip master 停滞陷阱；docx-rs reader 存在→蓝海校准为渲染级读侧）+ `ADR-O1`（六面建议稿 + 两开放项）+ `SECURITY_POLICY v1`（威胁六项/四层防线/限额表） |
| T3 spike 三探针 | ✅（t9-e） | `office-line/spike`（Cargo.lock 钉版本）：读面/写面/敌意输入三探针全过；**谎言头被 zip 9.x 内建拒绝 + 巨量 dimension 对 calamine 无害**两实证 → 安全策略校准回填 |
| T4 收口 + 裁决问题整理 | ✅（t9-f，本报告） | §4 内核轮裁决问题整理（证据先行） |

## 2. 测试结果

**mdcore 门禁（T1 后，全绿）**：

| 门禁 | T1 前 | T1 后 | 说明 |
|------|-------|-------|------|
| G1 默认面 | 267/267 | **290/290** | +18 sanitize_gates +5 文档测试 |
| G2-G4 PDF 面 | 79 / 81+6i / 109+6i | **恒等** | PDF 质量零波及（mdc-pdf lib 面零改动） |
| G5 perf | 6/6 | **6/6** | |
| G6 scan | 0 | **0** | 产品源零上游品牌词 |
| G7 wasm32 | 过 | **过** | |
| G8 clippy 四面 | 2 wasm 基线 | **2 wasm 基线零新** | office 面 0 警告 |
| G9 fmt | 0 | **0** | |
| web 三门禁 | 18/15/28 | **18/15/28** | web 面零改动 |
| Cargo.lock | — | **+4 行** | 纯包注册块（零三方依赖机器验证，NOTICES 零增量） |

**T1 关键测试面**：码位边界全扫（0..0x2000 + 星面）三档对照断言（损坏
档 = 产物零转义集残留；剥除档 = 往返零丢失）；字面量消歧回归组 11 例；
**4000 例 LCG 往返性质测试**（敌意字母表：形态构件 + 转义集 + 多字节
+ 实体字符，decode(encode(s)) == s 恒成立）。

**T3 探针数字**（office-e §8 全表）：读 500,000 格 139.05ms；写
1,000,000 格 971.50ms / 3.33MB；推荐件读写往返互证通过；声明预扫
0.01ms 零数据读取。

## 3. 剩余阻塞

**零阻塞**：任务清单全部完成，无停工项。

## 4. 内核轮裁决问题整理（证据先行，零擅自定谳）

Office 内核轮（op 集/求逆/事务——boundary-audit X1 模式移植 +
office-a §13 三层命令分类）启动前，以下四项建议用户裁决：

| # | 问题 | 证据基础 | 建议（非裁决） |
|---|------|----------|----------------|
| R-O1 | 内核轮启动与**格式优先序** | xlsx 证据链最厚（office-a 内核结论全部来自表格域 + office-c 公式 + T3 探针全 xlsx）；docx 模型级读写已开（docx-rs）但内核证据薄 | xlsx 先行、docx 随后 |
| R-O2 | **docx 渲染级读侧**（ADR-O1 O1-6） | Rust 空位（office-e §5）；目标线 = office-d §6.3（结构忠实 + 显式分页 + 按需渲染，不做内容驱动重排） | 生成态先模型级（docx-rs）过渡；渲染级立项单议 |
| R-O3 | **IO 依赖落地批准**（引入 mdcore） | ADR-O1 六面 + T3 互证；引入 = 首次 Office 线三方依赖进产品仓（NOTICES/SBOM 增量流程启动） | 批准后另轮落地（读骨架起步） |
| R-O4 | xlsx **编辑往返**能力（ADR-O1 O1-4，umya 按需） | 当前需求面 = 读 + 生成态写；读改写一体属编辑态，与内核轮耦合 | 随内核轮同议 |

与 L3 留裁决项（D1/D2/D3）的交界：本轮零触碰；内核轮若涉及协同
（MUTATION 广播面）则依赖 D1，本地编辑内核不依赖。

## 5. 遗留与登记

- MD 线 R-1..R-6 维持留裁决（本轮零依赖，office 域独立推进）。
- docx-rs 维护面观察项（FORK_CHANGES.md 下游 fork 功能合入线）：
  升级时读 CHANGELOG + 往返测试先行（ADR-O1 风险表）。
- SECURITY_POLICY §3 条目数上限（10,000）待 IO 模块立项实测。
- web/tests/results/ 已入 .gitignore（t9-c 过程修正，运行产物不落库
  纪律固化）。

## 6. 执行链与复现

- 执行链：t9-a（恢复+基线+clippy 修复，mdcore d3123ba）→ t9-b（PLAN
  v1 提交）→ t9-c（T1，mdcore 2a4cb6c）→ t9-d（T2 三件）→ t9-e（T3
  探针）→ t9-f（T4 本报告 + HANDOFF 同步）。
- Git 纪律：mdcore 两提交均门禁绿后提交、推送前 ls-remote 核实、PAT
  extraheader 单次传递零残留（每次推送后 grep 0）；一次 amend 发生在
 推送前（运行产物误入修正，零历史改写）。
- 证据分级：全部新结论带锚点（office-e 五仓一手）或测试名（T1 门禁）
  或探针数字（T3）；office-e 零【推断】标注。
