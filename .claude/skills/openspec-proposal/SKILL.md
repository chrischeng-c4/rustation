---
name: openspec-proposal
description: Generate OpenSpec proposals using Gemini CLI for cost-efficient exploration. Gemini reads all context, explores codebase, and generates complete proposal files. 40x cheaper than Claude with 1M token context.
---

# OpenSpec Proposal Skill (Gemini-Powered)

Generate specification-driven change proposals using Gemini CLI in headless mode.

## When to Use This Skill

Automatically use when the user:
- Requests a **new feature** or **capability**
- Asks to **add/modify requirements**
- Says "**create a proposal**", "**spec this**", "**plan a feature**"
- Mentions **architecture changes** or **new patterns**
- Describes work that's **non-trivial** (>500 LOC, >5 files, complex logic)

**Do NOT use** for:
- Simple bug fixes (< 100 LOC, single file)
- Documentation-only changes
- Quick tweaks or refactoring

---

## Instructions

### Guardrails
- This skill uses **Gemini CLI** to generate proposals (cost-efficient, large context)
- Gemini will **read code and generate spec files**, but **NOT write implementation code**
- System prompt is in `/GEMINI.md` (OpenSpec Instructions section)
- Keep changes tightly scoped to the requested outcome
- Identify vague or ambiguous details and ask follow-up questions before calling Gemini

### Steps

1. **Preparation**
   - Ask user for `change-id` if not provided (must be verb-led kebab-case: `add-*`, `update-*`, etc.)
   - Clarify the user's request - ensure you understand what they want
   - Create change directory:
     ```bash
     mkdir -p "openspec/changes/<change-id>/specs"
     ```

2. **Call Gemini via command**
   Use Gemini's OpenSpec command (defined in `.gemini/commands/openspec/proposal.toml`):

   ```bash
   # Call Gemini command and save output
   gemini /openspec:proposal "<user-request>" -o text > /tmp/gemini-proposal-output.txt
   ```

   This will:
   - Automatically read `GEMINI.md` for system prompt (OpenSpec Instructions section)
   - Execute the proposal command logic
   - Return output with FILE markers

3. **Parse Gemini output and create files**
   Use the parser script:
   ```bash
   cat /tmp/gemini-proposal-output.txt | \
     .claude/skills/openspec-proposal/scripts/parse-and-create-files.sh "<change-id>"
   ```

   This extracts FILE markers and creates files in `openspec/changes/<change-id>/`

4. **Validate with OpenSpec**
   ```bash
   openspec validate <change-id> --strict
   ```

   If validation fails:
   - Show errors to user
   - Debug with: `openspec show <change-id> --json --deltas-only`
   - Offer to fix common issues (scenario format, MODIFIED requirements, etc.)
   - Can manually edit files or re-run Gemini with clarifications

5. **Present results to user**
   Show:
   - ✅ Generated files list
   - ✅ Validation status (pass/fail with details)
   - 📄 Summary of proposal.md (read and summarize key points)
   - 📋 Task count (count from tasks.md)
   - ⏭️ Next steps: "Review → Approve → Use `openspec-apply` to implement"

---

## Examples

### Example: User Request
```
User: "I want to add Docker Compose support to the project management feature"
```

**Your response:**
1. Ask clarifying questions about requirements
2. Create `openspec/changes/add-docker-compose/`
3. Draft spec deltas for new Docker Compose capabilities
4. Create ordered task list
5. Validate with `openspec validate add-docker-compose --strict`
6. Present proposal for approval

### Example: Proposal Structure
```
openspec/changes/add-docker-compose/
├── proposal.md          # Why, what, scope
├── tasks.md             # [ ] Task checklist
├── design.md            # Architecture decisions (if needed)
└── specs/
    └── docker-compose/
        └── spec.md      # ## ADDED Requirements
```

---

## Reference

- Search existing requirements: `rg -n "Requirement:|Scenario:" openspec/specs`
- Explore codebase: `rg <keyword>`, `ls`, or direct file reads
- Validation help: `openspec show <spec> --type spec`
- Full workflow: See `openspec/AGENTS.md`

---

## After Completion

Present the proposal to the user:
1. Summarize the change
2. List spec deltas created
3. Show task count and highlights
4. Confirm validation passed
5. Ask: "Should I proceed with implementation?" (triggers `openspec-apply` skill)
