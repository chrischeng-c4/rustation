# Feature Specification: Extended Test Command

**Feature Branch**: `038-test-command`
**Created**: 2025-12-10
**Status**: Draft
**Input**: User description: "Extended test command [[ ]] with regex support, pattern matching, and bash-compatible conditional expressions"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Conditional Testing (Priority: P1)

Users need to perform reliable string and numeric comparisons in shell scripts with clearer syntax and fewer quoting issues than the traditional `[` command. The `[[` command provides a more robust testing facility that handles variables without requiring extensive quoting.

**Why this priority**: This is the foundation of the feature - basic conditionals are used in virtually every shell script for control flow decisions. Without this, the feature provides no value.

**Independent Test**: Can be fully tested by writing simple if statements with string/numeric comparisons and verifying they produce correct exit codes. Delivers immediate value for basic scripting needs.

**Acceptance Scenarios**:

1. **Given** a variable with a string value, **When** user tests `[[ "$var" == "expected" ]]`, **Then** command returns exit code 0 for match, 1 for non-match
2. **Given** two numeric variables, **When** user tests `[[ $a -lt $b ]]`, **Then** command correctly compares numeric values
3. **Given** a variable with spaces in its value, **When** user tests `[[ $var == value ]]` without quotes, **Then** command handles it correctly without word splitting errors
4. **Given** an empty variable, **When** user tests `[[ -z $var ]]`, **Then** command returns true (exit 0)
5. **Given** a non-empty variable, **When** user tests `[[ -n $var ]]`, **Then** command returns true (exit 0)

---

### User Story 2 - Pattern Matching and Regex (Priority: P2)

Users need to match strings against patterns (glob-style) and regular expressions for input validation, file filtering, and data processing in scripts. This enables sophisticated string matching without external tools like `grep`.

**Why this priority**: Pattern matching is a common scripting need but can be implemented after basic comparisons. It significantly enhances the test command's utility for real-world scripts.

**Independent Test**: Can be tested independently by writing test cases with pattern matching operators (`==`, `!=` with globs, `=~` with regex) and verifying they match/reject expected patterns. Delivers value for validation and filtering tasks.

**Acceptance Scenarios**:

1. **Given** a filename string, **When** user tests `[[ $filename == *.txt ]]`, **Then** command returns true for .txt files
2. **Given** an email string, **When** user tests `[[ $email =~ ^[a-z]+@[a-z]+\.[a-z]+$ ]]`, **Then** command validates basic email format
3. **Given** a version string, **When** user tests `[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]`, **Then** command validates semantic versioning format
4. **Given** a path string, **When** user tests `[[ $path != /tmp/* ]]`, **Then** command returns false for paths under /tmp

---

### User Story 3 - Complex Conditional Logic (Priority: P3)

Users need to combine multiple test conditions with logical AND (`&&`) and OR (`||`) operators within a single test expression, reducing the need for nested if statements and improving script readability.

**Why this priority**: While useful for complex conditions, most scripts can work with simple conditionals and separate if statements. This is an enhancement for code clarity but not essential for functionality.

**Independent Test**: Can be tested independently by writing complex conditional expressions with multiple clauses joined by && and || operators, verifying correct short-circuit evaluation. Delivers value for cleaner, more maintainable scripts.

**Acceptance Scenarios**:

1. **Given** multiple variables, **When** user tests `[[ $a -gt 0 && $b -lt 100 ]]`, **Then** command evaluates both conditions with AND logic
2. **Given** multiple conditions, **When** user tests `[[ $x == "foo" || $x == "bar" ]]`, **Then** command evaluates with OR logic and short-circuits
3. **Given** mixed conditions, **When** user tests `[[ -f $file && -r $file ]]`, **Then** command verifies file exists and is readable
4. **Given** complex expression, **When** user tests `[[ ( $a -gt 5 && $b -lt 10 ) || $c == "override" ]]`, **Then** command respects grouping with parentheses

---

### Edge Cases

- What happens when regex pattern is invalid or malformed?
- How does system handle unset variables in test expressions?
- What happens when comparing incompatible types (string comparison on numbers, numeric comparison on strings)?
- How are empty strings handled in pattern matching?
- What happens with nested parentheses in complex expressions?
- How does system handle special characters in glob patterns (*, ?, [, ])?
- What happens when testing file attributes on non-existent files?
- How are whitespace and special characters handled in unquoted variables?
- What happens with unicode characters in regex patterns?
- How does system handle very long strings or patterns that might impact performance?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support `[[` and `]]` as test command delimiters
- **FR-002**: System MUST support string equality operators (`==`, `!=`)
- **FR-003**: System MUST support numeric comparison operators (`-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`)
- **FR-004**: System MUST support string test operators (`-z` for empty, `-n` for non-empty)
- **FR-005**: System MUST support file test operators (`-f`, `-d`, `-e`, `-r`, `-w`, `-x`, `-s`)
- **FR-006**: System MUST support glob-style pattern matching with `==` and `!=` operators
- **FR-007**: System MUST support regex pattern matching with `=~` operator
- **FR-008**: System MUST support logical AND operator (`&&`) within test expressions
- **FR-009**: System MUST support logical OR operator (`||`) within test expressions
- **FR-010**: System MUST support negation with `!` operator
- **FR-011**: System MUST support parentheses `( )` for grouping conditions
- **FR-012**: System MUST not perform word splitting on unquoted variables within `[[ ]]`
- **FR-013**: System MUST not perform pathname expansion on unquoted variables within `[[ ]]`
- **FR-014**: System MUST return exit code 0 for true conditions and 1 for false conditions
- **FR-015**: System MUST return exit code 2 for syntax errors or invalid expressions
- **FR-016**: System MUST store regex capture groups in BASH_REMATCH array after successful `=~` match
- **FR-017**: System MUST support string lexicographic comparison operators (`<`, `>`)
- **FR-018**: System MUST short-circuit evaluation for `&&` and `||` operators (stop evaluating when result is determined)

### Key Entities

- **Test Expression**: A conditional expression enclosed in `[[ ]]` that evaluates to true or false
- **Comparison Operator**: String, numeric, or file test operators that compare values
- **Pattern**: Glob-style wildcard pattern or POSIX extended regular expression
- **Logical Operator**: AND (`&&`), OR (`||`), or NOT (`!`) operators for combining conditions
- **BASH_REMATCH**: Array variable containing regex capture groups from successful pattern matches

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can write conditional expressions using `[[ ]]` syntax that correctly evaluate to true or false in under 5 seconds to write
- **SC-002**: String comparisons handle unquoted variables without word splitting errors in 100% of test cases
- **SC-003**: Pattern matching with glob patterns correctly matches or rejects strings in 100% of test scenarios
- **SC-004**: Regex pattern matching validates input strings with 100% accuracy for valid POSIX ERE patterns
- **SC-005**: Complex conditional expressions with multiple clauses evaluate correctly in 100% of test cases
- **SC-006**: Short-circuit evaluation prevents unnecessary condition evaluation in 100% of applicable cases
- **SC-007**: File test operators correctly identify file attributes in 100% of test cases
- **SC-008**: Invalid expressions return appropriate error codes (exit 2) in 100% of error cases

## Dependencies *(mandatory)*

### Feature Dependencies

- **017-conditional-control-flow**: Required for if/then/elif/else where `[[ ]]` is primarily used
- **014-environment-variables**: Required for variable expansion within test expressions
- **010-command-substitution**: Test expressions may contain command substitutions that need evaluation

### Assumptions

- POSIX extended regular expressions (ERE) will be used for `=~` operator (not PCRE)
- Glob pattern matching will follow standard shell glob syntax (*, ?, [...])
- BASH_REMATCH array behavior will match bash's implementation
- File test operators will check actual filesystem attributes, not cached data
- Numeric comparisons will support signed integers within standard shell integer ranges
- String comparisons will be case-sensitive by default (case-insensitive requires separate operators or shopt)

## Constraints *(optional)*

### Technical Constraints

- Regex pattern matching limited to POSIX ERE syntax for compatibility
- Performance target: simple expressions must evaluate in < 1ms, complex expressions in < 10ms
- Regex patterns limited to reasonable length (< 10KB) to prevent ReDoS attacks
- Maximum expression nesting depth of 32 levels to prevent stack overflow

### Compatibility Requirements

- Must maintain bash compatibility for common `[[ ]]` use cases
- Should reject or error on bash-specific features not yet implemented (e.g., `=~` with BASH_REMATCH on right side variables)

## Out of Scope *(optional)*

- Case-insensitive pattern matching operators (may be added in future with `nocasematch` option)
- Advanced regex features beyond POSIX ERE (lookbehinds, atomic groups, etc.)
- Arithmetic evaluation within test expressions (use `(( ))` instead)
- Extended test operators from zsh or other shells not found in bash

## Clarifications *(document decisions made)*

**Decisions made during specification**:

1. **Regex syntax**: Using POSIX Extended Regular Expressions (ERE) as the standard for `=~` operator, matching bash behavior and avoiding PCRE dependency
2. **Word splitting behavior**: Variables within `[[ ]]` will NOT undergo word splitting or pathname expansion, even when unquoted (this is a key advantage over `[` command)
3. **Error handling**: Syntax errors and invalid operators return exit code 2 (distinct from false condition which returns 1)
4. **BASH_REMATCH**: Will be implemented as a regular array variable, populated after successful regex match with full match at index 0 and capture groups at subsequent indices
5. **Performance limits**: Regex patterns capped at 10KB to prevent Regular Expression Denial of Service (ReDoS) attacks
6. **Operator precedence**: Standard precedence: `!` (highest), `-z`/`-n`/file tests, comparison operators, `&&`, `||` (lowest); parentheses for explicit grouping
