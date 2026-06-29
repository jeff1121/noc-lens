//! Tauri IPC 指令（前後端契約進入點）。
//!
//! 對應 contracts/tauri-commands.md。所有指令回傳 `Result<T, AppError>`，
//! `AppError` 會序列化為 `{ code, message }`。

use std::{fs, path::Path};

use noc_lens_backend::ai::{self, OpenAiProvider};
use noc_lens_backend::crypto;
use noc_lens_backend::db::{device, group, report, schedule, settings, snapshot};
use noc_lens_backend::models::{
    Device, Group, JobRun, NewDevice, NewScheduledJob, QueryResult, Report, ReportScope,
    ScheduledJob, StatusSnapshot, UpdateDevice, UpdateScheduledJob,
};
use noc_lens_backend::scheduler::run_job_once;
use noc_lens_backend::services::import::{self, ImportResult};
use noc_lens_backend::ssh::client::RusshExecutor;
use noc_lens_backend::ssh::{run_query, DEFAULT_QUERY_CONCURRENCY, MAX_QUERY_CONCURRENCY};
use noc_lens_backend::AppError;
use serde::Serialize;
use tauri::State;

use crate::AppState;

// ---- 設備（Device）----

#[tauri::command(rename_all = "snake_case")]
pub async fn device_list(
    state: State<'_, AppState>,
    group_id: Option<String>,
) -> Result<Vec<Device>, AppError> {
    device::list(&state.pool, group_id.as_deref()).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn device_create(
    state: State<'_, AppState>,
    input: NewDevice,
) -> Result<Device, AppError> {
    device::create(&state.pool, input).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn device_update(
    state: State<'_, AppState>,
    id: String,
    patch: UpdateDevice,
) -> Result<Device, AppError> {
    device::update(&state.pool, &id, patch).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn device_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    device::delete(&state.pool, &id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn device_import(
    state: State<'_, AppState>,
    content: String,
) -> Result<ImportResult, AppError> {
    import::import_csv_str(&state.pool, &content).await
}

// ---- 群組／標籤（Group）----

#[tauri::command(rename_all = "snake_case")]
pub async fn group_list(state: State<'_, AppState>) -> Result<Vec<Group>, AppError> {
    group::list(&state.pool).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn group_create(state: State<'_, AppState>, name: String) -> Result<Group, AppError> {
    group::create(&state.pool, &name).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn group_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    group::delete(&state.pool, &id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn group_assign(
    state: State<'_, AppState>,
    device_id: String,
    group_ids: Vec<String>,
) -> Result<(), AppError> {
    group::assign(&state.pool, &device_id, &group_ids).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn groups_for_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<Vec<Group>, AppError> {
    group::groups_for_device(&state.pool, &device_id).await
}

// ---- 即時查詢（SSH）----

#[tauri::command(rename_all = "snake_case")]
pub async fn query_devices(
    state: State<'_, AppState>,
    device_ids: Vec<String>,
) -> Result<Vec<QueryResult>, AppError> {
    let conc = settings::get(&state.pool, "ssh.max_concurrency")
        .await?
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_QUERY_CONCURRENCY);
    Ok(run_query(&state.pool, &RusshExecutor, &device_ids, conc).await)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn snapshot_list(
    state: State<'_, AppState>,
    device_id: String,
    from: Option<String>,
    to: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<StatusSnapshot>, AppError> {
    device::get(&state.pool, &device_id).await?;
    let resolved_limit = if from.is_none() && to.is_none() {
        limit.or(Some(50))
    } else {
        limit
    };
    snapshot::list_by_device_filtered(
        &state.pool,
        &device_id,
        from.as_deref(),
        to.as_deref(),
        resolved_limit,
    )
    .await
}

// ---- 排程（Schedule）----

#[tauri::command(rename_all = "snake_case")]
pub async fn schedule_list(state: State<'_, AppState>) -> Result<Vec<ScheduledJob>, AppError> {
    schedule::list(&state.pool).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn schedule_create(
    state: State<'_, AppState>,
    input: NewScheduledJob,
) -> Result<ScheduledJob, AppError> {
    let job = schedule::create(&state.pool, input).await?;
    state.scheduler.reload().await?;
    Ok(job)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn schedule_update(
    state: State<'_, AppState>,
    id: String,
    patch: UpdateScheduledJob,
) -> Result<ScheduledJob, AppError> {
    let job = schedule::update(&state.pool, &id, patch).await?;
    state.scheduler.reload().await?;
    Ok(job)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn schedule_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    schedule::delete(&state.pool, &id).await?;
    state.scheduler.reload().await?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn schedule_toggle(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<ScheduledJob, AppError> {
    let job = schedule::set_enabled(&state.pool, &id, enabled).await?;
    state.scheduler.reload().await?;
    Ok(job)
}

/// 立即觸發一次排程執行（供測試與手動巡檢）。
#[tauri::command(rename_all = "snake_case")]
pub async fn schedule_run_now(state: State<'_, AppState>, id: String) -> Result<JobRun, AppError> {
    run_job_once(&state.pool, &RusshExecutor, &id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn job_run_list(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Vec<JobRun>, AppError> {
    schedule::run_list(&state.pool, &job_id).await
}

// ---- AI 報告（Report）----

#[tauri::command(rename_all = "snake_case")]
pub async fn report_generate(
    state: State<'_, AppState>,
    scope: ReportScope,
    title: Option<String>,
) -> Result<Report, AppError> {
    let pool = &state.pool;
    let base_url = settings::get(pool, "ai.base_url")
        .await?
        .unwrap_or_default();
    let model = settings::get(pool, "ai.model").await?.unwrap_or_default();
    let Some(key) = crypto::get_ai_key()? else {
        return Err(AppError::AiConfigMissing(
            "請先於設定填入 AI 端點與金鑰".to_string(),
        ));
    };
    if base_url.trim().is_empty() {
        return Err(AppError::AiConfigMissing(
            "請先於設定填入 AI 端點與金鑰".to_string(),
        ));
    }
    let model = if model.trim().is_empty() {
        "gpt-4o-mini".to_string()
    } else {
        model
    };
    let normalized_base_url = OpenAiProvider::validate_base_url(&base_url)?;
    let provider = OpenAiProvider::new(normalized_base_url.clone(), key, model)?;
    ai::generate(pool, &provider, scope, title, Some(&normalized_base_url)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn report_list(state: State<'_, AppState>) -> Result<Vec<Report>, AppError> {
    report::list(&state.pool).await
}

#[derive(Serialize)]
pub struct ReportExportResult {
    pub path: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn report_export(
    state: State<'_, AppState>,
    id: String,
    out_path: String,
    format: String,
) -> Result<ReportExportResult, AppError> {
    let trimmed_path = out_path.trim();
    if trimmed_path.is_empty() {
        return Err(AppError::Validation("匯出路徑不可為空".to_string()));
    }

    let report = report::get(&state.pool, &id).await?;
    match format.trim().to_ascii_lowercase().as_str() {
        "md" => write_report_markdown(Path::new(trimmed_path), &report)?,
        "pdf" => write_report_pdf(Path::new(trimmed_path), &report)?,
        _ => return Err(AppError::Validation("format 須為 md 或 pdf".to_string())),
    }

    Ok(ReportExportResult {
        path: trimmed_path.to_string(),
    })
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

#[tauri::command(rename_all = "snake_case")]
pub async fn settings_get(state: State<'_, AppState>) -> Result<SettingsDto, AppError> {
    let pool = &state.pool;
    let ai_base_url = settings::get(pool, "ai.base_url")
        .await?
        .unwrap_or_default();
    let ai_model = settings::get(pool, "ai.model").await?.unwrap_or_default();
    let ssh_max_concurrency = settings::get(pool, "ssh.max_concurrency")
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .map(|v| v.min(MAX_QUERY_CONCURRENCY as u32).max(1))
        .unwrap_or(DEFAULT_QUERY_CONCURRENCY as u32);
    let ai_key_set = crypto::get_ai_key().map(|k| k.is_some()).unwrap_or(false);
    Ok(SettingsDto {
        ai_base_url,
        ai_model,
        ssh_max_concurrency,
        ai_key_set,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn settings_set(
    state: State<'_, AppState>,
    ai_base_url: Option<String>,
    ai_model: Option<String>,
    ssh_max_concurrency: Option<u32>,
) -> Result<(), AppError> {
    let pool = &state.pool;
    if let Some(v) = ai_base_url {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            settings::set(pool, "ai.base_url", "").await?;
        } else {
            let normalized = OpenAiProvider::validate_base_url(trimmed)?;
            settings::set(pool, "ai.base_url", &normalized).await?;
        }
    }
    if let Some(v) = ai_model {
        settings::set(pool, "ai.model", &v).await?;
    }
    if let Some(v) = ssh_max_concurrency {
        validate_ssh_concurrency(v)?;
        settings::set(pool, "ssh.max_concurrency", &v.to_string()).await?;
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn settings_set_ai_key(
    _state: State<'_, AppState>,
    api_key: String,
) -> Result<(), AppError> {
    crypto::set_ai_key(&api_key)
}

fn validate_ssh_concurrency(value: u32) -> Result<(), AppError> {
    if value == 0 || value > MAX_QUERY_CONCURRENCY as u32 {
        return Err(AppError::Validation(format!(
            "SSH 同時連線上限需介於 1 到 {}",
            MAX_QUERY_CONCURRENCY
        )));
    }
    Ok(())
}

fn write_report_markdown(path: &Path, report: &Report) -> Result<(), AppError> {
    let mut content = String::new();
    content.push_str("# ");
    content.push_str(&report.title);
    content.push_str("\n\n");
    content.push_str("- 產生時間：");
    content.push_str(&report.generated_at);
    content.push('\n');
    if let Some(endpoint) = &report.model_endpoint {
        content.push_str("- 模型端點：");
        content.push_str(endpoint);
        content.push('\n');
    }
    content.push_str("\n---\n\n");
    content.push_str(&report.summary_md);
    fs::write(path, content)?;
    Ok(())
}

fn write_report_pdf(path: &Path, report: &Report) -> Result<(), AppError> {
    let mut lines = Vec::new();
    lines.push(report.title.clone());
    lines.push(format!("產生時間：{}", report.generated_at));
    if let Some(endpoint) = &report.model_endpoint {
        lines.push(format!("模型端點：{endpoint}"));
    }
    lines.push(String::new());
    for line in report.summary_md.lines() {
        lines.extend(wrap_pdf_line(line, 72));
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    let pages = lines
        .chunks(45)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();
    let font_obj = 3 + pages.len() * 2;
    let mut objects = Vec::new();
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    let kids = (0..pages.len())
        .map(|i| format!("{} 0 R", 3 + i * 2))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!(
        "<< /Type /Pages /Kids [{kids}] /Count {} >>",
        pages.len()
    ));

    for (index, page_lines) in pages.iter().enumerate() {
        let page_obj = 3 + index * 2;
        let content_obj = page_obj + 1;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] \
             /Resources << /Font << /F1 {font_obj} 0 R >> >> /Contents {content_obj} 0 R >>"
        ));
        let content = pdf_page_stream(page_lines);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.len(),
            content
        ));
    }

    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());

    let mut bytes = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (idx, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", idx + 1, object).as_bytes());
    }
    let xref_offset = bytes.len();
    bytes.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    bytes.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    fs::write(path, bytes)?;
    Ok(())
}

fn pdf_page_stream(lines: &[String]) -> String {
    let mut stream = String::from("BT\n/F1 11 Tf\n50 800 Td\n15 TL\n");
    for line in lines {
        stream.push_str(&pdf_utf16_hex(line));
        stream.push_str(" Tj\nT*\n");
    }
    stream.push_str("ET\n");
    stream
}

fn pdf_utf16_hex(value: &str) -> String {
    let mut bytes = vec![0xFE, 0xFF];
    for unit in value.encode_utf16() {
        bytes.push((unit >> 8) as u8);
        bytes.push((unit & 0xFF) as u8);
    }
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    format!("<{hex}>")
}

fn wrap_pdf_line(line: &str, width: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let mut wrapped = Vec::new();
    let mut current = String::new();
    for ch in line.chars() {
        current.push(ch);
        if current.chars().count() >= width {
            wrapped.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        wrapped.push(current);
    }
    wrapped
}
