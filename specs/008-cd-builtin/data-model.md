# Data Model: cd Builtin Command

**Feature**: 008-cd-builtin
**Date**: 2025-11-27

## Entities

### Environment Variables (Existing)

The cd builtin uses the existing EnvironmentManager to read and write environment variables. No new data structures are required.

| Variable | Type | Description | Owned By |
|----------|------|-------------|----------|
| HOME | String | User's home directory path | System (inherited) |
| PWD | String | Current working directory | Shell (managed by cd) |
| OLDPWD | String | Previous working directory | Shell (managed by cd) |
| CDPATH | String | Colon-separated search paths | User (optional config) |

### State Transitions

```
┌─────────────────────────────────────────────────────────────┐
│                     cd Command Flow                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Input: cd [path]                                           │
│           │                                                 │
│           ▼                                                 │
│  ┌────────────────┐                                        │
│  │ Parse Argument │                                        │
│  └────────┬───────┘                                        │
│           │                                                 │
│           ▼                                                 │
│  ┌────────────────────────────────────────┐                │
│  │ Resolve Target Path                     │                │
│  │  - No arg → HOME                       │                │
│  │  - "-" → OLDPWD                        │                │
│  │  - "~..." → Expand tilde               │                │
│  │  - Relative → Check cwd, then CDPATH   │                │
│  │  - Absolute → Use directly             │                │
│  └────────┬───────────────────────────────┘                │
│           │                                                 │
│           ▼                                                 │
│  ┌────────────────┐     ┌─────────────────┐                │
│  │ Validate Path  │────▶│ Error: Not found│                │
│  │  - Exists?     │ No  │ or Not a dir    │                │
│  │  - Is dir?     │     └─────────────────┘                │
│  │  - Accessible? │                                        │
│  └────────┬───────┘                                        │
│           │ Yes                                             │
│           ▼                                                 │
│  ┌────────────────────────────────────────┐                │
│  │ Update State                            │                │
│  │  1. OLDPWD = current PWD               │                │
│  │  2. std::env::set_current_dir(target)  │                │
│  │  3. PWD = canonicalized target         │                │
│  └────────┬───────────────────────────────┘                │
│           │                                                 │
│           ▼                                                 │
│  ┌────────────────┐                                        │
│  │ Return 0       │                                        │
│  │ (Success)      │                                        │
│  └────────────────┘                                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Validation Rules

### Path Validation

1. **Existence**: Target path must exist in filesystem
2. **Type**: Target must be a directory (not file, symlink to file, etc.)
3. **Permission**: User must have execute permission on directory

### Environment Variable Validation

1. **HOME**: Must be set for `cd` with no args or tilde expansion
2. **OLDPWD**: Must be set for `cd -`
3. **CDPATH**: Optional; if set, must be colon-separated valid paths

## Relationships

```
EnvironmentManager (existing)
       │
       │ provides get/set for
       │
       ▼
┌──────────────────┐
│  cd Builtin      │
│  ─────────────   │
│  - HOME (read)   │
│  - PWD (r/w)     │
│  - OLDPWD (r/w)  │
│  - CDPATH (read) │
└──────────────────┘
       │
       │ calls
       │
       ▼
┌──────────────────┐
│ std::env         │
│ ─────────────    │
│ set_current_dir  │
└──────────────────┘
```

## No New Data Structures

The cd builtin requires no new structs or enums. It operates purely on:
- String arguments from command line
- Environment variables via EnvironmentManager
- Filesystem via std::path and std::env
