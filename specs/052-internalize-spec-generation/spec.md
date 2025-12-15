# Feature 052: Internalize Spec Generation

## Overview

Move spec generation logic from bash shell scripts into Rust code within rstn. This eliminates the dependency on `.specify/scripts/bash/create-new-feature.sh` and provides direct, native integration with Claude Code CLI in headless mode.

## Problem Statement

Current spec generation architecture has several limitations:

1. **External dependency**: Relies on bash shell script that must be maintained separately
2. **Limited control**: Cannot easily customize generation behavior or handle edge cases
3. **Error handling**: Difficult to provide detailed error messages from shell script
4. **Testing**: Shell scripts harder to unit test than Rust code
5. **Portability**: Bash script may have platform-specific issues
6. **Integration**: Indirect communication between rstn and Claude Code CLI

## Dependencies

**Depends on:**
- Feature 051 (Interactive Specify Flow) - Must be implemented first to provide the UI

**Reason:**
- This feature replaces the backend logic that 051 calls
- 051 provides the UX, 052 provides the implementation

## User Stories

### As an rstn maintainer
- I want spec generation logic in Rust
- So that I can maintain a single codebase with consistent tooling

### As an rstn user
- I want faster spec generation
- So that I spend less time waiting for specs to be created

### As an rstn developer
- I want better error messages when spec generation fails
- So that I can understand and fix issues quickly

### As a contributor
- I want to test spec generation logic
- So that I can ensure reliability and catch regressions

## Requirements

### Functional Requirements

**FR-1: Feature Number Allocation**
- Read `specs/features.json` to determine next feature number
- Find highest existing number in `specs/` directory as fallback
- Validate number doesn't already exist
- Support 3-digit zero-padded format (e.g., "051")

**FR-2: Feature Name Generation**
- Convert description to kebab-case name
- Remove special characters (keep alphanumeric and hyphens)
- Limit length to 50 characters
- Handle Unicode characters appropriately

**FR-3: Directory Structure**
- Create `specs/{NNN}-{name}/` directory
- Set up subdirectories:
  - `checklists/` (if needed)
  - `contracts/` (if needed)
- Create placeholder files: `spec.md`, `plan.md`, `tasks.md`

**FR-4: Spec Generation via Claude Code**
- Call Claude Code CLI in headless mode
- Pass feature description as prompt
- Use spec template from `.specify/templates/spec-template.md`
- Capture generated spec content
- Write to `specs/{NNN}-{name}/spec.md`

**FR-5: Features Catalog Update**
- Update `specs/features.json` with new feature entry
- Include: number, name, title, status, phase
- Preserve existing entries and formatting
- Atomic write (temp file + rename for safety)

**FR-6: Error Handling**
- Validate Claude Code CLI is available
- Handle network/API failures gracefully
- Rollback on partial failure (clean up directories)
- Return detailed error information to UI

### Non-Functional Requirements

**NFR-1: Performance**
- Spec generation: <30 seconds (depends on Claude API)
- File operations: <100ms
- No blocking UI thread (use async/tokio)

**NFR-2: Reliability**
- Atomic operations (temp files for safety)
- Rollback on error (clean up partial state)
- Validate all operations succeed before committing
- Comprehensive error messages

**NFR-3: Testability**
- Unit testable components (number allocation, name generation)
- Integration tests with mock Claude CLI
- Error path testing
- No hidden dependencies on shell environment

**NFR-4: Maintainability**
- Clear separation of concerns
- Well-documented API
- Rust idioms and best practices
- Type-safe error handling

## Architecture

### Module Structure

**New module:** `crates/rstn-core/src/specify/mod.rs`
```rust
pub mod number_allocator;  // Feature number allocation
pub mod name_generator;     // Kebab-case name generation
pub mod directory_setup;    // Create directory structure
pub mod spec_generator;     // Claude Code integration
pub mod catalog_updater;    // Update features.json
pub mod workflow;           // Orchestrate the full flow
```

### Core Types

```rust
/// Feature information for new spec
#[derive(Debug, Clone)]
pub struct NewFeature {
    pub number: String,        // "051"
    pub name: String,          // "interactive-specify-flow"
    pub title: String,         // Human-readable title
    pub description: String,   // User's input description
}

/// Result of spec generation
#[derive(Debug)]
pub struct SpecResult {
    pub feature: NewFeature,
    pub spec_path: PathBuf,
    pub spec_content: String,
}

/// Errors that can occur during spec generation
#[derive(Debug, thiserror::Error)]
pub enum SpecifyError {
    #[error("Failed to allocate feature number: {0}")]
    NumberAllocation(String),

    #[error("Failed to create directory structure: {0}")]
    DirectorySetup(#[source] std::io::Error),

    #[error("Claude Code CLI not found or not executable")]
    ClaudeNotAvailable,

    #[error("Failed to generate spec via Claude: {0}")]
    SpecGeneration(String),

    #[error("Failed to update features catalog: {0}")]
    CatalogUpdate(#[source] std::io::Error),

    #[error("Rollback failed after error: {0}")]
    RollbackFailed(String),
}
```

### Main API

```rust
/// Generate a new feature spec from description
pub async fn generate_spec(
    description: String,
    workspace_root: PathBuf,
) -> Result<SpecResult, SpecifyError> {
    // 1. Allocate feature number
    let number = allocate_feature_number(&workspace_root)?;

    // 2. Generate feature name
    let name = generate_feature_name(&description);
    let title = extract_title(&description);

    // 3. Create feature struct
    let feature = NewFeature {
        number: number.clone(),
        name: name.clone(),
        title,
        description: description.clone(),
    };

    // 4. Create directory structure
    let feature_dir = setup_feature_directory(&workspace_root, &feature)?;

    // 5. Generate spec via Claude Code
    match generate_spec_content(&feature).await {
        Ok(spec_content) => {
            // 6. Write spec file
            write_spec_file(&feature_dir, &spec_content)?;

            // 7. Update features catalog
            update_features_catalog(&workspace_root, &feature)?;

            Ok(SpecResult {
                feature,
                spec_path: feature_dir.join("spec.md"),
                spec_content,
            })
        }
        Err(e) => {
            // Rollback: Remove created directories
            rollback_directory(&feature_dir)?;
            Err(e)
        }
    }
}
```

### Component Details

**1. Number Allocator**
```rust
pub fn allocate_feature_number(workspace_root: &Path) -> Result<String, SpecifyError> {
    // Read specs/features.json
    // Find highest number in use
    // Increment by 1
    // Format as zero-padded 3-digit string
}
```

**2. Name Generator**
```rust
pub fn generate_feature_name(description: &str) -> String {
    // Take first sentence or first 50 chars
    // Convert to lowercase
    // Replace spaces with hyphens
    // Remove special characters
    // Trim to max 50 chars
}

pub fn extract_title(description: &str) -> String {
    // Take first line or first sentence
    // Capitalize appropriately
    // Max 100 chars
}
```

**3. Directory Setup**
```rust
pub fn setup_feature_directory(
    workspace_root: &Path,
    feature: &NewFeature,
) -> Result<PathBuf, SpecifyError> {
    // Create specs/{NNN}-{name}/
    // Create placeholder spec.md with frontmatter
    // Return path to feature directory
}
```

**4. Spec Generator (Claude Integration)**
```rust
pub async fn generate_spec_content(
    feature: &NewFeature,
) -> Result<String, SpecifyError> {
    // 1. Check Claude Code CLI is available
    check_claude_cli_available()?;

    // 2. Load spec template
    let template = load_spec_template()?;

    // 3. Build prompt
    let prompt = format!(
        "Generate a feature specification for: {}\n\n\
         Use this template:\n{}\n\n\
         Feature description: {}",
        feature.title,
        template,
        feature.description
    );

    // 4. Call Claude Code CLI
    let output = tokio::process::Command::new("claude")
        .arg("--headless")
        .arg("--prompt")
        .arg(&prompt)
        .output()
        .await?;

    // 5. Parse response
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(SpecifyError::SpecGeneration(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
    }
}

fn check_claude_cli_available() -> Result<(), SpecifyError> {
    // Run `which claude` or `claude --version`
    // Return error if not found
}
```

**5. Catalog Updater**
```rust
#[derive(Debug, Serialize, Deserialize)]
struct FeatureEntry {
    number: String,
    name: String,
    title: String,
    status: String,
    phase: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FeaturesCatalog {
    project: String,
    description: String,
    features: Vec<FeatureEntry>,
}

pub fn update_features_catalog(
    workspace_root: &Path,
    feature: &NewFeature,
) -> Result<(), SpecifyError> {
    let catalog_path = workspace_root.join("specs/features.json");

    // 1. Read existing catalog
    let mut catalog: FeaturesCatalog = serde_json::from_str(
        &fs::read_to_string(&catalog_path)?
    )?;

    // 2. Add new entry
    catalog.features.push(FeatureEntry {
        number: feature.number.clone(),
        name: feature.name.clone(),
        title: feature.title.clone(),
        status: "draft".to_string(),
        phase: 1,
    });

    // 3. Write atomically (temp file + rename)
    let temp_path = catalog_path.with_extension("json.tmp");
    fs::write(&temp_path, serde_json::to_string_pretty(&catalog)?)?;
    fs::rename(&temp_path, &catalog_path)?;

    Ok(())
}
```

**6. Rollback**
```rust
fn rollback_directory(feature_dir: &Path) -> Result<(), SpecifyError> {
    if feature_dir.exists() {
        fs::remove_dir_all(feature_dir)
            .map_err(|e| SpecifyError::RollbackFailed(e.to_string()))?;
    }
    Ok(())
}
```

## Integration with Feature 051

**In `crates/rstn/src/tui/app.rs`:**
```rust
impl App {
    fn handle_view_action(&mut self, action: ViewAction) {
        match action {
            // ... existing actions ...

            ViewAction::GenerateSpec { description } => {
                let workspace_root = self.workspace_root.clone();

                // Spawn async task
                let tx = self.event_tx.clone();
                tokio::spawn(async move {
                    match rstn_core::specify::generate_spec(description, workspace_root).await {
                        Ok(result) => {
                            let _ = tx.send(Event::SpecifyGenerationCompleted {
                                spec: result.spec_content,
                                number: result.feature.number,
                                name: result.feature.name,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Event::SpecifyGenerationFailed {
                                error: e.to_string(),
                            });
                        }
                    }
                });
            }

            // ... other actions ...
        }
    }
}
```

## Testing Strategy

### Unit Tests

**Number Allocator:**
- Empty specs directory → returns "001"
- Existing features → returns next number
- Gaps in numbering → uses highest + 1
- Invalid features.json → falls back to directory scan

**Name Generator:**
- Simple description → correct kebab-case
- Special characters → removed appropriately
- Long description → truncated to 50 chars
- Unicode characters → handled correctly

**Directory Setup:**
- Creates directory structure
- Handles existing directory (error)
- Creates placeholder files
- Proper permissions

**Catalog Updater:**
- Adds entry to empty catalog
- Appends to existing catalog
- Preserves existing entries
- Atomic write (temp file)

### Integration Tests

**Full Workflow (with mock Claude CLI):**
```rust
#[tokio::test]
async fn test_full_spec_generation_workflow() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_workspace(&temp_dir);

    let result = generate_spec(
        "Add user authentication".to_string(),
        temp_dir.path().to_owned(),
    ).await.unwrap();

    assert_eq!(result.feature.number, "001");
    assert_eq!(result.feature.name, "add-user-authentication");
    assert!(result.spec_path.exists());
    assert!(result.spec_content.contains("Feature"));

    // Verify catalog updated
    let catalog = read_features_catalog(&temp_dir.path());
    assert_eq!(catalog.features.len(), 1);
}
```

**Error Handling:**
- Claude CLI not available
- Permission denied on directory creation
- Disk full during write
- Invalid JSON in features.json
- Rollback after partial failure

### Manual Tests
- Generate spec with various descriptions
- Test on different platforms (macOS, Linux)
- Test with different Claude Code CLI versions
- Test rollback behavior on errors

## Migration Plan

### Phase 1: Implement Core (feature 052)
- Implement all modules in rstn-core
- Write comprehensive tests
- Keep shell script as fallback

### Phase 2: Integration (after 051)
- Replace shell script call in 051 with Rust implementation
- Test both paths work
- Add feature flag to choose implementation

### Phase 3: Deprecation
- Default to Rust implementation
- Remove feature flag
- Archive shell script (keep for reference)
- Update documentation

## Success Metrics

**Performance:**
- Spec generation time ≤ current shell script time
- File operations < 100ms
- Memory usage < 10MB per generation

**Reliability:**
- 100% success rate when Claude CLI succeeds
- 100% rollback success rate on errors
- Zero data corruption (atomic writes)

**Maintainability:**
- >80% test coverage
- All public APIs documented
- Zero clippy warnings

## Dependencies

**New:**
- `serde_json` - Already in Cargo.toml
- `tokio::process` - Already using tokio
- `thiserror` - Already in Cargo.toml

**No new external dependencies required**

## Future Enhancements

- Support multiple spec templates
- Streaming output during generation
- Batch spec generation
- Spec validation before save
- Integration with GitHub issues API
- Custom naming strategies

## Notes

- This feature eliminates dependency on bash shell scripts
- Provides foundation for future spec-kit enhancements
- Rust implementation is more testable and maintainable
- Pattern can be applied to other shell script migrations (plan, tasks, etc.)
- Feature 051 must be implemented first to provide the UI layer
