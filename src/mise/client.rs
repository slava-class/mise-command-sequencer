use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::{
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
}
