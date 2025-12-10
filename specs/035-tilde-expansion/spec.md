# Feature Specification: Tilde Expansion

**Feature Branch**: `035-tilde-expansion`
**Created**: 2025-12-08
**Status**: Draft
**Input**: User description: "Tilde expansion (~, ~user, ~+, ~-)"

## User Scenarios & Testing

### User Story 1 - Home Directory Shortcut (Priority: P1)

As a shell user, I want to use `~` as a shortcut to my home directory so that I can quickly navigate and reference files without typing the full path.

**Why this priority**: This is the most common use case for tilde expansion. Users expect `~` to work as a home directory alias in all POSIX-compliant shells. This forms the MVP.

**Independent Test**: Can be fully tested by entering `cd ~` and verifying the shell changes to the user's home directory, or by using `ls ~/Documents` to list home directory contents.

**Acceptance Scenarios**:

1. **Given** user is in any directory, **When** user types `cd ~`, **Then** shell changes to user's home directory (value of $HOME)
2. **Given** user types `echo ~`, **When** command executes, **Then** shell prints the user's home directory path
3. **Given** user types `ls ~/Documents`, **When** command executes, **Then** shell expands `~` to home directory and lists Documents folder
4. **Given** `~` appears mid-path like `~/projects/rust-station`, **When** used in any command, **Then** `~` expands to home directory with path appended

---

### User Story 2 - Working Directory Shortcuts (Priority: P2)

As a shell user, I want to use `~+` for current directory and `~-` for previous directory so that I can quickly reference these locations in commands without typing full paths.

**Why this priority**: Common productivity feature in bash/zsh that enhances navigation. Depends on PWD/OLDPWD environment variables already supported in rush.

**Independent Test**: Can be tested by changing directories twice, then using `echo ~+` (shows current) and `echo ~-` (shows previous), verifying correct paths are displayed.

**Acceptance Scenarios**:

1. **Given** user is in `/home/user/projects`, **When** user types `echo ~+`, **Then** shell prints `/home/user/projects` (current PWD)
2. **Given** user was in `/tmp` then changed to `/home/user`, **When** user types `echo ~-`, **Then** shell prints `/tmp` (OLDPWD)
3. **Given** user types `cd ~-`, **When** command executes, **Then** shell changes to the previous directory (OLDPWD)
4. **Given** user types `ls ~+/src`, **When** command executes, **Then** shell lists files in current directory's src subfolder

---

### User Story 3 - Other User's Home Directory (Priority: P3)

As a shell user, I want to use `~username` to reference another user's home directory so that I can access shared files or navigate to team members' directories.

**Why this priority**: Less commonly used feature, typically needed in multi-user systems or team environments. Requires system user lookup which may fail on some systems.

**Independent Test**: Can be tested by typing `echo ~root` (or any valid username) and verifying it expands to that user's home directory path.

**Acceptance Scenarios**:

1. **Given** user `john` exists on system, **When** user types `echo ~john`, **Then** shell prints john's home directory path
2. **Given** user `root` exists, **When** user types `ls ~root`, **Then** shell lists root user's home directory (if permissions allow)
3. **Given** username `nonexistent` does not exist, **When** user types `echo ~nonexistent`, **Then** shell returns literal `~nonexistent` (no expansion)
4. **Given** username contains special chars, **When** user types `~user-name.test`, **Then** shell handles the username correctly (up to first `/` or space)

---

### Edge Cases

- What happens when `HOME` environment variable is not set? (Should leave `~` unexpanded or use system default)
- What happens when `OLDPWD` is not set and user types `~-`? (Should leave `~-` unexpanded)
- What happens with tilde in quoted strings like `'~'` or `"~"`? (Should NOT expand in single quotes, SHOULD expand in double quotes per bash behavior)
- What happens with escaped tilde like `\~`? (Should NOT expand, literal tilde)
- What happens with multiple tildes like `~ ~` or `~/~/file`? (Each tilde expands independently)
- What happens when tilde appears after the first word like `echo hello ~`? (Should still expand - tilde expansion happens before word splitting)
- What happens with `~+` when PWD contains spaces or special characters? (Should expand correctly with proper quoting)
- What happens with ambiguous usernames like `~use` when users `use`, `user`, `username` all exist? (Should use exact match `use`)

## Requirements

### Functional Requirements

- **FR-001**: System MUST expand `~` at the beginning of a word to the user's home directory (value of `$HOME` environment variable)
- **FR-002**: System MUST expand `~+` to the current working directory (value of `$PWD` environment variable)
- **FR-003**: System MUST expand `~-` to the previous working directory (value of `$OLDPWD` environment variable)
- **FR-004**: System MUST expand `~username` to the home directory of the specified user by looking up the system user database
- **FR-005**: System MUST NOT expand tilde when it appears within single quotes (`'~'`)
- **FR-006**: System MUST expand tilde when it appears within double quotes (`"~"`)
- **FR-007**: System MUST NOT expand tilde when it is escaped with backslash (`\~`)
- **FR-008**: System MUST only expand tilde when it appears at the start of a word (before any slash or at word boundary)
- **FR-009**: System MUST preserve literal tilde when username in `~username` does not exist in system user database
- **FR-010**: System MUST handle cases where `HOME`, `PWD`, or `OLDPWD` environment variables are unset by leaving the tilde pattern unexpanded
- **FR-011**: System MUST expand tilde in paths like `~/path/to/file` to `$HOME/path/to/file`
- **FR-012**: System MUST expand `~username/path` to `<user_home>/path` when username exists
- **FR-013**: System MUST integrate tilde expansion into the shell's expansion pipeline at the appropriate stage (after alias expansion, before other expansions)

### Key Entities

- **Tilde Pattern**: A word starting with `~` optionally followed by a username, `+`, `-`, or a path separator (`/`)
- **Expansion Result**: The resolved absolute path that replaces the tilde pattern
- **User Database**: System user information used to resolve `~username` patterns (accessed via standard system APIs)

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can navigate to their home directory using `~` in under 1 second
- **SC-002**: All common tilde expansion patterns (`~`, `~user`, `~+`, `~-`) work correctly in 100% of manual tests
- **SC-003**: Tilde expansion is transparent to users - no visible performance impact compared to typing full paths
- **SC-004**: Edge cases (quoted tildes, nonexistent users, unset variables) are handled gracefully without shell crashes or errors
- **SC-005**: Feature maintains compatibility with bash/zsh tilde expansion behavior for common use cases (95% behavioral parity)

## Assumptions

1. The rush shell already supports `HOME`, `PWD`, and `OLDPWD` environment variables (dependencies 013, 014)
2. System provides standard user database lookup APIs (getpwnam or equivalent)
3. Tilde expansion occurs early in the expansion pipeline, similar to bash (before variable expansion, after alias expansion)
4. Performance impact of user database lookups is acceptable for `~username` patterns (cached or fast enough)
5. The shell runs on POSIX-compatible systems (Linux, macOS, BSD)

## Dependencies

- Feature 013 (cd-builtin): Provides OLDPWD support for `~-` expansion
- Feature 014 (environment-variables): Provides HOME, PWD, OLDPWD variables

## Out of Scope

- Tilde expansion in heredocs or here-strings (may be added later if needed)
- Custom tilde expansion rules or user-defined tilde prefixes
- Tilde expansion for Windows-style paths (C:\~\file) - POSIX-only
- Performance optimization for `~username` lookups (basic implementation first)
- Caching of user database lookups (can be added as optimization later)
