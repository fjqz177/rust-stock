use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};

use crate::{App, AppState, Stock};

//处理键盘、鼠标事件
pub fn on_events(event: Event, app: &mut App) {
    let total = app.stocks.len();
    let sel = app.stocks_state.selected().unwrap_or(0);
    let selsome = app.stocks_state.selected().is_some() && sel < total;
    match app.state {
        AppState::Normal => {
            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Release {
                    let code = key.code;
                    if code == KeyCode::Char('q') {
                        app.should_exit = true;
                    } else if code == KeyCode::Char('r') {
                        app.refresh_stocks();
                    } else if code == KeyCode::Char('n') {
                        //新建stock
                        app.state = AppState::Adding;
                        app.input = String::new();
                    } else if code == KeyCode::Char('d') && selsome {
                        //删除当前选中的stock
                        app.stocks.remove(sel);
                        if let Err(e) = app.save_stocks() {
                            app.error = e.to_string();
                        }
                        app.stocks_state.select(None);
                    } else if code == KeyCode::Char('u') && selsome && sel > 0 {
                        //将选中stock往上移动一位
                        app.stocks.swap(sel, sel - 1);
                        if let Err(e) = app.save_stocks() {
                            app.error = e.to_string();
                        }
                        app.stocks_state.select(Some(sel - 1));
                    } else if code == KeyCode::Char('j') && selsome && sel < total - 1 {
                        //将选中stock往下移动一位
                        app.stocks.swap(sel, sel + 1);
                        if let Err(e) = app.save_stocks() {
                            app.error = e.to_string();
                        }
                        app.stocks_state.select(Some(sel + 1));
                    } else if code == KeyCode::Up && total > 0 {
                        //注意这里如果不加判断直接用sel - 1, 在sel为0时会导致异常
                        app.stocks_state
                            .select(Some(if sel > 0 { sel - 1 } else { 0 }));
                    } else if code == KeyCode::Down && total > 0 {
                        app.stocks_state
                            .select(Some(if sel < total - 1 { sel + 1 } else { sel }));
                    }
                }
            } else if let Event::Mouse(mouse) = event {
                match mouse.kind {
                    MouseEventKind::Up(_button) => {
                        let row = mouse.row as usize;
                        if row >= 2 && row < total + 2 {
                            app.stocks_state.select(Some(row - 2));
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        if total > 0 {
                            app.stocks_state
                                .select(Some(if sel > 0 { sel - 1 } else { 0 }));
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if total > 0 {
                            app.stocks_state.select(Some(if sel < total - 1 {
                                sel + 1
                            } else {
                                sel
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }

        AppState::Adding => match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Enter => {
                    app.state = AppState::Normal;
                    if app.input.len() > 0 {
                        app.stocks.push(Stock::new(app.input.as_str()));
                        app.refresh_stocks();
                        if let Err(e) = app.save_stocks() {
                            app.error = e.to_string();
                        }
                    }
                }
                KeyCode::Esc => {
                    app.state = AppState::Normal;
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                _ => {}
            },
            _ => {}
        },
    }
}

//处理定时事件
//注意：App::on_tick 已经在 app.rs 中处理了 channel 消息和 tick 计数
//所以这里的 on_tick 应该调用 app.on_tick()
pub fn on_tick(app: &mut App) {
    app.on_tick();
}
