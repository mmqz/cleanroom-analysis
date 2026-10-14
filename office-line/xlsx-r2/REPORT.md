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

**遗留登记（不阻塞）**：①DOCX 切片按需增量面（页眉页脚/脚注/
图片/域——引擎能力在，未实测不暴露）；②XLSX 样式保真编辑需求
出现时另议（umya 或自研写侧，ADR 纪律）；③门 2 双读成本优化
（受信源跳过已裁决不设——维持至有基准证据）；④docx-rs 升级时
fork 线观察项 + 往返测试首跑纪律。