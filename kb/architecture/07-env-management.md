---
title: "Environment File Management"
description: "Managing dotfiles across git worktrees"
category: architecture
status: active
last_updated: 2025-12-26
version: 1.0.0
tags: [architecture, env, dotfiles, worktree]
weight: 7
---

# Environment File Management

## 1. Overview

When creating git worktrees, gitignored files (like `.env`) are not included. Env Management solves this by automatically copying dotfiles from the source worktree to new worktrees.

### Problem

```
# Main worktree has .env
~/projects/app/
  +-- .env              # gitignored, contains secrets
  +-- .claude/          # gitignored, Claude config
  +-- src/

# New worktree is missing these files
~/projects/app-feature/
  +-- src/              # Code is there
  +-- (no .env!)        # Must manually copy
  +-- (no .claude!)     # Must manually copy
```

### Solution

rstn automatically copies configured dotfiles when creating worktrees.

---

## 2. Configuration

### EnvConfig (Project-level)

```
EnvConfig
+-- tracked_patterns: Vec<String>    # Files/folders to copy
+-- auto_copy_enabled: bool          # Auto-copy on worktree creation
+-- source_worktree: Option<String>  # Default source (usually main)
+-- last_copy_result: Option<EnvCopyResult>
```

### Default Patterns

```
[".env", ".envrc", ".claude/", ".vscode/"]
```

### Customization

Users can add/remove patterns at the project level:
- Add: `.nvmrc`, `.tool-versions`, `config/local.yml`
- Remove: `.vscode/` if using JetBrains

---

## 3. Copy Strategy

### Copy on Create (Default)

When `auto_copy_enabled` is true:

1. User creates new worktree
2. rstn creates worktree via `git worktree add`
3. rstn copies tracked files from source to new worktree
4. Notification shows copy results

### Copy Behavior

| Source | Destination | Action |
|--------|-------------|--------|
| File exists | File missing | Copy |
| File exists | File exists | Skip (no overwrite) |
| Dir exists | Dir missing | Copy recursively |
| Dir exists | Dir exists | Merge (skip existing) |
| Missing | - | Skip |

### Manual Copy

Users can manually trigger copy via Env page:
- Select source worktree
- Select target worktree
- Choose patterns to copy
- Execute copy

---

## 4. UI Integration

### Env Button (Second Bar)

Located on the right side of the worktree bar:

```
| [main] [feature/auth] [+]                    [Env]  |
```

### Env Page

```
+--------------------------------------------------+
| Environment Files                                 |
+--------------------------------------------------+
| Source: [main (default)]                         |
|                                                   |
| Tracked Patterns:                                |
| [x] .env                                         |
| [x] .envrc                                       |
| [x] .claude/                                     |
| [x] .vscode/                                     |
| [ ] .nvmrc                                       |
| [+ Add pattern]                                  |
|                                                   |
| [ ] Auto-copy on worktree creation              |
|                                                   |
| [Copy to: feature/auth] [Copy Now]              |
+--------------------------------------------------+
```

### Notification (Toast)

After auto-copy:
```
"Copied 3 env files to feature/auth"
```

On error:
```
"Failed to copy .env: Permission denied"
```

---

## 5. Actions

### CopyEnvFiles

```
CopyEnvFiles {
    from_worktree_path: String,
    to_worktree_path: String,
    patterns: Option<Vec<String>>,  // None = use tracked_patterns
}
```

### SetEnvTrackedPatterns

```
SetEnvTrackedPatterns {
    patterns: Vec<String>,
}
```

### SetEnvAutoCopy

```
SetEnvAutoCopy {
    enabled: bool,
}
```

---

## 6. Backend Implementation

### env.rs Module

```
Functions:
- default_patterns() -> Vec<String>
- list_env_files(dir: &str, patterns: &[String]) -> Vec<String>
- copy_env_files(from: &str, to: &str, patterns: &[String]) -> Result<CopyEnvResult>
```

### CopyEnvResult

```
CopyEnvResult
+-- copied_files: Vec<String>           # Successfully copied
+-- failed_files: Vec<(String, String)> # (path, error)
+-- timestamp: String                   # ISO 8601
```

### Integration Point

In `lib.rs`, `Action::AddWorktree` handler:

```
1. Execute git worktree add
2. If env_config.auto_copy_enabled:
   a. Get source worktree (env_config.source_worktree or main)
   b. Call env::copy_env_files()
   c. Dispatch AddNotification with results
3. Refresh worktree list
```

---

## 7. Security Considerations

### Sensitive Files

- `.env` files often contain secrets
- rstn only copies within the same machine
- No network transmission
- No storage outside worktree directories

### Best Practices

- Keep patterns minimal
- Review copied files after worktree creation
- Use `.gitignore` to prevent accidental commits
