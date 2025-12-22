//! MCP command implementation

use crate::domain::mcp;
use crate::{Result, RscliError};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use std::path::PathBuf;

pub async fn list(verbose: bool) -> Result<()> {
    let registry_path = mcp::find_registry_path()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!(
        "{}",
        format!("Loading MCP registry from: {}", registry_path.display()).bright_blue()
    );
    println!();

    let registry = mcp::load_registry(&registry_path)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!(
        "{}",
        format!("Found {} MCP server(s)", registry.servers.len())
            .bright_green()
            .bold()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    if verbose {
        table.set_header(vec![
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Command").fg(Color::Cyan),
            Cell::new("Args").fg(Color::Cyan),
            Cell::new("Description").fg(Color::Cyan),
        ]);

        for server in &registry.servers {
            table.add_row(vec![
                Cell::new(&server.name),
                Cell::new(&server.command),
                Cell::new(server.args.join(" ")),
                Cell::new(server.description.as_deref().unwrap_or("-")),
            ]);
        }
    } else {
        table.set_header(vec![
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Command").fg(Color::Cyan),
            Cell::new("Description").fg(Color::Cyan),
        ]);

        for server in &registry.servers {
            table.add_row(vec![
                Cell::new(&server.name),
                Cell::new(&server.command),
                Cell::new(server.description.as_deref().unwrap_or("-")),
            ]);
        }
    }

    println!("{table}");
    println!();

    // Show component mappings if available
    if !registry.component_mappings.is_empty() {
        println!("{}", "Component Mappings:".bright_cyan().bold());
        for (component, servers) in &registry.component_mappings {
            println!("  {}: {}", component.bright_yellow(), servers.join(", "));
        }
        println!();
    }

    Ok(())
}

pub async fn generate(component: Option<String>, output: Option<PathBuf>) -> Result<()> {
    let registry_path = mcp::find_registry_path()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    let registry = mcp::load_registry(&registry_path)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!("{}", "Generating MCP configuration...".bright_blue());
    if let Some(ref comp) = component {
        println!("Component: {}", comp.bright_yellow());
    } else {
        println!("Including: {}", "All servers".bright_yellow());
    }
    println!();

    let config = mcp::generate_mcp_config(&registry, component.as_deref())
        .map_err(|e| RscliError::Other(e.into()))?;

    let output_path = output.unwrap_or_else(|| PathBuf::from(".mcp.json"));

    mcp::write_mcp_config(&config, &output_path)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!(
        "{}",
        "✓ MCP configuration generated successfully!".green().bold()
    );
    println!(
        "Output: {}",
        output_path.display().to_string().bright_cyan()
    );
    println!(
        "Servers: {}",
        config.mcp_servers.len().to_string().bright_green()
    );
    println!();

    Ok(())
}

pub async fn validate() -> Result<()> {
    let config_path = PathBuf::from(".mcp.json");

    if !config_path.exists() {
        println!("{}", "✗ .mcp.json not found".red().bold());
        println!("Run 'rscli mcp generate' to create one");
        return Err(RscliError::Other(anyhow::anyhow!("MCP config not found")));
    }

    println!("{}", "Validating MCP configuration...".bright_blue());
    println!();

    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    let config: mcp::McpConfig =
        serde_json::from_str(&content).map_err(|e| RscliError::Other(e.into()))?;

    let warnings = mcp::validate_mcp_config(&config).map_err(|e| RscliError::Other(e.into()))?;

    if warnings.is_empty() {
        println!("{}", "✓ Configuration is valid!".green().bold());
        println!(
            "Servers configured: {}",
            config.mcp_servers.len().to_string().bright_green()
        );
    } else {
        println!("{}", "⚠ Configuration has warnings:".yellow().bold());
        for warning in &warnings {
            println!("  - {}", warning.yellow());
        }
    }
    println!();

    Ok(())
}

pub async fn info(server_name: String) -> Result<()> {
    let registry_path = mcp::find_registry_path()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    let registry = mcp::load_registry(&registry_path)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    let server = registry
        .servers
        .iter()
        .find(|s| s.name == server_name)
        .ok_or_else(|| RscliError::Other(anyhow::anyhow!("Server '{}' not found", server_name)))?;

    println!(
        "{}",
        format!("MCP Server: {}", server.name).bright_blue().bold()
    );
    println!();
    println!("{}: {}", "Command".bright_cyan(), server.command);
    println!("{}: {}", "Args".bright_cyan(), server.args.join(" "));

    if let Some(ref desc) = server.description {
        println!("{}: {}", "Description".bright_cyan(), desc);
    }

    if let Some(ref env) = server.env {
        println!();
        println!("{}", "Environment:".bright_cyan());
        for (key, value) in env {
            println!("  {} = {}", key.bright_yellow(), value);
        }
    }
    println!();

    Ok(())
}
