## Proposal Generated: refactor-frontend-logic-to-backend

### Files Created
- ✅ proposal.md (26 lines)
- ✅ tasks.md (5 sections, 16 tasks)
- ✅ diagrams.md (4 diagrams: state, flow, sequence, UI layout)
- ✅ specs/file-explorer/spec.md (3 requirements: ADDED expansion, MODIFIED navigation, REMOVED list display)
- ✅ specs/chat-assistant/spec.md (3 requirements: ADDED chat logic)
- ✅ specs/change-management/spec.md (3 requirements: ADDED validation logic)

### Summary
- Requirements: 9 ADDED, 2 MODIFIED, 1 REMOVED
- Implementation tasks: 16
- Affected capabilities: file-explorer, chat-assistant (new), change-management (new), docker-management (code only)
- Affected code: `desktop/src/renderer/src/features/explorer/`, `packages/core/src/reducer/`, `desktop/src/renderer/src/features/chat/`, `desktop/src/renderer/src/features/workflows/`

### Validation
Manually verified:
- Directory structure created.
- Scenarios use `#### Scenario:` format.
- Diagrams use Mermaid syntax.
- All modified requirements include full text.

### Next Steps
1. Review files: `ls openspec/changes/refactor-frontend-logic-to-backend`
2. Inspect details: `openspec show refactor-frontend-logic-to-backend --json --deltas-only` (when available)
3. Approve proposal before implementation
