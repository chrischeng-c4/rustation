---
title: "Implement Prompt to Claude Workflow"
status: "in-progress"
priority: "high"
last_updated: 2025-12-23
---

# Task: Implement Prompt to Claude Workflow

## Source
- `kb/architecture/10-workflow-prompt-claude.md`
- `kb/architecture/02-state-first-mvi.md`

## Todo List

### Phase 1: State & Types
- [x] Define `PromptClaudeData` in `rstn/state/workflows/prompt.py`.
- [ ] Update `AppState` / `WorkflowState` to support structured workflow data.
- [ ] Add "Prompt Claude" to the default command list in `WorktreeViewState`.

### Phase 2: Domain & Infrastructure
- [ ] Implement `SessionConfigManager` to generate `/tmp/rstn/{session_id}/mcp-config.json`.
- [ ] Update `RunClaudeCli` effect in `rstn/effect/__init__.py` to support full parameters (max_turns, etc.).
- [ ] Implement raw logging of Claude CLI output in the executor.

### Phase 3: Effect Executor Implementation
- [ ] Implement async subprocess execution for `claude` CLI.
- [ ] Implement JSONL parser for `stream-json` output.
- [ ] Map JSONL events to `ClaudeStreamDelta`, `ClaudeCompleted`, etc.

### Phase 4: Reducer Logic (FSM)
- [ ] Handle `WorkflowStartRequested` for `prompt-claude`:
    - Transition status to `RUNNING`.
    - Dispatch `RunClaudeCli` effect.
- [ ] Handle `ClaudeStreamDelta`:
    - Append text to `PromptClaudeData.output`.
- [ ] Handle `ClaudeCompleted`:
    - Update status to `COMPLETED`.
    - Capture `session_id` and final metrics.

### Phase 5: UI & Interaction
- [ ] Implement "Prompt Mode" in `WorktreeView` (triggered by command).
- [ ] Map `Enter` key in input mode to dispatch `WorkflowStartRequested`.
- [ ] Ensure Content Area scrolls to bottom during streaming output.
- [ ] Display cost/session metrics in the status bar or content footer.

### Phase 6: Verification
- [ ] Write state transition tests for the workflow lifecycle.
- [ ] Verify serialization/deserialization of `PromptClaudeData`.
- [ ] Manual test with real `claude` CLI and a mock MCP config.
