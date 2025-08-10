use anyhow::Result;
use ratatui::layout::Rect;
use std::{collections::VecDeque, time::Instant};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tui_input::Input;

use crate::mise::MiseClient;
use crate::models::{AppEvent, AppState, MiseTask, MiseTaskInfo, SequenceState};
use crate::ui::button_layout::ButtonHoverState;
use crate::ui::sequence_builder::TableLayout;

pub mod event_handlers;
pub mod sequence_management;
pub mod task_management;

pub struct App {
    pub client: MiseClient,
    pub tasks: Vec<MiseTask>,
    pub selected_task: usize,
    pub scroll_offset: usize,
    pub state: AppState,
    pub task_info: Option<MiseTaskInfo>,
    pub task_output: VecDeque<String>,
    pub should_quit: bool,
    pub last_updated: Instant,
    pub event_tx: mpsc::UnboundedSender<AppEvent>,
    pub task_output_rx: Option<mpsc::UnboundedReceiver<String>>,
    pub sequence_state: SequenceState,
    pub table_layout: Option<TableLayout>,
    pub show_output_pane: bool,
    pub task_running: bool,
    pub running_task_name: Option<String>,
    pub running_task_handle: Option<JoinHandle<()>>,
    pub button_hover_state: Option<ButtonHoverState>,
    pub current_visible_height: usize,
    pub output_scroll_offset: usize,
    pub current_output_visible_height: usize,
    pub output_follow_mode: bool,
    pub pending_delete_task: Option<String>,
    pub delete_dialog_area: Option<Rect>,
    pub rename_input: Option<Input>,
    pub original_task_name: Option<String>,
}

impl App {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            client: MiseClient::new(),
            tasks: vec![],
            selected_task: 0,
            scroll_offset: 0,
            state: AppState::SequenceBuilder,
            task_info: None,
            task_output: VecDeque::new(),
            should_quit: false,
            last_updated: Instant::now(),
            event_tx,
            task_output_rx: None,
            sequence_state: SequenceState::new(3),
            table_layout: None,
            show_output_pane: false,
            task_running: false,
            running_task_name: None,
            running_task_handle: None,
            button_hover_state: None,
            current_visible_height: 0,
            output_scroll_offset: 0,
            current_output_visible_height: 0,
            output_follow_mode: true,
            pending_delete_task: None,
            delete_dialog_area: None,
            rename_input: None,
            original_task_name: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.refresh_tasks().await?;
        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    // Check for new task output
    pub fn poll_task_output(&mut self) {
        if let Some(ref mut rx) = self.task_output_rx {
            while let Ok(output) = rx.try_recv() {
                if self.event_tx.send(AppEvent::TaskOutput(output)).is_err() {
                    eprintln!("Warning: Failed to send TaskOutput event");
                }
            }
        }
    }

    pub fn select_next(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = (self.selected_task + 1) % self.tasks.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = if self.selected_task > 0 {
                self.selected_task - 1
            } else {
                self.tasks.len() - 1
            };
        }
    }

    pub fn ensure_selected_task_visible(&mut self, visible_height: usize) {
        if self.tasks.is_empty() || visible_height == 0 {
            return;
        }

        // Ensure selected task is within visible range
        if self.selected_task < self.scroll_offset {
            // Selected task is above visible area, scroll up
            self.scroll_offset = self.selected_task;
        } else if self.selected_task >= self.scroll_offset + visible_height {
            // Selected task is below visible area, scroll down
            self.scroll_offset = self.selected_task.saturating_sub(visible_height - 1);
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize, visible_height: usize) {
        if self.tasks.is_empty() {
            return;
        }

        let max_scroll = self.tasks.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + lines).min(max_scroll);
    }

    pub fn get_visible_tasks(&self, visible_height: usize) -> (Vec<&MiseTask>, usize) {
        if self.tasks.is_empty() {
            return (vec![], 0);
        }

        let end = (self.scroll_offset + visible_height).min(self.tasks.len());
        let visible_tasks = self.tasks[self.scroll_offset..end].iter().collect();
        let selected_in_visible = if self.selected_task >= self.scroll_offset
            && self.selected_task < self.scroll_offset + visible_height
        {
            Some(self.selected_task - self.scroll_offset)
        } else {
            None
        };

        (visible_tasks, selected_in_visible.unwrap_or(0))
    }

    pub fn back_to_list(&mut self) {
        self.state = AppState::SequenceBuilder;
        self.task_info = None;
        self.task_output.clear();
        self.task_output_rx = None;
        self.show_output_pane = false;
        self.task_running = false;
        self.running_task_name = None;
        self.running_task_handle = None;
        self.button_hover_state = None;
        self.current_visible_height = 0;
        self.output_scroll_offset = 0;
        self.current_output_visible_height = 0;
        self.output_follow_mode = true;
        self.rename_input = None;
        self.original_task_name = None;
    }

    pub fn scroll_output_up(&mut self, lines: usize) {
        self.output_scroll_offset = self.output_scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_output_down(&mut self, lines: usize) {
        let visible_height = self.current_output_visible_height;
        if self.task_output.len() <= visible_height || visible_height == 0 {
            return;
        }

        let max_scroll = self.task_output.len().saturating_sub(visible_height);
        self.output_scroll_offset = (self.output_scroll_offset + lines).min(max_scroll);
    }

    pub fn scroll_output_half_page_up(&mut self) {
        let half_page = self.current_output_visible_height / 2;
        self.scroll_output_up(half_page.max(1));
    }

    pub fn scroll_output_half_page_down(&mut self) {
        let half_page = self.current_output_visible_height / 2;
        self.scroll_output_down(half_page.max(1));
    }

    pub fn auto_scroll_output_to_bottom(&mut self) {
        let visible_height = self.current_output_visible_height;
        if self.task_output.len() > visible_height && visible_height > 0 {
            self.output_scroll_offset = self.task_output.len() - visible_height;
        } else {
            self.output_scroll_offset = 0;
        }
    }

    pub fn scroll_output_to_top(&mut self) {
        self.output_scroll_offset = 0;
    }

    pub fn scroll_output_to_bottom(&mut self) {
        self.auto_scroll_output_to_bottom();
    }

    pub fn toggle_output_follow_mode(&mut self) {
        self.output_follow_mode = !self.output_follow_mode;
        // If we're enabling follow mode, immediately scroll to bottom
        if self.output_follow_mode {
            self.auto_scroll_output_to_bottom();
        }
    }

    pub async fn start_rename_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            // Initialize rename mode
            self.state = AppState::Renaming(task.name.clone());
            self.original_task_name = Some(task.name.clone());
            self.rename_input = Some(Input::new(task.name.clone()));
        }
        Ok(())
    }

    pub async fn save_rename(&mut self) -> Result<()> {
        let new_name = if let Some(ref input) = self.rename_input {
            input.value().trim().to_string()
        } else {
            self.cancel_rename();
            return Ok(());
        };

        // Validate the new name
        if new_name.is_empty() {
            // Cancel if empty name
            self.cancel_rename();
            return Ok(());
        }

        let original_name = if let Some(ref name) = self.original_task_name {
            name.clone()
        } else {
            self.cancel_rename();
            return Ok(());
        };

        if new_name != original_name {
            // Update the task name via MiseClient
            match self.client.rename_task(&original_name, &new_name).await {
                Ok(()) => {
                    // Refresh the task list to reflect changes
                    self.refresh_tasks().await?;

                    // Check if the name was actually changed due to conflicts
                    let final_name = self
                        .tasks
                        .iter()
                        .find(|task| task.name.starts_with(&new_name))
                        .map(|task| task.name.as_str())
                        .unwrap_or(&new_name);

                    if final_name != new_name {
                        self.task_output.push_back(format!("Task '{original_name}' renamed to '{final_name}' (name adjusted to avoid conflicts)"));
                    } else {
                        self.task_output
                            .push_back(format!("Task '{original_name}' renamed to '{final_name}'"));
                    }
                    self.show_output_pane = true;
                }
                Err(e) => {
                    self.task_output
                        .push_back(format!("Failed to rename task '{original_name}': {e}"));
                    self.show_output_pane = true;
                }
            }
        }

        // Exit rename mode
        self.cancel_rename();
        Ok(())
    }

    pub fn cancel_rename(&mut self) {
        self.state = AppState::SequenceBuilder;
        self.rename_input = None;
        self.original_task_name = None;
    }

    pub fn is_task_running(&self, task_name: &str) -> bool {
        self.running_task_name
            .as_ref()
            .is_some_and(|name| name == task_name)
    }

    pub fn is_any_task_running(&self) -> bool {
        self.task_running || self.sequence_state.is_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MiseTask;

    fn create_test_app() -> App {
        let (tx, _rx) = mpsc::unbounded_channel();
        App::new(tx)
    }

    #[test]
    fn test_app_new() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let app = App::new(tx);

        assert_eq!(app.tasks.len(), 0);
        assert_eq!(app.selected_task, 0);
        assert_eq!(app.scroll_offset, 0);
        assert_eq!(app.state, AppState::SequenceBuilder);
        assert!(app.task_info.is_none());
        assert_eq!(app.task_output.len(), 0);
        assert!(!app.should_quit);
        assert!(app.task_output_rx.is_none());
        assert_eq!(app.sequence_state.num_steps, 3);
        assert!(app.table_layout.is_none());
        assert!(app.button_hover_state.is_none());
        assert_eq!(app.current_visible_height, 0);
    }

    #[test]
    fn test_should_quit() {
        let mut app = create_test_app();

        assert!(!app.should_quit());

        app.should_quit = true;
        assert!(app.should_quit());
    }

    #[test]
    fn test_select_next_empty_list() {
        let mut app = create_test_app();

        app.select_next();
        assert_eq!(app.selected_task, 0);
    }

    #[test]
    fn test_select_next_with_tasks() {
        let mut app = create_test_app();
        app.tasks = vec![
            MiseTask::new("task1".to_string(), "source1".to_string()),
            MiseTask::new("task2".to_string(), "source2".to_string()),
            MiseTask::new("task3".to_string(), "source3".to_string()),
        ];

        assert_eq!(app.selected_task, 0);

        app.select_next();
        assert_eq!(app.selected_task, 1);

        app.select_next();
        assert_eq!(app.selected_task, 2);

        // Test wraparound
        app.select_next();
        assert_eq!(app.selected_task, 0);
    }

    #[test]
    fn test_select_previous_empty_list() {
        let mut app = create_test_app();

        app.select_previous();
        assert_eq!(app.selected_task, 0);
    }

    #[test]
    fn test_select_previous_with_tasks() {
        let mut app = create_test_app();
        app.tasks = vec![
            MiseTask::new("task1".to_string(), "source1".to_string()),
            MiseTask::new("task2".to_string(), "source2".to_string()),
            MiseTask::new("task3".to_string(), "source3".to_string()),
        ];
        app.selected_task = 1;

        app.select_previous();
        assert_eq!(app.selected_task, 0);

        // Test wraparound from 0
        app.select_previous();
        assert_eq!(app.selected_task, 2);

        app.select_previous();
        assert_eq!(app.selected_task, 1);
    }

    #[test]
    fn test_select_single_task() {
        let mut app = create_test_app();
        app.tasks = vec![MiseTask::new("task1".to_string(), "source1".to_string())];

        app.select_next();
        assert_eq!(app.selected_task, 0);

        app.select_previous();
        assert_eq!(app.selected_task, 0);
    }

    #[test]
    fn test_back_to_list() {
        let mut app = create_test_app();

        // Set up some state
        app.state = AppState::Detail("test".to_string());
        app.task_info = Some(MiseTaskInfo {
            name: "test".to_string(),
            aliases: Vec::new(),
            description: "desc".to_string(),
            source: "source".to_string(),
            depends: Vec::new(),
            depends_post: Vec::new(),
            wait_for: Vec::new(),
            env: Vec::new(),
            dir: None,
            hide: false,
            raw: false,
            sources: Vec::new(),
            outputs: Vec::new(),
            shell: None,
            quiet: false,
            silent: false,
            tools: std::collections::HashMap::new(),
            run: Vec::new(),
            file: None,
            usage_spec: serde_json::Value::Null,
        });
        app.task_output.push_back("output1".to_string());
        app.task_output.push_back("output2".to_string());

        app.back_to_list();

        assert_eq!(app.state, AppState::SequenceBuilder);
        assert!(app.task_info.is_none());
        assert_eq!(app.task_output.len(), 0);
        assert!(app.task_output_rx.is_none());
    }

    #[test]
    fn test_poll_task_output_no_receiver() {
        let mut app = create_test_app();

        // Should not panic when no receiver is set
        app.poll_task_output();
    }

    #[test]
    fn test_ansi_color_bleeding_prevention() {
        let mut app = create_test_app();

        // Simulate colored STDOUT/STDERR output that could cause bleeding
        let colored_stdout = "STDOUT: \x1b[32mSuccess message";
        let colored_stderr = "STDERR: \x1b[31mError message";
        let plain_text = "Plain text line";

        // Add the output lines directly to simulate the event handling
        app.task_output.push_back(colored_stdout.to_string());
        app.task_output.push_back(colored_stderr.to_string());
        app.task_output.push_back(plain_text.to_string());

        // Verify the lines were added
        assert_eq!(app.task_output.len(), 3);
        assert_eq!(app.task_output[0], colored_stdout);
        assert_eq!(app.task_output[1], colored_stderr);
        assert_eq!(app.task_output[2], plain_text);

        // The actual bleeding prevention is tested in the UI layer
        // through the ensure_ansi_reset function tests
    }

    #[test]
    fn test_multiple_colored_lines_sequence() {
        let mut app = create_test_app();

        // Simulate a sequence of colored lines like what might come from a build process
        let lines = vec![
            "STDOUT: \x1b[36mBuilding project...",
            "STDOUT: \x1b[32m✓ Compiled successfully",
            "STDERR: \x1b[33mWarning: deprecated function",
            "STDOUT: \x1b[32m✓ Tests passed",
            "Plain summary line",
        ];

        for line in &lines {
            app.task_output.push_back(line.to_string());
        }

        assert_eq!(app.task_output.len(), 5);
        for (i, expected_line) in lines.iter().enumerate() {
            assert_eq!(app.task_output[i], *expected_line);
        }
    }
}
