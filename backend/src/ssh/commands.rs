//! 各品牌的指標查詢指令對應。
//!
//! 僅唯讀查詢指令；指令依各品牌官方「日常維護」文件選用：
//! - Cisco：docs/cisco-cmd.md（IOS/IOS-XE/NX-OS）
//! - Aruba：docs/aruba-cmd.md（AOS-CX）
//! - FortiGate：docs/fortigate-cmd.md（FortiOS 7.x）
//! - Palo Alto：docs/palo-alto-cmd.md（PAN-OS 11.x）
//!
//! 實際指令依韌體版本可能仍需微調。

use crate::models::Brand;

/// 支援查詢的指標名稱（固定順序）。
pub const METRICS: [&str; 6] = ["cpu", "memory", "module", "interface", "loading", "traffic"];

/// 回傳某品牌的 (指標, 指令) 清單。
pub fn for_brand(brand: Brand) -> Vec<(&'static str, String)> {
    let map: [(&str, &str); 6] = match brand {
        // Cisco IOS/IOS-XE/NX-OS（docs/cisco-cmd.md）
        Brand::Cisco => [
            ("cpu", "show processes cpu sorted"),
            ("memory", "show processes memory sorted"),
            ("module", "show inventory"),
            ("interface", "show ip interface brief"),
            ("loading", "show processes cpu sorted"),
            ("traffic", "show interfaces counters"),
        ],
        // Aruba AOS-CX（docs/aruba-cmd.md）
        Brand::Aruba => [
            ("cpu", "show system resource-utilization"),
            ("memory", "show system resource-utilization"),
            ("module", "show environment"),
            ("interface", "show interface brief"),
            ("loading", "show system resource-utilization"),
            ("traffic", "show interface"),
        ],
        // FortiGate FortiOS（docs/fortigate-cmd.md）
        Brand::FortigateNgfw => [
            ("cpu", "get system performance status"),
            ("memory", "get system performance status"),
            ("module", "get system status"),
            ("interface", "get system interface physical"),
            ("loading", "get system performance status"),
            ("traffic", "diagnose netlink interface list"),
        ],
        // Palo Alto PAN-OS（docs/palo-alto-cmd.md）
        Brand::PaloAlto => [
            ("cpu", "show system resources"),
            ("memory", "show system resources"),
            ("module", "show system environmentals"),
            ("interface", "show interface all"),
            ("loading", "show system resources"),
            ("traffic", "show interface hardware"),
        ],
    };
    map.iter().map(|(m, c)| (*m, c.to_string())).collect()
}
