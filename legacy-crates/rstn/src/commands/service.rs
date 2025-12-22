//! Service command implementation (simplified)

use crate::domain::service;
use crate::{Result, RscliError};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

pub async fn list() -> Result<()> {
    println!("{}", "Checking development services...".bright_blue());
    println!();

    let services = service::list_services()
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("Service").fg(Color::Cyan),
        Cell::new("Status").fg(Color::Cyan),
        Cell::new("PID").fg(Color::Cyan),
        Cell::new("Command").fg(Color::Cyan),
    ]);

    for svc in &services {
        let status_cell = match svc.state {
            service::ServiceState::Running => Cell::new("Running").fg(Color::Green),
            service::ServiceState::Stopped => Cell::new("Stopped").fg(Color::DarkGrey),
            service::ServiceState::Unknown => Cell::new("Unknown").fg(Color::Yellow),
        };

        let pid_str = svc
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(&svc.name),
            status_cell,
            Cell::new(pid_str),
            Cell::new(&svc.command),
        ]);
    }

    println!("{table}");
    println!();

    let running_count = services
        .iter()
        .filter(|s| s.state == service::ServiceState::Running)
        .count();
    println!(
        "Running: {}/{}",
        running_count.to_string().bright_green(),
        services.len()
    );
    println!();

    Ok(())
}

pub async fn status(name: String) -> Result<()> {
    println!(
        "{}",
        format!("Checking status of '{}'...", name).bright_blue()
    );
    println!();

    let svc = service::get_service_status(&name)
        .await
        .map_err(|e| RscliError::Other(e.into()))?;

    println!("{}: {}", "Service".bright_cyan(), svc.name);
    println!("{}: {}", "Command".bright_cyan(), svc.command);

    let status_str = match svc.state {
        service::ServiceState::Running => "Running".green(),
        service::ServiceState::Stopped => "Stopped".red(),
        service::ServiceState::Unknown => "Unknown".yellow(),
    };
    println!("{}: {}", "Status".bright_cyan(), status_str);

    if let Some(pid) = svc.pid {
        println!("{}: {}", "PID".bright_cyan(), pid);
    }

    println!();

    Ok(())
}
