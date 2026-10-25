# R-O2 回归套件（r-o2-t3）：DOCX → LibreOffice headless → PDF 渲染管线回归报告

> 任务口径：把 r-o2-t1/t2 的 spike 证据（lo_convert_spike.py 样本 A/B/C +
> lo_convert_spike2.py 样本 D–I）固化为**确定性、CI 可跑的回归套件**——
> 9 有效夹具用例（硬断言）+ 5 畸形输入用例（行为钉定），冷/暖双次转换、
> 逐断言 expected/actual 记录、逐转换墙钟 + 峰值 RSS、JSON 全量报告。
> 两个原 spike 脚本**未改动、复跑仍全过**（SPIKE PASS / SPIKE2 PASS，
> 2026-10-17 复验）。

## 0. 一句话结论

`python3 regress.py` 一条命令 16 秒跑完 14 用例：**9 PASS + 5 PINNED +
0 FAIL（exit 0）**；夹具字节级确定性门（sha256 台账）同工作区复验通过；
超时治理机制验证通过（杀整进程组、零 soffice 残留、套件非零退出、无挂起）。

## 1. 环境（全部本机实测）

| 项 | 值 |
|---|---|
| soffice | `/usr/bin/soffice` → LibreOffice **25.2.3.2** 520(Build:2) |
| poppler | pdftotext / pdfinfo / pdfimages / pdffonts / pdftohtml 全部 **25.03.0**（pdftohtml 自 r-o2-t5 起为链接检视面，§10） |
| python | 3.12.15（**stdlib only**，零第三方依赖——r-o2-t5 维持：链接注记检视走 poppler pdftohtml，不引入 PyMuPDF/pypdf） |
| `/usr/bin/time` | **不存在**（已核实）→ 峰值 RSS 用替代口径（§5.2） |
| 内核 | Linux；/proc/*/comm 可读（soffice 残留检测用） |

**LO 版本差警示（维持 office-f §5.2 登记）**：本机 25.2.3.2 vs 上游
26.8.1。全部断言（含钉定）须在上游版本复跑；钉定破裂 = 版本差信号
（复核后更新钉定），不是套件缺陷。

## 2. 用例矩阵（最终全量运行实测：`python3 regress.py`，2026-10-17）

结果口径：**冷** = 全新 `-env:UserInstallation` profile（含 LO 首启建档），
**暖** = 同 profile 复用第二次转换；断言一律在暖跑 PDF 上评估（冷跑页数
另记录，冷/暖一致性 14/14 一致）。墙钟 = rss_wrapper.py 内测得的
soffice 专属墙钟（排除 wrapper 开销）。RSS = 该次转换子进程峰值
（kB→MB 换算，口径见 §5.2），格式 冷/暖。

| 用例 | 类别 | 断言面（expected） | actual | 墙钟 冷/暖 | RSS 冷/暖 | 结果 |
|---|---|---|---|---|---|---|
| basic-structure | valid | 页数=2；7 关键串；表在页 1、分页后段落在页 2 | 全过 | 0.57s / 0.27s | 221/186 MB | PASS |
| explicit-page-breaks | valid | 页数恰 6（硬断言）；「第 N 页内容」逐页定位（6/6） | 全过 | 0.57s / 0.26s | 221/182 MB | PASS |
| table-spanning-pages | valid | 页数≥2；行001-甲在页 1；行080-丙在页 2≠页 1；表头「列一」在场且在末行页重复（tblHeader）；行040-乙在场 | 全过（2 页） | 0.60s / 0.30s | 221/189 MB | PASS |
| headers-footers | valid | 页数恰 3；页眉+页脚锚串逐页（3/3）在场 | 全过 | 0.57s / 0.26s | 221/183 MB | PASS |
| section-boundaries | valid | 页数恰 2；页 1 = 595×842pt、页 2 = 842×595pt（pdfinfo -f/-l）；两节文本各自落位 | 全过（精确匹配） | 0.56s / 0.26s | 221/183 MB | PASS |
| missing-font | valid | rc=0；缺失字体行文本在场；pdffonts 无 TotallyMissingFont123；替换字体嵌入≥1 | 全过（NotoSerifSC/Carlito） | 0.56s / 0.26s | 221/183 MB | PASS |
| embedded-image | valid | pdfimages ≥1；图片前后文本在场 | 全过（1 张） | 0.57s / 0.28s | 220/188 MB | PASS |
| composite | valid | 页数≥4；页眉逐页（4/4）；页 1=595×842pt 且横向页 842×595pt 在场；5 关键串；图片≥1 | 全过（4 页） | 0.61s / 0.31s | 220/191 MB | PASS |
| real-fixture | valid | sha256 与已提交副本一致；页数恰 1；8 关键串；pdfimages ≥1 | 全过（2 对象：image+smask） | 0.58s / 0.30s | 221/189 MB | PASS |
| garbage-bytes | malformed-pinned | 钉定：rc=0（冷/暖）+ 乱码 PDF 产出（文本导入兜底，§3.1 新发现）；文本内容仅记录不断言 | 全过 | 1.21s / 0.85s | 255/254 MB | PINNED |
| truncated-zip | malformed-pinned | 钉定：PDF 不产出（冷/暖）+ 输出含 load 失败消息；rc 记为观察值（实测 0，§3.2 新发现） | 全过 | 0.54s / 0.24s | 221/179 MB | PINNED |
| bad-xml | malformed-pinned | 同 truncated-zip | 全过 | 0.54s / 0.25s | 220/172 MB | PINNED |
| missing-content-types | malformed-pinned | 同 truncated-zip | 全过 | 0.54s / 0.23s | 221/179 MB | PINNED |
| drawing-at-paragraph-level | malformed-pinned | 钉定：rc=0 + PDF 产出 + 文本完好 + **pdfimages 计数恰 0**（静默丢弃，r-o2-t2 登记发现①） | 全过（0 张） | 0.56s / 0.27s | 221/184 MB | PINNED |

**汇总：14 用例 = 9 pass + 5 pinned + 0 fail；断言总数 131，全部通过；
exit 0。** 全套件墙钟约 16s（含每用例冷+暖两次转换）。稳定性：连续
3 次全量运行绿（out/regression-report*.json 三份在档），跨运行核对
result/页数/图片数/逐页尺寸全部一致（无 flake）；夹具 sha256 台账跨
5+ 次运行字节级一致。

逐断言 expected/actual、每次转换的 stdout/stderr 摘要、rc、PDF 字节数、
冷/暖页数一致性、归一化文本摘录：全部在 `out/regression-report.json`
（`--json-out` 可改路径；out/ 已 gitignore）。

## 3. 畸形输入行为发现（本轮新证据，供 SECURITY_POLICY/实施轮登记）

三条已登记发现（office-f §8.2）——w:drawing 必须 run 级（静默丢弃）、
pdftotext CJK/拉丁边界插空格、真件对照判别法——之外，本轮实测新增：

### 3.1 新发现④：垃圾字节 → LO 文本导入兜底，rc=0 + 乱码 PDF「成功」产出

4096B 确定性伪随机字节（非 zip、无 PK 头）：LibreOffice **不拒绝**，
按 Writer 文本文档打开（stdout 出现 `convert ... as a Writer document`），
rc=0，产出 1 页 PDF，内文为乱码 CJK/私有区字形。敌意输入面含义：
**"转换成功"（rc=0 + PDF 在场）完全不能证明输入是合法 DOCX**——消毒
五面前置（office-f §5.3 口径）因此更必要；本行为已钉定为回归锚。

### 3.2 新发现⑤：加载失败时退出码仍为 0 —— 退出码不是失败信号

truncated-zip / bad-xml / missing-content-types 三类：rc=**0**、PDF 不产出、
失败信息只在 **stderr**（`Error: source file could not be loaded`）。
即 LO 25.2.3.2 的 `--convert-to` 在源文件无法加载时**不返回非零**。
检出策略（本轮实证并在套件中执行）：**PDF 产出核验 + 输出消息扫描**，
rc 仅作观察值记录（版本差敏感，不钉死）。这把登记发现①（静默丢弃）的
教训推广为通用口径：**对 LO 子进程，一切以转换后产物断言为准**。

### 3.3 命名勘误（登记）：section-boundaries 节 2 尺寸 = A4 旋转对，非 A5

spike2 措辞「节 2 A5 横向」沿用误差：夹具实际 pgSz 16838×11906 twip
（842×595pt）= **A4 横向**尺寸（真 A5 横向应为 595×420pt）。本套件按
实测 pt 精确断言（595×842 → 842×595），夹具尺寸与断言自洽；措辞勘误
在此登记，不改前序报告文本（office-f §8.1 F 行的 pt 断言本身正确）。

### 3.4 行为钉定语义（用例类别 malformed-pinned）

通过 = 钉定行为（LO 25.2.3.2 实测）被确认。**钉定破裂 = LO 行为变化**
（典型场景：上游 26.8.1 复跑）→ 用例 FAIL 并在 JSON 报告标
`pin_broken`，处置 = 人工复核后更新钉定（不是套件 bug）。drawing-at-
paragraph-level 用例同时是**检出策略证明**：敌意/畸形输入的要素丢失只能
靠转换后要素计数断言（pdfimages/pdffonts/文本串）抓到——rc 与「转换
成功」都不可信。

### 3.5 环境常量噪声（登记）

每次转换 stderr 都带 `Warning: failed to launch javaldx - java may not
function correctly`（本机 Java 环境缺失，与转换成败无关）——stderr 摘要
按原样记录，不作失败判据。

## 4. 超时治理验证（实测证据）

机制：每次转换经独立 rss_wrapper.py 进程（`start_new_session=True`，
wrapper 为进程组组长，soffice 启动脚本 exec 为 soffice.bin 同组）；
`proc.communicate(timeout=T)` 超时 → `os.killpg(SIGKILL)` 杀整组 →
`/proc/*/comm` 扫描核验零 soffice 残留（扣除套件启动前基线 PID）→
用例记 FAIL（`no_timeout_*` 断言）→ 套件继续跑完 → **exit 非零、无挂起**。

验证一（确定性触发，推荐自检命令；冷转换实测最短 0.53s，0.3s 必触发）：

```
$ python3 regress.py --timeout 0.3 --filter basic-structure --json-out out/timeout-verify-0.3.json
[ 1/1] basic-structure  FAIL  冷 0.50s 暖 0.50s 峰值RSS n/a 页 -  (15 断言)
       ✗ no_timeout_cold / no_timeout_warm / exit_code_* / pdf_produced_warm / ...（依赖项连带失败，逐条记录）
       ⏱ cold 转换超时（0.3s）：kill=killed_pg，soffice 残留 PID []
       ⏱ warm 转换超时（0.3s）：kill=killed_pg，soffice 残留 PID []
汇总: 1 用例 = 0 pass + 0 pinned + 1 fail
SUITE FAIL        # EXIT=1
```

验证二（任务原示例 `--timeout 1`；本机冷转换 0.53–1.21s，1s 处于边界带，
在实测最慢用例 garbage-bytes（冷 1.21s）上触发）：

```
$ python3 regress.py --timeout 1 --filter garbage-bytes --json-out out/timeout-verify-1.json
[ 1/1] garbage-bytes  FAIL  冷 1.20s 暖 1.20s 峰值RSS n/a 页 -  (6 断言)
       ⏱ cold 转换超时（1s）：kill=killed_pg，soffice 残留 PID []
       ⏱ warm 转换超时（1s）：kill=killed_pg，soffice 残留 PID []
SUITE FAIL        # EXIT=1（JSON: cold/warm timeout=True, kill=killed_pg, leftover=[]）
```

套件前后 `/proc` 扫描 soffice 族进程均为空（杀干净，无孤儿 soffice.bin）。
默认超时 60s（`--timeout` 可调，接受小数）。

## 5. 指标口径（诚实登记）

### 5.1 冷/暖墙钟

- 冷 = 该用例全新 profile 目录（`/tmp/ro2-regress-profiles/<case>`，跑前
  rmtree）首次转换；暖 = 同目录立即复跑第二次。断言在暖跑产物上评估。
- 墙钟取 rss_wrapper.py 内 `subprocess.run(soffice)` 的实测值（排除
  wrapper python 启动开销）；超时记录的 wall_s 为父进程口径（转换未完成）。
- 实测规律（本机 25.2.3.2）：profile 复用使暖跑比冷跑快约 2 倍
  （0.54–0.61s → 0.23–0.31s；garbage-bytes 1.21→0.85s）。

### 5.2 峰值 RSS（/usr/bin/time 缺失的替代口径）

每次转换由**全新** rss_wrapper.py 子进程执行 soffice 并在等待结束后读
`getrusage(RUSAGE_CHILDREN).ru_maxrss`（Linux 单位 kB）——RUSAGE_CHILDREN
覆盖该 wrapper 已等待的全部子进程（soffice 启动脚本 exec 为 soffice.bin
即直接子进程；被其等待的后代亦计入 max）。因 wrapper 每次新建，数值为
**该次转换**的子进程峰值，而非跨用例累积高水位。局限（如实登记）：
不覆盖未被等待/后台存活的进程；被 SIGKILL 的超时转换不取值（n/a）。
实测：冷 ~220MB（profile 建档开销），暖 172–191MB；garbage-bytes 最高
255MB（文本导入兜底路径）。

### 5.3 夹具确定性（套件级门）

- 手搓夹具内容零时间戳/零随机；zip 条目统一 date_time=(1980,1,1,0,0,0)；
  garbage 字节用 sha256 计数器流（确定性伪随机）。
- 每次运行全量重建 14 件夹具并记 sha256 台账（`out/fixture-hashes.json`），
  与上次运行比对——不一致 = 确定性回归，套件 FAIL。本轮已跨 5+ 次运行
  复验字节级一致（同一平台/同一 python 口径；跨平台字节级一致不主张）。
- real-fixture 为二进制副本（sha256 = 7a42c91b54a50365… 与 mdcore
  web/tests/fixtures 原件一致），套件断言副本与已提交副本字节一致。

## 6. 诚实边界（不主张面）

1. **要素级覆盖 ≠ 通用 DOCX 保真**：通过只证明所列要素（分页/跨页表/
   页眉页脚/分节/字体替换/图片/真件）在该夹具形状下转换不失败且文本层、
   结构断言过——不是视觉级保真（字体替换视觉近似、表格度量、色彩管理、
   页眉页脚 first/even 槽位变体均不主张；office-f §8.3 口径维持）。
2. **poppler 工具仅作检视面**（页数/尺寸/文本/图片/字体计数），不是
   替代渲染器；渲染管线本体 = DOCX → LibreOffice headless → PDF。
3. **mdc-pdf 消费腿未覆盖**（Q3-gated 实施轮项）：PDF 文本层在
   mdc-pdf/pdfium 口径下的抽取质量差异（含 CJK 空格归一化口径）仍待
   实施轮实测——本套件的 pdftotext 归一化口径只是检视侧断言口径。
4. **小夹具 KB 级口径**（1.2–37KB）：大文档/并发转换延迟与资源面未覆盖
   （office-f §5.1 遗留维持，实施轮对齐 bench_limits 模式）。
5. **LO 25.2.3.2 本机实测**：上游 26.8.1 复跑为版本差核对前置；钉定
   破裂处置见 §3.4。
6. 套件假定运行期间无其他 soffice 进程（启动时记录基线 PID 并在残留
   检测中扣除；有并发 soffice 时残留判定可能失真——CI 独占容器无此问题）。

## 7. 复现命令（全部在 `office-line/spike-ro2/` 下）

```bash
python3 regress.py --list                 # 用例清单（14 件）
python3 regress.py                        # 全量：9 pass + 5 pinned + 0 fail，exit 0，~16s
python3 regress.py --filter table         # 子串过滤
python3 regress.py --timeout 0.3 --filter basic-structure   # 超时自检（预期 FAIL + exit 1）
python3 regress.py --json-out /tmp/r.json # 报告落点可改（默认 out/regression-report.json）
python3 lo_convert_spike.py               # 前序 spike 1（A/B/C）——未改动，复跑 PASS
python3 lo_convert_spike2.py              # 前序 spike 2（D–I）——未改动，复跑 PASS
```

依赖：`soffice`、`pdftotext`、`pdfinfo`、`pdfimages`、`pdffonts` 在 PATH；
python 3.12 stdlib only。产物目录 `out/`（gitignored）；LO profile 在
`/tmp/ro2-regress-profiles/`（可随时删）。

## 8. 文件清单（本任务新增，全部在 spike-ro2/ 内）

| 文件 | 角色 |
|---|---|
| `regress.py` | 套件主体：用例注册表 + 转换执行（超时/杀组/残留检测）+ 断言记录器 + JSON 报告 + CLI |
| `ro2_fixtures.py` | 确定性夹具构建器（9 有效 + 5 畸形；承两轮 spike 的抽取副本，参数化 + 固定 zip 时间戳） |
| `rss_wrapper.py` | 每转换独立子进程：rc/stdout/stderr/墙钟/子进程峰值 RSS 的 JSON 报告 |
| `fixtures/import_sample.docx` | 已提交真件副本（sha256 与 mdcore 原件一致；套件自包含） |
| `REGRESSION.md` | 本报告 |
| `out/`（gitignored） | 运行产物：夹具、冷/暖 PDF、JSON 报告、sha256 台账 |

退出码语义：**exit 0 ⟺ 全部用例 pass/pinned 且夹具确定性门通过**；
任一 fail（含钉定破裂、超时、**冷/暖页数不一致**）→ exit 1；
`--filter` 无匹配 → exit 2。

**e-review 收严（anchor-t3 同批）**：冷/暖页数一致性从「警告打印」
升级为**硬门**（非确定性转换 = 用例 FAIL）——非确定性 flake 不再
可能静默通过。

## 9. r-o2-t4 扩展轮（用户 Task 3c，2026-10-17）：转换失败 / 畸形补充 / 跨进程确定性

**用例 14 → 21**（9 有效硬断言不变 + 畸形钉定 5 → 9 + 转换失败/治理
钉定 0 → 3），总断言 131 → **188**。全部新形状**先探针观测（LO
25.2.3.2 实测）后钉定**——探针观测结论逐条入 `ro2_fixtures.py` 构建
器注释。

### 9.1 转换失败面（conv-fail-pinned，3 用例）

| 用例 | 形状 | 钉定行为 |
|---|---|---|
| `nonexistent-source` | 源文件不存在 | **rc=0 族的极端成员**：连文件都不在场也不返回非零；无 PDF + stderr `source file could not be loaded`——产物断言是唯一可靠检出 |
| `empty-source-file` | 0 字节 .docx | **PDF 产出**：以 Writer 文档装载（`as a Writer document`）→ 1 页零文本空 PDF——静默空产物族 |
| `pdf-renamed-docx` | 合法 PDF 字节 + .docx 扩展 | **内容嗅探改道**：以 **Draw** 文档装载（`as a Draw document`，draw_pdf_Export）→ 1 页——**扩展名不决定导入路径**；治理含义：文件类型纪律必须在调用 LO 之前由我方执行（office-f §8.4 检出策略通则的输入侧对偶） |

### 9.2 畸形补充面（malformed-pinned，+4 用例）

| 用例 | 形状 | 钉定行为 |
|---|---|---|
| `foreign-zip` | 合法 zip 但非 OOXML（hello.txt 单条目） | load 失败族（rc=0 无 PDF） |
| `missing-document-part` | 包骨架在（Content_Types + _rels）但缺 word/document.xml | load 失败族——与 missing-content-types 互补（缺主部件 vs 缺类型清单，分别钉定） |
| `wrong-root-element` | document.xml 根 = html | **静默空产物**：rc=0 + PDF 产出 + 1 页零文本——『转换成功 ≠ 内容合法』族第三成员（garbage-bytes 乱码 / wrong-root 空） |
| `declared-size-bomb` | zip LFH 声明解压 1GB / 实际 32B 截断 | **快速 load 失败**（实测 ~0.6s < 15s 门）——无分配放大、无悬挂；资源面回归锚 |

### 9.3 跨进程产物确定性门（套件级新门，8 断言）

同夹具（basic-structure）两次**全新 profile** 冷转换的逐面对照：

- **字节级不同一（钉定）**：PDF 元数据面差异（CreationDate 等）——
  跨进程转换的字节 sha256 天然不稳定；
- **内容级同一（硬断言）**：页数相等 + 归一化文本逐字节相等 + 字体
  清单相等 + 尺寸同量级（<1KB 差）。

**消费含义（mdc-pdf 腿 / Q3 输入）**：产物指纹、缓存、去重策略必须
走**内容级摘要**（页数 + 文本 + 要素计数），不能走字节级 sha256。
若未来 LO 版本变为字节确定 → `pin_bytes_differ` 破裂 = 版本差信号
（可升级为更强断言）。

### 9.4 台账语义精化（套件扩展的正确形态）

sha256 台账对比语义精化：**新增**夹具键 = 良性扩展（首跑基线落账，
不判失败）；**变更/移除**键 = 非确定性回归（维持硬门）。此前语义将
新增键也计为 mismatch——套件自身扩展（本轮 14 → 21）会误触门。

### 9.5 断言面工程注记

滤镜名断言（writer_pdf_Export / draw_pdf_Export）在长仓路径下超出
240B stdout 摘要窗——改用**位置稳健的文档类型短语**（`as a Writer
document` / `as a Draw document`，出现于源路径后、摘要窗内）；滤镜
名在探针观测结论中登记，不作为断言依赖。

## 10. r-o2-t5 锚点存活轮（批次 2 联动项，2026-10-17）：office-g §13 联动件

> 触发链：用户裁 DECISION_SHEET 批次 2（Option A Phase 1+2 全批
> 合入 mdcore main c4856f7）→ office-g §13 登记的联动项「LO 侧
> 回归套件可同批扩锚点存活用例」即本节。用例 +1（21→22），断言
> +16（188→204）；全部断言先探针观测后钉定（临时探针跑完即删）。

### 10.1 用例 anchor-survival（kind: valid，16 断言）

夹具形状**精确镜像** mdcore c4856f7 发射面（office-io docx.rs
T1/T5 钉定形状）：`w:hyperlink w:anchor` 属性形零 rels（anchor_link）
+ `bookmarkStart/End` 对（bookmark_paragraph，链接段页 1 / 书签
目标段页 2 经显式分页分离——内链跨页导航才具存活语义）+ 孤立书签
（orphanTarget 无入链）+ 外链对照（r:id 形 TargetMode=External）。
mdcore 侧钉 XML 形状、本套件钉 LO 行为——**产品 API 形状 → LO →
PDF 链接存活全链闭合**。

| # | 断言 | 钉定 | 实测 |
|---|---|---|---|
| 1-5 | 通用（rc/超时/产出/页数=2） | 链接段页 1 + 书签目标段页 2 | 全过 |
| 6-10 | 文本存活五串（标题/跳转到/链接文本/目标段/孤立段） | 双侧归一化包含 | 全过 |
| 11 | internal_link_hrefs_p1 = 1 | **w:anchor → 恰一内链 GOTO 注记存活** | anchor-survival.html#2 |
| 12 | internal_link_target_href = `{stem}.html#2` | **目标页 = 2 = 书签所在页**（pdftohtml fragment 承载） | 精确匹配 |
| 13 | external_link_hrefs_p1 = 1（URL 精确） | 外链 → URI 注记（类型可区分：fragment 形 vs 原始 URL） | 全过 |
| 14 | link_count_p2 = 0 | 书签目标页零链接注记 | 全过 |
| 15 | internal_link_text_contains_linktext | 内链注记与链接文本 run 关联（归一化包含） | 全过 |

### 10.2 行为发现（探针观测，供治理登记）

1. **锚点存活定谳**：`w:hyperlink w:anchor` 经 LO → PDF **GOTO
   Link 注记存活**，目标为书签所在页的直接 XYZ 目的地（72, 769.889,
   zoom 0——坐标为观察值不断言，布局敏感）。**C 路线保留内部导航**
   ——R-O2 管线不因转换丢失文档内链。
2. **书签不导出命名目的地**（PyMuPDF probe 一次性观测
   resolve_names() = 空，未入套件依赖）：LO 在转换期把书签消解为
   直接目的地，PDF 命名树不出现书签名——**mdc-pdf 消费腿的链接
   到达形态 = GOTO 显式目的地**，命名目的地而依赖的检索/去重策略
   不可行（内容级摘要口径维持 r-o2-t4 结论）。
3. **孤立书签零可观测面**：无入链书签在 PDF 无注记、无目的地、无
   outline 差异——治理含义：书签定义单独存在时对渲染产物零影响。
4. **Heading1 → PDF outline 条目**（观察值，未断言——样式映射面
   非本轮语义）：标题样式段进大纲面板。
5. **pdftohtml 链接关联为文本块粒度**（断言面工程注记）：`<a>`
   包裹可吸收相邻非链接文本（实测吸收了前置普通 run「跳转到 」
   但不含后置「。」）——链接文本断言用**包含语义**不用相等；依赖
   poppler pdftohtml 25.03.0（套件依赖面零三方维持）。

### 10.3 门禁与台账

全量 22 用例 = 10 pass + 12 pinned + 0 fail（204 断言，SUITE PASS）
**双次复跑稳定**（台账非首次零漂移 + 夹具确定性门 + 跨进程确定性门
恒过）；新夹具键 anchor-survival = 良性首跑基线（§9.4 语义）。LO
版本差警示维持（§1）：全部断言含 26.8.1 上游复跑前置。

## 10.5 LO 26.8.1 跨版本复跑（ro2-t6，2026-10-18）——版本差信号命中与定性

**背景**：套件设计语义「LO 版本升级导致钉定破裂 = 版本差信号，
需复核后更新钉定」（§1）+ #18 上游复跑前置。TDF 官方 26.8.1.1
deb（sha256 台账：tarball 219,791,834B）解包独立目录 PATH 前置
运行，与系统 25.2.3.2 零接触。

**结果**：22 用例 = 21 维持（10 pass + 11 pinned）+ **1 钉定破裂**
（garbage-bytes，双次复跑稳定）+ 夹具确定性门 pass + 跨进程
确定性门 pass。

**版本差信号定性（良性——fail-closed 改进）**：

| 面 | LO 25.2.3.2（本机基线钉定） | LO 26.8.1.1（实测） |
|---|---|---|
| garbage-bytes rc | 0（静默兜底） | **1（显式失败）** |
| garbage-bytes PDF 产出 | True（乱码文本导入兜底——r-o2-t4 发现） | **False（零产物）** |
| 其余 21 用例 | 全维持 | 全维持（含 anchor-survival：内链 GOTO 存活指向书签页 **在 26.8.1 同成立**） |

**处置**：本机基线钉定**维持 25.2.3.2 口径不更新**（基线跟随批次 3
G5 #7 裁决——发行版打包 25.2.x 推荐默认未变）；26.8.1 行为作为
版本差证据落档。**裁决增量证据**：若批次 3 改选 26.x 基线，唯一
钉定更新面 = garbage-bytes 单用例四断言（rc 0→1 / PDF 产物
True→False，方向 = fail-closed 语义改进，安全面利好）；anchor
存活等关键行为跨版本稳定 = 版本升级风险显著收窄。

**复现**：`PATH=<lo26>/opt/libreoffice26.8/program:$PATH python3
regress.py`（tarball 于 tool-results/lo26/，不入仓）。


## 10.6 渲染守护器就绪原型（ro2-t7，render_guard.py）——批次 3 机械项预置

**性质**：批次 3 G5 #7（Q3 子进程 + 宿主预装）实施轮机械项的 spike
就绪原型（d1e-t3 先例模式——决策无关预置，裁决后产品化落位）。

三机械面单守护器（`render_guard.py`，stdlib only）：
- **G1 单实例并发锁**：flock 排他 + 阻塞串行化 + 锁超时
  fail-closed——内存上界 = 单转换 footprint（LO 并发转换 = RSS
  上界钳制实施形态）；
- **G2 RSS 看门狗**：/proc 进程树 VmRSS 轮询（后代展开）——软限
  告警记录不杀 / 硬限 SIGKILL 整进程组（防 OOM 拖垮宿主）；
- **G3 超时杀组**：r-o2-t3 既有语义守护器化收编（start_new_session
  独立进程组）。

四断言组验证（render_guard_test.py，全过 2026-10-18）：
| 组 | 断言 | 实测 |
|---|---|---|
| T1 串行化 | 3 并发真实 soffice 转换全成功 + 锁等待证据 + 总墙钟串行下界 | 锁等待 [0.704, 1.024, 0.0]s，总 1.39s |
| T2 RSS 正常面 | 软限告警不误杀 + 峰值记录 + 树进程计数 | 峰值 187MB，树 2 进程 |
| T3 RSS 硬限 | 合成吞噬者 500MB + 硬限 200MB → SIGKILL 组 | 峰值 243MB 触杀，rc=-9 |
| T4 超时杀组 | sleep 30 + 限时 2s → 按时杀组 | wall 2.015s，rc=-9 |

**产品化映射**（裁决后）：套件内联的超时/杀组逻辑（regress.py）
收敛到守护器单点；单实例锁与 RSS 硬限 = 部署参数（SECURITY_
POLICY 限额表挂点）；RSS 口径与 rss_wrapper 峰值口径互补（看门狗
= 在飞防护，wrapper = 事后计量）。


## 10.7 LO 26.2.6 still 线复跑（maint-r1，2026-10-25）：批次 3 三选证据腿补齐

> 触发链：TDF 版本线核对发现 **25.2 分支 EOL**（25.2.7 = final
> maintenance release，官方公告 2025-10-30，明示建议 25.2.x 用户升
> 25.8.x——TDF Community Blog 检索 datetime 直证）→ 批次 3 版本基线
> 候选面扩为 25.2.x / 26.2.x / 26.8.x 三选，其中 26.2.x（still 活跃
> 线）无实测 → 本轮补齐。性质 = §10.5 同构机械复跑：零代码改动、
> 零钉定翻动、纯证据腿。

### 10.7.1 环境

- 来源：TDF 官方 `stable/26.2.6/deb/x86_64`
  `LibreOffice_26.2.6_Linux_x86-64_deb.tar.gz`（217,285,735 B，gzip
  完整性验证；解包内目录 `LibreOffice_26.2.6.3_...`——四段构建号）；
- 安装形态：用户前缀解包 `/home/z/.lo262/opt/libreoffice26.2`（42
  deb 全量 data.tar 解包，无 root 无 dpkg；脚本
  `scripts/unpack_lo2626.sh`）+ PATH 垫片 `shim/soffice`；系统基线
  25.2.3.2 零改动——多版本并行在场；
- 版本串实测：`LibreOffice 26.2.6.3 8221e31b3ac356a1623c672912a3d2b492f7e3d1`；
- 套件：regress.py 零改动，`--json-out out/regression-report-lo2626.json`
  （首跑）+ `regression-report-lo2626-run2.json`（复跑），同工作区
  先后完成。

### 10.7.2 结果（三版本对照）

| 面 | 25.2.3.2（基线） | 26.2.6.3（still） | 26.8.1.1（fresh，§10.5） |
|---|---|---|---|
| 总判 | 10 pass + 12 pinned + 0 fail | **10 pass + 12 pinned + 0 fail（零破裂）** | 10 pass + 11 pinned + 1 pin broken |
| garbage-bytes | PINNED（rc=0 + 乱码产出） | **PINNED（rc=0 + 页 1 产出，冷 1.34s / RSS 274MB）** | 破裂（rc=1 + 零产出） |
| 10 有效夹具 | 全 PASS | **全 PASS 零漂移**（含 anchor-survival 16 断言三钉定） | 全 PASS 零漂移 |
| 12 畸形钉定 | 全 PINNED | **全 PINNED 恒** | 11 PINNED + 1 破裂 |
| 确定性门 ×2 | pass | **pass（双次复跑稳定，逐用例同判）** | pass |

### 10.7.3 版本差信号时间窗收窄

garbage-bytes 的 rc=0 垃圾字节兜底面：**25.2.3.2 开放 → 26.2.6.3
仍开放 → 26.8.1.1 收窄**——上游收窄变更发生在 **26.2.x → 26.8.x
之间**（非 25.2→26.2）。锚点存活三钉定（GOTO 存活 / 目标页 =
书签页 / 孤立书签零可观测）跨三版本全恒。

### 10.7.4 治理含义（呈批次 3）

1. **26.2.x = 零钉定翻动的最平滑升级路径**：行为面与 25.2.x 基线
   完全一致（22/22 同判、零破裂）+ still 线活跃维护（26.2.6，
   2026-09-04）——若裁决升 26.x，26.2.x 的回归面代价为零（对照
   26.8.x 的一处良性破裂）；
2. **25.2.x = EOL 存量**：TDF 官方维护已移出（25.2.7 final，
   2025-10-30）——若维持钉 25.2.x，接受安全修复停止面（知情登记）；
3. 26.8.x = fresh 更严行为（fail-closed 方向，§10.5 定性），若选
   26.8.x 需翻钉 garbage-bytes 单用例（批次 3 后置项）；
4. 版本差证据链三腿齐备（25.2.x 基线 / 26.2.x 零漂移 / 26.8.x 一
   破裂）——批次 3 版本基线三选的裁决输入完备，不再缺实测腿。

### 10.7.5 复现

```bash
PATH="/home/z/.lo262/shim:$PATH" \
  python3 regress.py --json-out out/regression-report-lo2626.json
```

诚实边界：同 §10.5（用户前缀解包形态，stderr python 告警为该形态
既有；RSS/墙钟为观察值：26.2.6 冷 0.36-1.34s / 暖 0.21-1.11s /
97-274MB 与两版本同区间——无 dpkg 预链接，归因不确定）。

## 10.8 第 19 次重置后环境复验 + TDF 分支 EOL 全景（maint-t3，2026-10-25）

### 10.8.1 环境复验（基线同判零漂移）

第 19 次环境重置（cargo/.lo26 丢失；系统 LO 25.2.3.2 与 `.lo262`
在场）后，套件工件（fixtures/ + regress.py + render_guard.py +
rss_wrapper.py 全在 git 内）零损失。全量复跑（系统 25.2.3.2 基线
口径）：**22 = 10 pass + 12 pinned + 0 fail，SUITE PASS**（夹具
确定性门 pass + 跨进程确定性门 pass 8 断言）——与 §10 基线完全
同判零漂移，与 §10.5（26.8.1）/ §10.7（26.2.6.3）三版本对照结论
互相印证。报告工件：`out/regression-report-maint-t3.json`。

### 10.8.2 TDF 分支 EOL 全景（批次 3 时效增量）

| 分支 | EOL 日期 | 状态（2026-10-25 时点） |
|---|---|---|
| 25.2 | 2025-11-30 | 死 11 个月（25.2.7 final 公告 2025-10-30） |
| 25.8 | 2026-06-12 | 死 4.5 个月（25.8.7 final 2026-05-12；官方引导存量升 26.2.5） |
| **26.2** | **2026-11-30** | **still 线活跃（26.2.6 于 2026-09-24）——EOL 仅剩 ~5 周** |
| 26.8 | 2027-06-13 | fresh 线（26.8.1 于 2026-10-08）——~7.5 个月窗口 |

一手源：TDF 26.8 发布新闻稿原文 "The LibreOffice 26.2 branch
reaches end of life on 30 November 2026"；25.8 EOL = TDF Blog
（"reached end of life on 12 June 2026... Users of 25.8.x
should update to 26.2.5"）+ Release Plan 25.8 wiki。25.2 EOL
公告建议的升级目标 25.8.x 自身已 EOL——**第四选排除**。

### 10.8.3 治理含义（呈批次 3，时效天平）

- 「钉 26.2.x」= 零翻动最平滑，但 5 周后该线自身变 EOL 存量
  （重蹈 25.2 覆辙，需再迁一次）；
- 「钉 26.8.x」= 翻钉 garbage-bytes 一项（良性 fail-closed，
  §10.5 定性），换 7.5 个月维护窗口；
- 时效证据不改变三选结构，但改变两选项的代价天平——DECISION_
  SHEET 批次 3 证据面已同步登记，裁决归用户。
