use serde::{Deserialize, Serialize};

use super::auth::AuthConfig;
use super::endpoint::Endpoint;
use super::environment::Environment;

/// Include directive for modular schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    pub path: String,
    #[serde(default)]
    pub prefix: Option<String>,
}

/// Changelog entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub endpoint: String,
    pub description: String,
    #[serde(default)]
    pub from_type: Option<String>,
    #[serde(default)]
    pub to_type: Option<String>,
}

/// Version changelog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChangelog {
    pub version: String,
    pub date: String,
    pub changes: Vec<ChangelogEntry>,
}

/// Error pattern definition (reusable across endpoints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: String,
    pub status: u16,
    pub description: String,
    pub example: serde_json::Value,
}

/// Error responses container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorResponses {
    #[serde(default)]
    pub patterns: Vec<ErrorPattern>,
}

/// Root Apinox schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApinoxSchema {
    pub apinox: String, // Schema spec version "1.0"
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,

    #[serde(default)]
    pub environments: Vec<Environment>,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub groups: Vec<GroupDef>,

    #[serde(default)]
    pub endpoints: Vec<Endpoint>,

    #[serde(default)]
    pub includes: Vec<Include>,

    #[serde(default)]
    pub changelog: Vec<VersionChangelog>,

    #[serde(default)]
    pub error_responses: Option<ErrorResponses>,
}

/// Group definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub auth: Option<String>,
}
