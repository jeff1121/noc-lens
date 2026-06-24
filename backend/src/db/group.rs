//! 群組／標籤（Group）資料存取，含設備多對多指派。

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Group;

/// 列出所有群組／標籤。
pub async fn list(pool: &SqlitePool) -> Result<Vec<Group>, AppError> {
    let rows = sqlx::query("SELECT * FROM device_group_tag ORDER BY name")
        .fetch_all(pool)
        .await?;
    rows.iter().map(map_group).collect()
}

/// 建立群組／標籤（名稱唯一）。
pub async fn create(pool: &SqlitePool, name: &str) -> Result<Group, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("群組名稱不可為空".to_string()));
    }
    if exists_name(pool, name).await? {
        return Err(AppError::DuplicateName(name.to_string()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO device_group_tag (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(&now)
        .execute(pool)
        .await?;

    Ok(Group {
        id,
        name: name.to_string(),
        created_at: now,
    })
}

/// 刪除群組／標籤。
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM device_group_tag WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("群組 {id}")));
    }
    Ok(())
}

/// 設定某設備所屬的群組（覆寫既有指派）。
pub async fn assign(
    pool: &SqlitePool,
    device_id: &str,
    group_ids: &[String],
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM device_group WHERE device_id = ?")
        .bind(device_id)
        .execute(&mut *tx)
        .await?;
    for gid in group_ids {
        sqlx::query("INSERT INTO device_group (device_id, group_id) VALUES (?, ?)")
            .bind(device_id)
            .bind(gid)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// 列出某設備所屬的群組。
pub async fn groups_for_device(pool: &SqlitePool, device_id: &str) -> Result<Vec<Group>, AppError> {
    let rows = sqlx::query(
        "SELECT g.* FROM device_group_tag g \
         JOIN device_group dg ON g.id = dg.group_id \
         WHERE dg.device_id = ? ORDER BY g.name",
    )
    .bind(device_id)
    .fetch_all(pool)
    .await?;
    rows.iter().map(map_group).collect()
}

async fn exists_name(pool: &SqlitePool, name: &str) -> Result<bool, AppError> {
    let row = sqlx::query("SELECT 1 AS hit FROM device_group_tag WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

/// 依名稱取得群組（匯入時用於對應既有群組）。
pub async fn find_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Group>, AppError> {
    let row = sqlx::query("SELECT * FROM device_group_tag WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    row.as_ref().map(map_group).transpose()
}

fn map_group(row: &sqlx::sqlite::SqliteRow) -> Result<Group, AppError> {
    Ok(Group {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        created_at: row.try_get("created_at")?,
    })
}
