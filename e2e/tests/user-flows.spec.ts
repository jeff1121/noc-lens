import { expect, test } from "@playwright/test";

async function importCsv(page: any, content: string) {
  await page.getByRole("button", { name: "匯入 CSV" }).click();
  await page.locator('input[type="file"]').setInputFiles({
    name: "devices.csv",
    mimeType: "text/csv",
    buffer: Buffer.from(content),
  });
  await expect(page.getByText(/成功：/)).toBeVisible();
  await page.getByRole("button", { name: "完成" }).click();
}

test("US1/US2：匯入設備、指派群組、篩選與多選查詢", async ({ page }) => {
  await page.goto("/");
  await importCsv(
    page,
    [
      "ip_address,username,password,note,brand,groups",
      "10.10.0.1,admin,secret,核心交換器,cisco,核心節點",
      "10.10.0.2,admin,secret,備援防火牆,fortigate_ngfw,邊界節點",
    ].join("\n"),
  );

  await expect(page.getByText("10.10.0.1")).toBeVisible();
  await expect(page.getByText("10.10.0.2")).toBeVisible();

  await page.getByRole("link", { name: "群組／標籤" }).click();
  await page
    .getByRole("combobox")
    .selectOption({ label: "10.10.0.1（admin）" });
  await expect(page.getByLabel("核心節點")).toBeChecked();
  await page.getByRole("button", { name: "儲存指派" }).click();
  await expect(page.getByText("群組指派已儲存")).toBeVisible();

  await page.getByRole("link", { name: "設備清單" }).click();
  await page.getByRole("combobox").selectOption({ label: "核心節點" });
  await expect(page.getByText("10.10.0.1")).toBeVisible();
  await expect(page.getByText("10.10.0.2")).toHaveCount(0);

  await page.locator('input[type="checkbox"]').nth(1).check();
  await page.getByRole("button", { name: "查詢已選 1 台" }).click();
  await expect(page.getByText("正常")).toBeVisible();
  await expect(page.getByText(/CPU \d+%/)).toBeVisible();
});

test("US3：建立排程、立即執行並產生歷史快照", async ({ page }) => {
  await page.goto("/");
  await importCsv(
    page,
    [
      "ip_address,username,password,note,brand,groups",
      "10.20.0.1,admin,secret,巡檢設備,cisco,巡檢群組",
    ].join("\n"),
  );

  await page.getByRole("link", { name: "排程監控" }).click();
  await page.getByPlaceholder("排程名稱（如：每日巡檢）").fill("E2E 巡檢");
  const form = page.locator("section").filter({ hasText: "新增排程" });
  await form.locator("select").nth(0).selectOption("group");
  await form.locator("select").nth(1).selectOption({ label: "巡檢群組" });
  await form.locator("select").nth(2).selectOption("interval");
  await form.locator('input[type="number"]').fill("15");
  await page.getByRole("button", { name: "新增排程" }).click();
  await expect(page.getByText("E2E 巡檢")).toBeVisible();

  await page.getByRole("button", { name: "立即執行" }).click();
  await expect(page.getByText("最近執行：")).toBeVisible();
  await expect(page.getByText(/成功\s+1/)).toBeVisible();

  await page.getByRole("link", { name: "設備清單" }).click();
  await page.getByText("10.20.0.1").click();
  await expect(page.getByText(/CPU 歷史趨勢/)).toBeVisible();
});

test("US4：儲存 AI 設定、產生報告並匯出", async ({ page }) => {
  await page.goto("/");
  await importCsv(
    page,
    [
      "ip_address,username,password,note,brand,groups",
      "10.30.0.1,admin,secret,報告設備,cisco,報告群組",
    ].join("\n"),
  );

  await page.getByRole("link", { name: "設定" }).click();
  await page.getByLabel("Base URL").fill("https://api.example.test/v1");
  await page.getByLabel("模型").fill("mock-model");
  await page.getByLabel(/API 金鑰/).fill("sk-e2e");
  await page.getByRole("button", { name: "儲存金鑰" }).click();
  await expect(page.getByText("已儲存")).toBeVisible();
  await page.getByRole("button", { name: "儲存設定" }).click();
  await expect(page.getByText("已儲存", { exact: true })).toBeVisible();

  await page.getByRole("link", { name: "AI 報告" }).click();
  await page.getByPlaceholder("報告標題（選填）").fill("E2E 維運摘要");
  await page.getByRole("button", { name: "產生報告" }).click();
  await expect(
    page.getByRole("heading", { name: "E2E 維運摘要" }),
  ).toBeVisible();
  await expect(page.getByText("設備健康摘要")).toBeVisible();

  page.once("dialog", (dialog) => dialog.accept("/tmp/noc-lens-e2e-report.md"));
  await page.getByRole("button", { name: "匯出 Markdown" }).click();
  await expect(
    page.getByText("已匯出：/tmp/noc-lens-e2e-report.md"),
  ).toBeVisible();
});
