# Phase 0 研究：noc-lens 技術決策

**功能**：[spec.md](./spec.md) ｜ **計畫**：[plan.md](./plan.md) ｜ **日期**：2026-06-24

本文件彙整實作前的關鍵技術選型，每項以「決策 / 理由 / 已評估之替代方案」呈現。

---

## 1. 桌面框架：Tauri 2

- **決策**：採 Tauri 2.x 作為跨平台桌面殼層，前端 WebView + Rust 後端。
- **理由**：產出體積小、記憶體佔用低；後端原生 Rust，可直接整合 SSH／加密／排程；官方支援 macOS／Windows 打包（.dmg/.app、.msi/.exe）。符合使用者指定（Tauri + Vue）。
- **替代方案**：Electron（體積大、需 Node 後端，不利於原生 SSH 與資源佔用）；原生各平台（開發成本高，違反跨平台需求）。

## 2. 前端堆疊：Vue 3 + Vite + Pinia + vue-router + Tailwind

- **決策**：Vue 3 `<script setup>` Composition API、Vite 建置、Pinia 狀態、vue-router 路由、Tailwind CSS（深色主題）。
- **理由**：使用者指定 Vue；Composition API 對 TypeScript 與邏輯重用友善；Pinia 為 Vue 3 官方狀態管理；Tailwind 利於落實設計 token 與深色模式一致性。
- **替代方案**：Options API（新專案不建議）、Vuex（已被 Pinia 取代）。

## 3. SSH 連線函式庫（Rust）：russh

- **決策**：採 `russh`（純 Rust 非同步 SSH 客戶端）以帳號／密碼認證連線並執行查詢指令。
- **理由**：純 Rust、無 C 相依，跨平台（macOS/Windows）交叉編譯與打包單純；非同步契合 tokio，可控制併發。
- **替代方案**：`ssh2`（libssh2 C 綁定，跨平台打包較麻煩）；外部 `ssh` CLI（依賴系統環境、解析脆弱）。

## 4. 本地資料庫：SQLite（sqlx）

- **決策**：以 SQLite 透過 `sqlx`（SQLite、非同步、編譯期查詢檢查）持久化設備、群組／標籤、狀態快照、排程、報告。
- **理由**：關聯式模型適合設備／群組／快照；單檔、零維運、跨平台；時間序列查詢量於 MVP（100–1000 台）足夠；sqlx 非同步契合 tokio。
- **替代方案**：DuckDB（OLAP 分析型，對 MVP 的交易式 CRUD 過度設計，列為未來重度歷史分析的擴充）；rusqlite（同步、需自管連線池）。

## 5. 認證憑證安全儲存：OS 金鑰庫 + AES-256-GCM 欄位加密

- **決策**：以 `keyring` crate 於 OS 金鑰庫（macOS Keychain／Windows Credential Manager）保存單一「主金鑰」；設備密碼以 AES-256-GCM 加密後存於 SQLite 密碼欄位。
- **理由**：避免明文（OWASP A02 加密失誤）；主金鑰不落地於檔案；單一主金鑰避免為數百台設備建立大量金鑰庫項目造成效能與上限問題。
- **替代方案**：明文儲存（違反 FR-023，禁止）；每台設備一個 keyring 項目（數量大時效能差、平台上限風險）。

## 6. 排程器：tokio-cron-scheduler

- **決策**：後端以 `tokio-cron-scheduler` 提供「固定間隔」與「每日固定時間」兩種 MVP 排程模式。
- **理由**：與 tokio 整合、輕量；涵蓋 MVP 常見需求（FR-013/014）。
- **替代方案**：手刻 tokio interval（功能不足、需自行處理每日時點）；完整 Cron 表達式（列未來擴充，避免過度設計）。

## 7. AI 整合：OpenAI 相容 API 端點（解析 FR-027）

- **決策**：以 `reqwest` 呼叫「可設定的 OpenAI 相容端點」（base URL + API 金鑰）。預設可指向雲端（OpenAI／Azure OpenAI），亦可指向本地（Ollama／LM Studio）使資料不離開本機。
- **理由**：單一介面同時滿足雲端與本地兩種隱私情境，避免重複實作（符合 MVP）；設定畫面標示雲端隱私影響（FR-027）。
- **替代方案**：綁定單一雲端 SDK（無法本地、隱私受限）；內嵌本地推論引擎（增加體積與運算需求，超出 MVP）。

## 8. 圖表：ApexCharts（vue3-apexcharts）

- **決策**：採 ApexCharts。CPU／Memory 使用率以 Gauge／Bullet；CPU／Memory／traffic 趨勢以 Line（多序列）；即時更新以 Streaming Area。
- **理由**：ui-ux-pro-max 圖表建議（趨勢→Line、效能對目標→Gauge/Bullet、即時→Streaming Area）；Vue 整合佳、體積較 D3 輕、互動（hover/zoom）開箱即用。
- **替代方案**：D3.js（彈性高但成本高）；Chart.js（可行，惟 Gauge 需外掛）。

## 9. 前端設計系統（依 ui-ux-pro-max）

- **風格**：深色 (OLED) 基底 + 即時監控 + 資料密集儀表板。
- **狀態色彩語意**：嚴重 `#EF4444`(critical/red)、警告 `#F59E0B`(warning/orange)、正常 `#22C55E`(normal/green)、更新中 `#3B82F6`(blue，可帶脈動動畫)。主色信任藍 `#3B82F6`。
- **字體**：標題／數據 `Fira Code`（等寬，利於對齊數值）、內文 `Fira Sans`。
- **效果與 UX**：載入用 skeleton／spinner（>300ms）；大型清單（1000+ 設備）以虛擬捲動；表格支援多選與批次操作、橫向捲動容器；hover 過場 150–300ms；focus 可見；尊重 `prefers-reduced-motion`；對比 ≥ 4.5:1；圖示用 SVG（Lucide/Heroicons），不用 emoji。
- **理由**：NOC 維運需長時間注視、快速辨識異常，深色 + 明確告警色 + 資料密集最契合（憲章原則 III）。

## 10. 大型清單虛擬化：vue-virtual-scroller

- **決策**：設備清單與歷史快照清單以 `vue-virtual-scroller`（或 TanStack Virtual）虛擬化渲染。
- **理由**：達成效能目標（1000+ 筆 ~60fps，憲章原則 IV）。
- **替代方案**：分頁（互動較差）；一次渲染全部（大量 DOM 卡頓）。

## 11. 品牌指令對應與解析

- **決策**：在 `backend/src/ssh/` 為各品牌（Cisco、Aruba、Fortigate-ngfw、Palo Alto Networks）建立指令對應表與輸出解析器，將原始 CLI 文字轉為結構化指標（CPU/Memory/module/interface/loading/traffic）。不適用之指標標示「不適用」（FR-010）。指令對應細節見 [contracts/brand-commands.md](./contracts/brand-commands.md)。
- **理由**：各品牌 CLI 差異大，需可維護的對應層；可參考既有 `cisco-configer` 技能之 Cisco 指令知識。
- **替代方案**：單一通用指令（不可行，各廠命令不同）。

## 12. SSH 併發控制

- **決策**：以 tokio `Semaphore` 限制同時連線數（預設 10，可設定），逐台回報成功／失敗，單台失敗不中斷其他（FR-011）。
- **理由**：保護本機資源與目標設備，符合效能與穩定需求。

## 13. 測試策略

- **決策**：Rust 核心以 `cargo test`（解析器、加密、排程邏輯以單元測試；DB 以整合測試）；前端以 Vitest；關鍵使用者流程以 Playwright **head 模式** E2E（依憲章原則 II 與工具約束）。
- **理由**：對應四個使用者故事的獨立可驗證性。

## 14. CI/CD 與 GitHub 自動化

- **儲存庫**：以 `gh` 建立 GitHub 儲存庫（屬實作階段、需使用者授權後執行）。
- **版本徽章**：README 置入版本 badge（shields.io，對應最新 Release tag）。
- **Release 頁面**：發佈時上傳 macOS／Windows 安裝檔至 GitHub Releases。
- **GitHub Actions 工作流程**：
  1. **程式碼審查**：對 PR 進行審查、修正問題、彙整摘要並產生報告（**zh-TW**）。
  2. **CodeQL 安全掃描**：執行 CodeQL，修正問題、彙整摘要並產生報告（**zh-TW**）。
  3. **建置／發行**：以 `macos-latest` + `windows-latest` 矩陣建置並發佈 Release。
  - **提交訊息**：需詳細且以 **zh-TW** 撰寫。
- **理由**：對應使用者明確要求；報告與提交訊息統一繁體中文（憲章工具約束）。

## 15. MVP 範圍裁決：排除的服務

- **決策**：v1 **不**引入 Redis、RabbitMQ、MariaDB、SqlServer、Docker 容器與 push.sh 推送流程。
- **理由**：本產品為單機桌面應用、以本地 SQLite 儲存，無伺服器端與跨服務通訊需求；引入上述將違反憲章原則 V（MVP、不過度設計）。若未來新增雲端同步／多人協作再行評估。
