# Data Model: Release Channels

**Feature**: 047-release-channels
**Date**: 2025-12-14

## Overview

This feature involves build tooling and installation, not data persistence. The "entities" are conceptual build-time configurations rather than stored data.

## Entities

### BuildProfile

Compile-time constant indicating build optimization level.

| Attribute | Type | Description |
|-----------|------|-------------|
| name | string | "debug" or "release" |
| debug_assertions | bool | true for debug, false for release |
| stripped | bool | false for debug (has symbols), true for release |

**State Transitions**: None (immutable at compile time)

**Relationships**:
- Determines `LogLevelDefault`
- Displayed in `VersionInfo`

---

### LogLevelDefault

Default logging verbosity based on build profile.

| Attribute | Type | Description |
|-----------|------|-------------|
| level | string | "trace", "debug", "info", "warn", "error" |
| source | string | "compile_time" (from cfg) or "settings" (from settings.json) |

**Derivation Rules**:
- debug_assertions=true → "trace"
- debug_assertions=false → "info"
- User can override via `RSTN_LOG` env var or settings.json

---

### VersionInfo

Runtime version information displayed to user.

| Attribute | Type | Description |
|-----------|------|-------------|
| version | string | Semantic version (e.g., "0.35.0") |
| build_profile | string | "debug" or "release" |
| git_hash | string | Short commit hash (optional) |
| build_date | string | ISO date of build (optional) |

**Display Format**: `{version} ({build_profile})`
**Example**: `0.35.0 (debug)` or `0.35.0 (release)`

---

### InstallationChannel

The method used to install binaries (not stored, conceptual).

| Channel | Command | Build Profile | Install Location |
|---------|---------|---------------|------------------|
| Local Dev | `just install-dev` | debug | `~/.local/bin/` |
| Local Release | `just install` | release | `~/.local/bin/` |
| Homebrew | `brew install rustation` | release | `/opt/homebrew/bin/` (ARM) or `/usr/local/bin/` (Intel) |

---

## Validation Rules

1. **BuildProfile**: Must be exactly "debug" or "release"
2. **LogLevelDefault**: Must be valid tracing level
3. **VersionInfo**: version must be valid semver

## Notes

- No database or file storage involved
- All "data" is compile-time constants or runtime computation
- Settings.json can override log level but defaults come from build profile
