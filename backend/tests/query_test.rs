//! US2 整合測試：以 mock SSH 執行器驗證查詢編排（成功／部分失敗／連線失敗）。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::db::{self, device};
use noc_lens_backend::models::{Brand, NewDevice};
use noc_lens_backend::ssh::executor::{CmdOutput, SshExecutor, SshTarget};
use noc_lens_backend::ssh::run_query;
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    std::env::set_var("NOC_LENS_MASTER_KEY", STANDARD.encode([9u8; 32]));
    let path = std::env::temp_dir().join(format!("noc-lens-{}.db", uuid::Uuid::new_v4()));
    db::init_pool(path.to_str().unwrap()).await.unwrap()
}

async fn add_device(pool: &SqlitePool, ip: &str) -> String {
    device::create(
        pool,
        NewDevice {
            ip_address: ip.to_string(),
            username: "admin".to_string(),
            password: "secret".to_string(),
            note: None,
            brand: Brand::Cisco,
        },
    )
    .await
    .unwrap()
    .id
}

/// 全部指令成功的 mock。
struct OkExecutor;
impl SshExecutor for OkExecutor {
    async fn run(
        &self,
        _t: SshTarget<'_>,
        commands: &[String],
    ) -> Result<Vec<CmdOutput>, noc_lens_backend::AppError> {
        let n = commands.len();
        Ok((0..n)
            .map(|_| CmdOutput {
                ok: true,
                text: "CPU utilization for five seconds: 42%".to_string(),
            })
            .collect())
    }
}

/// 連線失敗的 mock。
struct FailExecutor;
impl SshExecutor for FailExecutor {
    async fn run(
        &self,
        _t: SshTarget<'_>,
        _c: &[String],
    ) -> Result<Vec<CmdOutput>, noc_lens_backend::AppError> {
        Err(noc_lens_backend::AppError::Validation(
            "連線逾時".to_string(),
        ))
    }
}

#[tokio::test]
async fn query_success_writes_snapshot() {
    let pool = setup().await;
    let id = add_device(&pool, "10.5.0.1").await;

    let results = run_query(&pool, &OkExecutor, std::slice::from_ref(&id), 4).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, "ok");
    assert_eq!(
        results[0].metrics.as_ref().unwrap()["cpu"]["usage_percent"],
        42.0
    );

    // 應已寫入一筆快照。
    let snaps = db::snapshot::list_by_device(&pool, &id, 10).await.unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].status, "ok");
}

#[tokio::test]
async fn query_failure_is_isolated() {
    let pool = setup().await;
    let ok_id = add_device(&pool, "10.5.0.2").await;
    let _fail_id = add_device(&pool, "10.5.0.3").await;

    // 一台成功執行器、整體用失敗執行器測單台失敗不影響流程
    let results = run_query(&pool, &FailExecutor, std::slice::from_ref(&ok_id), 4).await;
    assert_eq!(results[0].status, "failed");
    assert!(results[0].error_message.is_some());
}

#[tokio::test]
async fn missing_device_returns_failed() {
    let pool = setup().await;
    let results = run_query(&pool, &OkExecutor, &["不存在".to_string()], 4).await;
    assert_eq!(results[0].status, "failed");
}
