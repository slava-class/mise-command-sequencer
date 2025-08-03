use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::ui::button_layout::{ActionButton, ButtonType, SequenceButton};
use crate::ui::constants::*;

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
    let chunks = if app.show_output_pane {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Matrix interface
                Constraint::Min(5),    // Task output
                Constraint::Length(3), // Controls
            ])
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Matrix interface
                Constraint::Length(3), // Controls
            ])
            .split(f.area())
    };

    draw_matrix_interface(app, f, chunks[0]);
    if app.show_output_pane {
        draw_task_output(app, f, chunks[1]);
        draw_controls(f, chunks[2]);
    } else {
        draw_controls(f, chunks[1]);
    }
}

fn draw_matrix_interface(app: &mut App, f: &mut Frame, area: Rect) {
    let num_steps = 3; // Default to 3 steps for now

    // Calculate and store table layout for mouse click detection
    app.table_layout = Some(calculate_table_layout(area, num_steps));

    // Calculate available height for task rows (subtract header + borders)
    let available_height = area.height.saturating_sub(3); // Header + top/bottom borders
    let visible_height = available_height as usize;

    // Store the current visible height for scroll calculations
    app.current_visible_height = visible_height;

    // Get visible tasks without automatically adjusting scroll
    let (visible_tasks, _selected_in_visible) = app.get_visible_tasks(visible_height);

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

    // Create rows for visible tasks only
    let mut rows = Vec::new();
    for (visible_index, task) in visible_tasks.iter().enumerate() {
        let mut cells = Vec::new();
        let actual_index = app.scroll_offset + visible_index;

        // Task name cell with selection indicator
        let task_name_style = if actual_index == app.selected_task {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let task_name_text = if actual_index == app.selected_task {
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

        // Action buttons with hover styling
        let action_buttons_cell = create_action_buttons_cell(app, actual_index);
        cells.push(action_buttons_cell);

        rows.push(Row::new(cells).height(1));
    }

    // Create the table with proper column constraints
    let mut constraints = vec![Constraint::Min(20)]; // Task name column
    for _ in 0..num_steps {
        constraints.push(Constraint::Length(8)); // Step columns
    }
    constraints.push(Constraint::Min(20)); // Actions column

    // Create table title with scroll indicators
    let mut title = APP_TITLE.to_string();
    if app.tasks.len() > visible_height {
        let total_tasks = app.tasks.len();
        let start_task = app.scroll_offset + 1;
        let end_task = (app.scroll_offset + visible_height).min(total_tasks);
        title = format!("{APP_TITLE} ({start_task}-{end_task}/{total_tasks})");
    }

    let table = Table::new(rows, constraints)
        .header(header)
        .block(Block::default().title(title).borders(Borders::ALL))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("");

    f.render_widget(table, area);

    // Add sequence controls embedded in the title bar
    render_sequence_controls_in_title(app, f, area);
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
        .block(
            Block::default()
                .title(TASK_OUTPUT_TITLE)
                .borders(Borders::ALL),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(output, area);
}

fn draw_controls(f: &mut Frame, area: Rect) {
    let controls = Paragraph::new(
        "↑/↓: Navigate | PgUp/PgDn/Mouse wheel: Scroll | 1/2/3: Toggle step | Enter: Run sequence | x: Run task | e: Edit | Tab: Info | c: Clear | q: Quit"
    )
    .block(
        Block::default()
            .title(CONTROLS_TITLE)
            .borders(Borders::ALL)
    )
    .style(Style::default().fg(Color::Gray));

    f.render_widget(controls, area);
}

fn create_action_buttons_cell(app: &App, task_index: usize) -> Cell {
    // Check if any button in this row is being hovered
    let hover_button = if let Some(hover_state) = &app.button_hover_state {
        match hover_state.button_type {
            ButtonType::Action {
                button,
                task_index: hovered_task_index,
            } => {
                if hovered_task_index == task_index {
                    Some(button)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    };

    // Check if this task is selected
    let is_selected = app.selected_task == task_index;

    // Create spans for each button with appropriate styling
    let mut spans = Vec::new();

    // Run button
    let run_style = if matches!(hover_button, Some(ActionButton::Run)) {
        Style::default().bg(Color::Green).fg(Color::Black)
    } else if is_selected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    spans.push(Span::styled(RUN_BUTTON_TEXT, run_style));

    // Cat button
    spans.push(Span::raw(BUTTON_SPACING));
    let cat_style = if matches!(hover_button, Some(ActionButton::Cat)) {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else if is_selected {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    spans.push(Span::styled(CAT_BUTTON_TEXT, cat_style));

    // Edit button
    spans.push(Span::raw(BUTTON_SPACING));
    let edit_style = if matches!(hover_button, Some(ActionButton::Edit)) {
        Style::default().bg(Color::Magenta).fg(Color::White)
    } else if is_selected {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    spans.push(Span::styled(EDIT_BUTTON_TEXT, edit_style));

    Cell::from(Line::from(spans))
}

fn create_sequence_controls_paragraph(app: &App) -> Paragraph {
    // Check if any sequence button is being hovered
    let hover_button = if let Some(hover_state) = &app.button_hover_state {
        match hover_state.button_type {
            ButtonType::Sequence(button) => Some(button),
            _ => None,
        }
    } else {
        None
    };

    // Create spans for sequence buttons with hover effects
    let mut spans = Vec::new();

    // Run sequence button
    let run_sequence_style = if matches!(hover_button, Some(SequenceButton::RunSequence)) {
        Style::default().bg(Color::Green).fg(Color::Black)
    } else {
        Style::default().fg(Color::Blue)
    };
    spans.push(Span::styled(RUN_SEQUENCE_BUTTON_TEXT, run_sequence_style));

    // Space between buttons
    spans.push(Span::raw(BUTTON_SPACING));

    // Clear button
    let clear_style = if matches!(hover_button, Some(SequenceButton::Clear)) {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().fg(Color::Blue)
    };
    spans.push(Span::styled(CLEAR_BUTTON_TEXT, clear_style));

    Paragraph::new(Line::from(spans))
}

fn render_sequence_controls_in_title(app: &App, f: &mut Frame, table_area: Rect) {
    let title_text = if app.tasks.len() > app.current_visible_height {
        let total_tasks = app.tasks.len();
        let start_task = app.scroll_offset + 1;
        let end_task = (app.scroll_offset + app.current_visible_height).min(total_tasks);
        format!("{APP_TITLE} ({start_task}-{end_task}/{total_tasks})")
    } else {
        APP_TITLE.to_string()
    };

    // Account for border and padding: left border (1) + space (1) + title + space (1)
    let title_offset = 3 + title_text.len();
    let controls_text = format!("{RUN_SEQUENCE_BUTTON_TEXT} {CLEAR_BUTTON_TEXT}");
    let controls_width = controls_text.len();

    // Position controls to the right, accounting for right border (1)
    let available_width = table_area.width as usize;
    let controls_start = if title_offset + controls_width + 2 <= available_width {
        available_width - controls_width - 2 // Leave space for right border
    } else {
        title_offset + 2 // Place after title if there's not enough space
    };

    let controls_area = Rect {
        x: table_area.x + controls_start as u16,
        y: table_area.y,
        width: controls_width as u16,
        height: 1,
    };

    // Only render if the area is valid and visible
    if controls_area.x + controls_area.width <= table_area.x + table_area.width {
        let sequence_controls = create_sequence_controls_paragraph(app);
        f.render_widget(sequence_controls, controls_area);
    }
}
