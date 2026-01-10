# Implementation Tasks: Fix File Comments Not Appearing

**Change ID**: `fix-file-comments-not-appearing`

---

## Investigation Phase

- [ ] Manual debug session with logging
  - [ ] Run `pnpm dev` and open DevTools
  - [ ] Add a comment to a file
  - [ ] Capture browser console logs
  - [ ] Capture terminal backend logs
  - [ ] Identify exact break point in state flow

- [ ] Analyze state subscription mechanism
  - [ ] Review `useAppState` hook implementation
  - [ ] Check if nested state changes trigger notifications
  - [ ] Verify IPC bridge transmits state updates
  - [ ] Check if `selected_comments` path is properly watched

---

## Fix Phase

- [ ] Implement state subscription fix
  - [ ] Based on investigation findings
  - [ ] Ensure `SetFileComments` triggers frontend update
  - [ ] Test state propagation manually

- [ ] Remove debug logging (after fix verified)
  - [ ] Remove console.log from `DetailPanel.tsx`
  - [ ] Remove console.log from `SourceCodeViewer.tsx`
  - [ ] Remove eprintln! from `lib.rs`
  - [ ] Remove eprintln! from `explorer.rs`

---

## Testing Phase

- [ ] Manual verification
  - [ ] Add comment → appears immediately
  - [ ] Close and reopen file → comment persists
  - [ ] Add multiple comments → all appear correctly
  - [ ] Comment count badge updates in file list

- [ ] Unit tests (if DOMMatrix fixed)
  - [ ] Fix PdfViewer test pollution
  - [ ] Run `SourceCodeViewer.comment.test.tsx`
  - [ ] Verify all tests pass

- [ ] E2E tests (if Playwright/Electron fixed)
  - [ ] Fix Playwright compatibility
  - [ ] Run `file-comments.spec.ts`
  - [ ] Verify all tests pass

---

## Optional: Tooling Fixes

### Fix Playwright/Electron Compatibility

- [ ] Option A: Downgrade Electron to 32.x
  - [ ] Update `package.json` Electron version
  - [ ] Test E2E suite

- [ ] Option B: Upgrade Playwright to 1.58+ (when stable)
  - [ ] Wait for stable release
  - [ ] Upgrade `@playwright/test`
  - [ ] Test E2E suite

- [ ] Option C: Use alternative E2E framework
  - [ ] Research alternatives (Puppeteer, Selenium)
  - [ ] Migrate test suite

### Fix PdfViewer Test Pollution

- [ ] Mock DOMMatrix in Vitest setup
  - [ ] Add global mock to `test/setup.ts`
  - [ ] Verify tests no longer fail on import

- [ ] Alternative: Lazy load PdfViewer
  - [ ] Use dynamic import in SourceCodeViewer
  - [ ] Only load when PDF file detected

---

## Completion Criteria

### Minimum (Required)
- [x] Debug logging added
- [ ] Manual testing identifies root cause
- [ ] Fix implemented and verified manually
- [ ] Comment functionality works end-to-end

### Ideal (Optional)
- [ ] Unit tests pass
- [ ] E2E tests pass
- [ ] Tooling issues resolved
- [ ] Full test coverage achieved

---

## Notes

- Debug logging is already in place and ready for manual testing
- Test files are created but cannot run due to tooling issues
- Backend flow is verified working via code analysis
- Issue is likely in frontend state subscription
