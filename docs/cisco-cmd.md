# Cisco 日常維護常用指令（Daily Maintenance CLI）

> 適用範圍：Cisco IOS / IOS-XE（Catalyst 交換器、ISR/ASR 路由器）與 NX-OS（Nexus 資料中心交換器）。
> 多數 `show` 指令在使用者模式即可執行；設定變更需進入 `configure terminal`。
> 慣例：`enable` 進入特權模式（提示字元 `#`），`configure terminal` 進入全域設定模式。

## 官方文件連結

| 文件 | 連結 | 驗證狀態 |
|---|---|---|
| Cisco IOS Configuration Fundamentals Command Reference | https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/fundamentals/command/cf_command_ref.html | 官方入口（curl 受 403 反爬蟲限制，瀏覽器可開啟） |
| Catalyst 9300 Command Reference | https://www.cisco.com/c/en/us/support/switches/catalyst-9300-series-switches/products-command-reference-list.html | 官方入口（同上 403） |
| Nexus 9000 Command Reference 清單 | https://www.cisco.com/c/en/us/support/switches/nexus-9000-series-switches/products-command-reference-list.html | 官方入口（同上 403） |
| Cisco IOS XE 17 系列文件總覽 | https://www.cisco.com/c/en/us/support/ios-nx-os-software/ios-xe-17/series.html | 官方入口（同上 403） |

> 註：Cisco 官網對無瀏覽器特徵的請求回應 HTTP 403，屬正常反爬蟲行為；上述為穩定的官方文件入口路徑，請以瀏覽器開啟並依裝置型號 / 軟體版本選擇對應 Command Reference。

---

## 一、系統資訊與版本

| 繁中說明 | 英文指令 |
|---|---|
| 顯示軟體版本、開機時間、型號、序號 | `show version` |
| 顯示系統運行時間與重啟原因 | `show version \| include uptime\|reason` |
| 顯示硬體庫存（模組、序號） | `show inventory` |
| 顯示目前環境（溫度、電源、風扇） | `show environment all` |
| 顯示 CPU 使用率 | `show processes cpu sorted` |
| 顯示記憶體使用狀況 | `show processes memory sorted` |
| （NX-OS）顯示系統資源 | `show system resources` |

## 二、設定檢視、儲存與備份

| 繁中說明 | 英文指令 |
|---|---|
| 檢視目前運行設定 | `show running-config` |
| 檢視已儲存的開機設定 | `show startup-config` |
| 只看某介面設定 | `show running-config interface Gi1/0/1` |
| 將運行設定存入開機設定（存檔） | `write memory` 或 `copy running-config startup-config` |
| 備份設定到 TFTP 伺服器 | `copy running-config tftp://192.0.2.10/backup.cfg` |
| 比對運行與開機設定差異 | `show archive config differences` |
| 顯示設定最後修改時間與人員 | `show running-config \| include Last configuration` |

## 三、介面與連線狀態

| 繁中說明 | 英文指令 |
|---|---|
| 顯示所有介面摘要狀態 | `show ip interface brief` |
| 顯示介面詳細（流量、錯誤、CRC） | `show interfaces` |
| 顯示單一介面狀態 | `show interfaces GigabitEthernet1/0/1` |
| 顯示介面錯誤計數摘要 | `show interfaces counters errors` |
| 顯示介面光功率 / SFP（DOM） | `show interfaces transceiver detail` |
| 清除介面計數器 | `clear counters` |
| 顯示介面被關閉（err-disabled）原因 | `show interfaces status err-disabled` |

## 四、二層：MAC、VLAN、STP、CDP/LLDP

| 繁中說明 | 英文指令 |
|---|---|
| 顯示 MAC 位址表 | `show mac address-table` |
| 顯示 VLAN 清單與埠對應 | `show vlan brief` |
| 顯示生成樹狀態（找 root、阻斷埠） | `show spanning-tree` |
| 顯示某 VLAN 的 STP | `show spanning-tree vlan 10` |
| 顯示相鄰 Cisco 裝置（CDP） | `show cdp neighbors detail` |
| 顯示相鄰裝置（標準 LLDP） | `show lldp neighbors detail` |
| 顯示 EtherChannel / Port-channel 狀態 | `show etherchannel summary` |

## 五、三層：路由、ARP、鄰居

| 繁中說明 | 英文指令 |
|---|---|
| 顯示路由表 | `show ip route` |
| 顯示某目的地如何被路由 | `show ip route 8.8.8.8` |
| 顯示 ARP 表 | `show ip arp` |
| 顯示 OSPF 鄰居 | `show ip ospf neighbor` |
| 顯示 BGP 摘要 | `show ip bgp summary` |
| 顯示 HSRP/VRRP 備援狀態 | `show standby brief` / `show vrrp brief` |

## 六、日誌與診斷

| 繁中說明 | 英文指令 |
|---|---|
| 顯示系統日誌 | `show logging` |
| 即時連線測試（Ping） | `ping 8.8.8.8` |
| 路徑追蹤 | `traceroute 8.8.8.8` |
| 延伸 Ping（指定來源、大小、次數） | `ping`（互動模式）→ 依提示輸入 |
| 顯示目前登入使用者 | `show users` |
| 顯示最近重大事件 | `show logging \| include %` |

## 七、韌體、授權與儲存

| 繁中說明 | 英文指令 |
|---|---|
| 顯示快閃記憶體內容（韌體檔） | `show flash:` |
| 顯示開機映像設定 | `show boot` 或 `show bootvar` |
| 顯示授權狀態（Smart Licensing） | `show license status` / `show license usage` |
| 顯示堆疊（Stack）成員狀態 | `show switch` |
| 驗證映像檔完整性 | `verify /md5 flash:image.bin` |

## 八、常見維護情境速查

- **接埠不通**：`show interfaces status` →（若 err-disabled）`show interfaces status err-disabled` → 修因後 `shutdown` / `no shutdown`。
- **找某 IP 接在哪個埠**：`show ip arp 192.0.2.50`（取得 MAC）→ `show mac address-table address xxxx.xxxx.xxxx`（取得埠）。
- **存檔再離線**：`write memory` 確認 `[OK]` 後再離開。
- **變更前先備份**：`copy running-config tftp://...` 或 `show running-config` 全文存底。

## 注意事項

1. **NX-OS 差異**：Nexus 平台部分指令不同，例如 `copy running-config startup-config` 同樣適用，但介面命名為 `Ethernet1/1`；功能需先 `feature <name>` 啟用。
2. **存檔習慣**：IOS 變更為立即生效但未存檔，務必 `write memory`，否則重開機後復原。
3. **分頁輸出**：長輸出可加 `| begin`、`| include`、`| exclude` 過濾；終止分頁按 `q`。
4. **唯讀優先**：日常巡檢以 `show` 指令為主，不影響運行；變更類指令請於維護時段並先備份。
