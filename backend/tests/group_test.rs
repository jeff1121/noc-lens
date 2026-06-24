//! US1 整合測試：Group CRUD、多對多指派、依群組篩選。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::db::{self, device, group};
use noc_lens_backend::models::{Brand, NewDevice};
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
        note: None,
        brand: Brand::Cisco,
    }
}

#[tokio::test]
async fn create_group_and_reject_duplicate() {
    let pool = setup().await;
    let g = group::create(&pool, "高雄三民區").await.unwrap();
    assert_eq!(g.name, "高雄三民區");

    let err = group::create(&pool, "高雄三民區").await.unwrap_err();
    assert_eq!(err.code(), "DUPLICATE_NAME");
}

#[tokio::test]
async fn assign_and_filter_by_group() {
    let pool = setup().await;
    let d1 = device::create(&pool, new_device("10.1.0.1")).await.unwrap();
    let d2 = device::create(&pool, new_device("10.1.0.2")).await.unwrap();
    let g = group::create(&pool, "高雄高中").await.unwrap();

    group::assign(&pool, &d1.id, &[g.id.clone()]).await.unwrap();

    // 依群組篩選只應回傳 d1。
    let in_group = device::list(&pool, Some(&g.id)).await.unwrap();
    assert_eq!(in_group.len(), 1);
    assert_eq!(in_group[0].id, d1.id);

    // d1 的群組清單應包含該群組。
    let groups = group::groups_for_device(&pool, &d1.id).await.unwrap();
    assert_eq!(groups.len(), 1);

    // d2 未指派，全部清單應有 2 台。
    let all = device::list(&pool, None).await.unwrap();
    assert_eq!(all.len(), 2);
    let _ = d2;
}

#[tokio::test]
async fn delete_group_removes_assignment() {
    let pool = setup().await;
    let d = device::create(&pool, new_device("10.1.0.3")).await.unwrap();
    let g = group::create(&pool, "臨時群組").await.unwrap();
    group::assign(&pool, &d.id, &[g.id.clone()]).await.unwrap();

    group::delete(&pool, &g.id).await.unwrap();
    let groups = group::groups_for_device(&pool, &d.id).await.unwrap();
    assert!(groups.is_empty(), "刪除群組後關聯應一併移除");
}
