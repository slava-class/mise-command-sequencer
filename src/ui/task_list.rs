use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn draw_task_list(app: &App, f: &mut Frame) {
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
        app.tasks.len(),
        app.last_updated.elapsed().as_secs_f32()
    );

    f.render_widget(
        Paragraph::new(header_text)
            .block(header)
            .alignment(Alignment::Center),
        chunks[0],
    );

    // Task list
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let style = if i == app.selected_task {
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
