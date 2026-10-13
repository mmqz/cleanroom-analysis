# mdc-office 消毒五面针对性安全测试设计（t10-e）

> 定位：xlsx-r1 PLAN §2 t10-e——用户指令 4 的设计交付：「复核 mdc-office
> 消毒模块……防重复编码、合法内容损坏、防护绕过；**现有性质测试不能
> 替代针对性安全测试**」。本文件 = 阶段 B（mdcore 落地）的测试设计
> 先行件：逐面攻击载荷表 + 期望输出 + 意图锚点。落地时按表实现为
> `crates/office/tests/sanitize_security.rs`（集成测试，与既有 23 项
> 性质/对照测试并存）。
>
> 依据：worklog t9-c 五面 API 形状（mdcore 2a4cb6c）+ office-d §5
> 逐面规格 + office-a §9（isSafeUrl 参照）+ 本轮 t10-d 读侧容忍度发现。

## 0. 设计原则（与既有测试的分界）

既有 23 项（t9-c）= **性质测试**：码位全扫、三档对照、4000 例 LCG
往返——回答"任意输入下不变量是否成立"。本设计 = **针对性安全测试**：
逐攻击向量精确输入 → 精确期望输出——回答"这个具体攻击是否被防住"。
两者不可互替：性质测试防未知形态的意外损坏，针对性测试防已知攻击
形态的防护缺口（一个防回归面、一个防绕过面）。

每用例四元组：ID / 输入 / 期望 / 攻击意图。**期望为精确值**（非
"包含/不包含"弱断言），落地时逐条 `assert_eq!`。

## 1. 面 1：csv_formula_guard（CSV 公式防护）

触发集 `=` `+` `-` `@` 首字符；三模式（前置 `'` 默认 / 前置空格 /
typed reject）。xlsx 路线豁免注记在档。

| ID | 输入 | 期望（默认模式） | 意图 |
|----|------|------------------|------|
| C-01 | `=cmd\|'/c calc'!A1` | `'=cmd\|'/c calc'!A1` | 经典 DDE 注入，直接触发 |
| C-02 | `+cmd\|'/c calc'!A1` | `'+cmd...`（同前缀） | `+` 触发集成员 |
| C-03 | `-2+3+cmd\|'...'` | `'-2+3+cmd...` | `-` 触发集成员（负数陷阱：`-2` 是合法数值形态，但 Excel 仍按公式解释——**必须消毒**，业界惯例一致） |
| C-04 | `@SUM(A1:A9)` | `'@SUM(A1:A9)` | `@` 触发集成员（隐式交集引用） |
| C-05 | ` =cmd\|...`（前置空格） | **原样不动** | 前置空格在 Excel 语义下已中和公式解释（这也是模式二的原理）——消毒即重复编码，须防 |
| C-06 | `'=cmd\|...`（已消毒形态） | **原样不动** | **防重复编码**：已带 `'` 前缀再叠加 = `''=...` 损坏内容（幂等性） |
| C-07 | `'hello`（合法撇号开头文本） | **原样不动** | **防合法内容损坏**：撇号开头≠已消毒的公式（只有 `'+-@` 后随形态才可能是消毒产物——保守判定为不动，宁漏消毒不损坏合法内容，`'h` 不构成公式风险） |
| C-08 | `＝cmd`（全角＝ U+FF1D） | **原样不动** | Unicode 同形攻击：Excel 不按全角＝ 解释公式——动了反而损坏 |
| C-09 | `=1+1`（普通公式，非恶意） | `'=1+1` | 非选择性消毒：guard 不做恶意判定（无法判定），全量触发——期望钉死"无启发式"契约 |
| C-10 | `\t=cmd`（制表符前缀） | 原样不动（首字符非触发集）+ 文档注记制表符语义 | 边界钉定：触发判定 = 字面首字符，`\t` 非触发集成员 |

模式对照补充：C-01 于模式二 → ` =cmd...`（前置空格）；C-01 于模式
三 → typed reject 错误（含原因串）。C-05/C-06/C-07 三例在全部三模式
下均不动（幂等/合法保护是模式无关不变量）。

## 2. 面 2：xml_control_chars（_xHHHH_ 保字符档）

转义集 = 0x00-0x08/0x0B/0x0C/0x0E-0x1F/U+FFFE/U+FFFF；`\t\n\r` 与
0x7F 放行；字面量消歧含逃逸边界闭合扩展（t9-c 设计）。

| ID | 输入 | 期望 | 意图 |
|----|------|------|------|
| X-01 | `a\x00b` | `a_x0000_b` | 基线：NUL 转义 |
| X-02 | `_x0000_`（字面量） | `_x005F_x0000_` | **防护绕过（解码面走私）**：若不消歧，原文照排 → 下游解码变真 NUL——消歧后解码回原文 `_x0000_` 零失真 |
| X-03 | `_x005F_x0000_`（已消歧形态再编码） | `_x005F__x005F_x0000_`（逐级消歧，见下注） | **防重复编码**：已消毒文本再过一遍不产生解码歧义（幂等性按"可解码性"定义，非字符串相等） |
| X-04 | `a\tb\nc\rd` + `\x7F` | **原样不动** | **防合法内容损坏**：XML 1.0 合法字符放行集钉死 |
| X-05 | `_x000A_`（字面量，合法 \n 的形态） | `_x005F_x000A_` | 同 X-02：合法字符的"伪装形态"同样须消歧（否则解码面把字面量变真换行——类型走私） |
| X-06 | `_X0000_`（大写 X） | **原样不动** | 转义约定大小写敏感（小写 `_x`）——大写是合法字面量，解码器不认；动 = 损坏 |
| X-07 | `_x000_`（3 位十六进制）/ `_x00000_`（5 位） | **原样不动** | 非法形态（HHHH 恰四位）：不是转义形态的字面量不消歧 |
| X-08 | `_x00G0_`（G 非十六进制） | **原样不动** | 同上：非十六进制位不构成形态 |
| X-09 | `_xFFFF_`（U+FFFF 非字符） | `_x005F_xFFFF_` | 非字符**字面量**消歧 + 该码位本身在转义集（若真出现则转义 `_xFFFF_`）——两路径对称钉死 |
| X-10 | `\u{FFFE}`（真非字符） | `_xFFFE_` | 转义集上界成员（多字节字符级判定，t9-c 逃逸边界闭合的实测点） |

X-03 注：幂等语义 = `decode(encode(x)) == x` 恒成立（含 x 本身已是
消歧形态）；字符串级 `encode(encode(x)) == encode(x)` **不**成立
（消歧前缀逐级增长）——这是**有意设计**（否则解码歧义），测试钉死
该契约并用文档防调用方重复应用。防重复编码的责任面：管线纪律
（每出口消毒一次），本面提供 `is_escaped_form` 探针 API 供管线自检。

## 3. 面 3：xml_entities（XML 实体编码）

五实体（`& < > " '`）+ 控制字符一体化（转义集同面 2）。

| ID | 输入 | 期望 | 意图 |
|----|------|------|------|
| E-01 | `<tag>&"attr"</tag>` | `&lt;tag&gt;&amp;&quot;attr&quot;&lt;/tag&gt;` | 基线全实体 |
| E-02 | `&amp;`（用户字面文本） | `&amp;amp;` | **契约钉死（防重复编码误设计）**：纯转义器必须把已转义形态当字面文本（否则解码失真）；幂等性故意不成立——测试锁死防"优化"成幂等（那会破坏往返） |
| E-03 | `x" onmouseover="alert(1)` | `x&quot; onmouseover=&quot;alert(1)` | **防护绕过**：属性注入经典载荷——引号转义后无法闭合属性 |
| E-04 | `]]>`（CDATA 终结符） | `]]&gt;` | CDATA 逃逸形态（XML 注入面） |
| E-05 | `a\x00b`（一体化路径） | `a_x0000_b` | 控制字符一体化兜底（与面 2 同输出——单一实现两出口） |
| E-06 | `&#x00;`（数字实体文本） | `&amp;#x00;` | **防护绕过（解码面走私）**：不转义 `&` 则下游解析器把数字实体还原成 NUL——& 转义即中和 |
| E-07 | `&#60;script&#62;` | `&amp;#60;script&amp;#62;` | 同上，标签走私形态 |

## 4. 面 4：html_escape（HTML 出口转义）

属性与文本位全转义（office-d §5.1 裁决 3 的 href 未转义兜底）。

| ID | 输入 | 期望 | 意图 |
|----|------|------|------|
| H-01 | `<img src=x onerror=alert(1)>` | `&lt;img src=x onerror=alert(1)&gt;` | 基线：标签注入中和 |
| H-02 | `" onmouseover="alert(1)` | `&quot; onmouseover=&quot;alert(1)` | 属性闭合 breakout |
| H-03 | `javascript:alert(1)`（作 href 值） | `javascript:alert(1)`（**字符面零变化**）+ 断言注记 | **边界实证（本设计的关键发现）**：字符转义对 URL scheme 攻击**无效**——scheme 无特殊字符可转。本用例把该边界钉死为测试，作为面 6（url_policy，§6）必要性的在档证据 |
| H-04 | `data:text/html;base64,PHNjcmlwdD4=` | 字符面零变化 + 注记 | 同 H-03：data URI 载荷 |
| H-05 | `' onclick='x`（单引号属性形态） | `&#39; onclick=&#39;x` | 单引号属性位 breakout（五实体全覆盖） |
| H-06 | `&amp;lt;` | `&amp;amp;lt;` | 同 E-02 契约：字面已转义文本按字面处理（往返保真） |

## 5. 面 5：field_whitelist（Word 域指令白名单）

默认七白（DATE/PAGE/NUMPAGES/TOC/PAGEREF/MERGEFIELD/SEQ）六黑
（INCLUDETEXT/INCLUDEPICTURE/DDE/DDEAUTO/LINK/RD）；全 token 前缀
扫描（含引号粘连）。

| ID | 输入 | 期望 | 意图 |
|----|------|------|------|
| F-01 | `INCLUDETEXT "c:\evil.docx"` | typed reject（黑名单首列） | 外部资源面基线 |
| F-02 | `DDEAUTO cmd /c calc` | typed reject | 执行面基线 |
| F-03 | `TOC \o "1-3" INCLUDETEXT evil` | typed reject | **防护绕过**：嵌套注入（合法域后挂黑名单域）——全 token 扫描的靶心用例 |
| F-04 | `MERGEFIELD x \d DDEAUTO` | typed reject | 同上，开关位后挂 |
| F-05 | `incluDeText "x"` | typed reject | 大小写逃逸（域关键字大小写不敏感——Word 同样不敏感，须对称） |
| F-06 | `TOC "INCLUDETEXT"` | typed reject | **引号粘连形态**（引号内黑名单 token 同样拒——Word 域解析不按引号隔离关键字） |
| F-07 | `DATE`/`PAGE`/`MERGEFIELD name` | **放行** | **防合法内容损坏**：七白成员精确放行 |
| F-08 | `DATED` / `PAGES` / `INCLUDETX` | **放行**（非黑名单 token，前缀不等于关键字） | **防合法内容损坏**：前缀扫描的误杀边界——`INCLUDETX` 不是 `INCLUDETEXT`，拒了就损坏合法（用户自造域）内容 |
| F-09 | `MERGEFIELD INCLUDETEXT_list` | typed reject + 文档注记 | 已知保守误杀：数据字段名以黑名单字开头——安全优先于可用性，登记不修复（病理形态，真实 MERGEFIELD 字段名不叫这个） |
| F-10 | `TOC \o INCLUDETEXTX` | 放行 | 同 F-08 边界：`INCLUDETEXTX` 前缀超集不等于关键字 |

## 6. 面 6（缺口登记）：url_policy——URL 策略独立面

**本轮复核发现**（对照用户指令 4 的五上下文与 t9-c 五面）：指令列
「CSV 公式防护、XML 字符合法性、XML 实体编码、HTML 转义、**URL
策略**」——前四上下文与 t9-c 四面一一对应；**URL 策略上下文在 t9-c
五面中没有独立承载面**（html_escape 承 href 注入位、field_whitelist
承域外链位，但 **scheme 级策略**——`javascript:`/`data:`/`vbscript:`
协议白名单——无模块）。H-03/H-04 用例实证：字符转义对该攻击面
无效。office-a §9 isSafeUrl（http/https/mailto 白名单）为一手参照。

**阶段 B 增补设计**（新 sanitize 模块 `url_policy`）：

- 策略：scheme 白名单（默认 `http` `https` `mailto` + xlsx 内部链接
  `internal`——rust_xlsxwriter write_url 语义）+ 相对路径放行 + 其余
  typed reject（带原因串）；`file:` `\\UNC` `javascript:` `data:`
  `vbscript:` 及未知 scheme 全拒。
- 测试表（核心载荷，落地时逐条精确断言）：

| ID | 输入 | 期望 | 意图 |
|----|------|------|------|
| U-01 | `javascript:alert(1)` | reject | scheme 注入基线（H-03 实证的补面） |
| U-02 | `JaVaScRiPt:alert(1)` | reject | 大小写逃逸（scheme 大小写不敏感） |
| U-03 | ` java\tscript:alert(1)`（前导空白/内嵌控制符） | reject | 规范化逃逸：空白与控制符在 scheme 解析中被剥除——按剥除后判定 |
| U-04 | `data:text/html;base64,...` | reject | data URI |
| U-05 | `vbscript:msgbox(1)` | reject | IE 族遗产面 |
| U-06 | `file:///etc/passwd` | reject | 本地文件面 |
| U-07 | `\\\\evil\\share`（UNC） | reject | UNC 路径面 |
| U-08 | `//evil.com/x`（协议相对） | **放行**（http(s) 家族语义）+ 注记 | 协议相对 URL 合法（页面继承 scheme）——误拒即损坏合法链接 |
| U-09 | `https://example.com/docs?q=1` | 放行 | 白名单基线 |
| U-10 | `mailto:a@b.c` | 放行 | 白名单基线 |
| U-11 | `internal:Target!A1` | 放行 | xlsx 内部链接（rust_xlsxwriter 语义） |
| U-12 | `internal:Sheet1!A1" style="x` | reject（引号/属性粘连） | 内部链接位的属性注入形态（与 html_escape 联防） |
| U-13 | `java\u0000script:alert(1)` | reject | NUL 截断逃逸（解析器差异面——t10-d mal7c 容忍度发现的直接联动面：**读侧不过滤 NUL，URL 面必须过滤**） |
| U-14 | `HTTPS://EXAMPLE.COM` | 放行 | 大小写不误杀白名单 |
| U-15 | `mailto:`（空余部） | 放行 | 边界：scheme 合法性独立于余部 |

- 与既有面联防注记：url_policy 判 scheme（结构位），html_escape 判
  字符（属性位）——出口管线顺序 = url_policy 先（reject 出）→
  html_escape 后（转义余部字符）；顺序反了会把 reject 证据转义掉
  （不破坏安全性但破坏错误信息），管线文档钉死。

## 7. 落地清单（阶段 B）

1. `crates/office/tests/sanitize_security.rs`：§1-§5 全用例逐条
   `assert_eq!`/`#[should_panic]`-free typed reject 断言（面内 API
   返回 Result，不 panic）。
2. `src/sanitize/url_policy.rs` 新模块 + §6 测试表（缺口补面）。
3. 用例计数预估：面 1×10（+三模式对照 3）+ 面 2×10 + 面 3×7 +
   面 4×6 + 面 5×10 + 面 6×15 ≈ **61 项针对性安全用例**（与既有
   23 项性质测试并存，合计 ~84）。
4. 文档同步：面 2 X-03 幂等契约、面 3 E-02 非幂等契约、面 5 F-09
   保守误杀、面 6 管线顺序——四处写入 crate 文档的"调用方契约"节。

## 8. 复核结论（本轮）

- 四面（csv/xml 字符/xml 实体/html）设计复核通过：无防护逻辑缺陷
  发现；两处**契约级**澄清入档（幂等性按面定义，非全局统一）。
- 一处**缺口**发现：URL 策略上下文无独立承载面（§6）——阶段 B
  补面，不阻塞本轮（消毒模块现状无绕过回归：html_escape 本就不
  宣称 scheme 防护，边界已在 H-03 钉死）。
- t10-d 联动：mal7c 读侧容忍 0x00 的发现将 U-13 从理论向量升级为
  实证联动面（读侧不过滤 → 消毒链必须过滤）。
