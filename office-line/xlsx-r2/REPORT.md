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

- mdcore main：18d5bb2 → **e8c3a13**（Phase 1）
- mdcore compliance-baseline：ddc51c1 → **048f1bb**（NOTICES v1.4）
- cleanroom-analysis：80499b4 → 本提交（xlsx-r2 报告 + xlsx-r1 三
  文档增量）

### 5. Phase 2/3 登记板

- Phase 2（下一增量）：mal7e 类静默降级 → 默认严格 typed 错误 +
  显式兼容模式（结构化警告）；门 2 与安全上限**保留不动**（受信源
  跳过已被否决——无基准证据）。
- Phase 3（XLSX 稳定后自动开始）：DOCX 小而可用读/写切片（复用
  ADR-O1 docx-rs 研究结论）；模型级处理与渲染保真分离；不支持的
  OOXML 特性与数据损失如实登记；不启渲染引擎、不重复生态调研。
