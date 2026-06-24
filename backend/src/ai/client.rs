//! OpenAI 相容端點客戶端（reqwest）。

use std::future::Future;

use reqwest::Client;
use serde_json::json;

use crate::ai::AiProvider;
use crate::error::AppError;

/// 以 OpenAI 相容 Chat Completions API 產生內容。
pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    model: String,
    http: Client,
}

impl OpenAiProvider {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            http: Client::new(),
        }
    }
}

impl AiProvider for OpenAiProvider {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> impl Future<Output = Result<String, AppError>> + Send {
        let url = format!("{}/chat/completions", self.base_url);
        let key = self.api_key.clone();
        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ],
            "temperature": 0.3
        });
        let http = self.http.clone();
        async move {
            let resp = http
                .post(&url)
                .bearer_auth(&key)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::AiUnavailable(e.to_string()))?;
            if !resp.status().is_success() {
                return Err(AppError::AiUnavailable(format!(
                    "AI 端點回應狀態 {}",
                    resp.status()
                )));
            }
            let val: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::AiUnavailable(e.to_string()))?;
            val["choices"][0]["message"]["content"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::AiUnavailable("AI 回應格式不正確".to_string()))
        }
    }
}
