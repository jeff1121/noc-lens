//! 領域模型與列舉。

use serde::{Deserialize, Serialize};

/// 支援的網路設備品牌。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Brand {
    Cisco,
    Aruba,
    FortigateNgfw,
    PaloAlto,
}

impl Brand {
    /// 資料庫／傳輸用的字串值。
    pub fn as_str(&self) -> &'static str {
        match self {
            Brand::Cisco => "cisco",
            Brand::Aruba => "aruba",
            Brand::FortigateNgfw => "fortigate_ngfw",
            Brand::PaloAlto => "palo_alto",
        }
    }

    /// 由字串解析品牌；不支援時回傳 `None`。
    pub fn parse(s: &str) -> Option<Brand> {
        match s {
            "cisco" => Some(Brand::Cisco),
            "aruba" => Some(Brand::Aruba),
            "fortigate_ngfw" => Some(Brand::FortigateNgfw),
            "palo_alto" => Some(Brand::PaloAlto),
            _ => None,
        }
    }
}

/// 設備（對外呈現，**不含**密碼）。
#[derive(Debug, Clone, Serialize)]
pub struct Device {
    pub id: String,
    pub ip_address: String,
    pub username: String,
    pub note: Option<String>,
    pub brand: Brand,
    pub created_at: String,
    pub updated_at: String,
}

/// 新增設備的輸入（含明文密碼，僅於建立時傳入）。
#[derive(Debug, Clone, Deserialize)]
pub struct NewDevice {
    pub ip_address: String,
    pub username: String,
    pub password: String,
    pub note: Option<String>,
    pub brand: Brand,
}

/// 更新設備的輸入（欄位皆為選填）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateDevice {
    pub ip_address: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub note: Option<String>,
    pub brand: Option<Brand>,
}

/// 群組／標籤。
#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// 狀態快照（某次查詢取得的設備狀態）。
#[derive(Debug, Clone, Serialize)]
pub struct StatusSnapshot {
    pub id: String,
    pub device_id: String,
    pub job_run_id: Option<String>,
    pub collected_at: String,
    /// `ok` / `partial` / `failed`
    pub status: String,
    pub error_message: Option<String>,
    /// 結構化指標（CPU/Memory/module/interface/loading/traffic）。
    pub metrics: serde_json::Value,
}

/// 即時查詢單台設備的結果（對應 contracts/tauri-commands.md 之 QueryResult）。
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub device_id: String,
    pub status: String,
    pub error_message: Option<String>,
    pub metrics: Option<serde_json::Value>,
    pub snapshot_id: Option<String>,
}

/// 排程工作。
#[derive(Debug, Clone, Serialize)]
pub struct ScheduledJob {
    pub id: String,
    pub name: String,
    /// `device` 或 `group`
    pub target_type: String,
    pub target_id: String,
    /// `interval` 或 `daily`
    pub schedule_kind: String,
    pub interval_minutes: Option<i64>,
    pub daily_time: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

/// 建立排程的輸入。
#[derive(Debug, Clone, Deserialize)]
pub struct NewScheduledJob {
    pub name: String,
    pub target_type: String,
    pub target_id: String,
    pub schedule_kind: String,
    pub interval_minutes: Option<i64>,
    pub daily_time: Option<String>,
}

/// 排程執行紀錄。
#[derive(Debug, Clone, Serialize)]
pub struct JobRun {
    pub id: String,
    pub job_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub total: i64,
    pub success_count: i64,
    pub failure_count: i64,
}

/// AI 報告。
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    pub scope_json: String,
    pub summary_md: String,
    pub generated_at: String,
    pub model_endpoint: Option<String>,
}

/// 報告引用範圍。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReportScope {
    pub device_ids: Option<Vec<String>>,
    pub group_ids: Option<Vec<String>>,
    pub from: Option<String>,
    pub to: Option<String>,
}
