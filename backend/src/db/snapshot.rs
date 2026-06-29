//! 狀態快照（StatusSnapshot）資料存取。

use chrono::{DateTime, Utc};
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
    list_by_device_filtered(pool, device_id, None, None, Some(limit)).await
}

/// 列出某設備指定期間的歷史快照（依採集時間新到舊）。
pub async fn list_by_device_filtered(
    pool: &SqlitePool,
    device_id: &str,
    from: Option<&str>,
    to: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<StatusSnapshot>, AppError> {
    let from = normalize_time(from)?;
    let to = normalize_time(to)?;
    if let (Some(from), Some(to)) = (&from, &to) {
        if from > to {
            return Err(AppError::Validation("from 不可晚於 to".to_string()));
        }
    }
    let limit = limit.map(|v| v.clamp(1, 500)).unwrap_or(-1);
    let rows = sqlx::query(
        "SELECT * FROM status_snapshot WHERE device_id = ? \
         AND (? IS NULL OR collected_at >= ?) \
         AND (? IS NULL OR collected_at <= ?) \
         ORDER BY collected_at DESC LIMIT ?",
    )
    .bind(device_id)
    .bind(from.as_deref())
    .bind(from.as_deref())
    .bind(to.as_deref())
    .bind(to.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.iter().map(map_snapshot).collect()
}

fn normalize_time(value: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let dt = DateTime::parse_from_rfc3339(value)
        .map_err(|_| AppError::Validation("時間格式須為 RFC3339".to_string()))?;
    Ok(Some(dt.with_timezone(&Utc).to_rfc3339()))
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
