//! Built-in default prompts embedded in the binary
//!
//! These prompts are embedded using include_str! and serve as fallbacks
//! when no project or user overrides are configured.

use super::SpecPhase;

/// Built-in specify prompt
pub const SPECIFY_PROMPT: &str = include_str!("templates/specify.md");

/// Built-in clarify prompt
pub const CLARIFY_PROMPT: &str = include_str!("templates/clarify.md");

/// Built-in plan prompt
pub const PLAN_PROMPT: &str = include_str!("templates/plan.md");

/// Built-in tasks prompt
pub const TASKS_PROMPT: &str = include_str!("templates/tasks.md");

/// Built-in implement prompt
pub const IMPLEMENT_PROMPT: &str = include_str!("templates/implement.md");

/// Built-in analyze prompt
pub const ANALYZE_PROMPT: &str = include_str!("templates/analyze.md");

/// Built-in checklist prompt
pub const CHECKLIST_PROMPT: &str = include_str!("templates/checklist.md");

/// Built-in review prompt
pub const REVIEW_PROMPT: &str = include_str!("templates/review.md");

/// Get the built-in prompt for a given phase
pub fn get_builtin_prompt(phase: SpecPhase) -> &'static str {
    match phase {
        SpecPhase::Specify => SPECIFY_PROMPT,
        SpecPhase::Clarify => CLARIFY_PROMPT,
        SpecPhase::Plan => PLAN_PROMPT,
        SpecPhase::Tasks => TASKS_PROMPT,
        SpecPhase::Implement => IMPLEMENT_PROMPT,
        SpecPhase::Analyze => ANALYZE_PROMPT,
        SpecPhase::Checklist => CHECKLIST_PROMPT,
        SpecPhase::Review => REVIEW_PROMPT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_prompts_non_empty() {
        assert!(!SPECIFY_PROMPT.is_empty());
        assert!(!CLARIFY_PROMPT.is_empty());
        assert!(!PLAN_PROMPT.is_empty());
        assert!(!TASKS_PROMPT.is_empty());
        assert!(!IMPLEMENT_PROMPT.is_empty());
        assert!(!ANALYZE_PROMPT.is_empty());
        assert!(!CHECKLIST_PROMPT.is_empty());
        assert!(!REVIEW_PROMPT.is_empty());
    }

    #[test]
    fn test_prompts_contain_frontmatter() {
        // All prompts should start with YAML frontmatter
        assert!(SPECIFY_PROMPT.starts_with("---"));
        assert!(CLARIFY_PROMPT.starts_with("---"));
        assert!(PLAN_PROMPT.starts_with("---"));
        assert!(TASKS_PROMPT.starts_with("---"));
        assert!(IMPLEMENT_PROMPT.starts_with("---"));
        assert!(ANALYZE_PROMPT.starts_with("---"));
        assert!(CHECKLIST_PROMPT.starts_with("---"));
        assert!(REVIEW_PROMPT.starts_with("---"));
    }

    #[test]
    fn test_get_builtin_prompt() {
        assert_eq!(get_builtin_prompt(SpecPhase::Specify), SPECIFY_PROMPT);
        assert_eq!(get_builtin_prompt(SpecPhase::Plan), PLAN_PROMPT);
        assert_eq!(get_builtin_prompt(SpecPhase::Tasks), TASKS_PROMPT);
    }
}
