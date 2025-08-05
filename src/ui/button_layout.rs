use ratatui::layout::Rect;

use super::constants::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionButton {
    Run,
    Cat,
    Edit,
    Delete,
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
pub enum ButtonType {
    Action {
        button: ActionButton,
        task_index: usize,
    },
    Sequence(SequenceButton),
    Dialog(DialogButton),
}

pub struct ActionButtonLayout {
    pub run_range: (u16, u16), // (start_col, end_col)
    pub cat_range: (u16, u16),
    pub edit_range: (u16, u16),
    pub delete_range: (u16, u16),
}

impl ActionButtonLayout {
    pub fn new(_actions_rect: &Rect) -> Self {
        // Calculate positions based on actual string lengths
        // Account for potential cell padding by adding an offset
        let cell_padding = 2; // Table cells typically have 1 character padding

        let run_start = cell_padding;
        let run_end = run_start + RUN_BUTTON_TEXT.len() - 1;

        let cat_start = run_end + 1 + BUTTON_SPACING.len();
        let cat_end = cat_start + CAT_BUTTON_TEXT.len() - 1;

        let edit_start = cat_end + 1 + BUTTON_SPACING.len();
        let edit_end = edit_start + EDIT_BUTTON_TEXT.len() - 1;

        let delete_start = edit_end + 1 + BUTTON_SPACING.len();
        let delete_end = delete_start + DELETE_BUTTON_TEXT.len() - 1;

        Self {
            run_range: (run_start as u16, run_end as u16),
            cat_range: (cat_start as u16, cat_end as u16),
            edit_range: (edit_start as u16, edit_end as u16),
            delete_range: (delete_start as u16, delete_end as u16),
        }
    }

    pub fn get_button_at_position(&self, relative_col: u16) -> Option<ActionButton> {
        if (self.run_range.0..=self.run_range.1).contains(&relative_col) {
            Some(ActionButton::Run)
        } else if (self.cat_range.0..=self.cat_range.1).contains(&relative_col) {
            Some(ActionButton::Cat)
        } else if (self.edit_range.0..=self.edit_range.1).contains(&relative_col) {
            Some(ActionButton::Edit)
        } else if (self.delete_range.0..=self.delete_range.1).contains(&relative_col) {
            Some(ActionButton::Delete)
        } else {
            None
        }
    }
}

pub struct SequenceButtonLayout {
    pub run_sequence_range: (u16, u16),
    pub add_as_task_range: (u16, u16),
    pub clear_range: (u16, u16),
}

impl SequenceButtonLayout {
    pub fn new(_controls_start_col: u16) -> Self {
        let run_sequence_start = 0;
        let run_sequence_end = run_sequence_start + RUN_SEQUENCE_BUTTON_TEXT.len() - 1;

        let add_as_task_start = run_sequence_end + 1 + BUTTON_SPACING.len();
        let add_as_task_end = add_as_task_start + ADD_AS_TASK_BUTTON_TEXT.len() - 1;

        let clear_start = add_as_task_end + 1 + BUTTON_SPACING.len();
        let clear_end = clear_start + CLEAR_BUTTON_TEXT.len() - 1;

        Self {
            run_sequence_range: (run_sequence_start as u16, run_sequence_end as u16),
            add_as_task_range: (add_as_task_start as u16, add_as_task_end as u16),
            clear_range: (clear_start as u16, clear_end as u16),
        }
    }

    pub fn get_button_at_position(&self, relative_col: u16) -> Option<SequenceButton> {
        if (self.run_sequence_range.0..=self.run_sequence_range.1).contains(&relative_col) {
            Some(SequenceButton::RunSequence)
        } else if (self.add_as_task_range.0..=self.add_as_task_range.1).contains(&relative_col) {
            Some(SequenceButton::AddAsTask)
        } else if (self.clear_range.0..=self.clear_range.1).contains(&relative_col) {
            Some(SequenceButton::Clear)
        } else {
            None
        }
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

        // With cell_padding = 2, ranges are calculated based on actual text lengths
        let expected_run_end = 2 + RUN_BUTTON_TEXT.len() - 1;
        let expected_cat_start = expected_run_end + 1 + BUTTON_SPACING.len();
        let expected_cat_end = expected_cat_start + CAT_BUTTON_TEXT.len() - 1;
        let expected_edit_start = expected_cat_end + 1 + BUTTON_SPACING.len();
        let expected_edit_end = expected_edit_start + EDIT_BUTTON_TEXT.len() - 1;

        assert_eq!(layout.run_range, (2, expected_run_end as u16));
        assert_eq!(
            layout.cat_range,
            (expected_cat_start as u16, expected_cat_end as u16)
        );
        assert_eq!(
            layout.edit_range,
            (expected_edit_start as u16, expected_edit_end as u16)
        );
    }

    #[test]
    fn test_get_button_at_position_run() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test run button range (2-6)
        assert_eq!(layout.get_button_at_position(2), Some(ActionButton::Run));
        assert_eq!(layout.get_button_at_position(4), Some(ActionButton::Run));
        assert_eq!(layout.get_button_at_position(6), Some(ActionButton::Run));
    }

    #[test]
    fn test_get_button_at_position_cat() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test cat button range (8-12)
        assert_eq!(layout.get_button_at_position(8), Some(ActionButton::Cat));
        assert_eq!(layout.get_button_at_position(10), Some(ActionButton::Cat));
        assert_eq!(layout.get_button_at_position(12), Some(ActionButton::Cat));
    }

    #[test]
    fn test_get_button_at_position_edit() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test edit button range (14-19)
        assert_eq!(layout.get_button_at_position(14), Some(ActionButton::Edit));
        assert_eq!(layout.get_button_at_position(17), Some(ActionButton::Edit));
        assert_eq!(layout.get_button_at_position(19), Some(ActionButton::Edit));
    }

    #[test]
    fn test_get_button_at_position_none() {
        let rect = create_test_rect();
        let layout = ActionButtonLayout::new(&rect);

        // Test positions outside button ranges
        assert_eq!(layout.get_button_at_position(7), None); // Between run and cat
        assert_eq!(layout.get_button_at_position(13), None); // Between cat and edit
        assert_eq!(layout.get_button_at_position(20), None); // After edit
        assert_eq!(layout.get_button_at_position(100), None); // Way outside
    }

    #[test]
    fn test_sequence_button_layout_creation() {
        let layout = SequenceButtonLayout::new(50);

        let expected_run_sequence_end = RUN_SEQUENCE_BUTTON_TEXT.len() - 1;
        let expected_add_as_task_start = expected_run_sequence_end + 1 + BUTTON_SPACING.len();
        let expected_add_as_task_end =
            expected_add_as_task_start + ADD_AS_TASK_BUTTON_TEXT.len() - 1;
        let expected_clear_start = expected_add_as_task_end + 1 + BUTTON_SPACING.len();
        let expected_clear_end = expected_clear_start + CLEAR_BUTTON_TEXT.len() - 1;

        assert_eq!(
            layout.run_sequence_range,
            (0, expected_run_sequence_end as u16)
        );
        assert_eq!(
            layout.add_as_task_range,
            (
                expected_add_as_task_start as u16,
                expected_add_as_task_end as u16
            )
        );
        assert_eq!(
            layout.clear_range,
            (expected_clear_start as u16, expected_clear_end as u16)
        );
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
        assert_ne!(ActionButton::Cat, ActionButton::Edit);
        assert_ne!(ActionButton::Delete, ActionButton::Run);
        assert_ne!(ActionButton::Delete, ActionButton::Cat);
        assert_ne!(ActionButton::Delete, ActionButton::Edit);
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
}
