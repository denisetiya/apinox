use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter definition (shared for path, query, header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(default = "default_param_type", rename = "type")]
    pub param_type: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub example: Option<serde_json::Value>,

    // Type constraints
    #[serde(default, rename = "min")]
    pub min_val: Option<f64>,
    #[serde(default, rename = "max")]
    pub max_val: Option<f64>,
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default, rename = "enum")]
    pub enum_vals: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,

    // Array
    #[serde(default)]
    pub items: Option<Box<Parameter>>,
    #[serde(default)]
    pub max_items: Option<usize>,
    #[serde(default)]
    pub min_items: Option<usize>,

    // Object
    #[serde(default)]
    pub fields: Option<HashMap<String, Parameter>>,
}

fn default_param_type() -> String {
    "string".to_string()
}

fn default_true() -> bool {
    true
}

/// Header definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub example: Option<String>,
}

/// Body field definition (for formdata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub description: Option<String>,

    // File-specific
    #[serde(default)]
    pub accept: Option<Vec<String>>,
    #[serde(default)]
    pub max_size_mb: Option<u64>,

    // Array-specific
    #[serde(default)]
    pub items: Option<Box<BodyField>>,
    #[serde(default)]
    pub max: Option<usize>,

    // String-specific
    #[serde(default, rename = "enum")]
    pub enum_vals: Option<Vec<String>>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

/// Body definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "type")]
    pub body_type: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,

    // Inline schema for json/urlencoded
    #[serde(default)]
    pub schema: Option<HashMap<String, FieldSchema>>,

    // Explicit fields for formdata
    #[serde(default)]
    pub fields: Option<Vec<BodyField>>,

    // Multiple examples
    #[serde(default)]
    pub examples: Vec<BodyExample>,

    // Binary
    #[serde(default)]
    pub encoding: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// Field schema (for json body)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    #[serde(default = "default_param_type", rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default, rename = "enum")]
    pub enum_vals: Option<Vec<serde_json::Value>>,
    #[serde(default, rename = "min")]
    pub min_val: Option<f64>,
    #[serde(default, rename = "max")]
    pub max_val: Option<f64>,
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub sub_fields: Option<HashMap<String, FieldSchema>>,
    #[serde(default)]
    pub sub_items: Option<Box<FieldSchema>>,
}

/// Body example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyExample {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub value: serde_json::Value,
}

/// Response example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseExample {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub value: serde_json::Value,
}

/// Response definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub schema: Option<HashMap<String, FieldSchema>>,
    #[serde(default)]
    pub examples: Vec<ResponseExample>,
    #[serde(default)]
    pub use_pattern: Option<String>, // reference to ErrorResponses pattern id
}

/// Per-endpoint server override
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndpointServers {
    #[serde(default)]
    pub production: Option<String>,
    #[serde(default)]
    pub development: Option<String>,
    #[serde(default)]
    pub staging: Option<String>,
}

/// Rate limit info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests: u64,
    pub window: String,
}

/// Inline auth override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthOverride {
    pub auth_type: String,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
}

/// Endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub tags: Option<Vec<String>>,

    // Auth
    #[serde(default)]
    pub auth: Option<String>,
    #[serde(default)]
    pub auth_override: Option<AuthOverride>,

    // Server override
    #[serde(default)]
    pub servers: Option<EndpointServers>,

    // Rate limit
    #[serde(default)]
    pub rate_limit: Option<RateLimit>,

    // Parameters
    #[serde(default)]
    pub path_params: Vec<Parameter>,
    #[serde(default)]
    pub query_params: Vec<Parameter>,
    #[serde(default)]
    pub headers: Vec<Header>,

    // Body
    #[serde(default)]
    pub body: Option<Body>,

    // Responses
    #[serde(default)]
    pub responses: Vec<Response>,
}
