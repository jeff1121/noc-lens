//! 報告彙整與生成：依範圍蒐集設備狀態，交由 AI 產生摘要並儲存。

use serde_json::{json, Value};

use crate::ai::AiProvider;
use crate::db::{device, report as report_db, snapshot};
use crate::error::AppError;
use crate::models::{Report, ReportScope};
use crate::SqlitePool;

const SYSTEM_PROMPT: &str = "你是一位資深網路維運（NOC）工程師助理。請以繁體中文、\
Markdown 格式產生易讀的設備健康摘要與維護報告，須涵蓋：\
（1）整體健康概況（正常／警告／嚴重數量）；\
（2）異常與需關注項目（高 CPU／記憶體、介面 down、模組異常等，並標明設備）；\
（3）趨勢觀察與建議。僅根據提供的資料作答，不得臆測。";

/// 依範圍產生 AI 摘要報告並儲存。
pub async fn generate<P>(
    pool: &SqlitePool,
    provider: &P,
    scope: ReportScope,
    title: Option<String>,
    model_endpoint: Option<&str>,
) -> Result<Report, AppError>
where
    P: AiProvider + Sync,
{
    let device_ids = resolve_device_ids(pool, &scope).await?;

    let mut devices_json = Vec::new();
    for id in &device_ids {
        let dev = device::get(pool, id).await?;
        let snaps = snapshot::list_by_device(pool, id, 1).await?;
        let latest = snaps
            .first()
            .map(|s| json!({ "status": s.status, "metrics": s.metrics }))
            .unwrap_or(Value::Null);
        devices_json.push(json!({
            "ip_address": dev.ip_address,
            "brand": dev.brand.as_str(),
            "latest": latest,
        }));
    }

    let input = json!({
        "range": { "from": scope.from, "to": scope.to },
        "device_count": devices_json.len(),
        "devices": devices_json,
    });

    let user_prompt = format!(
        "以下為設備狀態資料（JSON）：\n```json\n{}\n```\n請產生維護摘要報告。",
        serde_json::to_string_pretty(&input).unwrap_or_default()
    );

    let summary = provider.complete(SYSTEM_PROMPT, &user_prompt).await?;
    let title = title.unwrap_or_else(|| "設備健康摘要報告".to_string());
    let scope_json = serde_json::to_string(&input).unwrap_or_default();

    report_db::insert(pool, &title, &scope_json, &summary, model_endpoint).await
}

async fn resolve_device_ids(
    pool: &SqlitePool,
    scope: &ReportScope,
) -> Result<Vec<String>, AppError> {
    let mut ids: Vec<String> = Vec::new();
    if let Some(d) = &scope.device_ids {
        ids.extend(d.clone());
    }
    if let Some(groups) = &scope.group_ids {
        for g in groups {
            for dev in device::list(pool, Some(g)).await? {
                ids.push(dev.id);
            }
        }
    }
    if ids.is_empty() {
        for dev in device::list(pool, None).await? {
            ids.push(dev.id);
        }
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}
