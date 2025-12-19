# Feature 045: Application Logging - Tasks

## Implementation Tasks

### Phase 1: Dependencies & Infrastructure
- [x] Add `tracing-appender = "0.2"` to rstn/Cargo.toml
- [x] Add `flate2 = "1.0"` to rstn/Cargo.toml
- [x] Add `chrono = "0.4"` to rstn/Cargo.toml
- [x] Add `rustation_home()` to rstn-core/paths.rs
- [x] Add `rstn_logs_dir()` to rstn-core/paths.rs
- [x] Add `rstn_log_file()` to rstn-core/paths.rs

### Phase 2: Settings
- [x] Add `logging_enabled` field to Settings struct
- [x] Add `log_level` field to Settings struct
- [x] Add `log_to_console` field to Settings struct
- [x] Add default functions for new fields
- [x] Update Default impl for Settings

### Phase 3: Logging Module
- [x] Create `crates/rstn/src/logging.rs`
- [x] Implement `init()` function
- [x] Implement `rotate_logs()` function
- [x] Implement `compress_file()` function
- [x] Implement `cleanup_old_logs()` function
- [x] Implement `log_file_path()` helper
- [x] Export module in lib.rs

### Phase 4: Integration
- [x] Remove `debug!` macro from main.rs
- [x] Remove `log_to_file!` macro from main.rs
- [x] Initialize logging in main() before Args::parse()
- [x] Replace macros with tracing calls in main.rs
- [x] Remove `log_to_file!` macro from tui/app.rs
- [x] Replace macros with tracing calls in tui/app.rs

### Phase 5: Testing
- [ ] Add unit tests for `log_file_path()`
- [ ] Add unit tests for `compress_file()`
- [ ] Add unit tests for `cleanup_old_logs()`
- [ ] Add integration test for logging initialization
- [ ] Add integration test for log rotation
- [ ] Verify all tests pass

### Phase 6: Documentation & Commit
- [x] Create spec.md
- [x] Create plan.md
- [x] Create tasks.md
- [ ] Git add all changes
- [ ] Git commit with conventional message

## Status
- **Current Phase**: Phase 5 (Testing)
- **Blockers**: None
- **Notes**: Implementation complete, need tests
