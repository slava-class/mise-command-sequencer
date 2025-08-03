use anyhow::Result;
use std::{collections::VecDeque, time::Instant};
use tokio::sync::mpsc;

use crate::mise::MiseClient;
use crate::models::{AppEvent, AppState, MiseTask, MiseTaskInfo, SequenceState};
use crate::ui::sequence_builder::TableLayout;

pub mod event_handlers;
pub mod sequence_management;
pub mod task_management;

pub struct App {
    pub client: MiseClient,
    pub tasks: Vec<MiseTask>,
    pub selected_task: usize,
    pub state: AppState,
    pub task_info: Option<MiseTaskInfo>,
    pub task_output: VecDeque<String>,
    pub should_quit: bool,
    pub last_updated: Instant,
    pub event_tx: mpsc::UnboundedSender<AppEvent>,
    pub task_output_rx: Option<mpsc::UnboundedReceiver<String>>,
    pub sequence_state: SequenceState,
    pub table_layout: Option<TableLayout>,
}

impl App {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            client: MiseClient::new(),
            tasks: vec![],
            selected_task: 0,
            state: AppState::SequenceBuilder,
            task_info: None,
            task_output: VecDeque::new(),
            should_quit: false,
            last_updated: Instant::now(),
            event_tx,
            task_output_rx: None,
            sequence_state: SequenceState::new(3),
            table_layout: None,
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
                let _ = self.event_tx.send(AppEvent::TaskOutput(output));
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

    pub fn back_to_list(&mut self) {
        self.state = AppState::List;
        self.task_info = None;
        self.task_output.clear();
        self.task_output_rx = None;
    }
}
