use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, MouseButton};
use std::time::Instant;

use super::App;
use crate::models::{AppEvent, AppState, SequenceEvent};

impl App {
    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::KeyPress(key) => self.handle_key(key).await?,
            AppEvent::MouseClick { button, row, col } => {
                self.handle_mouse_click(button, row, col).await?
            }
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

    pub async fn handle_mouse_click(
        &mut self,
        button: MouseButton,
        row: u16,
        col: u16,
    ) -> Result<()> {
        // Only handle left mouse button clicks
        if button != MouseButton::Left {
            return Ok(());
        }

        match &self.state {
            AppState::SequenceBuilder => {
                self.handle_sequence_builder_click(row, col).await?;
            }
            _ => {
                // Handle clicks in other states if needed
            }
        }
        Ok(())
    }

    async fn handle_sequence_builder_click(&mut self, row: u16, col: u16) -> Result<()> {
        // Get the stored table layout for accurate column detection
        let Some(table_layout) = &self.table_layout else {
            return Ok(());
        };

        // Calculate which task row was clicked (accounting for header and borders)
        // The table area starts at table_layout.table_area.y
        // Header row is at y + 1, data rows start at y + 2
        let table_start_row = table_layout.table_area.y;
        if row >= table_start_row + 2 {
            let task_index = (row - table_start_row - 2) as usize;
            if task_index < self.tasks.len() {
                // Update selected task
                self.selected_task = task_index;

                // Use the calculated column rectangles for accurate hit detection
                let num_steps = 3;

                // Column 0: Task name
                // Columns 1-3: Step columns
                // Column 4: Actions column

                // Check step columns (1, 2, 3)
                for step in 0..num_steps {
                    let column_index = step + 1; // Steps start at column 1
                    if column_index < table_layout.column_rects.len() {
                        let column_rect = table_layout.column_rects[column_index];
                        if col >= column_rect.x && col < column_rect.x + column_rect.width {
                            self.toggle_current_task_step(step)?;
                            return Ok(());
                        }
                    }
                }

                // Check actions column (last column)
                if let Some(actions_rect) = table_layout.column_rects.last() {
                    if col >= actions_rect.x && col < actions_rect.x + actions_rect.width {
                        // Calculate button positions within the actions column
                        // Actions text: "[run] [cat] [edit]"
                        let relative_col = col - actions_rect.x;
                        if (0..=4).contains(&relative_col) {
                            // "run" button
                            self.run_current_task().await?;
                        } else if (6..=10).contains(&relative_col) {
                            // "cat" button
                            self.show_current_task_content().await?;
                        } else if (12..=17).contains(&relative_col) {
                            // "edit" button
                            self.edit_current_task().await?;
                        }
                    }
                }
            }
        }
        // Check for sequence control buttons in the title area
        else if row == table_start_row {
            // The sequence controls are rendered in the top-right of the title
            // This is a rough approximation - could be improved with more precise layout tracking
            let terminal_width = table_layout.table_area.width;
            let controls_start = terminal_width.saturating_sub(30);

            if col >= controls_start {
                let relative_col = col - controls_start;
                if (0..=13).contains(&relative_col) {
                    // "Run sequence" button
                    let _ = self
                        .event_tx
                        .send(AppEvent::Sequence(SequenceEvent::RunSequence));
                } else if (15..=21).contains(&relative_col) {
                    // "Clear" button
                    let _ = self
                        .event_tx
                        .send(AppEvent::Sequence(SequenceEvent::ClearSequence));
                }
            }
        }
        Ok(())
    }
}
