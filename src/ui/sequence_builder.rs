use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub struct TableLayout {
    pub table_area: Rect,
    pub column_rects: Vec<Rect>,
}

pub fn calculate_table_layout(area: Rect, num_steps: usize) -> TableLayout {
    // Create constraints matching the table in draw_matrix_interface
    let mut constraints = vec![Constraint::Min(20)]; // Task name column
    for _ in 0..num_steps {
        constraints.push(Constraint::Length(8)); // Step columns
    }
    constraints.push(Constraint::Min(20)); // Actions column

    // Calculate column positions using ratatui's layout system
    let column_layout = Layout::horizontal(constraints).split(area);

    TableLayout {
        table_area: area,
        column_rects: column_layout.to_vec(),
    }
}

pub fn draw_sequence_builder(app: &mut App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Matrix interface
            Constraint::Min(5),    // Task output
            Constraint::Length(3), // Controls
        ])
        .split(f.area());

    draw_matrix_interface(app, f, chunks[0]);
    draw_task_output(app, f, chunks[1]);
    draw_controls(f, chunks[2]);
}

fn draw_matrix_interface(app: &mut App, f: &mut Frame, area: Rect) {
    let num_steps = 3; // Default to 3 steps for now

    // Calculate and store table layout for mouse click detection
    app.table_layout = Some(calculate_table_layout(area, num_steps));

    // Create headers: Task Name, Step 1, Step 2, Step 3, Actions
    let mut header_cells =
        vec![Cell::from("Task Name").style(Style::default().add_modifier(Modifier::BOLD))];
    for i in 1..=num_steps {
        header_cells.push(
            Cell::from(format!("Step {i}")).style(Style::default().add_modifier(Modifier::BOLD)),
        );
    }
    header_cells.push(Cell::from("Actions").style(Style::default().add_modifier(Modifier::BOLD)));

    let header = Row::new(header_cells).height(1);

    // Create rows for each task
    let mut rows = Vec::new();
    for (index, task) in app.tasks.iter().enumerate() {
        let mut cells = Vec::new();

        // Task name cell with selection indicator
        let task_name_style = if index == app.selected_task {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let task_name_text = if index == app.selected_task {
            format!("> {}", task.name)
        } else {
            format!("  {}", task.name)
        };

        cells.push(Cell::from(task_name_text).style(task_name_style));

        // Step toggle cells
        for step in 0..num_steps {
            let is_enabled = app
                .sequence_state
                .is_task_enabled_for_step(&task.name, step);
            let symbol = if is_enabled { "●" } else { " " };
            let style = if is_enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            cells.push(Cell::from(symbol).style(style));
        }

        // Action buttons
        cells.push(Cell::from("[run] [cat] [edit]").style(Style::default().fg(Color::Cyan)));

        rows.push(Row::new(cells).height(1));
    }

    // Create the table with proper column constraints
    let mut constraints = vec![Constraint::Min(20)]; // Task name column
    for _ in 0..num_steps {
        constraints.push(Constraint::Length(8)); // Step columns
    }
    constraints.push(Constraint::Min(20)); // Actions column

    let table = Table::new(rows, constraints)
        .header(header)
        .block(
            Block::default()
                .title("Available Tasks")
                .borders(Borders::ALL),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("");

    // Add sequence controls to the right side of the title
    let title_area = Rect {
        x: area.x + area.width - 30,
        y: area.y,
        width: 30,
        height: 1,
    };

    let sequence_controls =
        Paragraph::new("[Run sequence] [Clear]").style(Style::default().fg(Color::Blue));

    f.render_widget(table, area);
    f.render_widget(Clear, title_area);
    f.render_widget(sequence_controls, title_area);
}

fn draw_task_output(app: &App, f: &mut Frame, area: Rect) {
    let mut output_text = Vec::new();

    // Show current step information if sequence is running
    if app.sequence_state.is_running {
        if let Some(current_step) = app.sequence_state.current_step {
            let tasks_for_step = app.sequence_state.get_tasks_for_step(current_step);
            if !tasks_for_step.is_empty() {
                output_text.push(Line::from(vec![
                    Span::styled(
                        format!(
                            "Step {}/{}: Running ",
                            current_step + 1,
                            app.sequence_state.num_steps
                        ),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(tasks_for_step.join(", "), Style::default().fg(Color::Cyan)),
                    Span::raw("..."),
                ]));
                output_text.push(Line::raw(""));
            }
        }
    }

    // Add task output
    for line in app.task_output.iter() {
        output_text.push(Line::raw(line.clone()));
    }

    let output = Paragraph::new(output_text)
        .block(Block::default().title("Task Output").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(output, area);
}

fn draw_controls(f: &mut Frame, area: Rect) {
    let controls = Paragraph::new(
        "↑/↓: Navigate | 1/2/3: Toggle step | Enter: Run sequence | x: Run task | e: Edit | Tab: Info | c: Clear | q: Quit"
    )
    .block(
        Block::default()
            .title("Controls")
            .borders(Borders::ALL)
    )
    .style(Style::default().fg(Color::Gray));

    f.render_widget(controls, area);
}
