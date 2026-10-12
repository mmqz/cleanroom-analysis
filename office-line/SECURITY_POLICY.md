# Office 线敌意输入安全策略（SECURITY_POLICY v1 草案）

> 状态：**v1 草案（限额表已按 T3 探针实测校准，2026-10-12）**（office-impl
> T2+T3 产出；循 pdf-a SECURITY_POLICY 先例）。日期：2026-10-12。输入：
> office-b §3.5（x2t 五件套一手限额参照）+ office-a §9（边界参照）+
> office-d §5（输出面）+ office-e §6/§8（Rust zip 限额一手证据 + 探针
> 实测）+ L2/03 敌意输入方法论（五层纵深/三行为学/沙箱四件套）。
> 生效面：mdc-office 后续 IO 模块 + 宿主接线层。

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

敌意**输出**面（LLM/用户可控文本落产物）：**消毒五面**（已落地
mdc-office sanitize，mdcore 2a4cb6c）——CSV 公式首字符 / XML 控制
字符保字符档 / XML 实体属性位 / HTML href / 域指令白名单。本策略
不重复其规格（见 mdcore crates/office）。

## 2. 输入防线（分层，L2/03 五层纵深裁剪）

### 2.1 容器层（zip 门，读前强制）

1. **声明求和预扫**（读任何条目前）：全条目 `uncompressed_size`
   求和 + 单条目上限对照 §3 限额表——超额即整档拒绝（typed reject，
   不进解压）。依据：office-e §6（zip 9.x 声明尺寸 pub 暴露）+
   office-b checkInputLimits 同构（docx/pptx 50MB、xlsx 300MB 按
   格式族分档）。
2. **读界**（库内建，非策略层）：zip `with_size_limit` 按声明值截
   断——实际膨胀超声明即错（反 T1）；声明值本身由 1 把门（反 T2）。
3. **条目路径校验**：entry name 拒绝绝对路径/`..` 逃逸/盘符位
   （反 T5；office-b checkPathTraversal 同构）。
4. **条目数上限**（DoS 面）：单档条目数 > 上限拒绝（§3 表）。

### 2.2 解析层（XML/模型门）

1. 巨量声明防御：**T3 探针实测校准（2026-10-12）**：calamine 不按
   dimension 声明预分配——Range 按实际单元格定尺寸（声明 A1:ZZZ9999999
   ≈1.7e12 格的档实测 0.05ms 正常提取，Range 2×2）。威胁 T3 在
   calamine 路径实测无害，降级为登记观察项（换解析件时重测）。
2. 共享字符串表/worksheet 解压内存上限：走 calamine
   `UnzipXMLSizeLimit` 落盘机制（office-e §1 一手在档）。
3. XML 解析器事件深度上限（嵌套炸弹面；quick-xml 无内建深度限，
   策略层包一层计数器——待 IO 模块立项时评估必要性）。

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
| 条目数上限 | 10,000 | 【本 v1 新设】 | 待 IO 模块实测 |
| 谎言声明头（实际膨胀 > 声明） | 拒绝 | zip 9.x 内建 | **实测拒绝**（错误串："File is larger than its declared uncompressed size"）|
| dimension 巨量声明 | 登记观察（不设前置门） | 【本 v1 新设→T3 降级】 | **实测无害**（calamine 按实际单元格定尺寸，0.05ms）|
| 共享字符串解压上限 | 256 MB | calamine 机制位（值自设） | 机制在档（UnzipXMLSizeLimit）|
| 路径逃逸条目 | 拒绝 | office-b 同构 | **实测识别**（zip enclosed_name None 判定）|

> 值分级说明：office-b 三值为**一手参照**；两项【本 v1 新设】中
> dimension 项经 T3 实测降级、条目数项待 IO 模块立项实测；谎言头/
> 路径逃逸两项由 T3 探针实证为库内建行为（策略层不重复实现，引用
> 即可）。

## 4. 输出面引用

消毒五面规格与实现 = mdcore `crates/office`（mdc-office，office-impl
T1，main 2a4cb6c）：五模块 API + 往返性质测试 + 三档对照断言在档。
宿主/生成态管线消费契约见该 crate crate 根文档（含 xlsx 路线 CSV
消毒豁免注记）。

## 5. 版本与复核纪律

- 本策略随 mdc-office IO 模块实装同步升 v1.0（草案→生效）；限额表
  数字变更走版本化（v1.x 内校准，破坏性变更升 v2）。
- 复核触发：T3 探针完成 / 首个敌意样本回归 / zip 或解析库大版本
  升级。
