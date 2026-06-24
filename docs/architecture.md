# noc-lens 架構說明

## 總覽

noc-lens 為 Tauri 2 桌面應用，採 Cargo workspace 組織，前後端分層：

```
frontend/   Vue 3 + Vite 前端（UI、Pinia、ApexCharts、vue-virtual-scroller）
src-tauri/  Tauri 殼層（IPC 指令、AppState、排程服務啟動）
backend/    Rust 核心函式庫（領域邏輯，可獨立 cargo test）
e2e/        Playwright（head 模式煙霧測試）
```

## 資料流

```
Vue 前端 ──invoke──▶ src-tauri commands ──▶ backend（db / ssh / scheduler / ai）──▶ SQLite
```

- 前端透過 `frontend/src/api/tauri.ts` 封裝 `invoke` 呼叫後端指令。
- 後端指令回傳 `Result<T, AppError>`；`AppError` 序列化為 `{ code, message(zh-TW) }`。

## backend 模組

| 模組 | 職責 |
|------|------|
| `models` | 領域模型與 `Brand` 列舉 |
| `error` | 統一 `AppError`（含錯誤碼） |
| `crypto` | AES-256-GCM 密碼加密 + OS 金鑰庫（含 AI 金鑰） |
| `db` | SQLite 存取：device / group / snapshot / schedule / report / settings |
| `services::import` | CSV 匯入 |
| `ssh` | `SshExecutor` 抽象、russh 客戶端、品牌指令對應、解析、查詢編排 |
| `scheduler` | `run_job_once` + `SchedulerService`（tokio-cron-scheduler） |
| `ai` | `AiProvider` 抽象、OpenAI 相容客戶端、報告生成 |

## 可測試性

關鍵外部互動（SSH、AI）皆以 trait 抽象（`SshExecutor`、`AiProvider`），
使編排與解析邏輯能以 mock 進行 `cargo test`，無需真實設備或 AI 端點。

## 品牌指令對應

見 [../specs/001-network-device-monitoring/contracts/brand-commands.md](../specs/001-network-device-monitoring/contracts/brand-commands.md)。
Cisco 解析較完整；其餘品牌採盡力解析，無法解析的指標標示「不適用（n/a）」。
