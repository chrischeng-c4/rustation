# Implementation Plan: Globbing and Wildcard Expansion

**Feature:** 009-globbing
**Created:** 2025-11-29

## Implementation Approach

Globbing will be implemented as a new module that expands wildcard patterns in command arguments:
1. **Pattern Matching** - Core algorithm to match filenames against patterns
2. **Glob Expansion** - Traverse directories and collect matching files
3. **Integration** - Call glob expansion after variable expansion, before parsing

## Architecture

### Module Structure
```
crates/rush/src/executor/
├── mod.rs              # Add glob module
├── execute.rs          # Integrate glob expansion in execute()
├── expansion.rs        # Existing variable expansion
├── glob.rs             # New: Glob implementation
└── ...
```

### Core Components

**glob.rs** - Main implementation
```rust
pub fn glob_expand(input: &str) -> Result<String>
    // Main entry point: expand glob patterns in input string
    // Processes each argument separately
    // Returns expanded command line or error

fn pattern_matches(pattern: &str, filename: &str) -> bool
    // Check if single filename matches pattern
    // Handles *, ?, [abc], [!abc]

fn expand_single_pattern(pattern: &str) -> Result<Vec<String>>
    // Expand one glob pattern to matching files
    // Returns sorted Vec or single pattern if no matches

fn match_character_set(pattern: &str, pos: &mut usize, ch: char) -> bool
    // Helper: Match [abc], [a-z], [!abc] patterns
    // Advances position through closing ]

fn glob_match_recursive(
    pattern: &str,
    pattern_pos: usize,
    text: &str,
    text_pos: usize,
) -> bool
    // Recursive backtracking matcher for * wildcards
    // Handles complex patterns with multiple *
```

## Implementation Steps

### Step 1: Create glob.rs Module

**File:** `crates/rush/src/executor/glob.rs`

Implement core pattern matching:

```rust
/// Expand glob patterns in a command line
/// Example: "ls *.txt" → "ls file1.txt file2.txt"
pub fn glob_expand(line: &str) -> Result<String> {
    // Split into arguments respecting quotes
    // For each argument:
    //   - If quoted (single or double): don't expand, strip quotes
    //   - If unquoted: expand glob pattern
    // Rejoin arguments
}

/// Match a filename against a glob pattern
fn pattern_matches(pattern: &str, filename: &str) -> bool {
    // Returns true if filename matches pattern
}

/// Expand a single glob pattern to matching files
fn expand_single_glob(pattern: &str) -> Result<Vec<String>> {
    // Separate pattern into directory and filename parts
    // Traverse directory (recursively if needed)
    // Match each file against filename pattern
    // Return sorted list or [pattern] if no matches
}

// Helpers for different pattern types
fn matches_glob_pattern(pattern: &str, text: &str) -> bool { }
fn matches_question_mark(pattern: &str, text: &str) -> bool { }
fn matches_bracket_expr(pattern: &str, pos: &mut usize, text: &str, tpos: &mut usize) -> bool { }
```

**Pattern Matching Algorithm:**

```
For each character in pattern:
  '*' → Match zero or more non-/ characters
        Use backtracking to try different lengths
  '?' → Match exactly one non-/ character
  '[' → Match character set until ']'
        [abc] → match a, b, or c
        [a-z] → match a through z
        [!abc] → match anything except a, b, c
  '\\' → Escape next character (literal match)
  other → Match literal character
```

**Tests (20+ tests):**
- `*` matches multiple files
- `*` matches zero files (no matches)
- `*` doesn't match `/`
- `?` matches single char
- `?` doesn't match multiple or zero chars
- `[abc]` matches set
- `[a-z]` matches range
- `[!abc]` matches negation
- Escape sequences `\*`, `\?`, `\[`
- Quoted strings not expanded
- Multiple patterns
- Nested paths `dir/*.txt`

### Step 2: Integrate into execute.rs

**File:** `crates/rush/src/executor/execute.rs`

Add glob expansion to execution flow:

```rust
pub fn execute(&mut self, line: &str) -> Result<i32> {
    // ... existing code ...

    // Step 1: Variable expansion
    let expanded_vars = expand_variables(line, self)?;

    // Step 2: NEW - Glob expansion
    let expanded_globs = glob_expand(&expanded_vars)?;

    // Step 3: Parse command
    let pipeline = parse_pipeline(&expanded_globs)?;

    // ... rest of execution ...
}
```

**Integration points:**
- Call `glob_expand()` after `expand_variables()`
- Before `parse_pipeline()`
- Error handling: Return `RushError` if glob fails

### Step 3: Handle Quote Preservation

**In glob.rs:**

```rust
// Respect quotes during glob expansion
fn expand_with_quote_handling(line: &str) -> Result<String> {
    let mut result = String::new();
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut escape_next = false;
    let mut current_arg = String::new();

    for ch in line.chars() {
        if escape_next {
            current_arg.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if !in_single_quote => {
                escape_next = true;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            ' ' if !in_double_quote && !in_single_quote => {
                // Argument boundary
                if !current_arg.is_empty() {
                    result.push_str(&expand_single_argument(&current_arg)?);
                    result.push(' ');
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    if !current_arg.is_empty() {
        result.push_str(&expand_single_argument(&current_arg)?);
    }

    Ok(result)
}
```

### Step 4: Add Directory Traversal

```rust
use std::fs;
use std::path::{Path, PathBuf};

fn expand_single_glob(pattern: &str) -> Result<Vec<String>> {
    // Split pattern into directory and filename parts
    let path = Path::new(pattern);
    let (dir_path, filename_pattern) = if let Some(parent) = path.parent() {
        if parent.as_os_str().is_empty() {
            (PathBuf::from("."), pattern)
        } else {
            (parent.to_path_buf(), path.file_name().unwrap().to_str().unwrap())
        }
    } else {
        (PathBuf::from("."), pattern)
    };

    // Read directory
    let entries = fs::read_dir(&dir_path)
        .map_err(|e| RushError::Execution(format!("glob: {}", e)))?;

    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        // Skip . and .. unless explicitly matched
        if name_str == "." || name_str == ".." {
            continue;
        }

        if pattern_matches(filename_pattern, &name_str) {
            let full_path = entry.path();
            matches.push(full_path.to_string_lossy().to_string());
        }
    }

    // Sort for consistent output
    matches.sort();

    // Return matches or literal pattern if no matches
    if matches.is_empty() {
        Ok(vec![pattern.to_string()])
    } else {
        Ok(matches)
    }
}
```

### Step 5: Register Module

**File:** `crates/rush/src/executor/mod.rs`

Add module declaration:
```rust
pub mod glob;
```

### Step 6: Add Unit Tests

**In glob.rs:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_asterisk_single_file() {
        // Pattern: *.txt
        // Expected: file1.txt file2.txt
    }

    #[test]
    fn test_glob_asterisk_no_matches() {
        // Pattern: *.xyz (no files)
        // Expected: *.xyz (literal pattern)
    }

    #[test]
    fn test_glob_question_mark() {
        // Pattern: file?.txt
        // Expected: file1.txt file2.txt (not file10.txt)
    }

    #[test]
    fn test_glob_bracket_set() {
        // Pattern: file[abc].txt
        // Expected: filea.txt fileb.txt filec.txt
    }

    #[test]
    fn test_glob_bracket_range() {
        // Pattern: file[0-9].txt
        // Expected: file0.txt file1.txt ... file9.txt
    }

    #[test]
    fn test_glob_bracket_negated() {
        // Pattern: file[!0-9].txt
        // Expected: filea.txt fileb.txt (not file1.txt)
    }

    #[test]
    fn test_glob_escape_sequence() {
        // Pattern: file\\*.txt
        // Expected: file*.txt (literal asterisk)
    }

    #[test]
    fn test_glob_quoted_string() {
        // Input: echo "*.txt"
        // Expected: *.txt (not expanded)
    }

    #[test]
    fn test_glob_nested_path() {
        // Pattern: dir/*.txt
        // Expected: dir/file1.txt dir/file2.txt
    }

    // ... 12+ more tests
}
```

### Step 7: Add Integration Tests

**File:** `crates/rush/tests/integration/glob_tests.rs`

```rust
#[test]
fn test_glob_ls_wildcard() {
    let mut executor = CommandExecutor::new();

    // Create test files
    // Run: ls *.txt
    // Verify: all .txt files listed
}

#[test]
fn test_glob_echo_pattern() {
    let mut executor = CommandExecutor::new();

    // Run: echo *.txt
    // Verify: outputs matched filenames (space-separated)
}

#[test]
fn test_glob_rm_multiple() {
    let mut executor = CommandExecutor::new();

    // Create test files
    // Run: rm temp-*
    // Verify: all matching files removed
}
```

## Testing Strategy

### Unit Tests (in glob.rs)
- Pattern matching: 15+ tests
  - `*` with various positions and counts
  - `?` single character matching
  - `[abc]` sets and negations
  - Escape sequences
  - Edge cases (empty pattern, special chars)

- Quote handling: 5+ tests
  - Double quotes prevent expansion
  - Single quotes prevent expansion
  - Escaped quotes

### Integration Tests
- Real file matching: 10+ tests
  - Create temp files, run glob, verify results
  - Multiple patterns in one command
  - Nested directories

### Manual Testing
```bash
# Create test files
mkdir -p /tmp/glob-test
cd /tmp/glob-test
touch file1.txt file2.txt file3.md README.md data.csv
touch test1 test2 fileA.txt

# Test cases
ls *.txt          # Should match: file1.txt, file2.txt, fileA.txt
ls *.md           # Should match: file3.md, README.md
ls file?.txt      # Should match: file1.txt, file2.txt (not fileA.txt)
ls [tr]*.txt      # Should match: (nothing - t and r files don't have .txt)
ls file[0-9]*     # Should match: file1.txt, file2.txt, file3.md
ls *[!.txt]       # Should match: test1, test2, fileA.txt (pattern, depends on impl)
echo file\*.txt   # Should output: file*.txt (literal)
```

## Edge Cases to Handle

1. **Path Handling:**
   - Absolute paths: `/tmp/*.txt`
   - Relative paths: `./dir/*.txt`
   - Parent references: `../*.txt`
   - Current dir: `./*.txt`

2. **Special Characters:**
   - Files with spaces: `"file with spaces.txt"`
   - Files starting with `-`
   - Files with glob chars in name: `file[1].txt`

3. **Quote and Escape Interactions:**
   - Single quotes: `'*.txt'` (literal)
   - Double quotes: `"*.txt"` (literal)
   - Escapes: `\*.txt` (literal)
   - Combinations: `"dir/\*.txt"`

4. **Empty and No-Match Cases:**
   - Empty directory
   - Pattern matching zero files
   - Only `.` and `..` in directory

5. **Filesystem Edge Cases:**
   - Symlinks: Follow or skip?
   - Permissions: Skip unreadable dirs?
   - Case sensitivity: Filesystem dependent

## Success Criteria

- [ ] All 5 user stories implemented
- [ ] Unit tests pass (20+ tests)
- [ ] Integration tests pass
- [ ] Pattern matching correct
  - `*` expands properly
  - `?` matches single char
  - `[abc]` and ranges work
  - `[!abc]` negation works
  - Escaping prevents expansion
- [ ] Quote handling correct
  - Quoted patterns not expanded
  - Escaped chars treated as literals
- [ ] No matches returns literal pattern
- [ ] No clippy warnings
- [ ] Formatted with cargo fmt

## Non-Goals

These are explicitly out of scope:
- `**` (recursive glob)
- `extglob` patterns
- POSIX character classes
- Brace expansion
- Performance optimization (beyond reasonable)

## Dependencies

- `std::fs` - Directory traversal
- `std::path` - Path handling
- Existing parser and expansion infrastructure

No new external dependencies required.
