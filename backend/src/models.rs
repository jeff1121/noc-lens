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
