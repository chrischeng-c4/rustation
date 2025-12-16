# Feature 059: Prompt Content Manager

**Feature Branch**: `059-prompt-content-manager`
**Created**: 2024-12-16
**Status**: Draft

## Overview

Centralized prompt content management for spec-kit commands. Manages prompt templates that are injected via `--system-prompt-file` or `--append-system-prompt` when invoking Claude Code CLI.

## Dependencies

**Depends on:**
- Feature 052 (Internalize Spec Generation) - Rust implementation for CLI invocation

## Problem Statement

Current spec-kit prompts are scattered across `.claude/commands/*.md` files in each project. This means:
1. rstn cannot control prompt behavior centrally
2. Different projects may have divergent prompt behavior
3. Updates require modifying files in user projects

## User Stories

### As an rstn maintainer
- I want to manage spec-kit prompts in rstn codebase
- So that I can update behavior without modifying user projects

### As an rstn user
- I want consistent spec-kit behavior across projects
- So that I have a predictable workflow

### As an rstn user
- I want to customize prompts when needed
- So that I can adapt to project-specific needs

## Requirements

### Functional Requirements

- **FR-1**: Store default prompt templates in rstn binary/resources
- **FR-2**: Load prompts at runtime for each spec-kit command
- **FR-3**: Support `--system-prompt-file` for full override
- **FR-4**: Support `--append-system-prompt` for additions
- **FR-5**: Allow project-level customization via `.rstn/prompts/`

### Prompt Types

- `specify.md` - Prompt for spec generation
- `clarify.md` - Prompt for clarification workflow
- `plan.md` - Prompt for plan generation
- `tasks.md` - Prompt for task breakdown
- `implement.md` - Prompt for implementation guidance
- `analyze.md` - Prompt for consistency analysis
- `checklist.md` - Prompt for checklist generation

## Architecture

### Prompt Resolution Order

1. Project override: `.rstn/prompts/{command}.md`
2. User override: `~/.config/rstn/prompts/{command}.md`
3. Built-in default: Embedded in rstn binary

### Integration

```rust
pub fn get_prompt(command: &str) -> String {
    // Check project override
    // Check user override
    // Fall back to built-in
}

pub fn invoke_claude_with_prompt(command: &str, args: &[&str]) {
    let prompt = get_prompt(command);
    Command::new("claude")
        .arg("--system-prompt-file")
        .arg(&prompt_path)
        // ... other args
}
```

## Success Criteria

- rstn controls spec-kit prompt behavior
- Users can override when needed
- Consistent behavior across projects by default
- Easy to update prompts in rstn releases
