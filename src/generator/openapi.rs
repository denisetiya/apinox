use anyhow::Result;
use serde_yaml::{Mapping, Value as YamlValue};

use crate::schema::auth::AuthType;
use crate::schema::root::ApinoxSchema;
use crate::generator::postman::resolve_response;

/// OpenAPI 3.1 YAML generator
pub fn generate(schema: &ApinoxSchema) -> Result<String> {
    let spec = build_spec(schema)?;
    Ok(serde_yaml::to_string(&spec)?)
}

fn build_spec(schema: &ApinoxSchema) -> Result<YamlValue> {
    let mut root = Mapping::new();

    // OpenAPI version
    root.insert(
        YamlValue::String("openapi".into()),
        YamlValue::String("3.1.0".into()),
    );

    // Info
    let mut info = Mapping::new();
    info.insert(
        YamlValue::String("title".into()),
        YamlValue::String(schema.name.clone()),
    );
    info.insert(
        YamlValue::String("version".into()),
        YamlValue::String(schema.version.clone()),
    );
    if let Some(ref desc) = schema.description {
        info.insert(
            YamlValue::String("description".into()),
            YamlValue::String(desc.clone()),
        );
    }
    root.insert(YamlValue::String("info".into()), YamlValue::Mapping(info));

    // Servers
    let mut servers = Vec::new();
    for env in &schema.environments {
        let mut server = Mapping::new();
        server.insert(
            YamlValue::String("url".into()),
            YamlValue::String(
                env.base_url
                    .clone()
                    .or_else(|| schema.base_url.clone())
                    .unwrap_or_default(),
            ),
        );
        server.insert(
            YamlValue::String("description".into()),
            YamlValue::String(env.name.clone()),
        );
        servers.push(YamlValue::Mapping(server));
    }

    if !schema.environments.is_empty() {
        root.insert(
            YamlValue::String("servers".into()),
            YamlValue::Sequence(servers),
        );
    }

    // Security schemes
    if !schema.auth.schemes.is_empty() {
        let mut security_schemes = Mapping::new();
        for scheme in &schema.auth.schemes {
            let mut sec = Mapping::new();
            match scheme.auth_type {
                AuthType::Http => {
                    sec.insert(
                        YamlValue::String("type".into()),
                        YamlValue::String("http".into()),
                    );
                    if let Some(ref s) = scheme.scheme {
                        sec.insert(
                            YamlValue::String("scheme".into()),
                            YamlValue::String(s.clone()),
                        );
                    }
                }
                AuthType::ApiKey => {
                    sec.insert(
                        YamlValue::String("type".into()),
                        YamlValue::String("apiKey".into()),
                    );
                    if let Some(ref k) = scheme.key {
                        sec.insert(
                            YamlValue::String("name".into()),
                            YamlValue::String(k.clone()),
                        );
                    }
                    if let Some(ref loc) = scheme.in_location {
                        sec.insert(
                            YamlValue::String("in".into()),
                            YamlValue::String(loc.clone()),
                        );
                    }
                }
                AuthType::Basic => {
                    sec.insert(
                        YamlValue::String("type".into()),
                        YamlValue::String("http".into()),
                    );
                    sec.insert(
                        YamlValue::String("scheme".into()),
                        YamlValue::String("basic".into()),
                    );
                }
                _ => {
                    sec.insert(
                        YamlValue::String("type".into()),
                        YamlValue::String("none".into()),
                    );
                }
            }
            security_schemes.insert(
                YamlValue::String(scheme.id.clone()),
                YamlValue::Mapping(sec),
            );
        }

        let mut components = Mapping::new();
        components.insert(
            YamlValue::String("securitySchemes".into()),
            YamlValue::Mapping(security_schemes),
        );
        root.insert(
            YamlValue::String("components".into()),
            YamlValue::Mapping(components),
        );
    }

    // Paths
    let mut paths = Mapping::new();
    for ep in &schema.endpoints {
        let path = ep.path.clone();
        let method = ep.method.to_lowercase();

        let mut operation = Mapping::new();

        // Operation ID
        operation.insert(
            YamlValue::String("operationId".into()),
            YamlValue::String(ep.id.clone()),
        );

        // Summary
        operation.insert(
            YamlValue::String("summary".into()),
            YamlValue::String(ep.name.clone()),
        );

        // Description
        if let Some(ref desc) = ep.description {
            operation.insert(
                YamlValue::String("description".into()),
                YamlValue::String(desc.clone()),
            );
        }

        // Tags
        if let Some(ref grp) = ep.group {
            let tag_name = schema
                .groups
                .iter()
                .find(|g| g.id == *grp)
                .map(|g| g.name.clone())
                .unwrap_or_else(|| grp.clone());
            operation.insert(
                YamlValue::String("tags".into()),
                YamlValue::Sequence(vec![YamlValue::String(tag_name)]),
            );
        }

        // Deprecated
        if ep.deprecated {
            operation.insert(
                YamlValue::String("deprecated".into()),
                YamlValue::Bool(true),
            );
        }

        // Parameters
        let mut params = Vec::new();
        for pp in &ep.path_params {
            params.push(build_param(pp, "path"));
        }
        for qp in &ep.query_params {
            params.push(build_param(qp, "query"));
        }
        for h in &ep.headers {
            let mut param = Mapping::new();
            param.insert(
                YamlValue::String("name".into()),
                YamlValue::String(h.name.clone()),
            );
            param.insert(
                YamlValue::String("in".into()),
                YamlValue::String("header".into()),
            );
            param.insert(
                YamlValue::String("required".into()),
                YamlValue::Bool(h.required),
            );
            if let Some(ref desc) = h.description {
                param.insert(
                    YamlValue::String("description".into()),
                    YamlValue::String(desc.clone()),
                );
            }
            params.push(YamlValue::Mapping(param));
        }

        if !params.is_empty() {
            operation.insert(
                YamlValue::String("parameters".into()),
                YamlValue::Sequence(params),
            );
        }

        // Request body
        if let Some(ref body) = ep.body {
            let mut req_body = Mapping::new();

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

            let mut media = Mapping::new();

            // Schema for json/urlencoded
            if let Some(ref schema_fields) = body.schema {
                let mut schema_obj = Mapping::new();
                schema_obj.insert(
                    YamlValue::String("type".into()),
                    YamlValue::String("object".into()),
                );
                let mut properties = Mapping::new();
                let mut required_fields = Vec::new();

                for (name, field) in schema_fields {
                    // Skip sensitive fields in public OpenAPI docs
                    if field.sensitive {
                        continue;
                    }
                    let mut prop = Mapping::new();
                    prop.insert(
                        YamlValue::String("type".into()),
                        YamlValue::String(field.field_type.clone()),
                    );
                    if let Some(ref desc) = field.description {
                        prop.insert(
                            YamlValue::String("description".into()),
                            YamlValue::String(desc.clone()),
                        );
                    }
                    if let Some(ref enums) = field.enum_vals {
                        let vals: Vec<YamlValue> = enums
                            .iter()
                            .map(|v| YamlValue::String(v.to_string()))
                            .collect();
                        prop.insert(YamlValue::String("enum".into()), YamlValue::Sequence(vals));
                    }
                    properties.insert(YamlValue::String(name.clone()), YamlValue::Mapping(prop));

                    if field.required {
                        required_fields.push(YamlValue::String(name.clone()));
                    }
                }

                schema_obj.insert(
                    YamlValue::String("properties".into()),
                    YamlValue::Mapping(properties),
                );
                if !required_fields.is_empty() {
                    schema_obj.insert(
                        YamlValue::String("required".into()),
                        YamlValue::Sequence(required_fields),
                    );
                }

                media.insert(
                    YamlValue::String("schema".into()),
                    YamlValue::Mapping(schema_obj),
                );
            }

            // Examples
            if !body.examples.is_empty() {
                let first = &body.examples[0];
                if let Ok(json_str) = serde_json::to_string_pretty(&first.value) {
                    media.insert(
                        YamlValue::String("example".into()),
                        YamlValue::String(json_str),
                    );
                }
            }

            let mut content = Mapping::new();
            content.insert(YamlValue::String(ct.into()), YamlValue::Mapping(media));
            req_body.insert(
                YamlValue::String("content".into()),
                YamlValue::Mapping(content),
            );
            if !body.required {
                req_body.insert(YamlValue::String("required".into()), YamlValue::Bool(false));
            }

            operation.insert(
                YamlValue::String("requestBody".into()),
                YamlValue::Mapping(req_body),
            );
        }

        // Responses (using resolve_response for pattern support)
        let mut responses = Mapping::new();
        for resp in &ep.responses {
            let (status, description, _) = resolve_response(schema, resp);
            let status_str = status.to_string();
            let mut resp_obj = Mapping::new();

            if !description.is_empty() {
                resp_obj.insert(
                    YamlValue::String("description".into()),
                    YamlValue::String(description),
                );
            }

            // Response schema
            if let Some(ref schema_fields) = resp.schema {
                let mut schema_obj = Mapping::new();
                schema_obj.insert(
                    YamlValue::String("type".into()),
                    YamlValue::String("object".into()),
                );
                let mut properties = Mapping::new();
                for (name, field) in schema_fields {
                    let mut prop = Mapping::new();
                    prop.insert(
                        YamlValue::String("type".into()),
                        YamlValue::String(field.field_type.clone()),
                    );
                    if let Some(ref desc) = field.description {
                        prop.insert(
                            YamlValue::String("description".into()),
                            YamlValue::String(desc.clone()),
                        );
                    }
                    properties.insert(YamlValue::String(name.clone()), YamlValue::Mapping(prop));
                }
                schema_obj.insert(
                    YamlValue::String("properties".into()),
                    YamlValue::Mapping(properties),
                );

                let mut media = Mapping::new();
                media.insert(
                    YamlValue::String("schema".into()),
                    YamlValue::Mapping(schema_obj),
                );
                let mut content = Mapping::new();
                content.insert(
                    YamlValue::String("application/json".into()),
                    YamlValue::Mapping(media),
                );
                resp_obj.insert(
                    YamlValue::String("content".into()),
                    YamlValue::Mapping(content),
                );
            }

            // Examples
            if !resp.examples.is_empty() {
                let mut examples = Mapping::new();
                for ex in &resp.examples {
                    let mut ex_obj = Mapping::new();
                    ex_obj.insert(
                        YamlValue::String("summary".into()),
                        YamlValue::String(ex.name.clone()),
                    );
                    if let Ok(json_str) = serde_json::to_string_pretty(&ex.value) {
                        ex_obj.insert(
                            YamlValue::String("value".into()),
                            YamlValue::String(json_str),
                        );
                    }
                    examples.insert(
                        YamlValue::String(ex.name.clone()),
                        YamlValue::Mapping(ex_obj),
                    );
                }
                resp_obj.insert(
                    YamlValue::String("examples".into()),
                    YamlValue::Mapping(examples),
                );
            }

            responses.insert(YamlValue::String(status_str), YamlValue::Mapping(resp_obj));
        }

        if !responses.is_empty() {
            operation.insert(
                YamlValue::String("responses".into()),
                YamlValue::Mapping(responses),
            );
        }

        // Security
        if let Some(ref auth_ref) = ep.auth {
            operation.insert(
                YamlValue::String("security".into()),
                YamlValue::Sequence(vec![YamlValue::Mapping({
                    let mut sec = Mapping::new();
                    sec.insert(
                        YamlValue::String(auth_ref.clone()),
                        YamlValue::Sequence(vec![]),
                    );
                    sec
                })]),
            );
        }

        // Insert into paths
        let path_entry = paths
            .entry(YamlValue::String(path))
            .or_insert_with(|| YamlValue::Mapping(Mapping::new()));

        if let YamlValue::Mapping(ref mut m) = path_entry {
            m.insert(YamlValue::String(method), YamlValue::Mapping(operation));
        }
    }

    root.insert(YamlValue::String("paths".into()), YamlValue::Mapping(paths));

    Ok(YamlValue::Mapping(root))
}

fn build_param(param: &crate::schema::endpoint::Parameter, location: &str) -> YamlValue {
    let mut p = Mapping::new();
    p.insert(
        YamlValue::String("name".into()),
        YamlValue::String(param.name.clone()),
    );
    p.insert(
        YamlValue::String("in".into()),
        YamlValue::String(location.into()),
    );
    p.insert(
        YamlValue::String("required".into()),
        YamlValue::Bool(param.required),
    );
    p.insert(
        YamlValue::String("schema".into()),
        YamlValue::Mapping({
            let mut s = Mapping::new();
            s.insert(
                YamlValue::String("type".into()),
                YamlValue::String(param.param_type.clone()),
            );
            if let Some(ref desc) = param.description {
                s.insert(
                    YamlValue::String("description".into()),
                    YamlValue::String(desc.clone()),
                );
            }
            s
        }),
    );

    if let Some(ref desc) = param.description {
        p.insert(
            YamlValue::String("description".into()),
            YamlValue::String(desc.clone()),
        );
    }

    YamlValue::Mapping(p)
}
