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
