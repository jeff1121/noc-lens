# FortiGate（FortiOS）日常維護常用指令（Daily Maintenance CLI）

> 適用範圍：Fortinet FortiGate 防火牆，FortiOS 7.x。
> FortiOS CLI 分兩大模式：`get` / `show` / `diagnose` / `execute`（操作）與 `config`（設定）。
> 慣例：`config` 進入設定樹，`end` 儲存離開、`abort` 放棄離開。多 VDOM 環境需先 `config vdom` → `edit <vdom>`。

## 官方文件連結

| 文件 | 連結 | 驗證狀態 |
|---|---|---|
| Fortinet Document Library（總入口） | https://docs.fortinet.com/ | ✓ 已驗證可達（HTTP 200） |
| FortiOS 7.4 CLI Reference | https://docs.fortinet.com/document/fortigate/7.4.0/cli-reference | ✓ 已驗證可達（HTTP 200） |
| FortiGate 7.4 Administration Guide | https://docs.fortinet.com/document/fortigate/7.4.0/administration-guide | ✓ 已驗證可達（HTTP 200） |

> 註：以上連結已用 curl 實測回應 HTTP 200，可直接於瀏覽器開啟。請依實際 FortiOS 版本（7.2 / 7.4 / 7.6）切換文件版本。

---

## 一、系統資訊與版本

| 繁中說明 | 英文指令 |
|---|---|
| 顯示系統狀態（版本、序號、運行時間） | `get system status` |
| 顯示系統效能（CPU、記憶體、連線數） | `get system performance status` |
| 顯示即時資源使用（類似 top） | `diagnose sys top` |
| 顯示記憶體 conserve 模式狀態 | `diagnose hardware sysinfo memory` |
| 顯示硬體溫度 / 風扇 / 電源 | `execute sensor detail` |
| 顯示開機時間與重啟原因 | `diagnose debug crashlog read`（檢視當機紀錄） |

## 二、設定檢視與備份

| 繁中說明 | 英文指令 |
|---|---|
| 顯示完整運行設定 | `show full-configuration` |
| 顯示精簡設定（非預設值） | `show` |
| 顯示某段設定（如防火牆政策） | `show firewall policy` |
| 備份設定到 TFTP | `execute backup config tftp <檔名> <TFTP IP>` |
| 備份設定到 USB | `execute backup config usb <檔名>` |
| 還原設定 | `execute restore config tftp <檔名> <TFTP IP>` |
| 顯示設定修訂版本 | `diagnose sys config-revision list` |

## 三、介面與連線狀態

| 繁中說明 | 英文指令 |
|---|---|
| 顯示所有介面 IP 與狀態 | `get system interface physical` |
| 顯示介面摘要（流量速率） | `diagnose netlink interface list` |
| 顯示某介面流量統計 | `diagnose hardware deviceinfo nic <介面名>` |
| 顯示 DHCP 租約清單 | `execute dhcp lease-list` |
| 偵測介面連線（Ping） | `execute ping 8.8.8.8` |
| 指定來源介面 Ping | `execute ping-options source <IP>` 後再 `execute ping <目標>` |
| 路徑追蹤 | `execute traceroute 8.8.8.8` |

## 四、路由與工作階段（Session）

| 繁中說明 | 英文指令 |
|---|---|
| 顯示路由表 | `get router info routing-table all` |
| 顯示某目的地路由 | `get router info routing-table details 8.8.8.8` |
| 顯示 ARP 表 | `get system arp` |
| 顯示即時工作階段數量 | `diagnose sys session stat` |
| 過濾並檢視特定工作階段 | `diagnose sys session filter dst 8.8.8.8` → `diagnose sys session list` |
| 清除工作階段過濾器 | `diagnose sys session filter clear` |
| 顯示 BGP / OSPF 鄰居 | `get router info bgp summary` / `get router info ospf neighbor` |

## 五、安全功能與 VPN

| 繁中說明 | 英文指令 |
|---|---|
| 顯示 IPsec VPN 通道狀態 | `get vpn ipsec tunnel summary` |
| 顯示 SSL VPN 連線使用者 | `get vpn ssl monitor` |
| 顯示防火牆政策命中計數 | `show firewall policy`（搭配 GUI 計數） |
| 顯示 UTM / IPS 即時事件 | `diagnose ips anomaly list` |
| 顯示防毒 / 簽章更新狀態 | `diagnose autoupdate versions` |
| 顯示 FortiGuard 連線狀態 | `diagnose debug rating` |

## 六、日誌與診斷

| 繁中說明 | 英文指令 |
|---|---|
| 顯示日誌設定 | `get log setting` |
| 即時檢視記憶體日誌 | `execute log filter category <類別>` → `execute log display` |
| 啟用即時除錯（流量診斷） | `diagnose debug flow`（需設 filter 後 `diagnose debug enable`） |
| 顯示 HA 高可用狀態 | `get system ha status` |
| 顯示 HA 成員健康 | `diagnose sys ha status` |
| 關閉所有除錯 | `diagnose debug disable` + `diagnose debug reset` |

## 七、韌體、授權與重啟

| 繁中說明 | 英文指令 |
|---|---|
| 顯示授權 / 合約狀態 | `get system fortiguard-service status` |
| 升級韌體（TFTP） | `execute restore image tftp <檔名> <TFTP IP>` |
| 顯示韌體分區 | `diagnose sys flash list` |
| 重新啟動裝置 | `execute reboot` |
| 關機 | `execute shutdown` |

## 八、常見維護情境速查

- **流量不通排查**：`diagnose debug flow` 是 FortiGate 最強排錯工具 — 設來源/目的 filter →`diagnose debug enable` → 觀察封包被哪條政策 / 路由 / NAT 處理 → 完成後務必 `diagnose debug disable`。
- **記憶體告警（conserve mode）**：`diagnose sys top` 找出佔用程序 → `get system performance status` 確認記憶體百分比。
- **變更前備份**：`execute backup config tftp ...`，FortiOS 也會自動保留 config-revision，可 `diagnose sys config-revision` 檢視。
- **VPN 不通**：`get vpn ipsec tunnel summary` 看通道是否 up，再 `diagnose debug flow` 追封包。

## 注意事項

1. **設定即時生效**：FortiOS 在 `config` 區塊 `end` 時即套用並自動儲存，**無需另存**；放棄變更用 `abort`。
2. **除錯務必收尾**：`diagnose debug flow` / `diagnose debug enable` 會持續輸出並耗資源，排錯後一定要 `diagnose debug disable` + `diagnose debug reset`。
3. **多 VDOM**：若啟用 VDOM，全域指令需先 `config global`，VDOM 內指令需先 `config vdom` → `edit <vdom>`。
4. **唯讀優先**：`get` / `show` / `diagnose ... stat` 類為唯讀；`execute reboot` / `shutdown` / `restore` 具高風險，請於維護時段操作並先備份。
