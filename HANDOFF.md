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
| L3 | 办公与画布线（Univer/Excalidraw/ComfyUI 等 16 候选） | ✅ **全阶段收官**：R1 P0 三项 + R2-P1 四项 + R2-P2 六项（office-d 含）+ **R2 总收口综合**（`pdf-line/06`，458 行：协同内核五方链、渲染八路线光谱、双态六条、AI 调用面窗口、生成态 A/B、安全三闸、许可 14 对象三档、D1/D2/D3 终裁问题）——14 篇报告全部经独立复审+修补；2026-10-11 含工作区重置事故恢复（重做轮 5 全套，推送面零损失）。**待用户裁决：D1 协同内核三问 / D2 双态立项 / D3 生成态主线**（06 §8.1）；后续轮：yrs 评估 pass 最优先（D1 门控）。**office-impl 开发启动轮已完成（2026-10-12，t9-a..f）**：mdc-office crate 落地 mdcore（消毒五面 + 4000 例往返性质测试，main 2a4cb6c）+ Rust 生态选型 ADR-O1（calamine/rust_xlsxwriter/docx-rs/zip2 线）+ SECURITY_POLICY v1 + spike 三探针（谎言头/巨量 dimension 两实证）。**xlsx-r1 内核轮（Round 1 XLSX 优先，2026-10-13）全轮完成（阶段 A + 阶段 B 同日闭环）**：阶段 A = 7 维语料 17 件 + C3 保真丢失矩阵定谳（样式/关系结构性丢，组合语义=重建不宣称编辑往返）+ 读骨架 harness 17/17 零 panic + 消毒针对性安全测试设计（URL 策略缺口发现 + 补面）+ 依赖闭包 50 crate 零 copyleft；**阶段 B（凭据同日解锁）= mdc-office-io 落地（workspace 第八成员，双 crate 方案 A：容器双门[声明四检查+谎言头策略层闭环]+calamine 四面+统一错误三分，main 2a4cb6c→880864e）+ mdc-office 消毒六面（url_policy 补面，纯 std 不破自锁）+ 靶向安全 44 用例 + 17 件语料集成 harness（G1 361/361 只增不改；PDF 四面零波及逐位复验；web 18/15/28）+ NOTICES v1.3（compliance-baseline 1adced2→ddc51c1：正文 90→101/账 28→33，zip=MIT 与 rust_xlsxwriter=双许可两修正回写）+ SECURITY_POLICY v1.0 生效**；四处设计-实现出入与两处许可修正全部钉定回写（SANITIZE_TESTS/DEPS/REPORT §7 审计链在档）；R-O1/R-O3 已由指令回放落定，R-O2/R-O4 维持裁决。**xlsx-r2 三阶段（Round 2，2026-10-14 收口，用户三阶段指令）+ 遗留①增量两轮全完成**：Phase 1 C2 写面产品化（rust_xlsxwriter 升产品依赖 + write.rs 正式 API 引擎零泄漏/验证前移/流式纪律 typed 拦截 + 全环 6 测试 + 锚定钉定 1 + 新建 vs 编辑边界）→ Phase 2 门 3 部件良构性（默认严格 typed + 显式兼容结构化警告 + 深度上限闭项，SECURITY_POLICY v1.1；mal7e 类静默空表闭环）→ Phase 3 DOCX 模型级切片（docx-rs 0.4.22 引擎接入闭包 +2 + 容器门复用 + 不支持面登记，NOTICES v1.5）；遗留① DOCX 增量：docx-r1（OPC 关系解析层 + 超链接目标四变体闭「目标丢弃」+ 页眉页脚六槽位读写 + r1 勘误：引擎容忍非 panic）+ docx-r2（脚注写 + PNG 图片写/媒体清单读[r1 勘误：new_with_dimensions 无门控] + PAGE/NUMPAGES 域 + first/even 槽位 + replace_default_header 踩踏修复 + 重名 zip 语义钉定；HYPERLINK 域 todo!() 崩溃面绝不暴露）；G1 362→426 全程零回归；遗留②③待用户裁决（XLSX 样式保真编辑另议 / 门 2 跳过维持否决）。**消费面阶段（Round 3，2026-10-14/15，t16-t22 七轮全收口）**：用户三项裁决落档[t16-pre：XLSX 维持 C3 读—改—重建 / 门 2 维持双读 / t16 端到端切片]；四部曲 = DOCX 导入[t16：import .docx→三门→模型→markdown→编辑器可编辑可撤销，window.__importDocx 契约] + XLSX 导入[t17：__importXlsx 契约 + sheet→GFM 表映射 + 字节入口] + DOCX 导出[t18：export_docx + Blob 下载 + 全圆环 export→import 逐字互证] + XLSX 导出[t19：栅格映射 + wasm32 双运行时崩溃修复（SystemTime→js_sys::Date + constant_memory target-split）]；三增量 = t20 消降级[WRITE_FACE_PROBE 一手核验 + DocxWriter 五扩 + 六行映射翻转，t18 降级整批消解] + t21[wasm32 测试面修复 + 门禁收严] + t22[多行 item 续行缩进]；G1 426→481 全程零回归；**机械增量清空，剩余缺口全部带裁决前置**（XLSX 类型推断策略 / 导入导出上限对称 / 富文本无消费方 / MarkSpan URL 域边界 / 读面往返承载扩列 / 自定义编号格式）。**Round 4 能力补全阶段（2026-10-15，r4-t1..t4 四轮全收口，用户 T0-T4 执行计划）**：t1 = XLSX 类型语义[CellData 九变体三分（原始值/推断类型/显式转换）+ FIDELITY §8 R1-R13 规则依据表 + replay_cell 类型保持重放（含 1904 纪元历法保持）——前述裁决前置项⑤闭环]；t2 = 资源上限与安全边界[SECURITY_POLICY v1.2 限额表 33 项全量校准（每项执行点锚点+测试锚点）+ 读面五类限制（输入字节双族/表数/稠密包围盒 4M）+ 写面五类（全簿 1M 格预算/表数 256 fd 校准/文本·URL·公式限）+ DOCX 段落/run 计数限 + **稀疏包围盒炸弹实证闭环**（1588B→550GB 分配 abort → 门 3 前置 typed 拒）+ bench_limits 六场景基准]；t3 = DOCX 读写保真[numPr/ind 读面完整承载 + docx-r1/FIDELITY.md 保真矩阵单一真源（24 项五态：完整 10/部分 4/降级 2 可诊断/丢失 8/未验证 0）+ 读→改→写全环 ZIP+rels 对比验证 + A/B/C 方案论证（B 最小版）]；t4 = 端到端收口[全门禁快照 G1 517/PDF 四面零漂移/perf 6/web 七套件全绿/NOTICES v1.5 恒]；G1 481→517 全程零回归，三轮 Cargo.lock 零 diff。**Round 5 DOCX 消费链补全阶段（2026-10-15，r5-t1..t3 三轮全收口，用户 T0-T3 执行计划）**：t1 = DOCX 列表导入映射[docx_to_markdown 消费 r4-t3 numbering 读面——此前 p.numbering 被完全忽略列表降级普通段落（FIDELITY §3 登记缺口闭项）：Bullet→`- `/Ordered→`{n}.`（计数器 key=(numId,ilvl) 不同实例独立 + 父项推进子级重起 + 中断清零重启——Word 语义镜像 + 非连续续号登记降级）/嵌套 2×min(ilvl,8) 空格（钳 8 对称写面九级域）/降级面五类 stats 计数可诊断 + 夹具列表节 + 原生 8 测试 + e2e 9→11]；t2 = 超链接锚点写回评估[tests-only 五探针 + CAPABILITY_NOTES §13 方案与裁决点（RunSpec+anchor 加性扩推荐 vs 现状维持——**公开模型扩面未实施，呈用户裁决**）]；t3 = 集成收口[REPORT.md r5 追记 + HANDOFF 双行同步 + 全门禁快照]；G1 517→530 全程零回归，两轮 Cargo.lock 零 diff。**yrs 评估 pass 完成（2026-10-16，yrs-t0..t4，collab 线）**：D1② 证据就绪——MIT 定谳（LICENSE 直读）+ 闭包 45 包零 copyleft（cargo metadata 逐件）+ **yrs 0.28 ↔ yjs 13.6.33 双语言 wire 互证实证**（双向精确重建）+ spike 六探针全过（含两大发现：①全量 update 编码字节非确定——状态收敛≠编码同一，支持 D1③ 权威快照单点编码；②单事务批量小串装载 commit 期内存二次放大——10k×8B→+415MB/20k→+1.58GB，机制锚点 squash_left+try_squash 算术核验，大块/分事务双零成本缓解）+ 开放问题面登记（#676 现行版 heap UAF【undo+并发 GC 面】/#373/#445 等）+ mdcore op 映射八行矩阵（四向核心面可行、溯源/事件面净收益）；零终裁，mdcore 零接触；见 `collab-line/yrs-a_yrs评估pass.md` + `collab-line/spike/`（六探针源码 + 钉版锁文件）。**D1-a 集成边界两轮完成（2026-10-17，d1b-t1/t2，用户裁 D1② 走 a 后执行）**：d1b-t1 = #676 定谳链闭环（GitHub API 复查仍 open 零修复 + 上游 API 面锚点 + spike P7 探针：#676 形状逐字移植[features=sync 对齐上报口径，lock 零 diff，p1-p5 零回归] + **ASan 首发确定性 UAF** + 双端 skip_gc 干净 + **混合部署证伪**[release 收敛但 ASan MIXED=1 仍 UAF → skip_gc 全编队不变量；release 收敛对内存安全零证实力] + 快照门/显式压缩点断言 + skip_gc 代价定量[RSS 3.6×/update 2.34×，显式压缩后 update 精确回基线]）；d1b-t2 = **D1-a 集成验证边界文档**（`collab-line/yrs-b_D1a集成验证边界.md` 四项边界[GC/UAF 隔离·事务粒度·撤销栈归属·稳定锚点跨模型映射] + 适配层契约草案 + 实施轮启动条件 + 撤回触发三条）；**呈用户确认边界文档后开实施轮**；见 worklog d1b-t1/t2。**R-O2 docx 渲染级读侧评估完成（2026-10-17，r-o2-t1）**：O1-6 开放项证据闭环——生态盘点一手核验（docx-preview Apache-2.0 活跃/能力上限 office-d §6.1 在档；mammoth BSD 语义面排除；LibreOffice 26.8.1 三许可仓直读[子进程形态走 ComfyUI 进程隔离先例]；Univer OSS docs 排版栈在但 docx 导入器 0 命中=Pro 面；SuperDoc/OnlyOffice AGPL+云查看器+pandoc 四件排除；crates.io 零渲染级 crate 空位实证）+ **C 路线 spike 全过**（LO headless→PDF→既有 mdc-pdf：手搓 OOXML 三样本——B 样本 5 分页符**恰 6 页硬断言**[显式分页保真定谳]、A 样本 8 关键串全在场+页数恰 2、mdcore t16 真实夹具过；冷 1024ms/热中位 304ms）+ 三路线对照（C 零新 crate 三要素即时满足[推荐起步]/A' docx-preview 过渡/B Rust 自研）+ 裁决问题三组（Q1 立项/Q2 路线含组合序/Q3 C 实施形态）——`office-line/office-f_docx渲染级读侧评估pass.md` + `office-line/spike-ro2/`（可复跑）；**呈用户裁决**。**MD 线 R-1..R-6 逐项处理完成（2026-10-17，md-r1r6-t1）**：六项零机械推进项定谳（R-1/R-2/R-3 停-产品语义、R-4 无事可做、R-5 维持延后、R-6 停-真机）+ 跨线联动并入（R-2 协同域撞号面被 yrs-b R4.4 消解=裁决范围收窄；R-3 与 B3.4 协同裁决；R-1 预留 B3.2 同源镜像）+ 收敛三组裁决问题（Q-R1 荐 A[默认零变化使能]/Q-R2 荐 c[协同域已覆盖]/Q-R3 荐维持无限至产品信号）——`md-line/reuse-impl/R1-R6逐项处理.md`；**呈用户裁决**；mdcore 零接触 | `pdf-line/05`（规划）+ `pdf-line/06`（综合）；`canvas-line/`（7）+ `office-line/`（5 研究报告含 office-f + office-e 选型 + ADR-O1 + SECURITY_POLICY v1.1 + spike×2 + office-impl PLAN/REPORT + xlsx-r1 七件[PLAN/FIDELITY/SANITIZE_TESTS/DEPS/CAPABILITIES/REPORT + 语料] + xlsx-r2 REPORT + docx-r1 四件[CAPABILITY_NOTES/REPORT/WRITE_FACE_PROBE/FIDELITY]）——15 篇报告 + 综合 + 开发轮五件 + 内核轮全套 + Round 2 三阶段与增量两轮 |
| L-MD | **MD 引擎（mdcore）研发与合规**（另一会话线，2026-10-10 登记） | ✅ R24 全收口 + **PDF 载体五轮 a-e 全部完成**（a 选型定谳 / b 骨架 / c 阅读面+性能门 / d 标注面+增量写互通 / e 收口：安全姿态对齐实装 8 族扫描+S6 form_probe+看门狗换代+L0/L4/L7，CONTRACT v1.3）；前置质量审计轮 T1（survey/review/4-fix/verify 链：F1 取消哨兵/F2 ByAnchor panic/O3/O4 修复+工具链钉住+fmt 清扫）；S6 探针复原（.gitignore 根因修复）；R24-d 壳实体 = 开发环境轮；mdcore main=`c4856f7`（演进链：03815b6 PDF-e → 15efc28 复用地基轮 T0-T5[settings 独立 crate + REUSE.md] → d3123ba clippy 口径修复 → 2a4cb6c mdc-office 消毒五面 → 880864e xlsx-r1 阶段 B[office-io 第八成员 + 消毒六面 + 靶向安全 44 + 语料 harness，G1 361] → 18d5bb2 阶段 B v2[第五面超链接 + 实体怪癖 IO 边界补偿，G1 361→362] → e8c3a13 xlsx-r2 Phase 1[写面产品化，G1 379] → 042a5e4 Phase 2[门 3 严格+兼容，G1 385] → 460f302 Phase 3[DOCX 模型级切片，G1 392] → 6f74b71 docx-r1[OPC 关系层 + 超链接目标 + 页眉页脚，G1 411] → cbe85c0 docx-r2[脚注/图片/域/first-even，G1 426] → ddeea4a t16[DOCX→web 端到端导入，G1 435] → de2ecf7 t17[XLSX 读面 web 消费，G1 447] → 9ee7f24 t18[DOCX 导出，G1 457] → 7705b44 t19[XLSX 导出，G1 469] → 5028ec4 t20[DOCX 导出消降级，G1 478] → e967cf2 t21[wasm32 测试面修复 + 门禁收严] → ef74282 t22[多行续行缩进，G1 481] → 1dd5cb2 r4-t1[XLSX 类型语义 CellData 九变体三分，G1 491] → 6da3d10 r4-t2[资源上限五类读+五类写+DOCX 计数限，稀疏包围盒炸弹闭环，G1 512] → 7055595 r4-t3[DOCX numPr/ind 读面承载 + FIDELITY 单一真源，G1 517] → a5c314b r5-t1[DOCX 列表导入映射——docx_import 消费 numbering 读面，e2e 9→11，G1 525] → 00ad88b r5-t2[超链接锚点写回评估 tests-only 五探针，G1 530] → b15c0ab r5-t3[写面资源限补检：tooltip 255 引擎镜像 + write_row 列域，G1 531] → c4856f7 office-g Phase 1+2[锚点写面实施批——DECISIONS #8 批次 2 裁决合入：RunSpec.anchor/anchor_link/bookmark_paragraph/InvalidInput 变体/V1-V7 校验/T1-T7，G1 538]；九门禁 538/79/81+6i/109+6i + perf 6/6 + web 七套件 18/15/28/11/13/9/9，scan 0 恒绿，t16 以来零新三方依赖[NOTICES v1.5 恒]，wasm32 --all-targets 双面收严进门禁[t21]；第 7 次平台重置后全门禁从零独立复验零漂移[t12-a/t12-b]，第 9/10 次重置恢复在档[t20-a-review]）；compliance-baseline=`5e144cc`（NOTICES 演进：v1.3 Office 首批入账 → v1.4 写面重分类 → v1.5 DOCX 切片 docx-rs+base64；第三方 105→116 零 UNKNOWN 恒） | `md-line/STATUS.md` + `md-line/pdf_carrier_plan.md` + `md-line/pdf-a/`；产品代码在私有仓 `mmqz/mdcore` |

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

> **当前态速览（2026-10-27 30 轮 UI 战役 dev-r1..r22·最新）**：用户
> 「持续推进30轮次包括ui的开发，记得及时提交代码」→ **issues #9/#10
> 全链闭合 + UI 深化三批交付**（mdcore main `cdc6e1f→51da472` 十一
> 提交全推送，测试面 802→**810/810** 核心零失败 + GUI 4/4 = 814 口径）：
> ①**#9（TR-1）行尾镜像修复**（markdown_to_tree 入口 CRLF/孤立 CR
> 统一 LF——模型=权威层三入口漏斗单点；tr1_line_endings 六件 +
> tr1_web_e2e 两件 = `__md()` 保存内容=所见闭环）；②**#10（TR-4）
> 导出格式消费**（export_docx_styled 加性面：镜像 office-io API +
> docx_options/docx_invalid_input 分层校验 + V9 漏斗单一真源 +
> ⚙ 预设面板[公文 A4=G2 LO 实测口径] + 鸭子探测旧 pkg 兼容 +
> TR-4 六件锁 + docx_export 套件 13→18）；③**UI 批 A**（格式工具
> 条 11 键 8 快捷键——程序化文本编辑模式定谳 + **forceFullSync
> 产品缺陷修复**[快路径字面吸收实证] + 状态栏 5 槽 + Ctrl+S/
> 自动保存提醒事件）；④**UI 批 B**（TOC 大纲 + 查找替换[零新增
> 依赖 + 单事务全替换 undo 一次撤全]）；⑤**UI 批 C+D**（字号
> Ctrl+=/-/0 + 打印样式 + 帮助面板 F1 + 行号槽 + README §8）；
> ⑥**敌意回归** hostile_ui 8 checks（空文档/200KB/emoji 代理对/
> BIDI/面板叠加/连击/localStorage 污染）。**web 测试面 15→18 套件
> 239+ checks 全绿**（DOM 契约零接触）；R-O2 22 用例 SUITE PASS
> （export_docx_styled 零 LO 漂移）；SBOM PASS；yrs 三锚点零实质
> 变化（#676 口径修正为 issue 非 PR）+ TDF 26.2.7 终版定档 10-29
> 倒计时中；**第 24 次重置恢复**（dev-r3 中途被吞，远端零损失
> 重做）；硬盘三拍治理（92%→33% 终态）。**剩余 = #3 UI 单后续
> 迭代空间 + 产品决策三项 + 方言裁决——待用户；dev-r23-r30 收尾
> 轮进行中**。
>
> **前一速览（2026-10-26 issues 驱动开发大会话·历史保留）**：用户
> 「查看issues，进行至少50轮次的开发，注意硬盘管理」→ mdcore
> **8 条 issues 全清单处理完毕**（main `483032f→cdc6e1f` 八提交，
> 测试面 782→**802/802** 零失败）：①**PR #1 审查合并**（启动态
> 收口 + perf 门相对判据——双面验证后 ff 合入，PR 自动 merged）；
> ②**G1 TOC 域**（docx-rs TOC build 完整勘误 + FieldSpec 五连
> 发射 + V8 + LO 跨层）；③**G2 格式面**（PageGeometry/
> Typography 公开类型 + LO 实测 90.1pt/14pt 精确生效）；④**G3
> 暗标清零**（R-11 BLOCKER 闭环——威胁面修正定谳：真实面在
> export_clean 原字节拷贝；anonymize 五部件 + ExportAnonymous
> 命令 + wasm 出口）；⑤**#5 PDF wasm 接线**（npm build:full +
> 桥激活 15/15）；⑥**#4 平台面**（Linux-only 平台门 +
> windows-msvc 交叉 check 首次全绿 + COMPLIANCE §8 矩阵真源）；
> ⑦**#3 UI 产品化**（主题三态 CSS 变量化 + 设置面板全 9 键位 +
> 顶栏分组 + 折叠 + 9 截图——CONTRACT 零变更，web 13 套件全绿）；
> ⑧**#8 敌意测试轮次**（87 用例四组外部独立工具验证：**零 crash
> 零数据损坏零 P0**，幂等+镜像 63 用例全过，导入召回 100%，敌意
> 5 面全拒含 billion-laughs，1MB 注入 220ms；登记 7 项 + TEST-
> ROUND-REPORT + 子 issue #9/#10 开单；已知 4 问题全部定谳）。
> 第 23 次重置恢复 + 硬盘三拍治理（22.8GiB 回收，终态 31%）。
> **剩余 = #9/#10 两张 P2 子单 + 产品决策三项 + 方言裁决——全部
> 待用户，工程侧无欠账**。
>
> **前一速览（2026-10-26 maint-r22 自主维护轮·历史保留）**：用户用户
> 「继续推进 + 全景进度报告」→ **第 22 次重置**（同签名：
> ~/.cargo 与双仓快照丢失，/tmp 全部幸存——scripts 21 件 +
> optroot 2.3G + lo26/lo268）恢复 + **11 面复验全绿零漂移**：
> ①双仓 fetch 甄别零独有 → ff 拉齐最新推送态（cleanroom
> **0571a6c**——含 maint-r21-t5/t6 增量 / mdcore **3e1f616**）+
> rustup 1.99.0 重装；②核心四面（782/782 零失败 + fmt 0 +
> clippy 恰 5 误报基线 + wasm32 rc=0）；③**GUI 壳 786 口径**
> 复验（optroot 必需件 586M 搬运复刻——.pc prefix 绝对路径
> 硬编码使 sed env 变体法被链接失败证伪，回退 t5 搬运法）=
> mdshell-native 4/4 = **786/786 全量**；④适配层 **57/57**；
> ⑤R-O2 **22=10+12+0 SUITE PASS** 同判零漂移（r22.json）；
> ⑥web 构建链重建（wasm-bindgen 0.2.129 锁版）+ **12 套件
> 179 checks** 逐件一致；⑦SBOM PASS（0 UNKNOWN/0 GPL）；
> ⑧**D1 就绪探针 p15/p16/p17 release ALL PASS**（批次 1 证据
> 链重置后独立成立）；⑨yrs 三锚点**零增量**（#682 open/
> #676 open/0.28.0 最新——升级窗口维持关闭）；⑩TDF 时效零增量
> （**26.2.7 终版 10-29 倒计时 3 天**未提前发布，26.8.1=
> Latest Branch）；⑪**磁盘满第 9 次事件治理**——rootfs 为
> overlay 形态（lower 层删除不回收，/tmp 幸存件 rm 后 df 不
> 降为实证），cargo clean 回收 7.4GiB → 32%。**剩余门控 =
> 批次 1/3/4 不变**（批次 1 D1 解锁包 = 最优先证据链本轮复验
> 在案；批次 3 三选 + 26.2.7 终版 10-29 倒计时最紧迫；批次 4
> 可整批按推荐默认）。
>
> **前一速览（2026-10-26 maint-r21 连续维护轮·历史保留）**：用户
> token 续能「继续开发/继续做持续做」→ **连续两次会话内重置**
> （第 20/21 次同签名：cargo/双仓快照丢失，/tmp 第 21 次**全部
> 幸存**）双轮恢复 + 交付面全面独立复验：①双仓 ff 拉齐推送态
> （cleanroom 00a5f41→maint-r21 增量 / mdcore **3e1f616**）+
> rustup 1.99.0 钉版两轮重装；②核心门禁四面**亲跑全绿**（782/782
> 零失败 + fmt 0 + clippy 恰 5 wasm_bindgen 误报基线 + wasm32
> rc=0）；③**适配层 57/57 零失败**（matrix24+hostile13+limit10+
> anchor6+soak4——a5-r5 全套件口径）；④R-O2 复跑 **22=10+12+0
> SUITE PASS** 同判零漂移双轮（r20/r21.json）；⑤**D1 就绪探针
> p15/p16/p17 release ALL PASS**（批次 1 证据链重置后独立成立）；
> ⑥**web 构建链重建 + 12 套件 179 checks 亲验全绿**（wasm-bindgen
> 0.2.129 钉版 + pkg_pdf 补腿）；⑦**SBOM 全闭包 PASS**（0
> UNKNOWN/0 GPL/0 白名单外）；⑧yrs 三锚点零增量（#682 open/
> #676 open/0.28.0 最新——升级窗口维持关闭）；⑨**TDF 时效增量
> 二（REGRESSION §10.9）**：**26.2.7 = 26.2 分支终版定档
> 2026-10-29（距今 3 天）**+ 26.8.1 已过早期风险点——批次 3
> 裁决窗口紧迫度上升；⑩（t5-t6 增量）**GUI 壳 786 口径闭差**
> ——optroot 必需件 586M 搬运清偿"待搬运"欠账 → source
> env.sh 后 mdshell-native 编译打通 + **4/4 全绿** + clippy
> rc=0 = 本环境首次 **786/786 全量亲验**（"gtk dev 缺结构性
> 受阻"登记翻案，Xvfb 实机维持 s5b 在档口径）；⑪**21 探针
> release 全覆盖**（本环境历史首次全族：p9 过时断言修复——
> c205fec 窗 0 缓解漏更探针地雷 5 天后首跑暴露，同款翻转修复
> 全绿；p20 cfg-1 panic=true 缺陷证据链维持）；⑫**ASan 适配层
> 57/57 零报告**（nightly-2026-10-15，超 d1e-t1 44/44 口径）。
> **剩余门控 = 批次 1/3/4 不变**（批次 1 D1 解锁包 = 最优先，
> 证据链本轮全部复验在案；批次 3 三选 + 终版倒计时；批次 4 可
> 整批按推荐默认）。
>
> **前一速览（2026-10-25 maint-t3 十轮自主维护轮·历史保留）**：
> 用户「继续至少10轮次」→ 第 19 次环境重置（cargo/.lo26/scripts
> 大部再丢）自愈 + **并行线交付面全面独立复验**十轮：①环境自愈
> （rustup 1.99.0 全组件 + wasm32）；②**mdcore c33bcab 门禁独立
> 复验——抓到 fmt 破裂**（pdfb_skeleton use 排序，并行线自报
> "全门禁复验"与实际不符）→ 修复提交 **`3e1f616`**；核心面
> **782/782 零失败**（786 口径差 4 = mdshell-native GUI 壳，本
> 环境 gtk dev 缺 + sudo 不可用受阻）+ clippy 核心 rc=0 +
> wasm32 rc=0 + fmt 修复后 0；③+28 提交面审：三不变量全过
> （yrs 0.28 钉版 / skip_gc B1.1 session.rs:193 / **mdcore 核心
> crate 零触碰**）+ office-io rebuild 新模块纯增量；④yrs 上游
> 三锚点零增量（PR #682 仍 OPEN / #676 仍 open / 0.28.0 仍最新）
> ——升级窗口维持关闭；⑤R-O2 套件复跑 = **22=10+12+0 SUITE
> PASS 基线完全同判零漂移**；⑥**TDF 版本线时效增量**（DECISION_
> SHEET 批次 3 已登记）：25.8.x = 死路（自身 EOL 2026-06-12）；
> **26.2 分支 EOL = 2026-11-30（仅剩 ~5 周）** vs 26.8 EOL
> 2027-06-13——「零翻动 vs 5 周短窗」「一良性翻钉 vs 7.5 月长窗」
> 权衡面完整呈递；⑦scripts /tmp 抢救 18 件（原版 webkit_closure/
> xvfb_shell_run/sbom_check 全收回；仅 verify_authenticode/
> nextbatch_search 2 件低优先级确认丢失）+ sbom_check_strict
> 重建版；⑧**web 面 12 件套件 179 checks 亲验全绿**（构建链自愈：
> wasm-bindgen 0.2.129 + npm + dist + pkg_pdf 补腿后
> integration_journey 7/7）；⑨**SBOM 合规门禁 PASS**（Linux 面
> 217 三方包 = 193 PASS + 24 WARN 系统库豁免 + **0 GPL 0
> UNKNOWN**——wry/tao/gtk 新依赖面合规）；⑩磁盘治理（cargo
> clean 6.4G + tmp 341M，100%→40%）。环境注记：/tmp/my-project/
> optroot（2.3G）= s5a 工件副本在场，GUI 壳复验欠账降级为"待
> 搬运"。
> **剩余门控 = DECISION_SHEET.md 批次 1/3/4 不变**（批次 1 D1
> 解锁包三件 = 最优先；批次 3 三选证据齐备 + 时效增量已呈递；
> 批次 4 可整批按推荐默认）。
>
> **前一速览（2026-10-25 maint-r1 上游跟踪维护轮·历史保留）**：
> 用户 token 续能「继续做」→ **第 18 次环境重置事故恢复 + 丢失轮
> 重做 + 推送收口**。事故链：上轮（同日早间）完成的 r-o2-t6 /
> maint-t1 / maint-t2 三提交（LO 26.8.1 复跑 + yrs 跟踪 + 26.2.6
> 复跑）在环境重置中**本地全丢**（仓库快照回退 ae4aa4d，提交对象
> 不存在于 reflog/fsck）；并行 Stage 5 会话线（workbench，10-19~
> 10-24）已把两仓推进 53+28 提交推送远端（cleanroom `f9efe19` /
> mdcore `c33bcab`——26.8.1 复跑证据被其 §10.5 等价覆盖）。本轮
> 双仓 ff 拉齐 + 重做三件未被覆盖的独有面并推送：①**yrs 上游
> 跟踪**（DECISIONS #19：仓转移 y-c-rs/yrs → **y-crdt/y-crdt**
> 旧路径 404 + #676 新家保号仍 open + **修复 PR #682 在途**——
> 根因面正中 + 0.28.0 无新 release）；②**TDF 25.2 分支 EOL
> 定谳**（25.2.7 final，2025-10-30——批次 3「钉 25.2.x」语境
> 更新：EOL 存量）；③**26.2.6.3 still 线复跑**（REGRESSION §10.7：
> 22 用例与基线**完全同判零破裂**双次稳定——**批次 3 三选证据腿
> 齐备**：25.2.x EOL / **26.2.x 零翻动最平滑** / 26.8.x 一良性
> 破裂）。环境注记：cargo/.lo26/.lo262 随重置丢失，本轮重装 26.2.6
> 前缀复跑；mdcore 零接触（ff 拉齐推送态，门禁以 s5b 速览口径
> 为准）。
>
> **前一速览（2026-10-24 s5b-wry-deb 实机收口·历史保留）**：
> 「继续做」+ token 续能——本会话从 Gate 0 重核基线出发，完成
> **wry/tao 壳实机集成 + §九.9 deb 打包面**（mdcore main =
> `c33bcab`，b4e6a12→c33bcab 已推送 ls-remote 校验）。**核心翻案**：
> 上轮登记「webkit2gtk/gtk3 沙箱缺席=真机集成**结构性受阻**」被推翻
> ——s5a-env 无 root 自愈（apt download 92 包闭包 + dpkg -x 1.1G
> 提取树 + LD_PRELOAD exec 路径 shim；39bd74c 先例放大）成立，
> wry/tao/gtk 依赖实体化（shell-native 蓝图→实体）。**实机三级递进
> （Xvfb 真进程亲跑）**：mdapp:// custom protocol 下完整 web 工作台
> 四表面探针全绿（blockCount:9/cmEditor/grid/ev/pdf/canvas/collab
> + EXIT=0）——宿主 A 定谳的实机兑现。**三项工程发现**：①wry 0.46.3
> IPC 缺陷（file:/// 空 authority 在 http::Uri InvalidFormat → 宿主
> panic；绕开=导航拦截通道，未修上游注记在档）；②WebKit file://
> wasm/ESM 限制（http 对照实证；build.mjs 的 file:// 假设修正）；
> ③mdapp:// 为桌面壳加载本地 bundle 的产品级形态。**§九.9**：deb
> 实体链全通（build-deb 两处修 + dist 捆绑 → mdcore_0.1.0_amd64.deb
> → dpkg -x 布局验证 → 包内二进制 Xvfb 首启冒烟全绿）；边界诚实
> 登记：dpkg -i 安装面无 root 未做（提取形模拟）、AppImage 未实跑
> （外部工具链未引入）。**真缺陷修复**：mdc-pdf pdfb_skeleton
> Instant::now→portable_now（wasm32 --all-targets 编译障）；wasm32
> 口径更新 --exclude mdshell-native。**合规**：SBOM 全量复跑 366
> 包 0 UNKNOWN/0 GPL/0 白名单外 + COMPLIANCE 增量段。**门禁全亲跑**：
> G1 **786/0** / fmt 0 / clippy mdshell-native 0 / wasm32 0 错 /
> scan PASS / web **15 套全 PASS** / lock +2813（wry/tao/gtk 闭包）。
> 文档欠账补记：p6-t2（06ba701）+ p6-t3..t6（b4e6a12）两条入档。
> **§九 剩余**：§九.9 残项（dpkg -i 安装面/AppImage/NSIS/.app——
> 需真机或外部工具链）；**§九.10 已收口**（s5c-host10：cargo
> metadata 零 GUI 引用 + mdshell::Host 双壳同契约实证，判定成立）；
> wry 壳完整功能（对接点 1-5：文件对话框/渐进进度/Ctrl+S——
> R24-d 开发环境轮范围）。
> 环境注记：磁盘满第 6/7 次事件清理恢复（9.3G→4.3G 固化模式）。

> **当前态速览（2026-10-23 p6-t1 阶段6启动件·历史保留）**：
用户「继续持续做不做完不停止用子代理加速和审核代码」——本会话
连续五轮收口（p5-t2 表格 → p5-t3 证据 → p5-t4 PDF → p5-t5 Canvas
→ p6-t1 集成旅程），**阶段 5 四表面全部落地 + 阶段 6 骨架启动**
（**mdcore main = 6daeace**）：p6-t1 = 四表面集成旅程 e2e
（integration_journey.py **7/7**——§九.1-4 实际使用路径：文档
编辑往返/表格开滚编导/证据八步骨架/PDF 增量保存/Canvas 双实例
收敛/跨表面共存零 pageerror；零产品代码增量）。p6-t2 wry 壳
编译面：webkit2gtk/gtk3 沙箱缺席——**真机集成环境阻塞**（开发
环境轮边界项，选型记录既列）。**本会话累计**：G1 730→**769**
（+39 测试）/ web 测试 5→**10 套**（新增 grid 20 + evidence 16 +
pdf 15 + canvas 15 + journey 7）/ 3 个 wasm 句柄（WasmBook 扩展
+ WasmWorkbench + WasmPdfSession + WasmCanvas）/ 2 新 crate 面
（office-io rebuild + mdc-pdf-wasm）+ 4 个对抗审查轮（2+1+1+1
MUST-FIX 全修）/ clippy 基线 2→4 同型登记 / NOTICES 增量登记
（pdfjs-dist 13 包 Apache-2.0）。**剩余（阶段 6 完整清单）**：
§九.5-8 web 面组合扩展 + §九.9 打包真机[环境阻塞] + §九.10 可选
宿主适配 + wry 壳[环境阻塞]。环境注记：磁盘满 5 次事件（清理
模式固化）；BigInt u64 教训三次入档。

> **当前态速览（2026-10-23 p5-t5 Canvas 表面 + 阶段5 四表面收口·历史保留）**：
用户「继续持续做不做完不停止用子代理加速和审核代码」——阶段 5
§八.5 Canvas 表面落地，**四表面全部收口**（**mdcore main =
2a67e77**）：**mdc-canvas 时钟注入面**（SessionOptions.clock_ms +
HostClock 适配器——wasm32 SystemTime trap 解，native 零变化，
canvas 23 测 + e2e_demo 零回归）+ **WasmCanvas wasm 出口**
（八命令 flat JSON/回执/读模型/指纹/协同四方法/事件轮询 base64/
静态 restore + CvError 全变体一码 + 测试 7）+ **web canvas_view.ts
1585 行**（SVG 保留模式四层 + 双实例直连协同演示[事件 updates 转发
+ encode_diff 追赶 + 收敛断言 + 远端不入本地栈] + CDP 拖拽 LWW 提交
粒度）+ e2e **15/15**（含 100 节点烟测 0.15s）。汇合修复：BigInt
四处（u64 第三课）+ Option<String> null 契约 + NaN 断言分层。
门禁（亲跑）：G1 **769/769** / fmt 0 / clippy 恰 4 同型 / scan
PASS / web **十套全绿**（smoke 15 + grid 20 + evidence 16 + pdf
15 + canvas 15 + contract 18 + 既有四套 42）。**阶段 5 四表面
验收面成立**（正文/表格/证据/PDF/Canvas 全有可重复交互测试与性能
数据；单一真相纪律全表面——wasm 会话唯一权威 + 宿主零第二数据源）。
**下一步 = wry 壳真机集成**（宿主选型 A 的桌面壳实体——开发环境
轮 R24-d 性质；选型记录既列前置项）或 **阶段 6 端到端质量验收**
（§九 十项清单：长文档/表格/PDF/人机共作/并发/sidecar 异常/超大
输入/退出恢复/打包实测/可选宿主适配）。环境注记：磁盘满第 5 次
事件（大测试二进制清理模式已固化）；clippy 基线 3→4 同型
（canvas restore 静态方法）。

> **当前态速览（2026-10-23 p5-t4 PDF 表面·历史保留）**：
用户「继续持续做不做完不停止用子代理加速和审核代码」——阶段 5
§八.4 PDF 表面全链落地 + **渲染路径决策定谳**（**mdcore main =
37399eb**）：**路线 B = pdf.js 渲染 + mdc-pdf lopdf wasm 写回**
（渲染分离、真相不分离——调研子代理实测背书：A pdfium-wasm 否决
[bblanchon 堆不可增长多页 OOM + wasm 无线程 S4 塌缩]；C 壳侧渲染 =
wry 落地后 native 升级位[protocol.rs 信封可替换]）。**Rust**：
mdc-pdf 便携化（PortableInstant——修 wasm trap 'time not
implemented'）+ AnnotationWriter trait product_bytes 产物取回面 +
新 crate **mdc-pdf-wasm**（pdf_open_scan + WasmPdfSession[new/
set_page_text/execute/attach_writer/flush_annotations/product_bytes]
+ 5 测试）。**web**：pdf_view.ts 1991 行（pdf.js legacy 显示面 +
canvas/textLayer + 64MB ImageBitmap LRU + L2 像素钳制镜像 + 四 kind
标注 + 增量保存链 + 搜索双面诚实[会话命中权威+pdf.js 几何] +
evidence-jump 钩子）+ pdfjs-dist@6.4.299（Apache-2.0——COMPLIANCE
登记 13 包 optional 传递面）+ 纯标准库 PDF 夹具生成器 + e2e
**15/15**（1 SKIP=加密档登记；含双修订链字节级锁）。**审查**：
1 MUST-FIX（COMPLIANCE 事实错误）+ 3 SHOULD-FIX（flush 文档归位/
L2 钳制+evict/双修订链锁）全修。门禁（亲跑）：G1 **767/767** /
fmt 0 / clippy 恰 3 同型 + wasm32 **0** 警告 / scan PASS / web
回归七套全绿。**下一步 = §八.5 Canvas 表面**（实际节点编辑与协同
操作——mdc-canvas headless 域模型/命令接口/证据关联已备，web 面消
费）→ wry 壳真机集成（选型记录既列）。环境注记：磁盘满第 4 次
事件（大测试二进制清理恢复）；wasm trap 定位面 = console_error_
panic_hook 已接线（pdf_wasm_init）。

> **当前态速览（2026-10-23 p5-t3 证据界面 wasm 化·历史保留）**：
用户「继续持续做不做完不停止用子代理加速和审核代码」——阶段 5
§八.1 壳与证据界面两前置件 + web 侧栏全链落地（**mdcore main =
3130b8a**）：**workbench 内存会话**（Io 双后端——Files/Memory 仅
分派 IO 边界，验证/级联/审计核心单一；start_memory/open_memory/
ingest_source_bytes/refresh_body_bytes/export_clean_bytes/
sidecar_json/attach_source_bytes）+ **WasmWorkbench wasm 出口**
（八步全方法 + 查询面 + 20 变体 wb_* 错误码一码一义 + 纯内核
parse_*）+ **web 证据侧栏**（evidence_panel.ts 950 行：三态徽标
卡片/审核操作/失效侧 reverify 分派/sidecar 导出恢复/干净正文导出/
claim 跳转钩子；flex 双栏布局既有 DOM 零破坏；BigInt u64 边界）。
**审查 3 MUST-FIX 全修**：M1 export_clean 次序漂移（拒绝路径幽灵
审计——三段拆分恢复零状态变化纪律）/M2 reverify 按定位器类型分派
（claim 侧失效绕过锚纪律静默恢复——改按失效侧分派 + e2e 锚重绑
回归锁）/M3 target_scale 截断（try_from）；S1-S5/N1/N2 全修。
门禁（亲跑）：G1 **751/751**（+8）/ fmt 0 / clippy 恰 3 同型基线 /
scan PASS / lock +2 内部边 / wasm32 0 错 / web 八套全绿（evidence
**16/16** + xlsx_grid 20 + 既有六套 75）+ pkg 重建复验。**下一步
= §八.4 PDF 表面**（页面缓存/缩放滚动/文字选区/批注及证据区域
叠加——复用 mdc-pdf pdfium 既有安全边界与 O(1) LRU）→ §八.5
Canvas 表面 → wry 壳真机集成。环境注记：磁盘满第 3 次事件
（target 增量 4.1G 清理恢复 51% 空闲）；clippy 基线 2→3 同因同型
（wasm_bindgen static_method_of——restore 新方法）。

> **当前态速览（2026-10-23 p5-t2 表格 UI 产品化·历史保留）**：
用户「继续持续做不做完不停止用子代理加速和审核代码」——阶段 5
§八.3 表格表面全链落地（**mdcore main = 823414a**）：
**Rust 侧** = office-io 新模块 rebuild（C3 读-改-重建产品化：
replay_cell 单格联接 + 编辑账本[窗内覆盖/窗外扩展/Empty 清空/后写胜]
+ 表名防线[清洗+重截去重恒可终止] + 敌意跳过与扁平化全部 skipped
登记 + FFFE/FFFF typed 拒）+ wasm export_with_edits/
last_rebuild_stats_json/sheets ext 双暴露（非原点锚定表建域契约）。
**web 侧** = xlsx_grid.ts 1162 行（三层虚拟化+冻结四层双轴钉扎+
DOM 池化局部更新+完整键盘导航+命令账本编辑[撤销栈语义]+TSV 复制
粘贴+证据高亮钩子+sheet 切换+导出 Blob/统计/警示）+ view tabs +
夹具生成器 + e2e 20 用例。**子代理双轨**（用户指令）：web 实施
子代理并行 + 对抗审查子代理——审查抓出 **2 MUST-FIX**（表名去重
死循环[release 挂死]/截断缺重 trim[断整簿导出]）+ 6 SHOULD-FIX
（extent 契约/文档错位/扁平化登记/写面域钳制/非有限数/测试盲区）
**全部修复 + 3 回归锚测试**。门禁（亲跑）：G1 **743/743** /
fmt 0 / clippy 双面恰 2 基线 / scan PASS / lock 零 diff / wasm32
0 错 / web 全回归（smoke 15 + 四套 42 + contract 18）+ 新 e2e
**20/20**。**下一步 = §八.1 壳与证据界面 wasm 化**（workbench
字节入口 + 内存 sidecar 两前置件）→ §八.4 PDF 表面 → §八.5
Canvas 表面 → wry 壳真机集成。环境第 19 次重置已恢复（rustup
1.99.0+wasm32+clippy+rustfmt + wasm-bindgen CLI 0.2.129 + 双仓
ff 拉齐）。

> **当前态速览（2026-10-22 p5-t1 阶段5启动·历史保留）**：
> 六阶段计划**阶段 5 启动**（mdcore `e8bbc7f` 已推送，G1 **730**）。
> 第一个前置件落地：**表格表面数据面 wasm 出口**（xlsx_grid 模块）——
> WasmBook 常驻句柄 + 可视窗切片（typed kind/原值/display 三件套，
> 窗外 = empty 填充 = 虚拟滚动语义）+ 单格面（编辑定位/证据高亮
> 钩子）；单一真相 = office-io 读面（read_book_from_bytes 同一套
> 安全门），宿主不建第二份表格数据；wasm32 全目标 0 错误 + web pkg
> 已重建。**阶段 5 路线图**（§八 按宿主 A=web 面实施）：⑴表格表面
> UI（虚拟网格产品化——bench.ts 原型 → 生产面：冻结区/单元格编辑/
> 键盘导航/复制粘贴/证据高亮，数据面已就绪）→ ⑵证据界面 wasm 化
> （workbench 字节入口[ingest_xlsx_from_bytes 待加]+内存 sidecar
> [SidecarStore 变体待加]→ 证据侧栏/审核报告/命令进度）→ ⑶PDF 表面
> （渲染路径设计决策待记：pdf.js 渲染 vs PDFium 真相分离）→ ⑷Canvas
> 表面（bench SVG 原型产品化）→ ⑸壳集成（wry 真机 = 选型记录既列
> 前置项）。mdcore main = **e8bbc7f**；cleanroom main = 本提交。

> **当前态速览（2026-10-22 p34-verify 阶段3/4核对）**：
> 六阶段计划**阶段 3/4 核对收口**（mdcore `fc25f06` 已推送，G1
> 728）。§六.1 撤销粒度**实证钉定**（新测试：逐字符 5 命令 = 5 栈项
> [捕获窗 0 = p20 缓解既定代价] + 逐项撤销全链回退正确 + 宿主聚合
> 形态恰 1 栈项——聚合责任在命令构造侧非捕获窗[p20 窗>0+交错
> origin = 缺陷路径]，阶段 5 表面实施项既有限制维持）；§六.2 内存
> 边界（R3 定量 437.9B/op + 软上限 clear_all 显式回执——S0 一页
> 清单在档复验）；§六.3 全部七项对照既有产物（ID 分离/同契约/
> 事件通知/并发冲突/保存恢复/会话本地撤销/真实编辑器组合测试）。
> §七 Canvas：headless 引擎 + e2e_demo 全链复跑 PASS（文档协作/
> 画布协作/证据关联/撤销重做/删除复原/跨端传播/快照恢复）——
> 「真实交互」= 阶段 5 表面项（宿主已定 A：web+wry）。演示件
> collab_body_demo 幕五次序适配阶段 2 门禁（先重验再过审）。
> **阶段 3/4 验收达成**（可重复验证在案）。mdcore main =
> **fc25f06**；cleanroom main = 本提交。**下一步 = 阶段 5 渲染
> 表面**（§八：宿主 A = web 面扩展——壳与证据界面[虚拟化列表 +
> 统一命令入口] / 正文[CM6 既有] / 表格[bench 虚拟网格产品化] /
> PDF / Canvas；wry 壳真机编译 = 选型记录既列前置项）。

> **当前态速览（2026-10-22 p2-r1 阶段2收口）**：
> 六阶段计划**阶段 2 收口——证据与命令基础七项复验全过**（mdcore
> `3d2e578` 已推送，G1 721→**727**）。七项对照：①完整关联链/②版本
> 变化失效识别/④三态分离/⑤导出与 sidecar 独立——既有测试复验过
> （八步闭环 + persistence_recovery 六测 + collab 组合三测）；③指纹/
> 定位不匹配不自动恢复 + ⑦命令统一入口——本轮**加固落地**：可配置
> 审核门槛 `ReviewPolicy` 三维 fail-closed（操作者前缀允许列表默认
> `human:`[AI 不可改审核态] / 失效关联不得直接过审[过审前必须显式
> 重验恢复 Valid] / 主张值门槛[达阈值计算型证据须 calc Passed]）——
> 会话级注入不入 sidecar、Workbench::review 单点接门、命令层映射
> Rejected、6 新测试；测试适配 4 处（先审后验旧序翻转为先验后审
> = 门禁次序守护）。⑥sidecar 丢失/损坏/外部编辑 = wrap-t1 六测
> 复验。**阶段 2 验收达成**：闭环可复现（727 全绿）/ UI 接入无需
> 第二权威数据（CM6 doc = 核投影 + 会话 = 唯一编辑面在案）/ 失效
> 与失败状态明确表达（三态正交 + 门禁拒绝零状态变化）。附带：
> bench_canvas wasm32 占位 main 漂移修复（t21 收严口径漏网——
> wasm32 --all-targets 0 错误复绿）；磁盘满事件第 2 次（LO 解压
> 残留 903M + tool-results 1.2G + debug targets 5.7G 清理——
> proto-native/target 已清，复跑需 rebuild）。mdcore main =
> **3d2e578**；cleanroom main = 本提交。**下一步 = 阶段 3 协作
> 引擎完成实施**（§六：撤销粒度/内存边界既有限制在档；真实编辑
> 器集成测试 = collab_body_demo 既有；按计划核对增量面）。

> **当前态速览（2026-10-22 p1-host 宿主定谳）**：
> 六阶段计划**阶段 1 后半收口——双宿主原型对照完成，主宿主定谳
> = A（web 面 + wry/tao 壳）**，选型记录落档
> `workbench-line/宿主选型记录.md`（实测数字/七维评估/未测量面/
> 降级触发条件全在案）。本轮交付：①共享对照数据集单一真源
> （proto-native --gen，10k 块中文正文/100 主张/500k 格表格/1k 节点
> 画布，确定性 LCG）；②原型 B = egui 0.37+glow+winit/X11 原生候选
> （`workbench-line/proto-native/`，mdcore path 依赖直连 + x11rb
> XTEST 真实输入注入，Xvfb 实跑）；③原型 A 基准页 =
> `mdcore/web/bench.html+bench.ts`（CM6/wasm 核/虚拟网格/SVG 画布，
> 零新 npm 依赖，Playwright CDP 真实输入）；④13 场景双侧对称实测
> 双轮稳定（帧预算：web 四类负载全落 60Hz vsync；原生软渲染表格
> 31.6ms 超预算、画布 21ms；内存 673MB vs 188MB；IME 证据 = web 面
> ime_cdp 28/28 复跑全过）。**未测量面诚实登记**：wry/webkit2gtk
> 壳沙箱不可编译（无 root 装系统库）——A 侧测的是 Chromium 引擎，
> 真机 wry 验证 = 阶段 5 前置项；原生 GPU 复测同列。环境恢复：
> 第 ~20 次重置（工具链 1.99.0 重装 + pdfium SHA256 验证重取 +
> G1 721/721 精确复现 + web 面 15/15）。mdcore main =
> 82884cc+web 基准页提交；cleanroom main = 本提交。**下一步 =
> 阶段 2 证据复验**（七项：XLSX/DOCX 定位关联、失效识别、指纹
> 不匹配不自动恢复、三态分离、导出独立、sidecar 损坏处理、命令
> 入口统一）。

> **当前态速览（2026-10-22 harden-r1 + 新计划接管）**：
> 用户下达《**文档工作台后续完整实施计划**》（六阶段 governing
> 指令，原文落档 `workbench-line/文档工作台完整实施计划.md`）——
> **前端⑤从「悬置裁决」升级为「阶段 1 基准 + 双原型对照后定谳」**。
> 本轮 harden-r1（mdcore `ef830dd` 已推送，G1 711→**721**）：
> ①交付报告 §4 风险②收口——mdc-collab `audit` 模块（应用前审计/
> 在位验证/repair 机械修复 + 端到端对照证明：不修复 = undo 删整
> 节点并发内容全编队丢失，修复 = 并发内容存活）+ set_block_type
> 形状外泄修复（audit 口径反噬发现）+ p18 死锁新面自纠
> （get_or_insert 持读事务调用 = 死锁）；②relay_demo 中枢可运行
> 演示（13 检查点，含检查点恢复诚实语义——快照 = 检查点时刻态）；
> ③上游 issue 草案两份备稿 ready-to-file（p20 bug 报告 + map-undo
> 语义澄清，复现件独立验证亲跑复现，`collab-line/upstream/`，
> **呈用户择机提交**）；④clippy 卫生（integ-r4/r5 漂移 + 新码
> 共 9 警告清零，复归恰 2 基线）。
>
> **五分类现状表（新计划阶段 0 §5 要求——实施侧盘点）**：
> - **已实现（代码在 main）**：14 crate 全家（mdcore MD 引擎 /
>   pdf / office+office-io / evidence / collab / canvas / mdbridge
>   / relay / workbench / settings / shell / wasm）+ web CM6 桥。
> - **已验证（测试/门禁亲跑）**：G1 721 零失败 / clippy 恰 2 基线
>   / fmt 0 / scan PASS / wasm32 0 错 / 三演示（e2e 13 +
>   collab_body 12 + relay 13 检查点）/ sidecar 六测试（wrap-t1）。
> - **部分集成**：web/ CM6 桥仅 MD 面（PDF 面为独立轻量视图，
>   Office/Canvas/证据侧栏无前端面）；shell-native 蓝图态
>   （R22-a..f 未实装）；协作撤销粒度 = 每命令一项（p20 窗 0
>   缓解的代价——宿主侧输入聚合待阶段 5）。
> - **延期（有明确边界与前置）**：宿主选型（阶段 1 双原型对照）；
>   渲染表面四类（阶段 5）；Windows 7 兼容目标（单独，不阻塞
>   主平台）。
> - **未实施**：阶段 1 四类性能基准（长文档/大表格/Canvas 基准
>   缺位——PDF perf 门在位）；阶段 6 端到端产品级验收；打包
>   分发实测（packaging/ 模板在位，未跑真机）。
>
> **下轮起点**：阶段 0 已收口（四测试面亲跑复现 + HANDOFF 五分类
> 表）、阶段 1 前半已收口（**四类性能基线在档**——
> `workbench-line/阶段1性能基线报告.md`：长文档/表格/PDF 6:6/Canvas
> 双档；三项结构性行动项 = 表格提交策略·Canvas 增量读·证据批量
> 通道）。**下一步 = 阶段 1 后半：双宿主原型对照（A：web/+wry 存量
> 复用；B：原生 UI 候选一枚）→ 选型记录 → 定主宿主**，随后阶段 2。
> mdcore main = **82884cc**（harden-r1 ef830dd + p1-bench）；cleanroom
> main = 本提交。
>
> **当前态速览（2026-10-22 集成冲刺全收口·终态）**：
> 用户「继续持续做」——交付报告 §3 六项待集成清单**工程侧全部
> 收口**（G1 642→**711**，四轮实施 + 一轮复验全推送）：
> **①** mdcore↔collab op 对接（前会话 1789493 + 本会话复验
> G1 667 精确复现 + clippy 卫生补丁 d3bbb12）；**③** 事件/订阅面
> （02067ad：双域事件层——延迟派发/一入口一事件/updates 逐事务
> 可转发/canvas touched；21 测试）；**④** awareness 在场层
> （e2dfc7c：presence 模块——null 离线语义全键集发现与修正/
> standalone 画布宿主路径；11 测试）；**②** 协同正文组合
> （80e097a：会话=唯一编辑面/文件=投影——选择性失效/重绑/撤销
> 回退/双端并发/干净导出全链；3 测试 + 五幕 12 检查点可运行演示
> `cargo run -p mdc-workbench --example collab_body_demo`）；
> **⑥** 服务端中继（61f1ea8：第 14 crate mdc-relay——传输无关
> 状态机/SyncReplica 双域门面/权威副本扇出/差量追平/在场聚合/
> 快照检查点恢复；9 测试）。
> 产品面 = mdcore **14 crate**（+mdbridge +relay），G1 **711** 零
> 失败，NOTICES v1.6 恒（零新三方）。
> **唯一剩余 = ⑤ 前端选型（用户裁决项——指令⑤纪律：Headless
> 优先，未选定不得指定框架；宿主经 wire 字节 + 命令接口 + 事件/
> 在场/中继面接入，全部前置件已就绪）**。mdcore main =
> **61f1ea8**；cleanroom main = 本提交。无未推送提交；无未收口
> 阶段。环境注记：第 17 次重置恢复 + 磁盘满事件（已清理复验）。
>
> **当前态速览（2026-10-22 integ-r2 事件/订阅面落地后）**：
> 集成项推进（用户「继续持续做」）：**项③ 事件/订阅面已落地**
> （mdcore `02067ad`：mdc-collab/mdc-canvas events 模块——来源
> 五态与公共入口一一对应/updates 逐事务可转发/一入口一事件/
> 失败不产事件/canvas touched 对象 ID 双路径提取；延迟派发纪律
> [事务释放后派发+回调不可重入+Drop 退订]；21 新测试；G1 688
> 零失败/clippy 恰 2 基线/fmt 0/scan PASS/wasm32 0 错）。同轮：
> integ-r1 推送态复验全绿（G1 667 精确复现）+ mdbridge 测试面
> clippy 漂移卫生补丁（`d3bbb12`）。**剩余待集成四项**（依赖序）：
> ④awareness（在线状态/光标——yrs sync feature 在位未暴露）→
> ②workbench↔协同组合（mdbridge 前置已齐）→ ⑥服务端中继 →
> ⑤前端（用户裁决项维持门控）。mdcore main = **02067ad**；
> cleanroom main = 本提交。下轮从集成项④ 起按依赖序恢复。
>
> **当前态速览（2026-10-22 integ-r1 落地确认 + 推送态复验 + 集成项①收口后）**：
> 环境第 17 次重置恢复（双仓 ff 拉齐 cleanroom 07a9681 / mdcore
> 1789493 + 工具链重装 1.99.0+nightly-2026-10-15+wasm32 + pdfium
> chromium/8086 取件 + fixtures 三件再生成）。**integ-r1 落地确认**
> （mdcore `1789493`，前会话推送但本仓零记录——本轮补账）：第 13
> crate **mdc-mdbridge**——mdcore op 层 ↔ mdc-collab 对接（交付报告
> §3.1 集成项①）：身份映射三面（family u64↔`md:{client}:{n}` 确定性
> 编解码/foreign 持久双向表/孤儿分配）+ plan→shadow 增量→collab
> 命令执行序 + CollabSession 唯一真相源（shadow 纯投影+失败自愈
> 重物化）+ dual_run 双跑锁 + publish 重映射语义；25 新测试。
> **推送态复验（本会话亲跑）**：G1 **667/667** 零失败（= integ-r1
> 声明口径精确复现）/ fmt 0 / scan PASS / wasm32 check --all-targets
> 零错误 / **clippy 发现 integ-r1 漂移**——其提交时 clippy 口径未含
> --all-targets 测试面，mdbridge 测试 7 警告（3 unused import + 4
> 不必要 mut）→ 卫生补丁 `d3bbb12` 清零（保留 81/376 两处真实可变
> 借用）→ 修复后恰 2 wasm_bindgen 基线恒，已推送。文档侧：交付报告
> §3.1 翻转已落地 + 本速览。**剩余待集成五项**（依依赖序推进）：
> ③事件/订阅面（前端宿主推式通知）→ ④awareness（在线状态/光标）
> → ②workbench↔协同组合（mdbridge 前置已齐）→ ⑥服务端中继 →
> ⑤前端（用户裁决项——未选型不得指定框架，维持门控）。mdcore
> main = **d3bbb12**；cleanroom main = 本提交。下轮从集成项③ 起
> 按依赖序恢复。
>
> **当前态速览（2026-10-21 协作与 Canvas 正式交付全链收口·终态）**：
> 用户 governing 指令①-④**全部完成**：①收尾验证（wrap-t1：
> sidecar 持久化/异常恢复——实抓并修复 Workbench::open() 三处恢复
> 保真缺口 + 六测试 + 门禁口径修正）→ ②协作引擎（collab-r1：
> mdc-collab 第 11 crate，23 测试——XmlFragment 载体七操作/命令
> 接口/身份双轨/双栈撤销/同步快照恢复；三处上游观察面产品收口 +
> 撤销耦合防护新发现）→ ③Canvas 引擎（canvas-r1：mdc-canvas 第
> 12 crate，13 测试——图形状独立领域模型/Map 键身份稳定/键级 LWW/
> 证据契约 CanvasObject 定位器；yrs map-undo 交错语义钉定）→
> ④端到端验证（e2e-r1：五幕 12 检查点可运行演示 `cargo run -p
> mdc-canvas --example e2e_demo`）。产品线 = mdcore **12 crate**，
> G1 **642** 零失败，NOTICES v1.6（yrs 闭包入账 + v1.5 后面漂移
> 补账——合规缺口修复 + 纪律恢复）。四分报告（已实现/已验证/
> 仍待集成/实际风险）= `collab-line/协作与Canvas正式交付报告.md`。
> **仍待集成六项**：mdcore op 层对接（BlockId 映射决策）/workbench
> 协同组合/事件订阅面/awareness/前端（未选型纪律维持）/服务端
> 中继。**风险四面**（全缓解有对策）：yrs 上游缺陷族（#676 skip_gc
> /p20 窗 0/map-undo 交错——上游询问候选）/撤销耦合写入方约定/
> D-1·SetMark 重建语义/撤销栈默认无界（建议部署设限）。mdcore
> main = **f91b6be**；cleanroom main = da67696。下轮从报告 §3
> 六项集成面按依赖序恢复（mdcore op 对接为最近前置）。
>
> **当前态速览（2026-10-21 wrap-t1 收尾验证轮后）**：用户新
> governing 指令 = 协作引擎与 Canvas 转为**正式交付目标**（不再
> 是"等真实多端需求"的延期项）。收尾轮完成：JSON sidecar 持久化
> 与异常恢复验证（mdcore **1db4a6c** 已推送）——实抓并修复
> `Workbench::open()` 三处恢复保真缺口（body_doc_id 置空 /
> body_version 与来源版本硬编码重启——版本回退碰撞），新增
> persistence_recovery 六测试（重开保真/跨会话幂等重放/版本单调/
> 四异常 fail-closed/中断发布残留/会话间改写 BodyStale），反向
> 验证成立（旧代码恰使两测试失败）。存量发现并修正：f6aa9e7 的
> wasm32 --all-targets 编译错误（symlink 测试 unix 门控）与
> drop_non_drop 漂移。门禁：G1 600→**606** / clippy 双面恰 2
> 基线零错误 / fmt 0 / scan PASS / wasm32 check --all-targets
> 零错误。**执行序列**（用户指令）：①收尾✅ → ②协作引擎实施
> （适配层产品化：p20 undo/redo 缺陷处置 + 内存边界 + 人类/AI
> 共用命令接口 + 并发 + 对象身份 + 故障恢复 + 可自动验证 E2E，
> 不停留在设计文档）→ ③Canvas 引擎（独立领域模型：节点/连线/
> 位置/尺寸/分组，同一命令接口）→ ④端到端验证（人类/AI/并发/
> 同步/撤销重做/删除复原/证据关联画布对象——可运行演示）。
> 前端不预设（Headless 优先，未选型不得指定框架）；四引擎领域
> 模型独立、不建跨引擎统一 CRDT、核心不依赖 GUI。每阶段完成
> 测试+审查+提交+推送+更新本文件；遇阻塞先做其他可执行任务。
>
> **当前态速览（2026-10-21 修订计划 S0-S5 全收口·终态）**：
> **计划执行完毕，按停止条件停**。演进链：S0 止损（c205fec）→
> S1 三轮（8cd97cf 契约 / c529b49 求值器 / 40e65a7 八步闭环，12
> 门禁全过）→ **S2 写入安全（f6aa9e7）**：PathPolicy 允许根 +
> 符号链接逃逸拒（五路径面全检）、命令信封分发（幂等重放/预期
> 版本/记录规则不毒化重试）、publish 发布单点、规模门（1000/500/
> 16MiB 恰过恰拒）、时间面结构性有界决策 → S3/S4/S5 决策记录
> （`workbench-line/S2-S5收口与阶段决策记录.md`：协作等真实多端
> 需求 / 渲染不阻塞不捆绑 / Canvas 后置——均计划原义无代码）。
> 产品面：mdcore 十 crate（+evidence +workbench，G1 538→**600**，
> 零新三方，NOTICES v1.5 恒）；验收 12 门禁全过（S1 报告 §2 映射
> + S2 门禁 11 升级为命令分发面钉定）。**待用户三项**：①sidecar
> JSON vs SQLite 追认 ②S3/S4 重启条件 ③前端选型与 wasm 接入
> 节奏。下轮会话从本速览恢复；无未推送提交、无未收口阶段。
>
> **当前态速览（2026-10-21 S1 三轮全收口后）**：
> **S1 完结**：契约层（8cd97cf）→ 求值器（c529b49）→ **编排层
> + 八步闭环（40e65a7）**——mdcore 第十 crate mdc-workbench
> （2041 行）组合 evidence + office-io 落地合成样例端到端闭环：
> 导入XLSX→登记DOCX→关联→计算（4210.000 含 rescale 轨迹）→
> 报告（三态分列无汇总列）→ 改写失效检测（来源侧 Stale 级联 +
> 正文侧 claim 锚断——双侧）→ 审核清单（复位拒绝）→ 干净导出
> （先三验证后产出：可解析+零标记+字节同一）。结构性纪律：正文
> 零写入 / AI 提议-程序验证（数值指纹摘录全程序提取，fail-closed
> 八拒绝路径）/ 无任意路径 SQL Shell。验收门禁 12 条显式定义
> 逐条映射测试（workbench-line/S1-证据系统MVP验收报告.md §2）；
> **三项选型决策显式登记待用户追认（§3：sidecar JSON 非 SQLite/
> PDF 接入后置 S4/FNV-1a）**。门禁：G1 538→**595** 零失败 /
> clippy 双面恰 2 基线 / fmt 0 / scan PASS / wasm32 check 过 /
> lock 纯成员块零新三方。**下一步 = S2 写入安全**（s2-r1：路径
> 允许列表/命令信封分发/发布单点/规模限制）→ S3/S4/S5 决策
> 记录轮 → 终收口。
>
> **当前态速览（2026-10-21 S1 第二轮求值器落地后）**：
> S1-R1 契约层之上，**mdc-evidence 表达式白名单求值器 + 复算闭环
> 落地并推送（main 8cd97cf→c529b49）**——expr.rs 白名单文法（四式
> 覆盖：数量×单价/求和/比例/单位换算；函数/幂/科学记法显式拒绝；
> 资源四限 fail-closed）+ 求值轨迹 + 单位传播（加减同类硬拒、
> 复合串仅报告——单位代数不发明）+ **recalc 四步链**（输入可用/
> 现值对照/求值/登记值对照——PrecisionMismatch 捕获篡改与精度
> 规则漂移）+ sidecar 复算级联（失败同因级联 + claimed_value
> 第五步正文主张值对照 + 审计）+ EvidenceRecord.claimed_value
> 加性字段（v1 数据 serde(default) 加载不破）。门禁：G1
> 564→**588**（+24）零失败 / clippy 双面恰 2 基线 / fmt 0 /
> scan PASS / lock 零 diff 零新三方。**下一轮 s1-r3**：工作台
> 编排层（新 crate 依赖倒置注入 office-io）+ 合成样例八步闭环
> （验收门禁 12 条显式定义逐条映射测试）。
>
> **当前态速览（2026-10-21 S1 第一轮 mdc-evidence 契约层落地后）**：
> S0 全收口（见下条）后立即进入 S1 通用基础：**mdcore 第九 crate
> `crates/evidence`（mdc-evidence）契约层落地并推送（main
> c4856f7→8cd97cf）**——DocumentRef/Locator（五变体 A1 域校验）/
> 三态正交（类型级排除单字段 verified）/Decimal 确定性十进制
> （checked_* + 三舍入 + 除法双路径论证）/Evidence+Calculation 双
> 记录（构造不变量 + 单向降级）/Command 信封（幂等/预期版本）/
> sidecar v1（schema 门 + 原子写 + **失效只降不升 + 显式重验唯一
> 恢复通道** + 幂等重放）。纯 serde 零新三方（NOTICES 零增量）。
> 门禁：G1 538→**564**（+26）零失败 / clippy 四面恰 2 基线 /
> fmt 0 / scan 0 / wasm32 过 / lock +8 行包块。**下一轮 s1-r2**：
> 表达式白名单求值器（四式 + CalcStatus 复算闭环）→ s1-r3 合成
> 样例八步端到端闭环（验收门禁 12 条见计划 §九）。
>
> **当前态速览（2026-10-21 修订实施计划 S0 收口后）**：用户新
> governing 指令 = 《AI + Human 文档工作台：修订实施计划》（S0-S5
> 五阶段），主线 = **外挂式文档证据系统**；用户澄清：标书只是需求
> 之一，**S1 优先通用基础功能**（数据契约/sidecar/薄命令层），
> 证据闭环为首个消费者，不为标书特化。**S0 六项全收口（s0-t2/t3，
> commit c205fec）**：①双仓状态核实（cleanroom 7ac6b9e / mdcore
> c4856f7 零接触，环境第 19 次重置恢复）；②p20 复现——**新观测
> 第三副面孔**：debug 口径 undo 阶段 Item::splice u32 下溢当场
> panic（上会话 57/57 为 release 口径——口径差揭穿），三副面孔
> 机制统一（offset=0/clock=0 下溢）；③**捕获窗 1000→0 缓解已执行**
> （硬编码非配置面，仅协作路径，mdcore COALESCE_MS 零接触）；
> ④粒度代价 + 五条未覆盖路径登记（修复已知触发条件≠全面安全）；
> ⑤撤销栈维持无限+监控（p19 定量 437.9B/op、6M≈2.4GiB；低成本
> 逐出面=无，上游 API 呈递稿在档；不虚报不清空）；⑥一页清单
> `collab-line/S0-资源限制与缺口清单.md`。门禁：适配层 **57/57
> 双口径**（debug+release；matrix24+hostile13+anchor6+limit10+
> soak4——soak-3 解钉转交错回归锁 + soak-4 blocks 载体新增）；
> clippy 增量零新（spike 存量 fmt/lint 漂移登记，清收留产品化轮）。
> **决策面变化**：G2 p20 裁 A 已执行；批次 1（D1 实施轮）被 S3
> 后置取代——协作产品化等真实多端需求，XmlFragment/B3.2/栈上限
> 批注不再阻塞主线；批次 3 渲染产品化后置 S4（验证与守护器在档）。
> **下一步 = S1 通用基础**：DocumentRef/Locator/Command/AuditEvent
> 最小稳定契约（三态正交：来源关联/计算/人工审核分开记录）+ 本地
> sidecar 存储（SQLite MVP + 模式版本）+ 薄命令层（幂等键/预期
> 版本/权限审计）+ 合成样例端到端闭环（八步：登记→关联→计算→
> 报告→失效检测→审核→干净导出）。验收门禁 12 条见计划 §九。

> **当前态速览（2026-10-19 五轮自主计划收口后）**：用户「继续
> 至少5轮低成本自决策别问我」五轮全收口（R1-R5 五提交：a5-r1
> 里程碑标签 / a5-r2 p18 D-3 换算完备 PoC / a5-r3 p19 撤销栈
> 定量 / a5-r4 soak+缺陷猎获 / a5-r5 生态快照收口）。**本轮重大
> 交付：yrs 0.28 新缺陷定谳（p20 四配置矩阵）**——双 UndoManager
> + 捕获窗>0 下非连续合并项 undo 仅应用首段 + 后续 redo panic
> （Store::materialize），**适配层现产配置（1000ms 窗）公开 API
> 可达**；上游未报告（最近邻 #674 机制不同）；处置三选项呈批于
> DECISION_GROUPS G2 + yrs-g §5（推荐 A：捕获窗改 0——cfg-2
> 实证干净；撤销粒度语义属产品决策待批注）；钉定测试 soak-3
> 在档（修复/改配置后必失败强制解钉）。**D-3 工程项完备化**
> （p18：c2b/b2c 双向 + 六语料穷举 + ReplaceText 四案逐字相等 +
> p18-F 事务锁纪律——TransactionMut::commit 不释放写锁，锁随
> Drop）。**撤销栈定量**（p19：437.9 B/操作最坏粒度 → 6M 窗口
> ≈ 2.4 GiB/端/文档——G3 栈上限裁决证据增量）。测试面 54→**57**
> （soak 随机收敛/长跑稳定/缺陷钉定三件）。收口门禁亲跑全绿：
> 适配层 57/57 + p15-p20 六探针 ALL PASS。双仓里程碑标签在远端
> （checkpoint/2026-10-19-*）。上游生态：0.28.0 后仅 ywasm 面
> 新提交（不影响本仓），#676 零漂移。**剩余门控 = 批次 1/3/4
> 用户批注 + G2 p20 捕获窗处置三选项**。
> 以下为第 18 次重置恢复时点速览（历史保留）：
>
> **当前态速览（2026-10-19 第 18 次重置恢复 + 停机复验后）**：
> 上一会话「至少10轮」十轮计划已在远端全收口（main=`defc055`，
> R1-R10 十提交全推送：d1p-t1..t5 就绪系列 + ro2-t6/t7 渲染线 +
> yrs-g 上游跟踪 + docs-audit 审计 + final 收口）。本会话环境第
> 18 次重置（本地回滚 ae4aa4d 假 ahead）→ fetch 核实远端真实态
> → ff 恢复 + 工具链重装（1.99.0 minimal + clippy/rustfmt +
> wasm32 target + nightly-2026-10-15[rust-src]，LO 25.2.3.2 系统
> 基线存活零重装，双仓 fileMode false）→ **全门禁恢复复验亲跑
> 全绿**：适配层 **54/54**（matrix25+hostile13+anchor6+limit10）
> + mdcore G1 **538/538** 零失败 + clippy 双面恰 2 wasm_bindgen
> 基线（lib.rs:212/221，另 2 行为摘要行零增量）+ fmt 0 +
> p15/p16/p17 release 复跑 ALL PASS + R-O2 **22 用例 SUITE PASS**
> （10 pass+12 pinned+0 fail，双确定性门恒过）+ scan 双口径
> 0 hit-paths PASS——重建环境完整复现推送终态。#676 重置恢复
> 复查（跟踪纪律项）：**仍 open / updated=2026-09-29 与 yrs-g
> 快照零漂移 / 撤回触发零触发 / yrs 0.28.0 = crates.io 最新钉版
> 维持**。双仓树净零漂移（mdcore=`c4856f7` 零接触维持）。
> **停机条件维持**：类别 1（可执行面）清空，剩余全部 =
> DECISION_SHEET.md 批次 1/3/4 用户裁决门控——按纪律如实报告，
> 不伪造工作。批次 1 批注即开实施轮（预告 3-5 轮）。
> 以下为十轮收口时点速览（历史保留）：
>
> **当前态速览（2026-10-18 十轮就绪冲刺收口后）**：十轮计划
> （用户「继续继续至少10轮」指令）全收口——**D1 就绪系列**
> （d1p-t1..t5：yrs-e 映射矩阵就绪件零死角 + p15 结构实证 +
> p16 操作保真环 + p17 撤销栈复验 + yrs-f 汇总呈批件，全部
> ALL PASS；**实测发现 D-3：yrs XmlText 偏移域 = UTF-8 字节域**，
> 适配层 char↔byte 边界换算工程项）；**渲染线就绪**（ro2-t6：
> LO 26.8.1.1 跨版本复跑——22 用例 21 维持 + 仅 garbage-bytes
> 钉定破裂 = **fail-closed 改进**，#18 复跑证据已就位，REGRESSION
> §10.5；ro2-t7：render_guard.py 三合一守护器[单实例锁/RSS
> 看门狗/超时杀组]四断言 ALL PASS，批次 3 机械项预置完毕，
> §10.6）；**上游跟踪**（yrs-g：#676 仍 open / 0.28.0 = 钉版 /
> 撤回触发零触发 + API 请求候选三项英文稿定稿）；**审计**
> （docs-audit 八项全过零漂移）。收口门禁亲跑全绿：mdcore
> **538/538** + 适配层 **54/54** + p15/p16/p17 ALL PASS + R-O2
> **22 用例 SUITE PASS**（系统 25.2.3.2 基线）。
> 以下为 D1 就绪包四件落档时点速览（历史保留）：
>
> **当前态速览（2026-10-18 D1 就绪包四件落档后）**：D1 就绪系列
> 完成（d1p-t1..t4 + 就绪包汇总 yrs-f）——yrs-e 映射矩阵就绪件
> （BlockTree→XmlFragment 全模型面零死角）+ p15 结构实证（九 kind
> 形状全等/wire 往返/开放键透传/5 锚点邻居编辑稳定）+ p16 操作
> 保真环（7 ops 翻译器全环对端收敛/重叠 marks 拆分/D-1 三断言）
> + p17 撤销栈复验（B3.2 分栈载体正交性同构/redo 新 ID 族/双轴
> 身份解耦）全部 ALL PASS；**实测发现 D-3：yrs XmlText 偏移域 =
> UTF-8 字节域**（适配层 char↔byte 边界换算工程项）。#676 复查
> 仍 open（2026-10-18）/ yrs 0.28.0 = crates.io 最新。DECISION_SHEET
> 批次 1 已插就绪注记——**批注即开实施轮**（预告 3-5 轮）。
> 批次 2 全链闭环维持（合入 c4856f7 + 联动 r-o2-t5 锚点存活轮
> 22 用例 204 断言 SUITE PASS）。
> 以下为 2026-10-17 批次 2 后速览（历史保留）：
>
> **当前态速览（2026-10-17 批次 2 裁决执行 + 联动项收口后）**：
> **DECISIONS #8 已裁决执行**——用户裁 Option A Phase 1+2 全批，
> office-g-phase1-anchor 经合并前置检查（merge-base=main HEAD /
> main 独有提交 0 / 差异 4 文件 +398/-3 零公开模型触碰）ff 合入
> mdcore main（`b15c0ab→c4856f7`）+ 全门禁绿复验（**G1 538/538**
> / clippy 双面恰 2 wasm_bindgen 基线 / fmt 0 / scan PASS / lock
> 零 diff / wasm32 check 过）+ 已推送；**锚点写读对称达成（读 4/
> 写 4），R4.5 强绑定升级路径开通**；§13 联动项**已执行**（r-o2-t5
> 锚点存活轮：R-O2 套件 21→22 用例/204 断言——w:anchor → GOTO
> 注记存活指向书签页定谳 + 书签不导出命名目的地治理发现，
> REGRESSION §10，双次复跑 SUITE PASS）。
> **剩余门控 = DECISION_SHEET.md 批次 1/3/4**（批次 1 D1 解锁包三件
> = 最优先；批次 3 Q3 渲染管线；批次 4 低优先合批）。第 16 次环境
> 重置已恢复（双仓 ff 拉齐 + 工具链重装 + 门禁亲跑复验）。上一冲刺
> 五件交付（decis-g1 + d1e-t1/t2/t3 + r-o2-t4 + r5-t3）与收口件
> （DECISION_SHEET.md）已推送在档（本仓 main=`b3a6faf` 基础上）。
> 本轮五件：①**决策组映射**
> （decis-g1：17 项 → G1 载体与边界/G2 撤销与锚点身份/G3 栈与资源
> 上限/G4 Office 锚点写 API/G5 渲染器部署治理，每组默认/备选/后果/
> 不可推断四件套）；②**适配层撤销×锚点回归**（d1e-t1：六面回归锁
> + redo_depth 观测面 + 复活新 ID 族跨端收敛钉定，44→54 测试 ASan
> 零报告）；③**资源限全覆盖**（d1e-t2 + mdcore r5-t3：适配层 3
> 缺口补检[RemoveBlock 键/bind 语义锚/encode_diff SV] + 10 测试全
> 入口边界矩阵；mdcore 写面 2 真缺口[tooltip 255 引擎镜像/write_row
> 列域]——G1 531）；④**R-O2 扩展**（r-o2-t4：14→21 用例/188 断言
> ——转换失败三面 + 畸形补充四面 + 跨进程确定性门[字节不同一钉定/
> 内容同一硬断言] + 内容嗅探改道发现）；⑤**锚点 Phase 1+2 就绪
> 补丁**（d1e-t3：mdcore 隔离分支 office-g-phase1-anchor@c4856f7 +
> office-line/*.patch 评审件——裁决 = 合入 main 一句话）。mdcore
> main = b15c0ab（r5-t3 写面补检）；环境第 14 次重置已恢复（1.99.0
> + nightly-ASan + LO 25.2.3.2）。
>
> **前一加固会话速览（2026-10-17：r-o2-t3 + d1d-t1/t2 + anchor-t2/t3
> + md-deci-t1 + e-review + e-gates）**：office 开发轮全线完成（mdcore main=00ad88b，本轮
> 全门禁亲跑复验零漂移：G1 530/0/1i + PDF 四面 68/79/81+6i/109+6i +
> perf 6/6 + clippy 双面恰 2 基线 + fmt/scan 0 + wasm32 双面 + web
> 七套件 18/15/28/11/13/9/9 + Cargo.lock 零 diff）。**本轮五件交付**：
> ①**R-O2 回归资产**（r-o2-t3：spike → 14 用例确定性 CI 套件[9 硬
> 断言 + 5 畸形钉定，131 断言 ~16s，exit 语义完整] + 超时治理进程组
> 杀除零残留 + 冷暖一致性硬门[e-review 收严] + 两条新发现[垃圾字节
> 文本导入兜底 rc=0 乱码产出/加载失败 rc=0——检出策略=产物断言
> 通则] + A5/A4 命名勘误）。②**适配层加固**（d1d-t1/t2 + yrs-d：
> 测试矩阵资产化 matrix 25 + hostile 13[含 ASan 全绿零报告] + p12
> 循环测量[20 轮同步/撤销全收敛 RSS 恒定 8.2MB，作者端墓碑 undo/redo
> 钉住 0% vs 对端 2% 观察，redo 返回栈项=B3.4 新形态] + p13
> XmlFragment 七面深验全过[D1 载体证据链补全] + p14 GC 证据实证
> [wire 恰缩 55B=Deleted 标记；公开 API 盲区；**maintenance_gc 与
> 未授权 GC 同族 wire 不可区分**——强制面=构造+部署+快照门定谳] +
> 三新发现[undo 复活=新 ID 族→锚点跟随需 redone 链映射上游候选/
> client_id 53 位 release 静默污染→已 fail-closed/死锁纪律三实例
> 复登] + e-review 三缺陷修复[侧车敌意 count 预分配 abort→物理上界
> 钳制+B1.6 收口/InsertBlockBefore 限额绕过→补检/hostile 回归锁]）。
> ③**锚点写面完整设计**（anchor-t2/t3：office-g §7-§13 六维度[调用
> 示例/三层身份模型/生命周期五行/迁移零破坏/校验 V1-V8/四备选最小
> 性] + e-review 勘误[Phase 2 新增 DocxError::InvalidInput 变体显式
> 登记]）——仍零代码变更，Phase 1/2 待裁决触发。④**裁决包三分类**
> （md-deci-t1：DECISIONS.md 20 项单一登记表[类别 1 恰一项已实施=
> STATUS §7-6 陈旧修正/类别 2 两件完备提案/类别 3 十七项精确决策
> 问题] + 三组双线联动复登）。⑤**独立审查**（e-review：五项清单
> 对抗审查四产物，三 SHOULD-FIX 全修复回归锁，全部数字断言逐位
> 复现）。**剩余门控全部收敛于 DECISIONS.md**（类别 3 十七项：D1
> 三裁决[载体证据链已完备]/D2/D3/R-O2 Q3[新增检出策略输入]/锚点
> Phase 触发/Q-R1-R3/t22 ①②④⑦/R-5/R-6/fuzz 排期/LO 26.8.1 复跑/
> 上游跟踪[#676 + 两项 API 请求候选]）。环境第 13 次重置已恢复
> （1.99.0 + nightly-2026-10-15 ASan + pdfium 8086 + fixtures 四件
> + web 全链）。凭据若再失效：全部成果已双仓推送，本地零未推送
> 状态。

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
