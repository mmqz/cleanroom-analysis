# XLSX 内核 Round 1 阶段 A 收口报告（xlsx-r1 REPORT）

> 定位：xlsx-r1 PLAN §2 t10-a..g 阶段 A（分析仓写面，零凭据可推进
> 项）完成记录 + 阶段 B（mdcore 落地）runbook。日期：2026-10-13。
> 用户六项子要求对账逐条在 §1。

## 1. 六项子要求对账

| # | 要求 | 状态 | 证据 |
|---|------|------|------|
| 1 | 依据 office-impl 全链 + 锁版本 + 许可证/闭包/边界 | ✅ | DEPS.md（51 外部 crate 全量许可证核验零 copyleft；zip 8.6/9.0 并存登记 + 对齐建议；两 AND 条款入 NOTICES 注意） |
| 2 | 三能力分层 + C3 最小技术验证未证明不宣称 | ✅ | CAPABILITIES.md（C1/C2 实证宣称、C3 不宣称）+ FIDELITY.md（五档丢失矩阵定谳：样式/关系结构性丢） |
| 3 | 7 维语料 + 修改前后对比 + 未触及内容检查 | ✅ | t10-b 17 件语料（manifest 逐件意图与校验）+ t10-c 双重对比（条目级四分 + 语义级逐格） |
| 4 | 消毒五面复核 + 针对性安全测试（性质测试不可替代） | ✅ 设计/⚠️ 落地 | SANITIZE_TESTS.md（61 用例攻击载荷表；四面复核通过 + URL 策略缺口发现 + 补面设计）；url_policy 产品代码 + 16 测试已写并在 spike 验证（24/24）；**五面既有 API 的 45 用例落地在阶段 B**（mdcore 不可达） |
| 5 | DOCX 独立模块边界 | ✅ | CAPABILITIES.md §2（不共用领域模型/不建统一接口/消毒纯函数例外/url_policy 同性） |
| 6 | 最小可用内核 + 门禁 + 提交实现/测试/使用文档 | ⏸️ 阶段 B | io + url_policy 产品代码已写并 24/24 验证（spike lib 形态）；mdcore 落地被凭据阻塞（§4） |

## 2. 阶段 B 架构决策：双 crate 落点（本轮新增，须裁决确认）

T1 在 mdc-office 落地了**零三方依赖自锁测试**（清单级
[dependencies] 缺席 + mdcore 依赖键缺席双保险）。IO 模块的
calamine/rust_xlsxwriter/zip 依赖**不能进 mdc-office**（自锁必红）。
两个选项：

- **方案 A（建议）**：新 crate `mdc-office-io`（workspace 第八成员）
  承载 IO 模块；mdc-office 保持纯 std（+ url_policy 第六面落
  sanitize 家族，纯 std 不破坏自锁）。循 mdc-pdf/mdc-settings/
  mdc-office 一事一 crate 先例。DEPS.md 闭包声明在新 crate。
- 方案 B：mdc-office feature-gate（`io` feature 带依赖，default
  off 保自锁）——单 crate 但自锁测试须按 feature 口径改写，
  弱化 T1 不变量。

按「已验证候选推进」纪律选 A（不改动 T1 既有测试基线 = 290 只增
不改不变量成立）；如裁决另有倾向，阶段 B 执行时改接线即可（代码
零变化，仅 crate 归属）。

## 3. 阶段 B runbook（凭据解锁后机械执行）

1. **克隆**：mdcore（main=2a4cb6c，远端核实无位移）。
2. **新 crate mdc-office-io**：`crates/office-io/`；`src/lib.rs` +
   `src/io.rs` = spike `src/io.rs` 直移（改 crate 名与模块路径注释）；
   Cargo.toml 依赖按 DEPS.md §1（**zip 对齐 8.6 建议**：先按
   `zip = "8.6"` 声明 → 重测谎言头拒绝[read_proto 单测 u-13 族] →
   不过则回退 zip 9 并存登记）；workspace members/default-members
   接线 + r24_packaging 版本锁七→八 crate 同步。
3. **mdc-office 增第六面**：spike `src/url_policy.rs` 直移
   `crates/office/src/sanitize/url_policy.rs` + sanitize mod 接线 +
   crate 文档「调用方契约」节（SANITIZE_TESTS.md §7 四处契约注记）。
4. **测试落地**：spike lib 24 测试随直移（io 8 + url_policy 16）；
   t10-e §1-§5 既有五面 45 用例 = 新 `crates/office/tests/
   sanitize_security.rs`（按 SANITIZE_TESTS.md 逐条；API 形状以
   mdcore 实际为准——worklog t9-c 记录与实际如有出入以实际为准并
   回写设计文档）；17 件语料 harness 断言移植为 mdc-office-io 集成
   测试（corpus.rs 构造函数内联）。
5. **NOTICES/SBOM 增量**：首批 Office 三方依赖 51 crate（DEPS.md
   §3 全表）；**encoding_rs BSD-3-Clause 与 unicode-ident
   Unicode-3.0 两 AND 条款带全文**；ASSET_CHECK 流程对账。
6. **门禁**：九门禁 + web 三门禁全绿（290 基线只增不改；PDF 四面
   零波及；clippy/fmt/scan 零新）。
7. **提交推送**：PAT extraheader 单次传递零残留纪律；CAPABILITIES.md
   crate 文档引用接线；SECURITY_POLICY v1 草案→v1.0（§5 纪律）。

## 4. 凭据状态与阻塞面（2026-10-13 实测）

- `GITHUB_TOKEN` 空值；`~/.git-credentials`/`~/.netrc` 不存在；
  mdcore 与 cleanroom-analysis 推送均不可达（私有仓认证）。
- **本地已就绪**：阶段 A 提交 1fd729c（xlsx-r1 五件 + spike 三 bin
  + lib 产品代码 24/24）；本报告 + HANDOFF/worklog 同步为第二次
  提交。凭据注入后：推送分析仓 → runbook §3 七步。

## 5. 阶段 A 实测数字汇总（证据锚点）

| 项 | 数字 | 来源 |
|----|------|------|
| 语料 | 17 件（正向 5 + 敌意 11 + 对照 1） | manifest.md |
| 读骨架 harness | 17/17 三分归类 + 零 panic（zip 8.6 对齐态，敌意四拒二错两容忍） | read_proto.rs |
| 保真对比 | 值 37/37 格零差异；公式 6/6；合并 3/3；样式/关系丢 | FIDELITY.md |
| lib 产品代码测试 | 25/25（io 9 含门 2 谎言头 + url_policy 16） | `cargo test --lib` |
| 依赖闭包 | 对齐后 50 外部 crate（zip 9 支线移出），零 copyleft | DEPS.md |
| 针对性安全测试设计 | 61 用例（45 待落地 + 16 已验证） | SANITIZE_TESTS.md |
| 敌意构造缺陷修正 | mal6c zip64 两处（EOCD u64 / CDFH extralen 位） | corpus.rs 注记 |
| 容忍度发现 | 3 项（0x00 入值 / 垃圾静默空表 / 缺根闭合容忍[lib 测试新发现]） | read_proto + io.rs tests |
| **安全发现（本轮最重要）** | **zip 8.6 读界无声明值强制**（谎言头全量读回零错误；zip 9 内建为 9.x 独有；calamine 内部 8.6）→ 策略层门 2 `verify_actual_sizes` 闭环，harness mal6a REJECT + lib 单测双验证 | lie86 探针 + SECURITY_POLICY §2.1 校准 |
| C2 样式写入验证 | style_probe XML 级 9/9（bold/字色/填充/边框/双 numFmt/cellXfs 应用链） | spike style_probe |

## 6. 遗留登记（不阻塞）

1. R-O2 docx 渲染级读侧（维持裁决）；R-O1/R-O3 由本轮指令回放
   落定（xlsx 先行 / 依赖按已验证候选推进）；R-O4 编辑往返由
   FIDELITY.md 定谳支撑（不宣称 + 丢失矩阵在档）。
2. ~~C2 样式写入正确性逐属性断言~~（**已闭项**：style_probe 探针
   XML 级 9/9——CAPABILITIES C2 边界同步升级；完整样式模型仍不
   宣称）。
3. ~~zip 8.6 对齐的谎言头重测~~（**已闭环**：lie86 探针定谳 8.6
   无内建 → 策略层门 2 实现 + 双层验证；runbook §3.2 对应步骤
   改为直移含门 2 的 io.rs）。
4. mal7e 垃圾部件静默降级的严格模式裁决（后续轮）。
5. 阶段 B 双 crate 决策的裁决确认（§2，默认 A 执行）。
6. 门 2 双读成本优化（受信源跳过开关）——登记待裁决（v1 已知
   成本，总量 ≤300MB 已锁）。
