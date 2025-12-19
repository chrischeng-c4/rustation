# Tasks: Release Channels

**Input**: Design documents from `/specs/047-release-channels/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: No automated tests requested. Manual verification per acceptance scenarios.

**Organization**: Tasks grouped by user story. US3 (Build Type ID) is foundational since both US1 and US2 depend on it.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Repository root**: `/Users/chrischeng/projects/rustation/`
- **rstn crate**: `crates/rstn/`
- **External repo**: `homebrew-rustation/` (github.com/chrischeng-c4/homebrew-rustation)

---

## Phase 1: Setup

**Purpose**: No setup required - existing Rust workspace project

> **SKIP**: Project already initialized with cargo workspace, justfile, and all dependencies.

---

## Phase 2: Foundational - Build Type Identification (US3) 🔧

**Purpose**: Core infrastructure that MUST complete before US1 or US2 can work properly

**Why Foundational**: Both the local dev channel (US1) and Homebrew channel (US2) need version output to display build type. This must be implemented first.

**Goal**: Version output displays build profile (debug/release) and log level defaults based on build type

**Independent Test**: Run `cargo build && ./target/debug/rstn --version` and `cargo build --release && ./target/release/rstn --version` - both should show build type

### Implementation

- [x] T001 [P] [US3] Add BUILD_PROFILE environment variable to `crates/rstn/build.rs`
- [x] T002 [P] [US3] Add cfg-based default_log_level() in `crates/rstn/src/settings.rs` (trace for debug, info for release)
- [x] T003 [US3] Add BUILD_PROFILE constant and build_info() function to `crates/rstn/src/version.rs`
- [x] T004 [US3] Update version display in `crates/rstn/src/main.rs` to use build_info()
- [x] T005 [US3] Verify: Build debug and release, confirm version output shows correct build type

**Checkpoint**: Version output now differentiates debug vs release builds

---

## Phase 3: User Story 1 - Local Development Installation (Priority: P1) 🎯 MVP

**Goal**: Developers can install debug builds with trace logging to `~/.local/bin` using a single command

**Independent Test**: Run `just install-dev` then `~/.local/bin/rstn --version` - should show "(debug)" and produce trace logs

### Implementation

- [x] T006 [P] [US1] Add `build-debug` recipe to `justfile`
- [x] T007 [P] [US1] Add `install-dev` recipe to `justfile` (builds debug, copies to ~/.local/bin)
- [x] T008 [P] [US1] Add `install-rstn-dev` recipe to `justfile` (quick single-binary install)
- [x] T009 [P] [US1] Add `which-build` recipe to `justfile` (diagnostic command using `file`)
- [x] T010 [US1] Verify: Run `just install-dev` and confirm debug binaries installed
- [x] T011 [US1] Verify: Run `just which-build` and confirm output shows "DEBUG"
- [x] T012 [US1] Verify: Run `~/.local/bin/rstn` and confirm trace-level logs in `~/.rustation/logs/rstn.log`

**Checkpoint**: Local development channel fully functional - developers can install debug builds

---

## Phase 4: User Story 2 - Homebrew Installation (Priority: P2)

**Goal**: End users can install rustation via Homebrew tap with release builds

**Independent Test**: Run `brew tap chrischeng-c4/rustation && brew install rustation` then verify `rush --version` shows "(release)"

### Implementation

- [x] T013 [US2] Create GitHub repository `chrischeng-c4/homebrew-rustation`
- [x] T014 [P] [US2] Create `Formula/rustation.rb` with build-from-source formula
- [x] T015 [P] [US2] Create `README.md` with tap installation instructions
- [x] T016 [US2] Create git tag `v0.35.0` on rustation repository for formula reference
- [x] T017 [US2] Push tag to origin: `git push origin v0.35.0`
- [x] T018 [US2] Verify: `brew tap chrischeng-c4/rustation` - SUCCESS (install blocked by Xcode CLI tools)
- [ ] T019 [US2] Verify: Run `brew audit --strict rustation` passes (blocked by Xcode CLI tools)
- [ ] T020 [US2] Verify: Run `rush --version` and `rstn --version` show "(release)" (blocked)

**Checkpoint**: Homebrew installation channel fully functional - users can install via brew

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation and final validation

- [x] T021 [P] Update `crates/rush/src/main.rs` to also display build profile in version (consistency)
- [x] T022 [P] Verify quickstart.md scenarios work end-to-end
- [x] T023 Verify existing `cargo test` suite still passes with new code (rstn: 86 passed)

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)     → SKIP (existing project)
       ↓
Phase 2 (US3)       → Foundational - BLOCKS US1 and US2
       ↓
Phase 3 (US1)  ←──→ Phase 4 (US2)   [Can run in parallel after Phase 2]
       ↓                ↓
Phase 5 (Polish)    → Depends on US1 + US2 completion
```

### User Story Dependencies

- **US3 (Build Type ID)**: No dependencies - foundational
- **US1 (Local Dev)**: Depends on US3 (needs build type in version)
- **US2 (Homebrew)**: Depends on US3 (needs build type in version) + git tag

### Task Dependencies Within Phases

**Phase 2 (US3)**:
- T001, T002 can run in parallel (different files)
- T003 depends on T001 (uses BUILD_PROFILE env var)
- T004 depends on T003 (uses build_info())
- T005 depends on T004 (verification)

**Phase 3 (US1)**:
- T006, T007, T008, T009 can run in parallel (all justfile additions)
- T010, T011, T012 are sequential verification steps

**Phase 4 (US2)**:
- T013 must be first (create repo)
- T014, T015 can run in parallel (different files in new repo)
- T016, T017 sequential (tag creation)
- T018, T019, T020 sequential verification

---

## Parallel Opportunities

### Phase 2 (Foundational)

```bash
# Launch these together:
Task T001: "Add BUILD_PROFILE to crates/rstn/build.rs"
Task T002: "Add cfg-based default_log_level() in crates/rstn/src/settings.rs"
```

### Phase 3 (US1)

```bash
# Launch all justfile recipes together:
Task T006: "Add build-debug recipe to justfile"
Task T007: "Add install-dev recipe to justfile"
Task T008: "Add install-rstn-dev recipe to justfile"
Task T009: "Add which-build recipe to justfile"
```

### Phase 4 (US2)

```bash
# After repo creation, launch these together:
Task T014: "Create Formula/rustation.rb"
Task T015: "Create README.md"
```

---

## Implementation Strategy

### MVP First (US3 + US1)

1. Complete Phase 2: Foundational (US3 - Build Type ID)
2. Complete Phase 3: User Story 1 (Local Dev)
3. **STOP and VALIDATE**: Run `just install-dev && rstn --version`
4. Local development channel is now usable

### Full Feature

1. Complete MVP above
2. Complete Phase 4: User Story 2 (Homebrew)
3. Complete Phase 5: Polish
4. Both channels fully operational

### Estimated Effort

| Phase | Tasks | Estimated Lines | PR |
|-------|-------|-----------------|-----|
| Phase 2 (US3) | 5 tasks | ~150 lines | PR #1 |
| Phase 3 (US1) | 7 tasks | ~50 lines | PR #2 |
| Phase 4 (US2) | 8 tasks | ~100 lines | PR #3 |
| Phase 5 (Polish) | 3 tasks | ~20 lines | PR #3 |

---

## Notes

- **No automated tests**: Manual verification per acceptance scenarios in spec.md
- **External repo**: US2 requires creating a new GitHub repository
- **Git tag**: US2 requires creating and pushing a release tag
- **Log verification**: US1 verification requires checking `~/.rustation/logs/rstn.log`

### Pull Request Strategy

**PR #1: Foundation + Build Type ID (US3)**
- T001-T005
- ~150 lines
- Merge to main first

**PR #2: Local Dev Channel (US1)**
- T006-T012
- ~50 lines
- Merge to main after PR #1

**PR #3: Homebrew Channel (US2) + Polish**
- T013-T023
- ~120 lines
- External repo + main repo polish
