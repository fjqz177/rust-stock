use serde::{Deserialize, Serialize};

// 股票数据结构体 - 领域模型
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stock {
    pub code: String,     // 股票代码 (f12)
    pub title: String,    // 股票名称 (f14)
    pub price: f64,       // 当前价格 (f2)
    pub percent: f64,     // 涨跌幅 (f3)
    pub change: f64,      // 涨跌额 (f4)
    pub amplitude: f64,   // 振幅 (f7)
    pub open: f64,        // 今开 (f17)
    pub yestclose: f64,   // 昨收 (f18)
    pub high: f64,        // 最高 (f15)
    pub low: f64,         // 最低 (f16)
    pub vol: f64,         // 成交量(手) (f5)
    pub amount: f64,      // 成交额 (f6)
    pub turnover: f64,    // 换手率 (f8)
    pub pe: f64,          // 市盈率 (f9)
    pub pb: f64,          // 市净率 (f23)
    pub ratio: f64,       // 量比 (f10)
    pub min_5_pct: f64,   // 5分钟涨跌幅 (f11)
    pub total_value: f64, // 总市值 (f20)
    pub cur_value: f64,   // 流通市值 (f21)
    pub speed: f64,       // 涨速 (f22)
    pub pct_60d: f64,     // 60日涨跌幅 (f24)
    pub pct_ytd: f64,     // 年初至今涨跌幅 (f25)
}

impl Stock {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            title: code.to_string(),
            price: 0.0,
            percent: 0.0,
            change: 0.0,
            amplitude: 0.0,
            open: 0.0,
            yestclose: 0.0,
            high: 0.0,
            low: 0.0,
            vol: 0.0,
            amount: 0.0,
            turnover: 0.0,
            pe: 0.0,
            pb: 0.0,
            ratio: 0.0,
            min_5_pct: 0.0,
            total_value: 0.0,
            cur_value: 0.0,
            speed: 0.0,
            pct_60d: 0.0,
            pct_ytd: 0.0,
        }
    }
}

// 原始 API 数据结构体 (DTO)
// 使用 serde 直接映射 API 字段，避免手动解析
#[derive(Deserialize, Debug)]
pub struct RawStock {
    #[serde(rename = "f12")]
    pub code: String, // 股票代码
    #[serde(rename = "f13")]
    pub mkt: u64, // 市场代码
    #[serde(rename = "f14")]
    pub title: String, // 股票名称
    #[serde(rename = "f2")]
    pub price: Option<f64>, // 可能为null (停牌等情况)
    #[serde(rename = "f3")]
    pub percent: Option<f64>,
    #[serde(rename = "f4")]
    pub change: Option<f64>,
    #[serde(rename = "f5")]
    pub vol: Option<f64>,
    #[serde(rename = "f6")]
    pub amount: Option<f64>,
    #[serde(rename = "f7")]
    pub amplitude: Option<f64>,
    #[serde(rename = "f8")]
    pub turnover: Option<f64>,
    #[serde(rename = "f9")]
    pub pe: Option<f64>,
    #[serde(rename = "f10")]
    pub ratio: Option<f64>,
    #[serde(rename = "f11")]
    pub min_5_pct: Option<f64>,
    #[serde(rename = "f15")]
    pub high: Option<f64>,
    #[serde(rename = "f16")]
    pub low: Option<f64>,
    #[serde(rename = "f17")]
    pub open: Option<f64>,
    #[serde(rename = "f18")]
    pub yestclose: Option<f64>,
    #[serde(rename = "f20")]
    pub total_value: Option<f64>,
    #[serde(rename = "f21")]
    pub cur_value: Option<f64>,
    #[serde(rename = "f22")]
    pub speed: Option<f64>,
    #[serde(rename = "f23")]
    pub pb: Option<f64>,
    #[serde(rename = "f24")]
    pub pct_60d: Option<f64>,
    #[serde(rename = "f25")]
    pub pct_ytd: Option<f64>,
}

impl From<RawStock> for Stock {
    fn from(raw: RawStock) -> Self {
        // 识别市场判断价格倍率: 116(港), 105-107(美), 155(英) 使用 1000, 其它使用 100
        let divisor = if raw.mkt == 116 || (raw.mkt >= 105 && raw.mkt <= 107) || raw.mkt == 155 {
            1000.0
        } else {
            100.0
        };

        Stock {
            code: raw.code,
            title: raw.title,
            price: raw.price.unwrap_or(0.0) / divisor,
            percent: raw.percent.unwrap_or(0.0) / 100.0,
            change: raw.change.unwrap_or(0.0) / divisor,
            amplitude: raw.amplitude.unwrap_or(0.0) / 100.0,
            open: raw.open.unwrap_or(0.0) / divisor,
            yestclose: raw.yestclose.unwrap_or(0.0) / divisor,
            high: raw.high.unwrap_or(0.0) / divisor,
            low: raw.low.unwrap_or(0.0) / divisor,
            vol: raw.vol.unwrap_or(0.0),
            amount: raw.amount.unwrap_or(0.0),
            turnover: raw.turnover.unwrap_or(0.0) / 100.0,
            pe: raw.pe.unwrap_or(0.0) / 100.0,
            pb: raw.pb.unwrap_or(0.0) / 100.0,
            ratio: raw.ratio.unwrap_or(0.0) / 100.0,
            min_5_pct: raw.min_5_pct.unwrap_or(0.0) / 100.0,
            total_value: raw.total_value.unwrap_or(0.0),
            cur_value: raw.cur_value.unwrap_or(0.0),
            speed: raw.speed.unwrap_or(0.0) / 100.0,
            pct_60d: raw.pct_60d.unwrap_or(0.0) / 100.0,
            pct_ytd: raw.pct_ytd.unwrap_or(0.0) / 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RawStock, Stock};

    #[test]
    fn raw_stock_scaling_divisor_1000_for_hk() {
        let raw = RawStock {
            code: "00700".to_string(),
            mkt: 116,
            title: "Tencent".to_string(),
            price: Some(123000.0),
            percent: Some(250.0),
            change: Some(500.0),
            vol: Some(0.0),
            amount: Some(0.0),
            amplitude: Some(300.0),
            turnover: Some(150.0),
            pe: Some(200.0),
            ratio: Some(110.0),
            min_5_pct: Some(120.0),
            high: Some(124000.0),
            low: Some(122000.0),
            open: Some(121000.0),
            yestclose: Some(120000.0),
            total_value: Some(0.0),
            cur_value: Some(0.0),
            speed: Some(80.0),
            pb: Some(90.0),
            pct_60d: Some(100.0),
            pct_ytd: Some(200.0),
        };

        let stock = Stock::from(raw);
        assert_eq!(stock.price, 123.0);
        assert_eq!(stock.change, 0.5);
        assert_eq!(stock.percent, 2.5);
        assert_eq!(stock.amplitude, 3.0);
    }
}
