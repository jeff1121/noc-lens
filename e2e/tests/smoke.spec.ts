import { test, expect } from "@playwright/test";

// 煙霧測試：前端外殼與導覽可正常渲染（head 模式）。
// 註：實際設備資料流程（匯入/查詢/排程/報告）依賴 Tauri IPC，
// 需於桌面應用（cargo tauri dev）內以 head 模式驗證，對應 US1–US4 的獨立測試情境。
test("側邊導覽與主要畫面可渲染", async ({ page }) => {
  await page.goto("/");

  // 標題
  await expect(page.getByText("noc-lens")).toBeVisible();

  // 導覽項目
  for (const label of ["設備清單", "群組／標籤", "排程監控", "AI 報告", "設定"]) {
    await expect(page.getByRole("link", { name: label })).toBeVisible();
  }

  // 切換到設定畫面
  await page.getByRole("link", { name: "設定" }).click();
  await expect(page.getByText("OpenAI 相容端點")).toBeVisible();
});
