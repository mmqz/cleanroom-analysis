//! 原型 B（原生 UI 候选）入口。
//!
//! 模式：
//!   proto-native --gen DIR    生成共享对照数据集（与 web 候选同源）
//!   proto-native --bench DIR  装载数据集 → egui 四面板 → 场景基准
//!                            （stdout 标记段输出 JSON）

mod app;
mod gen;
mod xtest;

use app::{BenchApp, LoadMetrics, XtestState};
use eframe::egui;
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const FONT_PATH: &str = "/usr/share/fonts/truetype/chinese/NotoSansSC[wght].ttf";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--gen") {
        let dir = args.get(pos + 1).expect("--gen <DIR>").clone();
        let out = gen::write_all(std::path::Path::new(&dir)).expect("gen datasets");
        for (name, bytes) in out {
            println!("{name}: {bytes} bytes");
        }
        return;
    }
    let bench_pos = args.iter().position(|a| a == "--bench").expect("--bench <DIR>");
    let dir = std::path::Path::new(args.get(bench_pos + 1).expect("--bench <DIR>"));

    let t0 = Instant::now();

    // —— 字体读取（真实启动成本：5.4MB CJK 变量字体）——
    let tf = Instant::now();
    let font_bytes = std::fs::read(FONT_PATH).ok();
    let font_read_ms = tf.elapsed().as_secs_f64() * 1000.0;

    // —— 数据集装载（读 + serde 解析 ×4）——
    let td = Instant::now();
    let doc: gen::Doc = serde_json::from_str(&std::fs::read_to_string(dir.join("doc.json")).expect("doc.json"))
        .expect("parse doc.json");
    let evidence: gen::Evidence =
        serde_json::from_str(&std::fs::read_to_string(dir.join("evidence.json")).expect("evidence.json"))
            .expect("parse evidence.json");
    let table: gen::Table =
        serde_json::from_str(&std::fs::read_to_string(dir.join("table.json")).expect("table.json"))
            .expect("parse table.json");
    let scene: gen::Canvas =
        serde_json::from_str(&std::fs::read_to_string(dir.join("canvas.json")).expect("canvas.json"))
            .expect("parse canvas.json");
    let data_load_ms = td.elapsed().as_secs_f64() * 1000.0;

    // —— 核心摄取（权威正文 = mdcore；from_markdown 全量）——
    let ti = Instant::now();
    let core = mdcore::Core::from_markdown(&doc.text);
    let core_ingest_ms = ti.elapsed().as_secs_f64() * 1000.0;

    let lines: Vec<String> = doc.text.split('\n').map(str::to_string).collect();
    let node_pos: Vec<egui::Pos2> = scene
        .nodes
        .iter()
        .map(|n| egui::pos2(n.x, n.y))
        .collect();

    let xtest = Arc::new(XtestState {
        plan: Mutex::new(None),
        fired: AtomicU8::new(0),
        err: Mutex::new(String::new()),
        stop: AtomicBool::new(false),
    });
    let injector = xtest::spawn_injector(xtest.clone());

    let load = LoadMetrics { font_read_ms, font_set_ms: 0.0, data_load_ms, core_ingest_ms };
    let font_bytes2 = font_bytes.clone();

    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    let res = eframe::run_native(
        "proto-native-bench",
        opts,
        Box::new(move |cc| {
            // CJK 字体装载（真实启动成本：FontDefinitions 注入）
            let ts = Instant::now();
            if let Some(bytes) = &font_bytes2 {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "noto_sc".into(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes.clone())),
                );
                for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    fonts.families.get_mut(&fam).unwrap().push("noto_sc".into());
                }
                cc.egui_ctx.set_fonts(fonts);
            }
            // 基准口径确定性：强制 1.0 缩放（宿主检测方差消除）+ 滚动动画关闭
            cc.egui_ctx.set_pixels_per_point(1.0);
            // 滚动动画关闭（基准口径：程序化跳转 = 立即生效）
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                cc.egui_ctx.style_mut_of(theme, |s| {
                    s.scroll_animation = egui::style::ScrollAnimation::none()
                });
            }
            let font_set_ms = ts.elapsed().as_secs_f64() * 1000.0;

            let app = BenchApp {
                t0,
                load: LoadMetrics { font_set_ms, ..load },
                core,
                lines,
                node_pos,
                evidence,
                table,
                scene,
                tab: app::Tab::Body,
                hl_lines: Default::default(),
                jump_line: None,
                canvas_origin: egui::Pos2::ZERO,
                last_ppp: 1.0,
                canvas_scroll_to: None,
                drag_target_idx: None,
                edit_cell: None,
                table_scroll_cmd: None,
                queue: BenchApp::scenario_queue(),
                cur: None,
                t_trigger: None,
                metrics: Vec::new(),
                ready: false,
                last_frame: None,
                edit_check: None,
                drag_anchor: None,
                xtest,
                emitted: false,
                frames_drawn: 0,
            };
            Ok(Box::new(app))
        }),
    );
    // 注入线程收尾（进程退出前 join，避免悬挂）
    // app emit 时已置 xtest.stop；run_native 返回即窗口关闭
    if let Err(e) = res {
        eprintln!("eframe error: {e}");
        std::process::exit(1);
    }
    // xtest handle 在 app 里 drop 后线程自然退出（stop 已置）
    let _ = injector.join();
}
