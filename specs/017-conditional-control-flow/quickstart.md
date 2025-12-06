# Quickstart: Conditional Control Flow

**Feature**: 017-conditional-control-flow
**Date**: 2025-12-06

## Overview

This guide covers implementation of if/then/else/elif/fi conditionals for the rush shell.

---

## Prerequisites

Before implementing, ensure:

1. **Existing functionality works**:
   ```bash
   cargo test -p rush
   cargo run -p rush -- -c "echo hello"
   ```

2. **Exit status tracking exists**:
   ```bash
   cargo run -p rush
   # In rush:
   true; echo $?    # Should print 0
   false; echo $?   # Should print 1
   ```

3. **Built-in true/false commands exist**:
   ```bash
   cargo run -p rush -- -c "true"   # Exit 0
   cargo run -p rush -- -c "false"  # Exit 1
   ```

---

## Development Workflow

### Step 1: Add AST Structures

Edit `crates/rush/src/executor/mod.rs`:

```rust
// Add after existing structures

#[derive(Debug, Clone, PartialEq)]
pub struct IfBlock {
    pub condition: CompoundList,
    pub then_block: CompoundList,
    pub elif_clauses: Vec<ElifClause>,
    pub else_block: Option<CompoundList>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElifClause {
    pub condition: CompoundList,
    pub then_block: CompoundList,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundList {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Simple(Pipeline),
    If(Box<IfBlock>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    If, Then, Elif, Else, Fi,
}
```

### Step 2: Extend Tokenizer

Edit `crates/rush/src/executor/parser.rs`:

```rust
// Add to Token enum
pub enum Token {
    // ... existing variants
    Keyword(Keyword),
    Semicolon,
    Newline,
}

// Add keyword recognition
fn tokenize_word(word: &str, in_command_position: bool) -> Token {
    if in_command_position {
        if let Some(kw) = Keyword::from_str(word) {
            return Token::Keyword(kw);
        }
    }
    Token::Word(word.to_string())
}
```

### Step 3: Add Conditional Parser

Create `crates/rush/src/executor/conditional.rs`:

```rust
use super::{IfBlock, ElifClause, CompoundList, Command, Keyword, Token};

pub fn parse_if_clause(tokens: &mut Peekable<impl Iterator<Item = Token>>)
    -> Result<IfBlock, SyntaxError>
{
    expect_keyword(tokens, Keyword::If)?;
    let condition = parse_compound_list(tokens)?;
    expect_keyword(tokens, Keyword::Then)?;
    let then_block = parse_compound_list(tokens)?;
    let (elif_clauses, else_block) = parse_else_part(tokens)?;
    expect_keyword(tokens, Keyword::Fi)?;

    Ok(IfBlock { condition, then_block, elif_clauses, else_block })
}
```

### Step 4: Add Conditional Executor

In same file or `execute.rs`:

```rust
impl CommandExecutor {
    pub fn execute_if_block(&mut self, block: &IfBlock) -> Result<i32> {
        let condition_exit = self.execute_compound_list(&block.condition)?;

        if condition_exit == 0 {
            return self.execute_compound_list(&block.then_block);
        }

        for elif in &block.elif_clauses {
            let elif_exit = self.execute_compound_list(&elif.condition)?;
            if elif_exit == 0 {
                return self.execute_compound_list(&elif.then_block);
            }
        }

        if let Some(ref else_block) = block.else_block {
            return self.execute_compound_list(else_block);
        }

        Ok(0)
    }
}
```

---

## Test Cases

### Basic Tests

```bash
# Test 1: Simple if/then/fi with true
cargo run -p rush -- -c 'if true; then echo "yes"; fi'
# Expected: yes

# Test 2: Simple if/then/fi with false
cargo run -p rush -- -c 'if false; then echo "yes"; fi'
# Expected: (no output)

# Test 3: if/then/else/fi
cargo run -p rush -- -c 'if false; then echo "yes"; else echo "no"; fi'
# Expected: no

# Test 4: elif
cargo run -p rush -- -c 'if false; then echo "1"; elif true; then echo "2"; fi'
# Expected: 2

# Test 5: Exit code
cargo run -p rush -- -c 'if true; then false; fi; echo $?'
# Expected: 1
```

### Integration Test File

Create `crates/rush/tests/conditionals.rs`:

```rust
use rush::executor::CommandExecutor;

#[test]
fn test_simple_if_true() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("if true; then echo yes; fi");
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_simple_if_false() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("if false; then echo yes; fi");
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_if_else() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("if false; then echo yes; else echo no; fi");
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_exit_code_propagation() {
    let mut executor = CommandExecutor::new();
    let result = executor.execute("if true; then false; fi");
    assert_eq!(result.unwrap(), 1);
}
```

---

## Debugging Tips

1. **Print tokens during development**:
   ```rust
   let tokens: Vec<Token> = tokenize(input).collect();
   println!("Tokens: {:?}", tokens);
   ```

2. **Print AST**:
   ```rust
   let ast = parse(input)?;
   println!("AST: {:#?}", ast);
   ```

3. **Check parser state on errors**:
   ```rust
   fn expect_keyword(&mut self, kw: Keyword) -> Result<()> {
       match self.peek() {
           Some(Token::Keyword(k)) if *k == kw => { self.next(); Ok(()) }
           other => {
               eprintln!("Expected {:?}, found {:?}", kw, other);
               Err(SyntaxError::MissingKeyword { expected: kw, ... })
           }
       }
   }
   ```

---

## Common Pitfalls

1. **Forgetting semicolon between condition and then**:
   - `if true then` is invalid
   - `if true; then` is correct
   - Parser should require `;` or newline

2. **Keyword recognition outside command position**:
   - `echo if then else` should NOT treat these as keywords
   - Check position before keyword conversion

3. **Exit code of empty blocks**:
   - `if true; then; fi` should return 0
   - Empty CompoundList returns 0

4. **Recursive nesting**:
   - Use `Box<IfBlock>` to avoid infinite size
   - Test 3+ levels of nesting

---

## Performance Verification

After implementation, verify performance:

```bash
# Build in release mode
cargo build --release -p rush

# Time conditional parsing
time ~/.local/bin/rush -c 'if true; then if true; then if true; then echo "nested"; fi; fi; fi'

# Should complete in <10ms
```
