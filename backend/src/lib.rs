//! noc-lens 核心函式庫。
//!
//! 提供設備管理、群組／標籤、加密儲存、SSH 查詢、排程與 AI 摘要等領域邏輯，
//! 供 Tauri 殼層（src-tauri）以 IPC 指令呼叫。

pub mod ai;
pub mod crypto;
pub mod db;
pub mod error;
pub mod models;
pub mod scheduler;
pub mod services;
pub mod ssh;

pub use error::AppError;
pub use sqlx::SqlitePool;
