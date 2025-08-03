use anyhow::Result;
use ratatui::crossterm::event::KeyCode;
use std::time::Instant;

use super::App;
use crate::models::{AppEvent, AppState};

impl App {
    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::KeyPress(key) => self.handle_key(key).await?,
            AppEvent::TasksRefreshed(tasks) => {
                self.tasks = tasks;
                self.last_updated = Instant::now();
            }
            AppEvent::TaskInfoLoaded(info) => {
                self.task_info = Some(*info);
            }
            AppEvent::TaskOutput(output) => {
                self.task_output.push_back(output);
                // Keep only the last 100 lines
                while self.task_output.len() > 100 {
                    self.task_output.pop_front();
                }
            }
            AppEvent::TaskCompleted => {
                // Task has completed, we could transition state or just stay
            }
            AppEvent::Tick => {
                // Handle periodic updates if needed
            }
        }
        Ok(())
    }

    pub async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match (&self.state, key) {
            (_, KeyCode::Char('q')) => self.should_quit = true,
            (_, KeyCode::Char('r')) => self.refresh_tasks().await?,

            (AppState::List, KeyCode::Down | KeyCode::Char('j')) => self.select_next(),
            (AppState::List, KeyCode::Up | KeyCode::Char('k')) => self.select_previous(),
            (AppState::List, KeyCode::Enter | KeyCode::Char(' ')) => {
                self.show_task_detail().await?
            }
            (AppState::List, KeyCode::Char('x')) => self.run_selected_task().await?,

            (AppState::Detail(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),
            (AppState::Detail(_), KeyCode::Char('x')) => self.run_selected_task().await?,

            (AppState::Running(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),

            _ => {}
        }
        Ok(())
    }
}
