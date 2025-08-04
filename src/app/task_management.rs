use anyhow::Result;
use tokio::sync::mpsc;

use super::App;
use crate::models::{AppEvent, AppState};

impl App {
    pub async fn refresh_tasks(&mut self) -> Result<()> {
        let client = self.client.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            match client.list_tasks().await {
                Ok(tasks) => {
                    if event_tx.send(AppEvent::TasksRefreshed(tasks)).is_err() {
                        eprintln!("Warning: Failed to send TasksRefreshed event");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to refresh tasks: {e}");
                }
            }
        });

        Ok(())
    }

    pub async fn run_selected_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            let (output_tx, output_rx) = mpsc::unbounded_channel();
            self.task_output_rx = Some(output_rx);
            self.task_output.clear();
            self.show_output_pane = true;
            self.task_running = true;

            let client = self.client.clone();
            let task_name = task.name.clone();
            let event_tx = self.event_tx.clone();

            self.state = AppState::Running(task_name.clone());

            let handle = tokio::spawn(async move {
                if let Err(e) = client.run_task(&task_name, &[], output_tx).await {
                    eprintln!("Failed to run task: {e}");
                }
                if event_tx.send(AppEvent::TaskCompleted).is_err() {
                    eprintln!("Warning: Failed to send TaskCompleted event");
                }
            });

            self.running_task_handle = Some(handle);
        }
        Ok(())
    }
}
