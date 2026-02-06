use crate::api;
use crate::model::Stock;
use crate::storage;
use chrono::{DateTime, Local};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread;
use tui::widgets::ListState;

pub enum AppState {
    Normal,
    Adding,
}

// 应用程序内部事件，用于异步传递数据
pub enum AppEvent {
    StocksFetched(Vec<Stock>),
    FetchError(String),
}

pub struct App {
    pub should_exit: bool,
    pub state: AppState,
    pub error: String,
    pub input: String,
    pub stocks: Vec<Stock>,
    // TUI的List控件需要这个state记录当前选中和滚动位置两个状态
    pub stocks_state: ListState,
    pub last_refresh: DateTime<Local>,
    pub tick_count: u128,

    // 异步通信通道
    rx: Receiver<AppEvent>,
    refresh_tx: SyncSender<Vec<String>>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (refresh_tx, refresh_rx) = mpsc::sync_channel::<Vec<String>>(1);
        let worker_tx = tx.clone();

        // 单一后台线程处理所有刷新请求，避免无限创建线程
        thread::spawn(move || {
            for codes in refresh_rx {
                match api::fetch_stocks(&codes) {
                    Ok(data) => {
                        let _ = worker_tx.send(AppEvent::StocksFetched(data));
                    }
                    Err(e) => {
                        let _ = worker_tx.send(AppEvent::FetchError(format!("{:?}", e)));
                    }
                }
            }
        });

        let mut app = Self {
            should_exit: false,
            state: AppState::Normal,
            input: String::new(),
            error: String::new(),
            stocks: Vec::new(),
            stocks_state: ListState::default(),
            last_refresh: Local::now(),
            tick_count: 0,
            rx,
            refresh_tx,
        };

        // 加载保存的股票代码
        app.load_stocks();
        // 初始刷新
        app.refresh_stocks();
        app
    }

    // 从存储加载股票
    fn load_stocks(&mut self) {
        match storage::load_stocks() {
            Ok(codes) => {
                self.stocks = codes.iter().map(|c| Stock::new(c)).collect();
            }
            Err(e) => {
                self.error = e.to_string();
            }
        }
    }

    // 保存股票到存储
    pub fn save_stocks(&self) -> storage::DynResult<()> {
        let codes: Vec<String> = self.stocks.iter().map(|s| s.code.clone()).collect();
        storage::save_stocks(&codes)
    }

    // 触发刷新股票数据
    pub fn refresh_stocks(&mut self) {
        if self.stocks.is_empty() {
            return;
        }

        let codes: Vec<String> = self.stocks.iter().map(|s| s.code.clone()).collect();

        // 限制并发刷新，请求队列满时丢弃新的请求
        match self.refresh_tx.try_send(codes) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => {
                self.error = "refresh worker disconnected".to_string();
            }
        }
    }

    // 仅处理通道消息，不递增 tick 计数
    pub fn drain_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::StocksFetched(new_data) => {
                    self.update_stocks(new_data);
                    self.last_refresh = Local::now();
                    self.error.clear();
                }
                AppEvent::FetchError(err_msg) => {
                    self.error = err_msg;
                }
            }
        }
    }

    // 处理通道消息 (需要在主循环中调用)
    pub fn on_tick(&mut self) {
        self.tick_count += 1;
        self.drain_events();

        // 定时刷新 (每60个tick)
        if self.tick_count % 60 == 0 {
            if let AppState::Normal = self.state {
                self.refresh_stocks();
            }
        }
    }

    // 更新股票数据，保留原有列表顺序和选中状态
    fn update_stocks(&mut self, new_data: Vec<Stock>) {
        // 遍历当前的 stocks，尝试从新数据中找到匹配项进行更新
        for stock in self.stocks.iter_mut() {
            let user_key = Self::normalize_code_for_match(&stock.code);
            // new_data 里的 stock.code 是 API 返回的 f12
            if let Some(match_stock) = new_data
                .iter()
                .find(|s| s.code.eq_ignore_ascii_case(&user_key))
            {
                // 更新除 code 和 title 以外的字段 (或者根据需要更新)
                // 注意：如果想保留用户输入的 code (如 x105.NVDA)，就不能直接覆盖 stock.code
                // 但原代码里：stock.title = item["f14"]...; stock.price = ...
                // 原代码并没有覆盖 stock.code。

                let original_code = stock.code.clone();
                // 覆盖字段
                *stock = match_stock.clone();
                // 还原用户输入的 code，以防下次匹配失败 (或者保持 API 的 code?)
                // 原代码里 stock.code 始终保持用户输入的值 (Stock::new(&s.as_str()))，
                // 只有 title 被 API 覆盖。
                stock.code = original_code;
            }
        }
    }

    fn normalize_code_for_match(code: &str) -> String {
        let stripped = if let Some(rest) = code.strip_prefix('x') {
            rest
        } else {
            code
        };
        let without_market = stripped.rsplit('.').next().unwrap_or(stripped);
        without_market.to_ascii_uppercase()
    }
}
