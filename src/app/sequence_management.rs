use anyhow::Result;
use tokio::sync::mpsc;

use super::App;
use crate::models::{AppEvent, SequenceEvent};

impl App {
    pub async fn handle_sequence_event(&mut self, event: SequenceEvent) -> Result<()> {
        match event {
            SequenceEvent::ToggleStep(task_name, step) => {
                let current_enabled = self
                    .sequence_state
                    .is_task_enabled_for_step(&task_name, step);
                // Toggle: if currently enabled, disable; if not enabled, enable
                self.sequence_state
                    .set_task_step(&task_name, step, !current_enabled);
            }
            SequenceEvent::RunSequence => {
                self.start_sequence_execution()?;
            }
            SequenceEvent::AddAsTask => {
                self.add_sequence_as_task().await?;
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

    pub async fn toggle_current_task_step(&mut self, step: usize) -> Result<()> {
        if let Some(selected_task) = self.tasks.get(self.selected_task) {
            let task_name = selected_task.name.clone();
            let event = SequenceEvent::ToggleStep(task_name, step);
            self.handle_sequence_event(event).await?;
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
                        if output_tx
                            .send(format!("Task '{task_name}' failed: {e}"))
                            .is_err()
                        {
                            eprintln!("Warning: Failed to send task failure message");
                        }
                        all_success = false;
                        break;
                    }
                }
            }

            if all_success {
                if event_tx
                    .send(AppEvent::Sequence(SequenceEvent::StepCompleted))
                    .is_err()
                {
                    eprintln!("Warning: Failed to send StepCompleted event");
                }
            } else if event_tx
                .send(AppEvent::Sequence(SequenceEvent::SequenceFailed(
                    "One or more tasks failed".to_string(),
                )))
                .is_err()
            {
                eprintln!("Warning: Failed to send SequenceFailed event");
            }
        });

        self.running_task_handle = Some(handle);

        Ok(())
    }

    async fn add_sequence_as_task(&mut self) -> Result<()> {
        if let Some(command) = self.sequence_state.generate_mise_task_command() {
            // Generate a task name based on current timestamp
            let task_name = format!("sequence-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));

            // Add task directly to mise.toml using TOML parsing
            let add_result = self.add_task_to_mise_toml(&task_name, &command).await;

            // Show feedback to user
            self.task_output.clear();
            self.show_output_pane = true;
            self.task_running = false;

            match add_result {
                Ok(()) => {
                    self.task_output
                        .push_back(format!("✓ Created task '{task_name}' successfully!"));
                    self.task_output.push_back(format!("Command: {command}"));

                    // Refresh task list to show the new task
                    self.refresh_tasks().await?;
                }
                Err(e) => {
                    self.task_output
                        .push_back(format!("✗ Error adding task to mise.toml: {e}"));
                }
            }
        } else {
            // No tasks enabled
            self.task_output.clear();
            self.show_output_pane = true;
            self.task_running = false;
            self.task_output
                .push_back("No tasks enabled in sequence. Enable some tasks first!".to_string());
        }
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
                if event_tx.send(AppEvent::TaskCompleted).is_err() {
                    eprintln!("Warning: Failed to send TaskCompleted event");
                }
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
                    let run_string = match run_config {
                        serde_json::Value::Array(arr) => {
                            // Convert array of strings to joined string
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<&str>>()
                                .join(" ")
                        }
                        serde_json::Value::String(s) => s.clone(),
                        _ => serde_json::to_string_pretty(run_config).unwrap_or_default(),
                    };
                    self.task_output.push_back(run_string);
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

    async fn add_task_to_mise_toml(&self, task_name: &str, command: &str) -> Result<()> {
        use std::fs;
        use toml::Value;

        // Read the current mise.toml file
        let toml_path = "mise.toml";
        let toml_content = fs::read_to_string(toml_path)
            .map_err(|e| anyhow::anyhow!("Failed to read mise.toml: {}", e))?;

        // Parse the TOML content
        let mut toml_value: Value = toml::from_str(&toml_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse mise.toml: {}", e))?;

        // Ensure the [tasks] table exists
        let tasks_table = toml_value
            .as_table_mut()
            .and_then(|table| table.entry("tasks").or_insert_with(|| Value::Table(toml::Table::new())).as_table_mut())
            .ok_or_else(|| anyhow::anyhow!("Could not access or create tasks table"))?;

        // Add the new task
        tasks_table.insert(task_name.to_string(), Value::String(command.to_string()));

        // Serialize back to TOML string
        let updated_toml = toml::to_string(&toml_value)
            .map_err(|e| anyhow::anyhow!("Failed to serialize TOML: {}", e))?;

        // Write back to file
        fs::write(toml_path, updated_toml)
            .map_err(|e| anyhow::anyhow!("Failed to write mise.toml: {}", e))?;

        Ok(())
    }
}
