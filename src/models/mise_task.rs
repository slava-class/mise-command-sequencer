use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseTask {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub source: String,
    pub depends: Vec<String>,
    pub depends_post: Vec<String>,
    pub wait_for: Vec<String>,
    pub env: Vec<serde_json::Value>,
    pub dir: Option<String>,
    pub hide: bool,
    pub raw: bool,
    pub sources: Vec<String>,
    pub outputs: Vec<String>,
    pub shell: Option<String>,
    pub quiet: bool,
    pub silent: bool,
    pub tools: HashMap<String, serde_json::Value>,
    pub run: Vec<String>,
    pub file: Option<String>,
}

impl MiseTask {
    pub fn new(name: String, source: String) -> Self {
        Self {
            name,
            aliases: Vec::new(),
            description: String::new(),
            source,
            depends: Vec::new(),
            depends_post: Vec::new(),
            wait_for: Vec::new(),
            env: Vec::new(),
            dir: None,
            hide: false,
            raw: false,
            sources: Vec::new(),
            outputs: Vec::new(),
            shell: None,
            quiet: false,
            silent: false,
            tools: HashMap::new(),
            run: Vec::new(),
            file: None,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.hide
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseTaskInfo {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub source: String,
    pub depends: Vec<String>,
    pub depends_post: Vec<String>,
    pub wait_for: Vec<String>,
    pub env: Vec<serde_json::Value>,
    pub dir: Option<String>,
    pub hide: bool,
    pub raw: bool,
    pub sources: Vec<String>,
    pub outputs: Vec<String>,
    pub shell: Option<String>,
    pub quiet: bool,
    pub silent: bool,
    pub tools: HashMap<String, serde_json::Value>,
    pub run: Vec<String>,
    pub file: Option<String>,
    pub usage_spec: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mise_task_creation() {
        let task = MiseTask::new("test".to_string(), "test-source".to_string());
        assert_eq!(task.name, "test");
        assert_eq!(task.source, "test-source");
        assert_eq!(task.description, "");
        assert!(!task.hide);
        assert!(task.aliases.is_empty());
    }

    #[test]
    fn test_mise_task_is_hidden() {
        let mut task = MiseTask::new("test".to_string(), "test-source".to_string());
        assert!(!task.is_hidden());

        task.hide = true;
        assert!(task.is_hidden());

        task.hide = false;
        assert!(!task.is_hidden());
    }
}
