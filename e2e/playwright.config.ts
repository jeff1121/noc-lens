import { defineConfig } from "@playwright/test";

// noc-lens 前端 E2E（本機以 head 模式執行，CI 使用 headless）。
// 對前端 dev server 進行煙霧測試；涉及 Tauri IPC 的資料流需在桌面應用內驗證。
const isCi = process.env.CI === "true";

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  use: {
    baseURL: "http://localhost:5173",
    headless: isCi ? true : false,
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    command: "npm --prefix ../frontend run dev",
    url: "http://localhost:5173",
    reuseExistingServer: !isCi,
    timeout: 60_000,
  },
});
