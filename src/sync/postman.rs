use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Configuration for syncing to Postman
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanSyncConfig {
    pub api_key: String,
    pub workspace_id: String,
    pub collection_id: Option<String>,
}

/// Result of a sync operation
#[derive(Debug)]
pub struct SyncResult {
    pub collection_id: String,
    pub collection_uid: String,
    pub url: String,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub workspace_type: String,
}

const POSTMAN_API: &str = "https://api.getpostman.com";

fn agent() -> ureq::Agent {
    ureq::Agent::new_with_defaults()
}

fn parse_json_response(mut resp: ureq::http::Response<ureq::Body>) -> Result<serde_json::Value> {
    let body_str = resp.body_mut().read_to_string().context("Failed to read response body")?;
    serde_json::from_str(&body_str).context("Failed to parse JSON response")
}

/// Sync a Postman collection JSON to a Postman workspace.
pub fn sync_collection(config: &PostmanSyncConfig, collection_json: &str) -> Result<SyncResult> {
    let collection_value: serde_json::Value =
        serde_json::from_str(collection_json).context("Failed to parse collection JSON")?;

    let a = agent();
    let mut request_body = HashMap::new();
    request_body.insert("collection", &collection_value);

    match &config.collection_id {
        Some(id) => {
            let url = format!("{}/collections/{}", POSTMAN_API, id);
            let resp = a
                .put(&url)
                .header("X-Api-Key", &config.api_key)
                .header("Content-Type", "application/json")
                .send_json(&request_body)
                .context("Failed to PUT to Postman API")?;

            let body = parse_json_response(resp)?;
            let collection = body
                .get("collection")
                .or_else(|| body.get("data"))
                .context("No collection in PUT response")?;

            let cid = collection
                .get("uid")
                .or_else(|| collection.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string();

            let cuid = collection
                .get("uid")
                .and_then(|v| v.as_str())
                .unwrap_or(&cid)
                .to_string();

            Ok(SyncResult {
                url: format!("https://www.postman.com/collection/{}", cuid),
                collection_id: cid,
                collection_uid: cuid,
                action: "updated".to_string(),
            })
        }
        None => {
            let url = format!("{}/collections", POSTMAN_API);
            let resp = a
                .post(&url)
                .header("X-Api-Key", &config.api_key)
                .header("Content-Type", "application/json")
                .send_json(&request_body)
                .context("Failed to POST to Postman API")?;

            let body = parse_json_response(resp)?;
            let collection = body
                .get("collection")
                .or_else(|| body.get("data"))
                .context("No collection in POST response")?;

            let cid = collection
                .get("uid")
                .or_else(|| collection.get("id"))
                .and_then(|v| v.as_str())
                .context("No collection UID in response")?
                .to_string();

            let cuid = collection
                .get("uid")
                .and_then(|v| v.as_str())
                .unwrap_or(&cid)
                .to_string();

            Ok(SyncResult {
                url: format!("https://www.postman.com/collection/{}", cuid),
                collection_id: cid,
                collection_uid: cuid,
                action: "created".to_string(),
            })
        }
    }
}

/// List all workspaces accessible with the given API key.
pub fn list_workspaces(api_key: &str) -> Result<Vec<WorkspaceInfo>> {
    let a = agent();
    let resp = a
        .get("https://api.getpostman.com/workspaces")
        .header("X-Api-Key", api_key)
        .call()
        .context("Failed to list workspaces")?;

    let body = parse_json_response(resp)?;
    let workspaces = body
        .get("workspaces")
        .and_then(|v| v.as_array())
        .context("No workspaces array in response")?;

    let result: Vec<WorkspaceInfo> = workspaces
        .iter()
        .filter_map(|ws| {
            let id = ws.get("id")?.as_str()?.to_string();
            let name = ws.get("name")?.as_str()?.to_string();
            let wt = ws
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("personal")
                .to_string();
            Some(WorkspaceInfo {
                id,
                name,
                workspace_type: wt,
            })
        })
        .collect();

    Ok(result)
}

/// Find a collection by name in a workspace.
pub fn find_collection(api_key: &str, workspace_id: &str, name: &str) -> Result<Option<String>> {
    let a = agent();
    let url = format!(
        "https://api.getpostman.com/collections?workspace={}",
        workspace_id
    );

    let resp = a
        .get(&url)
        .header("X-Api-Key", api_key)
        .call()
        .context("Failed to list collections")?;

    let body = parse_json_response(resp)?;
    let collections = body
        .get("collections")
        .and_then(|v| v.as_array())
        .context("No collections array in response")?;

    for col in collections {
        if let Some(col_name) = col.get("name").and_then(|v| v.as_str()) {
            if col_name == name {
                let uid = col
                    .get("uid")
                    .or_else(|| col.get("id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                return Ok(uid);
            }
        }
    }

    Ok(None)
}
