//! Justfile parser for extracting commands and descriptions.

use napi_derive::napi;
use std::fs;
use std::path::Path;
use std::process::Command;

/// A command parsed from a justfile
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JustCommand {
    /// Command name (e.g., "test", "build")
    pub name: String,
    /// Description from comment above the command
    pub description: Option<String>,
    /// The recipe/shell commands
    pub recipe: String,
}

/// Parse a justfile and extract all commands
pub fn parse_justfile(path: &str) -> Result<Vec<JustCommand>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read justfile: {}", e))?;

    let mut commands = Vec::new();
    let mut pending_description: Option<String> = None;
    let mut current_name: Option<String> = None;
    let mut current_description: Option<String> = None;
    let mut current_recipe = String::new();
    let mut in_recipe = false;

    for line in content.lines() {
        // Check for comment (description for next command)
        // A comment at the start of a line (not indented) ends the current recipe
        if line.starts_with('#') {
            in_recipe = false;
            let comment = line.trim_start_matches('#').trim();
            pending_description = Some(comment.to_string());
            continue;
        }

        // Check for command definition (name followed by colon)
        if !line.starts_with(' ') && !line.starts_with('\t') && line.contains(':') {
            // Save previous command if exists
            if let Some(name) = current_name.take() {
                commands.push(JustCommand {
                    name,
                    description: current_description.take(),
                    recipe: current_recipe.trim().to_string(),
                });
                current_recipe = String::new();
            }

            // Parse new command name
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if !parts.is_empty() {
                let name = parts[0].trim();
                // Skip if it looks like a variable assignment
                if !name.contains('=') && !name.is_empty() {
                    current_name = Some(name.to_string());
                    // The pending description belongs to THIS command
                    current_description = pending_description.take();
                    in_recipe = true;
                }
            }
            continue;
        }

        // Collect recipe lines (indented with spaces or tabs)
        if in_recipe && (line.starts_with(' ') || line.starts_with('\t')) {
            current_recipe.push_str(line.trim());
            current_recipe.push('\n');
        } else if in_recipe && line.trim().is_empty() {
            // Empty line might still be part of recipe
            current_recipe.push('\n');
        } else {
            // Non-indented non-empty line ends recipe
            if !line.trim().is_empty() {
                in_recipe = false;
            }
        }

        // Reset pending description if not followed by command
        if !line.starts_with('#') && !line.trim().is_empty() && current_name.is_none() {
            pending_description = None;
        }
    }

    // Save last command
    if let Some(name) = current_name {
        commands.push(JustCommand {
            name,
            description: current_description,
            recipe: current_recipe.trim().to_string(),
        });
    }

    Ok(commands)
}

/// Run a just command in a directory
pub fn run_just_command(command: &str, cwd: &str) -> Result<String, String> {
    let cwd_path = Path::new(cwd);
    if !cwd_path.exists() {
        return Err(format!("Directory does not exist: {}", cwd));
    }

    let output = Command::new("just")
        .arg(command)
        .current_dir(cwd_path)
        .output()
        .map_err(|e| format!("Failed to run just: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(format!("{}{}", stdout, stderr))
    } else {
        Err(format!("Command failed:\n{}{}", stdout, stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_justfile() {
        let content = r#"
# Install to ~/.local/bin
install:
    uv pip install -e ".[dev]"
    mkdir -p ~/.local/bin

# Run tests
test:
    uv run pytest tests/ -v
"#;
        // Write temp file
        let temp_path = "/tmp/test_justfile";
        fs::write(temp_path, content).unwrap();

        let commands = parse_justfile(temp_path).unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].name, "install");
        assert_eq!(commands[0].description, Some("Install to ~/.local/bin".to_string()));
        assert_eq!(commands[1].name, "test");
        assert_eq!(commands[1].description, Some("Run tests".to_string()));

        fs::remove_file(temp_path).ok();
    }
}
