use super::mise_task::{MiseTask, MiseTaskInfo};
use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub enum AppEvent {
    #[allow(dead_code)]
    Quit,
    KeyPress(KeyCode),
    TasksRefreshed(Vec<MiseTask>),
    TaskInfoLoaded(Box<MiseTaskInfo>),
    TaskOutput(String),
    TaskCompleted,
    Tick,
}
