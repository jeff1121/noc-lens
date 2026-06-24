import { defineConfig } from "@playwright/test";

// noc-lens 前端 E2E（依憲章以 head 模式執行）。
// 對前端 dev server 進行煙霧測試；涉及 Tauri IPC 的資料流需在桌面應用內驗證。
export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  use: {
    baseURL: "http://localhost:5173",
    headless: false, // head 模式
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    command: "npm --prefix ../frontend run dev",
    url: "http://localhost:5173",
    reuseExistingServer: true,
    timeout: 60_000,
  },
});
