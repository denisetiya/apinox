use anyhow::Result;

use crate::schema::root::ApinoxSchema;

/// Generate Markdown API documentation from an ApinoxSchema.
pub fn generate(schema: &ApinoxSchema) -> Result<String> {
    let mut out = String::with_capacity(4096);

    // ── Title & meta ──────────────────────────────────────────────
    out.push_str(&format!("# {} API Documentation\n\n", schema.name));
    out.push_str(&format!("**Version:** `{}`\n", schema.version));
    if let Some(ref desc) = schema.description {
        out.push_str(&format!("\n{}\n", desc));
    }
    if let Some(ref url) = schema.base_url {
        out.push_str(&format!("\n**Base URL:** `{}`\n", url));
    }

    // ── Environments ──────────────────────────────────────────────
    if !schema.environments.is_empty() {
        out.push_str("\n## Environments\n\n");
        for env in &schema.environments {
            let url = env
                .base_url
                .as_deref()
                .or(schema.base_url.as_deref())
                .unwrap_or("—");
            out.push_str(&format!("- **{}** — `{}`", env.name, url));
            if let Some(ref d) = env.description {
                out.push_str(&format!(" — {}", d));
            }
            out.push('\n');
        }
    }

    // ── Authentication ────────────────────────────────────────────
    if !schema.auth.schemes.is_empty() {
        out.push_str("\n## Authentication\n\n");
        if let Some(ref default_id) = schema.auth.default {
            out.push_str(&format!("**Default scheme:** `{}`\n\n", default_id));
        }
        for scheme in &schema.auth.schemes {
            let type_label = match scheme.auth_type {
                crate::schema::auth::AuthType::Http => {
                    if let Some(ref s) = scheme.scheme {
                        format!("HTTP {}", s.to_uppercase())
                    } else {
                        "HTTP".to_string()
                    }
                }
                crate::schema::auth::AuthType::ApiKey => "API Key".to_string(),
                crate::schema::auth::AuthType::Basic => "HTTP Basic".to_string(),
                crate::schema::auth::AuthType::Oauth2 => "OAuth 2.0".to_string(),
                crate::schema::auth::AuthType::None => "None".to_string(),
            };
            out.push_str(&format!("### {} ({})\n\n", scheme.id, type_label));
            if let Some(ref desc) = scheme.description {
                out.push_str(&format!("{}\n\n", desc));
            }
            if let Some(ref hdr) = scheme.header {
                out.push_str(&format!("**Header:** `{}`\n", hdr));
            }
            if let Some(ref prefix) = scheme.prefix {
                out.push_str(&format!("**Prefix:** `{}`\n", prefix));
            }
            if let Some(ref key) = scheme.key {
                out.push_str(&format!("**Key:** `{}`\n", key));
            }
            if let Some(ref loc) = scheme.in_location {
                out.push_str(&format!("**Location:** `{}`\n", loc));
            }
            out.push('\n');
        }
    }

    // ── Group endpoints ───────────────────────────────────────────
    // Collect unique group ids present in endpoints
    let mut seen_groups: Vec<Option<String>> = Vec::new();
    for ep in &schema.endpoints {
        if !seen_groups.contains(&ep.group) {
            seen_groups.push(ep.group.clone());
        }
    }

    // Order: group definitions first, then unknown groups
    let mut ordered_groups = Vec::new();
    for gd in &schema.groups {
        ordered_groups.push(Some(gd.id.clone()));
    }
    for g in &seen_groups {
        if !ordered_groups.contains(g) {
            ordered_groups.push(g.clone());
        }
    }

    let mut grouped: Vec<(Option<String>, Vec<&crate::schema::endpoint::Endpoint>)> = Vec::new();
    for gid_opt in &ordered_groups {
        let eps: Vec<&crate::schema::endpoint::Endpoint> = schema
            .endpoints
            .iter()
            .filter(|e| e.group == *gid_opt)
            .collect();
        if eps.is_empty() {
            continue;
        }
        grouped.push((gid_opt.clone(), eps));
    }

    // Table of contents
    if !grouped.is_empty() {
        out.push_str("## Table of Contents\n\n");
        for (gid, eps) in &grouped {
            let title = group_title(schema, gid.as_deref());
            let anchor = make_anchor(&title);
            out.push_str(&format!(
                "- [{} ({} endpoint{})](#{})\n",
                title,
                eps.len(),
                if eps.len() == 1 { "" } else { "s" },
                anchor
            ));
        }
        out.push('\n');
    }

    // ── Endpoints per group ───────────────────────────────────────
    for (gid, eps) in &grouped {
        let title = group_title(schema, gid.as_deref());
        out.push_str(&format!("## {}\n\n", title));

        // Group description
        if let Some(gd) = schema.groups.iter().find(|g| Some(g.id.clone()) == *gid) {
            if let Some(ref desc) = gd.description {
                out.push_str(&format!("{}\n\n", desc));
            }
        }

        for ep in eps {
            render_endpoint(schema, ep, &mut out);
        }
    }

    Ok(out)
}

// ──────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────

fn group_title(schema: &ApinoxSchema, gid: Option<&str>) -> String {
    match gid {
        Some(id) => {
            if let Some(gd) = schema.groups.iter().find(|g| g.id == id) {
                gd.name.clone()
            } else {
                id.to_string()
            }
        }
        None => "Other Endpoints".to_string(),
    }
}

fn method_badge(method: &str) -> &'static str {
    match method.to_uppercase().as_str() {
        "GET" => "🟢",
        "POST" => "🔵",
        "PUT" => "🟠",
        "DELETE" => "🔴",
        "PATCH" => "🟣",
        "HEAD" => "⚪",
        "OPTIONS" => "⚪",
        _ => "⚪",
    }
}

fn make_anchor(text: &str) -> String {
    text.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

fn render_endpoint(
    schema: &ApinoxSchema,
    ep: &crate::schema::endpoint::Endpoint,
    out: &mut String,
) {
    let badge = method_badge(&ep.method);
    let method_upper = ep.method.to_uppercase();
    let deprecated_tag = if ep.deprecated {
        " ⚠️ *DEPRECATED*"
    } else {
        ""
    };

    out.push_str(&format!(
        "### {} `{}` {}\n\n",
        badge, ep.path, deprecated_tag
    ));
    out.push_str(&format!("**{}** — `{}`\n\n", ep.name, method_upper));

    if let Some(ref desc) = ep.description {
        out.push_str(&format!("{}\n\n", desc));
    }

    // Tags
    if let Some(ref tags) = ep.tags {
        if !tags.is_empty() {
            out.push_str(&format!(
                "**Tags:** {}\n\n",
                tags.iter()
                    .map(|t| format!("`{}`", t))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Rate limit
    if let Some(ref rl) = ep.rate_limit {
        out.push_str(&format!(
            "**Rate Limit:** {} requests per {}\n\n",
            rl.requests, rl.window
        ));
    }

    // Path parameters
    if !ep.path_params.is_empty() {
        out.push_str("#### Path Parameters\n\n");
        out.push_str("| Name | Type | Required | Description |\n");
        out.push_str("|------|------|----------|-------------|\n");
        for p in &ep.path_params {
            let req = if p.required { "✅" } else { "❌" };
            let desc = p.description.as_deref().unwrap_or("—");
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                p.name, p.param_type, req, desc
            ));
        }
        out.push('\n');
    }

    // Query parameters
    if !ep.query_params.is_empty() {
        out.push_str("#### Query Parameters\n\n");
        out.push_str("| Name | Type | Required | Description |\n");
        out.push_str("|------|------|----------|-------------|\n");
        for p in &ep.query_params {
            let req = if p.required { "✅" } else { "❌" };
            let desc = p.description.as_deref().unwrap_or("—");
            let deprecated = if p.deprecated { " *(deprecated)*" } else { "" };
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {}{} |\n",
                p.name, p.param_type, req, desc, deprecated
            ));
        }
        out.push('\n');
    }

    // Headers
    if !ep.headers.is_empty() {
        out.push_str("#### Headers\n\n");
        out.push_str("| Name | Value | Required | Description |\n");
        out.push_str("|------|-------|----------|-------------|\n");
        for h in &ep.headers {
            let req = if h.required { "✅" } else { "❌" };
            let desc = h.description.as_deref().unwrap_or("—");
            let val = h
                .value
                .as_deref()
                .map(|v| format!("`{}`", v))
                .unwrap_or_else(|| "—".to_string());
            out.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                h.name, val, req, desc
            ));
        }
        out.push('\n');
    }

    // Request body
    if let Some(ref body) = ep.body {
        out.push_str("#### Request Body\n\n");
        out.push_str(&format!("**Content-Type:** `{}`\n", body.body_type));
        if let Some(ref ct) = body.content_type {
            out.push_str(&format!("**Actual Content-Type:** `{}`\n", ct));
        }
        if body.required {
            out.push_str("**Required:** ✅\n");
        } else {
            out.push_str("**Required:** ❌\n");
        }
        if let Some(ref desc) = body.description {
            out.push_str(&format!("\n{}\n", desc));
        }

        // Schema fields (json/urlencoded)
        if let Some(ref schema_fields) = body.schema {
            out.push_str("\n**Schema:**\n\n");
            out.push_str("| Field | Type | Required | Description |\n");
            out.push_str("|-------|------|----------|-------------|\n");
            for (name, field) in schema_fields {
                let req = if field.required { "✅" } else { "❌" };
                let desc = field.description.as_deref().unwrap_or("—");
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    name, field.field_type, req, desc
                ));
            }
            out.push('\n');
        }

        // Formdata fields
        if let Some(ref fields) = body.fields {
            out.push_str("\n**Fields:**\n\n");
            out.push_str("| Name | Type | Required | Description |\n");
            out.push_str("|------|------|----------|-------------|\n");
            for f in fields {
                let req = if f.required { "✅" } else { "❌" };
                let desc = f.description.as_deref().unwrap_or("—");
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    f.name, f.field_type, req, desc
                ));
            }
            out.push('\n');
        }

        // Body examples
        if !body.examples.is_empty() {
            out.push_str("**Examples:**\n\n");
            for ex in &body.examples {
                out.push_str(&format!("*{}*", ex.name));
                if let Some(ref desc) = ex.description {
                    out.push_str(&format!(" — {}", desc));
                }
                out.push_str("\n\n");
                if let Ok(pretty) = serde_json::to_string_pretty(&ex.value) {
                    out.push_str(&format!("```json\n{}\n```\n\n", pretty));
                }
            }
        }
    }

    // cURL example
    render_curl(schema, ep, out);

    // Responses
    if !ep.responses.is_empty() {
        out.push_str("#### Responses\n\n");
        for resp in &ep.responses {
            let status_emoji = match resp.status {
                200..=299 => "✅",
                300..=399 => "🔀",
                400..=499 => "⚠️",
                500..=599 => "❌",
                _ => "❓",
            };
            out.push_str(&format!(
                "##### {} {} `{}`\n\n",
                status_emoji, resp.status, resp.description
            ));

            // Response headers
            if !resp.headers.is_empty() {
                out.push_str("**Response Headers:**\n\n");
                for h in &resp.headers {
                    let val = h.value.as_deref().unwrap_or("*dynamic*");
                    out.push_str(&format!("- `{}`: `{}`\n", h.name, val));
                }
                out.push('\n');
            }

            // Response examples
            for ex in &resp.examples {
                out.push_str(&format!("*{}*\n\n", ex.name));
                if let Some(ref desc) = ex.description {
                    out.push_str(&format!("{}\n\n", desc));
                }
                if let Ok(pretty) = serde_json::to_string_pretty(&ex.value) {
                    out.push_str(&format!("```json\n{}\n```\n\n", pretty));
                }
            }
        }
    }

    out.push_str("---\n\n");
}

fn render_curl(schema: &ApinoxSchema, ep: &crate::schema::endpoint::Endpoint, out: &mut String) {
    out.push_str("#### Example cURL\n\n");

    let base = schema
        .base_url
        .as_deref()
        .unwrap_or("https://api.example.com");

    let mut path = ep.path.clone();

    // Substitute path params with example values
    for pp in &ep.path_params {
        let placeholder = format!("{{{}}}", pp.name);
        let value = match &pp.example {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => match pp.param_type.as_str() {
                "integer" => "1".to_string(),
                "uuid" => "550e8400-e29b-41d4-a716-446655440000".to_string(),
                _ => format!("{}-example", pp.name),
            },
        };
        path = path.replace(&placeholder, &value);
    }

    let full_url = format!("{}{}", base.trim_end_matches('/'), path);

    let method_upper = ep.method.to_uppercase();

    // Build header flags
    let mut header_flags: Vec<String> = Vec::new();
    for h in &ep.headers {
        let val = h
            .example
            .as_deref()
            .or(h.value.as_deref())
            .unwrap_or("value");
        header_flags.push(format!("  -H '{}: {}'", h.name, val));
    }

    // Build body flags
    let mut body_flags: Vec<String> = Vec::new();

    if let Some(ref body) = ep.body {
        match body.body_type.as_str() {
            "json" => {
                let json_str = body
                    .examples
                    .first()
                    .and_then(|ex| serde_json::to_string_pretty(&ex.value).ok())
                    .unwrap_or_else(|| "{}".to_string());
                let indented = json_str.lines().collect::<Vec<_>>().join("\n   ");
                body_flags.push(format!("  -d '{}'", indented));
            }
            "formdata" => {
                if let Some(ref fields) = body.fields {
                    for f in fields {
                        let val = match f.field_type.as_str() {
                            "file" => format!("@/path/to/{}", f.name),
                            _ => format!("\"example-{}\"", f.name),
                        };
                        body_flags.push(format!("  -F '{}={}'", f.name, val));
                    }
                }
            }
            "urlencoded" => {
                if let Some(ref schema_fields) = body.schema {
                    let pairs: Vec<String> = schema_fields
                        .iter()
                        .map(|(name, field)| {
                            let val = match field.field_type.as_str() {
                                "integer" => "1".to_string(),
                                "boolean" => "true".to_string(),
                                "float" => "1.0".to_string(),
                                _ => format!("example-{}", name),
                            };
                            format!("{}={}", name, val)
                        })
                        .collect();
                    body_flags.push(format!("  -d '{}'", pairs.join("&")));
                }
            }
            "binary" => {
                body_flags.push("  --data-binary @/path/to/file".to_string());
            }
            "raw" => {
                let raw = body
                    .examples
                    .first()
                    .map(|ex| match &ex.value {
                        serde_json::Value::String(s) => s.clone(),
                        v => v.to_string(),
                    })
                    .unwrap_or_default();
                body_flags.push(format!("  --data '{}'", raw));
            }
            _ => {}
        }
    }

    // Build query string
    let query_string = if !ep.query_params.is_empty() {
        let pairs: Vec<String> = ep
            .query_params
            .iter()
            .map(|p| {
                let val = match p.example {
                    Some(serde_json::Value::String(ref s)) => s.clone(),
                    Some(ref v) => v.to_string(),
                    None => match p.param_type.as_str() {
                        "integer" => "1".to_string(),
                        "boolean" => "true".to_string(),
                        _ => format!("example-{}", p.name),
                    },
                };
                format!("{}={}", p.name, val)
            })
            .collect();
        format!("?{}", pairs.join("&"))
    } else {
        String::new()
    };

    let url_with_query = format!("{}{}", full_url, query_string);

    // Assemble curl command
    let mut curl = format!("curl -X {} '{}'", method_upper, url_with_query);

    header_flags.sort();
    body_flags.sort();

    for hf in &header_flags {
        curl.push_str(&format!(" \\\n{}", hf));
    }
    for bf in &body_flags {
        curl.push_str(&format!(" \\\n{}", bf));
    }

    out.push_str(&format!("```bash\n{}\n```\n\n", curl));
}
