# office-b：ONLYOFFICE Docs 深读报告（11 维全量）

> **净室分析报告**（线 A 办公 P1 第二篇）。日期：2026-10-11。
> 对象（浅克隆六仓）：ONLYOFFICE/DocumentServer @ `f580eb58`（2026-07-22，
> 版本 9.4.0， submodule 聚合仓）、ONLYOFFICE/core @ `3250a848`（x2t C++
> 转换内核）、ONLYOFFICE/server @ `d6df308`（Node DocService/FileConverter）、
> ONLYOFFICE/sdkjs @ `72b0421`（JS 编辑器）、ONLYOFFICE/DocumentBuilder @
> `21c47ae`（docbuilder 壳仓）、ONLYOFFICE/sdkjs-plugins @ `06c4b85`
> （core/sdkjs/server 三仓同步于 2026-05-19，对应同一次 release train）。
> 许可：五主仓均 **AGPL-3.0 + Ascensio §7 附加条款**（见 §11）。
> 方法：单代理全仓只读勘察（三段 x2t/sdkjs/DocService 逐文件读关键路径）+
> raw/官网一手声明逐处标注；证据锚点 `仓/路径:行号`（六仓分别以前缀
> DocumentServer= / core= / server= / sdkjs= / DocumentBuilder= /
> sdkjs-plugins= 区分）；【推断】= 分析员判断。
> 外部一手源：ONLYOFFICE 官方博客《The year we went open source》
> （onlyoffice.com/blog/2026/06/the-year-we-went-open-source/，
> 2026-06-29 发布，文中核验）。
>
> 一句话定性：**「内部二进制格式为真相源」的三段式办公引擎——x2t(C++
> +嵌入式 V8) 把一切格式折进 DOCY/XLSY/PPTY 私有二进制，sdkjs 在浏览器
> 里对内存模型做「可序列化 undo 项 = 协同 change」的服务器全序 OT，
> DocService(Node) 只做 store-and-forward 编排——三段用 XML 任务文件、
> socket.io、RabbitMQ 三种协议缝合；OOXML 保真的工程支撑是
> assemblyFormatAsOrigin + 原格式回写而非无损模型（【推断】跨格式往返必有损；
> 同格式编辑往返稳定，见 §4），而无 OS 沙箱的转换进程与 shadow-eval 宏沙箱
> 是其安全债务面**。

## 0. 架构速览

```
DocumentServer（聚合仓，18 个 submodule 指针：6 主仓 + 10 插件 + 模板/
  格式表）＝发布组装事实
  ├─ core     = x2t 转换器（C++）：OOXML/ODF/二进制/PDF/RTF/HTML/EPUB/DjVu/
  │            XPS/HWP/OFD/Visio 读写 + OfficeCrypt 解密 + DocxRenderer；
  │            内嵌 doctrenderer（V8/macOS 用 JavaScriptCore）跑 sdkjs 脚本
  │            → x2t 实为 C++/JS 混合引擎
  ├─ sdkjs    = 三编辑器 + PDF 编辑器（word/cell/slide/pdf 四模块）
  │            渲染：word=每页位图缓存+单画布，cell/slide=画布，pdf=自研 wasm 引擎
  ├─ server   = Node：DocService(socket.io 协同编排) + FileConverter(拉队列
  │            → spawn x2t/docbuilder) + SpellChecker + WOPI 客户端
  └─ web-apps = 宿主 UI（未克隆，插件 iframe 宿主在此，登记为外部事实）

打开文档：浏览器 ──openDocument──▶ DocService commandOpen
  ──队列任务(toFile="Editor.bin")──▶ x2t 把源文件转成内部二进制
  ◀──签名 URL── 浏览器下载 Editor.bin → sdkjs BinaryFileReader 建内存模型
保存文档：sdkjs 全量重序列化 → savefile 分片上传 → x2t (m_bFromChanges)
  在 DoctRenderer 里对旧 bin 应用 changes.json → 新 bin → 转目标格式
协同：sdkjs 把 undo 项二进制化 → saveChanges(带 deleteIndex) →
  server 全序落 SQL(doc_changes) + pubsub 广播 → 各端 undo 自有→按序应用→重放
```

三段职责引证：`server/DocService/sources/canvasservice.js:596`（开档任务
`dataQueue.setToFile('Editor.bin')`）；`core/X2tConverter/src/ASCConverters.cpp:1768`
（TCD_* 方向 switch）；`sdkjs/common/apiBase.js:3102`（`this.CoAuthoringApi =
new AscCommon.CDocsCoApi()`）。

## 1. 数据模型与持久化完整性

**真相源 = 私有二进制格式，不是 OOXML**。Editor.bin 头部魔数与版本明文：
`DOCY/XLSY/PPTY`（`sdkjs/common/editorscommon.js:786-800` 按 4 字节魔数分派
编辑器；`server/Common/sources/formatchecker.js:726-737` 服务端同款检查）。
文件头 = `Signature + ";v" + 版本 + ";" + 长度 + ";"`（`sdkjs/word/api.js:
13520,13541`；写头 `sdkjs/word/Editor/Serialize2.js:1916-1920`），且正文内
再 `WriteLong(c_oSerFormat.Version)`（`Serialize2.js:2188`）；版本常量
`{Version:5, Signature:"DOCY"}`（`sdkjs/word/apiDefines.js:142-145`）。格式
族谱系：内核枚举里这套二进制叫 `AVS_OFFICESTUDIO_FILE_TEAMLAB_DOCY/XLSY/
PPTY/VSDY`（0x1000 段，项目前身 Teamlab 遗产）与 `FILE_CANVAS_*`
（0x2000 段）（`core/Common/OfficeFileFormats.h:130-155`）。

**打开 = 二进制反序列化为 JS 对象树，全内存**：`OpenDocumentFromBin →
AscCommonWord.BinaryFileReader(LogicDocument).Read(gObject)`
（`sdkjs/word/api.js:1488-1498`；Reader 定义 `sdkjs/word/Editor/Serialize2.js:7807`）。
对象树是手写类型体系：CDocument（`sdkjs/word/Editor/Document.js`，
29158 行）→ Content 数组 → Paragraph（`sdkjs/word/Editor/Paragraph.js:67`，
全文件 21126 行）→ Run/Item 树；**每个对象带字符串 Id 并注册进全局
TableId**（`sdkjs/common/TableId.js:123,615` 的 `g_oTableId`）——Id 是
协同 change 寻址与锁定的主键（§3）。

**就地编辑，无快照层**（与 Univer「快照即模型」对照）：sdkjs 没有
IWorkbookData 式快照，模型对象被 History 项直接改写；持久化 = 保存时
**全量重序列化**：`asc_nativeGetFileData → AscCommonWord.BinaryFileWriter.
Write()`（`sdkjs/word/api.js:13523-13545`；同族 `asc_nativeGetFile/2/3`
:13506-13521）。服务器侧保存即「旧 Editor.bin + changes → 新 Editor.bin」
（§4），单文档体积上限受 `maxChangesSize 150MB` 等配置约束
（`server/Common/config/default.json:514-520`）。

**浏览器也能直开 OOXML，但由宿主声明**：`openDocument` 按数据分派——
`bSerFormat` 走二进制；否则 `asc_isSupportFeature("ooxml") &&
checkOOXMLSignature(data)` 时 `OpenDocumentFromZip`（`sdkjs/word/api.js:
8444-8467`），即 `jsZlib` 解包 + `CDocument.fromZip` 的 **JS 版 OOXML 解析器**
（`sdkjs/word/api.js:1500-1532`）。`asc_isSupportFeature` 查
`window.Asc.Addons[type]`（`sdkjs/common/apiBase.js:5217-5220`）——桌面版
注入 addon、DocumentServer 网页版不注入。【推断】同一 sdkjs 维护着
C++(x2t) 与 JS 双份 OOXML 读写，网页版走服务端转换是产品/性能选择而非
能力缺失；对 mdcore 的启示：**读写器单实现（Rust）双编译目标（native+
wasm）**优于双实现（§13）。

**格式嗅探三处实现**：内核 C++（`core/Common/OfficeFileFormatChecker.h`）、
Node（`server/Common/sources/formatchecker.js:720-789`，含 ZIP 尾部
central directory 搜 `xl/workbook.xml` 的启发式）、JS（`editorscommon.js:
783-818`，读 `[Content_Types].xml` 判 word/cell/slide）。【推断】三份嗅探
是三段演进的历史沉积，行为一致性靠回归测试维持——Rust 自研应单点实现
共享。

## 2. 渲染管线

**word = 单画布 + 每页位图缓存 + 懒渲染**。宿主页面给 `id_viewer` /
`id_viewer_overlay` 两个 canvas（`sdkjs/word/Drawing/HtmlPage.js:428-439`）；
每页一个 `CDrawingPage{cachedImage, IsRecalculate, RecalculateTime}`
（`sdkjs/word/Drawing/DrawingDocument.js:1020-1032`），由 CacheManager 管理位
图池。绘制循环按可视窗口 `m_lDrawingFirst..m_lDrawingEnd` 迭代：未缓存的页
`StartRenderingPage`，滚出窗口的页 `StopRenderingPage`，全量重绘时才清缓存
（`HtmlPage.js:2961-3041`）；失效入口是 `ClearCachePages/FirePaint`
（`DrawingDocument.js:6417,6460`），`FirePaint` 只是触发 `OnScroll` 重算可
视区（`:6460-6462`）。【推断】这是「视口级懒位图」而非 Univer 的条带
diff——word 的分页模型让页成为天然缓存单元，比表格的滚动条带更粗粒
度但实现更简单。

**cell = 全画布**：`WorksheetView`（`sdkjs/cell/view/WorksheetView.js`，
30688 行）经 `DrawingContext` 持 canvas（`sdkjs/cell/graphics/
DrawingContext.js:515`，原生环境换 NativeContext）；sheet 滚动/重算均
以 canvas 绘制完成（水印、边框缓存等 `WorksheetView.js:4370,7703`）。
**slide = 画布 + WebGL 转场**（`sdkjs/slide/Drawing/TransitionsGL.js`、
Morph 过渡 `MorphTransition.js`）。三编辑器共享 `sdkjs/common/Drawings`
形状引擎（Format/Data.js 等约 40+ 文件）。

**PDF 编辑器 = 自研 wasm 渲染引擎**：`sdkjs/pdf/src/engine/` 是
`drawingfile.wasm` + JS 胶水（`pdf/src/engine/drawingfile.js:133` 加载
wasm；`viewer.js:219` 降级 `_ie.js`），C++ 侧对应 `core/PdfFile/PdfEditor.*`
——PDF 已是第四个一等编辑器（`sdkjs/pdf/api.js:65 PDFEditorApi.prototype.
openDocument`，含自己的 `pdf/src/CollaborativeEditing.js`、annotations、
forms 目录）。

**字体与文本测量 = wasm 引擎**：`sdkjs/common/libfont/engine/fonts.wasm`
（同一目录 `textshaper.js`/`textmeasurer.js`/`glyphstring.js` 是 JS 门面，
`engine.js:285-331` 直调 wasm 的 FT_Glyph 测量/矢量化回调）。仓内还有
`common/zlib/engine/zlib.wasm`、`common/hash/hash/engine.wasm`、
`common/spell/spell/spell.wasm`——【推断】**五枚 wasm 引擎全部由 core
同源 C++ 编译**（仓内 wasm 为提交的构建产物而非源码，同源关系由 sdkjs
与 builder 共用同一套构建/加载脚本佐证，
`DocumentBuilder/DoctRenderer.config:1-20`）。这与 mdc-pdf 的
native/wasm 双引擎路线同构（§13/§14）。

## 3. 编辑内核（OT 协同）

**change = 可序列化的 undo 项**（架构基石）：`CChangesBase{Class,
Undo, Redo, WriteToBinary, ReadFromBinary, Load≈Redo, Merge,
RefreshRecalcData, CheckLock}`（`sdkjs/common/HistoryCommon.js:4885-4944`），
类型注册表 `changesFactory`（:4861）+ 类型间冲突关系表 `changesRelationMap`
（:4871）。线上格式两种：二进制（`editor.binaryChanges=true` 时直接
Uint8）或 `len;base64`（`sdkjs/common/CollaborativeEditingBase.js:115-137`）；
开关由服务端配置下发（`default.json:518` `binaryChanges:false` 默认关，
`sdkjs/common/apiBase.js:1993-1995` 接收）。

**OT 机制 = 服务器全序 + 「先撤自有→按序应用外来→重放自有」**：
`Apply_OtherChanges` 依服务器顺序逐个 `Changes.Apply_Data()`
（`CollaborativeEditingBase.js:399-431`）；应用外来变更前先
`m_nUndoBeforeApply` 次 `UndoGlobalChanges` 撤销本地未发送变更
（:184,362-367），随后本地历史重放完成位置变换——**不写经典
transform 函数，用 undo/replay 换变换**。位置追踪（光标/选区/书签）
由 `CDocumentPositionsManager` 挂在变更流上（:203-217）。【推断】此
方案把 OT 难点转嫁给「undo 必须完备可逆」，正是 CChangesBase 强制
Undo/Redo 成对的原因；对 mdcore 的直接印证：**逆 op 化 undo 不是可
选项而是协同正确性的承重墙**（office-a §13-2 结论再次成立）。

**发送协议**（word 实现）：`Send_Changes` 从 History.Points 取自
SavedIndex 之后的项，序列化为 `CCollaborativeChanges.m_pData`
（`sdkjs/word/Editor/CollaborativeEditing.js:76-125`）；单人模式算出
`deleteIndex`（服务器据此截断已落库变更，单人 undo 擦历史）
（:99-107；服务端 `server/DocService/sources/DocsCoServer.js:3608-3619`）；
多人模式每次保存清空本地 undo（:161-174），与服务器头注释的保存方案
互证（`DocsCoServer.js:46-53`：单人 autosave 5 秒/多人整包保存 10 分钟）。

**悲观锁**：表格域用行列锁（`getLock` 消息，`DocsCoServer.js:3528`；
客户端 `sdkjs/cell/model/CollaborativeEditing.js:163` 的 `lock`），结构性
变更（插删行列）后服务器**重算锁区间**（`recalcIndexColumns/Rows`，
`DocsCoServer.js:3651-3664`）——表格 OT 被「锁住冲突区+重算索引」简化。
文字域锁段落/图形对象（`word/Editor/CollaborativeEditing.js:262-347` 的
`OnEnd_CheckLock`，含内容控件跳锁白名单 :402-414）。

**客户端 WS 协议**（`sdkjs/common/docscoapi.js`，1957 行）：socket.io
transports `["websocket","polling"]`，auth 携带 `data+token+session`
（JWT 双令牌，:1788-1792）；服务端→客户端消息枚举
auth/message/cursor/meta/getLock/releaseLock/connectState/saveChanges/
authChanges/saveLock/unSaveLock/savePartChanges/drop/disconnectReason/
waitAuth/documentOpen/warning/license/session/refreshToken（:1858-1918）；
客户端→服务端 openDocument/message/cursor/getLock/saveChanges/…
（对端 `DocsCoServer.js:1961-2046` 的 switch，两表互为镜像——**协议
无 schema 文件，两端 switch 即契约**）。大变更分片上行
（`_saveChangesChunks`，docscoapi.js:1184-1214）；saveChanges 可改道
`EncryptionWorker` 做端到端加密（:1006；服务端识别 `ENCRYPTED;` 前缀
`server/FileConverter/sources/converter.js:885`，加密档不走服务端明文
装配）。另有请求-响应式 `rpc` 通道（responseKey 关联：sendForm/
saveRelativeFromChanges/wopi_RenameFile/pathurls，`DocsCoServer.js:
1302-1353`）。

**版本对齐**：变更索引由服务器单调分配（`puckerIndex`，
`DocsCoServer.js:3601,3645`），客户端以 `changesIndex/syncChangesIndex`
对齐（savePartChanges 应答 :3741；客户端 `_updateChanges`
docscoapi.js:1214）；断线重连 auth 流程补拉 + `authChanges`
（`DocsCoServer.js:2598 authRestore`；:2041 `authChangesAck`）。
服务器侧变更持久化在 SQL（默认 postgres，`doc_changes`/`task_result`
两表，`default.json:370` sql 段），内存运行时退化到 editorDataMemory
（`convertermaster.js:49` 的 embedded converter 注释）。

## 3.5 三段切分专项：进程模型、IPC/协议面、信任边界

【对 Rust 自研沙箱切分最直接的对位样本，单独深写】

**进程模型**（生产形态）：
1. **DocService（Node 常驻）**：socket.io 房间编排 + Express 路由
   （`server/DocService/sources/server.js:234-362`：/command /converter
   /upload /downloadas /savefile /downloadfile /docbuilder /wopi/*
   /ai-proxy /plugins.json /healthcheck）+ 集群 pubsub（RabbitMQ，
   `default.json:35`）+ 多租户（tenantManager）。
2. **FileConverter（Node 常驻 × N）**：cluster.fork 工作进程数 =
   `ceil(availableParallelism × maxprocesscount)` **且被 license.count
   截断**（`server/FileConverter/sources/convertermaster.js:59-77`；OSS
   license 硬编码 count=1，`server/Common/sources/license.js:43-68`——
   【推断】开源版实际并发=CPU 数×1，商业版靠 license 放大，这是
   「OSS 代码完备、能力靠授权数封顶」的教科书式边界）。内存队列模式
   下 converter 嵌入 DocService 进程内跑（convertermaster.js:45-49）。
3. **x2t / docbuilder（C++ 一次性子进程）**：每个任务 spawn 一个，
   跑完即退。任务文件 IPC：x2t 收 `params.xml`（+可选 hidden.xml 存
   密码），docbuilder 收 CLI 参数（`--check-fonts/--save-use-only-names/
   --argument/--options` + 脚本路径）（`converter.js:1124-1148`）。
   生命周期：visibilityTimeout(300s) 到点先 `stdin.end()+stdout/
   stderr.destroy()+kill()`（`converter.js:1162-1171`）——**超时杀进程
   而非软中断**，因为 x2t 内部无取消机制。

**IPC/协议面汇总**：

| 边界 | 载体 | 契约 |
|------|------|------|
| 集成商↔DocService | HTTP+JWT | /command /converter /docbuilder（`server.js:234-285`） |
| 浏览器↔DocService | socket.io(WS) | 消息 switch（§3 两表） |
| DocService↔转换 | RabbitMQ 队列 | TaskQueueData JSON（`commondefines.js`） |
| FileConverter↔x2t | **XML 任务文件 + argv + 进程退出码** | `<TaskQueueDataConvert>` 字段（`converter.js:230-278`：m_sFileFrom/To、m_nFormatTo、m_bPaid、m_bEmbeddedFonts、m_bFromChanges、m_sFontDir、m_oThumbnail、m_oInputLimits、m_oTextParams、m_sJsonParams…）；退出码取负数约定（exitCodesReturn/MinorError，`converter.js:168-186`；`server/Common/sources/constants.js:256` CONVERT_CELLLIMITS=-96） |
| x2t↔JS 引擎 | 内嵌 V8/JSC + XML Settings 串 | `getDoctXml` 的 `<Settings><SrcFileType>…<Changes>`（`core/X2tConverter/src/cextracttools.cpp:774-854`）；JS 回传 `asc_nativeGetFileData` TypedArray（`core/DesktopEditor/doctrenderer/doctrenderer.cpp:315-344`）；引擎双后端 `core/DesktopEditor/doctrenderer/js_internal/v8/` + `js_internal/jsc/`（jsc_base.mm=Apple 平台）【推断】 |
| DocService↔集成商回调 | HTTP POST（callbackUrl/WOPI） | OutputSfcData 状态机（`canvasservice.js:1405-1468`） |

**信任边界与隔离手段**（x2t 是唯一直面敌意文件的段）：
- 内存上限：env `X2T_MEMORY_LIMIT` 默认 4GiB → `limit_memory()`
  （RLIMIT，非 debug/移动端生效）（`core/X2tConverter/src/main.cpp:
  100-109`；`core/DesktopEditor/common/SystemUtils.h:64-65`）。
- zip 炸弹：`checkInputLimits` 按格式族比对解压后总量（docx/pptx/vsdx
  50MB、xlsx 300MB，模板 `*.xml`）（`cextracttools.cpp:939-963`；
  `default.json:604-629` inputLimits 段；序列化进任务
  `converter.js:276,377-404`）。
- 下载配额：maxDownloadBytes 100MB、超时 2m、重试 3 次（`converter.js:
  510-518` 及 default config §FileConverter）。
- 路径逃逸：`utils.checkPathTraversal`（`converter.js:1212`）。
- 出网控制：FileTransporter 默认放行、env `allowNetworkRequest` 可关
  （`core/Common/Network/FileTransporter/src/FileTransporter_private.h:
  244-261`）；html/epub/ofd 内嵌图片拉取有 `allowPrivateIP` 私网守卫
  （`core/EpubFile/src/CEpubFile.cpp:204-205`、`core/OFDFile/src/Utils/
  Utils.h:185-186`、`core/HtmlFile2/htmlfile2.cpp:1579-1580`）；选项由
  任务 XML `<options>` 节点灌入（`cextracttools.h:683-685` →
  `core/DesktopEditor/common/ProcessEnv.h:44-49`）。
- **结构性缺口【推断+事实】**：x2t 以 FileConverter 同一 OS 用户裸跑，
  `spawnOptions` 默认空对象继承全环境（`converter.js:1152-1155`），无
  seccomp/容器/chroot——隔离手段只有「内存限额+超时杀+下载配额」三
  件套。这与 x2t 内嵌 V8 执行 sdkjs（任意 JS 能进该进程的执行面）
  叠加后，转换进程沦陷=服务账户沦陷。**对 Rust 工作台的反向结论：
  转换/解析子系统必须独立 jailed 进程（或 wasm 线性内存隔离），且
  不得在同一进程嵌 JS 引擎**——ONLYOFFICE 用 20 年时间证明了三件套
  「够用但不优雅」。
- 崩溃即状态：x2t 崩溃通过非零/负退出码与 errorfiles 上传落档
  （stdout/stderr+ExitCode 写 console.txt，`converter.js:985-1027`），
  不影响主进程稳定——**进程隔离的第一收益是崩溃域隔离**。

**DocService 信任边界**：对浏览器不信任（session JWT + view 用户拒绝
getLock/saveChanges，`DocsCoServer.js:1951-1959`）；对集成商半信任
（checkClientIp 过滤 + 可选 inbox JWT，**默认 token.enable 全 false**
（`default.json:474-480`）——【推断】开箱即用的 DocumentServer 是
「内网部署假设」，JWT 是生产建议而非默认强制）；对 x2t 输出不信任
（退出码/产物校验 + rollback 重试，§4）。

## 4. 格式进出保真（核心维）

**转换架构 = 万物过二进制枢纽**：`fromInputParams` 按方向枚举 TCD_*
分派（`ASCConverters.cpp:1683-1852`：docx2doct/doct2docx/xlsx2xlst/
pkg2bin/pptx2pptt/vsdx2vsdt…），所有「源格式→内部二进制」「内部二进制
→目标格式」对偶函数构成转换矩阵；直转对（如 docx→pdf）内部仍是
docx→doct→pdf 两跳（`fromDocxDir` 判 `bIsNeedDoct`，`ASCConverters.cpp:
290-309`）。**这不是「无格式损失」的设计，而是「以内部模型为交换标
准」的设计**——保真上限 = 内部模型的表达力。

**OOXML 读入（C++ 主路径）**：OOX 类型化元素模型（`core/OOXML/
DocxFormat/` 目录：Document/Paragraph/Drawing/…逐元素 cpp/h）解析后由
`BinDocxRW::CDocxSerializer` 折进二进制（`core/OOXML/Binary/Document/
DocWrapper/DocxSerializer.h:53-92`；写侧 `saveToFile` 走
`BinaryFileWriter.intoBindoc`（DocxSerializer.cpp:204,255），读回
`loadFromFile` 走 `BinaryFileReader.ReadFile()`（:323,429-430））。邮件
合并路径完整展示「bin→docx 目录→zip(+MSCrypt)」流程
（`ASCConverters.cpp:218-245`）。XLSX/PPTX/VSDX 各有同名 Serializer
（`core/OOXML/Binary/Sheets|Presentation/`）。

**保存 = 从变更重建，而非增量写**：`apply_changes` 在 DoctRenderer 里
让 sdkjs 打开旧 bin、按 `changes/changesN.json` 顺序应用、重新全量序列
化为 `XWithChanges.bin`（`cextracttools.cpp:856-915`；变更清单由 Node 端
从 SQL 分批拉出写成 JSON 数组，`converter.js` 的 processChanges，含
changesHistory.json 作者轨迹）。**坏变更降级策略**：应用出错且错误串
带 index 时，跳到该 index 重试（:879-900）——单条损坏变更不废整次
保存。媒体文件从 changes 提取回填 changes.zip（:905-937）。

**「无格式损失」承诺的工程支撑面**（事实登记）：
1. **assemblyFormatAsOrigin=true 默认**（`default.json:345`）：编辑过的
   文档按原格式回装，不强制转 docx/xlsx/pptx。
2. **失败回滚**：从变更装配非 OOXML 目标失败时自动回退为 OOXML 输出
   （`converter.js:1273-1292`，日志明言 "rollback to save changes to
   ooxml. See assemblyFormatAsOrigin"）。
3. **宏与 VBA 保留**：`m_bMacro/doct2docm`（`ASCConverters.cpp:1839-1845`）；
   VBA 以 vbaProject.bin 原样搬运（`core/MsBinaryFile/Common/Vba/
   VbaReader.h`；`-detectmacro` CLI 探测，`main.cpp:155-161`）；JS 宏内嵌
   二进制并协同分发（`sdkjs/common/macros.js:74-95` 的
   Write/Read_ToBinary2）。
4. **加密档处理**：OfficeCryptReader（RC4/Agile/ECMAStructured）解出
   明文转换、保存时 `dir2zipMscrypt` 重新加密（`ASCConverters.cpp:242`）；
   密码走 hidden.xml 不落主任务文件（`converter.js:283-296`）。
5. **语义限额**：xlsx 行/单元格超限转 `CONVERT_ROWLIMITS/CELLLIMITS`
   （`core/OOXML/Binary/Sheets/Reader/BinaryWriterS.cpp:9160`）——宁可
   拒绝也不静默截断。

**往返 quirks 处理面**：版本历史保留每次 Editor.bin+changes.zip（可回
滚到任意版本重放）；`m_sConvertToOrigin`（converter.js:260）；csv 编码/
分隔符、LCID、时区戳（任务 XML 字段 `converter.js:239-258`）传参保语
义；PDFA/oform 扩展 PDF 按源格式升级（`changeFormatToExtendedPdf`，
`converter.js:450-465`）。【推断】**模型必然有损**：内部模型是显式
类型树，OOXML 的未知元素/扩展 part（第三方插件写入的自定义 XML、
grab-bag 类属性）没有「原样保留」通道（全仓 grep 无 keep-raw/
preserve-unknown 机制，DocxSerializer 逐已知元素写入）——与
LibreOffice 的做法对照是明确取舍：**换格式必有损，但同格式编辑往返
（docx→编辑→docx）因 assemblyFormatAsOrigin+全量重序列化而稳定**。
对 mdcore：若做 OOXML 出口，需显式设计「未知 part 透传仓」才能超越
此点。

## 5. 协同协议（与保真约束共存；CRDT 对照留给 collab-a，此处只登记事实）

- **OT 与保真的共存机制**：变更只操作内部模型对象（Id 寻址），序列
  化为二进制/基64；装配时以服务器全序重放（§4 apply_changes）——
  **保真不被协议承诺，被「装配=确定性重放」承诺**：同一份 bin+同一
  变更序列在任何机器上装配结果一致（x2t 内嵌 V8 跑同一 sdkjs 代码，
  `DocumentBuilder/DoctRenderer.config:1-20` 保证浏览器/服务器同代码）。
  【推断】这是 ONLYOFFICE 用「同代码双执行环境」替代「协议级语义
  规范」的隐性设计契约——Rust 自研对应物是「同 crate 双编译目标」。
- 快慢双模式：`m_bFast`（`CollaborativeEditingBase.js:212`）+ 服务器
  单人/多人保存方案（`DocsCoServer.js:46-53` 头注释）。
- 集群一致性：pubsub 广播变更（RabbitMQ，`cfgPubSubMaxChanges` 超限
  只发通知不带载荷，`DocsCoServer.js:3683-3689`）；SHARD_ID 随机分片
  标识（:187）；`internal/cluster/inactive` 优雅停机路由
  （`server.js:287-291`）。
- 遗失变更兜底：forgotten files 吸收失主变更，下次打开优先用 forgotten
  （`canvasservice.js:580-586`；方案注释 `DocsCoServer.js:55-61`）。
- 端到端加密协同：change 走 EncryptionWorker，服务器只见密文
  （§3）；加密档的开档走 `reopen+NeedPassword` 状态机
  （`canvasservice.js:607-660`）。
- presence/聊天/评论/光标：cursor/message/meta 消息（§3 协议表）；
  WOPI 客户端模式给 Collabora 兼容端点（`server.js:336-340` 的
  /lool/convert-to、/cool/convert-to）——生态兼容策略。
- 【登记】仓内无任何 CRDT 实现（Yjs/Automerge 零引用）；协同数学
  全部押注「服务器全序 + undo/replay」。与 Yjs 路线的优劣对比按任务
  分工留给 collab-a 报告。

## 6. 插件与扩展边界

**插件宿主**（登记为外部事实）：iframe 宿主在 web-apps（未克隆）；
sdkjs 侧接口面完备可证：`asc_pluginRun/pluginStop/pluginButtonClick`
（`sdkjs/common/apiBase.js:3669-3684`）、函数体序列化进沙盒执行的
`callCommand`（scope 注入 + `value.toString()`，`apiBase.js:5900-5920`）、
插件方法面 `pluginMethod_GetVersion/AddOleObject/PasteHtml/GetMacros/
SetMacros/GetVBAMacros/StartAction…`（`sdkjs/common/apiBase_plugins.js:
205-535`）。插件清单由服务器静态目录 + `/plugins.json` 下发并 fs.watch
热更（`server.js:373-390`）。官方插件库 sdkjs-plugins @06c4b85 为示例
集（helloworld/搜索替换/内容控件/外部监听器等，README 列举）。

**宏 = 文档内嵌 JS，shadow-realm 执行**：宏存进文档二进制并参与协同
（`sdkjs/common/macros.js:74-95`）；`runAuto` 支持 autostart 宏（:107-126）。
执行环境 `_safePluginEval`（:443-510）构造影子作用域：
`Function:{}`（函数构造器封死）、`window/document/self/globalThis:{}`、
`XMLHttpRequest` 换包装版、`setTimeout/setInterval` 换安全版、直接
`import()/import` 语法拒绝（:484-493）、`eval` 用属性 getter 放行一次
后变桩防逃逸（:536-570）、generator.next 短路（:454-456）、执行期
`AscFonts.IsCheckSymbols` 冻结字体面（:579-590）。
**网络边界**：宏 XHR 每请求过权限回调，拒绝时报 "User doesn't allow
this request."（:330-436）。【推断】这是**同 realm 影子作用域而非隔离
环境**——防御目标是「误伤」而非「恶意」；与 mdcore 宏面对照：Rust
工作台应做真隔离（独立 wasm 实例/Worker + 显式能力 API），并把
「每请求用户授权」这个交互模式抄走。

**VBA 边界**：`pluginMethod_GetVBAMacros` 只读取展示（`apiBase_plugins.js:
482`），VBA 无执行引擎【推断：全仓无 VBA 解释器；宏运行时只有 JS】；
DDE 之类动态数据交换不存在（Office 家族特性未复刻，外部数据只有
`ExternalDataLoader`/OData 连接器面，`sdkjs/common/ExternalDataLoader.js`）。

## 7. AI 调用面

**服务器内建 AI 代理（一手证据，本篇新发现）**：DocService 有
`/ai-proxy` 路由（`server.js:362`）+ `ai/aiProxyHandler.js`（395 行）：
浏览器/AI 插件 → DocService 会话 JWT 校验（`aiProxyHandler.js:170-188`）
→ **配置源 allowlist 匹配**（origin 精确相等防 userinfo-SSRF 与子域
绕过，:207-218）→ 按厂商注入密钥（Anthropic `x-api-key`、Gemini query
key、默认 Bearer，:119-138）→ 可选出网代理改写（:241-269）→ 流式
pipeline 转发（:305-310）→ statsD `coauth.aiProxy` 计时（:336-338）。
插件设置端点把 provider key 清空后才发客户端（`getPluginSettingsForInterface`
:359-389）；配置面 `aiSettings{actions,providers,customProviders,models,
timeout,allowedCorsOrigins,proxy}`（:48-54；`default.json` providers 默认
空——管理员侧填 OpenAI/DeepSeek/Anthropic/Gemini/Groq/xAI/Mistral/
Together 等，见 :133 注释枚举）。**密钥永不进浏览器**是这套设计的
核心约束，与 mdcore 的 AI 代理需求直接同构。

**Document Builder = 脚本化生成 office 文件的最成熟先例**：
- C++ API：`CDocBuilder{OpenFile/CreateFile/SaveFile/CloseFile/
  ExecuteCommand/Run/RunTextW/RunTextA/SetProperty}`，参数
  `--use-doctrenderer-scheme/--check-fonts/--cache-scripts/
  --save-use-only-names/--argument(JSON)/--fonts-*`，跨文档数据交换走
  `GlobalVariable`（`core/DesktopEditor/doctrenderer/docbuilder.h:
  349-479`，注释极详尽）。
- 脚本层：builder 脚本（.docbuilder，JS 语法）跑在 DoctRenderer 的
  V8/JSC 里，全局 `Api` 即 builder API——`sdkjs/word/apiBuilder.js`
  33489 行定义 Api/ApiDocument/ApiRange/ApiParagraph…及
  `CMarkdownConverter`（:271，Markdown→文档转换器内建）；cell/slide
  各有 apiBuilder.js（29634 行级）。**builder 与编辑器共享同一模型
  代码**（DoctRenderer.config:1-20 加载 sdk-all.js）。
- 服务器化：`/docbuilder` 端点接受 URL 或请求体内联脚本
  （`server/DocService/sources/converterservice.js:460-525`，脚本存
  `script.docbuilder`），spawnProcess 以 builderParams 分支启动
  docbuilder（`converter.js:1139-1148`）。
- 语言绑定：Python/Java/.NET/COM 官方封装（`core/DesktopEditor/
  doctrenderer/docbuilder.python|.java|.net|.com` 目录）。
- **限制面**（工程事实）：单实例同时只持一个文件（CloseFile 才能换，
  docbuilder.h:399-420）；保存=全量重序列化（无增量写）；脚本对宿主
  进程全权限（同 x2t 信任模型）；Readme 能力表标注 Document Builder
  Service/Automation API 仅 Developer Edition 打包
  （`DocumentServer/Readme.md:89-90`）——【推断】AGPL 仓代码完备，
  商业差异在发行打包与授权（§11），builder 二进制是否随 Community
  Docker 分发未在仓内可证，登记为不确定项。
- **对 AI 生成 office 文件的启示**：LLM 产出 .docbuilder 脚本或直接
  调 builder API = 「模型→结构化中间表示→布局引擎→OOXML」流水线，
  优于让 LLM 直接吐 XML/HTML；mdcore 的 AI 面若做「op 流式喂入 +
  builder 式高级 API」即站在这个先例上。

## 8. 性能与资源边界

**转换侧（可观察事实）**：
- 并发模型：converter 工作进程数 = CPU × maxprocesscount(默认 1)，
  license 截顶（`convertermaster.js:59-77`）；单任务独占一个 x2t 进程；
  队列 rabbitmq + visibilityTimeout 300s + retentionPeriod 900s
  （`default.json:35` 附近）+ 死信重投 maxRedeliveredCount=2（converter
  配置）；`handleDeadLetter` 补偿（`DocsCoServer.js:1354-1361`）。
- 资源三件套：内存 4GiB（§3.5）、zip 解压限额（§3.5）、下载 100MB/
  2m/3 次（§3.5）；上传图片限额 25MB/类型白名单（`default.json:
  325-326`）。
- 全链路 statsD 打点：conv.downloadFile/downloadFileFromStorage/
  spawnSync/postProcess/deleteFolderRecursive/allconvert + coauth.
  data.auth/getLock/saveChanges + coauth.openDocument.* + coauth.aiProxy
  （`converter.js:1231-1313`；`DocsCoServer.js:2049-2053`）——**性能
  观测是一等公民**，但仓内无 benchmark 套件（诚实登记：仅
  `server/tests/perf/checkFileExpire.js` 一个冒烟脚本）。
- 语义限额：xlsx ROWLIMITS/CELLLIMITS 拒绝式（§4）。
- 协议级背压：websocketMaxPayloadSize 1.5MB、maxChangesSize 150MB、
  maxRequestChanges 20000（`default.json:514-520,340`）——大变更集分片
  + SQL 分页。

**编辑侧（代码级推断，标注）**：
- word 全量重算粒度=页缓存失效 + 懒重绘（§2），结构变更触发
  RefreshRecalcData 链【推断：无增量排版引擎证据，排版重算以段落/
  页为单位整块标脏——`HistoryCommon.js:4924-4928` 的
  RefreshRecalcData 钩子】。
- 字形/测量走 wasm（§2），AllFonts.js 预编译字体表（x2t
  `-create-allfonts`，`main.cpp:172-195`）。
- 【推断】单文档内存模型上限 ~百 MB 级（150MB 变更上限反推），千万
  行级表格不在设计目标内（CELLLIMITS 佐证）。

## 9. 敌意输入与执行边界（x2t 与 DocService 两层分别登记）

**x2t 层**（直面敌意文件）：
- zip 炸弹：inputLimits 解压总量比对（§3.5，`cextracttools.cpp:939-963`）；
  下载字节/超时/重试配额；内存 RLIMIT 4GiB；超时 SIGKILL。
- 格式混淆：三方嗅探（§1）；服务端 `getDocumentFormatByFile` 复核
  （`converter.js:450-465` changeFormatToExtendedPdf 内调用）。
- 崩溃域：独立进程 + 退出码语义 + errorfiles 取证上传（`converter.js:
  985-1027`）+ `maxRedeliveredCount` 死信。
- 出网：默认允许、env 可关 + 私网 IP 守卫（§3.5）——【推断】默认
  方向是「可用性优先」：文档引用外链图片要能拉，SSRF 面靠管理员
  配置收敛。
- **无 OS 沙箱**（§3.5 结构性缺口，重复登记于此维）。
- DRM/保护文件：CONVERT_DRM/DRM_UNSUPPORTED/NEED_PARAMS/PASSWORD 退出
  码族（`converter.js:178`），`openProtectedFile` 配置控制是否强制密码。

**DocService 层**：
- JWT 默认全关（`default.json:474-480`）——**开箱即用即无签名校验**
  （生产硬化靠 `services.CoAuthoring.token.enable.*`），诚实登记为部署
  风险面。
- 路由面收敛：`utils.checkClientIp` 过滤管理类端点（`server.js:234-291`）；
  上传类型/尺寸白名单（§8）；`checkPathTraversal`（§3.5）。
- SSRF 防线：下载走 `utils.checkHostFilter`（`converter.js:520`，
  requestSSRF 专项单测存在 `server/tests/unit/requestSSRF.tests.js`——
  有测试背书的守卫）。
- 多租户隔离：tenantManager 按 tenant 分 storage/配置/license
  （`DocsCoServer.js:105`）；storage-fs 默认 + S3/Azure 适配器
  （`server/Common/sources/storage/` 目录）。
- 会话治理：view 用户禁写路径（§3.5）、session idle/absolute 过期
  配置（`DocsCoServer.js:127-128`）、`sendDataDisconnectReason` 断链
  理由码体系。
- 浏览器层：宏 shadow-eval 弱沙箱（§6，登记为最大浏览器侧执行
  边界）；E2E 加密使服务器对内容不可见（§3）。

**汇总判断【推断】**：ONLYOFFICE 的安全预算花在「进程外转换 + 配额
+ 退出码」，而非「强隔离」；对抗性场景（处理不可信第三方文档）下
其部署要求依赖外部加固（容器/网络策略）。Rust 自研若做 jailed
converter + 无宏默认开，即在此维全面超越。

## 10. 质量体系

- **server**：jest 单测 15 文件（docsCoServer/converterPaths/
  converterTimeout/mailService/memoryGuard/pdfSigning/request/
  requestSSRF/runtimeInproc/runtimeMemory*/runtimeStandaloneSmoke/
  sample/utils/utilsDocService，`server/tests/unit/`）+ 集成测试目录
  （withServerInstance）+ 存储 CI（azure/fs/s3 三个 workflow）+
  lint-format CI + **CodeQL 每周六全仓**（cron `0 0 * * 6`，
  `server/.github/workflows/codeql.yaml`）。
- **core**：CodeQL workflow；`core/Test/Applications/` 为手工测试程序
  集（AVSOfficeEWSEditorTest/OCRTest/WSHtmlFileTest/DocxFormatCodeGen/
  MemoryLimit/ParseLogs 等）+ `Test/CoAuthoring/`；X2tConverter 自带
  win32Test/qmake/TestOOOXml2Odf 测试工程——**C++ 无自动化单测体系，
  靠测试程序 + 海量手工回归**（Test 目录 23MB 语料）。
- **sdkjs**：**仓内无单测体系**（无 jest/vitest 等 spec 套件；`pdf/test/`
  为浏览器手动测试页——index.html + 手写 js 脚本，非自动化断言；其余仅
  build 工具 + `sonar-project.properties` SonarCloud 接入 + jshint 配置）；
  CI 只有 check-build.yml（构建冒烟）+ codeql.yaml——【推断】sdkjs 的质量门
  在发布流程外围（QA 部门 + Sonar），仓库自身门禁弱于 server 仓，
  是六仓中测试债最重的一段。
- **版本纪律**：DocumentServer CHANGELOG 9.4.0（2026-07-22）与
  core/sdkjs/server 的 2026-05-19 同步指针构成 release train；每仓独立
  CHANGELOG；`core/Common/version.txt` 2.5.565.0；server package.json
  1.0.1（版本号语义分离——聚合仓版本才是产品版本）。
- **供应链**：npm-shrinkwrap 锁版本；3DPARTY.md/3rd-Party.txt/
  3d-party-lic-report（SBOM 报告目录）三份许可清单（server 仓）。

## 11. 净室合规

**许可证核验（一手）**：五主仓（DocumentServer/core/server/sdkjs/
DocumentBuilder）LICENSE = GNU AGPL-3.0 全文 + Ascensio **§7 附加条款**
（`server/LICENSE.txt:663-727`，五仓同款）；**第六仓 sdkjs-plugins 例外
= Apache-2.0**（`sdkjs-plugins/LICENSE` 首页，示例插件仓不携带 AGPL
附加条款）。§7 附加条款五条内容：
①保留版权/归属/法律声明；②修改版必须显著标注修改日期与「基于
ONLYOFFICE (Ascensio)」；③UI 必须提供可访问的 Appropriate Legal
Notices；④不授予商标权（引 Trademark Policy）；⑤非代码内容（插画/
图标/文案）CC BY-SA 4.0。每源文件头部 SPDX `AGPL-3.0-only`。
【推断】§7 附加条款均为「澄清性/署名性」条款（AGPL 允许的 additional
terms），未削减四项自由；但对**再分发者**构成署名与 UI 提示义务，
fork 改造时是真实合规成本；对我们（只读净室）零义务。

**「100% 开源」声明核验（对 2026-06 博客）**：博客《The year we went
open source》（2026-06-29，作者 Dasha，canonical URL
onlyoffice.com/blog/2026/06/the-year-we-went-open-source/，发布元数据
2026-06-29T15:15:41+00:00）原文："We officially opened the source code
of the entire application under AGPL v3, making ONLYOFFICE a 100% open
source solution"——**语境是 2014 年的历史回顾**（16 周年系列），且明
确指「当时的整套应用」；博客另载 Desktop Editors 2016 年才开源。
**当前实际状态（仓内一手证据）**：
- 开源（AGPL+附加条款）：x2t 内核、sdkjs 四编辑器、Node 服务端、
  web-apps（公开仓）、DocumentBuilder、字体/词典/模板
  子模块（DocumentServer .gitmodules 全 9+ 指针均为 ONLYOFFICE 公开
  仓）；示例插件仓 sdkjs-plugins 为 Apache-2.0（见本节首段）。
- **闭源/商业残留**：①Mobile web editors——Community Edition 无
  （`DocumentServer/Readme.md:98`，EE/DE 才有）；②Document Builder
  Service 与 Automation API 打包仅 Developer Edition（:89-90）；③
  White label（:95）为 **DE-only**（表值 -/-/+）；Admin Panel（:97）、
  Clusterization（:80）为 EE/DE；④EE/DE 许可为 Proprietary（:81）——即**「100% 开源」对
  桌面/网页编辑器核心成立，对全家桶不成立**；官方表格自己就是边界
  声明。⑤OSS license.js 硬编码 PACKAGE_TYPE_OS + 连接数无上限
  （`server/Common/sources/license.js:43-73`；
  `server/Common/sources/constants.js:94-95` LICENSE_CONNECTIONS_OS=
  0x7fffffff），商业能力（advancedApi/branding=false）在 OSS 分支被
  显式关掉——**代码同源、特性由 license 值切换**的双轨制。
- 内嵌第三方：core 3dParty（cryptopp 等，`core/3DPARTY.md`）、server
  依赖全宽松许可（`server/3DPARTY.md`：aws-sdk/amqplib/axios/
  underscore…）；npm shrinkwrap 锁定。
- **对净室路线的定性**：只读分析 + 行为模仿零义务（AGPL 触发点在
  分发衍生作品/网络服务化其代码）；未来若考虑「借鉴其转换矩阵经
  验」实现 Rust 版，接口级/格式级知识不构成衍生；**不建议**直接
  引用其代码（AGPL 传染 + 附加条款署名负担）。与 office-a（Apache）
  对照，ONLYOFFICE 的许可对二次开发者显著更重——这是它作为「开源
  但商业友好度低」样本的价值。

## 12. 能力矩阵（dsh 插件面 + MCP 工具面双口径）

| 能力 | OSS 仓内（实证） | 闭源/打包层（Readme 声明） |
|------|----------------|--------------------------|
| 程序化读写 | builder API（apiBuilder.js 33k 行）+ docbuilder CLI + `/docbuilder` 端点（converterservice.js:460） | Document Builder Service / Automation API 仅 DE 打包（Readme.md:89-90） |
| 无头运行 | docbuilder 二进制 + DoctRenderer（与编辑器同代码） | —— |
| 输出验证 | `-create-js-snapshots`、缩略图 m_oThumbnail、forcesave 回调 | —— |
| AI 面 | /ai-proxy 服务器代理（密钥不出服务器）+ aiSettings 多厂商配置 | 移动端 AI（闭源 mobile web editors） |
| MCP 工具面 | **零产品实现**（六仓零 MCP 产品代码；唯一仓内引用=core 测试程序一处 github-mcp-server 镜像调用 `core/Test/Applications/process/main.cpp:76`；agent 面=REST/builder 脚本） | 官方 roadmap 已列 AI 插件 MCP 支持（`DocumentServer/ROADMAP.md:22`）——窗口收窄信号 |
| 插件面 | plugins.json 静态分发 + iframe 插件 + callCommand | 插件市场分发渠道（站外） |
| 协同 | 全量 OT（OSS 完备：docscoapi + DocsCoServer） | 集群化（EE/DE） |
| 格式进出 | x2t 全矩阵（OSS 完备） | —— |

**口径结论**：与 Univer 相反（Univer 把导入导出/协同服务端全放 Pro），
ONLYOFFICE 把**编辑器+转换+协同内核全开源**，闭源的是移动端 UI 与
打包层分发——「内核开源、外围收费」。但两者 MCP/agent 原生工具面
都是空白：办公套件界尚无「内核内置 MCP 服务」先例，mdcore 若做
（mdshell host-only 面）仍是差异化空位。**【窗口期风险登记】该空位
并非无限期**：ONLYOFFICE 官方 roadmap 已载 "Support for MCP and web
search in the AI plugin"（`DocumentServer/ROADMAP.md:22`），HyperFormula
的 MCP/AI SDK/LangChain 三外置面也全部挂 coming soon（office-c §10）
——「内核内置 MCP」在两大对标件均已进入官方 roadmap 排队，差异化
窗口应按「roadmap 已列、交付未发生」口径登记（与 office-c 对同级
信号的登记力度对齐）。

## 13. 对工作台的可移植结论（按优先级）

1. **三段切分照搬为「 jailed 转换进程 + 编排服务 + 前端引擎」**：
   x2t 的「XML 任务文件 + 退出码 + 超时杀 + 内存限额」四件套是被
   20 年生产验证的最低成本 IPC；Rust 版直接升级：转换器为独立进程
   + seccomp/landlock + JSON 任务（弃 XML）+ 结构化错误码。**禁止在
   转换进程内嵌 JS 引擎**（ONLYOFFICE 的 x2t 内嵌 V8 是其安全面最
   大结构性弱点）。
2. **输入限额三件套照抄**：按格式族的解压总量上限（compressed+
   uncompressed+文件名模板）、下载字节/超时/重试配额、语义限额
   （行/单元格数拒绝式）——三件全部进 mdc-pdf/pdf-line 已有的敌意
   输入清单（与 pdf-line/03 报告互补）。
3. **change = 可序列化逆操作**：CChangesBase 形状（Type id + 目标
   Id + Undo/Redo + Binary 双向 + Merge + RefreshRecalc）可直接映射
   为 Rust enum Op + trait OpSerde；协同正确性依赖「undo 可完备反
   演」，mdcore 的 apply_ops 返回逆 op 设计（office-a §13-2）与
   此互证——**两条独立证据链指向同一结论，升级为强结论**。
4. **「服务器全序 + undo/replay」OT 的成本模型**：无需写 per-type
   transform，代价是每次外来变更批次要撤销/重放本地未发送操作——
   在 op 粒度小、批次频繁的 md 文档上成本可控；mdcore 若走此路线
   可完全避开 ot-json1 的类型体操（与 office-a 的 TextX/JSONX 路线
   二选一，留 collab-a 裁决）。
5. **保真策略移植**：assemblyFormatAsOrigin（存回原格式）+ 失败回退
   OOXML + 全量重序列化保存 + 「坏变更跳 index 重试」的装配降级
   ——四条策略全部是纯工程决策，可直接进 mdcore 的 OOXML 出口
   设计文档；并补 ONLYOFFICE 缺失的「未知 part 透传仓」实现超越。
6. **每页位图缓存 + 懒渲染**（word 路线）适配 mdcore 的 PDF 页面
   视图：CDrawingPage{cachedImage,IsRecalculate,RecalculateTime} +
   可视窗口 first/end 索引 + CacheManager 解锁/复用——比 Excalidraw
   per-element 缓存更适合页式文档（与 canvas-a 结论拼成双场景手册）。
7. **wasm 引擎族模式**：fonts/zlib/hash/spell/pdf-draw 五引擎【推断】
   从同一 C++ 源编译 native+wasm 双目标（同源依据见 §2）、
   DoctRenderer.config 声明式加载
   ——mdcore 的 mdc-pdf 双引擎（engine_pdfium native + wasm）已是
   同构，可扩展到字体/压缩/拼写三枚引擎统一治理。
8. **AI 代理安全模式**：会话 JWT + provider origin 精确匹配（防
   userinfo-SSRF/子域绕过）+ 密钥服务端持有 + 设置端点脱敏
   （getPluginSettingsForInterface 模式）——mdcore 的 AI 网关照此
   检查单实现。
9. **宏面反例**：shadow-realm eval 不抄；「每网络请求用户授权」的
   交互模型抄；真隔离（独立 wasm/Worker + 能力化 API）才是目标态。
10. **license 双轨制参考**：OSS 分支代码完备、特性由 license 值切换
    （advancedApi/branding=false、converter worker 数封顶）——
    mdcore 若有付费层，「代码全开 + 运行时授权开关」比「闭源仓库
    分叉」维护成本低得多。

## 14. 与工作台架构的对照（一页）

| 决策点 | ONLYOFFICE 选择 | mdcore 现状/规划 | 取舍建议 |
|--------|----------------|----------------|---------|
| 真相源 | 私有二进制（DOCY 族） | op-first + snapshot | 坚持 op-first；若加缓存层学其「头魔数+版本」显式自描述 |
| 模型 | 全内存对象树+Id 注册表 | 树结构+anchor | TableId 的 Id 寻址可借（协同寻址主键） |
| undo | change 即 undo 项 | apply_ops 返回逆 op | 两条证据链互证，按现设计收口 |
| OT | 服务器全序+undo/replay+悲观锁 | 单机 | collab-a 与 Yjs 对照后裁决；表格域锁+索引重算是低成本表格协同 |
| 转换 | x2t 独立进程+内嵌 V8 | mdc-pdf 双引擎 | 抄进程隔离、弃内嵌 JS、加 OS 级 jail |
| 格式策略 | 内部模型枢纽+原格式回装 | PDF 进出已定 | OOXML 立项时抄 4 条策略+补未知 part 透传 |
| 渲染 | 每页位图缓存（word）/画布（cell/slide） | 双 canvas | 页式文档用页位图缓存路线 |
| AI 面 | 服务器 AI 代理+builder 脚本 | mdshell host-only | AI 网关检查单照抄；MCP 原生面仍是我方空位（窗口收窄风险登记见 §12） |
| 宏 | shadow-eval 同 realm | 未规划 | 反例登记；真隔离+能力 API |
| 许可 | AGPL+附加条款 | 私有仓 | 只读净室零义务；不引代码 |

## 附：证据锚点密度统计

本报告合计反引号证据锚点 **130 处实例（去重 122 处；重复引用 8 处＝
跨节 7 + §12 表/文同锚 1）**——计数口径统一为「含 `:行号` 的反引号锚点实例，代码围栏与
URL 不计；§10 的无行号纯路径锚（`server/tests/unit/`、`codeql.yaml`
等）与 §12 表内裸引不计入分项」；分项（实例口径）：§0 3 / §1 15 /
§2 12 / §3 18 / §3.5 27 / §4 14 / §5 9 / §6 4 / §7 5 / §8 8 / §9 8 /
§10 0 / §11 4 / §12 3，另头部 sha 六处、外部一手源两处。密度与
office-a（~110 处）同量级偏上；六仓关键路径（DocsCoServer 消息循环、
converter spawn 协议、x2t fromInputParams/apply_changes/
checkInputLimits、sdkjs 序列化与 OT、macros 沙箱、aiProxyHandler、
docbuilder API、default.json 配置面）均有一手行号锚定。【推断】显式标注
22 处（含本轮按复审处方补标的 3 处：头部跨格式往返、§2 wasm 同源、
§13-7 同款引用）。复审：已完成独立复审（r3-review-b，2026-10-11：
反引号锚点抽验 20/20 + 定向补验 30+ 全中），其必改 2 项（§11 六仓许可
口径、§12 MCP 表述）与 7 项轻微已按处方修补入本版（sdkjs-plugins=
Apache-2.0 / MCP 零产品实现 + ROADMAP 窗口信号 / 【推断】补标与计数 /
White label DE-only / §0 插件 11→10 / §10 pdf/test 手动测试页 /
附录计数口径统一 / aiProxyHandler 395 行）。
