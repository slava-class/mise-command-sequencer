use super::mise_task::MiseTask;
use super::sequence::SequenceEvent;
use ratatui::crossterm::event::{KeyEvent, MouseButton};

#[cfg(test)]
use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    KeyPress(KeyEvent),
    MouseClick {
        button: MouseButton,
        row: u16,
        col: u16,
    },
    MouseScroll {
        direction: ScrollDirection,
        row: u16,
        col: u16,
    },
    MouseMove {
        row: u16,
        col: u16,
    },
    TasksRefreshed(Vec<MiseTask>),
    TaskOutput(String),
    TaskCompleted,
    TaskCancelled,
    Tick,
    Sequence(SequenceEvent),
}

#[derive(Debug, Clone)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_app_event_simple_variants() {
        let quit = AppEvent::Quit;
        let key_event = KeyEvent::new(
            KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::NONE,
        );
        let key_press = AppEvent::KeyPress(key_event);
        let task_completed = AppEvent::TaskCompleted;
        let tick = AppEvent::Tick;

        match quit {
            AppEvent::Quit => assert!(true),
            _ => panic!("Expected Quit variant"),
        }

        match key_press {
            AppEvent::KeyPress(key) => assert_eq!(key.code, KeyCode::Enter),
            _ => panic!("Expected KeyPress variant"),
        }

        match task_completed {
            AppEvent::TaskCompleted => assert!(true),
            _ => panic!("Expected TaskCompleted variant"),
        }

        match tick {
            AppEvent::Tick => assert!(true),
            _ => panic!("Expected Tick variant"),
        }
    }

    #[test]
    fn test_mouse_click_event() {
        let mouse_event = AppEvent::MouseClick {
            button: MouseButton::Left,
            row: 10,
            col: 20,
        };

        match mouse_event {
            AppEvent::MouseClick { button, row, col } => {
                assert_eq!(button, MouseButton::Left);
                assert_eq!(row, 10);
                assert_eq!(col, 20);
            }
            _ => panic!("Expected MouseClick variant"),
        }
    }

    #[test]
    fn test_task_output_event() {
        let output_event = AppEvent::TaskOutput("test output".to_string());

        match output_event {
            AppEvent::TaskOutput(output) => {
                assert_eq!(output, "test output");
            }
            _ => panic!("Expected TaskOutput variant"),
        }
    }

    #[test]
    fn test_tasks_refreshed_event() {
        let task = MiseTask::new("test".to_string(), "source".to_string());
        let tasks_event = AppEvent::TasksRefreshed(vec![task]);

        match tasks_event {
            AppEvent::TasksRefreshed(tasks) => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].name, "test");
            }
            _ => panic!("Expected TasksRefreshed variant"),
        }
    }

    #[test]
    fn test_sequence_event() {
        let seq_event = AppEvent::Sequence(SequenceEvent::RunSequence);

        match seq_event {
            AppEvent::Sequence(SequenceEvent::RunSequence) => assert!(true),
            _ => panic!("Expected Sequence variant with RunSequence"),
        }
    }

    #[test]
    fn test_mouse_scroll_event() {
        let scroll_up = AppEvent::MouseScroll {
            direction: ScrollDirection::Up,
            row: 5,
            col: 10,
        };

        let scroll_down = AppEvent::MouseScroll {
            direction: ScrollDirection::Down,
            row: 15,
            col: 25,
        };

        match scroll_up {
            AppEvent::MouseScroll {
                direction,
                row,
                col,
            } => {
                assert!(matches!(direction, ScrollDirection::Up));
                assert_eq!(row, 5);
                assert_eq!(col, 10);
            }
            _ => panic!("Expected MouseScroll variant"),
        }

        match scroll_down {
            AppEvent::MouseScroll {
                direction,
                row,
                col,
            } => {
                assert!(matches!(direction, ScrollDirection::Down));
                assert_eq!(row, 15);
                assert_eq!(col, 25);
            }
            _ => panic!("Expected MouseScroll variant"),
        }
    }

    #[test]
    fn test_mouse_move_event() {
        let mouse_move = AppEvent::MouseMove { row: 12, col: 34 };

        match mouse_move {
            AppEvent::MouseMove { row, col } => {
                assert_eq!(row, 12);
                assert_eq!(col, 34);
            }
            _ => panic!("Expected MouseMove variant"),
        }
    }

    #[test]
    fn test_mouse_move_event_creation() {
        let event = AppEvent::MouseMove { row: 0, col: 0 };

        // Test that we can create MouseMove events with edge case values
        match event {
            AppEvent::MouseMove { row, col } => {
                assert_eq!(row, 0);
                assert_eq!(col, 0);
            }
            _ => panic!("Expected MouseMove variant"),
        }

        let event_max = AppEvent::MouseMove {
            row: u16::MAX,
            col: u16::MAX,
        };

        match event_max {
            AppEvent::MouseMove { row, col } => {
                assert_eq!(row, u16::MAX);
                assert_eq!(col, u16::MAX);
            }
            _ => panic!("Expected MouseMove variant"),
        }
    }
}
