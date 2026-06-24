import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 期望固定的開發伺服器埠號
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: "es2021",
    sourcemap: false,
  },
});
