import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath } from "node:url";

// Tauri 期望固定的開發伺服器埠號
const useE2eTauriMock = process.env.VITE_E2E_MOCK_TAURI === "1";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  resolve: {
    alias: useE2eTauriMock
      ? {
          "@tauri-apps/api/core": fileURLToPath(
            new URL("./src/e2e/tauri-core-mock.ts", import.meta.url),
          ),
        }
      : {},
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: "es2021",
    sourcemap: false,
  },
});
