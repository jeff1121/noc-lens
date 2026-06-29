import { defineConfig } from "@playwright/test";

// noc-lens 前端 E2E（本機以 head 模式執行，CI 使用 headless）。
// 透過 Vite alias 使用 Tauri IPC mock，覆蓋主要使用者流程。
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
    env: { VITE_E2E_MOCK_TAURI: "1" },
    url: "http://localhost:5173",
    reuseExistingServer: !isCi,
    timeout: 60_000,
  },
});
