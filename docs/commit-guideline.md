# 提交訊息規範（noc-lens）

所有提交訊息 **使用繁體中文（zh-TW）**，並力求詳細、可追溯。

## 格式

```
<type>(<scope>): <簡述>

<本文：說明變更內容、理由與影響，可分多段或條列>
```

## type

| type | 用途 |
|------|------|
| feat | 新功能 |
| fix | 修正缺陷 |
| docs | 文件 |
| refactor | 重構（不改變行為） |
| test | 測試 |
| chore | 建置／工具／雜項 |
| perf | 效能 |
| style | 格式（不影響邏輯） |

## scope（選填）

對應模組或使用者故事，如 `us1`、`us2`、`backend`、`frontend`、`ci`。

## 範例

```
feat(us2): 實作 SSH 即時查詢設備狀態（多品牌）

- 新增 russh 客戶端與品牌指令對應。
- 查詢編排具併發上限，逐台回報。
- 前端 DeviceDetail 加入儀表與趨勢圖。
```

## 設定 commit template

```bash
git config commit.template .gitmessage
```
