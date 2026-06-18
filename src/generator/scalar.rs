use anyhow::Result;
use serde_json::{json, Value};

use crate::schema::auth::AuthType;
use crate::schema::root::ApinoxSchema;

/// Generate a standalone Scalar HTML docs page from an ApinoxSchema.
///
/// Embeds the OpenAPI 3.1 spec as inline JSON and loads Scalar from CDN.
/// Fully self-contained — no local dependencies.
pub fn generate(schema: &ApinoxSchema) -> Result<String> {
    let openapi_json = build_openapi_json(schema)?;

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{title} — API Reference</title>
  <style>
    /* Dark theme base */
    body {{
      margin: 0;
      padding: 0;
      background: #0d1117;
      color: #e6edf3;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }}
    html {{
      background: #0d1117;
    }}
    /* Scalar overrides for dark mode */
    .scalar-api-reference {{
      --scalar-color-1: #e6edf3;
      --scalar-color-2: #8b949e;
      --scalar-color-3: #58a6ff;
      --scalar-sidebar-background: #161b22;
      --scalar-sidebar-border-color: #30363d;
      --scalar-customer-theme: dark;
    }}
  </style>
</head>
<body>
  <div id="scalar-api-reference"></div>

  <script id="api-reference" type="application/json">
{openapi_json}
  </script>

  <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"##,
        title = escape_html(&schema.name),
        openapi_json = openapi_json,
    );

    Ok(html)
}

/// Build an OpenAPI 3.1 spec as a JSON string from the schema.
fn build_openapi_json(schema: &ApinoxSchema) -> Result<String> {
    let mut spec = json!({
        "openapi": "3.1.0",
        "info": {
            "title": schema.name,
            "version": schema.version,
        }
    });

    if let Some(ref desc) = schema.description {
        spec["info"]["description"] = json!(desc);
    }

    // Servers
    if !schema.environments.is_empty() {
        let servers: Vec<Value> = schema
            .environments
            .iter()
            .map(|env| {
                let url = env
                    .base_url
                    .clone()
                    .or_else(|| schema.base_url.clone())
                    .unwrap_or_default();
                let mut s = json!({ "url": url, "description": env.name });
                if let Some(ref desc) = env.description {
                    s["description"] = json!(desc);
                }
                s
            })
            .collect();
        spec["servers"] = json!(servers);
    } else if let Some(ref url) = schema.base_url {
        spec["servers"] = json!([{ "url": url }]);
    }

    // Security schemes
    if !schema.auth.schemes.is_empty() {
        let mut schemes = serde_json::Map::new();
        for scheme in &schema.auth.schemes {
            let mut sec = match scheme.auth_type {
                AuthType::Http => {
                    let mut s = json!({ "type": "http" });
                    if let Some(ref sch) = scheme.scheme {
                        s["scheme"] = json!(sch);
                    }
                    s
                }
                AuthType::ApiKey => {
                    let mut s = json!({ "type": "apiKey" });
                    if let Some(ref k) = scheme.key {
                        s["name"] = json!(k);
                    }
                    if let Some(ref loc) = scheme.in_location {
                        s["in"] = json!(loc);
                    }
                    s
                }
                AuthType::Basic => json!({ "type": "http", "scheme": "basic" }),
                AuthType::Oauth2 => json!({ "type": "oauth2" }),
                AuthType::None => json!({ "type": "apiKey", "in": "header", "name": "X-No-Auth" }),
            };
            if let Some(ref desc) = scheme.description {
                sec["description"] = json!(desc);
            }
            schemes.insert(scheme.id.clone(), sec);
        }
        spec["components"] = json!({ "securitySchemes": schemes });
    }

    // Paths
    let mut paths = serde_json::Map::new();
    for ep in &schema.endpoints {
        let method_key = ep.method.to_lowercase();

        let mut operation = json!({
            "operationId": ep.id,
            "summary": ep.name,
        });

        if let Some(ref desc) = ep.description {
            operation["description"] = json!(desc);
        }

        // Tags
        if let Some(ref grp) = ep.group {
            let tag_name = schema
                .groups
                .iter()
                .find(|g| g.id == *grp)
                .map(|g| g.name.clone())
                .unwrap_or_else(|| grp.clone());
            operation["tags"] = json!([tag_name]);
        }

        if ep.deprecated {
            operation["deprecated"] = json!(true);
        }

        // Parameters
        let mut params: Vec<Value> = Vec::new();
        for pp in &ep.path_params {
            params.push(build_param_json(pp, "path"));
        }
        for qp in &ep.query_params {
            params.push(build_param_json(qp, "query"));
        }
        for h in &ep.headers {
            let mut p = json!({
                "name": h.name,
                "in": "header",
                "required": h.required,
            });
            if let Some(ref desc) = h.description {
                p["description"] = json!(desc);
            }
            params.push(p);
        }
        if !params.is_empty() {
            operation["parameters"] = json!(params);
        }

        // Request body
        if let Some(ref body) = ep.body {
            let ct = body
                .content_type
                .as_deref()
                .unwrap_or(match body.body_type.as_str() {
                    "json" => "application/json",
                    "formdata" => "multipart/form-data",
                    "urlencoded" => "application/x-www-form-urlencoded",
                    "binary" => "application/octet-stream",
                    _ => "text/plain",
                });

            let mut media = serde_json::Map::new();

            // Schema
            if let Some(ref schema_fields) = body.schema {
                media.insert("schema".to_string(), build_body_schema_json(schema_fields));
            }

            // First example
            if let Some(ex) = body.examples.first() {
                media.insert("example".to_string(), ex.value.clone());
            }

            let mut content = serde_json::Map::new();
            content.insert(ct.to_string(), Value::Object(media));

            let mut req_body = json!({
                "content": content,
            });
            if !body.required {
                req_body["required"] = json!(false);
            }
            operation["requestBody"] = req_body;
        }

        // Responses
        let mut responses = serde_json::Map::new();
        for resp in &ep.responses {
            let status_str = resp.status.to_string();
            let mut resp_obj = json!({
                "description": if resp.description.is_empty() { "Response".to_string() } else { resp.description.clone() },
            });

            // Response schema
            if let Some(ref schema_fields) = resp.schema {
                let props = build_body_schema_json(schema_fields);
                let media = json!({ "schema": props });
                let content = json!({ "application/json": media });
                resp_obj["content"] = content;
            }

            // First example
            if let Some(ex) = resp.examples.first() {
                let mut ex_obj = json!({
                    "summary": ex.name,
                    "value": ex.value,
                });
                if let Some(ref desc) = ex.description {
                    ex_obj["description"] = json!(desc);
                }
                resp_obj["examples"] = json!({ ex.name.clone(): ex_obj });
            }

            responses.insert(status_str, resp_obj);
        }
        if !responses.is_empty() {
            operation["responses"] = Value::Object(responses);
        }

        // Security
        if let Some(ref auth_ref) = ep.auth {
            operation["security"] = json!([{ auth_ref: [] }]);
        }

        // Insert into paths
        let path_entry = paths.entry(ep.path.clone()).or_insert_with(|| json!({}));
        path_entry[method_key] = operation;
    }

    spec["paths"] = Value::Object(paths);

    Ok(serde_json::to_string_pretty(&spec)?)
}

fn build_param_json(param: &crate::schema::endpoint::Parameter, location: &str) -> Value {
    let mut p = json!({
        "name": param.name,
        "in": location,
        "required": param.required,
        "schema": {
            "type": param.param_type,
        }
    });

    if let Some(ref desc) = param.description {
        p["description"] = json!(desc);
    }

    p
}

fn build_body_schema_json(
    schema_fields: &std::collections::HashMap<String, crate::schema::endpoint::FieldSchema>,
) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for (name, field) in schema_fields {
        let mut prop = json!({
            "type": field.field_type,
        });
        if let Some(ref desc) = field.description {
            prop["description"] = json!(desc);
        }
        if let Some(ref enums) = field.enum_vals {
            prop["enum"] = json!(enums);
        }
        if field.required {
            required.push(name.clone());
        }
        properties.insert(name.clone(), prop);
    }

    let mut obj = json!({
        "type": "object",
        "properties": properties,
    });
    if !required.is_empty() {
        obj["required"] = json!(required);
    }
    obj
}

/// Escape HTML special characters for safe embedding in HTML attributes/content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
