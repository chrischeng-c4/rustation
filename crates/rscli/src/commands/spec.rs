//! Spec-kit workflow commands

use crate::runners::{bash, python};
use crate::{Result, RscliError};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Feature {
    id: String,
    name: String,
    description: String,
    status: String,
    #[serde(default)]
    phase: u32,
}

#[derive(Debug, Deserialize)]
struct FeatureCatalog {
    project: String,
    description: String,
    features: Vec<Feature>,
}

/// Get repository root
fn get_repo_root() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout)
            .map_err(|e| RscliError::Other(e.into()))?
            .trim()
            .to_string();
        Ok(PathBuf::from(path))
    } else {
        std::env::current_dir().map_err(Into::into)
    }
}

/// Get current feature from git branch
fn get_current_feature() -> Result<Option<String>> {
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)
            .map_err(|e| RscliError::Other(e.into()))?
            .trim()
            .to_string();

        // Extract feature number from branch name (e.g., "038-test-command" -> "038")
        if let Some(num) = branch.split('-').next() {
            if num.chars().all(|c| c.is_numeric()) && num.len() == 3 {
                return Ok(Some(num.to_string()));
            }
        }
    }

    Ok(None)
}

/// Load features from features.json
fn load_features() -> Result<Vec<Feature>> {
    let repo_root = get_repo_root()?;
    let features_path = repo_root.join("specs/features.json");

    if !features_path.exists() {
        return Err(RscliError::Other(anyhow::anyhow!(
            "features.json not found at {}",
            features_path.display()
        )));
    }

    let content = fs::read_to_string(&features_path)?;
    let catalog: FeatureCatalog = serde_json::from_str(&content)?;

    Ok(catalog.features)
}

/// Show feature status
pub async fn status(feature_num: Option<String>, _verbose: bool) -> Result<()> {
    let feature = match feature_num {
        Some(num) => num,
        None => get_current_feature()?.ok_or_else(|| {
            RscliError::Other(anyhow::anyhow!(
                "Could not detect feature from branch. Please specify feature number."
            ))
        })?,
    };

    println!("{}", format!("Feature {} Status", feature).bright_blue().bold());
    println!("{}", "=".repeat(50));
    println!();

    // Check if spec directory exists
    let repo_root = get_repo_root()?;
    let spec_dir = repo_root.join(format!("specs/{:0>3}-*", feature));

    // Find the spec directory
    let spec_dirs: Vec<_> = glob::glob(&spec_dir.to_string_lossy())
        .map_err(|e| RscliError::Other(e.into()))?
        .filter_map(|r| r.ok())
        .collect();

    if spec_dirs.is_empty() {
        println!("{}", format!("Feature {} not found", feature).red());
        return Ok(());
    }

    let spec_dir = &spec_dirs[0];

    // Check for spec artifacts
    let spec_md = spec_dir.join("spec.md");
    let plan_md = spec_dir.join("plan.md");
    let tasks_md = spec_dir.join("tasks.md");

    println!("{}", "Spec Artifacts:".bright_cyan());
    print_file_status(&spec_md, "spec.md");
    print_file_status(&plan_md, "plan.md");
    print_file_status(&tasks_md, "tasks.md");
    println!();

    // Load feature info from features.json
    let features = load_features()?;
    if let Some(feat) = features.iter().find(|f| f.id == feature) {
        println!("{}", "Feature Info:".bright_cyan());
        println!("  Name: {}", feat.name);
        println!("  Status: {}", feat.status);
        println!("  Description: {}", feat.description);
    }

    Ok(())
}

fn print_file_status(path: &PathBuf, name: &str) {
    if path.exists() {
        let metadata = fs::metadata(path).ok();
        let size = metadata.map(|m| m.len()).unwrap_or(0);
        println!("  {} {} ({} bytes)", "✓".green(), name, size);
    } else {
        println!("  {} {}", "✗".red(), name);
    }
}

/// List all features
pub async fn list(_verbose: bool) -> Result<()> {
    let features = load_features()?;

    println!("{}", "Rush Shell Features".bright_blue().bold());
    println!("{}", "=".repeat(80));
    println!();

    println!(
        "{:<6} {:<12} {:<40} {:}",
        "ID".bright_cyan(),
        "Status".bright_cyan(),
        "Name".bright_cyan(),
        "Description".bright_cyan()
    );
    println!("{}", "-".repeat(80));

    let total = features.len();
    for feature in features {
        let status_colored = match feature.status.as_str() {
            "complete" => feature.status.green(),
            "in-progress" => feature.status.yellow(),
            "blocked" => feature.status.red(),
            _ => feature.status.normal(),
        };

        println!(
            "{:<6} {:<12} {:<40} {}",
            feature.id, status_colored, feature.name, feature.description
        );
    }

    println!();
    println!("Total: {} features", total);

    Ok(())
}

/// Create new feature
pub async fn create(description: &str, short_name: Option<&str>, _verbose: bool) -> Result<()> {
    println!("{}", "Creating new feature...".bright_blue());
    println!();

    // Build arguments for the script
    let mut args = vec!["--json"];
    if let Some(name) = short_name {
        args.push("--short-name");
        args.push(name);
    }
    args.push(description);

    // Call the bash script
    let output = bash::run_script("create-new-feature", &args).await?;

    // Parse JSON output
    #[derive(Debug, Deserialize)]
    struct CreateResult {
        #[serde(rename = "BRANCH_NAME")]
        branch_name: String,
        #[serde(rename = "SPEC_FILE")]
        spec_file: String,
    }

    let result: CreateResult = serde_json::from_str(&output)?;

    println!("{}", "✓ Feature created".green());
    println!("  Branch: {}", result.branch_name.bright_blue());
    println!("  Spec: {}", result.spec_file.bright_blue());
    println!();
    println!("{}", "→ Next steps:".bright_blue());
    println!("  1. Edit the spec file");
    println!("  2. Run: rscli spec run specify");

    Ok(())
}

/// Run spec workflow phase
pub async fn run(phase: &str, feature_num: Option<String>, _verbose: bool) -> Result<()> {
    let feature = match feature_num {
        Some(num) => num,
        None => get_current_feature()?.ok_or_else(|| {
            RscliError::Other(anyhow::anyhow!(
                "Could not detect feature from branch. Please specify feature number."
            ))
        })?,
    };

    println!(
        "{}",
        format!("Running {} phase for feature {}...", phase, feature)
            .bright_blue()
            .bold()
    );
    println!();

    // Call Python speckit
    python::run_speckit(phase, &[&feature]).await?;

    println!();
    println!("{}", format!("✓ {} phase complete", phase).green());

    Ok(())
}
