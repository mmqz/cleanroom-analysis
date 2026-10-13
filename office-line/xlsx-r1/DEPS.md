# XLSX IO 依赖锁定记录（t10-f）

> 定位：xlsx-r1 PLAN §2 t10-f——用户指令 1：「依赖按已验证候选方案推进，
> **锁定版本并记录许可证、依赖闭包、适用边界**」。数据源 = spike
> Cargo.lock（提交钉版本）+ crates.io API 逐一核验（2026-10-13）+
> office-e 一手核验（浅克隆 LICENSE 直读，2026-10-12）。
> 生效面：mdcore 阶段 B 落地（`crates/office-io` 依赖声明——双 crate
> 方案 A，REPORT.md §2；2026-10-13 已落地，mdcore main 880864e）。

## 1. 根依赖（锁定版本）

| crate | 锁定版本 | 许可证（一手核验） | 角色 | 适用边界 |
|-------|----------|--------------------|------|----------|
| calamine | **0.36.1** | MIT | 读 | **仅读取**（值/公式文本/合并/表名四 API 面）；无样式/关系可观测面（FIDELITY.md §4） |
| rust_xlsxwriter | **0.99.1**（feature: constant_memory） | **MIT OR Apache-2.0**（阶段 B 修正：registry manifest 双许可直读；首版记 MIT。按 MIT 履行，ADR-C1 §1 允许域） | 写（dev 面：测试语料构造） | **仅从零新建**；不宣称已有工作簿编辑往返（FIDELITY.md 定谳） |
| zip | **8.6.0**（对齐后唯一版本，与传递面一致） | **MIT**（阶段 B 修正：manifest 与 LICENSE 文件均单 MIT 直读；首版记 MIT OR Apache-2.0 系上游仓历史版本口径混淆。三链核验在档） | 容器双门 | 门 1 声明值四检查 + 门 2 尺寸诚实性（策略层，见下） |

**版本并存登记 + 对齐定谳（2026-10-13 实测）**：calamine 0.36.1 与
rust_xlsxwriter 0.99.1 的传递依赖均为 **zip ^8.6.0**；spike 曾直依赖
zip 9（探针/预扫用）。**已对齐 zip = "8.6" 并全量回归**（lib 25/25 +
harness 17/17 + corpus + fidelity，API 差异仅 name() 形态[9.x
Result<Cow>→8.6 &str]，预扫面等价）。

**对齐的关键安全发现（lie86 探针定谳）**：zip **8.6 读界无声明值
强制**——谎言头条目（声明 16B 实际 1MB，CRC 正确）read_to_end
全量读回零错误（仅 EOF CRC 校验，攻击者可算对绕过）；T3 3b 验证的
内建拒绝（"File is larger than its declared uncompressed size"）为
**zip 9.x 独有**。而 calamine 0.36.1 内部依赖 8.6 → 实际读路径无
此防线。**闭环**：策略层 `verify_actual_sizes`（io.rs）逐条目有界
读取计数，实际 > 声明即 typed reject（门 2）——harness mal6a 升级
为 REJECT + lib 单测双验证。SECURITY_POLICY §2.1 第 2 项同步校准
（"库内建"→"策略层实现"）。

## 2. 供应链陷阱重申（ADR-O1 O1-5）

zip-rs/zip **master 分支停滞于 0.6.x（2024-05）**，活跃线在
zip-rs/zip2 仓（crates.io 包名仍为 `zip`，9.x）。依赖声明与审计须
以 crates.io `zip` 为准，**不得克隆 zip-rs/zip master 作参照**
（office-e §0 一手核验锚点）。

## 3. 依赖闭包（spike Cargo.lock 全量 51 外部 crate，2026-10-13）

分组（角色面）与许可证（crates.io API 逐条核验；`src` 列 = 本地
registry 源码 Cargo.toml 直读，`api` 列 = crates.io API）：

**根件（3）**：calamine 0.36.1 MIT · rust_xlsxwriter 0.99.1 MIT ·
zip 8.6.0 MIT OR Apache-2.0（对齐后 zip 9.0.2 支线移出闭包）

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

**许可证总评**：spike 闭包 50 外部 crate **全部宽松许可，零 copyleft**。
OR 型多许可（Unlicense OR MIT / Zlib OR Apache-2.0 / 三选一等）
一律按 MIT 或 Apache-2.0 选项消费（权利保留选择权）。两个 AND
附加条款需 NOTICES 增量注意：①encoding_rs 的 BSD-3-Clause
（署名条款）；②unicode-ident 的 Unicode-3.0（Unicode 署名条款）。
**阶段 B 落地分布（NOTICES v1.3 实测）**：mdcore lock 新增 16 外部
crate = 9 进产品 normal 面（calamine/zip/quick-xml/codepage/
fast-float2/atoi_simd/debug_unsafe/typed-path/zopfli）+ 2 项由
NOTICES v1.2 分类账提升进编译面（rustversion[atoi_simd proc-macro
硬边]/zlib-rs[zip→flate2 zlib-rs 后端边]——两者与 encoding_rs/
unicode-ident 等 12 项原 PDF 面传递依赖因 office-io 进 default 面而
同入，分类账与正文的全量对账见 NOTICES v1.3 §SBOM 复跑）+ 7 项
dev/lock 面（rust_xlsxwriter dev + tempfile 族 6）；encoding_rs 与
unicode-ident 两 AND 条款已随 v1.2 在档（BSD-3-Clause / Unicode-3.0
全文随附义务兑现），v1.3 零新 AND 条款。

## 4. 适用边界（能力宣称纪律——与 CAPABILITIES.md 互引）

1. **读 = calamine 面**：值矩阵/公式文本/合并范围/表名（read_proto
   四 API 面）；样式/关系引用/元数据字节保真**不在面内**。读门 =
   声明值四检查 + 谎言头闭环（策略层 verify_actual_sizes）。
2. **写 = 从零新建**：rust_xlsxwriter constant_memory 流式（T3 探针
   100 万格 972ms 实测）。
3. **已有工作簿编辑往返 = 未证明支持**：t10-c FIDELITY.md 丢失矩阵
   定谳（样式/关系结构性丢失）——**不得宣称**。
4. **zip 预扫门 = 输入防线**：四检查 + typed reject + 门 2 尺寸
   诚实性（read_proto 17/17 实证，zip 8.6 对齐态）。

## 5. 版本纪律

- ~~阶段 B 依赖声明（`crates/office-io/Cargo.toml`，双 crate 方案 A
  ——REPORT.md §2）按本文件锁定版本书写~~ **已落地（2026-10-13，
  mdcore main 880864e）**：`crates/office-io` [dependencies] =
  calamine "0.36" + zip "8.6"（deflate，default-features off）；
  [dev-dependencies] = rust_xlsxwriter（0.99 constant_memory，语料
  构造）+ flate2（谎言头测试流构造）。Cargo.lock 钉死闭包；NOTICES
  v1.3 入账（16 新 crate：9 进产品 normal 面 + rustversion/zlib-rs
  由 v1.2 分类账提升 + 7 项 dev/lock 面）；升版触发 SECURITY_POLICY
  §5 复核纪律（zip 或解析库大版本升级 = 复核触发）。
- 本文件与 spike Cargo.lock 同步维护（新增依赖 = 本文件增行 +
  许可证核验补全，缺一不合并）。
