use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseTask {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub hide: Option<bool>,
    pub alias: Option<String>,
}

impl MiseTask {
    pub fn new(name: String, source: String) -> Self {
        Self {
            name,
            description: None,
            source,
            hide: None,
            alias: None,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.hide.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseTaskInfo {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub file: Option<String>,
    pub dir: Option<String>,
    pub hide: Option<bool>,
    pub alias: Option<String>,
    pub run: Option<serde_json::Value>,
    pub depends: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mise_task_creation() {
        let task = MiseTask::new("test".to_string(), "test-source".to_string());
        assert_eq!(task.name, "test");
        assert_eq!(task.source, "test-source");
        assert_eq!(task.description, None);
        assert_eq!(task.hide, None);
        assert_eq!(task.alias, None);
    }

    #[test]
    fn test_mise_task_is_hidden() {
        let mut task = MiseTask::new("test".to_string(), "test-source".to_string());
        assert!(!task.is_hidden());

        task.hide = Some(true);
        assert!(task.is_hidden());

        task.hide = Some(false);
        assert!(!task.is_hidden());
    }
}
