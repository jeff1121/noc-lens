//! CSV 設備清單匯入服務。
//!
//! 預期表頭（至少）：`ip_address,username,password,note,brand,groups`。
//! - `note`、`groups` 為選填；`groups` 以分號 `;` 分隔多個群組名稱。
//! - `brand` 必填且須為支援品牌。
//! - 逐列處理，單列失敗不影響其他列，最終回報成功數與失敗明細。

use serde::Serialize;
use sqlx::SqlitePool;

use crate::db::{device, group};
use crate::error::AppError;
use crate::models::{Brand, NewDevice};

/// 匯入結果。
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub success: usize,
    pub failed: Vec<ImportFailure>,
}

/// 單列匯入失敗明細。
#[derive(Debug, Serialize)]
pub struct ImportFailure {
    /// 資料列號（不含表頭，從 1 起算）。
    pub row: usize,
    pub reason: String,
}

/// 由 CSV 檔案路徑匯入設備清單。
pub async fn import_csv(pool: &SqlitePool, csv_path: &str) -> Result<ImportResult, AppError> {
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| AppError::File(format!("無法讀取 {csv_path}：{e}")))?;
    import_csv_str(pool, &content).await
}

/// 由 CSV 內容字串匯入設備清單（供前端讀檔後直接傳入）。
pub async fn import_csv_str(pool: &SqlitePool, content: &str) -> Result<ImportResult, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| AppError::Parse(format!("CSV 表頭解析失敗：{e}")))?
        .clone();
    let col = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let ip_idx = col("ip_address")
        .ok_or_else(|| AppError::Parse("缺少必要欄位 ip_address".to_string()))?;
    let user_idx = col("username")
        .ok_or_else(|| AppError::Parse("缺少必要欄位 username".to_string()))?;
    let pass_idx = col("password")
        .ok_or_else(|| AppError::Parse("缺少必要欄位 password".to_string()))?;
    let brand_idx =
        col("brand").ok_or_else(|| AppError::Parse("缺少必要欄位 brand".to_string()))?;
    let note_idx = col("note");
    let groups_idx = col("groups");

    let mut result = ImportResult {
        success: 0,
        failed: Vec::new(),
    };

    for (i, record) in reader.records().enumerate() {
        let row_no = i + 1;
        let record = match record {
            Ok(r) => r,
            Err(e) => {
                result.failed.push(ImportFailure {
                    row: row_no,
                    reason: format!("資料列解析失敗：{e}"),
                });
                continue;
            }
        };

        let get = |idx: usize| record.get(idx).unwrap_or("").trim().to_string();
        let brand_raw = get(brand_idx);
        let brand = match Brand::parse(&brand_raw) {
            Some(b) => b,
            None => {
                result.failed.push(ImportFailure {
                    row: row_no,
                    reason: format!("不支援的品牌：{brand_raw}"),
                });
                continue;
            }
        };

        let new_device = NewDevice {
            ip_address: get(ip_idx),
            username: get(user_idx),
            password: get(pass_idx),
            note: note_idx.map(get).filter(|s| !s.is_empty()),
            brand,
        };

        let created = match device::create(pool, new_device).await {
            Ok(d) => d,
            Err(e) => {
                result.failed.push(ImportFailure {
                    row: row_no,
                    reason: e.to_string(),
                });
                continue;
            }
        };

        // 處理群組（選填）：不存在則建立，再指派。
        if let Some(gidx) = groups_idx {
            let raw = get(gidx);
            if !raw.is_empty() {
                if let Err(e) = assign_groups(pool, &created.id, &raw).await {
                    // 設備已建立，群組指派失敗僅記錄為部分失敗原因。
                    result.failed.push(ImportFailure {
                        row: row_no,
                        reason: format!("設備已建立但群組指派失敗：{e}"),
                    });
                }
            }
        }

        result.success += 1;
    }

    Ok(result)
}

async fn assign_groups(
    pool: &SqlitePool,
    device_id: &str,
    raw: &str,
) -> Result<(), AppError> {
    let mut group_ids = Vec::new();
    for name in raw.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let g = match group::find_by_name(pool, name).await? {
            Some(g) => g,
            None => group::create(pool, name).await?,
        };
        group_ids.push(g.id);
    }
    if !group_ids.is_empty() {
        group::assign(pool, device_id, &group_ids).await?;
    }
    Ok(())
}
