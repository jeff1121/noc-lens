//! 設備 CLI 輸出解析：將原始文字轉為結構化指標。
//!
//! 各品牌輸出格式差異大，此處採盡力（best-effort）解析；無法解析的指標
//! 以字串 `"n/a"` 標示（對應 FR-010），不回傳臆測數值。

use regex::Regex;
use serde_json::{json, Value};

use crate::models::Brand;

const NA: &str = "n/a";

/// 解析單一指標。
pub fn parse_metric(_brand: Brand, metric: &str, raw: &str) -> Value {
    match metric {
        "cpu" => parse_cpu(raw),
        "memory" => parse_memory(raw),
        "loading" => parse_loading(raw),
        "interface" => parse_interface(raw),
        _ => Value::String(NA.to_string()), // module / traffic：MVP 暫以 n/a
    }
}

fn first_percent(text: &str) -> Option<f64> {
    let re = Regex::new(r"(\d+(?:\.\d+)?)\s*%").ok()?;
    re.captures(text)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
}

fn all_percents(text: &str) -> Vec<f64> {
    Regex::new(r"(\d+(?:\.\d+)?)\s*%")
        .map(|re| {
            re.captures_iter(text)
                .filter_map(|c| c.get(1))
                .filter_map(|m| m.as_str().parse::<f64>().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn parse_cpu(raw: &str) -> Value {
    match first_percent(raw) {
        Some(v) => json!({ "usage_percent": v }),
        None => Value::String(NA.to_string()),
    }
}

fn parse_memory(raw: &str) -> Value {
    // 先嘗試百分比
    if let Some(v) = first_percent(raw) {
        return json!({ "usage_percent": v });
    }
    // 再嘗試 Total / Used 數值（如 Cisco「Total: N ... Used: M」）
    let num = |label: &str| -> Option<f64> {
        Regex::new(&format!(r"(?i){label}\s*:?\s*(\d+)"))
            .ok()
            .and_then(|re| re.captures(raw))
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
    };
    if let (Some(total), Some(used)) = (num("Total"), num("Used")) {
        if total > 0.0 {
            let pct = (used / total) * 100.0;
            return json!({
                "usage_percent": (pct * 10.0).round() / 10.0,
                "used": used,
                "total": total
            });
        }
    }
    Value::String(NA.to_string())
}

fn parse_loading(raw: &str) -> Value {
    let pcts = all_percents(raw);
    if pcts.is_empty() {
        return Value::String(NA.to_string());
    }
    // Cisco CPU 行常見：five seconds / one minute / five minutes
    json!({
        "samples_percent": pcts,
    })
}

fn parse_interface(raw: &str) -> Value {
    let lower = raw.to_lowercase();
    let up = lower.matches(" up").count();
    let down = lower.matches("down").count();
    let total = raw.lines().filter(|l| !l.trim().is_empty()).count();
    if up == 0 && down == 0 {
        return Value::String(NA.to_string());
    }
    json!({ "total_lines": total, "up": up, "down": down })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cisco_cpu_percentage() {
        let raw = "CPU utilization for five seconds: 23%/0%; one minute: 18%; five minutes: 15%";
        let v = parse_metric(Brand::Cisco, "cpu", raw);
        assert_eq!(v["usage_percent"], 23.0);
    }

    #[test]
    fn memory_from_total_used() {
        let raw = "Processor Pool Total: 200 Used: 50 Free: 150";
        let v = parse_metric(Brand::Cisco, "memory", raw);
        assert_eq!(v["usage_percent"], 25.0);
    }

    #[test]
    fn interface_up_down_count() {
        let raw = "Gi0/1 up up\nGi0/2 administratively down down\nGi0/3 up up";
        let v = parse_metric(Brand::Cisco, "interface", raw);
        assert_eq!(v["up"], 4); // 兩行各含兩個 " up"
        assert!(v["down"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn unparseable_is_na() {
        let v = parse_metric(Brand::PaloAlto, "module", "random text");
        assert_eq!(v, serde_json::Value::String("n/a".to_string()));
    }
}
