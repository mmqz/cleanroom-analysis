//! collab-adapter — D1-a 隔离适配层原型（yrs-b §6 契约草案的实证载体）。
//!
//! **本 crate 是唯一接触 yrs 的地方**（B1.2 单点构造；隔离不变量 (a)：
//! 公开 API 零 `yrs::` 类型——撤除 yrs 时只拆本 crate）。原型轮范围
//! （d1c-t1）：
//! - 构造面：`open()` 无条件 `skip_gc: true`（B1.1 全编队不变量——
//!   `SessionOptions` 不暴露该开关，调用方不可覆盖）；
//! - 装载面：`load_bulk_text`（B2.1(a) 大块路径，A2 形状）/ `load_blocks`
//!   （B2.1(b) 小批分事务路径）/ `edit_stream`（B2.1(b) 逐笔路径）——
//!   **不暴露裸长事务入口**（B2.1 禁止形状 API 面不可表达）；
//! - 同步面：`state_vector` / `encode_diff` / `apply_update`（B1.6 输入
//!   限额 + panic 隔离 catch_unwind）；
//! - 撤销面（冒烟）：每 origin 一栈（B3.2 mdcore Source::Human/Ai 一一
//!   映射；捕获窗 1000ms 对齐 mdcore COALESCE_MS 语义连续性）。
//!
//! 原型不做（后续轮/实施轮）：锚点面 bind/resolve（d1c-t3）、快照面
//! 维护窗压缩的完整语义（d1c-t3）、事件面、栈上限逐出（B3.4 API 面
//! 充分性 = yrs-b §9 待核项）。
//!
//! 载体口径：interim 载体 = `text` Text 根 + `blocks` Array-of-Map 根
//! （结构载体选型 XmlFragment vs Array-of-Map 的证据轮在 d1c-t3——本轮
//! 装载/撤销/同步面与载体选择正交）。

use std::collections::HashSet;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use yrs::undo::Options as UndoOptions;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::{
    Array, Assoc, ClientID, Doc, GetString, ID, In, IndexedSequence, Map, MapPrelim,
    Options as DocOptions, Origin, ReadTxn, StateVector, StickyIndex, Text, Transact,
    UndoManager, Update,
};

/// 适配层错误（公开面零 yrs 类型；引擎错误转译为语义化变体）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// 输入超过限额（B1.6——数值对齐 SECURITY_POLICY v1.2 读面口径，实施轮定参）
    Limit { size: usize, limit: usize },
    /// 对端 wire 数据不合法（解码失败）
    Decode(String),
    /// 引擎 panic 被隔离捕获（B1.6 panic 边界）
    Panic(String),
    /// 调用面语义错误（参数不合法等）
    InvalidInput(String),
    /// 锚点悬空：目标块已被删除（R4.3 解析失败面显式化——不静默返回邻居）
    AnchorDangling { client: u64, clock: u32 },
    /// 锚点侧车格式不合法/版本不受支持（R4.3 验收：格式版本化）
    SidecarFormat(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::Limit { size, limit } => {
                write!(f, "input {}B exceeds limit {}B", size, limit)
            }
            AdapterError::Decode(m) => write!(f, "wire decode failed: {}", m),
            AdapterError::Panic(m) => write!(f, "engine panic isolated: {}", m),
            AdapterError::InvalidInput(m) => write!(f, "invalid input: {}", m),
            AdapterError::AnchorDangling { client, clock } => {
                write!(f, "anchor dangling (deleted target): ({}, {})", client, clock)
            }
            AdapterError::SidecarFormat(m) => write!(f, "anchor sidecar format: {}", m),
        }
    }
}

impl std::error::Error for AdapterError {}

/// 编辑来源语义（B3.2：mdcore `Source::Human/Ai` 一一映射；未归类面在
/// 适配层入向处一律归 Human——保守面，AI 误标 Human 的代价更轻）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditOrigin {
    Human,
    Ai,
}

impl EditOrigin {
    /// origin 字节串常量表（B3.2——适配层单点定义，禁散布）
    fn wire(self) -> Origin {
        match self {
            EditOrigin::Human => Origin::from("mdc:human"),
            EditOrigin::Ai => Origin::from("mdc:ai"),
        }
    }
}

/// 结构化块规格（AI 生成态装载输入形状；块内载体表示是适配层私事）。
#[derive(Debug, Clone)]
pub struct BlockSpec {
    pub block_id: String,
    pub text: String,
}

/// 会话选项。
///
/// **刻意不含 skip_gc**（B1.1：全编队不变量，构造期无条件强制，调用方
/// 不可覆盖——本类型的存在本身就是该不变量的 API 面证明）。
///
/// **刻意不含 undo_stack_limit**（B3.4 定谳：yrs 0.28 UndoManager 的
/// observer_added/updated/popped/cleared 字段全部私有、无公开 observe
/// API、无栈逐出/容量 API——栈上限不可经公开 API 实现；d1c-t2 源码
/// 核验。选项 = 上游 API 请求 / 粗粒度 clear_all 阈值 / 无限维持
/// 现状——与 mdcore R-3 协同裁决（yrs-b B3.4 联动））。
#[derive(Debug, Clone)]
pub struct SessionOptions {
    pub client_id: u64,
    /// 入向字节限额（B1.6；默认 52MiB = SECURITY_POLICY v1.2 声明总量族
    /// 口径的占位对齐——实施轮按最终部署面定参）。
    pub input_byte_limit: usize,
}

impl Default for SessionOptions {
    fn default() -> Self {
        SessionOptions {
            client_id: 7_000_001,
            input_byte_limit: 52 * 1024 * 1024,
        }
    }
}

/// 交互流单笔编辑（B2.1(b) 逐笔路径——每笔独立事务 = 一个 undo 栈项
/// 候选 + 一个 origin 溯源单位，B2.3 三同点决策的入参面）。
#[derive(Debug, Clone)]
pub enum StreamEdit {
    /// 在 text 根末尾追加（pos 语义 = 字节偏移；原型轮只用追加形状）
    AppendText { text: String },
    /// 追加一个结构化块
    InsertBlock { block: BlockSpec },
    /// 在某块之前插入（中段插入——锚点保持验证的结构性编辑面）
    InsertBlockBefore { before: String, block: BlockSpec },
    /// 删除块（按 block_id 定位；不存在 → InvalidInput）
    RemoveBlock { block_id: String },
}

/// 跨模型锚点本位币（R4.1：CRDT ID 的不透明包装——三模型语义锚的
/// 唯一交换格式；永不重编，撤 yrs 时随适配层拆除）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CrdtAnchor {
    pub client: u64,
    pub clock: u32,
}

/// 三模型语义锚（各线在自己模型内产生，R4.2 入向面）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticAnchor {
    /// MD 线：BlockId 寻址（boundary-audit p2 双轨中的意图/锚点轨）
    MdBlock { block_id: String },
    /// Office 线：书签名 + 位置快照（R4.5 弱绑定形态——锚点写面维持
    /// 现状公开 API 时的入向退化面；block_index = bind 时刻位置）
    OfficeBookmark { name: String, block_index: u32 },
    /// PDF 线：标注引用（页号 + 标签 + bind 时刻的块位置——页语义在
    /// 消费侧，适配层只登记引用→位置映射）
    PdfAnnotRef { page: u32, label: String, block_index: u32 },
}

/// 锚点解析结果（R4.3 出向：当前结构位置 + 可辨的语义重建）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAnchor {
    /// 目标块当前下标（含随结构性编辑的漂移——ID 稳定，下标漂移）
    pub block_index: u32,
    /// 目标块当前 block_id（删除后不可辨 → AnchorDangling 错误面）
    pub block_id: String,
}

/// 大块装载分块尺寸（B2.1(a)：≥KB 级；8KiB 对齐 yrs-a §4.5 A2 实测口径）。
const BULK_CHUNK_BYTES: usize = 8 * 1024;
/// 结构化块小批尺寸（B2.1(b) "每小批独立事务"；512 = 20k 块 → 40 事务）。
const BLOCK_BATCH: usize = 512;

/// D1-a 隔离适配层原型。
pub struct CollabAdapter {
    doc: Doc,
    undo_human: UndoManager,
    undo_ai: UndoManager,
    options: SessionOptions,
    /// R4.2 映射表（会话/文档级资产；侧车序列化面见 export_anchor_sidecar）
    anchor_table: HashMap<SemanticAnchor, CrdtAnchor>,
}

impl CollabAdapter {
    /// 构造面唯一入口（B1.2 单点构造）。
    ///
    /// B1.1：`skip_gc: true` 无条件设置——本函数体内是整个原型唯一的
    /// `DocOptions` 触点；#676 上游 UAF 未修复期间这是全编队不变量。
    pub fn open(options: SessionOptions) -> Result<CollabAdapter, AdapterError> {
        // B1.1 全编队不变量（无条件，无 if，无配置面）
        let mut opts = DocOptions::with_client_id(ClientID::new(options.client_id));
        opts.skip_gc = true;
        let doc = Doc::with_options(opts);

        // interim 载体（结构载体选型证据轮 d1c-t3 前的承载面）
        let text = doc.get_or_insert_text("text");
        let blocks = doc.get_or_insert_array("blocks");
        let _ = (text.len(&doc.transact()), blocks.len(&doc.transact()));

        // B3.2：每 origin 一栈；捕获窗 1000ms = mdcore COALESCE_MS 语义连续性
        let mut undo_human = Self::new_undo_manager(&doc, EditOrigin::Human);
        let mut undo_ai = Self::new_undo_manager(&doc, EditOrigin::Ai);
        undo_human.expand_scope(&doc, &doc.get_or_insert_text("text"));
        undo_human.expand_scope(&doc, &doc.get_or_insert_array("blocks"));
        undo_ai.expand_scope(&doc, &doc.get_or_insert_text("text"));
        undo_ai.expand_scope(&doc, &doc.get_or_insert_array("blocks"));

        Ok(CollabAdapter {
            doc,
            undo_human,
            undo_ai,
            options,
            anchor_table: HashMap::new(),
        })
    }

    fn new_undo_manager(doc: &Doc, origin: EditOrigin) -> UndoManager {
        let mut tracked = HashSet::new();
        tracked.insert(origin.wire());
        let _ = doc; // doc 仅用于生命周期锚定说明；scope 在 open() 内展开
        UndoManager::with_options(UndoOptions {
            capture_timeout_millis: 1000,
            tracked_origins: tracked,
            capture_transaction: None,
            timestamp: Arc::new(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0)
            }),
            init_undo_stack: vec![],
            init_redo_stack: vec![],
        })
    }

    // ── 装载面 ─────────────────────────────────────────────────────

    /// B2.1(a) 大块路径：整段文本装载（AI 生成态典型形状）。
    ///
    /// origin 标注（B2.2/B3.2）：生成态装载走 Ai 面；捕获窗内的事务切分
    /// 合并为**一个 undo 栈项**（"一笔装载"语义）+ 单一 origin 溯源。
    /// 内部把内容切为 ≥8KiB 大块后在**单事务**内逐块入文档（yrs-a §4.5
    /// A2 实测零放大的形状）；输入字节超限 → `Limit`（B1.6）。
    pub fn load_bulk_text(
        &self,
        origin: EditOrigin,
        content: &str,
    ) -> Result<(), AdapterError> {
        if content.len() > self.options.input_byte_limit {
            return Err(AdapterError::Limit {
                size: content.len(),
                limit: self.options.input_byte_limit,
            });
        }
        let text = self.doc.get_or_insert_text("text");
        let mut txn = self.doc.transact_mut_with(origin.wire());
        let bytes = content.as_bytes();
        let mut pos = 0usize;
        while pos < bytes.len() {
            // 切分点回退到 UTF-8 字符边界（content 为 &str，边界内安全切片）
            let mut end = (pos + BULK_CHUNK_BYTES).min(bytes.len());
            while end > pos && !content.is_char_boundary(end) {
                end -= 1;
            }
            let chunk = &content[pos..end];
            if !chunk.is_empty() {
                text.insert(&mut txn, pos as u32, chunk);
            }
            pos = end;
        }
        txn.commit();
        Ok(())
    }

    /// B2.1(b) 小批分事务路径：结构化块批量装载。
    ///
    /// origin 标注同 `load_bulk_text`（捕获窗内小批合并为一个栈项）；
    /// 每 512 块一事务（"每小批独立事务"）；总量超限 → `Limit`。
    pub fn load_blocks(
        &self,
        origin: EditOrigin,
        blocks: &[BlockSpec],
    ) -> Result<(), AdapterError> {
        let total: usize = blocks
            .iter()
            .map(|b| b.block_id.len() + b.text.len() + 64)
            .sum();
        if total > self.options.input_byte_limit {
            return Err(AdapterError::Limit {
                size: total,
                limit: self.options.input_byte_limit,
            });
        }
        let array = self.doc.get_or_insert_array("blocks");
        for batch in blocks.chunks(BLOCK_BATCH) {
            let mut txn = self.doc.transact_mut_with(origin.wire());
            for b in batch {
                let map = MapPrelim::from([
                    ("block_id", In::Any(b.block_id.clone().into())),
                    ("text", In::Any(b.text.clone().into())),
                ]);
                let idx = array.len(&txn);
                array.insert(&mut txn, idx, map);
            }
            txn.commit();
        }
        Ok(())
    }

    /// B2.1(b) 逐笔路径：交互流单笔编辑（每笔独立事务 + origin 标注）。
    pub fn edit_stream(&self, origin: EditOrigin, edit: StreamEdit) -> Result<(), AdapterError> {
        match edit {
            StreamEdit::AppendText { text } => {
                if text.len() > self.options.input_byte_limit {
                    return Err(AdapterError::Limit {
                        size: text.len(),
                        limit: self.options.input_byte_limit,
                    });
                }
                let t = self.doc.get_or_insert_text("text");
                let mut txn = self.doc.transact_mut_with(origin.wire());
                let pos = t.len(&txn);
                t.insert(&mut txn, pos, &text);
                txn.commit();
                Ok(())
            }
            StreamEdit::InsertBlock { block } => {
                let size = block.block_id.len() + block.text.len() + 64;
                if size > self.options.input_byte_limit {
                    return Err(AdapterError::Limit {
                        size,
                        limit: self.options.input_byte_limit,
                    });
                }
                let array = self.doc.get_or_insert_array("blocks");
                let map = MapPrelim::from([
                    ("block_id", In::Any(block.block_id.clone().into())),
                    ("text", In::Any(block.text.clone().into())),
                ]);
                let mut txn = self.doc.transact_mut_with(origin.wire());
                let idx = array.len(&txn);
                array.insert(&mut txn, idx, map);
                txn.commit();
                Ok(())
            }
            StreamEdit::InsertBlockBefore { before, block } => {
                let array = self.doc.get_or_insert_array("blocks");
                let map = MapPrelim::from([
                    ("block_id", In::Any(block.block_id.clone().into())),
                    ("text", In::Any(block.text.clone().into())),
                ]);
                let mut txn = self.doc.transact_mut_with(origin.wire());
                let len = array.len(&txn);
                let mut found: Option<u32> = None;
                for i in 0..len {
                    if let Some(yrs::Out::YMap(m)) = array.get(&txn, i) {
                        if let Some(yrs::Out::Any(yrs::Any::String(id))) =
                            m.get(&txn, "block_id")
                        {
                            if &*id == before {
                                found = Some(i);
                                break;
                            }
                        }
                    }
                }
                match found {
                    Some(i) => {
                        array.insert(&mut txn, i, map);
                        txn.commit();
                        Ok(())
                    }
                    None => {
                        drop(txn);
                        Err(AdapterError::InvalidInput(format!(
                            "before-block {:?} not found",
                            before
                        )))
                    }
                }
            }
            StreamEdit::RemoveBlock { block_id } => {
                let array = self.doc.get_or_insert_array("blocks");
                let mut txn = self.doc.transact_mut_with(origin.wire());
                let len = array.len(&txn);
                let mut found: Option<u32> = None;
                for i in 0..len {
                    if let Some(yrs::Out::YMap(m)) = array.get(&txn, i) {
                        if let Some(yrs::Out::Any(yrs::Any::String(id))) =
                            m.get(&txn, "block_id")
                        {
                            if &*id == block_id {
                                found = Some(i);
                                break;
                            }
                        }
                    }
                }
                match found {
                    Some(i) => {
                        array.remove(&mut txn, i);
                        txn.commit();
                        Ok(())
                    }
                    None => {
                        // 未发生任何变更：事务自然 drop（空事务 commit 语义 = 无操作）
                        drop(txn);
                        Err(AdapterError::InvalidInput(format!(
                            "block_id {:?} not found",
                            block_id
                        )))
                    }
                }
            }
        }
    }

    // ── 同步面 ─────────────────────────────────────────────────────

    /// 本端状态向量（不透明 wire 字节——对端用于差量请求）。
    pub fn state_vector(&self) -> Vec<u8> {
        self.doc.transact().state_vector().encode_v1()
    }

    /// 差量编码（remote_sv = 对端 `state_vector()` 产物）。
    pub fn encode_diff(&self, remote_sv: &[u8]) -> Result<Vec<u8>, AdapterError> {
        let sv = StateVector::decode_v1(remote_sv)
            .map_err(|e| AdapterError::Decode(e.to_string()))?;
        Ok(self.doc.transact().encode_diff_v1(&sv))
    }

    /// 入向 update（B1.6：字节限额前置 + panic 隔离）。
    pub fn apply_update(&self, bytes: &[u8]) -> Result<(), AdapterError> {
        if bytes.len() > self.options.input_byte_limit {
            return Err(AdapterError::Limit {
                size: bytes.len(),
                limit: self.options.input_byte_limit,
            });
        }
        let doc = &self.doc;
        let r = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let update = Update::decode_v1(bytes)
                .map_err(|e| AdapterError::Decode(e.to_string()))?;
            let mut txn = doc.transact_mut();
            txn.apply_update(update)
                .map_err(|e| AdapterError::Decode(e.to_string()))?;
            txn.commit();
            Ok(())
        }));
        match r {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(p) => Err(AdapterError::Panic(panic_message(&p))),
        }
    }

    // ── 撤销面（B3.2 冒烟；深验 d1c-t2）────────────────────────────

    /// 撤销指定 origin 面的最近栈项（栈本地——B3.1 永不序列化进 update）。
    pub fn undo(&mut self, origin: EditOrigin) -> bool {
        self.manager_mut(origin).undo_blocking()
    }

    /// 重做指定 origin 面。
    pub fn redo(&mut self, origin: EditOrigin) -> bool {
        self.manager_mut(origin).redo_blocking()
    }

    /// 撤销栈深度（origin 分栈口径；B3.4 上限参数的观测面）。
    pub fn undo_depth(&self, origin: EditOrigin) -> usize {
        self.manager_ref(origin).undo_stack().len()
    }

    fn manager_mut(&mut self, origin: EditOrigin) -> &mut UndoManager {
        match origin {
            EditOrigin::Human => &mut self.undo_human,
            EditOrigin::Ai => &mut self.undo_ai,
        }
    }

    fn manager_ref(&self, origin: EditOrigin) -> &UndoManager {
        match origin {
            EditOrigin::Human => &self.undo_human,
            EditOrigin::Ai => &self.undo_ai,
        }
    }

    // ── 维护面（B1.3 受控压缩点）────────────────────────────────

    /// B1.3 受控压缩点：全栈 `clear_all`（B3.3 前置**内置**——不依赖
    /// 调用方纪律）后显式 `txn.gc(None)`。
    ///
    /// 语义：只允许在两类受控点调用（会话撤销栈清空后 / 权威端维护窗）
    /// ——本方法即前者的适配层形态；**压缩后撤销历史不可恢复**
    /// （clear_all 已清栈），这是 B1.3 的代价面。返回压缩前后全量
    /// update 字节数（供探针断言 wire 治理效果）。
    pub fn maintenance_gc(&mut self) -> Result<(usize, usize), AdapterError> {
        let before = self.state_vector_full_update_len();
        // B3.3 压缩前置：先清全部撤销栈（撤销栈是 GC 安全的引用持有方，
        // P7b4 实证 clear_all → gc 安全）
        self.undo_human.clear_all();
        self.undo_ai.clear_all();
        let doc = &self.doc;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            let mut txn = doc.transact_mut();
            txn.gc(None);
            txn.commit();
        }))
        .map_err(|p| AdapterError::Panic(panic_message(&p)))?;
        let after = self.state_vector_full_update_len();
        Ok((before, after))
    }

    /// 全量 update 字节数（维护面观测——非同步协议面）。
    fn state_vector_full_update_len(&self) -> usize {
        self.doc
            .transact()
            .encode_diff_v1(&StateVector::default())
            .len()
    }

    // ── 锚点面（R4.2/R4.3——用户验证面"跨模型锚点映射"）────────────

    /// R4.2 入向映射：语义锚 → 当前解析位置 → CRDT ID，成对入映射表。
    ///
    /// MdBlock 按 block_id 定位；OfficeBookmark/PdfAnnotRef 按 bind 时刻
    /// 位置快照定位（R4.5 弱绑定形态——锚点写面维持现状公开 API 期间
    /// 的入向退化面，书签定义静默丢弃面由消费侧悬空警告兜底）。
    pub fn bind(&mut self, semantic: &SemanticAnchor) -> Result<CrdtAnchor, AdapterError> {
        let array = self.doc.get_or_insert_array("blocks");
        let txn = self.doc.transact();
        let len = array.len(&txn);
        // 定位 bind 时刻的目标下标
        let target_index: u32 = match semantic {
            SemanticAnchor::MdBlock { block_id } => {
                let mut found = None;
                for i in 0..len {
                    if let Some(yrs::Out::YMap(m)) = array.get(&txn, i) {
                        if let Some(yrs::Out::Any(yrs::Any::String(id))) = m.get(&txn, "block_id") {
                            if &*id == block_id {
                                found = Some(i);
                                break;
                            }
                        }
                    }
                }
                found.ok_or_else(|| {
                    AdapterError::InvalidInput(format!("md block {:?} not found", block_id))
                })?
            }
            SemanticAnchor::OfficeBookmark { block_index, .. }
            | SemanticAnchor::PdfAnnotRef { block_index, .. } => {
                if *block_index >= len {
                    return Err(AdapterError::InvalidInput(format!(
                        "block_index {} out of range (len {})",
                        block_index, len
                    )));
                }
                *block_index
            }
        };
        // Assoc::After + 下标 N → 第 N 项自身首 item 的 ID（R4.1 本位币）；
        // （Assoc::Before 在 index 0 时退化为 Root 上下文无 ID，故用 After）
        let sticky = array
            .sticky_index(&txn, target_index, Assoc::After)
            .ok_or_else(|| AdapterError::InvalidInput("sticky_index 解析失败".into()))?;
        let id = sticky
            .id()
            .cloned()
            .ok_or_else(|| AdapterError::InvalidInput("空集合无 ID 可锚（Root 上下文）".into()))?;
        let anchor = CrdtAnchor {
            client: id.client.get(),
            clock: id.clock,
        };
        self.anchor_table.insert(semantic.clone(), anchor);
        Ok(anchor)
    }

    /// R4.3 出向解析：CRDT ID → StickyIndex → 当前结构位置 → 语义重建。
    ///
    /// 悬空面显式化：目标块已删除 → `AnchorDangling`（不静默返回邻居）。
    pub fn resolve(&self, anchor: CrdtAnchor) -> Result<ResolvedAnchor, AdapterError> {
        let array = self.doc.get_or_insert_array("blocks");
        let txn = self.doc.transact();
        let sticky =
            StickyIndex::from_id(ID::new(ClientID::new(anchor.client), anchor.clock), Assoc::After);
        let offset = sticky
            .get_offset(&txn)
            .ok_or(AdapterError::AnchorDangling {
                client: anchor.client,
                clock: anchor.clock,
            })?;
        let idx = offset.index;
        if idx >= array.len(&txn) {
            // 目标删除后位置塌缩到序列尾外
            return Err(AdapterError::AnchorDangling {
                client: anchor.client,
                clock: anchor.clock,
            });
        }
        // 悬空检测：当前位置块的自身首 ID 必须等于锚点 ID
        // （删除后的墓碑会把 get_offset 塌缩到邻居——静默错配即 R4.3 反例）
        let here = array
            .sticky_index(&txn, idx, Assoc::After)
            .and_then(|s| s.id().cloned());
        match here {
            Some(id2)
                if id2.client.get() == anchor.client && id2.clock == anchor.clock =>
            {
                let block_id = array
                    .get(&txn, idx)
                    .and_then(|out| match out {
                        yrs::Out::YMap(m) => match m.get(&txn, "block_id") {
                            Some(yrs::Out::Any(yrs::Any::String(s))) => Some(s.to_string()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .ok_or_else(|| {
                        AdapterError::InvalidInput("位置块缺 block_id 字段（载体损坏）".into())
                    })?;
                Ok(ResolvedAnchor {
                    block_index: idx,
                    block_id,
                })
            }
            _ => Err(AdapterError::AnchorDangling {
                client: anchor.client,
                clock: anchor.clock,
            }),
        }
    }

    /// 映射表查询（语义锚 → 已登记的 CRDT ID；None = 未登记）。
    pub fn anchor_of(&self, semantic: &SemanticAnchor) -> Option<CrdtAnchor> {
        self.anchor_table.get(semantic).copied()
    }

    /// R4.3 验收：映射表侧车序列化（适配层自有格式，版本化，不进三方
    /// 文档格式）。布局：magic "MDCA" + ver u8 + count u32 + 逐条
    /// {tag u8, 变长字段, anchor client u64 LE / clock u32 LE}。
    pub fn export_anchor_sidecar(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.anchor_table.len() * 32);
        out.extend_from_slice(b"MDCA");
        out.push(1u8); // 版本 1
        out.extend_from_slice(&(self.anchor_table.len() as u32).to_le_bytes());
        // 确定性序：HashMap 无序 → 按 (tag, 字段) 排序保证字节稳定
        let mut entries: Vec<(&SemanticAnchor, &CrdtAnchor)> = self.anchor_table.iter().collect();
        entries.sort_by(|a, b| semantic_key(a.0).cmp(&semantic_key(b.0)));
        for (sem, anc) in entries {
            match sem {
                SemanticAnchor::MdBlock { block_id } => {
                    out.push(1u8);
                    push_str(&mut out, block_id);
                }
                SemanticAnchor::OfficeBookmark { name, block_index } => {
                    out.push(2u8);
                    push_str(&mut out, name);
                    out.extend_from_slice(&block_index.to_le_bytes());
                }
                SemanticAnchor::PdfAnnotRef { page, label, block_index } => {
                    out.push(3u8);
                    out.extend_from_slice(&page.to_le_bytes());
                    push_str(&mut out, label);
                    out.extend_from_slice(&block_index.to_le_bytes());
                }
            }
            out.extend_from_slice(&anc.client.to_le_bytes());
            out.extend_from_slice(&anc.clock.to_le_bytes());
        }
        out
    }

    /// 侧车导入（替换式；返回导入条数）。
    pub fn import_anchor_sidecar(&mut self, bytes: &[u8]) -> Result<usize, AdapterError> {
        if bytes.len() < 9 || &bytes[0..4] != b"MDCA" {
            return Err(AdapterError::SidecarFormat("magic 缺失".into()));
        }
        if bytes[4] != 1u8 {
            return Err(AdapterError::SidecarFormat(format!(
                "不支持的版本 {}",
                bytes[4]
            )));
        }
        let count = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
        let mut pos = 9usize;
        let mut table = HashMap::with_capacity(count);
        let read_str = |pos: &mut usize| -> Result<String, AdapterError> {
            if *pos + 4 > bytes.len() {
                return Err(AdapterError::SidecarFormat("字符串长度越界".into()));
            }
            let n = u32::from_le_bytes([
                bytes[*pos],
                bytes[*pos + 1],
                bytes[*pos + 2],
                bytes[*pos + 3],
            ]) as usize;
            *pos += 4;
            if *pos + n > bytes.len() {
                return Err(AdapterError::SidecarFormat("字符串体越界".into()));
            }
            let s = String::from_utf8(bytes[*pos..*pos + n].to_vec())
                .map_err(|_| AdapterError::SidecarFormat("UTF-8 解码失败".into()))?;
            *pos += n;
            Ok(s)
        };
        let read_u32 = |pos: &mut usize| -> Result<u32, AdapterError> {
            if *pos + 4 > bytes.len() {
                return Err(AdapterError::SidecarFormat("u32 越界".into()));
            }
            let v = u32::from_le_bytes([
                bytes[*pos],
                bytes[*pos + 1],
                bytes[*pos + 2],
                bytes[*pos + 3],
            ]);
            *pos += 4;
            Ok(v)
        };
        for _ in 0..count {
            let tag = *bytes
                .get(pos)
                .ok_or_else(|| AdapterError::SidecarFormat("tag 越界".into()))?;
            pos += 1;
            let semantic = match tag {
                1u8 => SemanticAnchor::MdBlock {
                    block_id: read_str(&mut pos)?,
                },
                2u8 => SemanticAnchor::OfficeBookmark {
                    name: read_str(&mut pos)?,
                    block_index: read_u32(&mut pos)?,
                },
                3u8 => SemanticAnchor::PdfAnnotRef {
                    page: read_u32(&mut pos)?,
                    label: read_str(&mut pos)?,
                    block_index: read_u32(&mut pos)?,
                },
                other => {
                    return Err(AdapterError::SidecarFormat(format!(
                        "未知语义锚 tag {}",
                        other
                    )))
                }
            };
            if pos + 12 > bytes.len() {
                return Err(AdapterError::SidecarFormat("anchor 载荷越界".into()));
            }
            let client = u64::from_le_bytes([
                bytes[pos],
                bytes[pos + 1],
                bytes[pos + 2],
                bytes[pos + 3],
                bytes[pos + 4],
                bytes[pos + 5],
                bytes[pos + 6],
                bytes[pos + 7],
            ]);
            pos += 8;
            let clock = u32::from_le_bytes([
                bytes[pos],
                bytes[pos + 1],
                bytes[pos + 2],
                bytes[pos + 3],
            ]);
            pos += 4;
            table.insert(semantic, CrdtAnchor { client, clock });
        }
        let n = table.len();
        self.anchor_table = table;
        Ok(n)
    }

    // ── 快照面（D1③ 权威快照——单点编码）─────────────────────────

    /// 权威快照：当前时刻状态的 update 编码（skip_gc 与快照同向，yrs-b
    /// §2.1 事实 6——只有 skip_gc 文档能产出）。**单点编码语义**：收敛
    /// 双端的本地全量编码字节可能不同（yrs-a 发现①），权威端以此为唯
    /// 一权威产物；对端以 `apply_update` 消费。
    pub fn authoritative_snapshot(&self) -> Result<Vec<u8>, AdapterError> {
        let doc = &self.doc;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            let snap = doc.transact_mut().snapshot();
            let mut enc = EncoderV1::new();
            doc.transact_mut()
                .encode_state_from_snapshot(&snap, &mut enc)
                .map_err(|e| AdapterError::InvalidInput(e.to_string()))?;
            Ok(enc.to_vec())
        }))
        .unwrap_or_else(|p| Err(AdapterError::Panic(panic_message(&p))))
    }

    // ── 探针面（#[doc(hidden)]——仅为探针/测试暴露，非产品面）────────

    /// B1.1 外部可观测证明：skip_gc 生效（读 DocOptions 单点真源）。
    #[doc(hidden)]
    pub fn __probe_skip_gc(&self) -> bool {
        self.doc.skip_gc()
    }

    /// StickyIndex wire 往返（yrs-b §5.3 验收第一项）：锚点 ID →
    /// StickyIndex → Encode → Decode → get_offset，返回解算下标；
    /// 与直接 get_offset 的一致性由探针断言。
    #[doc(hidden)]
    pub fn __probe_sticky_wire_roundtrip(
        &self,
        anchor: CrdtAnchor,
    ) -> Result<Option<u32>, AdapterError> {
        let array = self.doc.get_or_insert_array("blocks");
        let txn = self.doc.transact();
        let sticky =
            StickyIndex::from_id(ID::new(ClientID::new(anchor.client), anchor.clock), Assoc::After);
        // wire 往返：encode → decode
        let bytes = sticky.encode_v1();
        let restored = StickyIndex::decode_v1(&bytes)
            .map_err(|e| AdapterError::Decode(e.to_string()))?;
        assert_eq!(restored, sticky, "wire 往返保真（StickyIndex Eq）");
        let direct = sticky.get_offset(&txn).map(|o| o.index);
        let via_wire = restored.get_offset(&txn).map(|o| o.index);
        assert_eq!(direct, via_wire, "直接与 wire 往返解算一致");
        let _ = array;
        Ok(via_wire)
    }

    /// text 根当前内容（探针断言用）。
    #[doc(hidden)]
    pub fn __probe_text(&self) -> String {
        self.doc.get_or_insert_text("text").get_string(&self.doc.transact())
    }

    /// blocks 根 block_id 序列（探针断言用）。
    #[doc(hidden)]
    pub fn __probe_block_ids(&self) -> Vec<String> {
        let array = self.doc.get_or_insert_array("blocks");
        let txn = self.doc.transact();
        let mut ids = Vec::with_capacity(array.len(&txn) as usize);
        for i in 0..array.len(&txn) {
            if let Some(yrs::Out::YMap(m)) = array.get(&txn, i) {
                if let Some(yrs::Out::Any(yrs::Any::String(id))) = m.get(&txn, "block_id") {
                    ids.push(id.to_string());
                }
            }
        }
        ids
    }

    /// blocks 根某块的 text（探针断言用；None = 块不存在）。
    #[doc(hidden)]
    pub fn __probe_block_text(&self, block_id: &str) -> Option<String> {
        let array = self.doc.get_or_insert_array("blocks");
        let txn = self.doc.transact();
        for i in 0..array.len(&txn) {
            if let Some(yrs::Out::YMap(m)) = array.get(&txn, i) {
                if let Some(yrs::Out::Any(yrs::Any::String(id))) = m.get(&txn, "block_id") {
                    if &*id == block_id {
                        if let Some(yrs::Out::Any(yrs::Any::String(t))) = m.get(&txn, "text") {
                            return Some(t.to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

/// panic payload 转字符串（不依赖 Debug 格式稳定性）。
fn panic_message(p: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

/// 语义锚的确定性排序键（侧车字节稳定用）。
fn semantic_key(s: &SemanticAnchor) -> (u8, String, u32, String) {
    match s {
        SemanticAnchor::MdBlock { block_id } => (1u8, block_id.clone(), 0, String::new()),
        SemanticAnchor::OfficeBookmark { name, block_index } => {
            (2u8, name.clone(), *block_index, String::new())
        }
        SemanticAnchor::PdfAnnotRef { page, label, block_index } => {
            (3u8, label.clone(), *page, format!("b{}", block_index))
        }
    }
}

/// 侧车字符串写入（u32 LE 长度前缀 + UTF-8 体）。
fn push_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}


