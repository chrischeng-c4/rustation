# Claude Code Headless Mode

> Source: https://code.claude.com/docs/en/headless

## Overview

Headless mode runs Claude Code programmatically without interactive UI.

## Basic Usage

```bash
claude -p "prompt" \
  --allowedTools "Bash,Read" \
  --permission-mode acceptEdits
```

## Output Formats

### Text (Default)
```bash
claude -p "Explain file src/main.rs"
```

### JSON
```bash
claude -p "query" --output-format json
```

Response structure:
```json
{
  "type": "result",
  "subtype": "success",
  "total_cost_usd": 0.003,
  "is_error": false,
  "duration_ms": 1234,
  "duration_api_ms": 800,
  "num_turns": 6,
  "result": "The response text...",
  "session_id": "abc123"
}
```

### Streaming JSON
```bash
claude -p "query" --output-format stream-json
```

Emits:
1. Initial `init` system message
2. User and assistant messages (as they stream)
3. Final `result` system message with stats

## Session Management

### Get Session ID from JSON Output
```bash
session_id=$(claude -p "start task" --output-format json | jq -r '.session_id')
```

### Continue Session
```bash
# Most recent
claude --continue "follow up"

# By session ID
claude --resume "$session_id" "update tests"

# Resume in print mode
claude --resume "$session_id" -p "fix issues"
```

## Multi-turn Conversations

```bash
# Start session
session_id=$(claude -p "Start review" --output-format json | jq -r '.session_id')

# Continue with context
claude -p --resume "$session_id" "Review first file"
claude -p --resume "$session_id" "Check compliance"
claude -p --resume "$session_id" "Generate summary"
```

## MCP Integration

```bash
claude -p "query" \
  --mcp-config servers.json \
  --allowedTools "Bash,Read,mcp__myserver"
```

## Best Practices

1. **Use JSON output for programmatic parsing**
2. **Handle errors gracefully** - check exit codes
3. **Use session management** for multi-turn context
4. **Consider timeouts** for long operations:
   ```bash
   timeout 300 claude -p "$prompt" || echo "Timed out"
   ```
5. **Respect rate limits** - add delays between requests

## Error Handling

```bash
if ! claude -p "$prompt" 2>error.log; then
    echo "Error occurred:" >&2
    cat error.log >&2
    exit 1
fi
```

## rstn Integration Notes

For rstn's spec generation workflow:

```bash
claude -p "prompt" \
  --max-turns 10 \
  --dangerously-skip-permissions \
  --output-format stream-json \
  --include-partial-messages \
  --mcp-config ~/.rstn/mcp-session.json \
  --system-prompt-file /path/to/prompt.md \
  --append-system-prompt "## MCP Integration instructions"
```

**Known Issue**: When using `--print` with `--output-format=stream-json`, Claude CLI may require `--verbose` flag (observed error: "When using --print, --output-format=stream-json requires --verbose").
