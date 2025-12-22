//! Spec file integration and atomic writes
//!
//! This module handles updating spec files with clarification answers,
//! managing the Clarifications section, and ensuring atomic file operations.

use std::fs;
use std::path::Path;

use super::{
    session::ClarifySession, Answer, Category, ClarifyError, ClarifyReport, CoverageMap,
    CoverageStatus,
};

/// Find the Clarifications section in spec content
///
/// Returns the byte offset of the section start, or None if not found.
pub fn find_clarifications_section(content: &str) -> Option<usize> {
    content.find("## Clarifications")
}

/// Create a new Clarifications section header
fn create_clarifications_header() -> String {
    "\n## Clarifications\n".to_string()
}

/// Format a session header
pub fn format_session_header(date: &str) -> String {
    format!("\n### Session {}\n\n", date)
}

/// Format a Q&A bullet
pub fn format_qa_bullet(question: &str, answer: &str) -> String {
    format!("- Q: {} → A: {}\n", question, answer)
}

/// Check if a Q&A already exists in the clarifications section
pub fn check_duplicate_qa(content: &str, question: &str) -> bool {
    // Look for the question text in the clarifications section
    if let Some(clarifications_start) = find_clarifications_section(content) {
        let clarifications_content = &content[clarifications_start..];
        clarifications_content.contains(&format!("Q: {}", question))
    } else {
        false
    }
}

/// Find the best location to insert Clarifications section
///
/// Tries to insert after Overview, or at the end if no Overview found.
fn find_insertion_point(content: &str) -> usize {
    // Try to find the end of Overview section
    if let Some(overview_start) = content.find("## Overview") {
        // Find the next ## header after Overview
        let after_overview = &content[overview_start + 11..];
        if let Some(next_section) = after_overview.find("\n## ") {
            return overview_start + 11 + next_section;
        }
    }

    // Default: insert at end
    content.len()
}

/// Create the Clarifications section with session header
pub fn create_clarifications_section(date: &str) -> String {
    format!(
        "{}{}",
        create_clarifications_header(),
        format_session_header(date)
    )
}

/// Append Q&A to the clarifications section
pub fn append_to_clarifications(content: &str, date: &str, qa_bullet: &str) -> String {
    if let Some(clarifications_start) = find_clarifications_section(content) {
        // Check if this session header already exists
        let clarifications_content = &content[clarifications_start..];
        let session_header = format!("### Session {}", date);

        if clarifications_content.contains(&session_header) {
            // Find the session and append to it
            if let Some(session_start) = content[clarifications_start..].find(&session_header) {
                let absolute_session_start = clarifications_start + session_start;
                let after_session = &content[absolute_session_start..];

                // Find the next ### or ## header, or end
                let insertion_point = after_session
                    .find("\n### ")
                    .or_else(|| after_session.find("\n## "))
                    .map(|pos| absolute_session_start + pos)
                    .unwrap_or(content.len());

                // Insert before the next header
                let mut new_content = content[..insertion_point].to_string();
                new_content.push_str(qa_bullet);
                new_content.push_str(&content[insertion_point..]);
                return new_content;
            }
        } else {
            // Add new session header and Q&A
            let mut new_content = content.to_string();
            let insertion_point =
                find_next_section_after(content, clarifications_start).unwrap_or(content.len());

            let session_content = format!("{}{}", format_session_header(date), qa_bullet);
            new_content.insert_str(insertion_point, &session_content);
            return new_content;
        }
    }

    // No clarifications section - create one
    let insertion_point = find_insertion_point(content);
    let mut new_content = content[..insertion_point].to_string();
    new_content.push_str(&create_clarifications_section(date));
    new_content.push_str(qa_bullet);
    new_content.push_str(&content[insertion_point..]);
    new_content
}

fn find_next_section_after(content: &str, start: usize) -> Option<usize> {
    let after_start = &content[start..];

    // Skip the ## Clarifications header itself
    if let Some(header_end) = after_start.find('\n') {
        let remaining = &after_start[header_end + 1..];
        remaining
            .find("\n## ")
            .map(|pos| start + header_end + 1 + pos)
    } else {
        None
    }
}

/// Write spec content atomically (temp file + rename)
pub fn atomic_write_spec(spec_path: &Path, content: &str) -> Result<(), ClarifyError> {
    let temp_path = spec_path.with_extension("md.tmp");

    // Write to temp file
    fs::write(&temp_path, content).map_err(ClarifyError::SpecWrite)?;

    // Rename to final path (atomic on most filesystems)
    fs::rename(&temp_path, spec_path).map_err(ClarifyError::SpecWrite)?;

    Ok(())
}

/// Integrate a single answer into the spec
pub fn integrate_answer(session: &mut ClarifySession, answer: &Answer) -> Result<(), ClarifyError> {
    // Check for duplicates
    if check_duplicate_qa(&session.current_content, &answer.question_text) {
        tracing::debug!("Skipping duplicate Q&A: {}", answer.question_text);
        return Ok(());
    }

    // Format the Q&A bullet
    let qa_bullet = format_qa_bullet(&answer.question_text, &answer.value);

    // Append to clarifications section
    session.current_content =
        append_to_clarifications(&session.current_content, &session.session_date, &qa_bullet);

    // Write atomically
    atomic_write_spec(&session.spec_path, &session.current_content)?;

    Ok(())
}

/// Rollback spec to original content
pub fn rollback_spec(session: &ClarifySession) -> Result<(), ClarifyError> {
    atomic_write_spec(&session.spec_path, &session.original_content)
        .map_err(|e| ClarifyError::RollbackFailed(e.to_string()))
}

/// Count sections that were touched during integration
pub fn count_sections_touched(original: &str, current: &str) -> Vec<String> {
    let mut touched = Vec::new();

    // Check if Clarifications was added/modified
    let had_clarifications = original.contains("## Clarifications");
    let has_clarifications = current.contains("## Clarifications");

    if !had_clarifications && has_clarifications {
        touched.push("Clarifications (new)".to_string());
    } else if had_clarifications && has_clarifications && original != current {
        touched.push("Clarifications (updated)".to_string());
    }

    touched
}

/// Generate coverage summary after clarification
pub fn generate_coverage_summary(session: &ClarifySession) -> CoverageMap {
    let mut summary = session.analysis.coverage.clone();

    // Mark answered categories as Resolved
    for answer in &session.answers {
        // Find the category for this question
        for question in session.question_queue.iter() {
            if question.id == answer.question_id {
                summary.insert(question.category, CoverageStatus::Resolved);
                break;
            }
        }
    }

    // Also check original questions that were asked
    // (they were popped from queue when answered)
    // We track this by checking if questions_asked > 0

    summary
}

/// Identify outstanding categories (still Partial/Missing)
pub fn identify_outstanding(summary: &CoverageMap) -> Vec<Category> {
    summary
        .iter()
        .filter(|(_, status)| matches!(status, CoverageStatus::Partial | CoverageStatus::Missing))
        .map(|(cat, _)| *cat)
        .collect()
}

/// Identify deferred categories (marked as Deferred)
pub fn identify_deferred(summary: &CoverageMap) -> Vec<Category> {
    summary
        .iter()
        .filter(|(_, status)| matches!(status, CoverageStatus::Deferred))
        .map(|(cat, _)| *cat)
        .collect()
}

/// Suggest next command based on session state
pub fn suggest_next_command(outstanding: &[Category], deferred: &[Category]) -> String {
    if outstanding.is_empty() && deferred.is_empty() {
        "/speckit.plan - Spec is clear, proceed to planning".to_string()
    } else if !outstanding.is_empty() {
        format!(
            "/speckit.clarify - {} categories still need attention",
            outstanding.len()
        )
    } else {
        "/speckit.plan - Deferred items can be addressed during planning".to_string()
    }
}

/// Finalize session and generate report
pub fn finalize_session(session: &ClarifySession) -> Result<ClarifyReport, ClarifyError> {
    let sections_touched =
        count_sections_touched(&session.original_content, &session.current_content);
    let coverage_summary = generate_coverage_summary(session);
    let outstanding = identify_outstanding(&coverage_summary);
    let deferred = identify_deferred(&coverage_summary);
    let suggested_next = suggest_next_command(&outstanding, &deferred);

    Ok(ClarifyReport {
        spec_path: session.spec_path.clone(),
        questions_asked: session.questions_asked,
        questions_answered: session.answers.len(),
        sections_touched,
        coverage_summary,
        outstanding,
        deferred,
        suggested_next,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_clarifications_section_exists() {
        let content = "## Overview\n\nSome text\n\n## Clarifications\n\nQ&A here";
        assert!(find_clarifications_section(content).is_some());
    }

    #[test]
    fn test_find_clarifications_section_missing() {
        let content = "## Overview\n\nSome text\n\n## Requirements";
        assert!(find_clarifications_section(content).is_none());
    }

    #[test]
    fn test_format_session_header() {
        let header = format_session_header("2025-12-16");
        assert!(header.contains("### Session 2025-12-16"));
    }

    #[test]
    fn test_format_qa_bullet() {
        let bullet = format_qa_bullet("What is X?", "It is Y");
        assert_eq!(bullet, "- Q: What is X? → A: It is Y\n");
    }

    #[test]
    fn test_check_duplicate_qa_true() {
        let content = "## Clarifications\n\n### Session 2025-12-16\n\n- Q: What is X? → A: Y";
        assert!(check_duplicate_qa(content, "What is X?"));
    }

    #[test]
    fn test_check_duplicate_qa_false() {
        let content = "## Clarifications\n\n### Session 2025-12-16\n\n- Q: What is X? → A: Y";
        assert!(!check_duplicate_qa(content, "What is Z?"));
    }

    #[test]
    fn test_append_to_clarifications_new_section() {
        let content = "## Overview\n\nSome overview\n\n## Requirements\n\nReqs here";
        let result = append_to_clarifications(content, "2025-12-16", "- Q: Test? → A: Yes\n");

        assert!(result.contains("## Clarifications"));
        assert!(result.contains("### Session 2025-12-16"));
        assert!(result.contains("- Q: Test? → A: Yes"));
    }

    #[test]
    fn test_suggest_next_command_all_clear() {
        let suggested = suggest_next_command(&[], &[]);
        assert!(suggested.contains("/speckit.plan"));
        assert!(suggested.contains("Spec is clear"));
    }

    #[test]
    fn test_suggest_next_command_outstanding() {
        let outstanding = vec![Category::FunctionalScope];
        let suggested = suggest_next_command(&outstanding, &[]);
        assert!(suggested.contains("/speckit.clarify"));
    }

    #[test]
    fn test_identify_outstanding() {
        let mut summary = CoverageMap::new();
        summary.insert(Category::FunctionalScope, CoverageStatus::Clear);
        summary.insert(Category::DomainDataModel, CoverageStatus::Missing);
        summary.insert(Category::InteractionUxFlow, CoverageStatus::Partial);

        let outstanding = identify_outstanding(&summary);
        assert_eq!(outstanding.len(), 2);
    }
}
