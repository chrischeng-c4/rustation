# Research: Internalize Spec Generation

**Feature**: 052-internalize-spec-generation
**Date**: 2025-12-16

## Research Tasks

### 1. Existing Shell Script Analysis

**Task**: Analyze `create-new-feature.sh` to understand exact behavior to replicate.

**Findings**:
- Location: `.specify/scripts/bash/create-new-feature.sh`
- Key functions:
  - `check_existing_branches()` - finds next available number (has bug: only checks same-name branches)
  - `generate_branch_name()` - converts description to kebab-case with stop word filtering
  - Creates directory structure and copies template
  - Creates git branch with feature number prefix

**Decision**: Replicate all functionality, but fix the number allocation bug (check ALL existing specs globally).

**Rationale**: The Rust implementation should be a superset of shell functionality, fixing known issues.

### 2. Feature Number Allocation Strategy

**Task**: Determine best approach for allocating feature numbers.

**Alternatives Considered**:
1. **Read features.json only** - Fast, but may miss orphaned directories
2. **Scan specs/ directory only** - Catches orphans, but ignores catalog
3. **Both: features.json + directory scan** - Most reliable, slightly slower

**Decision**: Use approach #3 (both sources), taking maximum of both.

**Rationale**:
- features.json is the source of truth but may be out of sync
- Directory scan catches manually created specs
- Maximum of both guarantees no conflicts
- Performance impact negligible (<10ms for 100 directories)

### 3. Name Generation Algorithm

**Task**: Define kebab-case conversion rules.

**Findings from shell script**:
- Stop words filtered: "i", "a", "an", "the", "to", "for", "of", "in", "on", "at", etc.
- Words <3 chars removed unless uppercase acronyms
- Take first 3-4 meaningful words
- Max length: 50 characters

**Decision**: Replicate exact algorithm in Rust with these rules:
1. Lowercase and split on non-alphanumeric
2. Filter stop words and short words
3. Take first 3-4 meaningful words
4. Join with hyphens
5. Truncate to 50 chars at word boundary

**Rationale**: Maintains backward compatibility with existing branch naming.

### 4. Claude CLI Integration

**Task**: Determine how to invoke Claude Code CLI from Rust.

**Alternatives Considered**:
1. **Direct process spawn** - `tokio::process::Command::new("claude")`
2. **Through shell** - `Command::new("sh").arg("-c").arg("claude ...")`
3. **SDK/Library** - No official Rust SDK available

**Decision**: Use direct process spawn (#1) with `tokio::process::Command`.

**Rationale**:
- No shell dependency, more portable
- Direct control over process lifecycle
- Easier timeout handling with tokio
- Can capture stdout/stderr separately

**Implementation Details**:
```rust
let output = tokio::process::Command::new("claude")
    .arg("--print")  // Non-interactive mode
    .arg("--dangerously-skip-permissions")  // Allow file writes
    .arg(&prompt)
    .current_dir(&workspace_root)
    .output()
    .await?;
```

### 5. Atomic File Operations

**Task**: Ensure file writes are atomic to prevent corruption.

**Decision**: Use temp file + rename pattern.

**Implementation**:
```rust
// Write to temp file
let temp_path = path.with_extension("tmp");
fs::write(&temp_path, content)?;

// Atomic rename
fs::rename(&temp_path, &path)?;
```

**Rationale**:
- `rename()` is atomic on POSIX systems
- Prevents partial writes on crash/interrupt
- Standard pattern used in databases and config systems

### 6. Error Recovery Strategy

**Task**: Define rollback behavior on partial failure.

**Decision**: Clean up created directories on any error after directory creation.

**Rollback Points**:
1. Number allocation fails → No cleanup needed
2. Directory creation fails → No cleanup needed
3. Template copy fails → Remove created directory
4. Claude CLI fails → Remove created directory
5. Catalog update fails → Remove created directory (or leave if spec is valid)

**Rationale**: Prefer clean state over partial artifacts. User can retry.

### 7. Testing Strategy

**Task**: Define testing approach without requiring Claude CLI.

**Decision**: Use mock for Claude CLI in tests, test real CLI in integration.

**Unit Tests** (no external deps):
- Number allocator: test with mock features.json and temp directories
- Name generator: pure function tests
- Catalog updater: test with temp files

**Integration Tests** (with mock CLI):
- Create mock `claude` script that returns canned responses
- Test full workflow in temp directory

**Manual Tests**:
- Real Claude CLI integration (requires API key)

## Summary of Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Number Allocation | features.json + directory scan | Most reliable |
| Name Generation | Replicate shell algorithm | Backward compatible |
| Claude CLI | Direct tokio::process spawn | No shell dependency |
| File Writes | Temp file + atomic rename | Corruption-safe |
| Error Recovery | Delete created directory | Clean state preferred |
| Testing | Mock Claude CLI | Fast, deterministic tests |
