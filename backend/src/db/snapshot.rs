//! 狀態快照（StatusSnapshot）資料存取。

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::StatusSnapshot;

/// 寫入一筆狀態快照，回傳其 id。
pub async fn insert(
    pool: &SqlitePool,
    device_id: &str,
    job_run_id: Option<&str>,
    status: &str,
    error_message: Option<&str>,
    metrics: &serde_json::Value,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let metrics_str = metrics.to_string();
    sqlx::query(
        "INSERT INTO status_snapshot \
         (id, device_id, job_run_id, collected_at, status, error_message, metrics_json) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(device_id)
    .bind(job_run_id)
    .bind(&now)
    .bind(status)
    .bind(error_message)
    .bind(&metrics_str)
    .execute(pool)
    .await?;
    Ok(id)
}

/// 列出某設備的歷史快照（依採集時間新到舊）。
pub async fn list_by_device(
    pool: &SqlitePool,
    device_id: &str,
    limit: i64,
) -> Result<Vec<StatusSnapshot>, AppError> {
    let rows = sqlx::query(
        "SELECT * FROM status_snapshot WHERE device_id = ? \
         ORDER BY collected_at DESC LIMIT ?",
    )
    .bind(device_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.iter().map(map_snapshot).collect()
}

fn map_snapshot(row: &sqlx::sqlite::SqliteRow) -> Result<StatusSnapshot, AppError> {
    let metrics_str: Option<String> = row.try_get("metrics_json")?;
    let metrics = metrics_str
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    Ok(StatusSnapshot {
        id: row.try_get("id")?,
        device_id: row.try_get("device_id")?,
        job_run_id: row.try_get("job_run_id")?,
        collected_at: row.try_get("collected_at")?,
        status: row.try_get("status")?,
        error_message: row.try_get("error_message")?,
        metrics,
    })
}
