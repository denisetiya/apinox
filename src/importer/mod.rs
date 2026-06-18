use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::schema::auth::{AuthConfig, AuthScheme, AuthType};
use crate::schema::endpoint::{
    Body, BodyExample, BodyField, Endpoint, FieldSchema, Header, Parameter, Response,
    ResponseExample,
};
use crate::schema::environment::Environment;
use crate::schema::root::{ApinoxSchema, GroupDef};

// ---------------------------------------------------------------------------
// Local OpenAPI / Swagger 2.0 serde structs
// ---------------------------------------------------------------------------

/// Top-level document — captures swagger 2.0 and openapi 3.x fields.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiDoc {
    #[serde(rename = "swagger")]
    swagger: Option<String>,
    #[serde(rename = "openapi")]
    openapi: Option<String>,

    info: Option<OpenApiInfo>,

    // Swagger 2.0
    host: Option<String>,
    base_path: Option<String>,
    schemes: Option<Vec<String>>,
    consumes: Option<Vec<String>>,
    produces: Option<Vec<String>>,
    #[serde(rename = "definitions")]
    definitions: Option<HashMap<String, OpenApiSchema>>,
    #[serde(rename = "securityDefinitions")]
    security_definitions: Option<HashMap<String, OpenApiSecurityDef>>,

    // OpenAPI 3.x
    servers: Option<Vec<OpenApiServer>>,
    components: Option<OpenApiComponents>,

    // Shared
    #[serde(rename = "paths")]
    paths: Option<HashMap<String, OpenApiPathItem>>,
    tags: Option<Vec<OpenApiTag>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiInfo {
    title: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiServer {
    url: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiTag {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiComponents {
    schemas: Option<HashMap<String, OpenApiSchema>>,
    #[serde(rename = "securitySchemes")]
    security_schemes: Option<HashMap<String, OpenApiSecurityDef>>,
}

// -- Paths ------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiPathItem {
    get: Option<OpenApiOperation>,
    post: Option<OpenApiOperation>,
    put: Option<OpenApiOperation>,
    patch: Option<OpenApiOperation>,
    delete: Option<OpenApiOperation>,
    head: Option<OpenApiOperation>,
    options: Option<OpenApiOperation>,
    #[serde(rename = "parameters")]
    parameters: Option<Vec<OpenApiParameter>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiOperation {
    summary: Option<String>,
    description: Option<String>,
    operation_id: Option<String>,
    tags: Option<Vec<String>>,
    #[serde(rename = "deprecated")]
    deprecated: Option<bool>,
    #[serde(rename = "parameters")]
    parameters: Option<Vec<OpenApiParameter>>,
    #[serde(rename = "responses")]
    responses: Option<HashMap<String, OpenApiResponse>>,

    // OpenAPI 3.x
    #[serde(rename = "requestBody")]
    request_body: Option<OpenApiRequestBody>,

    // Swagger 2.0
    consumes: Option<Vec<String>>,
    produces: Option<Vec<String>>,
    security: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiParameter {
    name: Option<String>,
    #[serde(rename = "in")]
    in_loc: Option<String>,
    required: Option<bool>,
    description: Option<String>,
    #[serde(rename = "type")]
    param_type: Option<String>,
    schema: Option<OpenApiSchema>,
    deprecated: Option<bool>,
    example: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiRequestBody {
    required: Option<bool>,
    description: Option<String>,
    content: Option<HashMap<String, OpenApiMediaType>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiMediaType {
    schema: Option<OpenApiSchema>,
    example: Option<serde_json::Value>,
    examples: Option<HashMap<String, OpenApiExampleVal>>,
}

#[derive(Debug, Deserialize)]
struct OpenApiExampleVal {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    description: Option<String>,
    value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiResponse {
    description: Option<String>,
    schema: Option<OpenApiSchema>,
    content: Option<HashMap<String, OpenApiMediaType>>,
    examples: Option<HashMap<String, OpenApiExampleVal>>,
}

// -- Schemas ----------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiSchema {
    #[serde(rename = "type")]
    schema_type: Option<String>,
    format: Option<String>,
    title: Option<String>,
    description: Option<String>,
    properties: Option<HashMap<String, OpenApiSchema>>,
    required: Option<Vec<String>>,
    items: Option<Box<OpenApiSchema>>,
    example: Option<serde_json::Value>,
    #[serde(rename = "$ref")]
    ref_path: Option<String>,
    enum_vals: Option<Vec<serde_json::Value>>,

    // additional properties to map
    #[serde(rename = "minimum")]
    minimum: Option<f64>,
    #[serde(rename = "maximum")]
    maximum: Option<f64>,
    #[serde(rename = "minLength")]
    min_length: Option<usize>,
    #[serde(rename = "maxLength")]
    max_length: Option<usize>,
    #[serde(rename = "pattern")]
    pattern: Option<String>,
    default: Option<serde_json::Value>,
}

// -- Security ---------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenApiSecurityDef {
    Http(OpenApiHttpSecurity),
    ApiKey(OpenApiApiKeySecurity),
    OAuth2(OpenApiOAuth2Security),
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiHttpSecurity {
    #[serde(rename = "type")]
    security_type: Option<String>,
    scheme: Option<String>,
    bearer_format: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiApiKeySecurity {
    #[serde(rename = "type")]
    security_type: Option<String>,
    #[serde(rename = "in")]
    in_loc: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct OpenApiOAuth2Security {
    #[serde(rename = "type")]
    security_type: Option<String>,
    description: Option<String>,
    flows: Option<serde_json::Value>,
}

// ===========================================================================
// Public API
// ===========================================================================

/// Import an OpenAPI 3.0/3.1 or Swagger 2.0 spec and convert it to ApinoxSchema.
pub fn import_openapi(path: &Path) -> Result<ApinoxSchema> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    let doc: OpenApiDoc = if path.extension().map_or(false, |e| e == "json") {
        serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in: {}", path.display()))?
    } else {
        serde_yaml::from_str(&content)
            .with_context(|| format!("Invalid YAML in: {}", path.display()))?
    };

    if let Some(ref swagger) = doc.swagger {
        if swagger == "2.0" {
            return convert_swagger_2(&doc).context("Converting Swagger 2.0");
        }
    }
    if doc.openapi.is_some() {
        return convert_openapi_3(&doc).context("Converting OpenAPI 3.x");
    }

    anyhow::bail!("Unrecognised spec: expected 'swagger: 2.0' or 'openapi: 3.x' top-level key")
}

// ===========================================================================
// Swagger 2.0 conversion
// ===========================================================================

fn convert_swagger_2(doc: &OpenApiDoc) -> Result<ApinoxSchema> {
    let info = doc.info.as_ref();
    let title = info
        .and_then(|i| i.title.clone())
        .unwrap_or_else(|| "Imported API".into());
    let version = info
        .and_then(|i| i.version.clone())
        .unwrap_or_else(|| "0.0.0".into());
    let description = info.and_then(|i| i.description.clone());

    // base_url from host + basePath + schemes
    let base_url = build_swagger_2_base_url(doc);

    // Groups from tags
    let groups = doc
        .tags
        .as_ref()
        .map(|t| {
            t.iter()
                .map(|t| GroupDef {
                    id: t.name.clone().unwrap_or_default(),
                    name: t.name.clone().unwrap_or_default(),
                    description: t.description.clone(),
                    auth: None,
                })
                .collect()
        })
        .unwrap_or_default();

    // Auth from securityDefinitions
    let auth = convert_security_definitions(doc.security_definitions.as_ref());

    // Endpoints
    let mut endpoints = Vec::new();
    let default_consumes = doc.consumes.clone();
    let default_produces = doc.produces.clone();

    if let Some(ref paths) = doc.paths {
        for (path, item) in paths {
            let shared_params = item.parameters.as_ref();
            let methods = [
                ("get", item.get.as_ref()),
                ("post", item.post.as_ref()),
                ("put", item.put.as_ref()),
                ("patch", item.patch.as_ref()),
                ("delete", item.delete.as_ref()),
                ("head", item.head.as_ref()),
                ("options", item.options.as_ref()),
            ];

            for (method, op) in methods {
                if let Some(op) = op {
                    let ep = convert_swagger_2_endpoint(
                        path,
                        method,
                        op,
                        shared_params,
                        default_consumes.as_ref(),
                        default_produces.as_ref(),
                    );
                    endpoints.push(ep);
                }
            }
        }
    }

    Ok(ApinoxSchema {
        apinox: "1.0".into(),
        name: title,
        version,
        description,
        base_url,
        environments: Vec::new(),
        auth,
        groups,
        endpoints,
        includes: Vec::new(),
        changelog: Vec::new(),
        error_responses: None,
    })
}

fn build_swagger_2_base_url(doc: &OpenApiDoc) -> Option<String> {
    let host = doc.host.as_deref()?;
    let scheme = doc
        .schemes
        .as_ref()
        .and_then(|s| s.first())
        .map(|s| s.as_str())
        .unwrap_or("https");
    let base = doc.base_path.as_deref().unwrap_or("");
    Some(format!("{}://{}{}", scheme, host, base))
}

fn convert_swagger_2_endpoint(
    path: &str,
    method: &str,
    op: &OpenApiOperation,
    shared_params: Option<&Vec<OpenApiParameter>>,
    default_consumes: Option<&Vec<String>>,
    default_produces: Option<&Vec<String>>,
) -> Endpoint {
    let id = op.operation_id.clone().unwrap_or_else(|| {
        format!(
            "{}-{}",
            method,
            path.trim_start_matches('/').replace('/', "-")
        )
    });
    let name = op
        .summary
        .clone()
        .or_else(|| op.operation_id.clone())
        .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path));

    // Merge parameters (operation-level override shared-level)
    let mut all_params: Vec<&OpenApiParameter> = Vec::new();
    if let Some(sp) = shared_params {
        all_params.extend(sp.iter());
    }
    if let Some(ref op_params) = op.parameters {
        all_params.retain(|sp| {
            !op_params
                .iter()
                .any(|op| op.name == sp.name && op.in_loc == sp.in_loc)
        });
        all_params.extend(op_params.iter());
    }

    let mut path_params = Vec::new();
    let mut query_params = Vec::new();

    for p in &all_params {
        let param = Parameter {
            name: p.name.clone().unwrap_or_default(),
            param_type: map_openapi_type(p.param_type.as_deref().unwrap_or("string")),
            required: p.required.unwrap_or(false),
            description: p.description.clone(),
            deprecated: p.deprecated.unwrap_or(false),
            example: p.example.clone(),
            min_val: p.schema.as_ref().and_then(|s| s.minimum),
            max_val: p.schema.as_ref().and_then(|s| s.maximum),
            min_length: p.schema.as_ref().and_then(|s| s.min_length),
            max_length: p.schema.as_ref().and_then(|s| s.max_length),
            pattern: p.schema.as_ref().and_then(|s| s.pattern.clone()),
            enum_vals: p.schema.as_ref().and_then(|s| s.enum_vals.clone()),
            format: p.schema.as_ref().and_then(|s| s.format.clone()),
            default: p.schema.as_ref().and_then(|s| s.default.clone()),
            items: None,
            max_items: None,
            min_items: None,
            fields: None,
        };
        match p.in_loc.as_deref() {
            Some("path") => path_params.push(param),
            Some("query") => query_params.push(param),
            _ => {
                // header/body etc.
            }
        }
    }

    // Body from consumes + inline parameters
    let body = build_swagger_2_body(op, default_consumes);

    // Responses
    let responses = op
        .responses
        .as_ref()
        .map(|r| {
            r.iter()
                .filter_map(|(code, resp)| {
                    let status = code.parse::<u16>().ok()?;
                    let description = resp.description.clone().unwrap_or_default();
                    let content_type = default_produces.as_ref().and_then(|p| p.first().cloned());
                    Some(Response {
                        status,
                        description,
                        headers: Vec::new(),
                        content_type,
                        schema: resp.schema.as_ref().map(|s| schema_to_field_map(s)),
                        examples: Vec::new(),
                        use_pattern: None,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let group = op.tags.as_ref().and_then(|t| t.first().cloned());

    Endpoint {
        id,
        name,
        group,
        method: method.to_uppercase(),
        path: path.into(),
        description: op.description.clone(),
        deprecated: op.deprecated.unwrap_or(false),
        tags: op.tags.clone(),
        auth: None,
        auth_override: None,
        servers: None,
        rate_limit: None,
        path_params,
        query_params,
        headers: Vec::new(),
        body,
        responses,
    }
}

fn build_swagger_2_body(
    op: &OpenApiOperation,
    default_consumes: Option<&Vec<String>>,
) -> Option<Body> {
    // Check for body parameter
    let all_params: Vec<&OpenApiParameter> = op
        .parameters
        .as_ref()
        .map(|p| p.iter().collect())
        .unwrap_or_default();
    let body_param = all_params
        .iter()
        .find(|p| p.in_loc.as_deref() == Some("body"));

    let consumes = op.consumes.as_ref().or(default_consumes);
    let content_type = consumes.and_then(|c| c.first().cloned());

    let body_type = match content_type.as_deref() {
        Some("application/json") => "json",
        Some("multipart/form-data") => "formdata",
        Some("application/x-www-form-urlencoded") => "urlencoded",
        _ => "json",
    };

    if let Some(bp) = body_param {
        let schema = bp.schema.as_ref().map(|s| schema_to_field_map(s));
        return Some(Body {
            body_type: body_type.to_string(),
            required: bp.required.unwrap_or(false),
            content_type: content_type.clone(),
            description: bp.description.clone(),
            schema,
            fields: None,
            examples: Vec::new(),
            encoding: None,
            mime_type: content_type.clone(),
        });
    }

    // form-data / urlencoded without explicit body param → infer from consumes
    if body_type == "formdata" || body_type == "urlencoded" {
        return Some(Body {
            body_type: body_type.to_string(),
            required: false,
            content_type: content_type.clone(),
            description: None,
            schema: None,
            fields: Some(Vec::new()),
            examples: Vec::new(),
            encoding: None,
            mime_type: content_type.clone(),
        });
    }

    None
}

// ===========================================================================
// OpenAPI 3.x conversion
// ===========================================================================

fn convert_openapi_3(doc: &OpenApiDoc) -> Result<ApinoxSchema> {
    let info = doc.info.as_ref();
    let title = info
        .and_then(|i| i.title.clone())
        .unwrap_or_else(|| "Imported API".into());
    let version = info
        .and_then(|i| i.version.clone())
        .unwrap_or_else(|| "0.0.0".into());
    let description = info.and_then(|i| i.description.clone());

    // base_url from first server
    let (base_url, environments) = build_oa3_servers(doc.servers.as_ref());

    // Groups from tags
    let groups = doc
        .tags
        .as_ref()
        .map(|t| {
            t.iter()
                .map(|tg| GroupDef {
                    id: tg.name.clone().unwrap_or_default(),
                    name: tg.name.clone().unwrap_or_default(),
                    description: tg.description.clone(),
                    auth: None,
                })
                .collect()
        })
        .unwrap_or_default();

    // Auth
    let security_schemes = doc
        .components
        .as_ref()
        .and_then(|c| c.security_schemes.as_ref());
    let auth = convert_security_definitions(security_schemes);

    // Endpoints
    let mut endpoints = Vec::new();
    if let Some(ref paths) = doc.paths {
        for (path, item) in paths {
            let shared_params = item.parameters.as_ref();
            let methods = [
                ("get", item.get.as_ref()),
                ("post", item.post.as_ref()),
                ("put", item.put.as_ref()),
                ("patch", item.patch.as_ref()),
                ("delete", item.delete.as_ref()),
                ("head", item.head.as_ref()),
                ("options", item.options.as_ref()),
            ];

            for (method, op) in methods {
                if let Some(op) = op {
                    let ep = convert_oa3_endpoint(path, method, op, shared_params);
                    endpoints.push(ep);
                }
            }
        }
    }

    Ok(ApinoxSchema {
        apinox: "1.0".into(),
        name: title,
        version,
        description,
        base_url,
        environments,
        auth,
        groups,
        endpoints,
        includes: Vec::new(),
        changelog: Vec::new(),
        error_responses: None,
    })
}

fn build_oa3_servers(servers: Option<&Vec<OpenApiServer>>) -> (Option<String>, Vec<Environment>) {
    let mut environments = Vec::new();
    let base_url;

    if let Some(servers) = servers {
        if let Some(first) = servers.first() {
            base_url = first.url.clone();
            for s in servers {
                environments.push(Environment {
                    name: s.description.clone().unwrap_or_else(|| "default".into()),
                    base_url: s.url.clone(),
                    description: s.description.clone(),
                    vars: HashMap::new(),
                });
            }
        } else {
            base_url = None;
        }
    } else {
        base_url = None;
    }

    (base_url, environments)
}

fn convert_oa3_endpoint(
    path: &str,
    method: &str,
    op: &OpenApiOperation,
    shared_params: Option<&Vec<OpenApiParameter>>,
) -> Endpoint {
    let id = op.operation_id.clone().unwrap_or_else(|| {
        format!(
            "{}-{}",
            method,
            path.trim_start_matches('/').replace('/', "-")
        )
    });
    let name = op
        .summary
        .clone()
        .or_else(|| op.operation_id.clone())
        .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path));

    // Merge parameters
    let mut all_params: Vec<&OpenApiParameter> = Vec::new();
    if let Some(sp) = shared_params {
        all_params.extend(sp.iter());
    }
    if let Some(ref op_params) = op.parameters {
        all_params.retain(|sp| {
            !op_params
                .iter()
                .any(|op| op.name == sp.name && op.in_loc == sp.in_loc)
        });
        all_params.extend(op_params.iter());
    }

    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut header_params = Vec::new();

    for p in &all_params {
        let param_type = p.param_type.clone().unwrap_or_else(|| {
            p.schema
                .as_ref()
                .and_then(|s| s.schema_type.clone())
                .unwrap_or_else(|| "string".into())
        });
        let param = Parameter {
            name: p.name.clone().unwrap_or_default(),
            param_type: map_openapi_type(&param_type),
            required: p.required.unwrap_or(false),
            description: p.description.clone(),
            deprecated: p.deprecated.unwrap_or(false),
            example: p
                .example
                .clone()
                .or_else(|| p.schema.as_ref().and_then(|s| s.example.clone())),
            min_val: p.schema.as_ref().and_then(|s| s.minimum),
            max_val: p.schema.as_ref().and_then(|s| s.maximum),
            min_length: p.schema.as_ref().and_then(|s| s.min_length),
            max_length: p.schema.as_ref().and_then(|s| s.max_length),
            pattern: p.schema.as_ref().and_then(|s| s.pattern.clone()),
            enum_vals: p.schema.as_ref().and_then(|s| s.enum_vals.clone()),
            format: p.schema.as_ref().and_then(|s| s.format.clone()),
            default: p.schema.as_ref().and_then(|s| s.default.clone()),
            items: None,
            max_items: None,
            min_items: None,
            fields: None,
        };
        match p.in_loc.as_deref() {
            Some("path") => path_params.push(param),
            Some("query") => query_params.push(param),
            Some("header") => {
                header_params.push(Header {
                    name: p.name.clone().unwrap_or_default(),
                    value: None,
                    required: p.required.unwrap_or(false),
                    description: p.description.clone(),
                    example: p.example.as_ref().and_then(|e| e.as_str().map(|s| s.to_string())),
                });
            }
            _ => {}
        }
    }

    // Body from requestBody
    let body = op.request_body.as_ref().and_then(|rb| {
        let content = rb.content.as_ref()?;
        let (ct, media) = content.iter().next()?;
        let body_type = match ct.as_str() {
            "application/json" => "json",
            "multipart/form-data" => "formdata",
            "application/x-www-form-urlencoded" => "urlencoded",
            "application/octet-stream" => "binary",
            _ => "json",
        };

        let schema = media.schema.as_ref().map(|s| schema_to_field_map(s));

        // Build fields for formdata from schema properties
        let fields = if body_type == "formdata" {
            media.schema.as_ref().and_then(|s| {
                s.properties.as_ref().map(|props| {
                    let required_set: Vec<&str> = s
                        .required
                        .as_ref()
                        .map(|r| r.iter().map(|s| s.as_str()).collect())
                        .unwrap_or_default();
                    props
                        .iter()
                        .map(|(name, prop)| {
                            let ft =
                                prop.schema_type.clone().unwrap_or_else(|| "string".into());
                            let ft =
                                if ft == "string" && prop.format.as_deref() == Some("binary") {
                                    "file"
                                } else {
                                    &ft
                                };
                            BodyField {
                                name: name.clone(),
                                field_type: map_openapi_type(ft),
                                required: required_set.contains(&name.as_str()),
                                sensitive: false,
                                description: prop.description.clone(),
                                accept: None,
                                max_size_mb: None,
                                items: None,
                                max: None,
                                enum_vals: prop.enum_vals.as_ref().map(|ev| {
                                    ev.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                }),
                                default: prop.default.clone(),
                            }
                        })
                        .collect()
                })
            })
        } else {
            None
        };

        // Collect body examples
        let examples = collect_body_media_examples(media);

        Some(Body {
            body_type: body_type.to_string(),
            required: rb.required.unwrap_or(false),
            content_type: Some(ct.clone()),
            description: rb.description.clone(),
            schema,
            fields,
            examples,
            encoding: None,
            mime_type: Some(ct.clone()),
        })
    });

    // Responses
    let responses = op
        .responses
        .as_ref()
        .map(|r| {
            r.iter()
                .filter_map(|(code, resp)| {
                    let status = code.parse::<u16>().ok()?;
                    let description = resp.description.clone().unwrap_or_default();

                    // OA3 responses use content map
                    let (content_type, schema, examples) = if let Some(ref content) = resp.content {
                        if let Some((ct, media)) = content.iter().next() {
                            let s = media.schema.as_ref().map(|s| schema_to_field_map(s));
                            let ex = collect_response_media_examples(media);
                            (Some(ct.clone()), s, ex)
                        } else {
                            (None, None, Vec::new())
                        }
                    } else {
                        (None, None, Vec::new())
                    };

                    Some(Response {
                        status,
                        description,
                        headers: Vec::new(),
                        content_type,
                        schema,
                        examples,
                        use_pattern: None,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let group = op.tags.as_ref().and_then(|t| t.first().cloned());

    Endpoint {
        id,
        name,
        group,
        method: method.to_uppercase(),
        path: path.into(),
        description: op.description.clone(),
        deprecated: op.deprecated.unwrap_or(false),
        tags: op.tags.clone(),
        auth: None,
        auth_override: None,
        servers: None,
        rate_limit: None,
        path_params,
        query_params,
        headers: header_params,
        body,
        responses,
    }
}

// ===========================================================================
// Auth conversion
// ===========================================================================

fn convert_security_definitions(defs: Option<&HashMap<String, OpenApiSecurityDef>>) -> AuthConfig {
    let defs = match defs {
        Some(d) => d,
        None => return AuthConfig::default(),
    };

    let mut schemes = Vec::new();
    let mut default_auth: Option<String> = None;

    for (name, def) in defs {
        match def {
            OpenApiSecurityDef::Http(http) => {
                let scheme = http.scheme.clone().unwrap_or_default();
                let (auth_type, prefix) = match scheme.as_str() {
                    "bearer" => (AuthType::Http, Some("Bearer ".into())),
                    "basic" => (AuthType::Basic, None),
                    _ => (AuthType::Http, None),
                };
                if default_auth.is_none() {
                    default_auth = Some(name.clone());
                }
                schemes.push(AuthScheme {
                    id: name.clone(),
                    auth_type,
                    scheme: Some(scheme),
                    header: Some("Authorization".into()),
                    prefix,
                    key: None,
                    in_location: None,
                    description: http.description.clone(),
                });
            }
            OpenApiSecurityDef::ApiKey(api_key) => {
                if default_auth.is_none() {
                    default_auth = Some(name.clone());
                }
                schemes.push(AuthScheme {
                    id: name.clone(),
                    auth_type: AuthType::ApiKey,
                    scheme: None,
                    header: None,
                    prefix: None,
                    key: api_key.name.clone(),
                    in_location: api_key.in_loc.clone(),
                    description: api_key.description.clone(),
                });
            }
            OpenApiSecurityDef::OAuth2(oauth) => {
                if default_auth.is_none() {
                    default_auth = Some(name.clone());
                }
                schemes.push(AuthScheme {
                    id: name.clone(),
                    auth_type: AuthType::Oauth2,
                    scheme: None,
                    header: None,
                    prefix: None,
                    key: None,
                    in_location: None,
                    description: oauth.description.clone(),
                });
            }
        }
    }

    AuthConfig {
        default: default_auth,
        schemes,
    }
}

// ===========================================================================
// Schema helpers
// ===========================================================================

fn schema_to_field_map(schema: &OpenApiSchema) -> HashMap<String, FieldSchema> {
    let mut map = HashMap::new();
    let required_set: Vec<&str> = schema
        .required
        .as_ref()
        .map(|r| r.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    if let Some(ref props) = schema.properties {
        for (name, prop) in props {
            let field_type = prop.schema_type.clone().unwrap_or_else(|| {
                if prop.ref_path.is_some() {
                    "object".into()
                } else {
                    "string".into()
                }
            });
            map.insert(
                name.clone(),
                FieldSchema {
                    field_type: map_openapi_type(&field_type),
                    required: required_set.contains(&name.as_str()),
                    sensitive: false,
                    description: prop.description.clone(),
                    default: prop.default.clone(),
                    enum_vals: prop.enum_vals.clone(),
                    min_val: prop.minimum,
                    max_val: prop.maximum,
                    min_length: prop.min_length,
                    max_length: prop.max_length,
                    pattern: prop.pattern.clone(),
                    format: prop.format.clone(),
                    sub_fields: None,
                    sub_items: None,
                },
            );
        }
    }

    map
}

/// Collect media examples as ResponseExample (for Response type).
fn collect_response_media_examples(media: &OpenApiMediaType) -> Vec<ResponseExample> {
    let mut examples = Vec::new();

    if let Some(ref ex) = media.example {
        examples.push(ResponseExample {
            name: "default".into(),
            description: None,
            value: ex.clone(),
        });
    }

    if let Some(ref ex_map) = media.examples {
        for (key, ex_val) in ex_map {
            examples.push(ResponseExample {
                name: key.clone(),
                description: ex_val
                    .summary
                    .clone()
                    .or_else(|| ex_val.description.clone()),
                value: ex_val.value.clone().unwrap_or(serde_json::Value::Null),
            });
        }
    }

    examples
}

/// Collect media examples as BodyExample (for Body type).
fn collect_body_media_examples(media: &OpenApiMediaType) -> Vec<BodyExample> {
    let mut examples = Vec::new();

    if let Some(ref ex) = media.example {
        examples.push(BodyExample {
            name: "default".into(),
            description: None,
            value: ex.clone(),
        });
    }

    if let Some(ref ex_map) = media.examples {
        for (key, ex_val) in ex_map {
            examples.push(BodyExample {
                name: key.clone(),
                description: ex_val
                    .summary
                    .clone()
                    .or_else(|| ex_val.description.clone()),
                value: ex_val.value.clone().unwrap_or(serde_json::Value::Null),
            });
        }
    }

    examples
}

/// Map OpenAPI schema type to Apinox type string.
fn map_openapi_type(ot: &str) -> String {
    match ot {
        "integer" => "integer".into(),
        "number" => "float".into(),
        "boolean" => "boolean".into(),
        "array" => "array".into(),
        "object" => "object".into(),
        _ => "string".into(),
    }
}
