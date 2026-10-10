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
| L3 | 办公与画布线（Univer/Excalidraw/ComfyUI 等 16 候选） | 📋 规划完成，**R1 待启动** | `pdf-line/05`（规划文档，P0 批次表在 §3/§7） |

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

## 6. 下一步任务（用户已确认方向，待开工）

L3 办公/画布线 **R1**，P0 三项（详见 `pdf-line/05` §7 批次表）：
- **office-a**：Univer 深读（The Office Harness for AI Agents，Apache-2.0）
- **canvas-a**：Excalidraw 深读（MIT）
- **canvas-d**：ComfyUI 协议 pass（Nodes 2.0 / Vue 架构）

R1 内容 = 仓库克隆 + 代码勘察（结构/治理/测试/许可），按 11 维裁剪矩阵输出。动画线 P3 暂缓。同法排 R2~R5。

## 7. 环境备忘

- 工作区：`/home/z/my-project/`；本地交付目录 `download/`；本仓本地克隆 `/home/z/my-project/cleanroom-analysis/`
- GitHub API 直连易限流：旁路 = 网页 HTML 抓取（`github.com/<user>?tab=repositories`）、`git ls-remote`、shields.io 徽章
- 克隆大仓用浅克隆（`--depth 1`）；上游仓存 `agents-audit/_upstream/`（本地，不进 git）
- 技能调用规则、图表生成、文档产出等系统级约束按你自己的系统提示执行
