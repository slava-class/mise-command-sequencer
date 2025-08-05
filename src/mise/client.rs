use anyhow::{Context, Result};
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
        let output = Command::new("mise")
            .args(["tasks", "ls", "--json"])
            .output()
            .await
            .context("Failed to execute mise tasks ls --json")?;

        if !output.status.success() {
            anyhow::bail!(
                "mise command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let tasks: Vec<MiseTask> = serde_json::from_slice(&output.stdout)
            .context("Failed to parse mise tasks JSON output")?;

        Ok(tasks)
    }

    /// Get detailed information about a specific task
    pub async fn get_task_info(&self, task_name: &str) -> Result<MiseTaskInfo> {
        let output = Command::new("mise")
            .args(["tasks", "info", task_name, "--json"])
            .output()
            .await
            .context("Failed to execute mise tasks info")?;

        if !output.status.success() {
            anyhow::bail!(
                "mise command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let task_info: MiseTaskInfo = serde_json::from_slice(&output.stdout)
            .context("Failed to parse mise task info JSON output")?;

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
}
