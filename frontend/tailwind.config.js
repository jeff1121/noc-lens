/** noc-lens 設計系統（依 ui-ux-pro-max：深色 OLED + 即時監控 + 資料密集儀表板） */
/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  theme: {
    extend: {
      colors: {
        // 背景層次（深色）
        bg: {
          base: "#0A0E27", // 主背景（午夜藍）
          surface: "#121826", // 卡片／面板
          raised: "#1E293B", // 浮起元素／hover
          border: "#293548",
        },
        // 文字
        ink: {
          primary: "#E2E8F0",
          secondary: "#94A3B8",
          muted: "#64748B",
        },
        // 主色（信任藍）
        brand: {
          DEFAULT: "#3B82F6",
          soft: "#60A5FA",
        },
        // 狀態語意色
        status: {
          critical: "#EF4444",
          warning: "#F59E0B",
          normal: "#22C55E",
          updating: "#3B82F6",
        },
      },
      fontFamily: {
        // 數據／標題用等寬，內文用 sans
        mono: ["Fira Code", "ui-monospace", "monospace"],
        sans: ["Fira Sans", "ui-sans-serif", "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [],
};
