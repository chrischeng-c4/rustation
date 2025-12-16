# Data Model: Internalize Spec Generation

**Feature**: 052-internalize-spec-generation
**Date**: 2025-12-16

## Core Types

### NewFeature

Represents a feature being created.

```rust
/// Information about a new feature being created
#[derive(Debug, Clone)]
pub struct NewFeature {
    /// Zero-padded 3-digit number (e.g., "052")
    pub number: String,

    /// Kebab-case short name (e.g., "internalize-spec-generation")
    pub name: String,

    /// Human-readable title (e.g., "Internalize Spec Generation")
    pub title: String,

    /// Original user description
    pub description: String,
}

impl NewFeature {
    /// Returns the full branch/directory name (e.g., "052-internalize-spec-generation")
    pub fn full_name(&self) -> String {
        format!("{}-{}", self.number, self.name)
    }
}
```

### SpecResult

Result of successful spec generation.

```rust
/// Result of successful spec generation
#[derive(Debug)]
pub struct SpecResult {
    /// The created feature info
    pub feature: NewFeature,

    /// Path to the created spec file
    pub spec_path: PathBuf,

    /// Generated spec content
    pub spec_content: String,

    /// Path to the feature directory
    pub feature_dir: PathBuf,
}
```

### SpecifyError

All errors that can occur during spec generation.

```rust
/// Errors that can occur during spec generation
#[derive(Debug, thiserror::Error)]
pub enum SpecifyError {
    #[error("Failed to allocate feature number: {0}")]
    NumberAllocation(String),

    #[error("Feature number {0} already exists")]
    NumberConflict(String),

    #[error("Failed to generate feature name from description")]
    NameGeneration,

    #[error("Failed to create directory structure: {0}")]
    DirectorySetup(#[source] std::io::Error),

    #[error("Directory already exists: {0}")]
    DirectoryExists(PathBuf),

    #[error("Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code")]
    ClaudeNotFound,

    #[error("Claude Code CLI execution failed: {0}")]
    ClaudeExecution(String),

    #[error("Claude Code CLI timed out after {0} seconds")]
    ClaudeTimeout(u64),

    #[error("Failed to read features catalog: {0}")]
    CatalogRead(#[source] std::io::Error),

    #[error("Failed to parse features catalog: {0}")]
    CatalogParse(#[source] serde_json::Error),

    #[error("Failed to write features catalog: {0}")]
    CatalogWrite(#[source] std::io::Error),

    #[error("Failed to read spec template: {0}")]
    TemplateRead(#[source] std::io::Error),

    #[error("Rollback failed while cleaning up: {0}")]
    RollbackFailed(String),

    #[error("Workspace root not found")]
    WorkspaceNotFound,
}
```

## Catalog Types

### FeaturesCatalog

The `specs/features.json` file structure.

```rust
/// The features.json catalog structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesCatalog {
    /// Project name
    pub project: String,

    /// Project description
    pub description: String,

    /// List of features
    pub features: Vec<FeatureEntry>,
}

/// A single feature entry in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEntry {
    /// Feature ID (e.g., "052")
    pub id: String,

    /// Feature short name (e.g., "internalize-spec-generation")
    pub name: String,

    /// Feature description
    pub description: String,

    /// Status: "draft", "in-progress", "complete"
    pub status: String,

    /// Phase number (typically 1)
    pub phase: u32,
}

impl FeaturesCatalog {
    /// Find the highest feature number in the catalog
    pub fn max_number(&self) -> u32 {
        self.features
            .iter()
            .filter_map(|f| f.id.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
    }

    /// Check if a feature number already exists
    pub fn has_number(&self, num: &str) -> bool {
        self.features.iter().any(|f| f.id == num)
    }
}
```

## Configuration

### SpecifyConfig

Optional configuration for spec generation.

```rust
/// Configuration for spec generation (all fields have defaults)
#[derive(Debug, Clone)]
pub struct SpecifyConfig {
    /// Timeout for Claude CLI in seconds (default: 120)
    pub claude_timeout_secs: u64,

    /// Maximum name length (default: 50)
    pub max_name_length: usize,

    /// Whether to create git branch (default: true)
    pub create_branch: bool,

    /// Custom spec template path (default: .specify/templates/spec-template.md)
    pub template_path: Option<PathBuf>,
}

impl Default for SpecifyConfig {
    fn default() -> Self {
        Self {
            claude_timeout_secs: 120,
            max_name_length: 50,
            create_branch: true,
            template_path: None,
        }
    }
}
```

## Internal Types

### NumberAllocationResult

Internal result from number allocation.

```rust
/// Result of number allocation
struct NumberAllocationResult {
    /// The allocated number as u32
    number: u32,

    /// Formatted as zero-padded string (e.g., "052")
    formatted: String,
}
```

### NameGenerationResult

Internal result from name generation.

```rust
/// Result of name generation
struct NameGenerationResult {
    /// Kebab-case name (e.g., "internalize-spec-generation")
    name: String,

    /// Human-readable title (e.g., "Internalize Spec Generation")
    title: String,
}
```

## File System Layout

### Specs Directory Structure

```text
specs/
├── features.json              # Catalog file
├── 001-rush-mvp/
│   ├── spec.md
│   ├── plan.md
│   └── tasks.md
├── 052-internalize-spec-generation/
│   ├── spec.md               # Created by this feature
│   ├── plan.md               # Created later by /speckit.plan
│   └── tasks.md              # Created later by /speckit.tasks
└── ...
```

### Template Location

```text
.specify/
└── templates/
    └── spec-template.md      # Spec template used for new features
```

## State Transitions

### Feature Creation Flow

```
[Start]
    │
    ▼
[Allocate Number]
    │ number: String
    ▼
[Generate Name]
    │ name: String, title: String
    ▼
[Create NewFeature]
    │ NewFeature { number, name, title, description }
    ▼
[Setup Directory]
    │ feature_dir: PathBuf
    ▼
[Generate Spec via Claude] ──Error──► [Rollback: Delete Directory]
    │ spec_content: String                      │
    ▼                                           ▼
[Write Spec File]                          [Return Error]
    │
    ▼
[Update Catalog]
    │
    ▼
[Return SpecResult]
```

## Validation Rules

### Number Validation
- Must be numeric (parseable as u32)
- Must not already exist in catalog
- Must not already exist as directory
- Format: 3 digits, zero-padded (001-999)

### Name Validation
- Must be non-empty after processing
- Max 50 characters
- Only lowercase letters, numbers, hyphens
- No leading/trailing hyphens
- No consecutive hyphens

### Description Validation
- Must be non-empty
- No minimum length (even single word is valid)
