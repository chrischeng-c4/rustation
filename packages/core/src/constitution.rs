//! Constitution system for modular coding guidelines.
//!
//! Supports the KB-First architecture with `.rstn/constitutions/` directory
//! containing multiple rule files with frontmatter metadata.

use std::collections::HashSet;
use std::path::Path;

/// Default global constitution template
pub const GLOBAL_TEMPLATE: &str = r#"---
name: "Global Rules"
type: global
priority: 100
required: true
token_estimate: 500
---

# Global Development Rules

> Project-wide principles that apply to all code.

## Code Style

- Follow language-specific conventions
- Maximum line length: 100 characters
- Use meaningful variable and function names
- Add comments for complex logic

## Testing Requirements

- All new features must include unit tests
- Critical paths require integration tests
- Tests must pass before merging

## Pull Request Conventions

- PR title should be clear and descriptive
- Include a summary of changes
- Keep PRs focused (<500 lines when possible)

## Documentation

- Update README for user-facing changes
- Add inline documentation for public APIs

## Security

- Never commit secrets or API keys
- Use environment variables for sensitive config
- Validate all user inputs
"#;

/// Rust language template
pub const RUST_TEMPLATE: &str = r#"---
name: "Rust Conventions"
type: language
language: rust
applies_to:
  - "**/*.rs"
tags:
  - backend
  - systems
priority: 50
required: false
token_estimate: 600
---

# Rust Development Rules

## Code Style

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Prefer `&str` over `String` for function parameters when possible
- Use `#[derive(...)]` for common traits

## Error Handling

- Use `thiserror` for library errors
- Use `anyhow` for application errors
- Avoid `unwrap()` in production code
- Use `?` operator for error propagation

## Testing

- Use `#[cfg(test)]` module for unit tests
- Use `#[tokio::test]` for async tests
- Mock external dependencies

## Performance

- Prefer iterators over loops where appropriate
- Use `&[T]` instead of `Vec<T>` for read-only access
- Consider `Cow<'_, T>` for flexible ownership
"#;

/// TypeScript language template
pub const TYPESCRIPT_TEMPLATE: &str = r#"---
name: "TypeScript Conventions"
type: language
language: typescript
applies_to:
  - "**/*.ts"
  - "**/*.tsx"
tags:
  - frontend
  - backend
priority: 50
required: false
token_estimate: 500
---

# TypeScript Development Rules

## Code Style

- Use `camelCase` for variables and functions
- Use `PascalCase` for types, interfaces, and classes
- Prefer `const` over `let`
- Use strict mode (`"strict": true`)

## Type Safety

- Avoid `any` type - use `unknown` if type is truly unknown
- Use explicit return types for public functions
- Prefer interfaces over type aliases for object shapes
- Use discriminated unions for state management

## Async Patterns

- Use `async/await` over raw Promises
- Handle errors with try/catch
- Use `Promise.all` for parallel operations

## React Patterns (if applicable)

- Use functional components with hooks
- Prefer named exports for components
- Keep components focused and small
"#;

/// Python language template
pub const PYTHON_TEMPLATE: &str = r#"---
name: "Python Conventions"
type: language
language: python
applies_to:
  - "**/*.py"
tags:
  - backend
  - scripts
  - ml
priority: 50
required: false
token_estimate: 500
---

# Python Development Rules

## Code Style

- Follow PEP 8 style guide
- Use `snake_case` for functions and variables
- Use `PascalCase` for classes
- Maximum line length: 100 characters

## Type Hints

- Use type hints for function signatures
- Use `from __future__ import annotations` for forward references
- Use `typing` module for complex types

## Testing

- Use `pytest` for testing
- Use fixtures for test setup
- Aim for high coverage on critical paths

## Virtual Environments

- Always use virtual environments
- Pin dependencies in `requirements.txt` or `pyproject.toml`
- Use `uv` or `pip-tools` for dependency management
"#;

/// React framework template
pub const REACT_TEMPLATE: &str = r#"---
name: "React Patterns"
type: language
language: typescript
applies_to:
  - "**/*.tsx"
  - "**/*.jsx"
tags:
  - frontend
  - ui
priority: 60
required: false
token_estimate: 500
---

# React Development Rules

## Component Patterns

- Use functional components with hooks
- Keep components small and focused
- Extract reusable logic into custom hooks
- Use composition over inheritance

## State Management

- Lift state up only when necessary
- Use context sparingly for global state
- Prefer local state when possible
- Use `useMemo` and `useCallback` for expensive operations

## Props

- Use TypeScript interfaces for prop types
- Destructure props in function signature
- Use default props with destructuring defaults

## Performance

- Memoize expensive components with `React.memo`
- Use virtualization for long lists
- Lazy load routes and heavy components
"#;

/// Detected languages in a project
#[derive(Debug, Default)]
pub struct DetectedLanguages {
    pub has_rust: bool,
    pub has_typescript: bool,
    pub has_python: bool,
    pub has_react: bool,
}

/// Check if a directory should be skipped during scanning
fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.')
        || name == "node_modules"
        || name == "target"
        || name == "dist"
        || name == "build"
        || name == "__pycache__"
        || name == "venv"
}

/// Scan a directory for language-specific files
pub fn detect_languages(project_path: &Path) -> DetectedLanguages {
    let mut result = DetectedLanguages::default();
    let mut extensions_found: HashSet<String> = HashSet::new();

    // Walk directory (limited depth to avoid performance issues)
    let walker = walkdir::WalkDir::new(project_path)
        .max_depth(5)
        .follow_links(true) // Follow symlinks for temp dirs
        .into_iter()
        .filter_entry(|e| {
            // Always allow root entry and files
            if e.depth() == 0 || e.file_type().is_file() {
                return true;
            }
            // Skip hidden and non-source directories
            let name = e.file_name().to_string_lossy();
            !should_skip_dir(&name)
        });

    for entry in walker.filter_map(|e| e.ok()) {
        // Collect file extensions
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                extensions_found.insert(ext.to_string_lossy().to_lowercase());
            }
        }
    }

    // Detect languages based on extensions
    result.has_rust = extensions_found.contains("rs");
    result.has_typescript = extensions_found.contains("ts") || extensions_found.contains("tsx");
    result.has_python = extensions_found.contains("py");
    result.has_react = extensions_found.contains("tsx") || extensions_found.contains("jsx");

    result
}

/// Check if constitution exists (modular or legacy)
pub fn constitution_exists(project_path: &Path) -> bool {
    let rstn_dir = project_path.join(".rstn");

    // Check for modular constitution (new system)
    let constitutions_dir = rstn_dir.join("constitutions");
    if constitutions_dir.exists() && constitutions_dir.is_dir() {
        // Check if there's at least one .md file
        if let Ok(entries) = std::fs::read_dir(&constitutions_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" {
                        return true;
                    }
                }
            }
        }
    }

    // Fallback: check for legacy constitution.md
    let legacy_file = rstn_dir.join("constitution.md");
    legacy_file.exists()
}

/// Create modular constitution files
pub async fn create_modular_constitution(project_path: &Path) -> Result<(), String> {
    let rstn_dir = project_path.join(".rstn");
    let constitutions_dir = rstn_dir.join("constitutions");

    // Create directories
    tokio::fs::create_dir_all(&constitutions_dir)
        .await
        .map_err(|e| format!("Failed to create constitutions directory: {}", e))?;

    // Detect languages
    let languages = detect_languages(project_path);

    // Always create global.md
    let global_path = constitutions_dir.join("global.md");
    tokio::fs::write(&global_path, GLOBAL_TEMPLATE)
        .await
        .map_err(|e| format!("Failed to write global.md: {}", e))?;

    // Create language-specific files based on detection
    if languages.has_rust {
        let rust_path = constitutions_dir.join("rust.md");
        tokio::fs::write(&rust_path, RUST_TEMPLATE)
            .await
            .map_err(|e| format!("Failed to write rust.md: {}", e))?;
    }

    if languages.has_typescript {
        let ts_path = constitutions_dir.join("typescript.md");
        tokio::fs::write(&ts_path, TYPESCRIPT_TEMPLATE)
            .await
            .map_err(|e| format!("Failed to write typescript.md: {}", e))?;
    }

    if languages.has_react {
        let react_path = constitutions_dir.join("react.md");
        tokio::fs::write(&react_path, REACT_TEMPLATE)
            .await
            .map_err(|e| format!("Failed to write react.md: {}", e))?;
    }

    if languages.has_python {
        let python_path = constitutions_dir.join("python.md");
        tokio::fs::write(&python_path, PYTHON_TEMPLATE)
            .await
            .map_err(|e| format!("Failed to write python.md: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_languages_rust() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        let detected = detect_languages(temp_dir.path());
        assert!(detected.has_rust);
        assert!(!detected.has_typescript);
        assert!(!detected.has_python);
    }

    #[test]
    fn test_detect_languages_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("index.ts"), "export {}").unwrap();
        std::fs::write(src_dir.join("App.tsx"), "export default App").unwrap();

        let detected = detect_languages(temp_dir.path());
        assert!(!detected.has_rust);
        assert!(detected.has_typescript);
        assert!(detected.has_react);
        assert!(!detected.has_python);
    }

    #[test]
    fn test_detect_languages_python() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("main.py"), "print('hello')").unwrap();

        let detected = detect_languages(temp_dir.path());
        assert!(!detected.has_rust);
        assert!(!detected.has_typescript);
        assert!(detected.has_python);
    }

    #[test]
    fn test_detect_languages_mixed() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn foo() {}").unwrap();
        std::fs::write(src_dir.join("index.ts"), "export {}").unwrap();
        std::fs::write(temp_dir.path().join("script.py"), "pass").unwrap();

        let detected = detect_languages(temp_dir.path());
        assert!(detected.has_rust);
        assert!(detected.has_typescript);
        assert!(detected.has_python);
    }

    #[test]
    fn test_constitution_exists_none() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!constitution_exists(temp_dir.path()));
    }

    #[test]
    fn test_constitution_exists_legacy() {
        let temp_dir = TempDir::new().unwrap();
        let rstn_dir = temp_dir.path().join(".rstn");
        std::fs::create_dir_all(&rstn_dir).unwrap();
        std::fs::write(rstn_dir.join("constitution.md"), "# Constitution").unwrap();

        assert!(constitution_exists(temp_dir.path()));
    }

    #[test]
    fn test_constitution_exists_modular() {
        let temp_dir = TempDir::new().unwrap();
        let constitutions_dir = temp_dir.path().join(".rstn").join("constitutions");
        std::fs::create_dir_all(&constitutions_dir).unwrap();
        std::fs::write(constitutions_dir.join("global.md"), GLOBAL_TEMPLATE).unwrap();

        assert!(constitution_exists(temp_dir.path()));
    }

    #[tokio::test]
    async fn test_create_modular_constitution() {
        let temp_dir = TempDir::new().unwrap();

        // Create some source files for detection
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn foo() {}").unwrap();
        std::fs::write(src_dir.join("index.ts"), "export {}").unwrap();

        // Create constitution
        create_modular_constitution(temp_dir.path()).await.unwrap();

        // Verify files created
        let constitutions_dir = temp_dir.path().join(".rstn").join("constitutions");
        assert!(constitutions_dir.join("global.md").exists());
        assert!(constitutions_dir.join("rust.md").exists());
        assert!(constitutions_dir.join("typescript.md").exists());
        assert!(!constitutions_dir.join("python.md").exists()); // No .py files
    }
}
