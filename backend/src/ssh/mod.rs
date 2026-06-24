//! SSH 連線、品牌指令對應、輸出解析與查詢編排。

pub mod client;
pub mod commands;
pub mod executor;
pub mod parsers;
pub mod query;

pub use executor::{CmdOutput, SshExecutor, SshTarget};
pub use query::run_query;
