use std::collections::HashMap;

use anyhow::Result;
use serde_json::{json, Value};

use crate::schema::auth::AuthType;
use crate::schema::endpoint::{Body, FieldSchema};
use crate::schema::root::ApinoxSchema;

/// Resolve a response, merging any referenced error pattern with endpoint-specific overrides.
/// Returns (status, description, example).
pub fn resolve_response(schema: &ApinoxSchema, resp: &crate::schema::endpoint::Response) -> (u16, String, Option<Value>) {
    let mut status = resp.status;
    let mut description = resp.description.clone();
    let mut example: Option<Value> = None;

    if let Some(ref pattern_id) = resp.use_pattern {
        if let Some(ref error_responses) = schema.error_responses {
            if let Some(pattern) = error_responses.patterns.iter().find(|p| p.id == *pattern_id) {
                if status == 0 {
                    status = pattern.status;
                }
                if description.is_empty() {
                    description = pattern.description.clone();
                }
                if resp.examples.is_empty() {
                    example = Some(pattern.example.clone());
                }
            }
        }
    }

    if let Some(first_ex) = resp.examples.first() {
        example = Some(first_ex.value.clone());
    }

    (status, description, example)
}

/// Postman Collection v2.1.0 generator
pub fn generate(schema: &ApinoxSchema) -> Result<String> {
    let collection = build_collection(schema)?;
    Ok(serde_json::to_string_pretty(&collection)?)
}

fn build_collection(schema: &ApinoxSchema) -> Result<Value> {
    // Group endpoints
    let grouped = group_endpoints(schema);

    // Build items (folders + requests)
    let mut items: Vec<Value> = Vec::new();

    // Grouped endpoints as folders
    for (group_name, endpoints) in &grouped {
        if endpoints.is_empty() {
            continue;
        }
        let group_items: Vec<Value> = endpoints
            .iter()
            .map(|ep| build_request(schema, ep))
            .collect();

        let group_auth = schema
            .groups
            .iter()
            .find(|g| &g.name == group_name || g.id == group_name.as_str())
            .and_then(|g| g.auth.as_ref())
            .map(|a| build_auth_for_scheme(schema, a));

        let mut folder = json!({
            "name": group_name,
            "item": group_items,
        });

        if let Some(auth) = group_auth {
            folder["auth"] = auth;
        }

        items.push(folder);
    }

    // Ungrouped endpoints at root
    if let Some(ungrouped) = grouped.get("__ungrouped__") {
        for ep in ungrouped {
            items.push(build_request(schema, ep));
        }
    }

    // Collection-level auth
    let collection_auth = schema
        .auth
        .default
        .as_ref()
        .map(|a| build_auth_for_scheme(schema, a));

    // Collection-level variables from first environment
    let variables: Vec<Value> = schema
        .environments
        .first()
        .map(|env| {
            env.vars
                .iter()
                .map(|(k, v)| {
                    json!({
                        "key": k,
                        "value": v.to_string(),
                        "type": "string"
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let mut collection = json!({
        "info": {
            "name": schema.name,
            "description": schema.description.as_deref().unwrap_or(""),
            "_postman_id": "",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items,
        "variable": variables,
    });

    if let Some(auth) = collection_auth {
        collection["auth"] = auth;
    }

    Ok(collection)
}

/// Group endpoints by their group field
fn group_endpoints(
    schema: &ApinoxSchema,
) -> HashMap<String, Vec<&crate::schema::endpoint::Endpoint>> {
    let mut map: HashMap<String, Vec<&crate::schema::endpoint::Endpoint>> = HashMap::new();

    for ep in &schema.endpoints {
        let key = ep
            .group
            .as_ref()
            .map(|g| {
                schema
                    .groups
                    .iter()
                    .find(|grp| grp.id == *g)
                    .map(|grp| grp.name.clone())
                    .unwrap_or_else(|| g.clone())
            })
            .unwrap_or_else(|| "__ungrouped__".to_string());

        map.entry(key).or_default().push(ep);
    }

    map
}

/// Build Postman auth object from a scheme ID
fn build_auth_for_scheme(schema: &ApinoxSchema, scheme_id: &str) -> Value {
    let scheme = schema.auth.schemes.iter().find(|s| s.id == scheme_id);

    match scheme {
        Some(s) => match s.auth_type {
            AuthType::Http => {
                let auth_type = s.scheme.as_deref().unwrap_or("bearer");

                match auth_type {
                    "bearer" => json!({
                        "type": "bearer",
                        "bearer": [{
                            "key": "token",
                            "value": "{{TOKEN}}",
                            "type": "string"
                        }]
                    }),
                    "basic" => json!({
                        "type": "basic",
                        "basic": [
                            {"key": "username", "value": "{{USERNAME}}", "type": "string"},
                            {"key": "password", "value": "{{PASSWORD}}", "type": "string"}
                        ]
                    }),
                    _ => json!({"type": "noauth"}),
                }
            }
            AuthType::ApiKey => {
                let key_name = s.key.as_deref().unwrap_or("X-API-Key");
                json!({
                    "type": "apikey",
                    "apikey": [
                        {"key": "key", "value": key_name, "type": "string"},
                        {"key": "value", "value": "{{API_KEY}}", "type": "string"},
                        {"key": "in", "value": s.in_location.as_deref().unwrap_or("header"), "type": "string"}
                    ]
                })
            }
            AuthType::Basic => json!({
                "type": "basic",
                "basic": [
                    {"key": "username", "value": "{{USERNAME}}", "type": "string"},
                    {"key": "password", "value": "{{PASSWORD}}", "type": "string"}
                ]
            }),
            AuthType::None | AuthType::Oauth2 => json!({"type": "noauth"}),
        },
        None => json!({"type": "noauth"}),
    }
}

/// Build a Postman request item from an endpoint
fn build_request(schema: &ApinoxSchema, ep: &crate::schema::endpoint::Endpoint) -> Value {
    let url = build_url(schema, ep);
    let method = ep.method.to_uppercase();

    // Headers
    let headers: Vec<Value> = ep
        .headers
        .iter()
        .map(|h| {
            let mut hdr = json!({
                "key": h.name,
                "value": h.value.as_deref().unwrap_or(""),
                "type": "text",
            });
            if let Some(ref desc) = h.description {
                hdr["description"] = json!(desc);
            }
            hdr
        })
        .collect();

    // Body
    let body = ep.body.as_ref().map(build_body);

    // Auth
    let auth = ep.auth.as_ref().map(|a| build_auth_for_scheme(schema, a));

    // Examples (using resolve_response for pattern support)
    let examples: Vec<Value> = ep
        .responses
        .iter()
        .flat_map(|resp| {
            let (status, description, resolved_example) = resolve_response(schema, resp);
            if let Some(ref ex_val) = resolved_example {
                vec![json!({
                    "name": format!("{} - {}", status, description),
                    "originalRequest": {},
                    "status": status_text(status),
                    "code": status,
                    "_postman_previewlanguage": "json",
                    "body": serde_json::to_string_pretty(ex_val).unwrap_or_default(),
                })]
            } else if resp.examples.is_empty() {
                vec![json!({
                    "name": format!("{} {}", status, description),
                    "originalRequest": {},
                    "status": status_text(status),
                    "code": status,
                })]
            } else {
                resp.examples
                    .iter()
                    .map(|ex| {
                        json!({
                            "name": format!("{} - {}", status, ex.name),
                            "status": status_text(status),
                            "code": status,
                            "_postman_previewlanguage": "json",
                            "body": serde_json::to_string_pretty(&ex.value).unwrap_or_default(),
                        })
                    })
                    .collect::<Vec<Value>>()
            }
        })
        .collect();

    // Description
    let description = ep.description.as_deref().unwrap_or("");

    let mut request = json!({
        "name": ep.name,
        "request": {
            "method": method,
            "header": headers,
            "url": url,
        },
        "response": examples,
    });

    if let Some(body) = body {
        request["request"]["body"] = body;
    }

    if let Some(auth) = auth {
        request["request"]["auth"] = auth;
    }

    if !description.is_empty() {
        request["request"]["description"] = json!(description);
    }

    if ep.deprecated {
        request["request"]["description"] = json!(format!("⚠ DEPRECATED: {}", description));
    }

    request
}

/// Build URL object for Postman
fn build_url(schema: &ApinoxSchema, ep: &crate::schema::endpoint::Endpoint) -> Value {
    let _base = ep
        .servers
        .as_ref()
        .and_then(|s| s.production.as_ref())
        .or(schema.base_url.as_ref())
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "{{base_url}}".to_string());

    let path = if ep.path.starts_with('/') {
        ep.path.clone()
    } else {
        format!("/{}", ep.path)
    };

    let full_url = format!("{{{{base_url}}}}{}", path);

    // Query params
    let query: Vec<Value> = ep
        .query_params
        .iter()
        .map(|p| {
            let mut q = json!({
                "key": p.name,
                "value": p.default.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "".into()),
                "description": p.description.as_deref().unwrap_or(""),
            });
            if p.required {
                q["disabled"] = json!(false);
            }
            q
        })
        .collect();

    // Path variables
    let path_vars: Vec<Value> = ep
        .path_params
        .iter()
        .map(|p| json!({
            "key": p.name,
            "value": p.example.as_ref().map(|v| v.to_string()).unwrap_or_else(|| format!(":{}", p.name)),
            "description": p.description.as_deref().unwrap_or(""),
        }))
        .collect();

    let mut url_obj = json!({
        "raw": full_url,
        "protocol": "https",
        "host": ["{{base_url}}"],
        "path": ep.path.trim_start_matches('/').split('/').collect::<Vec<&str>>(),
    });

    if !query.is_empty() {
        url_obj["query"] = json!(query);
    }
    if !path_vars.is_empty() {
        url_obj["variable"] = json!(path_vars);
    }

    url_obj
}

/// Build Postman request body
fn build_body(body: &Body) -> Value {
    match body.body_type.as_str() {
        "json" => {
            let example_json = body
                .examples
                .first()
                .map(|ex| serde_json::to_string_pretty(&ex.value).unwrap_or_default())
                .unwrap_or_else(|| {
                    // Generate faker-based example from schema if available
                    let generated = faker_object(&body.schema);
                    serde_json::to_string_pretty(&generated).unwrap_or_else(|_| "{}".into())
                });

            json!({
                "mode": "raw",
                "raw": example_json,
                "options": {
                    "raw": {
                        "language": "json"
                    }
                }
            })
        }
        "formdata" => {
            let fields: Vec<Value> = body
                .fields
                .as_ref()
                .map(|fields| {
                    fields
                        .iter()
                        .map(|f| {
                            if f.field_type == "file" {
                                json!({
                                    "key": f.name,
                                    "type": "file",
                                    "src": "",
                                    "description": f.description.as_deref().unwrap_or(""),
                                })
                            } else {
                                let default_val = if let Some(ref d) = f.default {
                                    d.to_string()
                                } else {
                                    faker_value(&f.field_type, &f.name)
                                };
                                json!({
                                    "key": f.name,
                                    "value": default_val,
                                    "type": "text",
                                    "description": f.description.as_deref().unwrap_or(""),
                                })
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            json!({
                "mode": "formdata",
                "formdata": fields,
            })
        }
        "urlencoded" => {
            let fields: Vec<Value> = body
                .schema
                .as_ref()
                .map(|schema| {
                    schema
                        .iter()
                        .map(|(k, v)| {
                            let val = if let Some(ref d) = v.default {
                                d.to_string()
                            } else {
                                faker_value(&v.field_type, k)
                            };
                            json!({
                                "key": k,
                                "value": val,
                                "type": "text",
                                "description": v.description.as_deref().unwrap_or(""),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            json!({
                "mode": "urlencoded",
                "urlencoded": fields,
            })
        }
        "binary" => json!({
            "mode": "file",
            "file": {
                "src": ""
            }
        }),
        "raw" => {
            let _ct = body.content_type.as_deref().unwrap_or("text/plain");
            let example = body
                .examples
                .first()
                .map(|ex| match &ex.value {
                    serde_json::Value::String(s) => s.clone(),
                    other => serde_json::to_string_pretty(other).unwrap_or_default(),
                })
                .unwrap_or_default();

            json!({
                "mode": "raw",
                "raw": example,
                "options": {
                    "raw": {
                        "language": "text"
                    }
                }
            })
        }
        _ => json!({"mode": "raw", "raw": ""}),
    }
}

/// Generate Postman dynamic variable based on field type/name
fn faker_value(field_type: &str, field_name: &str) -> String {
    // Check name patterns first (more specific)
    let name_lower = field_name.to_lowercase();
    if name_lower.contains("email") {
        return "{{$randomEmail}}".into();
    }
    if name_lower.contains("phone") || name_lower.contains("mobile") {
        return "{{$randomPhoneNumber}}".into();
    }
    if name_lower.contains("name") && !name_lower.contains("user_name") {
        return "{{$randomFullName}}".into();
    }
    if name_lower.contains("url") || name_lower.contains("website") {
        return "{{$randomUrl}}".into();
    }
    if name_lower.contains("city") {
        return "{{$randomCity}}".into();
    }
    if name_lower.contains("country") {
        return "{{$randomCountry}}".into();
    }
    if name_lower.contains("color") {
        return "{{$randomColor}}".into();
    }
    if name_lower.contains("word") || name_lower.contains("title") {
        return "{{$randomWords}}".into();
    }
    if name_lower.contains("price") || name_lower.contains("amount") || name_lower.contains("total") {
        return "{{$randomInt}}".into();
    }
    if name_lower.contains("quantity") || name_lower.contains("count") {
        return "{{$randomInt}}".into();
    }

    // Then check type patterns
    match field_type {
        "email" => "{{$randomEmail}}".into(),
        "uuid" => "{{$guid}}".into(),
        "uri" | "url" => "{{$randomUrl}}".into(),
        "date" => "{{$randomDateFuture}}".into(),
        "datetime" => "{{$timestamp}}".into(),
        "integer" | "float" => "{{$randomInt}}".into(),
        "boolean" => "true".into(),
        _ => "{{$randomWord}}".to_string(),
    }
}

/// Check if a field is sensitive (should be masked in generated output)
#[allow(dead_code)]
fn is_sensitive(field_name: &str) -> bool {
    let name = field_name.to_lowercase();
    name.contains("password")
        || name.contains("secret")
        || name.contains("token")
        || name.contains("api_key")
        || name.contains("apikey")
        || name.contains("authorization")
        || name.contains("private_key")
}

/// HTTP status text
fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}

/// Generate a faker value respecting enum, pattern, sub_fields, and sub_items.
pub fn faker_value_for_field(field: &FieldSchema, field_name: &str) -> Value {
    // 1. Enum-aware: pick from enum if present
    if let Some(ref enums) = field.enum_vals {
        if !enums.is_empty() {
            // Pick first as a reasonable default for docs
            return enums[0].clone();
        }
    }

    // 2. Custom pattern
    if let Some(ref pat) = field.pattern {
        return Value::String(faker_pattern(pat));
    }

    // 3. Nested object (sub_fields)
    if let Some(ref sub_fields) = field.sub_fields {
        let obj = faker_object(&Some(sub_fields.clone()));
        return obj;
    }

    // 4. Array with sub_items
    if field.field_type == "array" {
        if let Some(ref items) = field.sub_items {
            let item1 = faker_value_for_field(items, field_name);
            let item2 = faker_value_for_field(items, field_name);
            return Value::Array(vec![item1, item2]);
        }
        return Value::Array(vec![Value::String("item1".into()), Value::String("item2".into())]);
    }

    // 5. Fallback to basic faker_value as a string
    Value::String(faker_value(&field.field_type, field_name))
}

/// Generate a complete JSON object with faker values for all fields.
pub fn faker_object(fields: &Option<HashMap<String, FieldSchema>>) -> Value {
    match fields {
        Some(schema) if !schema.is_empty() => {
            let map: serde_json::Map<String, Value> = schema
                .iter()
                .map(|(name, field)| {
                    let val = if let Some(ref d) = field.default {
                        d.clone()
                    } else {
                        faker_value_for_field(field, name)
                    };
                    (name.clone(), val)
                })
                .collect();
            Value::Object(map)
        }
        _ => Value::Object(serde_json::Map::new()),
    }
}

/// Generate a value from a custom pattern string.
/// Supports: `prefix_[a-z0-9]+`, `prefix_[0-9]+`, `date:YYYY-MM-DD`, etc.
fn faker_pattern(pattern: &str) -> String {
    // date: format patterns
    if let Some(date_fmt) = pattern.strip_prefix("date:") {
        // Simple date formatting — generate 2026-06-17 style dates
        let now = "2026-06-17";
        return date_fmt
            .replace("YYYY", &now[0..4])
            .replace("MM", &now[5..7])
            .replace("DD", &now[8..10]);
    }

    // Generic pattern: match prefix + character class groups
    // e.g. "usr_[a-z0-9]+" → "usr_a1b2c3"
    // e.g. "ord_[0-9]+" → "ord_001"
    if let Some(open_bracket) = pattern.find('[') {
        let prefix = &pattern[..open_bracket];
        let rest = &pattern[open_bracket..];

        if let Some(close_bracket) = rest.find(']') {
            let char_class = &rest[1..close_bracket];
            let suffix = &rest[close_bracket + 1..]; // typically "+" or empty

            let count = if suffix.contains('+') { 6 } else { 3 };

            let generated: String = (0..count)
                .map(|i| {
                    if char_class.contains('a') && char_class.contains('0') {
                        // mix of alpha and digits
                        if i % 2 == 0 {
                            (b'a' + (i as u8) % 26) as char
                        } else {
                            (b'0' + (i as u8) % 10) as char
                        }
                    } else if char_class.contains('a') || char_class.contains('A') {
                        (b'a' + (i as u8) % 26) as char
                    } else if char_class.contains('0') || char_class.contains('9') {
                        (b'0' + (i as u8) % 10) as char
                    } else {
                        'x'
                    }
                })
                .collect();

            return format!("{}{}", prefix, generated);
        }
    }

    // No recognizable pattern — return the pattern string itself
    pattern.to_string()
}
