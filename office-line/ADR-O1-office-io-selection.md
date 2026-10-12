# ADR-O1：Office IO 面选型（xlsx 读写 + docx 读写 + zip 容器）

> 状态：**建议稿**（office-impl T2 产出，循 ADR-P1 先例；最终采纳与
> 落地排期留用户裁决）。日期：2026-10-12。证据 = office-e 一手核验
> （office-line/office-e_Rust生态选型pass.md，锚点在档）。

## 决策

| # | 面 | 选型 | 版本基线 | 触发条件 |
|---|----|------|----------|----------|
| O1-1 | xlsx 读 | **calamine** | 0.36.x | mdc-office IO 模块立项 |
| O1-2 | xlsx 写 | **rust_xlsxwriter**（constant_memory） | 0.99.x | 同上 |
| O1-3 | docx 写 + 模型级读 | **docx-rs**（docx-core） | 0.4.2x | 同上 |
| O1-4 | xlsx 读改写一体 | umya-spreadsheet（按需引入，不预进） | 3.1.x | 编辑往返特性立项时 |
| O1-5 | zip 容器 | **zip**（**zip-rs/zip2 线**，≥9.x；禁 zip-rs/zip master 0.6.x 停滞线） | 9.x | 同 O1-1（容器限额门） |
| O1-6 | docx 渲染级读侧 | **空位登记，不自研先定** | — | 用户裁决（见开放项） |

## 依据摘要

1. **许可全宽松**（五对象 4×MIT + 1×MIT/Apache 双许可，LICENSE 文件
   一手直读）——引入零净室义务。
2. **能力对口**：xlsx 读（calamine 格式覆盖 xls/xlsx/xlsb/ods + 惰性
   + UnzipXMLSizeLimit 内存限额）；xlsx 流式写（rust_xlsxwriter
   constant_memory，2026-10-01 仍在修该模式性能——在役证据）；
   docx 同模型读写（docx-rs read_docx 进 writer 同一 Docx 模型，
   reader 78 子模块）。
3. **与 office-d JS 四件套的分工**：JS 四件套 = dsh 生成态出口的
   程序化拼装层（office-d §8 路线 A）；本选型 = **引擎侧**（Rust）
   的 IO 面——消毒五面（mdc-office sanitize，已落地 T1）+ 解析/
   产出。两层并存不冲突：dsh 直拼走 JS，引擎托管走 Rust，消毒面
   两层共用（引擎侧已备）。
4. **zip 版本陷阱**（供应链）：crates.io `zip` 的活跃源 = zip-rs/zip2
   仓；zip-rs/zip 仓 master 停滞 0.6.x（2024-05）。锁定获取渠道为
   crates.io（Cargo 依赖即天然走此），升级审计对照 zip2 仓 CHANGELOG。

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| docx-rs 上游合入下游 fork 功能线（FORK_CHANGES.md 在档） | 升级时读 CHANGELOG + 锁 Cargo.lock 精确版本；破坏性变更先跑读侧往返测试 |
| zip 9.x 依赖线与 calamine/rust_xlsxwriter 内嵌 zip 8.x 并存 | 同 crates.io 名单一包解析，Cargo 统一收敛到 lock 版本；不引第二 zip 实现 |
| 声明尺寸谎言头（虚报巨型声明值） | 预扫求和门（SECURITY_POLICY §2.1）——读前拒绝，不进解压 |
| 渲染级 docx 读侧自研成本（office-d §6.3 目标线） | 开放项 O1-6 留裁决：先模型级（docx-rs）满足生成态校验往返，渲染级立项单议 |

## 落地前置（后续轮，非本轮）

1. mdcore NOTICES/SBOM 增量流程（新增三方依赖走 compliance_release
   登记纪律——首次引 Office 线依赖时的合规增量）。
2. 敌意输入探针校准（T3 spike 的限额实测数字回填 SECURITY_POLICY
   §2 限额表）。
3. mdc-office IO 模块设计（读骨架 = office-d §6.3 数据骨架级起步）。

## 开放项（留用户裁决）

- **O1-6**：docx 渲染级读侧（结构忠实 + 显式分页 + 按需渲染）立项
  与否、时机（先 JS 参照件过渡 vs 直接 Rust 自研）。
- **O1-4**：xlsx 编辑往返（读改写一体）是否近期能力需求。
