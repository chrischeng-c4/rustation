//! Spec analysis and coverage mapping
//!
//! This module analyzes spec files for ambiguities using the 10-category
//! taxonomy and generates coverage maps.

use std::collections::HashMap;
use std::path::Path;

use super::{AnalysisResult, Category, ClarifyError, CoverageMap, CoverageStatus};

/// Load and read spec file content
pub fn load_spec_content(spec_path: &Path) -> Result<String, ClarifyError> {
    if !spec_path.exists() {
        return Err(ClarifyError::SpecNotFound(spec_path.to_path_buf()));
    }

    std::fs::read_to_string(spec_path).map_err(ClarifyError::SpecRead)
}

/// Parse spec content into sections (by ## headers)
pub fn parse_spec_sections(content: &str) -> Vec<(&str, &str)> {
    let mut sections = Vec::new();
    let mut current_header = "";
    let mut current_content_start = 0;

    for (idx, line) in content.lines().enumerate() {
        if line.starts_with("## ") {
            // Save previous section if exists
            if !current_header.is_empty() || idx > 0 {
                let section_end = content
                    .lines()
                    .take(idx)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                let section_content =
                    &content[current_content_start..section_end.min(content.len())];
                sections.push((current_header, section_content.trim()));
            }

            current_header = line.trim_start_matches("## ").trim();
            current_content_start = content
                .lines()
                .take(idx + 1)
                .map(|l| l.len() + 1)
                .sum::<usize>();
        }
    }

    // Add final section
    if !current_header.is_empty() {
        let section_content = &content[current_content_start..];
        sections.push((current_header, section_content.trim()));
    }

    sections
}

/// Count keyword matches for a category in content
pub fn scan_category(content: &str, category: Category) -> usize {
    let content_lower = content.to_lowercase();
    category
        .keywords()
        .iter()
        .map(|keyword| content_lower.matches(&keyword.to_lowercase()).count())
        .sum()
}

/// Classify coverage status from match count and placeholder detection
pub fn classify_coverage(match_count: usize, has_placeholders: bool) -> CoverageStatus {
    match (match_count, has_placeholders) {
        (0, _) => CoverageStatus::Missing,
        (1..=2, true) => CoverageStatus::Partial,
        (1..=2, false) => CoverageStatus::Partial,
        (_, true) => CoverageStatus::Partial,
        (_, false) => CoverageStatus::Clear,
    }
}

/// Detect TODO/TBD/placeholder markers in content
pub fn detect_placeholders(content: &str) -> bool {
    let content_lower = content.to_lowercase();
    let placeholder_markers = ["todo", "tbd", "tbc", "fixme", "placeholder", "[...]", "xxx"];

    placeholder_markers
        .iter()
        .any(|marker| content_lower.contains(marker))
}

/// Analyze a spec file for ambiguities
///
/// Returns an AnalysisResult with coverage status for each category.
pub fn analyze_spec(spec_path: &Path) -> Result<AnalysisResult, ClarifyError> {
    let content = load_spec_content(spec_path)?;
    analyze_spec_content(&content)
}

/// Analyze spec content directly (useful for testing)
pub fn analyze_spec_content(content: &str) -> Result<AnalysisResult, ClarifyError> {
    let has_global_placeholders = detect_placeholders(content);

    let mut coverage = CoverageMap::new();
    let mut match_counts = HashMap::new();
    let mut needs_clarification = Vec::new();

    for category in Category::all() {
        let count = scan_category(content, *category);
        match_counts.insert(*category, count);

        let status = classify_coverage(count, has_global_placeholders);
        coverage.insert(*category, status);

        if matches!(status, CoverageStatus::Partial | CoverageStatus::Missing) {
            needs_clarification.push(*category);
        }
    }

    Ok(AnalysisResult {
        coverage,
        needs_clarification,
        match_counts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_category_functional_scope() {
        let content = "The goal is to achieve success. Must have feature X.";
        let count = scan_category(content, Category::FunctionalScope);
        assert!(count >= 3); // goal, success, must
    }

    #[test]
    fn test_scan_category_case_insensitive() {
        let content = "GOAL and Goal and goal";
        let count = scan_category(content, Category::FunctionalScope);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_detect_placeholders_positive() {
        assert!(detect_placeholders("TODO: implement this"));
        assert!(detect_placeholders("This is TBD"));
        assert!(detect_placeholders("FIXME: broken"));
    }

    #[test]
    fn test_detect_placeholders_negative() {
        assert!(!detect_placeholders("This is complete content"));
        assert!(!detect_placeholders("No markers here"));
    }

    #[test]
    fn test_classify_coverage() {
        assert_eq!(classify_coverage(0, false), CoverageStatus::Missing);
        assert_eq!(classify_coverage(0, true), CoverageStatus::Missing);
        assert_eq!(classify_coverage(1, true), CoverageStatus::Partial);
        assert_eq!(classify_coverage(5, false), CoverageStatus::Clear);
        assert_eq!(classify_coverage(5, true), CoverageStatus::Partial);
    }

    #[test]
    fn test_analyze_spec_content_empty() {
        let result = analyze_spec_content("").unwrap();
        assert_eq!(result.needs_clarification.len(), 10); // All categories missing
    }

    #[test]
    fn test_analyze_spec_content_partial() {
        let content =
            "## Overview\n\nThe goal is to add authentication.\n\nTODO: Define requirements";
        let result = analyze_spec_content(content).unwrap();

        // FunctionalScope should be partial (has goal but has TODO)
        assert_eq!(
            result.coverage.get(&Category::FunctionalScope),
            Some(&CoverageStatus::Partial)
        );
    }

    #[test]
    fn test_parse_spec_sections() {
        let content = "## Overview\n\nSome overview text.\n\n## Requirements\n\nRequirements here.";
        let sections = parse_spec_sections(content);

        assert!(!sections.is_empty());
    }
}
