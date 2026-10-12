# MD 复用地基实施轮报告（reuse-impl REPORT）

> 阶段：T0-T5 全部完成（PLAN v1 任务清单连续执行，2026-10-12）。
> mdcore 三个工作流提交已推送：26aa4af（阻断缺口四项）→ f08e376
> （settings 独立 crate 化）→ 15efc28（REUSE.md）。起点 main=03815b6
> （审计基线），终态工作区零改动。本报告按用户指令三要素组织：
> 实际完成项 / 测试结果 / 剩余阻塞。

## 1. 实际完成项（对照任务清单逐项）

| 任务 | 状态 | 交付物（mdcore 锚点） |
|------|------|----------------------|
| T0 环境 + 改前基线 | ✅ | 平台回滚事故零损失恢复（远端为真源）；工具链 1.99.0 + pdfium 取件 + fixtures 现场生成 + web 栈重建；**改前 12 门禁全绿认证**（259/78/80+6i/108+6i/perf6/scan0/wasm32/clippy2基线/fmt + 契约16/smoke15/IME28） |
| T2a Source 镜像锁双向化 | ✅ | `pdfb_skeleton::source_mirror_bidirectional_exact_count`：(c) mdcore 侧恰五变体计数锁（文件文本）+ (d) pdf 侧编译期穷尽锁（const fn 无通配 match）。**红绿演示在档**：注入第六变体 Ghost → 精确报警实测六变体 → 撤销后绿 |
| T2b P3 测试锁四条 | ✅ | `crates/mdcore/tests/t3_reuse_locks.rs`：G1 push_entry 清 redo / G4a save→open ID 重编号 / G4b 重开 undo 物化撞号 / G6 coalesced 事件共享 revision。全部现状锁标注 |
| T2c wasm source 校验 | ✅ | `_ => Source::View` 静默兜底 → 封闭映射 typed reject（parse_source_str 纯函数 + 2 单测；实测 main.ts 全调用点仅 'view'，零破坏面） |
| T2d 契约面卫生 | ✅ | CONTRACT §3 勘误（__wasmMemory/__heapTrace 渐进专属 + 版本残留 1→1.1）；契约门 16→**18** 检（F-1 双侧现状锁 + F-3 __view 断言） |
| T3 settings 独立 crate | ✅ | `crates/settings`（mdc-settings）：settings.rs 整体迁移（git rename 血缘 96%）；唯一 mdcore 引用改本地常量（r24_settings:365 跨 crate 锁背书单源）；shell re-export 路径零破坏；闭包锁测试固化域无关；r24_packaging 版本锁同步六 crate |
| T4 REUSE.md | ✅ | 91 行复用工具包：12 单元消费视角转录 + 快速指引 + 九门禁基线 + 六条消费契约注记；零上游品牌词（scan 0） |
| T5 终跑收口 | ✅ | 见 §2；mdcore 工作区归零 |

## 2. 测试结果（改前 → 改后，全绿）

| 门禁 | 改前 | 改后 | 增量解释 |
|------|------|------|----------|
| G1 默认面 | 259/259 | **267/267** | +4 t3_reuse_locks +2 wasm 单测 +1 镜像锁 +1 闭包锁（settings 内联测试随迁净零） |
| G2 lopdf | 78/78 | **79/79** | +1 镜像锁（pdfb_skeleton 属此面） |
| G3 pdfium | 80+6i | **81+6i** | 同上 |
| G4 dual | 108+6i | **109+6i** | 同上 |
| G5 perf | 6/6 | **6/6** | 数字量级与阈值纪律一致（打开 0.17ms 级 / p95 亚毫秒级 / 文本抽取 27.6ms@500 页） |
| G6 scan | 0 | **0** | 含 REUSE.md 新文件 |
| G7 wasm32 | 过 | **过** | 2 条在档基线警告不变 |
| G8 clippy | 2 基线 | **2 基线零新** ⚠️勘误见 §7 | |
| G9 fmt | 0 | **0** | |
| 契约门 | 16/16 | **18/18** | +F-1 缺席锁/在场锁（F-3 并入既有检） |
| smoke | 15/15 | **15/15** | |
| IME CDP | 28/28 | **28/28** | |

**PDF 质量零波及证明**：mdc-pdf lib 面零改动（唯一触碰 = 测试面加锁）；
PDF 四面 + perf 6/6 改前改后全绿（唯一计数变化 = 本轮自加的镜像锁）。

## 3. 本轮实测新发现（审计推断 → 测试验证）

1. **G4b 撞号机制实测成立**（P2-U2【推断】→【测试验证】）：构型
   [1,2,3,4]→插 MID(物化5)→删 p3→save→重开重解析 [1,2,3,4]（p2 顶占
   ID 3）→undo 重插 p3(id=3)→**树内 [1,2,3,3,4] 重复 BlockId**，注册表
   覆盖无守卫、先注册块从 id_index 失联。撞点（p2/p3）与审计预测
   （p3/p4）不同但机制一致。修复方案（register 守卫 = 行为变更）留裁决。
2. **G4a 漂移具体形状**：活树 [1,4,2,3] → 重开 [1,2,3,4]——MID 的物化
   ID 4 漂为 2，ID 4 语义转移给 p3。跨档 BlockId 消费者必失联（现状锁）。
3. **P1 口径校准**：审计 P1 称"session/settings 均用 Core"——settings
   半句实测为**仅 :102 一处常量取源**（COALESCE_MS），零 Core 类型依赖。
   这是抽取缝极小的直接证据（T3 得以整体迁移零重构）。

## 4. 剩余阻塞与登记项

**零阻塞**：任务清单全部完成，无停工项。

登记留裁决（PLAN 分诊表 #5-#7 + 本轮新增 1 项）：
| # | 项 | 现状 | 选项 |
|---|-----|------|------|
| R-1 | undo_coalesce_ms 死设置 | 存储链齐备不驱动行为；REUSE.md 已注明保留键身份 | A 接线（History 配置入口+装配+用例）/ B schema 正式保留键声明 |
| R-2 | 重开 undo 物化 ID 撞号（本轮 G4b 实测） | 现状锁文档化（t3_reuse_locks） | register 守卫 / ID 持久化进 meta / 维持现状+文档 |
| R-3 | undo 栈无上限（G5） | 资源策略 | 容量上限 + 逐出策略（产品决策） |
| R-4 | F-4 __bridge 运行时宽于声明 | 低风险登记 | 锁"无多余"脆性大，维持登记 |
| R-5 | MD 面基准 B-1..B-10 | 无消费者性能要求 | 统一 profile 口径后另轮建基准 |
| R-6 | 真机壳实体（U-5/R24-d） | 开发环境轮 | 既有排期不变 |

## 5. 未知项清单状态（对照审计 REPORT §7）

- U-1 web 三门禁：上轮已消除（本轮改前/改后各复验一次，18/15/28）
- U-2 save→open ID 漂移边界：**本轮消除**（G4a/G4b 两条现状锁，机制与
  边界均实测在档）
- U-4 wasm source 静默兜底：**本轮消除**（typed reject + 单测 + 三门禁）
- U-3 MD 性能量级：维持【未知】（R-5）
- U-5 真机壳：维持（R-6）

## 6. 审计链与复现

- 执行链：t8-plan（任务清单提交 3dac17e）→ T0 恢复+基线 → T2（26aa4af）
  → T3（f08e376）→ T4（15efc28）→ T5 终跑 + 本报告。
- mdcore Git 纪律：三提交均门禁绿后提交、推送前远端核实（03815b6 无
  位移后依次推进）、PAT extraheader 单次传递零残留（每次推送后 grep 0）。
- 本轮证据分级：全部新结论带 mdcore 锁点或测试名；性能仅引用门禁实测
  数字；characterization 测试显式标注现状锁身份。

## 7. 勘误（t9-a 复核发现，2026-10-12）

**G8 clippy 行勘误**：上表"2 基线零新"实测口径为 `cargo clippy --workspace`
（REUSE.md §4 当时命令面，不含测试目标）。按 gates.md PDF-e 起的正式口径
（四面 --all-targets），26aa4af 新增的镜像锁测试含 1 条
manual_pattern_char_comparison 告警（pdfb_skeleton.rs:1107，.split 闭包
写法）——t8 收口时该面漏检。t9-a（mdcore d3123ba）已修复：改字符数组
模式（语义零变化，dual 109+6i 复验不变）+ REUSE.md §4 命令恢复四面
--all-targets 口径。修复后四面实测 = 仅 2 条 wasm 在档基线，零新增。
