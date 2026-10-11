# office-d：写侧管线与读侧对照 pass（重做轮）

> 线 A 办公 P2 批次收官项。方法：净室 pass——五仓浅克隆（--depth 1）
> 只读勘察，证据锚点 `路径:行号`，【推断】显式标注，一手源优先。
> 本轮为工作区重置后全新研究，一切以一手源重新验证落锚（不引用丢失产物内容）。
>
> 对象：写侧四件套（工作台「生成态」）+ 读侧对照锚点（docx-preview、SheetJS HTML 面）。
> 姊妹篇：office-a（Univer）、office-b（ONLYOFFICE）、office-c（HyperFormula）。
> 引用纪律：office-b 仅复引其已复审事实（§2 渲染管线、§7 Builder），克制引用。

## 0. 对象与版本锚定（含许可逐件核验）

五对象许可与活跃度全部一手核验（仓内 LICENSE 文件 + git log + npm registry dist-tags，核验日=克隆日）：

| 对象 | GitHub 仓 | 版本（仓 HEAD = npm latest） | 许可（LICENSE 一手） | 活跃度 |
|---|---|---|---|---|
| docx | dolanmiu/docx | 9.10.0（HEAD 2026-10-10 "Version bump to 9.10.0"） | **MIT**（LICENSE:1-3，Copyright 2016 Dolan） | 高频发布 |
| exceljs | exceljs/exceljs | 4.4.0（npm latest 2023-10-19；仓 HEAD 2024-01-12 fix table column style，package.json 仍 4.4.0） | **MIT**（Copyright 2014-2019 Guyon Roche） | 低维护态 |
| PptxGenJS | gitbrent/PptxGenJS | 4.0.1（HEAD 2025-06-25 "version-401"） | **MIT**（Copyright 2015-2022 Brent Ely） | 稳定维护 |
| SheetJS CE | SheetJS/sheetjs | npm `xlsx` 冻结于 0.18.5（2022-03-24）；GitHub master 冻结于 515d1c6 "mini refresh"；仓内 package.json 已是 0.18.12 开发版 | **Apache-2.0**（LICENSE:1-4） | GitHub 侧已停更（见下） |
| docx-preview | **VolodymyrBaydalka/docxjs**（npm 包名 docx-preview，registry repository 字段指向此仓） | 0.4.x（HEAD 2026-09-21 "updated version"） | **Apache-2.0**（LICENSE:1-4） | 活跃 |

**SheetJS 托管状态专项核验**（任务指定自查项）：
- GitHub 默认分支 `github` 已成「壳仓」：仅剩 LICENSE/README/demos/package.json/test_files，最后提交 2024-01-31 "exit stage left"（`git log` 一手核验）——源码实体不在该分支。
- README:4-11 顶部声明：新源码仓为 git.sheetjs.com/sheetjs/sheetjs，"SheetJS CE remains truly open source under the Apache 2.0 License"；脚本与模块经 cdn.sheetjs.com 分发（README:15）；GitHub `master` 分支包含至 515d1c6 的全部提交（README:17-19）。`git ls-remote` 核验：仅存 gh-pages/github/master 三分支，master HEAD=515d1c6 与自述一致。
- npm `xlsx` 包 dist-tags latest=0.18.5（2022-03-24），**新版只走 cdn.sheetjs.com**——供应链面：npm 拉到的永远是三年旧版（见 §8-4）。
- 本报告 sheetjs 证据全部取自 master@515d1c6 的 `bits/` 源码（make 拼装前的真实源层）。

**docx-preview 许可核验纠正**（任务指定自查项）：npm registry latest 的 license 字段=Apache-2.0，仓 LICENSE 头=Apache License 2.0——**「历史印象 MIT」不成立**，以一手为准：Apache-2.0。

净室纪律：五仓全部只读克隆于 /home/z/my-project/analyses-workdir/office-pipeline/，未复制任何源码。

## 1. 流式写与内存边界

### 1.1 docx：全内存拼装，`toStream` 是伪流式

- 编译面：`Compiler.compile()`（src/export/packer/next-compiler.ts:136-181）一次性把全部 XML 部件 `zip.file(path, encodeUtf8(data))` 塞进 **JSZip**（依赖清单 package.json dependencies：jszip ^3.10.1、xml ^1.0.1）；document.xml 由 `xml()`（xml 包）单次序列化为整段字符串（next-compiler.ts:184-197）；媒体与字体二进制直接入 zip（:163-178，字体经 `obfuscate` 转 odttf）。
- 输出面：`Packer.pack` 调 `zip.generateAsync({type, compression:"DEFLATE"})`（src/export/packer/packer.ts:75-80），**未启用 JSZip 的 streamFiles**；`Packer.toStream`（packer.ts:171-189）先在内存生成完整 nodebuffer，再**单次 `emit("data", z)` + `emit("end")`**——名为 stream，实为 buffer 包装。
- 内存形态：模型树 + 全部 XML 字符串 + JSZip 内部副本 + 输出 buffer，**同一时刻约四份同量级副本并存**。【推断】无背压、无分片；大文档内存峰值≈O(文档体积)×4，媒体另计。
- 伪流式与真流式的语义差异：调用方拿到首字节的时间=全文档生成完成时间。

### 1.2 exceljs：四库中唯一真流式（Archiver 管道）

- `Excel.stream.xlsx.WorkbookWriter`（lib/stream/xlsx/workbook-writer.js:22-62）：基于 **Archiver**（`const Archiver = require('archiver')`，:2；`this.zip = Archiver('zip')` :50）+ `this.zip.pipe(this.stream)`（:58）——**边生成边压缩边落盘/落流**。
- 部件级流：`_openStream(path)` 为每个 zip 条目开 `StreamBuf({bufSize:65536, batch:true})` 并 `zip.append(stream, {name})`（workbook-writer.js:68-75）；工作表流获取后 `pause()` 挂背压（lib/stream/xlsx/worksheet-writer.js:198-207）。
- 行级增量：行 XML 由 `_writeRow` 即时序列化进流（worksheet-writer.js:219-225）；**行不自动提交**——官方理由是跨行合并单元格（`mergeCells('A1:B2')` 需要缓冲两行，README:2421 示例）；`row.commit()` 后「row object and contents can be freed」（README:2407 原文）。
- 提交序列：`workbook.commit()` 先补齐媒体与未提交工作表，再补 contentTypes/app/core/sharedStrings/styles/rels，最后 finalize（workbook-writer.js:97-112）。
- 内存权衡项：`useSharedStrings` 默认 false（"creates a smaller xlsx file but may use more memory"，workbook-writer.js:32-34）；`useStyles` 默认 false 时样式走 `StylesXform.Mock`（:37）——流式模式默认**丢样式**，要样式必须显式开且付出性能开销（README 选项表）。
- 限制面：`destroy()` 直接抛错（worksheet-writer.js:209-213，注释明言流式写不可中途放弃）；普通 `Workbook.xlsx.writeBuffer()` 仍全内存。

### 1.3 pptxgenjs：全内存，`stream()` 输出实为整包 Buffer

- `exportPresentation`（src/pptxgen.ts:479-564）：STEP1 媒体先 base64 编码（:484-491）→ STEP2 每张 slide/layout/notes 的 XML 逐个 `zip.file()`（:500-548，全部为内存字符串）→ `zip.generateAsync()`（:551-561）。JSZip 是其唯一运行时依赖（README:82 "Includes the sole dependency (JSZip)"）。
- `stream(props)` 分支：`generateAsync({type:'nodebuffer'})`（pptxgen.ts:552-554）——**返回完整 Buffer**；README:44 宣称 "Export as **base64**, **Blob**, **Buffer**, or **Node stream**"，实现与文案不符（与 docx toStream 同款伪流式）。
- 压缩默认关闭：browser 分支与 STREAM 分支均为 `props.compression ? 'DEFLATE' : 'STORE'`（:554,:560）——**默认产物不压缩**，媒体型 deck 体积显著更大。
- 图表附加层：每个图表数据用独立 JSZip 生成内嵌 xlsx（src/gen-charts.ts:24-36 `createExcelWorksheet`）——图表越多内存峰值越高。

### 1.4 sheetjs：二进制写全内存；文本输出有真流式

- `XLSX.write`（xlsx 路线）：`write_zip_type` → `write_zip_denouement` → `z.generate(oopts)`（bits/88_write.js:22-57，自研 zip writer，非 JSZip）——全内存一次性产出。
- CSV 写：`write_string_type(sheet_to_csv(...), o, "\ufeff")`（bits/88_write.js:141）——整表字符串拼接 + UTF-8 BOM。
- **真流式仅文本三件套**：`XLSX.stream.to_csv / to_html / to_json`（bits/97_node.js:4-33, 35-59, 61-121）——`Readable._read` 按需逐行 push，Node 侧可管道消费。**xlsx/xlsb 等二进制格式无流式写**。
- 显式边界：单元格字符串 >32767 字符直接抛错（bits/67_wsxml.js:290，Excel 上限内置校验）。

### 1.5 内存边界对照表

| 维度 | docx 9.10 | exceljs 4.4 | pptxgenjs 4.0.1 | sheetjs CE |
|---|---|---|---|---|
| 生成模式 | 全内存拼装 | **真流式**（Archiver 管道）+ 全内存两条路线 | 全内存拼装 | 二进制全内存；csv/html/json 可流式 |
| 「stream」API 语义 | 伪（buffer 单次 emit） | 真（逐行/逐部件） | 伪（nodebuffer 返回值） | 真（仅文本格式） |
| zip 引擎 | JSZip（未开 streamFiles） | Archiver | JSZip | 自研（bits/21_ziputils） |
| 内存释放点 | 无 | `row.commit()`/`worksheet.commit()` | 无 | 无（文本流按行产出） |
| 内置体积/条目防线 | 无 | 无（仅语法校验） | 无 | 字符串 32767 上限（67_wsxml.js:290） |
| 默认压缩 | DEFLATE | Archiver 默认 DEFLATE | **STORE（不压缩）** | compression 可选（88_write.js:35） |

## 2. 模板/占位机制

### 2.1 docx：内建 patchDocument（模板补丁体系，9.x 特性面）

- 入口 `patchDocument({data, patches, outputType,...})`：接受**现成 .docx 二进制**（Buffer/Uint8Array/ArrayBuffer/Blob/Stream/JSZip，src/patcher/from-docx.ts:45-47 InputDataType），JSZip 解包 → xml-js 转 JSON 遍历（from-docx.ts:10,53）→ 命中 `{{placeholder}}` 注入补丁 → js2xml 重序列化。
- 四种 PatchType（docs/usage/templates.md:79-203）：
  - `PARAGRAPH`：行内替换（TextRun/ImageRun/超链接，templates.md:83-103）；
  - `DOCUMENT`：块级替换（段落/表格，:105-138）；
  - `TABLE_ROWS`：**模板表格行循环填充**（保模板表格样式，字段形如 `{{items.name}}`，支持嵌套表格与多行分组，:140-203）；
  - 图表补丁：`DrawingPatch`/`TemplateChart`（from-docx.ts:52-56；src/charts/template/ 存在 patch-chart.ts，+同名 spec patch-chart.spec.ts）——【推断】可对模板内既有图表做数据替换。
- 附加能力：页眉页脚占位同补（templates.md:390-414）；`keepOriginalStyles` 保留占位符原格式（:361-388）；脚注尾注注入（:300-338）；**`estimatePageNumbers`（docx/layout）在补丁后做页码估算并把 TOC/页码域写 clean**（:340-359）——即 docx 自带一个轻量布局引擎用于页码域收尾。
- 转义层注意：patcher 的 XML 重序列化是「xml-js + 手工补丁」拼凑（`withAmpersandsEscaped` 预转义 & 再 `attributeValueFn` 补 < > '，from-docx.ts:615-647）——注释自述原因是 xml-js 的转义语义坑；**与主路径（xml 包 escapeForXML）不是同一套代码**。
- 样式逃生舱：`externalStyles` 允许整段外部 styles.xml 原样嵌入（docs/usage/styling-with-xml.md）。

### 2.2 exceljs：无模板机制

- 从零构建 API（addWorksheet/addRow/addTable/addImage）；无「加载模板 xlsx 再改」的官方路径（流式 Reader 是读数据而非改写保真）。【推断】xlsx 模板若要复用只能整文件读入再写出，theme 保真有专门处理（README 0.4.2 变更记录：解析时保留 theme 文件，仅 Document 版 Workbook，非流式 Reader/Writer）。

### 2.3 pptxgenjs：母版/版式占位（不是模板文件机制）

- `defineSlideMaster` + placeholder 体系：slide 挂 layout，导出前 `addPlaceholdersToSlideLayouts` 自动补空占位对象（pptxgen.ts:495-498）；master/layout 的占位符（title/body/image 等）由子 slide 继承样式与位置。
- **不能加载/修改既有 .pptx 文件**：全仓无 parse/load 入口（grep 核验，README 亦无此宣称）——纯生成器。
- HTML `<table>` → 多页 slides 的便捷层（`tableToSlides`，src/gen-tables.ts:181,523）——「HTML 当模板」的替代形态。

### 2.4 sheetjs：CE 无模板

- README:7-10 自述 Pro 才有 "Edit complex templates with ease"。CE 的模板近似法：读入 → 改单元格 → 写出（样式会丢，见 §4）。

### 2.5 模板能力对照表

| 能力 | docx | exceljs | pptxgenjs | sheetjs CE |
|---|---|---|---|---|
| 加载现成文件改写 | ✓ patchDocument | △ 读写往返（theme 保留，样式面见 §4） | ✗ | △ 读写往返 |
| 占位符语法 | ✓ `{{key}}`（分隔符可配） | ✗ | ✗ | ✗ |
| 表格行循环填充 | ✓ TABLE_ROWS | ✗ | ✗ | ✗ |
| 母版/样式继承 | ✓ externalStyles + theme | △ 主题文件保留 | ✓ defineSlideMaster | ✗ |
| 占位后重排版 | △ estimatePageNumbers（页码估算） | n/a | n/a | n/a |

## 3. 格式特性覆盖矩阵（四库 × OOXML 特性面）

锚点均为仓内一手核验（源码存在性 + API 面）：

| 特性面 | docx 9.10 | exceljs 4.4 | pptxgenjs 4.0.1 | sheetjs CE |
|---|---|---|---|---|
| 文本/段落样式 | ✓（styles/numbering/themes 子系统） | ✓（font/fill/border/alignment/numFmt） | ✓（run 属性 + master/theme） | ✗ 写侧（见下） |
| 表格 | ✓ Table/TableRow/Cell | ✓ addTable（filterButton/汇总行，README:1237-1238；autofilter 写 worksheet-writer.js:234） | ✓ addTable + tableToSlides | ✗（xlsx 无 tableParts 写） |
| 图片 | ✓ jpg/png/gif/bmp/svg+fallback（src/file/media/data.ts:58,66） | ✓ addImage（lib/doc/image.js） | ✓ image/SVG/GIF/音视频/YouTube（README:33） | ✗（Pro） |
| 图表 | ✓ 原生 Word charts（`docx/charts` 子包：bar/line/pie/doughnut/radar/scatter/bubble/combo/stock/mermaid 等，docs/usage/chart-*.md 19 篇） | **✗**（全仓无 chart 写 API；README 唯一 chart 记录是「读文件含 chartsheet 不崩」，:2959） | ✓ 10 种基型（CHART_NAME 枚举 src/core-enums.ts:42）+ combo；图表数据内嵌 xlsx | ✗（Pro） |
| 公式 | ✓ OMML Math + **LaTeX→OMML 解析器**（src/math/latex-to-math.ts） | ✓ cell.formula + 共享公式 + 数组公式（README Formula Value/Shared Formula 节） | ✗（无 OMML/LaTeX 面，grep oMath/latex 零命中） | ✓ cell.f → `<f>`（含数组公式 ff，bits/67_wsxml.js:296-298）；**无求值引擎**（office-c HyperFormula 补位） |
| 域（field） | ✓ SimpleField 任意指令 + PageNumber/PageRef/TOC/MailMerge（§5.3） | ✗（xlsx 无域概念） | △ 仅内建 slidenum 域（gen-xml.ts:1600 `<a:fld type="slidenum">`） | ✗ |
| 页眉/页脚/打印 | ✓ Header/Footer + 分节 | ✓ headerFooter/pageSetup（worksheet-writer.js:123-168 默认值面） | ✓ notes/master | ✗ |
| 批注/修订 | ✓ comments + change tracking（docs/usage/comments.md、change-tracking.md） | ✓ 批注（sheet-comments-writer.js）；修订 ✗ | ✗ | ✗ |
| 脚注/尾注 | ✓ | n/a | n/a | n/a |
| 条件格式 | n/a | ✓ conditionalFormatting | n/a | ✗ |
| 数据验证 | n/a | ✓ dataValidations | n/a | ✗ |
| 透视表 | n/a | △ 有限支持（PR #2551 "Add pivot table **with limitations**"，README:21；lib/doc/pivot-table.js） | n/a | ✗（Pro） |
| 超链接 | ✓ | ✓ | ✓ | ✓（bits/67_wsxml.js:604-618） |
| 合并单元格 | ✓ | ✓ | ✓（cell colspan/rowspan） | ✓（!merges 写，67_wsxml.js:597） |
| 列宽/行高 | △ 表格列宽 | ✓ | ✓ | ✓（!cols/!rows，67_wsxml.js:574,498） |
| defined names | n/a | ✓ | n/a | ✓（bits/72_wbxml.js:222-229） |
| 嵌入字体 | ✓ odttf obfuscate（next-compiler.ts:176-178） | ✗ | ✗ | ✗ |
| 形状/画布 | ✓ shapes/watermarks 子包 | ✗ | ✓ shapes（含 connector） | ✗ |
| 输出格式 | .docx | .xlsx/.csv | .pptx | xls/xlsx/xlsb/ods/csv/html/txt 等（多格式读写） |

**sheetjs CE 写侧样式的一手裁决**：`get_cell_style`（bits/66_wscommon.js:60-82）构建 cellXfs 时**只填 numFmtId，fontId/fillId/borderId 一律 0**（:73-80）——即 CE 写 xlsx 仅保数字格式，字体/填充/边框写侧不支持；读侧 cellStyles 可部分解析（bits/47_styxml.js:304+）。与 README:8 "let out your inner Picasso with styling"（Pro 广告语）互证。

## 4. OOXML 规格子集取舍（各库明确边界，一手核验）

- **docx**：无公开「不支持清单」，特性面=实现覆盖（docs/usage 63 篇即能力边界）。显式不支持项可核验的有：无任意 XML 注入 API（ImportedXmlComponent 类已不在 src 中，grep 零命中）；样式仅 externalStyles 整段替换（§2.1）。【推断】SmartArt/公式旧格式（OMML 之外的 eq 域）等未实现面即不支持面。
- **exceljs**：仅支持 **xlsx**（File I/O 文档只列 XLSX 与 CSV；无 xlsb/xls/ods 写）；图表写全缺（§3）；透视表有限（PR 标题自带 limitations）；流式 Writer 默认丢样式（useStyles:false，§1.2）。
- **pptxgenjs**：仅 .pptx 生成（无读取、无 ppt/odp）；无公式对象；SmartArt 无；兼容性声明为「PowerPoint/Keynote/LibreOffice」（README:27）——即产出需在主流三引擎下可打开。
- **sheetjs CE**：写侧多格式但**样式/图表/图片/透视全缺**（README:7-10 Pro 边界 + §3 一手裁决）；公式只写不算（无求值引擎——office-c 已裁决 HyperFormula 为补位件）。

## 5. 敌意输出面（本 pass 核心安全面）

> 场景定义：LLM/用户可控文本经四库落为产物文件，产物再被 Excel/Word/浏览器打开——
> 消毒责任在哪一层。本节逐一核验四库有没有做。

### 5.1 CSV/公式首字符（= + - @）消毒——四库逐一裁决

**docx：不产 CSV——面不适用（N/A）**。输出仅 .docx（XML 包），无 CSV 出口；但其文本进 XML 的转义面见 §5.2。

**exceljs：无消毒**。CSV 写出（lib/csv/csv.js:109-171）默认 map 对普通值直接 `return value`（:151）；公式单元格写**计算结果** `value.result || ''`（:133-135）——注意这仅指 exceljs 自己的 formula 对象，**用户 addRow 塞入的普通字符串 `'=HYPERLINK(...)'` 原样进入 CSV**。语法层 quoting 由 fast-csv 完成（csv.js:2,:118）——quoting 只防 CSV 语法歧义（分隔符/引号/换行），**不防公式注入**。

**pptxgenjs：不产 CSV——N/A**（文本走 XML 转义面，§5.2）。

**sheetjs：无消毒，且公式以 `=` 前缀主动输出**。`make_csv_row`（bits/90_utils.js:102-123）：
- 普通单元格：`format_cell` 原样取格式化文本（:111），仅当含分隔符/引号/换行时加 CSV 引号（:112，语法层）；唯一字面量特例 `if(txt == "ID") txt = '"ID"'`（:113，Lotus 兼容遗留）。
- 公式单元格：**`txt = '=' + val.f`（:114-116）——以 = 开头重建公式文本**，含逗号时加引号。即：sheet_to_csv 把公式当作「应被目标应用重算的内容」输出——**公式注入是特性而非被防的面**（一手代码裁决，无歧义）。

**裁决（dsh 管线层责任的依据）**：
1. **四库零家实现 = + - @ 首字符消毒**（能产 CSV 的两家——exceljs、sheetjs——均无；sheetjs 更是主动产出公式首字符）。
2. 因此 dsh 生成态必须在**出口处**统一消毒：对 CSV 出口做首字符改写（业界惯例：前置 `'` 或空格，或按目标应用策略拒写）；对 xlsx 路线**不必**消毒首字符（Excel 对 .xlsx 内 t="s" 字符串不重解释为公式——公式注入只在「CSV/TSV 粘贴与打开」时发生），但对 sheetjs 写出的 csv/html 出口必须消毒。
3. sheetjs HTML 出口的附加敌意面：超链接 `<a href="' + cell.l.Target + '">` **未经 HTML 转义**（bits/79_html.js:85）——Target 含引号/事件属性可直接注入 HTML 属性。dsh 若用 sheet_to_html 面向浏览器，需自建转义层。

### 5.2 XML 转义层达标面（逐库一手核验）

| 库 | 转义函数 | 锚点 | 5 实体（& < > " '） | XML 1.0 非法控制字符（0x00-0x08/0x0B/0x0C/0x0E-0x1F） | 备注 |
|---|---|---|---|---|---|
| docx | xml 包 `escapeForXML`（依赖 xml@^1.0.1） | xml-1.0.1 包 lib/escapeForXML.js:1-14（npm 拉取核验）；调用面 xml 包 lib/xml.js:197,209,278 | ✓ | **✗ 原样透传** | 正则 `/([&"<>'])/g` 五实体映射，其余不处理 |
| docx patcher 路径 | xml-js `js2xml` + 手工预转义 | src/patcher/from-docx.ts:615-647 | △（& 预转义 + 属性 < > '） | **✗** | 与主路径不同一套代码（§2.1） |
| exceljs | `utils.xmlEncode` | lib/utils/utils.js:75-117（regex :36-37；剥除分支 :101-108） | ✓（switch 五码位） | **△ 剥除档**：regex 命中 \x00-\x08/\x0B-\x0C/\x0E-\x1F/\x7F 即剥除（\t\n\r 不在集内，放行） | 产出**合法 XML 但静默丢字符**；文本/属性全经 xml-stream.js:12,:90 汇流（单元格文本 text-xform.js:15 writeText）；另剥 XML 1.0 本为合法的 0x7F；沿革：PR #259 引入剥除（README:2935）、PR #286 放行 \n\r、#275 补超链接转义（同 :2939） |
| pptxgenjs | `encodeXmlEntities` | src/gen-utils.ts:63-67 | ✓（链式 replace 五实体） | **✗ 原样透传** | 调用覆盖：文本 run（gen-xml.ts:1065）/图表标签（gen-charts.ts:167-226,841）/docProps（gen-xml.ts:1517-1520）均过；**例外**：`name="${slideItemObj.options.objectName}"` 等属性位未过（gen-xml.ts:559）——程序员可控选项位存在属性注入缺口 |
| sheetjs | `escapexml` | bits/22_xmlutils.js:63-67 | ✓ | **✓ 唯一转义保留（保字符档）**：`charegex=/[\u0000-\u0008\u000b-\u001f\uFFFE-\uFFFF]/` → `_xHHHH_`（OOXML 规定的字符转义约定） | 写侧 `<t>/<v>/<f>` 全过（bits/42_sstxml.js:234；67_wsxml.js:279,300）；HTML 面 `escapehtml` 控制字符→`&#xHHHH;`（:70-74） |

**裁决**：
1. **控制字符面三档实况**（一手代码级，非同质面）：**损坏档**（docx/pptxgenjs）——非法控制字符原样透传，文本含 0x0B 等字符时产出非法 XML，Word/Excel 直接拒开，这是「文档损坏」故障面而非注入面，对管线可靠性致命；**剥除档**（exceljs）——xmlEncode 对非法集+0x7F 静默剥除（证据链见上表行），产出合法 XML 但**静默丢字符**，保真故障面，与损坏档故障形态不同但同样不可接受；**保字符档**（sheetjs）——以 `_xHHHH_` 约定转义，读写闭环，四库唯一达标。
2. dsh 生成态走 docx/pptxgenjs 路线**必须在喂入前过滤/转义控制字符**（防拒开级损坏）；走 exceljs 路线同样必须前置过滤——理由是**防静默丢字符**（库剥除而非报错，产物合法但内容失真）；一处统一做，勿信库；xlsx 路线若走 sheetjs 可白嫖其转义层。
3. 【推断】sheetjs 的 `_xHHHH_` 机制存在字面量歧义面（正文里恰好形如 `_0041_` 的文本会被反向解码），业务无关紧要，登记不展开。

### 5.3 docx 域指令白名单有无

- **SimpleField：无白名单，任意指令直通**。`new SimpleField(instruction)` 构造函数把 instruction 字符串原样写入 `w:instr` 属性（src/file/paragraph/run/simple-field.ts:66-74），**零校验零过滤**——INCLUDETEXT（外链文件注入读取）、DOCVARIABLE、DDE、SEQ、AUTOFIELD 等危险/非预期域均可由 LLM 输出注入，Word 打开后按域语义执行。
- 内建域面为**结构化构造、面受限**：TOC `FieldInstruction` 按 ITableOfContentsOptions 开关拼接（src/file/table-of-contents/field-instruction.ts:53-104，参数只能进 TOC 开关位）；PageNumber/PageReference（src/file/paragraph/run/page-number.ts、src/file/paragraph/links/pageref-field-instruction.ts）为固定模板；`SimpleMailMergeField` 仅 MERGEFIELD 便捷封装（simple-field.ts:91-95）。
- **裁决**：docx 库定位是低层构建器，域安全完全外移给调用方。dsh 若暴露「生成 docx」给 LLM，**必须自建域指令白名单**（建议白名单：DATE/PAGE/NUMPAGES/TOC/PAGEREF/MERGEFIELD/SEQ），黑名单首列 INCLUDETEXT/INCLUDEPICTURE（外部资源面）/DDE（执行面）。

### 5.4 敌意输出面小结

| 面 | docx | exceljs | pptxgenjs | sheetjs | dsh 责任 |
|---|---|---|---|---|---|
| CSV 公式首字符消毒 | N/A | **无** | N/A | **无且主动产 =** | 出口统一消毒 |
| HTML 输出转义 | n/a | n/a | n/a | 链接 href 未转义 | HTML 出口转义 |
| XML 5 实体 | ✓ | ✓ | ✓（个别属性位缺口） | ✓ | pptx 属性位兜底 |
| XML 非法控制字符 | **✗ 损坏档**（拒开级） | **△ 剥除档**（合法 XML、静默丢字符） | **✗ 损坏档**（拒开级） | ✓ 保字符档（_xHHHH_） | 前置字符过滤 |
| 域指令白名单 | **无** | n/a | 内建仅 slidenum | n/a | 域白名单 |

## 6. 读侧对照（对照锚点，非深读）

### 6.1 docx-preview（DOM 渲染）保真上限与性能

**机制**：JSZip 解包 → 按关系图加载 parts（document/styles/numbering/theme/footnotes/headers/footers/settings/comments，src/word-document.ts:53-68）→ document-parser XML→DOM 模型 → HtmlRenderer 生成 HTML，`renderAsync` **一次性 append 全部节点**（src/docx-preview.ts:63-77）。

**分页保真上限（官方自认+代码核验）**：
- 仅三种分页源：手动分页符 `<w:br w:type="page"/>`、编辑器残留 `lastRenderedPageBreak`（**默认忽略**，`ignoreLastRenderedPageBreak: true`，docx-preview.ts:40 + README:107）、分节属性变化（isPageBreakSection 仅比较页面尺寸/方向，src/html-renderer.ts:413-420）。
- README:101 原文：**"Realtime page breaking is not implemented because it requires re-calculation of sizes on each insertion and that could affect performance a lot"**——内容驱动的自动分页明确不做。
- 页容器 `section.docx { overflow: hidden }`（html-renderer.ts:527）——**超出页高的内容被直接裁剪**，即「页」只是样式化容器而非真实排版结果。
- TOC 域不支持（README:92 "fields is not supported yet"）——目录渲染空白。

**浮动对象保真上限**（src/document-parser.ts:842-944 parseDrawingWrapper）：
- 支持映射仅三种：wrapTopAndBottom→display:block、wrapNone→relative 定位、anchor 左右对齐→CSS float（:907-928）；**wrapSquare/wrapTight/wrapThrough 无 case 静默丢弃**；distL/T/R/B 环绕距离注释为 TODO（:846-850）；`behindDoc` 读取后未使用（:854）。
- graphicData 仅处理 `pic`（:933-944）——**图表/形状/SmartArt 一律不渲染**。

**字体度量保真上限**：
- 嵌入字体可还原：odttf 反混淆（word-document.ts:212-225 deobfuscate）→ `@font-face` + object URL（html-renderer.ts:169-198）。
- 排版交给浏览器 CSS（`white-space: pre-wrap; overflow-wrap: break-word`，html-renderer.ts:533）——**无 Word 级排版引擎**；tab stops 对齐仅 experimental 且是渲染完成后延迟 500ms 用像素测量回修（html-renderer.ts:1436-1447）。

**大文档性能瓶颈**：全量解析 + 全量 DOM 化（无虚拟化/懒渲染，docx-preview.ts:63-77）；图片/字体异步任务 `Promise.allSettled` 收尾（html-renderer.ts:133）。【推断】千页级文档 = 单次百万级 DOM 节点，首屏时间与内存均线性劣化；与 office-b §2 的 word 编辑器（每页位图缓存 + 视口级懒渲染，office-b 报告 §2「word = 单画布 + 每页位图缓存 + 懒渲染」）是两个数量级的差距。

**与 OnlyOffice viewer 对照（office-b §2 复引，克制）**：OnlyOffice word 渲染 = canvas 位图页缓存 + wasm 字体度量引擎（office-b 报告 §2）；docx-preview = CSS/DOM 近似。差距定级：分页（真排版 vs 容器裁剪）、字体（度量引擎 vs 浏览器栈）、浮动对象（完整 Drawing 引擎 vs 三种映射）三面均为**保真代差**，但 docx-preview 换来了零 wasm、可复制文本、可 CSS 定制的工程便利。

### 6.2 SheetJS HTML 输出面定位登记

- `sheet_to_html`（bits/79_html.js:116-127）+ `make_html_row`（:60-92）：输出**纯数据骨架**——`<table><tr><td>` + 合并单元格 rowspan/colspan（:78-79）+ `data-t/data-v/data-z` 数据属性与坐标 id（:82-87）；**无样式/列宽行高/图表/图片**（w 仅 `escapehtml(cell.w...)`，:76）；editable 选项加 contenteditable span（:80）；有流式版 `XLSX.stream.to_html`（bits/97_node.js:35-59）。
- 敌意面：`<a href="' + cell.l.Target + '">'` 未转义（:85，§5.1）。
- **定位登记**：数据交换/兜底预览层，非保真渲染——「表格保真读侧」在开源 JS 面是空白（Univer/OnlyOffice 是编辑器级方案，成本面完全不同）。

### 6.3 校准目标：自研读侧合理目标线（一句话）

**「docx-preview 级结构忠实 + 三处超额（显式分页符全支持、wrapSquare 至少降级近似、嵌入字体还原）+ SheetJS 式数据骨架兜底，按需渲染解决大文档性能」——不做内容驱动重排版**（分页只信文档显式标记，这是 docx-preview 官方与 OnlyOffice 工程投入反差共同给出的性价比线）。

## 7. 两条生成路线对照：程序化拼装 vs 模板服务端渲染

**路线 A 程序化拼装（本层四件套）**：进程内 JS/TS API，声明式对象树（docx/exceljs）或命令式 add*（pptxgenjs）；产物由 §1-§5 各面决定；无外部进程依赖。

**路线 B 模板服务端渲染（office-b Builder，复引其 §7 已复审事实）**：
- Document Builder = C++ 独立进程 + .docbuilder 脚本（JS 语法跑在 DoctRenderer 的 V8/JSC）+ 全局 `Api` 层（office-b §7：apiBuilder.js word 侧 33489 行，builder 与编辑器共享同一模型代码）；
- 服务器化：`/docbuilder` 端点接受 URL 或内联脚本，spawnProcess 启动 docbuilder（office-b §7 converterservice.js:460-525）；
- 限制面（office-b §7 已复审）：单实例同时只持一个文件；保存=全量重序列化（无增量写）；**脚本对宿主进程全权限**；Document Builder Service/Automation API 仅 Developer Edition 打包（AGPL 仓代码完备但发行打包差异，office-b §7 登记的不确定项）。

**调用面对照（dsh 生成态设计输入）**：

| 维度 | A：程序化拼装（四件套） | B：Builder 模板渲染（office-b §7） |
|---|---|---|
| 调用形态 | 进程内声明式 API | 子进程脚本执行（服务端） |
| 模板保真 | docx patchDocument（占位级）；exceljs/pptxgenjs 弱 | **编辑器同源模型**，保真即编辑器保真 |
| 布局/重排 | docx 仅 estimatePageNumbers（页码估算） | 完整排版引擎（页码/目录域真实收尾） |
| 公式 | exceljs 写不算；office-c HyperFormula 补位 | 编辑器公式引擎同源 |
| 安全面 | §5 裁决：消毒/转义/域白名单全在调用方 | 脚本全权限信任模型（x2t 同级）；进程隔离但代码即权力 |
| 资源面 | §1 对照表；exceljs 唯一真流式 | 单实例单文件；全量重序列化；独立进程成本 |
| 部署依赖 | npm 三 MIT + Apache 一件，零外部进程 | C++ 二进制 + 发行打包许可不确定项 |
| LLM 适配 | **结构化中间表示好构造**（对象树/JSON 可序列化） | LLM 产 .docbuilder 脚本（office-b §7 「模型→结构化中间表示→布局引擎→OOXML」先例） |

【推断】dsh 生成态的合理组合：常规量/结构化数据 → 路线 A（四件套直拼，dsh 补消毒层）；模板保真/复杂排版 → 路线 B 思路但按 office-b §7 限制面评估（或自研「结构化中间表示 + A 路线渲染」折中）。

## 8. 对 dsh 生成态/读态的可移植结论

1. **生成态选型基线**（§1/§3）：xlsx 大批量 → exceljs `WorkbookWriter`（唯一真流式，行 commit 释放）；docx → docx 9.10（活跃度最高、特性面最全：Math/图表/patcher）；pptx → pptxgenjs（默认 STORE 记得开 DEFLATE）；纯数据表格进出 → sheetjs CE（多格式 IO + 最完整转义层）。四件许可全宽松（写侧四件 = 3×MIT + 1×Apache-2.0；加读侧对照 docx-preview 后的五对象口径才是 3×MIT + Apache-2.0×2），净室零义务。
2. **消毒是 dsh 管线层责任（本 pass 核心裁决，§5）**：四库零家做 CSV 公式首字符消毒（sheetjs 反而主动产 `=` 前缀）；控制字符面三档——docx/pptxgenjs 损坏档（拒开级）、exceljs 剥除档（合法 XML 但静默丢字符）、唯 sheetjs `_xHHHH_` 保字符达标；docx 域指令无白名单。dsh 生成态出口三件套：字符过滤器（喂入前）+ CSV 首字符消毒（出口处）+ 域白名单（docx 面向 LLM 时）。
3. **读态目标线**（§6.3）：结构忠实 + 显式分页 + 数据骨架兜底，不做内容驱动重排；docx-preview（Apache-2.0）与 sheet_to_html 作对照锚与兜底件。
4. **供应链注意**（§0）：sheetjs npm 包冻结 0.18.5（2022-03），新版仅 cdn.sheetjs.com 分发；GitHub 仓成壳（源码迁 git.sheetjs.com）。dsh 若采用需固定获取渠道与版本哈希，并评估 0.18.5 已知漏洞面；exceljs 处低维护态（npm 4.4.0 / 2023-10）——采用前评估接管成本。

## 附：证据锚点统计

- 五仓浅克隆只读（dolanmiu/docx@9.10.0、exceljs/exceljs@4.4.0、gitbrent/PptxGenJS@4.0.1、SheetJS/sheetjs@master 515d1c6、VolodymyrBaydalka/docxjs@0.4.x）+ npm registry 元数据五件 + xml@1.0.1 包源码一件。
- 锚点类型：源码路径:行号（主）、LICENSE/package.json（许可与版本）、README/docs（能力声明）、office-b 报告 §2/§7（复引，克制）。
- 【推断】标注：7 处（docx 内存倍数、§2.1 图表补丁能力、§2.2 exceljs 模板复用、§4 docx SmartArt 未实现面、sheetjs _xHHHH_ 歧义、docx-preview 大文档劣化、dsh 组合策略）。

