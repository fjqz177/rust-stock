use std::{
    collections::HashMap,
    fs,
    io::Stdout,
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Local};
use http_req::request;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tui::{backend::CrosstermBackend, widgets::ListState};

pub mod aio;
pub mod events;
pub mod widget;

// 通用结果类型别名
pub type DynResult = Result<(), Box<dyn std::error::Error>>;
// Crossterm终端类型别名
pub type CrossTerminal = tui::Terminal<CrosstermBackend<Stdout>>;
// TUI帧类型别名
pub type TerminalFrame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>;

// 股票代码数据库文件路径
pub const DB_PATH: &str = ".stocks.json";

// 股票数据结构体
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

// 股票数据结构体实现
impl Stock {
    pub fn new(code: &String) -> Self {
        Self {
            code: code.clone(),
            title: code.clone(),
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

// 应用程序状态枚举
pub enum AppState {
    Normal,
    Adding,
}

// 主应用程序结构体
pub struct App {
    pub should_exit: bool,
    pub state: AppState,
    pub error: Arc<Mutex<String>>,
    pub input: String,
    pub stocks: Arc<Mutex<Vec<Stock>>>,
    //TUI的List控件需要这个state记录当前选中和滚动位置两个状态
    pub stocks_state: ListState,
    pub last_refresh: Arc<Mutex<DateTime<Local>>>,
    pub tick_count: u128,
}

// 主应用程序结构体实现
impl App {
    pub fn new() -> Self {
        let mut app = Self {
            should_exit: false,
            state: AppState::Normal,
            input: String::new(),
            error: Arc::new(Mutex::new(String::new())),
            stocks: Arc::new(Mutex::new([].to_vec())),
            //ListState:default为未选择，因为可能stocks为空，所以不能自动选第一个
            stocks_state: ListState::default(),
            last_refresh: Arc::new(Mutex::new(Local::now())),
            tick_count: 0,
        };
        app.load_stocks().unwrap_or_default();
        app.refresh_stocks();
        return app;
    }

    pub fn save_stocks(&self) -> DynResult {
        let db = dirs_next::home_dir().unwrap().join(DB_PATH);
        //每个stock单独存一个对象，是考虑将来的扩展性
        let stocks = self.stocks.lock().unwrap();
        let lists: Vec<_> = stocks
            .iter()
            .map(|s| HashMap::from([("code", &s.code)]))
            .collect();
        fs::write(
            &db,
            serde_json::to_string(&HashMap::from([("stocks", lists)]))?,
        )?;
        Ok(())
    }

    pub fn load_stocks(&mut self) -> DynResult {
        //用unwrap_or_default屏蔽文件不存在时的异常
        let content =
            fs::read_to_string(dirs_next::home_dir().unwrap().join(DB_PATH)).unwrap_or_default();
        //如果直接转换stocks，必须所有key都对上, 兼容性不好
        //self.stocks = serde_json::from_str(&content).unwrap_or_default();

        //先读成Map再转换，可以增加兼容性，
        let json: Map<String, Value> = serde_json::from_str(&content).unwrap_or_default();
        let mut data = self.stocks.lock().unwrap();
        data.clear();
        data.append(
            &mut json
                .get("stocks")
                .unwrap_or(&json!([]))
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    Stock::new(
                        &s.as_object()
                            .unwrap()
                            .get("code")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string(),
                    )
                })
                .collect(),
        );

        Ok(())
    }

    pub fn refresh_stocks(&mut self) {
        let stock_clone = self.stocks.clone();
        let err_clone = self.error.clone();
        let last_refresh_clone = self.last_refresh.clone();
        let codes = self.get_codes();
        if codes.len() > 0 {
            thread::spawn(move || {
                let mut writer = Vec::new();
                let url = format!("http://push2.eastmoney.com/api/qt/ulist.np/get?secids={}&fields=f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13,f14,f15,f16,f17,f18,f20,f21,f22,f23,f24,f25", codes);
                let ret = request::get(url, &mut writer);
                let mut locked_err = err_clone.lock().unwrap();
                if let Err(err) = ret {
                    *locked_err = format!("{:?}", err);
                } else {
                    let content = String::from_utf8_lossy(&writer);
                    let v: Value = serde_json::from_str(&content).unwrap_or(json!({}));
                    if let Some(diff) = v["data"]["diff"].as_array() {
                        let mut stocks = stock_clone.lock().unwrap();
                        for item in diff {
                            let item_code = item["f12"].as_str().unwrap_or("");
                            let mkt = item["f13"].as_u64().unwrap_or(0); // 获取市场ID
                            for stock in stocks.iter_mut() {
                                if stock.code.contains(item_code) {
                                    // 识别市场判断价格倍率: 116(港), 105-107(美), 155(英) 使用 1000, 其它使用 100
                                    let divisor =
                                        if mkt == 116 || (mkt >= 105 && mkt <= 107) || mkt == 155 {
                                            1000.0
                                        } else {
                                            100.0
                                        };

                                    stock.title =
                                        item["f14"].as_str().unwrap_or(&stock.code).to_string();
                                    stock.price = item["f2"].as_f64().unwrap_or(0.0) / divisor;
                                    stock.percent = item["f3"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.change = item["f4"].as_f64().unwrap_or(0.0) / divisor;
                                    stock.amplitude = item["f7"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.open = item["f17"].as_f64().unwrap_or(0.0) / divisor;
                                    stock.yestclose = item["f18"].as_f64().unwrap_or(0.0) / divisor;
                                    stock.high = item["f15"].as_f64().unwrap_or(0.0) / divisor;
                                    stock.low = item["f16"].as_f64().unwrap_or(0.0) / divisor;
                                    stock.vol = item["f5"].as_f64().unwrap_or(0.0);
                                    stock.amount = item["f6"].as_f64().unwrap_or(0.0);
                                    stock.turnover = item["f8"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.pe = item["f9"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.pb = item["f23"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.ratio = item["f10"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.min_5_pct = item["f11"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.total_value = item["f20"].as_f64().unwrap_or(0.0);
                                    stock.cur_value = item["f21"].as_f64().unwrap_or(0.0);
                                    stock.speed = item["f22"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.pct_60d = item["f24"].as_f64().unwrap_or(0.0) / 100.0;
                                    stock.pct_ytd = item["f25"].as_f64().unwrap_or(0.0) / 100.0;
                                }
                            }
                        }
                        let mut last_refresh = last_refresh_clone.lock().unwrap();
                        *last_refresh = Local::now();
                        *locked_err = String::new();
                    } else {
                        *locked_err = String::from("数据解析错误");
                    }
                }
            });
        }
    }

    pub fn get_codes(&self) -> String {
        let codes: Vec<String> = self
            .stocks
            .lock()
            .unwrap()
            .iter()
            .map(|stock| to_secid(&stock.code))
            .collect();
        codes.join(",")
    }
}

// 根据用户输入股票代码生成secid字符串
fn to_secid(code: &str) -> String {
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
