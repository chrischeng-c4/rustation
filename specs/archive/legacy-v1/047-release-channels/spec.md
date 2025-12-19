# Feature Specification: Release Channels

**Feature Branch**: `047-release-channels`
**Created**: 2025-12-14
**Status**: Draft
**Input**: User description: "Set up dual release channels - local development (debug build with trace logging to ~/.local/bin/rstn) and Homebrew release distribution via personal tap"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Local Development Installation (Priority: P1)

As a developer working on rustation, I want to install debug builds with verbose logging to `~/.local/bin` so that I can troubleshoot issues and see detailed execution traces during development.

**Why this priority**: This is the primary daily workflow for active development. Without verbose logging, debugging shell behavior is significantly harder.

**Independent Test**: Can be fully tested by running an installation command and verifying the installed binary has debug symbols and produces trace-level logs.

**Acceptance Scenarios**:

1. **Given** I am in the rustation project directory, **When** I run the development installation command, **Then** debug binaries are installed to `~/.local/bin/rstn` and `~/.local/bin/rush`
2. **Given** a debug build is installed, **When** I run `rstn`, **Then** trace-level logging is enabled by default without requiring additional configuration
3. **Given** a debug build is installed, **When** I check the version, **Then** it clearly indicates this is a debug build (not a release build)

---

### User Story 2 - Homebrew Installation (Priority: P2)

As an end user, I want to install rustation via Homebrew so that I can easily install, update, and manage the shell tools using a familiar package manager.

**Why this priority**: This enables broader distribution to users who are not developers and expect standard package management workflows.

**Independent Test**: Can be fully tested by tapping the Homebrew repository and installing the formula, then verifying both binaries work correctly.

**Acceptance Scenarios**:

1. **Given** I have Homebrew installed, **When** I tap the rustation repository and install, **Then** both `rush` and `rstn` binaries are available in my PATH
2. **Given** I have rustation installed via Homebrew, **When** I run `brew upgrade rustation`, **Then** I receive the latest released version
3. **Given** I installed via Homebrew, **When** I run `rstn`, **Then** it uses release-optimized settings (info-level logging, smaller binary)

---

### User Story 3 - Build Type Identification (Priority: P3)

As a developer or user, I want to easily determine whether my installed binary is a debug or release build so that I can verify my installation type and troubleshoot appropriately.

**Why this priority**: Reduces confusion when users report issues and ensures developers know which build they're running.

**Independent Test**: Can be fully tested by checking version output of both debug and release builds and verifying they display different build type indicators.

**Acceptance Scenarios**:

1. **Given** I have rstn installed, **When** I run `rstn --version`, **Then** the output includes the build type (debug or release)
2. **Given** I want to check my local installation, **When** I run a diagnostic command, **Then** I can see whether debug or release binaries are installed

---

### Edge Cases

- What happens when both debug and release versions exist in different PATH locations?
- How does system handle installation when `~/.local/bin` does not exist?
- What happens when Homebrew installation conflicts with existing `~/.local/bin` installation?

## Requirements *(mandatory)*

### Functional Requirements

**Local Development Channel:**

- **FR-001**: System MUST provide a command to build and install debug binaries to `~/.local/bin`
- **FR-002**: Debug builds MUST include debug symbols (not stripped)
- **FR-003**: Debug builds MUST default to trace-level logging without user configuration
- **FR-004**: Release builds MUST default to info-level logging for production use
- **FR-005**: System MUST provide a command to check which build type is currently installed

**Homebrew Release Channel:**

- **FR-006**: System MUST be installable via Homebrew tap (`chrischeng-c4/homebrew-rustation`)
- **FR-007**: Homebrew formula MUST install both `rush` and `rstn` binaries
- **FR-008**: Homebrew formula MUST build from source using release optimization
- **FR-009**: Homebrew installation MUST pass standard Homebrew formula tests

**Version Information:**

- **FR-010**: Version output MUST display build profile (debug or release)
- **FR-011**: Version output MUST be consistent across both binaries (rush and rstn)

### Key Entities

- **Build Profile**: Indicates whether the binary was compiled with debug or release settings
- **Installation Channel**: The method used to install (local development vs Homebrew)
- **Log Level Default**: The logging verbosity that applies when no explicit configuration is provided

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developer can install debug builds to `~/.local/bin` with a single command
- **SC-002**: Debug builds produce trace-level logs by default (verifiable by running rstn and checking log output)
- **SC-003**: Release builds produce info-level logs by default (quieter output in production)
- **SC-004**: Users can install both rush and rstn via `brew tap chrischeng-c4/rustation && brew install rustation`
- **SC-005**: Version output clearly differentiates debug vs release builds (100% of the time)
- **SC-006**: Homebrew formula passes `brew audit` validation
- **SC-007**: Both installation channels produce working binaries that pass existing test suites

## Assumptions

- Users have Rust toolchain installed for local development builds
- Homebrew users have standard macOS/Linux Homebrew setup
- `~/.local/bin` is in the user's PATH for local development
- The personal Homebrew tap repository will be created at `chrischeng-c4/homebrew-rustation`
- Binary names remain `rush` and `rstn` (no channel-specific suffixes)
