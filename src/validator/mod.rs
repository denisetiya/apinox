use std::collections::HashSet;
use std::fmt;

use crate::schema::root::ApinoxSchema;

/// Validation severity
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Single validation message
#[derive(Debug, Clone)]
pub struct ValidationMsg {
    pub severity: Severity,
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.path, self.message)
    }
}

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub messages: Vec<ValidationMsg>,
}

impl ValidationResult {
    pub fn errors(&self) -> Vec<&ValidationMsg> {
        self.messages
            .iter()
            .filter(|m| m.severity == Severity::Error)
            .collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationMsg> {
        self.messages
            .iter()
            .filter(|m| m.severity == Severity::Warning)
            .collect()
    }

    pub fn has_errors(&self) -> bool {
        self.messages.iter().any(|m| m.severity == Severity::Error)
    }

    pub fn summary(&self) -> String {
        let errs = self.errors().len();
        let warns = self.warnings().len();
        format!("{} error(s), {} warning(s)", errs, warns)
    }
}

pub struct Validator;

impl Validator {
    pub fn validate(schema: &ApinoxSchema) -> ValidationResult {
        let mut msgs = Vec::new();

        Self::validate_root(schema, &mut msgs);
        Self::validate_duplicate_ids(schema, &mut msgs);
        Self::validate_auth_refs(schema, &mut msgs);
        Self::validate_group_refs(schema, &mut msgs);
        Self::validate_path_params(schema, &mut msgs);
        Self::validate_methods(schema, &mut msgs);
        Self::validate_response_examples(schema, &mut msgs);
        Self::check_warnings(schema, &mut msgs);

        ValidationResult { messages: msgs }
    }

    // ── Root validation ──────────────────────────────────────

    fn validate_root(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        if s.name.is_empty() {
            msgs.push(ValidationMsg {
                severity: Severity::Error,
                path: "root".into(),
                message: "Missing required field: name".into(),
            });
        }
        if s.version.is_empty() {
            msgs.push(ValidationMsg {
                severity: Severity::Error,
                path: "root".into(),
                message: "Missing required field: version".into(),
            });
        }
        if s.apinox != "1.0" {
            msgs.push(ValidationMsg {
                severity: Severity::Error,
                path: "root".into(),
                message: format!("Unsupported schema version: {} (expected 1.0)", s.apinox),
            });
        }
    }

    // ── Duplicate ID checks ──────────────────────────────────

    fn validate_duplicate_ids(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        let mut seen = HashSet::new();
        for ep in &s.endpoints {
            if !seen.insert(&ep.id) {
                msgs.push(ValidationMsg {
                    severity: Severity::Error,
                    path: format!("endpoint:{}", ep.id),
                    message: format!("Duplicate endpoint ID: {}", ep.id),
                });
            }
        }

        let mut seen_groups = HashSet::new();
        for grp in &s.groups {
            if !seen_groups.insert(&grp.id) {
                msgs.push(ValidationMsg {
                    severity: Severity::Error,
                    path: format!("group:{}", grp.id),
                    message: format!("Duplicate group ID: {}", grp.id),
                });
            }
        }

        // Duplicate response example names per endpoint
        for ep in &s.endpoints {
            let mut seen_ex = HashSet::new();
            for resp in &ep.responses {
                for ex in &resp.examples {
                    let key = format!("{}:{}", resp.status, ex.name);
                    if !seen_ex.insert(key.clone()) {
                        msgs.push(ValidationMsg {
                            severity: Severity::Error,
                            path: format!("endpoint:{}/response/{}", ep.id, resp.status),
                            message: format!("Duplicate example name: {}", ex.name),
                        });
                    }
                }
            }
        }
    }

    // ── Auth reference checks ────────────────────────────────

    fn validate_auth_refs(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        let scheme_ids: HashSet<&str> = s.auth.schemes.iter().map(|a| a.id.as_str()).collect();

        // Check default auth exists
        if let Some(ref default_auth) = s.auth.default {
            if !scheme_ids.contains(default_auth.as_str()) {
                msgs.push(ValidationMsg {
                    severity: Severity::Error,
                    path: "auth".into(),
                    message: format!(
                        "Default auth scheme '{}' not found in schemes",
                        default_auth
                    ),
                });
            }
        }

        // Check endpoint auth refs
        for ep in &s.endpoints {
            if let Some(ref auth_ref) = ep.auth {
                if !scheme_ids.contains(auth_ref.as_str()) {
                    msgs.push(ValidationMsg {
                        severity: Severity::Error,
                        path: format!("endpoint:{}", ep.id),
                        message: format!("Auth scheme '{}' not found in schemes", auth_ref),
                    });
                }
            }
        }

        // Check group auth refs
        for grp in &s.groups {
            if let Some(ref auth_ref) = grp.auth {
                if !scheme_ids.contains(auth_ref.as_str()) {
                    msgs.push(ValidationMsg {
                        severity: Severity::Error,
                        path: format!("group:{}", grp.id),
                        message: format!("Auth scheme '{}' not found in schemes", auth_ref),
                    });
                }
            }
        }
    }

    // ── Group reference checks ───────────────────────────────

    fn validate_group_refs(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        let group_ids: HashSet<&str> = s.groups.iter().map(|g| g.id.as_str()).collect();

        for ep in &s.endpoints {
            if let Some(ref grp) = ep.group {
                if !group_ids.contains(grp.as_str()) {
                    msgs.push(ValidationMsg {
                        severity: Severity::Error,
                        path: format!("endpoint:{}", ep.id),
                        message: format!("Group '{}' not found", grp),
                    });
                }
            }
        }
    }

    // ── Path param ↔ path matching ───────────────────────────

    fn validate_path_params(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        for ep in &s.endpoints {
            // Extract {params} from path
            let mut path_params = HashSet::new();
            let mut chars = ep.path.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '{' {
                    let mut param = String::new();
                    while let Some(&next) = chars.peek() {
                        if next == '}' {
                            chars.next();
                            break;
                        }
                        param.push(next);
                        chars.next();
                    }
                    if !param.is_empty() {
                        path_params.insert(param);
                    }
                }
            }

            // Check declared path_params match
            let declared: HashSet<&str> = ep.path_params.iter().map(|p| p.name.as_str()).collect();

            for pp in &path_params {
                if !declared.contains(pp.as_str()) {
                    msgs.push(ValidationMsg {
                        severity: Severity::Error,
                        path: format!("endpoint:{}", ep.id),
                        message: format!(
                            "Path param '{}' in '{}' not declared in path_params",
                            pp, ep.path
                        ),
                    });
                }
            }

            for dp in &declared {
                if !path_params.contains(*dp) {
                    msgs.push(ValidationMsg {
                        severity: Severity::Error,
                        path: format!("endpoint:{}", ep.id),
                        message: format!(
                            "Declared path param '{}' not found in path '{}'",
                            dp, ep.path
                        ),
                    });
                }
            }
        }
    }

    // ── HTTP method validation ───────────────────────────────

    fn validate_methods(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        let valid = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
        for ep in &s.endpoints {
            let method_upper = ep.method.to_uppercase();
            if !valid.contains(&method_upper.as_str()) {
                msgs.push(ValidationMsg {
                    severity: Severity::Error,
                    path: format!("endpoint:{}", ep.id),
                    message: format!("Invalid HTTP method: {}", ep.method),
                });
            }
        }
    }

    // ── Response example checks ──────────────────────────────

    fn validate_response_examples(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        for ep in &s.endpoints {
            if ep.responses.is_empty() {
                msgs.push(ValidationMsg {
                    severity: Severity::Warning,
                    path: format!("endpoint:{}", ep.id),
                    message: "No responses defined".into(),
                });
            }
            for resp in &ep.responses {
                if resp.examples.is_empty() {
                    msgs.push(ValidationMsg {
                        severity: Severity::Warning,
                        path: format!("endpoint:{}/response/{}", ep.id, resp.status),
                        message: "No examples for this response".into(),
                    });
                }
            }
        }
    }

    // ── Soft warnings ────────────────────────────────────────

    fn check_warnings(s: &ApinoxSchema, msgs: &mut Vec<ValidationMsg>) {
        // Unused auth schemes
        let mut used_auth = HashSet::new();
        if let Some(ref d) = s.auth.default {
            used_auth.insert(d.clone());
        }
        for ep in &s.endpoints {
            if let Some(ref a) = ep.auth {
                used_auth.insert(a.clone());
            }
        }
        for grp in &s.groups {
            if let Some(ref a) = grp.auth {
                used_auth.insert(a.clone());
            }
        }
        for scheme in &s.auth.schemes {
            if !used_auth.contains(&scheme.id) {
                msgs.push(ValidationMsg {
                    severity: Severity::Warning,
                    path: format!("auth:{}", scheme.id),
                    message: "Auth scheme not used by any endpoint or group".into(),
                });
            }
        }

        // Endpoint without description
        for ep in &s.endpoints {
            if ep.description.is_none() {
                msgs.push(ValidationMsg {
                    severity: Severity::Info,
                    path: format!("endpoint:{}", ep.id),
                    message: "Missing description".into(),
                });
            }
        }

        // Unused environments
        // (basic check — just report them as info)
        for env in &s.environments {
            msgs.push(ValidationMsg {
                severity: Severity::Info,
                path: format!("environment:{}", env.name),
                message: format!("Environment '{}' defined", env.name),
            });
        }

        // Group without endpoints
        let ep_groups: HashSet<&str> = s
            .endpoints
            .iter()
            .filter_map(|e| e.group.as_deref())
            .collect();
        for grp in &s.groups {
            if !ep_groups.contains(grp.id.as_str()) {
                msgs.push(ValidationMsg {
                    severity: Severity::Info,
                    path: format!("group:{}", grp.id),
                    message: "Group has no endpoints".into(),
                });
            }
        }
    }
}
