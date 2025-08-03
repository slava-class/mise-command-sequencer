use ratatui::crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};

use crate::models::app_event::ScrollDirection;
use crate::models::AppEvent;

pub fn spawn_input_handler(event_tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut last_mouse_pos = (0u16, 0u16);

        loop {
            if event::poll(Duration::from_millis(16)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) => {
                        if key.kind == KeyEventKind::Press
                            && event_tx.send(AppEvent::KeyPress(key)).is_err()
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
                        MouseEventKind::Moved => {
                            let current_pos = (mouse.row, mouse.column);

                            // Only send if position actually changed
                            if current_pos != last_mouse_pos {
                                if event_tx
                                    .send(AppEvent::MouseMove {
                                        row: mouse.row,
                                        col: mouse.column,
                                    })
                                    .is_err()
                                {
                                    break;
                                }
                                last_mouse_pos = current_pos;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            sleep(Duration::from_millis(8)).await; // ~120 FPS for smoother input
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
