use std::io::Stdout;
use tui::backend::CrosstermBackend;

// 导出模块
pub mod api;
pub mod app;
pub mod events;
pub mod model;
pub mod storage;
pub mod widget;

// 重新导出常用类型，方便外部使用
pub use app::{App, AppState};
pub use model::Stock;

// 通用结果类型别名
pub type DynResult = Result<(), Box<dyn std::error::Error>>;
// Crossterm终端类型别名
pub type CrossTerminal = tui::Terminal<CrosstermBackend<Stdout>>;
// TUI帧类型别名
pub type TerminalFrame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>;
