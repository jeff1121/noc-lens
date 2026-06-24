# 契約：AI 摘要與報告

**功能**：[../spec.md](../spec.md) ｜ **日期**：2026-06-24

`backend/src/ai/` 透過「OpenAI 相容端點」（base URL + 金鑰，FR-027）將狀態資料轉為可讀摘要。本檔定義輸入彙整與輸出契約。

## 輸入（後端彙整後送交模型）

由 `report_generate` 的 `scope` 取得對應 StatusSnapshot，後端彙整為精簡 JSON（避免送出明文密碼等敏感欄位）：

```json
{
  "range": { "from": "2026-06-20T00:00:00Z", "to": "2026-06-24T00:00:00Z" },
  "devices": [
    {
      "ip_address": "10.1.1.1",
      "brand": "cisco",
      "group_names": ["高雄三民區"],
      "latest": { "cpu": 23.5, "memory": 61.2, "interfaces_down": 1, "status": "ok" },
      "trend": { "cpu_max": 88.0, "memory_max": 75.0 }
    }
  ]
}
```

> 隱私：使用雲端端點時，上述設備狀態資料將離開本機；設定畫面須明示（FR-027）。**不得**包含密碼或金鑰。

## 輸出（模型回傳，後端存為 Report.summary_md）

- 格式：Markdown（zh-TW）。
- MUST 至少涵蓋（FR-019）：
  1. **整體健康概況**（正常／警告／嚴重設備數）。
  2. **異常與需關注項目**（高 CPU/Memory、介面 down、模組異常等，含設備識別）。
- SHOULD 包含趨勢觀察（針對期間報告，對應 US4 場景 2）。
- 可追溯性：摘要引用之設備／指標 MUST 能對應回 scope 內的實際資料（FR-022）。

## 錯誤行為

| 情境 | 行為 |
|------|------|
| 未設定端點／金鑰 | 回 `AI_CONFIG_MISSING`，導引至設定畫面 |
| 端點逾時／不可用／額度用盡 | 回 `AI_UNAVAILABLE`，明確提示；既有狀態資料不受影響（FR-021） |
| 回傳內容不完整 | 標示為部分結果，保留原始狀態資料供重試 |
