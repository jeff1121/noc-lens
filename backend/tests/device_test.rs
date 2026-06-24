//! US1 整合測試：Device CRUD、重複 IP、不支援品牌。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::db::{self, device};
use noc_lens_backend::models::{Brand, NewDevice, UpdateDevice};
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    std::env::set_var("NOC_LENS_MASTER_KEY", STANDARD.encode([9u8; 32]));
    let path = std::env::temp_dir().join(format!("noc-lens-{}.db", uuid::Uuid::new_v4()));
    db::init_pool(path.to_str().unwrap()).await.unwrap()
}

fn new_device(ip: &str) -> NewDevice {
    NewDevice {
        ip_address: ip.to_string(),
        username: "admin".to_string(),
        password: "secret".to_string(),
        note: Some("測試設備".to_string()),
        brand: Brand::Cisco,
    }
}

#[tokio::test]
async fn create_and_get_device() {
    let pool = setup().await;
    let created = device::create(&pool, new_device("10.0.0.1")).await.unwrap();
    assert_eq!(created.ip_address, "10.0.0.1");
    assert_eq!(created.brand, Brand::Cisco);

    let fetched = device::get(&pool, &created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);

    // 密碼可正確解密回原文。
    let pw = device::get_password(&pool, &created.id).await.unwrap();
    assert_eq!(pw, "secret");
}

#[tokio::test]
async fn reject_duplicate_ip() {
    let pool = setup().await;
    device::create(&pool, new_device("10.0.0.2")).await.unwrap();
    let err = device::create(&pool, new_device("10.0.0.2"))
        .await
        .unwrap_err();
    assert_eq!(err.code(), "DUPLICATE_IP");
}

#[tokio::test]
async fn reject_invalid_ip() {
    let pool = setup().await;
    let err = device::create(&pool, new_device("not-an-ip"))
        .await
        .unwrap_err();
    assert_eq!(err.code(), "VALIDATION");
}

#[tokio::test]
async fn update_and_delete_device() {
    let pool = setup().await;
    let created = device::create(&pool, new_device("10.0.0.3")).await.unwrap();

    let patch = UpdateDevice {
        username: Some("operator".to_string()),
        brand: Some(Brand::Aruba),
        ..Default::default()
    };
    let updated = device::update(&pool, &created.id, patch).await.unwrap();
    assert_eq!(updated.username, "operator");
    assert_eq!(updated.brand, Brand::Aruba);

    device::delete(&pool, &created.id).await.unwrap();
    let err = device::get(&pool, &created.id).await.unwrap_err();
    assert_eq!(err.code(), "NOT_FOUND");
}
