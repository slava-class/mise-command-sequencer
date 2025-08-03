#[derive(Debug, PartialEq)]
pub enum AppState {
    Detail(String),
    Running(String),
    SequenceBuilder,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_variants() {
        let detail_state = AppState::Detail("task1".to_string());
        let running_state = AppState::Running("task2".to_string());
        let builder_state = AppState::SequenceBuilder;

        assert_eq!(detail_state, AppState::Detail("task1".to_string()));
        assert_eq!(running_state, AppState::Running("task2".to_string()));
        assert_eq!(builder_state, AppState::SequenceBuilder);
    }

    #[test]
    fn test_app_state_pattern_matching() {
        let states = vec![
            AppState::Detail("test".to_string()),
            AppState::Running("build".to_string()),
            AppState::SequenceBuilder,
        ];

        for state in states {
            match state {
                AppState::Detail(task) => assert!(!task.is_empty()),
                AppState::Running(task) => assert!(!task.is_empty()),
                AppState::SequenceBuilder => assert!(true),
            }
        }
    }
}
