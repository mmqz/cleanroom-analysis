# MD 引擎线（mdcore）状态登记 — 2026-10-10

> 本文件由 MD 引擎研发会话登记（Task 24），把 **MD 引擎线**的结论层同步进合并仓，
> 使新代理只看本仓 + `mmqz/mdcore` 即可掌握全貌。**分区铁律**：审计底稿（解密源
> 对照针集、命中明细、四源材料）只在私有仓 `mmqz/typora-re-analysis`，永不外移；
> 本目录只存结论与计划。

## 1. MD 引擎是什么

用户 AI 工作台的 **MD 引擎组成面**（对应 `pdf-line/05` §1 组成面盘点表中的
「MD 引擎：开发中」）——实际进度已远超"开发中"：

- **架构**：mdcore 编辑核（Rust/ropey 四 crate）+ CM6 web 桥（wasm-bindgen）
  + 宿主壳（tao+wry+GTK3，ADR-H1——Slint 因 GPL 证与 ADR-C1 排除）
- **核心资产**：Document Model + Command Boundary + Stable Block Identity +
  Revision/Anchor + Human+AI 共用状态（commit_transaction_as Source::Ai 溯源）
- **门禁终态（R23-e）**：cargo 148/148、wasm 过、smoke 15/15、契约门 13/13、
  IME CDP 28/28、产品面品牌词 0、SBOM 367 项全白名单、双 profile 反编译自检 CLEAN

## 2. 合规终态（双门证据链闭合）

- **零痕迹 + 零雷同**双门全过：四路审计全 CLEAN（Zed 系 GPL 全量源 0 命中 /
  Typora 解密源 298 针 0 violation / delta 81 万行 302 针 0 violation /
  Obsidian d.ts 136 针 0 命中）。
- ADR-C1 闭源最严姿态：GPL/AGPL 零容忍、上游特有命名零容忍、同功能异结构
  （改名式合规不构成免责）。
- 常驻门禁：`compliance/scan.sh`（denylist 20 词四源）每提交必跑。

## 3. 仓库地图（MD 线）

| 仓 | 性质 | 内容 |
|---|---|---|
| `mmqz/mdcore`（私有） | **产品仓** | main=66d1ea6 三提交链（快照基线→compliance 基建→README/政策）；compliance-baseline=3843156；mother/r23-snapshot=24268ca |
| `mmqz/typora-re-analysis`（私有） | 研究/审计底稿仓 | 主档 `project_panorama_master_2026-10-07.md`（§31 R24 编排 / §32 PDF 裁决）、`plans/pdf_carrier_plan_2026-10-10.md`、`projects/prototype/gates.md`、`decisions/`（ADR-B1~B7/C1/H1/View-01）、compliance_rounds//fullcross//behavior/ |
| 本仓 `md-line/` | 结论层同步 | 本状态文件 + PDF 载体工程计划副本 |

## 4. R24 产品化编排（主档 §31 摘录）

**主题**：打包分发 / 多窗口 / 设置面。五轮 a-e；每轮收口双绿
（`compliance/scan.sh` + `cargo test --workspace`）；契约变更走 CONTRACT v1
升版纪律。

| 轮 | 主题 | 关键点 |
|---|---|---|
| R24-a | 设置面核侧 | schema 分层（editor/ai/storage/appearance）、TOML sidecar、默认合并/未知键前向兼容/损坏降级；CONTRACT v1.1 |
| R24-b | 多窗口核侧 | SessionManager、窗口↔document_id 映射、资源上限、会话持久化（恢复上次打开集）；CONTRACT v1.1 session 面 |
| R24-c | 打包分发基建 | CARGO_PKG_VERSION 单源、AppImage/deb/NSIS/.app 配置落 `packaging/`（实体验证留 d）、分发合规核对单 |
| R24-d | 宿主壳合入（**开发环境轮**） | R22-a..f 实体按 CONTRACT v1 合入 main、settings/多窗口接线、三平台产物冒烟、R20 路径 3 复测、**真机 IME 人工验收执行**（协议 web/README §6） |
| R24-e | 阶段收口 | 门禁终态全复跑、SBOM/NOTICES 增量（打包新依赖）、主档销项、双仓推送 |

## 5. PDF 载体工程计划（已裁决提前）

- **用户裁决（2026-10-10）**：PDF 从格式扩展序列第 4 位提前为第二载体；HTML/
  DOCX/XLSX 顺延登记。
- **计划**：见 `md-line/pdf_carrier_plan_2026-10-10.md`（五轮 PDF-a 侦察选型
  spike / PDF-b mdc-pdf 骨架 / PDF-c 阅读面 / PDF-d 标注面+Acrobat 互通 /
  PDF-e 收口）。
- **与 L2 的衔接**：`pdf-line/01~04`（PdfCraft 净室分析）= 分析别人怎么做；
  本计划 = 我们怎么做。L2 的 17 维方法论、敌意输入边界（03 篇：限额五层纵深/
  三行为学/沙箱四件套）直接作为 PDF-a 安全策略表的输入资产。
- **并行性**：PDF-a/b 与 R24-a/b/c 无 crate 面冲突，可并行。

## 6. 遗留四项定性（主档 §31 定谳）

| 项 | 定性 | 时机 |
|---|---|---|
| 真机 IME 人工验收 | 人工物理项 | R24-d 顺带 |
| 定名后商标检索 | 用户决策前置 | **R24-c 前**（阻塞打包命名） |
| R20 路径 3 ingest 预分配 | 性能复测 | R24-d |
| log 内存杠杆 + merge v1（ADR-B7 §4） | 功能增强 | R24 后排队 |

## 7. MD 线当前可开工项

1. ~~**R24-a 设置面核侧**~~ ✅ **已完成（2026-10-10，mdcore main=2de27a2）**：
   mdshell::settings 四层 schema + TOML sidecar + CONTRACT v1.1
   （`__settings`/`__setSetting`/`mdc-settings-changed`）；语义锁 14 条；
   门禁 cargo 164/164 + 契约门 16/16；NOTICES v1.1 增补（toml_edit 族 8 项，
   compliance-baseline=8c26272）。
2. **R24-b 多窗口核侧**（本环境可做，不依赖壳实体；CONTRACT v1.1 增 session 面）
3. **R24-c 打包基建**（同上，产物验证留 R24-d；**前置阻塞：定名商标检索**）
4. **PDF-a 侦察收口 + spike**（与上述并行无冲突）
5. R24-d 壳实体合入 = **开发环境轮**（需要 R22-a..f 实体所在环境执行）
