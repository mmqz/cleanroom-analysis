# HANDOFF — 新会话 AI 接手指南

> 你是新接手的 AI。本文是唯一必读入口：读完本文件 + `worklog.md`，即可无缝接续全部工作，无需追问用户历史。

## 1. 项目是什么

用户在推进 **AI 工作台引擎净室分析**（clean-room analysis）研究：为其自研 AI 工作台（组成面：dsh 框架已定 / MD 引擎开发中 / 办公全家桶 + 画布待选型）系统性审计开源生态，输出可复核的深度分析文档。全部工作在 `mmqz`（GitHub 用户名）账号下进行。

## 2. 工作阶段与当前状态

| 阶段 | 对象 | 状态 | 成果位置 |
|------|------|------|----------|
| L0 | AGENTS.md 规范安全审计（vercel/AGENTS.md 等） | ✅ 完成（2026-10-08~09） | `agents-md-audit/`（9 篇分析 + 5 证据 + 5 工具） |
| L1 | Adobe Acrobat Pro DC 26.002.21931 安装器审计（R1~R6，含 AGS 取证） | ✅ 完成 | `acrobat-audit/`（4 分析 + 25 证据 + 工具链） |
| L2 | PdfCraft 净室开发分析（PDF 线四篇） | ✅ 完成收官（无剩余事项结论成立） | `pdf-line/`（01~04） |
| L3 | 办公与画布线（Univer/Excalidraw/ComfyUI 等 16 候选） | ✅ **全阶段收官**：R1 P0 三项 + R2-P1 四项 + R2-P2 六项（office-d 含）+ **R2 总收口综合**（`pdf-line/06`，458 行：协同内核五方链、渲染八路线光谱、双态六条、AI 调用面窗口、生成态 A/B、安全三闸、许可 14 对象三档、D1/D2/D3 终裁问题）——14 篇报告全部经独立复审+修补；2026-10-11 含工作区重置事故恢复（重做轮 5 全套，推送面零损失）。**待用户裁决：D1 协同内核三问 / D2 双态立项 / D3 生成态主线**（06 §8.1）；后续轮：yrs 评估 pass 最优先（D1 门控）。**office-impl 开发启动轮已完成（2026-10-12，t9-a..f）**：mdc-office crate 落地 mdcore（消毒五面 + 4000 例往返性质测试，main 2a4cb6c）+ Rust 生态选型 ADR-O1（calamine/rust_xlsxwriter/docx-rs/zip2 线）+ SECURITY_POLICY v1 + spike 三探针（谎言头/巨量 dimension 两实证）。**xlsx-r1 内核轮（Round 1 XLSX 优先，2026-10-13）全轮完成（阶段 A + 阶段 B 同日闭环）**：阶段 A = 7 维语料 17 件 + C3 保真丢失矩阵定谳（样式/关系结构性丢，组合语义=重建不宣称编辑往返）+ 读骨架 harness 17/17 零 panic + 消毒针对性安全测试设计（URL 策略缺口发现 + 补面）+ 依赖闭包 50 crate 零 copyleft；**阶段 B（凭据同日解锁）= mdc-office-io 落地（workspace 第八成员，双 crate 方案 A：容器双门[声明四检查+谎言头策略层闭环]+calamine 四面+统一错误三分，main 2a4cb6c→880864e）+ mdc-office 消毒六面（url_policy 补面，纯 std 不破自锁）+ 靶向安全 44 用例 + 17 件语料集成 harness（G1 361/361 只增不改；PDF 四面零波及逐位复验；web 18/15/28）+ NOTICES v1.3（compliance-baseline 1adced2→ddc51c1：正文 90→101/账 28→33，zip=MIT 与 rust_xlsxwriter=双许可两修正回写）+ SECURITY_POLICY v1.0 生效**；四处设计-实现出入与两处许可修正全部钉定回写（SANITIZE_TESTS/DEPS/REPORT §7 审计链在档）；R-O1/R-O3 已由指令回放落定，R-O2/R-O4 维持裁决。**xlsx-r2 三阶段（Round 2，2026-10-14 收口，用户三阶段指令）+ 遗留①增量两轮全完成**：Phase 1 C2 写面产品化（rust_xlsxwriter 升产品依赖 + write.rs 正式 API 引擎零泄漏/验证前移/流式纪律 typed 拦截 + 全环 6 测试 + 锚定钉定 1 + 新建 vs 编辑边界）→ Phase 2 门 3 部件良构性（默认严格 typed + 显式兼容结构化警告 + 深度上限闭项，SECURITY_POLICY v1.1；mal7e 类静默空表闭环）→ Phase 3 DOCX 模型级切片（docx-rs 0.4.22 引擎接入闭包 +2 + 容器门复用 + 不支持面登记，NOTICES v1.5）；遗留① DOCX 增量：docx-r1（OPC 关系解析层 + 超链接目标四变体闭「目标丢弃」+ 页眉页脚六槽位读写 + r1 勘误：引擎容忍非 panic）+ docx-r2（脚注写 + PNG 图片写/媒体清单读[r1 勘误：new_with_dimensions 无门控] + PAGE/NUMPAGES 域 + first/even 槽位 + replace_default_header 踩踏修复 + 重名 zip 语义钉定；HYPERLINK 域 todo!() 崩溃面绝不暴露）；G1 362→426 全程零回归；遗留②③待用户裁决（XLSX 样式保真编辑另议 / 门 2 跳过维持否决）。**消费面阶段（Round 3，2026-10-14/15，t16-t22 七轮全收口）**：用户三项裁决落档[t16-pre：XLSX 维持 C3 读—改—重建 / 门 2 维持双读 / t16 端到端切片]；四部曲 = DOCX 导入[t16：import .docx→三门→模型→markdown→编辑器可编辑可撤销，window.__importDocx 契约] + XLSX 导入[t17：__importXlsx 契约 + sheet→GFM 表映射 + 字节入口] + DOCX 导出[t18：export_docx + Blob 下载 + 全圆环 export→import 逐字互证] + XLSX 导出[t19：栅格映射 + wasm32 双运行时崩溃修复（SystemTime→js_sys::Date + constant_memory target-split）]；三增量 = t20 消降级[WRITE_FACE_PROBE 一手核验 + DocxWriter 五扩 + 六行映射翻转，t18 降级整批消解] + t21[wasm32 测试面修复 + 门禁收严] + t22[多行 item 续行缩进]；G1 426→481 全程零回归；**机械增量清空，剩余缺口全部带裁决前置**（XLSX 类型推断策略 / 导入导出上限对称 / 富文本无消费方 / MarkSpan URL 域边界 / 读面往返承载扩列 / 自定义编号格式）。**Round 4 能力补全阶段（2026-10-15，r4-t1..t4 四轮全收口，用户 T0-T4 执行计划）**：t1 = XLSX 类型语义[CellData 九变体三分（原始值/推断类型/显式转换）+ FIDELITY §8 R1-R13 规则依据表 + replay_cell 类型保持重放（含 1904 纪元历法保持）——前述裁决前置项⑤闭环]；t2 = 资源上限与安全边界[SECURITY_POLICY v1.2 限额表 33 项全量校准（每项执行点锚点+测试锚点）+ 读面五类限制（输入字节双族/表数/稠密包围盒 4M）+ 写面五类（全簿 1M 格预算/表数 256 fd 校准/文本·URL·公式限）+ DOCX 段落/run 计数限 + **稀疏包围盒炸弹实证闭环**（1588B→550GB 分配 abort → 门 3 前置 typed 拒）+ bench_limits 六场景基准]；t3 = DOCX 读写保真[numPr/ind 读面完整承载 + docx-r1/FIDELITY.md 保真矩阵单一真源（24 项五态：完整 10/部分 4/降级 2 可诊断/丢失 8/未验证 0）+ 读→改→写全环 ZIP+rels 对比验证 + A/B/C 方案论证（B 最小版）]；t4 = 端到端收口[全门禁快照 G1 517/PDF 四面零漂移/perf 6/web 七套件全绿/NOTICES v1.5 恒]；G1 481→517 全程零回归，三轮 Cargo.lock 零 diff。**Round 5 DOCX 消费链补全阶段（2026-10-15，r5-t1..t3 三轮全收口，用户 T0-T3 执行计划）**：t1 = DOCX 列表导入映射[docx_to_markdown 消费 r4-t3 numbering 读面——此前 p.numbering 被完全忽略列表降级普通段落（FIDELITY §3 登记缺口闭项）：Bullet→`- `/Ordered→`{n}.`（计数器 key=(numId,ilvl) 不同实例独立 + 父项推进子级重起 + 中断清零重启——Word 语义镜像 + 非连续续号登记降级）/嵌套 2×min(ilvl,8) 空格（钳 8 对称写面九级域）/降级面五类 stats 计数可诊断 + 夹具列表节 + 原生 8 测试 + e2e 9→11]；t2 = 超链接锚点写回评估[tests-only 五探针 + CAPABILITY_NOTES §13 方案与裁决点（RunSpec+anchor 加性扩推荐 vs 现状维持——**公开模型扩面未实施，呈用户裁决**）]；t3 = 集成收口[REPORT.md r5 追记 + HANDOFF 双行同步 + 全门禁快照]；G1 517→530 全程零回归，两轮 Cargo.lock 零 diff。**yrs 评估 pass 完成（2026-10-16，yrs-t0..t4，collab 线）**：D1② 证据就绪——MIT 定谳（LICENSE 直读）+ 闭包 45 包零 copyleft（cargo metadata 逐件）+ **yrs 0.28 ↔ yjs 13.6.33 双语言 wire 互证实证**（双向精确重建）+ spike 六探针全过（含两大发现：①全量 update 编码字节非确定——状态收敛≠编码同一，支持 D1③ 权威快照单点编码；②单事务批量小串装载 commit 期内存二次放大——10k×8B→+415MB/20k→+1.58GB，机制锚点 squash_left+try_squash 算术核验，大块/分事务双零成本缓解）+ 开放问题面登记（#676 现行版 heap UAF【undo+并发 GC 面】/#373/#445 等）+ mdcore op 映射八行矩阵（四向核心面可行、溯源/事件面净收益）；零终裁，mdcore 零接触；见 `collab-line/yrs-a_yrs评估pass.md` + `collab-line/spike/`（六探针源码 + 钉版锁文件） | `pdf-line/05`（规划）+ `pdf-line/06`（综合）；`canvas-line/`（7）+ `office-line/`（4 研究报告 + office-e 选型 + ADR-O1 + SECURITY_POLICY v1.1 + spike + office-impl PLAN/REPORT + xlsx-r1 七件[PLAN/FIDELITY/SANITIZE_TESTS/DEPS/CAPABILITIES/REPORT + 语料] + xlsx-r2 REPORT + docx-r1 四件[CAPABILITY_NOTES/REPORT/WRITE_FACE_PROBE/FIDELITY]）——14 篇报告 + 综合 + 开发轮五件 + 内核轮全套 + Round 2 三阶段与增量两轮 |
| L-MD | **MD 引擎（mdcore）研发与合规**（另一会话线，2026-10-10 登记） | ✅ R24 全收口 + **PDF 载体五轮 a-e 全部完成**（a 选型定谳 / b 骨架 / c 阅读面+性能门 / d 标注面+增量写互通 / e 收口：安全姿态对齐实装 8 族扫描+S6 form_probe+看门狗换代+L0/L4/L7，CONTRACT v1.3）；前置质量审计轮 T1（survey/review/4-fix/verify 链：F1 取消哨兵/F2 ByAnchor panic/O3/O4 修复+工具链钉住+fmt 清扫）；S6 探针复原（.gitignore 根因修复）；R24-d 壳实体 = 开发环境轮；mdcore main=`00ad88b`（演进链：03815b6 PDF-e → 15efc28 复用地基轮 T0-T5[settings 独立 crate + REUSE.md] → d3123ba clippy 口径修复 → 2a4cb6c mdc-office 消毒五面 → 880864e xlsx-r1 阶段 B[office-io 第八成员 + 消毒六面 + 靶向安全 44 + 语料 harness，G1 361] → 18d5bb2 阶段 B v2[第五面超链接 + 实体怪癖 IO 边界补偿，G1 361→362] → e8c3a13 xlsx-r2 Phase 1[写面产品化，G1 379] → 042a5e4 Phase 2[门 3 严格+兼容，G1 385] → 460f302 Phase 3[DOCX 模型级切片，G1 392] → 6f74b71 docx-r1[OPC 关系层 + 超链接目标 + 页眉页脚，G1 411] → cbe85c0 docx-r2[脚注/图片/域/first-even，G1 426] → ddeea4a t16[DOCX→web 端到端导入，G1 435] → de2ecf7 t17[XLSX 读面 web 消费，G1 447] → 9ee7f24 t18[DOCX 导出，G1 457] → 7705b44 t19[XLSX 导出，G1 469] → 5028ec4 t20[DOCX 导出消降级，G1 478] → e967cf2 t21[wasm32 测试面修复 + 门禁收严] → ef74282 t22[多行续行缩进，G1 481] → 1dd5cb2 r4-t1[XLSX 类型语义 CellData 九变体三分，G1 491] → 6da3d10 r4-t2[资源上限五类读+五类写+DOCX 计数限，稀疏包围盒炸弹闭环，G1 512] → 7055595 r4-t3[DOCX numPr/ind 读面承载 + FIDELITY 单一真源，G1 517] → a5c314b r5-t1[DOCX 列表导入映射——docx_import 消费 numbering 读面，e2e 9→11，G1 525] → 00ad88b r5-t2[超链接锚点写回评估 tests-only 五探针，G1 530]；九门禁 530/79/81+6i/109+6i + perf 6/6 + web 七套件 18/15/28/11/13/9/9，scan 0 恒绿，t16 以来零新三方依赖[NOTICES v1.5 恒]，wasm32 --all-targets 双面收严进门禁[t21]；第 7 次平台重置后全门禁从零独立复验零漂移[t12-a/t12-b]，第 9/10 次重置恢复在档[t20-a-review]）；compliance-baseline=`5e144cc`（NOTICES 演进：v1.3 Office 首批入账 → v1.4 写面重分类 → v1.5 DOCX 切片 docx-rs+base64；第三方 105→116 零 UNKNOWN 恒） | `md-line/STATUS.md` + `md-line/pdf_carrier_plan.md` + `md-line/pdf-a/`；产品代码在私有仓 `mmqz/mdcore` |

> **L-MD 与本仓的关系**：MD 引擎是工作台组成面之一（§1 的「MD 引擎开发中」
> 即它，进度见 `md-line/STATUS.md`）。其审计底稿在私有仓
> `mmqz/typora-re-analysis`（分区铁律：底稿不出该仓）；本仓 `md-line/` 只存
> 结论层。L2 PdfCraft 分析 = 分析别人怎么做 PDF；MD 线的 PDF 载体计划 =
> 我们怎么做 PDF——两者互补，L2 的 17 维方法论与敌意输入边界（03 篇）是
> PDF-a 安全策略表的直接输入。

## 3. 仓库结构

```
cleanroom-analysis/
├── HANDOFF.md          ← 本文件
├── README.md           ← 总索引（人类读者入口）
├── worklog.md          ← 全程工作日志（事实登记，追加式）
├── pdf-line/           ← L2 成果：01 架构剖析 / 02 生态缺口 / 03 敌意输入边界 / 04 收官报告 / 05 = L3 规划
├── agents-md-audit/    ← L0 成果：analysis / evidence / source / tools / README / WORKLOG
├── acrobat-audit/      ← L1 成果：analysis / evidence / tools / source(哈希登记+配置) / README / WORKLOG
└── tools/push_to_github.sh ← 零落盘推送脚本（模板）
```

注：Adobe 闭源二进制原件（AcroPro.msi 13MB、setup.exe、AGS 的 EXE/MSI 载荷）**不在本仓**——留在原仓库 [mmqz/acrobat-pro-audit](https://github.com/mmqz/acrobat-pro-audit)；本仓的 `acrobat-audit/source/artifact-hashes.txt` 登记了其完整哈希，可随时从 Adobe 官方 CDN（trials.adobe.com，TLS+Authenticode 四重验证过）重新获取并复核。

## 4. 必读优先级

1. 本文件
2. `worklog.md`（倒序读最后 5 个 Task 即可掌握近期状态；全文按需检索）
3. 接手 L3 时：`pdf-line/05` 的 §3（16 候选总览表）、§7（批次表，Task ID 已分配 office-a/canvas-a/canvas-d 起）、§2（17 维→11 维方法论裁剪）
4. 需要方法论背景时：`pdf-line/04`（17 维矩阵终稿）与 `pdf-line/03`（敌意输入固定章节：限额五层纵深/三行为学/沙箱四件套）

## 5. 工作纪律（必须遵守）

1. **净室纪律**：只基于公开一手源（官方仓库/文档/标准/CVE/厂商公告/可复核 API 与 git 元数据）；不接触闭源代码；二手数据（商业对比文、聚合站）仅作旁证并标注立场。
2. **token 零落盘**：用户 token 只经环境变量单次传递（`GITHUB_TOKEN=ghp_xxx sh tools/push_to_github.sh`），用毕在磁盘零残留（已多次 grep 验证）；绝不写入任何文件/remote URL/.git/config。本网关 git smart HTTP 仅接受 Basic(user:token)（Bearer 401），GitHub API 接受 Bearer——`tools/push_to_github.sh` 已内置该机制。
3. **事实登记**：每篇文档末尾附"已核带源/待核"清单；关键数字交叉验证。
4. **成果单一仓库**：全部新成果提交进本仓库（用户指示），不再另建仓库。
5. **worklog 追加式**：每完成一个 Task 追加一节（Task ID/Agent/Task/Work Log/Stage Summary），永不覆写。

## 6. 下一步任务（两条线并行，均已确认方向）

> **当前态速览（2026-10-16 yrs 评估 pass 收口后）**：office 开发轮全线完成
> （office-impl → xlsx-r1/r2 → docx-r1/r2 → 消费面 t16-t22 → Round 4 →
> Round 5，mdcore main=00ad88b，G1 530 全绿，机械增量清空）；**collab 线
> yrs 评估 pass 完成**（D1② 证据就绪，见 `collab-line/yrs-a_yrs评估pass.md`）。
> **全部剩余缺口带用户裁决前置，无无门控可推进项——阻塞呈报**：
> ①**D1 协同内核三问**（06 §8.1；D1② 证据已就绪：yrs-a §7——(a) yrs 起步
> 实证支撑与风险清单成对在档）②**D2 双态立项 / D3 生成态主线**（06 §8.1）
> ③**office-io 锚点写面裁决三问**（r5-t2 CAPABILITY_NOTES §13.4：Option A
> RunSpec+anchor 加性扩 vs 0 维持；书签写面同批否；Internal 不可达陈述项）
> ④**R-O2 docx 渲染级读侧**（ADR-O1 O1-6）⑤**MD 线 R-1..R-6**（reuse-impl
> REPORT：死设置接线/撞号修复/栈上限/F-4/基准/真机壳）⑥裁决前置清单
> （t22-review：code lang/fence、MarkSpan URL 域边界、多列族/自定义编号、
> 富文本）。凭据若再失效：全部成果已双仓推送，本地零未推送状态。

**线 A — L3 办公/画布线 R1**（P0 三项，详见 `pdf-line/05` §7 批次表）：
- ~~**canvas-a**：Excalidraw 深读（MIT）~~ ✅ **完成（2026-10-10）**：11 维全量
  + dsh/MCP 双口径能力矩阵 + 10 条可移植结论（版本三元组/双修改通道/
  双 canvas 骨架/LWW 适用边界/敌意输入三层防线等），见
  `canvas-line/canvas-a_Excalidraw深读报告.md`
- ~~**canvas-d**：ComfyUI 协议 pass（graph 模型/执行/通信/敌意输入四问题）~~
  ✅ **完成（2026-10-10）**：GPL-3.0 实证定谳（进程外协议集成唯一路线）；
  四问题全部实证回答（编辑图/执行图分离、merkle 祖先签名缓存、增量
  拓扑溶解、零认证+custom node 任意代码面）；协议最小面六端点 + 安全
  姿势五条 + 引擎可移植资产六条；已经独立复审代理 ~50 锚点抽查，见
  `canvas-line/canvas-d_ComfyUI协议pass.md`
- ~~**office-a**：Univer 深读（The Office Harness for AI Agents，Apache-2.0）~~
  ✅ **完成（2026-10-10）**：11 维全量 + dsh/MCP 双口径能力矩阵 + 10 条
  可移植结论（三层命令分类/视口双缓冲/公式引擎 RTree 增量重算/协同版本
  骨架/agent 同一 op 通道等）；已经独立复审代理 60+ 锚点抽查（9 处轻微
  修订落地），见 `office-line/office-a_Univer深读报告.md`

**R1 P0 收官**：三项全部完成。后续 R2+ 候选（依 `pdf-line/05` §7 批次表）
待用户裁决优先级；当前主线索转线 B PDF-b（mdc-pdf 骨架）。

**线 B — MD 引擎线（mdcore）R24/PDF**（详见 `md-line/STATUS.md` §4/§5/§7）：
- ~~R24-a 设置面核侧 / R24-b 多窗口核侧 / R24-c 打包基建~~ ✅ **均完成
  （2026-10-10，含质量审计轮 md-audit-2 修复 2 实bug，187/187 双绿）**
- ~~PDF-a 侦察收口 + spike~~ ✅ **完成**：ADR-P1 定谳 pdfium+lopdf 双引擎
  （mupdf AGPL 排除）+ 安全策略 v1，见 `md-line/pdf-a/`
- ~~PDF-b/c/d~~ ✅ **全部完成（2026-10-10，本会话）**：骨架 32 锁 +
  阅读面（pdfium feature + O(1) LRU + panic 隔离工人 + 性能门：打开
  0.18ms/翻页缓存 400×）+ 标注面（lopdf 增量写四 kind + 前缀逐位保持
  + pdfium 交叉读互通预证）；详见 gates.md PDF-b/c/d 三节。
- **PDF-e 阶段收口**（门禁全复跑 + SBOM/NOTICES 增量（pdfium-render/
  lopdf/传递依赖 + pdfium 二进制 licenses）+ gates PDF-e 段）= 下一会话
  首选任务；其后按用户裁决回线 A R2 或 R24-d 开发环境轮。
- R24-d 壳实体合入 = 开发环境轮；R24-e 收口。
- MD 线工作纪律差异：产品仓 `mmqz/mdcore` 每轮收口双绿（`compliance/scan.sh` + `cargo test --workspace`），契约变更走 CONTRACT v1 升版纪律。

**优先级裁决权在用户**：接手后先向用户确认两条线的开工顺序（可全并行）。

## 7. 环境备忘

- 工作区：`/home/z/my-project/`；本地交付目录 `download/`；本仓本地克隆 `/home/z/my-project/cleanroom-analysis/`
- GitHub API 直连易限流：旁路 = 网页 HTML 抓取（`github.com/<user>?tab=repositories`）、`git ls-remote`、shields.io 徽章
- 克隆大仓用浅克隆（`--depth 1`）；上游仓存 `agents-audit/_upstream/`（本地，不进 git）
- 技能调用规则、图表生成、文档产出等系统级约束按你自己的系统提示执行
