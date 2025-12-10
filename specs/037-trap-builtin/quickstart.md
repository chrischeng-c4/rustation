# Quickstart: Trap Builtin

**Feature**: Trap Builtin (037)
**Audience**: rush shell users
**Purpose**: Quick examples for common trap command usage patterns

## Basic Usage

### Register a Simple Cleanup Handler

Clean up temporary files when interrupted:

```bash
# Create a cleanup handler for Ctrl+C
trap 'rm /tmp/myapp.lock' INT

# Now when user presses Ctrl+C:
# 1. Handler executes: rm /tmp/myapp.lock
# 2. Shell continues (doesn't terminate)
```

### Multiple Signals, Same Handler

Handle multiple termination signals with one command:

```bash
# Register cleanup for both INT and TERM
trap 'cleanup_function' INT TERM

# Works for Ctrl+C (INT) and kill command (TERM)
```

### EXIT Pseudo-Signal

Always run cleanup when shell exits (any reason):

```bash
trap 'echo "Shell exiting, cleaning up..."; rm /tmp/*' EXIT

# Executes on:
# - Normal exit (typing 'exit' or EOF)
# - Error exit (script failure)
# - Signal termination (killed by another signal)
```

## Common Patterns

### Temporary File Cleanup

```bash
#!/usr/bin/env rush

# Create temp file
TMPFILE=$(mktemp /tmp/myapp.XXXXXX)

# Ensure cleanup on any exit
trap "rm -f $TMPFILE" EXIT

# Do work with temp file
echo "Processing..." > "$TMPFILE"
process_file "$TMPFILE"

# File automatically deleted when script exits
```

### Lock File Management

```bash
#!/usr/bin/env rush

LOCKFILE="/var/run/myapp.lock"

# Create lock file
touch "$LOCKFILE"

# Remove lock on exit or interruption
trap "rm -f $LOCKFILE" EXIT INT TERM

# Critical section - only one instance runs
do_critical_work

# Lock automatically removed on exit
```

### Logging Script Termination

```bash
#!/usr/bin/env rush

LOGFILE="/var/log/myapp.log"

# Log when script terminates
trap 'echo "$(date): Script terminated" >> $LOGFILE' EXIT

# Log interruptions
trap 'echo "$(date): Interrupted by user" >> $LOGFILE' INT

# Script execution...
run_long_task

# Termination logged automatically
```

### Signal-Specific Actions

```bash
#!/usr/bin/env rush

# Different actions for different signals
trap 'echo "Reloading config..."; reload_config' HUP
trap 'echo "Shutting down gracefully..."; shutdown' TERM
trap 'echo "Caught Ctrl+C, cleanup..."; cleanup; exit' INT

# HUP  (kill -HUP $$)  → Reload config
# TERM (kill -TERM $$) → Graceful shutdown
# INT  (Ctrl+C)        → Cleanup and exit
```

## Debugging and Inspection

### List Active Traps

See what signal handlers are currently registered:

```bash
$ trap 'rm /tmp/lock' INT
$ trap 'echo Exiting' EXIT
$ trap
trap -- 'rm /tmp/lock' INT
trap -- 'echo Exiting' EXIT
```

### Check Specific Signal

```bash
$ trap 'cleanup' INT
$ trap | grep INT
trap -- 'cleanup' INT
```

## Dynamic Trap Management

### Clear a Trap Handler

Remove handler and restore default signal behavior:

```bash
# Register handler
$ trap 'cleanup' INT

# Clear handler (empty string)
$ trap "" INT

# Now Ctrl+C terminates shell immediately (default behavior)
```

### Replace a Trap Handler

To replace existing handler, first clear then register:

```bash
# Initial handler
$ trap 'handler1' INT

# Try to replace (ERROR - not allowed)
$ trap 'handler2' INT
trap: trap already exists for signal INT (use 'trap "" INT' to clear first)

# Correct approach: clear then register
$ trap "" INT
$ trap 'handler2' INT
```

## Advanced Patterns

### Nested Cleanup

Chain cleanup operations:

```bash
#!/usr/bin/env rush

cleanup() {
    echo "Cleaning up..."
    stop_services
    remove_temp_files
    close_connections
    echo "Cleanup complete"
}

trap cleanup EXIT INT TERM QUIT
```

### Conditional Cleanup

Skip cleanup if script succeeded:

```bash
#!/usr/bin/env rush

CLEANUP_NEEDED=true

cleanup() {
    if [ "$CLEANUP_NEEDED" = true ]; then
        echo "Cleaning up failed run..."
        rm -rf /tmp/partial_data
    fi
}

trap cleanup EXIT

# Do work
run_task

# Mark success
CLEANUP_NEEDED=false

# EXIT trap still runs but skips cleanup
```

### Real-Time Signals

Use real-time signals for custom IPC:

```bash
#!/usr/bin/env rush

# Custom signal handlers for application-specific events
trap 'echo "Rotate logs"; rotate_logs' RTMIN+1
trap 'echo "Dump stats"; dump_stats' RTMIN+2
trap 'echo "Toggle debug"; toggle_debug' RTMIN+3

# Send signals from other processes:
# kill -s RTMIN+1 $$ → Rotate logs
# kill -s RTMIN+2 $$ → Dump stats
```

### Signal Number Usage

Use signal numbers when names aren't convenient:

```bash
# Signal 2 is SIGINT
trap 'echo "Caught signal 2 (INT)"' 2

# Signal 15 is SIGTERM
trap 'echo "Caught signal 15 (TERM)"' 15
```

## Best Practices

### Always Use Quotes

```bash
# ✅ Good - quotes prevent expansion when registering
trap 'rm $TMPFILE' EXIT

# ❌ Bad - $TMPFILE expanded at registration time
trap "rm $TMPFILE" EXIT
```

### Use Functions for Complex Handlers

```bash
# ✅ Good - readable and testable
cleanup() {
    echo "Cleaning up..."
    rm /tmp/lock
    kill $CHILD_PID
}
trap cleanup EXIT

# ❌ Bad - hard to read and debug
trap 'echo "Cleaning up..."; rm /tmp/lock; kill $CHILD_PID' EXIT
```

### Test Your Trap Handlers

```bash
# Register handler
trap 'echo "Trap executed"' INT

# Test by sending signal to yourself
kill -INT $$

# Output: Trap executed
```

### Combine EXIT with Specific Signals

```bash
# Common pattern: specific signal behavior + general cleanup
trap 'echo "Interrupted"; exit 130' INT    # Ctrl+C exits with 130
trap 'echo "Terminated"; exit 143' TERM    # TERM exits with 143
trap 'cleanup_resources' EXIT              # Always cleanup
```

## Common Pitfalls

### Forgetting to Quote

```bash
# ❌ Wrong - shell expands variables at registration
TMPFILE=/tmp/myfile
trap "rm $TMPFILE" EXIT  # Expands to: trap "rm /tmp/myfile" EXIT

# If you later change TMPFILE:
TMPFILE=/tmp/different
# EXIT still removes /tmp/myfile (not /tmp/different)

# ✅ Correct - single quotes prevent expansion
trap 'rm $TMPFILE' EXIT  # Expands when trap executes
```

### Trying to Trap SIGKILL or SIGSTOP

```bash
# ❌ Error - these signals cannot be caught
$ trap 'cleanup' KILL
trap: cannot trap SIGKILL: signal cannot be caught

$ trap 'cleanup' STOP
trap: cannot trap SIGSTOP: signal cannot be caught

# ✅ Use EXIT for cleanup on all terminations
$ trap 'cleanup' EXIT
```

### Forgetting to Clear Before Replacing

```bash
# ❌ Error - duplicate trap not allowed
$ trap 'old_handler' INT
$ trap 'new_handler' INT
trap: trap already exists for signal INT (use 'trap "" INT' to clear first)

# ✅ Clear first
$ trap "" INT
$ trap 'new_handler' INT
```

## Performance Tips

### Lightweight Handlers

Trap handlers should execute quickly (<100ms):

```bash
# ✅ Good - fast operations
trap 'rm /tmp/lock; exit' INT

# ⚠️ Slow - avoid long-running operations in handlers
trap 'backup_entire_database; compress_logs; send_email' INT
```

### Defer Heavy Work

```bash
# ✅ Good - set flag, do work later
INTERRUPTED=false
trap 'INTERRUPTED=true' INT

# In main loop:
if [ "$INTERRUPTED" = true ]; then
    cleanup_slowly
    exit
fi
```

## Troubleshooting

### Handler Not Executing

Check if trap is registered:

```bash
$ trap
# No output = no traps registered
```

Verify signal name:

```bash
# All these are valid for SIGINT:
trap 'handler' INT
trap 'handler' SIGINT
trap 'handler' int
trap 'handler' 2
```

### Handler Executed Multiple Times

EXIT trap only executes once - check for multiple registrations:

```bash
$ trap
trap -- 'cleanup' EXIT
trap -- 'cleanup' EXIT  # Duplicate registration somehow

# Clear and re-register:
$ trap "" EXIT
$ trap 'cleanup' EXIT
```

## Examples by Use Case

### Web Server Script

```bash
#!/usr/bin/env rush

# PID file for server process
PIDFILE=/var/run/webserver.pid

# Cleanup on exit
cleanup() {
    echo "Shutting down web server..."
    if [ -f "$PIDFILE" ]; then
        kill $(cat "$PIDFILE")
        rm "$PIDFILE"
    fi
}

trap cleanup EXIT INT TERM

# Start server
start_webserver &
echo $! > "$PIDFILE"

# Wait
wait
```

### Data Processing Pipeline

```bash
#!/usr/bin/env rush

INPUT=/data/input.txt
OUTPUT=/data/output.txt
PARTIAL=/tmp/partial.txt

# Cleanup partial data on failure
trap 'rm -f $PARTIAL' EXIT INT

# Process data
process_data "$INPUT" > "$PARTIAL"
validate_data "$PARTIAL"

# Success - move partial to final
mv "$PARTIAL" "$OUTPUT"

# No cleanup needed
trap "" EXIT
```

### Long-Running Daemon

```bash
#!/usr/bin/env rush

RUNNING=true

# Graceful shutdown
shutdown() {
    echo "Shutdown signal received"
    RUNNING=false
}

# Reload configuration
reload() {
    echo "Reload signal received"
    load_config
}

trap shutdown INT TERM
trap reload HUP

# Main loop
while [ "$RUNNING" = true ]; do
    do_work
    sleep 1
done

echo "Daemon stopped"
```

## Next Steps

- Read full specification: [spec.md](spec.md)
- Understand data model: [data-model.md](data-model.md)
- Review API contract: [contracts/trap-api.md](contracts/trap-api.md)
- Implementation plan: [plan.md](plan.md)

## Related Commands

- `kill` - Send signals to processes
- `jobs` - List background jobs
- `fg` / `bg` - Foreground/background job control
- `set` - Set shell options
