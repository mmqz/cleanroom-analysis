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
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use yrs::undo::Options as UndoOptions;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encode, Encoder};
use yrs::{
    Array, ClientID, Doc, GetString, In, Map, MapPrelim, Options as DocOptions, Origin,
    ReadTxn, StateVector, Text, Transact, UndoManager, Update,
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
    /// 删除块（按 block_id 定位；不存在 → InvalidInput）
    RemoveBlock { block_id: String },
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

    // ── 探针面（#[doc(hidden)]——仅为探针/测试暴露，非产品面）────────

    /// B1.1 外部可观测证明：skip_gc 生效（读 DocOptions 单点真源）。
    #[doc(hidden)]
    pub fn __probe_skip_gc(&self) -> bool {
        self.doc.skip_gc()
    }

    /// 快照门（yrs-b §2.1 事实 6：skip_gc 与权威快照同向——Ok 即证
    /// skip_gc；对照面默认 GC → Err(Error::Gc)）。权威快照的完整语义
    /// （单点编码 + 维护窗压缩）在 d1c-t3 实装。
    #[doc(hidden)]
    pub fn __probe_snapshot_gate(&self) -> Result<Vec<u8>, AdapterError> {
        let doc = &self.doc;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            let snap = doc.transact_mut().snapshot();
            let mut enc = yrs::updates::encoder::EncoderV1::new();
            doc.transact_mut()
                .encode_state_from_snapshot(&snap, &mut enc)
                .map_err(|e| AdapterError::InvalidInput(e.to_string()))?;
            Ok(enc.to_vec())
        }))
        .unwrap_or_else(|p| Err(AdapterError::Panic(panic_message(&p))))
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


