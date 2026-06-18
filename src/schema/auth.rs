use serde::{Deserialize, Serialize};

/// Authentication scheme type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Http,
    #[serde(alias = "apiKey")]
    ApiKey,
    Basic,
    Oauth2,
    None,
}

/// Authentication scheme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthScheme {
    pub id: String,
    #[serde(rename = "type")]
    pub auth_type: AuthType,

    // HTTP auth
    #[serde(default)]
    pub scheme: Option<String>, // bearer, basic, etc.
    #[serde(default)]
    pub header: Option<String>, // Authorization, etc.
    #[serde(default)]
    pub prefix: Option<String>, // "Bearer ", etc.

    // API Key auth
    #[serde(default)]
    pub key: Option<String>, // Header key name
    #[serde(default)]
    pub in_location: Option<String>, // "header", "query"

    #[serde(default)]
    pub description: Option<String>,
}

/// Auth configuration block
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub schemes: Vec<AuthScheme>,
}
