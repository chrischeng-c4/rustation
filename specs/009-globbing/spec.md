# Specification: Globbing and Wildcard Expansion

**Feature ID:** 009-globbing
**Status:** Planned - Specification Complete, Implementation Pending
**Created:** 2025-11-29

## Overview

Implement shell globbing (wildcard expansion) for the rush shell, including:
- `*` - Match zero or more characters in a filename
- `?` - Match exactly one character
- `[abc]` - Match any character in the set
- `[a-z]` - Match any character in the range
- `[!abc]` - Match any character NOT in the set
- Tilde expansion (`~` → home directory)

Globbing allows users to perform powerful batch operations on files and directories matching patterns.

## Motivation

Globbing is fundamental to shell productivity. Without it, users cannot:
- Select multiple files: `ls *.txt`
- Find specific files: `find . -name "*.rs"`
- Process file patterns: `rm /tmp/temp-*`
- List by extension: `echo *.log`
- Avoid hardcoding file paths: `cat file?.txt`

## User Stories

### US1: Basic Wildcard Matching
**As a** shell user
**I want to** use `*` to match filenames
**So that** I can work with multiple files matching a pattern

**Acceptance Criteria:**
- `ls *.txt` expands to all `.txt` files in current directory
- `ls *.txt` matches zero files without error (expands to literal `*.txt`)
- `ls dir/*.rs` expands files in subdirectory
- `echo *` lists all files in current directory
- Exit code 0 on success, 1 on error
- Works in command arguments (not filenames themselves)

**Examples:**
```bash
$ ls
file1.txt file2.txt data.csv script.rs
$ ls *.txt
file1.txt
file2.txt
$ ls *.log
*.log          # No matches - outputs literal pattern
$ ls dir/*.rs
dir/main.rs
dir/lib.rs
```

### US2: Single Character Matching
**As a** shell user
**I want to** use `?` to match single characters
**So that** I can find files with specific naming patterns

**Acceptance Criteria:**
- `ls file?.txt` matches `file1.txt`, `fileA.txt`, etc.
- `ls ??.rs` matches exactly 2-character filenames
- `?` does NOT match `/` (path separator)
- Multiple `?` allowed: `ls ?????.log` matches 5-character names
- Works in any position of filename

**Examples:**
```bash
$ ls
file1.txt file2.txt file10.txt
$ ls file?.txt
file1.txt
file2.txt
$ ls file??.txt
file10.txt
```

### US3: Character Sets and Ranges
**As a** shell user
**I want to** use `[abc]` and `[a-z]` for character matching
**So that** I can match specific file patterns

**Acceptance Criteria:**
- `[abc]` matches any single character in the set
- `[a-z]` matches any character in range (inclusive)
- `[0-9]` matches digits
- `[a-zA-Z]` matches letters (case-insensitive patterns)
- `[!abc]` matches any character NOT in set
- Works with multiple ranges: `[a-z0-9]`
- Hyphen in set: `[a-z-]` or `[-a-z]` treats `-` as literal

**Examples:**
```bash
$ ls
file1.txt file2.txt fileA.txt fileB.txt
$ ls file[0-9].txt
file1.txt
file2.txt
$ ls file[a-z].txt
fileA.txt       # Assuming case-sensitive
$ ls file[!0-9].txt
fileA.txt
fileB.txt
```

### US4: Negated Character Sets
**As a** shell user
**I want to** exclude files matching a pattern
**So that** I can work with specific file subsets

**Acceptance Criteria:**
- `[!abc]` matches characters NOT in set
- `[!0-9]` matches non-digit characters
- Works with ranges: `[!a-z]` matches non-lowercase
- Can be combined: `[!0-9.]` matches non-digits and non-dots

**Examples:**
```bash
$ ls
file1.txt file2.txt fileA.txt fileB.txt
$ ls file[!0-9].txt
fileA.txt
fileB.txt
$ ls *[!.txt]
file1.rs
script.py
```

### US5: Escaping Glob Characters
**As a** shell user
**I want to** use literal glob characters in filenames
**So that** I can work with files containing `*`, `?`, or `[`

**Acceptance Criteria:**
- `\*` matches literal `*` character
- `\?` matches literal `?` character
- `\[` matches literal `[` character
- `\\` matches literal `\` character
- Single quotes prevent globbing: `echo '*'` outputs literal `*`
- Double quotes allow variable expansion but prevent globbing: `echo "$VAR"` but glob chars literal

**Examples:**
```bash
$ ls
file*.txt question?.md [readme].txt
$ ls file\*.txt
file*.txt
$ ls '[readme].txt'
[readme].txt
$ echo "*.txt"
*.txt
```

## Technical Requirements

### Implementation Approach

1. **Glob Pattern Matching:**
   - Implement pattern matching algorithm (e.g., using iterative state machine)
   - Support `*`, `?`, `[...]`, `[!...]` patterns
   - Respect quotes (prevent globbing in quoted strings)
   - Handle escape sequences

2. **Integration Point:**
   - Expand globs in command arguments AFTER variable expansion
   - Before command execution
   - Pattern: `expand_variables()` → `expand_globs()` → parse/execute

3. **File Discovery:**
   - Use `std::fs::read_dir()` to traverse directories
   - Match each filename against pattern
   - Return sorted list of matching paths

4. **Special Cases:**
   - `*` in middle of path: `dir/*.txt` (works)
   - `*` as entire filename: `ls *` (works)
   - `*` at start: `*.txt` (works)
   - No matches: Return unmodified pattern (shell behavior)
   - Literal `.` and `..` should not match `*`

### Storage and Data Structures

```rust
pub struct GlobPattern {
    segments: Vec<PatternSegment>,  // Path components
}

pub enum PatternSegment {
    Literal(String),               // /absolute/path
    Glob(String),                  // *.txt or **/
    WildcardDir,                   // **
}
```

### Pattern Matching Algorithm

```rust
pub fn glob_match(pattern: &str, path: &str) -> bool {
    // Match pattern against single path
    // * matches any sequence of non-/ characters
    // ? matches single non-/ character
    // [abc] matches character set
    // [a-z] matches range
    // [!abc] matches negated set
}

pub fn expand_glob(pattern: &str) -> Vec<String> {
    // Return sorted list of matching paths
    // If no matches, return [pattern] (literal)
}
```

### Integration with Variable Expansion

Variable expansion happens FIRST, then globbing:

```rust
pub fn execute(&mut self, line: &str) -> Result<i32> {
    let expanded_vars = expand_variables(line, self)?;    // $VAR → value
    let expanded_globs = expand_globs(&expanded_vars)?;   // *.txt → file1.txt file2.txt
    let pipeline = parse_pipeline(&expanded_globs)?;      // Parse expanded command
    self.execute_pipeline(pipeline)
}
```

## Success Metrics

1. **Functionality:**
   - All 5 user stories pass acceptance tests
   - `*` matches zero or more chars correctly
   - `?` matches exactly one char
   - `[abc]` and ranges work
   - `[!abc]` negation works
   - Escaping prevents expansion

2. **Compatibility:**
   - Behavior matches bash glob expansion
   - No matches returns literal pattern
   - Quoted patterns not expanded
   - Escaped characters treated as literals

3. **Testing:**
   - Unit tests for pattern matching (20+ tests)
   - Integration tests for glob expansion (15+ tests)
   - Edge cases: empty dirs, special files, symlinks

## Out of Scope

These features are NOT included:
- `**` (recursive glob) - May be added later
- `extglob` patterns (`?(pattern)`, `+(pattern)`)
- POSIX character classes (`[:alnum:]`, `[:digit:]`)
- Brace expansion (`{a,b,c}`)
- Tilde expansion in paths (handled by variable expansion)

These may be added in future iterations.

## Dependencies

- `std::fs` - Directory traversal and file metadata
- `std::path` - Path manipulation
- Existing parser and expansion infrastructure

No new external dependencies required.

## Timeline

**Estimated Effort:** 3-4 hours
- Specification: 30 min (this document)
- Implementation: 2-2.5 hours
  - Pattern matching: 1 hour
  - Glob expansion: 45 min
  - Integration: 15-30 min
- Testing: 45 min - 1 hour
- Documentation: 15 min

**Target Completion:** Current or next session
