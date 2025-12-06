# Research: Conditional Control Flow

**Feature**: 017-conditional-control-flow
**Date**: 2025-12-06
**Status**: Complete

## Executive Summary

This research covers parser design patterns, POSIX grammar specification, and implementation strategies for if/then/else/elif/fi conditionals in the rush shell. Key decisions: use recursive descent parsing, follow POSIX grammar for compatibility, and implement AST nodes with Box<> for recursive structures.

---

## 1. POSIX Shell Grammar for Conditionals

### Decision
Follow POSIX shell grammar for `if_clause` as defined in IEEE Std 1003.1.

### Rationale
- POSIX compliance ensures script portability
- bash uses POSIX as baseline (SC-001 from spec references bash compatibility)
- Well-documented grammar eliminates ambiguity

### POSIX Grammar (Yacc notation)

From the [POSIX Shell Command Language specification](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html):

```yacc
compound_command : brace_group
                 | subshell
                 | for_clause
                 | case_clause
                 | if_clause
                 | while_clause
                 | until_clause
                 ;

if_clause        : If compound_list Then compound_list else_part Fi
                 | If compound_list Then compound_list           Fi
                 ;

else_part        : Elif compound_list Then compound_list
                 | Elif compound_list Then compound_list else_part
                 | Else compound_list
                 ;

compound_list    : linebreak term
                 | linebreak term separator
                 ;
```

### Key Observations
1. `If`, `Then`, `Elif`, `Else`, `Fi` are reserved words (tokens)
2. `compound_list` can be multiple commands separated by `;` or newlines
3. `else_part` is recursive to support multiple `elif` clauses
4. The condition is itself a `compound_list` (any command sequence)

### Alternatives Considered
- **Fish-style syntax** (`if/else if/end`, no `then`): Rejected for POSIX compatibility
- **Custom syntax**: Rejected - no benefit, adds learning curve

---

## 2. Parser Design Pattern

### Decision
Use recursive descent parsing with a two-phase approach:
1. **Tokenization**: Convert input to token stream with keyword recognition
2. **Parsing**: Recursively build AST from token stream

### Rationale
- Recursive descent maps naturally to grammar productions
- Easy to implement in Rust with enums and match statements
- Current rush parser already uses a similar pattern for pipelines
- No external parser generator needed (keeps Rust-native per constitution)

### Implementation Pattern

Based on [Building a Recursive Descent Parser in Rust](https://jacksontheel.com/posts/building-a-recursive-descent-parser-in-rust/):

```rust
// Token stream with peek capability
struct Parser<'a> {
    tokens: Peekable<impl Iterator<Item = Token>>,
    input: &'a str,
}

impl Parser<'_> {
    fn parse_if_clause(&mut self) -> Result<IfBlock> {
        self.expect(Token::If)?;
        let condition = self.parse_compound_list()?;
        self.expect(Token::Then)?;
        let then_block = self.parse_compound_list()?;
        let else_part = self.parse_else_part()?;
        self.expect(Token::Fi)?;
        Ok(IfBlock { condition, then_block, else_part })
    }

    fn parse_else_part(&mut self) -> Result<Option<ElsePart>> {
        match self.peek() {
            Some(Token::Elif) => { /* recursive elif */ }
            Some(Token::Else) => { /* final else */ }
            Some(Token::Fi) => Ok(None),
            _ => Err(SyntaxError::unexpected(...))
        }
    }
}
```

### Stack Overflow Mitigation
Rust does not perform tail-call optimization. For deeply nested conditionals:
- Use explicit stack instead of recursion if nesting exceeds ~100 levels
- Current spec requires 10 levels - safe for recursion
- Add depth counter with error if exceeded

### Alternatives Considered
- **nom parser combinator**: Overkill for this grammar, adds dependency
- **pest PEG parser**: Good for complex grammars, but adds build complexity
- **Hand-rolled state machine**: Less readable, harder to maintain

---

## 3. AST Structure Design

### Decision
Define AST nodes using Rust enums with Box<> for recursive structures.

### Rationale
- Rust requires Box<> for recursive enum variants
- Enums with data provide type safety
- Matches current `Pipeline` and `PipelineSegment` patterns in codebase

### Proposed Structures

```rust
/// A complete if/elif/else/fi block
pub struct IfBlock {
    /// The condition command(s) - exit code determines branch
    pub condition: CompoundList,
    /// Commands to execute if condition succeeds (exit 0)
    pub then_block: CompoundList,
    /// Optional elif clauses
    pub elif_clauses: Vec<ElifClause>,
    /// Optional else block
    pub else_block: Option<CompoundList>,
}

/// An elif clause within an if block
pub struct ElifClause {
    pub condition: CompoundList,
    pub then_block: CompoundList,
}

/// A sequence of commands (simple commands, pipelines, or nested compounds)
pub struct CompoundList {
    pub commands: Vec<Command>,
}

/// Union of all command types
pub enum Command {
    Simple(Pipeline),           // Existing pipeline execution
    If(Box<IfBlock>),          // Nested conditional
    // Future: While, For, Case, etc.
}
```

### Memory Considerations
- Box<IfBlock> adds one pointer indirection (~8 bytes)
- Vec allocates on heap - appropriate for variable-length elif chains
- No concern for <10MB memory budget with typical scripts

### Alternatives Considered
- **Arena allocation**: Premature optimization for shell scripts
- **Rc<RefCell<>>**: Unnecessary - AST is immutable after parsing
- **Single enum with all fields optional**: Less type-safe

---

## 4. Keyword Recognition

### Decision
Recognize `if`, `then`, `elif`, `else`, `fi` as reserved keywords only in command position.

### Rationale
- POSIX specifies keywords are reserved "when the first word"
- Allows `echo if then else elif fi` to work (words as arguments)
- FR-001 from spec: "when they appear as the first word of a command or after `;`"

### Implementation

```rust
fn is_keyword(word: &str, position: TokenPosition) -> bool {
    if !matches!(position, TokenPosition::CommandStart | TokenPosition::AfterSemicolon) {
        return false;
    }
    matches!(word, "if" | "then" | "elif" | "else" | "fi")
}

enum Token {
    Keyword(Keyword),
    Word(String),
    Pipe,
    Semicolon,
    // ... existing tokens
}

enum Keyword {
    If, Then, Elif, Else, Fi,
    // Future: While, Do, Done, For, In, Case, Esac
}
```

### Edge Cases
- `if=value` - `if` followed by `=` is NOT a keyword (variable assignment)
- `\if` - Escaped keyword is a literal word
- `"if"` - Quoted keyword is a literal word

---

## 5. Exit Code Semantics

### Decision
Follow POSIX exit code conventions:
- Condition: 0 = true (success), non-zero = false (failure)
- Block exit: Exit code of last command in executed branch
- Empty block: Exit code 0

### Rationale
- POSIX standard behavior
- Matches bash reference implementation (SC-001)
- Already used by rush for `$?` variable

### Implementation

```rust
fn execute_if_block(&mut self, block: &IfBlock) -> Result<i32> {
    // Execute condition and check exit code
    let condition_exit = self.execute_compound_list(&block.condition)?;

    if condition_exit == 0 {
        return self.execute_compound_list(&block.then_block);
    }

    // Try elif clauses
    for elif in &block.elif_clauses {
        let elif_exit = self.execute_compound_list(&elif.condition)?;
        if elif_exit == 0 {
            return self.execute_compound_list(&elif.then_block);
        }
    }

    // Fall through to else
    if let Some(ref else_block) = block.else_block {
        return self.execute_compound_list(else_block);
    }

    // No branch executed, return 0
    Ok(0)
}
```

### Edge Cases
- `if false; then; fi` - Empty then block, returns 0
- `if true; then false; fi` - Returns 1 (last command's exit code)
- `if nonexistent-cmd; then echo "yes"; fi` - Condition fails (127), no then execution

---

## 6. Continuation Prompt for Interactive Mode

### Decision
Use `> ` as continuation prompt when if construct is incomplete.

### Rationale
- Common convention (bash, zsh use `> `)
- Clear visual indicator of incomplete input
- Matches PS2 prompt convention

### Implementation Strategy

```rust
// In REPL loop
enum InputState {
    Normal,
    WaitingForIfComplete { depth: usize, buffer: String },
}

fn read_complete_command(&mut self) -> Result<String> {
    let mut buffer = String::new();
    let mut depth = 0;

    loop {
        let prompt = if depth > 0 { "> " } else { "$ " };
        let line = self.editor.read_line(prompt)?;

        buffer.push_str(&line);
        buffer.push('\n');

        // Update depth based on keywords
        depth += count_opens(&line); // if, while, for, case
        depth -= count_closes(&line); // fi, done, esac

        if depth == 0 {
            return Ok(buffer);
        }
    }
}
```

### Alternatives Considered
- **Indented continuation**: More complex, not standard
- **Numbered depth indicator**: `2> ` - Too verbose, unfamiliar

---

## 7. Error Messages

### Decision
Provide specific error messages indicating expected token and location.

### Rationale
- FR-010: "meaningful error messages indicating what was expected"
- SC-002: "clear messages indicating the expected token and location"

### Error Message Format

```text
rush: syntax error near unexpected token 'fi'
rush: syntax error: expected 'then' after 'if' condition
rush: syntax error: 'elif' without preceding 'if'
```

### Implementation

```rust
pub enum SyntaxError {
    UnexpectedToken { found: Token, expected: &'static str, line: usize, col: usize },
    UnmatchedKeyword { keyword: Keyword, line: usize },
    MissingKeyword { expected: Keyword, after: &'static str, line: usize },
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UnexpectedToken { found, expected, .. } =>
                write!(f, "syntax error near unexpected token '{}', expected {}", found, expected),
            // ...
        }
    }
}
```

---

## 8. Integration with Existing Parser

### Decision
Extend current `parse_pipeline()` to detect compound commands and delegate to specialized parsers.

### Rationale
- Minimizes changes to existing working code
- Current parser handles simple commands well
- Compound commands need different parsing strategy

### Integration Points

1. **Tokenizer** (`parser.rs`):
   - Add Keyword variant to Token enum
   - Recognize keywords in command position

2. **Parser Entry** (`parser.rs`):
   - Check first token for compound command keywords
   - Dispatch to `parse_if_clause()` if `If` token
   - Fall through to existing pipeline parsing otherwise

3. **Executor** (`execute.rs`):
   - Match on Command enum
   - Delegate to `conditional.rs` for IfBlock execution

---

## References

- [POSIX Shell Command Language](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [Building a Recursive Descent Parser in Rust](https://jacksontheel.com/posts/building-a-recursive-descent-parser-in-rust/)
- [Bash Grammar BNF](https://www.oreilly.com/library/view/learning-the-bash/1565923472/apds02.html)
- [CS61 Shell Section](https://cs61.seas.harvard.edu/site/2019/Section7/)
