use crate::model::{RawStock, Stock};
use http_req::request;
use serde_json::Value;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn fetch_stocks(stock_codes: &[String]) -> DynResult<Vec<Stock>> {
    if stock_codes.is_empty() {
        return Ok(Vec::new());
    }

    // 转换代码为 secid
    let secids: Vec<String> = stock_codes.iter().map(|s| to_secid(s)).collect();
    let query_ids = secids.join(",");

    let url = format!("https://push2.eastmoney.com/api/qt/ulist.np/get?secids={}&fields=f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13,f14,f15,f16,f17,f18,f20,f21,f22,f23,f24,f25", query_ids);

    let mut writer = Vec::new();
    request::get(url, &mut writer)?;

    let content = String::from_utf8_lossy(&writer);
    let v: Value = serde_json::from_str(&content)?;

    let mut stocks = Vec::new();

    if let Some(diff) = v["data"]["diff"].as_array() {
        for item in diff {
            // 使用 serde 将 JSON直接转换为 RawStock
            let raw: RawStock = serde_json::from_value(item.clone())?;
            // 再转换为领域模型 Stock
            stocks.push(Stock::from(raw));
        }
    }

    Ok(stocks)
}

// 根据用户输入股票代码生成secid字符串
pub fn to_secid(code: &str) -> String {
    let code_lower = code.to_lowercase();

    // 0. 全手动模式 (x 开头，直接透传 x 之后的所有内容，如 x105.NVDA)
    if code_lower.starts_with('x') {
        return code[1..].to_string();
    }

    // 1. 全盲试模式：根据输入特征组合多个市场
    let is_numeric = code.chars().all(|c| c.is_ascii_digit());
    if is_numeric {
        // 数字类：尝试 沪(1), 深北(0), 港(116), 英(155)
        format!("1.{},0.{},116.{}", code, code, code)
    } else {
        // 字母类：尝试 美(105-107), 英(155)
        format!("105.{},106.{},107.{},155.{}", code, code, code, code)
    }
}

#[cfg(test)]
mod tests {
    use super::to_secid;

    #[test]
    fn to_secid_manual_x_prefix() {
        assert_eq!(to_secid("x105.NVDA"), "105.NVDA".to_string());
    }

    #[test]
    fn to_secid_numeric_blind() {
        assert_eq!(
            to_secid("600519"),
            "1.600519,0.600519,116.600519".to_string()
        );
    }

    #[test]
    fn to_secid_alpha_blind() {
        assert_eq!(
            to_secid("NVDA"),
            "105.NVDA,106.NVDA,107.NVDA,155.NVDA".to_string()
        );
    }

    #[test]
    fn to_secid_uk_trailing_dot() {
        // 英股代码如 RR. (劳斯莱斯) 含字母和 '.'，应走字母分支
        assert_eq!(
            to_secid("RR."),
            "105.RR.,106.RR.,107.RR.,155.RR.".to_string()
        );
    }
}
