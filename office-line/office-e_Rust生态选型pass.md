# office-e：Rust OOXML 生态选型 pass（office-impl T2 侦察报告）

> Task ID：office-e / t9-d ｜ 日期：2026-10-12 ｜ 性质：**选型侦察 pass**
> （一手核验：五仓浅克隆只读 + LICENSE 文件直读 + Cargo.toml/源码锚点）
> 输入：office-impl PLAN v1 T2（用户指令第 3 条另轮启动）+ 05 §8.1
> Rust 生态留白 + office-d §8 JS 四件套分工定位。
> 上游仓：`office-line/_upstream/`（本地，不入 git——净室分区纪律）。

## 0. 对象与版本锚定（含许可逐件核验）

| 对象 | 仓（一手克隆） | 版本（Cargo.toml 直读） | 许可（LICENSE 文件直读） | HEAD 提交 |
|------|--------------|------------------------|--------------------------|-----------|
| xlsx 读 | tafia/calamine | 0.36.1（Cargo.toml:3） | **MIT**（LICENSE-MIT.md，© 2016 Johann Tuffe） | 2026-09-02（deps: update quick-xml） |
| xlsx 写 | jmcnamara/rust_xlsxwriter | 0.99.1（Cargo.toml:11） | **MIT OR Apache-2.0** 双文件（LICENSE_MIT + LICENSE_Apache2.0） | 2026-10-01（constant_memory 二次扫修复——流式面在役维护） |
| xlsx 读写对照 | MathNya/umya-spreadsheet | 3.1.1（Cargo.toml:3） | **MIT**（LICENSE，© 2020 MathNya） | 2026-10-12（3.1.1 release 准备） |
| docx 写+读 | bokuweb/docx-rs | docx-core 0.4.22（docx-core/Cargo.toml:3） | **MIT**（LICENSE，© 2020 bokuweb） | 2026-10-05（perf(reader) 优化 #930） |
| zip 容器 | **zip-rs/zip2**（crates.io 名 `zip`） | 9.0.2（Cargo.toml:3） | **MIT**（LICENSE，© 2014 Mathijs van de Nes） | 2026-10-11（9.0.2 CHANGELOG） |

许可结论：五对象全宽松（4×MIT + 1×MIT/Apache 双许可），净室零义务，
与 office-d JS 面（3×MIT + Apache）同档——**引入无许可障碍**。

⚠️ **供应链陷阱登记（一手发现）**：`zip-rs/zip` 仓的默认分支 master
**停滞在 0.6.x 线**（HEAD 2024-05-10，Cargo.toml:3 = 0.6.6）；活跃开发
线在 **zip-rs/zip2** 仓（crates.io 包名仍是 `zip`，当前 9.0.2）。获取
渠道必须走 crates.io / zip2 仓，**不得克隆 zip-rs/zip master**。

## 1. xlsx 读面：calamine

- **格式覆盖**（README:11-12）：excel 族（`xls`/`xlsx`/`xlsm`/`xlsb`/
  `xla`/`xlam`）+ `ods`——五对象中覆盖最广（含二进制 xls 与 ods）。
- **惰性加载**（README:115-117）：xlsx/xlsb 支持按需 sheet 迭代，
  xls/ods 整体载入。
- **内存限额机制**（README:374，一手文档）：`UnzipXMLSizeLimit` 选项
  ——worksheet 与共享字符串表解压超限时**落系统临时目录**（非全内存），
  大文件读有界。这是敌意输入姿态的核心正面证据。
- **入口 API**（src/lib.rs:493/503）：`open_workbook` / `open_workbook_
  from_rs`（文件与字节流双入口）。
- **依赖面**（Cargo.toml:16-26）：zip 8.6（default-features=false +
  deflate）、quick-xml 0.42、encoding_rs、fast-float2、byteorder、
  codepage、atoi_simd、log、serde——容器与 XML 均为成熟件。

## 2. xlsx 写面：rust_xlsxwriter

- **流式写**（README:184；src/lib.rs:280）：`constant_memory` feature
  （tempfile 依赖，Cargo.toml:64）——大批量写内存最小化；HEAD 提交
  （2026-10-01）正是该模式的二次扫描性能修复——**流式面在役维护中**
  （对照 office-d §1.2：JS 面 exceljs WorkbookWriter 唯一真流式，
  Rust 面对应件即此）。
- **性能选项**（Cargo.toml:50-52）：`zlib` feature 1.5× 加速（引 C 编
  译器）；默认零 C 依赖（zip 8.3 default-features=false，Cargo.toml:17）。
- **rust-version 1.88**（Cargo.toml:13）——低于本环境 1.99.0，无门槛。

## 3. xlsx 读写一体（对照件）：umya-spreadsheet

- 定位（README:10）：纯 Rust xlsx 读+写。**编辑往返场景**（读→改→写
  保结构）的候选——calamine 只读、rust_xlsxwriter 只写，读改写一体
  唯此件。
- 依赖面（Cargo.toml:15-36）较重：aes/cbc/cfb（加密件）、fancy-regex、
  chrono、quick-xml 0.41（serialize feature）、zip 8.6。
- 活跃度极高（HEAD = 3.1.1 release 准备，2026-10-12 当日）。

## 4. docx 写+读面：docx-rs

- **读面存在且同模型**（README:6/62；docx-core/src/reader/read_docx.rs）：
  `read_docx` / `read_docx_with_options`——解析进**与 writer 同一个
  `Docx` 模型**（往返一致的设计承诺）。reader 模块面可观：
  docx-core/src/reader/ 下 **78 个子模块文件**（数了一手）。
- **wasm 面**：docx-wasm 伴随 crate（仓内目录一手在位）——JS 侧消费
  先例。
- ⚠️ **维护面观察项（一手发现）**：仓内存 FORK_CHANGES.md（"gaugo123/
  docx-rs fork adds features MDtoWord needs, pending upstream
  adoption"）——上游已合入下游 fork 功能（HEAD perf(reader) 即来自
  该线 PR #930）。非阻塞，登记为供应链观察项（升级时读 CHANGELOG）。

## 5. docx 读面盘点结论（发现即结论）

**Rust 模型级 docx 读写存在**（docx-rs，§4）——05 §8.1 "开源 OOXML
进出 = 蓝海"的读侧**需校准**：蓝海不在模型解析（已有人做），在
**渲染保真级读侧**（office-d §6.3 目标线：docx-preview 级结构忠实 +
显式分页 + 按需渲染——docx-rs reader 是模型解析，不产出布局/渲染）。
与 Univer xlsx 全 Pro、OnlyOffice AGPL 的格局合并看：**xlsx 引擎级
读写开源件齐备（选型问题），docx 渲染级读侧 Rust 空位（自研问题）**。

## 6. zip 容器防御面（安全策略 v1 的工程依据）

- **声明尺寸暴露**（src/types.rs:33/58）：条目 `uncompressed_size`
  （header 声明值，pub 字段）——**读前预扫求和**可行（策略层对照
  限额表拒绝超额档，即 office-b `checkInputLimits` 的 Rust 对应）。
- **读界**（src/types.rs:106；src/crc32.rs:19/38/45）：`size_limit`
  取声明值 + `Crc32Reader::with_size_limit`——"Fail with an error
  once more than size_limit bytes have been read"（**反谎言头炸弹**：
  实际膨胀超声明即截断报错，不按实际尺寸放内存）。
- 防御模型结论：**声明求和预扫（策略层）+ 声明值读界（库内建）**
  两层闭环；声明值本身虚报巨型（如 10GB）由预扫拒绝——预扫是必须的
  门，不是可选优化。

## 7. 选型矩阵与 ADR-O1 摘要（详见 ADR-O1）

| 面 | 推荐 | 备选 | 关键依据 |
|----|------|------|----------|
| xlsx 读 | **calamine** | umya | 格式覆盖最广 + 惰性 + 内存限额 + MIT |
| xlsx 写 | **rust_xlsxwriter** | umya | constant_memory 流式在役 + 双许可 |
| xlsx 读改写一体 | umya-spreadsheet | — | 编辑往返场景按需引入 |
| docx 写 | **docx-rs** | — | 唯一成熟 Rust 件 + 同模型读 + wasm |
| docx 读（模型级） | docx-rs reader | — | 同模型往返 |
| docx 读（渲染级） | **空位登记** | docx-preview（JS 参照） | 自研裁决项（T4 整理） |
| zip 容器 | **zip（zip2 线 ≥9.x）** | — | 限额 API 在位；master 停滞陷阱 §0 |

## 8. T3 探针实测（office-line/spike，2026-10-12，release 构建）

| 探针 | 内容 | 结果 | 时间 |
|---|---|---|---|
| 1 读·常规 | 3 sheet×2000 行×26 列（156,000 格） | 打开+全量提取+抽点断言+全量对账通过 | 6.00 ms + 42.24 ms |
| 1 读·大表 | 1 sheet×50,000 行×10 列（500,000 格） | 5 万行对账通过 | 53.14 ms + 139.05 ms |
| 2 写·流式 | constant_memory 100,000 行×10 列（100 万格） | 写+保存通过，产物 3.33 MB | 971.50 ms |
| 2 写读往返 | calamine 重开+末格断言+行数对账 | 通过（O1-1+O1-2 组合互证） | 267.02 ms |
| 3a 声明预扫 | 3 条目（含 2MB 件与路径逃逸名） | 声明求和 + 逃逸名识别（零数据读取） | 0.01 ms |
| 3b 谎言头 | deflate 声明 16B/实际膨胀 1MB | **库内建拒绝**（"File is larger than its declared uncompressed size"） | — |
| 3c 巨量 dimension | 声明 A1:ZZZ9999999（≈1.7e12 格） | **无害**（Range 按实际单元格定尺寸 2×2，正常提取） | 0.05 ms |

探针结论：①读面量级合格（50 万格全量提取 139ms 级）；②写面流式
在役（100 万格 972ms 级 + 3.33MB）；③**推荐件组合互证成立**（写读
往返断言全过）；④敌意输入三条全部有库内建或机制级答案（预扫机制
实证 + 谎言头内建拒绝 + dimension 无害）——SECURITY_POLICY §3 限额
表已按实测校准回填。

## 附：锚点统计

- 五仓浅克隆只读（tafia/calamine@0.36.1 HEAD、jmcnamara/
  rust_xlsxwriter@0.99.1 HEAD、MathNya/umya-spreadsheet@3.1.1 HEAD、
  bokuweb/docx-rs@docx-core 0.4.22 HEAD、zip-rs/zip2@9.0.2 HEAD；
  另 zip-rs/zip master 0.6.6 停滞线核验后弃用）。
- 锚点类型：LICENSE/Cargo.toml 直读（版本与许可）、README/源码
  路径:行号（能力与限额）、git log（活跃度）、目录清点（reader 78
  子模块）。
- 【推断】标注：0 处（本 pass 全部一手直读；渲染级读侧空位为对照
  office-d §6.3 目标线的校准结论，依据在档）。
