//! SSH 執行器抽象。
//!
//! 將「實際 SSH 連線」抽象為 `SshExecutor` trait，使查詢編排與解析邏輯可用
//! mock 進行單元測試，真實連線則由 russh 實作（client.rs）。

use std::future::Future;

use crate::error::AppError;

/// SSH 連線目標。
pub struct SshTarget<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: &'a str,
    pub password: &'a str,
}

/// 單一指令的輸出結果。
#[derive(Debug, Clone)]
pub struct CmdOutput {
    /// 指令是否成功執行。
    pub ok: bool,
    /// 標準輸出（成功）或錯誤訊息（失敗）。
    pub text: String,
}

/// SSH 執行器：以單一連線執行多道唯讀指令。
///
/// - 連線／認證失敗 → 回傳 `Err`（整台設備視為失敗）。
/// - 個別指令失敗 → 於對應 `CmdOutput.ok = false` 標示，不影響其他指令。
pub trait SshExecutor {
    fn run(
        &self,
        target: SshTarget<'_>,
        commands: &[String],
    ) -> impl Future<Output = Result<Vec<CmdOutput>, AppError>> + Send;
}
