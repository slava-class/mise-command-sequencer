use anyhow::Result;
use tokio::sync::mpsc;

use super::App;
use crate::models::{AppEvent, SequenceEvent};

impl App {
    pub fn handle_sequence_event(&mut self, event: SequenceEvent) -> Result<()> {
        match event {
            SequenceEvent::ToggleStep(task_name, step) => {
                let current_enabled = self
                    .sequence_state
                    .is_task_enabled_for_step(&task_name, step);
                self.sequence_state
                    .set_task_step(&task_name, step, !current_enabled);
            }
            SequenceEvent::RunSequence => {
                self.start_sequence_execution()?;
            }
            SequenceEvent::ClearSequence => {
                self.sequence_state.clear_all();
            }
            SequenceEvent::StepCompleted => {
                if !self.sequence_state.advance_step() {
                    let _ = self
                        .event_tx
                        .send(AppEvent::Sequence(SequenceEvent::SequenceCompleted));
                } else {
                    self.execute_current_step()?;
                }
            }
            SequenceEvent::SequenceCompleted => {
                self.sequence_state.reset_execution();
                self.task_running = false;
            }
            SequenceEvent::SequenceFailed(error) => {
                self.task_output
                    .push_back(format!("Sequence failed: {error}"));
                self.sequence_state.reset_execution();
                self.task_running = false;
            }
        }
        Ok(())
    }

    pub fn toggle_current_task_step(&mut self, step: usize) -> Result<()> {
        if let Some(selected_task) = self.tasks.get(self.selected_task) {
            let task_name = selected_task.name.clone();
            let event = SequenceEvent::ToggleStep(task_name, step);
            self.handle_sequence_event(event)?;
        }
        Ok(())
    }

    fn start_sequence_execution(&mut self) -> Result<()> {
        if self.sequence_state.is_running {
            return Ok(());
        }

        self.sequence_state.start_execution();
        self.task_output.clear();
        self.show_output_pane = true;
        self.task_running = true;
        self.execute_current_step()?;
        Ok(())
    }

    fn execute_current_step(&mut self) -> Result<()> {
        if let Some(current_step) = self.sequence_state.current_step {
            let tasks_for_step = self.sequence_state.get_tasks_for_step(current_step);

            if tasks_for_step.is_empty() {
                // No tasks for this step, advance to next
                let _ = self
                    .event_tx
                    .send(AppEvent::Sequence(SequenceEvent::StepCompleted));
                return Ok(());
            }

            // Execute all tasks for this step sequentially
            self.execute_tasks_for_step(tasks_for_step)?;
        }
        Ok(())
    }

    fn execute_tasks_for_step(&mut self, tasks: Vec<String>) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let client = self.client.clone();

        // Create a channel for task output
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        self.task_output_rx = Some(output_rx);

        // Spawn task execution for all tasks in sequence
        let handle = tokio::spawn(async move {
            let mut all_success = true;

            for task_name in tasks {
                match client.run_task(&task_name, &[], output_tx.clone()).await {
                    Ok(_) => {
                        // Task completed successfully
                    }
                    Err(e) => {
                        let _ = output_tx.send(format!("Task '{task_name}' failed: {e}"));
                        all_success = false;
                        break;
                    }
                }
            }

            if all_success {
                let _ = event_tx.send(AppEvent::Sequence(SequenceEvent::StepCompleted));
            } else {
                let _ = event_tx.send(AppEvent::Sequence(SequenceEvent::SequenceFailed(
                    "One or more tasks failed".to_string(),
                )));
            }
        });

        self.running_task_handle = Some(handle);

        Ok(())
    }

    pub async fn run_current_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            let (output_tx, output_rx) = mpsc::unbounded_channel();
            self.task_output_rx = Some(output_rx);
            self.task_output.clear();
            self.show_output_pane = true;
            self.task_running = true;

            let client = self.client.clone();
            let task_name = task.name.clone();
            let event_tx = self.event_tx.clone();

            let handle = tokio::spawn(async move {
                if let Err(e) = client.run_task(&task_name, &[], output_tx).await {
                    eprintln!("Failed to run task: {e}");
                }
                let _ = event_tx.send(AppEvent::TaskCompleted);
            });

            self.running_task_handle = Some(handle);
        }
        Ok(())
    }

    pub async fn edit_current_task(&mut self) -> Result<()> {
        if let Some(selected_task) = self.tasks.get(self.selected_task) {
            self.edit_task(selected_task.name.clone()).await?;
        }
        Ok(())
    }

    pub async fn edit_task(&self, task_name: String) -> Result<()> {
        // Get task info to find the file path
        match self.client.get_task_info(&task_name).await {
            Ok(task_info) => {
                if let Some(file_path) = task_info.file {
                    // Open in VSCode
                    let _ = tokio::process::Command::new("code").arg(&file_path).spawn();
                } else {
                    // Try to open the current directory if no specific file
                    let _ = tokio::process::Command::new("code").arg(".").spawn();
                }
            }
            Err(_) => {
                // Fallback to opening current directory
                let _ = tokio::process::Command::new("code").arg(".").spawn();
            }
        }
        Ok(())
    }

    pub async fn show_current_task_content(&mut self) -> Result<()> {
        if let Some(selected_task) = self.tasks.get(self.selected_task) {
            self.show_task_content(selected_task.name.clone()).await?;
        }
        Ok(())
    }

    pub async fn show_task_content(&mut self, task_name: String) -> Result<()> {
        // Get task info and display its content
        match self.client.get_task_info(&task_name).await {
            Ok(task_info) => {
                self.task_output.clear();
                self.show_output_pane = true;
                self.task_running = false;
                self.task_output
                    .push_back(format!("=== Task: {task_name} ==="));

                if let Some(description) = &task_info.description {
                    self.task_output
                        .push_back(format!("Description: {description}"));
                }

                if let Some(file) = &task_info.file {
                    self.task_output.push_back(format!("File: {file}"));
                }

                if let Some(run_config) = &task_info.run {
                    self.task_output.push_back("Run configuration:".to_string());
                    self.task_output
                        .push_back(serde_json::to_string_pretty(run_config).unwrap_or_default());
                }

                if let Some(depends) = &task_info.depends {
                    self.task_output
                        .push_back(format!("Dependencies: {}", depends.join(", ")));
                }
            }
            Err(e) => {
                self.task_output.clear();
                self.show_output_pane = true;
                self.task_running = false;
                self.task_output
                    .push_back(format!("Failed to get task info: {e}"));
            }
        }
        Ok(())
    }
}
