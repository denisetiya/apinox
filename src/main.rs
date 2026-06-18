use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use notify::Watcher;

use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};

use apinox::generator::{self, OutputFormat};
use apinox::parser::Parser;
use apinox::sync::postman::{self, PostmanSyncConfig};
use apinox::validator::Validator;

#[derive(ClapParser)]
#[command(
    name = "apinox",
    about = "Schema-first API documentation generator",
    version,
    long_about = "Apinox generates Postman Collections, OpenAPI specs, Insomnia exports, Hurl scripts, Markdown docs, and Scalar interactive docs from a single YAML/JSON schema."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate schema file (syntax + references + duplicates)
    Validate {
        /// Schema file path (.yml or .json)
        path: PathBuf,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Build output files from schema
    Build {
        /// Schema file path (.yml or .json)
        path: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "./dist")]
        output: PathBuf,

        /// Output format: postman, openapi, markdown, scalar, insomnia, hurl, all
        #[arg(short, long, default_value = "all")]
        format: String,

        /// Skip validation before building
        #[arg(long)]
        skip_validate: bool,
    },

    /// Watch schema file and auto-rebuild on changes
    Watch {
        /// Schema file path (.yml or .json)
        path: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "./dist")]
        output: PathBuf,

        /// Output format: postman, openapi, markdown, scalar, insomnia, hurl, all
        #[arg(short, long, default_value = "all")]
        format: String,
    },

    /// Show diff/changelog between two schema versions
    Diff {
        /// Schema file path (must contain changelog entries)
        path: PathBuf,

        /// From version
        from: String,

        /// To version
        to: String,

        /// Output format: text, markdown
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },

    /// Generate migration guide between two schema versions (schema diff mode)
    Migrate {
        /// Old schema file path
        old: PathBuf,

        /// New schema file path
        new: PathBuf,

        /// Output file (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import OpenAPI/Swagger spec to Apinox schema
    Import {
        /// OpenAPI 3.x or Swagger 2.0 spec file (.yml, .yaml, or .json)
        path: PathBuf,

        /// Output Apinox schema file (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Sync Postman collection to Postman workspace
    Sync {
        /// Schema file path (.yml or .json)
        path: PathBuf,

        /// Postman API key (or set POSTMAN_API_KEY env / .apinox.toml)
        #[arg(long, env = "POSTMAN_API_KEY")]
        postman_key: Option<String>,

        /// Postman workspace ID (or set POSTMAN_WORKSPACE_ID env / .apinox.toml)
        #[arg(long, env = "POSTMAN_WORKSPACE_ID")]
        workspace: Option<String>,

        /// Existing Postman collection ID to update (creates new if omitted)
        #[arg(long)]
        collection_id: Option<String>,

        /// Collection name to search for (auto-finds and updates if found)
        #[arg(long)]
        collection_name: Option<String>,

        /// Config file path
        #[arg(long, default_value = ".apinox.toml")]
        config: PathBuf,

        /// Skip validation before building
        #[arg(long)]
        skip_validate: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path, format } => cmd_validate(&path, &format),
        Commands::Build {
            path,
            output,
            format,
            skip_validate,
        } => cmd_build(&path, &output, &format, skip_validate),
        Commands::Watch {
            path,
            output,
            format,
        } => cmd_watch(&path, &output, &format),
        Commands::Diff { path, from, to, format } => cmd_diff(&path, &from, &to, &format),
        Commands::Migrate { old, new, output } => cmd_migrate(&old, &new, output.as_deref()),
        Commands::Import { path, output } => cmd_import(&path, output.as_deref()),
        Commands::Sync {
            path,
            postman_key,
            workspace,
            collection_id,
            collection_name,
            config,
            skip_validate,
        } => cmd_sync(&path, postman_key, workspace, collection_id, collection_name, &config, skip_validate),
    }
}

fn cmd_validate(path: &Path, format: &str) -> Result<()> {
    let schema = Parser::parse_file(path).context("Failed to parse schema")?;

    let result = Validator::validate(&schema);

    match format {
        "json" => {
            let output = serde_json::json!({
                "valid": !result.has_errors(),
                "summary": result.summary(),
                "messages": result.messages.iter().map(|m| {
                    serde_json::json!({
                        "severity": m.severity.to_string(),
                        "path": m.path,
                        "message": m.message,
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            for msg in &result.messages {
                let icon = match msg.severity {
                    apinox::validator::Severity::Error => "\x1b[31merror\x1b[0m",
                    apinox::validator::Severity::Warning => "\x1b[33mwarn \x1b[0m",
                    apinox::validator::Severity::Info => "\x1b[34minfo \x1b[0m",
                };
                println!("  [{}] {}: {}", icon, msg.path, msg.message);
            }
            println!();
            println!(
                "  Result: {} ({})",
                if result.has_errors() { "FAIL" } else { "PASS" },
                result.summary()
            );
        }
    }

    if result.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}

fn parse_format(fmt: &str) -> Vec<OutputFormat> {
    match fmt {
        "postman" => vec![OutputFormat::Postman],
        "openapi" => vec![OutputFormat::Openapi],
        "markdown" => vec![OutputFormat::Markdown],
        "scalar" => vec![OutputFormat::Scalar],
        "insomnia" => vec![OutputFormat::Insomnia],
        "hurl" => vec![OutputFormat::Hurl],
        _ => vec![
            OutputFormat::Postman,
            OutputFormat::Openapi,
            OutputFormat::Markdown,
            OutputFormat::Scalar,
            OutputFormat::Insomnia,
            OutputFormat::Hurl,
        ],
    }
}

fn run_build(path: &Path, output_dir: &Path, format: &str, skip_validate: bool) -> Result<bool> {
    let schema = Parser::parse_file(path).context("Failed to parse schema")?;

    // Validate
    if !skip_validate {
        let result = Validator::validate(&schema);

        for msg in result.warnings() {
            eprintln!("  [\x1b[33mwarn \x1b[0m] {}: {}", msg.path, msg.message);
        }

        if result.has_errors() {
            for msg in result.errors() {
                eprintln!("  [\x1b[31merror\x1b[0m] {}: {}", msg.path, msg.message);
            }
            eprintln!();
            anyhow::bail!(
                "Validation failed ({}). Use --skip-validate to ignore.",
                result.summary()
            );
        }
    }

    std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    let slug = schema.name.replace(' ', "-").to_lowercase();
    let formats = parse_format(format);

    let mut generated = Vec::new();

    for fmt in &formats {
        let (dir_name, ext) = match fmt {
            OutputFormat::Postman => ("postman", ".postman_collection.json"),
            OutputFormat::Openapi => ("openapi", ".openapi.yaml"),
            OutputFormat::Markdown => ("docs", ".md"),
            OutputFormat::Scalar => ("docs", ".scalar.html"),
            OutputFormat::Insomnia => ("insomnia", ".insomnia.json"),
            OutputFormat::Hurl => ("hurl", ".hurl"),
        };

        let out_dir = output_dir.join(dir_name);
        std::fs::create_dir_all(&out_dir)?;

        let content = generator::generate(&schema, fmt.clone())?;
        let out_path = out_dir.join(format!("{}{}", slug, ext));
        std::fs::write(&out_path, &content)?;
        generated.push(out_path.display().to_string());
    }

    // Also generate hurl shell script if hurl is included
    if formats.contains(&OutputFormat::Hurl) {
        
        if let Ok(hurl_out) = hurl::generate_all(&schema) {
            let sh_path = output_dir.join("hurl").join(format!("{}.sh", slug));
            std::fs::write(&sh_path, &hurl_out.shell_script)?;
            generated.push(sh_path.display().to_string());
        }
    }

    println!("\x1b[32m  \x1b[1mGenerated:\x1b[0m");
    for g in &generated {
        println!("    {}", g);
    }
    println!();
    println!(
        "  \x1b[32mBuild complete\x1b[0m for {} v{}",
        schema.name, schema.version
    );

    Ok(true)
}

fn cmd_build(
    path: &Path,
    output_dir: &Path,
    format: &str,
    skip_validate: bool,
) -> Result<()> {
    run_build(path, output_dir, format, skip_validate)?;
    Ok(())
}

fn cmd_watch(path: &Path, output_dir: &Path, format: &str) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(tx)
        .context("Failed to create file watcher")?;

    let watch_path = path
        .parent()
        .unwrap_or(Path::new("."));
    watcher.watch(
        watch_path,
        notify::RecursiveMode::NonRecursive,
    )?;

    // Initial build
    println!("\x1b[34m  Watching\x1b[0m {} for changes...", path.display());
    println!();

    if let Err(e) = run_build(path, output_dir, format, true) {
        eprintln!("  \x1b[31mBuild error:\x1b[0m {}", e);
    }

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    // Debounce: small delay
                    std::thread::sleep(Duration::from_millis(300));

                    // Check if our file was modified
                    let path_str = path.to_string_lossy().to_string();
                    let changed = event.paths.iter().any(|p| {
                        p.to_string_lossy().contains(&path_str)
                    });

                    if changed {
                        println!("\x1b[33m  Rebuilding...\x1b[0m");
                        if let Err(e) = run_build(path, output_dir, format, true) {
                            eprintln!("  \x1b[31mBuild error:\x1b[0m {}", e);
                        } else {
                            println!("\x1b[32m  Done\x1b[0m\n");
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {}", e),
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn cmd_diff(path: &Path, from: &str, to: &str, format: &str) -> Result<()> {
    let output = apinox::diff::load_and_diff(path, from, to)
        .context("Failed to generate diff")?;

    match format {
        "text" => {
            // Strip markdown headers for plain text
            let text = output.lines()
                .filter(|l| !l.starts_with("#"))
                .collect::<Vec<_>>()
                .join("\n");
            println!("{}", text);
        }
        _ => println!("{}", output),
    }

    Ok(())
}

fn cmd_migrate(old_path: &Path, new_path: &Path, output: Option<&Path>) -> Result<()> {
    let old_schema = Parser::parse_file(old_path)
        .context("Failed to parse old schema")?;
    let new_schema = Parser::parse_file(new_path)
        .context("Failed to parse new schema")?;

    let diff = apinox::diff::diff_schemas(&old_schema, &new_schema);
    let guide = apinox::diff::generate_migration_guide(&diff);

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &guide)?;
            println!(
                "\x1b[32m  Migration guide written to:\x1b[0m {}",
                out_path.display()
            );
        }
        None => println!("{}", guide),
    }

    Ok(())
}

fn cmd_import(path: &Path, output: Option<&Path>) -> Result<()> {
    let schema = apinox::importer::import_openapi(path)
        .context("Failed to import OpenAPI/Swagger spec")?;

    let yaml = serde_yaml::to_string(&schema)
        .context("Failed to serialize Apinox schema")?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &yaml)
                .with_context(|| format!("Failed to write: {}", out_path.display()))?;
            println!(
                "\x1b[32m  Imported\x1b[0m → {}",
                out_path.display()
            );
            println!(
                "  \x1b[32m{} endpoints\x1b[0m converted",
                schema.endpoints.len()
            );
        }
        None => {
            print!("{}", yaml);
        }
    }

    Ok(())
}

/// Config file structure for `.apinox.toml`
#[derive(serde::Deserialize, Default)]
struct ApinoxConfig {
    sync: Option<SyncConfig>,
}

#[derive(serde::Deserialize, Default)]
struct SyncConfig {
    postman: Option<postman::PostmanSyncConfig>,
}

/// Load config from `.apinox.toml` file (if it exists)
fn load_config(config_path: &std::path::Path) -> ApinoxConfig {
    match std::fs::read_to_string(config_path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => ApinoxConfig::default(),
    }
}
fn cmd_sync(
    path: &Path,
    postman_key: Option<String>,
    workspace: Option<String>,
    collection_id: Option<String>,
    collection_name: Option<String>,
    config_path: &Path,
    skip_validate: bool,
) -> Result<()> {
    // Load config from .apinox.toml
    let file_config = load_config(config_path);

    // Resolve config: CLI args > env vars > .apinox.toml
    let api_key = postman_key
        .or_else(|| {
            file_config
                .sync
                .as_ref()
                .and_then(|s| s.postman.as_ref())
                .map(|p| p.api_key.clone())
        })
        .context("Postman API key required. Use --postman-key, POSTMAN_API_KEY env, or .apinox.toml")?;

    let workspace_id = workspace
        .or_else(|| {
            file_config
                .sync
                .as_ref()
                .and_then(|s| s.postman.as_ref())
                .map(|p| p.workspace_id.clone())
        })
        .context("Postman workspace ID required. Use --workspace, POSTMAN_WORKSPACE_ID env, or .apinox.toml")?;

    let existing_collection_id = collection_id.or_else(|| {
        file_config
            .sync
            .as_ref()
            .and_then(|s| s.postman.as_ref())
            .and_then(|p| p.collection_id.clone())
    });

    // Parse and validate schema
    let schema = Parser::parse_file(path).context("Failed to parse schema")?;

    if !skip_validate {
        let result = Validator::validate(&schema);
        if result.has_errors() {
            for msg in result.errors() {
                eprintln!("  [\x1b[31merror\x1b[0m] {}: {}", msg.path, msg.message);
            }
            anyhow::bail!("Validation failed ({}). Use --skip-validate to ignore.", result.summary());
        }
    }

    // Generate Postman collection
    let collection_json = generator::generate(&schema, OutputFormat::Postman)
        .context("Failed to generate Postman collection")?;

    // Resolve collection_id: explicit arg > file config > name lookup
    let mut resolved_collection_id = existing_collection_id;

    if resolved_collection_id.is_none() {
        if let Some(ref name) = collection_name {
            println!("\x1b[34m  Searching\x1b[0m for collection '{}'...", name);
            match postman::find_collection(&api_key, &workspace_id, name) {
                Ok(Some(id)) => {
                    println!("\x1b[32m  Found\x1b[0m collection: {}", id);
                    resolved_collection_id = Some(id);
                }
                Ok(None) => {
                    println!("\x1b[33m  Not found\x1b[0m. Will create new collection.");
                }
                Err(e) => {
                    eprintln!("  \x1b[33mwarn \x1b[0m Failed to search collections: {}", e);
                }
            }
        }
    }

    let sync_config = PostmanSyncConfig {
        api_key,
        workspace_id,
        collection_id: resolved_collection_id,
    };

    println!("\x1b[34m  Syncing\x1b[0m to Postman...");
    let result = postman::sync_collection(&sync_config, &collection_json)?;

    let action_icon = match result.action.as_str() {
        "created" => "\x1b[32m  Created\x1b[0m",
        "updated" => "\x1b[33m  Updated\x1b[0m",
        _ => "\x1b[34m  Synced\x1b[0m",
    };

    println!();
    println!("  {} collection:", action_icon);
    println!("    ID:   {}", result.collection_id);
    println!("    UID:  {}", result.collection_uid);
    println!("    URL:  {}", result.url);
    println!();

    Ok(())
}

// Hurl generate_all wrapper
mod hurl {
    use apinox::schema::root::ApinoxSchema;
    use anyhow::Result;

    pub struct HurlOutput {
        pub shell_script: String,
    }

    pub fn generate_all(schema: &ApinoxSchema) -> Result<HurlOutput> {
        let out = apinox::generator::hurl::generate_all(schema)?;
        Ok(HurlOutput { shell_script: out.curl_script })
    }
}