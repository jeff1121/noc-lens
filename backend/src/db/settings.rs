//! 應用設定（AppSetting）資料存取。
//!
//! 敏感值（如 AI 金鑰）不存於此表，而存於 OS 金鑰庫。

use sqlx::{Row, SqlitePool};

use crate::error::AppError;

/// 取得設定值。
pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, AppError> {
    let row = sqlx::query("SELECT value FROM app_setting WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    match row {
        Some(r) => Ok(Some(r.try_get("value")?)),
        None => Ok(None),
    }
}

/// 設定值（upsert）。
pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app_setting (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
