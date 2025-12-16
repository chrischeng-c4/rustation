# Tasks: Interactive Task Generation

**Feature**: 055-interactive-task-generation
**Created**: 2025-12-17
**Status**: Ready for implementation

## Phase 1: Setup

- [ ] T001 [P] Create `ParsedTask` struct in worktree.rs
- [ ] T002 [P] Implement task parsing from markdown format

## Phase 2: State Management

- [ ] T003 Add `TaskListState` to track selection and ordering
- [ ] T004 Add task list mode to `SpecifyState` review phase
- [ ] T005 [P] Implement `reorder_task_up()` method
- [ ] T006 [P] Implement `reorder_task_down()` method

## Phase 3: User Interface

- [ ] T007 Add keyboard handlers for J/K (move task up/down)
- [ ] T008 Update review rendering to show numbered task list
- [ ] T009 Add visual indicator for selected task
- [ ] T010 Add visual indicator for [P] parallel and [US] markers

## Phase 4: Integration

- [ ] T011 Implement `serialize_tasks()` to convert back to markdown
- [ ] T012 Wire up Enter to save reordered tasks
- [ ] T013 Add confirmation message after save

## Phase 5: Testing

- [ ] T014 [P] Unit test: task parsing from markdown
- [ ] T015 [P] Unit test: task serialization to markdown
- [ ] T016 [P] Unit test: reorder up/down logic
- [ ] T017 Integration test: full workflow (generate → reorder → save)

## Dependencies

```
T001, T002 (parallel) → T003 → T004
T005, T006 (parallel) → T007
T008, T009, T010 (parallel)
T011 → T012 → T013
T014, T015, T016 (parallel) → T017
```

## Estimates

| Phase | Tasks | Complexity |
|-------|-------|------------|
| Setup | 2 | Low |
| State | 4 | Medium |
| UI | 4 | Medium |
| Integration | 3 | Low |
| Testing | 4 | Low |
| **Total** | **17** | **~250-350 lines** |
