# Tasks: MCP Task Completion

**Feature**: 063-mcp-task-completion
**Created**: 2024-12-17
**Status**: Ready for implementation

## Phase 1: Tool Definition

- [x] T001 Create CompleteTaskArgs struct
- [x] T002 Define JSON schema for rstn_complete_task

## Phase 2: Event Type

- [x] T003 Add McpTaskCompleted variant to Event enum

## Phase 3: Task Completion Logic

- [x] T004 Add complete_task_by_id() to SpecifyState
- [x] T005 Implement task lookup by ID
- [x] T006 Mark task complete and save to file
- [x] T007 Get next incomplete task info

## Phase 4: Tool Handler

- [x] T008 Implement handle_complete_task async function
- [x] T009 Send McpTaskCompleted event
- [x] T010 Return response with next_task info
- [x] T011 Register rstn_complete_task tool

## Phase 5: Event Handling

- [x] T012 Handle McpTaskCompleted in app.rs
- [x] T013 Refresh worktree view task list
- [x] T014 Update progress indicator

## Phase 6: Testing

- [x] T015 Unit test: task marked complete
- [x] T016 Unit test: file updated
- [x] T017 Unit test: next task returned
- [x] T018 Integration test: full flow

## Dependencies

```
T001 → T002
T003
T004 → T005 → T006 → T007
T002, T007 → T008 → T009 → T010 → T011
T003, T011 → T012 → T013 → T014
T014 → T015 → T016 → T017 → T018
```

## Notes

- Validation is optional (skip_validation flag)
- Reuse existing task list infrastructure
- Progress format: "X/Y tasks complete"
