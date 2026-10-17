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
| poppler | pdftotext / pdfinfo / pdfimages / pdffonts 全部 **25.03.0** |
| python | 3.12.15（**stdlib only**，零第三方依赖） |
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
任一 fail（含钉定破裂、超时）→ exit 1；`--filter` 无匹配 → exit 2。
