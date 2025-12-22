//! Worktree command implementation

use crate::domain::git::worktree;
use crate::{Result, RscliError};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use std::path::PathBuf;

pub async fn list(verbose: bool) -> Result<()> {
    let worktrees = worktree::list_worktrees()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    if worktrees.is_empty() {
        println!("{}", "No worktrees found".yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} worktree(s)", worktrees.len())
            .bright_blue()
            .bold()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Header
    if verbose {
        table.set_header(vec![
            Cell::new("Path").fg(Color::Cyan),
            Cell::new("Branch").fg(Color::Cyan),
            Cell::new("Commit").fg(Color::Cyan),
            Cell::new("Feature").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
        ]);
    } else {
        table.set_header(vec![
            Cell::new("Path").fg(Color::Cyan),
            Cell::new("Branch").fg(Color::Cyan),
            Cell::new("Feature").fg(Color::Cyan),
        ]);
    }

    // Add rows
    for wt in &worktrees {
        let path_str = wt.path.to_string_lossy();
        let branch_str = wt.branch.as_deref().unwrap_or("(detached)");

        // Parse feature info
        let feature_str = if let Some(ref branch) = wt.branch {
            worktree::parse_feature_branch(branch)
                .map(|f| format!("{} - {}", f.number, f.name))
                .unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };

        // Status
        let status = if wt.is_bare {
            "bare"
        } else if wt.locked.is_some() {
            "locked"
        } else if wt.is_detached {
            "detached"
        } else {
            "active"
        };

        if verbose {
            table.add_row(vec![
                Cell::new(&path_str),
                Cell::new(branch_str),
                Cell::new(&wt.commit[..7.min(wt.commit.len())]),
                Cell::new(feature_str),
                Cell::new(status),
            ]);
        } else {
            table.add_row(vec![
                Cell::new(&path_str),
                Cell::new(branch_str),
                Cell::new(feature_str),
            ]);
        }
    }

    println!("{table}");
    println!();

    Ok(())
}

pub async fn status() -> Result<()> {
    let current_path = worktree::get_current_worktree()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;
    let current_branch = worktree::get_current_branch()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!("{}", "Current Worktree Status".bright_blue().bold());
    println!();
    println!("Path:   {}", current_path.display());

    if let Some(branch) = current_branch {
        println!("Branch: {}", branch.bright_green());

        // Try to parse feature info
        if let Some(feature) = worktree::parse_feature_branch(&branch) {
            println!();
            println!("{}", "Feature Info:".bright_cyan());
            println!("  Number: {}", feature.number);
            println!("  Name:   {}", feature.name);
            if let Some(component) = feature.component {
                println!("  Component: {}", component);
            }
        }
    } else {
        println!("Branch: {}", "(detached HEAD)".yellow());
    }

    println!();

    Ok(())
}

pub async fn create(feature: String, base_path: Option<PathBuf>) -> Result<()> {
    println!(
        "{}",
        format!("Creating worktree for feature '{}'...", feature).bright_blue()
    );

    let worktree_path = worktree::create_worktree(&feature, base_path)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!();
    println!("{}", "✓ Worktree created successfully!".green().bold());
    println!("Path: {}", worktree_path.display());
    println!("Branch: {}", format!("feature/{}", feature).bright_green());
    println!();
    println!("To switch to this worktree:");
    println!(
        "  {}",
        format!("cd {}", worktree_path.display()).bright_cyan()
    );
    println!();

    Ok(())
}

pub async fn remove(path: String, force: bool) -> Result<()> {
    if force {
        println!(
            "{}",
            format!("Force removing worktree '{}'...", path).yellow()
        );
    } else {
        println!(
            "{}",
            format!("Removing worktree '{}'...", path).bright_blue()
        );
    }

    worktree::remove_worktree(&path, force)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!("{}", "✓ Worktree removed successfully!".green().bold());
    println!();

    Ok(())
}
