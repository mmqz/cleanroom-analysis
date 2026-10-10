# PdfCraft 净室 PDF 引擎 · 架构与设计深度剖析

> **文档性质**：净室分析报告（分析员视角，不含任何实现代码）
> **分析对象**：github.com/storytold/pdfcraft（v0.4.0，仓库状态截至 2026-10-09）
> **用途**：为"用 Rust 自行实现一个 PDF 引擎"提供架构参考
> **聚焦**：数据完整性体系、渲染管线、格式转换层（按约定跳过 Agent 自动化）
> **来源方式**：全部结论来自该仓库公开的模块文档注释、crate README、ROADMAP、vendor 清单与 CI 配置；关键结论均标注出处文件，可自行核对

---

## 0. 净室纪律：他们怎么做，你怎么做才合规

在进入架构之前，必须先钉死"净室"在这个项目里的确切含义，因为这决定了你模仿时的安全边界。

**PdfCraft 的净室含义**（出处：crates/filters/README.md、vendor/README.md、ROADMAP.md）：

1. **输入只来自公开规范**。每一层实现都显式标注规范的出处：流过滤器写明"依据 ISO 32000-2 §7.4、RFC 1950/1951、TIFF 6.0 §14、RFC 2083 §6"；加密写明 ISO 32000-2 §7.6；内容流写明 §7.8.2/§8/§9；页树继承写明 §7.7.3.4。实现行为与规范冲突时以规范为准，规范未覆盖的现实世界损伤按"容错+计数"处理，而不是去观察 Acrobat 的行为再模仿。
2. **Acrobat 只充当黑盒对照（oracle），且从未系统使用**。ROADMAP 明确承认"与 Acrobat 的保真度未测量、无系统性并排对比"，对照工作目前依赖的是 qpdf、pdftotext、pdfsig、OpenSSL、Poppler 这些**开源工具**，而不是 Adobe 的产品。规划中的保真度测试夹具（plan/acrobat/）同样声明"净室规则适用"。
3. **依赖与资产纪律**。CI 用 cargo-deny 做许可证/来源/安全审计（deny.toml）；vendor 目录只允许携带"宽容许可 + 小补丁"的上游 crate，每补丁必须在源码中标注、必须配一个测试、必须争取上游合并、上游发布后必须移除；**vendor 任何 copyleft 代码被 ADR-0001 明确禁止**。捆绑字体全部来自 OFL/公开领域（craft-fonts），且刻意不打包任何厂商的字体度量文件——Helvetica 的度量是"按字符类别估算的自有近似值"，够用于换行对齐而不冒充排版级精度（crates/fonts/README.md）。
4. **商标隔离**。Adobe 商标仅用于描述兼容的工作流；ArtCraft 品牌资产明确排除在 MIT/Apache 许可之外，fork 必须移除。

**对你的含义**：PdfCraft 本身是 MIT/Apache-2.0 双许可的开源代码，阅读、学习甚至直接复用它都不存在法律障碍——你面对 Adobe 才需要净室纪律。如果你追求的是"和它一样的方法论"，那么真正的净室承诺只有三条：不反编译 Adobe 产品、不引用 Adobe 内部文档、行为争议以 ISO 32000-2 及其引用的公开 RFC 为准。

---

## 1. 总体架构：先声明分层表，再让 crate 长进来

### 1.1 规模与现状

截至本报告基准日：工作区共 30 个已实现 crate，分层表中另预留约 15 个尚未落地的槽位；约 7.2 万行"保留下来的、带测试的 Rust"（他们的口径是 kept, tested lines）；约 600 个测试；功能平价 51.0%（806 项被跟踪的 Acrobat Pro 离线功能），按工作量加权约 30–35%。开发方式为 Claude Opus 5.5 连续 agent 小时（约 65–75 小时产出上述规模，约每小时千行），人类角色是审查与决策（ROADMAP.md）。

### 1.2 分层表（CI 机器强制执行，出处：xtask/src/layers.rs）

```
L8  apps + tooling（豁免）        pdfcraft · cli · web · xtask
L7  平台与前端                    platform · ui-common* · ui-egui · automation
L6  门面（façade）                engine
L5  交互                          viewport · tools          （预留）
L4  功能特性                      edit · organize · security* · redact · xfdf · sign
                                  ocr · create · export · optimize · preflight
                                  prepress* · a11y · compare · measure · search*
                                  print · media* · ai*
L3  核心服务                      render · text* · annot · forms · js · xfa · ml*
L2  核心模型                      content · model · color* · fonts · ops*
L1  对象层（standalone）          cos（仅可依赖 filters/crypt/geom）
L0  地基（standalone，可发布）    geom · filters · crypt · arlington*
                                  * 尚未落地的预留槽位
```

执行规则（全部由 `cargo xtask layers` 在 CI 中机器校验，plan/architecture.md §3）：

- **只能向下依赖**；同层依赖需要进入白名单 SIDEWAYS，且白名单是刻意收敛的——例如 `forms → js` 被明确缺席（表单与脚本运行时不得互相纠缠）；现存的同层边仅 redact→edit、a11y→preflight、prepress→preflight、export→ocr、sign→security、ui-egui→ui-common/platform、automation→platform 等八条。
- **L0/L1 是 standalone**：cos 只准依赖 filters、crypt、geom 三个工作区内 crate（STANDALONE_DEPS），保证这五个地基 crate 可以独立发布、无 UI 传染。
- **UI 工具链（egui/winit/rfd/wgpu 前缀）只允许出现在 L7 及以上**（UI_MIN_LAYER=7），由依赖名前缀匹配机器执行。
- 每个新 crate 必须先登记进这张表，未登记即报错——**架构是一张先于代码存在的有限表，crate 是往槽位里生长的内容**。这是本项目中值得原样抄走的一条工程决策。

### 1.3 质量门（cargo xtask ci，串行停首败）

`fmt --check` → `clippy -D warnings` → `test` → `layers`（分层校验）→ `wasm`（L8 以下全部 crate 编译到 wasm32-unknown-unknown）→ `assets`（资产许可对 ATTRIBUTION.toml 审计）→ `deny`（cargo-deny）→ `parity`（功能平价计量）。

另一个全局细节：几乎每个 crate 头部都有同一条 lint 禁令组——unwrap/expect/panic/unimplemented/todo/unreachable 全部 deny。配合"解析器永不 panic"的模块级承诺，这实际上是**把"敌意输入不得崩溃进程"写进了编译期纪律**。

---

## 2. 数据完整性体系（L1 cos + L6 engine）

这是该项目设计密度最高的部分。核心思路一句话：**对象图就是文档模型（ADR-0010），编辑永远发生在"原始字节之上的写时复制覆盖层"，而"保存"有两条物理路径，其中增量路径保证原始字节永不改变。**

### 2.1 容错解析与修复（crates/cos/src/parser.rs、document.rs）

- **解析器形态**：字节切片 + 位置的游标式词法/语法分析，逐字节边界检查，嵌套深度上限 128，跳过的坏数据**不分配内存**、只计数。目标态是"任何输入都不 panic、不越界"。
- **惰性加载**：xref 建索引后对象按需解析，解析结果进缓存；`Document` 的原始字节、xref 索引、解析缓存全部以共享句柄持有——**克隆一个 Document 的代价接近零**，这是整个撤销/快照体系的地基。
- **修复策略**（"M1 in progress"注释）：支持全部 xref 形态（经典表/xref 流/对象流），xref 损坏时退化为**扫描全文件重建对象索引**。流对象的 `/Length` 错误时按多策略重试定位 `stream` 关键字；由于链式流的 Length 可能又指向另一个流，重试会造成指数级重复解析，解法是线程本地的**解析记忆化**（对已解析过的间接 Length 建缓存）。
- **循环检测**：对象加载可重入（对象流的循环引用），用一个线程本地的"加载中栈"识别环。页树环单独在渲染侧防御（vendor 补丁，见 §3.6）。

### 2.2 两条写出路径（crates/cos/src/writer.rs）

| 路径 | 行为 | 用途与不变量 |
|---|---|---|
| **增量写** | 只追加被编辑的对象 + 一段与文件上一修订**同风格**的 xref（表或流）+ 带 `/Prev` 的 trailer | 原始字节一个都不改 → **对早期修订的数字签名保持有效**；被修改对象在新修订里获得新字节区间 |
| **全量写** | 从 trailer 出发做**可达性垃圾回收**，仅写出可达对象，**稠密重编号**，经典 xref 表，对象流打包压缩 | "另存为"/GC 路径；体积优化；丢弃增量历史的重写 |

辅助设计：**确定性序列化**——同一对象图序列化出的字节完全一致（同对象→同字节）。这让"保存前后对比"可以用字节级工具（qpdf --check）做 oracle，也让回归测试可以哈希比对。加密文档的闭环是"加载时解密、保存时以同密钥重加密、磁盘上永远保持密文"（cos 模块注释：decrypt on load, re-encrypt on save）。

### 2.3 引擎层闭环（crates/engine/src/lib.rs 模块注释与 Document 结构）

引擎把 cos 的能力包装成对前端可见的会话语义，四个关键分离：

1. **working file（工作文件）与 display bytes（显示字节）分离**。工作文件=保存时写出的字节；显示字节=工作文件 + "仅为显示而生成、永不落盘"的外观流（针对没有自带外观的注释）。保存、签名、校验读到的东西永远是工作文件，**显示层的临时加工不可能污染数据层**。
2. **编辑→增量写→刷新视图**的一致性回路：每次编辑成功后立即用增量写产出新的工作文件，视图从这个文件刷新——"你看到的就是保存将写出的"。保存成功后**重定基底（rebase）**：以刚写入的字节为新基准，下一次增量只包含新编辑。
3. **撤销/重做 = COW 快照栈**：编辑在克隆上执行，成功后前一状态入栈；克隆共享未修改数据，快照廉价；撤销在保存后依然有效。
4. **崩溃安全**：每文档维护一个单调**生成计数器**（编辑/撤销/重做/保存都会递增）；自动保存按分钟对比计数器决定是否产出**恢复快照**；快照中的加密文档保持加密；意外退出后下次启动提供恢复。写盘本身是**原子换名**（临时副本→swap），并用 qpdf 独立校验写出的文件（README "Saves you can trust"）。

**签名与全量写的冲突是 fail-closed 的**：引擎显式定义了"此文档已签名，全量重写会使签名失效（请增量保存）"的错误——需要重写的编辑（例如某些脱敏操作要求全量重写）在签名文档上直接失败，而不是静默破坏签名（engine 中 PAdES B-LT 路径注释为"永不重写已签名字节的增量更新"）。

### 2.4 "不丢数据"原则（crates/organize/src/lib.rs）

组织类操作（合并/拆分/旋转/删除页）的模块级承诺：**不理解的数据永不删除**。具体的结构策略：重构页树前，先把可继承页属性（Resources/MediaBox/CropBox/Rotate，§7.7.3.4）**物化到每个叶子页**，再把页树重建为单个平铺 /Pages 节点——中间节点在全量保存时回收，在增量保存时原样保留（反正不可达）。合并/提取时：链接与命名目标**重接线**到复制后的页；表单字段保持交互；图层默认态保留；附件跟随；并要求"组合文档的每一页与其来源**逐像素一致**"。

### 2.5 资源限额：把 DoS 防御做成一层横切面

数据完整性的另一半是"敌意文件不能吃掉内存与 CPU"。已固化的限额族（散布于 cos、filters、vendor 补丁）：

- 解码**总输出限额**约束每一个中间缓冲（包括 predictor 之前的缓冲），超限返回结构化错误而非 OOM；未过滤的原始流数据同样受按流限额约束（2026-10-09 的 ROADMAP log：empty filter arrays 与 Crypt 已应用处也纳入 raw 限额）；
- 任何图像/掩码超过 2^28 像素直接拒绝解码（防 `/W 4294967295` 类炸弹）；
- 嵌套绘制（form/Type3 字形/平铺模式互相嵌套）共享**每页 5 万次**的预算；
- 线宽、dash 切片数、平铺单元尺寸均有上限或钳制（详见 §3.6 的补丁清单）。

### 2.6 自研借鉴清单（数据完整性）

- 把"对象图即文档模型"定死为 ADR，能省掉绝大多数"领域模型 vs 文件格式"双轨同步的坑；你的引擎里不需要 PDF 之外的中间文档格式。
- COW + 廉价克隆是撤销、后台任务、崩溃恢复三者共用的**同一个**机制，不要做三套。
- 增量写保持"上一修订同风格 xref"这一细节值得抄：它让追加修订对第三方工具的兼容性最大化。
- "display 与 working file 分离"应从第一天就是不变量，事后分离代价极高。
- 确定性序列化（含字典键排序等稳定顺序）是让字节级 oracle 成为可能的前提。
- 限额做成结构化错误（可测试），不做全局内存监控。

---

## 3. 渲染管线（L3 render + bootstrap 策略）

### 3.1 总体策略：bootstrap 先行，公共 API 冻结（ADR-0004）

PdfCraft 当前的渲染是**双借用**的：位图栅格化走开源 crate `hayro`（其解释器 + CPU 后端 vello_cpu），结构检查（大纲/注释/字段/图层/附件）走 `lopdf`。关键的工程设计不是"借"，而是**借的方式**：

- render crate 对外暴露的公共 API（RenderPool/RenderedPage/Tile/inspect 系列等）就是引擎与 UI 依赖的全部表面，**替换实现被约束在这个 crate 内部**——上游换掉不影响任何调用方（render/src/lib.rs 模块注释原话："the swap stays internal to this crate"）。
- 借来的代码不是黑盒依赖而是 **vendor + 补丁**：vendor 目录携带上游源码，每个补丁三点纪律——源码内标注、必须配一个回归测试、应争取上游合并且上游发布后移除。这使 bootstrap 阶段就能对敌意输入做深度加固（见 §3.6），而这些加固知识在未来自研渲染器里同样有效。
- 目标态（M2，已完成 15%）：以自研 `model`（L2 类型视图）+ `content`（L2 容错内容流解析）+ 自研字体引擎 + **DisplayList 设备抽象**替换 hayro；把目前散落在 organize/annot/forms 里的私有页树遍历器逐步收编进 model（model/README.md 明说这是 ADR-0004 的迁移方向）。

### 3.2 栅格化服务：RenderPool 的六个设计点（render/src/raster.rs）

```
请求队列 ──> RenderPool（N 个 worker 线程）
              每 worker：私有解析器 + 私有渲染缓存
              每请求：携带 caller tag（回传比对，过期即丢弃）
              每页渲染：catch_unwind 包裹
              panic 后：丢弃该 worker 的解析器与缓存并重建，worker 存活
```

1. **每 worker 私有解析器与缓存**：共享字节、不共享解析状态——避免锁，且一个 worker 的状态污染不扩散。
2. **tag 化的过期丢弃**：请求带上调用者标签原样回传；UI 在缩放/翻页后直接丢弃旧 tag 的结果，渲染端不必取消已在途的计算。
3. **配合取消位**：vendor 补丁给解释器加了 cancelled 设置——渲染池被 drop（编辑发生、文档关闭）后，在途解释在下一个操作符前停下，不做"没人要答案的完整渲染"。
4. **catch_unwind 页级隔离**：单页 panic 产出一个带 error 的 RenderedPage（页面上显示错误而非崩程序），worker 重建后继续渲染后续页——"一页坏文件不能带走整个应用"。
5. **tile 化大页**：大页面分块渲染（补丁提供瓦片偏移参数），深层缩放仍锐利；测试断言"瓦片拼接 == 整页渲染"（tiles_match_full_render）。
6. **像素预算**：MAX_PIXELS/MAX_SIDE 全局上限，防超大页+超大缩放的组合爆炸。

### 3.3 文本提取：Device 抽象的双用途（render/src/text.rs）

文本提取不是独立解析器，而是解释器的**第二个设备实现**：忽略一切绘制，只记录每个字形的 Unicode 与视图空间包围盒。这个设计把"渲染"与"提取"统一在同一条内容流解释管线上：

- Unicode 回退链：ToUnicode → 字体编码 → 字形名推断（上游处理）；
- 坐标统一在**页视图空间**（原点=显示页左上、y 向下、/Rotate 已应用）——UI 只用一次缩放就能映射到屏幕，搜索/选择/复制共用同一坐标系；
- 之上再建：基线聚类的行分组 → 词间隙切词 → 阅读顺序输出（多栏、RTL、CJK 均在语义内）→ 不区分大小写的搜索与选择；
- **验收口径是量化 oracle**：对 pdftotext 的 word-F1 达 0.98（ROADMAP M2 条目）——"提取好了"在本项目里有数字定义；
- 垂直排版（WMode 1）按字形空间负 y 处理，使竖排日文列可搜索（vendor 清单备注，#126）。

规划中的独立 `text` crate（L3 槽位已预留）将承接这套能力，脱离 hayro 的字形枚举。

### 3.4 自研渲染器的目标态拼图

已在分层表里预留、尚未完工的自研部件：`model`（页树/继承属性/view_matrix——显示空间到用户空间的矩阵，含 /Rotate 语义）、`content`（容错内容流解析：操作数+源 span，坏字节跳过并计数、绝不 fatal，且**可回写**）、`color`/`ops`（L2 预留）、DisplayList 设备族（ADR-0004 的核心：解释一次，多设备消费——栅格设备、文本设备、未来的结构设备）。这等于把 hayro 的"解释器+设备"两层拆开：解释器吃自研 model/content 的输出，设备可插拔。

### 3.5 字体体系：不内嵌、不购买、Type 3 回退（crates/fonts/README.md）

这是对自研最"值钱"的一节，因为它展示了在**零字体资产**约束下如何获得可用的文本输出：

- 生成的注释/表单外观不需要字体程序：Helvetica 度量用"按字符类别的自有近似"做换行与对齐（明确声明不用于排版级排版）；文本用 WinAnsi 编码写进 PDF 字面字符串；
- **CJK 写入 PDF 的路径是 Type 3 字体**：从 craft-fonts（OFL 许可）的字面提取轮廓路径与步进宽度，逐字形生成 Type 3 字形程序，配 ToUnicode 映射——**不嵌入任何字体文件**，保存重开后文本提取/复制/搜索仍然完整；FontDescriptor 记录来源字面族与风格供辨认；
- **阿拉伯文经 harfrust 整形**（连写形式、合字、标记定位成"簇"），每簇一个 Type 3 字形 + ToUnicode 条目；行布局按 Unicode 双向算法排显示序，RTL 段落右对齐收尾；
- 字体缺失时**显式失败**（返回"无字体"错误、什么都不写），而不是静默输出问号——除了只读场景保留旧文档的 `?` 行为。

### 3.6 渲染长尾：一份真实的敌意输入清单（vendor/README.md）

bootstrap 期间积累的 vendor 补丁表，等于把"PDF 渲染器为什么难"逐条记录在案。每条都有：故障模式 → 加固策略 → 回归测试名。择要归类：

| 类别 | 具体故障（真实触发） | 加固 |
|---|---|---|
| 自引用结构 | 平铺模式引用自身、Type3 字形扇出 8×16 层 → 栈溢出/挂死 | 嵌套预算（每页 5 万）+ 深度上限 |
| 宽度炸弹 | CID `/W` 范围 `0–4294967295` 挂起；简字 `/Widths` 数组 34 GB 分配 | 范围钳制到 MAX_CID；宽度数组按码位 0–255 分配、非数值项跳过 |
| 像素炸弹 | `/W 4294967295` 模板掩码 → 4 GiB；JBIG2 每边 65535 → 43 亿像素逐个解码 | 2^28 像素硬上限，超限拒绝解码 |
| 路径爆炸 | `/LW 9e18` 线宽 → 10 GB 描边扩展；dash 模式把一条线切成数十亿段 | 线宽钳制 4×画布周长；dash 切片数超 25 万画实线 |
| 算法复杂度 | 内联图像数据含 "EI" → 50 张图超十分钟；错误 `/Length` 扫描到后续对象的 stream | EI 前瞻限 256 候选；Length 恢复仅当 stream 是下一 token |
| 结构环 | 对象流内 /Kids 循环 → 进程 abort | resolve_pages 访问集+深度帽 |
| 规范歧义修复 | `re W* S` 的裁剪在 S 后生效（ISO §8.5.4），旧实现只在 n 上生效 → 裁剪失效 | 按规范改为任意路径绘制操作符结束路径时生效 |
| 外观退化 | 无外观的高亮注释完全不画；`/BBox` 退化为线/点的外观产生无穷矩阵 | 按 /QuadPoints 自绘；退化外观跳过 |

**方法论价值**：这些不是假想的威胁模型，是 9 天内夜跑 fuzz + 真实语料实际炸出来的。你的引擎也应把"每个 fuzz 发现 → 补丁 + 命名回归测试"作为固定仪式（他们连"上游未修"的 bug 都带着测试先扛下来，且补丁会持续提给上游）。

### 3.7 自研借鉴清单（渲染）

- **bootstrap 但冻结 API**：先用现成渲染器把查看器层全部建起来（能看见进度），同时把自研替换面收敛在一个 crate 内；
- 解释器/设备分离是复用关键：栅格、文本提取、（未来的）结构输出共享一次解释；
- 坐标系尽早统一到"显示空间"（含 /Rotate），所有上层功能（选择、搜索、注释摆放）只做一次映射；
- 字体先走 Type 3 路线拿到"能写入 CJK 且可提取"的闭环，真字体引擎（子集化、回流）放在后期里程碑；
- 像素/嵌套/宽度的限额族从第一天设置，且每条限额都要有对应的合成回归文件。

---

## 4. 格式转换层

PdfCraft 的"转换"不是单一模块，而是四条各有独立净室依据的通道。共同特征：**都以 COS 对象图为源点或汇点，中间表示极小化**。

### 4.1 流过滤器：L0、可独立发布、行为表精确到容错语义（crates/filters/README.md）

过滤器族覆盖 Flate/LZW/ASCIIHex/ASCII85/RunLength 的完整**双向**（decode+encode）实现，加上 TIFF(2)/PNG(10–15) 两族 predictor 的编码与解码；DCT/JPX/JBIG2/CCITT/Crypt/未知过滤器统一返回 Unsupported——**调用者保留原始编码数据**（策略：宁可不解码，不可丢数据）。

三个值得抄的行为设计：

1. **decode_tolerant 语义**：坏流不整体失败，而是"返回已解码前缀 + partial 标志"，且**链条继续**（后续过滤器照常处理部分输出，predictor 也应用于部分数据）。渲染一个 99% 完好的损坏页成为可能。
2. **容错规则表逐条定义**（README 的 Behaviour 表）：Flate 容忍坏 Adler-32、无 zlib 头退化为原始 deflate、忽略尾随字节；LZW 容忍缺 EOD/初始 clear、EarlyChange 0/1 都支持；ASCII85 容忍 `<~` 前缀缺失、`z` 零组、多种 EOD 形态。每一条都是"规范怎么说 + 现实文件怎么坏"的交叉产物，且**写进了 README 当契约**。
3. **验证是三层的**：手算向量单元测试（含 ISO 32000-2 附录的官方 LZW 例子）；对 weezl 的双向交叉验证（LZW 双方向 × 两种 early-change）；属性测试做 encode→decode 往返 × 过滤器 × predictor × 颜色数 × 位深 × 列数的组合空间，再加随机字节/随机参数/翻转有效流的 no-panic 测试。另有独立 cargo-fuzz 目录做夜跑。

净室依据链完整：ISO 32000-2 §7.4、RFC 1950/1951（经 flate2 的纯 Rust 后端）、TIFF 6.0 §14、RFC 2083 §6。

### 4.2 注释与表单数据交换：XFDF/FDF/XML/CSV/tab（crates/xfdf/src/lib.rs，L4）

- 每种格式有独立规范出处：XFDF=ISO 19444-1、FDF=ISO 32000-2 §12.7.8、XML/CSV/tab=Acrobat 的"导出数据格式"**行为**（这里承认了从 Acrobat 的公开功能面推导，而非其代码）；
- **导入幂等语义**：注释以其 /NM（名称）为键，已存在则替换——重复导入不产生重复注释；字段值导入**经过表单自身的校验链**（格式/验证/计算，JS 引擎参与重算）——导入不是盲写；
- 导出覆盖注释的几何/颜色/标志/作者/日期/回复/弹出窗/墨迹/四边形/线端等完整字段族与嵌套字段值。

### 4.3 文档级导出：极小中间表示 + 结构启发（crates/export/src/lib.rs，L4）

引擎先把每页降为 `Page = 段落块 + 图像块`：段落含文本、包围盒、字号、粗斜体；图像含编码字节与包围盒；按阅读顺序排列。写出国（Word/HTML/RTF）从这一个小结构生成流式文档：**标题=字号大于正文的段落**（启发式显式声明，不假装是文档结构分析）。这个设计的要害是：PDF→办公格式的全部语义损失被**集中在一处中间表示**，各导出国不重复解析 PDF，新增导出格式只写一个"中间表示→目标格式"的单向转换。

### 4.4 页图像与创建（create，L4；engine 的 convert_to_pdf 入口）

- 图像导出支持 PNG/JPEG/TIFF，含"导出文档全部图像"（对页资源枚举解码）；图像导入（创建 PDF）支持 PNG/JPEG/TIFF(多页)/GIF/BMP，DPI 策略三选：内嵌分辨率 / 72 DPI / 自定义（1–1200，不重采样）；
- 文件类型识别收敛在 create 的 source_kind，多文件创建共享引擎单一入口，Open/Insert/Create/转换工具全部复用同一条路径（2026-10-08 log）；
- 页尺寸合法性以 ISO 32000-2 附录 C 的 14400 单位上限机器校验。

### 4.5 打印：先造"打印就绪 PDF"再交给系统（crates/print/src/lib.rs，L4）

impose（拼版）把要打印的页排成**新的、未加密的、全量 GC 过的 PDF**：一页一纸/适/实际大小/缩放居中、n-up 网格（2/4/6/9/16/自定义、四种页序、页框、自动旋转）、小册子（骑马订、正反面、左右订）、海报（跨页放大+重叠+裁切标记）、单面裁切堆叠。之后 spool 交给系统打印（macOS/Linux 走 CUPS）。设计要害：**打印是 PDF→PDF 的转换**，光栅化留给系统打印栈，引擎不碰打印机驱动——跨平台差异被压缩到 spool 一个点。

### 4.6 OCR：外部引擎 + 不可见文本层（crates/ocr/src/lib.rs，L4）

识别用 ocrs（MIT/Apache，模型 CC-BY-SA-4.0 由 xtask 拉取，ATTRIBUTION.toml 记账）；调用方渲染页为像素，识别出词，text_layer 把词按用户空间位置写成**渲染模式 3 的不可见文本**覆盖在原词上——精确复刻 Acrobat "Searchable Image (Exact)" 的语义：**原图一个像素都不动**。拉丁字母范围，其余语言在路线图上。净室视角值得注意：OCR 整条链没有 Adobe 参与，语义等价物是公开可描述的功能行为。

### 4.7 优化：按"使用处"测量而非按对象（crates/optimize/src/lib.rs，L4）

图像有效分辨率=**绘制处的最小分辨率**（穿过 form XObjects 的所有用法取最小），超过阈值的彩色/灰度图做 bicubic 重采样到目标分辨率并以 JPEG/Flate 重压缩——**新对象只在更小时替换旧对象**。不能忠实解码的（CMYK 等色彩空间、掩码、decode 数组、JP2/JBIG2/CCITT、>8bpc）一律不动。可丢弃对象（缩略图/交替图像/结构树/打印设置）显式列举；Flate 清理（无过滤器的流压缩之）与资源合并/去重配套。这个"有效分辨率"定义是 Acrobat Reduce File Size 行为的公开化描述，且**每条保守条件都写成了文档**。

### 4.8 内容编辑的持久化设计：marked-content 标记（crates/edit/src/lib.rs，L4）

页眉页脚/水印/背景/Bates 编号的存储方案是纯 PDF 手法，值得细看：每个标记是**追加到页 /Contents 的独立内容流**（背景类则前插），绘制在显示空间（/Rotate 后），并用**marked-content artifact** 包裹——type=Pagination、subtype 按类，附带私有 /PCMark 条目（流字典与流内均有）。效果：①阅读器与屏幕阅读器按规范跳过 artifact，不把它当页面内容；②更新/删除标记**无需解码页内容**，按 /PCMark 找回即可；③原始内容被 q…Q 包裹（两个微型流），图形状态互不泄漏；④文字用标准 Helvetica（/PCHelv）加进页资源，不需要字体资产。**用规范内建机制做自描述标记**，是"在别人的文档里安全地加东西"的标准答案。

### 4.9 脱敏（redact，L4）与转换安全性

脱敏是"最危险的格式转换"（删除必须彻底），其设计全部围绕 fail-closed：标记区域下的字形（同行其余字符保位）、内联图像、路径全删；图像/矢量部分覆盖则**只删覆盖部分**；form XObject **重写为新对象**（共享它的其他页不受影响）；重叠的注释/链接/字段一并删除；最后**验证遍**重读每个脱敏页，任何残留字形/内联图像使整个操作失败（调用方保留原文档）。保存要求全量重写（所以与签名文档冲突，见 §2.3）。

### 4.10 自研借鉴清单（转换）

- 过滤器 crate 做成 L0 可独立发布 + 行为表当公开契约 + 三层测试，是所有"规范驱动"模块的模板；
- 一切转换围绕 COS 对象图进出，中间表示压到最小（导出的 Page、文本的字形列表）；
- "解码不了就原样保留"优先于"尽力而为的转换"——数据不丢优先于功能完整；
- 删除类转换（脱敏）必须配验证遍与 fail-closed；
- 用规范的 marked-content 机制存自产数据，永远可被第三方工具理解/跳过。

---

## 5. 质量与验证体系：oracle 密度决定可信度

PdfCraft 的每个"已实现"声明背后挂着的不是自圆其说的测试，而是**外部工具与公开语料构成的 oracle 网**：

| Oracle | 覆盖面 | 口径 |
|---|---|---|
| pdf.js 测试语料（983 文件，CI 拉取到 corpus/） | 打开/编辑/保存稳健性 | 958 个文件通过 open/edit/save 往返，0 崩溃；7 个加密文件全部可开 |
| qpdf | 保存正确性 | 每次保存后的文件独立校验（结构完整性） |
| pdftotext | 文本提取质量 | word-F1 0.98 |
| pdfsig / OpenSSL | 签名编解码 | PAdES 签名/验证往返 |
| Poppler | 保存重开渲染 | 注释外观在保存重开后渲染一致（含四向旋转） |
| weezl | LZW 编解码 | 双方向 × early-change 0/1 交叉验证 |
| cargo-fuzz（夜跑） | 敌意输入 | 每个发现转为合成回归测试（vendor 表即积累） |
| cargo xtask check | 回归基线 | 对语料跑 CLI 稳健性扫掠，与 xtask/baselines 比对 |

两点工程文化值得单独记录：

1. **诚实会计**。ROADMAP 的"honest assessment"主动暴露：398 个"已发货"功能里 159 个只有一条测试、仅约 5 个引用外部 oracle；"shipped ≠ as good as Acrobat"；fuzz 曾连出 7 个 OOM + 11 个挂起。每个会话在 log 里一行记录"动了什么 + 总体百分比是否变化"。**平价数字由 xtask 对 TOML 化的功能清单自动计量**（800+ 项），不靠感觉。
2. **oracle 位置**。oracle 全部挂在 xtask/CI 层而不是散在 crate 里（corpus 拉取、baselines、text-oracle、robustness sweep 都是 xtask 子命令），第三方工具依赖被隔离在构建工具层，crate 本体保持零外部工具依赖。

---

## 6. 施工顺序映射：把他们的 M0–M14 翻译成你的路线

他们的里程碑依赖关系（ROADMAP"Critical path"）是：**M0 骨架 → M1 COS → M2 渲染 → M3 查看器 → M4 引擎** 严格串行；M4 之后 M5–M12（注释/表单/编辑/安全/签名/OCR/优化/无障碍）可按 crate 并行。

| 阶段 | 内容 | 验收口径（借用他们的） |
|---|---|---|
| M0 | 工作区 + 分层表 + xtask 门 | layers/wasm/deny/ci 全绿；分层表先于 crate 存在 |
| M1 | filters + crypt + cos（解析/修复/xref/写出） | 语料 open/edit/save 0 崩溃；增量+全量写出过 qpdf；加密文件可开 |
| M2 | model + content + 渲染 bootstrap | 文本提取 F1 对 pdftotext ≥0.98；瓦片==整页 |
| M3 | 查看器（先看后改） | 大文档滚动流畅；坏页显示错误不崩程序 |
| M4 | engine：命令注册/撤销/保存/组织 | 编辑在克隆上跑；自动保存+崩溃恢复；签名文档仅增量 |
| 之后 | 注释→表单→页面标记→脱敏→签名→OCR→导出→优化→preflight | 按你的目标用户裁剪；每个功能"存在+至少一条具体测试" |

**对单人自研的裁剪建议**（结合其风险表——文本编辑保真、XFA 动态布局、签名链验证、渲染长尾是四大吞时怪兽）：

- 若目标是"看/搜/提取/转换"引擎：M0–M4 + 文本导出即可成器，恰好绕开全部四大怪兽；
- 渲染先用借来的（hayro 是 Apache/MIT，合法），把精力花在 cos 与 content 的质量上——这是他们 M2 只完成 15% 却先发产品的底气；
- CJK 写入走 Type 3 + ToUnicode 的捷径，把真字体子集化推迟到有真实需求时；
- 从第一天起跑夜 fuzz；每个崩溃配命名回归文件，这份"坏文件博物馆"就是你引擎的免疫系统。

---

## 附录 A：crate 速查表（30 个已实现 crate）

| 层 | crate | 一句话职责（依模块注释） |
|---|---|---|
| L0 | geom | 共享几何原语（PDF 用户空间 y 向上，点单位） |
| L0 | filters | 全部流过滤器的编解码 + predictor，属性测试 |
| L0 | crypt | 标准安全处理器 R2–R6（RC4→AES-256），SASLprep，可建加密字典；不知 COS 为何物 |
| L1 | cos | COS 对象图：容错惰性解析/修复/COW 编辑/增量与全量写出（ADR-0010） |
| L2 | model | 对象图之上的类型视图：页树/继承属性/view_matrix |
| L2 | content | 内容流容错解析（操作数+span，可回写），矩阵算术 |
| L2 | fonts | 生成外观所需：Helvetica 近似度量/WinAnsi/Type 3 回退/OFL 字面管理 |
| L3 | render | 栅格化（RenderPool/tile）+ 文档检查 + 文本提取（bootstrap：hayro+lopdf） |
| L3 | annot | 注释：构建器与外观流（便笺/标记/形状/墨迹/文本框）、回复/状态 |
| L3 | forms | 交互表单：字段模型、填充时**再生成外观** |
| L3 | js | 沙箱化的 Acrobat 表单 JavaScript（boa） |
| L3 | xfa | XFA 表单模型与布局 |
| L4 | organize | 页操作/合并/拆分/书签/页标签；不丢数据原则 |
| L4 | measure | 2D 测量注释（距离/周长/面积） |
| L4 | redact | 脱敏：按标记删除 + 验证遍 fail-closed |
| L4 | edit | 页面标记（页眉页脚/水印/背景/Bates）+ 可编辑的添加内容 |
| L4 | print | 拼版成打印就绪 PDF + CUPS spool |
| L4 | xfdf | XFDF/FDF/XML/CSV/tab 数据交换，幂等导入 |
| L4 | ocr | ocrs 识别 + 不可见文本层 |
| L4 | export | Word/HTML/RTF 导出（极小中间表示） |
| L4 | create | 从空白/图像/文本创建 PDF，source_kind 识别 |
| L4 | optimize | 按使用处测分辨率的重采样/重压缩/丢弃/清理 |
| L4 | sign | DER/X.509/CMS/PKCS#12、PAdES B-B 签名与验证 |
| L4 | a11y | 无障碍检查器（32 规则） |
| L4 | compare | 文本差异比较 |
| L4 | preflight | PDF/A 标准符合 |
| L6 | engine | 门面：会话/编辑历史/工具目录；frontends 唯一入口 |
| L7 | platform | 平台胶水 |
| L7 | ui-egui | 桌面与 Web 界面 |
| L7 | automation | 无头工具表/CLI run/MCP（本文按约定从略） |

## 附录 B：按序精读地图（文档头注释是最佳入口）

1. `xtask/src/layers.rs` —— 分层表全文（架构的立法文件）
2. `ROADMAP.md` —— 里程碑/诚实评估/每会话 log（方法论样本）
3. `crates/filters/README.md` —— 规范驱动模块的完整样板（依据链+行为表+三层测试）
4. `crates/cos/src/{lib,document,parser,writer}.rs` 头注释 —— 数据完整性全部语义
5. `crates/engine/src/lib.rs` 头注释 + Document 结构区 —— 会话/工作文件/rebase/恢复
6. `crates/render/src/{raster,text}.rs` 头注释 —— RenderPool 与 Device 抽象
7. `crates/{model,content,fonts}/README.md` —— 自研渲染器与字体的规划拼图
8. `vendor/README.md` —— 敌意输入加固全表（17+6+3+5+3 条补丁与测试名）
9. `crates/{organize,edit,redact,export,print,ocr,optimize,xfdf}/src/lib.rs` 头注释 —— 转换层各通道语义

## 附录 C：本报告引用的规范与外部依赖

- **规范**：ISO 32000-2:2020（PDF 2.0，主规范：§7.2–7.5 词法/对象/xref、§7.4 过滤器、§7.6 加密、§7.7.3.4 页树继承、§7.8.2/§8/§9 内容流与文本、§12.5.6.23 脱敏注释、§12.7.8 FDF、§14.8.2.2 marked-content、附录 C 尺寸上限）；ISO 19444-1（XFDF）；RFC 1950/1951（zlib/deflate）；TIFF 6.0 §14（predictor 2）；RFC 2083 §6（PNG predictor）
- **被借用/对照的开源件**：hayro 家族（Apache/MIT，渲染 bootstrap，已 vendor 打补丁）、lopdf（MIT，检查 bootstrap，已 vendor）、qpdf/pdftotext/pdfsig/Poppler（oracle）、weezl（LZW 交叉验证）、flate2/miniz_oxide（纯 Rust inflate）、ocrs+rten（OCR）、harfrust（阿拉伯整形）、boa（JS 沙箱）、vello_cpu（CPU 光栅后端）
- **字体资产**：craft-fonts（OFL 1.1：BIZ UDPGothic/UDMincho、Shippori Mincho、Noto Sans Telugu 等）

> **结语**：这份架构最值得带走的不是任何单个机制，而是三件事的互相咬合——**一张机器执行的分层表**（结构纪律）、**一张 oracle 网**（事实纪律）、**一条"原始字节不动"的保存哲学**（数据纪律）。数据完整性靠 COW+增量写闭环；渲染靠 bootstrap 冻结 API 换时间；转换靠"规范驱动 + 解不了就保留"的保守性。自研引擎按此骨架生长，其余是填空。

