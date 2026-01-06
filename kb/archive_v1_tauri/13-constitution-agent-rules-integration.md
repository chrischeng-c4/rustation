# Constitution + Agent Rules Integration

> **Status**: Design
> **Last Updated**: 2025-01-05
> **Related**: `10-constitution-system.md`

---

## 1. Problem Statement

Currently, two separate systems control Claude's behavior:

| System | Location | Mechanism |
|--------|----------|-----------|
| **Constitution** | Workflows Tab | Modular rules, context-aware selection |
| **Agent Rules** | Project Tab | Profile-based, full prompt replacement |

**Issues**:
1. User confusion: Two places to configure AI behavior
2. Overlapping functionality: Both affect Claude's instructions
3. Inconsistent scope: Constitution (worktree) vs Agent Rules (project)

---

## 2. Proposed Integration

### Unified Model: Constitution Modes

Constitution becomes a **two-mode system**:

```
Constitution Management
├── Mode 1: Rules (Modular)      ← Current Constitution
│   └── Context-aware, token-budgeted
│
└── Mode 2: Presets (Profiles)   ← Current Agent Rules
    └── Full prompt replacement
```

### Mode Definitions

| Mode | Description | Use Case |
|------|-------------|----------|
| **Rules** | Multiple `.rstn/constitutions/*.md` files combined based on context | Large projects, monorepos, context-sensitive guidelines |
| **Presets** | Single profile replaces entire system prompt | Quick persona switching, specialized tasks |

### User Choice

When Constitution is active:
- User selects **mode**: Rules OR Presets
- **Rules mode**: Current constitution behavior (auto-selection)
- **Presets mode**: Select one profile (full replacement)

---

## 3. State Design

### Before (Separate)

```
Project
├── agent_rules_config: AgentRulesConfig  ← Project scope
│   ├── enabled: bool
│   ├── active_profile_id: string
│   └── profiles: AgentProfile[]
│
└── worktrees[]
    └── tasks
        └── constitution_*                 ← Worktree scope
```

### After (Integrated)

```
Project
└── worktrees[]
    └── tasks
        └── constitution: ConstitutionConfig
            ├── mode: 'rules' | 'presets'
            │
            ├── rules_config:              ← Mode 1: Rules
            │   ├── exists: bool
            │   └── content: string
            │
            └── presets_config:            ← Mode 2: Presets
                ├── active_preset_id: string?
                └── presets: ConstitutionPreset[]
```

### New Types

```typescript
type ConstitutionMode = 'rules' | 'presets'

interface ConstitutionPreset {
  id: string
  name: string
  prompt: string
  is_builtin: boolean
  created_at: string
  updated_at: string
}

interface ConstitutionConfig {
  mode: ConstitutionMode

  // Rules mode
  rules_exists: boolean
  rules_content: string | null

  // Presets mode
  active_preset_id: string | null
  presets: ConstitutionPreset[]
}
```

---

## 4. UI Design

### Constitution Panel (Integrated)

```
┌─────────────────────────────────────────────────────┐
│ Constitution Management                              │
│                                                      │
│ Mode: [Rules ▾] [Presets]                           │
│                                                      │
├─────────────────────────────────────────────────────┤
│                                                      │
│ ┌─────────────────────────────────────────────────┐ │
│ │ [Rules Mode Panel]                              │ │
│ │                                                 │ │
│ │ .rstn/constitutions/                            │ │
│ │ ├── global.md     ✓ Always loaded              │ │
│ │ ├── rust.md       ✓ Active (language match)    │ │
│ │ ├── napi-rs.md    ✓ Active (path match)        │ │
│ │ └── react.md      ○ Inactive                   │ │
│ │                                                 │ │
│ │ Token usage: 2,800 / 4,000                      │ │
│ │                                                 │ │
│ │ [Regenerate] [Edit Rules]                       │ │
│ └─────────────────────────────────────────────────┘ │
│                                                      │
└─────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────┐
│ Constitution Management                              │
│                                                      │
│ Mode: [Rules] [Presets ▾]                           │
│                                                      │
├─────────────────────────────────────────────────────┤
│                                                      │
│ ┌─────────────────────────────────────────────────┐ │
│ │ [Presets Mode Panel]                            │ │
│ │                                                 │ │
│ │ Active Preset: [Rust Expert ▾]                  │ │
│ │                                                 │ │
│ │ ⚠️ Custom preset will REPLACE default          │ │
│ │    CLAUDE.md instructions                       │ │
│ │                                                 │ │
│ │ Available Presets:                              │ │
│ │ ├── ⭐ Rust Expert (built-in)                   │ │
│ │ ├── ⭐ TypeScript Expert (built-in)            │ │
│ │ ├── ⭐ Code Reviewer (built-in)                │ │
│ │ └── 📝 My Custom Preset                        │ │
│ │                                                 │ │
│ │ [+ New Preset] [Edit] [Delete]                  │ │
│ └─────────────────────────────────────────────────┘ │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 5. Migration Plan

### Phase 1: State Migration

1. Add `constitution.mode` field (default: 'rules')
2. Copy `agent_rules_config.profiles` → `constitution.presets`
3. Copy `agent_rules_config.active_profile_id` → `constitution.active_preset_id`
4. Deprecate `agent_rules_config` at project level

### Phase 2: UI Migration

1. Remove Agent Rules page from sidebar
2. Add mode toggle to Constitution Panel
3. Add Presets sub-panel to Constitution Panel

### Phase 3: Cleanup

1. Remove `AgentRulesConfig` type
2. Remove Agent Rules actions
3. Update KB documentation

---

## 6. Actions (After Integration)

### Unified Actions

| Action | Description |
|--------|-------------|
| `SetConstitutionMode` | Switch between 'rules' and 'presets' |
| `SelectConstitutionPreset` | Set active preset (presets mode) |
| `CreateConstitutionPreset` | Create new custom preset |
| `UpdateConstitutionPreset` | Edit existing preset |
| `DeleteConstitutionPreset` | Remove custom preset |

### Deprecated Actions

| Old Action | Replacement |
|------------|-------------|
| `SetAgentRulesEnabled` | `SetConstitutionMode` |
| `SelectAgentProfile` | `SelectConstitutionPreset` |
| `CreateAgentProfile` | `CreateConstitutionPreset` |
| `UpdateAgentProfile` | `UpdateConstitutionPreset` |
| `DeleteAgentProfile` | `DeleteConstitutionPreset` |

---

## 7. Claude CLI Integration

### Rules Mode

```bash
# Concatenated rules injected via MCP or context
claude -p "..." --mcp-config ~/.rstn/mcp.json
```

### Presets Mode

```bash
# Full prompt replacement via system-prompt-file
claude -p "..." --system-prompt-file /tmp/rstn-preset-{id}.md
```

---

## 8. Open Questions

1. **Scope**: Should presets be worktree-level (like rules) or project-level (like current agent rules)?
   - **Recommendation**: Worktree-level for consistency

2. **Default Mode**: What should be the default mode for new projects?
   - **Recommendation**: Rules mode (more powerful, better UX for most cases)

3. **Preset + Rules Hybrid**: Should we allow combining a preset with additional rules?
   - **Recommendation**: No (keep it simple, one mode at a time)

---

## 9. Benefits

1. **Simplified UX**: One place for all AI behavior configuration
2. **Consistent Scope**: Both modes at worktree level
3. **Clear Mental Model**: Mode toggle makes the choice explicit
4. **Reduced Code**: Single panel instead of two

---

## 10. Implementation Checklist

- [ ] Update `app_state.rs` with new `ConstitutionConfig`
- [ ] Add migration for existing `AgentRulesConfig`
- [ ] Update reducer with new actions
- [ ] Update TypeScript types
- [ ] Create `PresetsPanel.tsx` component
- [ ] Add mode toggle to `ConstitutionPanel.tsx`
- [ ] Remove `AgentRulesPage.tsx`
- [ ] Update sidebar navigation
- [ ] Update E2E tests
- [ ] Update KB docs

---

## References

- `kb/architecture/10-constitution-system.md` - Current Constitution design
- `apps/desktop/src/renderer/src/features/agent-rules/` - Current Agent Rules implementation
