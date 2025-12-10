# Extended Test Command Quick Start

**Feature**: 038-test-command
**Purpose**: Learn `[[ ]]` test command in 5 minutes

## What is `[[ ]]`?

The extended test command `[[ ]]` is a powerful builtin for conditional testing in shell scripts. It's safer and more feature-rich than the traditional `[` command.

**Key Benefits**:
- ✅ No word splitting - variables don't need quotes
- ✅ Pattern matching - glob and regex support
- ✅ Clearer syntax - logical `&&` and `||` operators
- ✅ Better errors - clear messages for mistakes

## Basic Examples

### String Comparison

```bash
# Check if variable equals a value
name="Alice"
[[ $name == "Alice" ]]
echo $?  # 0 (true)

# Check if variable is different
[[ $name != "Bob" ]]
echo $?  # 0 (true)

# No quotes needed! (unlike [ ])
file="my file.txt"
[[ $file == "my file.txt" ]]  # Works without quoting $file
```

### Numeric Comparison

```bash
# Compare numbers
count=10
[[ $count -eq 10 ]]  # Equal
[[ $count -ne 5 ]]   # Not equal
[[ $count -gt 5 ]]   # Greater than
[[ $count -ge 10 ]]  # Greater or equal
[[ $count -lt 20 ]]  # Less than
[[ $count -le 10 ]]  # Less or equal
```

### String Tests

```bash
# Check if string is empty
empty=""
[[ -z $empty ]]
echo $?  # 0 (true)

# Check if string is non-empty
name="Alice"
[[ -n $name ]]
echo $?  # 0 (true)
```

### File Tests

```bash
# Check if file exists
[[ -e /etc/passwd ]]

# Check if regular file
[[ -f /etc/passwd ]]

# Check if directory
[[ -d /tmp ]]

# Check if readable
[[ -r /etc/passwd ]]

# Check if writable
[[ -w /tmp/myfile ]]

# Check if executable
[[ -x /usr/bin/ls ]]

# Check if file is non-empty
[[ -s /var/log/system.log ]]
```

## Pattern Matching

### Glob Patterns

```bash
# Match filenames
filename="document.txt"
[[ $filename == *.txt ]]
echo $?  # 0 (true)

# Match paths
path="/home/user/data"
[[ $path == /home/* ]]
echo $?  # 0 (true)

# Wildcards
# * = zero or more characters
# ? = exactly one character
# [abc] = one character from set

word="cat"
[[ $word == c?t ]]    # true (? matches 'a')
[[ $word == [cbr]at ]] # true ([cbr] matches 'c')
```

### Regex Patterns

```bash
# Validate email format
email="user@example.com"
[[ $email =~ ^[a-z]+@[a-z]+\.[a-z]+$ ]]
echo $?  # 0 (true)

# Validate phone number
phone="555-1234"
[[ $phone =~ ^[0-9]{3}-[0-9]{4}$ ]]
echo $?  # 0 (true)

# Extract parts with BASH_REMATCH
version="v1.2.3"
[[ $version =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]
echo "${BASH_REMATCH[0]}"  # v1.2.3 (full match)
echo "${BASH_REMATCH[1]}"  # 1 (major)
echo "${BASH_REMATCH[2]}"  # 2 (minor)
echo "${BASH_REMATCH[3]}"  # 3 (patch)
```

## Logical Operators

### AND Operator (`&&`)

```bash
# Both conditions must be true
age=25
[[ $age -gt 18 && $age -lt 65 ]]
echo $?  # 0 (true)

# Check file exists AND is readable
file="/etc/passwd"
[[ -f $file && -r $file ]]
```

### OR Operator (`||`)

```bash
# Either condition can be true
mode="dev"
[[ $mode == "dev" || $mode == "test" ]]
echo $?  # 0 (true)

# Check file exists OR directory exists
[[ -f /tmp/file || -d /tmp/dir ]]
```

### NOT Operator (`!`)

```bash
# Negate a condition
[[ ! -f /nonexistent ]]
echo $?  # 0 (true - file doesn't exist)

# Not equal
[[ ! $name == "Bob" ]]
```

### Grouping with Parentheses

```bash
# Override precedence
status="active"
count=5
[[ ( $status == "active" || $status == "pending" ) && $count -gt 0 ]]
```

## Common Patterns

### Safe File Checks

```bash
# Check if file exists and is readable before reading
config="/etc/myapp.conf"
if [[ -f $config && -r $config ]]; then
    source "$config"
else
    echo "Config file not found or not readable"
fi
```

### Input Validation

```bash
# Validate user input
read -p "Enter your age: " age
if [[ $age =~ ^[0-9]+$ && $age -ge 0 && $age -le 150 ]]; then
    echo "Valid age: $age"
else
    echo "Invalid age"
fi
```

### Environment Detection

```bash
# Check environment type
if [[ $ENVIRONMENT == "production" ]]; then
    echo "Running in production mode"
elif [[ $ENVIRONMENT == "staging" || $ENVIRONMENT == "dev" ]]; then
    echo "Running in non-production mode"
else
    echo "Unknown environment"
fi
```

### Extension Checking

```bash
# Process files by extension
for file in *.{txt,md,log}; do
    if [[ $file == *.txt ]]; then
        echo "Text file: $file"
    elif [[ $file == *.md ]]; then
        echo "Markdown file: $file"
    elif [[ $file == *.log ]]; then
        echo "Log file: $file"
    fi
done
```

### URL Validation

```bash
# Basic URL validation
url="https://example.com/path"
if [[ $url =~ ^https?://[a-zA-Z0-9.-]+(/.*)?$ ]]; then
    echo "Valid URL"
else
    echo "Invalid URL"
fi
```

### Version Comparison

```bash
# Parse and compare semantic versions
current="1.2.3"
required="1.1.0"

if [[ $current =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
    major=${BASH_REMATCH[1]}
    minor=${BASH_REMATCH[2]}
    patch=${BASH_REMATCH[3]}

    if [[ $major -gt 1 || ( $major -eq 1 && $minor -ge 1 ) ]]; then
        echo "Version requirement met"
    fi
fi
```

## Tips and Best Practices

### 1. Quotes are Optional for Variables

```bash
# Both work the same:
[[ $var == "value" ]]   # Quotes optional
[[ "$var" == "value" ]] # Quotes explicit

# But quotes ARE needed for literal strings with spaces:
[[ $var == hello world ]]  # ERROR - word splitting
[[ $var == "hello world" ]] # CORRECT
```

### 2. Pattern Must Not Be Quoted

```bash
# WRONG - pattern treated as literal string:
[[ $filename == "*.txt" ]]

# CORRECT - pattern expands:
[[ $filename == *.txt ]]

# WRONG - regex treated as literal:
[[ $email =~ "^[a-z]+@[a-z]+$" ]]

# CORRECT - regex matches:
[[ $email =~ ^[a-z]+@[a-z]+$ ]]
```

### 3. Use Short-Circuit Evaluation

```bash
# Efficient - stops if file doesn't exist:
[[ -f $file && -r $file ]] && cat "$file"

# Efficient - stops if first condition true:
[[ $DEBUG == "true" || $VERBOSE == "true" ]] && set -x
```

### 4. Combine Tests Logically

```bash
# Check multiple conditions clearly:
if [[ -f $file && -s $file && -r $file ]]; then
    echo "File exists, is non-empty, and is readable"
fi
```

### 5. Use String Tests for Empty Checks

```bash
# Preferred:
[[ -z $var ]]  # True if empty
[[ -n $var ]]  # True if non-empty

# Works but less clear:
[[ $var == "" ]]  # True if empty
[[ $var != "" ]]  # True if non-empty
```

## Error Examples

### Syntax Errors

```bash
# Missing closing bracket:
[[ $var == "value"
# Error: missing ']]'

# Invalid operator:
[[ $a === $b ]]
# Error: invalid operator '==='

# Unbalanced parentheses:
[[ ( $a == $b ]]
# Error: unmatched '('
```

### Type Errors

```bash
# Non-numeric operand:
[[ "abc" -eq 123 ]]
# Error: "abc" is not a valid integer

# Invalid regex:
[[ $var =~ "[unclosed" ]]
# Error: invalid regex pattern
```

## Differences from `[ ]`

| Feature | `[[ ]]` | `[ ]` |
|---------|---------|-------|
| Variable quoting | Optional | **Required** |
| Pattern matching | ✅ Yes | ❌ No |
| Regex support | ✅ `=~` | ❌ No |
| Logical AND | `&&` | `-a` |
| Logical OR | `\|\|` | `-o` |
| Word splitting | ❌ Never | ✅ Always |

**Migration Tip**: If you're used to `[ ]`, just remember:
- Use `[[ ]]` instead of `[ ]`
- Change `-a` to `&&` and `-o` to `||`
- You can now skip quotes around variables!

## Quick Reference

### String Operators
- `==` - Equal
- `!=` - Not equal
- `<` - Less than (lexicographic)
- `>` - Greater than (lexicographic)
- `=~` - Regex match
- `-z` - Empty string
- `-n` - Non-empty string

### Numeric Operators
- `-eq` - Equal
- `-ne` - Not equal
- `-lt` - Less than
- `-le` - Less or equal
- `-gt` - Greater than
- `-ge` - Greater or equal

### File Operators
- `-e` - Exists
- `-f` - Regular file
- `-d` - Directory
- `-r` - Readable
- `-w` - Writable
- `-x` - Executable
- `-s` - Non-empty file

### Logical Operators
- `!` - NOT
- `&&` - AND (short-circuit)
- `||` - OR (short-circuit)
- `( )` - Grouping

## Next Steps

- Use `[[ ]]` in your if statements and while loops
- Replace old `[ ]` tests with `[[ ]]` for safety
- Leverage pattern matching for validation
- Explore BASH_REMATCH for parsing structured data

**Documentation**: See [test-api.md](contracts/test-api.md) for complete API specification.
