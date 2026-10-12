# P0 基线校准记录（baseline.md）

> 状态：**阻塞（凭据未随会话保留）**——2026-10-11 16:07 UTC（北京时间
> 2026-10-12 00:07）登记。
> 对应计划：PLAN.md v1.2 §4 P0；阻塞登记依据：§2 纪律④（v1.1）+ 用户
> 2026-10-12 指令"mdcore 代码结论必须来自本轮实际访问与核验，不能以
> 历史文档代替"。

## 0. 阻塞记录（纪律④逐条登记）

| # | 阻塞项 | 原因（本轮实测） | 影响面 | 解除路径 |
|---|--------|------------------|--------|----------|
| BL-1 | mdcore 克隆不可执行 | 私有仓；本会话 GITHUB_TOKEN 未设置（上轮 2026-10-11 提供的 PAT 按"零落盘"纪律不跨会话保留——HANDOFF §5.2）；匿名 `git ls-remote` 要求认证（fatal: could not read Username）；匿名 GitHub API HTTP 403 | P0 ①克隆 ②SHA 验证 ④结构盘点 ⑤B1-B14 对账；P1-P6 连带阻塞（硬依赖 P0） | 用户经命令行单次传递 PAT（零落盘），即行克隆 |
| BL-2 | 门禁复现不可执行 | 依赖 BL-1（无本地工作树与工具链） | P0 ③四面门禁 + scan.sh + 契约门 + wasm32 check 数字无法落级【测试验证】 | 同 BL-1 |
| BL-3 | B1-B14 对账不可执行 | 依赖 BL-1/BL-2（对账对象 = 本轮代码与门禁实测） | PLAN §3 基线表全部 14 条维持【文档记录】级 | 同 BL-1 |

**登记纪律执行说明**：

- 按用户 2026-10-12 指令，本轮【零】mdcore 代码级结论输出；PLAN §3 与
  worklog 中所有 mdcore 内部结构引用继续维持【文档记录】标注，显式标记
  "本轮未复验"。
- 历史报告（worklog t2 权威复跑数字、gates.md 登记等）不作为本轮验证
  替代，仅在 P0 解除后作为对账**对照面**使用。
- 本轮已完成的可验证部分见 §1（环境实测）；P0 解除后的执行清单见 §2。

## 1. 环境实测记录（本轮可验证部分，2026-10-11 16:07 UTC）

| 检查项 | 结果 | 证据 |
|--------|------|------|
| 容器时钟 | 2026-10-11 16:07 UTC（= 北京时间 2026-10-12 00:07，与用户时区设置一致） | `date` |
| cleanroom-analysis 本地 HEAD | main = 48d0536（v1 草案提交，本地领先 origin/main 1 提交；origin/main = d9466fe） | `git log` / `git status -sb`（ahead 1） |
| 工作区未提交改动 | PLAN.md / STATUS.md / worklog.md（t6-r1 的 v1.1 修订工作区落地，在档登记一致） | `git status --short` |
| GITHUB_TOKEN 环境变量 | 未设置 | `env` 检查（GITHUB_TOKEN set: NO） |
| mdcore 匿名可达性 | 不可达（私有仓要求认证） | `git ls-remote https://github.com/mmqz/mdcore.git` → fatal: could not read Username；`api.github.com/repos/mmqz/mdcore` → HTTP 403 |
| 本地 mdcore 克隆 | 不存在（工作区两轮重置 + P0 从未执行——worklog t6-r1"mdcore 零接触（未克隆）"） | 文件系统检查 |
| 其他凭据残留 | ~/.git-credentials、~/.netrc 均不存在（零落盘纪律合规）；~/.gitconfig 仅 safe.directory/user.name/init.defaultBranch 三项 | 文件检查 |

## 2. P0 执行清单（凭据到位后立即执行；只读红线 + 零凭据残留纪律）

1. **凭据单次传递验证**：`GITHUB_TOKEN=*** git ls-remote https://github.com/mmqz/mdcore.git`
   ——预期三分支（以克隆当刻实测为准）：main=03815b6 /
   compliance-baseline=1adced2 / mother/r23-snapshot=24268ca（B11 对照面）。
2. **净工作区克隆**：`git clone --depth 1 https://mmqz:${GITHUB_TOKEN}@github.com/mmqz/mdcore.git mdcore`
   （浅克隆 main 即可：门禁复现与结构盘点只需工作树；SHA 验证用 ls-remote；
   compliance-baseline 如需对账按需 fetch）→ **立即** `git remote set-url origin https://github.com/mmqz/mdcore.git`
   清除 URL 内凭据。
3. **零凭据残留验证**：`.git/config` 全文 + 克隆目录全仓 grep 实际凭据值
   零命中（既有 "ghp_" 模式字面量按 t6-r1 先例甄别为检查机制模式）。
4. **工具链对齐**：读 `rust-toolchain.toml`（预期钉 1.99.0——B11/PLAN P0）；
   PDF 面依赖按 t1 轮先例重建（fetch-pdfium.sh）。
5. **门禁复现**（全部亲跑，数字与 gates.md 对照）：`compliance/scan.sh` +
   `cargo test --workspace`（默认面 + lopdf + pdfium + dual 四面）+ 契约门
   + wasm32 check（+ fmt/clippy 对照在档基线：wasm 桥 2 条 clippy）。
   对照数字（B11，历史登记）：默认 259/259 + lopdf 78/78 + pdfium
   80/80+6i + dual 108/108+6i + perf 6/6 + scan 0。
6. **结构盘点**：根 Cargo.toml workspace 成员实测 vs B1"四 crate +
   crates/pdf"口径对账；模块/测试套件/文档清单枚举（P1 输入）。
7. **B1-B14 逐条对账落级**：五级证据分级 × 确认/证伪/修正三态 → 回写
   本文件 §3；修正项回写 PLAN §3 并在 worklog 登记（差异不静默）。
8. **只读红线验证**：`git status` 干净（零工作区改动）；凭据用毕零残留
   复验（同 3）。

## 3. B1-B14 对账表（待执行——凭据到位后填写）

| # | 事实摘要（PLAN §3 全文为准） | 本轮落级 | 锚点 | 差异登记 |
|---|------------------------------|----------|------|----------|
| B1 | 架构 = mdcore 块核 + CM6 wasm 桥 + tao/wry/GTK3 宿主壳 | 待执行 | — | — |
| B2 | 核心资产五件套（Document Model / Command Boundary / Stable Block Identity / Revision+Anchor / Source::Ai 溯源） | 待执行 | — | — |
| B3 | 单一变异入口 apply_ops | 待执行 | — | — |
| B4 | invert_op 求逆 + history.rs 合并窗 + COALESCE_MS 跨 crate 常量锁 | 待执行 | — | — |
| B5 | 事务失败全回滚（T1-F10 + t1_quality_fixes 锁） | 待执行 | — | — |
| B6 | adopt_external 成功清脏标记 + fingerprint.rs 确定性指纹 | 待执行 | — | — |
| B7 | SessionManager（BTreeMap + MRU + 上限 16 + 一窗一档 + SessionRecord schema v1） | 待执行 | — | — |
| B8 | 设置面四层 schema + TOML sidecar 原子写 + 三层降级 + 注释往返保留 | 待执行 | — | — |
| B9 | mdc-pdf 经 DocumentEngine 集成 + 六命令带 Source + mdcore://pdf 协议族 + CONTRACT §6 | 待执行 | — | — |
| B10 | CONTRACT.md v1.3 + __contractVersion=1.1 + contract_freeze.rs 双锚 | 待执行 | — | — |
| B11 | 门禁终态数字（见 §2.5 对照）+ main=03815b6 / compliance-baseline=1adced2 | 待执行（ls-remote 部分可先行） | — | — |
| B12 | sync_resync/revision 体系存在性（协同现状第一核查项） | 待执行 | — | — |
| B13 | engine_pdfium.rs OnceLock 单例（失败可重试）；Core 无 Clone/Debug | 待执行 | — | — |
| B14 | op-first 树模型（非快照即模型） | 待执行 | — | — |
