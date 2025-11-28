# Specification: cd Builtin Command

**Feature ID:** 013-cd-builtin
**Status:** Draft
**Created:** 2025-11-28
**Updated:** 2025-11-28

## Overview

Implement the `cd` (change directory) builtin command for the rush shell. The `cd` command is fundamental to shell navigation and must be a builtin (not an external command) because it needs to change the shell process's current working directory.

## Motivation

Every shell requires a `cd` command to navigate the filesystem. Without it, users cannot change directories within the shell session. This is a critical missing feature that prevents rush from being usable as a daily driver shell.

## User Stories

### US1: Change to Absolute Path
**As a** shell user
**I want to** change to an absolute directory path
**So that** I can navigate to any location in the filesystem

**Acceptance Criteria:**
- `cd /tmp` changes current directory to /tmp
- `cd /usr/local/bin` changes to /usr/local/bin
- Invalid paths print error message and return exit code 1
- Current directory remains unchanged on error

**Examples:**
```bash
$ pwd
/home/user
$ cd /tmp
$ pwd
/tmp
$ cd /nonexistent
rush: cd: /nonexistent: No such file or directory
$ pwd
/tmp
```

### US2: Change to Relative Path
**As a** shell user
**I want to** change to a relative directory path
**So that** I can navigate relative to my current location

**Acceptance Criteria:**
- `cd src` changes to ./src if it exists
- `cd ..` moves up one directory level
- `cd ../..` moves up two directory levels
- `cd ./foo/bar` changes to foo/bar relative to current directory

**Examples:**
```bash
$ pwd
/home/user/project
$ cd src
$ pwd
/home/user/project/src
$ cd ..
$ pwd
/home/user/project
$ cd ../..
$ pwd
/home/user
```

### US3: Change to Home Directory
**As a** shell user
**I want to** quickly return to my home directory
**So that** I can easily get back to my home base

**Acceptance Criteria:**
- `cd` with no arguments changes to home directory
- `cd ~` changes to home directory
- Home directory is determined from $HOME environment variable
- Error if $HOME is not set

**Examples:**
```bash
$ pwd
/tmp
$ cd
$ pwd
/home/user
$ cd /tmp
$ cd ~
$ pwd
/home/user
```

### US4: Change to Previous Directory
**As a** shell user
**I want to** toggle between current and previous directory
**So that** I can quickly switch between two locations

**Acceptance Criteria:**
- `cd -` changes to previous directory
- Prints the new current directory path
- Tracks OLDPWD across directory changes
- Error if no previous directory exists

**Examples:**
```bash
$ pwd
/home/user
$ cd /tmp
$ pwd
/tmp
$ cd -
/home/user
$ pwd
/home/user
$ cd -
/tmp
```

## Technical Requirements

### Core Functionality
1. **Directory Resolution:**
   - Expand `~` to home directory
   - Resolve `.` and `..` correctly
   - Handle both absolute and relative paths
   - Follow symlinks by default

2. **Error Handling:**
   - Return exit code 0 on success
   - Return exit code 1 on error
   - Print descriptive error messages to stderr
   - Common errors:
     - Directory doesn't exist
     - Permission denied
     - Not a directory
     - Too many arguments

3. **Environment Variables:**
   - Read $HOME for home directory
   - Update PWD after successful cd
   - Update OLDPWD to previous PWD before changing
   - Handle missing $HOME gracefully

### Implementation Constraints
- Must be a builtin (cannot exec external cd command)
- Must use std::env::set_current_dir() to change process CWD
- Should validate directory exists before changing
- Should update environment variables consistently

## Success Metrics

1. **Functionality:**
   - All 4 user stories pass acceptance tests
   - Error cases handled correctly
   - Edge cases work (permissions, symlinks, etc.)

2. **Compatibility:**
   - Behavior matches bash/zsh for common cases
   - Error messages are clear and helpful
   - Exit codes follow POSIX conventions

3. **Testing:**
   - Unit tests for all user stories
   - Integration tests for directory changes
   - Error path coverage

## Out of Scope

The following features are NOT included in this spec:
- `cd ~username` - expand to other users' home directories
- CDPATH support - searching multiple directories
- Physical vs logical paths (`-P` and `-L` flags)
- Directory stack (pushd/popd) - separate feature
- Custom error handling hooks

These may be added in future iterations if needed.

## Dependencies

- Rust std::env module for CWD manipulation
- Rust std::path for path resolution
- Access to environment variables (HOME, PWD, OLDPWD)

## Timeline

**Estimated Effort:** 2-3 hours
- Specification: 30 min (this document)
- Implementation: 1-1.5 hours
- Testing: 30-45 min
- Documentation: 15-30 min

**Target Completion:** Same session
