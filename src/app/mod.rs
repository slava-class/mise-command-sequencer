use anyhow::Result;
use ratatui::layout::Rect;
use std::{collections::VecDeque, time::Instant};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

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
    pub running_task_handle: Option<JoinHandle<()>>,
    pub button_hover_state: Option<ButtonHoverState>,
    pub current_visible_height: usize,
    pub output_scroll_offset: usize,
    pub current_output_visible_height: usize,
    pub output_follow_mode: bool,
    pub pending_delete_task: Option<String>,
    pub delete_dialog_area: Option<Rect>,
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
            running_task_handle: None,
            button_hover_state: None,
            current_visible_height: 0,
            output_scroll_offset: 0,
            current_output_visible_height: 0,
            output_follow_mode: true,
            pending_delete_task: None,
            delete_dialog_area: None,
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
        self.running_task_handle = None;
        self.button_hover_state = None;
        self.current_visible_height = 0;
        self.output_scroll_offset = 0;
        self.current_output_visible_height = 0;
        self.output_follow_mode = true;
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
            description: Some("desc".to_string()),
            source: "source".to_string(),
            file: None,
            dir: None,
            hide: None,
            alias: None,
            run: None,
            depends: None,
            env: None,
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
}
