use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::{App, AppState, Stock};
use unicode_width::UnicodeWidthStr;

// 版本号
const VERSION: &str = env!("CARGO_PKG_VERSION");

//计算所有的屏幕窗口区域,供后续render使用
pub fn main_chunks(area: Rect) -> Vec<Rect> {
    // 整体布局分为四行：标题栏、内容区、帮助说明区、状态栏
    let parent = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // 标题栏
                Constraint::Min(5),    // 中间行情内容
                Constraint::Length(9), // 帮助说明区
                Constraint::Length(1), // 状态栏
            ]
            .as_ref(),
        )
        .split(area);

    // 中间行情内容分为左右两部分：左侧列表和右侧详情
    let center = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(parent[1]);

    // 计算新建stock时的弹框位置
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(area);

    // 弹框内部分为三列，居中显示输入框
    let popline = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(popup[1]);

    // 返回所有需要的区域
    vec![
        parent[0],  // 标题
        center[0],  // 列表
        center[1],  // 详情
        parent[2],  // 帮助
        parent[3],  // 状态
        popline[1], // 弹窗输入
    ]
}

// 构造股票列表控件
pub fn stock_list(stocks: &Vec<Stock>) -> List<'_> {
    // 构造ListItem列表
    let items: Vec<_> = stocks
        .iter()
        .map(|stock| {
            ListItem::new(Spans::from(vec![
                Span::styled(
                    format!("{:+.2}% ", stock.percent),
                    Style::default().fg(if stock.percent < 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
                Span::styled(stock.title.clone(), Style::default()),
            ]))
        })
        .collect();

    // 构造List控件
    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("列表")
                .border_type(BorderType::Plain),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
}

// 构造股票详情控件
pub fn stock_detail(app: &App) -> Paragraph<'_> {
    // 获取当前选中股票的索引和数据
    let sel = app.stocks_state.selected().unwrap_or(0);
    let stocks = app.stocks.lock().unwrap();

    // 辅助函数：根据数值返回对应的颜色
    let colorize = |val: f64| {
        if val > 0.0 {
            Color::Red
        } else if val < 0.0 {
            Color::Green
        } else {
            Color::White
        }
    };

    // 辅助函数：格式化 KV 对，实现 Label 左对齐，Value 右对齐
    let render_kv = |label: &str, value: String, style: Style| -> Vec<Span> {
        let label_width: usize = 12; // Label 区域固定宽度
        let value_width: usize = 12; // Value 区域固定宽度 (增加宽度容纳大市值)

        // 处理 Label 左对齐 (计算中文字符宽度)
        let l_w = UnicodeWidthStr::width(label);
        let l_pad = " ".repeat(label_width.saturating_sub(l_w));

        // 处理 Value 右对齐 (同样需要计算中文字符宽度，防止“亿”等字符导致错位)
        let v_w = UnicodeWidthStr::width(value.as_str());
        let v_pad = " ".repeat(value_width.saturating_sub(v_w));

        vec![
            Span::raw(format!("{}{}", label, l_pad)),
            Span::styled(format!("{}{}", v_pad, value), style),
            Span::raw("   "), // 增加项之间的间距
        ]
    };

    let mut lines = Vec::new();

    if app.stocks_state.selected().is_some() && sel < stocks.len() {
        let stock = stocks.get(sel).unwrap();
        let c = colorize(stock.change);

        // 第一行：标题
        lines.push(Spans::from(vec![
            Span::styled(
                format!("{} ", stock.title),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            ),
            Span::styled(
                format!("({})", stock.code),
                Style::default().fg(Color::Gray),
            ),
        ]));
        lines.push(Spans::from(vec![Span::raw("")]));

        // 第二行：现价、涨跌、涨幅
        let mut row2 = Vec::new();
        row2.extend(render_kv(
            " 现价:",
            format!("{:.2}", stock.price),
            Style::default().fg(c).add_modifier(Modifier::BOLD),
        ));
        row2.extend(render_kv(
            "涨跌:",
            format!("{:+.2}", stock.change),
            Style::default().fg(c),
        ));
        row2.extend(render_kv(
            "幅度:",
            format!("{:+.2}%", stock.percent),
            Style::default().fg(colorize(stock.percent)),
        ));
        lines.push(Spans::from(row2));

        // 第三行：5分涨跌、涨速、量比
        let mut row3 = Vec::new();
        row3.extend(render_kv(
            " 5分钟涨跌:",
            format!("{:+.2}%", stock.min_5_pct),
            Style::default().fg(colorize(stock.min_5_pct)),
        ));
        row3.extend(render_kv(
            "涨速:",
            format!("{:+.2}", stock.speed),
            Style::default().fg(colorize(stock.speed)),
        ));
        row3.extend(render_kv(
            "量比:",
            format!("{:.2}", stock.ratio),
            Style::default(),
        ));
        lines.push(Spans::from(row3));

        lines.push(Spans::from(vec![Span::raw(
            "-------------------------------------------------------------------------------",
        )]));

        // 第四行：今开、最高、成交量
        let mut row4 = Vec::new();
        row4.extend(render_kv(
            " 今开:",
            format!("{:.2}", stock.open),
            Style::default(),
        ));
        row4.extend(render_kv(
            "最高:",
            format!("{:.2}", stock.high),
            Style::default(),
        ));
        row4.extend(render_kv(
            "成交量:",
            format!("{:.2}万", stock.vol / 10000.0),
            Style::default(),
        ));
        lines.push(Spans::from(row4));

        // 第五行：昨收、最低、成交额
        let mut row5 = Vec::new();
        row5.extend(render_kv(
            " 昨收:",
            format!("{:.2}", stock.yestclose),
            Style::default(),
        ));
        row5.extend(render_kv(
            "最低:",
            format!("{:.2}", stock.low),
            Style::default(),
        ));
        row5.extend(render_kv(
            "成交额:",
            format!("{:.2}亿", stock.amount / 100000000.0),
            Style::default(),
        ));
        lines.push(Spans::from(row5));

        // 第六行：市盈、市净、60日
        let mut row6 = Vec::new();
        row6.extend(render_kv(
            " 市盈(动):",
            format!("{:.2}", stock.pe),
            Style::default(),
        ));
        row6.extend(render_kv(
            "市净:",
            format!("{:.2}", stock.pb),
            Style::default(),
        ));
        row6.extend(render_kv(
            "60日涨跌:",
            format!("{:+.2}%", stock.pct_60d),
            Style::default().fg(colorize(stock.pct_60d)),
        ));
        lines.push(Spans::from(row6));

        // 第七行：总市值、流通、年初
        let mut row7 = Vec::new();
        row7.extend(render_kv(
            " 总市值:",
            format!("{:.2}亿", stock.total_value / 100000000.0),
            Style::default(),
        ));
        row7.extend(render_kv(
            "流通市值:",
            format!("{:.2}亿", stock.cur_value / 100000000.0),
            Style::default(),
        ));
        row7.extend(render_kv(
            "年初至今:",
            format!("{:+.2}%", stock.pct_ytd),
            Style::default().fg(colorize(stock.pct_ytd)),
        ));
        lines.push(Spans::from(row7));
    } else {
        lines.push(Spans::from(vec![Span::styled(
            "请在左侧选择股票以查看详细行情",
            Style::default().fg(Color::Gray),
        )]));
    }

    Paragraph::new(lines).alignment(Alignment::Left).block(
        Block::default().title(" 行情详情 ").borders(Borders::ALL), // .border_type(BorderType::Rounded),
    )
}

// 构造帮助说明控件
pub fn help_panel() -> Paragraph<'static> {
    let info = "【输入示例】\n\
                1. 纯数字: 600519 (盲试沪深/港)\n\
                2. 纯字母: NVDA (盲试美/英)\n\
                3. 全手动: x市场.代码 (如 x105.AAPL)\n\
                【项目地址/反馈】 https://github.com/fjqz177/rust-stock";

    Paragraph::new(info)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .title("帮助指南")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
}

// 构造股票代码输入控件
pub fn stock_input(app: &App) -> Paragraph<'_> {
    Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("输入证券代码"))
}

// 构造标题栏控件
pub fn title_bar(app: &App, rect: Rect) -> Paragraph<'_> {
    let left = format!("Stock v{}", VERSION);
    let error = app.error.lock().unwrap();
    let right = if error.is_empty() {
        app.last_refresh
            .lock()
            .unwrap()
            .format("最后更新 %H:%M:%S")
            .to_string()
    } else {
        error.clone()
    };
    Paragraph::new(Spans::from(vec![
        Span::raw(left.clone()),
        //使用checked_sub防止溢出
        Span::raw(
            " ".repeat(
                (rect.width as usize)
                    .checked_sub(right.width() + left.width())
                    .unwrap_or(0),
            ),
        ),
        Span::styled(
            right,
            Style::default().fg(if error.is_empty() {
                Color::White
            } else {
                Color::Red
            }),
        ),
    ]))
    .alignment(Alignment::Left)
}

// 构造按键提示栏控件
pub fn status_bar(app: &mut App) -> Paragraph<'_> {
    Paragraph::new(
        match app.state {
            AppState::Normal => {
                "退出[Q] | 新建[N] | 删除[D] | 刷新[R] | 上移[U] | 下移[J] | 选择[↑/↓]"
            }
            AppState::Adding => "确认[Enter] | 取消[ESC] | 输入代码(如 600519, 000001, NVDA)",
        }
        .to_string(),
    )
    .alignment(Alignment::Left)
}
