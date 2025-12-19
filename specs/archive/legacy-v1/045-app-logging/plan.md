# Feature 045: Application Logging - Implementation Plan

## Architecture

### Technology Stack
- **tracing**: Structured logging framework
- **tracing-subscriber**: Log collection and formatting
- **tracing-appender**: File output with rolling support
- **flate2**: Gzip compression for rotated logs
- **chrono**: Timestamp generation for rotated file names

### Component Design

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                              │
│  1. Settings::load()                                         │
│  2. logging::init(&settings)                                 │
│  3. tracing::info!("rstn starting")                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       logging.rs                             │
│  - init(): Initialize tracing subscriber                     │
│  - rotate_logs(): Rename current log with timestamp          │
│  - compress_file(): Gzip rotated log in background           │
│  - cleanup_old_logs(): Delete logs older than 7 days         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       settings.rs                            │
│  + logging_enabled: bool                                     │
│  + log_level: String                                         │
│  + log_to_console: bool                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   rstn-core/paths.rs                         │
│  + rustation_home() -> ~/.rustation                          │
│  + rstn_logs_dir() -> ~/.rustation/logs                      │
│  + rstn_log_file() -> ~/.rustation/logs/rstn.log            │
└─────────────────────────────────────────────────────────────┘
```

### Initialization Flow
1. `main()` loads Settings from `~/.rustation/settings.json`
2. `logging::init()` called with settings
3. If `logging_enabled == false`, return early
4. Get log directory from `rstn_core::paths::rstn_logs_dir()`
5. Create directory if needed
6. Call `rotate_logs()` to handle existing log
7. Create `tracing_appender::rolling::never()` writer
8. Build `EnvFilter` from settings or `RSTN_LOG` env var
9. Initialize subscriber with file layer (and optional console layer)
10. Log initial message confirming logging is active

### Log Rotation Flow
1. Check if `rstn.log` exists and has content
2. Generate timestamp: `YYYYMMDD-HHMMSS`
3. Rename to `rstn.{timestamp}.log`
4. Spawn background thread to:
   - Compress to `rstn.{timestamp}.log.gz`
   - Delete uncompressed file
   - Run `cleanup_old_logs()` to remove files older than 7 days

## File Changes

| File | Changes |
|------|---------|
| `crates/rstn/Cargo.toml` | Add tracing-appender, flate2, chrono |
| `crates/rstn-core/src/paths.rs` | Add rustation_home(), rstn_logs_dir(), rstn_log_file() |
| `crates/rstn/src/settings.rs` | Add logging_enabled, log_level, log_to_console |
| `crates/rstn/src/logging.rs` | New module: init, rotation, compression |
| `crates/rstn/src/lib.rs` | Export logging module |
| `crates/rstn/src/main.rs` | Initialize logging, remove old macros |
| `crates/rstn/src/tui/app.rs` | Replace log_to_file! with tracing |

## Testing Strategy

### Unit Tests
- `log_file_path()` returns correct path
- Compression produces valid gzip
- Cleanup removes only old files

### Integration Tests
- Logging creates directory
- Log file is written
- Rotation on startup works
- Compression in background completes
