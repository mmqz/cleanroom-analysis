//! XTEST 合成输入注入线程（拖拽场景）。
//!
//! 为什么不用 egui input_mut 注入：egui 的 PointerState 在 begin_pass
//! 由原始事件推导，pass 中途注入不更新指针状态（拖拽检测不触发）。
//! XTEST 走 X 服务器 → winit → egui 全链 = 与 web 候选 CDP 真实输入
//! 完全对称的测量面。
//!
//! 线程模型：主线程在场景触发时写入 DragPlan；本线程按墙上时钟执行
//! press → 60×motion(16ms) → release；成功置 fired=1；失败记 err。
//! stop 标志随时可终止。

use crate::app::XtestState;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub struct DragPlan {
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub steps: u32,
    pub interval_ms: u64,
}

const BUTTON_PRESS: u8 = 4;
const BUTTON_RELEASE: u8 = 5;
const MOTION_NOTIFY: u8 = 6;
const BUTTON_LEFT: u8 = 1;

pub fn spawn_injector(state: std::sync::Arc<XtestState>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // 等待计划（最多 30s——整个基准运行的上界）
        let plan = loop {
            if state.stop.load(Ordering::SeqCst) {
                return;
            }
            if let Some(p) = state.plan.lock().unwrap().take() {
                break p;
            }
            thread::sleep(Duration::from_millis(2));
        };
        let conn = match x11rb::connect(None) {
            Ok((c, _)) => c,
            Err(e) => {
                *state.err.lock().unwrap() = format!("x11rb connect: {e}");
                return;
            }
        };
        let r = run_plan(&conn, &plan);
        match r {
            Ok(()) => state.fired.store(1, Ordering::SeqCst),
            Err(e) => *state.err.lock().unwrap() = e.to_string(),
        }
    })
}

fn run_plan<C: x11rb::connection::Connection + ?Sized>(
    conn: &C,
    plan: &DragPlan,
) -> Result<(), Box<dyn std::error::Error>> {
    use x11rb::protocol::xtest::fake_input;
    // 按下
    fake_input(
        conn,
        BUTTON_PRESS,
        BUTTON_LEFT,
        0,
        0,
        plan.start.0 as i16,
        plan.start.1 as i16,
        0,
    )?;
    conn.flush()?;
    thread::sleep(Duration::from_millis(40));
    // 60 步线性移动
    for k in 1..=plan.steps {
        let f = k as f32 / plan.steps as f32;
        let x = plan.start.0 + (plan.end.0 - plan.start.0) * f;
        let y = plan.start.1 + (plan.end.1 - plan.start.1) * f;
        fake_input(conn, MOTION_NOTIFY, 0, 0, 0, x as i16, y as i16, 0)?;
        conn.flush()?;
        thread::sleep(Duration::from_millis(plan.interval_ms));
    }
    // 释放（终点）
    fake_input(
        conn,
        BUTTON_RELEASE,
        BUTTON_LEFT,
        0,
        0,
        plan.end.0 as i16,
        plan.end.1 as i16,
        0,
    )?;
    conn.flush()?;
    thread::sleep(Duration::from_millis(20));
    Ok(())
}
