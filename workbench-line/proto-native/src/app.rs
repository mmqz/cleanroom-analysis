//! 原型 B（原生 UI 候选）——egui 四面板对照应用。
//!
//! 架构纪律镜像 web 候选：mdcore 核 = 唯一权威正文（path 依赖直连，
//! 「核心不依赖 UI」由本 crate 编译即证）；面板 = 投影 + 命令回核。
//!
//! 测量口径：
//! - 触发帧 = 场景引擎在渲染前置位（本帧渲染即含效果）；settle 型
//!   场景在触发帧 + 2 帧后记录（web 侧 = 2×rAF 对称）。
//! - 拖拽 = XTEST 合成真实 X 事件（X 服务器 → winit → egui → GL →
//!   present 全链），与 web 候选 CDP 真实输入对称。
//! - 滚动/跳转 = 程序化 API（scroll_to_rect / scroll_with_delta），
//!   web 侧对称用 scrollIntoView / scrollTop。

use eframe::egui;
use serde_json::json;
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::gen::{Canvas, Evidence, Table};
use crate::xtest::DragPlan;

pub const LINE_H: f32 = 20.0;
pub const CELL_W: f32 = 90.0;
pub const CELL_H: f32 = 26.0;
pub const N_SCROLL_STEPS: u32 = 40;
pub const N_DRAG_STEPS: u32 = 60;
pub const N_HIT: u32 = 100;
pub const N_FPS: u32 = 300;
pub const SETTLE_FRAMES: u32 = 2;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Body,
    Table,
    Canvas,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettleKind {
    DocOpen,
    DocEdit,
    DocJumpEnd,
    EvidenceRender,
    EvidenceJump,
    TableOpen,
    TableEdit,
    CanvasRender,
}

#[derive(Clone)]
pub enum Scen {
    Settle { name: &'static str, kind: SettleKind, left: u32 },
    TableScroll { step: u32, intervals: Vec<f32> },
    CanvasDrag { phase: u8, frames: u32, intervals: Vec<f32>, t_start: Option<Instant> },
    Fps { left: u32, intervals: Vec<f32> },
}

pub struct LoadMetrics {
    pub font_read_ms: f64,
    pub font_set_ms: f64,
    pub data_load_ms: f64,
    pub core_ingest_ms: f64,
}

/// XTEST 拖拽计划（主线程计算，注入线程按时钟执行）。
pub struct XtestState {
    pub plan: Mutex<Option<DragPlan>>,
    pub fired: AtomicU8,   // 0=未 1=已下发 2=注入线程报告错误
    pub err: Mutex<String>,
    pub stop: AtomicBool,
}

pub struct BenchApp {
    pub t0: Instant,
    pub load: LoadMetrics,
    pub drag_target_idx: Option<usize>,
    pub evidence: Evidence,
    pub table: Table,
    pub scene: Canvas,
    pub core: mdcore::Core,
    pub lines: Vec<String>,
    pub tab: Tab,
    pub hl_lines: HashSet<usize>,
    pub jump_line: Option<usize>,
    pub node_pos: Vec<egui::Pos2>,
    pub canvas_origin: egui::Pos2,
    pub last_ppp: f32,
    pub canvas_scroll_to: Option<usize>,
    pub edit_cell: Option<(usize, usize)>,
    pub table_scroll_cmd: Option<f32>,
    pub queue: VecDeque<Scen>,
    pub cur: Option<Scen>,
    pub t_trigger: Option<Instant>,
    pub metrics: Vec<serde_json::Value>,
    pub ready: bool,
    pub last_frame: Option<Instant>,
    pub edit_check: Option<(u64, bool)>,
    pub drag_anchor: Option<[f32; 2]>,
    pub xtest: Arc<XtestState>,
    pub emitted: bool,
    pub frames_drawn: u64,
}

impl BenchApp {
    pub fn scenario_queue() -> VecDeque<Scen> {
        let s = |name: &'static str, kind: SettleKind| Scen::Settle { name, kind, left: SETTLE_FRAMES };
        VecDeque::from(vec![
            s("doc_open", SettleKind::DocOpen),
            s("doc_edit", SettleKind::DocEdit),
            s("doc_jump_end", SettleKind::DocJumpEnd),
            s("evidence_render", SettleKind::EvidenceRender),
            s("evidence_jump", SettleKind::EvidenceJump),
            s("table_open", SettleKind::TableOpen),
            Scen::TableScroll { step: 0, intervals: vec![] },
            s("table_edit", SettleKind::TableEdit),
            s("canvas_render", SettleKind::CanvasRender),
            Scen::CanvasDrag { phase: 0, frames: 0, intervals: vec![], t_start: None },
            Scen::Fps { left: N_FPS, intervals: vec![] },
        ])
    }

    fn hit_test(&self, p: egui::Pos2) -> Option<usize> {
        self.node_pos.iter().enumerate().find(|(_, np)| {
            p.x >= np.x && p.x <= np.x + 120.0 && p.y >= np.y && p.y <= np.y + 36.0
        }).map(|(i, _)| i)
    }

    fn canvas_hit_bench(&self) -> (f64, f64, usize) {
        let mut rng = crate::gen::Lcg::new(77);
        let t = Instant::now();
        let mut hits = 0usize;
        for _ in 0..N_HIT {
            let x = rng.below(7000) as f32 + 20.0;
            let y = rng.below(2200) as f32 + 20.0;
            if self.hit_test(egui::pos2(x, y)).is_some() {
                hits += 1;
            }
        }
        let total_ms = t.elapsed().as_secs_f64() * 1000.0;
        (total_ms, total_ms / N_HIT as f64, hits)
    }

    fn do_edit(&mut self) {
        let target = 5000usize;
        let node = self.core.tree.top_level()[target];
        let path: Vec<usize> = self.core.tree.path_of(node).expect("path").to_vec();
        let rev_before = self.core.revision();
        let res = self.core.replace_text(
            mdcore::anchor::TextTarget::ByPath { path, start: 5, end: 5 },
            "✎".to_string(),
        );
        let ok = res.is_ok();
        let line_idx = target; // doc_open 后投影 = 块 1:1 行（无分隔行）
        if let Some(l) = self.lines.get_mut(line_idx) {
            // 核心 op 的 5 = 字符偏移（ropey char 语义）；String::insert 是
            // 字节索引——CJK 行须经字符边界换算（实施发现：直接 insert(5) 在
            // 标题行 panic——'## ' 后首个 CJK 字符横跨字节 3-5）
            let byte_idx = l.char_indices().nth(5).map(|(b, _)| b).unwrap_or(l.len());
            l.insert(byte_idx, '✎');
        }
        self.edit_check = Some((rev_before, ok));
    }

    fn jump_to_claim(&mut self, k: usize) {
        let c = self.evidence.claims[k].clone();
        let line = c.block; // 1:1 投影
        self.hl_lines = HashSet::from([line]);
        self.jump_line = Some(line);
    }
}

impl eframe::App for BenchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let t_now = Instant::now();
        self.frames_drawn += 1;
        ui.ctx().request_repaint();
        // 窗口原点追踪（egui 坐标 → X 根坐标换算；Xvfb 全屏 ≈ (0,0)）
        self.last_ppp = ui.ctx().pixels_per_point();

        // —— 场景引擎（渲染前置位：触发帧即渲染效果）——
        self.step_scenarios(t_now);

        if !self.ready {
            self.ready = true;
            self.metrics.push(json!({
                "name": "startup",
                "ms": self.t0.elapsed().as_secs_f64() * 1000.0,
                "extra": {
                    "font_read_ms": self.load.font_read_ms,
                    "font_set_ms": self.load.font_set_ms,
                    "data_load_ms": self.load.data_load_ms,
                    "core_ingest_ms": self.load.core_ingest_ms,
                }
            }));
        }

        egui::Panel::top("tabs").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Body, "正文");
                ui.selectable_value(&mut self.tab, Tab::Table, "表格");
                ui.selectable_value(&mut self.tab, Tab::Canvas, "画布");
                ui.separator();
                ui.weak(format!("frames={}", self.frames_drawn));
            });
        });

        egui::Panel::left("evidence")
            .resizable(false)
            .default_size(280.0)
            .show(ui, |ui| self.render_evidence(ui));

        egui::CentralPanel::default().show(ui, |ui| match self.tab {
            Tab::Body => self.render_body(ui),
            Tab::Table => self.render_table(ui),
            Tab::Canvas => self.render_canvas(ui),
        });

        if self.emitted || (self.cur.is_none() && self.queue.is_empty() && self.ready) {
            self.emit_if_done(ui.ctx());
        }
    }
}

impl BenchApp {
    fn render_body(&mut self, ui: &mut egui::Ui) {
        let n = self.lines.len();
        let total_h = n as f32 * LINE_H;
        let hl = self.hl_lines.clone();
        let jump = self.jump_line.take();
        egui::ScrollArea::vertical().show_viewport(ui, |ui, vp| {
            let origin = ui.next_widget_position();
            let w = ui.available_width().max(1.0);
            ui.allocate_space(egui::vec2(w, total_h));
            let first = ((vp.min.y / LINE_H).floor() as usize).saturating_sub(1);
            let last = (((vp.max.y / CELL_H.max(LINE_H)).ceil() as usize) + 1).min(n);
            let painter = ui.painter();
            for i in first..last {
                let pos = origin + egui::vec2(0.0, i as f32 * LINE_H);
                if hl.contains(&i) {
                    painter.rect_filled(
                        egui::Rect::from_min_size(pos, egui::vec2(w, LINE_H)),
                        2.0,
                        egui::Color32::from_rgb(255, 240, 200),
                    );
                }
                painter.text(
                    pos,
                    egui::Align2::LEFT_TOP,
                    &self.lines[i],
                    egui::FontId::proportional(15.0),
                    egui::Color32::from_gray(40),
                );
            }
            if let Some(target) = jump {
                let r = egui::Rect::from_min_size(
                    origin + egui::vec2(0.0, target as f32 * LINE_H),
                    egui::vec2(10.0, LINE_H),
                );
                ui.scroll_to_rect(r, Some(egui::Align::Center));
            }
        });
    }

    fn render_evidence(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("证据侧栏（100 条）");
            let mut clicked73 = false;
            for (k, c) in self.evidence.claims.iter().enumerate() {
                let color = match c.status.as_str() {
                    "verified" => egui::Color32::from_rgb(22, 120, 60),
                    "stale" => egui::Color32::from_rgb(170, 60, 30),
                    _ => egui::Color32::from_gray(120),
                };
                let resp = ui
                    .horizontal_wrapped(|ui| {
                        ui.colored_label(color, format!("[{}] {}", c.status, c.id));
                        ui.weak(format!("块 {} · 值 {}", c.block, c.value));
                    })
                    .response;
                if k == 73 && resp.clicked() {
                    clicked73 = true;
                }
            }
            if clicked73 {
                self.jump_to_claim(73);
            }
        });
    }

    fn render_table(&mut self, ui: &mut egui::Ui) {
        let rows = self.table.rows;
        let cols = self.table.cols;
        let total = egui::vec2(cols as f32 * CELL_W, rows as f32 * CELL_H);
        let edit = self.edit_cell;
        let scroll_cmd = self.table_scroll_cmd.take();
        egui::ScrollArea::both().show_viewport(ui, |ui, vp| {
            if let Some(d) = scroll_cmd {
                ui.scroll_with_delta(egui::vec2(0.0, d));
            }
            let origin = ui.next_widget_position();
            ui.allocate_space(total);
            let c0 = ((vp.min.x / CELL_W).floor() as usize).saturating_sub(1);
            let c1 = (((vp.max.x / CELL_W).ceil() as usize) + 1).min(cols);
            let r0 = ((vp.min.y / CELL_H).floor() as usize).saturating_sub(1);
            let r1 = (((vp.max.y / CELL_H).ceil() as usize) + 1).min(rows);
            let painter = ui.painter();
            for r in r0..r1 {
                for c in c0..c1 {
                    let pos = origin + egui::vec2(c as f32 * CELL_W, r as f32 * CELL_H);
                    let rect = egui::Rect::from_min_size(pos, egui::vec2(CELL_W, CELL_H));
                    let edited = edit == Some((r, c));
                    painter.rect_filled(
                        rect,
                        1.0,
                        if edited {
                            egui::Color32::from_rgb(255, 235, 200)
                        } else if r % 2 == 0 {
                            egui::Color32::from_gray(250)
                        } else {
                            egui::Color32::WHITE
                        },
                    );
                    painter.rect_stroke(rect, 1.0, egui::Stroke::new(1.0, egui::Color32::from_gray(210)), egui::StrokeKind::Middle);
                    painter.text(
                        pos + egui::vec2(6.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        &self.table.cells[r][c],
                        egui::FontId::monospace(12.0),
                        egui::Color32::from_gray(45),
                    );
                }
            }
        });
    }

    fn render_canvas(&mut self, ui: &mut egui::Ui) {
        let total = egui::vec2(7300.0, 2320.0);
        let scroll_to = self.canvas_scroll_to.take();
        egui::ScrollArea::both()
            .scroll_source(egui::containers::scroll_area::ScrollSource { drag: egui::containers::scroll_area::DragScroll::Never, ..Default::default() })
            .show_viewport(ui, |ui, _vp| {
                let origin = ui.next_widget_position();
                self.canvas_origin = origin;
                let scene_rect = egui::Rect::from_min_size(origin, total);
                ui.allocate_space(total);
                let painter = ui.painter();
                for e in &self.scene.edges {
                    let a = self.node_pos[e.from];
                    let b = self.node_pos[e.to];
                    painter.line_segment(
                        [a + egui::vec2(60.0, 18.0), b + egui::vec2(60.0, 18.0)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 190, 205)),
                    );
                }
                for (i, np) in self.node_pos.iter().enumerate() {
                    let rect = egui::Rect::from_min_size(*np, egui::vec2(120.0, 36.0));
                    let hot = self.drag_target_idx == Some(i);
                    painter.rect_filled(
                        rect,
                        4.0,
                        if hot {
                            egui::Color32::from_rgb(255, 224, 130)
                        } else {
                            egui::Color32::from_rgb(214, 226, 245)
                        },
                    );
                    painter.rect_stroke(
                        rect,
                        4.0,
                        if hot {
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 120, 20))
                        } else {
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 130, 160))
                        },
                        egui::StrokeKind::Middle,
                    );
                    painter.text(
                        *np + egui::vec2(8.0, 8.0),
                        egui::Align2::LEFT_TOP,
                        &self.scene.nodes[i].label,
                        egui::FontId::proportional(13.0),
                        egui::Color32::from_gray(40),
                    );
                }
                if let Some(idx) = scroll_to {
                    // scroll_to_rect 取屏幕坐标（= origin + 内容坐标——与
                    // render_body 跳转同口径；初版误传内容坐标致滚动过冲）
                    let np = self.node_pos[idx];
                    let r = egui::Rect::from_min_size(
                        origin + np.to_vec2(),
                        egui::vec2(120.0, 36.0),
                    );
                    ui.scroll_to_rect(r, Some(egui::Align::Center));
                }
                let resp = ui.allocate_rect(scene_rect, egui::Sense::click_and_drag());
                if resp.drag_started() {
                    if let Some(p) = resp.interact_pointer_pos() {
                        let local = p - origin.to_vec2();
                        self.drag_target_idx = self.hit_test(local);
                    }
                }
                if resp.dragged() {
                    if let (Some(idx), Some(p)) = (self.drag_target_idx, resp.interact_pointer_pos()) {
                        let local = p - origin.to_vec2();
                        if let Some(np) = self.node_pos.get_mut(idx) {
                            *np = local - egui::vec2(60.0, 18.0);
                        }
                    }
                }
                if resp.drag_stopped() {
                    self.drag_target_idx = None;
                }
            });
    }

    // ———— 场景引擎 ————

    fn step_scenarios(&mut self, t_now: Instant) {
        if !self.ready {
            return;
        }
        if self.cur.is_none() {
            if let Some(mut scen) = self.queue.pop_front() {
                self.t_trigger = Some(t_now);
                self.trigger(&mut scen);
                self.cur = Some(scen);
            }
        }

        let Some(mut scen) = self.cur.take() else { return };
        match &mut scen {
            Scen::Settle { name, kind, left } => {
                if *left == 0 {
                    self.record_settle(name, *kind, t_now);
                } else {
                    *left -= 1;
                    self.cur = Some(scen);
                }
            }
            Scen::TableScroll { step, intervals } => {
                if let Some(t) = self.last_frame {
                    intervals.push((t_now.duration_since(t).as_secs_f64() * 1000.0) as f32);
                }
                if *step < N_SCROLL_STEPS {
                    self.table_scroll_cmd = Some(50.0 * CELL_H);
                    *step += 1;
                    self.cur = Some(scen);
                } else {
                    let ms = self.trigger_ms(t_now);
                    self.metrics.push(json!({
                        "name": "table_scroll", "ms": ms,
                        "extra": {"steps": N_SCROLL_STEPS, "rows_moved": N_SCROLL_STEPS * 50,
                                   "mean_frame_ms": mean(intervals), "p95_frame_ms": p95(intervals)}
                    }));
                }
            }
            Scen::CanvasDrag { phase, frames, intervals, t_start } => {
                // 详见 trigger()：phase 0 = 先滚到节点 500（3 帧稳定），phase 1 起按下计划
                if let Some(t) = self.last_frame {
                    intervals.push((t_now.duration_since(t).as_secs_f64() * 1000.0) as f32);
                }
                match *phase {
                    0 => {
                        *frames += 1;
                        if *frames > 3 {
                            // 滚动稳定后按新鲜坐标下发 XTEST 计划
                            self.fire_drag_plan();
                            *t_start = Some(t_now);
                            *frames = 0;
                            *phase = 1;
                        }
                        self.cur = Some(scen);
                    }
                    1 => {
                        *frames += 1;
                        let done = self.xtest.fired.load(Ordering::SeqCst) == 1;
                        let timeout = t_start.map(|t| t_now.duration_since(t) > Duration::from_secs(5)).unwrap_or(false);
                        if done || timeout {
                            // 注入完成后多等 5 帧收尾
                            *phase = 2;
                            *frames = 0;
                        }
                        self.cur = Some(scen);
                    }
                    _ => {
                        *frames += 1;
                        if *frames > 5 {
                            let ms = self.trigger_ms(t_now);
                            let d = self.drag_anchor.unwrap_or([0.0, 0.0]);
                            let xerr = self.xtest.err.lock().unwrap().clone();
                            self.metrics.push(json!({
                                "name": "canvas_drag", "ms": ms,
                                "extra": {"mean_frame_ms": mean(intervals), "p95_frame_ms": p95(intervals),
                                           "node500_dx": self.node_pos[500].x - d[0],
                                           "node500_dy": self.node_pos[500].y - d[1],
                                           "xtest_error": if xerr.is_empty() { serde_json::Value::Null } else { json!(xerr) }}
                            }));
                        } else {
                            self.cur = Some(scen);
                        }
                    }
                }
            }
            Scen::Fps { left, intervals } => {
                if let Some(t) = self.last_frame {
                    intervals.push((t_now.duration_since(t).as_secs_f64() * 1000.0) as f32);
                }
                if *left > 0 {
                    *left -= 1;
                    self.cur = Some(scen);
                } else {
                    let (hit_total_ms, hit_mean_ms, hits) = self.canvas_hit_bench();
                    self.metrics.push(json!({
                        "name": "canvas_hit", "ms": hit_total_ms,
                        "extra": {"n": N_HIT, "mean_us": hit_mean_ms * 1000.0, "hits": hits}
                    }));
                    self.metrics.push(json!({
                        "name": "fps_sustain",
                        "ms": mean(intervals) * intervals.len() as f64,
                        "extra": {"frames": intervals.len(), "mean_frame_ms": mean(intervals),
                                   "p95_frame_ms": p95(intervals),
                                   "min_frame_ms": intervals.iter().cloned().fold(f32::INFINITY, f32::min),
                                   "max_frame_ms": intervals.iter().cloned().fold(0.0f32, f32::max)}
                    }));
                }
            }
        }
        self.last_frame = Some(t_now);
    }

    fn trigger(&mut self, scen: &mut Scen) {
        match scen {
            Scen::Settle { kind, .. } => match kind {
                SettleKind::DocOpen => {
                    let md = mdcore::markdown::tree_to_markdown(&self.core.tree);
                    self.lines = md.split('\n').map(str::to_string).collect();
                }
                SettleKind::DocEdit => self.do_edit(),
                SettleKind::DocJumpEnd => {
                    self.jump_line = Some(self.lines.len().saturating_sub(2))
                }
                SettleKind::EvidenceRender => {
                    self.hl_lines.clear();
                }
                SettleKind::EvidenceJump => self.jump_to_claim(73),
                SettleKind::TableOpen => self.tab = Tab::Table,
                SettleKind::TableEdit => {
                    self.table.cells[2005][7] = "EDITED-✎".to_string();
                    self.edit_cell = Some((2005, 7));
                }
                SettleKind::CanvasRender => self.tab = Tab::Canvas,
            },
            Scen::TableScroll { .. } => {
                self.tab = Tab::Table;
                self.table_scroll_cmd = Some(50.0 * CELL_H);
            }
            Scen::CanvasDrag { .. } => {
                // 先滚动画布到节点 500（触发帧渲染即滚动；3 帧后 fire_drag_plan）
                self.canvas_scroll_to = Some(500);
            }
            Scen::Fps { .. } => {}
        }
    }

    fn record_settle(&mut self, name: &str, kind: SettleKind, t_now: Instant) {
        let ms = self.trigger_ms(t_now);
        let extra = match kind {
            SettleKind::DocOpen => json!({"lines": self.lines.len()}),
            SettleKind::DocEdit => {
                let (rev_before, ok) = self.edit_check.unwrap_or((0, false));
                json!({"rev_before": rev_before, "rev_after": self.core.revision(),
                       "op_ok": ok, "text_has_edit": self.lines.get(5000).map(|l| l.contains('✎')).unwrap_or(false)})
            }
            SettleKind::DocJumpEnd => json!({"lines": self.lines.len()}),
            SettleKind::EvidenceRender => json!({"claims": self.evidence.claims.len()}),
            SettleKind::EvidenceJump => json!({"claim": "c-073", "hl_lines": self.hl_lines.len()}),
            SettleKind::TableOpen => json!({"rows": self.table.rows, "cols": self.table.cols}),
            SettleKind::TableEdit => json!({"cell": "r2005c7", "val": self.table.cells[2005][7]}),
            SettleKind::CanvasRender => json!({"nodes": self.scene.nodes.len(), "edges": self.scene.edges.len()}),
        };
        self.metrics.push(json!({"name": name, "ms": ms, "extra": extra}));
    }

    fn fire_drag_plan(&mut self) {
        self.drag_anchor = Some([self.node_pos[500].x, self.node_pos[500].y]);
        // canvas_origin = 内容原点的绝对屏幕坐标（含滚动偏移与窗口原点）；
        // X 根坐标 = (canvas_origin + 节点场景坐标) × pixels_per_point
        let ppp = self.last_ppp;
        let sx = (self.canvas_origin.x + self.node_pos[500].x + 60.0) * ppp;
        let sy = (self.canvas_origin.y + self.node_pos[500].y + 18.0) * ppp;
        let s = egui::pos2(sx, sy);
        let e = s + egui::vec2(37.0 * ppp, 21.0 * ppp);
        *self.xtest.plan.lock().unwrap() = Some(DragPlan {
            start: (s.x, s.y),
            end: (e.x, e.y),
            steps: N_DRAG_STEPS,
            interval_ms: 16,
        });
    }

    fn trigger_ms(&self, t_now: Instant) -> f64 {
        self.t_trigger.map(|t| t_now.duration_since(t).as_secs_f64() * 1000.0).unwrap_or(0.0)
    }

    fn emit_if_done(&mut self, ctx: &egui::Context) {
        if self.emitted {
            return;
        }
        self.emitted = true;
        self.xtest.stop.store(true, Ordering::SeqCst);
        let rss = read_rss();
        let out = json!({
            "side": "native",
            "env": {"renderer": "egui 0.37 + glow + winit/X11（Xvfb 软渲染）", "os": std::env::consts::OS},
            "scenarios": self.metrics.clone(),
            "memory": rss,
        });
        println!("\n===BENCH_JSON===\n{out}\n===END===");
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

pub fn mean(v: &[f32]) -> f64 {
    if v.is_empty() { return 0.0; }
    v.iter().map(|x| *x as f64).sum::<f64>() / v.len() as f64
}

pub fn p95(v: &[f32]) -> f64 {
    if v.is_empty() { return 0.0; }
    let mut s: Vec<f64> = v.iter().map(|x| *x as f64).collect();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() as f64 * 0.95) as usize).min(s.len() - 1)]
}

pub fn read_rss() -> serde_json::Value {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let (mut rss, mut hwm) = (0, 0);
    for line in status.lines() {
        if let Some(x) = line.strip_prefix("VmRSS:") {
            rss = x.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        }
        if let Some(x) = line.strip_prefix("VmHWM:") {
            hwm = x.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        }
    }
    json!({"vm_rss_kb": rss, "vm_hwm_kb": hwm})
}
