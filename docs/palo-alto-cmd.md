# Palo Alto Networks（PAN-OS）日常維護常用指令（Daily Maintenance CLI）

> 適用範圍：Palo Alto Networks 次世代防火牆，PAN-OS 11.x。
> PAN-OS CLI 分兩種模式：**Operational mode**（操作，提示字元 `>`）與 **Configuration mode**（設定，輸入 `configure` 進入，提示字元 `#`）。
> 慣例：設定模式以 `commit` 才會生效；`exit` 回操作模式。

## 官方文件連結

| 文件 | 連結 | 驗證狀態 |
|---|---|---|
| Palo Alto Networks TechDocs（總入口） | https://docs.paloaltonetworks.com/ | ✓ 已驗證可達（HTTP 200） |
| PAN-OS 11.1 CLI Quick Start | https://docs.paloaltonetworks.com/pan-os/11-1/pan-os-cli-quick-start | ✓ 已驗證可達（HTTP 200） |
| PAN-OS CLI Command Reference | https://docs.paloaltonetworks.com/pan-os/11-1/pan-os-cli-quick-start/use-the-cli/cli-command-reference | ✓ 入口已驗證（依 TechDocs 導覽至 CLI 指令參考） |

> 註：以上連結已用 curl 實測回應 HTTP 200，可直接於瀏覽器開啟。請依實際 PAN-OS 版本（10.2 / 11.0 / 11.1 / 11.2）切換文件版本。

---

## 一、系統資訊與版本

| 繁中說明 | 英文指令 |
|---|---|
| 顯示系統資訊（版本、序號、型號、運行時間） | `show system info` |
| 顯示系統資源使用（類似 top） | `show system resources` |
| 持續監看系統資源 | `show system resources follow` |
| 顯示系統環境（電源、風扇、溫度） | `show system environmentals` |
| 顯示磁碟使用量 | `show system disk-space` |
| 顯示各管理程序狀態 | `show system software status` |

## 二、設定檢視、儲存與備份

| 繁中說明 | 英文指令 |
|---|---|
| 顯示目前運行設定（操作模式） | `show config running` |
| 顯示候選設定差異（設定模式） | `show config diff`（需先 `configure`） |
| 套用設定變更（生效） | `commit` |
| 驗證設定是否有誤（不套用） | `commit force`／先 `validate full` |
| 匯出設定檔到 SCP/TFTP | `scp export configuration from running-config.xml to <user@host:path>` |
| 顯示設定稽核（誰改了什麼） | `show config audit`（或於 GUI Config Audit） |
| 還原到先前設定版本 | `load config version <版本號>` → `commit` |

## 三、介面與連線狀態

| 繁中說明 | 英文指令 |
|---|---|
| 顯示所有介面狀態 | `show interface all` |
| 顯示單一介面詳細 | `show interface ethernet1/1` |
| 顯示硬體介面計數（錯誤/丟包） | `show interface hardware` |
| 連線測試（Ping） | `ping host 8.8.8.8` |
| 指定來源 Ping | `ping source <IP> host 8.8.8.8` |
| 路徑追蹤 | `traceroute host 8.8.8.8` |
| 顯示 ARP 表 | `show arp all` |

## 四、路由與工作階段（Session）

| 繁中說明 | 英文指令 |
|---|---|
| 顯示路由表 | `show routing route` |
| 顯示某 FIB 轉送 | `show routing fib` |
| 顯示即時工作階段數與資訊 | `show session info` |
| 依條件過濾工作階段 | `show session all filter destination 8.8.8.8` |
| 顯示單一工作階段詳細 | `show session id <session-id>` |
| 顯示 BGP / OSPF 鄰居 | `show routing protocol bgp summary` / `show routing protocol ospf neighbor` |

## 五、安全政策、威脅與 VPN

| 繁中說明 | 英文指令 |
|---|---|
| 測試某流量會命中哪條安全政策 | `test security-policy-match source <IP> destination <IP> destination-port <埠> protocol <號>` |
| 顯示 IPsec VPN 通道狀態 | `show vpn ipsec-sa` |
| 顯示 GlobalProtect 連線使用者 | `show global-protect-gateway current-user` |
| 顯示威脅 / 內容簽章版本 | `show system info \| match version` |
| 顯示動態更新排程 | `request system software check`（檢查更新） |
| 顯示 User-ID 對應 | `show user ip-user-mapping all` |

## 六、日誌與診斷

| 繁中說明 | 英文指令 |
|---|---|
| 即時檢視流量日誌 | `show log traffic direction equal backward` |
| 即時檢視威脅日誌 | `show log threat direction equal backward` |
| 檢視系統日誌 | `show log system direction equal backward` |
| 顯示管理面 CPU/程序 | `show system state filter sys.monitor.*`（進階） |
| 顯示 HA 高可用狀態 | `show high-availability state` |
| 顯示 HA 同步狀態 | `show high-availability all` |

## 七、韌體、授權與重啟

| 繁中說明 | 英文指令 |
|---|---|
| 顯示授權狀態 | `request license info` |
| 顯示已安裝 PAN-OS 軟體版本 | `show system software status` |
| 檢查可用更新 | `request system software check` |
| 下載指定版本 | `request system software download version <版本>` |
| 重新啟動系統 | `request restart system` |
| 重啟資料面（不重開整機） | `request restart dataplane` |

## 八、常見維護情境速查

- **流量被擋排查**：`test security-policy-match ...` 直接算出命中政策，是 PAN-OS 最實用的排錯指令；再用 `show session all filter ...` 看實際工作階段。
- **設定變更流程**：`configure` → 修改 → `show config diff` 確認 → `commit`（務必 commit，否則不生效）→ `exit`。
- **commit 失敗**：先 `validate full` 找出錯誤，修正後再 `commit`。
- **HA 雙機巡檢**：`show high-availability state` 確認 active/passive 與同步狀態正常。

## 注意事項

1. **務必 commit**：設定模式的變更存於「候選設定」，**未 `commit` 不會生效**；這與 FortiOS 即時生效不同，是 PAN-OS 最常見的疏忽點。
2. **變更前比對**：`commit` 前先 `show config diff` 檢視候選與運行差異，避免誤送。
3. **版本對應**：CLI 指令隨 PAN-OS 大版本略有差異，請對照實際版本的官方 CLI Reference。
4. **唯讀優先**：`show` / `test` / `ping` 類為唯讀安全；`request restart ...`、`commit`、`load config` 具影響，請於維護時段並先匯出設定備份。
