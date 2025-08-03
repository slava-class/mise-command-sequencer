use anyhow::Result;
use ratatui::crossterm::event::KeyCode;
use std::time::Instant;

use super::App;
use crate::models::{AppEvent, AppState, SequenceEvent};

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
            AppEvent::Sequence(sequence_event) => {
                self.handle_sequence_event(sequence_event)?;
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
            (AppState::List, KeyCode::Char('s')) => {
                self.state = AppState::SequenceBuilder;
            }

            (AppState::Detail(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),
            (AppState::Detail(_), KeyCode::Char('x')) => self.run_selected_task().await?,

            (AppState::Running(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),

            // Sequence Builder controls
            (AppState::SequenceBuilder, KeyCode::Down | KeyCode::Char('j')) => self.select_next(),
            (AppState::SequenceBuilder, KeyCode::Up | KeyCode::Char('k')) => self.select_previous(),
            (AppState::SequenceBuilder, KeyCode::Char('1')) => self.toggle_current_task_step(0)?,
            (AppState::SequenceBuilder, KeyCode::Char('2')) => self.toggle_current_task_step(1)?,
            (AppState::SequenceBuilder, KeyCode::Char('3')) => self.toggle_current_task_step(2)?,
            (AppState::SequenceBuilder, KeyCode::Enter) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::Sequence(SequenceEvent::RunSequence));
            }
            (AppState::SequenceBuilder, KeyCode::Char('c')) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::Sequence(SequenceEvent::ClearSequence));
            }
            (AppState::SequenceBuilder, KeyCode::Char('x')) => self.run_current_task().await?,
            (AppState::SequenceBuilder, KeyCode::Char('e')) => self.edit_current_task().await?,
            (AppState::SequenceBuilder, KeyCode::Tab) => self.show_current_task_content().await?,
            (AppState::SequenceBuilder, KeyCode::Esc | KeyCode::Char('b')) => {
                self.state = AppState::List;
            }

            _ => {}
        }
        Ok(())
    }
}
