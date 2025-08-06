#[derive(Debug, PartialEq)]
pub enum AppState {
    Detail(String),
    Running(String),
    SequenceBuilder,
    Renaming(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_variants() {
        let detail_state = AppState::Detail("task1".to_string());
        let running_state = AppState::Running("task2".to_string());
        let builder_state = AppState::SequenceBuilder;
        let renaming_state = AppState::Renaming("task3".to_string());

        assert_eq!(detail_state, AppState::Detail("task1".to_string()));
        assert_eq!(running_state, AppState::Running("task2".to_string()));
        assert_eq!(builder_state, AppState::SequenceBuilder);
        assert_eq!(renaming_state, AppState::Renaming("task3".to_string()));
    }

    #[test]
    fn test_app_state_pattern_matching() {
        let states = vec![
            AppState::Detail("test".to_string()),
            AppState::Running("build".to_string()),
            AppState::SequenceBuilder,
            AppState::Renaming("rename".to_string()),
        ];

        for state in states {
            match state {
                AppState::Detail(task) => assert!(!task.is_empty()),
                AppState::Running(task) => assert!(!task.is_empty()),
                AppState::SequenceBuilder => assert!(true),
                AppState::Renaming(task) => assert!(!task.is_empty()),
            }
        }
    }
}
