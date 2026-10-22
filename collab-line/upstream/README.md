# Upstream issue drafts — yrs 0.28（上游询问候选，呈用户择机提交）

> **性质**：交付报告 §4.1「上游询问候选」的落档件。两份英文 issue
> 草案（p20 捕获窗缺陷 / map-undo 交错语义），每份自含最小复现件
> 与证据链，**ready-to-file**。
>
> **纪律**：
> - 以用户 GitHub 身份（mmqz）向上游 y-crdt/yrs 提交 issue 属外部
>   动作——**呈用户裁决后执行**，本目录仅备稿。
> - 提交前复验三件：①yrs 版本仍为 0.28.0 或复现件在新版重跑仍
>   触发（crates.io 已核 2026-10-22：0.28.0 = 最新）；②上游
>   issue 搜索无同题在档（分析轮已核无报告——提交日复检）；
>   ③复现件独立可跑（`cargo script` 或新 crate 依赖 yrs 0.28）。
> - 提交仓库：`yjs/y-crdt`（yrs 的 GitHub 家）。

## 文件

| 草案 | 对应缺陷/语义 | 产品侧缓解（在档） |
|---|---|---|
| `p20-capture-window-undo-bug.md` | 双 UndoManager + 捕获窗>0 + 同窗非连续合并项 → undo 部分应用 + redo panic（`Store::materialize` unwrap None） | 窗 0 硬编码 + 交错回归锁（undo_redo.rs `p20_interleaved_undo_regression_lock`） |
| `map-undo-interleaved-semantics.md` | 同会话同键交错写的撤销 = 逐笔回退（历史点值遮蔽后写）+ pop-until-productive（一次 undo 消耗多栈项——单次撤空键首写时键整个移除） | 语义钉定测试（canvas undo_redo.rs `per_actor_interleaved_undo`）+ 已知语义登记 |

## 复现件验证状态（2026-10-22 亲跑）

草案内最小复现件已在独立验证 crate 逐字落地并运行
（`scripts/upstream_repro_verify/`，yrs 0.28.0 = crates.io 最新）：

- **p20**（release profile）：`after undo-human: "A1H2A2"`（部分撤销）
  + `redo-human PANICKED`（store.rs:340 unwrap None）——两症状与
  草案声明逐字一致。**附加发现**：debug profile 下更早触发
  `block.rs:652 subtract with overflow`（同一缺陷路径的溢出检查面
  ——草案已注明，可作维护者定位辅助）。
- **map-undo**（release profile）：`3 → 2 → 1` 逐笔回退 + 单次
  human undo 消耗全部 3 栈项且键整体移除（`<none>`）——与草案
  声明一致（行为 2 的最强形态：H1 为键首写，撤空即删键）。

## 与产品的关系

两件均为「有缓解/有钉定、无上游答案」——若上游确认缺陷并修复，
p20 窗 0 硬编码可解除（回归锁强制解钉，见 mdc-collab lib.rs 模块
文档）；map-undo 若确认为预期语义，产品语义登记从「探针钉定」
升「上游确认」（无行为变更）。#676（heap UAF）为另一上游跟踪项
（DECISIONS #19），已有在档 issue，不在本目录。
