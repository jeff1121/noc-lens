# Aruba（HPE Aruba Networking）日常維護常用指令（Daily Maintenance CLI）

> 適用範圍：AOS-CX（現行主力交換器，如 6300/8400 系列）為主，並標註 ArubaOS-Switch（舊稱 Provision，2930/3810 系列）差異。
> AOS-CX 語法與 Cisco IOS 高度相似但非完全相同。
> 慣例：登入後即特權層級，`configure terminal` 進入設定模式。

## 官方文件連結

| 文件 | 連結 | 驗證狀態 |
|---|---|---|
| Aruba AOS-CX Command Reference（10.13） | https://www.arubanetworks.com/techdocs/AOS-CX/10.13/HTML/cli_8400/ | 官方入口（curl 受 403 反爬蟲限制，瀏覽器可開啟） |
| HPE Aruba Networking 產品文件總入口 | https://www.arubanetworks.com/support-services/product-documentation/ | 官方入口（同上 403） |
| HPE Aruba Networking TechDocs | https://www.arubanetworks.com/techdocs/ | 官方入口（同上 403） |
| HPE 支援中心（韌體 / 授權） | https://support.hpe.com/ | 官方入口（同上 403） |

> 註：Aruba / HPE 官網對無瀏覽器特徵的請求回應 HTTP 403，屬正常反爬蟲行為。請以瀏覽器開啟 TechDocs，依 AOS-CX 版本（如 10.x）選擇對應機型的 Command Reference。

---

## 一、系統資訊與版本

| 繁中說明 | 英文指令 |
|---|---|
| 顯示軟體版本與型號 | `show version` |
| 顯示系統摘要（運行時間、序號） | `show system` |
| 顯示硬體資源使用（CPU / 記憶體） | `show system resource-utilization` |
| 顯示環境（溫度、電源、風扇） | `show environment` |
| 顯示電源供應器狀態 | `show environment power-supply` |
| 顯示風扇狀態 | `show environment fan` |

## 二、設定檢視、儲存與備份

| 繁中說明 | 英文指令 |
|---|---|
| 檢視目前運行設定 | `show running-config` |
| 檢視已儲存的開機設定 | `show startup-config` |
| 只看某介面設定 | `show running-config interface 1/1/1` |
| 儲存設定（存檔） | `write memory` 或 `copy running-config startup-config` |
| 備份設定到 TFTP | `copy running-config tftp://192.0.2.10/backup.cfg` |
| 顯示設定檢查點（checkpoint）清單 | `show checkpoint` |
| 比對兩個檢查點差異 | `checkpoint diff <name1> <name2>` |

## 三、介面與連線狀態

| 繁中說明 | 英文指令 |
|---|---|
| 顯示所有介面簡表（連線/速率） | `show interface brief` |
| 顯示單一介面詳細 | `show interface 1/1/1` |
| 顯示介面流量統計 | `show interface 1/1/1 extended` |
| 顯示介面光模組（DOM / transceiver） | `show interface transceiver detail` |
| 顯示 PoE 供電狀態 | `show power-over-ethernet` |
| 清除介面統計 | `clear interface 1/1/1 statistics` |

## 四、二層：MAC、VLAN、STP、LLDP

| 繁中說明 | 英文指令 |
|---|---|
| 顯示 MAC 位址表 | `show mac-address-table` |
| 顯示 VLAN 清單 | `show vlan` |
| 顯示某 VLAN 詳細 | `show vlan 10` |
| 顯示生成樹狀態 | `show spanning-tree` |
| 顯示 STP 詳細（含 root） | `show spanning-tree detail` |
| 顯示 LLDP 鄰居 | `show lldp neighbor-info` |
| 顯示鏈路聚合（LAG）狀態 | `show lacp interfaces` |

## 五、三層：路由、ARP、鄰居

| 繁中說明 | 英文指令 |
|---|---|
| 顯示路由表 | `show ip route` |
| 顯示 ARP 表 | `show arp` |
| 顯示 IP 介面狀態 | `show ip interface brief` |
| 顯示 OSPF 鄰居 | `show ospf neighbors` |
| 顯示 BGP 摘要 | `show bgp ipv4 unicast summary` |
| 顯示 VRRP 備援狀態 | `show vrrp` |
| 顯示 VSX 雙機狀態（資料中心） | `show vsx status` |

## 六、日誌與診斷

| 繁中說明 | 英文指令 |
|---|---|
| 顯示系統日誌 | `show logging -r`（`-r` 為最新在前） |
| 顯示某事件等級日誌 | `show events` |
| 即時連線測試（Ping） | `ping 8.8.8.8` |
| 路徑追蹤 | `traceroute 8.8.8.8` |
| 顯示目前登入者 | `show users` |
| 即時介面除錯（謹慎使用） | `diagnostic`（依機型） |

## 七、韌體、授權與堆疊

| 繁中說明 | 英文指令 |
|---|---|
| 顯示已安裝韌體映像（主/備分區） | `show images` |
| 顯示開機分區設定 | `show boot` |
| 顯示授權狀態 | `show license` |
| 顯示 VSF 堆疊成員（前端堆疊） | `show vsf` |
| 顯示 LED 定位燈狀態 | `show led-locator` |

## 八、常見維護情境速查

- **接埠不通**：`show interface brief` 看 admin/oper 狀態 →（必要時）進設定 `interface 1/1/1` 後 `shutdown` / `no shutdown`。
- **存檔再離線**：`write memory` 確認成功訊息後再離開。
- **變更前建檢查點**：`copy running-config checkpoint <name>`，出錯可 `rollback checkpoint <name>` 快速復原（AOS-CX 特色）。
- **找某 IP 在哪個埠**：`show arp`（取 MAC）→ `show mac-address-table`（取埠）。

## 注意事項

1. **AOS-CX vs ArubaOS-Switch**：本檔以 **AOS-CX** 為主。舊款 **ArubaOS-Switch（2930/3810）** 語法不同，例如：版本用 `show system`、設定儲存同為 `write memory`、介面顯示用 `show interfaces brief`，鄰居用 `show lldp info remote-device`。請先確認機型與作業系統。
2. **介面命名**：AOS-CX 採 `member/slot/port`（如 `1/1/1`）。
3. **檢查點（Checkpoint）與回滾**：AOS-CX 內建設定檢查點與一鍵 `rollback`，是日常維護變更前的最佳保險，善加利用。
4. **唯讀優先**：日常巡檢以 `show` 為主；變更請於維護時段並先建立檢查點 / 備份。
