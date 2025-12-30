//! Context Generation system for Living Context Layer.
//!
//! Generates initial context files by analyzing the codebase structure with AI.

use std::path::Path;

use crate::context_sync::extract_json_from_response;

/// Summary of codebase structure for AI analysis
#[derive(Debug, Clone)]
pub struct CodebaseSummary {
    /// Package configuration (package.json or Cargo.toml content)
    pub package_config: String,
    /// README content
    pub readme: String,
    /// Directory tree structure
    pub directory_tree: String,
    /// List of key source files
    pub key_files: Vec<String>,
}

/// Response from Claude for context generation
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenerateContextResponse {
    pub files: GeneratedFiles,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GeneratedFiles {
    pub product: FileContent,
    pub tech_stack: FileContent,
    pub architecture: FileContent,
    pub recent_changes: FileContent,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileContent {
    pub content: String,
}

impl GenerateContextResponse {
    /// Parse JSON string into GenerateContextResponse
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let json_str = extract_json_from_response(json_str);
        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse generate context response: {}", e))
    }
}

/// Read codebase summary for AI analysis
pub fn read_codebase_summary(project_path: &Path) -> CodebaseSummary {
    // Read package.json or Cargo.toml
    let package_config = std::fs::read_to_string(project_path.join("package.json"))
        .or_else(|_| std::fs::read_to_string(project_path.join("Cargo.toml")))
        .or_else(|_| std::fs::read_to_string(project_path.join("pyproject.toml")))
        .unwrap_or_else(|_| "(no package config found)".to_string());

    // Read README
    let readme = std::fs::read_to_string(project_path.join("README.md"))
        .or_else(|_| std::fs::read_to_string(project_path.join("readme.md")))
        .unwrap_or_else(|_| "(no README found)".to_string());

    // Build directory tree
    let directory_tree = build_directory_tree(project_path);

    // Find key files
    let key_files = find_key_files(project_path);

    CodebaseSummary {
        package_config,
        readme,
        directory_tree,
        key_files,
    }
}

/// Build a tree-like representation of the directory structure
fn build_directory_tree(path: &Path) -> String {
    let mut result = Vec::new();
    build_tree_recursive(path, "", &mut result, 0, 3);
    result.join("\n")
}

fn build_tree_recursive(
    path: &Path,
    prefix: &str,
    result: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    // Directories to skip
    let skip_dirs = [
        "node_modules",
        "target",
        ".git",
        ".idea",
        ".vscode",
        "dist",
        "build",
        "__pycache__",
        ".next",
        "out",
        ".turbo",
        "coverage",
    ];

    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.') || name == ".rstn"
            })
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                !skip_dirs.contains(&name.as_str())
            })
            .collect(),
        Err(_) => return,
    };

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let name = entry.file_name().to_string_lossy().to_string();
        let connector = if is_last { "└── " } else { "├── " };

        if entry.path().is_dir() {
            result.push(format!("{}{}{}/", prefix, connector, name));
            let new_prefix = format!("{}{}   ", prefix, if is_last { " " } else { "│" });
            build_tree_recursive(&entry.path(), &new_prefix, result, depth + 1, max_depth);
        } else {
            result.push(format!("{}{}{}", prefix, connector, name));
        }
    }
}

/// Find key source files in the project
fn find_key_files(path: &Path) -> Vec<String> {
    let mut key_files = Vec::new();

    // Common entry point patterns
    let entry_patterns = [
        "src/main.rs",
        "src/lib.rs",
        "src/index.ts",
        "src/index.tsx",
        "src/main.ts",
        "src/App.tsx",
        "src/app.py",
        "main.py",
        "app.py",
        "index.js",
        "index.ts",
    ];

    for pattern in entry_patterns {
        let file_path = path.join(pattern);
        if file_path.exists() {
            key_files.push(pattern.to_string());
        }
    }

    // Limit to first 10 files
    key_files.truncate(10);
    key_files
}

/// Build the prompt for context generation
pub fn build_generate_context_prompt(summary: &CodebaseSummary) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    format!(
        r#"You are a context curator for a software project. Analyze the codebase and generate comprehensive Living Context files.

## Codebase Information

### Package Configuration
```
{package_config}
```

### README
```
{readme}
```

### Directory Structure
```
{directory_tree}
```

### Key Source Files
{key_files}

## Instructions

Generate complete content for each Living Context file based on your analysis:

1. **product.md** - Product overview, target users, key features (based on README and package info)
2. **tech-stack.md** - Languages, frameworks, libraries with versions and purposes
3. **system-architecture.md** - Component overview, data flow, key patterns
4. **recent-changes.md** - Initialize with an empty table (no change history yet)

Each file MUST include YAML frontmatter with: name, type, last_updated, token_estimate

## Output Format

Return a JSON object with this exact structure:

```json
{{
  "files": {{
    "product": {{
      "content": "---\nname: \"Product\"\ntype: product\nlast_updated: \"{today}\"\ntoken_estimate: 300\n---\n\n# Product Overview\n\n[content here]"
    }},
    "tech_stack": {{
      "content": "---\nname: \"Tech Stack\"\ntype: tech-stack\nlast_updated: \"{today}\"\ntoken_estimate: 400\n---\n\n# Technology Stack\n\n| Technology | Version | Purpose |\n|------------|---------|---------||\n| ... | ... | ... |"
    }},
    "architecture": {{
      "content": "---\nname: \"System Architecture\"\ntype: architecture\nlast_updated: \"{today}\"\ntoken_estimate: 500\n---\n\n# System Architecture\n\n[content here]"
    }},
    "recent_changes": {{
      "content": "---\nname: \"Recent Changes\"\ntype: recent-changes\nlast_updated: \"{today}\"\ntoken_estimate: 200\n---\n\n# Recent Changes\n\n| Date | Change | Impact |\n|------|--------|--------|\n"
    }}
  }},
  "summary": "Generated context for [project name]"
}}
```

Rules:
- Be concise but comprehensive
- Use markdown tables where appropriate
- Extract real version numbers from package config
- Infer architecture from directory structure
- Use today's date ({today}) for last_updated

Respond ONLY with the JSON object, no additional text."#,
        package_config = truncate_content(&summary.package_config, 2000),
        readme = truncate_content(&summary.readme, 3000),
        directory_tree = summary.directory_tree,
        key_files = if summary.key_files.is_empty() {
            "(no key files found)".to_string()
        } else {
            summary.key_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
        },
        today = today,
    )
}

/// Truncate content to a maximum length
fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...(truncated)", &content[..max_len])
    }
}

/// Write generated context files to disk
pub fn write_generated_context(
    project_path: &Path,
    response: &GenerateContextResponse,
) -> Result<(), String> {
    let context_dir = project_path.join(".rstn").join("context");

    // Create directory if needed
    std::fs::create_dir_all(&context_dir)
        .map_err(|e| format!("Failed to create context directory: {}", e))?;

    // Write each file
    std::fs::write(context_dir.join("product.md"), &response.files.product.content)
        .map_err(|e| format!("Failed to write product.md: {}", e))?;

    std::fs::write(
        context_dir.join("tech-stack.md"),
        &response.files.tech_stack.content,
    )
    .map_err(|e| format!("Failed to write tech-stack.md: {}", e))?;

    std::fs::write(
        context_dir.join("system-architecture.md"),
        &response.files.architecture.content,
    )
    .map_err(|e| format!("Failed to write system-architecture.md: {}", e))?;

    std::fs::write(
        context_dir.join("recent-changes.md"),
        &response.files.recent_changes.content,
    )
    .map_err(|e| format!("Failed to write recent-changes.md: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_content() {
        let short = "hello";
        assert_eq!(truncate_content(short, 100), "hello");

        let long = "a".repeat(200);
        let truncated = truncate_content(&long, 50);
        assert!(truncated.ends_with("...(truncated)"));
        assert!(truncated.len() < 200);
    }

    #[test]
    fn test_parse_generate_context_response() {
        let json = r##"{
            "files": {
                "product": {"content": "# Product"},
                "tech_stack": {"content": "# Tech Stack"},
                "architecture": {"content": "# Architecture"},
                "recent_changes": {"content": "# Recent Changes"}
            },
            "summary": "Generated context"
        }"##;

        let response = GenerateContextResponse::from_json(json).unwrap();
        assert_eq!(response.files.product.content, "# Product");
        assert_eq!(response.summary, "Generated context");
    }
}
