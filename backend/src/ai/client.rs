//! OpenAI 相容端點客戶端（reqwest）。

use std::future::Future;
use std::net::IpAddr;
use std::time::Duration;

use reqwest::{redirect::Policy, Client, Url};
use serde_json::json;

use crate::ai::AiProvider;
use crate::error::AppError;

/// 以 OpenAI 相容 Chat Completions API 產生內容。
pub struct OpenAiProvider {
    chat_url: Url,
    api_key: String,
    model: String,
    http: Client,
}

impl OpenAiProvider {
    pub fn new(base_url: String, api_key: String, model: String) -> Result<Self, AppError> {
        let chat_url = chat_completions_url(&base_url)?;
        let http = Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::AiUnavailable(e.to_string()))?;

        Ok(Self {
            chat_url,
            api_key,
            model,
            http,
        })
    }

    pub fn validate_base_url(base_url: &str) -> Result<String, AppError> {
        let url = base_url_url(base_url)?;
        Ok(url.as_str().trim_end_matches('/').to_string())
    }
}

impl AiProvider for OpenAiProvider {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> impl Future<Output = Result<String, AppError>> + Send {
        let url = self.chat_url.clone();
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
                .post(url)
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

fn chat_completions_url(base_url: &str) -> Result<Url, AppError> {
    let mut url = base_url_url(base_url)?;
    url.path_segments_mut()
        .map_err(|_| AppError::Validation("AI Base URL 必須可作為路徑基底".to_string()))?
        .extend(["chat", "completions"]);
    Ok(url)
}

fn base_url_url(base_url: &str) -> Result<Url, AppError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AppError::Validation("AI Base URL 不可為空".to_string()));
    }

    let url = Url::parse(trimmed)
        .map_err(|e| AppError::Validation(format!("AI Base URL 格式不正確：{e}")))?;
    if url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(AppError::Validation(
            "AI Base URL 不可包含帳密、查詢字串或 fragment".to_string(),
        ));
    }

    match url.scheme() {
        "https" => {
            if is_blocked_ip_literal(&url) {
                return Err(AppError::Validation(
                    "AI Base URL 不可使用非 loopback 的私有、link-local 或特殊 IP".to_string(),
                ));
            }
        }
        "http" if is_loopback_host(&url) => {}
        _ => {
            return Err(AppError::Validation(
                "AI Base URL 僅允許 https；本機模型端點可使用 http://localhost 或 http://127.0.0.1"
                    .to_string(),
            ));
        }
    }

    Ok(url)
}

fn is_loopback_host(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let normalized = host.to_ascii_lowercase();
    normalized == "localhost"
        || normalized.ends_with(".localhost")
        || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn is_blocked_ip_literal(url: &Url) -> bool {
    let Some(ip) = url.host_str().and_then(|host| host.parse::<IpAddr>().ok()) else {
        return false;
    };

    match ip {
        IpAddr::V4(ip) => {
            !ip.is_loopback()
                && (ip.is_private()
                    || ip.is_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast())
        }
        IpAddr::V6(ip) => {
            !ip.is_loopback()
                && (ip.is_unique_local()
                    || ip.is_unicast_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_remote_endpoint() {
        let url = chat_completions_url("https://api.openai.com/v1").unwrap();
        assert_eq!(url.as_str(), "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn accepts_loopback_http_endpoint() {
        let url = chat_completions_url("http://localhost:11434/v1").unwrap();
        assert_eq!(url.as_str(), "http://localhost:11434/v1/chat/completions");
    }

    #[test]
    fn rejects_plain_http_remote_endpoint() {
        let err = chat_completions_url("http://example.com/v1").unwrap_err();
        assert_eq!(err.code(), "VALIDATION");
    }

    #[test]
    fn rejects_private_ip_literal_endpoint() {
        let err = chat_completions_url("https://192.168.1.10/v1").unwrap_err();
        assert_eq!(err.code(), "VALIDATION");
    }

    #[test]
    fn rejects_query_or_fragment_endpoint() {
        let err = chat_completions_url("https://api.openai.com/v1?x=1").unwrap_err();
        assert_eq!(err.code(), "VALIDATION");
    }
}
