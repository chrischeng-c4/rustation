# Specification: Stderr Redirection

**Feature**: 010-stderr-redirection
**Status**: Draft
**Priority**: P1 (Essential)
**Created**: 2025-11-27

## Overview

Extend rush shell redirection to support stderr (file descriptor 2) and combined output redirections, completing POSIX-compliant I/O redirection support.

## Problem Statement

Currently rush only supports stdout redirection (`>`, `>>`). Users cannot redirect error output separately or combine streams, which is essential for scripting and error handling.

## User Stories

### US1: Stderr Redirection (Priority: P1)

**As a** shell user
**I want** to redirect stderr to a file
**So that** I can capture or suppress error messages

**Acceptance Criteria:**
- `cmd 2> errors.txt` redirects stderr to file (overwrite)
- `cmd 2>> errors.txt` appends stderr to file
- Stdout remains on terminal when only stderr is redirected
- File is created if it doesn't exist

### US2: Stderr to Stdout (Priority: P1)

**As a** shell user
**I want** to redirect stderr to stdout
**So that** I can process all output together in pipelines

**Acceptance Criteria:**
- `cmd 2>&1` redirects stderr to wherever stdout goes
- Works with pipes: `cmd 2>&1 | grep error`
- Order matters: `cmd > file 2>&1` captures both to file
- `cmd 2>&1 > file` sends stderr to terminal, stdout to file

### US3: Combined Output (Priority: P2)

**As a** shell user
**I want** shorthand syntax for redirecting both streams
**So that** I can capture all output easily

**Acceptance Criteria:**
- `cmd &> file` redirects both stdout and stderr to file
- `cmd &>> file` appends both streams to file
- Equivalent to `cmd > file 2>&1`

## Functional Requirements

### FR1: New Redirection Operators
- `2>` - Redirect stderr to file (overwrite)
- `2>>` - Redirect stderr to file (append)
- `2>&1` - Redirect stderr to stdout
- `&>` - Redirect both stdout and stderr to file
- `&>>` - Append both stdout and stderr to file

### FR2: File Descriptor Semantics
- FD 1 = stdout (default for `>`)
- FD 2 = stderr
- `n>` syntax where n is a digit redirects that fd

### FR3: Redirection Order
- Redirections are processed left-to-right
- `cmd > out.txt 2>&1` - both to out.txt
- `cmd 2>&1 > out.txt` - stderr to terminal, stdout to out.txt

### FR4: Error Handling
- Invalid fd numbers produce error
- Missing file path after operator produces error
- Permission denied handled gracefully

## Non-Functional Requirements

### NFR1: Compatibility
- Match bash/zsh stderr redirection semantics
- Existing stdout redirection must continue working

### NFR2: Performance
- No performance regression for simple commands
- Redirections set up before fork/exec

## Out of Scope

- Here documents (`<<`)
- Process substitution (`<()`, `>()`)
- File descriptor closing (`2>&-`)
- Arbitrary fd numbers beyond 0-2

## Technical Notes

### Parser Changes
Add new tokens:
- `RedirectStderr` (2>)
- `RedirectStderrAppend` (2>>)
- `RedirectStderrToStdout` (2>&1)
- `RedirectBoth` (&>)
- `RedirectBothAppend` (&>>)

### Execution Changes
Modify pipeline execution to:
1. Handle fd 2 redirections
2. Support dup2() for 2>&1
3. Process redirections in order

## Test Scenarios

1. `ls /nonexistent 2> err.txt` - stderr captured, stdout empty
2. `ls /nonexistent 2>&1 | wc -l` - error in pipeline
3. `echo hello &> all.txt` - both streams to file
4. `cmd > out.txt 2>&1` vs `cmd 2>&1 > out.txt` - order matters
