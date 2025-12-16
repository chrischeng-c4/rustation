# Quickstart: Internalize Spec Generation

**Feature**: 052-internalize-spec-generation
**Date**: 2025-12-16

## Overview

This feature adds native Rust spec generation to `rstn-core`, replacing the external bash script. The implementation lives in a new `specify` module.

## Module Location

```
crates/rstn-core/src/specify/
├── mod.rs                 # Public API
├── number_allocator.rs    # Feature number allocation
├── name_generator.rs      # Kebab-case name generation
├── directory_setup.rs     # Directory creation
├── spec_generator.rs      # Claude CLI integration
└── catalog_updater.rs     # features.json management
```

## Quick Usage

### From Rust Code (rstn-core API)

```rust
use rstn_core::specify::{generate_spec, SpecifyConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from("/path/to/project");
    let description = "Add user authentication with OAuth2".to_string();

    // With default config
    let result = generate_spec(description, workspace_root, None).await?;

    println!("Created feature: {}", result.feature.full_name());
    println!("Spec file: {:?}", result.spec_path);

    Ok(())
}
```

### With Custom Config

```rust
use rstn_core::specify::{generate_spec, SpecifyConfig};

let config = SpecifyConfig {
    claude_timeout_secs: 60,     // Shorter timeout
    max_name_length: 30,         // Shorter names
    create_branch: false,        // Don't create git branch
    ..Default::default()
};

let result = generate_spec(description, workspace_root, Some(config)).await?;
```

## API Reference

### Main Function

```rust
/// Generate a new feature spec from a description
///
/// # Arguments
/// * `description` - User's feature description
/// * `workspace_root` - Path to workspace root (where specs/ directory is)
/// * `config` - Optional configuration (uses defaults if None)
///
/// # Returns
/// * `Ok(SpecResult)` - On success, with feature info and paths
/// * `Err(SpecifyError)` - On any failure (directories cleaned up)
pub async fn generate_spec(
    description: String,
    workspace_root: PathBuf,
    config: Option<SpecifyConfig>,
) -> Result<SpecResult, SpecifyError>;
```

### Result Types

```rust
pub struct SpecResult {
    pub feature: NewFeature,     // Feature info (number, name, title)
    pub spec_path: PathBuf,      // Path to spec.md
    pub spec_content: String,    // Generated spec content
    pub feature_dir: PathBuf,    // Path to feature directory
}

pub struct NewFeature {
    pub number: String,          // "052"
    pub name: String,            // "internalize-spec-generation"
    pub title: String,           // "Internalize Spec Generation"
    pub description: String,     // Original description
}
```

### Error Handling

```rust
use rstn_core::specify::{generate_spec, SpecifyError};

match generate_spec(desc, root, None).await {
    Ok(result) => {
        // Success - feature created
    }
    Err(SpecifyError::ClaudeNotFound) => {
        // Claude CLI not installed
        eprintln!("Install Claude: npm install -g @anthropic-ai/claude-code");
    }
    Err(SpecifyError::ClaudeTimeout(secs)) => {
        // Claude took too long
        eprintln!("Generation timed out after {}s", secs);
    }
    Err(SpecifyError::NumberConflict(num)) => {
        // Feature number already exists
        eprintln!("Feature {} already exists", num);
    }
    Err(e) => {
        // Other errors
        eprintln!("Failed to generate spec: {}", e);
    }
}
```

## Component Functions (Internal)

These are internal but may be useful for testing:

```rust
// Number allocation
pub fn allocate_feature_number(workspace_root: &Path) -> Result<String, SpecifyError>;

// Name generation
pub fn generate_feature_name(description: &str) -> String;
pub fn extract_title(description: &str) -> String;

// Directory setup
pub fn setup_feature_directory(
    workspace_root: &Path,
    feature: &NewFeature,
) -> Result<PathBuf, SpecifyError>;

// Catalog management
pub fn update_features_catalog(
    workspace_root: &Path,
    feature: &NewFeature,
) -> Result<(), SpecifyError>;
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generation() {
        assert_eq!(
            generate_feature_name("Add user authentication"),
            "user-authentication"
        );
        assert_eq!(
            generate_feature_name("I want to implement OAuth2 login"),
            "implement-oauth2-login"
        );
    }

    #[test]
    fn test_title_extraction() {
        assert_eq!(
            extract_title("Add user authentication with OAuth2"),
            "Add User Authentication with OAuth2"
        );
    }
}
```

### Integration Tests (with temp directory)

```rust
#[tokio::test]
async fn test_full_workflow() {
    let temp = tempfile::tempdir().unwrap();
    setup_test_workspace(&temp.path());

    let result = generate_spec(
        "Add test feature".to_string(),
        temp.path().to_owned(),
        None,
    ).await.unwrap();

    assert_eq!(result.feature.number, "001");
    assert!(result.spec_path.exists());
}
```

## Integration with TUI (Feature 051)

The TUI will call this API when the user submits a feature description:

```rust
// In app.rs
fn handle_specify_submit(&mut self, description: String) {
    let workspace_root = self.workspace_root.clone();
    let tx = self.event_tx.clone();

    tokio::spawn(async move {
        match rstn_core::specify::generate_spec(description, workspace_root, None).await {
            Ok(result) => {
                let _ = tx.send(Event::SpecGenerated {
                    number: result.feature.number,
                    name: result.feature.name,
                    spec_content: result.spec_content,
                });
            }
            Err(e) => {
                let _ = tx.send(Event::SpecGenerationFailed {
                    error: e.to_string(),
                });
            }
        }
    });
}
```

## Dependencies

All dependencies already exist in workspace:

- `tokio` - Async runtime, process spawning
- `serde`, `serde_json` - JSON serialization
- `thiserror` - Error types
- `tracing` - Logging

No new dependencies required.

## Migration from Shell Script

The Rust implementation replaces `.specify/scripts/bash/create-new-feature.sh`:

| Shell Script | Rust Module |
|--------------|-------------|
| `check_existing_branches()` | `number_allocator.rs` |
| `generate_branch_name()` | `name_generator.rs` |
| Directory creation | `directory_setup.rs` |
| Template copy | `directory_setup.rs` |
| git branch | (deferred to TUI) |

**Key Improvement**: The Rust implementation checks ALL existing specs globally, fixing the shell script bug that only checked same-name branches.
