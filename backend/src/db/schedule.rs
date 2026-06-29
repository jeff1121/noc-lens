//! 排程工作（ScheduledJob）與執行紀錄（JobRun）資料存取。

use chrono::{NaiveTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{JobRun, NewScheduledJob, ScheduledJob, UpdateScheduledJob};

// ---- ScheduledJob ----

pub async fn list(pool: &SqlitePool) -> Result<Vec<ScheduledJob>, AppError> {
    let rows = sqlx::query("SELECT * FROM scheduled_job ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    rows.iter().map(map_job).collect()
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<ScheduledJob, AppError> {
    let row = sqlx::query("SELECT * FROM scheduled_job WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("排程 {id}")))?;
    map_job(&row)
}

pub async fn create(pool: &SqlitePool, input: NewScheduledJob) -> Result<ScheduledJob, AppError> {
    let input = normalize(input)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO scheduled_job \
         (id, name, target_type, target_id, schedule_kind, interval_minutes, daily_time, enabled, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.target_type)
    .bind(&input.target_id)
    .bind(&input.schedule_kind)
    .bind(input.interval_minutes)
    .bind(&input.daily_time)
    .bind(&now)
    .execute(pool)
    .await?;
    get(pool, &id).await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    patch: UpdateScheduledJob,
) -> Result<ScheduledJob, AppError> {
    let current = get(pool, id).await?;
    let merged = normalize(NewScheduledJob {
        name: patch.name.unwrap_or(current.name),
        target_type: patch.target_type.unwrap_or(current.target_type),
        target_id: patch.target_id.unwrap_or(current.target_id),
        schedule_kind: patch.schedule_kind.unwrap_or(current.schedule_kind),
        interval_minutes: patch.interval_minutes.unwrap_or(current.interval_minutes),
        daily_time: patch.daily_time.unwrap_or(current.daily_time),
    })?;

    let r = sqlx::query(
        "UPDATE scheduled_job SET \
         name = ?, target_type = ?, target_id = ?, schedule_kind = ?, interval_minutes = ?, daily_time = ? \
         WHERE id = ?",
    )
    .bind(&merged.name)
    .bind(&merged.target_type)
    .bind(&merged.target_id)
    .bind(&merged.schedule_kind)
    .bind(merged.interval_minutes)
    .bind(&merged.daily_time)
    .bind(id)
    .execute(pool)
    .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("排程 {id}")));
    }
    get(pool, id).await
}

pub async fn set_enabled(
    pool: &SqlitePool,
    id: &str,
    enabled: bool,
) -> Result<ScheduledJob, AppError> {
    let r = sqlx::query("UPDATE scheduled_job SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(id)
        .execute(pool)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("排程 {id}")));
    }
    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let r = sqlx::query("DELETE FROM scheduled_job WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("排程 {id}")));
    }
    Ok(())
}

fn normalize(mut input: NewScheduledJob) -> Result<NewScheduledJob, AppError> {
    input.name = input.name.trim().to_string();
    input.target_type = input.target_type.trim().to_string();
    input.target_id = input.target_id.trim().to_string();
    input.schedule_kind = input.schedule_kind.trim().to_string();

    if input.name.is_empty() {
        return Err(AppError::Validation("排程名稱不可為空".to_string()));
    }
    if input.target_id.is_empty() {
        return Err(AppError::Validation("排程目標不可為空".to_string()));
    }
    match input.target_type.as_str() {
        "device" | "group" => {}
        _ => {
            return Err(AppError::Validation(
                "target_type 須為 device 或 group".to_string(),
            ))
        }
    }
    match input.schedule_kind.as_str() {
        "interval" => {
            if input.interval_minutes.unwrap_or(0) <= 0 {
                return Err(AppError::Validation(
                    "interval_minutes 須大於 0".to_string(),
                ));
            }
            input.daily_time = None;
        }
        "daily" => {
            let daily_time = input.daily_time.as_deref().unwrap_or("").trim();
            if daily_time.is_empty() {
                return Err(AppError::Validation("daily_time 須為 HH:mm".to_string()));
            }
            NaiveTime::parse_from_str(daily_time, "%H:%M")
                .map_err(|_| AppError::Validation("daily_time 須為有效 HH:mm".to_string()))?;
            input.daily_time = Some(daily_time.to_string());
            input.interval_minutes = None;
        }
        _ => {
            return Err(AppError::Validation(
                "schedule_kind 須為 interval 或 daily".to_string(),
            ))
        }
    }
    Ok(input)
}

fn map_job(row: &sqlx::sqlite::SqliteRow) -> Result<ScheduledJob, AppError> {
    let enabled: i64 = row.try_get("enabled")?;
    Ok(ScheduledJob {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        target_type: row.try_get("target_type")?,
        target_id: row.try_get("target_id")?,
        schedule_kind: row.try_get("schedule_kind")?,
        interval_minutes: row.try_get("interval_minutes")?,
        daily_time: row.try_get("daily_time")?,
        enabled: enabled != 0,
        created_at: row.try_get("created_at")?,
    })
}

// ---- JobRun ----

/// 建立一筆執行紀錄（開始），回傳 run id。
pub async fn run_start(pool: &SqlitePool, job_id: &str, total: i64) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO job_run (id, job_id, started_at, total, success_count, failure_count) \
         VALUES (?, ?, ?, ?, 0, 0)",
    )
    .bind(&id)
    .bind(job_id)
    .bind(&now)
    .bind(total)
    .execute(pool)
    .await?;
    Ok(id)
}

/// 完成執行紀錄並寫入成功／失敗數。
pub async fn run_finish(
    pool: &SqlitePool,
    run_id: &str,
    success: i64,
    failure: i64,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE job_run SET finished_at = ?, success_count = ?, failure_count = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(success)
    .bind(failure)
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn run_get(pool: &SqlitePool, run_id: &str) -> Result<JobRun, AppError> {
    let row = sqlx::query("SELECT * FROM job_run WHERE id = ?")
        .bind(run_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("執行紀錄 {run_id}")))?;
    map_run(&row)
}

pub async fn run_list(pool: &SqlitePool, job_id: &str) -> Result<Vec<JobRun>, AppError> {
    let rows = sqlx::query("SELECT * FROM job_run WHERE job_id = ? ORDER BY started_at DESC")
        .bind(job_id)
        .fetch_all(pool)
        .await?;
    rows.iter().map(map_run).collect()
}

fn map_run(row: &sqlx::sqlite::SqliteRow) -> Result<JobRun, AppError> {
    Ok(JobRun {
        id: row.try_get("id")?,
        job_id: row.try_get("job_id")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        total: row.try_get("total")?,
        success_count: row.try_get("success_count")?,
        failure_count: row.try_get("failure_count")?,
    })
}
