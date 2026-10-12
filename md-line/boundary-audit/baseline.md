# P0 基线校准记录（baseline.md）

> 状态：**完成（2026-10-12 执行轮）**。对应计划：PLAN.md v1.2 §4 P0。
> 执行时间：2026-10-11 16:07 UTC ~ 2026-10-11 17:30 UTC（北京时间 10-12
> 00:07~01:30）。执行者：主代理（Super Z）。
> 上轮（t7）阻塞 BL-1~BL-3 已全部解除，见 §0。

## 0. 阻塞登记与解除

| # | 上轮阻塞项 | 本轮状态 | 解除记录 |
|---|-----------|----------|----------|
| BL-1 | mdcore 克隆不可执行（凭据未随会话保留） | ✅ 解除 | 用户 2026-10-12 消息内提供 PAT + 开工指令；extraheader 方式克隆（凭据零入文件，.git/config 实测干净 URL + 全仓 token 值 grep 零命中） |
| BL-2 | 门禁复现不可执行 | ✅ 解除 | 依赖 BL-1；工具链 1.99.0 按rust-toolchain.toml 安装（rustc 1.99.0 (b940084d7 2026-09-28)），pdfium 取件 + fixtures 现场生成，九项门禁全部亲跑 |
| BL-3 | B1-B14 对账不可执行 | ✅ 解除 | 依赖 BL-1/BL-2；§3 全表落级（14 条全部【代码确认】级为主，含 1 条证伪校准、2 条口径修正） |

## 1. 环境与门禁复现（本轮实测）

### 1.1 仓库与版本

| 项 | 实测值 | 与 B11 预期对照 |
|----|--------|----------------|
| mdcore main | `03815b6258472f2b57f222b6a571d7ad3cf52520`（PDF-e 收口提交） | ✓ 一致 |
| mdcore compliance-baseline | `1adced22c3fa2787e91265b7a47c6dc5ebfa1b9b` | ✓ 一致 |
| mdcore mother/r23-snapshot | `24268cadcb02c4748ee69e36424e7ab4560f0b1f` | ✓ 一致 |
| 克隆方式 | `--depth 1 --branch main`，extraheader Basic 认证（凭据零落盘，实测零残留） | — |
| 工作树状态 | 克隆后全程只读，`git status --porcelain` 恒空 | ✓ 只读红线合规 |

### 1.2 门禁九项（全部本轮亲跑）

| # | 门禁 | 命令 | 本轮结果 | B11 对照 |
|---|------|------|----------|----------|
| G1 | 默认面 | `cargo test --workspace` | **259 passed / 0 failed**（mdcore 149 + mdc-pdf 67 + mdshell 43 套件合计） | ✓ 259/259 |
| G2 | lopdf 面 | `cargo test -p mdc-pdf --features engine-lopdf` | **78 passed / 0 failed** | ✓ 78/78 |
| G3 | pdfium 面 | `cargo test -p mdc-pdf --features engine-pdfium` | **80 passed + 6 ignored / 0 failed** | ✓ 80/80+6i |
| G4 | dual 面 | `cargo test -p mdc-pdf --features engine-pdfium,engine-lopdf` | **108 passed + 6 ignored / 0 failed** | ✓ 108/108+6i |
| G5 | 性能门 | `--test pdfc_perf -- --ignored --nocapture` | **6/6 ok**：打开 TTI 500 页中位 0.17ms（阈 1ms）/ 首页渲染 8.21ms（阈 25ms）/ 顺序渲染 p95 0.97ms（阈 5ms）/ LRU 命中 5.41µs（阈 20µs）/ 会话端到端冒烟 / 文本抽取 26.48ms@500 页（阈 100ms） | ✓ perf 6/6（数字量级与 gates.md 登记一致） |
| G6 | 合规扫描 | `bash compliance/scan.sh` | **PASS: scope=product, denylist terms=20, hit-paths=0** | ✓ scan 0 |
| G7 | wasm32 | `cargo check --workspace --target wasm32-unknown-unknown` | **通过**（mdcore-wasm 2 条警告 = 在档基线） | ✓ wasm32 pass |
| G8 | clippy | `cargo clippy --workspace` | **仅 mdcore-wasm 2 条在档基线警告**（unused variable: static_method_of ×2），零新增 | ✓ clippy 零新 |
| G9 | fmt | `cargo fmt --all -- --check` | **exit 0（干净）** | ✓ fmt 0 |

**结论：B11 门禁数字四面 + perf + scan + wasm32 + clippy + fmt 九项全部本轮复现，全绿，零漂移。**【测试验证】

门禁环境重建记录（可复现性）：rustup 1.29.1 → toolchain 1.99.0（minimal + wasm32 target + rustfmt/clippy）；pdfium 二进制经 `md-line/pdf-a/spike/fetch-pdfium.sh` 取件（chromium/8086 linux-x64 non-V8，SHA256 验证通过）；fixtures 由 spike 现场生成（`cargo run --release --bin pdf_a_spike` → small.pdf/large.pdf + `--bin formfill_probe` → form_probe.pdf/small.pdf，lopdf 构造零外部素材）。pdfium/pytest 定位序实测命中兄弟目录 spike 取件位（`crates/pdf/tests` 相对路径 `../../../cleanroom-analysis/md-line/pdf-a/spike/{fixtures,bin/lib}`）。

### 1.3 workspace 结构盘点（Cargo.toml 实测）

| 项 | 实测 | 说明 |
|----|------|------|
| workspace members | **5 crate**：`crates/mdcore`、`crates/mdcore-wasm`、`crates/shell`（mdshell）、`crates/shell-native`（mdshell-native）、`crates/pdf`（mdc-pdf） | Cargo.toml:3 |
| default-members | 4（mdcore / shell / shell-native / pdf，**不含 mdcore-wasm**） | Cargo.toml:4 |
| shell-native 形态 | 蓝图态：69 行、零依赖（tao/wry/gtk 全部注释态），"需 webkit2gtk-4.1 系统库，沙箱不可编译" | shell-native/Cargo.toml + lib.rs |
| 版本单源 | workspace.package version=0.1.0，各 crate `version.workspace = true`（R24-c） | Cargo.toml:10-11 |
| 依赖单源 | serde 1.0.229 / serde_json 1.0.151 workspace 级（T1 fix-c） | Cargo.toml:19-21 |
| mdcore 依赖 | ropey 1.6 + serde(derive,rc) + serde_json（仅三项） | crates/mdcore/Cargo.toml |
| mdshell 依赖 | mdcore(path) + serde + serde_json + toml_edit 0.25.17 | crates/shell/Cargo.toml |
| mdc-pdf 依赖 | 默认零引擎依赖；engine-pdfium → pdfium-render 0.9.4（pin，non-V8，thread_safe）；engine-lopdf → lopdf 0.45.0（pin，default-features=false） | crates/pdf/Cargo.toml |
| mdcore-wasm 依赖 | mdcore(path) + serde_json + wasm-bindgen 0.2 | crates/mdcore-wasm/Cargo.toml |
| web/ | CM6 桥 TS 单文件 main.ts 1452 行 + tests（contract_gate.py / ime_cdp.py / web_smoke.py 等） | web/ |

规模实测：mdcore src+tests 12,504 行（13 模块 + 14 测试文件）；mdshell 2,348 行；mdc-pdf 10,755 行（15 模块 + 7 测试文件）；mdcore-wasm 217 行（单文件）；shell-native 69 行。合计约 25.9K 行。

## 2. B1-B14 对账表（本轮落级）

> 落级三态：确认 / 证伪 / 修正。证据级别按 PLAN §2 五级。锚点格式 `路径:行号`。

| # | 事实 | 落级 | 本轮证据锚点 | 差异登记 |
|---|------|------|--------------|----------|
| B1 | 架构 = mdcore 块核（Rust/ropey）+ CM6 wasm 桥 + tao/wry/GTK3 宿主壳 | **修正** | 【代码确认】Cargo.toml:3-4；shell-native/src/lib.rs（69 行蓝图）；web/src/main.ts（CM6）；crates/mdcore/Cargo.toml（ropey） | 修正点：crate 数量口径。"四 crate + crates/pdf"不准确——**workspace members = 5**（mdcore/mdcore-wasm/shell/shell-native/pdf），default-members = 4（含 shell-native、不含 mdcore-wasm）。宿主壳 = 蓝图态（shell-native 零依赖注释态，R24-d 实体在开发环境） |
| B2 | 核心资产五件套 | **确认** | 【代码确认】BlockTree（tree.rs:98）/ Command Boundary（lib.rs:253 apply_ops + §10.2 十原语）/ Stable Block Identity（id.rs:12 BlockId + IdAllocator）/ Revision+Anchor（revision.rs:30 / anchor.rs:18）/ Source::Ai 溯源（history.rs:24 Source 五值 + lib.rs:610 commit_transaction_as） | 无 |
| B3 | 单一变异入口 apply_ops | **确认** | 【代码确认】lib.rs:253（唯一公开变异入口，全部命令原语内部转调 lib.rs:397-407 等）；lib.rs:349 apply_raw_batch 仅 undo/redo/replay 内部使用；【测试验证】lib.rs:1053 system_apply_ops_filtered_from_undo_stack 等全套通过 | 无 |
| B4 | invert_op 求逆 + history 合并窗 + COALESCE_MS 跨 crate 常量锁 + SetMark 不变量 | **确认** | 【代码确认】op.rs:243 invert_op（7 op 全覆盖，RemoveBlock 逆 = 先序重建子树带原 ID）；history.rs:20 `COALESCE_MS: u64 = 1000`；settings.rs:102 `undo_coalesce_ms: mdcore::history::COALESCE_MS`（跨 crate 锁实证）；op.rs:297-323 SetMark 求逆不对称修复（no-op → 同向逆）；【测试验证】r24_quality_fixes 4/4、t1_quality_fixes 5/5 通过 | 无 |
| B5 | 事务失败全回滚 | **确认** | 【代码确认】lib.rs:267-283（backup=tree.clone() + 失败恢复）；lib.rs:879-887/911-916 undo/redo 失败路径（T1-F10：条目压回 + boundary 快照拨回 + 树恢复）；【测试验证】t1_quality_fixes 通过 | 无 |
| B6 | adopt_external（成功清脏）+ fingerprint 确定性指纹 | **确认** | 【代码确认】external.rs:101 adopt_external（System 事务 + external.rs:160 clear_interaction + :163 mark_saved 清脏）；fingerprint.rs:31-59 FNV-1a 64 手写、:95 TreeFingerprint 下标 memo 无 HashMap、结构+内容双指纹；【测试验证】r23_external 14/14、r23_fingerprint 15/15 通过 | 无 |
| B7 | SessionManager（BTreeMap + MRU + 上限 16 + 一窗一档 + SessionRecord v1） | **确认** | 【代码确认】session.rs:68-73（BTreeMap + focus_order Vec + limit）；:39 DEFAULT_MAX_SESSIONS=16；:42 SESSION_RECORD_VERSION=1；:149-167 open 三重硬门（DuplicateWindow/DocumentAlreadyOpen/LimitReached）；:191 snapshot_record 焦点序导出；【测试验证】r24_session 套件通过 + session.rs:259 default_limit_is_sixteen | 无 |
| B8 | 设置面四层 schema + TOML sidecar + 降级 | **确认** | 【代码确认】settings.rs:54-60（editor/ai/storage/appearance 四层）；:32 SCHEMA_VERSION=1；toml_edit 格式保持（注释/排版往返保留）；:157-170 LoadStatus 六态（Clean/Created/Migrated/KeyDegraded/FileDegraded/FutureSchema）；FutureSchema 拒写（:146-153 SaveError）；R24-a1 写读形态对称修复（:22-25）；【测试验证】r24_settings 套件通过 | 无 |
| B9 | mdc-pdf 经 DocumentEngine（object-safe）集成 + 六命令带 Source + mdcore://pdf 协议族 | **确认** | 【代码确认】engine.rs:17-37 trait DocumentEngine（page_count/page_geometry/outline/extract_text 四方法，object-safe `&dyn`）；command.rs:94-121 六命令（ExtractText/Search/Outline/AnnotateAdd/AnnotateEdit/AnnotateList，每变体带 source）；command.rs:23-31 Source = mdcore 语义镜像（"语义镜像而非类型复用，crate 间零 path 依赖"）；protocol.rs:14-32（SCHEME=mdcore/FACE=pdf/URL_PREFIX=mdcore://pdf//EVENT_COMMAND=mdc-pdf-cmd/EVENT_EVENT=mdc-pdf-event）；document.rs:172 execute 单一调度；CONTRACT.md §6 | 无 |
| B10 | 契约治理 v1.3 + __contractVersion=1.1 + contract_freeze 双锚 | **确认** | 【代码确认】CONTRACT.md:1 "冻结 v1.3"；main.ts:1443 `window.__contractVersion = 1.1`（CONTRACT.md:62 的 "`__contractVersion: 1`" 为 §3 R23-d 冻结 v1 段历史文本，:110 现值 1.1）；contract_freeze.rs:20/40/67/90 四测试（事件族词面锁 + 文档存在性 + v1.1 设置面 + §5 会话面）+ :83 编译期常量锁 `SCHEMA_VERSION == 1` | 无（worklog 口径与代码一致；CONTRACT.md:62 行文本身有历史段落值 1 与现值 1.1 并存，非漂移——升版记录 :142 明载 1→1.1） |
| B11 | 门禁终态数字 + 双分支 SHA | **确认** | 【测试验证】§1.2 九项全绿零漂移（259/259、78/78、80/80+6i、108/108+6i、perf 6/6、scan 0、wasm32 过、clippy 2 基线零新、fmt 0）；§1.1 三分支 SHA 一致 | 无。**注**：smoke 15/15 与 IME CDP 28/28 未复跑（需 node/playwright 环境，t2 轮亦未复跑——与 B11 登记口径一致，登记为【文档记录：本轮未复验】） |
| B12 | sync_resync/revision 体系 + OT/CRDT 零登记 | **证伪校准** | 【代码确认】tests/sync_resync.rs:1 = **R9 轮 sync_from_markdown（全量重同步兜底）语义测试**（10 用例：revision 前进/undo 撤 sync/diff 最小性/嵌套 diff/管道一致性/ID 冲突回归等）——**非协同同步体系**；revision.rs = 本地单调计数 + JSONL append 日志（重放面）；全仓 grep：crdt/yjs/yrs/websocket/broadcast/operational transform/lamport/vector clock **全部 0 命中** | **校准**：collab-a §6.3 转述的"mdcore 既有 sync_resync/revision 体系（可与 CRDT 叠加）"系对该测试文件名的望文生义——sync_resync 实为 **CM6 桥本地重同步兜底**（view 与核的文档级对账），不含任何网络/多端/并发语义；"可与 CRDT 叠加"仅为源报告推断，本轮证伪其"既有协同资产"暗示。**现状 = 零协同实现**（P4 将按六能力矩阵正式登记） |
| B13 | engine_pdfium.rs OnceLock 单例（失败可重试）；Core 无 Clone/Debug | **确认+补充** | 【代码确认】engine_pdfium.rs:76 `static PDFIUM: OnceLock<&'static Pdfium>`（Box::leak 单例）+ :80 `BIND_LOCK: OnceLock<Mutex<()>>`（绑定串行化）+ :353/371 失败可重试（OnceLock 未写入则下次重绑）；session.rs:67 注释实证 Core 不可 Clone/Debug（时钟闭包字段）；**补充发现**：mdcore 唯一全局状态 = document.rs:25 `ENTROPY_SEQ: AtomicU64`（UUIDv7 熵计数器，原子操作线程安全）——全仓 static/lazy_static/thread_local 清点共 3 处实际可变全局（详见 P6） | 无实质差异；ENTROPY_SEQ 为本轮新登记点 |
| B14 | op-first 树模型（非快照即模型） | **确认（表述微调）** | 【代码确认】op log（RevisionLog JSONL）可重放重建文档状态（lib.rs:966 replay_jsonl，op 内 ID 已物化）；snapshot 为独立导出面（snapshot.rs:44，**不含 undo 栈**——跨会话历史走 History JSON，snapshot 只承载文档态+分配器水位）；模型本体 = BlockTree arena（tree.rs:98）。06 §1.2 判据 1 转述与代码事实相符：变异必经 op 管线、日志自含可重放 | 微调："非快照即模型"宜表述为"**op log 自含可重放 + 树为唯一语义真理 + snapshot 仅导出面**"——三者关系比单句转述更精确 |

**对账结论**：14 条中 **11 确认 / 1 证伪校准（B12）/ 2 修正（B1 口径、B13 补充、B14 微调计 3 处差异登记）**。全部差异不阻塞后续阶段；B12 校准直接改变 P4 的叙事基线（既有协同资产 = 零，sync_resync 是本地机制）。

## 3. P0 遗留与移交

- smoke 15/15、IME CDP 28/28、契约门 16/16（python 面）未复跑——需 node/npm + playwright 环境，非本环境必需项（t2 权威轮亦未复跑，B11 口径一致）；登记【文档记录：本轮未复验】，不阻塞 P1-P6（Rust 面证据已 100% 本轮亲证）。
- web 构建（`cd web && npm ci && npm run build`）未执行——wasm32 check 已过（G7），web 构建属 R24-d 开发环境轮事项。
- 本文件 + p1-structure.md 共同构成 P0-P1 阶段报告；worklog 登记 Task `t7-md-boundary-p0`（执行段）。
