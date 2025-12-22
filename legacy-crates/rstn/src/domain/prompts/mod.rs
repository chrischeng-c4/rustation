//! Prompt Content Manager for spec-kit commands
//!
//! Provides centralized management of prompts used by spec-kit commands.
//! Prompts are resolved in order: project override → user override → built-in default.

mod builtin;

use std::path::PathBuf;
use std::{fs, io};

use crate::domain::paths;

/// Phase identifiers for spec-kit commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecPhase {
    Specify,
    Clarify,
    Plan,
    Tasks,
    Implement,
    Analyze,
    Checklist,
    Review,
}

impl SpecPhase {
    /// Get the filename for this phase's prompt
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Specify => "specify.md",
            Self::Clarify => "clarify.md",
            Self::Plan => "plan.md",
            Self::Tasks => "tasks.md",
            Self::Implement => "implement.md",
            Self::Analyze => "analyze.md",
            Self::Checklist => "checklist.md",
            Self::Review => "review.md",
        }
    }

    /// Get the command name for this phase
    pub fn command(&self) -> &'static str {
        match self {
            Self::Specify => "/speckit.specify",
            Self::Clarify => "/speckit.clarify",
            Self::Plan => "/speckit.plan",
            Self::Tasks => "/speckit.tasks",
            Self::Implement => "/speckit.implement",
            Self::Analyze => "/speckit.analyze",
            Self::Checklist => "/speckit.checklist",
            Self::Review => "/speckit.review",
        }
    }

    /// Parse phase from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "specify" => Some(Self::Specify),
            "clarify" => Some(Self::Clarify),
            "plan" => Some(Self::Plan),
            "tasks" => Some(Self::Tasks),
            "implement" => Some(Self::Implement),
            "analyze" => Some(Self::Analyze),
            "checklist" => Some(Self::Checklist),
            "review" => Some(Self::Review),
            _ => None,
        }
    }
}

/// Manages prompt content resolution and loading
#[derive(Debug, Clone)]
pub struct PromptManager {
    /// Project root directory (for project-level overrides)
    project_root: Option<PathBuf>,
    /// User config directory
    user_dir: PathBuf,
}

impl PromptManager {
    /// Create a new PromptManager
    ///
    /// # Arguments
    /// * `project_root` - Optional project root for project-level prompt overrides
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let user_dir = paths::config_dir()
            .map(|p| p.join("prompts"))
            .unwrap_or_else(|_| {
                // Fall back to a reasonable default if config_dir fails
                dirs::home_dir()
                    .map(|h| h.join(".config").join("rustation").join("prompts"))
                    .unwrap_or_else(|| PathBuf::from("/tmp/rstn-prompts"))
            });
        Self {
            project_root,
            user_dir,
        }
    }

    /// Get prompt content for a spec-kit phase
    ///
    /// Resolution order:
    /// 1. Project override: `{project_root}/.rstn/prompts/{phase}.md`
    /// 2. User override: `~/.config/rustation/prompts/{phase}.md`
    /// 3. Built-in default: Embedded in binary
    pub fn get_prompt(&self, phase: SpecPhase) -> String {
        let filename = phase.filename();

        // 1. Check project override
        if let Some(ref project_root) = self.project_root {
            let project_path = project_root.join(".rstn").join("prompts").join(filename);
            if let Ok(content) = fs::read_to_string(&project_path) {
                return content;
            }
        }

        // 2. Check user override
        let user_path = self.user_dir.join(filename);
        if let Ok(content) = fs::read_to_string(&user_path) {
            return content;
        }

        // 3. Fall back to built-in default
        builtin::get_builtin_prompt(phase).to_string()
    }

    /// Get the resolution source for a prompt (for debugging/info)
    pub fn get_prompt_source(&self, phase: SpecPhase) -> PromptSource {
        let filename = phase.filename();

        // Check project override
        if let Some(ref project_root) = self.project_root {
            let project_path = project_root.join(".rstn").join("prompts").join(filename);
            if project_path.exists() {
                return PromptSource::Project(project_path);
            }
        }

        // Check user override
        let user_path = self.user_dir.join(filename);
        if user_path.exists() {
            return PromptSource::User(user_path);
        }

        // Built-in
        PromptSource::Builtin
    }

    /// Write prompt content to a temporary file for use with --system-prompt-file
    ///
    /// Returns the path to the temporary file
    pub fn write_temp_prompt(&self, content: &str) -> io::Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join("rstn-prompts");
        fs::create_dir_all(&temp_dir)?;

        let temp_file = temp_dir.join(format!("prompt-{}.md", std::process::id()));
        fs::write(&temp_file, content)?;

        Ok(temp_file)
    }

    /// Get prompt for a phase and write to temp file
    ///
    /// Convenience method combining get_prompt and write_temp_prompt
    pub fn prepare_prompt_file(&self, phase: SpecPhase) -> io::Result<PathBuf> {
        let content = self.get_prompt(phase);
        self.write_temp_prompt(&content)
    }

    /// List all available prompt overrides
    pub fn list_overrides(&self) -> Vec<(SpecPhase, PromptSource)> {
        let phases = [
            SpecPhase::Specify,
            SpecPhase::Clarify,
            SpecPhase::Plan,
            SpecPhase::Tasks,
            SpecPhase::Implement,
            SpecPhase::Analyze,
            SpecPhase::Checklist,
            SpecPhase::Review,
        ];

        phases
            .iter()
            .map(|&phase| (phase, self.get_prompt_source(phase)))
            .collect()
    }
}

/// Source of a prompt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptSource {
    /// Project-level override (.rstn/prompts/)
    Project(PathBuf),
    /// User-level override (~/.config/rustation/prompts/)
    User(PathBuf),
    /// Built-in default (embedded in binary)
    Builtin,
}

impl std::fmt::Display for PromptSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Project(path) => write!(f, "project: {}", path.display()),
            Self::User(path) => write!(f, "user: {}", path.display()),
            Self::Builtin => write!(f, "built-in"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_filename() {
        assert_eq!(SpecPhase::Specify.filename(), "specify.md");
        assert_eq!(SpecPhase::Plan.filename(), "plan.md");
        assert_eq!(SpecPhase::Tasks.filename(), "tasks.md");
    }

    #[test]
    fn test_phase_from_str() {
        assert_eq!(SpecPhase::from_str("specify"), Some(SpecPhase::Specify));
        assert_eq!(SpecPhase::from_str("PLAN"), Some(SpecPhase::Plan));
        assert_eq!(SpecPhase::from_str("invalid"), None);
    }

    #[test]
    fn test_builtin_prompts_exist() {
        let manager = PromptManager::new(None);

        // All phases should have built-in prompts
        for phase in [
            SpecPhase::Specify,
            SpecPhase::Clarify,
            SpecPhase::Plan,
            SpecPhase::Tasks,
            SpecPhase::Implement,
            SpecPhase::Analyze,
            SpecPhase::Checklist,
            SpecPhase::Review,
        ] {
            let prompt = manager.get_prompt(phase);
            assert!(
                !prompt.is_empty(),
                "Prompt for {:?} should not be empty",
                phase
            );
        }
    }
}
