## 1. Backend Implementation
- [x] 1.1 Update `Comment` struct in `packages/core/src/app_state.rs` to add `line_number: Option<usize>`.
- [x] 1.2 Update `Action::AddFileComment` in `packages/core/src/actions.rs` to include `line_number`.
- [x] 1.3 Update `DbManager::run_migrations` in `packages/core/src/db.rs` to add `line_number` column (handle migration logic or documented manual step).
- [x] 1.4 Update `DbManager::add_comment` and `DbManager::get_comments` to handle `line_number`.
- [x] 1.5 Update `packages/core/src/lib.rs` to propagate `line_number` from DB to actions and vice versa.

## 2. Frontend Implementation
- [x] 2.1 Install `prism-react-renderer` for syntax highlighting.
- [x] 2.2 Refactor `SourceCodeViewer.tsx` to support `comments` prop and `onAddComment` callback.
- [x] 2.3 Implement syntax highlighting in `SourceCodeViewer.tsx` using prism-react-renderer.
- [x] 2.4 Implement inline comment UI (gutter click to add, display comments between lines).
- [x] 2.5 Fix scrolling issues in `DetailPanel.tsx` and `SourceCodeViewer.tsx` (proper flex/overflow propagation).
- [x] 2.6 Update `DetailPanel.tsx` to pass comments to `SourceCodeViewer` and handle `onAddComment`.

## 3. Testing
- [x] 3.1 Rust tests pass (`cargo test --package rstn-core` - 169 passed)
- [x] 3.2 TypeScript compilation passes for modified files
- [x] 3.3 napi-rs build succeeds (`pnpm build` in packages/core)
- [ ] 3.4 Visual verification of syntax highlighting (manual)
- [ ] 3.5 Visual verification of inline comment creation and persistence (manual)
- [ ] 3.6 Visual verification of scrolling on large files (manual)

## 4. Documentation
- [x] 4.1 Spec delta created in `openspec/changes/fix-file-preview/specs/file-explorer/spec.md`
- [x] 4.2 Spec delta created in `openspec/changes/fix-file-preview/specs/shared-ui/spec.md`

## 5. Additional Fixes (Follow-up)
- [x] 5.1 Fix breadcrumb navigation - Updated `reducer/explorer.rs` to push history on `ExploreDir` action
- [x] 5.2 Fix initial directory load - Updated `ExplorerPage.tsx` to trigger load when entries are empty
- [x] 5.3 Add more syntax highlighting languages - Extended `getLanguageFromPath()` with 80+ languages
- [x] 5.4 IDE-style file list - Replaced column layout with compact name + git status icons in `FileTable.tsx`
- [x] 5.5 Removed tabs in DetailPanel - Only preview tab remains (simplified UI)
- [x] 5.6 Fix NavigateBack/Forward/Up - Added async handling in `lib.rs` to trigger `ExploreDir` after state update

## 6. Markdown Rendering
- [x] 6.1 Add `mermaid` dependency to `desktop/package.json`
- [x] 6.2 Create `MarkdownPreview.tsx` component with Mermaid diagram support
- [x] 6.3 Update `SourceCodeViewer.tsx` to route `.md`/`.mdx` files to `MarkdownPreview`
- [x] 6.4 Add view mode toggle (preview/source) for markdown files with inline comment support in source mode
- [ ] 6.5 Visual verification of markdown rendering (manual)
- [ ] 6.6 Visual verification of mermaid diagrams (manual)
- [ ] 6.7 Visual verification of inline comments in markdown source mode (manual)
