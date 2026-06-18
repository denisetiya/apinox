use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use crate::schema::auth::AuthScheme;
use crate::schema::endpoint::Endpoint;
use crate::schema::root::{ApinoxSchema, GroupDef};

// ---------------------------------------------------------------------------
// Diff data structures
// ---------------------------------------------------------------------------

/// A single field-level change between two endpoint versions.
#[derive(Debug, Clone)]
pub struct ChangeEntry {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// Field-level diff for a single endpoint.
#[derive(Debug, Clone)]
pub struct EndpointDiff {
    pub endpoint_id: String,
    pub changes: Vec<ChangeEntry>,
}

/// Top-level result of diffing two schemas.
#[derive(Debug)]
pub struct DiffResult {
    pub version_from: String,
    pub version_to: String,
    pub added_endpoints: Vec<Endpoint>,
    pub removed_endpoints: Vec<Endpoint>,
    pub modified_endpoints: Vec<EndpointDiff>,
    pub added_groups: Vec<GroupDef>,
    pub removed_groups: Vec<GroupDef>,
    pub added_auth_schemes: Vec<AuthScheme>,
    pub removed_auth_schemes: Vec<AuthScheme>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn endpoint_map(s: &ApinoxSchema) -> HashMap<String, &Endpoint> {
    s.endpoints.iter().map(|ep| (ep.id.clone(), ep)).collect()
}

fn group_map(s: &ApinoxSchema) -> HashMap<String, &GroupDef> {
    s.groups.iter().map(|g| (g.id.clone(), g)).collect()
}

fn auth_map(s: &ApinoxSchema) -> HashMap<String, &AuthScheme> {
    s.auth.schemes.iter().map(|a| (a.id.clone(), a)).collect()
}

#[allow(dead_code)]
fn json_to_string(v: &serde_json::Value) -> String {
    v.to_string()
}

fn opt_str(o: &Option<String>) -> String {
    match o {
        Some(v) => v.clone(),
        None => String::from("(none)"),
    }
}

fn params_to_string(params: &[crate::schema::endpoint::Parameter]) -> String {
    if params.is_empty() {
        return "(empty)".to_string();
    }
    let parts: Vec<String> = params
        .iter()
        .map(|p| {
            format!(
                "{} ({}, {})",
                p.name,
                p.param_type,
                if p.required { "required" } else { "optional" }
            )
        })
        .collect();
    parts.join(", ")
}

fn body_to_string(body: &Option<crate::schema::endpoint::Body>) -> String {
    match body {
        Some(b) => format!("type={}, required={}", b.body_type, b.required),
        None => "(none)".to_string(),
    }
}

fn responses_to_string(resps: &[crate::schema::endpoint::Response]) -> String {
    if resps.is_empty() {
        return "(empty)".to_string();
    }
    let parts: Vec<String> = resps
        .iter()
        .map(|r| format!("{}: {}", r.status, r.description))
        .collect();
    parts.join(", ")
}

fn compare_option_str(
    changes: &mut Vec<ChangeEntry>,
    field: &str,
    old: &Option<String>,
    new: &Option<String>,
) {
    let ov = opt_str(old);
    let nv = opt_str(new);
    if ov != nv {
        changes.push(ChangeEntry {
            field: field.to_string(),
            old_value: ov,
            new_value: nv,
        });
    }
}

/// Compare two endpoints and return field-level changes (empty vec = identical).
fn diff_endpoint_fields(old: &Endpoint, new: &Endpoint) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();

    // method
    if old.method != new.method {
        changes.push(ChangeEntry {
            field: "method".into(),
            old_value: old.method.clone(),
            new_value: new.method.clone(),
        });
    }

    // path
    if old.path != new.path {
        changes.push(ChangeEntry {
            field: "path".into(),
            old_value: old.path.clone(),
            new_value: new.path.clone(),
        });
    }

    // name
    if old.name != new.name {
        changes.push(ChangeEntry {
            field: "name".into(),
            old_value: old.name.clone(),
            new_value: new.name.clone(),
        });
    }

    // description
    compare_option_str(
        &mut changes,
        "description",
        &old.description,
        &new.description,
    );

    // group
    compare_option_str(&mut changes, "group", &old.group, &new.group);

    // auth
    compare_option_str(&mut changes, "auth", &old.auth, &new.auth);

    // deprecated
    if old.deprecated != new.deprecated {
        changes.push(ChangeEntry {
            field: "deprecated".into(),
            old_value: old.deprecated.to_string(),
            new_value: new.deprecated.to_string(),
        });
    }

    // tags
    let old_tags = old.tags.as_ref().map(|t| t.join(", "));
    let new_tags = new.tags.as_ref().map(|t| t.join(", "));
    compare_option_str(&mut changes, "tags", &old_tags, &new_tags);

    // path_params
    let ov = params_to_string(&old.path_params);
    let nv = params_to_string(&new.path_params);
    if ov != nv {
        changes.push(ChangeEntry {
            field: "path_params".into(),
            old_value: ov,
            new_value: nv,
        });
    }

    // query_params
    let ov = params_to_string(&old.query_params);
    let nv = params_to_string(&new.query_params);
    if ov != nv {
        changes.push(ChangeEntry {
            field: "query_params".into(),
            old_value: ov,
            new_value: nv,
        });
    }

    // body
    let ov = body_to_string(&old.body);
    let nv = body_to_string(&new.body);
    if ov != nv {
        changes.push(ChangeEntry {
            field: "body".into(),
            old_value: ov,
            new_value: nv,
        });
    }

    // responses
    let ov = responses_to_string(&old.responses);
    let nv = responses_to_string(&new.responses);
    if ov != nv {
        changes.push(ChangeEntry {
            field: "responses".into(),
            old_value: ov,
            new_value: nv,
        });
    }

    changes
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Diff two ApinoxSchemas and produce a structured DiffResult.
pub fn diff_schemas(old: &ApinoxSchema, new: &ApinoxSchema) -> DiffResult {
    let old_map = endpoint_map(old);
    let new_map = endpoint_map(new);

    // Added: in new but not old
    let added_endpoints: Vec<Endpoint> = new
        .endpoints
        .iter()
        .filter(|ep| !old_map.contains_key(&ep.id))
        .cloned()
        .collect();

    // Removed: in old but not new
    let removed_endpoints: Vec<Endpoint> = old
        .endpoints
        .iter()
        .filter(|ep| !new_map.contains_key(&ep.id))
        .cloned()
        .collect();

    // Modified: in both -> field-level diff
    let modified_endpoints: Vec<EndpointDiff> = new
        .endpoints
        .iter()
        .filter_map(|new_ep| {
            old_map.get(&new_ep.id).map(|old_ep| {
                let changes = diff_endpoint_fields(old_ep, new_ep);
                EndpointDiff {
                    endpoint_id: new_ep.id.clone(),
                    changes,
                }
            })
        })
        .filter(|ed| !ed.changes.is_empty())
        .collect();

    // Groups
    let old_groups = group_map(old);
    let new_groups = group_map(new);
    let added_groups: Vec<GroupDef> = new
        .groups
        .iter()
        .filter(|g| !old_groups.contains_key(&g.id))
        .cloned()
        .collect();
    let removed_groups: Vec<GroupDef> = old
        .groups
        .iter()
        .filter(|g| !new_groups.contains_key(&g.id))
        .cloned()
        .collect();

    // Auth schemes
    let old_auth = auth_map(old);
    let new_auth = auth_map(new);
    let added_auth_schemes: Vec<AuthScheme> = new
        .auth
        .schemes
        .iter()
        .filter(|a| !old_auth.contains_key(&a.id))
        .cloned()
        .collect();
    let removed_auth_schemes: Vec<AuthScheme> = old
        .auth
        .schemes
        .iter()
        .filter(|a| !new_auth.contains_key(&a.id))
        .cloned()
        .collect();

    DiffResult {
        version_from: old.version.clone(),
        version_to: new.version.clone(),
        added_endpoints,
        removed_endpoints,
        modified_endpoints,
        added_groups,
        removed_groups,
        added_auth_schemes,
        removed_auth_schemes,
    }
}

/// Generate a Markdown migration guide from a DiffResult.
pub fn generate_migration_guide(diff: &DiffResult) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!(
        "# Migration Guide: {} → {}\n\n",
        diff.version_from, diff.version_to
    ));

    // Summary
    let total_changes = diff.added_endpoints.len()
        + diff.removed_endpoints.len()
        + diff.modified_endpoints.len()
        + diff.added_groups.len()
        + diff.removed_groups.len()
        + diff.added_auth_schemes.len()
        + diff.removed_auth_schemes.len();
    md.push_str(&format!("**Total changes:** {}\n\n", total_changes));

    if total_changes == 0 {
        md.push_str("No changes detected between these versions.\n");
        return md;
    }

    // ── Breaking Changes ──
    let has_breaking = !diff.removed_endpoints.is_empty()
        || diff.modified_endpoints.iter().any(|ed| {
            ed.changes
                .iter()
                .any(|c| c.field == "method" || c.field == "path")
        });

    if has_breaking {
        md.push_str("## ⚠️ Breaking Changes\n\n");

        if !diff.removed_endpoints.is_empty() {
            md.push_str("### Removed Endpoints\n\n");
            for ep in &diff.removed_endpoints {
                md.push_str(&format!(
                    "- ~~`{} {}`~~ ({}) — {}\n",
                    ep.method,
                    ep.path,
                    ep.id,
                    ep.description.as_deref().unwrap_or("no description")
                ));
            }
            md.push('\n');
        }

        let breaking_mods: Vec<&EndpointDiff> = diff
            .modified_endpoints
            .iter()
            .filter(|ed| {
                ed.changes
                    .iter()
                    .any(|c| c.field == "method" || c.field == "path")
            })
            .collect();
        if !breaking_mods.is_empty() {
            md.push_str("### Changed Methods/Paths\n\n");
            for ed in &breaking_mods {
                md.push_str(&format!("#### `{}`\n\n", ed.endpoint_id));
                for c in &ed.changes {
                    if c.field == "method" || c.field == "path" {
                        md.push_str(&format!(
                            "- **{}:** `{}` → `{}`\n",
                            c.field, c.old_value, c.new_value
                        ));
                    }
                }
                md.push('\n');
            }
        }
    }

    // ── New Features ──
    let has_new = !diff.added_endpoints.is_empty()
        || !diff.added_auth_schemes.is_empty()
        || !diff.added_groups.is_empty();

    if has_new {
        md.push_str("## ✨ New Features\n\n");

        if !diff.added_endpoints.is_empty() {
            md.push_str("### New Endpoints\n\n");
            for ep in &diff.added_endpoints {
                md.push_str(&format!(
                    "- `{} {}` ({}) — {}\n",
                    ep.method,
                    ep.path,
                    ep.id,
                    ep.description.as_deref().unwrap_or("no description")
                ));
            }
            md.push('\n');
        }

        if !diff.added_groups.is_empty() {
            md.push_str("### New Groups\n\n");
            for g in &diff.added_groups {
                md.push_str(&format!(
                    "- **{}** ({}) — {}\n",
                    g.name,
                    g.id,
                    g.description.as_deref().unwrap_or("no description")
                ));
            }
            md.push('\n');
        }

        if !diff.added_auth_schemes.is_empty() {
            md.push_str("### New Auth Schemes\n\n");
            for a in &diff.added_auth_schemes {
                md.push_str(&format!(
                    "- **{}** ({:?}) — {}\n",
                    a.id,
                    a.auth_type,
                    a.description.as_deref().unwrap_or("no description")
                ));
            }
            md.push('\n');
        }
    }

    // ── Modified Endpoints ──
    let non_breaking_mods: Vec<&EndpointDiff> = diff
        .modified_endpoints
        .iter()
        .filter(|ed| {
            !ed.changes
                .iter()
                .any(|c| c.field == "method" || c.field == "path")
        })
        .collect();

    if !non_breaking_mods.is_empty() {
        md.push_str("## 📝 Modified Endpoints\n\n");
        for ed in &non_breaking_mods {
            md.push_str(&format!("### `{}`\n\n", ed.endpoint_id));
            md.push_str("| Field | Old | New |\n");
            md.push_str("|-------|-----|-----|\n");
            for c in &ed.changes {
                md.push_str(&format!(
                    "| {} | `{}` | `{}` |\n",
                    c.field, c.old_value, c.new_value
                ));
            }
            md.push('\n');
        }
    }

    // ── Removed Groups / Auth Schemes ──
    if !diff.removed_groups.is_empty() {
        md.push_str("## 🗑️ Removed Groups\n\n");
        for g in &diff.removed_groups {
            md.push_str(&format!("- **{}** ({})\n", g.name, g.id));
        }
        md.push('\n');
    }

    if !diff.removed_auth_schemes.is_empty() {
        md.push_str("## 🔒 Removed Auth Schemes\n\n");
        for a in &diff.removed_auth_schemes {
            md.push_str(&format!("- **{}** ({:?})\n", a.id, a.auth_type));
        }
        md.push('\n');
    }

    // ── Migration Steps ──
    md.push_str("## 🔧 Migration Steps\n\n");

    if !diff.removed_endpoints.is_empty() {
        md.push_str("1. **Remove calls** to deleted endpoints:\n");
        for ep in &diff.removed_endpoints {
            md.push_str(&format!("   - `{}` {}\n", ep.method, ep.path));
        }
        md.push('\n');
    }

    if !diff.modified_endpoints.is_empty() {
        md.push_str("2. **Update requests** for modified endpoints:\n");
        for ed in &diff.modified_endpoints {
            let summary: Vec<String> = ed
                .changes
                .iter()
                .map(|c| format!("{}: `{}` → `{}`", c.field, c.old_value, c.new_value))
                .collect();
            md.push_str(&format!(
                "   - `{}`: {}\n",
                ed.endpoint_id,
                summary.join("; ")
            ));
        }
        md.push('\n');
    }

    if !diff.added_endpoints.is_empty() {
        md.push_str("3. **Integrate** new endpoints as needed:\n");
        for ep in &diff.added_endpoints {
            md.push_str(&format!("   - `{} {}` ({})\n", ep.method, ep.path, ep.id));
        }
        md.push('\n');
    }

    if !diff.added_auth_schemes.is_empty() {
        md.push_str("4. **Configure** new auth schemes if required:\n");
        for a in &diff.added_auth_schemes {
            md.push_str(&format!("   - {} ({:?})\n", a.id, a.auth_type));
        }
        md.push('\n');
    }

    md
}

// ---------------------------------------------------------------------------
// Load-and-diff from changelog
// ---------------------------------------------------------------------------

/// Load a single schema file and generate a diff summary from its changelog
/// entries between `from_version` and `to_version`.
///
/// This is for cases where you don't have two separate schema files but the
/// schema contains changelog entries describing what changed.
pub fn load_and_diff(path: &Path, from_version: &str, to_version: &str) -> Result<String> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read schema file: {}", path.display()))?;
    let schema: ApinoxSchema = serde_yaml::from_str(&data)
        .with_context(|| format!("failed to parse schema file: {}", path.display()))?;

    // Collect changelog entries between from_version and to_version (exclusive).
    // We include entries whose version is > from_version and <= to_version.
    let mut entries_between: Vec<(String, String, crate::schema::root::ChangelogEntry)> =
        Vec::new();
    let mut collecting = false;

    for changelog in &schema.changelog {
        if changelog.version == from_version {
            collecting = true;
            continue;
        }
        if changelog.version == to_version {
            for entry in &changelog.changes {
                entries_between.push((
                    changelog.version.clone(),
                    changelog.date.clone(),
                    entry.clone(),
                ));
            }
            break;
        }
        if collecting {
            for entry in &changelog.changes {
                entries_between.push((
                    changelog.version.clone(),
                    changelog.date.clone(),
                    entry.clone(),
                ));
            }
        }
    }

    let mut md = String::new();
    md.push_str(&format!(
        "# Migration Guide: {} → {}\n\n",
        from_version, to_version
    ));
    md.push_str(&format!(
        "**Schema:** {} (version {})\n\n",
        schema.name, schema.version
    ));

    if entries_between.is_empty() {
        md.push_str("No changelog entries found between these versions.\n");
        return Ok(md);
    }

    md.push_str(&format!("**Total changes:** {}\n\n", entries_between.len()));

    // Group by change type (inferred from from_type/to_type presence)
    let mut breaking = Vec::new();
    let mut additions = Vec::new();
    let mut modifications = Vec::new();

    for item in &entries_between {
        let entry = &item.2;
        let label = if entry.from_type.is_some() && entry.to_type.is_some() {
            "modified"
        } else if entry.to_type.is_some() {
            "added"
        } else if entry.from_type.is_some() {
            "removed"
        } else {
            "modified"
        };

        match label {
            "added" => additions.push(item),
            "removed" => breaking.push(item),
            _ => modifications.push(item),
        }
    }

    if !breaking.is_empty() {
        md.push_str("## ⚠️ Breaking Changes (Removals)\n\n");
        for (ver, _date, entry) in &breaking {
            md.push_str(&format!(
                "- **{}** (`{}`): {}\n",
                entry.endpoint, ver, entry.description
            ));
        }
        md.push('\n');
    }

    if !additions.is_empty() {
        md.push_str("## ✨ New Features\n\n");
        for (ver, _date, entry) in &additions {
            md.push_str(&format!(
                "- **{}** (`{}`): {}\n",
                entry.endpoint, ver, entry.description
            ));
        }
        md.push('\n');
    }

    if !modifications.is_empty() {
        md.push_str("## 📝 Modifications\n\n");
        for (ver, _date, entry) in &modifications {
            let mut detail = format!("**{}** (`{}`): {}", entry.endpoint, ver, entry.description);
            if let (Some(from), Some(to)) = (&entry.from_type, &entry.to_type) {
                detail.push_str(&format!(" (type: `{}` → `{}`)", from, to));
            }
            md.push_str(&format!("- {}\n", detail));
        }
        md.push('\n');
    }

    // Migration steps
    md.push_str("## 🔧 Migration Steps\n\n");
    md.push_str("Review the changes above and update your client code accordingly.\n\n");

    if !breaking.is_empty() {
        md.push_str("1. **Remove** calls to deleted endpoints:\n");
        for (_, _, entry) in &breaking {
            md.push_str(&format!(
                "   - `{}`: {}\n",
                entry.endpoint, entry.description
            ));
        }
        md.push('\n');
    }

    if !modifications.is_empty() {
        md.push_str("2. **Update** modified endpoints:\n");
        for (_, _, entry) in &modifications {
            md.push_str(&format!(
                "   - `{}`: {}\n",
                entry.endpoint, entry.description
            ));
        }
        md.push('\n');
    }

    if !additions.is_empty() {
        md.push_str("3. **Integrate** new endpoints:\n");
        for (_, _, entry) in &additions {
            md.push_str(&format!(
                "   - `{}`: {}\n",
                entry.endpoint, entry.description
            ));
        }
        md.push('\n');
    }

    Ok(md)
}
