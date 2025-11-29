# Implementation Tasks: Globbing and Wildcard Expansion

**Feature:** 009-globbing
**Status:** Ready for Implementation
**Estimated Duration:** 8-10 hours
**Target Complexity:** Medium

---

## Task Breakdown

### Phase 1: Core Pattern Matching (3-4 hours)

#### Task 1.1: Create glob.rs Module
- **Objective:** Set up the glob module with basic structure
- **Scope:**
  - Create `/crates/rush/src/executor/glob.rs`
  - Define public `glob_expand()` function signature
  - Add module declaration to `mod.rs`
  - Add basic documentation

**Acceptance Criteria:**
- ✅ File created at correct location
- ✅ Module compiles with no errors
- ✅ Function signatures defined (but not implemented)
- ✅ Module properly exported

**Test Coverage:** Module can be imported and compiled

---

#### Task 1.2: Implement Pattern Matching Algorithm
- **Objective:** Core pattern matching logic for glob patterns
- **Functions to implement:**
  1. `fn pattern_matches(pattern: &str, filename: &str) -> bool`
     - Returns true if filename matches glob pattern
     - Handles `*`, `?`, `[abc]`, `[a-z]`, `[!abc]`

  2. `fn glob_match_recursive(pattern_pos, text_pos) -> bool`
     - Recursive backtracking matcher for `*` wildcards
     - Handles multiple `*` in pattern
     - Ensures `*` doesn't match `/` (path separator)

  3. `fn match_character_set(pattern, pos, ch) -> bool`
     - Handles `[abc]` character sets
     - Handles `[a-z]` ranges
     - Handles `[!abc]` negation
     - Handles `-` as literal character

**Acceptance Criteria:**
- ✅ `pattern_matches("*.txt", "file.txt")` returns true
- ✅ `pattern_matches("*.txt", "file.rs")` returns false
- ✅ `pattern_matches("file?.txt", "file1.txt")` returns true
- ✅ `pattern_matches("file?.txt", "file12.txt")` returns false
- ✅ `pattern_matches("[abc].txt", "a.txt")` returns true
- ✅ `pattern_matches("[a-z].txt", "x.txt")` returns true
- ✅ `pattern_matches("[!abc].txt", "d.txt")` returns true
- ✅ `pattern_matches("*", "path/to/file")` doesn't match `/`

**Test Coverage:** 15+ unit tests for pattern matching

---

#### Task 1.3: Implement Glob Expansion Logic
- **Objective:** Traverse filesystem and expand patterns to matching files
- **Functions to implement:**
  1. `fn expand_single_pattern(pattern: &str) -> Result<Vec<String>>`
     - Separate pattern into directory and filename parts
     - Traverse directory using `std::fs::read_dir()`
     - Match each file against filename pattern
     - Return sorted Vec of matches
     - If no matches, return [pattern] unchanged

  2. `fn should_expand_argument(arg: &str) -> bool`
     - Check if argument contains glob metacharacters
     - Return false for quoted arguments
     - Return true if contains `*`, `?`, `[`

**Acceptance Criteria:**
- ✅ Pattern with matches: expands to matching files
- ✅ Pattern with no matches: returns literal pattern unchanged
- ✅ Files returned in sorted order
- ✅ Handles relative paths: `./dir/*.txt`
- ✅ Handles absolute paths: `/tmp/*.log`
- ✅ Returns proper file paths (not just names)

**Test Coverage:** 10+ unit tests for expansion logic

---

### Phase 2: Integration with Executor (2-3 hours)

#### Task 2.1: Integrate Glob Expansion into execute()
- **Objective:** Call glob expansion at the right point in execution pipeline
- **Changes to `/crates/rush/src/executor/execute.rs`:**
  1. Import glob module: `use super::glob::glob_expand;`
  2. In `execute()` method, after variable expansion, add glob expansion
  3. Call: `let expanded_line = glob_expand(&expanded_line)?;`

**Execution Order:**
```
Original input
    ↓
Variable expansion ($VAR, etc.)
    ↓
Glob expansion (*.txt, etc.) ← ADD HERE
    ↓
Parse pipeline
    ↓
Execute command
```

**Acceptance Criteria:**
- ✅ Glob expansion happens after variable expansion
- ✅ Glob expansion happens before pipeline parsing
- ✅ Variables can be used in glob patterns: `ls $DIR/*.txt`
- ✅ Quoted patterns not expanded: `echo "*.txt"`
- ✅ No performance regression

**Test Coverage:** 5+ integration tests

---

#### Task 2.2: Handle Quote Escaping
- **Objective:** Ensure quoted patterns are not expanded
- **Requirements:**
  1. Single quotes prevent expansion: `echo '*.txt'` → literal `*.txt`
  2. Double quotes prevent expansion: `echo "*.txt"` → literal `*.txt`
  3. Backslash escaping: `echo \*.txt` → literal `*.txt`

**Implementation:**
- Modify `glob_expand()` to parse quoted arguments
- Skip glob expansion for quoted arguments
- Remove quotes from result

**Acceptance Criteria:**
- ✅ Single-quoted patterns not expanded
- ✅ Double-quoted patterns not expanded
- ✅ Escaped metacharacters treated as literal
- ✅ Quotes removed from final output

**Test Coverage:** 5+ quote-handling tests

---

### Phase 3: Comprehensive Testing (2-3 hours)

#### Task 3.1: Unit Tests for Pattern Matching
- **Location:** In `glob.rs` `#[cfg(test)]` section
- **Coverage:**
  - Basic wildcards: `*`, `?`
  - Character sets: `[abc]`, `[a-z]`, `[!abc]`
  - Complex patterns: `file[0-9]?.txt`
  - Edge cases: empty pattern, pattern-only, etc.

**Minimum Tests:** 15

---

#### Task 3.2: Unit Tests for Expansion
- **Location:** In `glob.rs` `#[cfg(test)]` section
- **Coverage:**
  - Single-pattern expansion
  - No-match handling
  - Sorting verification
  - Path handling (relative, absolute)

**Minimum Tests:** 10

---

#### Task 3.3: Integration Tests
- **Location:** `/crates/rush/tests/integration_test.rs`
- **Scenarios:**
  1. `ls *.txt` - Basic wildcard expansion
  2. `echo [abc].txt` - Character set expansion
  3. `echo '*.txt'` - Quoted pattern (no expansion)
  4. `echo \*.txt` - Escaped pattern (no expansion)
  5. `ls nonexistent/*.txt` - No matches (literal)
  6. Variable + glob: `DIR=. && ls $DIR/*.rs`
  7. Multiple patterns: `ls *.txt *.rs`
  8. Mixed quoted/unquoted: `ls *.txt '*.rs'`

**Minimum Tests:** 8

---

#### Task 3.4: Acceptance Testing
- **Manual testing scenarios:**
  1. ✅ `ls *.txt` correctly lists .txt files
  2. ✅ `rm file[0-9].tmp` matches and removes correctly
  3. ✅ `cat doc?.md` works with single-char patterns
  4. ✅ Pattern errors handled gracefully
  5. ✅ Non-matching patterns show literal

**Success Criteria:**
- All manual tests pass
- User workflow matches expectations
- No unexpected errors

---

### Phase 4: Polish and Optimization (1 hour)

#### Task 4.1: Documentation and Comments
- **Add to glob.rs:**
  - Module-level documentation with examples
  - Function documentation with examples
  - Algorithm explanation for complex logic

**Requirements:**
- ✅ Every public function documented
- ✅ Examples showing usage
- ✅ Edge cases documented

---

#### Task 4.2: Error Handling
- **Verify error cases:**
  - Directory not found
  - Permission denied
  - Invalid patterns
  - Filesystem errors

**Implementation:**
- Return proper error messages via `RushError`
- Test error conditions

---

#### Task 4.3: Performance Optimization
- **If needed:**
  - Cache glob results (optional)
  - Optimize filesystem traversal
  - Optimize pattern matching

**Target:** No performance degradation compared to shell

---

## Dependencies

### Internal Dependencies
- ✅ `executor/expansion.rs` - Variable expansion (must happen first)
- ✅ `executor/execute.rs` - Integration point
- ✅ `executor/parser.rs` - Receives expanded input

### External Dependencies
- `std::fs` - Filesystem operations
- `std::path` - Path handling
- `std::io` - Error handling

### No Breaking Changes
- Does not modify existing APIs
- Does not affect other features
- Backwards compatible

---

## Testing Strategy

### Unit Tests (40+ tests)
- Pattern matching logic
- Glob expansion logic
- Quote handling

### Integration Tests (8+ tests)
- End-to-end command execution
- Real filesystem operations
- Quote and escape handling

### Manual Testing
- Real shell usage scenarios
- Performance checks
- Edge case validation

---

## Success Criteria

### Functional
- ✅ All 5 user stories pass acceptance criteria
- ✅ 216+ tests total (208+ existing + 8+ new)
- ✅ No regression in other features

### Quality
- ✅ Code follows project patterns
- ✅ Comprehensive documentation
- ✅ No clippy warnings
- ✅ No unsafe code

### Performance
- ✅ No noticeable slowdown
- ✅ Filesystem operations efficient
- ✅ Pattern matching fast

---

## Rollback Plan

If issues arise:
1. Disable glob expansion in execute.rs
2. Keep glob.rs module for future fixes
3. Restore previous commit if needed

---

## Next Steps After Implementation

1. ✅ Implement all tasks in order
2. ✅ Run full test suite: `cargo test -p rush`
3. ✅ Check with clippy: `cargo clippy -p rush`
4. ✅ Format code: `cargo fmt`
5. ✅ Create commit with implementation
6. ✅ Update spec.md to mark feature complete
7. ✅ Update README.md to show 9 complete features

---

**Estimated Total Time:** 8-10 hours
**Complexity:** Medium
**Risk Level:** Low (isolated module, no breaking changes)

