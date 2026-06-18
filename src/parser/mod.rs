use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::schema::endpoint::Endpoint;
use crate::schema::root::{ApinoxSchema, GroupDef};

/// Partial schema for include files (only endpoints + groups)
#[derive(Debug, Clone, Deserialize, Default)]
struct IncludeFile {
    #[serde(default)]
    endpoints: Vec<Endpoint>,
    #[serde(default)]
    groups: Vec<GroupDef>,
}

pub struct Parser;

impl Parser {
    /// Parse schema from file (auto-detect YAML/JSON)
    pub fn parse_file(path: &Path) -> Result<ApinoxSchema> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;

        let mut schema = if path.extension().map_or(false, |e| e == "json") {
            serde_json::from_str::<ApinoxSchema>(&content)
                .with_context(|| format!("Invalid JSON in: {}", path.display()))?
        } else {
            serde_yaml::from_str::<ApinoxSchema>(&content)
                .with_context(|| format!("Invalid YAML in: {}", path.display()))?
        };

        // Resolve includes
        let base_dir = path.parent().unwrap_or(Path::new("."));
        Self::resolve_includes(&mut schema, base_dir)?;

        Ok(schema)
    }

    /// Parse from raw string (for tests / stdin)
    pub fn parse_str(content: &str, format: &str) -> Result<ApinoxSchema> {
        match format {
            "json" => serde_json::from_str(content).context("Invalid JSON"),
            "yaml" | "yml" => serde_yaml::from_str(content).context("Invalid YAML"),
            _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
        }
    }

    /// Parse include file (partial schema — only endpoints + groups)
    fn parse_include_file(path: &Path) -> Result<IncludeFile> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read include: {}", path.display()))?;

        if path.extension().map_or(false, |e| e == "json") {
            serde_json::from_str(&content)
                .with_context(|| format!("Invalid JSON in include: {}", path.display()))
        } else {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Invalid YAML in include: {}", path.display()))
        }
    }

    /// Resolve all include directives, pulling endpoints from referenced files
    fn resolve_includes(schema: &mut ApinoxSchema, base_dir: &Path) -> Result<()> {
        let includes = schema.includes.clone();
        for include in &includes {
            let include_path = base_dir.join(&include.path);
            if !include_path.exists() {
                anyhow::bail!("Include file not found: {}", include_path.display());
            }

            let child = Self::parse_include_file(&include_path)?;

            // Merge endpoints from child, with optional prefix
            let prefix = include.prefix.as_deref().unwrap_or("");
            for mut ep in child.endpoints {
                if !prefix.is_empty() {
                    ep.id = format!("{}-{}", prefix, ep.id);
                }
                schema.endpoints.push(ep);
            }

            // Merge groups
            for grp in child.groups {
                schema.groups.push(grp);
            }
        }

        Ok(())
    }
}
