# Feature Specification: cd Builtin Command

**Feature Branch**: `008-cd-builtin`
**Created**: 2025-11-27
**Status**: Draft
**Input**: User description: "Implement cd builtin command with support for cd -, cd ~, and CDPATH"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Directory Navigation (Priority: P1)

Users can change their current working directory using the `cd` command with absolute or relative paths, making rush usable for everyday navigation tasks.

**Why this priority**: This is the fundamental use case - without basic cd, users cannot navigate the filesystem, making the shell unusable for most tasks.

**Independent Test**: Run `cd /tmp` and verify the working directory changes to `/tmp`. Run `cd ..` and verify navigation to parent directory.

**Acceptance Scenarios**:

1. **Given** I am in `/home/user`, **When** I run `cd /tmp`, **Then** my working directory becomes `/tmp`
2. **Given** I am in `/home/user/projects`, **When** I run `cd ..`, **Then** my working directory becomes `/home/user`
3. **Given** I am in `/home/user`, **When** I run `cd projects/rush`, **Then** my working directory becomes `/home/user/projects/rush`
4. **Given** I run `cd` with no arguments, **When** the command executes, **Then** my working directory becomes my home directory

---

### User Story 2 - Tilde Expansion (Priority: P1)

Users can use `~` as a shortcut for their home directory in cd commands, providing quick access to home-relative paths.

**Why this priority**: Tilde expansion is essential for efficient navigation and is expected by all shell users. Combined with US1, this provides MVP functionality.

**Independent Test**: Run `cd ~` and verify navigation to home directory. Run `cd ~/Documents` and verify navigation to Documents folder.

**Acceptance Scenarios**:

1. **Given** I am in `/tmp`, **When** I run `cd ~`, **Then** my working directory becomes my home directory (e.g., `/Users/chris`)
2. **Given** I am in `/tmp`, **When** I run `cd ~/Documents`, **Then** my working directory becomes `/Users/chris/Documents`
3. **Given** HOME is set to `/home/testuser`, **When** I run `cd ~`, **Then** my working directory becomes `/home/testuser`

---

### User Story 3 - Previous Directory Navigation (Priority: P2)

Users can quickly return to their previous directory using `cd -`, enabling efficient back-and-forth navigation between two directories.

**Why this priority**: This is a productivity feature that significantly speeds up common workflows but isn't required for basic shell usage.

**Independent Test**: Run `cd /tmp`, then `cd /var`, then `cd -` and verify return to `/tmp`. Verify the previous directory is printed.

**Acceptance Scenarios**:

1. **Given** I navigated from `/home/user` to `/tmp`, **When** I run `cd -`, **Then** my working directory becomes `/home/user` and `/home/user` is printed
2. **Given** I run `cd -` twice, **When** both commands complete, **Then** I return to my original directory
3. **Given** I just started the shell (no previous directory), **When** I run `cd -`, **Then** I see an error message "cd: OLDPWD not set"

---

### User Story 4 - CDPATH Search (Priority: P3)

Users can configure a CDPATH environment variable containing directories to search when cd'ing to a relative path, enabling quick navigation to frequently-used project directories.

**Why this priority**: CDPATH is an advanced feature that power users expect, but most users can work effectively without it.

**Independent Test**: Set `CDPATH=/home/user/projects:/home/user/documents`, then run `cd myproject` and verify it finds `/home/user/projects/myproject`.

**Acceptance Scenarios**:

1. **Given** CDPATH is `/projects:/documents` and `/projects/rush` exists, **When** I run `cd rush`, **Then** my working directory becomes `/projects/rush`
2. **Given** CDPATH is set but current directory has a matching subdirectory, **When** I run `cd subdir`, **Then** the local `./subdir` takes precedence over CDPATH
3. **Given** CDPATH contains multiple paths and the target exists in the second path, **When** I run `cd target`, **Then** the first match in CDPATH order is used
4. **Given** CDPATH is set but no match is found anywhere, **When** I run `cd nonexistent`, **Then** I see an error "cd: nonexistent: No such file or directory"

---

### Edge Cases

- What happens when target directory doesn't exist? → Error message "cd: [path]: No such file or directory"
- What happens when target exists but is not a directory (is a file)? → Error "cd: [path]: Not a directory"
- What happens when user lacks permission to access target? → Error "cd: [path]: Permission denied"
- What happens when HOME is not set and user runs `cd` or `cd ~`? → Error "cd: HOME not set"
- What happens with symlinks? → Follow symlinks (default behavior)
- What happens with `cd ""` (empty string)? → Change to home directory (same as `cd`)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST change the current working directory when given a valid absolute path
- **FR-002**: System MUST change the current working directory when given a valid relative path
- **FR-003**: System MUST change to HOME directory when `cd` is run with no arguments
- **FR-004**: System MUST expand `~` to the HOME environment variable value at the start of paths
- **FR-005**: System MUST track the previous directory in OLDPWD environment variable
- **FR-006**: System MUST change to OLDPWD when `cd -` is run
- **FR-007**: System MUST print the new directory path when `cd -` is used
- **FR-008**: System MUST search CDPATH directories when a relative path doesn't exist in current directory
- **FR-009**: System MUST prioritize current directory over CDPATH matches
- **FR-010**: System MUST display appropriate error messages for invalid paths, permission errors, and missing environment variables
- **FR-011**: System MUST update PWD environment variable after successful directory change

### Key Entities

- **PWD**: Current working directory path, updated after each successful cd
- **OLDPWD**: Previous working directory path, updated before each successful cd
- **HOME**: User's home directory path, used for `~` expansion and `cd` with no args
- **CDPATH**: Colon-separated list of directories to search for relative paths

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can navigate to any accessible directory using absolute or relative paths
- **SC-002**: `cd ~` and `cd` navigate to home directory in under 100ms
- **SC-003**: `cd -` toggles between two directories correctly 100% of the time
- **SC-004**: CDPATH searches complete in under 100ms for up to 10 search paths
- **SC-005**: All error conditions display user-friendly messages that include the problematic path
- **SC-006**: PWD and OLDPWD environment variables are always consistent with actual directory state

## Assumptions

- The shell already has an EnvironmentManager that can get/set PWD, OLDPWD, HOME, and CDPATH
- The shell can execute builtins that modify shell state (already proven with export, set, jobs, fg, bg)
- Standard Unix path conventions apply (/ separator, . for current, .. for parent)
- Symlinks are followed by default (no -P/-L options in MVP)
