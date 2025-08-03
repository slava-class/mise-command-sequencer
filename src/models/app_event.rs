use super::mise_task::{MiseTask, MiseTaskInfo};
use super::sequence::SequenceEvent;
use ratatui::crossterm::event::{KeyCode, MouseButton};

#[derive(Debug, Clone)]
pub enum AppEvent {
    #[allow(dead_code)]
    Quit,
    KeyPress(KeyCode),
    MouseClick {
        button: MouseButton,
        row: u16,
        col: u16,
    },
    TasksRefreshed(Vec<MiseTask>),
    TaskInfoLoaded(Box<MiseTaskInfo>),
    TaskOutput(String),
    TaskCompleted,
    Tick,
    Sequence(SequenceEvent),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_event_simple_variants() {
        let quit = AppEvent::Quit;
        let key_press = AppEvent::KeyPress(KeyCode::Enter);
        let task_completed = AppEvent::TaskCompleted;
        let tick = AppEvent::Tick;

        match quit {
            AppEvent::Quit => assert!(true),
            _ => panic!("Expected Quit variant"),
        }

        match key_press {
            AppEvent::KeyPress(KeyCode::Enter) => assert!(true),
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
}
