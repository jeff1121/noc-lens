//! 本地 SQLite 資料存取層。

pub mod device;
pub mod group;
pub mod schedule;
pub mod settings;
pub mod snapshot;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::AppError;

/// 初始化資料庫連線池並套用 migrations。
///
/// `db_path` 為 SQLite 檔案路徑（不存在時自動建立）。
pub async fn init_pool(db_path: &str) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
