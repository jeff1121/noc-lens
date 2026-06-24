//! 設備（Device）資料存取。

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::crypto;
use crate::error::AppError;
use crate::models::{Brand, Device, NewDevice, UpdateDevice};

/// 列出設備；提供 `group_id` 時僅列出該群組內的設備。
pub async fn list(pool: &SqlitePool, group_id: Option<&str>) -> Result<Vec<Device>, AppError> {
    let rows = if let Some(gid) = group_id {
        sqlx::query(
            "SELECT d.* FROM device d \
             JOIN device_group dg ON d.id = dg.device_id \
             WHERE dg.group_id = ? ORDER BY d.ip_address",
        )
        .bind(gid)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query("SELECT * FROM device ORDER BY ip_address")
            .fetch_all(pool)
            .await?
    };

    rows.iter().map(map_device).collect()
}

/// 取得單一設備。
pub async fn get(pool: &SqlitePool, id: &str) -> Result<Device, AppError> {
    let row = sqlx::query("SELECT * FROM device WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("設備 {id}")))?;
    map_device(&row)
}

/// 建立設備（密碼加密後儲存）。
pub async fn create(pool: &SqlitePool, input: NewDevice) -> Result<Device, AppError> {
    validate_ip(&input.ip_address)?;
    if exists_ip(pool, &input.ip_address, None).await? {
        return Err(AppError::DuplicateIp(input.ip_address));
    }

    let enc = crypto::encrypt(&input.password)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO device (id, ip_address, username, password_enc, note, brand, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.ip_address)
    .bind(&input.username)
    .bind(&enc)
    .bind(&input.note)
    .bind(input.brand.as_str())
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

/// 更新設備（僅更新提供的欄位）。
pub async fn update(pool: &SqlitePool, id: &str, patch: UpdateDevice) -> Result<Device, AppError> {
    let existing = get(pool, id).await?;

    if let Some(ip) = &patch.ip_address {
        validate_ip(ip)?;
        if exists_ip(pool, ip, Some(id)).await? {
            return Err(AppError::DuplicateIp(ip.clone()));
        }
    }

    let ip = patch.ip_address.unwrap_or(existing.ip_address);
    let username = patch.username.unwrap_or(existing.username);
    let note = patch.note.or(existing.note);
    let brand = patch.brand.unwrap_or(existing.brand);
    let now = Utc::now().to_rfc3339();

    if let Some(password) = patch.password {
        let enc = crypto::encrypt(&password)?;
        sqlx::query(
            "UPDATE device SET ip_address = ?, username = ?, password_enc = ?, note = ?, \
             brand = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&ip)
        .bind(&username)
        .bind(&enc)
        .bind(&note)
        .bind(brand.as_str())
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE device SET ip_address = ?, username = ?, note = ?, brand = ?, \
             updated_at = ? WHERE id = ?",
        )
        .bind(&ip)
        .bind(&username)
        .bind(&note)
        .bind(brand.as_str())
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    }

    get(pool, id).await
}

/// 刪除設備。
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM device WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("設備 {id}")));
    }
    Ok(())
}

/// 取得設備的解密後密碼（僅供 SSH 查詢等後端內部使用，不對前端回傳）。
pub async fn get_password(pool: &SqlitePool, id: &str) -> Result<String, AppError> {
    let row = sqlx::query("SELECT password_enc FROM device WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("設備 {id}")))?;
    let enc: Vec<u8> = row.try_get("password_enc")?;
    crypto::decrypt(&enc)
}

/// 是否已存在相同 IP（`exclude_id` 用於更新時排除自身）。
pub async fn exists_ip(
    pool: &SqlitePool,
    ip: &str,
    exclude_id: Option<&str>,
) -> Result<bool, AppError> {
    let row = if let Some(eid) = exclude_id {
        sqlx::query("SELECT 1 AS hit FROM device WHERE ip_address = ? AND id <> ?")
            .bind(ip)
            .bind(eid)
            .fetch_optional(pool)
            .await?
    } else {
        sqlx::query("SELECT 1 AS hit FROM device WHERE ip_address = ?")
            .bind(ip)
            .fetch_optional(pool)
            .await?
    };
    Ok(row.is_some())
}

fn validate_ip(ip: &str) -> Result<(), AppError> {
    ip.parse::<std::net::IpAddr>()
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("不合法的 IP 位址：{ip}")))
}

fn map_device(row: &sqlx::sqlite::SqliteRow) -> Result<Device, AppError> {
    let brand_str: String = row.try_get("brand")?;
    let brand =
        Brand::parse(&brand_str).ok_or_else(|| AppError::UnsupportedBrand(brand_str.clone()))?;
    Ok(Device {
        id: row.try_get("id")?,
        ip_address: row.try_get("ip_address")?,
        username: row.try_get("username")?,
        note: row.try_get("note")?,
        brand,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
