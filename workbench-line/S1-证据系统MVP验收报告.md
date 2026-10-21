# S1 证据系统 MVP 验收报告（三轮收口）

> 2026-10-21 s1-r3 落档。governing 指令 = 用户《AI + Human 文档
> 工作台：修订实施计划》（S0-S5）+ 澄清"通用基础优先，标书只是
> 需求之一"。产品代码：mdcore `crates/evidence`（mdc-evidence，
> s1-r1/r2）+ `crates/workbench`（mdc-workbench，s1-r3）。
> 提交链：8cd97cf（r1 契约）→ c529b49（r2 求值器）→ 40e65a7
> （r3 编排+闭环）。

## 1. 交付物

| 轮 | 内容 | 提交 |
|---|---|---|
| s1-r1 | 数据契约（DocumentRef/Locator 五变体/三态正交/Decimal/Evidence+Calculation 双记录/Command 信封/AuditEvent）+ sidecar v1 | 8cd97cf |
| s1-r2 | 表达式白名单求值器（四式/资源四限/轨迹/单位传播）+ recalc 四步链 + sidecar 复算级联 + claimed_value 加性字段 | c529b49 |
| s1-r3 | mdc-workbench 编排层（第十 crate）+ 合成样例八步端到端闭环 + 审核动作 + 正文侧失效/恢复双侧通道 + 干净导出三验证 | 40e65a7 |

门禁：G1 538→**595**（evidence 26→53 + workbench 0→4）零失败；
clippy native+wasm32 双面恰 2 wasm_bindgen 在档基线；fmt 0；scan
0 hit-paths；wasm32 check --all-targets 过；Cargo.lock +8+9 行均纯
新成员块，**零新三方依赖**（NOTICES v1.5 恒）。

## 2. 验收门禁 12 条（§九）与测试映射

> 计划原文经会话下发；本表按计划原则逐条映射到可执行测试
> （mdcore crates/workbench/tests/eight_step_loop.rs 等）。

| # | 门禁 | 执行锚点 |
|---|---|---|
| 1 | 正文在 Word/WPS 正常打开且无工作台痕迹 | `eight_step_closed_loop`：导出=正文原样字节 + 结构完整（5 块）；`clean_export_detects_markers`：污染正文被拒 + 失败零产出；结构性保证：workbench 对正文零写 API |
| 2 | sidecar 独立（不内嵌正文、可分离） | `eight_step_closed_loop` 末段：删除 sidecar 后正文与导出照常读取 |
| 3 | 关键数字可溯源（定位器+摘录+指纹） | `propose_binding`：claim 段落定位器 + source 单元格定位器 + 程序构造摘录（值+公式双通道指纹） |
| 4 | 计算保留输入/公式/精度可复算 | `CalculationRecord` 全留痕（输入+表达式+精度规则+中间轨迹+登记值）+ `recalc_current` Passed + 报告含 rescale 轨迹行 |
| 5 | 来源变更 → 证据失效（只降不升） | `eight_step_closed_loop`：C7 改写 → 全部证据 Stale；`degrade_linkage` 单向语义（r1 单测钉定） |
| 6 | 正文变更 → 主张锚失效 | `source_correction_full_recovery_path`：小计段改写 → refresh_body → claim 锚断 Stale（与来源侧正交，审计区分 side） |
| 7 | 恢复唯一通道 = 显式重验（无静默重绑） | `reverify_source`/`reverify_claim` 双侧通道 + 恢复路径测试全链（重验→复算→新证→过审）；任何 →Valid 直赋仅存在于两重验函数（审计前置） |
| 8 | 三态分离呈现（无单一 verified） | `render_report` 三列分列 + 刻意无汇总列；失效不改审核面（Stale 后 pending 仍 2）类型级排除（status 模块） |
| 9 | AI 只提关联和推导；程序验证；不可验证 → 待人工 | 全部提议落 `Pending`（验证通过 ≠ 过审）；数值/指纹/摘录程序从真实文件提取；fail-closed 矩阵测试（八拒绝路径） |
| 10 | 审核清单 + 人工通过/拒绝留痕 | `review`（Pending→Approved/Rejected，改判允许，复位拒绝）+ 审计 `evidence_review`；审计序号单调无空洞断言 |
| 11 | 无任意路径/SQL/Shell | 结构性：两 crate 零进程/网络/查询面；路径显式入参+会话注册表；S2 命令层加允许列表与信封（下轮） |
| 12 | 合成样例，零真实敏感资料 | 全部测试夹具合成（temp_dir + 合成 XLSX/DOCX）；sidecar 敏感面纪律：零网络代码（结构性） |

## 3. 选型决策登记（与计划原文的偏差，显式呈报）

1. **sidecar 存储 = JSON 文件，非计划原文"SQLite 默认"**。
   决策依据：①零新三方纪律（NOTICES 零增量是 office 线以来的
   合规门禁，SQLite 经 rusqlite 引入 libsqlite3-sys 编译闭包）；
   ②sidecar 与正文同目录、一文档一文件、可随文档携带/删除（G2
   独立性最自然形态）；③MVP 规模（每 sidecar 几十条记录）查询
   面够用。功能等价面全落地：模式版本门/原子写/失效/重验/幂等
   重放。升级路径：若记录规模或查询需求超阈值，Store 抽象 +
   SQLite 后端（S2+ 评估）。**此项待用户追认或指示改向。**
2. **PDF 只读来源 = 契约就位、接入后置**。计划原文"PDF 只读
   可选"——`Locator::PdfPageText` 已在契约（r1），真实 PDF 提取
   接入按 S4 节奏（不阻塞 MVP）。
3. **指纹 = FNV-1a 64**（非密码学，算法前缀编码，sha256 升级
   路径在档）——r1 决策维持，威胁模型 = 意外失效检测而非对抗
   篡改。

## 4. 八步闭环运行实录（合成样例）

1. **导入 XLSX**：`ingest_source`（语义指纹 = 非空格
   sheet|A1|值表示 序）
2. **登记 DOCX**：`Workbench::start`（正文指纹 = 段落规范化文本）
3. **关联**：`propose_binding` ×2（引用型 e-quote / 计算型
   e-calc）——全部 Pending，摘录程序构造
4. **计算**：`propose_calculation`（qty×price=4210.000，目标标度
   3 含 rescale 轨迹）+ `recalc_current` → e-calc.calc=Passed
5. **报告**：三态分列 + 计算详情 + 求值轨迹
6. **改写失效检测**：C7 336.8→400.0 → 重登记 → 全部 Stale →
   复算级联 InputUnavailable（linkage 门槛）+ 计算记录
   InputMismatch（现值对照）
7. **审核清单**：pending 2（三态正交实证）→ e-calc 人工 Rejected
   → 复位拒绝
8. **干净导出**：三验证（可解析+零标记+字节同一）→ 先验证后
   产出 → sidecar 可删独立

恢复路径（另一测试）：重验→旧记录 InputMismatch→旧证据拒→新
计算 5000.00→正文更正 claim 锚断→新证据→复算通过→过审——
§四.5 双侧（来源/正文）+ §四.3（人工归宿）全链闭环。

## 5. S1 后剩余（按计划顺序）

- **S2 写入安全**（下一轮 s2-r1）：路径允许列表（PathPolicy
  根目录约束+逃逸检查）、命令信封分发（幂等键+预期版本接线到
  八步操作）、临时区生成后发布（已有 save/export 原子写——
  归一为单点 publish 纪律）、规模时间限制（evidence/calc 条数、
  sidecar 字节上限——与 SECURITY_POLICY 限额表同纪律）。
- **S3 协作引擎可选子系统**：S0 缓解后产品化再定（XmlFragment
  候选在档）——决策记录轮。
- **S4 DOCX→PDF**：不阻塞 MVP、不捆绑 LO——决策记录轮
  （R-O2 验证件与守护器在档）。
- **S5 Canvas**：后置——决策记录轮。
