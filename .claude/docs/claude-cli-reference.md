# Claude Code CLI Reference

> Source: https://code.claude.com/docs/en/cli-reference

## Core Commands

| Command | Description |
|---------|-------------|
| `claude` | Start interactive REPL |
| `claude "query"` | Start REPL with initial prompt |
| `claude -p "query"` | Query via SDK, then exit (print mode) |
| `claude -c` | Continue most recent conversation |
| `claude -r "<session>"` | Resume session by ID or name |

## Print Mode (`-p`) Flags

| Flag | Description | Requirements |
|------|-------------|--------------|
| `-p, --print` | Print response without interactive mode | Pairs with `--output-format` |
| `--output-format` | Format: `text`, `json`, `stream-json` | Print mode only |
| `--input-format` | Input: `text`, `stream-json` | Print mode only |
| `--include-partial-messages` | Include partial streaming events | Requires: `--print` + `--output-format=stream-json` |
| `--verbose` | Enable verbose logging | Works in both modes |
| `--max-turns` | Limit agentic turns | Print mode only |

## System Prompt Flags

| Flag | Modes | Notes |
|------|-------|-------|
| `--system-prompt` | Interactive + Print | Mutually exclusive with `--system-prompt-file` |
| `--system-prompt-file` | Print only | Mutually exclusive with `--system-prompt` |
| `--append-system-prompt` | Interactive + Print | **Recommended for most use cases** |

## Session Management

| Flag | Description |
|------|-------------|
| `-c, --continue` | Load most recent conversation |
| `-r, --resume` | Resume by ID or name |
| `--session-id` | Use specific session UUID |
| `--fork-session` | Create new ID when resuming |

## Tool and Permission Management

| Flag | Description |
|------|-------------|
| `--tools` | Specify available tools |
| `--allowedTools` | Tools without permission prompts |
| `--disallowedTools` | Remove tools from context |
| `--dangerously-skip-permissions` | Skip all permission prompts |

## MCP Configuration

| Flag | Description |
|------|-------------|
| `--mcp-config` | Load MCP servers from JSON |
| `--strict-mcp-config` | Only use specified MCP config |

## Key Constraints

1. `-p` requires `--output-format` for structured output
2. `--include-partial-messages` requires both `--print` AND `--output-format=stream-json`
3. `--system-prompt` and `--system-prompt-file` are mutually exclusive
4. **IMPORTANT**: When using `--print` with `--output-format=stream-json`, may require `--verbose` (observed in practice)

## Common Usage Patterns

```bash
# Basic print mode with JSON
claude -p "query" --output-format json

# Streaming with partial messages
claude -p "query" --output-format stream-json --include-partial-messages

# With MCP config
claude -p "query" --mcp-config ./mcp.json

# Resume session
claude -p --resume SESSION_ID "follow up query"

# With custom system prompt
claude -p --append-system-prompt "Custom instructions" "query"
```
