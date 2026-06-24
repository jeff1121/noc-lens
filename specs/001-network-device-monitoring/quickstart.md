# Quickstart：noc-lens 開發與驗證指南

**功能**：[spec.md](./spec.md) ｜ **計畫**：[plan.md](./plan.md) ｜ **日期**：2026-06-24

本檔為「可執行的驗證指南」，說明環境需求、啟動方式與對應四個使用者故事的驗收情境。實作細節請見 plan.md、data-model.md 與 contracts/。

---

## 先決條件

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.80+（含 `cargo`） | 後端核心 + Tauri 殼層 |
| Node.js | 20+（含 `npm`/`pnpm`） | 前端 Vue/Vite |
| Tauri CLI | 2.x（`cargo install tauri-cli` 或 `@tauri-apps/cli`） | 桌面建置與開發 |
| 平台依賴 | macOS：Xcode CLT；Windows：MSVC + WebView2 | Tauri 打包 |
| Playwright | 由 `e2e/` 安裝 | E2E（head 模式） |

> AI 功能：於設定畫面填入 OpenAI 相容端點 base URL 與金鑰（雲端或本地 Ollama/LM Studio 皆可）。

## 安裝與啟動（開發）

```bash
# 1. 安裝前端依賴
cd frontend && npm install && cd ..

# 2. 啟動桌面開發模式（Tauri 會同時起前端 dev server 與 Rust 後端）
cargo tauri dev
```

## 測試

```bash
# Rust 核心（單元 + 整合）
cargo test

# 前端單元測試
cd frontend && npm run test:unit

# E2E（依憲章以 head 模式執行）
cd e2e && npx playwright test --headed
```

## 建置與發行（macOS / Windows）

```bash
# 本機建置安裝檔（依當前 OS 產出 .dmg/.app 或 .msi/.exe）
cargo tauri build
```

> CI 以 GitHub Actions `macos-latest` + `windows-latest` 矩陣建置並上傳至 GitHub Releases（詳見 research.md「CI/CD」）。

---

## 驗收情境（對應使用者故事）

### US1：設備清單與分組／標籤管理（P1）

1. 準備一份 CSV（欄位 `ip_address,username,password,note,brand,groups`）。
2. 於「設備清單」匯入 → 確認顯示成功筆數與失敗（含原因）筆數。
3. 建立群組「高雄三民區」「高雄高中」並指派設備 → 依群組篩選可見對應設備。
- **預期**：對應 [contracts/tauri-commands.md](./contracts/tauri-commands.md) 之 `device_import`、`group_create`、`group_assign`、`device_list`。

### US2：透過 SSH 即時查詢設備狀態（P2）

1. 新增一台可連線、品牌受支援的設備。
2. 選取該設備，執行「即時查詢」。
- **預期**：30 秒內回傳整理後的 CPU/Memory/module/interface/loading/traffic；連線/認證失敗時顯示明確原因且不影響其他設備（`query_devices`）。

### US3：排程監控與本地儲存（P3）

1. 對「高雄三民區」群組建立「每日固定時間」或「固定間隔」排程。
2. 等待或觸發一次執行。
- **預期**：自動收集並寫入本地；可於設備歷史依時間檢視快照；JobRun 記錄成功/失敗明細；重啟後資料仍在（`schedule_create`、`snapshot_list`、`job_run_list`）。

### US4：AI 摘要與報告（P3）

1. 於設定填入 AI 端點與金鑰。
2. 對一段期間的資料產生報告並匯出。
- **預期**：輸出涵蓋整體健康與異常重點的 Markdown 摘要；可匯出檔案；AI 不可用時明確提示且不影響原始資料（`report_generate`、`report_export`，見 [contracts/ai-report.md](./contracts/ai-report.md)）。

---

## 設計系統參考（前端）

- 風格：深色 (OLED) + 即時監控 + 資料密集儀表板。
- 狀態色：嚴重 `#EF4444`、警告 `#F59E0B`、正常 `#22C55E`、更新中 `#3B82F6`。
- 字體：`Fira Code`（數據/標題）、`Fira Sans`（內文）。
- 清單虛擬化、skeleton 載入、SVG 圖示、對比 ≥ 4.5:1、focus 可見、尊重 `prefers-reduced-motion`。
