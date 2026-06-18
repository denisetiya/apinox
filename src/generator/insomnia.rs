use anyhow::Result;
use serde_json::{json, Value};

use crate::schema::auth::AuthType;
use crate::schema::endpoint::Body;
use crate::schema::root::ApinoxSchema;

/// Insomnia v4 export generator
pub fn generate(schema: &ApinoxSchema) -> Result<String> {
    let export = build_export(schema)?;
    Ok(serde_json::to_string_pretty(&export)?)
}

/// Generate a unique ID for Insomnia resources
fn make_id(prefix: &str, id: &str) -> String {
    format!("apinox_{}_{}", prefix, id)
}

/// Build the Insomnia v4 export envelope
fn build_export(schema: &ApinoxSchema) -> Result<Value> {
    let mut resources: Vec<Value> = Vec::new();

    // Workspace
    let workspace_id = make_id("ws", &schema.name.to_lowercase().replace(' ', "_"));
    resources.push(json!({
        "_id": workspace_id,
        "_type": "workspace",
        "name": schema.name,
        "description": schema.description.as_deref().unwrap_or(""),
        "scope": "collection",
    }));

    // Workspace cookie jar (empty, Insomnia expects it)
    resources.push(json!({
        "_id": format!("{}_cookie_jar", workspace_id),
        "_type": "cookie_jar",
        "parentId": workspace_id,
        "cookies": [],
    }));

    // Environment resources — one per ApinoxEnvironment, plus sub-environments
    let base_env_id = format!("{}_env_base", workspace_id);
    let mut env_ids: Vec<String> = Vec::new();

    if schema.environments.is_empty() {
        // Default base environment
        resources.push(json!({
            "_id": base_env_id,
            "_type": "environment",
            "parentId": workspace_id,
            "name": "Base Environment",
            "data": {
                "base_url": schema.base_url.as_deref().unwrap_or("http://localhost"),
            },
        }));
        env_ids.push(base_env_id);
    } else {
        // First environment is the base
        let first = &schema.environments[0];
        let base_vars = build_env_data(schema, first);
        resources.push(json!({
            "_id": base_env_id,
            "_type": "environment",
            "parentId": workspace_id,
            "name": "Base Environment",
            "data": base_vars,
        }));
        env_ids.push(base_env_id.clone());

        // Sub-environments for remaining environments
        for env in schema.environments.iter().skip(1) {
            let sub_id = make_id("env", &env.name.to_lowercase().replace(' ', "_"));
            let sub_vars = build_env_data(schema, env);
            resources.push(json!({
                "_id": sub_id,
                "_type": "environment",
                "parentId": base_env_id,
                "name": env.name,
                "data": sub_vars,
            }));
            env_ids.push(sub_id);
        }
    }

    // Request groups from schema groups
    for group in &schema.groups {
        let group_id = make_id("folder", &group.id);
        resources.push(json!({
            "_id": group_id,
            "_type": "request_group",
            "parentId": workspace_id,
            "name": group.name,
            "description": group.description.as_deref().unwrap_or(""),
        }));
    }

    // Collect group ID mappings for endpoints
    let group_id_map: std::collections::HashMap<String, String> = schema
        .groups
        .iter()
        .map(|g| (g.id.clone(), make_id("folder", &g.id)))
        .collect();

    // Endpoints → request resources
    for ep in &schema.endpoints {
        let request_id = make_id("req", &ep.id);
        let parent_id = ep
            .group
            .as_ref()
            .and_then(|g| group_id_map.get(g))
            .cloned()
            .unwrap_or_else(|| workspace_id.clone());

        let mut request = json!({
            "_id": request_id,
            "_type": "request",
            "parentId": parent_id,
            "name": ep.name,
            "method": ep.method.to_uppercase(),
            "url": build_url(schema, ep),
            "headers": build_headers(ep),
            "parameters": build_query_params(ep),
        });

        // Body
        if let Some(ref body) = ep.body {
            request["body"] = build_body(body);
        }

        // Description
        let mut desc = ep.description.as_deref().unwrap_or("").to_string();
        if ep.deprecated {
            desc = format!("⚠ DEPRECATED: {}", desc);
        }
        if !desc.is_empty() {
            request["description"] = json!(desc);
        }

        // Authentication
        let auth = resolve_auth(schema, ep);
        if let Some(auth_obj) = auth {
            request["authentication"] = auth_obj;
        }

        resources.push(request);
    }

    // Collection-level auth as a sub-environment variable hint
    // Insomnia handles auth per-request; we embed it via the authentication field above.

    Ok(json!({
        "__export_format": 4,
        "__export_source": "apinox",
        "__export_date": chrono::Utc::now().to_rfc3339(),
        "resources": resources,
    }))
}

/// Build environment data map from an Apinox Environment
fn build_env_data(
    schema: &ApinoxSchema,
    env: &crate::schema::environment::Environment,
) -> serde_json::Map<String, Value> {
    let mut data = serde_json::Map::new();

    // Base URL
    let base = env
        .base_url
        .as_deref()
        .or(schema.base_url.as_deref())
        .unwrap_or("http://localhost");
    data.insert("base_url".to_string(), json!(base));

    // Environment-specific vars
    for (k, v) in &env.vars {
        data.insert(k.clone(), v.clone());
    }

    data
}

/// Build the full URL for an endpoint
fn build_url(schema: &ApinoxSchema, ep: &crate::schema::endpoint::Endpoint) -> String {
    let base = ep
        .servers
        .as_ref()
        .and_then(|s| s.production.as_ref())
        .or(schema.base_url.as_ref())
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "{{ _.base_url }}".to_string());

    let path = if ep.path.starts_with('/') {
        ep.path.clone()
    } else {
        format!("/{}", ep.path)
    };

    format!("{}{}", base, path)
}

/// Build Insomnia headers array from endpoint headers
fn build_headers(ep: &crate::schema::endpoint::Endpoint) -> Value {
    let headers: Vec<Value> = ep
        .headers
        .iter()
        .map(|h| {
            json!({
                "name": h.name,
                "value": h.value.as_deref().unwrap_or(""),
                "description": h.description.as_deref().unwrap_or(""),
                "disabled": false,
            })
        })
        .collect();

    json!(headers)
}

/// Build Insomnia query parameters array
fn build_query_params(ep: &crate::schema::endpoint::Endpoint) -> Value {
    let params: Vec<Value> = ep
        .query_params
        .iter()
        .map(|p| {
            let default_val = p
                .default
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            json!({
                "name": p.name,
                "value": default_val,
                "description": p.description.as_deref().unwrap_or(""),
                "disabled": false,
            })
        })
        .collect();

    json!(params)
}

/// Build Insomnia body object from endpoint body
fn build_body(body: &Body) -> Value {
    match body.body_type.as_str() {
        "json" => {
            let example = body
                .examples
                .first()
                .map(|ex| serde_json::to_string_pretty(&ex.value).unwrap_or_default())
                .unwrap_or_else(|| "{}".to_string());

            json!({
                "mimeType": "application/json",
                "text": example,
            })
        }
        "formdata" => {
            let params: Vec<Value> = body
                .fields
                .as_ref()
                .map(|fields| {
                    fields
                        .iter()
                        .map(|f| {
                            let default_val = f
                                .default
                                .as_ref()
                                .map(|v| v.to_string())
                                .unwrap_or_default();
                            json!({
                                "name": f.name,
                                "value": default_val,
                                "type": f.field_type,
                                "description": f.description.as_deref().unwrap_or(""),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            json!({
                "mimeType": "multipart/form-data",
                "params": params,
            })
        }
        "urlencoded" => {
            let params: Vec<Value> = body
                .schema
                .as_ref()
                .map(|schema| {
                    schema
                        .iter()
                        .map(|(k, v)| {
                            let default = v
                                .default
                                .as_ref()
                                .map(|d| d.to_string())
                                .unwrap_or_default();
                            json!({
                                "name": k,
                                "value": default,
                                "description": v.description.as_deref().unwrap_or(""),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            json!({
                "mimeType": "application/x-www-form-urlencoded",
                "params": params,
            })
        }
        "binary" => {
            json!({
                "mimeType": body
                    .mime_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
                "fileName": "",
            })
        }
        "raw" => {
            let ct = body.content_type.as_deref().unwrap_or("text/plain");
            let example = body
                .examples
                .first()
                .map(|ex| match &ex.value {
                    serde_json::Value::String(s) => s.clone(),
                    other => serde_json::to_string_pretty(other).unwrap_or_default(),
                })
                .unwrap_or_default();

            json!({
                "mimeType": ct,
                "text": example,
            })
        }
        _ => json!({
            "mimeType": "text/plain",
            "text": "",
        }),
    }
}

/// Resolve authentication for an endpoint
/// Returns Insomnia authentication object
fn resolve_auth(schema: &ApinoxSchema, ep: &crate::schema::endpoint::Endpoint) -> Option<Value> {
    // Endpoint-level auth override takes priority
    let scheme_id = ep.auth.as_deref().or(schema.auth.default.as_deref())?;

    let scheme = schema.auth.schemes.iter().find(|s| s.id == scheme_id)?;

    Some(match scheme.auth_type {
        AuthType::Http => {
            let auth_type = scheme.scheme.as_deref().unwrap_or("bearer");
            match auth_type {
                "bearer" => json!({
                    "type": "bearer",
                    "token": "{{ _.token }}",
                    "prefix": scheme.prefix.as_deref().unwrap_or("Bearer"),
                }),
                "basic" => json!({
                    "type": "basic",
                    "username": "{{ _.username }}",
                    "password": "{{ _.password }}",
                }),
                _ => json!({ "type": "none" }),
            }
        }
        AuthType::ApiKey => {
            let key_name = scheme.key.as_deref().unwrap_or("X-API-Key");
            json!({
                "type": "apikey",
                "key": key_name,
                "value": "{{ _.api_key }}",
                "in": scheme.in_location.as_deref().unwrap_or("header"),
            })
        }
        AuthType::Basic => json!({
            "type": "basic",
            "username": "{{ _.username }}",
            "password": "{{ _.password }}",
        }),
        AuthType::Oauth2 => json!({
            "type": "oauth2",
            "access_token": "{{ _.access_token }}",
            "token_type": "bearer",
        }),
        AuthType::None => json!({ "type": "none" }),
    })
}
