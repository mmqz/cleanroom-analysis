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
| L3 | 办公与画布线（Univer/Excalidraw/ComfyUI 等 16 候选） | ✅ R1 P0 三项 + R2-P1 四项 + **R2-P2 前半五项全部完成**（2026-10-11，canvas-c 792 行评级表 / canvas-e 365 行五路线光谱 / canvas-f 550 行双强度灰度协议 / penpot-a 426 行三层混合渲染+wasm 二线 / affine-a 140 行双视图投影+build-vs-buy 核验；三次代理故障均按写/审分离纪律补救重试；均经独立复审+修补收口）；**剩余：office-d 写侧管线 pass + 读侧对照（轮5）+ R2 总收口综合** | `pdf-line/05`（规划）；`canvas-line/` + `office-line/` + `collab-line/`（十三篇报告） |
| L-MD | **MD 引擎（mdcore）研发与合规**（另一会话线，2026-10-10 登记） | ✅ R24 全收口 + **PDF 载体五轮 a-e 全部完成**（a 选型定谳 / b 骨架 / c 阅读面+性能门 / d 标注面+增量写互通 / e 收口：安全姿态对齐实装 8 族扫描+S6 form_probe+看门狗换代+L0/L4/L7，CONTRACT v1.3）；前置质量审计轮 T1（survey/review/4-fix/verify 链：F1 取消哨兵/F2 ByAnchor panic/O3/O4 修复+工具链钉住+fmt 清扫）；SBOM/NOTICES v1.2（42→105 零 UNKNOWN，ASSET_CHECK 首个分发资产）；S6 探针复原（.gitignore 根因修复）；R24-d 壳实体 = 开发环境轮；mdcore main=`03815b6`（PDF-e 已提交：四面 259/78/80+6/108+6 + perf 6/6，scan 0 恒绿）；compliance-baseline=`1adced2`（NOTICES v1.2 在档） | `md-line/STATUS.md` + `md-line/pdf_carrier_plan.md` + `md-line/pdf-a/`；产品代码在私有仓 `mmqz/mdcore` |

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
