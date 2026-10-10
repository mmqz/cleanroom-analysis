# ADR-P1：PDF 渲染与解析引擎选型定谳（PDF-a）

> 状态：**已裁决（2026-10-10，PDF-a spike 实测背书）**
> 决策人：用户裁决序（PDF 提前为第二载体）+ 本轮工程定谳
> 输入：pdf_carrier_plan.md §2 预判 + 三探针 spike 实测 + pdf-line/02
> 开源 PDF 生态缺口分析 + pdf-line/03 行业坐标

## 裁决

| 部件 | 选型 | 版本（pin） | 许可 | 角色 |
|---|---|---|---|---|
| 渲染 / 文本 / 搜索 | **pdfium**（bblanchon/pdfium-binaries 预编译轨）经 **pdfium-render** binding | chromium/8086（SHA256 588577cf…bb935）+ pdfium-render 0.9.4 | pdfium BSD-3 风格 ✅ / pdfium-render Apache-2.0/MIT ✅ | 阅读面全链 |
| 对象级写入（标注落盘） | **lopdf** | 0.45.0 | MIT ✅ | 标注增量写 |
| 取件 | fetch-pdfium.sh（版本 pin + SHA256 + non-V8 构建断言） | — | — | 供应链纪律 |

**排除记录**（竞品与落选理由，均为一手源事实）：

| 落选 | 理由 |
|---|---|
| mupdf | **AGPL ❌——ADR-C1 闭源最严姿态全面禁入**（GPL 传染，无豁免路径） |
| pdf-extract / pdf_text / 造轮子解析器 | 维护性/覆盖差（pdf-line/02 生态缺口分析）；文本抽取已有 pdfium 完整接口，无重写价值 |
| pdfium-v8-* 构建 | 安全策略 S1.1：JS 引擎不接（行业坐标：完整 JS 实现的引擎无一例外靠平台沙箱兜底，库形态不可继承） |
| qpdf（写路径候选） | 结构变换强但无 Rust binding 主轨；lopdf 原生 Rust + IncrementalDocument 0.45 API 已覆盖标注写场景（spike 实证） |

## spike 实测数据（2026-10-10，本沙箱 x64 linux，探针源码 spike/）

| 探针 | 场景 | 实测 |
|---|---|---|
| 1 打开 | 3 页 / 500 页 load + 首页文本抽取 | load 0ms（两档）；文本抽取首次 19ms（预热）后 0ms |
| 2 渲染 | 1200px 宽光栅化（Letter） | 单页 4ms；连渲 5 页 17ms（~3.4ms/页，页缓存行为正常） |
| 3 标注写 | lopdf IncrementalDocument 增量写 Highlight | 原 1026B → 1497B（+471B 追加）；**原字节前缀逐位保持**；lopdf 重载 /Annots=1 ✓；**pdfium 交叉读回 type=Highlight ✓（引擎互通坐实）** |

## 决策理由

1. **pdfium 是能力/成熟度/许可三重最优解**：Chrome 同款引擎（字体覆盖与
   渲染正确性行业背书）、BSD-3 风格许可（ADR-C1 白名单族）、文本接口
   完整（探针 1 文本抽取直接可用）。
2. **双引擎分工 = 各取所长**：pdfium 的**增量写接口弱**（FPDF 生态主打
   读侧）；lopdf 的 IncrementalDocument（0.45）原生增量语义且**签名文档
   友好**（原字节前缀保持，探针 3b 断言背书）——读写分离比单引擎全栈
   更稳。
3. **互通性从第一天成立**：探针 3d 证明 lopdf 写的标准 Annot 对象被
   pdfium 读回且类型正确——PDF-d 的 Acrobat 互通验证（plan §5）在引擎
   层已有预证。
4. **安全策略（SECURITY_POLICY.md）与选型互锁**：non-V8 构建断言（S1.1）
   直接由 args.gn 实证（`pdf_enable_v8=false` + `pdf_enable_xfa=false`）。

## 后果与义务

1. **供应链义务**：pdfium 预编译二进制三平台 pin（linux 已 pin 8086 +
   SHA256；windows/macOS 在 PDF-b 补 pin）+ licenses/ 全量入
   THIRD-PARTY-NOTICES（PDF-e，含 freetype/harfbuzz/icu 等 ~20 项传递
   声明——包内 licenses/ 目录已备）。
2. **性能义务**：PDF-c 性能门（打开 TTI / 翻页 p95 / 1000+ 页基线）——
   本轮 500 页 0ms/4ms 数据为强正向信号但非门（spike 档 ≠ 产品档）。
3. **版本升级纪律**：pdfium 版本升级 = 重跑 fetch（SHA256 复验）+ 三探针
   + NOTICES 更新，三件一体。
4. **AcroForm 验证项**（S6 登记）：non-V8 构建是否含 formfill 模块 =
   PDF-b 补探针（影响表单字段渲染，v0 边界内）。

## 与计划的差异

无差异——pdf_carrier_plan §2 预判（pdfium + lopdf 双引擎）经实测确认，
本 ADR 将预判升级为定谳并附排除记录与义务清单。
