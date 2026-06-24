# Implementation Plan: noc-lens 排程式網路設備監控與 AI 摘要報告

**Branch**: `001-network-device-monitoring` | **Date**: 2026-06-24 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-network-device-monitoring/spec.md`

## Summary

noc-lens 是一套跨平台桌面應用程式，協助 NOC 維運人員集中管理網路設備、透過 SSH 即時或排程查詢日常維護狀態（CPU、Memory、module、interface、loading、traffic），於本地保存歷史資料，並由 AI（OpenAI 相容端點）產生易讀摘要與報告。

技術上採 Tauri 2 桌面殼層 + Vue 3/Vite 前端 + Rust 後端核心（SSH、排程、加密儲存、AI 整合），本地以 SQLite 持久化。前端設計依 `ui-ux-pro-max` 採「深色 (OLED) + 即時監控 + 資料密集儀表板」設計系統；依憲章以 zh-TW 撰寫文件與註解，並以 Playwright head 模式驗證前端。MVP 不引入 Redis／RabbitMQ／伺服器端資料庫與容器（桌面應用以本地 SQLite 儲存即可）。

## Technical Context

**Language/Version**: Rust 1.80+（後端核心 + Tauri 殼層）、TypeScript 5 + Vue 3（前端）

**Primary Dependencies**: Tauri 2.x、Vue 3 + Vite、Pinia、vue-router、Tailwind CSS、ApexCharts（vue3-apexcharts）、vue-virtual-scroller；Rust：tokio、russh（SSH）、sqlx（SQLite）、tokio-cron-scheduler（排程）、keyring（OS 金鑰庫）、aes-gcm（欄位加密）、reqwest（OpenAI 相容 API）、serde

**Storage**: 本地 SQLite（設備、群組／標籤、狀態快照、排程、報告）；設備密碼以 AES-256-GCM 加密，主金鑰存於 OS 金鑰庫（Keychain／Credential Manager）

**Testing**: Rust `cargo test`（單元 + 整合）；前端 Vitest（單元）；Playwright（E2E，依憲章以 head 模式）

**Target Platform**: 桌面 — 預設發行 macOS 與 Windows（Linux 可建置，非預設發行）

**Project Type**: 跨平台桌面應用（Tauri；frontend + Rust backend，以 Cargo workspace 組織）

**Performance Goals**: 設備清單以虛擬捲動支援 1000+ 筆並維持 ~60fps；一般 UI 互動 p95 < 200ms；單台查詢 < 30s（通常數秒）；SSH 併發具上限（預設 10）以保護本機與目標設備

**Constraints**: 設備密碼加密儲存、不得明文外露；除 AI（雲端時）與 SSH 連線外可離線運作；唯讀查詢、不對設備變更組態

**Scale/Scope**: ~100–1000 台設備；主要畫面約 6 個（設備清單、設備狀態詳情、群組／標籤、排程、報告、設定）

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

依憲章 v1.0.0（[constitution.md](../../.specify/memory/constitution.md)）逐項檢核：

- **I. 程式品質優先** ✅ 採 MVP 範圍，主動移除未用到的 Redis／RabbitMQ／伺服器資料庫／容器；以 Cargo workspace 清楚分層，無投機式抽象。
- **II. 測試標準** ✅ 核心邏輯（SSH 解析、加密、排程、AI）以 cargo test 覆蓋；前端 Vitest；前端流程以 Playwright head 模式驗證。
- **III. 使用者體驗一致性** ✅ 前端依 `ui-ux-pro-max` 產出之「深色 + 即時監控 + 資料密集」設計系統，狀態色（正常／警告／嚴重）一致。
- **IV. 效能要求** ✅ 已訂明確指標（虛擬捲動 60fps、p95 < 200ms、SSH 併發上限、增量儲存）並列於 Performance Goals。
- **V. MVP 與簡約優先** ✅ 僅實作需求所需；複雜的伺服器端服務與容器列為未來擴充，不納入 v1。
- **工具約束** ✅ 前端設計用 ui-ux-pro-max；需即時資訊時用 Felo-Search；前端測試用 playwright-cli head 模式；文件／註解 zh-TW。

**結果**：無違反，Complexity Tracking 留空。

## Project Structure

### Documentation (this feature)

```text
specs/001-network-device-monitoring/
├── plan.md              # 本檔（/speckit.plan 產出）
├── research.md          # Phase 0 產出
├── data-model.md        # Phase 1 產出
├── quickstart.md        # Phase 1 產出
├── contracts/           # Phase 1 產出（tauri-commands / brand-commands / ai-report）
└── tasks.md             # Phase 2 產出（/speckit.tasks，非本指令建立）
```

### Source Code (repository root)

```text
frontend/                 # Vue 3 + Vite + Tauri 前端
├── src/
│   ├── components/       # UI 元件（表格、狀態徽章、圖表卡片…）
│   ├── views/            # 畫面：DeviceList / DeviceDetail / Groups / Schedules / Reports / Settings
│   ├── stores/           # Pinia 狀態
│   ├── router/           # vue-router
│   ├── api/              # 封裝 Tauri invoke 呼叫（對應 IPC 契約）
│   └── styles/           # Tailwind + 設計 token（深色主題）
└── tests/                # Vitest 單元測試

src-tauri/                # Tauri 殼層（Rust）
├── src/
│   ├── commands/         # Tauri IPC 指令（前後端契約進入點）
│   └── main.rs
├── icons/                # 應用程式圖示
├── tauri.conf.json
└── Cargo.toml

backend/                  # Rust 核心函式庫（Cargo workspace 成員）
├── src/
│   ├── models/           # 領域模型（Device / Group / StatusSnapshot / ScheduledJob / Report）
│   ├── ssh/              # SSH 連線、品牌指令對應、輸出解析
│   ├── db/               # SQLite 存取（sqlx）與遷移
│   ├── crypto/           # 密碼加密與金鑰管理（keyring + aes-gcm）
│   ├── scheduler/        # 排程執行（tokio-cron-scheduler）
│   └── ai/               # OpenAI 相容端點整合、摘要／報告生成
└── tests/                # cargo 整合測試

e2e/                      # Playwright E2E（head 模式，依憲章）
infra/                    # 打包／發行輔助腳本（CI 使用）；MVP 不含伺服器端服務
docs/                     # 文件（zh-TW）
.github/workflows/        # CI：程式碼審查、CodeQL、跨平台建置與發行
```

**Structure Decision**: 採 **Cargo workspace** 組織三個程式碼根目錄——`frontend/`（Vue 前端）、`src-tauri/`（薄 Tauri 殼層，負責 IPC 指令與系統整合）、`backend/`（純 Rust 核心函式庫，承載 SSH／DB／加密／排程／AI 等領域邏輯）。如此既符合使用者偏好的「frontend／backend」分層，又保留 Tauri 慣用結構，並讓核心邏輯可獨立以 `cargo test` 驗證。

**容器與外部服務（MVP 範圍裁決）**：本桌面應用以本地 SQLite 持久化，**不需要** Redis（快取）、RabbitMQ（訊息佇列）、MariaDB／SqlServer（伺服器端資料庫）或 Docker 容器；依憲章原則 V（MVP、不過度設計）將其全數排除於 v1，並於 [research.md](./research.md) 記錄裁決理由。`infra/` 僅保留發行打包輔助。

**發行平台**：預設發行 macOS 與 Windows；GitHub Actions 以 `macos-latest` 與 `windows-latest` 矩陣建置並上傳至 Release 頁面（詳見 research.md「CI/CD」段）。

## Complexity Tracking

> 本功能 Constitution Check 無違反項目，無需記錄複雜度例外。
