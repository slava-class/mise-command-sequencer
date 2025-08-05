use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton};
use std::time::Instant;

use super::App;
use crate::models::app_event::ScrollDirection;
use crate::models::{AppEvent, AppState, SequenceEvent};
use crate::ui::button_layout::{
    ActionButton, ActionButtonLayout, ButtonHoverState, ButtonType, SequenceButtonLayout,
};
use crate::ui::constants::*;

const DEFAULT_SCROLL_AMT: usize = 10;

impl App {
    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::KeyPress(key) => self.handle_key(key).await?,
            AppEvent::MouseClick { button, row, col } => {
                self.handle_mouse_click(button, row, col).await?
            }
            AppEvent::MouseScroll {
                direction,
                row: _,
                col: _,
            } => self.handle_mouse_scroll(direction).await?,
            AppEvent::MouseMove { row, col } => self.handle_mouse_move(row, col).await?,
            AppEvent::TasksRefreshed(tasks) => {
                self.tasks = tasks;
                self.last_updated = Instant::now();
            }
            AppEvent::TaskOutput(output) => {
                self.task_output.push_back(output);
                // Keep only the last 100 lines
                while self.task_output.len() > 100 {
                    self.task_output.pop_front();
                    // Adjust scroll offset to maintain position when lines are removed
                    if self.output_scroll_offset > 0 {
                        self.output_scroll_offset = self.output_scroll_offset.saturating_sub(1);
                    }
                }

                // Auto-scroll to bottom if we're at or near the bottom
                if self.show_output_pane
                    && self.task_running
                    && self.current_output_visible_height > 0
                {
                    let visible_height = self.current_output_visible_height;
                    let total_lines = self.task_output.len();

                    // If we're within a few lines of the bottom, keep following
                    if total_lines > visible_height {
                        let max_scroll = total_lines - visible_height;
                        if self.output_scroll_offset >= max_scroll.saturating_sub(3) {
                            self.output_scroll_offset = max_scroll;
                        }
                    }
                }
            }
            AppEvent::TaskCompleted => {
                self.task_running = false;
                self.running_task_handle = None;
            }
            AppEvent::TaskCancelled => {
                // Only update state if we're not already cancelled
                if self.task_running || self.running_task_handle.is_some() {
                    self.task_running = false;
                    self.running_task_handle = None;
                    self.task_output
                        .push_back("Task cancelled by user".to_string());
                }
            }
            AppEvent::Tick => {
                // Handle periodic updates if needed
            }
            AppEvent::Sequence(sequence_event) => {
                self.handle_sequence_event(sequence_event).await?;
            }
        }
        Ok(())
    }

    pub async fn handle_key(&mut self, key_event: KeyEvent) -> Result<()> {
        let KeyEvent {
            code: key,
            modifiers,
            ..
        } = key_event;
        match (&self.state, key) {
            (_, KeyCode::Char('q')) => self.should_quit = true,
            (_, KeyCode::Char('r')) => self.refresh_tasks().await?,
            // Handle Ctrl+C to cancel running tasks
            (_, KeyCode::Char('c')) if self.task_running => {
                // First mark as cancelled to prevent new tasks from starting
                self.task_running = false;

                if let Some(handle) = self.running_task_handle.take() {
                    handle.abort();
                    // Send cancellation event after handle is safely removed
                    if self.event_tx.send(AppEvent::TaskCancelled).is_err() {
                        eprintln!("Warning: Failed to send TaskCancelled event");
                    }
                }
            }

            (AppState::Detail(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),
            (AppState::Detail(_), KeyCode::Char('x')) => self.run_selected_task().await?,

            (AppState::Running(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),

            // Output scrolling controls when output pane is visible (must come before regular navigation)
            (AppState::SequenceBuilder, KeyCode::Up)
                if modifiers.contains(KeyModifiers::SHIFT) && self.show_output_pane =>
            {
                self.scroll_output_up(3);
            }
            (AppState::SequenceBuilder, KeyCode::Down)
                if modifiers.contains(KeyModifiers::SHIFT) && self.show_output_pane =>
            {
                self.scroll_output_down(3);
            }
            (AppState::SequenceBuilder, KeyCode::Char('u')) if self.show_output_pane => {
                self.scroll_output_half_page_up();
            }
            (AppState::SequenceBuilder, KeyCode::Char('d')) if self.show_output_pane => {
                self.scroll_output_half_page_down();
            }

            // Sequence Builder controls
            (AppState::SequenceBuilder, KeyCode::Down | KeyCode::Char('j')) => {
                self.select_next();
                self.ensure_selected_task_visible(self.current_visible_height);
            }
            (AppState::SequenceBuilder, KeyCode::Up | KeyCode::Char('k')) => {
                self.select_previous();
                self.ensure_selected_task_visible(self.current_visible_height);
            }
            (AppState::SequenceBuilder, KeyCode::PageDown) => {
                let visible_height = self.current_visible_height.max(1);
                self.scroll_down(visible_height, visible_height);
                // Don't change selected task - let it go out of view if needed
            }
            (AppState::SequenceBuilder, KeyCode::PageUp) => {
                let visible_height = self.current_visible_height.max(1);
                self.scroll_up(visible_height);
                // Don't change selected task - let it go out of view if needed
            }
            (AppState::SequenceBuilder, KeyCode::Char('1')) => {
                self.toggle_current_task_step(0).await?
            }
            (AppState::SequenceBuilder, KeyCode::Char('2')) => {
                self.toggle_current_task_step(1).await?
            }
            (AppState::SequenceBuilder, KeyCode::Char('3')) => {
                self.toggle_current_task_step(2).await?
            }
            (AppState::SequenceBuilder, KeyCode::Enter) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::Sequence(SequenceEvent::RunSequence));
            }
            (AppState::SequenceBuilder, KeyCode::Char('c') | KeyCode::Char('C')) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::Sequence(SequenceEvent::ClearSequence));
            }
            (AppState::SequenceBuilder, KeyCode::Char('a')) => {
                let _ = self
                    .event_tx
                    .send(AppEvent::Sequence(SequenceEvent::AddAsTask));
            }
            (AppState::SequenceBuilder, KeyCode::Char('x')) => self.run_current_task().await?,
            (AppState::SequenceBuilder, KeyCode::Char('e')) => self.edit_current_task().await?,
            (AppState::SequenceBuilder, KeyCode::Tab) => self.show_current_task_content().await?,
            (AppState::SequenceBuilder, KeyCode::Esc | KeyCode::Char('b')) => {
                if self.show_output_pane && !self.task_running {
                    // Close output pane if task is finished
                    self.show_output_pane = false;
                    self.task_output.clear();
                    self.task_output_rx = None;
                    self.output_scroll_offset = 0;
                } else {
                    self.state = AppState::SequenceBuilder;
                }
            }

            _ => {}
        }
        Ok(())
    }

    pub async fn handle_mouse_scroll(&mut self, direction: ScrollDirection) -> Result<()> {
        match &self.state {
            AppState::SequenceBuilder => {
                // Use the current visible height that was calculated during the last UI render
                // This automatically accounts for whether the output pane is open or not
                let visible_height = if self.current_visible_height > 0 {
                    self.current_visible_height
                } else {
                    DEFAULT_SCROLL_AMT // Fallback if not yet calculated
                };

                match direction {
                    ScrollDirection::Up => {
                        self.scroll_up(3); // Scroll 3 lines at a time
                    }
                    ScrollDirection::Down => {
                        self.scroll_down(3, visible_height); // Scroll 3 lines at a time
                    }
                }
            }
            _ => {
                // Handle scrolling in other states if needed
            }
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
            let visible_task_index = (row - table_start_row - 2) as usize;
            let actual_task_index = self.scroll_offset + visible_task_index;
            if actual_task_index < self.tasks.len() {
                // Update selected task
                self.selected_task = actual_task_index;

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
                            self.toggle_current_task_step(step).await?;
                            return Ok(());
                        }
                    }
                }

                // Check actions column (last column)
                if let Some(actions_rect) = table_layout.column_rects.last() {
                    if col >= actions_rect.x && col < actions_rect.x + actions_rect.width {
                        let action_layout = ActionButtonLayout::new(actions_rect);
                        let relative_col = col - actions_rect.x;

                        if let Some(button) = action_layout.get_button_at_position(relative_col) {
                            match button {
                                ActionButton::Run => self.run_current_task().await?,
                                ActionButton::Cat => self.show_current_task_content().await?,
                                ActionButton::Edit => self.edit_current_task().await?,
                            }
                        }
                    }
                }
            }
        }
        // Check for sequence control buttons in the title area
        else if row == table_start_row {
            if let Some((controls_start_col, controls_width)) =
                self.calculate_sequence_controls_position(table_layout)
            {
                if col >= controls_start_col && col < controls_start_col + controls_width as u16 {
                    let relative_col = col - controls_start_col;
                    let sequence_layout = SequenceButtonLayout::new(0);

                    if let Some(button) = sequence_layout.get_button_at_position(relative_col) {
                        match button {
                            crate::ui::button_layout::SequenceButton::RunSequence => {
                                let _ = self
                                    .event_tx
                                    .send(AppEvent::Sequence(SequenceEvent::RunSequence));
                            }
                            crate::ui::button_layout::SequenceButton::AddAsTask => {
                                let _ = self
                                    .event_tx
                                    .send(AppEvent::Sequence(SequenceEvent::AddAsTask));
                            }
                            crate::ui::button_layout::SequenceButton::Clear => {
                                let _ = self
                                    .event_tx
                                    .send(AppEvent::Sequence(SequenceEvent::ClearSequence));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn handle_mouse_move(&mut self, row: u16, col: u16) -> Result<()> {
        match &self.state {
            AppState::SequenceBuilder => {
                self.handle_sequence_builder_hover(row, col)?;
            }
            _ => {
                // Clear hover state in other states
                self.button_hover_state = None;
            }
        }
        Ok(())
    }

    fn handle_sequence_builder_hover(&mut self, row: u16, col: u16) -> Result<()> {
        // Early return if no table layout
        let Some(table_layout) = &self.table_layout else {
            if self.button_hover_state.is_some() {
                self.button_hover_state = None;
            }
            return Ok(());
        };

        let table_start_row = table_layout.table_area.y;
        let mut new_hover_state = None;

        // Check for sequence button hover in the title area
        if row == table_start_row {
            if let Some((controls_start_col, controls_width)) =
                self.calculate_sequence_controls_position(table_layout)
            {
                if col >= controls_start_col && col < controls_start_col + controls_width as u16 {
                    let relative_col = col - controls_start_col;
                    let sequence_layout = SequenceButtonLayout::new(0);

                    if let Some(button) = sequence_layout.get_button_at_position(relative_col) {
                        new_hover_state = Some(ButtonHoverState::new(
                            ButtonType::Sequence(button),
                            row,
                            col,
                        ));
                    }
                }
            }
        }
        // Check for action button hover in task rows
        else if row >= table_start_row + 2 {
            let visible_task_index = (row - table_start_row - 2) as usize;
            let actual_task_index = self.scroll_offset + visible_task_index;

            if actual_task_index < self.tasks.len() {
                // Check actions column (last column)
                if let Some(actions_rect) = table_layout.column_rects.last() {
                    if col >= actions_rect.x && col < actions_rect.x + actions_rect.width {
                        let action_layout = ActionButtonLayout::new(actions_rect);
                        let relative_col = col - actions_rect.x;

                        if let Some(button) = action_layout.get_button_at_position(relative_col) {
                            new_hover_state = Some(ButtonHoverState::new(
                                ButtonType::Action {
                                    button,
                                    task_index: actual_task_index,
                                },
                                row,
                                col,
                            ));
                        }
                    }
                }
            }
        }

        // Only update hover state if it actually changed
        if self.button_hover_state != new_hover_state {
            self.button_hover_state = new_hover_state;
        }

        Ok(())
    }

    fn calculate_sequence_controls_position(
        &self,
        table_area: &crate::ui::sequence_builder::TableLayout,
    ) -> Option<(u16, usize)> {
        // Pre-calculate constants to avoid repeated allocations
        static CONTROLS_TEXT_LEN: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let controls_width = *CONTROLS_TEXT_LEN.get_or_init(|| {
            format!("{RUN_SEQUENCE_BUTTON_TEXT} {ADD_AS_TASK_BUTTON_TEXT} {CLEAR_BUTTON_TEXT}")
                .len()
        });

        let title_text_len = if self.tasks.len() > self.current_visible_height {
            let total_tasks = self.tasks.len();
            let start_task = self.scroll_offset + 1;
            let end_task = (self.scroll_offset + self.current_visible_height).min(total_tasks);
            // Calculate length without allocating the full string
            APP_TITLE.len() + format!(" ({start_task}-{end_task}/{total_tasks})").len()
        } else {
            APP_TITLE.len()
        };

        let title_offset = 3 + title_text_len; // Border + space + title + space
        let available_width = table_area.table_area.width as usize;

        let controls_start = if title_offset + controls_width + 2 <= available_width {
            available_width - controls_width - 2
        } else {
            title_offset + 2
        };

        let controls_start_col = table_area.table_area.x + controls_start as u16;

        // Return start column and width if valid
        if controls_start_col + controls_width as u16
            <= table_area.table_area.x + table_area.table_area.width
        {
            Some((controls_start_col, controls_width))
        } else {
            None
        }
    }
}
