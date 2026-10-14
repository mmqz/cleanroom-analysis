# Office 线敌意输入安全策略（SECURITY_POLICY v1.1）

> 状态：**v1.1 生效（2026-10-14，xlsx-r2 Phase 2 门 3 实装同步升版：
> §2.1 第 5 项[XML 部件良构性门 + 默认严格/显式兼容两模式] + §2.2.3
> 深度上限闭项 + §3 两行新增）**。前身：v1.0 生效（2026-10-13，
> mdc-office-io 实装同步升版——§5 纪律兑现：草案→生效）；v1 草案
> （2026-10-12 T2+T3 产出，限额表已按探针实测校准；2026-10-13 lie86
> 谎言头项校准）——输入：office-b §3.5（x2t 五件套一手限额参照）+
> office-a §9（边界参照）+ office-d §5（输出面）+ office-e §6/§8
> （Rust zip 限额一手证据 + 探针实测）+ L2/03 敌意输入方法论
> （五层纵深/三行为学/沙箱四件套）。
> 生效面：mdc-office-io（XLSX 读路径 + 门 3 已实装，mdcore main 042a5e4）+
> mdc-office sanitize 六面 + 宿主接线层（§2.3 待接线）。

## 1. 威胁模型

敌意**输入**面（打开/解析非受信 office 文件）：

| # | 威胁 | 载体 | 参照 |
|---|------|------|------|
| T1 | zip 解压炸弹（实际膨胀 ≫ 声明） | OOXML 容器（xlsx/docx/pptx 均 zip） | office-b §3.5 |
| T2 | 谎言声明头（声明值本身巨型） | zip 条目 uncompressed_size | office-e §6 |
| T3 | 巨量单元格/超界 dimension 声明 | worksheet XML | 05 §4.1 问 6 |
| T4 | 深度爆炸公式/巨量区间引用 | 公式体（公式引擎立项后生效） | 05 §4.2 |
| T5 | 条目路径逃逸（`../` 绝对路径） | zip entry name | office-b §3.5 |
| T6 | 外部资源拉取（INCLUDEPICTURE/外链） | 文档体 | office-d §5.3 |

敌意**输出**面（LLM/用户可控文本落产物）：**消毒六面**（已落地
mdc-office sanitize，mdcore main 880864e；第六面 url_policy 于
xlsx-r1 阶段 B 补面）——CSV 公式首字符 / XML 控制字符保字符档 /
XML 实体属性位 / HTML href / 域指令白名单 / URL scheme 白名单。
本策略不重复其规格（见 mdcore crates/office）。

## 2. 输入防线（分层，L2/03 五层纵深裁剪）

### 2.1 容器层（zip 门，读前强制）

1. **声明求和预扫**（读任何条目前）：全条目 `uncompressed_size`
   求和 + 单条目上限对照 §3 限额表——超额即整档拒绝（typed reject，
   不进解压）。依据：office-e §6（zip 9.x 声明尺寸 pub 暴露）+
   office-b checkInputLimits 同构（docx/pptx 50MB、xlsx 300MB 按
   格式族分档）。（实现：xlsx-r1 io.rs `prescan`，门 1）
2. **实际尺寸诚实性验证**（**策略层实现，2026-10-13 校准**）：逐
   条目有界读取计数，实际 > 声明即 typed reject。校准依据：T3 3b
   曾按 zip **9.x** 实测记录为"库内建拒绝"——xlsx-r1 lie86 探针
   定谳 zip **8.6 读界无声明值强制**（正确 CRC 谎言头条目全量读回
   零错误），而 calamine 0.36.1 内部依赖 8.6 → 实际读路径无此
   防线，改由策略层补（io.rs `verify_actual_sizes`，门 2；成本 =
   合法档双读，优化留后续）。（反 T1；zip 9 的内建拒绝为 9.x 独有
   ——升 zip 大版本时重测此行）
3. **条目路径校验**：entry name 拒绝绝对路径/`..` 逃逸/盘符位
   （反 T5；office-b checkPathTraversal 同构；enclosed_name None
   判定）。（实现：`prescan`，门 1）
4. **条目数上限**（DoS 面）：单档条目数 > 上限拒绝（§3 表）。
   （实现：`prescan`，门 1；mal6d 实测校准）
5. **XML 部件良构性验证（门 3，v1.1 新增 2026-10-14——xlsx-r2
   Phase 2 实装）**：.xml/.rels 后缀部件在门 2 的既有解压趟内
   **融合单趟**流式结构校验（quick-xml 0.41：标签配对
   [check_end_names] + 根元素在位唯一 + EOF 未闭合元素[截断检测]
   + 元素深度上限 §3 表）；畸形部件**不再静默变空/部分数据**
   （mal7e 实证：垃圾部件降级空表零错误信号）——默认严格 typed
   reject（`ReadError::InvalidPart`）；显式兼容模式降级为结构化
   警告（`Book.warnings`，数据照常提取）。**门 1/门 2 与全部安全
   上限两模式恒等**（兼容不放松安全防线）。门 2 双读受信源跳过
   开关经裁决**不设**（无基准证据不得加跳过——2026-10-14 用户
   指令）。控制字符发现恒登记警告（XML 1.0 非法字节——值面容忍
   行为不变[mal7c 登记：再输出须过 mdc-office 消毒链]，仅提升
   可见性）。（实现：io.rs `verify_pass`/`validate_xml_structure`）

### 2.2 解析层（XML/模型门）

1. 巨量声明防御：**T3 探针实测校准（2026-10-12）**：calamine 不按
   dimension 声明预分配——Range 按实际单元格定尺寸（声明 A1:ZZZ9999999
   ≈1.7e12 格的档实测 0.05ms 正常提取，Range 2×2）。威胁 T3 在
   calamine 路径实测无害，降级为登记观察项（换解析件时重测）。
2. 共享字符串表/worksheet 解压内存上限：走 calamine
   `UnzipXMLSizeLimit` 落盘机制（office-e §1 一手在档）。
3. XML 解析器事件深度上限（嵌套炸弹面；quick-xml 无内建深度限，
   策略层包一层计数器——~~待 IO 模块立项时评估必要性~~ **已闭项**
   （2026-10-14 xlsx-r2 Phase 2 门 3 实装：LIMIT_XML_DEPTH=1000，
   边界对测钉定；截断检测的深度计数器顺带承载，零额外成本））。

### 2.3 资源层（进程面，宿主接线时生效）

参照 office-b x2t 五件套：内存上限（RLIMIT 级，宿主装配）/ 打开
超时 / 下载配额（100MB·2m·3 次，若 IO 模块带远程取件）/ 私网守卫
（外链拉取禁私网目标）。本层属宿主接线轮，登记不在 v1 实装。

### 2.4 语义层（域/公式门，随能力立项生效）

外链域（INCLUDETEXT/INCLUDEPICTURE/LINK）输入侧同样拒绝（与输出
侧白名单对称——office-d §5.3）；公式深度/区间引用限额（T4 威胁，
公式引擎立项时定）。

## 3. 输入限额表（T3 探针已校准，2026-10-12）

| 维度 | v1 值 | 参照锚点 | T3 校准结果 |
|------|------|----------|-------------|
| zip 声明总量（xlsx 族） | 300 MB | office-b default.json:604-629 | 机制实证（预扫 0.01ms 零数据读取）|
| zip 声明总量（docx/pptx 族） | 50 MB | 同上 | 同机制 |
| 单条目声明尺寸 | = 同族总量上限 | 本策略对称设计 | 同机制 |
| 条目数上限 | 10,000 | 【本 v1 新设】 | **实测校准**（xlsx-r1 mal6d：10,001 件门 1 拒绝）|
| XML 部件元素嵌套深度 | 1,000 | 【v1.1 新设 2026-10-14】 | **边界对测**（xlsx-r2 Phase 2 门 3：999+root 过 / 1000+root 拒；真实 xlsx 部件深度个位到十位级）|
| 畸形 XML 部件 | 严格拒绝 / 兼容警告 | 【v1.1 新设 2026-10-14】 | **实测钉定**（xlsx-r2 Phase 2：mal7e 垃圾/截断/多根/空部件/超深 全拒 + 兼容模式全语料复扫归类）|
| 谎言声明头（实际膨胀 > 声明） | 拒绝 | **策略层**（io.rs 门 2；2026-10-13 校准） | **实测拒绝**（zip 8.6 无内建[lie86 探针]，策略层拦；zip 9 内建为 9.x 独有[T3 3b]）|
| dimension 巨量声明 | 登记观察（不设前置门） | 【本 v1 新设→T3 降级】 | **实测无害**（calamine 按实际单元格定尺寸，0.05ms）|
| 共享字符串解压上限 | 256 MB | calamine 机制位（值自设） | 机制在档（UnzipXMLSizeLimit）|
| 路径逃逸条目 | 拒绝 | office-b 同构 | **实测识别**（zip enclosed_name None 判定）|

> 值分级说明：office-b 三值为**一手参照**；两项【本 v1 新设】中
> dimension 项经 T3 实测降级、条目数项经 xlsx-r1 mal6d 实测校准；
> 谎言头项由 T3 探针实证为 zip 9.x 内建行为后，**xlsx-r1 lie86
> 探针校准为 8.6 无内建、策略层实现**（双版本行为差异在档）；
> 路径逃逸项由 T3 探针实证（zip enclosed_name None 判定，
> xlsx-r1 门 1 引用）。

## 4. 输出面引用

消毒六面规格与实现 = mdcore `crates/office`（mdc-office，office-impl
T1 + xlsx-r1 阶段 B，main 880864e）：六模块 API（含 url_policy）+
往返性质测试 + 三档对照断言 + 靶向安全测试（xlsx-r1 阶段 B 落地
44 用例）在档。宿主/生成态管线消费契约见该 crate crate 根文档
（含 xlsx 路线 CSV 消毒豁免注记 + 幂等/非幂等契约分立 + URL 管线
顺序）。

输入面读路径实现 = mdcore `crates/office-io`（mdc-office-io，xlsx-r1
阶段 B，main 880864e）：双门（§2.1）+ calamine 四 API 面 + 统一错误
三分；限额常量即 §3 表接线（LIMIT_TOTAL_DECLARED / LIMIT_SINGLE_
ENTRY / LIMIT_ENTRY_COUNT）。

## 5. 版本与复核纪律

- ~~本策略随 mdc-office IO 模块实装同步升 v1.0（草案→生效）~~
  **已兑现（2026-10-13）**：mdc-office-io 实装（mdcore main 880864e）
  → 本文件升 v1.0 生效。限额表数字变更走版本化（v1.x 内校准，
  破坏性变更升 v2）。
- 复核触发：T3 探针完成 ✅ / 首个敌意样本回归（阶段 B 已落地：17
  件语料 harness 17/17 + 靶向安全 44 用例全绿——后续每轮回归仍触发
  复核）/ zip 或解析库大版本升级（**已知敏感点：zip 8.6→9 迁移
  会改变谎言头读界行为**——§2.1 第 2 项与门 2 实现须同步重评）。
- 校准历史：v1 草案（2026-10-12 T3）→ 谎言头项校准（2026-10-13
  xlsx-r1 lie86 探针：8.6 无内建强制，改策略层实现）→ **v1.0 生效**
  （2026-10-13 xlsx-r1 阶段 B：mdc-office-io 实装同步，门 1+门 2
  产品化，限额常量接线，语料回归全绿）。
