# Technical Research: Extended Test Command

**Feature**: 038-test-command
**Date**: 2025-12-10
**Purpose**: Document technical decisions and alternatives for `[[ ]]` implementation

## Research Questions

### 1. How should we parse `[[ ]]` expressions?

**Decision**: Dedicated expression parser in `parser/test_expr.rs`

**Rationale**:
- `[[` is a shell keyword, not a simple command
- Requires special parsing rules (no word splitting, no pathname expansion)
- Complex grammar with operators, parentheses, and precedence
- Different tokenization rules than regular commands

**Alternatives Considered**:
1. **Treat as builtin command** - Rejected: Would trigger word splitting before we can prevent it
2. **Parse in main parser** - Rejected: Pollutes command parser with special cases
3. **Dedicated parser** - ✅ Selected: Clean separation, proper precedence handling

**Implementation Approach**:
- Add `TestExpression` variant to AST
- Parse `[[` as keyword in main parser
- Delegate expression parsing to `test_expr::parse()`
- Return `TestExpression` node for evaluation

### 2. How should we handle regex pattern matching?

**Decision**: Use `regex` crate with POSIX ERE syntax

**Rationale**:
- **regex crate**: Industry standard, maintained by Rust team
- POSIX ERE compatibility matches bash `=~` behavior
- Safe from ReDoS attacks (bounded execution time)
- Zero-overhead abstractions via lazy_static for compiled patterns

**Alternatives Considered**:
1. **PCRE2 via pcre2 crate** - Rejected: Heavier dependency, incompatible with bash
2. **Custom regex engine** - Rejected: Reinventing the wheel, security risks
3. **regex crate with ERE** - ✅ Selected: Best balance of safety, performance, compatibility

**Pattern Compilation Strategy**:
- Compile patterns at evaluation time (not parse time)
- Cache compiled patterns per execution (not across commands)
- Return clear error messages for invalid patterns
- Enforce 10KB pattern length limit

**BASH_REMATCH Population**:
```rust
// After successful match:
let captures: Vec<String> = regex_match.iter()
    .map(|m| m.map_or("", |m| m.as_str()).to_string())
    .collect();
executor.variable_manager_mut().set("BASH_REMATCH", captures);
```

### 3. How should we implement glob pattern matching?

**Decision**: Manual glob matching with standard shell glob rules

**Rationale**:
- Simple pattern language: `*`, `?`, `[...]`
- No external dependencies needed
- Full control over edge cases (empty patterns, escaped chars)
- Performance: Direct string iteration faster than regex for globs

**Alternatives Considered**:
1. **glob crate** - Rejected: Designed for filesystem globbing, not string matching
2. **Convert glob to regex** - Rejected: Overcomplicated, harder to debug
3. **Manual implementation** - ✅ Selected: Simple, fast, full control

**Implementation**:
```rust
fn glob_match(pattern: &str, text: &str) -> bool {
    // Recursive matching:
    // * matches zero or more chars
    // ? matches exactly one char
    // [abc] matches one char in set
    // [a-z] matches one char in range
    // Escaped \* matches literal *
}
```

### 4. How should we handle operator precedence and grouping?

**Decision**: Recursive descent parser with precedence climbing

**Rationale**:
- Clear precedence: `!` > comparisons > `&&` > `||`
- Parentheses `( )` override precedence
- Short-circuit evaluation requires left-to-right parsing
- Recursive descent naturally handles nested expressions

**Precedence Table**:
```
Highest:  ! (negation)
          -z, -n, -f, -d, -e, -r, -w, -x, -s (unary tests)
          ==, !=, <, >, =~, -eq, -ne, -lt, -le, -gt, -ge (binary ops)
          && (logical AND)
Lowest:   || (logical OR)
```

**Alternatives Considered**:
1. **Pratt parser** - Rejected: Overkill for simple precedence table
2. **Shunting yard** - Rejected: Harder to implement short-circuit evaluation
3. **Recursive descent** - ✅ Selected: Simple, supports short-circuit naturally

**Short-Circuit Implementation**:
```rust
// For &&: evaluate left, if false return false without evaluating right
// For ||: evaluate left, if true return true without evaluating right
match operator {
    LogicalOp::And => {
        let left = eval(left_expr)?;
        if !left { return Ok(false); }  // Short-circuit
        eval(right_expr)
    }
    LogicalOp::Or => {
        let left = eval(left_expr)?;
        if left { return Ok(true); }  // Short-circuit
        eval(right_expr)
    }
}
```

### 5. How should we handle file test operators?

**Decision**: Use `nix` crate for safe syscall wrappers

**Rationale**:
- `nix` crate already in project (used for signals, process management)
- Safe wrappers around `stat()`, `access()` syscalls
- Cross-platform abstractions (macOS/Linux)
- Type-safe file permission checks

**File Tests Implementation**:
```rust
use nix::sys::stat::{stat, FileStat};
use nix::unistd::access;

fn test_file_exists(path: &str) -> Result<bool> {
    Ok(stat(path).is_ok())
}

fn test_file_readable(path: &str) -> Result<bool> {
    Ok(access(path, AccessFlags::R_OK).is_ok())
}

fn test_file_directory(path: &str) -> Result<bool> {
    stat(path)
        .map(|st| st.st_mode & libc::S_IFMT == libc::S_IFDIR)
        .map_err(|_| RushError::FileTestFailed(path.to_string()))
}
```

**Alternatives Considered**:
1. **std::fs metadata** - Considered: Works but less granular than nix
2. **libc directly** - Rejected: Unsafe, platform-specific
3. **nix crate** - ✅ Selected: Safe, already available, cross-platform

### 6. How should we integrate with existing variable expansion?

**Decision**: Reuse existing expansion pipeline before evaluation

**Rationale**:
- Variables must be expanded: `$var`, `${var}`, etc.
- Command substitution should work: `$(...)`
- But NO word splitting or pathname expansion within `[[ ]]`
- Existing expansion module can be configured per-context

**Integration Strategy**:
```rust
// In test expression evaluation:
1. Take raw tokens from parser
2. Apply variable expansion (but not word splitting/globbing)
3. Evaluate expanded values against operators
```

**Expansion Configuration**:
```rust
let expansion_config = ExpansionConfig {
    word_splitting: false,      // Critical: prevent splitting
    pathname_expansion: false,  // Critical: prevent globbing
    tilde_expansion: true,      // Allow: ~ → /home/user
    variable_expansion: true,   // Allow: $var → value
    command_substitution: true, // Allow: $(cmd) → output
};
```

## Technology Stack Summary

| Component | Technology | Version | Rationale |
|-----------|-----------|---------|-----------|
| Language | Rust | 1.75+ | Core project language |
| Regex | regex crate | latest | POSIX ERE, safe from ReDoS |
| File ops | nix crate | 0.29+ | Already in project, safe syscalls |
| Glob matching | Manual | N/A | Simple, no dependencies |
| Parser | Recursive descent | N/A | Handles precedence naturally |
| Testing | cargo test | N/A | Standard Rust testing |

## Performance Considerations

### Optimization Strategies

1. **Pattern Compilation**:
   - Compile regex patterns once per expression evaluation
   - Don't cache across commands (memory vs speed tradeoff)
   - Enforce pattern size limits (10KB) to prevent ReDoS

2. **Short-Circuit Evaluation**:
   - `&&` and `||` MUST stop evaluating when result determined
   - Prevents unnecessary file tests, regex matches
   - Critical for performance in scripts with complex conditions

3. **File Test Caching**:
   - DON'T cache stat() results (filesystem can change)
   - Accept microsecond syscall overhead for correctness
   - Users can structure expressions to minimize tests

4. **String Allocation**:
   - Use string slices where possible
   - Clone only when storing in BASH_REMATCH
   - Prefer stack allocation for temporary values

### Benchmark Targets

- Simple comparison (`[[ $a == $b ]]`): < 1ms
- Regex match (`[[ $email =~ pattern ]]`): < 10ms
- Complex expression with 5 clauses: < 10ms
- File test operators: < 1ms (syscall overhead)

## Security Considerations

1. **ReDoS Protection**: Pattern length limit (10KB) prevents catastrophic backtracking
2. **Path Traversal**: File tests use provided paths directly (user responsibility)
3. **Variable Injection**: All inputs treated as data, not code
4. **Resource Limits**: Bounded execution time for all operations

## Open Questions

None - all technical decisions resolved during research phase.
