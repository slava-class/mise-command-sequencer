use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::models::AppState;
use crate::ui::button_layout::{
    ActionButton, ButtonStyleManager, ButtonTheme, ButtonType, DialogButton, SequenceButton,
};
use crate::ui::constants::*;

fn ensure_ansi_reset(line: &str) -> String {
    const ANSI_RESET: &str = "\x1b[0m";

    // Check if line already ends with ANSI reset sequence
    if line.ends_with(ANSI_RESET) {
        line.to_string()
    } else {
        // Add reset sequence to end of line to prevent color bleeding
        format!("{line}{ANSI_RESET}")
    }
}

pub struct TableLayout {
    pub table_area: Rect,
    pub column_rects: Vec<Rect>,
}

pub fn calculate_table_layout(area: Rect, num_steps: usize) -> TableLayout {
    // Create constraints matching the table in draw_matrix_interface
    let mut constraints = vec![Constraint::Min(20)]; // Task name column
    for _ in 0..num_steps {
        constraints.push(Constraint::Length(8)); // Step columns (8 chars to fit "Sequence" header)
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
                Constraint::Length(5), // Controls (3 lines + borders)
            ])
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Matrix interface
                Constraint::Length(5), // Controls (3 lines + borders)
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

    // Position cursor over hovered button to simulate cursor pointer
    if let Some(hover_state) = &app.button_hover_state {
        f.set_cursor_position((hover_state.col, hover_state.row));
    }

    // Draw confirmation dialog if there's a pending delete
    if let Some(task_name) = app.pending_delete_task.clone() {
        draw_delete_confirmation_dialog(f, app, &task_name);
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

    // Add "Sequence" header in the first step column
    header_cells.push(Cell::from("Sequence").style(Style::default().add_modifier(Modifier::BOLD)));

    // Add empty header for second step column
    header_cells.push(Cell::from(""));
    // Add empty header for third step column
    header_cells.push(Cell::from(""));

    // Actions header - center it manually by adding equal padding on both sides
    // "Actions" is 7 chars, so we add spaces to center it
    header_cells.push(Cell::from("Actions").style(Style::default().add_modifier(Modifier::BOLD)));

    let header = Row::new(header_cells).height(1);

    // Create rows for visible tasks only
    let mut rows = Vec::new();
    for (visible_index, task) in visible_tasks.iter().enumerate() {
        let mut cells = Vec::new();
        let actual_index = app.scroll_offset + visible_index;

        // Check if any button for this task is being hovered
        let is_task_hovered = if let Some(hover_state) = &app.button_hover_state {
            match hover_state.button_type {
                ButtonType::Action {
                    task_index: hovered_task_index,
                    ..
                } => hovered_task_index == actual_index,
                ButtonType::Step {
                    task_index: hovered_task_index,
                    ..
                } => hovered_task_index == actual_index,
                _ => false,
            }
        } else {
            false
        };

        // Task name cell with selection indicator, hover state, and rename input
        let task_name_cell = create_task_name_cell(app, actual_index, task, is_task_hovered);
        cells.push(task_name_cell);

        // Step button cells
        for step in 0..num_steps {
            let step_button_cell = create_step_button_cell(app, actual_index, step);
            cells.push(step_button_cell);
        }

        // Action buttons with hover styling
        let action_buttons_cell = create_action_buttons_cell(app, actual_index);
        cells.push(action_buttons_cell);

        rows.push(Row::new(cells).height(1));
    }

    // Create the table with proper column constraints
    let mut constraints = vec![Constraint::Min(20)]; // Task name column
    for _ in 0..num_steps {
        constraints.push(Constraint::Length(7)); // Step columns (8 chars to fit "Sequence" header)
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

fn draw_task_output(app: &mut App, f: &mut Frame, area: Rect) {
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

    // Calculate available height for actual task output
    // Account for borders (2) and any header lines we've added
    let header_lines = output_text.len() as u16;
    let available_height = area.height.saturating_sub(2).saturating_sub(header_lines) as usize;

    // Store this for use in event handlers
    app.current_output_visible_height = available_height;

    let total_output_lines = app.task_output.len();

    // Ensure scroll offset is within bounds
    let max_scroll = if total_output_lines > available_height && available_height > 0 {
        total_output_lines - available_height
    } else {
        0
    };
    app.output_scroll_offset = app.output_scroll_offset.min(max_scroll);

    // Apply scroll offset to the task output
    let start_index = app.output_scroll_offset;
    let end_index = (start_index + available_height).min(total_output_lines);

    // Add visible lines with ANSI color support
    for line in app
        .task_output
        .iter()
        .skip(start_index)
        .take(end_index.saturating_sub(start_index))
    {
        // Ensure line has ANSI reset to prevent color bleeding
        let normalized_line = ensure_ansi_reset(line);

        // Parse ANSI escape sequences and convert to ratatui Text
        match ansi_to_tui::IntoText::into_text(&normalized_line) {
            Ok(parsed_text) => {
                // Extract lines from the parsed text and add them
                for parsed_line in parsed_text.lines {
                    output_text.push(parsed_line);
                }
            }
            Err(_) => {
                // Fallback to raw text if parsing fails
                output_text.push(Line::raw(normalized_line));
            }
        }
    }

    // Create title with scroll indicators
    let mut title = TASK_OUTPUT_TITLE.to_string();
    if available_height > 0 && total_output_lines > available_height {
        let visible_start = start_index + 1;
        let visible_end = end_index;
        title = format!("{TASK_OUTPUT_TITLE} ({visible_start}-{visible_end}/{total_output_lines})");
    }

    let output = Paragraph::new(output_text)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(output, area);
}

fn draw_controls(f: &mut Frame, area: Rect) {
    let controls = Paragraph::new(vec![
        Line::from("Navigation: ↑/↓: Select task | PgUp/PgDn/Mouse wheel: Scroll | q: Quit | g/G/F: Output scroll"),
        Line::from("Task Actions: x: Run task | e: Edit | c: Rename | D: Delete | Tab: Info"),
        Line::from("Sequence Actions: 1/2/3: Toggle step | Enter: Run sequence | a: Add as task | Ctrl+L: Clear"),
    ])
    .block(
        Block::default()
            .title(CONTROLS_TITLE)
            .borders(Borders::ALL)
    )
    .style(Style::default().fg(Color::Gray));

    f.render_widget(controls, area);
}

fn create_task_name_cell<'a>(
    app: &App,
    task_index: usize,
    task: &crate::models::MiseTask,
    is_task_hovered: bool,
) -> Cell<'a> {
    // Check if this task is being renamed
    let is_renaming = if let AppState::Renaming(ref renaming_task) = app.state {
        renaming_task == &task.name
    } else {
        false
    };

    if is_renaming {
        // In rename mode, show the input widget
        if let Some(ref input) = app.rename_input {
            let prefix = if task_index == app.selected_task {
                "> "
            } else {
                "  "
            };
            let input_text = format!("{}{}", prefix, input.value());

            // Style for rename input - highlight it differently
            let style = Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD);

            Cell::from(input_text).style(style)
        } else {
            // Fallback if input is not initialized
            let prefix = if task_index == app.selected_task {
                "> "
            } else {
                "  "
            };
            let text = format!("{}{}", prefix, task.name);
            Cell::from(text).style(Style::default().fg(Color::Red))
        }
    } else {
        // Normal mode - regular task name display
        let task_name_style = if is_task_hovered {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if task_index == app.selected_task {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let task_name_text = if task_index == app.selected_task {
            format!("> {}", task.name)
        } else {
            format!("  {}", task.name)
        };

        // Apply faded style if another task is being renamed
        let style = if matches!(app.state, AppState::Renaming(_)) {
            Style::default().fg(Color::DarkGray)
        } else {
            task_name_style
        };

        Cell::from(task_name_text).style(style)
    }
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

    // Check if we're in rename mode for this task
    let is_renaming = if let AppState::Renaming(ref renaming_task) = app.state {
        renaming_task == &app.tasks[task_index].name
    } else {
        false
    };

    // Create spans for each button with appropriate styling
    let mut spans = Vec::new();

    // Different buttons based on rename mode
    let buttons = if is_renaming {
        // In rename mode, show only save and cancel buttons for this task
        vec![
            (
                SAVE_BUTTON_TEXT,
                ActionButton::Save,
                ButtonTheme::ACTION_SAVE,
            ),
            (
                CANCEL_BUTTON_TEXT,
                ActionButton::Cancel,
                ButtonTheme::ACTION_CANCEL,
            ),
        ]
    } else {
        // Normal mode: show all action buttons but fade them if another task is being renamed
        let _is_other_task_renaming = matches!(app.state, AppState::Renaming(_));

        vec![
            (RUN_BUTTON_TEXT, ActionButton::Run, ButtonTheme::ACTION_RUN),
            (CAT_BUTTON_TEXT, ActionButton::Cat, ButtonTheme::ACTION_CAT),
            (
                EDIT_BUTTON_TEXT,
                ActionButton::Edit,
                ButtonTheme::ACTION_EDIT,
            ),
            (
                RENAME_BUTTON_TEXT,
                ActionButton::Rename,
                ButtonTheme::ACTION_RENAME,
            ),
            (
                DELETE_BUTTON_TEXT,
                ActionButton::Delete,
                ButtonTheme::ACTION_DELETE,
            ),
        ]
    };

    for (i, (text, button_type, theme)) in buttons.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(BUTTON_SPACING));
        }

        let is_hovered = matches!(hover_button, Some(bt) if bt == *button_type);

        // For non-rename mode buttons when another task is being renamed, apply faded style
        let style = if !is_renaming && matches!(app.state, AppState::Renaming(_)) {
            // Faded/disabled style
            Style::default().fg(Color::DarkGray)
        } else {
            ButtonStyleManager::create_button_style(*theme, is_hovered, is_selected, None)
        };

        spans.push(Span::styled(*text, style));
    }

    Cell::from(Line::from(spans))
}

fn create_step_button_cell(app: &App, task_index: usize, step_index: usize) -> Cell {
    use crate::ui::constants::{STEP_1_TEXT, STEP_2_TEXT, STEP_3_TEXT, STEP_DISABLED_TEXT};

    // Check if this step button is being hovered
    let is_hovered = if let Some(hover_state) = &app.button_hover_state {
        match hover_state.button_type {
            ButtonType::Step {
                step_index: hovered_step,
                task_index: hovered_task,
            } => hovered_step == step_index && hovered_task == task_index,
            _ => false,
        }
    } else {
        false
    };

    // Check if this task is enabled for this step
    let task_name = &app.tasks[task_index].name;
    let is_enabled = app
        .sequence_state
        .is_task_enabled_for_step(task_name, step_index);

    // Determine the text to display
    let text = if is_enabled {
        match step_index {
            0 => STEP_1_TEXT,
            1 => STEP_2_TEXT,
            2 => STEP_3_TEXT,
            _ => STEP_DISABLED_TEXT,
        }
    } else {
        STEP_DISABLED_TEXT
    };

    // Create spans with explicit positioning like action buttons
    let mut spans = Vec::new();

    // Add padding to position the button within the 8-character column
    // Column is 8 chars, button is 7 chars, so we need 1 char padding
    // We'll left-align it with no leading padding for now

    // Apply faded style if in rename mode
    let style = if matches!(app.state, AppState::Renaming(_)) {
        Style::default().fg(Color::DarkGray)
    } else {
        ButtonStyleManager::create_button_style(
            ButtonTheme::STEP,
            is_hovered,
            false,
            Some(is_enabled),
        )
    };

    spans.push(Span::styled(text, style));

    // Add trailing space to fill the 8-character column (7-char button + 1 space)
    // spans.push(Span::raw(" "));

    Cell::from(Line::from(spans))
}

fn draw_delete_confirmation_dialog(f: &mut Frame, app: &mut App, task_name: &str) {
    // Create a centered dialog area
    let area = f.area();
    let dialog_width = 60.min(area.width - 4);
    let dialog_height = 11;

    let dialog_area = Rect {
        x: (area.width - dialog_width) / 2,
        y: (area.height - dialog_height) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Store dialog area for mouse click detection
    app.delete_dialog_area = Some(dialog_area);

    // Clear the background area first
    f.render_widget(Clear, dialog_area);

    // Check for hover states on dialog buttons
    let hover_button = if let Some(hover_state) = &app.button_hover_state {
        match hover_state.button_type {
            ButtonType::Dialog(button) => Some(button),
            _ => None,
        }
    } else {
        None
    };

    // Create the dialog content using constants
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                DELETE_DIALOG_QUESTION_PREFIX,
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("'{task_name}'"),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                DELETE_DIALOG_QUESTION_SUFFIX,
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            DELETE_DIALOG_WARNING,
            Style::default().fg(Color::Red),
        )]),
        Line::from(vec![Span::styled(
            DELETE_DIALOG_VERSION_CONTROL_TIP,
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                DELETE_DIALOG_INSTRUCTIONS,
                Style::default().fg(Color::White),
            ),
            Span::styled(
                DELETE_DIALOG_DELETE_KEY,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                DELETE_DIALOG_DELETE_ACTION,
                Style::default().fg(Color::White),
            ),
            Span::styled(
                DELETE_DIALOG_CANCEL_KEYS,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("/", Style::default().fg(Color::White)),
            Span::styled(
                DELETE_DIALOG_CANCEL_KEYS_ALT,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                DELETE_DIALOG_CANCEL_ACTION,
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(create_dialog_buttons_line(hover_button)),
    ];

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .title(DELETE_DIALOG_TITLE)
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(dialog, dialog_area);
}

fn create_dialog_buttons_line(hover_button: Option<DialogButton>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Dialog buttons using semantic compression
    let buttons = [
        (
            DELETE_DIALOG_BUTTON_TEXT,
            DialogButton::Delete,
            ButtonTheme::DIALOG_DELETE,
        ),
        (
            CANCEL_DIALOG_BUTTON_TEXT,
            DialogButton::Cancel,
            ButtonTheme::DIALOG_CANCEL,
        ),
    ];

    for (i, (text, button_type, theme)) in buttons.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("     ", Style::default()));
        }

        let is_hovered = matches!(hover_button, Some(bt) if bt == *button_type);
        let style = ButtonStyleManager::create_button_style(*theme, is_hovered, false, None);
        spans.push(Span::styled(*text, style));
    }

    spans
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

    // Sequence buttons using semantic compression
    let buttons = [
        (
            RUN_SEQUENCE_BUTTON_TEXT,
            SequenceButton::RunSequence,
            ButtonTheme::SEQUENCE,
        ),
        (
            ADD_AS_TASK_BUTTON_TEXT,
            SequenceButton::AddAsTask,
            ButtonTheme::SEQUENCE_ADD,
        ),
        (
            CLEAR_BUTTON_TEXT,
            SequenceButton::Clear,
            ButtonTheme::SEQUENCE_CLEAR,
        ),
    ];

    for (i, (text, button_type, theme)) in buttons.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(BUTTON_SPACING));
        }

        let is_hovered = matches!(hover_button, Some(bt) if bt == *button_type);

        // Apply faded style if in rename mode
        let style = if matches!(app.state, AppState::Renaming(_)) {
            Style::default().fg(Color::DarkGray)
        } else {
            ButtonStyleManager::create_button_style(*theme, is_hovered, false, None)
        };

        spans.push(Span::styled(*text, style));
    }

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
    let controls_text =
        format!("{RUN_SEQUENCE_BUTTON_TEXT} {ADD_AS_TASK_BUTTON_TEXT} {CLEAR_BUTTON_TEXT}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_ansi_reset_already_has_reset() {
        let line = "STDOUT: Some output\x1b[0m";
        let result = ensure_ansi_reset(line);
        assert_eq!(result, line);
    }

    #[test]
    fn test_ensure_ansi_reset_missing_reset() {
        let line = "STDERR: Error message";
        let result = ensure_ansi_reset(line);
        assert_eq!(result, "STDERR: Error message\x1b[0m");
    }

    #[test]
    fn test_ensure_ansi_reset_empty_string() {
        let line = "";
        let result = ensure_ansi_reset(line);
        assert_eq!(result, "\x1b[0m");
    }

    #[test]
    fn test_ensure_ansi_reset_with_color_codes() {
        let line = "\x1b[31mSTDOUT: Red text\x1b[32m";
        let result = ensure_ansi_reset(line);
        assert_eq!(result, "\x1b[31mSTDOUT: Red text\x1b[32m\x1b[0m");
    }

    #[test]
    fn test_ensure_ansi_reset_partial_escape_sequence() {
        let line = "Normal text\x1b[31";
        let result = ensure_ansi_reset(line);
        assert_eq!(result, "Normal text\x1b[31\x1b[0m");
    }

    #[test]
    fn test_ensure_ansi_reset_multiple_resets_in_middle() {
        let line = "Text\x1b[0m more text";
        let result = ensure_ansi_reset(line);
        assert_eq!(result, "Text\x1b[0m more text\x1b[0m");
    }

    #[test]
    fn test_ensure_ansi_reset_stdout_stderr_prefixes() {
        // Test the specific use case that was causing bleeding
        let stdout_line = "STDOUT: \x1b[32mSuccess message";
        let stderr_line = "STDERR: \x1b[31mError message";

        let stdout_result = ensure_ansi_reset(stdout_line);
        let stderr_result = ensure_ansi_reset(stderr_line);

        assert_eq!(stdout_result, "STDOUT: \x1b[32mSuccess message\x1b[0m");
        assert_eq!(stderr_result, "STDERR: \x1b[31mError message\x1b[0m");
    }
}
