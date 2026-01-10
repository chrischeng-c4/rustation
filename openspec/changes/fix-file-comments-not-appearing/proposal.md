# Fix File Comments Not Appearing After Submission

**Change ID**: `fix-file-comments-not-appearing`
**Type**: Bug Fix
**Status**: 🔴 Blocked - Requires Investigation
**Created**: 2026-01-10

---

## Why

### Problem Statement

Users cannot see inline comments after submitting them through the Explorer file viewer. The comment is successfully saved to SQLite database, but fails to appear in the UI. This breaks the core functionality of the file commenting feature.

### Current Behavior (Broken)

1. User opens a file in Explorer
2. User hovers over line number and clicks "Add Comment"
3. User types comment content and clicks "Submit"
4. ✅ Textarea disappears (async fix applied)
5. ❌ Comment does NOT appear in the UI
6. Comment IS saved to database (verified via backend logs)

### Expected Behavior

1. User opens a file in Explorer
2. User hovers over line number and clicks "Add Comment"
3. User types comment content and clicks "Submit"
4. ✅ Textarea remains visible until backend confirms save
5. ✅ Textarea disappears after save completes
6. ✅ Comment appears immediately below the line
7. ✅ Comment persists after closing/reopening file

### User Impact

- **Severity**: High
- **Affected Users**: All users attempting to add inline comments
- **Workaround**: None - feature is completely non-functional from user perspective

---

## What

### Changes Required

#### 1. **Investigate State Subscription Issue**
- Identify why frontend doesn't receive state updates after `SetFileComments`
- Suspected issue: `useAppState` hook not detecting nested state changes
- Location: `worktree.explorer.selected_comments` update not propagating

#### 2. **Fix State Propagation**
- Ensure `SetFileComments` action triggers state subscription notifications
- Verify frontend re-renders when `selected_comments` changes

#### 3. **Add Integration Tests**
- Unit tests for comment functionality (already created but blocked)
- E2E tests for end-to-end flow (already created but blocked)

### Technical Changes

**Files to Investigate**:
- `desktop/src/renderer/src/hooks/useAppState.tsx` - State subscription mechanism
- `packages/core/src/lib.rs` - State notification after reducer
- `desktop/src/preload/index.ts` - IPC bridge for state updates

**Files Already Modified** (Debug Logging):
- `desktop/src/renderer/src/features/explorer/DetailPanel.tsx`
- `desktop/src/renderer/src/components/shared/SourceCodeViewer.tsx`
- `packages/core/src/lib.rs`
- `packages/core/src/reducer/explorer.rs`

---

## Impact

### Before Fix
- ❌ Comments saved to database but invisible
- ❌ Users confused - appears to be completely broken
- ❌ No visual feedback that save succeeded

### After Fix
- ✅ Comments appear immediately after submission
- ✅ Visual confirmation that comment was saved
- ✅ Feature works as designed

### Testing Blockers

1. **E2E Tests Blocked**: Playwright 1.57 + Electron 33 incompatibility
   - Error: `--remote-debugging-port=0` not supported
   - Affects ALL E2E tests in project

2. **Unit Tests Blocked**: PdfViewer DOMMatrix errors
   - Error: `DOMMatrix is not defined`
   - Pollutes entire test suite

### Risks

- **Low Risk**: Bug is already present, fix can't make it worse
- **High Confidence**: Backend flow verified working via logs
- **Narrow Scope**: Fix likely isolated to state subscription

---

## Architecture Analysis

### Verified Working ✅

```
User Action
    ↓
Frontend: handleCommentSubmit → await onAddComment()
    ↓
Frontend: DetailPanel dispatches AddFileComment
    ↓
Backend: lib.rs handles AddFileComment
    ↓
Backend: db_mgr.add_comment() saves to SQLite ✅
    ↓
Backend: Triggers SelectFile action
    ↓
Backend: SelectFile loads comments from DB
    ↓
Backend: Dispatches SetFileComments action
    ↓
Reducer: Updates worktree.explorer.selected_comments ✅
```

### Suspected Break Point ❌

```
Reducer: State updated
    ↓
    ??? ← BREAK POINT
    ↓
Frontend: useAppState receives update (NOT HAPPENING)
    ↓
Frontend: Component re-renders (NOT HAPPENING)
    ↓
UI: Comment appears (NOT HAPPENING)
```

### Debug Logging Added

**Frontend** (Browser Console):
- `[DetailPanel]` - Comment submission flow
- `[SourceCodeViewer]` - Comment grouping and rendering

**Backend** (Terminal):
- `[Backend] AddFileComment` - Save and reload flow
- `[Backend] SelectFile` - Comment loading from DB
- `[Reducer] SetFileComments` - State update confirmation

**Status**: Ready for manual testing to identify exact break point

---

## Dependencies

### Blocked By
- None (can be debugged with manual testing)

### Optional Improvements
1. Fix Playwright/Electron E2E compatibility
2. Fix PdfViewer DOMMatrix test pollution
3. Mock DOMMatrix in Vitest setup

---

## References

- Planning Document: `/Users/chrischeng/.claude/plans/hazy-forging-book.md`
- Unit Test: `desktop/src/renderer/src/components/shared/__tests__/SourceCodeViewer.comment.test.tsx`
- E2E Test: `desktop/e2e/file-comments.spec.ts`
- Backend Action: `packages/core/src/lib.rs:3877-3931`
- Reducer: `packages/core/src/reducer/explorer.rs:30-42`

---

## Next Steps

1. **Manual Debug Session**:
   ```bash
   cd desktop
   pnpm dev
   # Open DevTools (Cmd+Option+I)
   # Add a comment to any file
   # Compare browser console + terminal logs
   ```

2. **Identify Break Point**:
   - If backend logs show comment saved → State subscription issue
   - If reducer logs show state updated → Frontend hook issue
   - If frontend logs show comment received → Rendering issue

3. **Implement Fix** based on findings

4. **Verify Fix**:
   - Manual testing
   - Unit tests (after fixing DOMMatrix)
   - E2E tests (after fixing Playwright/Electron)

---

## Related Work

- ✅ **Completed**: YAML syntax highlighting enhancement
- ✅ **Completed**: Async fix for comment textarea (prevents premature disappearance)
- 🔴 **Blocked**: This bug (comments not appearing)
- ⏸️ **Deferred**: Comprehensive test coverage (blocked by tooling issues)
