use super::mise_task::{MiseTask, MiseTaskInfo};
use super::sequence::SequenceEvent;
use ratatui::crossterm::event::{KeyCode, MouseButton};

#[derive(Debug, Clone)]
pub enum AppEvent {
    #[allow(dead_code)]
    Quit,
    KeyPress(KeyCode),
    MouseClick {
        button: MouseButton,
        row: u16,
        col: u16,
    },
    TasksRefreshed(Vec<MiseTask>),
    TaskInfoLoaded(Box<MiseTaskInfo>),
    TaskOutput(String),
    TaskCompleted,
    Tick,
    Sequence(SequenceEvent),
}
