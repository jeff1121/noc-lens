# 契約：品牌指令對應與狀態解析

**功能**：[../spec.md](../spec.md) ｜ **日期**：2026-06-24

各品牌以不同 CLI 指令取得相同指標。後端 `backend/src/ssh/` 依品牌套用對應指令並解析輸出為統一 `metrics_json`（見 [../data-model.md](../data-model.md)）。下表指令依各品牌官方「日常維護」文件選定（見 `docs/cisco-cmd.md`、`docs/aruba-cmd.md`、`docs/fortigate-cmd.md`、`docs/palo-alto-cmd.md`）；不適用者標示「n/a」（FR-010）。

> 安全：僅執行唯讀查詢指令，禁止任何組態變更（FR-009）。

## 指標 → 品牌指令對照

| 指標 | Cisco (IOS/IOS-XE/NX-OS) | Aruba (AOS-CX) | Fortigate-NGFW (FortiOS) | Palo Alto (PAN-OS) |
|------|--------------------------|----------------|--------------------------|--------------------|
| CPU | `show processes cpu sorted` | `show system resource-utilization` | `get system performance status` | `show system resources` |
| Memory | `show processes memory sorted` | `show system resource-utilization` | `get system performance status` | `show system resources` |
| module | `show inventory` | `show environment` | `get system status` | `show system environmentals` |
| interface | `show ip interface brief` | `show interface brief` | `get system interface physical` | `show interface all` |
| loading | `show processes cpu sorted` | `show system resource-utilization` | `get system performance status` | `show system resources` |
| traffic | `show interfaces counters` | `show interface` | `diagnose netlink interface list` | `show interface hardware` |

> 此對照已實作於 [../../../backend/src/ssh/commands.rs](../../../backend/src/ssh/commands.rs)。

## 解析輸出契約

- 每品牌一個 parser 模組：輸入原始 CLI 文字，輸出對應指標欄位。
- 解析失敗或欄位缺漏 → 該指標填 `"n/a"`，並於 snapshot `status` 視情況標 `partial`，不得回傳臆測數值（spec 邊界情況）。
- 可重用 `cisco-configer` 技能的 Cisco 指令知識作為 Cisco parser 的依據。

## 品牌列舉值（與 data-model 一致）

`cisco` ｜ `aruba` ｜ `fortigate_ngfw` ｜ `palo_alto`
