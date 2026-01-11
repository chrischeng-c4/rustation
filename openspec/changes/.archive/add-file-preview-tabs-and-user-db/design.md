## Context
We are modernizing the file explorer to match standard IDE expectations (VSCode-like tabs) and centralizing persistence to support a multi-project, single-instance architecture.

## Goals
- **UX**: Provide familiar tab management (Preview vs Pinned).
- **Architecture**: Centralize structured data storage to `~/.rstn/state.db`.
- **Data Isolation**: Ensure data is segmented by `project_id` within the single DB.

## Decisions

### 1. Database Location & Schema
- **Decision**: Use `~/.rstn/state.db` (Linux/Mac) or `%USERPROFILE%\.rstn\state.db` (Windows).
- **Schema Strategy**: Add `project_id` TEXT column to all tables.
- **Project ID**: Use the 8-char hex hash of the project path (same as `persistence.rs` uses for JSON state). This ensures consistency across sessions.
- **Connection**: `DbManager` will open the global DB once. Queries will always require `project_id`.

### 2. Tab State Management
- **Decision**: Manage tab state in Rust (`ExplorerState`), not React local state.
- **Reason**: Maintains "State-First" architecture. Persistence of open tabs (session restore) becomes trivial (just save `ExplorerState`).
- **Structure**:
  ```rust
  struct FileTab {
      path: String,
      is_pinned: bool,
      // Future: scroll_pos: f64
  }
  struct ExplorerState {
      // ... existing fields
      tabs: Vec<FileTab>,
      active_tab_path: Option<String>,
  }
  ```

### 3. Preview Logic
- **Constraint**: Only ONE "Preview" tab can exist at a time.
- **Logic**:
  - `open_file(path)` called:
    - If `path` is already in `tabs`: Set as active.
    - If `path` not in `tabs`:
      - Find existing tab where `is_pinned == false`.
      - If found: Replace it with new path (keep index).
      - If not found: Append new tab with `is_pinned = false`.
      - Set as active.
  - `pin_tab(path)` called:
    - Find tab. Set `is_pinned = true`.

## Risks
- **Data Loss**: We are explicitly NOT migrating old `.rstn/rstn.db` data. This is acceptable for "Fresh Start" per requirements.
- **Concurrency**: Multiple projects (if we ever support multi-window) accessing SQLite. `WAL` mode is already enabled, which handles this well.
