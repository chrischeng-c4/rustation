# Feature 045: Application Logging

## Overview
Add structured application logging to rstn for debugging and troubleshooting.

## Requirements

### Functional Requirements
1. **Log Storage**: Write logs to `~/.rustation/logs/rstn.log`
2. **Log Levels**: Support standard levels (error, warn, info, debug, trace)
3. **Structured Logging**: Use tracing for structured log events with context
4. **Log Rotation**: Rotate logs on application startup
5. **Compression**: Compress rotated logs to `.gz` format
6. **Cleanup**: Auto-delete logs older than 7 days

### Configuration
Logging is controlled via `~/.rustation/settings.json`:

```json
{
  "logging_enabled": true,
  "log_level": "debug",
  "log_to_console": false
}
```

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `logging_enabled` | bool | `true` | Enable/disable file logging |
| `log_level` | string | `"debug"` | Minimum log level |
| `log_to_console` | bool | `false` | Also output to stderr |

### Environment Variables
- `RSTN_LOG`: Override log level filter (e.g., `RSTN_LOG=trace`)

## Non-Functional Requirements
1. **Performance**: Logging should not noticeably impact application startup
2. **Disk Space**: Compression reduces storage by ~90%
3. **Reliability**: Logging failures should not crash the application

## File Structure
```
~/.rustation/
└── logs/
    ├── rstn.log                    # Current log file
    ├── rstn.20241214-163621.log.gz # Compressed rotated log
    └── rstn.20241213-120000.log.gz # Older rotated log
```

## Log Format
```
2025-12-14T08:42:21.095842Z  INFO rstn::logging: crates/rstn/src/logging.rs:85: rstn logging initialized version="0.1.0" log_level=debug
```

## Acceptance Criteria
- [ ] Logs written to ~/.rustation/logs/rstn.log
- [ ] Settings control logging behavior
- [ ] Old logs are rotated and compressed
- [ ] Logs older than 7 days are deleted
- [ ] Unit tests pass
- [ ] Integration tests pass
