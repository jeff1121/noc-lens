//! US3 整合測試：歷史快照依時間排序。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::db::{self, device, snapshot};
use noc_lens_backend::models::{Brand, NewDevice};
use serde_json::json;
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    std::env::set_var("NOC_LENS_MASTER_KEY", STANDARD.encode([9u8; 32]));
    let path = std::env::temp_dir().join(format!("noc-lens-{}.db", uuid::Uuid::new_v4()));
    db::init_pool(path.to_str().unwrap()).await.unwrap()
}

#[tokio::test]
async fn snapshots_listed_newest_first() {
    let pool = setup().await;
    let id = device::create(
        &pool,
        NewDevice {
            ip_address: "10.8.0.1".to_string(),
            username: "admin".to_string(),
            password: "secret".to_string(),
            note: None,
            brand: Brand::Cisco,
        },
    )
    .await
    .unwrap()
    .id;

    for pct in [10.0, 20.0, 30.0] {
        snapshot::insert(
            &pool,
            &id,
            None,
            "ok",
            None,
            &json!({ "cpu": { "usage_percent": pct } }),
        )
        .await
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    let snaps = snapshot::list_by_device(&pool, &id, 10).await.unwrap();
    assert_eq!(snaps.len(), 3);
    // 最新（30）在最前
    assert_eq!(snaps[0].metrics["cpu"]["usage_percent"], 30.0);
    assert_eq!(snaps[2].metrics["cpu"]["usage_percent"], 10.0);
}
