## 1. Foundation & Architecture
- [x] 1.1 Create new Rust crate `crates/rstn/` in workspace (changed from apps/ to crates/ per Zed pattern)
- [x] 1.2 Configure `crates/rstn-core/` for direct Rust usage (moved from packages/core, removed napi-rs)
- [x] 1.3 Initialize GPUI application entry point (`main.rs`)
- [ ] 1.4 Implement global state integration (`AppState` -> GPUI `Model`)
- [ ] 1.5 **BLOCKED**: GPUI build requires Metal Toolchain (Xcode 26 beta issue)

## 2. Shell & Layout
- [x] 2.1 Implement Window/Shell layout (Sidebar, Main Content, Status Bar)
- [x] 2.2 Implement `Sidebar` view with feature navigation
- [x] 2.3 Implement `Theme` system using GPUI styling
- [x] 2.4 Create `crates/rstn-ui/` component library
- [x] 2.5 Implement PageHeader, EmptyState components
- [ ] 2.6 Verify UI renders correctly (BLOCKED by Metal Toolchain)

## 3. Core Features Migration
- [x] 3.0 Create `crates/rstn-views/` feature views library
- [x] 3.1 Port `Tasks` view (TaskCard, LogPanel, 50/50 split layout)
- [x] 3.2 Port `Dockers` view (ServiceCard, grouping, status badges)
- [ ] 3.3 Port `Explorer` view (File tree, git status, preview)
- [ ] 3.4 Port `Terminal` view (PTY integration + ANSI rendering)
- [ ] 3.5 Port `Chat` view (Message rendering, input)
- [ ] 3.6 Port `Workflows` view (Constitution, Change Management)
- [ ] 3.7 Port `Settings` view (Configuration UI)
- [ ] 3.8 Integrate views with main app tab routing

## 4. MCP & Advanced Features
- [ ] 4.1 Port `MCP` inspector view
- [ ] 4.2 Port `A2UI` dynamic renderer
- [ ] 4.3 Port `Context Engine` visualizations

## 5. Cleanup & Polish
- [ ] 5.1 Verify feature parity with Electron app
- [ ] 5.2 Remove `desktop/` (Electron) directory
- [ ] 5.3 Update `justfile` and CI/CD pipelines
