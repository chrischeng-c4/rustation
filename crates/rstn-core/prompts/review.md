---
description: Review PR against issue requirements to verify implementation matches specification
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Verify that a PR implementation matches the linked issue/spec requirements. This command runs after `/speckit.implement` and before merge to ensure code aligns with specifications.

## Operating Constraints

**READ-ONLY Analysis**: Do **not** modify any files. Output a structured review report with pass/fail findings and recommendations.

## Execution Steps

### 1. Identify PR and Issue

If PR number provided in arguments, use it. Otherwise detect current branch's PR:

```bash
gh pr view --json number,title,body,headRefName
```

Parse the PR body to find linked issue (look for patterns like `closes #N`, `fixes #N`, `Parent: #N`, or issue URLs).

If no linked issue found, prompt user to provide issue number.

### 2. Load Issue Specification

Fetch the linked issue:

```bash
gh issue view {number} --json title,body,comments
```

Extract from issue body:
- Requirements/acceptance criteria
- User stories
- Technical constraints
- Any spec content synced from spec.md

Also check issue comments for:
- Plan details
- Clarifications
- Decision records

### 3. Load PR Changes

Get the PR diff and changed files:

```bash
gh pr diff {pr_number}
gh pr view {pr_number} --json files
```

Identify:
- New files created
- Modified files
- Test files added/modified
- Documentation changes

### 4. Requirements Coverage Analysis

For each requirement/acceptance criterion in the issue:

| Requirement | Status | Evidence | Notes |
|-------------|--------|----------|-------|
| Req 1 | PASS/FAIL/PARTIAL | file:line or "not found" | details |

**Status definitions:**
- **PASS**: Clear implementation found matching requirement
- **PARTIAL**: Partially implemented, missing aspects noted
- **FAIL**: No implementation found for requirement
- **N/A**: Requirement not applicable to this PR scope

### 5. Test Coverage Check

Verify tests exist for implemented functionality:

- Unit tests for new functions/modules
- Integration tests for new features
- Edge case coverage per spec
- Test files match implementation files

| Component | Has Tests? | Test File | Coverage Notes |
|-----------|------------|-----------|----------------|
| module_x | YES/NO | path/to/test | notes |

### 6. Documentation Check

Verify documentation updated:

- `docs/` folder changes if feature adds user-facing functionality
- Code comments for complex logic
- README updates if applicable

### 7. Generate Review Report

Output structured Markdown report:

## PR Review Report

**PR**: #{number} - {title}
**Issue**: #{issue_number} - {issue_title}
**Branch**: {branch_name}

### Requirements Coverage

| # | Requirement | Status | Evidence |
|---|-------------|--------|----------|
| 1 | ... | PASS | file:line |

**Coverage**: X/Y requirements (Z%)

### Test Coverage

| Component | Tests | File |
|-----------|-------|------|
| ... | YES/NO | ... |

### Documentation

- [ ] docs/ updated
- [ ] Code comments adequate
- [ ] README updated (if needed)

### Issues Found

| Severity | Issue | Recommendation |
|----------|-------|----------------|
| HIGH | Missing impl for req X | Add implementation |
| MEDIUM | No tests for Y | Add unit tests |

### Recommendation

**APPROVE** / **REQUEST CHANGES** / **NEEDS DISCUSSION**

{Summary of what needs to be done before merge}

## 8. Provide Next Actions

Based on findings:

- If all PASS: "Ready to merge. Run `gh pr merge --merge`"
- If PARTIAL/FAIL: List specific items to address before merge
- If tests missing: "Add tests for: X, Y, Z"
- If docs missing: "Update docs/ for: feature description"

## Operating Principles

### Review Guidelines

- **Be specific**: Reference exact files and line numbers
- **Be constructive**: Provide actionable recommendations
- **Prioritize**: Focus on blocking issues first
- **Trust specs**: Issue spec is source of truth for requirements
- **Check tests**: No untested code should merge

### Severity Levels

- **HIGH**: Missing core requirement, broken functionality, no tests for critical path
- **MEDIUM**: Partial implementation, missing edge cases, incomplete docs
- **LOW**: Style issues, minor improvements, optional enhancements

## Context

$ARGUMENTS
