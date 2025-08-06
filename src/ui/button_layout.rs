use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::constants::*;

/// Generic button range representing start and end positions
type ButtonRange = (u16, u16);

/// Calculate sequential button ranges from a list of button texts
/// Each button is placed after the previous one with spacing
fn calculate_sequential_button_ranges(
    button_texts: &[&str],
    start_position: u16,
) -> Vec<ButtonRange> {
    let mut ranges = Vec::new();
    let mut current_pos = start_position;

    for (i, &text) in button_texts.iter().enumerate() {
        let start = current_pos;
        let end = start + text.len() as u16 - 1;
        ranges.push((start, end));

        // Add spacing for next button (except for the last one)
        if i < button_texts.len() - 1 {
            current_pos = end + 1 + BUTTON_SPACING.len() as u16;
        }
    }

    ranges
}

/// Find which button range contains the given position
fn find_button_at_position<T: Copy>(
    ranges: &[ButtonRange],
    buttons: &[T],
    position: u16,
) -> Option<T> {
    for (range, &button) in ranges.iter().zip(buttons.iter()) {
        if (range.0..=range.1).contains(&position) {
            return Some(button);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionButton {
    Run,
    Cat,
    Edit,
    Rename,
    Delete,
    Save,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SequenceButton {
    RunSequence,
    AddAsTask,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogButton {
    Delete,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepButton {
    Step1,
    Step2,
    Step3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType {
    Action {
        button: ActionButton,
        task_index: usize,
    },
    Sequence(SequenceButton),
    Dialog(DialogButton),
    Step {
        step_index: usize,
        task_index: usize,
    },
}

pub struct ActionButtonLayout {
    ranges: Vec<ButtonRange>,
    buttons: Vec<ActionButton>,
    rename_mode: bool,
}

impl ActionButtonLayout {
    pub fn new(_actions_rect: &Rect) -> Self {
        Self::new_with_mode(_actions_rect, false)
    }

    pub fn new_with_mode(_actions_rect: &Rect, rename_mode: bool) -> Self {
        if rename_mode {
            // Rename mode: only show save and cancel buttons
            const RENAME_BUTTON_TEXTS: &[&str] = &[
                crate::ui::constants::SAVE_BUTTON_TEXT,
                crate::ui::constants::CANCEL_BUTTON_TEXT,
            ];
            const RENAME_BUTTONS: &[ActionButton] = &[ActionButton::Save, ActionButton::Cancel];

            let ranges = calculate_sequential_button_ranges(RENAME_BUTTON_TEXTS, 0);

            Self {
                ranges,
                buttons: RENAME_BUTTONS.to_vec(),
                rename_mode,
            }
        } else {
            // Normal mode: show all action buttons
            const ACTION_BUTTON_TEXTS: &[&str] = &[
                RUN_BUTTON_TEXT,
                CAT_BUTTON_TEXT,
                EDIT_BUTTON_TEXT,
                RENAME_BUTTON_TEXT,
                DELETE_BUTTON_TEXT,
            ];
            const ACTION_BUTTONS: &[ActionButton] = &[
                ActionButton::Run,
                ActionButton::Cat,
                ActionButton::Edit,
                ActionButton::Rename,
                ActionButton::Delete,
            ];

            let ranges = calculate_sequential_button_ranges(ACTION_BUTTON_TEXTS, 0);

            Self {
                ranges,
                buttons: ACTION_BUTTONS.to_vec(),
                rename_mode,
            }
        }
    }

    pub fn get_button_at_position(&self, relative_col: u16) -> Option<ActionButton> {
        find_button_at_position(&self.ranges, &self.buttons, relative_col)
    }

    // Compatibility methods for existing code that accesses ranges directly
    pub fn run_range(&self) -> ButtonRange {
        if !self.rename_mode {
            self.ranges[0]
        } else {
            (0, 0)
        }
    }
    pub fn cat_range(&self) -> ButtonRange {
        if !self.rename_mode {
            self.ranges[1]
        } else {
            (0, 0)
        }
    }
    pub fn edit_range(&self) -> ButtonRange {
        if !self.rename_mode {
            self.ranges[2]
        } else {
            (0, 0)
        }
    }
    pub fn rename_range(&self) -> ButtonRange {
        if !self.rename_mode {
            self.ranges[3]
        } else {
            (0, 0)
        }
    }
    pub fn delete_range(&self) -> ButtonRange {
        if !self.rename_mode {
            self.ranges[4]
        } else {
            (0, 0)
        }
    }
    pub fn save_range(&self) -> ButtonRange {
        if self.rename_mode {
            self.ranges[0]
        } else {
            (0, 0)
        }
    }
    pub fn cancel_range(&self) -> ButtonRange {
        if self.rename_mode {
            self.ranges[1]
        } else {
            (0, 0)
        }
    }
}

pub struct SequenceButtonLayout {
    ranges: Vec<ButtonRange>,
    buttons: Vec<SequenceButton>,
}

impl SequenceButtonLayout {
    pub fn new(_controls_start_col: u16) -> Self {
        // Define the sequence of sequence buttons
        const SEQUENCE_BUTTON_TEXTS: &[&str] = &[
            RUN_SEQUENCE_BUTTON_TEXT,
            ADD_AS_TASK_BUTTON_TEXT,
            CLEAR_BUTTON_TEXT,
        ];
        const SEQUENCE_BUTTONS: &[SequenceButton] = &[
            SequenceButton::RunSequence,
            SequenceButton::AddAsTask,
            SequenceButton::Clear,
        ];

        let ranges = calculate_sequential_button_ranges(SEQUENCE_BUTTON_TEXTS, 0);

        Self {
            ranges,
            buttons: SEQUENCE_BUTTONS.to_vec(),
        }
    }

    pub fn get_button_at_position(&self, relative_col: u16) -> Option<SequenceButton> {
        find_button_at_position(&self.ranges, &self.buttons, relative_col)
    }

    // Compatibility methods for existing code that accesses ranges directly
    pub fn run_sequence_range(&self) -> ButtonRange {
        self.ranges[0]
    }
    pub fn add_as_task_range(&self) -> ButtonRange {
        self.ranges[1]
    }
    pub fn clear_range(&self) -> ButtonRange {
        self.ranges[2]
    }
}

pub fn get_dialog_button_at_position(
    dialog_area: Rect,
    click_row: u16,
    click_col: u16,
) -> Option<DialogButton> {
    use crate::ui::constants::{CANCEL_DIALOG_BUTTON_TEXT, DELETE_DIALOG_BUTTON_TEXT};

    // Check if click is within dialog area
    if click_row < dialog_area.y
        || click_row >= dialog_area.y + dialog_area.height
        || click_col < dialog_area.x
        || click_col >= dialog_area.x + dialog_area.width
    {
        return None;
    }

    // The buttons are on the last content line of the dialog (accounting for borders)
    let button_row = dialog_area.y + dialog_area.height - 2; // -2 for border and button line
    if click_row != button_row {
        return None;
    }

    // Button positions relative to dialog start
    // [Delete]     [Cancel]
    // Buttons are centered in the dialog with some padding
    let content_width = dialog_area.width.saturating_sub(4); // Account for borders
    let delete_button_len = DELETE_DIALOG_BUTTON_TEXT.len() as u16;
    let cancel_button_len = CANCEL_DIALOG_BUTTON_TEXT.len() as u16;
    let gap = 5; // Space between buttons
    let total_buttons_width = delete_button_len + gap + cancel_button_len;

    // Center the buttons within the dialog
    let buttons_start_offset = (content_width.saturating_sub(total_buttons_width)) / 2;
    let content_start = dialog_area.x + 2; // Account for border

    let delete_button_start = content_start + buttons_start_offset + 1; // +1 to align with actual rendering
    let delete_button_end = delete_button_start + delete_button_len;
    let cancel_button_start = delete_button_end + gap;
    let cancel_button_end = cancel_button_start + cancel_button_len;

    if click_col >= delete_button_start && click_col < delete_button_end {
        Some(DialogButton::Delete)
    } else if click_col >= cancel_button_start && click_col < cancel_button_end {
        Some(DialogButton::Cancel)
    } else {
        None
    }
}

pub struct StepButtonLayout {
    range: ButtonRange,
    num_steps: usize,
}

impl StepButtonLayout {
    pub fn new(_step_rect: &Rect) -> Self {
        use crate::ui::constants::STEP_1_TEXT; // All step texts have the same length

        // Step buttons start at position 0 and have the same width
        Self {
            range: (0, STEP_1_TEXT.len() as u16 - 1),
            num_steps: 3,
        }
    }

    pub fn get_step_button_at_position(
        &self,
        step_index: usize,
        relative_col: u16,
    ) -> Option<usize> {
        // Validate step index
        if step_index >= self.num_steps {
            return None;
        }

        // All steps have the same range
        if (self.range.0..=self.range.1).contains(&relative_col) {
            Some(step_index)
        } else {
            None
        }
    }

    // Compatibility methods for existing code
    pub fn step_1_range(&self) -> ButtonRange {
        self.range
    }
    pub fn step_2_range(&self) -> ButtonRange {
        self.range
    }
    pub fn step_3_range(&self) -> ButtonRange {
        self.range
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonHoverState {
    pub button_type: ButtonType,
    pub row: u16,
    pub col: u16,
}

impl ButtonHoverState {
    pub fn new(button_type: ButtonType, row: u16, col: u16) -> Self {
        Self {
            button_type,
            row,
            col,
        }
    }
}

/// Theme definition for different button types
#[derive(Debug, Clone, Copy)]
pub enum ButtonTheme {
    Action {
        normal: Color,
        hover_bg: Color,
        hover_fg: Color,
    },
    Sequence {
        normal: Color,
        hover_bg: Color,
        hover_fg: Color,
    },
    Dialog {
        normal: Color,
        hover_bg: Color,
        hover_fg: Color,
    },
    Step {
        enabled_bg: Color,
        enabled_fg: Color,
        disabled: Color,
        hover_bg: Color,
        hover_fg: Color,
    },
}

impl ButtonTheme {
    pub const ACTION_RUN: Self = Self::Action {
        normal: Color::Cyan,
        hover_bg: Color::Green,
        hover_fg: Color::Black,
    };

    pub const ACTION_CAT: Self = Self::Action {
        normal: Color::Cyan,
        hover_bg: Color::Blue,
        hover_fg: Color::White,
    };

    pub const ACTION_EDIT: Self = Self::Action {
        normal: Color::Cyan,
        hover_bg: Color::Magenta,
        hover_fg: Color::White,
    };

    pub const ACTION_RENAME: Self = Self::Action {
        normal: Color::Yellow,
        hover_bg: Color::Yellow,
        hover_fg: Color::Black,
    };

    pub const ACTION_DELETE: Self = Self::Action {
        normal: Color::Red,
        hover_bg: Color::Red,
        hover_fg: Color::White,
    };

    pub const ACTION_SAVE: Self = Self::Action {
        normal: Color::Green,
        hover_bg: Color::Green,
        hover_fg: Color::Black,
    };

    pub const ACTION_CANCEL: Self = Self::Action {
        normal: Color::Gray,
        hover_bg: Color::Gray,
        hover_fg: Color::Black,
    };

    pub const SEQUENCE: Self = Self::Sequence {
        normal: Color::Blue,
        hover_bg: Color::Green,
        hover_fg: Color::Black,
    };

    pub const SEQUENCE_CLEAR: Self = Self::Sequence {
        normal: Color::Blue,
        hover_bg: Color::Red,
        hover_fg: Color::White,
    };

    pub const SEQUENCE_ADD: Self = Self::Sequence {
        normal: Color::Blue,
        hover_bg: Color::Cyan,
        hover_fg: Color::Black,
    };

    pub const DIALOG_DELETE: Self = Self::Dialog {
        normal: Color::Red,
        hover_bg: Color::Red,
        hover_fg: Color::White,
    };

    pub const DIALOG_CANCEL: Self = Self::Dialog {
        normal: Color::Gray,
        hover_bg: Color::Gray,
        hover_fg: Color::Black,
    };

    pub const STEP: Self = Self::Step {
        enabled_bg: Color::Green,
        enabled_fg: Color::Black,
        disabled: Color::DarkGray,
        hover_bg: Color::Green,
        hover_fg: Color::Black,
    };
}

/// Semantic compression for button styling patterns
pub struct ButtonStyleManager;

impl ButtonStyleManager {
    /// Create button style based on state and theme
    pub fn create_button_style(
        theme: ButtonTheme,
        is_hovered: bool,
        is_selected: bool,
        is_enabled: Option<bool>, // For step buttons
    ) -> Style {
        if is_hovered {
            return Self::hovered_style(theme);
        }

        if is_selected {
            return Self::selected_style(theme);
        }

        if let Some(enabled) = is_enabled {
            return Self::state_style(theme, enabled);
        }

        Self::normal_style(theme)
    }

    fn hovered_style(theme: ButtonTheme) -> Style {
        match theme {
            ButtonTheme::Action {
                hover_bg, hover_fg, ..
            } => Style::default().bg(hover_bg).fg(hover_fg),
            ButtonTheme::Sequence {
                hover_bg, hover_fg, ..
            } => Style::default().bg(hover_bg).fg(hover_fg),
            ButtonTheme::Dialog {
                hover_bg, hover_fg, ..
            } => Style::default().bg(hover_bg).fg(hover_fg),
            ButtonTheme::Step {
                hover_bg, hover_fg, ..
            } => Style::default().bg(hover_bg).fg(hover_fg),
        }
    }

    fn selected_style(theme: ButtonTheme) -> Style {
        match theme {
            ButtonTheme::Action { normal, .. } => {
                Style::default().fg(normal).add_modifier(Modifier::BOLD)
            }
            ButtonTheme::Sequence { normal, .. } => {
                Style::default().fg(normal).add_modifier(Modifier::BOLD)
            }
            ButtonTheme::Dialog { normal, .. } => {
                Style::default().fg(normal).add_modifier(Modifier::BOLD)
            }
            ButtonTheme::Step {
                enabled_bg,
                enabled_fg,
                ..
            } => Style::default()
                .bg(enabled_bg)
                .fg(enabled_fg)
                .add_modifier(Modifier::BOLD),
        }
    }

    fn state_style(theme: ButtonTheme, is_enabled: bool) -> Style {
        match theme {
            ButtonTheme::Step {
                enabled_bg,
                enabled_fg,
                disabled,
                ..
            } => {
                if is_enabled {
                    Style::default()
                        .bg(enabled_bg)
                        .fg(enabled_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(disabled)
                }
            }
            other => Self::normal_style(other), // Fall back to normal for non-step buttons
        }
    }

    fn normal_style(theme: ButtonTheme) -> Style {
        match theme {
            ButtonTheme::Action { normal, .. } => Style::default().fg(normal),
            ButtonTheme::Sequence { normal, .. } => Style::default().fg(normal),
            ButtonTheme::Dialog { normal, .. } => Style::default().fg(normal),
            ButtonTheme::Step {
                enabled_bg,
                enabled_fg,
                ..
            } => Style::default()
                .bg(enabled_bg)
                .fg(enabled_fg)
                .add_modifier(Modifier::BOLD),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn create_test_rect() -> Rect {
        Rect {
            x: 10,
            y: 5,
            width: 20,
            height: 1,
        }
    }

    #[test]
    fn test_action_button_layout_creation() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test that ranges are calculated correctly using our semantic compression
        const ACTION_BUTTON_TEXTS: &[&str] = &[
            RUN_BUTTON_TEXT,
            CAT_BUTTON_TEXT,
            EDIT_BUTTON_TEXT,
            RENAME_BUTTON_TEXT,
            DELETE_BUTTON_TEXT,
        ];
        let expected_ranges = calculate_sequential_button_ranges(ACTION_BUTTON_TEXTS, 0);

        assert_eq!(layout.run_range(), expected_ranges[0]);
        assert_eq!(layout.cat_range(), expected_ranges[1]);
        assert_eq!(layout.edit_range(), expected_ranges[2]);
        assert_eq!(layout.rename_range(), expected_ranges[3]);
        assert_eq!(layout.delete_range(), expected_ranges[4]);
    }

    #[test]
    fn test_get_button_at_position_run() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test run button range (0-4)
        assert_eq!(layout.get_button_at_position(0), Some(ActionButton::Run));
        assert_eq!(layout.get_button_at_position(2), Some(ActionButton::Run));
        assert_eq!(layout.get_button_at_position(4), Some(ActionButton::Run));
    }

    #[test]
    fn test_get_button_at_position_cat() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test cat button range (6-10)
        assert_eq!(layout.get_button_at_position(6), Some(ActionButton::Cat));
        assert_eq!(layout.get_button_at_position(8), Some(ActionButton::Cat));
        assert_eq!(layout.get_button_at_position(10), Some(ActionButton::Cat));
    }

    #[test]
    fn test_get_button_at_position_edit() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test edit button range (12-17)
        assert_eq!(layout.get_button_at_position(12), Some(ActionButton::Edit));
        assert_eq!(layout.get_button_at_position(15), Some(ActionButton::Edit));
        assert_eq!(layout.get_button_at_position(17), Some(ActionButton::Edit));
    }

    #[test]
    fn test_get_button_at_position_rename() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test rename button range (19-26)
        assert_eq!(
            layout.get_button_at_position(19),
            Some(ActionButton::Rename)
        );
        assert_eq!(
            layout.get_button_at_position(22),
            Some(ActionButton::Rename)
        );
        assert_eq!(
            layout.get_button_at_position(26),
            Some(ActionButton::Rename)
        );
    }

    #[test]
    fn test_get_button_at_position_none() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test positions outside button ranges
        assert_eq!(layout.get_button_at_position(5), None); // Between run and cat
        assert_eq!(layout.get_button_at_position(11), None); // Between cat and edit
        assert_eq!(layout.get_button_at_position(18), None); // Between edit and rename
        assert_eq!(layout.get_button_at_position(27), None); // Between rename and delete
        assert_eq!(layout.get_button_at_position(35), None); // After delete
        assert_eq!(layout.get_button_at_position(100), None); // Way outside
    }

    #[test]
    fn test_sequence_button_layout_creation() {
        let layout = SequenceButtonLayout::new(50);

        // Test that ranges are calculated correctly using our semantic compression
        const SEQUENCE_BUTTON_TEXTS: &[&str] = &[
            RUN_SEQUENCE_BUTTON_TEXT,
            ADD_AS_TASK_BUTTON_TEXT,
            CLEAR_BUTTON_TEXT,
        ];
        let expected_ranges = calculate_sequential_button_ranges(SEQUENCE_BUTTON_TEXTS, 0);

        assert_eq!(layout.run_sequence_range(), expected_ranges[0]);
        assert_eq!(layout.add_as_task_range(), expected_ranges[1]);
        assert_eq!(layout.clear_range(), expected_ranges[2]);
    }

    #[test]
    fn test_sequence_button_get_button_at_position() {
        let layout = SequenceButtonLayout::new(50);

        // Test run sequence button range (0-13)
        assert_eq!(
            layout.get_button_at_position(0),
            Some(SequenceButton::RunSequence)
        );
        assert_eq!(
            layout.get_button_at_position(7),
            Some(SequenceButton::RunSequence)
        );
        assert_eq!(
            layout.get_button_at_position(13),
            Some(SequenceButton::RunSequence)
        );

        // Test add as task button range (15-27)
        assert_eq!(
            layout.get_button_at_position(15),
            Some(SequenceButton::AddAsTask)
        );
        assert_eq!(
            layout.get_button_at_position(21),
            Some(SequenceButton::AddAsTask)
        );
        assert_eq!(
            layout.get_button_at_position(27),
            Some(SequenceButton::AddAsTask)
        );

        // Test clear button range (29-35)
        assert_eq!(
            layout.get_button_at_position(29),
            Some(SequenceButton::Clear)
        );
        assert_eq!(
            layout.get_button_at_position(32),
            Some(SequenceButton::Clear)
        );
        assert_eq!(
            layout.get_button_at_position(35),
            Some(SequenceButton::Clear)
        );

        // Test positions outside ranges
        assert_eq!(layout.get_button_at_position(14), None); // Between run sequence and add
        assert_eq!(layout.get_button_at_position(28), None); // Between add and clear
        assert_eq!(layout.get_button_at_position(36), None); // After clear
    }

    #[test]
    fn test_button_hover_state_creation() {
        let button_type = ButtonType::Action {
            button: ActionButton::Run,
            task_index: 5,
        };
        let hover_state = ButtonHoverState::new(button_type, 10, 20);

        assert_eq!(hover_state.row, 10);
        assert_eq!(hover_state.col, 20);
        match hover_state.button_type {
            ButtonType::Action { button, task_index } => {
                assert_eq!(button, ActionButton::Run);
                assert_eq!(task_index, 5);
            }
            _ => panic!("Expected Action button type"),
        }
    }

    #[test]
    fn test_action_button_enum_variants() {
        // Test that all ActionButton variants are different
        assert_ne!(ActionButton::Run, ActionButton::Cat);
        assert_ne!(ActionButton::Run, ActionButton::Edit);
        assert_ne!(ActionButton::Run, ActionButton::Rename);
        assert_ne!(ActionButton::Run, ActionButton::Delete);
        assert_ne!(ActionButton::Run, ActionButton::Save);
        assert_ne!(ActionButton::Run, ActionButton::Cancel);
        assert_ne!(ActionButton::Cat, ActionButton::Edit);
        assert_ne!(ActionButton::Cat, ActionButton::Rename);
        assert_ne!(ActionButton::Delete, ActionButton::Rename);
        assert_ne!(ActionButton::Save, ActionButton::Cancel);
    }

    #[test]
    fn test_sequence_button_enum_variants() {
        // Test that all SequenceButton variants are different
        assert_ne!(SequenceButton::RunSequence, SequenceButton::Clear);
    }

    #[test]
    fn test_dialog_button_enum_variants() {
        // Test that all DialogButton variants are different
        assert_ne!(DialogButton::Delete, DialogButton::Cancel);
    }

    #[test]
    fn test_get_dialog_button_at_position_delete() {
        let dialog_area = Rect {
            x: 10,
            y: 5,
            width: 40,
            height: 11,
        };

        // Test click on delete button
        // Button row is at y + height - 2 = 5 + 11 - 2 = 14
        let button_row = 14;

        // Calculate expected delete button position
        // content_start = 10 + 2 = 12
        // content_width = 40 - 4 = 36
        // delete_button_len = "[Delete]".len() = 8
        // cancel_button_len = "[Cancel]".len() = 8
        // gap = 5
        // total_buttons_width = 8 + 5 + 8 = 21
        // buttons_start_offset = (36 - 21) / 2 = 7
        // delete_button_start = 12 + 7 + 1 = 20
        let delete_button_start = 20;
        let delete_button_end = delete_button_start + 8; // 28

        // Test clicks within delete button range
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, delete_button_start),
            Some(DialogButton::Delete)
        );
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, delete_button_start + 3),
            Some(DialogButton::Delete)
        );
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, delete_button_end - 1),
            Some(DialogButton::Delete)
        );
    }

    #[test]
    fn test_get_dialog_button_at_position_cancel() {
        let dialog_area = Rect {
            x: 10,
            y: 5,
            width: 40,
            height: 11,
        };

        let button_row = 14; // y + height - 2

        // Calculate expected cancel button position
        // From previous test: delete_button_end = 28, gap = 5
        // cancel_button_start = 28 + 5 = 33
        let cancel_button_start = 33;
        let cancel_button_end = cancel_button_start + 8; // 41

        // Test clicks within cancel button range
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, cancel_button_start),
            Some(DialogButton::Cancel)
        );
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, cancel_button_start + 3),
            Some(DialogButton::Cancel)
        );
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, cancel_button_end - 1),
            Some(DialogButton::Cancel)
        );
    }

    #[test]
    fn test_get_dialog_button_at_position_outside_dialog() {
        let dialog_area = Rect {
            x: 10,
            y: 5,
            width: 40,
            height: 11,
        };

        // Test clicks outside dialog area
        assert_eq!(get_dialog_button_at_position(dialog_area, 0, 20), None); // Above dialog
        assert_eq!(get_dialog_button_at_position(dialog_area, 20, 20), None); // Below dialog
        assert_eq!(get_dialog_button_at_position(dialog_area, 10, 5), None); // Left of dialog
        assert_eq!(get_dialog_button_at_position(dialog_area, 10, 55), None); // Right of dialog
    }

    #[test]
    fn test_get_dialog_button_at_position_wrong_row() {
        let dialog_area = Rect {
            x: 10,
            y: 5,
            width: 40,
            height: 11,
        };

        // Test clicks on wrong rows within dialog
        assert_eq!(get_dialog_button_at_position(dialog_area, 6, 25), None); // Wrong row
        assert_eq!(get_dialog_button_at_position(dialog_area, 13, 25), None); // Wrong row
        assert_eq!(get_dialog_button_at_position(dialog_area, 15, 25), None); // Wrong row
    }

    #[test]
    fn test_get_dialog_button_at_position_between_buttons() {
        let dialog_area = Rect {
            x: 10,
            y: 5,
            width: 40,
            height: 11,
        };

        let button_row = 14;

        // Test clicks in the gap between buttons (around column 28-33)
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, 28),
            None
        );
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, 30),
            None
        );
        assert_eq!(
            get_dialog_button_at_position(dialog_area, button_row, 32),
            None
        );
    }

    #[test]
    fn test_action_button_layout_rename_mode() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new_with_mode(&rect, true);

        // Test that rename mode layout has correct buttons and ranges
        assert_eq!(layout.get_button_at_position(0), Some(ActionButton::Save));
        assert_eq!(layout.get_button_at_position(3), Some(ActionButton::Save));
        assert_eq!(layout.get_button_at_position(5), Some(ActionButton::Save));

        // Test cancel button range (starts after save button + spacing)
        // "[save]" = 6 chars, " " = 1 char spacing, "[cancel]" starts at position 7
        assert_eq!(layout.get_button_at_position(7), Some(ActionButton::Cancel));
        assert_eq!(
            layout.get_button_at_position(10),
            Some(ActionButton::Cancel)
        );
        assert_eq!(
            layout.get_button_at_position(14),
            Some(ActionButton::Cancel)
        );

        // Test outside ranges
        assert_eq!(layout.get_button_at_position(6), None); // Between buttons
        assert_eq!(layout.get_button_at_position(15), None); // After cancel button
    }

    #[test]
    fn test_action_button_layout_normal_mode() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new_with_mode(&rect, false);

        // Test that normal mode still works
        assert_eq!(layout.get_button_at_position(0), Some(ActionButton::Run));
        assert_eq!(layout.get_button_at_position(6), Some(ActionButton::Cat));
        assert_eq!(layout.get_button_at_position(12), Some(ActionButton::Edit));
    }

    #[test]
    fn test_save_and_cancel_range_methods() {
        let rect = create_test_rect();

        // Test rename mode ranges
        let rename_layout = ActionButtonLayout::new_with_mode(&rect, true);
        assert_eq!(rename_layout.save_range(), (0, 5)); // "[save]" = 6 chars, so end at 5
        assert_eq!(rename_layout.cancel_range(), (7, 14)); // "[cancel]" = 8 chars starting at 7

        // Test normal mode ranges (should be (0, 0))
        let normal_layout = ActionButtonLayout::new_with_mode(&rect, false);
        assert_eq!(normal_layout.save_range(), (0, 0));
        assert_eq!(normal_layout.cancel_range(), (0, 0));
    }

    #[test]
    fn test_normal_mode_ranges_in_rename_mode() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new_with_mode(&rect, true);

        // All normal mode ranges should return (0, 0) when in rename mode
        assert_eq!(layout.run_range(), (0, 0));
        assert_eq!(layout.cat_range(), (0, 0));
        assert_eq!(layout.edit_range(), (0, 0));
        assert_eq!(layout.rename_range(), (0, 0));
        assert_eq!(layout.delete_range(), (0, 0));
    }
}
