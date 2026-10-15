# Office 线敌意输入安全策略（SECURITY_POLICY v1.2）

> 状态：**v1.2 生效（2026-10-17，Round 4 T2 导入/导出资源上限全面
> 实装同步升版：§2.1 门 0[输入字节上限四入口] + 门 1 族分化[docx 族
> 50MB 执行点补齐] + 门 3 增稠密包围盒扫描；§2.2 读面语义层两限
> [表数/格面积]；§3 限额表全量校准[每项：执行点锚点 + 默认值 +
> 测试锚点——「仅文档宣称」项全部翻转为「已执行」或明确标注无
> 执行点]；§3.1 可复现基准）**。前身：v1.1（2026-10-14，门 3 实装）；
> v1.0（2026-10-13，mdc-office-io 实装）；v1 草案（2026-10-12
> T2+T3 产出；2026-10-13 lie86 谎言头项校准）——输入：office-b
> §3.5（x2t 五件套一手限额参照）+ office-a §9（边界参照）+ office-d
> §5（输出面）+ office-e §6/§8（Rust zip 限额一手证据 + 探针实测）
> + L2/03 敌意输入方法论（五层纵深/三行为学/沙箱四件套）。
> 生效面：mdc-office-io（XLSX/DOCX 读路径全门 + 写面资源限，
> mdcore r4-t2 提交）+ mdc-office sanitize 六面 + mdcore-wasm 错误码
> 接线（`xlsx_limit`/`docx_limit`）。

## 1. 威胁模型

敌意**输入**面（打开/解析非受信 office 文件）：

| # | 威胁 | 载体 | 参照 | v1.2 状态 |
|---|------|------|------|-----------|
| T1 | zip 解压炸弹（实际膨胀 ≫ 声明） | OOXML 容器（xlsx/docx/pptx 均 zip） | office-b §3.5 | 门 2 拦（既有） |
| T2 | 谎言声明头（声明值本身巨型） | zip 条目 uncompressed_size | office-e §6 | 门 1 拦（既有） |
| T3 | 巨量单元格/超界 dimension 声明 | worksheet XML | 05 §4.1 问 6 | **稀疏包围盒炸弹闭项**（v1.2：T3 原判「实测无害」仅对 dimension 声明成立；实际载荷格坐标分布经 calamine `Range::from_sparse` 稠密化——1588B 两格探针实测 ~550GB 分配 abort → 门 3 坐标扫描修复） |
| T3b | 稀疏包围盒炸弹（载荷格坐标极散 → 稠密化分配爆炸） | worksheet XML c/@r 坐标 | r4-t2 探针（分析仓 spike sparse_bomb） | **门 3 拦（v1.2 新增）** |
| T4 | 深度爆炸公式/巨量区间引用 | 公式体（公式引擎立项后生效） | 05 §4.2 | 登记未立项 |
| T5 | 条目路径逃逸（`../` 绝对路径） | zip entry name | office-b §3.5 | 门 1 拦（既有） |
| T5b | 前缀垃圾/伪装载荷（SFX 形态超大输入） | 输入文件字节 | office-b checkInputLimits 同构 | **门 0 拦（v1.2 新增）** |
| T6 | 外部资源拉取（INCLUDEPICTURE/外链） | 文档体 | office-d §5.3 | 输出面白名单对称（既有） |
| T7 | 空表名洪水（CPU 面：巨量 `<sheet>` 名逐表重解析） | workbook.xml | r4-t2 分析 | **读面语义层拦（v1.2 新增）** |

敌意**输出**面（LLM/用户可控文本落产物）：**消毒六面**（已落地
mdc-office sanitize，mdcore main 880864e；第六面 url_policy 于
xlsx-r1 阶段 B 补面）——CSV 公式首字符 / XML 控制字符保字符档 /
XML 实体属性位 / HTML href / 域指令白名单 / URL scheme 白名单。
本策略不重复其规格（见 mdcore crates/office）。

**资源面（v1.2 扩充：产出面无界增长）**：合法载荷的内存/文件描述符
无界消耗（非敌意输入，属资源上限缺位）——读面全量提取（网格稠密化）
与写面引擎模型（内存 Vec / 每表 tempfile）——§3 写面限全部为此面。

## 2. 输入防线（分层，L2/03 五层纵深裁剪）

### 2.1 容器层（zip 门，读前强制）

0. **输入字节上限（门 0，v1.2 新增——先于任何全量读/分配）**：
   四入口（xlsx/docx × path/bytes）统一字节纪律——path 入口查
   元数据尺寸**先于** `fs::read`/打开（DOCX 路径此前 `fs::read`
   全量入内存无前置检查 = 真实内存面）；bytes 入口查入参 len。
   xlsx 族 = 族声明总量 + 2MB 结构边距（302MB）；docx 族 = 50MB +
   边距（52MB）。office 文档无 SFX 合法形态——超限即前缀垃圾/
   伪装载荷（T5b）。typed `ReadError::PrescanRejected`，两模式恒拒。
   （实现：io.rs `check_input_size` / docx.rs 门 0；测试：
   `input_size_limit_path_entry` / `input_size_limit_bytes_entry` /
   docx 双入口测试——恰=限过 + 限+1 拒 + 极端 4× 拒）
1. **声明求和预扫**（读任何条目前）：全条目 `uncompressed_size`
   求和 + 单条目上限对照 §3 限额表——超额即整档拒绝（typed reject，
   不进解压）。依据：office-e §6 + office-b checkInputLimits 同构
   （docx/pptx 50MB、xlsx 300MB 按格式族分档）。**v1.2 族分化
   落地**：此前 DOCX 读路径误用 xlsx 族 300MB 常量（v1.1 §3 docx
   族行无执行点）——本轮 `prescan_docx` 共享核参数化对齐 50MB。
   （实现：io.rs `prescan`/`prescan_docx`/`prescan_family`，门 1；
   测试：既有 mal6c/entry_count + `prescan_docx_family_limits` +
   `docx_family_declared_limit_end_to_end`——单条目/总量/恰=族限
   三面）
2. **实际尺寸诚实性验证**（策略层实现，2026-10-13 校准）：逐条目
   有界读取计数，实际 > 声明即 typed reject。校准依据：xlsx-r1
   lie86 探针定谳 zip **8.6 读界无声明值强制**（calamine 0.36.1
   内部依赖 8.6）→ 策略层补（io.rs `verify_actual_sizes`，门 2）。
   （反 T1；zip 9 的内建拒绝为 9.x 独有——升 zip 大版本时重测）
3. **条目路径校验**：entry name 拒绝绝对路径/`..` 逃逸/盘符位
   （反 T5；enclosed_name None 判定）。（实现：`prescan`，门 1）
4. **条目数上限**（DoS 面）：单档条目数 > 上限拒绝（§3 表）。
   （实现：`prescan`，门 1；mal6d 实测校准）
5. **XML 部件良构性验证 + 稠密包围盒扫描（门 3，v1.1 立 / v1.2
   增扫描）**：.xml/.rels 后缀部件在门 2 的既有解压趟内**融合单趟**
   流式校验（quick-xml：标签配对 + 根元素在位唯一 + EOF 未闭合
   元素 + 元素深度上限 §3 表）+ **r4-t2 坐标镜像扫描**（`GridScan`
   ——镜像 calamine 0.36.1 cells_reader 位置语义：row/@r 显式定位
   + 行结束游标推进 + c/@r 显式/顺序定位 + 载荷格 = 有 v/is/f 子
   [值面与公式面并集上界——styled 空格不计]）→ 单部件**稠密包围盒
   面积**检查先于 calamine `from_sparse` 稠密化分配（反 T3b——
   1588B 探针 → ~550GB 分配 abort 的闭环）。畸形部件默认严格
   typed reject（`ReadError::InvalidPart`）；显式兼容模式降级为
   结构化警告。**门 0/1/2 与全部安全上限（含面积/表数/Σ）两模式
   恒等**（兼容不放松安全防线）。门 2 双读受信源跳过开关经裁决
   **不设**（无基准证据不得加跳过——2026-10-14 用户指令）。
   （实现：io.rs `verify_pass`/`validate_xml_structure`/`GridScan`；
   测试：`sparse_grid_bomb_rejected_both_modes` /
   `grid_area_boundary_exact_limit_passes_over_rejected` /
   `grid_scan_mirror_semantics` + 既有 mal7e 全家）

### 2.2 解析/语义层（XML/模型门）

1. 巨量 dimension 声明：**T3 探针实测校准（2026-10-12）**：calamine
   不按 dimension 声明预分配（声明 A1:ZZZ9999999 档实测 0.05ms，
   Range 2×2）。**v1.2 切面注记**：该结论仅覆盖 dimension 声明面；
   **实际载荷格坐标分布**的稠密化炸弹是另一面（T3b，v1.2 闭项）。
2. 共享字符串表/worksheet 解压内存上限：走 calamine
   `UnzipXMLSizeLimit` 落盘机制（office-e §1 一手在档）。
3. XML 解析器事件深度上限（**v1.1 已闭项**：LIMIT_XML_DEPTH=1000，
   边界对测钉定）。
4. **工作表数量上限（v1.2 新增——语义层）**：workbook.xml ≤300MB
   可载 ~1200 万个 `<sheet>` 名（每名 ~25B）——空表名洪水 CPU 面
   （逐表重解析部件，T7）在提取循环之前 typed 拒。值 1,000 =
   C1 大档能力宣称面（真实工作簿百表级以内；wasm 渲染窗 16 表为
   消费子集，两层关系登记不收敛——读面全提取 + 渲染层截断计数）。
   （实现：io.rs `extract_sheets` names.len() 检查；测试：
   `sheet_count_limit_enforced`——恰 1000 全读 + 1001 拒 + compat 同）
5. **稠密包围盒格总面积上限（v1.2 新增——语义层）**：门 3 单部件
   面积检查之上，`extract_sheets` 再做**全簿累计**（多表名指向同
   部件的倍增保留面；values + formulas 两网格面积求和）——超限
   typed 拒且错误带表名（可诊断）。值 4,000,000 = C1 宣称面
   （50 万格）的 8× 边距；三网格峰值预算见 §3.1 基准。
   （实现：io.rs `extract_sheets` 累计；测试：
   `cumulative_grid_area_over_limit_rejected` + 边界测试通过面）

### 2.3 资源层（进程面，宿主接线时生效）

参照 office-b x2t 五件套：内存上限（RLIMIT 级，宿主装配）/ 打开
超时 / 下载配额（100MB·2m·3 次，若 IO 模块带远程取件）/ 私网守卫
（外链拉取禁私网目标）。本层属宿主接线轮，登记不在 v1/v1.2 实装
（**§3.1 基准即宿主装配的资源预算一手输入**）。

### 2.4 语义层（域/公式门，随能力立项生效）

外链域（INCLUDETEXT/INCLUDEPICTURE/LINK）输入侧同样拒绝（与输出
侧白名单对称——office-d §5.3）；公式深度/区间引用限额（T4 威胁，
公式引擎立项时定）。

## 3. 输入/输出限额表（v1.2 全量校准——每项：执行点锚点 + 默认值 + 测试锚点）

读面（office-io io.rs / docx.rs）：

| 维度 | v1.2 值 | 执行点锚点 | 测试锚点 | 校准依据 |
|------|---------|-----------|---------|---------|
| 输入字节上限（xlsx 族，path+bytes） | 302 MB（=声明总量+2MB 边距） | io.rs 门 0 `check_input_size`（四入口之一） | `input_size_limit_path_entry`/`_bytes_entry` | **v1.2 新设**（原无限制——office-b checkInputLimits 同构） |
| 输入字节上限（docx 族，path+bytes） | 52 MB（=50MB+边距） | docx.rs `read_docx_with_options` 元数据前置 + bytes 入口 len | `docx_input_size_limit_*` | **v1.2 新设**（原 fs::read 无前置 = 真实内存面） |
| zip 声明总量（xlsx 族） | 300 MB | io.rs `prescan`（门 1） | mal6c/T3 既有 | 机制实证（预扫 0.01ms 零数据读取）；office-b default.json:604-629 |
| zip 声明总量（docx 族） | 50 MB | io.rs `prescan_docx`（**v1.2 执行点补齐**——此前误用 xlsx 300MB 常量） | `prescan_docx_family_limits` + `docx_family_declared_limit_end_to_end` | office-b checkInputLimits 格式族分档；v1.1 §3 该行无执行点 → v1.2 闭环 |
| 单条目声明尺寸 | = 同族总量上限 | `prescan_family` single_limit 参数 | 同上（单条目面） | 对称设计 |
| 条目数上限 | 10,000 | `prescan` 门 1 | `entry_count_rejected` + mal6d | **实测校准**（10,001 件拒） |
| XML 部件元素嵌套深度 | 1,000 | `validate_xml_structure`（门 3 融合趟） | `gate3_depth_limit_enforced`（999+root 过/1000+root 拒） | v1.1 实测钉定 |
| 畸形 XML 部件 | 严格拒 / 兼容警告 | 门 3 `verify_pass` | mal7e 全家 + 兼容语料复扫 | v1.1 实测钉定 |
| 谎言声明头（实际 > 声明） | 拒绝 | 门 2 `verify_pass` 计数 | `lie_header_rejected_at_gate2` + mal6a | **策略层**（zip 8.6 无内建，lie86 探针） |
| **稠密包围盒格面积（单部件）** | 4,000,000 | 门 3 `GridScan.area()`（**v1.2 新增**） | `sparse_grid_bomb_rejected_both_modes` + 边界 4M 过/4M+2000 拒 | **稀疏炸弹探针闭环**（1588B → ~550GB 分配 abort） |
| **稠密包围盒格面积（全簿 Σ）** | 4,000,000 | `extract_sheets` 累计（**v1.2 新增**） | `cumulative_grid_area_over_limit_rejected` | 多表名倍增面 |
| **工作表数量（读面）** | 1,000 | `extract_sheets` 循环前（**v1.2 新增**） | `sheet_count_limit_enforced` | 空表名洪水 CPU 面（T7）；C1 宣称面 |
| dimension 巨量声明 | 登记观察（不设前置门） | **无执行点（有意不设**——实测无害，calamine 按实际格定尺寸） | T3 探针 | 实测无害 0.05ms |
| 共享字符串解压上限 | 256 MB | calamine `UnzipXMLSizeLimit` 机制位（值自设） | 机制在档 | office-e §1 |
| 路径逃逸条目 | 拒绝 | 门 1 enclosed_name 判定 | `path_escape_rejected` | 实测识别 |
| 文本长度（DOCX 读） | 不设独立限 | **无执行点（有意不设**——容器界族总量 50MB 内整包 XML；模型字符串为其子集，无安全增量） | — | v1.2 登记定谳 |
| 媒体文件大小（DOCX 读） | 不设独立限 | **无执行点（有意不设**——族界联合：单条目 ≤50MB + 全包媒体 Σ ≤50MB，ImageAsset 全量内存受族界约束） | D3 测试附带 | v1.2 登记定谳 |
| 关系数量（DOCX） | 10,000/部件 | docx.rs `insert_relationship`（两模式恒拒） | 既有 rels 测试 | 既有实测 |

写面（office-io write.rs / docx.rs——**v1.2 新设整面**，资源限
typed `WriteError::Limit`/`DocxError::Limit`，wasm 码
`xlsx_limit`/`docx_limit`；格式限 typed InvalidInput）：

| 维度 | v1.2 值 | 执行点锚点 | 测试锚点 | 校准依据 |
|------|---------|-----------|---------|---------|
| 行列维度（XLSX 写） | 2^20−1 / 2^14−1 | write.rs `check_coord` | `coordinate_bounds_enforced` | Excel 物理边界（既有） |
| **单元格数（全簿累计）** | 1,000,000 | write/write_row/merge（面积）/write_hyperlink 共享计数器——写时拒 | `cell_budget_limit_and_arithmetic` | T3 探针 100 万格实测面；基准 §3.1 |
| **表数（XLSX 写）** | **256**（b 轮校准，原设 512 被证伪） | `add_sheet_inner`（先于引擎建 tempfile） | `sheet_count_write_limit` + `streaming_sheet_count_256_fd_face` | **fd 实测**：引擎 constant_memory 每表急切 tempfile（含非流式表）；512 双簿并发 = EMFILE@1024 → 256 双簿并发余量 1.8× |
| **单格文本长度** | 32,767 chars | `write()` Text 面 / FormulaCache::Text / merge 左上格 / 超链接显示文本 | `text_and_formula_length_limits` | 引擎 MAX_STRING_LEN 同值镜像（探针 32767 过/32768 拒） |
| **超链接 URL 长度** | **2,080 chars** | `write_hyperlink` | 同上 | **引擎真值镜像**（url.rs MAX_URL_LEN——探针 2080 过/2081 拒；a 轮「同 32767」被证伪） |
| **公式文本长度** | 8,192 chars | `write()` Formula 面 | 同上 | Excel 规格限（引擎不检） |
| 输出文件大小（XLSX） | 不设独立限 | **无执行点（有意不设**——代理界 = 格数 × 文本长度） | — | 组合界登记 |
| 临时文件（XLSX 流式） | 不设独立限 | **无执行点（有意不设**——联合界 = 表数 + 格数两限） | — | 组合界登记 |
| **段落数（DOCX 写）** | 1,000,000 | docx.rs `DocxWriter` 计数 → `save_to_vec` 漏斗 + `check_document_scale` 预检单点 | `document_scale_limit_boundaries` + `docx_save_funnel_rejects_over_scale_no_artifact` | 与 MAX_CELLS 同量级；基准 §3.1（100k = 390MiB 实测，1M 沙箱 OOM——上限为防御界非可达承诺） |
| **run 数（DOCX 写）** | 4,000,000 | 同上 | 同上 | 段:run ≈ 1:4 量级（富文本典型） |
| 文本长度（DOCX 写） | 不设独立限 | **无执行点（有意不设**——Word 无 32767 类规格限；docx-rs 不检；caller 已持树文本 = 引擎边界登记） | — | v1.2 登记定谳 |
| 输出文件大小（DOCX） | 不设独立限 | **无执行点（有意不设**——代理界 = 段/run 两限） | — | 组合界登记 |

> 值分级说明：office-b 三值为**一手参照**；「无执行点（有意不设）」
> 各项均给出**不设理由**（组合界/子集界/实测无害/无规格限）——区别
> 于 v1.1 的「登记待校准」未定态；其余每项均有产品码执行点 + 测试
> 锚点（**v1.2 起限额表 = 产品码单一真源的文档镜像**，真源 =
> io.rs/write.rs/docx.rs 常量与执行点）。

### 3.1 可复现基准（合法大载荷；**不设门禁阈值**——宿主装配资源预算的一手输入）

产品码 = office-io `examples/bench_limits.rs`（release，沙箱
2 CPU/3GB RAM；耗时 = Instant，峰值 RSS = /proc/self/status
VmHWM——与 GNU /usr/bin/time -v 的 Maximum resident set size
同源口径，沙箱无该二进制登记替代）：

| 场景 | 耗时 | 峰值 RSS |
|------|------|---------|
| 读 500k 格（500×1000，1.58MB 档，全门全链） | 517 ms | 38.5 MiB |
| 读 4M 格（2000×2000——恰=格面积限） | 4410 ms | 278.9 MiB |
| 写 1M 格非流式（恰=MAX_CELLS，save_to_vec） | 1139 ms | 143.4 MiB |
| 写 1M 格流式（save_to_path） | 1221 ms | 5.1 MiB |
| DOCX 写 100k 段 | 801 ms | 390.0 MiB |
| DOCX 写 1M 段（恰=MAX_PARAGRAPHS） | — | **沙箱 OOM kill（exit 137）**；100k 线性外推 ≈3.9GB——限值为防御上界，实际包络受宿主内存约束（未实测不宣称可达） |

## 4. 输出面引用

消毒六面规格与实现 = mdcore `crates/office`（mdc-office，
office-impl T1 + xlsx-r1 阶段 B，main 880864e）：六模块 API（含
url_policy）+ 往返性质测试 + 三档对照断言 + 靶向安全测试（xlsx-r1
阶段 B 落地 44 用例）在档。宿主/生成态管线消费契约见该 crate crate
根文档（含 xlsx 路线 CSV 消毒豁免注记 + 幂等/非幂等契约分立 + URL
管线顺序）。

输入面读路径实现 = mdcore `crates/office-io`（mdc-office-io）：
门 0/1/2+3 四门（§2.1）+ calamine 四 API 面 + 读面语义层两限
（§2.2）+ 统一错误三分；写面资源/格式限 = write.rs/docx.rs 常量
与执行点（§3 写面表）。限额常量即 §3 表接线（LIMIT_* / MAX_*）。
wasm 消费面错误码 = mdcore-wasm（`container_prescan` 既有 +
`xlsx_limit`/`docx_limit` v1.2 新增——web 断言零改动兼容）。

## 5. 版本与复核纪律

- ~~本策略随 mdc-office IO 模块实装同步升 v1.0~~ **已兑现
  （2026-10-13）**；~~v1.1 门 3 实装同步~~ **已兑现（2026-10-14）**；
  **v1.2 随 r4-t2 资源上限全面实装同步升版（2026-10-17）**。
  限额表数字变更走版本化（v1.x 内校准，破坏性变更升 v2）。
- 复核触发：T3 探针完成 ✅ / 首个敌意样本回归（阶段 B 已落地，每轮
  回归触发）✅ / zip 或解析库大版本升级（**已知敏感点：zip 8.6→9
  迁移会改变谎言头读界行为**——§2.1 第 2 项与门 2 实现须同步重评）
  / **calamine 升版须复核 `GridScan` 坐标镜像语义**（单点同源纪律
  ——io.rs GridScan 文档 + `grid_scan_mirror_semantics` 测试即
  复核锚点）/ **rust_xlsxwriter 升版须复核 MAX_URL_TEXT/MAX_CELL_
  TEXT 引擎镜像值**（url.rs MAX_URL_LEN / worksheet.rs
  MAX_STRING_LEN）。
- 校准历史：v1 草案（2026-10-12 T3）→ 谎言头项校准（2026-10-13
  lie86）→ **v1.0 生效**（2026-10-13，mdc-office-io 实装）→
  **v1.1**（2026-10-14，门 3 实装）→ **v1.2**（2026-10-17，
  r4-t2：门 0 + 族分化 + 稀疏包围盒闭环[T3b] + 读面语义层两限 +
  写面资源/格式限整面 + docx 族执行点补齐 + 基准 §3.1）。
