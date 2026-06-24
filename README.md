# noc-lens

![version](https://img.shields.io/github/v/tag/jeff1121/noc-lens?label=version&sort=semver)
![license](https://img.shields.io/badge/license-MIT-blue)
![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey)

> 跨平台桌面版網路設備監控工具：透過 SSH 排程查詢日常維護狀態，並由 AI 產生易讀摘要與報告。

noc-lens 協助 NOC（網路維運中心）人員集中管理網路設備、透過 SSH 即時或排程查詢設備狀態
（CPU、Memory、module、interface、loading、traffic），於本地保存歷史資料，並由 AI
（OpenAI 相容端點）產生維護摘要報告。

## 功能

- **設備清單與分組／標籤**：手動新增或 CSV 匯入；自訂群組（如「高雄三民區」）。
- **多品牌 SSH 查詢**：Cisco、Aruba、Fortigate-NGFW、Palo Alto Networks。
- **排程監控**：固定間隔／每日固定時間自動查詢，本地保存歷史與趨勢。
- **AI 摘要報告**：依狀態資料產生繁體中文 Markdown 報告，可匯出。

## 技術棧

| 層 | 技術 |
|----|------|
| 桌面殼層 | Tauri 2 |
| 前端 | Vue 3 + Vite + TypeScript + Tailwind CSS + Pinia + ApexCharts |
| 後端核心 | Rust（russh、sqlx/SQLite、tokio-cron-scheduler、keyring + AES-256-GCM、reqwest） |
| 本地儲存 | SQLite |

## 開發

詳見 [specs/001-network-device-monitoring/quickstart.md](specs/001-network-device-monitoring/quickstart.md)。

```bash
# 前端依賴
cd frontend && npm install && cd ..

# 開發模式（Tauri）
cargo tauri dev

# 測試
cargo test                       # 後端
cd frontend && npm run build     # 前端建置
```

## 安全

- 設備密碼以 AES-256-GCM 加密儲存，主金鑰存於 OS 金鑰庫（不明文落地）。
- AI 採可設定的 OpenAI 相容端點：可指向雲端或本地（如 Ollama／LM Studio）；
  使用雲端時設備狀態資料會離開本機（設定畫面會提示）。

## 授權

MIT
