# Feature Specification: Command Aliases

## Overview
Aliases allow users to create shortcuts for commonly used commands. Rush will support defining, listing, and removing aliases using the `alias` and `unalias` builtins.

## User Stories

### US1: Define Aliases
**As a** shell user
**I want** to create command aliases
**So that** I can use short names for frequently used commands

**Acceptance Criteria:**
- `alias name=value` creates an alias
- Alias names can contain letters, numbers, and underscores
- Alias values can be any command string
- Alias values are NOT expanded when defined
- Aliases persist for the session (not saved to file)

**Examples:**
```bash
# Simple alias
$ alias ll='ls -la'
$ ll
[lists files with -la flags]

# Alias with pipes
$ alias lsg='ls | grep'
$ lsg txt
[lists only files matching txt]

# Alias with redirections
$ alias save='echo "data" > output.txt'
$ save
[creates output.txt with "data"]
```

### US2: List Aliases
**As a** shell user
**I want** to view defined aliases
**So that** I know what shortcuts are available

**Acceptance Criteria:**
- `alias` with no arguments lists all aliases
- Format: `alias name='value'`
- Aliases sorted alphabetically
- Empty output if no aliases defined

**Examples:**
```bash
$ alias ll='ls -la'
$ alias lsg='ls | grep'
$ alias
alias ll='ls -la'
alias lsg='ls | grep'
```

### US3: Remove Aliases
**As a** shell user
**I want** to remove aliases
**So that** I can reclaim the name or stop using a shortcut

**Acceptance Criteria:**
- `unalias name` removes an alias
- Error if alias doesn't exist
- Can remove multiple aliases: `unalias name1 name2`
- No wildcard support (explicit names only)

**Examples:**
```bash
$ alias ll='ls -la'
$ unalias ll
$ ll
rush: ll: command not found

$ unalias nonexistent
rush: unalias: nonexistent: not found
```

### US4: Alias Expansion
**As a** shell user
**I want** aliases to be expanded automatically
**So that** I can use them like regular commands

**Acceptance Criteria:**
- Aliases expanded before command execution
- Only the first word is checked for aliases
- Alias expansion happens once (no recursive expansion)
- Arguments after alias are preserved
- Aliases work with pipes, redirections, and other features

**Examples:**
```bash
$ alias ll='ls -la'
$ ll /tmp
[runs: ls -la /tmp]

$ alias g='grep'
$ echo hello | g hello
[runs: echo hello | grep hello]

$ alias ll='ls -la'
$ ll > files.txt
[runs: ls -la > files.txt]
```

## Technical Requirements

### Storage
- Aliases stored in-memory (HashMap<String, String>)
- Session-only (not persisted to file)
- Case-sensitive names

### Parsing
- Alias definition: `alias name=value`
- No spaces around `=`
- Value can be quoted or unquoted
- Quotes in value preserved

### Expansion
- Check first word of command against aliases
- Replace first word with alias value
- Preserve remaining arguments
- No recursive expansion (expanded value not re-checked for aliases)

### Builtins
- `alias` - List all aliases or define new alias
- `alias name=value` - Define alias
- `unalias name` - Remove alias
- `unalias -a` - Remove all aliases (optional enhancement)

## Out of Scope
- Persistent aliases (saving to ~/.rushrc)
- Recursive alias expansion
- Alias functions (aliases that accept arguments with $1, $2)
- Global aliases (aliases that can appear anywhere, not just first word)

## Error Handling
- Invalid alias name: "rush: alias: invalid name"
- Missing `=` in definition: "rush: alias: usage: alias name=value"
- Unalias nonexistent: "rush: unalias: name: not found"

## Testing Strategy
- Unit tests for alias storage (add, get, remove, list)
- Unit tests for alias parsing
- Integration tests for alias command
- Integration tests for unalias command
- Integration tests for alias expansion
- Edge cases: empty value, quotes, special characters
