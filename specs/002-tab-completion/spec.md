# Feature Specification: Tab Completion

**Feature Branch**: `002-tab-completion`
**Created**: 2025-11-16
**Status**: Draft
**Input**: User description: "Add tab completion to rush shell that allows users to complete command names, file paths, and basic command flags by pressing Tab."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Command Name Completion (Priority: P1)

As a user typing commands in rush, I want command names to auto-complete when I press Tab so I can type commands faster and discover available commands without leaving the shell.

**Why this priority**: Command completion is the most fundamental tab completion feature. Users expect this in every modern shell. Without it, rush feels incomplete compared to bash/zsh/fish.

**Independent Test**: Can be fully tested by typing partial command names (e.g., `gi<TAB>`) and verifying completions appear from executables in PATH. Delivers immediate value by speeding up common command entry.

**Acceptance Scenarios**:

1. **Given** user types `git st` in the prompt, **When** user presses Tab, **Then** rush completes to `git status`
2. **Given** user types `ca` in the prompt, **When** user presses Tab and multiple matches exist (cat, cargo, cal), **Then** rush displays a menu of matching commands
3. **Given** user types `nonexistent` in the prompt, **When** user presses Tab, **Then** no completion occurs (beep or no action)
4. **Given** user types `pyt` in the prompt, **When** user presses Tab, **Then** rush completes to `python` (or python3 if that's the only match)

---

### User Story 2 - File and Directory Path Completion (Priority: P2)

As a user working with files, I want file and directory paths to auto-complete when I press Tab so I can navigate the filesystem quickly without typing full paths or using `ls` to check names.

**Why this priority**: Path completion is essential for daily file operations. Users constantly reference files/directories and manual typing is error-prone and slow. This is the second most common completion use case after command names.

**Independent Test**: Can be fully tested by typing partial paths (e.g., `ls src/r<TAB>`) and verifying directory/file completions. Works independently of command completion.

**Acceptance Scenarios**:

1. **Given** user types `ls src/re` in the prompt, **When** user presses Tab, **Then** rush completes to `ls src/repl/` (if it's the only match)
2. **Given** user types `cat README` in the prompt, **When** user presses Tab, **Then** rush completes to `cat README.md`
3. **Given** user types `cd ~/proj` in the prompt, **When** user presses Tab and multiple directories match, **Then** rush shows menu of matching directories
4. **Given** user types `cat /etc/hos` in the prompt, **When** user presses Tab, **Then** rush completes to `cat /etc/hosts`
5. **Given** path contains spaces (e.g., `cat "My Doc`), **When** user presses Tab, **Then** rush completes with proper quoting (e.g., `cat "My Documents/"`)

---

### User Story 3 - Flag Completion for Common Commands (Priority: P3)

As a user working with frequently-used commands, I want their flags to auto-complete when I press Tab so I can discover available options and avoid looking up documentation.

**Why this priority**: Flag completion is a quality-of-life improvement that helps with command discovery. It's less critical than basic command/path completion but significantly improves UX for complex commands. This can start with a small set of popular commands.

**Independent Test**: Can be fully tested by typing known commands with partial flags (e.g., `git --<TAB>`) and verifying flag suggestions. Works independently of other completion types.

**Acceptance Scenarios**:

1. **Given** user types `git --` in the prompt, **When** user presses Tab, **Then** rush shows common git flags (--version, --help, etc.)
2. **Given** user types `cargo b` in the prompt, **When** user presses Tab, **Then** rush completes to `cargo build` (subcommand completion)
3. **Given** user types `ls -` in the prompt, **When** user presses Tab, **Then** rush shows common ls flags (-l, -a, -h, -r, -t, etc.)
4. **Given** user types `git commit -m` in the prompt, **When** user presses Tab after the flag, **Then** no completion occurs (string argument expected)

---

### Edge Cases

- What happens when no matches are found? (No action or audible beep, cursor stays in place)
- What happens when there's exactly one match? (Auto-complete immediately)
- What happens when there are hundreds of matches? (Display message like "200+ matches, type more characters" instead of overwhelming menu)
- How does completion handle symlinks? (Follow symlinks when completing paths)
- What happens with hidden files/directories (starting with `.`)? (Show them in completions, especially if user has typed the `.`)
- How does completion handle case sensitivity? (Case-insensitive matching on macOS, case-sensitive on Linux - follow platform conventions)
- What happens when PATH changes during session? (Re-scan PATH when completing commands)
- What happens with partial matches that are also complete valid commands? (e.g., typing `cat` when `cat` and `catch` both exist - show both in menu)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST trigger completion when user presses Tab key
- **FR-002**: System MUST complete command names from executables found in PATH environment variable
- **FR-003**: System MUST complete file and directory paths relative to current working directory
- **FR-004**: System MUST complete absolute paths starting with `/` or `~`
- **FR-005**: System MUST display a menu when multiple completions match the input
- **FR-006**: System MUST automatically complete when exactly one completion matches the input
- **FR-007**: System MUST perform case-insensitive matching for command and path completion on macOS
- **FR-008**: System MUST handle paths with spaces by properly quoting completed paths
- **FR-009**: System MUST complete common flags for well-known commands (git, cargo, ls, cd, cat, echo, grep, find)
- **FR-010**: System MUST limit displayed completions when matches exceed reasonable menu size (suggest threshold: 50 items)
- **FR-011**: System MUST work with zero configuration (no setup files or manual PATH configuration)
- **FR-012**: System MUST support hidden files/directories in path completion
- **FR-013**: System MUST allow user to cycle through menu options using Tab or arrow keys
- **FR-014**: System MUST preserve user's partial input if no completions match

### Key Entities *(data involved)*

- **Command**: Represents an executable program available in PATH
  - Attributes: name, full path to executable
  - Used for command name completion

- **Path Entry**: Represents a file or directory in the filesystem
  - Attributes: name, path, type (file/directory), permissions
  - Used for file/path completion

- **Flag Definition**: Represents known flags for common commands
  - Attributes: command name, flag name, flag description
  - Used for flag/option completion

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Completions appear within 100ms of Tab press for typical scenarios (fewer than 1000 PATH executables, directories with fewer than 1000 entries)
- **SC-002**: Command completion works immediately after rush installation with no configuration
- **SC-003**: Path completion correctly handles 95% of common filesystem scenarios (directories, files, hidden files, paths with spaces)
- **SC-004**: Users can discover and complete at least 20 common flags across git, cargo, and ls commands
- **SC-005**: Completion menu is readable and navigable for up to 50 simultaneous matches
- **SC-006**: No false positives - completion only suggests valid executables, paths, or flags
- **SC-007**: Users can complete common workflows (e.g., `git status`, `cargo build`, `ls -la`) 50% faster than typing manually

## Assumptions

- User's PATH environment variable contains standard executable directories
- Filesystem is readable and accessible (permissions allow directory listing)
- Terminal supports standard key bindings (Tab key sends expected control sequence)
- Most common commands users need completion for are: git, cargo, npm, ls, cd, cat, grep, find
- Users prefer case-insensitive matching on macOS (common convention)
- Completion menus with >50 items are overwhelming and should be limited

## Out of Scope

The following are explicitly NOT included in this feature:

- **Custom completion definitions**: Users cannot define their own completion rules (deferred to future configuration system)
- **Context-aware argument completion**: Completing specific arguments based on command semantics (e.g., git branch names, cargo package names)
- **Fuzzy matching**: Only prefix matching is supported (e.g., `gts` won't match `git status`)
- **Completion preview/documentation**: No inline help text or flag descriptions in completion menu
- **Remote path completion**: Completing paths on remote systems (ssh, URLs)
- **History-based completion**: Suggesting completions from command history
- **Alias completion**: Completing user-defined aliases (requires alias feature first)
