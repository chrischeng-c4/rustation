# Implementation Plan: Feature 008 - Shell Aliases

**Feature**: Shell Aliases
**Branch**: `008-aliases`
**Status**: Ready for Implementation

## Architecture Overview

### Components

1. **AliasManager** (`executor/aliases.rs`)
   - In-memory HashMap<String, String> for alias storage
   - Methods: add, remove, get, list, expand
   - Circular alias detection

2. **Alias Builtin** (`executor/builtins/alias.rs`)
   - `alias` command implementation
   - Parse `alias name='value'` syntax
   - Display aliases when called without args

3. **Unalias Builtin** (`executor/builtins/unalias.rs`)
   - `unalias name` command implementation
   - Remove aliases by name

4. **Alias Expansion** (modify `executor/execute.rs`)
   - Expand aliases before command execution
   - Prevent circular expansion

5. **Persistence** (`config.rs` or `aliases.rs`)
   - Save to `~/.config/rush/aliases`
   - Load on shell startup
   - Simple text format: `alias_name='command'`

### Data Flow

```
User Input -> Alias Expansion -> Command Parsing -> Execution
                  |
                  v
            AliasManager
                  |
                  v
            ~/.config/rush/aliases (persistence)
```

## Technical Decisions

1. **Storage Format**: Plain text, one alias per line
   - Format: `alias_name='expanded command'`
   - Easy to edit manually
   - Compatible with bash alias export format

2. **Circular Detection**: Track expansion depth
   - Max 10 levels of expansion
   - Error if exceeded

3. **Alias Priority**: Aliases checked before builtins
   - Matches bash behavior
   - User can override builtins with aliases

4. **Argument Handling**: Simple concatenation
   - `alias ll='ls -la'` + `ll foo` = `ls -la foo`
   - Arguments appended to expanded command

## Estimated Effort

- Phase 1 (Core): 2-3 hours
- Phase 2 (Persistence): 1-2 hours
- Phase 3 (Polish): 1 hour
- **Total**: 4-6 hours
