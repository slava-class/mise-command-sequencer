use anyhow::{Context, Result};
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    io,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc,
    time::sleep,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseTask {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub hide: Option<bool>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseTaskInfo {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub file: Option<String>,
    pub dir: Option<String>,
    pub hide: Option<bool>,
    pub alias: Option<String>,
    pub run: Option<serde_json::Value>,
    pub depends: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

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

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn mise run command")?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

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

        let _ = output_tx.send(final_message);

        Ok(())
    }
}

#[derive(Debug)]
pub enum AppState {
    TaskList,
    TaskDetail(String),
    TaskRunning(String),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    KeyPress(KeyCode),
    TasksRefreshed(Vec<MiseTask>),
    TaskInfoLoaded(MiseTaskInfo),
    TaskOutput(String),
    TaskCompleted,
    Tick,
}

pub struct App {
    client: MiseClient,
    tasks: Vec<MiseTask>,
    selected_task: usize,
    state: AppState,
    task_info: Option<MiseTaskInfo>,
    task_output: VecDeque<String>,
    should_quit: bool,
    last_updated: Instant,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    task_output_rx: Option<mpsc::UnboundedReceiver<String>>,
}

impl App {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            client: MiseClient::new(),
            tasks: vec![],
            selected_task: 0,
            state: AppState::TaskList,
            task_info: None,
            task_output: VecDeque::new(),
            should_quit: false,
            last_updated: Instant::now(),
            event_tx,
            task_output_rx: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.refresh_tasks().await?;
        Ok(())
    }

    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::KeyPress(key) => self.handle_key(key).await?,
            AppEvent::TasksRefreshed(tasks) => {
                self.tasks = tasks;
                self.last_updated = Instant::now();
            }
            AppEvent::TaskInfoLoaded(info) => {
                self.task_info = Some(info);
            }
            AppEvent::TaskOutput(output) => {
                self.task_output.push_back(output);
                // Keep only the last 100 lines
                while self.task_output.len() > 100 {
                    self.task_output.pop_front();
                }
            }
            AppEvent::TaskCompleted => {
                // Task has completed, we could transition state or just stay
            }
            AppEvent::Tick => {
                // Handle periodic updates if needed
            }
        }
        Ok(())
    }

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

    pub fn select_next(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = (self.selected_task + 1) % self.tasks.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = if self.selected_task > 0 {
                self.selected_task - 1
            } else {
                self.tasks.len() - 1
            };
        }
    }

    pub async fn show_task_detail(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected_task) {
            let client = self.client.clone();
            let task_name = task.name.clone();
            let event_tx = self.event_tx.clone();

            self.state = AppState::TaskDetail(task_name.clone());

            tokio::spawn(async move {
                match client.get_task_info(&task_name).await {
                    Ok(info) => {
                        let _ = event_tx.send(AppEvent::TaskInfoLoaded(info));
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

            self.state = AppState::TaskRunning(task_name.clone());

            tokio::spawn(async move {
                if let Err(e) = client.run_task(&task_name, &[], output_tx).await {
                    eprintln!("Failed to run task: {e}");
                }
                let _ = event_tx.send(AppEvent::TaskCompleted);
            });
        }
        Ok(())
    }

    pub fn back_to_list(&mut self) {
        self.state = AppState::TaskList;
        self.task_info = None;
        self.task_output.clear();
        self.task_output_rx = None;
    }

    pub async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match (&self.state, key) {
            (_, KeyCode::Char('q')) => self.should_quit = true,
            (_, KeyCode::Char('r')) => self.refresh_tasks().await?,

            (AppState::TaskList, KeyCode::Down | KeyCode::Char('j')) => self.select_next(),
            (AppState::TaskList, KeyCode::Up | KeyCode::Char('k')) => self.select_previous(),
            (AppState::TaskList, KeyCode::Enter | KeyCode::Char(' ')) => {
                self.show_task_detail().await?
            }
            (AppState::TaskList, KeyCode::Char('x')) => self.run_selected_task().await?,

            (AppState::TaskDetail(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),
            (AppState::TaskDetail(_), KeyCode::Char('x')) => self.run_selected_task().await?,

            (AppState::TaskRunning(_), KeyCode::Esc | KeyCode::Char('b')) => self.back_to_list(),

            _ => {}
        }
        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    // Check for new task output
    pub fn poll_task_output(&mut self) {
        if let Some(ref mut rx) = self.task_output_rx {
            while let Ok(output) = rx.try_recv() {
                let _ = self.event_tx.send(AppEvent::TaskOutput(output));
            }
        }
    }
}

impl App {
    pub fn draw(&self, f: &mut Frame) {
        match &self.state {
            AppState::TaskList => self.draw_task_list(f),
            AppState::TaskDetail(task_name) => self.draw_task_detail(f, task_name),
            AppState::TaskRunning(task_name) => self.draw_task_running(f, task_name),
        }
    }

    fn draw_task_list(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header
        let header = Block::default()
            .borders(Borders::ALL)
            .title("Mise Tasks")
            .border_style(Style::default().fg(Color::Blue));

        let header_text = format!(
            "Found {} tasks (Last updated: {:.1}s ago)",
            self.tasks.len(),
            self.last_updated.elapsed().as_secs_f32()
        );

        f.render_widget(
            Paragraph::new(header_text)
                .block(header)
                .alignment(Alignment::Center),
            chunks[0],
        );

        // Task list
        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let style = if i == self.selected_task {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };

                let content = format!(
                    "{} {}",
                    task.name,
                    task.description.as_deref().unwrap_or("")
                );

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(list, chunks[1]);

        // Footer
        let footer = Block::default().borders(Borders::ALL).title("Controls");

        let controls = "↑/k: Up | ↓/j: Down | Enter: Details | x: Run | r: Refresh | q: Quit";

        f.render_widget(
            Paragraph::new(controls)
                .block(footer)
                .alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn draw_task_detail(&self, f: &mut Frame, task_name: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header
        let header = Block::default()
            .borders(Borders::ALL)
            .title(format!("Task Details: {task_name}"))
            .border_style(Style::default().fg(Color::Blue));

        f.render_widget(
            Paragraph::new("Task Information")
                .block(header)
                .alignment(Alignment::Center),
            chunks[0],
        );

        // Task details
        if let Some(info) = &self.task_info {
            let mut details = vec![
                format!("Name: {}", info.name),
                format!("Source: {}", info.source),
            ];

            if let Some(desc) = &info.description {
                details.push(format!("Description: {desc}"));
            }

            if let Some(file) = &info.file {
                details.push(format!("File: {file}"));
            }

            if let Some(dir) = &info.dir {
                details.push(format!("Directory: {dir}"));
            }

            if let Some(alias) = &info.alias {
                details.push(format!("Alias: {alias}"));
            }

            if let Some(deps) = &info.depends {
                details.push(format!("Dependencies: {}", deps.join(", ")));
            }

            if let Some(env) = &info.env {
                details.push("Environment Variables:".to_string());
                for (key, value) in env {
                    details.push(format!("  {key} = {value}"));
                }
            }

            let detail_text = details.join("\n");

            f.render_widget(
                Paragraph::new(detail_text)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(Wrap { trim: true }),
                chunks[1],
            );
        } else {
            f.render_widget(
                Paragraph::new("Loading task information...")
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .alignment(Alignment::Center),
                chunks[1],
            );
        }

        // Footer
        let footer = Block::default().borders(Borders::ALL).title("Controls");

        let controls = "Esc/b: Back | x: Run Task | q: Quit";

        f.render_widget(
            Paragraph::new(controls)
                .block(footer)
                .alignment(Alignment::Center),
            chunks[2],
        );
    }

    fn draw_task_running(&self, f: &mut Frame, task_name: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header
        let header = Block::default()
            .borders(Borders::ALL)
            .title(format!("Running Task: {task_name}"))
            .border_style(Style::default().fg(Color::Green));

        f.render_widget(
            Paragraph::new("Task Execution")
                .block(header)
                .alignment(Alignment::Center),
            chunks[0],
        );

        // Task output
        let output_text: String = self
            .task_output
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        f.render_widget(
            Paragraph::new(output_text)
                .block(Block::default().borders(Borders::ALL).title("Output"))
                .wrap(Wrap { trim: true })
                .scroll((self.task_output.len().saturating_sub(10) as u16, 0)),
            chunks[1],
        );

        // Footer
        let footer = Block::default().borders(Borders::ALL).title("Controls");

        let controls = "Esc/b: Back to List | q: Quit";

        f.render_widget(
            Paragraph::new(controls)
                .block(footer)
                .alignment(Alignment::Center),
            chunks[2],
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create event channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Create app
    let mut app = App::new(event_tx.clone());
    app.initialize().await.context("Failed to initialize app")?;

    // Spawn input handling task
    let input_event_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press
                        && input_event_tx.send(AppEvent::KeyPress(key.code)).is_err()
                    {
                        break;
                    }
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Spawn tick task
    let tick_event_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            if tick_event_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    // Main event loop
    while let Some(event) = event_rx.recv().await {
        app.handle_event(event).await?;
        app.poll_task_output();

        terminal.draw(|f| app.draw(f))?;

        if app.should_quit() {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
