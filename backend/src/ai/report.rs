//! 報告彙整與生成：依範圍蒐集設備狀態，交由 AI 產生摘要並儲存。

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::ai::AiProvider;
use crate::db::{device, group, report as report_db, snapshot};
use crate::error::AppError;
use crate::models::{Report, ReportScope, StatusSnapshot};
use crate::SqlitePool;

const SYSTEM_PROMPT: &str = "你是一位資深網路維運（NOC）工程師助理。請以繁體中文、\
Markdown 格式產生易讀的設備健康摘要與維護報告，須涵蓋：\
（1）整體健康概況（正常／警告／嚴重數量）；\
（2）異常與需關注項目（高 CPU／記憶體、介面 down、模組異常等，並標明設備）；\
（3）趨勢觀察與建議。僅根據提供的資料作答，不得臆測。";

const REPORT_MAX_SNAPSHOTS_PER_DEVICE: i64 = 200;

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
        let groups = group::groups_for_device(pool, id).await?;
        let snaps = snapshot::list_by_device_filtered(
            pool,
            id,
            scope.from.as_deref(),
            scope.to.as_deref(),
            Some(REPORT_MAX_SNAPSHOTS_PER_DEVICE),
        )
        .await?;
        let latest = snaps.first().map(snapshot_to_json).unwrap_or(Value::Null);
        let snapshots = snaps.iter().map(snapshot_to_json).collect::<Vec<_>>();
        devices_json.push(json!({
            "ip_address": dev.ip_address,
            "brand": dev.brand.as_str(),
            "group_names": groups.into_iter().map(|g| g.name).collect::<Vec<_>>(),
            "snapshot_count": snaps.len(),
            "latest": latest,
            "trend": build_trend(&snaps),
            "snapshots": snapshots,
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

fn snapshot_to_json(snapshot: &StatusSnapshot) -> Value {
    json!({
        "collected_at": &snapshot.collected_at,
        "status": &snapshot.status,
        "error_message": &snapshot.error_message,
        "metrics": &snapshot.metrics,
    })
}

fn build_trend(snapshots: &[StatusSnapshot]) -> Value {
    let mut cpu_values = Vec::new();
    let mut memory_values = Vec::new();
    let mut interface_down_values = Vec::new();
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();

    for snapshot in snapshots {
        *status_counts.entry(snapshot.status.clone()).or_default() += 1;
        if let Some(value) = metric_f64(&snapshot.metrics, &["cpu", "usage_percent"]) {
            cpu_values.push(value);
        }
        if let Some(value) = metric_f64(&snapshot.metrics, &["memory", "usage_percent"]) {
            memory_values.push(value);
        }
        if let Some(value) = metric_i64(&snapshot.metrics, &["interface", "down"]) {
            interface_down_values.push(value);
        }
    }

    json!({
        "cpu_max": max_f64(&cpu_values),
        "cpu_avg": avg_f64(&cpu_values),
        "memory_max": max_f64(&memory_values),
        "memory_avg": avg_f64(&memory_values),
        "interface_down_max": interface_down_values.into_iter().max(),
        "status_counts": status_counts,
    })
}

fn metric_f64(value: &Value, path: &[&str]) -> Option<f64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_f64()
}

fn metric_i64(value: &Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_i64()
}

fn max_f64(values: &[f64]) -> Option<f64> {
    values.iter().copied().reduce(f64::max)
}

fn avg_f64(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
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
