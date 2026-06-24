//! 統一錯誤型別。
//!
//! `AppError` 會序列化為 `{ code, message }`（message 為繁體中文），
//! 供前端依 `code` 判斷錯誤類型、向使用者顯示 `message`。

use serde::ser::{Serialize, SerializeStruct, Serializer};

/// 應用程式錯誤。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("資料庫錯誤：{0}")]
    Db(String),
    #[error("驗證錯誤：{0}")]
    Validation(String),
    #[error("找不到資料：{0}")]
    NotFound(String),
    #[error("IP 位址重複：{0}")]
    DuplicateIp(String),
    #[error("不支援的品牌：{0}")]
    UnsupportedBrand(String),
    #[error("名稱重複：{0}")]
    DuplicateName(String),
    #[error("檔案錯誤：{0}")]
    File(String),
    #[error("解析錯誤：{0}")]
    Parse(String),
    #[error("加密錯誤：{0}")]
    Crypto(String),
    #[error("金鑰庫錯誤：{0}")]
    Keyring(String),
}

impl AppError {
    /// 對應的錯誤碼（供前端判斷）。
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Db(_) => "DB_ERROR",
            AppError::Validation(_) => "VALIDATION",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::DuplicateIp(_) => "DUPLICATE_IP",
            AppError::UnsupportedBrand(_) => "UNSUPPORTED_BRAND",
            AppError::DuplicateName(_) => "DUPLICATE_NAME",
            AppError::File(_) => "FILE_ERROR",
            AppError::Parse(_) => "PARSE_ERROR",
            AppError::Crypto(_) => "CRYPTO_ERROR",
            AppError::Keyring(_) => "KEYRING_ERROR",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Db(e.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        AppError::Db(e.to_string())
    }
}

impl From<csv::Error> for AppError {
    fn from(e: csv::Error) -> Self {
        AppError::Parse(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::File(e.to_string())
    }
}

/// 函式庫統一回傳型別。
pub type AppResult<T> = Result<T, AppError>;
