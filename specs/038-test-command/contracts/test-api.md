# Test Command API Specification

**Feature**: 038-test-command
**Date**: 2025-12-10
**Purpose**: Define the execution API and behavior contracts for `[[ ]]`

## Command Syntax

```bash
[[ expression ]]
```

**Key Properties**:
- `[[` and `]]` are shell keywords (not commands)
- No word splitting or pathname expansion within expression
- Returns exit code based on evaluation result

## Exit Codes

| Exit Code | Meaning | Example |
|-----------|---------|---------|
| 0 | Condition is **true** | `[[ 5 -gt 3 ]]` succeeds |
| 1 | Condition is **false** | `[[ 5 -lt 3 ]]` fails |
| 2 | Syntax error or evaluation failure | `[[ 5 -badop 3 ]]` errors |

## Operator Categories

### 1. String Comparison Operators

#### Equality Operators

**`==`** - String equality (exact match)
```bash
[[ "$var" == "expected" ]]
```
- Returns 0 if strings are identical
- Returns 1 if strings differ
- Case-sensitive comparison

**`!=`** - String inequality
```bash
[[ "$var" != "unexpected" ]]
```
- Returns 0 if strings differ
- Returns 1 if strings are identical

#### Lexicographic Operators

**`<`** - String less than (lexicographic)
```bash
[[ "apple" < "banana" ]]
```
- Returns 0 if left string < right string (alphabetically)
- Returns 1 otherwise
- Case-sensitive: 'A' < 'a' (uppercase before lowercase)

**`>`** - String greater than (lexicographic)
```bash
[[ "banana" > "apple" ]]
```
- Returns 0 if left string > right string (alphabetically)
- Returns 1 otherwise

### 2. Numeric Comparison Operators

All numeric operators parse operands as integers.

**`-eq`** - Numeric equality
```bash
[[ $count -eq 10 ]]
```
- Returns 0 if numbers are equal
- Returns 2 if operands are not valid integers

**`-ne`** - Numeric inequality
```bash
[[ $count -ne 0 ]]
```
- Returns 0 if numbers are not equal

**`-lt`** - Numeric less than
```bash
[[ $value -lt 100 ]]
```
- Returns 0 if left < right (numerically)

**`-le`** - Numeric less than or equal
```bash
[[ $value -le 100 ]]
```
- Returns 0 if left ≤ right

**`-gt`** - Numeric greater than
```bash
[[ $value -gt 0 ]]
```
- Returns 0 if left > right

**`-ge`** - Numeric greater than or equal
```bash
[[ $value -ge 0 ]]
```
- Returns 0 if left ≥ right

### 3. String Test Operators (Unary)

**`-z`** - String is empty
```bash
[[ -z "$var" ]]
```
- Returns 0 if string length is zero
- Returns 1 if string has content

**`-n`** - String is non-empty
```bash
[[ -n "$var" ]]
```
- Returns 0 if string has content
- Returns 1 if string length is zero

### 4. File Test Operators (Unary)

**`-e`** - File exists
```bash
[[ -e /path/to/file ]]
```
- Returns 0 if file exists (any type)
- Returns 1 if file does not exist

**`-f`** - Regular file exists
```bash
[[ -f /path/to/file ]]
```
- Returns 0 if file exists and is a regular file
- Returns 1 otherwise (directory, symlink, non-existent)

**`-d`** - Directory exists
```bash
[[ -d /path/to/dir ]]
```
- Returns 0 if path exists and is a directory
- Returns 1 otherwise

**`-r`** - File is readable
```bash
[[ -r /path/to/file ]]
```
- Returns 0 if file exists and current user has read permission
- Returns 1 otherwise

**`-w`** - File is writable
```bash
[[ -w /path/to/file ]]
```
- Returns 0 if file exists and current user has write permission
- Returns 1 otherwise

**`-x`** - File is executable
```bash
[[ -x /path/to/file ]]
```
- Returns 0 if file exists and current user has execute permission
- Returns 1 otherwise

**`-s`** - File exists and is non-empty
```bash
[[ -s /path/to/file ]]
```
- Returns 0 if file exists and size > 0 bytes
- Returns 1 if file doesn't exist or is empty

### 5. Pattern Matching Operators

**Glob Pattern Matching** - Use `==` or `!=` with wildcard patterns

```bash
[[ "$filename" == *.txt ]]
[[ "$path" != /tmp/* ]]
```

**Pattern Syntax**:
- `*`: Matches zero or more characters
- `?`: Matches exactly one character
- `[abc]`: Matches one character from set
- `[a-z]`: Matches one character from range
- `[!abc]`: Matches one character NOT in set

**Behavior**:
- Pattern must be on RIGHT side of operator
- Pattern is NOT quoted (quoting disables pattern matching)
- Returns 0 if string matches pattern
- Case-sensitive matching

**Regex Pattern Matching** - Use `=~` with POSIX ERE patterns

```bash
[[ "$email" =~ ^[a-z]+@[a-z]+\.[a-z]+$ ]]
```

**Pattern Syntax**: POSIX Extended Regular Expressions
- `.`: Matches any character
- `^`: Anchors to start of string
- `$`: Anchors to end of string
- `*`: Zero or more of preceding element
- `+`: One or more of preceding element
- `?`: Zero or one of preceding element
- `[...]`: Character class
- `(...)`: Capture group
- `|`: Alternation

**Behavior**:
- Regex pattern on RIGHT side
- Pattern must NOT be quoted (quoting treats as literal string)
- Returns 0 if string matches regex
- Populates BASH_REMATCH array on success
- Pattern length limited to 10KB (prevents ReDoS)

**BASH_REMATCH Population**:
```bash
[[ "user@example.com" =~ ^([a-z]+)@([a-z]+)\.([a-z]+)$ ]]
echo "${BASH_REMATCH[0]}"  # Full match: user@example.com
echo "${BASH_REMATCH[1]}"  # First capture: user
echo "${BASH_REMATCH[2]}"  # Second capture: example
echo "${BASH_REMATCH[3]}"  # Third capture: com
```

### 6. Logical Operators

**`!`** - Logical negation (highest precedence)
```bash
[[ ! -f /path/to/file ]]
```
- Returns 0 if expression is false
- Returns 1 if expression is true

**`&&`** - Logical AND (short-circuit)
```bash
[[ -f "$file" && -r "$file" ]]
```
- Evaluates left expression first
- If left is false (exit 1), returns 1 WITHOUT evaluating right
- If left is true, evaluates right and returns its result
- Returns 0 only if BOTH expressions true

**`||`** - Logical OR (short-circuit)
```bash
[[ "$mode" == "dev" || "$mode" == "test" ]]
```
- Evaluates left expression first
- If left is true (exit 0), returns 0 WITHOUT evaluating right
- If left is false, evaluates right and returns its result
- Returns 0 if EITHER expression true

**`( )` - Grouping (override precedence)**
```bash
[[ ( "$a" == "foo" || "$a" == "bar" ) && -f "$file" ]]
```
- Groups expressions to override operator precedence
- Forces evaluation order

## Operator Precedence

From highest to lowest:

1. **Grouping**: `( )`
2. **Negation**: `!`
3. **Unary tests**: `-z`, `-n`, `-f`, `-d`, `-e`, `-r`, `-w`, `-x`, `-s`
4. **Binary comparisons**: `==`, `!=`, `<`, `>`, `=~`, `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`
5. **Logical AND**: `&&`
6. **Logical OR**: `||`

## Variable Expansion Rules

Within `[[ ]]`:
- **YES**: Variable expansion (`$var`, `${var}`)
- **YES**: Command substitution (`$(cmd)`, `` `cmd` ``)
- **YES**: Arithmetic expansion (`$((expr))`)
- **YES**: Tilde expansion (`~`, `~user`)
- **NO**: Word splitting (spaces in variables don't split)
- **NO**: Pathname expansion (globs in variables don't expand)

**Example**:
```bash
var="hello world"
[[ $var == "hello world" ]]  # TRUE - no word splitting
[[ "$var" == "hello world" ]] # Also TRUE - quotes optional
```

## Error Handling

### Syntax Errors (Exit 2)

**Missing closing bracket**:
```bash
[[ $var == "value"
# Error: missing ']]'
```

**Invalid operator**:
```bash
[[ $a === $b ]]
# Error: invalid operator '==='
```

**Wrong operand count**:
```bash
[[ -eq 5 ]]
# Error: -eq requires two operands
```

**Unbalanced parentheses**:
```bash
[[ ( $a == $b ]]
# Error: unmatched '('
```

### Type Errors (Exit 2)

**Non-numeric operand for numeric operator**:
```bash
[[ "abc" -eq 123 ]]
# Error: "abc" is not a valid integer
```

**Invalid regex pattern**:
```bash
[[ $var =~ "[unclosed" ]]
# Error: invalid regex pattern: unclosed bracket
```

**Pattern too long**:
```bash
[[ $var =~ "$(cat huge_pattern.txt)" ]]
# Error: regex pattern exceeds 10KB limit
```

### Evaluation Errors (Exit 2)

**File test on inaccessible path**:
```bash
[[ -f /root/secret ]]
# May return 1 (file doesn't exist or no permission)
# Only returns exit 2 if path is fundamentally invalid
```

## Performance Contracts

### Simple Expressions
- **Target**: < 1ms evaluation time
- **Examples**: `[[ $a == $b ]]`, `[[ $n -gt 0 ]]`, `[[ -f /etc/passwd ]]`

### Regex Matching
- **Target**: < 10ms evaluation time (typical case)
- **Worst Case**: < 200ms (complex patterns)
- **Protection**: 10KB pattern limit prevents ReDoS

### Complex Expressions
- **Target**: < 10ms for 5 clauses with logical operators
- **Short-circuit**: AND/OR stop evaluating early when result determined

## Thread Safety

- **Stateless**: Each test evaluation is independent
- **BASH_REMATCH**: Stored per-executor (not global)
- **File tests**: Use system calls (atomic at OS level)
- **Regex**: Compiled per-evaluation (not shared)

## Compatibility

### Bash Compatibility

**Supported**:
- All operators listed above
- POSIX ERE regex syntax
- BASH_REMATCH array
- Short-circuit evaluation
- No word splitting/pathname expansion

**Not Supported** (may be added in future):
- Case-insensitive matching (requires `shopt -s nocasematch`)
- `=` as alias for `==` (use `==` explicitly)
- Extended glob patterns (`@(...)`, `!(...)`, etc.)

### Differences from `[` Command

| Feature | `[[ ]]` | `[ ]` |
|---------|---------|-------|
| Word splitting | NO | YES |
| Pathname expansion | NO | YES |
| Pattern matching | YES (glob, regex) | NO |
| Logical operators | `&&` and `\|\|` | `-a` and `-o` |
| Regex support | `=~` operator | NO |
| Parentheses grouping | `( )` | `\( \)` (escaped) |
| Quoting required | Optional for variables | Required |

## Usage Examples

See [quickstart.md](../quickstart.md) for comprehensive examples.
