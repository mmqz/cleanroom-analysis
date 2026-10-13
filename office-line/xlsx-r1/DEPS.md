# XLSX IO 依赖锁定记录（t10-f）

> 定位：xlsx-r1 PLAN §2 t10-f——用户指令 1：「依赖按已验证候选方案推进，
> **锁定版本并记录许可证、依赖闭包、适用边界**」。数据源 = spike
> Cargo.lock（提交钉版本）+ crates.io API 逐一核验（2026-10-13）+
> office-e 一手核验（浅克隆 LICENSE 直读，2026-10-12）。
> 生效面：mdcore 阶段 B 落地（`crates/office` IO 模块依赖声明）。

## 1. 根依赖（锁定版本）

| crate | 锁定版本 | 许可证（一手核验） | 角色 | 适用边界 |
|-------|----------|--------------------|------|----------|
| calamine | **0.36.1** | MIT | 读 | **仅读取**（值/公式文本/合并/表名四 API 面）；无样式/关系可观测面（FIDELITY.md §4） |
| rust_xlsxwriter | **0.99.1**（feature: constant_memory） | MIT | 写 | **仅从零新建**；不宣称已有工作簿编辑往返（FIDELITY.md 定谳） |
| zip | **9.0.2**（spike 直依赖）/ **8.6.0**（传递） | MIT OR Apache-2.0 | 容器预扫门 | 声明求和/单条目/条目数/路径逃逸四检查（read_proto.rs 实证） |

**版本并存登记**：calamine 0.36.1 与 rust_xlsxwriter 0.99.1 的传递
依赖均为 **zip ^8.6.0**；spike 直依赖 zip 9（探针/预扫用）。两代
并存 = Cargo 语义允许（不同 major 不冲突），但 **阶段 B 建议对齐
zip = "8.6"**：预扫 API 面（`size() -> u64` / `enclosed_name() ->
Option<PathBuf>`）在 8.6.0 逐一在场（本文件核验日直读 8.6.0 源码），
对齐后闭包收敛（去 zip 9 支线），`with_size_limit` 谎言头内建拒绝
在 8.6 同在（T3 3b 探针时为 9.x 实测，阶段 B 落地时**须以 8.6 重测**
——重测项登记，不假设跨 major 行为一致）。

## 2. 供应链陷阱重申（ADR-O1 O1-5）

zip-rs/zip **master 分支停滞于 0.6.x（2024-05）**，活跃线在
zip-rs/zip2 仓（crates.io 包名仍为 `zip`，9.x）。依赖声明与审计须
以 crates.io `zip` 为准，**不得克隆 zip-rs/zip master 作参照**
（office-e §0 一手核验锚点）。

## 3. 依赖闭包（spike Cargo.lock 全量 51 外部 crate，2026-10-13）

分组（角色面）与许可证（crates.io API 逐条核验；`src` 列 = 本地
registry 源码 Cargo.toml 直读，`api` 列 = crates.io API）：

**根件（3）**：calamine 0.36.1 MIT · rust_xlsxwriter 0.99.1 MIT ·
zip 9.0.2 MIT OR Apache-2.0

**XML/解析（4）**：quick-xml 0.41.0 MIT（api）· serde 1.0.229
MIT OR Apache-2.0 · serde_derive 1.0.229 MIT OR Apache-2.0（api）·
serde_core 1.0.229 MIT OR Apache-2.0

**编码/码页（4）**：encoding_rs 0.8.42 (Apache-2.0 OR MIT) AND
BSD-3-Clause · codepage 0.1.3 Apache-2.0 OR MIT（api）·
simdutf8 0.1.5 MIT OR Apache-2.0 · fast-float2 0.2.4 MIT OR
Apache-2.0（api）

**数值/SIMD 工具（6）**：atoi_simd 0.18.1 MIT OR Apache-2.0（api）·
byteorder 1.5.0 Unlicense OR MIT · memchr 2.8.3 Unlicense OR MIT ·
debug_unsafe 0.1.4 Apache-2.0 OR MIT · rustversion 1.0.23 Apache-2.0
OR MIT · core_detect 1.0.0 MIT OR Apache-2.0 · multiversion_no_op
1.0.0 Apache-2.0 OR MIT

**压缩（6）**：flate2 1.1.10 MIT OR Apache-2.0 · zip 8.6.0（传递）
MIT OR Apache-2.0 · crc32fast 1.5.2 MIT OR Apache-2.0（api）·
miniz_oxide 0.9.1 MIT OR Zlib OR Apache-2.0 · adler2 2.0.1 0BSD OR
MIT OR Apache-2.0 · simd-adler32 0.3.10 MIT（api）· zlib-rs 0.6.8
Zlib

**zip 支线（4）**：zopfli 0.8.4 Apache-2.0（api）· typed-path
0.12.3 MIT OR Apache-2.0（api）· indexmap 2.14.2 Apache-2.0 OR
MIT · equivalent 1.0.2 Apache-2.0 OR MIT · hashbrown 0.17.1 MIT OR
Apache-2.0

**临时文件/IO（5）**：tempfile 3.27.0 MIT OR Apache-2.0 · fastrand
2.5.0 Apache-2.0 OR MIT · once_cell 1.21.4 MIT OR Apache-2.0 ·
getrandom 0.4.3 MIT OR Apache-2.0 · cfg-if 1.0.5 MIT OR Apache-2.0

**平台/系统（12）**：rustix 1.1.5 Apache-2.0 WITH LLVM-exception OR
Apache-2.0 OR MIT · linux-raw-sys 0.12.1 同前 · nix 0.31.3 MIT
（api）· libc 0.2.190 MIT OR Apache-2.0 · errno 0.3.14 MIT OR
Apache-2.0 · bitflags 2.13.2 MIT OR Apache-2.0 · cfg_aliases 0.2.2
MIT（api）· r-efi 6.0.0 MIT OR Apache-2.0 OR LGPL-2.1-or-later
（api）· windows-sys 0.61.2 MIT OR Apache-2.0（api）·
windows-link 0.2.1 MIT OR Apache-2.0（api）· log 0.4.34 MIT OR
Apache-2.0（api）· scopeguard 1.2.0 MIT OR Apache-2.0

**proc-macro/编译期（8）**：proc-macro2 1.0.107 · quote 1.0.47 ·
syn 3.0.8 · unicode-ident 1.0.27（(MIT OR Apache-2.0) AND
Unicode-3.0，api）· bumpalo 3.20.3 MIT OR Apache-2.0 —— 前三者均
MIT OR Apache-2.0（api）

**许可证总评**：51 外部 crate **全部宽松许可，零 copyleft**。
OR 型多许可（Unlicense OR MIT / Zlib OR Apache-2.0 / 三选一等）
一律按 MIT 或 Apache-2.0 选项消费（权利保留选择权）。两个 AND
附加条款需 NOTICES 增量注意：①encoding_rs 的 BSD-3-Clause
（署名条款）；②unicode-ident 的 Unicode-3.0（Unicode 署名条款）。
阶段 B NOTICES/SBOM 增量流程须把这两条带全文（mdcore r24
NOTICES 纪律）。

## 4. 适用边界（能力宣称纪律——与 CAPABILITIES.md 互引）

1. **读 = calamine 面**：值矩阵/公式文本/合并范围/表名（read_proto
   四 API 面）；样式/关系引用/元数据字节保真**不在面内**。
2. **写 = 从零新建**：rust_xlsxwriter constant_memory 流式（T3 探针
   100 万格 972ms 实测）。
3. **已有工作簿编辑往返 = 未证明支持**：t10-c FIDELITY.md 丢失矩阵
   定谳（样式/关系结构性丢失）——**不得宣称**。
4. **zip 预扫门 = 输入防线**：四检查 + typed reject（read_proto
   17/17 实证）；谎言头内建拒绝重测项见 §1。

## 5. 版本纪律

- 阶段 B 依赖声明（`crates/office/Cargo.toml`）按本文件锁定版本书写；
  Cargo.lock 钉死闭包；升版触发 SECURITY_POLICY §5 复核纪律
  （zip 或解析库大版本升级 = 复核触发）。
- 本文件与 spike Cargo.lock 同步维护（新增依赖 = 本文件增行 +
  许可证核验补全，缺一不合并）。
