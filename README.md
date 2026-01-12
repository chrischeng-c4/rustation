# rustation

**rustation** is a developer workbench powered by GPUI, designed to streamline development workflows with integrated tools for task management, Docker orchestration, AI assistance, and more.

## 🏗️ Architecture

rustation is built with a modern Rust-first architecture using GPUI (Zed's UI framework):

```
┌─────────────────────────────────────────────────────┐
│ GPUI Frontend (crates/rstn)                        │
│   ├─ State Management (Model<AppState>)            │
│   ├─ UI Rendering (GPUI views)                     │
│   └─ Event Dispatch (AppAction)                    │
├─────────────────────────────────────────────────────┤
│ View Layer (crates/rstn-views)                     │
│   ├─ TasksView, DockersView, ExplorerView          │
│   ├─ TerminalView, ChatView, WorkflowsView         │
│   ├─ McpView, SettingsView                         │
│   └─ Material Design 3 Components                  │
├─────────────────────────────────────────────────────┤
│ UI Components (crates/rstn-ui)                     │
│   ├─ MaterialTheme (MD3 color system)              │
│   ├─ ShellLayout, Sidebar, PageHeader              │
│   └─ Reusable UI primitives                        │
├─────────────────────────────────────────────────────┤
│ Core Logic (crates/rstn-core)                      │
│   ├─ AppState (State-First Architecture)           │
│   ├─ Reducers (State transitions)                  │
│   ├─ Services (Docker, Git, MCP, etc.)             │
│   └─ Business Logic                                │
└─────────────────────────────────────────────────────┘
```

### State-First Architecture

All application state is **JSON-serializable** and managed through a single source of truth:
- **AppState**: Complete application state tree
- **Reducers**: Pure functions for state transitions
- **Model<T>**: GPUI's reactive state management

## ✨ Features

### 🎯 Core Views

1. **Tasks** - Justfile command runner
   - Execute project tasks with a single click
   - Real-time output display
   - Status indicators (Running, Success, Failed)

2. **Dockers** - Container management dashboard
   - View running containers
   - Service status monitoring
   - Built-in service templates

3. **Explorer** - File browser with Git integration
   - File tree navigation
   - Git status indicators
   - File details panel

4. **Terminal** - Integrated PTY terminal (UI Shell complete)
   - Session management
   - Full ANSI color support (coming soon)
   - Multi-session support (coming soon)

5. **Chat** - AI conversation interface
   - Chat history from state
   - Message role indicators (User/Assistant/System)
   - Claude API integration (coming soon)

6. **Workflows** - OpenSpec change management
   - Constitution rules management
   - Change proposal tracking
   - Review gate workflow
   - Context engine integration

7. **MCP** - MCP server inspector
   - Server health monitoring
   - Tools list display
   - JSON-RPC 2.0 integration

8. **Settings** - Configuration interface
   - Theme, project, and MCP settings
   - Category-based organization
   - Real-time settings display

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.75+ with `cargo`
- **macOS** (GPUI currently supports macOS primarily)
- **Xcode Command Line Tools** (for Metal shader compilation)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rustation.git
cd rustation

# Build the project
cargo build --release

# Run rustation
cargo run --bin rstn
```

### Development

```bash
# Run in development mode with logging
RUST_LOG=info cargo run --bin rstn

# Run tests
cargo test --package rstn-core

# Check code without building
cargo check
```

## 📦 Project Structure

```
rustation/
├── crates/
│   ├── rstn/           # Main GPUI application
│   │   ├── src/
│   │   │   ├── main.rs     # Application entry point
│   │   │   └── state.rs    # GPUI state wrapper
│   │   └── Cargo.toml
│   ├── rstn-core/      # Core business logic
│   │   ├── src/
│   │   │   ├── app_state.rs    # State definition
│   │   │   ├── reducer/        # State transitions
│   │   │   ├── docker.rs       # Docker management
│   │   │   ├── justfile.rs     # Justfile parsing
│   │   │   ├── worktree.rs     # Git worktree
│   │   │   ├── terminal.rs     # PTY support
│   │   │   ├── mcp_server.rs   # MCP server
│   │   │   └── ...
│   │   └── Cargo.toml
│   ├── rstn-ui/        # Reusable UI components
│   │   ├── src/
│   │   │   ├── theme.rs        # Material Design 3 theme
│   │   │   ├── layout.rs       # ShellLayout
│   │   │   └── ...
│   │   └── Cargo.toml
│   └── rstn-views/     # Feature-specific views
│       ├── src/
│       │   ├── tasks.rs        # TasksView
│       │   ├── dockers.rs      # DockersView
│       │   ├── explorer.rs     # ExplorerView
│       │   ├── terminal.rs     # TerminalView
│       │   ├── chat.rs         # ChatView
│       │   ├── workflows.rs    # WorkflowsView
│       │   ├── mcp.rs          # McpView
│       │   └── settings.rs     # SettingsView
│       └── Cargo.toml
├── openspec/           # OpenSpec specifications
│   ├── specs/          # Feature specifications
│   └── changes/        # Change proposals
├── dev-docs/           # Engineering documentation
├── docs/               # User documentation
└── Cargo.toml          # Workspace manifest
```

## 🧪 Testing

rustation follows a comprehensive testing strategy:

### Unit Tests

```bash
# Run rstn-core unit tests (182 tests)
cargo test --package rstn-core

# Run rstn-ui tests
cargo test --package rstn-ui

# Run rstn-views tests
cargo test --package rstn-views
```

### UI Integration Tests (Planned)

```bash
# UI tests (requires Xcode/Metal - cannot run yet)
cargo test --test '*' --features gpui/test-support

# Specific view tests
cargo test --test tasks_view_test
cargo test --test dockers_view_test
```

**Status**: Test code written but cannot execute without Xcode installation.
See [UI Testing Plan](openspec/UI_TESTING_PLAN.md) for details.

### Test Coverage

Current test coverage:
- **rstn-core**: 182 unit tests ✅
- **rstn/state.rs**: 18 accessor tests ✅
- **UI tests**: Test code written, execution blocked by Metal ⚠️
- **Integration tests**: Planned
- **E2E tests**: Planned

**Three-Layer Testing Strategy**:
1. **State Tests** (Layer 1): ✅ 200+ tests passing (no Xcode required)
2. **View Integration Tests** (Layer 2): 📝 Planned, cannot run without Xcode
3. **Interactive Tests** (Layer 3): 📝 Planned, requires event handlers

See [UI Testing Plan](openspec/UI_TESTING_PLAN.md) for comprehensive testing strategy.

### Known Testing Issues

- **GPUI/Metal**: UI tests require full Xcode (not just Command Line Tools)
- **Workaround**: Use GitHub Actions CI with macOS runners (has Xcode pre-installed)
- **Doc tests**: 5 doc tests currently failing (non-blocking, documentation examples)

## 📖 Documentation

- **[Engineering Handbook](dev-docs/)** - Architecture, development guides, and contribution guidelines
- **[OpenSpec](openspec/)** - Feature specifications and change proposals
- **[User Manual](docs/)** - How to use rustation (coming soon)

## 🔄 Migration Status

rustation is currently migrating from Electron to GPUI:

**Overall Progress**: 88% (Phase 6 - 50% complete)

✅ **Complete**:
- Phase 1-5: Foundation, UI components, and all 8 views
- Stage 1: Backend data integration
- Stage 2: State management system
- Stage 3: All view integrations (Explorer, Terminal, Chat, MCP, Workflows, Settings)

🟡 **In Progress**:
- Stage 4: Polish & Testing

⏸️ **Deferred**:
- Full PTY terminal integration (alacritty_terminal)
- Claude API client integration
- Interactive features (button clicks, form inputs)
- Keyboard shortcuts

See [tasks.md](openspec/changes/migrate-to-gpui/tasks.md) for detailed progress.

## 🛠️ Development Principles

rustation is built on three core principles:

### 1. State-First Architecture
All state must be JSON-serializable for:
- State persistence
- Testing
- Debugging
- Time-travel capabilities

### 2. YAGNI (You Aren't Gonna Need It)
- Start with minimal viable solutions
- Delete aggressively
- Add features only when immediately needed

### 3. Automated Verification
Everything must be checkable without human intervention:
- All features have tests
- State transitions are tested
- No manual testing required

## 🤝 Contributing

Contributions are welcome! Please see:
- [CLAUDE.md](CLAUDE.md) - AI coding principles and guidelines
- [dev-docs/workflow/contribution-guide.md](dev-docs/workflow/contribution-guide.md) - Development workflow
- [dev-docs/workflow/definition-of-done.md](dev-docs/workflow/definition-of-done.md) - Feature completion checklist

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **GPUI** - Powered by [Zed's GPUI framework](https://github.com/zed-industries/zed)
- **Material Design 3** - UI design system by Google
- **OpenSpec** - Specification-driven development workflow

---

**Status**: 🚧 Active Development (GPUI Migration)
**Version**: 0.1.0 (Pre-release)
