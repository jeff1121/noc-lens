//! AI 報告（Report）資料存取。

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Report;

pub async fn insert(
    pool: &SqlitePool,
    title: &str,
    scope_json: &str,
    summary_md: &str,
    model_endpoint: Option<&str>,
) -> Result<Report, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO report (id, title, scope_json, summary_md, generated_at, model_endpoint) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(title)
    .bind(scope_json)
    .bind(summary_md)
    .bind(&now)
    .bind(model_endpoint)
    .execute(pool)
    .await?;
    get(pool, &id).await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Report, AppError> {
    let row = sqlx::query("SELECT * FROM report WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("報告 {id}")))?;
    map_report(&row)
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Report>, AppError> {
    let rows = sqlx::query("SELECT * FROM report ORDER BY generated_at DESC")
        .fetch_all(pool)
        .await?;
    rows.iter().map(map_report).collect()
}

fn map_report(row: &sqlx::sqlite::SqliteRow) -> Result<Report, AppError> {
    Ok(Report {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        scope_json: row.try_get("scope_json")?,
        summary_md: row.try_get("summary_md")?,
        generated_at: row.try_get("generated_at")?,
        model_endpoint: row.try_get("model_endpoint")?,
    })
}
