# Data Model: Extended Test Command

**Feature**: 038-test-command
**Date**: 2025-12-10
**Purpose**: Define core entities for `[[ ]]` test command implementation

## Core Entities

### TestExpression

Represents a complete test expression enclosed in `[[ ]]`.

**Attributes**:
- `expression`: Root expression node (can be nested)
- `source_location`: Position in script for error messages

**Relationships**:
- Contains one Expression (root of expression tree)
- Used by CommandExecutor for evaluation

**Validation**:
- Must have balanced `[[` and `]]` delimiters
- Expression must be non-empty
- Must parse successfully (syntax validation)

**State Transitions**:
```
Parsed → Evaluated → Result (true/false/error)
```

---

### Expression (enum)

Represents a node in the expression tree.

**Variants**:
1. **UnaryOp**: Single operand with operator (e.g., `! expr`, `-z str`, `-f file`)
2. **BinaryOp**: Two operands with operator (e.g., `a == b`, `x -lt y`)
3. **LogicalOp**: Two expressions with `&&` or `||`
4. **Grouped**: Expression wrapped in `( )`
5. **Literal**: String or variable value

**Attributes by Variant**:
- **UnaryOp**: `operator` (UnaryOperator), `operand` (Box<Expression>)
- **BinaryOp**: `left` (String), `operator` (BinaryOperator), `right` (String)
- **LogicalOp**: `left` (Box<Expression>), `operator` (LogicalOperator), `right` (Box<Expression>)
- **Grouped**: `inner` (Box<Expression>)
- **Literal**: `value` (String)

**Relationships**:
- Recursive structure (expressions contain expressions)
- Evaluated by TestEvaluator

---

### UnaryOperator (enum)

Operators that take a single operand.

**Variants**:
- **Negation**: `!` - Logical NOT
- **StringEmpty**: `-z` - True if string is empty
- **StringNonEmpty**: `-n` - True if string is non-empty
- **FileExists**: `-e` - True if file exists
- **FileRegular**: `-f` - True if regular file
- **FileDirectory**: `-d` - True if directory
- **FileReadable**: `-r` - True if readable
- **FileWritable**: `-w` - True if writable
- **FileExecutable**: `-x` - True if executable
- **FileNonEmpty**: `-s` - True if file exists and size > 0

**Usage**: Applied to single operand, returns boolean

---

### BinaryOperator (enum)

Operators that compare two values.

**Variants**:

**String Operators**:
- **StringEqual**: `==` - True if strings equal
- **StringNotEqual**: `!=` - True if strings not equal
- **StringLess**: `<` - True if left < right (lexicographic)
- **StringGreater**: `>` - True if left > right (lexicographic)
- **GlobMatch**: `==` with glob pattern - True if string matches pattern
- **GlobNotMatch**: `!=` with glob pattern - True if string doesn't match
- **RegexMatch**: `=~` - True if string matches regex

**Numeric Operators**:
- **NumericEqual**: `-eq` - True if numbers equal
- **NumericNotEqual**: `-ne` - True if numbers not equal
- **NumericLess**: `-lt` - True if left < right
- **NumericLessEqual**: `-le` - True if left ≤ right
- **NumericGreater**: `-gt` - True if left > right
- **NumericGreaterEqual**: `-ge` - True if left ≥ right

**Behavior**:
- String operators: Compare as strings (case-sensitive)
- Numeric operators: Parse as integers, error if non-numeric
- Glob operators: Pattern on right side, string on left
- Regex operator: POSIX ERE pattern on right, string on left

---

### LogicalOperator (enum)

Operators that combine boolean expressions.

**Variants**:
- **And**: `&&` - True if both expressions true (short-circuit)
- **Or**: `||` - True if either expression true (short-circuit)

**Short-Circuit Behavior**:
- **And**: If left is false, don't evaluate right (return false)
- **Or**: If left is true, don't evaluate right (return true)

**Precedence**: `&&` has higher precedence than `||`

---

### TestEvaluator

Evaluates test expressions to boolean results.

**Attributes**:
- `executor`: Reference to CommandExecutor (for variables, filesystem)

**Methods**:
- `evaluate(expression)`: Returns `Result<bool>` (true/false/error)
- `evaluate_unary(op, operand)`: Handle unary operators
- `evaluate_binary(left, op, right)`: Handle binary operators
- `evaluate_logical(left, op, right)`: Handle logical operators with short-circuit
- `expand_value(token)`: Expand variables in token

**Relationships**:
- Uses CommandExecutor for variable expansion
- Uses filesystem APIs for file tests
- Uses regex crate for pattern matching

**Error Handling**:
- Invalid operator → TestError::InvalidOperator
- Type mismatch (non-numeric for -eq) → TestError::TypeMismatch
- Invalid regex → TestError::InvalidPattern
- File test error → TestError::FileTestFailed

---

### GlobMatcher

Performs shell glob pattern matching on strings.

**Attributes**: None (pure functions)

**Methods**:
- `matches(pattern, text)`: Returns `bool` - true if text matches pattern

**Pattern Rules**:
- `*`: Matches zero or more characters
- `?`: Matches exactly one character
- `[abc]`: Matches one character in set
- `[a-z]`: Matches one character in range
- `[!abc]`: Matches one character NOT in set
- `\*`: Escaped literal asterisk

**Implementation**: Recursive matching algorithm

---

### RegexMatcher

Performs POSIX ERE regex matching.

**Attributes**:
- Compiled regex patterns (created per evaluation)

**Methods**:
- `matches(pattern, text)`: Returns `Result<bool>`
- `get_captures(pattern, text)`: Returns `Result<Vec<String>>` for BASH_REMATCH

**Pattern Rules**:
- POSIX Extended Regular Expression syntax
- Maximum pattern length: 10KB
- Captures stored in order (full match at index 0, groups at 1+)

**Error Handling**:
- Invalid syntax → TestError::InvalidPattern
- Pattern too long → TestError::PatternTooLong

---

### BASH_REMATCH Array

Special variable populated after successful regex match.

**Attributes**:
- Array of strings
- Index 0: Full match
- Index 1+: Capture groups

**Lifecycle**:
1. Cleared before each `=~` evaluation
2. Populated only if match succeeds
3. Persists until next `=~` or shell reset
4. Accessible via `${BASH_REMATCH[0]}`, `${BASH_REMATCH[1]}`, etc.

**Storage**: Stored in CommandExecutor's VariableManager as array variable

---

## Entity Relationships

```
TestExpression
  └─ contains Expression (root)
       ├─ UnaryOp
       │    ├─ operator: UnaryOperator
       │    └─ operand: Expression
       ├─ BinaryOp
       │    ├─ left: String
       │    ├─ operator: BinaryOperator
       │    └─ right: String
       ├─ LogicalOp
       │    ├─ left: Expression
       │    ├─ operator: LogicalOperator
       │    └─ right: Expression
       └─ Grouped
            └─ inner: Expression

TestEvaluator
  ├─ uses CommandExecutor (variable expansion)
  ├─ uses GlobMatcher (glob patterns)
  └─ uses RegexMatcher (regex patterns)

BASH_REMATCH
  └─ stored in VariableManager (CommandExecutor)
```

## Data Flow

### Parsing Flow
```
Source: [[ $var == "value" ]]
  ↓
Lexer: ["[[", "$var", "==", "\"value\"", "]]"]
  ↓
Parser: TestExpression(BinaryOp($var, ==, "value"))
  ↓
AST node returned to executor
```

### Evaluation Flow
```
TestExpression
  ↓
TestEvaluator.evaluate()
  ↓
Expand variables: $var → actual_value
  ↓
Apply operator: actual_value == "value"
  ↓
Return Result<bool>
  ↓
CommandExecutor sets exit code: 0 (true) or 1 (false)
```

### Regex Match Flow
```
[[ $email =~ ^[a-z]+@[a-z]+$ ]]
  ↓
Expand: $email → "user@example.com"
  ↓
Compile regex: ^[a-z]+@[a-z]+$
  ↓
Match: "user@example.com" against pattern
  ↓
Success: Populate BASH_REMATCH[0] = "user@example.com"
  ↓
Return true (exit 0)
```

## Validation Rules

### Syntax Validation
- Balanced delimiters: `[[` must have matching `]]`
- Valid operators: Only supported operators allowed
- Operand counts: Unary ops take 1, binary ops take 2
- Parentheses balanced: Each `(` has matching `)`

### Runtime Validation
- Numeric operators: Operands must parse as integers
- File operators: Paths must be valid (not necessarily exist)
- Regex patterns: Must compile successfully
- Pattern length: Regex patterns ≤ 10KB

### Error Codes
- **Exit 0**: Condition is true
- **Exit 1**: Condition is false
- **Exit 2**: Syntax error, invalid operator, or evaluation failure
