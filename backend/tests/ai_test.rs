//! US4 整合測試：AI 報告生成（以 mock provider）。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::ai::{generate, AiProvider};
use noc_lens_backend::db::{self, device, report};
use noc_lens_backend::models::{Brand, NewDevice, ReportScope};
use noc_lens_backend::AppError;
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    std::env::set_var("NOC_LENS_MASTER_KEY", STANDARD.encode([9u8; 32]));
    let path = std::env::temp_dir().join(format!("noc-lens-{}.db", uuid::Uuid::new_v4()));
    db::init_pool(path.to_str().unwrap()).await.unwrap()
}

struct MockProvider;
impl AiProvider for MockProvider {
    async fn complete(&self, _system: &str, user: &str) -> Result<String, AppError> {
        // 確認 user prompt 含有設備資料
        let has_data = user.contains("device");
        if has_data {
            Ok("## 設備健康摘要\n\n整體：1 台正常。".to_string())
        } else {
            Ok("無資料".to_string())
        }
    }
}

struct FailProvider;
impl AiProvider for FailProvider {
    async fn complete(&self, _system: &str, _user: &str) -> Result<String, AppError> {
        Err(AppError::AiUnavailable("端點逾時".to_string()))
    }
}

#[tokio::test]
async fn generate_stores_report() {
    let pool = setup().await;
    device::create(
        &pool,
        NewDevice {
            ip_address: "10.9.0.1".to_string(),
            username: "admin".to_string(),
            password: "secret".to_string(),
            note: None,
            brand: Brand::Cisco,
        },
    )
    .await
    .unwrap();

    let rpt = generate(
        &pool,
        &MockProvider,
        ReportScope::default(),
        None,
        Some("mock"),
    )
    .await
    .unwrap();
    assert!(rpt.summary_md.contains("設備健康摘要"));
    assert_eq!(rpt.model_endpoint.as_deref(), Some("mock"));

    let all = report::list(&pool).await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn ai_unavailable_propagates() {
    let pool = setup().await;
    let err = generate(&pool, &FailProvider, ReportScope::default(), None, None)
        .await
        .unwrap_err();
    assert_eq!(err.code(), "AI_UNAVAILABLE");
}
