---
description: "Task list for noc-lens network device monitoring"
---

# Tasks: noc-lens 排程式網路設備監控與 AI 摘要報告

**Input**: Design documents from `specs/001-network-device-monitoring/`

**Prerequisites**: [plan.md](./plan.md)（必要）、[spec.md](./spec.md)、[research.md](./research.md)、[data-model.md](./data-model.md)、[contracts/](./contracts/)、[quickstart.md](./quickstart.md)

**Tests**: 依憲章原則 II（測試標準），本任務清單**包含**測試任務（Rust `cargo test`、前端 Vitest、Playwright head 模式 E2E）。

**Organization**: 任務依使用者故事分組，使每個故事可獨立實作、獨立測試、獨立交付。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可平行執行（不同檔案、無未完成相依）
- **[Story]**: 對應使用者故事（US1–US4）
- 每個任務含明確檔案路徑

## Path Conventions

- `frontend/`（Vue 3 + Vite + Tauri 前端）、`src-tauri/`（Tauri 殼層）、`backend/`（Rust 核心函式庫）、`e2e/`（Playwright）、`infra/`、`docs/`、`.github/workflows/`
- 結構依 [plan.md](./plan.md) 之 Cargo workspace 配置

---

## Phase 1: Setup（共用基礎建設）

**Purpose**: 專案初始化與基本結構

- [x] T001 建立 Cargo workspace 根結構與 `Cargo.toml`（workspace 成員：`src-tauri`、`backend`），並建立 `frontend/`、`e2e/`、`infra/`、`docs/`、`.github/workflows/` 目錄骨架，per [plan.md](./plan.md)
- [x] T002 [P] 初始化 `frontend/`（Vue 3 + Vite + TypeScript），安裝 Pinia、vue-router、Tailwind CSS、vue3-apexcharts、vue-virtual-scroller in frontend/package.json
- [x] T003 [P] 初始化 `src-tauri/`（Tauri 2，`tauri.conf.json` 設定 `frontendDist` 指向 frontend、bundle 目標 macOS/Windows）in src-tauri/
- [x] T004 [P] 初始化 `backend/` Rust 函式庫 crate，加入 tokio、russh、sqlx(SQLite)、tokio-cron-scheduler、keyring、aes-gcm、reqwest、serde in backend/Cargo.toml
- [ ] T005 [P] 設定 lint/format：Rust（rustfmt.toml、clippy）與前端（ESLint、Prettier）in repo 根與 frontend/
- [ ] T006 [P] 初始化 `e2e/` Playwright（設定 head 模式執行）in e2e/playwright.config.ts

**Checkpoint**: 專案可建置、可啟動空殼 `cargo tauri dev`

---

## Phase 2: Foundational（阻塞性前置）

**Purpose**: 所有使用者故事開始前必須完成的核心基礎

**⚠️ CRITICAL**: 本階段完成前，任何使用者故事不得開工

- [x] T007 建立 SQLite schema 與 sqlx migrations（Device、Group、DeviceGroup、StatusSnapshot、ScheduledJob、JobRun、Report、AppSetting）per [data-model.md](./data-model.md) in backend/src/db/migrations/
- [x] T008 [P] 定義領域模型與列舉（含 `Brand` enum：cisco/aruba/fortigate_ngfw/palo_alto）in backend/src/models/
- [x] T009 [P] 實作統一錯誤型別 `AppError`（code 列舉 + zh-TW message）in backend/src/error.rs
- [x] T010 [P] 實作 crypto 模組：keyring 主金鑰管理 + AES-256-GCM 密碼加解密（FR-023/024）in backend/src/crypto/mod.rs
- [x] T011 實作 DB 連線初始化與 repository 基礎 helper（依賴 T007）in backend/src/db/mod.rs
- [x] T012 建立 src-tauri 指令註冊骨架與 `AppError` 序列化對應（invoke handler、main.rs 啟動 backend 狀態）in src-tauri/src/main.rs、src-tauri/src/commands/mod.rs
- [x] T013 [P] 前端基礎骨架：vue-router（6 畫面占位：DeviceList/DeviceDetail/Groups/Schedules/Reports/Settings）、Pinia 根、Tauri `invoke` 封裝 in frontend/src/{router,stores,api}/
- [x] T014 [P] 前端設計 token 與版面：Tailwind 深色 (OLED) 主題、狀態色（critical `#EF4444`/warning `#F59E0B`/normal `#22C55E`/updating `#3B82F6`）、Fira Code/Fira Sans 字型、Sidebar+Content 版面、SVG 圖示，per [research.md](./research.md) §9 in frontend/src/styles/、frontend/src/layouts/
- [x] T015 [P] AppSetting 存取與一般設定指令（settings_get/settings_set，含 ssh_max_concurrency）per [contracts/tauri-commands.md](./contracts/tauri-commands.md) in backend/src/db/settings.rs、src-tauri/src/commands/settings.rs

**Checkpoint**: 基礎就緒，使用者故事可開始平行開發

---

## Phase 3: User Story 1 - 設備清單與分組／標籤管理 (Priority: P1) 🎯 MVP

**Goal**: 集中管理設備（IP/帳號/密碼/備註/品牌），支援匯入與自訂群組／標籤分類檢視。

**Independent Test**: 匯入一份 CSV、為部分設備指派群組／標籤、依群組篩選並檢視清單；無需任何 SSH 查詢即可完成並交付價值。

### Tests for User Story 1 ⚠️（先寫、先失敗）

- [x] T016 [P] [US1] backend 整合測試：Device CRUD + 重複 IP + 不支援品牌 in backend/tests/device_test.rs
- [x] T017 [P] [US1] backend 整合測試：CSV 匯入（成功/失敗逐列回報）in backend/tests/import_test.rs
- [x] T018 [P] [US1] backend 整合測試：Group CRUD + 多對多指派 + 依群組篩選 in backend/tests/group_test.rs

### Implementation for User Story 1

- [x] T019 [P] [US1] Device repository（list/create/update/delete，寫入時密碼加密）in backend/src/db/device.rs
- [x] T020 [P] [US1] Group repository（list/create/delete/assign，DeviceGroup 多對多）in backend/src/db/group.rs
- [x] T021 [US1] CSV 匯入服務（解析、欄位驗證、重複 IP/品牌檢查、逐列結果）依賴 T019 in backend/src/services/import.rs
- [x] T022 [US1] Tauri 指令：device_list/device_create/device_update/device_delete/device_import in src-tauri/src/commands/device.rs
- [x] T023 [US1] Tauri 指令：group_list/group_create/group_delete/group_assign in src-tauri/src/commands/group.rs
- [x] T024 [P] [US1] 前端 DeviceList 畫面（vue-virtual-scroller 表格、多選、群組篩選、狀態徽章占位、skeleton）in frontend/src/views/DeviceList.vue
- [x] T025 [P] [US1] 前端 設備新增/編輯表單元件（驗證、品牌下拉、密碼欄、focus/對比合規）in frontend/src/components/DeviceForm.vue
- [x] T026 [P] [US1] 前端 群組管理畫面（建立/刪除/指派）in frontend/src/views/Groups.vue
- [x] T027 [US1] 前端 匯入對話框（選檔、呼叫 device_import、成功/失敗結果列表）in frontend/src/components/ImportDialog.vue
- [x] T028 [US1] 前端 Pinia store：devices、groups in frontend/src/stores/{devices,groups}.ts
- [ ] T029 [US1] E2E（head）：匯入 → 指派群組 → 依群組篩選 in e2e/us1.spec.ts

**Checkpoint**: US1 可獨立運作並通過測試（MVP 可交付）

---

## Phase 4: User Story 2 - 透過 SSH 即時查詢設備狀態（多品牌） (Priority: P2)

**Goal**: 選取設備即時 SSH 查詢 CPU/Memory/module/interface/loading/traffic，依品牌套用指令並結構化呈現。

**Independent Test**: 新增一台可連線且品牌受支援的設備，執行一次查詢，確認狀態正確擷取並易讀呈現；連線/認證失敗時顯示明確原因且不影響其他設備。

### Tests for User Story 2 ⚠️（先寫、先失敗）

- [x] T030 [P] [US2] backend 單元測試：各品牌輸出解析器（cisco/aruba/fortigate_ngfw/palo_alto，含 n/a 指標）in backend/tests/parser_test.rs
- [x] T031 [P] [US2] backend 整合測試：查詢流程（成功/認證失敗/逾時、併發上限、逐台回報）以 mock SSH in backend/tests/query_test.rs

### Implementation for User Story 2

- [x] T032 [P] [US2] SSH 客戶端（russh，密碼認證、執行唯讀指令、逾時控制）in backend/src/ssh/client.rs
- [x] T033 [P] [US2] 品牌指令對應表 per [contracts/brand-commands.md](./contracts/brand-commands.md) in backend/src/ssh/commands.rs
- [x] T034 [US2] 各品牌輸出解析器 → metrics_json（不適用標 `n/a`，FR-010）依賴 T033 in backend/src/ssh/parsers/
- [x] T035 [US2] 查詢編排服務（tokio Semaphore 併發上限、逐台成功/失敗、寫入 StatusSnapshot）依賴 T032、T034 in backend/src/ssh/query.rs
- [x] T036 [US2] Tauri 指令：query_devices per [contracts/tauri-commands.md](./contracts/tauri-commands.md) in src-tauri/src/commands/query.rs
- [x] T037 [P] [US2] 前端 DeviceDetail 畫面（結構化狀態、ApexCharts gauge=CPU/Mem、line=trend、skeleton 載入）in frontend/src/views/DeviceDetail.vue
- [x] T038 [US2] 前端 即時查詢觸發與逐台結果/錯誤呈現（DeviceList 多選查詢 + DeviceDetail）in frontend/src/views/DeviceList.vue、frontend/src/stores/query.ts
- [ ] T039 [US2] E2E（head）：對設備執行即時查詢並檢視狀態結果 in e2e/us2.spec.ts

**Checkpoint**: US1 與 US2 皆可獨立運作

---

## Phase 5: User Story 3 - 排程監控與本地狀態資料儲存 (Priority: P3)

**Goal**: 對設備/群組設定排程（固定間隔/每日固定時間），自動查詢並於本地保存歷史，可回溯檢視。

**Independent Test**: 對群組設排程、觸發一次，確認自動收集並保存、可於歷史檢視快照、重啟後資料仍在。

### Tests for User Story 3 ⚠️（先寫、先失敗）

- [ ] T040 [P] [US3] backend 整合測試：排程建立/觸發/JobRun 成功失敗統計、快照寫入、重啟後保留 in backend/tests/schedule_test.rs
- [ ] T041 [P] [US3] backend 整合測試：snapshot_list 依 collected_at 排序 in backend/tests/history_test.rs

### Implementation for User Story 3

- [ ] T042 [P] [US3] ScheduledJob/JobRun repository in backend/src/db/schedule.rs
- [ ] T043 [US3] 排程器（tokio-cron-scheduler，interval/daily，呼叫 US2 查詢服務、寫 JobRun + snapshots、重疊保護）依賴 T035、T042 in backend/src/scheduler/mod.rs
- [ ] T044 [US3] Tauri 指令：schedule_list/create/update/delete/toggle、job_run_list、snapshot_list in src-tauri/src/commands/schedule.rs
- [ ] T045 [P] [US3] 前端 Schedules 畫面（建立 interval/daily、啟用切換、執行紀錄明細）in frontend/src/views/Schedules.vue
- [ ] T046 [P] [US3] 前端 設備歷史趨勢分頁（時間序列 line chart）in frontend/src/views/DeviceDetail.vue
- [ ] T047 [US3] 前端 Pinia store：schedules、history in frontend/src/stores/{schedules,history}.ts
- [ ] T048 [US3] E2E（head）：建立排程 → 觸發 → 檢視歷史 in e2e/us3.spec.ts

**Checkpoint**: US1/US2/US3 皆可獨立運作

---

## Phase 6: User Story 4 - AI 摘要與報告生成 (Priority: P3)

**Goal**: 針對單次或一段期間資料由 AI（OpenAI 相容端點）產生可讀摘要與報告，可匯出。

**Independent Test**: 設定 AI 端點後，從已收集資料產生並匯出報告，確認內容對應實際資料；AI 不可用時明確提示且不影響原始資料。

### Tests for User Story 4 ⚠️（先寫、先失敗）

- [ ] T049 [P] [US4] backend 整合測試：report_generate（彙整去敏感欄位、AI_CONFIG_MISSING、AI_UNAVAILABLE）以 mock 端點 in backend/tests/ai_test.rs

### Implementation for User Story 4

- [ ] T050 [P] [US4] AI 客戶端（reqwest，OpenAI 相容端點，base URL + 金鑰）in backend/src/ai/client.rs
- [ ] T051 [US4] 報告彙整與生成服務（scope → 精簡 JSON、排除密碼/金鑰、summary_md、存 Report）per [contracts/ai-report.md](./contracts/ai-report.md) 依賴 T050 in backend/src/ai/report.rs
- [ ] T052 [US4] AI 金鑰寫入 keyring：settings_set_ai_key；settings_get 回傳 ai_key_set（不回傳金鑰，FR-024）in src-tauri/src/commands/settings.rs
- [ ] T053 [US4] Tauri 指令：report_generate/report_list/report_export（md/pdf）in src-tauri/src/commands/report.rs
- [ ] T054 [P] [US4] 前端 Reports 畫面（產生、檢視 Markdown、匯出）in frontend/src/views/Reports.vue
- [ ] T055 [P] [US4] 前端 Settings 畫面（AI base_url/model/金鑰、SSH 併發、雲端隱私提示，FR-027）in frontend/src/views/Settings.vue
- [ ] T056 [US4] E2E（head）：設定 AI → 產生報告 → 匯出 in e2e/us4.spec.ts

**Checkpoint**: 四個使用者故事皆可獨立運作

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: 跨故事的品質、發行與自動化

- [ ] T057 [P] GitHub Actions：跨平台建置與發行（matrix `macos-latest` + `windows-latest`，產出 .dmg/.app 與 .msi/.exe 並上傳 GitHub Releases）in .github/workflows/release.yml
- [ ] T058 [P] GitHub Actions：程式碼審查工作流程 → 修正問題 + 摘要 + 產生報告（**zh-TW**）in .github/workflows/code-review.yml
- [ ] T059 [P] GitHub Actions：CodeQL 安全掃描 → 修正問題 + 摘要 + 產生報告（**zh-TW**）in .github/workflows/codeql.yml
- [ ] T060 [P] README（shields.io 版本徽章、專案說明，**zh-TW**）in README.md
- [ ] T061 [P] 提交訊息規範（詳細、**zh-TW**）與 commit template in docs/commit-guideline.md、.gitmessage
- [ ] T062 [P] 應用程式圖示與 Tauri bundle 設定（macOS/Windows，含 Release 圖示）in src-tauri/icons/、src-tauri/tauri.conf.json
- [ ] T063 效能驗證：設備清單虛擬捲動 ~60fps、UI 互動 p95 < 200ms、SSH 併發上限（量測並記錄）對齊 plan Performance Goals
- [ ] T064 安全強化複查：密碼於 UI/log/匯出報告皆不明文外露、金鑰僅存 keyring（FR-023/024、OWASP A02）
- [ ] T065 [P] docs/ 補充（架構說明、品牌指令對應，**zh-TW**）in docs/
- [ ] T066 執行 [quickstart.md](./quickstart.md) 全情境驗證（US1–US4）
- [ ] T067 以 `gh repo create` 建立 GitHub 儲存庫並推送、啟用 Release 頁面（**需使用者授權後執行**）

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 無相依，可立即開始
- **Foundational (Phase 2)**: 依賴 Setup 完成 — 阻塞所有使用者故事
- **User Stories (Phase 3–6)**: 皆依賴 Foundational 完成
  - 之後可平行進行；或依優先序 P1 → P2 → P3 循序
  - US3 的排程器依賴 US2 的查詢服務（T043 依賴 T035）；US4 獨立於 US2/US3
- **Polish (Phase 7)**: 依賴欲交付之使用者故事完成

### User Story Dependencies

- **US1 (P1)**: Foundational 後即可開始，無對其他故事相依
- **US2 (P2)**: Foundational 後即可開始，獨立可測
- **US3 (P3)**: Foundational 後可開始；**排程執行**重用 US2 查詢服務（如先做 US3，需先具備 T035 查詢編排）
- **US4 (P3)**: Foundational 後可開始；對已收集資料生成報告，獨立可測（可用既有/手動資料驗證）

### Within Each User Story

- 測試先寫且先失敗 → 模型/Repository → 服務 → Tauri 指令 → 前端 → E2E
- backend repository 先於服務；服務先於 Tauri 指令；指令先於前端串接

### Parallel Opportunities

- Setup 中 T002–T006（[P]）可平行
- Foundational 中 T008、T009、T010、T013、T014、T015（[P]）可平行
- 每個故事的測試任務（[P]）可平行先行
- 不同 `.rs`/`.vue` 檔案的 [P] 任務可平行；同檔案任務需循序
- Foundational 完成後，US1/US2/US4 可由不同人平行開發

---

## Parallel Example: User Story 1

```text
# 先平行啟動 US1 測試（先失敗）：
T016 [P] backend/tests/device_test.rs
T017 [P] backend/tests/import_test.rs
T018 [P] backend/tests/group_test.rs

# 再平行實作不同檔案：
T019 [P] backend/src/db/device.rs
T020 [P] backend/src/db/group.rs
T024 [P] frontend/src/views/DeviceList.vue
T025 [P] frontend/src/components/DeviceForm.vue
T026 [P] frontend/src/views/Groups.vue
```

---

## Implementation Strategy

### MVP First（僅 User Story 1）

1. 完成 Phase 1（Setup）→ Phase 2（Foundational）
2. 完成 Phase 3（US1：設備清單與分組）
3. **停下驗證**：以 quickstart US1 情境獨立測試 → 可作為 MVP 交付

### Incremental Delivery（增量交付）

1. Setup + Foundational → 基礎就緒
2. 加入 US1 → 測試 → 交付（MVP：集中管理設備）
3. 加入 US2 → 測試 → 交付（即時 SSH 查詢）
4. 加入 US3 → 測試 → 交付（排程 + 歷史）
5. 加入 US4 → 測試 → 交付（AI 報告）
6. Phase 7 Polish（CI/CD、發行、安全/效能複查）

每個故事完成即為一個可獨立運作、可展示的增量。
