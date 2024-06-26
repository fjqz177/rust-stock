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

pub type DynResult = Result<(), Box<dyn std::error::Error>>;
pub type CrossTerminal = tui::Terminal<CrosstermBackend<Stdout>>;
pub type TerminalFrame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>;

pub const DB_PATH: &str = ".stocks.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stock {
    pub title: String,
    pub code: String,
    pub price: f64,
    pub percent: f64,
    pub open: f64,      //今开
    pub yestclose: f64, //昨收
    pub high: f64,      //最高
    pub low: f64,       //最低
                        //pub slice: Vec<f64>
}

impl Stock {
    pub fn new(code: &String) -> Self {
        Self {
            code: code.clone(),
            title: code.clone(),
            price: 0.0,
            percent: 0.0,
            open: 0.0,
            yestclose: 0.0,
            high: 0.0,
            low: 0.0,
            //slice:vec![],
        }
    }
}

pub enum AppState {
    Normal,
    Adding,
}
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
                let ret = request::get(
                    format!("{}{}", "http://api.money.126.net/data/feed/", codes),
                    &mut writer,
                );
                let mut locked_err = err_clone.lock().unwrap();
                if let Err(err) = ret {
                    *locked_err = format!("{:?}", err);
                } else {
                    let content = String::from_utf8_lossy(&writer);
                    if content.starts_with("_ntes_quote_callback") {
                        let mut stocks = stock_clone.lock().unwrap();
                        //网易的返回包了一个js call，用skip,take,collect实现一个substring剥掉它
                        let json: Map<String, Value> = serde_json::from_str(
                            &content
                                .chars()
                                .skip(21)
                                .take(content.len() - 23)
                                .collect::<String>(),
                        )
                        .unwrap();
                        for stock in stocks.iter_mut() {
                            //如果code不对,返回的json里不包括这个对象, 用unwrap_or生成一个空对象,防止异常
                            let obj = json
                                .get(&stock.code)
                                .unwrap_or(&json!({}))
                                .as_object()
                                .unwrap()
                                .to_owned();
                            stock.title = obj
                                .get("name")
                                .unwrap_or(&json!(stock.code.clone()))
                                .as_str()
                                .unwrap()
                                .to_owned();
                            stock.price = obj.get("price").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.percent =
                                obj.get("percent").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.open = obj.get("open").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.yestclose = obj
                                .get("yestclose")
                                .unwrap_or(&json!(0.0))
                                .as_f64()
                                .unwrap();
                            stock.high = obj.get("high").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.low = obj.get("low").unwrap_or(&json!(0.0)).as_f64().unwrap();

                            // if json.contains_key(&stock.code) {
                            //     let mut writer2 = Vec::new();
                            //     request::get(format!("http://img1.money.126.net/data/hs/time/today/{}.json",stock.code), &mut writer2)?;
                            //     println!("{:?}", format!("http://img1.money.126.net/data/hs/time/today/{}.json",stock.code));
                            //     let json2: Map<String, Value> = serde_json::from_str(&String::from_utf8_lossy(&writer2).to_string())?;
                            //     stock.slice = json2.get("data").unwrap().as_array().unwrap()
                            //         .iter().map(|item| item.as_array().unwrap().get(2).unwrap().as_f64().unwrap())
                            //         .collect();
                            // }
                        }
                        let mut last_refresh = last_refresh_clone.lock().unwrap();
                        *last_refresh = Local::now();
                        *locked_err = String::new();
                    } else {
                        *locked_err = String::from("服务器返回错误");
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
            .map(|stock| stock.code.clone())
            .collect();
        codes.join(",")
    }
}
