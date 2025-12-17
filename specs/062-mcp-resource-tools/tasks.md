# Tasks: MCP Resource Tools

**Feature**: 062-mcp-resource-tools
**Created**: 2024-12-17
**Status**: Ready for implementation

## Phase 1: rstn_read_spec Tool

- [x] T001 Create ReadSpecArgs struct
- [x] T002 Implement artifact_to_filename mapping
- [x] T003 Implement handle_read_spec handler
- [x] T004 Handle file not found gracefully
- [x] T005 Register rstn_read_spec tool

## Phase 2: rstn_get_context Tool

- [x] T006 Create FeatureContext response struct
- [x] T007 Implement handle_get_context handler
- [x] T008 Reuse detect_current_feature() logic
- [x] T009 Register rstn_get_context tool

## Phase 3: Testing

- [x] T010 Unit test: read_spec returns correct content
- [x] T011 Unit test: read_spec handles missing file
- [x] T012 Unit test: get_context returns correct data
- [x] T013 Integration test: Claude can read spec

## Dependencies

```
T001 → T002 → T003 → T004 → T005
T006 → T007 → T008 → T009
T005, T009 → T010 → T011 → T012 → T013
```

## Notes

- Read files fresh each call (no caching)
- Return helpful error messages for missing files
- Context detection reuses existing app.rs logic
