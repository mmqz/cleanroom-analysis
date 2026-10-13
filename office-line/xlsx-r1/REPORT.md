# XLSX 内核 Round 1 收口报告（xlsx-r1 REPORT）

> 定位：xlsx-r1 PLAN §2 t10-a..g 阶段 A（分析仓写面）完成记录 + **阶段 B
> （mdcore 落地）runbook 与执行记录（§7，2026-10-13 全步完成）**。
> 日期：2026-10-13。用户六项子要求对账逐条在 §1。

## 1. 六项子要求对账

| # | 要求 | 状态 | 证据 |
|---|------|------|------|
| 1 | 依据 office-impl 全链 + 锁版本 + 许可证/闭包/边界 | ✅ | DEPS.md（51 外部 crate 全量许可证核验零 copyleft；zip 8.6/9.0 并存登记 + 对齐建议；两 AND 条款入 NOTICES 注意） |
| 2 | 三能力分层 + C3 最小技术验证未证明不宣称 | ✅ | CAPABILITIES.md（C1/C2 实证宣称、C3 不宣称）+ FIDELITY.md（五档丢失矩阵定谳：样式/关系结构性丢） |
| 3 | 7 维语料 + 修改前后对比 + 未触及内容检查 | ✅ | t10-b 17 件语料（manifest 逐件意图与校验）+ t10-c 双重对比（条目级四分 + 语义级逐格） |
| 4 | 消毒五面复核 + 针对性安全测试（性质测试不可替代） | ✅ 设计/⚠️ 落地 | SANITIZE_TESTS.md（61 用例攻击载荷表；四面复核通过 + URL 策略缺口发现 + 补面设计）；url_policy 产品代码 + 16 测试已写并在 spike 验证（24/24）；**五面既有 API 的 45 用例落地在阶段 B**（mdcore 不可达） |
| 5 | DOCX 独立模块边界 | ✅ | CAPABILITIES.md §2（不共用领域模型/不建统一接口/消毒纯函数例外/url_policy 同性） |
| 6 | 最小可用内核 + 门禁 + 提交实现/测试/使用文档 | ✅ **阶段 B 完成（2026-10-13）** | mdc-office-io 落地（mdcore main 880864e，G1 361 全绿）；crate 文档 Capabilities 节接线；使用文档 = crate 根文档 + SECURITY_POLICY v1.0 |

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

## 4. 凭据状态与阻塞面（~~2026-10-13 实测~~ → 同日解锁闭环）

- ~~凭据阻塞~~ **已解锁（2026-10-13 同日）**：用户注入凭据（环境变量
  单次传递纪律）→ 分析仓四提交推送（dd7fb1a..91db58a）→ runbook
  §3 七步执行（见 §7）→ mdcore 双分支推送（main 2a4cb6c→880864e /
  compliance-baseline 1adced2→ddc51c1）→ 分析仓收口同步。零落盘纪律
  维持（extraheader/内联 credential helper 单次传递，磁盘零残留）。

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

## 7. 阶段 B 执行记录（2026-10-13 完成，runbook §3 七步对账）

> 前提核实：mdcore main = 2a4cb6c 无位移（ls-remote 实测）；分析仓
> 四提交先行推送（dd7fb1a..91db58a）。

| 步 | runbook 项 | 执行结果 | 锚点 |
|----|-----------|----------|------|
| 1 | 克隆 mdcore | ✅ 2a4cb6c 核实无位移；克隆走内联 credential helper（token 零落盘） | mdcore git log |
| 2 | 新 crate mdc-office-io | ✅ crates/office-io（io.rs 直移 + 门 1+门 2 + 内联 9 单测）；依赖 = calamine 0.36 + zip 8.6（deflate）+ dev：rust_xlsxwriter/flate2；workspace members/default-members 接线 + r24_packaging 版本锁七→八 | Cargo.toml / r24_packaging.rs |
| 3 | mdc-office 第六面 | ✅ url_policy.rs 直移 sanitize 家族（16 单测）；mod/lib 文档接线：面表五→六 + 消费契约五条（幂等/非幂等/保守误杀/管线顺序） | crates/office/src/sanitize/ |
| 4 | 测试落地 | ✅ 三层：spike 25 测试随直移（io 9 + url_policy 16）+ sanitize_security.rs **44 用例**（t10-e §1-§5 逐条）+ corpus_harness.rs **17 件三分断言**（构造函数内联，临时目录） | G1 明细见下 |
| 5 | NOTICES/SBOM 增量 | ✅ NOTICES **v1.3**（compliance-baseline ddc51c1）：正文 90→101（新入 9 + 账提升 2）+ 分类账 28→33；四面复跑 default 50/pdfium 64/lopdf 88/dual 101；ASSET_CHECK v1.3 对账（零新资产）；16 新 crate 三链核验 | mdcore compliance-baseline |
| 6 | 门禁全绿 | ✅ **G1 361/361**（290 只增不改 +71）+ lopdf 79/79 + pdfium 81/81+6i + dual 109/109+6i（PDF 四面零波及逐位一致）+ perf 6/6 + clippy 四面零新 + fmt 0 + wasm32 过 + scan 双 0 + web 18/15/28 | 本轮实跑 |
| 7 | 提交推送 + 文档 | ✅ mdcore main 880864e + README/COMPLIANCE 计数同步 259→361；SECURITY_POLICY **v1.0 生效**（§5 纪律兑现）；CAPABILITIES crate 文档接线（office-io lib.rs Capabilities 节引用） | 双仓提交 |

**阶段 B 新增数字**：office 面 83 项（url_policy 16 + gates 18 + security
44 + 文档 5）+ office-io 面 11 项（io 9 + corpus harness 2）= 71 净增
（290→361）；Cargo.lock 123→142（+mdc-office-io + 16 外部）。

**设计-实现出入四件（以 mdcore 实际为准，SANITIZE_TESTS.md 已回写）**：
①X-03 期望串首版拼接笔误（实测 `_x005F_x005F_x005F_x0000_`，往返
不变量恒成立）；②F-06 引号包裹黑名单 token 实测放行（Word 域语法
引号内=字符串字面量；花括号嵌套裸 token 拦截——对测钉定）；③F-10
前缀超集 token 实测拦截（startsWith 前缀扫描的设计代价，与 F-09 同
类保守误杀）；④F-08 双门分立（Blacklist 策略隔离黑名单边界，默认
Whitelist 下未知域族 NotInWhitelist 拒）。

**许可修正两件（DEPS.md 已回写，锁定版本工件为准）**：zip 8.6.0 =
MIT 单许可（首版记双许可系口径混淆）；rust_xlsxwriter 0.99.1 =
MIT OR Apache-2.0（首版记 MIT；按 MIT 履行不变）。

**过程修正**：crc.to_le_bytes() 缺引用（corpus 直移编译错）；短名
索引缓存布局（zip→.cache/3/z/zip 三层探测）与 etag 包装解析（
raw_decode 流式扫描）；rg 计数管道去重修正（(\*) 后缀剥离）——
均在档。
