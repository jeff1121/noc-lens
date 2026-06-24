//! 各品牌的指標查詢指令對應（草案，對應 contracts/brand-commands.md）。
//!
//! 僅唯讀查詢指令。實際指令依韌體版本可能需校正。

use crate::models::Brand;

/// 支援查詢的指標名稱（固定順序）。
pub const METRICS: [&str; 6] = ["cpu", "memory", "module", "interface", "loading", "traffic"];

/// 回傳某品牌的 (指標, 指令) 清單。
pub fn for_brand(brand: Brand) -> Vec<(&'static str, String)> {
    let map: [(&str, &str); 6] = match brand {
        Brand::Cisco => [
            ("cpu", "show processes cpu | include CPU utilization"),
            ("memory", "show memory statistics"),
            ("module", "show module"),
            ("interface", "show ip interface brief"),
            ("loading", "show processes cpu | include CPU utilization"),
            ("traffic", "show interfaces counters"),
        ],
        Brand::Aruba => [
            ("cpu", "show system resource-utilization"),
            ("memory", "show system resource-utilization"),
            ("module", "show modules"),
            ("interface", "show interface brief"),
            ("loading", "show system resource-utilization"),
            ("traffic", "show interface"),
        ],
        Brand::FortigateNgfw => [
            ("cpu", "get system performance status"),
            ("memory", "get system performance status"),
            ("module", "get system status"),
            ("interface", "get system interface"),
            ("loading", "get system performance status"),
            ("traffic", "get system interface"),
        ],
        Brand::PaloAlto => [
            ("cpu", "show system resources"),
            ("memory", "show system resources"),
            ("module", "show system environmentals"),
            ("interface", "show interface all"),
            ("loading", "show system resources"),
            ("traffic", "show counter interface all"),
        ],
    };
    map.iter().map(|(m, c)| (*m, c.to_string())).collect()
}
