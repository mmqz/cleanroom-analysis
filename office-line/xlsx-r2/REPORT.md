# xlsx-r2 — XLSX 写面产品化 + 读面强健化 + DOCX 切片（三阶段）

> 依据：用户三阶段指令（2026-10-14）——Phase 1 XLSX（rust_xlsxwriter
> 升产品依赖 + 正式写 API + create→save→reopen→verify + 新建/编辑
> 边界区分 + 许可/SBOM/文档）/ Phase 2 Robustness（mal7e 类静默降级
> → 默认严格 typed 错误 + 显式兼容模式）/ Phase 3 DOCX（小而可用
> 读/写切片，模型级）。执行纪律：连续执行不停在规划/研究；复用
> xlsx-r1 全部既有代码/测试/探针/报告；MD/PDF 行为不变、引擎边界
> 独立；每连贯增量 commit/push；报告以实际提交与测试证据为准。
>
> 裁决回放（xlsx-r1 REPORT §6 遗留五项 → 本轮用户指令逐项落定）：
> ①xlsx-r2 C2 写面产品化 = 批准（本 Phase 1）；②mal7e 严格模式 =
> 批准（Phase 2）；③门 2 双读受信源跳过 = **否决**（无基准证据不得
> 加跳过，安全上限保留）；④R-O2 docx 渲染级读侧 = **改道**（Phase 3
> 只做模型级读/写切片，不启渲染引擎）；⑤D1-D3 CRDT/yrs/融合/统一
> 引擎 = **全部延后**至真实跨引擎需求出现。

## Phase 1 — C2 写面产品化（2026-10-14 完成）

### 1. 落地面

| 项 | 结果 | 锚点 |
|----|------|------|
| 依赖升级 | rust_xlsxwriter 0.99.1（constant_memory）dev → 产品 [dependencies]；Cargo.lock **零 diff**（同版本纯面重分类） | mdcore e8c3a13 `crates/office-io/Cargo.toml` |
| 正式写 API | `src/write.rs`（新模块）：WorkbookWriter/SheetWriter + CellValue 四型 + FormulaCache 三型 + CellStyle（style_probe 9/9 验证面）+ HyperlinkSpec（五形态）+ 流式表 | 同上 |
| 引擎边界独立 | 公开签名零 rust_xlsxwriter 类型（引擎类型零泄漏）；输入验证前移为自有 typed 契约（表名全规则/坐标边界/合并次序） | 同上 |
| 反静默丢失 | constant_memory 流式表行序纪律 typed 强制：引擎对已刷写行回退写**静默丢弃**（源级一手核验 worksheet.rs insert_cell `row < current_row` → eprintln+return）→ 包装层行高水位 typed reject | write.rs `check_coord` |
| 全环测试 | `tests/write_roundtrip.rs` 6 测试（create → save → **reopen 经产品读路径含容器双门** → verify）：五面全环 / 流式 2000 行 / 样式 XML 9/9 / 控制字符 XML 安全（全部件扫描）/ 读-改-重建证据 / save_to_vec | 同上 |
| 行为钉定 | `tests/range_anchor_pinned.rs`：calamine 范围锚定语义（值范围锚首格/公式范围锚首个公式格）——开发期实证发现相对坐标跨面配对产生**自洽假通过**（详见 §2 开发发现） | 同上 |
| 新建 vs 编辑边界 | write 模块文档 + lib.rs Capabilities 节：仅新建语义；C3 唯一口径 = 读-改-重建（五面保/样式元数据丢，FIDELITY v2 矩阵）；重放不可信读数据须先过 mdc-office 消毒链（消费方责任） | lib.rs + write.rs 模块文档 |
| NOTICES/SBOM | **v1.4**（compliance-baseline 048f1bb）：正文 cargo 101→106（账提升 5）+ 分类账 33→28 + 四面 50/64/88/101→58/71/93/106 + 新正文族 LLVM-exception×2 + ASSET_CHECK 零新资产 | mdcore 048f1bb |
| 分析仓文档 | CAPABILITIES.md（C2 产品面升级 + 证据链）/ DEPS.md（角色行 + §5 增量）/ FIDELITY.md（§6 产品级重放注记） | cleanroom-analysis 本提交 |

### 2. 开发期发现与修复（一手证据）

1. **calamine 范围锚定双语义**：`worksheet_range` 锚定首个非空格、
   `worksheet_formula` 锚定首个公式格——`Range::get` 坐标相对各自
   锚点。跨面配对（值↔公式文本）用相对坐标会产生**整体错位的自洽
   假通过**：值面按位缓存值保留 + 公式列均匀平移后相对重锚，值面
   与公式面两处对账均不可见错位。实测捕获于 write_roundtrip
   `read_modify_rebuild_path_evidence` 首版；修复 = 绝对坐标配对
   （fidelity.rs abs 键先例）；钉定测试 `range_anchor_pinned.rs`
   留档防回归。
2. **rust_xlsxwriter 控制字符转义一手核验**（xmlwriter.rs 源级）：
   `\x00`-`\x1F`（`\t`/`\n` 除外）按 Excel 约定转义 `_xHHHH_`；字面
   `_xHHHH_` 形态加前导转义（`_x0000_` → `_x005F_x0000_`）——与
   mdc-office 消毒面 X-03 同一约定。写产物恒为合法 XML 1.0（全部件
   扫描测试钉定）；值面往返零失真（calamine unescape_xml 逆转）。
   写面 XML 级结构性免疫控制字符注入——语义级消毒（CSV 公式守卫
   等）仍属消费方 mdc-office 链责任（契约在 write.rs 模块文档）。
3. **constant_memory 静默丢弃陷阱**：见 §1 表「反静默丢失」行——
   引擎以 eprintln 警告代替类型化错误，产品包装层以 typed
   InvalidInput 前置拦截（Phase 2 mal7e 纪律的写侧对偶）。
4. **字符串落点**：非流式表字符串进 sharedStrings.xml（非 worksheet
   部件）——控制字符断言须全部件扫描（首版测试查错部件捕获）。

### 3. 门禁证据（mdcore e8c3a13 实跑）

- **G1 `cargo test --workspace`：379/379**（基线 362 + 写面单测 10 +
  全环 6 + 锚定 1；全测试结果行 ok 零 fail）；
- **PDF 四面零波及**：lopdf 79/79 + pdfium 81/81+6i + dual
  109/109+6i（与 18d5bb2 推送态逐位一致）；
- **clippy 四面**：default/lopdf/pdfium/dual --all-targets 零新
  （mdc-wasm 2 条 static_method_of 在档基线维持）；
- **fmt 0 diff**；**scan.sh 双 0**（product + --all）；
- **wasm32 check 过**；**web 冒烟全过**（W1-W13 + G1/G2 门 PASS，
  0 failed——wasm-bindgen 0.2.129 按锁重建 + npm install + dist
  构建）；
- **feature 面载荷按 T1-F12 bootstrap 重建**：fetch-pdfium.sh
  （chromium/8086 SHA256 验证）+ spike 三探针复测全过 + fixtures
  现场生成（平台重置导致的环境恢复，非代码变更）。

### 4. 版本链

- mdcore main：18d5bb2 → **e8c3a13**（Phase 1）→ **042a5e4**（Phase 2）
- mdcore compliance-baseline：ddc51c1 → **048f1bb**（NOTICES v1.4，
  Phase 1；Phase 2 零增量）
- cleanroom-analysis：80499b4 → 本仓提交链（xlsx-r2 报告 + xlsx-r1
  文档增量 + SECURITY_POLICY v1.1）

### 5. Phase 2 — 读面强健化（2026-10-14 完成）

用户指令裁决落地：mal7e 类问题（畸形/无效包部件不得静默变空数据）
→ 默认严格 typed 错误 + 显式兼容模式（结构化警告）；门 2 双读
受信源跳过**否决**（无基准证据不得加跳过——安全上限全保留）。

| 项 | 结果 | 锚点 |
|----|------|------|
| 门 3 部件良构性 | .xml/.rels 部件流式结构校验（quick-xml 0.41 直依赖——原 calamine 传递，闭包原位零增量零 lock 变化）：标签配对[check_end_names] + 根在位唯一 + EOF 未闭合[截断] + 深度上限 LIMIT_XML_DEPTH=1000[嵌套炸弹——SECURITY_POLICY §2.2.3 挂账闭项] | mdcore 042a5e4 io.rs `verify_pass`/`validate_xml_structure` |
| 融合单趟 | 门 3 融合在门 2 既有解压趟内（CountingReader 适配器：尺寸计数 + 控制字节扫描）——**零额外解压成本**（双读总数不变，与裁决一致） | 同上 |
| 默认严格 | `ReadError::InvalidPart{part, detail}` typed reject；`read_book` 签名不变行为收敛 | 同上 |
| 显式兼容 | `ReadOptions{compat}` + `CompatibilityWarning{part, kind, detail}` 随 `Book.warnings`；兼容只放宽本层验证，引擎行为不变（mal7a 兼容下仍引擎 ParseError）；门 1/门 2/安全上限两模式**恒等** | 同上 |
| API 兼容 | `verify_actual_sizes` 公开独立面保持（仅尺寸——历史契约不变，独立测试钉定） | io.rs + lib 单测 |
| 行为收敛 | mal7e OK→REJECT（兼容 Ok+警告+空表）；mal7a ERR→REJECT（兼容 Err[引擎错]）；缺根闭合截断容忍→REJECT（兼容警告+值读回——xlsx-r1 分界在兼容模式完整保持）；mal7c 值面容忍**不变**+恒登记 ControlChars 警告 | corpus_harness 三测试 |

**语料三分归类（Phase 2 后）**：拒绝 5（门 1/2 四件 + 门 3 mal7e）/
解析错误 5 / 容忍 1（mal7c + 警告面）——严格/兼容双期望表逐位对照
在档（兼容模式复扫仅门 3 两项变化）。

**门禁（mdcore 042a5e4 实跑）**：G1 **385/385**（379+6）+ lopdf
79/79 + pdfium 81/81+6i + dual 109/109+6i（PDF 四面零波及）+ clippy
四面 2 wasm 基线零新 + fmt 0 + scan 0 + wasm32 过。

**文档同步**：SECURITY_POLICY **v1.1**（§2.1 第 5 项门 3 + §2.2.3
闭项 + §3 两行 + 生效锚）；CAPABILITIES（C1 宣称语 + §3 容忍度
Phase 2 后口径）；DEPS（quick-xml 直依赖行）。NOTICES **零增量**
（quick-xml 原在正文 v1.4——传递→直依赖不改变闭包面）。

### 6. Phase 3 — DOCX 小而可用读/写切片（2026-10-14 完成）

用户指令落地：小而可用切片（复用 ADR-O1 docx-rs 研究结论）；模型级
处理与渲染保真分离；不支持 OOXML 特性与数据损失登记；不启渲染
引擎、不重复生态调研（零新调研——全部一手核验限于已选版本 API 面
与行为钉定）。

| 项 | 结果 | 锚点 |
|--|--|--|
| 引擎接入 | docx-rs 0.4.22（ADR-O1 O1-3；default-features=false 关 image 巨树）+ base64 0.22.1 = 闭包仅 +2；三链核验 + fork 线观察项纪律常驻化 | mdcore 460f302 Cargo.toml + NOTICES v1.5 |
| 模块边界 | DOCX 独立模块（CAPABILITIES §2 裁决）：自有模型（DocxDocument/Block/RunData/TableData）+ 自有错误面（DocxError）；docx-rs 类型零泄漏；共享面仅容器门基础设施（zip+OPC 同构）与 mdc-office 消毒链 | crates/office-io/src/docx.rs |
| 读面 | 容器三门复用（与 XLSX 同一门实现——含严格/兼容两模式与结构化警告）+ 模型提取：块序/样式 ID 透传/run 级文本加粗斜体/超链接内文本[目标丢弃]/表格单元格拼接[多段 \n 连接]/Tab·Break·CR 文本保真；字节入口同三门不绕门 | docx.rs `read_docx`* |
| 写面 | DocxWriter 仅新建语义（与 XLSX 写面边界同构）：Heading1-9 样式定义[outline 级模型语义无字体面] + 富文本段落 + 文本表格 + save_to_path/vec | docx.rs `DocxWriter` |
| 不支持面登记 | 读侧跳过：图形/图片/超链接目标/修订追踪/域/书签/批注/内容控件/页眉页脚/脚注尾注/编号语义；写侧未暴露：图片/表格样式/域/目录/页眉页脚/编号列表；渲染语义（布局/分页/字体度量）整体面外 | docx.rs 模块文档登记表 |
| 行为差异钉定 | docx-rs 读引擎对原始控制字符**自身硬拒**（Parse）——较 calamine/XLSX 值面容忍更严；门 3 ControlChars 警告在该错误路径不可达（引擎错误优先）；两模式同拒 | tests/docx_roundtrip.rs |
| 测试 | 5 测试：写读全环（6 块序逐面断言）+ 容器门复用钉定（截断/垃圾/路径逃逸 + 严格/兼容双模式）+ 字节入口同门 + save_to_vec 重开 + 控制字符引擎行为 | 同上 |

**门禁（mdcore 460f302 实跑）**：G1 **392/392**（385+7）+ PDF 四面
79/81+6i/109+6i 零波及 + clippy 四面 2 wasm 基线零新 + fmt 0 +
scan 0 + wasm32 过。四面闭包 58/71/93/106 → **62/75/95/108**。

### 7. 三阶段总表（2026-10-14 收口）

| 阶段 | 交付 | mdcore | 分析仓/合规 |
|--|--|--|--|
| Phase 1 XLSX 写面产品化 | rust_xlsxwriter 升产品依赖 + write.rs 正式写 API（引擎零泄漏/验证前移/流式纪律 typed 拦截）+ 全环测试 6 + 锚定钉定 1 + 新建/编辑边界 + 消毒契约 | e8c3a13 | NOTICES v1.4（048f1bb）+ 0dec786 |
| Phase 2 读面强健化 | 门 3 部件良构性（融合单趟零额外解压）+ 默认严格 typed 错误 + 显式兼容模式结构化警告 + 深度上限闭项 + 语料重归类 | 042a5e4 | SECURITY_POLICY v1.1 + 95cefa6 |
| Phase 3 DOCX 切片 | docx-rs 引擎接入（闭包 +2）+ 模型级读/写 + 容器门复用 + 不支持面登记 + 行为差异钉定 | 460f302 | NOTICES v1.5（5e144cc）+ 本提交 |

**最终测试计数**：G1 **392/392**（xlsx-r1 阶段 B 基线 362 → +30：
Phase 1 写面 10+6+1 / Phase 2 门 3 重钉+新增与兼容复扫 6 / Phase 3
DOCX 2+5[+行为钉定并入]）；PDF 四面零波及逐位一致；web 冒烟全过。

**遗留登记（2026-10-14 用户裁决后状态）**：
①DOCX 切片按需增量面——**已穷尽**（docx-r1/r2 两轮：页眉页脚/
脚注/图片/域/first-even 全部裁决，暴露有实测、受限有锚点）；
②XLSX 样式保真编辑——**已裁决：维持 C3「读—改—重建」边界，不引入
umya-spreadsheet，样式/宏/图表等保真限制在 FIDELITY 显式登记**
（丢失矩阵补宏/图表行 + calamine vba 潜在能力锚点）；③门 2 双读
受信源跳过——**已裁决：维持否决，不增跳过开关**（无基准证据，
安全上限全保留）；④docx-rs 升级时
fork 线观察项 + 往返测试首跑纪律。

### 8. Round 2.5 / t17 — XLSX web 消费切片（2026-10-14 完成）

t16（DOCX→mdc-wasm→web 端到端）模式的对称展开：XLSX 读面经 wasm
桥导入 web 编辑器——「import .xlsx」按钮 → 容器三门 → calamine 五面
→ 表→GFM 表 markdown → 编辑器核心重建（可编辑/可撤销）。**只读消费**
——XLSX 维持 C3 读—改—重建裁决，本轮不经 wasm 写 xlsx。

| 项 | 结果 | 锚点 |
|--|--|--|
| 字节入口 | office-io `read_book_from_bytes(_with_options)` 增量（wasm32 无文件系统；动机 = docx.rs `read_docx_from_bytes` 同款）+ 五面提取抽共享核 `extract_sheets`（路径/字节两入口同一实现，行为单一真源） | io.rs（+39 行，路径面行为不变既有测试全绿） |
| wasm 模块 | `mdcore-wasm/src/xlsx_import.rs`：纯映射内核 + bindgen 壳；**calamine/zip/rust_xlsxwriter 类型零具名**（单元格经 Data Display 值面渲染——wasm 导出面零引擎类型从结构上保证）；夹具产线单一真源 `build_sample_xlsx`（写面 WorkbookWriter = C2→C1 栈复证） | mdcore main（t17 提交） |
| 错误码族 | `container_prescan`/`container_corrupt`/`container_invalid_part` + `xlsx_parse`（与 t16 `docx_parse` 同族命名）；**族差异登记**：`container_parse` 不产生——office-io XLSX 读面过门后为单一 `ParseError`（无 docx 式 Container/Parse 二分） | xlsx_import.rs 模块文档错误码表 |
| 映射规则 | 每表 `## {表名}` + GFM 管道表（首个使用行 = 表头）；单元格 Data Display 值面 + `\|` 转义/换行拍平；**公式格 = 缓存值**（Excel 显示语义——「缓存缺失读回 Float(0.0) 两义性」在档使「有缓存用缓存否则公式文本」不可实现）；合并 = 左上值+空（值矩阵自然编码）；超链接目标丢弃（显示文本保留）；空表 = `(empty sheet)` 注记 | 同上（映射规则节） |
| 规模上限 | MAX_SHEETS 16 / MAX_ROWS 50 / MAX_COLS 32（使用区前缀）→ `sheets_dropped`/`rows_dropped`/`cols_dropped` stats | 同上 |
| stats | sheets/tables/empty_sheets/rows/cells/formulas_flattened/merged_ranges/hyperlinks_dropped/三 dropped/warnings[]（12 字段——丢弃面计数显式化） | 同上 |
| 测试 | 原生 +12（mdcore-wasm 10：写读映射全环精确等值/值面渲染/公式策略/合并/上限三 dropped/空表/错误码四臂/垃圾×3/截断/夹具漂移守卫 + office-io io 2：字节路径等价/字节敌意）；web 第五套 `xlsx_import.py` 9 检查（真实 file input 精确 markdown/块型 heading+table×2+paragraph/stats+状态行/活编辑器/undo/敌意垃圾+截断零触碰/契约完整 0 pageerror） | G1 435→447；web 18/15/28/9/9 |
| 门禁 | G1 **447/0/1i** + PDF 三面 79/81+6i/109+6i 零漂移 + clippy 四面恰 2 wasm 基线（新代码 1 警修净）+ fmt 0 + scan 0+0 + wasm32 过 + tsc 清洁 + **Cargo.lock 零 diff**（无任何新依赖——office-io 已在图） | mdcore main（t17 提交） |
| 包体 | wasm 1,829,489 → **2,066,405 B（+236,916 B，+12.95%，实测）**；dist/main.js +3,662 B。归因：calamine xlsx 读机制进入活调用图（t16 docx 路径从未调用——cdylib 死码剔除下不在包内）+ rust_xlsxwriter 经夹具产线 pub 面（t16 docx 写面入包同款先例）。任务书「~unchanged」预期**被实测推翻**，诚实登记（无门禁线） | build-wasm.sh 产物实测 |
| t16 零回归 | docx_import.py 9/9 亲跑过 + web_smoke 15/15 + contract_gate 18/18 + ime_cdp 28/28（MD 面独立复证） | web/tests/results/ |

**用户故事达成**：web 编辑器「import .xlsx」→ 工作簿真实导入为可编辑
markdown 文档（每表一节 + GFM 表 + 布尔/数值/公式缓存值/合并/管道转义
单元格），敌意输入 typed 拒绝且编辑器零触碰——XLSX 读面端到端消费
切片全绿收口（C1 消费面宣称见 xlsx-r1/CAPABILITIES §1 增量注记）。

**登记边界（非缺陷）**：numFmt 显示形态不宣称（值面：`4.5`/`9`/
Excel 序列数日期）；公式求值不发生（缓存值即所见）；超链接目标丢弃
（计数显式）；样式/图表/图片不进入 markdown（C1 面外，FIDELITY 真源）；
表名换行拍平；使用区含仅格式空格的扩张（Excel 使用区语义）。
### 9. Round 3 / t19 — XLSX 导出切片（2026-10-14 完成）

t18（DOCX 导出）模式的对称展开：编辑器当前文档 → office-io 写面
（`WorkbookWriter` 新建工作簿）经 wasm 桥 → `.xlsx` 字节下载。与
t17（XLSX 导入）合流 = **XLSX 往返闭环**，office 消费四切片
（t16/t17/t18/t19）全部收口。表数据面全环 = export → t17
`import_xlsx` → GFM 逐格等值（e2e + 原生双证）。

| 项 | 结果 | 锚点 |
|--|--|--|
| wasm 模块 | `mdcore-wasm/src/xlsx_export.rs`（~590L）：模块文档（栅格映射表/sheet 命名策略/错误码表/已知边界——单一真源随码走）+ 纯内核（sanitize_sheet_name/unique_sheet_name/table_sheet_name/collect_text_rows/collect_tables/write_document_sheet/write_table_sheet/export_xlsx_core）+ `WasmCore::export_xlsx` 增量方法（t18 `export_docx` 同款壳）；GFM 管道语义与 t18 单一真源共享（docx_export.rs `text_lines`/`table_rows` pub(crate) 化，零行为变化） | mdcore main（t19 提交） |
| 栅格映射 | **Table → 每表一独立 sheet**（文档序；分隔行剔除 + `\|` 反转义 + 单元格换行拍平[共享 GFM 语义]；**全部格 Text 类型不推断**——前导零/日期式字符串不失真 + t17 读回 Display 逐字等值；ragged 行无补齐；空文本格不落格；0 行表跳过[防御]；行 2^20/列 2^14 防御截断）；**非表块 → 前置 Document 表**（heading 加粗行[CellStyle.bold = style_probe 验证面]、level 丢；paragraph/code 逐字行；quote 容器丢子块平铺；list marker/depth 丢；divider = 空行）；**空文档 → 单空表 Sheet1**（Excel 工作簿至少一表） | xlsx_export.rs 模块文档映射表 |
| sheet 命名策略 | 表名 = 表头 A1 清洗（`\n\r\t`→空格、剥 `[]:*?/\`、trim 空白+单引号、截 31 chars、重 trim）→ 空/`History`（忽略大小写）→ 回退 `Table {n}`；精确冲突 → ` 2`/` 3` 后缀去重（基底重截保 ≤31）；与 `Document` 冲突同规则（文本块先占位）；清洗保证恒过 write.rs typed 表名验证（InvalidInput 构造不可达） | 同上（命名策略节） |
| 错误码 | 单族 `xlsx_build`（WriteError 完备映射；Engine = save_to_vec 打包失败唯一现实路径）——t18 `docx_build` 同构；映射层全函数：降级永不报错 | 同上（错误码表） |
| web 面 | `export .xlsx` 动作钮 + `__exportXlsx`/`__lastXlsxExport` 契约（t18 约定镜像：not_ready 兜底 + JSON 错误面 + Blob/a.download `document.xlsx`）；状态行 `exported: X KB · N sheets`（N = `__blocks` 块型近似——wasm 命名策略的显示面） | web/index.html + web/src/main.ts t19 块 |
| **wasm32 运行时崩溃面（两项一手发现）** | ①`Workbook::new()`→`DocProperties::new()`→`utc_now()`→`SystemTime::now()`（rust_xlsxwriter workbook.rs:419/properties.rs:376/datetime.rs:1308）——wasm32 无时间系统运行时 panic（wasm trap "unreachable"）；修复 = 引擎 `wasm` 特性（`js_sys::Date::now()`，cfg = feature"wasm" AND target"wasm32" → 原生行为零变化）。②修复①后仍 panic "no filesystem on this platform"（std sys/paths/unsupported.rs:52）：`constant_memory` 特性使 `Worksheet::new()`（worksheet.rs:1742）对**每张表（含非流式普通表）**急切 `tempfile_in(temp_dir())`——wasm32 无文件系统；修复 = rust_xlsxwriter 依赖边**目标分裂**（非 wasm32: constant_memory+wasm / wasm32: 仅 wasm）+ office-io `add_sheet_streaming` `#[cfg(not(wasm32))]` 条件化（流式面 = 引擎 tempfile 结构性质，wasm32 缺席而非降级——3 个流式测试同 cfg 门）。诊断 = 临时 panic hook（console.error）读取真实 panic 消息后移除 | office-io Cargo.toml t19 注记 + write.rs 模块文档 + mdcore t19 提交 |
| Cargo.lock | 恰 +2 行（rust_xlsxwriter 条目 dependencies 数组 +js-sys +wasm-bindgen）；**两包均已在 lock**（chrono/console_error_panic_hook 等既有边）——零新包、零版本变动。任务书「零 diff」字面预期对 wasm32 运行时崩溃修复不可达（无 API 旁路：`set_properties` 替换发生在 `Workbook::new()` panic 之后），最小必然 diff 如实登记（t16 path 边先例） | git diff Cargo.lock |
| 测试 | 原生 +12（mdcore-wasm 43/43：表→栅格精确/命名策略全分支[长名截断/非法字符/去重/History/空表头/Document 冲突]/线性块 Document 表序/code 逐字/all-Text 类型判别[cell_debug `Some(String("4.5"))`/前导零 `007` 零损]/管道转义格/ragged 无补齐/空树最小工作簿/纯表无 Document/16386 宽行防御截断/**t17 全环×2**[表逐字等值 + Document 表 GFM 等值]）；web 第七套 `xlsx_export.py` 9 检查（port 8936：UI/真实按钮+valid zip/workbook.xml 表名/sharedStrings all-Text 容器证据+styles `<b/>`/降级块+只读/空文档/全环 `__importXlsx` 规范形逐字/契约零页错） | G1 457→**469**；web 八套件 |
| 门禁 | G1 **469/0/1i** + PDF 四面 68/79/81+6i/109+6i 零漂移 + clippy 四面恰 2 wasm 基线（PDF 三面 0）+ fmt 0 + scan 0+0 + wasm32 过（mdcore-wasm + office-io 双面）+ tsc 清洁 + **八套件 18/15/28/9/9/8/9 全绿**（t16/t17/t18 零回归亲证——终版 dist 复跑） | mdcore main（t19 提交） |
| 包体与归因勘误 | wasm 3,105,817 → **3,950,549 B（+844,732 B，+27.2%）**；dist/main.js 554,265 → 557,256（+2,991 B）。归因：rust_xlsxwriter 写面机构（OOXML 序列化 + zip 打包 + styles/theme/sharedStrings 机构）**首次从 export_xlsx 可达**——**t17「pub fixture builder 保活 rust_xlsxwriter」归因勘误成立**：`build_sample_xlsx` 为 `#[doc(hidden)]` pub 非 wasm 导出面 → cdylib 导出图死码剥离（t18 对 docx-rs 同款勘误的 XLSX 对偶；本轮实测再证） | build-wasm.sh 产物实测 |

**用户故事达成**：web 编辑器「export .xlsx」→ 当前文档真实导出为
`.xlsx` 下载（表块 → 逐表栅格 + 线性块 → Document 表平铺），导出
只读（编辑器零触碰），全环 re-import 表数据 GFM 逐格还原——XLSX
往返闭环与 office 消费四切片（t16/t17/t18/t19）全部收口。

**登记边界（非缺陷）**：数值/布尔类型语义不宣称（全 Text——Excel
侧求值/排序按文本；文本零损与类型语义二选一，取文本零损）；单元格
富文本不宣称（marks 全降级纯文本——rust_xlsxwriter 富文本 API 不在
office-io 包装面，未实测不宣称）；heading 加粗为唯一样式面（level
不可分辨）；流式表 wasm32 结构性缺席（引擎 tempfile 背书——API
条件化，非降级）；t17 导入侧规模上限（16 表/50 行/32 列）是导入面
边界，不在导出面（导出无上限截断除 2^20/2^14 物理边界）。

## 10. Round 4 / r4-t1 — XLSX 类型推断与转换规则（2026-10-17 完成）

> 定位：Round 4 T1——用户裁决「默认保留标识符/前导零/数字文本不
> 擅自转数字；日期转换必须有明确依据；公式读取结果与公式文本区分；
> 不支持类型给可诊断结果」的落地轮（t22-review 前置裁决项⑤
> 「XLSX 数值/布尔类型推断策略」的答案）。规则依据表唯一真源 =
> xlsx-r1/FIDELITY.md §8（R1-R13，每条规则一行：规则/依据/测试
> 锚点）；本节为工程收口记录。

### 10.1 落地面（office-io 只增不改；既有调用方零破坏）

| 项 | 结果 | 锚点 |
|--|--|--|
| 类型化值模型 | `io::CellData` 九变体（Text/Number/Boolean/DateTime{serial,is_1904}/DateTimeIso/Duration{serial,is_1904}/DurationIso/Error(String)/Empty）——原始值/推断类型/显式转换三分：变体即 kind（日期类携带依据），`display()` = t17 Display 语义同源，`as_iso_datetime()`/`as_iso_duration()` = 显式转换（无据 → None），`to_write_parts()` = kind 分派 → (write::CellValue, Option<CellStyle>) 成对返回（防样式漏配） | mdcore r4-t1 提交 io.rs |
| 类型化矩阵 | `io::CellGrid`（与 values 同一 used-range 包围盒/坐标系——单点派生自 values 零漂移；`get` 相对 + `get_abs` 绝对双坐标面 = range_anchor_pinned 锚定语义免坑）+ `SheetData.typed` 字段（Default 兼容） | 同上 |
| 公式安全联接 | `SheetData::formula_at(row, col)`（公式文本独立通道——锚定换算内化；无 `=` 前缀在档）+ `SheetData::replay_cell(row, col)`（读-改-重建单调用点：公式两通道联接[缓存 kind 映射含 DateTime→序列数缓存+日期样式、Error→码文本缓存降级] + kind 分派） | 同上 |
| 纪元 | `Xlsx::has_1904_epoch()` 构造期 eager 解析进模型；写面联接 epoch_to_1900（+1462 换算，写面引擎恒 1900 纪元——历法日期保持） | 同上 |
| GFM 映射 | xlsx_import 渲染切至 `typed.get().display()`（显示面字节零变化——既有断言即钉定 + 新等值测试 + web 七套件零断言改动）；模块自此零 calamine 面接触 | mdcore-wasm xlsx_import.rs |
| wasm 导出面 | 零代码变化；模块文档 lang 语义登记：BlockTree 文本模型无类型载荷，「数字文本不擅自转数字」裁决使文本推断不可行 → 全 Text 维持为正确语义；类型保持联接在 office-io 层 replay_cell | mdcore-wasm xlsx_export.rs |

### 10.2 探针与往返实测（kind + 值双面）

- **T1.1 探针**（tests/cell_types.rs 前四测试，raw XML zip 自构造
  14 格七类+边界）：数字/前导零文本/布尔/日期（builtin numFmt14 +
  自定义 yyyy-mm-dd）/纯序列数（不认日期）/ISO（t="d"）/公式
  （=1+1 两通道）/空格/错误（#DIV/0!）/2^53+1 长整数/浮点形态
  （0.1+0.2）/时长（[h]:mm:ss）/t="str"——Data 面 + Display 面
  逐格钉定（44927 = 2023-01-01；1904 纪元同 serial = 2027-01-02）。
- **kind 保持往返**（`kind_preserving_replay_roundtrip`）：探针档 →
  replay_cell 全格重放 → 重读——Text/Number/Boolean kind 保持
  （前导零 `00123` 零损）；DateTime（双 numFmt 面）与 Duration
  kind 往返（写 Number(serial)+日期格式 → calamine 经 numFmt 证据
  再判型）；纯序列数仍 Number（kind 不漂移）；公式两通道往返
  （缓存 Number(2) + 文本 "1+1"）；错误/ISO → Text（写面降级
  登记，值保持）；2^53+1 精度随存储面恒在；1904 链历法日期保持
  （44927 → 46389 → 2027-01-02）。
- **显示面等值**（`typed_display_equivalence_with_data_display`）：
  typed.display() 与 Data::to_string 全格逐字等值——t17 消费切换
  安全依据。

### 10.3 门禁（全部亲跑）

| 门 | 结果 |
|--|--|
| G1 | **491/0/1i**（481 基线 +10：office-io 84→93[cell_types.rs +9]、mdcore-wasm 49→50[+1]；既有测试零改动）；一次全量跑 r23_fingerprint 出现 14+1 负载敏感 flake（mdcore 域时间型测试，与本轮零代码路径交集）——隔离 8/8 绿 + 后续两次全量 491/0/1i 绿，如实登记 |
| PDF 四面 | 68/79/81+6i/109+6i 零漂移 + perf --ignored 双面 6/6 |
| clippy | native --all-targets 恰 2（wasm_bindgen 基线）+ PDF 三面 0 + wasm32 --all-targets 2（同基线） |
| fmt / scan / lock | fmt 0；scan 0（全仓凭据模式）；Cargo.lock 零 diff（零新依赖）；wasm32 check + --all-targets 过 |
| web 七套件 | **18/15/28/9/9/9/13 全绿**（contract/smoke/ime/docx_import/xlsx_import/xlsx_export/docx_export——零断言改动，显示面兼容 e2e 亲证） |
| 包体 | wasm 3,970,827 → **3,969,548 B（−1,279 B / −0.032%**，CellData::display 替代 wasm 面 calamine Data Display 路径净缩）；dist/main.js 557,211 B 零变化 |

### 10.4 遗留与后续

- BlockTree 扩单元格类型载荷（BlockKind::Table 表格带类型）= mdcore
  域上游裁决——若立项，wasm 导出面可升级为按模型 kind 分派
  （replay 通道已就绪）；
- numFmt 显示形态（¥/百分比/日历显示）维持 C1 不宣称（值面语义
  ——样式面 calamine 不可观测，FIDELITY §3 丢失矩阵不变）；
- 公式求值引擎（T4 威胁面）未立项——缓存值即所见维持。
