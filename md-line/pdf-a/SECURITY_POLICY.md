# mdc-pdf 安全策略表 v1（PDF-a 交付）

> 日期：2026-10-10。地位：mdc-pdf（PDF-b 起实现）的安全规格基线——每条策略
> 绑定**输入资产出处**与**实现层锚点**，PDF-b/c/d 逐条落地并在语义锁中背书。
> 输入资产：acrobat-audit R1-R6（安装器/行为/网络/风险/能力面/AGS 取证）+
> pdf-line/03（敌意输入与执行边界：限额五层纵深 + 三行为学 + JS 沙箱四件套）
> + pdf-line/03 §3.4 行业 JS 决策坐标（一手源）。
>
> 总原则（继承 ADR-C1 闭源最严姿态）：**库形态产品没有平台沙箱可继承，
> 就把边界做在引擎选型 + 结构层拒绝 + 限额 + 弃置上**——攻击面压缩优先于
> 事后 containment。

## S1 引擎能力面（选型即安全决策）

| # | 策略 | 实现 | 出处 |
|---|---|---|---|
| S1.1 | **JS 引擎不接**：pdfium 取件只用 non-V8 构建（`pdfium-v8-*` 禁取）；`args.gn` 实证 `pdf_enable_v8=false` | fetch-pdfium.sh（ARTIFACT 面 + 注释禁 v8 变体 + SHA256 pin）+ rust-toolchain.toml 工具链钉住 + ASSET_CHECK A-1 args.gn 五项实证（**无 CI**——PDF-e 按实修正：原「PDF-b CI 校验」措辞为文档先于实现） | pdf-line/03 §3.4（Acrobat 走了 16 年才把沙箱推到强制、pdf.js GHSA-2026-08 宿主域任意 JS、PDFium CVE-2026-6306）；Acrobat R5/R6 |
| S1.2 | **XFA 不启用**：`pdf_enable_xfa=false` 实证 | 同上 | pdf-line/03 §2.2 L3（XfaError 限额族的存在本身证明其攻击面） |
| S1.3 | **mupdf 全面禁入**（AGPL 传染，ADR-C1） | 依赖红线（compliance 门禁 + NOTICES 审计） | pdf_carrier_plan §2 |
| S1.4 | **零网络**：mdc-pdf 进程面零外联能力（无 HTTP client 依赖、无遥测） | 依赖面审计（SBOM 白名单族）+ 代码审查锁 | acrobat-audit R5（36 PE 零网络导入 = 干净安装链的判定法）；R6（AGS 上报面 = 反面教材） |

## S2 危险动作与对象族（打开时检测 + 告警，只读不杀）

| # | 策略 | 行为 | 出处 |
|---|---|---|---|
| S2.1 | `/OpenAction`、`/AA`（附加动作）在位 → 打开时**告警 + 不执行**（v0 不实现任何动作解释器） | 检测 = lopdf 对象图扫描；执行面不存在（能力面即防御） | pdf_carrier_plan §3（R2 行为面） |
| S2.2 | `/Launch`、`/URI` 动作 → 同 S2.1（检测告警，不执行、不转交宿主打开） | 同上 | 同上 |
| S2.3 | `/EmbeddedFiles`、`/RichMedia`、`/JavaScript`、`/Names` JS 树在位 → 打开时告警（"文档含嵌入文件/富媒体/脚本对象"） | 同上 | R1/R3 字符串挖掘法（对象族枚举是审计的第一层） |
| S2.4 | 检测结果 = **结构化报告**（对象族清单 + 页位），不阻断打开（只读检测，用户自己决定去留；v0 无杀进程面） | `open_scan()` → `OpenAssessment`（**PDF-e 已实装**：mdc-pdf security_scan.rs，engine-lopdf 面 8 族 + S6 form_probe；产品面出口三打通：open_scan / `LopdfWriter::assessment()` / 会话 open 评估流转；告警不阻断打开） | pdf_carrier_plan §3 |
| S2.5 | URL 出站（标注内链接点击）若 v0 实现：scheme 白名单 https/http/mailto + 长度上限 + 零宽/Bidi 拒绝 + 单 pending 闸（七层防线照抄级参照） | PDF-c 链接面 | pdf-line/03 §3.2（LaunchUrl 七层防线） |

## S3 资源边界（限额五层纵深 + 三行为学，逐层限额）

| 层 | 策略 | 行为学 | v0 限额（PDF-b 定稿数值，此处立框架） |
|---|---|---|---|
| L0 结构层 | 解压输出上限（防 zip 炸弹）；对象嵌套深度上限；xref 对象数上限（ISO 附录 C 8,388,607）；对象流环 seen-set | ❶ 硬拒绝（结构化错误） | decode 输出 1 GiB / depth 128 / xref 8M（对齐行业值） |
| L2 渲染层 | 位图请求钳制（MAX_SIDE/MAX_PIXELS）；渲染超时看门狗；**逐页 panic 捕获**（catch_unwind → 该页 error，工人重建，一坏页不毒全档） | ❷ 钳制 + ❶ 拒绝 + ❸ 弃置 | MAX_SIDE 8192 / MAX_PIXELS 64M / 看门狗 20s（对齐 pdfcraft 实测值） |
| L3 解释层 | v0 **无 JS/XFA/表单脚本**（S1.1/S1.2）——此层 v0 只剩文本抽取预算 | —（面不存在即免疫） | — |
| L4 会话层 | 打开整档预算（页数 × 对象数的总量算）；超预算 ❷ 降级（只读前 N 页 + 告知） | ❷ 钳制降级 | OPEN_BUDGET 3s 对齐（参照） |
| L7 输入层 | 打开文件大小上限；标注写入的作者名长度；粘贴/导入面 | ❶ | 文件 1 GiB / 作者 200 字符 |

三行为学定义（pdf-line/03 §2.1，全库统一错误语义）：
❶ **硬拒绝** = 结构化错误返回（调用方决定中止/跳过）；❷ **钳制/降级** =
收敛到安全值继续 + 告知；❸ **弃置** = 承认无法中断，执行体留在后台、
结果永不采用（渲染看门狗路径）——**不假装能杀线程**。

## S4 沙箱与进程边界

| # | 策略 | 出处 |
|---|---|---|
| S4.1 | pdfium 调用独立渲染工人线程（**v0 单工人常态**；看门狗超时 → 工人换代重建，字节快照重开），与 UI/文档状态线程隔离（**PDF-e 实装**——原「线程池」措辞按实修正） | Acrobat R5 能力面方法论；pdf-line/03 §2.2（worker rebuild 模式） |
| S4.2 | 渲染工人 panic 隔离：catch_unwind → 页级 error（工人存活）；看门狗超时 → 工人换代；factory panic → 断连不悬挂（**PDF-e 实装 + T2-fix 终修**） | pdf-line/03 raster.rs 模式学（"one bad page can never take down the app"） |
| S4.3 | pdfium FFI 边界最小化：只在 mdc-pdf::render 内裸调；业务层只见安全抽象 | ADR-C1 姿态 |
| S4.4 | 增量写路径（lopdf）与渲染路径（pdfium）**分线程、共享只读**（文件字节快照隔离：写期间渲染用打开时快照，不读半截文件） | 本表新增（spike 探针 3 增量语义的直接推论） |
| S4.5 | 崩溃转储/日志本地化不外发（S1.4 的运行时面） | acrobat-audit R6 反面教材（AGS 上报面） |

## S5 加密与签名文档

| # | 策略 | 出处 |
|---|---|---|
| S5.1 | v0 只读支持：加密文档打开（标准安全处理器 password 面透传宿主）；**仍加密态拒绝增量写**（lopdf 0.45 `check_incremental_save_supported` 原生 guard——探针 3 实证路径） | pdf-line/03 §4.2（PDF 加密行业事实）+ spike 实测 |
| S5.2 | 签名文档：增量写保持原字节前缀（spike 探针 3b 断言语义入锁）——**不自证签名有效**（验证 = v1 登记项） | pdf_carrier_plan §2（lopdf 选型理由） |

## S6 已知缺口登记（诚实边界，pdf-line/03 §3.3 同款纪律）

| 缺口 | 定性 | 处置 |
|---|---|---|
| ~~AcroForm formfill 是否编入 non-V8 构建~~ **已定谳（S6 探针，2026-10-10）**：formfill 模块**存在**——`PdfDocument::form() = Some`、`form_type = Acrobat`、`field_values` 实枚举成功（/Tx 字段名值可读；对照组无表单文档 = None）。**安全含义**：表单字段是**被动可读面**而非执行面（JS 引擎已由构建层排除——args.gn `pdf_enable_v8=false`，字段 /A 动作与 format/calculate 脚本无解释器可跑）；处置纪律 = form 字段按 S2 同款对待：打开时枚举进 SecurityReport 元数据并告警，**永不初始化交互式 formfill 环境执行动作**（mdc-pdf v0 不调 InitFormFillEnvironment 的交互路径，只读 field_values）。探针证据：`spike/src/bin/formfill_probe.rs` + `fixtures/form_probe.pdf`（后者由探针段 a 现场生成，非跟踪对象——净克隆跑探针即复现）。**复原注记（2026-10-11）**：上列探针 bin 曾缺失于版本控制（历史登记与仓不符——定谳提交 010b519 报文称 "formfill_probe bin" 已入，实际文件未随提交入库，净克隆不可复现）；本日按登记语义重建入库（两段：lopdf 生成 form_probe.pdf + pdfium 只读复测，断言与对照组与 2026-10-10 定谳口径逐项一致）并复测闭环：探针全过 + mdc-pdf dual 面 15 号锁 `mount_preserves_existing_page_annots` 转绿（89过1败 → 90过0败）| 已闭环（PDF-c 轮补做；2026-10-11 探针复原重建+复测闭环；**PDF-e：form 字段枚举进 `OpenAssessment.form_probe` 已实装**）|
| pdfium 预编译二进制 = 供应链面（三平台分发链） | 中风险（plan §7） | 版本 pin + SHA256（fetch-pdfium.sh 已闭环）+ licenses/ 全量入 NOTICES（PDF-e） |
| 嵌套调用循环 / 渲染看门狗弃置线程的累积 | 低（v0 无 JS 面） | S3-L2 看门狗（**PDF-e 已实装**：换代+弃置线程登记；收报面待宿主接线轮补注——T2-verify2 移交） |
| 敌意 PDF 的 fuzz 战役（pdf-line/03 §5 方法论） | v0 只做限额与 panic 捕获，fuzz 属强化项 | PDF-e 后登记排期 |

## 实现锚点（PDF-b 起的落地清单）

1. `fetch-pdfium.sh` 升级为 mdc-pdf 基建（本表 S1.1/S1.2 校验并入）。
2. `open_scan()`（S2.4）= 打开时 lopdf 对象图扫描 → SecurityReport（PDF-b）。—— ✅ **PDF-e 实装**（security_scan.rs，8 族 + form_probe）。
3. 限额常量表（S3）= mdc-pdf::limits 模块 + 每条配对测试（pdfcraft 式
   "限额 = 逐条配回归测试的资产"纪律）。
4. 渲染工人池 + 看门狗 + panic 捕获（S4.1/S4.2）= PDF-c。—— ✅ **PDF-e 实装**（单工人 + 看门狗换代；见 S4.1/S4.2 修正行）。
5. 增量写语义锁（S5.2）= PDF-d 往返锁（探针 3 的断言升级为语义锁）。
