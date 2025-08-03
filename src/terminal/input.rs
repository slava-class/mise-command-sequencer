use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};

use crate::models::AppEvent;

pub fn spawn_input_handler(event_tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press
                        && event_tx.send(AppEvent::KeyPress(key.code)).is_err()
                    {
                        break;
                    }
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
