# canvas-d：ComfyUI 协议 pass（四问题定向）

> **净室分析报告**（线 A 画布 P0 第二篇）。日期：2026-10-10。
> 对象：comfyanonymous/ComfyUI @ `0df64eb`（master 浅克隆）。许可证：**GPL-3.0**
> （LICENSE，674 行标准全文，已核，规划期"印象 GPL-3.0 待核"就此定谳）。
> 方法：两路并行勘察代理（A：图模型+执行引擎 / B：通信协议+敌意输入）
> + 主代理交叉综合；结论标注 `路径:行号` 证据锚点，【推断】= 分析员判断。
> 路径均为仓库根相对路径（`execution.py`/`server.py`/`nodes.py` 等在根目录，
> `comfy_execution/` 下为 graph/caching/validation/graph_utils）。报告已经
> 独立复审代理（canvas-d-1r，~50 锚点抽查）审定后修订定稿。
> 定位：协议 pass 而非 11 维深读——回答规划设定的四问题（graph 模型 /
> 执行 / 通信 / 敌意输入），为工作台「进程外集成 ComfyUI」提供协议面与
> 安全边界的一手输入。
>
> 一句话定性：**零认证、本地优先的节点图执行服务——编辑图与执行图彻底
> 分离（后端从不解析画布 JSON），merkle 式祖先签名缓存 + 增量拓扑溶解
> 调度是引擎精髓；安全模型显式把信任外包给部署者与节点作者**。

## 0. 架构速览

```
Python 单进程 aiohttp 服务
  ├─ server.py          PromptServer：全部公开路由（原路径 + /api/* 镜像双前缀）
  ├─ execution.py       调度主循环所在（根目录）：PromptQueue / validate_prompt
  │                     编排 / PromptExecutor.execute / execute_async 主循环
  ├─ api_server/        仅 /internal 子应用（前端专用：logs/folder_paths/files）
  ├─ comfy_execution/   图结构+缓存+验证（graph.py 拓扑溶解 / caching.py 双缓存 /
  │                     validation.py 连接类型与环检查）
  ├─ comfy/             运行时（model_management.py 含中断原语与显存管理）
  ├─ nodes.py           内置节点 + NODE_CLASS_MAPPINGS 注册 + custom node 装载
  ├─ app/               资产系统(alembic_db)/用户管理/子图蓝图/governance(签名管控)
  └─ custom_nodes/      任意 Python（官方安全边界之外）
```

## 1. Q1 图模型

### 1.1 双 JSON 格式：编辑图与执行图彻底分离

- [事实] 后端只消费**执行格式（prompt/API JSON）**：扁平 map
  `{"<node_id>": {"class_type", "inputs", "_meta"}}`，widget 值内联为常量
  （`tests/inference/graphs/default_graph_sdxl1_0.json`：`"model": ["4", 0]`）。
- [事实] 编辑器 workflow 格式（litegraph 风格 `nodes`/`links` 数组、
  `widgets_values`、`groups`、`mode`、`definitions.subgraphs`）只存在于
  JSON 文件层（`blueprints/Image Blur.json` 顶层键 `['definitions',
  'last_link_id','last_node_id','links','nodes','revision','version']`）；
  后端 Python 对 `widgets_values`/`links` 数组零解析，前端为外部 pip 包
  `comfyui_frontend_package`（`app/frontend_management.py:234-242`）。
- [推断] workflow→prompt 转换（bypass/muted 剔除、reroute 消解、subgraph
  展开）完全在前端仓库完成，本仓库无转换器；后端仅从
  `extra_pnginfo.workflow` 提取 workflow_id 构建元数据，不解析画布结构
  （`server.py:91-106`）。

### 1.2 节点定义契约（V1 与 V3 双轨）

- [事实] V1：模块级 `NODE_CLASS_MAPPINGS` dict 注册（`nodes.py:2305-2314`），
  类属性 `INPUT_TYPES()`（返回 required/optional/hidden 三桶）、
  `RETURN_TYPES`、`FUNCTION`、`CATEGORY`、`OUTPUT_NODE`
  （`nodes.py:59-72`；`comfy/comfy_types/node_typing.py:214-318`）。
- [事实] V3 新契约：`comfy_entrypoint()` 返回 `ComfyExtension`，节点为
  `io.ComfyNode` + `define_schema()`，经 `GET_SCHEMA()` 注册
  （`nodes.py:2316-2342`）。
- [事实] widget 类型系统：`IO` StrEnum 内置类型 + 逗号分隔联合类型
  （`"FLOAT,INT"`）+ `*` 通配（`node_typing.py:16-71`）；options 含
  default/min/max/step/lazy/rawLink/forceInput 等十余种
  （`node_typing.py:96-180`）。后端经 `/object_info` 把 INPUT_TYPES 原样
  序列化给前端（`server.py:817-864`）。

### 1.3 边表示与类型检查

- [事实] 连接表示：`inputs` 值为 `[node_id: str, slot: int]` 二元组；
  `is_link` 判定即「长度 2、首元素 str、次元素 int/float」
  （`comfy_execution/graph_utils.py:1-10`）；长度≠2 报 `bad_linked_input`
  （`execution.py:929-942`，根目录）。
- [事实] 连接级类型检查在 `validate_inputs`：取上游
  `RETURN_TYPES[slot]` 与本输入类型比对，规则为**逗号分隔集合有交集即
  通过**（非严格），`*`(ANY) 直接放行（`comfy_execution/validation.py:
  193-247`）。节点可声明 `VALIDATE_INPUTS`（V1）/`validate_inputs`（V3）
  接管自定义校验：声明了某参数名即跳过该参数的内置 min/max/combo 检查
  （`execution.py:1031-1092`）。

### 1.4 图级特性的分层归属

- [事实] group/reroute/mute(2)/bypass(4) 是**纯编辑器概念**：后端无任何
  `mode`/`bypass`/`Reroute` 处理代码。
- [事实] subgraph 双轨：(a) 静态蓝图——`app/subgraph_manager.py` 扫描
  `custom_nodes/*/subgraphs/*.json` 与 `blueprints/*.json`，经
  `/global_subgraphs` 下发给前端（`subgraph_manager.py:122-132`），后端不
  展开；(b) 运行时动态展开——V3 节点返回 `expand=graph`，引擎注入
  `DynamicPrompt.ephemeral_prompt` 并建独立子缓存
  （`execution.py:590-613`）。
- [事实] 循环：`StartLoop/EndLoop` 声明 `loop_boundary`，验证期
  `validate_loops` 做配对/嵌套/逃逸检查（`comfy_execution/validation.py:
  42-190`）；执行期 `StartLoop.execute` 用 `GraphBuilder` 把循环体复制 N
  份再 expand（`comfy_extras/nodes_loop.py:10-102,213-233`）。即**后端图
  是 DAG，循环靠图重写实现**。

### 1.5 validate_prompt 语义

- [事实] 全局拒绝（HTTP 400，`server.py:1207-1209`）：节点类未注册 →
  `missing_node_type`；无任何 OUTPUT_NODE → `prompt_no_outputs`；**全部**
  输出验证失败 → `prompt_outputs_failed_validation`
  （`execution.py:1148-1298`）。
- [事实] 部分失败可执行：只要有一个 good_output 即入队，坏输出被剔除，
  `node_errors` 附 `dependent_outputs` 回传（`execution.py:1230-1258`）
  ——**错误以输出为边界隔离**。
- [事实] 孤儿节点（不可达任何输出）既不验证也不执行：验证只从输出节点
  DFS 回溯（`execution.py:1208-1212`）；缺 required 输入 →
  `required_input_missing`；环 → `dependency_cycle`
  （`execution.py:856-925`）。

## 2. Q2 执行引擎

### 2.1 队列模型：全局单队列串行

- [事实] `/prompt` 先 `validate_prompt` 再入队
  `prompt_queue.put((number, prompt_id, prompt, extra_data,
  outputs_to_execute, sensitive))`（`server.py:1178,1204`）。队列是 heapq
  优先队列，`front:true` 取负插队（取负在 `server.py:1148-1150`，heapq
  入队在 `execution.py:1315-1319`）；历史上限
  10000（`execution.py:1302`）。
- [事实] 消费端是**唯一**的 `prompt_worker` 守护线程（`main.py:622`），
  阻塞 `q.get()` 循环（`main.py:359-419`）→ prompt 间无并行（GPU 串行化
  是设计前提）。

### 2.2 调度循环与 async 节点

- [事实] 每次 prompt 新建事件循环 `asyncio.run(execute_async(...))`
  （`execution.py:729-730`）；主循环 `stage_node_execution()` → `execute()`
  → 按 SUCCESS/PENDING/FAILURE 推进（`execution.py:784-799`）。
- [事实] prompt 内并行仅限 async 节点：同步节点函数内联调用
  （`execution.py:313-316`）；coroutine 节点包装成 `asyncio.Task`，未立即
  完成则记 `pending_async_nodes` 并返回 PENDING，调度器继续跑别的节点
  （`execution.py:301-312,557-565`）。`--fast` 是数值优化（fp16/fp8），
  非并行执行器（`comfy/cli_args.py:195-199`）。
- [事实] 惰性求值：节点声明 `check_lazy_status`，执行前先问需要哪些输入，
  未就绪的动态加依赖边后 PENDING（`execution.py:504-523`）。

### 2.3 输出缓存：merkle 式祖先签名

- [事实] 双缓存：`outputs`（节点级输出 `{ui, outputs}`，
  `execution.py:109-142`）与 `objects`（节点实例池，键
  `(node_id, class_type)`，`caching.py:68-81`）；四种策略
  CLASSIC/LRU/NONE/RAM_PRESSURE（默认按显存压力驱逐，
  `execution.py:121-154`）。
- [事实] **缓存键 = 本节点+全部祖先的输入签名**：递归收集有序祖先，签名
  为 `[class_type, is_changed值, (输入名, 常量|("ANCESTOR",祖先序号,slot))]`
  （`caching.py:83-148`）——上游任何变化自然改变下游键，**无需逐级失效
  传播**。`IS_CHANGED` 返回 `NaN` 即永不命中（`nodes_loop.py:236-237`）。
- [事实] 命中路径：`execution_cached` 事件广播（`execution.py:762-774`）；
  每次 prompt 开始 `clean_unused()` 清旧条目（`caching.py:195-198`）；
  RAM 压力按 tensor 字节数+指数加权打分驱逐（`caching.py:551-620`）。

### 2.4 中断与部分执行

- [事实] 中断 = 全局 bool + `threading.RLock`
  （`comfy/model_management.py:2187-2200`）；`InterruptProcessingException`
  继承 **BaseException**（防被节点 `except Exception` 吞掉，
  `model_management.py:2184-2185`）；每节点调用前检查并**消费标志**
  （`model_management.py:2202-2208`）；`/interrupt` 支持按 prompt_id 定向
  且在队列锁内原子判定，防误伤后继 prompt（`server.py:1233-1261`；
  `execution.py:1376-1393`）。
- [事实] 部分执行：`/prompt` 可带 `partial_execution_targets`（节点 id
  列表，`server.py:1172-1174`），验证与执行列表都只从这些节点回溯
  （`execution.py:781-782,1181-1183`）——重跑时未变分支全走缓存。

### 2.5 错误处理：以输出为边界

- [事实] 验证错误（HTTP 400）与运行时错误分离；后者捕获后构造
  `{node_id, exception_message, exception_type, traceback,
  current_inputs}`（OOM 附提示并卸载模型，`execution.py:629-658`）→ 主循环
  `break` **终止整个 prompt**（`execution.py:793-795`），已执行输出保留
  在缓存（`executed` 集合随 `execution_error` 回报，
  `execution.py:703-714`）。
- [事实] `ExecutionBlocker`：节点可返回它使下游静默阻断或带消息报错
  （`graph_utils.py:140-155`；`execution.py:525-542`）。

### 2.6 执行顺序：增量拓扑溶解（非预计算拓扑序）

- [事实] `TopologicalSort` 维护 `pendingNodes/blockCount/blocking`，
  `get_ready_nodes()` 取 blockCount==0 者（`comfy_execution/graph.py:
  117-215`）；`ExecutionList` 支持「staged 节点退回+动态加边」（docstring
  自称 "topological dissolve"，`graph.py:217-236`）；挑节点用 UX 启发式：
  输出节点/async 节点优先，其次距输出 1~2 跳者（`graph.py:321-358`）。

## 3. Q3 通信协议

### 3.1 REST 面（全部路由在根 server.py，双前缀挂载）

- [事实] 本 HEAD **不存在** legacy/app 与 api_server 双轨：全部公开路由
  集中在根 `server.py`（1568 行）；`api_server/` 仅 `/internal` 子应用
  （`api_server/routes/internal/internal_routes.py:8-19`，注释明言"供
  ComfyUI 前端专用、不保证稳定"）。所有路由同时以原路径与 `/api` 前缀
  注册（`server.py:1301-1313`）。
- [事实] 核心端点：`POST /prompt`（body：prompt DAG + client_id +
  extra_data + 可选 partial_execution_targets；响应
  `{prompt_id, number, node_errors}`，`server.py:1138-1209`）；
  `GET/POST /queue`（查询/清空/删除，`server.py:1130,1219`）；
  `GET /history[/{id}]`、`POST /history`（`server.py:1111-1285`）；
  `GET /object_info[/{node_class}]`（`server.py:866,879`）；
  `POST /upload/image|mask`（`server.py:525,531`）；`GET /view`
  （`server.py:577`）；`GET /system_stats`（含版本、sys.argv、RAM/VRAM，
  `server.py:752`）；`POST /interrupt`、`/free`；`GET/POST /api/jobs*`
  （任务查询/取消，`server.py:887-1109`）；资产系统 `/api/assets/*`
  （上传/标签/哈希寻址，`app/assets/api/routes.py:467-1132`）。
- [事实] `blueprints/` 不是路由，是内置子图工作流 JSON，经
  `/global_subgraphs` 暴露（`app/subgraph_manager.py:93-128`）。

### 3.2 WebSocket：文本+二进制双通道复用

- [事实] `GET /ws`：`clientId` 查询参数即会话 id，传入即**踢掉旧连接**
  （`self.sockets.pop(sid)`，`server.py:330-388`）——会话接管无需认证。
  首条消息可协商 `feature_flags`（`server.py:363-377`）。
- [事实] 事件全集（文本帧 JSON `{type, data}`）：`execution_start` /
  `executing {node, display_node, prompt_id}` / `executed {node, output}` /
  `execution_cached {nodes[]}` / `execution_error` /
  `execution_interrupted` / `execution_success` /
  `status {exec_info:{queue_remaining}}`（连接即推）/
  `progress_state {nodes{value,max,state}}`
  （`execution.py:497-826`；`server.py:348,1485`；
  `comfy_execution/progress.py:184-186`）。
- [事实] 二进制预览复用同一连接：帧 = 4 字节大端事件号 + 载荷
  （`server.py:1375-1382`；`protocol.py:2-7` 定义 PREVIEW_IMAGE=1/
  UNENCODED=2/TEXT=3/PREVIEW_IMAGE_WITH_METADATA=4）；预览帧内含 4 字节
  图像类型（JPEG/PNG）+ 编码图像（`server.py:1395-1443`）。

### 3.3 节点发现 /object_info

- [事实] 遍历 `NODE_CLASS_MAPPINGS`，对每节点调用 `INPUT_TYPES()` 聚合
  input/output/category/python_module/api_node 等字段
  （`server.py:817-877`）；combo 候选值即**实时目录扫描结果**内嵌进响应
  （如 CheckpointLoader 列全部 checkpoint 文件，`nodes.py:626`；
  LoadImage 列 input 目录全部图片，`nodes.py:1750-1754`）。
- [推断] 载荷随模型库与 custom node 数线性膨胀（MB 级常态），是前端动态
  UI 的单一数据源；集成方应按节点拉取
  （`/object_info/{node_class}`）并做增量缓存。

### 3.4 认证与多用户：零认证

- [事实] **无任何认证**。`--multi-user` 只按 header `comfy-user` 区分存储，
  header 值匹配 `users.json` 即通过，无密码/token
  （`app/user_manager.py:47-70`）；`POST /users` 可匿名创建用户
  （`user_manager.py:137-148`）。
- [事实] `alembic_db` 是**资产数据库**（9 个迁移版本全为 assets/标签），
  不是用户库；用户存于 `user/<id>/users.json`。
- [事实] 默认 `--listen 127.0.0.1`（`comfy/cli_args.py:63`）；默认 CORS
  中间件拒绝 `Sec-Fetch-Site: cross-site` 且 loopback 场景 Host≠Origin
  返回 403（防恶意网页向 127.0.0.1 提交 prompt，`server.py:199-237`）；
  `--enable-cors-header` 才回显 ACAO（`server.py:155-170`）。

## 4. Q4 敌意输入与安全边界

### 4.1 custom node = 任意代码执行（官方显式声明）

- [事实] 启动即扫 `custom_nodes/`（`nodes.py:2368-2377`），
  `load_custom_node` 用 `spec_from_file_location + exec_module` 直接执行
  任意 .py（`nodes.py:2266-2276`）；更早还有 `prestartup_script.py`
  机制（`main.py:182-224`）。
- [事实] 官方 SECURITY.md 安全模型：**"custom node 是任意 Python 代码，
  与用户自装软件同级信任；默认 localhost，任何能访问 URL 者皆可信"**
  （`SECURITY.md:5-12`），并把 `--listen` 暴露、旧版 torch.load、特定
  custom node 问题全部划出漏洞范围（`SECURITY.md:26-29`）。
- [事实] 企业治理新方向：`app/governance.py` 支持 Ed25519 签名策略文件，
  对 pack 做 blake3 摘要 allowlist/blocklist，拒绝 .pyc 与符号链接
  （`governance.py:87-104,177-223`），当前默认关闭
  （`GOVERNANCE_REQUIRED=False`，`governance.py:18`）。

### 4.2 路径处理：三目录沙箱 + GHSA 修复结晶

- [事实] `/upload/image`：`subfolder` 经 normpath 拼接后
  `os.path.commonpath` 校验不逃出 upload_dir（`server.py:471-476`）。
- [事实] `/view`：`filename` 拒绝首字符 `/` 与含 `..`（`server.py:604-606`），
  `subfolder` commonpath 校验，最终 `os.path.basename` 兜底
  （`server.py:615-621`）；支持 `[output]/[input]/[temp]` 注解语法切换根
  目录（`folder_paths.py:257-270`）。
- [事实] 防逃逸规范实现 `is_within_directory`（realpath 归一防符号链接，
  `folder_paths.py:326-340`）；HTTP 可达目录仅 output/temp/input 三类
  （`folder_paths.py:220-227`）。近期安全修复有回归测试锚点
  （GHSA-779p-m5rp-r4h4 系列，`tests-unit/security_test/`）。
- [推断] `/upload` 与 `/view` 的 subfolder 校验用 abspath+commonpath（词法）
  而非 realpath，与 `is_within_directory` 不同；input 目录内预置符号链接
  可绕过——属纵深差异，非官方认可漏洞。

### 4.3 模型文件：safetensors/pickle 双轨，无完整性校验

- [事实] `.safetensors/.sft` 走 `safetensors.safe_open`
  （`comfy/utils.py:163-178`）；其余（.ckpt/.pt/.pth/.bin/.pkl）走
  `torch.load(..., weights_only=True)`（`comfy/utils.py:192`），依赖
  PyTorch≥2.x 受限 unpickler。
- [推断] 模型文件**无签名/哈希完整性校验**即加载；governance 的
  `models` 密封摘要形式未列入 `_ENFORCED_FORMS`（`governance.py:43-45`），
  仅为预留。

### 4.4 prompt 注入 → 路径逃逸

- [事实] LoadImage 的 `image` 参数经 `get_annotated_filepath` 强制
  containment（`nodes.py:1764-1765`）；SaveImage 的 `filename_prefix`
  （含子目录与 %变量%）经 `get_save_image_path` containment 并显式报错
  "不允许存到 output 外"（`folder_paths.py:547-557`）；模型名经
  `relpath` 归一压缩绝对路径与 `..`（`folder_paths.py:450-451`）。
- [推断] 内置节点路径面收口较完整；历史逃逸多发生在第三方 custom node
  ——与 SECURITY.md 把责任外包给节点作者的立场一致。

### 4.5 生态治理与密钥流

- [事实] `comfy_api_nodes/`（约 50 个合作伙伴 API 节点：OpenAI/Gemini/
  Kling 等）标注 `API_NODE` 并在 `/object_info` 透出（`server.py:856-857`）；
  密钥以隐藏输入随 prompt `extra_data` 传入，服务端移入队列元组
  "sensitive" 槽并从 `GET /queue` 响应中剥除（`execution.py:162`；
  `server.py:109-111,1199-1204`）；请求日志对 authorization/x-api-key
  头脱敏（`comfy_api_nodes/util/request_logger.py:12,77`）。

## 5. 许可证与集成合规定谳

- [事实] LICENSE = GPL-3.0 标准全文（674 行）。
- 定谳：与规划预判一致——**不可链接、不可复制代码入 mdcore 系产品仓**；
  进程外协议集成（REST + WS 客户端）不引入衍生作品义务。本 pass 的全部
  产出为协议/行为描述（事实与推断标注），零代码复制，净室合规。
- 【推断】协议本身（端点、事件名、帧格式）为事实标准接口而非版权性
  表达，自研客户端无传染风险；业界进程边界先例充分。正式商用前建议
  法务口径复核一次。

## 6. 对 mdcore 工作台的集成输入

### 6.1 协议最小面（进程外集成六端点）

`POST /prompt`（带 client_id）→ `GET /object_info[/{node}]`（发现+动态
UI）→ `GET /history/{id}`（终态）→ `POST /interrupt` 或
`/api/jobs/{id}/cancel`（取消）→ `GET /view`（取产物）→ `GET /ws`
（进度订阅）。注意双前缀（`/api/*` 镜像）与 WS 文本/二进制双通道帧格式
（4 字节大端事件号）。

### 6.2 安全姿态（集成五条）

1. ComfyUI 零认证、header 即身份、clientId 可随意接管会话 → Rust 网关
   绑 127.0.0.1、由工作台持有唯一凭证，绝不直接暴露 `--listen 0.0.0.0`。
2. 把 ComfyUI 进程当**不可信代码执行环境**（custom node = 任意 Python +
   prestartup 脚本）：容器/沙箱隔离 + 默认
   `--disable-all-custom-nodes` + 白名单。
3. 复刻其路径沙箱语义：三目录 + `[annotation]` 语法 + realpath
   containment + `/view` basename 兜底 + 危险 content-type 强制
   attachment/nosniff（`folder_paths.py:279-323`）——官方踩坑（GHSA-779p）
   后的结晶，值得 Rust 侧等价实现。
4. 模型加载姿态：优先 safetensors；pickle 系必须受限反序列化；Rust 侧
   模型仓库应补哈希/签名门禁（ComfyUI 默认没有）。
5. 密钥不走工作流 JSON：ComfyUI 的 partner 密钥经 prompt extra_data 流转
   （仅从队列列表脱敏）；工作台应设计独立 secrets 通道，避免密钥进入
   可导出的 workflow 文件。

### 6.3 引擎层可移植资产（对 mdcore 自研节点图内核的启示）

1. **两级图格式**：编辑图（位置/mode/组）与执行图（class_type+inputs
   引用式 DAG）分离，转换放客户端，内核只认后者——验证/缓存/执行全部
   简化。
2. **merkle 式缓存键**：「自身+祖先全链输入签名」内容寻址，上游变更
   自动失效下游，零失效传播成本；Rust 可用 Hash 派生实现同等语义。
3. **增量拓扑溶解**：blockCount 计数 + 动态加边 + staged 退回，天然
   支持 lazy input、动态图展开、循环重写——优于预计算拓扑序。
4. **中断语义**：全局标志+原子消费+每任务复位+BaseException 防吞；
   多线程 Rust 需换 per-task CancellationToken，但「检查点=节点边界」
   粒度可沿用。
5. **PENDING 三态协议**：SUCCESS/PENDING/FAILURE + external block，让
   async 与同步节点共存于一个调度器；Rust 映射 tokio::task + Notify。
6. **验证边界**：只从 OUTPUT_NODE 回溯；孤儿天然跳过；部分输出失败不
   拒绝整个 prompt，`node_errors + dependent_outputs` 结构可直接借鉴。

## 7. 四问题结论一览

| 问题 | 一句话结论 | 关键锚点 |
|------|-----------|---------|
| graph 模型 | 编辑图/执行图彻底分离，后端只认扁平 prompt JSON；类型检查为逗号并集宽松匹配；循环=图重写 | `graph_utils.py:1-10`、`validation.py:193-247` |
| 执行 | 全局单队列串行 + prompt 内 async 并行 + merkle 祖先签名缓存 + 增量拓扑溶解调度 | `main.py:622`、`caching.py:83-148`、`graph.py:217-236` |
| 通信 | REST 全路由双前缀 + WS 文本/二进制双通道；/object_info 是动态 UI 单一数据源；零认证 | `server.py:1301-1313`、`server.py:1375-1382` |
| 敌意输入 | custom node=任意代码（官方显式声明信任外包）；三目录路径沙箱收口完整；模型无完整性校验 | `SECURITY.md:5-12`、`folder_paths.py:326-340` |
