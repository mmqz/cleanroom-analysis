# MD 现有模块复用审计报告（REPORT.md）

> 阶段：P7 汇总（PLAN v1.2）。审计对象：mmqz/mdcore main=03815b6（本轮克隆，
> 全程只读零改动）。执行轮：2026-10-12。证据基线：P0 门禁九项全绿复现 +
> B1-B14 对账（baseline.md）+ P1-P6 六份阶段产物（p1/p2/p3/p4/p5/p6），
> 均经独立复审代理核验：P2-P5 全量锚点零事实性错锚；P1 勘误经 P2/P5
> 复核回写；P6 复审由 P7 终审代理合并覆盖；复审轻微项均已回写。
> 补验（同执行轮）：U-1 web 三门禁已复跑全绿（§6/§7），mdcore 仍零改动。
> 定位：**复用事实与方案输入，不是架构终裁**——D1/D2/D3 仅记录相关发现
> （用户 2026-10-12 指令），最终裁决由用户作出。

## 0. 三必答问题（逐条回答）

**Q1：MD 现有模块中，哪些可直接复用、哪些需要最小适配、哪些不可复用？**
→ 见 §2 三分类清单（12 项：5 可直接复用 / 5 需要最小适配 / 2 不可复用[域外]）。

**Q2：每一项的适配成本与验证标准是什么？**
→ 见 §2 逐项四要素（代码证据 / 适配成本 / 对 PDF 质量的影响 / 验证标准）；
通用验证基线 = P0 已复现的九项门禁（默认 259 + lopdf 78 + pdfium 80+6i +
dual 108+6i + perf 6/6 + scan 0 + wasm32 + clippy 基线 + fmt）。

**Q3：模块复用对 PDF 质量有什么影响？**
→ 总判定【代码确认】：**mdc-pdf 与 mdcore 零编译依赖**（pdf/Cargo.toml 无
mdcore 项），PDF 的布局/分页/格式保真由 pdfium（固定版式引擎）与 mdc-pdf
自有代码（lopdf 增量写/缓存/工人/安全面）决定，MD 模块不在任何 PDF 质量
守护测试的被测对象里（P6 Q1-Q8 逐项锚点）。唯三耦合点：① `Source` 五值
**语义镜像**（非类型复用）——镜像锁只校验既有五值在场，mdcore 若新增第六
值会**单向静默漂移**（pdfb_skeleton.rs:1040-1079 只读文件文本锁五值）；
② `mdcore://` scheme 命名共享（CONTRACT §2/§6 词面）；③ workspace 版本
单源。§2 逐项的"对 PDF 质量的影响"栏均以此为基线判定。

## 1. 审计对象与方法

- 对象 = P1-P6 实测模块清单：mdcore 块核（13 模块）/ mdcore-wasm 桥
  （23 导出）/ mdshell（session+settings）/ mdc-pdf（15 模块，仅作复用
  先例与质量基线）/ 治理面（CONTRACT v1.3 + scan + 契约门）。
- 证据分级按 PLAN §2 五级；本报告全部关键判断有 P1-P6 锚点背书（`路径:行号`）。
- 复用消费者口径：**不预设**（画布/其他引擎/工具均可能）；"跨域"指非 MD
  块语义的领域模型。

## 2. 逐项三分类清单（三分类 × 四要素）

### 2.1 可直接复用（不改 MD 核心即可调用）

| # | 模块/机制 | 代码证据 | 对 PDF 质量的影响 | 验证标准 |
|---|----------|----------|------------------|----------|
| R1 | **命令调用协议**（apply_ops 唯一变异入口 + 十原语 + commit_transaction_as 溯源；wasm 面 apply_ops/replace_text_at_anchor/undo/redo） | lib.rs:253/387-618/610；mdcore-wasm lib.rs:36/59/79/85；P5 §3。旁路警示：Core 字段 pub tree/history/log 直访在类型层敞开，复用方需自律或包装 | 无波及（零 path 依赖） | 默认面 259 复跑 + 契约门 16/16（web 面） |
| R2 | **序列化访问面**（md 明文 / blocks_json / snapshot / history JSON / revision JSONL 五面） | markdown.rs:32、debug.rs:39-69、snapshot.rs:44、history.rs:251、revision.rs:60；P2 §5 | 无波及 | 默认面 259（含 r13 往返 19 条、r23_fingerprint 15 条）复跑 |
| R3 | **undo/redo 闭环**（MD 文档域内消费：invert_op 求逆 × 1s 合并窗 × 栈管理 + 失败全回滚） | op.rs:243、history.rs:20/91、lib.rs:875-932；P3 §2-§3（五不变量代码+测试双背书） | 无波及 | t1_quality_fixes 5 + r24_quality_fixes 4 + r11_stress 复跑 |
| R4 | **渐进打开与分片 dump**（extend_from_markdown 块边界防御 + blocks_json_range 先序游标增量建图） | lib.rs:849-869、debug.rs:69；document_progressive 11 条 | 无波及 | document_progressive + r16_blocks_range 复跑 |
| R5 | **设置面基础设施**（四层 schema + TOML sidecar 格式保持 + 六态降级 + FutureSchema 拒写）——域无关 | settings.rs:54/32/157；r24_settings 全套 | 无波及（PDF 面未消费设置，web 快照面除外） | r24_settings 复跑 + contract_gate 设置面 3 检 |

### 2.2 需要最小适配（适配内容 + 落点；不改核即可由消费方完成）

| # | 模块/机制 | 代码证据 | 适配成本（内容+落点） | 对 PDF 质量的影响 | 验证标准 |
|---|----------|----------|----------------------|------------------|----------|
| A1 | **变更订阅**：take_events 为拉取式单消费者 drain 队列；推送/多订阅子面**缺失** | lib.rs:237-239；main.ts:781/892 同属一个 CM6View；P5 §2 | 宿主侧 tap 层：单一排空点 → 分发到 N 订阅者（先例：设置面 CustomEvent 推送多订阅 main.ts:1440）；落点=消费方包装，零改核 | 无波及 | 新增：tap 层契约测试（多订阅者各收全量副本、顺序保持）+ 既有 259 复跑 |
| A2 | **按 ID 取块**：无 get_block 族 API；lookup_id→node 两步全 pub 可组合 | tree.rs:150/130；lib.rs:672-689（核自身即此组合）；P5 §4 | 消费方两步组合包装（Rust 直调）；JS 面走 blocks_json 过滤（注意 dump 面不含 marks，debug.rs:16-23）；若核侧新增薄 API 属改核，留裁决 | 无波及 | 新增：包装层往返测试（id→node→blocks_json 三面一致） |
| A3 | **块级监听**：事件为事务 op 粒度；InsertBlock 物化带 id、其余六类 op 只带 path（不对称） | op.rs:115-119 vs :40-65；lib.rs:77-83；P5 §5 | 消费方自建 path→id 换算索引 + 事件过滤层（BlockMap 先例：main.ts:158-271/844）；落点=消费方 | 无波及 | 新增：过滤层契约测试（六类 op 均可定位块）+ ime_cdp 28/28（若动桥） |
| A4 | **会话管理**：SessionManager 一窗一档 + 上限 16；DocumentSession 持 Core（类型绑定 MD 核） | session.rs:68/39/149-167；P6 §3 | 第三模型/多实例场景：宿主侧再实例化一个管理器（类型独立），或扩展 DocumentSession 为 enum（改核，留裁决）；v0 不改核即可用"每域一个 SessionManager"模式 | 无波及（mdc-pdf 会话未入 SessionManager——P6 §4a 实证） | r24_session 复跑 + 新增：双管理器并存测试 |
| A5 | **外部重载决策链**：FileStamp 探针 + plan_reload 三态纯函数（文件监视场景可直接用）；身份匹配依赖 MD 语义（域内可用） | external.rs:30-41/61-69/101；P4 §3 | 若消费方仅需"变更检测+三态决策"可直取纯函数；身份保持采纳（adopt_external）域内可用 | 无波及 | r23_external 14 + r23_fingerprint 15 复跑 |

### 2.3 不可复用（作为代码；跨域消费时）

| # | 模块/机制 | 代码证据 | 不可复用原因（+可移植模式） | 对 PDF 质量的影响 | 验证标准（若未来移植） |
|---|----------|----------|---------------------------|------------------|----------------------|
| X1 | **op 集 + 求逆矩阵**（跨域）：Op 七变体封闭于 MD 块语义（NodePath/BlockKind/Attrs）；invert_op 需 state-before | op.rs:24-66/243-344；P3 §2 | 异构域（画布等）需自有 op 集 + 自有求逆——代码不可移植，**架构模式可移植**（原子 op + 求逆 + 合并窗 + 事务回滚，P3 §4 先决条件清单即移植说明书） | 无波及（mdc-pdf 无 Op/Event 镜像——P6 §4b 实证） | 移植后新建域内等价测试套件（对齐 P3 五不变量） |
| X2 | **undo 栈容器**（跨域）：History/Entry 单态绑定 mdcore::Op | history.rs:42-66 | 泛化 = 改 mdcore（留裁决）；跨域复用即重写。**合并窗/边界快照/交互史作废三机制为可移植模式**（clear_interaction：ADR-B7 先例） | 无波及 | 同 X1 |

> 注：X1/X2 的"不可复用"限定**跨域**场景；MD 文档域内消费均为 R3 可直接
> 复用。mdc-pdf 侧印证：PDF 标注**无 undo 系统**（六命令无 undo/redo，
> Source::Undo/Redo 在 pdf src 零消费，P6 §4b）——两域互不依赖的现状本身
> 就是"跨域不复用 op/undo 代码"的已实现先例。

### 2.4 参考先例（非分类项，O4 双重角色）

**DocumentEngine 集成模式**（engine.rs:17-37，object-safe 4 方法 + Box 所有权
传递 + 装配期页数对账 document.rs:233-246 + 引擎/写器独立装配）：作为"第二
文档模型接入"的已实现样本登记。**本轮不推导任何未来消费者应采用相同接入
方式**（PLAN §1.1 O4 纪律）；其事实价值 = 证明并列模型零 path 依赖的可行性
与集成点形状。PdfSession 单形态状态机 + 六命令 + typed 错误 + flush 会话
API（非命令）的分层，同为先例事实。

## 3. 协同现状与约束（P4 附属登记，零选型）

- **六项能力全缺失**【代码确认 + grep 搜索记录】：多端同步 / OT / CRDT /
  广播 / 离线 / 自动冲突解决——全仓零实现；三个相邻机制经边界判定**不算**
  该能力：take_events（单消费者 drain，非广播）、adopt_external（文件监视
  重载，非离线合并）、plan_reload OfferChoice（人机 UX 决策，非自动冲突）。
- **B12 校准**：sync_resync = CM6 桥本地全量重同步（R9），非协同体系；
  revision = 本地文档级 u64 计数 + JSONL 日志。
- **约束事实**（p4-collab-status.md C1-C10 + K1-K8 全文，摘要）：coalesced
  事务不推进 revision（同号多记录→revision 非全局唯一序号）；undo/redo
  推进且入日志；结构命令 = 精确相等门禁 + 拒绝 + 重读重试（绝不静默
  rebase）；单写者模型（&mut self + 一窗一档 + document_id 拒换牌）；
  revision 三路径持久化（save meta / open 恢复 / replay 末条）。
- 本节不进行 CRDT/OT 选型（用户裁决范围）。

## 4. D1/D2/D3 相关发现登记（仅发现，不终裁）

- **D1（协同内核）相关**：现状零协同实现（§3）；若未来引入任何并发版本
  体系，p4 C/K 约束清单是事实基座（尤其 revision 非唯一序号、单写者模型、
  事件单消费者三点）。既有资产 = 无（B12 校准后）。
- **D2（双态立项）相关**：MD 面已有稳定块身份 + op 流 + AI 溯源三件基础
  事实；缺口 = 推送式多订阅事件、块级订阅、按 ID 直取（A1-A3 适配面即
  缺口清单）；两域并列零依赖先例 = mdc-pdf。AFFiNE 两视图投影所需的
  "共享状态层"在现状中不存在对应物（事件面是唯一跨消费通道且为拉取式）。
- **D3（生成态主线）相关**：Source::Ai 已是一等公民（commit_transaction_as
  溯源 + ByAnchor+expect 保护 + r12 全套语义锁）；PDF 面 Origin Human/Ai
  溯源已落盘互通（/T 编码 " [ai]" 后缀，Acrobat 可读——writer.rs:58-75）。
  生成态管线的"命令通道"两侧均已就位。
- 上述仅为事实发现；立项与裁决由用户作出。

## 5. 内容引用扩展点（一行登记）

BlockId 跨会话稳定性 = 条件保真（snapshot/replay 全保真；save→open 仅
先序 1..n 递增时一致——结构性编辑后漂移，P2 §3 U1/G4 未测边界）→ 内容
引用的可行性依赖 meta 面扩展（如 ID 持久化进 meta），属未来裁决项，本轮
不实现不设计。

## 6. 质量缺口与验证标准总表（P7 汇总输入）

**现有守护（复用后必须保持全绿）**：九项门禁（§0 Q2）+ 契约门 16/16 +
smoke 15/15 + IME CDP 28/28——web 三门禁已于执行轮补验全绿【测试验证】
（2026-10-12：node 24 + wasm-bindgen 0.2.129 精确构建 + playwright
chromium；契约门 16/16 含 settings 三检，smoke 15/15 含 1MB/20448 块
实测 typing p95 12.6ms，IME CDP 28/28；原 U-1 消除）。

**复审发现的缺口（登记不修，全部留裁决）**：
1. Source 镜像锁单向（mdcore 新增 Source 值 → mdc-pdf 静默漂移）——建议
   未来加双向计数锁（改动属 mdcore/mdc-pdf 测试面，非本轮）；
2. settings.undo_coalesce_ms 死设置（存储链齐备、零消费者——should_coalesce
   直读常量 history.rs:96）；
3. P3 六项测试锁缺口（push_entry 清 redo 无专属用例 / undo 不恢复选区 /
   wasm source 静默兜底 / save→open ID 漂移未测 / undo 栈无上限 / coalesced
   事件 revision 消费契约）；
4. P5 契约漂移 4 项（F-1 __wasmMemory/__heapTrace 非恒在且门禁不可见 /
   F-2 CONTRACT §3:62 版本号残留 1 / F-3 __view 锁面不全 / F-4 __bridge
   运行时宽于声明面——均文档/门禁级，非行为缺陷）；
5. MD 面性能基准空位（仅 1 条软冒烟断言 r23_fingerprint.rs:404-464 +
   r14 打点无时间断言；P6 B-1..B-10 待基准清单）。

## 7. 遗留未知项清单（状态 + 消除路径）

| # | 未知项 | 级别 | 消除路径 |
|---|--------|------|----------|
| U-1 | ~~smoke 15/15 / IME CDP 28/28 / 契约门 16/16 当前状态~~ | ✅ 已消除【测试验证】（执行轮补验 16/16 + 15/15 + 28/28 全绿，2026-10-12，web/tests/ 三脚本复跑） | 已完成 |
| U-2 | save→open 结构性编辑后 Block ID 漂移的确切边界 | 【未知】（机制已证，边界未测） | 新增往返测试（P3 G4 处方） |
| U-3 | MD 面大文档性能（打开/编辑/序列化量级） | 【未知】（无基准） | P6 B-1..B-10 逐项建基准 |
| U-4 | wasm source 静默兜底（非法字符串→View）的实际可达性 | 【推断】（误用频率未量化，依据链见 P3 U2） | 桥侧输入校验 + 用例 |
| U-5 | 真机壳实体行为（R24-d 开发环境轮事项） | 【未知】（本环境无实体） | 开发环境轮 |

## 8. 审计链与可复现性

- 执行链：P0（主代理：门禁九项 + B1-B14）→ P1（主代理：结构地图）→
  P2+P3（勘察代理 → 复审代理全量核验）→ P4（勘察 → 复审）→ P5（勘察 →
  复审）→ P6（勘察）→ P7（本报告）。PLAN §5.3 写审分离执行；P6 复审由
  P7 终审代理合并覆盖（见 worklog）。
- 全部阶段产物在 `md-line/boundary-audit/`（PLAN/baseline/p1-p6/REPORT）；
  mdcore 仓全程只读（每阶段 `git status --porcelain` 零改动复核在档）。
- 本报告 ≤300 行（PLAN P7 验收）；零选型表述（§3 显式声明）；零性能宣称
  （性能相关全部引用 P0 实测门禁数字或标【未知】）；零终裁性表述（§4
  显式声明"裁决由用户作出"）。
