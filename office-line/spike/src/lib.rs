//! office_impl_spike 库面——xlsx-r1 阶段 B 预置产品代码（分析仓验证态）。
//!
//! 定位：本文件群 = mdcore 落地（阶段 B）的**直移源**，在分析仓内以
//! `cargo test` 完成编译与单测验证，解锁后按 xlsx-r1/REPORT.md §3
//! runbook 接线。两个模块对应阶段 B 的**双 crate 落点**（架构决策
//! 见 REPORT.md §2）：
//!
//! - [`io`]——XLSX 读路径（容器预扫门 + calamine 包装 + 统一错误面）。
//!   落点 = **新 crate mdc-office-io**（mdcore workspace 第八成员）：
//!   T1 已在 mdc-office 落地零三方依赖自锁测试（清单级
//!   [dependencies] 缺席断言），IO 的 calamine/rust_xlsxwriter/zip
//!   依赖不能进 mdc-office——新 crate 保自锁不变量，循 mdc-pdf/
//!   mdc-settings 一事一 crate 先例。
//! - [`url_policy`]——URL scheme 白名单（消毒第六面，t10-e §6 缺口
//!   补面）。落点 = **现有 mdc-office sanitize 家族**（纯 std 零依赖，
//!   与五面同构）。
//!
//! 依赖边界：io 模块依赖 calamine + zip（阶段 B 声明于 mdc-office-io，
//!   版本按 xlsx-r1/DEPS.md 锁定）；url_policy 纯 std。两模块互不依赖。

pub mod io;
pub mod url_policy;
