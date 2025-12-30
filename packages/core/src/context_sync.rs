//! Context Sync system for CESDD Phase 4.
//!
//! Extracts valuable information from completed changes
//! and updates Living Context files.

use std::path::Path;

/// Build the prompt for Claude to extract context updates from a completed change.
///
/// The response should be a structured JSON object that we can parse to update context files.
pub fn build_context_sync_prompt(
    proposal_content: &str,
    plan_content: &str,
    existing_context: &str,
) -> String {
    format!(
        r#"You are a context curator for a software project. A change has been completed and archived.

## Your Task

Analyze the completed change (proposal + plan) and extract valuable information to update the project's Living Context.

## Input: Completed Change

### Proposal
{proposal_content}

### Plan
{plan_content}

## Input: Current Living Context
{existing_context}

## Instructions

Extract the following types of information from the change:

1. **Tech Stack Updates**: New technologies, libraries, or tools added
2. **Architecture Changes**: New components, patterns, or design decisions
3. **Key Decisions**: Important choices made and their rationale
4. **Recent Changes Summary**: A one-line summary of what was done

## Output Format

Respond with a JSON object in this exact format:

```json
{{
  "tech_stack_additions": [
    {{ "name": "library-name", "version": "1.0", "purpose": "what it's for" }}
  ],
  "architecture_updates": [
    {{ "component": "component-name", "description": "what it does", "location": "where in codebase" }}
  ],
  "key_decisions": [
    {{ "decision": "what was decided", "rationale": "why", "date": "YYYY-MM-DD" }}
  ],
  "recent_change_summary": "One-line summary of the change"
}}
```

If no updates are needed for a category, use an empty array `[]`.

Only include information that is:
- New (not already in the existing context)
- Significant (worth remembering for future development)
- Accurate (clearly stated in the proposal/plan)

Respond ONLY with the JSON object, no additional text."#
    )
}

/// Parse the JSON response from Claude into structured context updates.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContextSyncResponse {
    pub tech_stack_additions: Vec<TechStackAddition>,
    pub architecture_updates: Vec<ArchitectureUpdate>,
    pub key_decisions: Vec<KeyDecision>,
    pub recent_change_summary: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TechStackAddition {
    pub name: String,
    pub version: String,
    pub purpose: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ArchitectureUpdate {
    pub component: String,
    pub description: String,
    pub location: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KeyDecision {
    pub decision: String,
    pub rationale: String,
    pub date: String,
}

impl ContextSyncResponse {
    /// Parse JSON string into ContextSyncResponse
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        // Try to extract JSON from the response (it might have markdown code blocks)
        let json_str = extract_json_from_response(json_str);

        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse context sync response: {}", e))
    }

    /// Check if there are any updates
    pub fn has_updates(&self) -> bool {
        !self.tech_stack_additions.is_empty()
            || !self.architecture_updates.is_empty()
            || !self.key_decisions.is_empty()
            || !self.recent_change_summary.is_empty()
    }
}

/// Build enhanced prompt for context sync with individual file contents.
/// This allows Claude to make more targeted updates to specific files.
pub fn build_enhanced_context_sync_prompt(
    proposal_content: &str,
    plan_content: &str,
    tech_stack_content: &str,
    architecture_content: &str,
    recent_changes_content: &str,
) -> String {
    format!(
        r#"You are a context curator for a software project. A change has been completed.

## Your Task

Analyze the completed change and extract updates for the project's Living Context files.

## Completed Change

### Proposal
{proposal_content}

### Implementation Plan
{plan_content}

## Current Context Files

### tech-stack.md (Current)
{tech_stack_content}

### system-architecture.md (Current)
{architecture_content}

### recent-changes.md (Current)
{recent_changes_content}

## Instructions

Extract updates for each context file:

1. **tech_stack**: New technologies/libraries added (only if truly new, not already in tech-stack.md)
2. **architecture**: New components or significant architectural changes
3. **recent_changes**: A summary entry for this change

## Output Format

```json
{{
  "tech_stack": [
    {{ "name": "library-name", "version": "1.0", "purpose": "what it's for" }}
  ],
  "architecture": [
    {{ "component": "component-name", "description": "what it does", "location": "where in codebase" }}
  ],
  "recent_changes": {{
    "date": "YYYY-MM-DD",
    "change": "One-line summary of what was done",
    "impact": "High/Medium/Low"
  }},
  "summary": "Brief explanation of what was extracted"
}}
```

Rules:
- Use empty array `[]` if no updates for tech_stack or architecture
- recent_changes is REQUIRED (use today's date: {today})
- Only include genuinely NEW information not already in context
- Be concise and accurate

Respond ONLY with the JSON object."#,
        today = chrono::Utc::now().format("%Y-%m-%d")
    )
}

/// Enhanced response structure for context sync
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EnhancedContextSyncResponse {
    #[serde(default)]
    pub tech_stack: Vec<TechStackAddition>,
    #[serde(default)]
    pub architecture: Vec<ArchitectureUpdate>,
    pub recent_changes: RecentChangeEntry,
    pub summary: String,
}

/// A single recent change entry
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RecentChangeEntry {
    pub date: String,
    pub change: String,
    pub impact: String,
}

impl EnhancedContextSyncResponse {
    /// Parse JSON string into EnhancedContextSyncResponse
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let json_str = extract_json_from_response(json_str);
        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse enhanced context sync response: {}", e))
    }
}

/// Apply context updates to the context files on disk.
/// Returns the number of files updated.
pub fn apply_context_updates(
    project_path: &Path,
    response: &EnhancedContextSyncResponse,
) -> Result<u32, String> {
    let context_dir = project_path.join(".rstn").join("context");
    let mut files_updated = 0;

    // Update tech-stack.md if there are additions
    if !response.tech_stack.is_empty() {
        let tech_stack_path = context_dir.join("tech-stack.md");
        if tech_stack_path.exists() {
            let content = std::fs::read_to_string(&tech_stack_path)
                .map_err(|e| format!("Failed to read tech-stack.md: {}", e))?;

            // Append new rows to the tech stack table
            let additions = format_tech_stack_additions(&response.tech_stack);
            let updated_content = append_to_markdown_table(&content, &additions);

            std::fs::write(&tech_stack_path, updated_content)
                .map_err(|e| format!("Failed to write tech-stack.md: {}", e))?;
            files_updated += 1;
        }
    }

    // Update system-architecture.md if there are updates
    if !response.architecture.is_empty() {
        let arch_path = context_dir.join("system-architecture.md");
        if arch_path.exists() {
            let content = std::fs::read_to_string(&arch_path)
                .map_err(|e| format!("Failed to read system-architecture.md: {}", e))?;

            // Append new architecture sections
            let updates = format_architecture_updates(&response.architecture);
            let updated_content = format!("{}\n{}", content.trim_end(), updates);

            std::fs::write(&arch_path, updated_content)
                .map_err(|e| format!("Failed to write system-architecture.md: {}", e))?;
            files_updated += 1;
        }
    }

    // Always update recent-changes.md
    let recent_path = context_dir.join("recent-changes.md");
    if recent_path.exists() {
        let content = std::fs::read_to_string(&recent_path)
            .map_err(|e| format!("Failed to read recent-changes.md: {}", e))?;

        // Prepend new change entry to the table (newest first)
        let entry = format!(
            "| {} | {} | {} |",
            response.recent_changes.date,
            response.recent_changes.change,
            response.recent_changes.impact
        );
        let updated_content = prepend_to_markdown_table(&content, &entry);

        std::fs::write(&recent_path, updated_content)
            .map_err(|e| format!("Failed to write recent-changes.md: {}", e))?;
        files_updated += 1;
    }

    Ok(files_updated)
}

/// Append rows to a markdown table (adds after the last row)
fn append_to_markdown_table(content: &str, new_rows: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut found_table = false;
    let mut last_table_row = 0;

    // Find the last table row (line starting with |)
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with('|') {
            found_table = true;
            last_table_row = i;
        }
    }

    if found_table {
        // Insert new rows after the last table row
        for (i, line) in lines.iter().enumerate() {
            result.push(line.to_string());
            if i == last_table_row {
                result.push(new_rows.to_string());
            }
        }
        result.join("\n")
    } else {
        // No table found, just append
        format!("{}\n{}", content, new_rows)
    }
}

/// Prepend rows to a markdown table (adds after the header row)
fn prepend_to_markdown_table(content: &str, new_row: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut inserted = false;

    for line in lines.iter() {
        result.push(line.to_string());

        // Insert after the separator row (the line with |---|---|)
        if !inserted && line.contains("---") && line.starts_with('|') {
            result.push(new_row.to_string());
            inserted = true;
        }
    }

    if !inserted {
        // No separator found, just append
        result.push(new_row.to_string());
    }

    result.join("\n")
}

/// Extract JSON from a response that might have markdown code blocks
pub fn extract_json_from_response(response: &str) -> String {
    // Try to find JSON in code block
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start + 7..].find("```") {
            return response[start + 7..start + 7 + end].trim().to_string();
        }
    }

    // Try to find JSON in generic code block
    if let Some(start) = response.find("```") {
        let after_first = &response[start + 3..];
        if let Some(newline) = after_first.find('\n') {
            if let Some(end) = after_first[newline..].find("```") {
                return after_first[newline..newline + end].trim().to_string();
            }
        }
    }

    // Try to find raw JSON (starts with {)
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return response[start..=end].to_string();
            }
        }
    }

    // Return as-is if no JSON found
    response.to_string()
}

/// Generate markdown content to append to tech-stack.md
pub fn format_tech_stack_additions(additions: &[TechStackAddition]) -> String {
    if additions.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for addition in additions {
        lines.push(format!(
            "| {} | {} | {} |",
            addition.name, addition.version, addition.purpose
        ));
    }
    lines.join("\n")
}

/// Generate markdown content to append to system-architecture.md
pub fn format_architecture_updates(updates: &[ArchitectureUpdate]) -> String {
    if updates.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for update in updates {
        lines.push(format!(
            "\n### {}\n\n{}\n\n*Location: {}*",
            update.component, update.description, update.location
        ));
    }
    lines.join("\n")
}

/// Generate markdown content to append to recent-changes.md
pub fn format_recent_changes(summary: &str, decisions: &[KeyDecision]) -> String {
    let mut lines = Vec::new();

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    lines.push(format!("| {} | {} | - |", today, summary));

    for decision in decisions {
        lines.push(format!(
            "\n**Decision ({})**: {}\n*Rationale: {}*",
            decision.date, decision.decision, decision.rationale
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_response_raw() {
        let response = r#"{"tech_stack_additions": [], "architecture_updates": [], "key_decisions": [], "recent_change_summary": "test"}"#;
        let json = extract_json_from_response(response);
        assert!(json.starts_with("{"));
        assert!(json.ends_with("}"));
    }

    #[test]
    fn test_extract_json_from_response_code_block() {
        let response = r#"Here's the JSON:

```json
{
  "tech_stack_additions": [],
  "architecture_updates": [],
  "key_decisions": [],
  "recent_change_summary": "test"
}
```"#;
        let json = extract_json_from_response(response);
        assert!(json.contains("tech_stack_additions"));
        assert!(json.contains("recent_change_summary"));
    }

    #[test]
    fn test_parse_context_sync_response() {
        let json = r#"{
            "tech_stack_additions": [
                {"name": "tokio", "version": "1.0", "purpose": "async runtime"}
            ],
            "architecture_updates": [
                {"component": "MCP Server", "description": "Added MCP protocol support", "location": "src/mcp/"}
            ],
            "key_decisions": [
                {"decision": "Use async/await", "rationale": "Better performance", "date": "2025-01-01"}
            ],
            "recent_change_summary": "Added MCP server support"
        }"#;

        let response = ContextSyncResponse::from_json(json).unwrap();
        assert_eq!(response.tech_stack_additions.len(), 1);
        assert_eq!(response.architecture_updates.len(), 1);
        assert_eq!(response.key_decisions.len(), 1);
        assert_eq!(response.recent_change_summary, "Added MCP server support");
        assert!(response.has_updates());
    }

    #[test]
    fn test_empty_response() {
        let json = r#"{
            "tech_stack_additions": [],
            "architecture_updates": [],
            "key_decisions": [],
            "recent_change_summary": ""
        }"#;

        let response = ContextSyncResponse::from_json(json).unwrap();
        assert!(!response.has_updates());
    }

    #[test]
    fn test_format_tech_stack_additions() {
        let additions = vec![
            TechStackAddition {
                name: "tokio".to_string(),
                version: "1.0".to_string(),
                purpose: "async runtime".to_string(),
            },
        ];
        let formatted = format_tech_stack_additions(&additions);
        assert!(formatted.contains("| tokio | 1.0 | async runtime |"));
    }

    #[test]
    fn test_format_architecture_updates() {
        let updates = vec![
            ArchitectureUpdate {
                component: "MCP Server".to_string(),
                description: "Handles MCP protocol".to_string(),
                location: "src/mcp/".to_string(),
            },
        ];
        let formatted = format_architecture_updates(&updates);
        assert!(formatted.contains("### MCP Server"));
        assert!(formatted.contains("*Location: src/mcp/*"));
    }

    #[test]
    fn test_build_context_sync_prompt() {
        let prompt = build_context_sync_prompt(
            "# Proposal\nAdd auth",
            "# Plan\n1. Do stuff",
            "# Context\nEmpty",
        );
        assert!(prompt.contains("# Proposal"));
        assert!(prompt.contains("# Plan"));
        assert!(prompt.contains("tech_stack_additions"));
    }
}
