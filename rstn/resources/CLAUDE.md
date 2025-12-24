# rstn Agent Context

You are being invoked by rstn TUI, a terminal-based development workflow manager.

## Required (Core Context)

### Environment

- You are running inside a worktree managed by rstn
- MCP server "rstn" is available for extended context
- The working directory is the worktree root

### Behavioral Rules

1. **KB-First Principle**: All architecture and behavior should be derived from Knowledge Base (`kb/`) documentation
2. **State-First Architecture**: All state must be JSON/YAML serializable - no hidden state
3. **Minimal Solutions**: Prefer simple, focused implementations over over-engineering
4. **No Speculation**: When you need deeper context, fetch it via MCP rather than guessing

### Code Quality

- All state structs derive `Serialize + Deserialize + Debug + Clone`
- Files should not exceed 500 lines; MUST split at 1000 lines
- Run `cargo test` and `cargo clippy` before considering work complete

## Optional (Available via MCP)

Additional context available by calling MCP tools:

| Resource | MCP Tool | Description |
|----------|----------|-------------|
| KB Architecture | `get_kb_doc("architecture")` | Full architecture docs |
| Workflow Guide | `get_kb_doc("workflow")` | SDD, testing guides |
| Project Structure | `get_project_structure()` | Current worktree layout |
| State Schema | `get_state_schema()` | AppState definition |

When you need deeper context, fetch it via MCP rather than guessing.
