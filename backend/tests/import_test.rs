//! US1 整合測試：CSV 匯入（成功／失敗逐列回報）。

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use noc_lens_backend::db::{self, device};
use noc_lens_backend::services::import;
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    std::env::set_var("NOC_LENS_MASTER_KEY", STANDARD.encode([9u8; 32]));
    let path = std::env::temp_dir().join(format!("noc-lens-{}.db", uuid::Uuid::new_v4()));
    db::init_pool(path.to_str().unwrap()).await.unwrap()
}

#[tokio::test]
async fn import_reports_success_and_failures() {
    let pool = setup().await;

    // 第 1 列正常；第 2 列品牌不支援；第 3 列與第 1 列 IP 重複。
    let csv = "ip_address,username,password,note,brand,groups\n\
        10.2.0.1,admin,pw1,核心交換器,cisco,高雄三民區;高雄高中\n\
        10.2.0.2,admin,pw2,防火牆,unknown_brand,\n\
        10.2.0.1,admin,pw3,重複,cisco,\n";

    let csv_path = std::env::temp_dir().join(format!("noc-import-{}.csv", uuid::Uuid::new_v4()));
    std::fs::write(&csv_path, csv).unwrap();

    let result = import::import_csv(&pool, csv_path.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(result.success, 1, "僅第 1 列應成功");
    assert_eq!(result.failed.len(), 2, "第 2、3 列應失敗");
    assert_eq!(result.failed[0].row, 2);
    assert_eq!(result.failed[1].row, 3);

    // 成功匯入的設備與其群組應存在。
    let devices = device::list(&pool, None).await.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].ip_address, "10.2.0.1");

    let groups = db::group::groups_for_device(&pool, &devices[0].id)
        .await
        .unwrap();
    assert_eq!(groups.len(), 2, "應自動建立並指派 2 個群組");
}
