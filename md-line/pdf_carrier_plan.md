# PDF 载体工程小计划（2026-10-10）

> 定位：主档 §1.1 版式族第一载体——**阅读 + 标注**，不做内容编辑（主档边界不变）。
> 前置状态：MD 第一载体已收尾（main 三分支就位，R24 编排见主档 §31）；本计划经
> 用户裁决将 PDF 从格式扩展序列第 4 位提前（主档 §25 允许调整条款）。
> 侦察资产：Acrobat 生态安全审计三仓（acrobat-pro-audit 主线 6 轮 +
> adobe-acrobat-dc-security-audit 前置 + agents-md-audit 规范）。

---

## 0. 一句话计划

新建 **mdc-pdf** 第二 Document Model（与 mdcore 块核并列，不互嵌），五轮
（PDF-a 侦察选型 / PDF-b 核骨架 / PDF-c 阅读面 / PDF-d 标注面 / PDF-e 收口），
渲染与解析选 **pdfium + lopdf**（双 BSD/MIT，ADR-C1 白名单族），安全策略直接
继承 Acrobat 审计成果（JS 不接 / OpenAction 禁用 / 沙箱边界）。

## 1. 架构关系：第二 Document Model，不是 mdcore 插件

PDF 是版式文档（页 + 绘制指令 + 对象图），与 mdcore 的块结构 Document Model
**异构**。强行塞进块核 = 重蹈"用文本编辑器架构做排版"的覆辙（Typora/typora-re
研究已证伪此路线）。

```text
┌─ 宿主壳（tao+wry+GTK3，复用）────────────────────────────┐
│   mdcore://app（MD 文档面） + mdcore://pdf（PDF 面新增）    │
├─ web 桥（CM6 只服务 MD；PDF 面用独立轻量视图层）──────────┤
├─ mdc-pdf（新 crate）──────────────────────────────────┤
│   页模型 / 渲染管线（pdfium）/ 文本层 / 标注模型（PDF Annot）│
│   Command 边界（Human+AI 共用状态理念复用）                 │
├─ mdcore（MD 块核，不动）────────────────────────────────┤
└──────────────────────────────────────────────────────────┘
```

共享面：宿主壳、Command 理念、sidecar 纪律、compliance 门禁、CONTRACT 升版纪律、
gates.md 收口纪律。
独立面：渲染、页导航、标注（复用 PDF 内建 Annot 对象，**不自研锚定层**——这与
mdcore anchor 哲学的分叉点：PDF 有标准标注模型，v0 目标就是与 Acrobat 互通）。

## 2. 技术选型（PDF-a 定谳，此处为预判）

| 部件 | 选型 | 许可 | 理由 |
|---|---|---|---|
| 渲染/文本/搜索 | **pdfium**（经 pdfium-render binding） | BSD-3 风格 ✅ | Chrome 同款引擎，成熟度/性能/字体覆盖最佳；文本接口完整 |
| 对象级写入（标注落盘） | **lopdf** | MIT ✅ | 增量更新（incremental update）保留原字节——签名文档友好 |
| 排除 | mupdf | **AGPL ❌** | ADR-C1 闭源最严姿态，全面禁入（GPL 传染） |
| 排除 | pdf-extract / 造轮子 | — | 维护性/覆盖差，无重写价值 |

风险点（PDF-a spike 验证）：pdfium **预编译二进制分发**（三平台版本 pin +
取件脚本 + BSD 声明随包入 NOTICES 流程）。

## 3. 安全策略（Acrobat 审计成果直接转化）

| 策略 | 内容 | 来源 |
|---|---|---|
| JS 引擎不接 | pdfium V8/JS 模块不启用；XFA 不启用 | acrobat-pro-audit R5/R6 |
| 危险动作默认禁 | /OpenAction /Launch /EmbeddedFiles /RichMedia 全禁（策略表 + 解析层拒绝） | R2 行为面 |
| 沙箱边界 | pdfium 调用独立线程、零网络、panic 隔离策略 | R5 能力面方法论 |
| 恶意载荷面 | 打开时扫描危险对象族并告警（只读检测，不杀进程） | R1/R3 字符串挖掘法 |

## 4. AI 接入（主档核心定位的 PDF 表达）

Command 边界 v0 面：`extract_text(range)` / `search(query)` /
`annotate.add|edit|list(kind, target, payload)` / `outline()`。
AI 生成标注（高亮 + 批注）与 Human 走**同一 Command 路径**（Source::Ai 溯源
模式从 mdcore 复用），PDF Annot 对象的 author 字段区分 Human/AI。

## 5. 五轮编排

| 轮 | 主题 | 交付 | 门禁 |
|---|---|---|---|
| PDF-a | 侦察收口 + spike + 选型 ADR-P1 | Acrobat 审计→安全策略表；三探针（打开/渲染/标注写）各 1 spike；ADR-P1（选型+排除记录）；pdfium 取件脚本 | spike 实测数据入 gates |
| PDF-b | mdc-pdf 骨架 | crate 骨架、页/大纲/标注三层模型、Command 边界定稿、宿主协议草案（mdcore://pdf）、CONTRACT v1.2 增章 | cargo 全绿 + scan.sh 0 |
| PDF-c | 阅读面 | 异步光栅化 + 页缓存（LRU）、文本层（选择/搜索/复制）、导航（大纲/缩略图/页码）；1000+ 页文档性能基线 | 性能门（打开 TTI/翻页 p95 入 gates） |
| PDF-d | 标注面 | v0 标注子集四种（Highlight/Note/FreeText/Ink）、lopdf 增量写、标注往返语义锁、Human+AI 共用标注状态；**Acrobat 互通验证**（产物被 Acrobat 打开标注可读） | 往返锁全绿 + 互通验证记录 |
| PDF-e | 阶段收口 | 门禁全复跑、SBOM/NOTICES 增量（pdfium 三平台二进制 + pdfium-render/lopdf）、gates PDF-e 段、双仓推送 | 收口全绿 |

并行性：PDF-a/b 与 R24-a/b/c 可并行（不同 crate 面，无冲突）；PDF 宿主接线
在 R24-d 壳实体合入后进行（或 PDF-e 若晚于 R24-d）。

## 6. 边界（v0 明确不做）

内容编辑（主档边界）、PDF 生成、OCR、表单填写（AcroForm v1 再议）、
批注合并/审阅工作流（v1）、Tagged PDF 语义重构（研究登记）。

## 7. 风险表

| 风险 | 等级 | 缓解 |
|---|---|---|
| pdfium 二进制分发链（版本/平台/许可声明） | 中 | 版本 pin + 取件脚本 + NOTICES 流程复用（PDF-a spike 即验） |
| 大文档性能（1000+ 页） | 中 | 页级懒光栅 + LRU；性能门入 PDF-c |
| 标注与 Acrobat 互通（字体/坐标/外观流） | 中 | v0 只用标准 Annot 子型零私有扩展；PDF-d 实测互通 |
| Windows/macOS 真机验证（本会话环境限制） | 低 | 与 R24-d 同法：核侧全自动化，实体真机项集中登记人工执行 |
| 商标/定名 | — | 沿用 TRADEMARK_RULES 定名门（R24-c 前，非 PDF 特有） |
