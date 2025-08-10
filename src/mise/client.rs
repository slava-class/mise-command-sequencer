use anyhow::{Context, Result};
use log::{debug, error, info, trace, warn};
use std::{path::Path, process::Stdio};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc,
};

use crate::models::{MiseTask, MiseTaskInfo};

#[derive(Clone)]
pub struct MiseClient;

impl Default for MiseClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MiseClient {
    pub fn new() -> Self {
        Self
    }

    /// List all available mise tasks
    pub async fn list_tasks(&self) -> Result<Vec<MiseTask>> {
        debug!("Starting mise tasks ls --json command");

        let output = Command::new("mise")
            .args(["tasks", "ls", "--json"])
            .output()
            .await
            .context("Failed to execute mise tasks ls --json")?;

        if !output.status.success() {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            error!("mise tasks ls command failed with stderr: {stderr_str}");
            anyhow::bail!("mise command failed: {}", stderr_str);
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        debug!("Raw JSON output ({} bytes)", stdout_str.len());

        if log::log_enabled!(log::Level::Trace) {
            trace!("Full JSON output: {stdout_str}");
        }

        let tasks: Vec<MiseTask> = match serde_json::from_slice::<Vec<MiseTask>>(&output.stdout) {
            Ok(tasks) => {
                info!("Successfully parsed {} tasks", tasks.len());
                tasks
            }
            Err(e) => {
                error!("JSON parsing error: {e}");
                debug!("Full raw output for debugging: {stdout_str}");
                return Err(anyhow::anyhow!(
                    "Failed to parse mise tasks JSON output: {}",
                    e
                ));
            }
        };

        Ok(tasks)
    }

    /// Get detailed information about a specific task
    pub async fn get_task_info(&self, task_name: &str) -> Result<MiseTaskInfo> {
        debug!("Starting mise tasks info for task: {task_name}");

        let output = Command::new("mise")
            .args(["tasks", "info", task_name, "--json"])
            .output()
            .await
            .context("Failed to execute mise tasks info")?;

        if !output.status.success() {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            error!(
                "mise tasks info command failed for task '{task_name}' with stderr: {stderr_str}"
            );
            anyhow::bail!("mise command failed: {}", stderr_str);
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        debug!(
            "Raw JSON output for task '{}' ({} bytes)",
            task_name,
            stdout_str.len()
        );

        if log::log_enabled!(log::Level::Trace) {
            trace!("Full JSON output for task '{task_name}': {stdout_str}");
        }

        let task_info: MiseTaskInfo = match serde_json::from_slice::<MiseTaskInfo>(&output.stdout) {
            Ok(task_info) => {
                info!("Successfully parsed task info for '{task_name}'");
                task_info
            }
            Err(e) => {
                error!("JSON parsing error for task '{task_name}': {e}");
                debug!("Full raw output for debugging task '{task_name}': {stdout_str}");
                return Err(anyhow::anyhow!(
                    "Failed to parse mise task info JSON output for task '{}': {}",
                    task_name,
                    e
                ));
            }
        };

        Ok(task_info)
    }

    /// Run a specific mise task and stream output
    pub async fn run_task(
        &self,
        task_name: &str,
        args: &[String],
        output_tx: mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        let mut cmd = Command::new("mise");
        cmd.arg("run").arg(task_name);

        for arg in args {
            cmd.arg(arg);
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("FORCE_COLOR", "1")
            .env("CLICOLOR_FORCE", "1")
            .env("TERM", "xterm-256color");

        let mut child = cmd.spawn().context("Failed to spawn mise run command")?;

        let stdout = child
            .stdout
            .take()
            .context("Failed to capture stdout from mise command")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to capture stderr from mise command")?;

        // Spawn tasks to read stdout and stderr
        let output_tx_clone = output_tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if output_tx_clone.send(format!("STDOUT: {line}")).is_err() {
                    break;
                }
            }
        });

        let output_tx_clone = output_tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if output_tx_clone.send(format!("STDERR: {line}")).is_err() {
                    break;
                }
            }
        });

        // Wait for the process to complete
        let status = child.wait().await?;

        let final_message = if status.success() {
            format!("Task '{task_name}' completed successfully")
        } else {
            format!(
                "Task '{}' failed with exit code: {:?}",
                task_name,
                status.code()
            )
        };

        if output_tx.send(final_message).is_err() {
            eprintln!("Warning: Failed to send task completion message");
        }

        Ok(())
    }

    /// Delete a mise task
    pub async fn delete_task(&self, task_name: &str) -> Result<()> {
        // First, get task info to determine if it's file-based or config-based
        let task_info = self.get_task_info(task_name).await?;

        let source = &task_info.source;
        if source.ends_with(".toml") {
            // Config-based task - remove from mise.toml
            self.delete_task_from_config(source, task_name).await?;
        } else {
            // File-based task - delete the file
            self.delete_task_file(source).await?;
        }

        Ok(())
    }

    /// Rename a mise task
    pub async fn rename_task(&self, old_name: &str, new_name: &str) -> Result<()> {
        info!("Starting rename operation: '{old_name}' -> '{new_name}'");

        // Validate new name
        if new_name.trim().is_empty() {
            warn!("Validation failed: new task name is empty");
            anyhow::bail!("New task name cannot be empty");
        }

        if old_name == new_name {
            debug!("No rename needed: names are identical");
            return Ok(()); // No change needed
        }

        // Get all existing tasks to check for conflicts
        debug!("Fetching existing tasks for conflict check");
        let existing_tasks = match self.list_tasks().await {
            Ok(tasks) => {
                debug!("Found {} existing tasks", tasks.len());
                tasks
            }
            Err(e) => {
                error!("Failed to fetch existing tasks: {e}");
                return Err(e);
            }
        };

        // Find a unique name if there's a conflict
        let final_new_name = self.find_unique_task_name(new_name, &existing_tasks, old_name);
        if final_new_name != new_name {
            info!("Name conflict resolved: '{new_name}' adjusted to '{final_new_name}'");
        }

        // First, get task info to determine if it's file-based or config-based
        debug!("Getting task info for '{old_name}'");
        let task_info = match self.get_task_info(old_name).await {
            Ok(info) => {
                debug!(
                    "Task info retrieved for '{}', source: {}",
                    old_name, info.source
                );
                info
            }
            Err(e) => {
                error!("Failed to get task info for '{old_name}': {e}");
                return Err(e);
            }
        };

        let source = &task_info.source;
        if source.ends_with(".toml") {
            // Config-based task - rename in mise.toml
            debug!("Config-based task detected, updating TOML file: {source}");
            self.rename_task_in_config(source, old_name, &final_new_name)
                .await?;
        } else {
            // File-based task - rename the file
            debug!("File-based task detected, renaming file: {source}");
            self.rename_task_file(source, old_name, &final_new_name)
                .await?;
        }

        info!("Rename operation completed successfully: '{old_name}' -> '{final_new_name}'");
        Ok(())
    }

    /// Find a unique task name by appending -1, -2, etc. if needed
    fn find_unique_task_name(
        &self,
        desired_name: &str,
        existing_tasks: &[MiseTask],
        old_name: &str,
    ) -> String {
        let mut candidate_name = desired_name.to_string();
        let mut counter = 1;

        // Check if the desired name conflicts with existing tasks (excluding the task being renamed)
        while existing_tasks
            .iter()
            .any(|task| task.name == candidate_name && task.name != old_name)
        {
            candidate_name = format!("{desired_name}-{counter}");
            counter += 1;
        }

        candidate_name
    }

    async fn delete_task_from_config(&self, config_path: &str, task_name: &str) -> Result<()> {
        // Read the TOML file
        let content = fs::read_to_string(config_path)
            .await
            .context("Failed to read mise.toml file")?;

        // Parse the TOML
        let mut config: toml::Table = content.parse().context("Failed to parse mise.toml file")?;

        // Remove the task from the [tasks] section
        if let Some(tasks) = config.get_mut("tasks") {
            if let Some(tasks_table) = tasks.as_table_mut() {
                if tasks_table.remove(task_name).is_some() {
                    // Write the updated content back
                    let updated_content =
                        toml::to_string(&config).context("Failed to serialize updated TOML")?;

                    fs::write(config_path, updated_content)
                        .await
                        .context("Failed to write updated mise.toml file")?;
                } else {
                    anyhow::bail!("Task '{}' not found in tasks section", task_name);
                }
            } else {
                anyhow::bail!("Tasks section is not a table");
            }
        } else {
            anyhow::bail!("No tasks section found in mise.toml");
        }

        Ok(())
    }

    async fn delete_task_file(&self, file_path: &str) -> Result<()> {
        if !Path::new(file_path).exists() {
            anyhow::bail!("Task file '{}' does not exist", file_path);
        }

        fs::remove_file(file_path)
            .await
            .context("Failed to delete task file")?;

        Ok(())
    }

    async fn rename_task_in_config(
        &self,
        config_path: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<()> {
        // Read the TOML file
        let content = fs::read_to_string(config_path)
            .await
            .context("Failed to read mise.toml file")?;

        // Parse the TOML
        let mut config: toml::Table = content.parse().context("Failed to parse mise.toml file")?;

        // Rename the task in the [tasks] section
        if let Some(tasks) = config.get_mut("tasks") {
            if let Some(tasks_table) = tasks.as_table_mut() {
                if let Some(task_config) = tasks_table.remove(old_name) {
                    // Add the task with the new name
                    tasks_table.insert(new_name.to_string(), task_config);

                    // Write the updated content back
                    let updated_content =
                        toml::to_string(&config).context("Failed to serialize updated TOML")?;

                    fs::write(config_path, updated_content)
                        .await
                        .context("Failed to write updated mise.toml file")?;
                } else {
                    anyhow::bail!("Task '{}' not found in tasks section", old_name);
                }
            } else {
                anyhow::bail!("Tasks section is not a table");
            }
        } else {
            anyhow::bail!("No tasks section found in mise.toml");
        }

        Ok(())
    }

    async fn rename_task_file(
        &self,
        file_path: &str,
        _old_name: &str,
        new_name: &str,
    ) -> Result<()> {
        let old_path = Path::new(file_path);

        if !old_path.exists() {
            anyhow::bail!("Task file '{}' does not exist", file_path);
        }

        // Get the directory and file extension
        let parent_dir = old_path
            .parent()
            .context("Failed to get parent directory of task file")?;
        let extension = old_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Create new file path with the new name
        let new_filename = if extension.is_empty() {
            new_name.to_string()
        } else {
            format!("{new_name}.{extension}")
        };

        let new_path = parent_dir.join(new_filename);

        // Check if target file already exists
        if new_path.exists() {
            anyhow::bail!("Target file '{}' already exists", new_path.display());
        }

        // Rename the file
        fs::rename(old_path, &new_path)
            .await
            .context("Failed to rename task file")?;

        Ok(())
    }
}
