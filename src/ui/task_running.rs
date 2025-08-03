use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn draw_task_running(app: &App, f: &mut Frame, task_name: &str) {
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
    let output_text: String = app
        .task_output
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    f.render_widget(
        Paragraph::new(output_text)
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .wrap(Wrap { trim: true })
            .scroll((app.task_output.len().saturating_sub(10) as u16, 0)),
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
