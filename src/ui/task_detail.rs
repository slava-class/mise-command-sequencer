use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn draw_task_detail(app: &App, f: &mut Frame, task_name: &str) {
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
    if let Some(info) = &app.task_info {
        let mut details = vec![
            format!("Name: {}", info.name),
            format!("Source: {}", info.source),
        ];

        if !info.description.is_empty() {
            details.push(format!("Description: {}", info.description));
        }

        if let Some(file) = &info.file {
            details.push(format!("File: {file}"));
        }

        if let Some(dir) = &info.dir {
            details.push(format!("Directory: {dir}"));
        }

        if !info.aliases.is_empty() {
            details.push(format!("Aliases: {}", info.aliases.join(", ")));
        }

        if !info.depends.is_empty() {
            details.push(format!("Dependencies: {}", info.depends.join(", ")));
        }

        if !info.env.is_empty() {
            details.push("Environment Variables:".to_string());
            for env_var in &info.env {
                if let Some(env_str) = env_var.as_str() {
                    details.push(format!("  {env_str}"));
                } else {
                    details.push(format!(
                        "  {}",
                        serde_json::to_string(env_var).unwrap_or_default()
                    ));
                }
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
