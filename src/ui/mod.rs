use ratatui::prelude::*;

use crate::app::App;
use crate::models::AppState;

pub mod button_layout;
pub mod sequence_builder;
pub mod task_detail;
pub mod task_running;

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
        match &self.state {
            AppState::Detail(task_name) => task_detail::draw_task_detail(self, f, task_name),
            AppState::Running(task_name) => task_running::draw_task_running(self, f, task_name),
            AppState::SequenceBuilder => sequence_builder::draw_sequence_builder(self, f),
        }
    }
}
