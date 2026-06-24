//! 密碼加密與金鑰管理。
//!
//! 設備密碼以 AES-256-GCM 加密後儲存（不得明文）。主金鑰預設保存於 OS 金鑰庫
//! （macOS Keychain／Windows Credential Manager／Linux Secret Service）。
//!
//! 為利於測試與 CI（避免存取真實金鑰庫），若環境變數 `NOC_LENS_MASTER_KEY`
//! 存在（base64 編碼的 32 位元組金鑰），則優先採用之。

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::AppError;

const KEYRING_SERVICE: &str = "noc-lens";
const KEYRING_ACCOUNT: &str = "master-key";
const ENV_MASTER_KEY: &str = "NOC_LENS_MASTER_KEY";
const NONCE_LEN: usize = 12;

/// 取得（或建立）32 位元組主金鑰。
fn master_key() -> Result<[u8; 32], AppError> {
    if let Ok(b64) = std::env::var(ENV_MASTER_KEY) {
        let bytes = STANDARD
            .decode(b64.trim())
            .map_err(|e| AppError::Crypto(format!("主金鑰 base64 解析失敗：{e}")))?;
        return to_key_array(&bytes);
    }
    master_key_from_keyring()
}

#[cfg(not(test))]
fn master_key_from_keyring() -> Result<[u8; 32], AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    match entry.get_password() {
        Ok(b64) => {
            let bytes = STANDARD
                .decode(b64.trim())
                .map_err(|e| AppError::Crypto(e.to_string()))?;
            to_key_array(&bytes)
        }
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            entry
                .set_password(&STANDARD.encode(key))
                .map_err(|e| AppError::Keyring(e.to_string()))?;
            Ok(key)
        }
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

#[cfg(test)]
fn master_key_from_keyring() -> Result<[u8; 32], AppError> {
    // 測試環境不存取真實金鑰庫；要求以環境變數提供金鑰。
    Err(AppError::Keyring(
        "測試環境請設定 NOC_LENS_MASTER_KEY".to_string(),
    ))
}

fn to_key_array(bytes: &[u8]) -> Result<[u8; 32], AppError> {
    if bytes.len() != 32 {
        return Err(AppError::Crypto(format!(
            "主金鑰長度需為 32 位元組，實際為 {}",
            bytes.len()
        )));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(bytes);
    Ok(key)
}

/// 將明文加密為 `nonce(12) || ciphertext` 位元組。
pub fn encrypt(plain: &str) -> Result<Vec<u8>, AppError> {
    let key = master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| AppError::Crypto(e.to_string()))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    let mut out = nonce_bytes.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// 將 `nonce(12) || ciphertext` 解密回明文。
pub fn decrypt(data: &[u8]) -> Result<String, AppError> {
    if data.len() < NONCE_LEN {
        return Err(AppError::Crypto("密文長度不足".to_string()));
    }
    let key = master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| AppError::Crypto(e.to_string()))?;
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    String::from_utf8(plain).map_err(|e| AppError::Crypto(e.to_string()))
}

const AI_KEY_ACCOUNT: &str = "ai-api-key";

/// 將 AI API 金鑰存入 OS 金鑰庫（不寫入資料庫）。
pub fn set_ai_key(key: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, AI_KEY_ACCOUNT)
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry
        .set_password(key)
        .map_err(|e| AppError::Keyring(e.to_string()))
}

/// 由 OS 金鑰庫讀取 AI API 金鑰；未設定時回傳 `None`。
pub fn get_ai_key() -> Result<Option<String>, AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, AI_KEY_ACCOUNT)
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    match entry.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_test_key() {
        std::env::set_var(ENV_MASTER_KEY, STANDARD.encode([7u8; 32]));
    }

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        set_test_key();
        let secret = "P@ssw0rd-測試";
        let enc = encrypt(secret).unwrap();
        assert_ne!(enc, secret.as_bytes());
        let dec = decrypt(&enc).unwrap();
        assert_eq!(dec, secret);
    }

    #[test]
    fn ciphertext_differs_each_time() {
        set_test_key();
        let a = encrypt("same").unwrap();
        let b = encrypt("same").unwrap();
        assert_ne!(a, b, "nonce 應使每次密文不同");
    }
}
