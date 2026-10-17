# 净室分析系列成果（Cleanroom Analysis）— 单一总仓库

AI 工作台引擎净室分析研究的**全部工作成果**集中仓库（用户指定单一入口）。覆盖三条已完成审计/分析线 + 一条待启动规划线。

净室纪律：全部分析仅基于公开一手源（官方仓库、文档、标准、CVE/厂商公告、可复核的 API 与 git 元数据）；不接触闭源代码；关键结论带出处交叉验证；二手数据仅作旁证并显式标注立场。

## 仓库结构

| 路径 | 内容 | 规模 |
|------|------|------|
| [HANDOFF.md](HANDOFF.md) | **新会话 AI 接手指南**（项目状态/纪律/下一步/环境备忘） | — |
| [worklog.md](worklog.md) | 全程工作日志（事实登记，追加式） | 60+ KB |
| [pdf-line/](pdf-line/) | L2·PdfCraft 净室分析四篇 + L3·办公/画布线规划 | 5 文档 1765 行 |
| [agents-md-audit/](agents-md-audit/) | L0·AGENTS.md 架构级安全审计（9 分析 + 5 证据 + 5 工具） | 20 文件 |
| [acrobat-audit/](acrobat-audit/) | L1·Acrobat Pro DC 安装器审计 R1~R6（4 分析 + 25 证据 + 工具链 + 哈希登记） | 70+ 文件 |
| [tools/push_to_github.sh](tools/push_to_github.sh) | 零落盘推送脚本（token 仅内存单次传递） | — |

## 三线成果速览

### L0 · AGENTS.md 规范审计（2026-10-08~09）
vercel/AGENTS.md 等规范的全量架构级安全审计：供应链验证 7 仓库、提示注入面、防御性代码逐条核查（vercel 凭据黑名单、skillguard 扫描器等）、9 篇分析 + 可复现工具。详见 `agents-md-audit/README.md`。

### L1 · Acrobat 安装器审计（2026-10-09）
Acrobat Pro DC 26.002.21931 安装器（TLS+MD5+Authenticode 四重验证）黑盒取证：MSI 数据库解构、36 PE 能力矩阵（零网络导入/零注入 API）、R6 AGS 取证（ARM 活体通道全链验签、gc_dispatchtable 网络面全景、SKIP_GC=1 官方阻断开关）。二进制原件在 [mmqz/acrobat-pro-audit](https://github.com/mmqz/acrobat-pro-audit)，本仓收分析与证据。详见 `acrobat-audit/README.md`。

### L2 · PdfCraft 净室分析（2026-10-09~10，已收官）
- `01` 架构深度剖析（359 行）：组件架构 / crate 全景 / 治理三层宪法 / 质量门
- `02` 开源 PDF 生态缺口与差异项（430 行）：六代引擎谱系 / 缺口矩阵 / 差异化机会 / 许可证终审
- `03` 敌意输入与执行边界（344 行）：vendor 35 处补丁分类学 / 限额五层纵深 / JS 沙箱 / fuzz 战役
- `04` 收官报告（244 行）：17 维评估矩阵 / 最终判定（净室开发成立）/ 增量跟踪口径

### L3 · 办公与画布线（2026-10-10~15 全阶段收官）+ collab 线
- `05` 规划文档（388 行）：16 候选总览、11 维方法论裁剪、批次表、排除清单
- **14 篇深读/pass 报告**（`canvas-line/` 7 + `office-line/` 4 + `office-e` 选型 + `pdf-line/06` 综合与裁决建议 458 行）全部经独立复审+修补；终裁问题 D1/D2/D3 留用户（06 §8.1）
- **collab 线**（`collab-line/`）：`collab-a` Yjs 协议 pass + `affine-a` AFFiNE 双态 pass + **`yrs-a` yrs 评估 pass（2026-10-16：MIT 定谳 / 双语言 wire 互证实证 / spike 六探针 / 批量装载内存放大发现 / 闭包 45 包零 copyleft——D1② 证据就绪）** + **`yrs-b` D1-a 集成验证边界（2026-10-17：#676 定谳链闭环[ASan 复现/混合部署证伪/skip_gc 全编队不变量] + 四项边界[GC/UAF 隔离·事务粒度·撤销栈归属·稳定锚点跨模型映射] + 适配层契约草案——用户裁 D1② 走 a 后产出，实施轮待边界确认）** + **`yrs-c` D1-a 适配层原型报告（2026-10-17 d1c-t1..t4：collab-adapter 原型 crate[crate 边界=可撤回边界，公开 API 22 fn 零 yrs 类型] + 用户三验证面闭环[内存上限/撤销语义/跨模型锚点映射] + P8-P10 探针 + ASan 门禁零报告 + P11 载体对照[XmlFragment vs Array-of-Map 六维，建议倾向 XmlFragment 呈裁决]——用户"有条件确认边界"执行轮）**
- **office 线**：office-f docx 渲染级读侧评估（2026-10-17：生态盘点一手核验 + LO headless 转换 spike 全过[显式分页保真硬断言] + 三路线对照与裁决问题——O1-6 开放项证据闭环，`office-line/office-f_docx渲染级读侧评估pass.md`） + office-f §8 复杂夹具扩展（2026-10-17 r-o2-t2：用户裁 C 路线进原型阶段后六类要素夹具全过[跨页表格/页眉页脚逐页/分节逐页尺寸/缺失字体替换/图片/组合] + 三工程发现[w:drawing 必须 run 级 LO 静默丢弃/CJK 空格归一化/真件对照判别法]） + office-g 锚点写面最小扩展方案（2026-10-17：用户裁③"暂维持公开 API 方案先行"执行——Phase 0 生效/Phase 1 RunSpec.anchor 实施就绪规格/Phase 2 bookmark_paragraph 正交件/兼容性一手全量清单[字面 4 处=1 免疫+3 机械补]/yrs-b R4.5 弱→强绑定升级路径）
- office 开发轮全线（office-impl/xlsx-r1/xlsx-r2/docx-r1/r2/消费面 t16-t22/Round 4/Round 5）见 `office-line/` 各 REPORT 与 `HANDOFF.md` L3 行

## 新 AI / 新读者入口

- **AI 接手**：读 `HANDOFF.md` → `worklog.md` 尾部 → `pdf-line/05` §3/§7
- **人类阅读**：`pdf-line/` 01→02→03→04 为 PDF 线闭环，05 为下一批规划；两条审计线各有子目录 README

## 关联仓库（原始推送存档）

- [mmqz/agents-md-audit](https://github.com/mmqz/agents-md-audit) — L0 原始仓（内容已并入本仓）
- [mmqz/acrobat-pro-audit](https://github.com/mmqz/acrobat-pro-audit) — L1 原始仓（含 Adobe 二进制原件，内容已并入本仓）
