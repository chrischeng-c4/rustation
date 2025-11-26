# Research: cd Builtin Command

**Feature**: 008-cd-builtin
**Date**: 2025-11-27

## Research Questions

### 1. How do existing shells implement cd?

**Decision**: Follow POSIX cd behavior with fish/zsh ergonomics

**Rationale**:
- POSIX defines standard cd behavior that users expect
- Fish and zsh add quality-of-life improvements (better errors, cd - printing)
- Consistency with existing shell knowledge reduces learning curve

**Alternatives Considered**:
- Custom cd behavior → Rejected: violates user expectations
- Bash-only behavior → Rejected: misses fish/zsh ergonomic improvements

### 2. How should tilde expansion work?

**Decision**: Expand `~` at the start of path to HOME environment variable

**Rationale**:
- `~` alone → HOME
- `~/path` → HOME/path
- `~user` → NOT SUPPORTED in MVP (would require passwd lookup)
- Expansion happens before any path resolution

**Alternatives Considered**:
- Support `~user` syntax → Deferred: requires passwd parsing, complexity not justified for MVP
- Expand tilde anywhere in path → Rejected: non-standard behavior

### 3. How should CDPATH work?

**Decision**: Search CDPATH only for relative paths that don't exist in current directory

**Rationale**:
- POSIX behavior: CDPATH is colon-separated list of directories
- Local directory takes precedence (implicit `.` at start of CDPATH)
- Print resolved path when CDPATH match used
- Absolute paths bypass CDPATH entirely

**Alternatives Considered**:
- Require explicit `.` in CDPATH → Rejected: violates principle of least surprise
- Search CDPATH first → Rejected: breaks expected local-first behavior

### 4. What standard library functions to use?

**Decision**: Use std::env::set_current_dir and std::path::Path

**Rationale**:
- `std::env::set_current_dir(path)` - Changes process working directory
- `std::path::Path::canonicalize()` - Resolves symlinks and normalizes path
- `std::path::Path::is_dir()` - Validates target is directory
- `std::fs::metadata()` - Check permissions

**Alternatives Considered**:
- nix crate chdir → Rejected: std is sufficient, fewer dependencies
- Manual path resolution → Rejected: std handles edge cases correctly

### 5. Error message format?

**Decision**: Follow bash/zsh error format: `cd: [path]: [error]`

**Rationale**:
- Consistent with user expectations from other shells
- Clear indication of which command failed
- Includes problematic path for debugging

**Examples**:
- `cd: /nonexistent: No such file or directory`
- `cd: /etc/passwd: Not a directory`
- `cd: /root: Permission denied`
- `cd: HOME not set`
- `cd: OLDPWD not set`

## Implementation Notes

### Environment Variable Usage

| Variable | Read | Write | Purpose |
|----------|------|-------|---------|
| HOME | Yes | No | Tilde expansion, `cd` with no args |
| PWD | Yes | Yes | Track current directory |
| OLDPWD | Yes | Yes | Track previous directory for `cd -` |
| CDPATH | Yes | No | Search paths for relative directories |

### Path Resolution Order

1. If path is `-`, use OLDPWD
2. If path is empty or not provided, use HOME
3. Expand tilde if path starts with `~`
4. If path is absolute, use directly
5. If path is relative and exists in cwd, use it
6. If path is relative and CDPATH set, search CDPATH
7. If no match found, error

### Edge Cases Resolved

- Empty string argument → Treat as no argument (go to HOME)
- Path with trailing slash → Strip and process normally
- Symlink targets → Follow symlinks (no -P/-L options in MVP)
- Non-UTF8 paths → Use OsString, convert for display only
