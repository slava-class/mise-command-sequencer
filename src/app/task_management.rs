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
                    let _ = event_tx.send(AppEvent::TasksRefreshed(tasks));
                }
                Err(e) => {
                    eprintln!("Failed to refresh tasks: {e}");
                }
            }
        });

        Ok(())
    }

    pub async fn show_task_detail(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            let client = self.client.clone();
            let task_name = task.name.clone();
            let event_tx = self.event_tx.clone();

            self.state = AppState::Detail(task_name.clone());

            tokio::spawn(async move {
                match client.get_task_info(&task_name).await {
                    Ok(info) => {
                        let _ = event_tx.send(AppEvent::TaskInfoLoaded(Box::new(info)));
                    }
                    Err(e) => {
                        eprintln!("Failed to get task info: {e}");
                    }
                }
            });
        }
        Ok(())
    }

    pub async fn run_selected_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            let (output_tx, output_rx) = mpsc::unbounded_channel();
            self.task_output_rx = Some(output_rx);
            self.task_output.clear();

            let client = self.client.clone();
            let task_name = task.name.clone();
            let event_tx = self.event_tx.clone();

            self.state = AppState::Running(task_name.clone());

            tokio::spawn(async move {
                if let Err(e) = client.run_task(&task_name, &[], output_tx).await {
                    eprintln!("Failed to run task: {e}");
                }
                let _ = event_tx.send(AppEvent::TaskCompleted);
            });
        }
        Ok(())
    }
}
