//! Intelligent Context Engine for AI workflows.
//!
//! Automatically gathers, ranks, and formats the most relevant information
//! from the project state to send to the LLM.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

// ============================================================================
// Core Types
// ============================================================================

/// Context for a single file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileContext {
    /// Absolute path to the file.
    pub path: String,
    /// File content (may be truncated if too large).
    pub content: String,
    /// Optional cursor/focus line (1-indexed).
    pub cursor_line: Option<usize>,
}

/// Aggregated AI context from multiple sources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AIContext {
    /// Currently open/active files.
    pub open_files: Vec<FileContext>,
    /// Last terminal/task output.
    pub terminal_last_output: Option<String>,
    /// Git status summary (branch, changes).
    pub git_status: String,
    /// Active errors from Docker, tasks, etc.
    pub active_errors: Vec<String>,
    /// Directory tree structure.
    pub directory_tree: Option<String>,
    /// Git diff of unstaged changes.
    pub git_diff: Option<String>,
}

impl AIContext {
    /// Estimate token count (rough: ~4 chars per token).
    pub fn estimate_tokens(&self) -> usize {
        let mut chars = 0;
        for file in &self.open_files {
            chars += file.path.len() + file.content.len();
        }
        if let Some(ref output) = self.terminal_last_output {
            chars += output.len();
        }
        chars += self.git_status.len();
        for error in &self.active_errors {
            chars += error.len();
        }
        if let Some(ref tree) = self.directory_tree {
            chars += tree.len();
        }
        if let Some(ref diff) = self.git_diff {
            chars += diff.len();
        }
        chars / 4
    }

    /// Format context as a system prompt string.
    pub fn to_system_prompt(&self) -> String {
        let mut parts = Vec::new();

        // Git status (high priority)
        if !self.git_status.is_empty() {
            parts.push(format!("## Git Status\n```\n{}\n```", self.git_status));
        }

        // Git diff (high priority)
        if let Some(ref diff) = self.git_diff {
            if !diff.is_empty() {
                parts.push(format!("## Unstaged Changes\n```diff\n{}\n```", diff));
            }
        }

        // Active errors (high priority)
        if !self.active_errors.is_empty() {
            let errors = self.active_errors.join("\n");
            parts.push(format!("## Active Errors\n```\n{}\n```", errors));
        }

        // Open files
        for file in &self.open_files {
            let cursor_info = file
                .cursor_line
                .map(|l| format!(" (cursor at line {})", l))
                .unwrap_or_default();
            parts.push(format!(
                "## File: {}{}\n```\n{}\n```",
                file.path, cursor_info, file.content
            ));
        }

        // Terminal output
        if let Some(ref output) = self.terminal_last_output {
            if !output.is_empty() {
                parts.push(format!("## Last Terminal Output\n```\n{}\n```", output));
            }
        }

        // Directory tree (low priority)
        if let Some(ref tree) = self.directory_tree {
            parts.push(format!("## Directory Structure\n```\n{}\n```", tree));
        }

        if parts.is_empty() {
            "No project context available.".to_string()
        } else {
            format!("# Project Context\n\n{}", parts.join("\n\n"))
        }
    }
}

// ============================================================================
// Context Gatherer Trait
// ============================================================================

/// Trait for gathering context from a specific source.
pub trait ContextGatherer: Send + Sync {
    /// Gather context from this source.
    /// Returns the gathered data and its priority (higher = more important).
    fn gather(&self, project_path: &Path) -> GatheredContext;

    /// Name of this gatherer for debugging.
    fn name(&self) -> &'static str;
}

/// Result of gathering context from a source.
#[derive(Debug, Clone, Default)]
pub struct GatheredContext {
    /// Priority (1-10, higher = more important).
    pub priority: u8,
    /// Estimated token count.
    pub tokens: usize,
    /// The gathered content.
    pub content: ContextContent,
}

/// Types of gathered content.
#[derive(Debug, Clone, Default)]
pub enum ContextContent {
    #[default]
    Empty,
    GitStatus(String),
    GitDiff(String),
    Files(Vec<FileContext>),
    Errors(Vec<String>),
    DirectoryTree(String),
    TerminalOutput(String),
}

// ============================================================================
// Git Gatherer
// ============================================================================

/// Gatherer for git status and diff.
pub struct GitGatherer;

impl ContextGatherer for GitGatherer {
    fn name(&self) -> &'static str {
        "git"
    }

    fn gather(&self, project_path: &Path) -> GatheredContext {
        let status = get_git_status(project_path);
        let diff = get_git_diff(project_path);

        let combined = format!("{}\n\n{}", status, diff);
        let tokens = combined.len() / 4;

        GatheredContext {
            priority: 8, // High priority
            tokens,
            content: ContextContent::GitStatus(combined),
        }
    }
}

/// Get git status for a project.
fn get_git_status(project_path: &Path) -> String {
    let output = Command::new("git")
        .args(["status", "--short", "--branch"])
        .current_dir(project_path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => String::new(),
    }
}

/// Get git diff (unstaged changes).
fn get_git_diff(project_path: &Path) -> String {
    let output = Command::new("git")
        .args(["diff", "--stat"])
        .current_dir(project_path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let diff = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Limit diff size to prevent token explosion
            if diff.len() > 2000 {
                format!("{}...\n(truncated)", &diff[..2000])
            } else {
                diff
            }
        }
        _ => String::new(),
    }
}

// ============================================================================
// File Gatherer
// ============================================================================

/// Gatherer for active/open files.
pub struct FileGatherer {
    /// Paths to files to include.
    pub file_paths: Vec<String>,
    /// Maximum content size per file (in chars).
    pub max_file_size: usize,
}

impl Default for FileGatherer {
    fn default() -> Self {
        Self {
            file_paths: Vec::new(),
            max_file_size: 10000, // ~2500 tokens per file max
        }
    }
}

impl ContextGatherer for FileGatherer {
    fn name(&self) -> &'static str {
        "files"
    }

    fn gather(&self, _project_path: &Path) -> GatheredContext {
        let mut files = Vec::new();
        let mut total_tokens = 0;

        for path in &self.file_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let truncated = if content.len() > self.max_file_size {
                    format!(
                        "{}...\n(truncated, {} more chars)",
                        &content[..self.max_file_size],
                        content.len() - self.max_file_size
                    )
                } else {
                    content
                };

                total_tokens += (path.len() + truncated.len()) / 4;

                files.push(FileContext {
                    path: path.clone(),
                    content: truncated,
                    cursor_line: None,
                });
            }
        }

        GatheredContext {
            priority: 10, // Highest priority
            tokens: total_tokens,
            content: ContextContent::Files(files),
        }
    }
}

// ============================================================================
// Docker Gatherer
// ============================================================================

/// Gatherer for Docker errors and status.
#[derive(Default)]
pub struct DockerGatherer {
    /// Container logs to include (container_id -> log lines).
    pub error_logs: Vec<String>,
}

impl ContextGatherer for DockerGatherer {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn gather(&self, _project_path: &Path) -> GatheredContext {
        if self.error_logs.is_empty() {
            return GatheredContext::default();
        }

        let tokens = self.error_logs.iter().map(|s| s.len()).sum::<usize>() / 4;

        GatheredContext {
            priority: 7, // High priority for errors
            tokens,
            content: ContextContent::Errors(self.error_logs.clone()),
        }
    }
}

// ============================================================================
// Directory Tree Gatherer
// ============================================================================

/// Gatherer for directory structure.
pub struct DirectoryGatherer {
    /// Maximum depth to traverse.
    pub max_depth: usize,
}

impl Default for DirectoryGatherer {
    fn default() -> Self {
        Self { max_depth: 2 }
    }
}

impl ContextGatherer for DirectoryGatherer {
    fn name(&self) -> &'static str {
        "directory"
    }

    fn gather(&self, project_path: &Path) -> GatheredContext {
        let tree = build_directory_tree(project_path, self.max_depth);
        let tokens = tree.len() / 4;

        GatheredContext {
            priority: 3, // Low priority
            tokens,
            content: ContextContent::DirectoryTree(tree),
        }
    }
}

/// Build a directory tree string.
fn build_directory_tree(path: &Path, max_depth: usize) -> String {
    let mut result = String::new();
    build_tree_recursive(path, "", max_depth, 0, &mut result);
    result
}

fn build_tree_recursive(
    path: &Path,
    prefix: &str,
    max_depth: usize,
    current_depth: usize,
    result: &mut String,
) {
    if current_depth > max_depth {
        return;
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    if path.is_dir() {
        result.push_str(&format!("{}{}/\n", prefix, name));

        // Skip common non-essential directories
        let skip_dirs = [
            "node_modules",
            ".git",
            "target",
            "dist",
            "build",
            ".next",
            "__pycache__",
            ".venv",
            "venv",
        ];

        if skip_dirs.contains(&name.as_str()) {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries.iter().take(20) {
                // Limit entries per directory
                let child_prefix = format!("{}  ", prefix);
                build_tree_recursive(
                    &entry.path(),
                    &child_prefix,
                    max_depth,
                    current_depth + 1,
                    result,
                );
            }

            if entries.len() > 20 {
                result.push_str(&format!("{}  ... and {} more\n", prefix, entries.len() - 20));
            }
        }
    } else {
        result.push_str(&format!("{}{}\n", prefix, name));
    }
}

// ============================================================================
// Terminal Output Gatherer
// ============================================================================

/// Gatherer for last terminal/task output.
pub struct TerminalGatherer {
    /// Last output from task execution.
    pub last_output: Option<String>,
    /// Maximum output size.
    pub max_size: usize,
}

impl Default for TerminalGatherer {
    fn default() -> Self {
        Self {
            last_output: None,
            max_size: 2000,
        }
    }
}

impl ContextGatherer for TerminalGatherer {
    fn name(&self) -> &'static str {
        "terminal"
    }

    fn gather(&self, _project_path: &Path) -> GatheredContext {
        let output = match &self.last_output {
            Some(o) if !o.is_empty() => {
                if o.len() > self.max_size {
                    format!(
                        "...{}\n(showing last {} chars)",
                        &o[o.len() - self.max_size..],
                        self.max_size
                    )
                } else {
                    o.clone()
                }
            }
            _ => return GatheredContext::default(),
        };

        let tokens = output.len() / 4;

        GatheredContext {
            priority: 6, // Medium-high priority
            tokens,
            content: ContextContent::TerminalOutput(output),
        }
    }
}

// ============================================================================
// Context Engine (Orchestrator)
// ============================================================================

/// The main context engine that orchestrates gathering and budgeting.
pub struct ContextEngine {
    /// Token budget for the context.
    pub token_budget: usize,
    /// Registered gatherers.
    gatherers: Vec<Box<dyn ContextGatherer>>,
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new(20000) // Default 20k token budget
    }
}

impl ContextEngine {
    /// Create a new context engine with a token budget.
    pub fn new(token_budget: usize) -> Self {
        Self {
            token_budget,
            gatherers: Vec::new(),
        }
    }

    /// Add a gatherer to the engine.
    pub fn add_gatherer(&mut self, gatherer: Box<dyn ContextGatherer>) {
        self.gatherers.push(gatherer);
    }

    /// Build context from all gatherers within the token budget.
    pub fn build(&self, project_path: &Path) -> AIContext {
        // Gather from all sources
        let mut gathered: Vec<(GatheredContext, &'static str)> = self
            .gatherers
            .iter()
            .map(|g| (g.gather(project_path), g.name()))
            .collect();

        // Sort by priority (highest first)
        gathered.sort_by(|a, b| b.0.priority.cmp(&a.0.priority));

        // Build context within budget
        let mut context = AIContext::default();
        let mut remaining_budget = self.token_budget;

        for (gc, _name) in gathered {
            if gc.tokens > remaining_budget {
                // Skip if over budget (could implement partial inclusion later)
                continue;
            }

            remaining_budget = remaining_budget.saturating_sub(gc.tokens);

            match gc.content {
                ContextContent::Empty => {}
                ContextContent::GitStatus(status) => {
                    context.git_status = status;
                }
                ContextContent::GitDiff(diff) => {
                    context.git_diff = Some(diff);
                }
                ContextContent::Files(files) => {
                    context.open_files.extend(files);
                }
                ContextContent::Errors(errors) => {
                    context.active_errors.extend(errors);
                }
                ContextContent::DirectoryTree(tree) => {
                    context.directory_tree = Some(tree);
                }
                ContextContent::TerminalOutput(output) => {
                    context.terminal_last_output = Some(output);
                }
            }
        }

        context
    }

    /// Convenience method to build and format as system prompt.
    pub fn build_system_prompt(&self, project_path: &Path) -> String {
        self.build(project_path).to_system_prompt()
    }
}

/// Create a default context engine with standard gatherers.
pub fn create_default_engine(token_budget: usize) -> ContextEngine {
    let mut engine = ContextEngine::new(token_budget);
    engine.add_gatherer(Box::new(GitGatherer));
    engine.add_gatherer(Box::new(DirectoryGatherer::default()));
    engine
}

/// Build context for a project with optional additional data.
pub fn build_context(
    project_path: &Path,
    active_files: Vec<String>,
    task_output: Option<String>,
    docker_errors: Vec<String>,
    token_budget: usize,
) -> AIContext {
    let mut engine = ContextEngine::new(token_budget);

    // Add git gatherer
    engine.add_gatherer(Box::new(GitGatherer));

    // Add file gatherer if there are active files
    if !active_files.is_empty() {
        engine.add_gatherer(Box::new(FileGatherer {
            file_paths: active_files,
            ..Default::default()
        }));
    }

    // Add terminal gatherer if there's output
    if task_output.is_some() {
        engine.add_gatherer(Box::new(TerminalGatherer {
            last_output: task_output,
            ..Default::default()
        }));
    }

    // Add docker gatherer if there are errors
    if !docker_errors.is_empty() {
        engine.add_gatherer(Box::new(DockerGatherer {
            error_logs: docker_errors,
        }));
    }

    // Add directory gatherer (low priority, will be cut if over budget)
    engine.add_gatherer(Box::new(DirectoryGatherer::default()));

    engine.build(project_path)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_ai_context_estimate_tokens() {
        let context = AIContext {
            open_files: vec![FileContext {
                path: "/test/file.rs".to_string(),
                content: "fn main() {}".to_string(), // 12 chars
                cursor_line: Some(1),
            }],
            terminal_last_output: Some("output".to_string()), // 6 chars
            git_status: "main".to_string(),                   // 4 chars
            active_errors: vec!["error1".to_string()],        // 6 chars
            directory_tree: None,
            git_diff: None,
        };

        // Total: 14 + 12 + 6 + 4 + 6 = 42 chars / 4 = 10 tokens
        let tokens = context.estimate_tokens();
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn test_ai_context_to_system_prompt() {
        let context = AIContext {
            open_files: vec![],
            terminal_last_output: None,
            git_status: "## main".to_string(),
            active_errors: vec![],
            directory_tree: None,
            git_diff: None,
        };

        let prompt = context.to_system_prompt();
        assert!(prompt.contains("Git Status"));
        assert!(prompt.contains("## main"));
    }

    #[test]
    fn test_empty_context_prompt() {
        let context = AIContext::default();
        let prompt = context.to_system_prompt();
        assert_eq!(prompt, "No project context available.");
    }

    #[test]
    fn test_directory_gatherer() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

        let gatherer = DirectoryGatherer { max_depth: 2 };
        let result = gatherer.gather(dir.path());

        assert!(result.priority > 0);
        if let ContextContent::DirectoryTree(tree) = result.content {
            assert!(tree.contains("src/"));
            assert!(tree.contains("Cargo.toml"));
        } else {
            panic!("Expected DirectoryTree content");
        }
    }

    #[test]
    fn test_file_gatherer() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "fn test() {}").unwrap();

        let gatherer = FileGatherer {
            file_paths: vec![file_path.to_string_lossy().to_string()],
            max_file_size: 1000,
        };
        let result = gatherer.gather(dir.path());

        assert_eq!(result.priority, 10); // Highest priority
        if let ContextContent::Files(files) = result.content {
            assert_eq!(files.len(), 1);
            assert!(files[0].content.contains("fn test()"));
        } else {
            panic!("Expected Files content");
        }
    }

    #[test]
    fn test_file_gatherer_truncation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.txt");
        let large_content = "x".repeat(20000);
        fs::write(&file_path, &large_content).unwrap();

        let gatherer = FileGatherer {
            file_paths: vec![file_path.to_string_lossy().to_string()],
            max_file_size: 100,
        };
        let result = gatherer.gather(dir.path());

        if let ContextContent::Files(files) = result.content {
            assert!(files[0].content.len() < 200);
            assert!(files[0].content.contains("truncated"));
        } else {
            panic!("Expected Files content");
        }
    }

    #[test]
    fn test_context_engine_priority() {
        let dir = tempdir().unwrap();

        // Create a file
        let file_path = dir.path().join("test.rs");
        fs::write(&file_path, "code").unwrap();

        let mut engine = ContextEngine::new(1000);

        // Add gatherers
        engine.add_gatherer(Box::new(FileGatherer {
            file_paths: vec![file_path.to_string_lossy().to_string()],
            max_file_size: 100,
        }));
        engine.add_gatherer(Box::new(DirectoryGatherer { max_depth: 1 }));

        let context = engine.build(dir.path());

        // Files should be included (priority 10)
        assert!(!context.open_files.is_empty());
        // Directory tree should also be included if within budget
        assert!(context.directory_tree.is_some());
    }

    #[test]
    fn test_docker_gatherer() {
        let dir = tempdir().unwrap();
        let gatherer = DockerGatherer {
            error_logs: vec!["Error: connection refused".to_string()],
        };
        let result = gatherer.gather(dir.path());

        assert_eq!(result.priority, 7);
        if let ContextContent::Errors(errors) = result.content {
            assert_eq!(errors.len(), 1);
            assert!(errors[0].contains("connection refused"));
        } else {
            panic!("Expected Errors content");
        }
    }

    #[test]
    fn test_terminal_gatherer() {
        let dir = tempdir().unwrap();
        let gatherer = TerminalGatherer {
            last_output: Some("Build successful".to_string()),
            max_size: 1000,
        };
        let result = gatherer.gather(dir.path());

        assert_eq!(result.priority, 6);
        if let ContextContent::TerminalOutput(output) = result.content {
            assert!(output.contains("Build successful"));
        } else {
            panic!("Expected TerminalOutput content");
        }
    }

    #[test]
    fn test_build_context_helper() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let context = build_context(
            dir.path(),
            vec![file_path.to_string_lossy().to_string()],
            Some("test passed".to_string()),
            vec!["docker error".to_string()],
            10000,
        );

        assert!(!context.open_files.is_empty());
        assert!(context.terminal_last_output.is_some());
        assert!(!context.active_errors.is_empty());
    }
}
