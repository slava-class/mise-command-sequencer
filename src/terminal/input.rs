use ratatui::crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};

use crate::models::app_event::ScrollDirection;
use crate::models::AppEvent;

pub fn spawn_input_handler(event_tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) => {
                        if key.kind == KeyEventKind::Press
                            && event_tx.send(AppEvent::KeyPress(key.code)).is_err()
                        {
                            break;
                        }
                    }
                    Ok(Event::Mouse(mouse)) => match mouse.kind {
                        MouseEventKind::Down(button) => {
                            if event_tx
                                .send(AppEvent::MouseClick {
                                    button,
                                    row: mouse.row,
                                    col: mouse.column,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if event_tx
                                .send(AppEvent::MouseScroll {
                                    direction: ScrollDirection::Up,
                                    row: mouse.row,
                                    col: mouse.column,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if event_tx
                                .send(AppEvent::MouseScroll {
                                    direction: ScrollDirection::Down,
                                    row: mouse.row,
                                    col: mouse.column,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
    });
}

pub fn spawn_tick_handler(event_tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(80));
        loop {
            interval.tick().await;
            if event_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });
}
