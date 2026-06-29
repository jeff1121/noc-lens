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

#[tokio::test]
async fn snapshots_can_be_filtered_by_time_range() {
    let pool = setup().await;
    let id = device::create(
        &pool,
        NewDevice {
            ip_address: "10.8.0.2".to_string(),
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
    let middle_time = snaps[1].collected_at.as_str();
    let from_middle = snapshot::list_by_device_filtered(&pool, &id, Some(middle_time), None, None)
        .await
        .unwrap();
    assert_eq!(from_middle.len(), 2);
    assert_eq!(from_middle[0].metrics["cpu"]["usage_percent"], 30.0);
    assert_eq!(from_middle[1].metrics["cpu"]["usage_percent"], 20.0);

    let to_middle = snapshot::list_by_device_filtered(&pool, &id, None, Some(middle_time), None)
        .await
        .unwrap();
    assert_eq!(to_middle.len(), 2);
    assert_eq!(to_middle[0].metrics["cpu"]["usage_percent"], 20.0);
    assert_eq!(to_middle[1].metrics["cpu"]["usage_percent"], 10.0);

    let err = snapshot::list_by_device_filtered(
        &pool,
        &id,
        Some(&snaps[0].collected_at),
        Some(&snaps[2].collected_at),
        None,
    )
    .await
    .unwrap_err();
    assert_eq!(err.code(), "VALIDATION");
}
