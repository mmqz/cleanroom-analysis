//! 共享对照数据集生成（双原型同载的单一真源）。
//!
//! 确定性：LCG 种子 42——两侧（web/native）与重跑之间字节一致。
//! 形状连续自 mdc-workbench bench_long_doc 的合成语义（中文正文、
//! 互异指纹、主张值嵌锚段），但独立为纯 JSON 载体（宿主对照测的是
//! UI 面，非证据引擎内环）。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 简易确定性 LCG（免 rand 依赖；宿主对照只需可复现，不需密码学质量）。
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 16
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocBlock {
    pub i: usize,
    pub kind: String, // heading1 | heading2 | paragraph | list
    pub text: String, // 不含 markdown 前缀（kind 即前缀语义）
}

#[derive(Serialize, Deserialize)]
pub struct Doc {
    pub format: u32,
    pub n_blocks: usize,
    pub blocks: Vec<DocBlock>,
    /// 全量 markdown 文本（blocks 以 "\n\n" 连接、按 kind 加前缀）。
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claim {
    pub id: String,
    pub block: usize,
    pub quote: String,
    pub value: String,
    pub status: String, // verified | stale | pending
}

#[derive(Serialize, Deserialize)]
pub struct Evidence {
    pub format: u32,
    pub claims: Vec<Claim>,
}

#[derive(Serialize, Deserialize)]
pub struct Table {
    pub format: u32,
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Canvas {
    pub format: u32,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub const N_BLOCKS: usize = 10_000;
pub const N_CLAIMS: usize = 100;
pub const T_ROWS: usize = 5_000;
pub const T_COLS: usize = 100;
pub const C_NODES: usize = 1_000;
pub const C_EDGES: usize = 1_500;

fn block_text(i: usize, kind: &str, rng: &mut Lcg) -> String {
    match kind {
        "heading1" => "工作台基准正文（合成）".to_string(),
        "heading2" => format!("第 {} 节：结构与语义", i / 500),
        "list" => format!("条目 {}：合成要点，指纹互异编号 {}。", i, rng.below(99_999)),
        _ => format!(
            "第 {} 段：基准合成正文，内容互异以使指纹独立，附带变化词 {} 与若干说明性文字用于排版测量。",
            i,
            rng.below(9_999_999)
        ),
    }
}

fn block_kind(i: usize) -> &'static str {
    if i == 0 {
        "heading1"
    } else if i.is_multiple_of(500) {
        "heading2"
    } else if i % 7 == 3 {
        "list"
    } else {
        "paragraph"
    }
}

fn md_line(b: &DocBlock) -> String {
    match b.kind.as_str() {
        "heading1" => format!("# {}", b.text),
        "heading2" => format!("## {}", b.text),
        "list" => format!("- {}", b.text),
        _ => b.text.clone(),
    }
}

pub fn gen_doc() -> Doc {
    let mut rng = Lcg::new(42);
    let blocks: Vec<DocBlock> = (0..N_BLOCKS)
        .map(|i| {
            let kind = block_kind(i);
            DocBlock { i, kind: kind.to_string(), text: block_text(i, kind, &mut rng) }
        })
        .collect();
    let text = blocks.iter().map(md_line).collect::<Vec<_>>().join("\n\n");
    Doc { format: 1, n_blocks: blocks.len(), blocks, text }
}

pub fn gen_evidence(doc: &Doc) -> Evidence {
    let mut rng = Lcg::new(43);
    // 锚段均匀铺开：段 1 起避开标题（与 bench_long_doc 同语义）。
    let claims: Vec<Claim> = (0..N_CLAIMS)
        .map(|k| {
            let block = 1 + (k as u64 * (N_BLOCKS as u64 - 2) / N_CLAIMS as u64) as usize;
            let b = &doc.blocks[block];
            let quote: String = b.text.chars().take(12).collect();
            let value = format!("{}", 100 + rng.below(900));
            let status = match k % 10 {
                0 => "stale",
                1 | 2 => "pending",
                _ => "verified",
            };
            Claim {
                id: format!("c-{k:03}"),
                block,
                quote,
                value,
                status: status.to_string(),
            }
        })
        .collect();
    Evidence { format: 1, claims }
}

pub fn gen_table() -> Table {
    let mut rng = Lcg::new(44);
    let cells: Vec<Vec<String>> = (0..T_ROWS)
        .map(|r| {
            (0..T_COLS)
                .map(|c| {
                    if r == 0 {
                        format!("列 {c}")
                    } else if (r + c) % 3 == 0 {
                        format!("{:.1}", ((r * 100 + c) % 1000) as f64 / 7.0)
                    } else {
                        format!("r{r}c{c}{}", rng.below(90) + 10)
                    }
                })
                .collect()
        })
        .collect();
    Table { format: 1, rows: T_ROWS, cols: T_COLS, cells }
}

pub fn gen_canvas() -> Canvas {
    let mut rng = Lcg::new(45);
    let nodes: Vec<Node> = (0..C_NODES)
        .map(|i| {
            let col = i % 40;
            let row = i / 40;
            Node {
                id: i,
                x: 40.0 + col as f32 * 180.0 + rng.below(40) as f32,
                y: 40.0 + row as f32 * 92.0 + rng.below(24) as f32,
                w: 120.0,
                h: 36.0,
                label: format!("节点 {i}"),
            }
        })
        .collect();
    let edges: Vec<Edge> = (0..C_EDGES)
        .map(|k| {
            let from = (k * 2) % C_NODES;
            let mut to = (from * 7 + 3 + k) % C_NODES;
            if to == from {
                to = (to + 1) % C_NODES;
            }
            Edge { from, to }
        })
        .collect();
    Canvas { format: 1, nodes, edges }
}

/// 生成四数据集到目录；返回 (文件名, 字节数) 清单。
pub fn write_all(dir: &Path) -> std::io::Result<Vec<(String, u64)>> {
    std::fs::create_dir_all(dir)?;
    let mut out = Vec::new();
    for (name, val) in [
        ("doc.json", serde_json::to_value(gen_doc()).unwrap()),
        ("evidence.json", serde_json::to_value(gen_evidence(&gen_doc())).unwrap()),
        ("table.json", serde_json::to_value(gen_table()).unwrap()),
        ("canvas.json", serde_json::to_value(gen_canvas()).unwrap()),
    ] {
        let s = serde_json::to_string(&val).unwrap();
        let p = dir.join(name);
        std::fs::write(&p, &s)?;
        out.push((name.to_string(), s.len() as u64));
    }
    Ok(out)
}
