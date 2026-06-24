//! AI 摘要與報告生成。
//!
//! 透過「OpenAI 相容端點」（FR-027）將設備狀態資料轉為可讀摘要。
//! `AiProvider` 抽象使報告彙整邏輯可用 mock 測試。

pub mod client;
pub mod report;

use std::future::Future;

use crate::error::AppError;

pub use client::OpenAiProvider;
pub use report::generate;

/// AI 供應者抽象：給定系統與使用者提示，回傳模型輸出。
pub trait AiProvider {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> impl Future<Output = Result<String, AppError>> + Send;
}
