//! Tauri IPC 指令（前後端契約進入點）。
//!
//! 對應 contracts/tauri-commands.md。所有指令回傳 `Result<T, AppError>`，
//! `AppError` 會序列化為 `{ code, message }`。

use noc_lens_backend::db::{device, group, settings, snapshot};
use noc_lens_backend::models::{Device, Group, NewDevice, QueryResult, StatusSnapshot, UpdateDevice};
use noc_lens_backend::services::import::{self, ImportResult};
use noc_lens_backend::ssh::client::RusshExecutor;
use noc_lens_backend::ssh::run_query;
use noc_lens_backend::AppError;
use serde::Serialize;
use tauri::State;

use crate::AppState;

// ---- 設備（Device）----

#[tauri::command]
pub async fn device_list(
    state: State<'_, AppState>,
    group_id: Option<String>,
) -> Result<Vec<Device>, AppError> {
    device::list(&state.pool, group_id.as_deref()).await
}

#[tauri::command]
pub async fn device_create(
    state: State<'_, AppState>,
    input: NewDevice,
) -> Result<Device, AppError> {
    device::create(&state.pool, input).await
}

#[tauri::command]
pub async fn device_update(
    state: State<'_, AppState>,
    id: String,
    patch: UpdateDevice,
) -> Result<Device, AppError> {
    device::update(&state.pool, &id, patch).await
}

#[tauri::command]
pub async fn device_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    device::delete(&state.pool, &id).await
}

#[tauri::command]
pub async fn device_import(
    state: State<'_, AppState>,
    content: String,
) -> Result<ImportResult, AppError> {
    import::import_csv_str(&state.pool, &content).await
}

// ---- 群組／標籤（Group）----

#[tauri::command]
pub async fn group_list(state: State<'_, AppState>) -> Result<Vec<Group>, AppError> {
    group::list(&state.pool).await
}

#[tauri::command]
pub async fn group_create(state: State<'_, AppState>, name: String) -> Result<Group, AppError> {
    group::create(&state.pool, &name).await
}

#[tauri::command]
pub async fn group_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    group::delete(&state.pool, &id).await
}

#[tauri::command]
pub async fn group_assign(
    state: State<'_, AppState>,
    device_id: String,
    group_ids: Vec<String>,
) -> Result<(), AppError> {
    group::assign(&state.pool, &device_id, &group_ids).await
}

#[tauri::command]
pub async fn groups_for_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<Vec<Group>, AppError> {
    group::groups_for_device(&state.pool, &device_id).await
}

// ---- 即時查詢（SSH）----

#[tauri::command]
pub async fn query_devices(
    state: State<'_, AppState>,
    device_ids: Vec<String>,
) -> Result<Vec<QueryResult>, AppError> {
    let conc = settings::get(&state.pool, "ssh.max_concurrency")
        .await?
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);
    Ok(run_query(&state.pool, &RusshExecutor, &device_ids, conc).await)
}

#[tauri::command]
pub async fn snapshot_list(
    state: State<'_, AppState>,
    device_id: String,
    limit: Option<i64>,
) -> Result<Vec<StatusSnapshot>, AppError> {
    snapshot::list_by_device(&state.pool, &device_id, limit.unwrap_or(50)).await
}

// ---- 設定（Settings）----

/// 設定檢視（不含敏感金鑰本身，僅回報是否已設定）。
#[derive(Serialize)]
pub struct SettingsDto {
    pub ai_base_url: String,
    pub ai_model: String,
    pub ssh_max_concurrency: u32,
    pub ai_key_set: bool,
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<SettingsDto, AppError> {
    let pool = &state.pool;
    let ai_base_url = settings::get(pool, "ai.base_url").await?.unwrap_or_default();
    let ai_model = settings::get(pool, "ai.model").await?.unwrap_or_default();
    let ssh_max_concurrency = settings::get(pool, "ssh.max_concurrency")
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(10);
    Ok(SettingsDto {
        ai_base_url,
        ai_model,
        ssh_max_concurrency,
        ai_key_set: false,
    })
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, AppState>,
    ai_base_url: Option<String>,
    ai_model: Option<String>,
    ssh_max_concurrency: Option<u32>,
) -> Result<(), AppError> {
    let pool = &state.pool;
    if let Some(v) = ai_base_url {
        settings::set(pool, "ai.base_url", &v).await?;
    }
    if let Some(v) = ai_model {
        settings::set(pool, "ai.model", &v).await?;
    }
    if let Some(v) = ssh_max_concurrency {
        settings::set(pool, "ssh.max_concurrency", &v.to_string()).await?;
    }
    Ok(())
}
