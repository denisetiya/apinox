use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub vars: HashMap<String, serde_json::Value>,
}
