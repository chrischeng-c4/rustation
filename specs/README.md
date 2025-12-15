# Rush Shell Specifications

Feature specifications for the rush shell project.

## Quick Reference

- **Feature List**: See [features.json](features.json) for complete feature catalog (001-044)
- **Current Status**: Phases 1-4 complete (001-026), Phases 5-8 planned (027-044)

## Directory Structure

```
specs/
├── features.json           # Master feature catalog
├── NNN-feature-name/
│   ├── spec.md             # What to build (requirements)
│   ├── plan.md             # How to build (architecture)
│   ├── tasks.md            # Implementation tasks
│   └── checklist.md        # QA checklist (optional)
└── README.md
```

## Feature Status

| Phase | Features | Status |
|-------|----------|--------|
| 1 | 001-016: Core & MVP | Complete |
| 2 | 017-026: Control Flow | Complete |
| 5 | 027-031: Scripting Foundations | Planned |
| 6 | 032-035: Parameter Power | Planned |
| 7 | 036-039: Shell Control | Planned |
| 8 | 040-044: Advanced Features | Planned |

## Workflow

Uses **spec-kit** for specification-driven development:

```
/speckit.specify  → spec.md
/speckit.plan     → plan.md
/speckit.tasks    → tasks.md
/speckit.implement → code + tests
```

See [CLAUDE.md](../CLAUDE.md) for full workflow details.

## Interactive Specify Workflow (Feature 051)

The `/speckit.specify` command provides an **interactive TUI workflow** for creating feature specifications without leaving rstn.

### Workflow Steps

1. **Trigger**: Select "Specify" from the Commands pane and press Enter
2. **Input Mode**: A dialog appears for your feature description
   - Type a description (minimum 10 characters)
   - Press `Enter` to submit
   - Press `Esc` to cancel
3. **Generating**: AI generates the spec (progress shown)
4. **Review Mode**: Preview the generated spec
   - `[Enter]` - Save spec to `specs/{NNN}-{name}/spec.md`
   - `[e]` - Edit the spec inline before saving
   - `[Esc]` - Cancel and discard the spec
5. **Edit Mode** (optional): Multi-line editor for quick tweaks
   - `[Ctrl+S]` - Save edited spec
   - `[Enter]` - Insert newline
   - `[Esc]` - Cancel edits, return to Review
   - Arrow keys, Home, End - Navigate
   - Backspace, Delete - Delete characters

### Benefits

- **No context switching**: Stay in the TUI throughout the workflow
- **Immediate feedback**: See the generated spec before saving
- **Quick edits**: Fix typos or add details without manual file editing
- **Safe workflow**: Multiple chances to cancel before committing

### Example Session

```
1. Navigate to "Specify" in Commands → Press Enter
2. Dialog: "Describe your feature..."
   Type: "Add dark mode toggle to settings"
   Press: Enter
3. Status: "Generating spec..." (2-3 seconds)
4. Review: [Generated spec displays]
   Notice a typo → Press: 'e'
5. Edit: Fix typo
   Press: Ctrl+S
6. Status: "Spec saved to specs/052-dark-mode-toggle/spec.md"
```

### Troubleshooting

**"Invalid input: description too short"**
- Ensure your description is at least 10 characters
- Provide enough context for meaningful spec generation

**"Spec generation failed"**
- Check network connection (if using remote AI)
- Check logs at `~/.rustation/logs/rstn.log`
- Retry by starting the workflow again

**Edit mode not responding**
- Ensure you're in Review Mode first
- Press 'e' to enter Edit Mode
- Check that focus is on Content area (yellow border)

## Adding Features

1. Create directory: `specs/NNN-feature-name/`
2. Run `/speckit.specify` with feature description
3. Run `/speckit.plan` to create implementation plan
4. Run `/speckit.tasks` to generate task breakdown
5. Update `features.json` with new entry
