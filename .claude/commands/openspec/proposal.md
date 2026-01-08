---
name: OpenSpec: Proposal (Gemini)
description: Generate OpenSpec proposal using Gemini CLI. Cost-efficient (40x cheaper) with large context (1M tokens) for heavy codebase exploration.
category: OpenSpec
tags: [openspec, change, gemini]
---
<!-- OPENSPEC:START -->
**What This Does**
This command delegates proposal generation to Gemini CLI, which reads GEMINI.md and executes `.gemini/commands/openspec/proposal.toml`. Gemini will explore the codebase and generate all spec files with FILE markers.

**Guardrails**
- This is a **wrapper command** that calls `gemini /openspec:proposal`
- Gemini will read code and generate specs, but NOT write implementation code
- You (Claude) orchestrate: prepare → call Gemini → parse → validate → present
- Ask clarifying questions BEFORE calling Gemini if user request is ambiguous

**Steps**

1. **Preparation**
   - Ask user for `change-id` if not provided (must be verb-led kebab-case like `add-feature`)
   - Clarify the user's request to ensure Gemini has clear instructions
   - Create change directory:
     ```bash
     mkdir -p "openspec/changes/<change-id>/specs"
     ```

2. **Gather context for Gemini**
   Run these commands and include their output in the Gemini prompt:
   ```bash
   openspec list --specs
   openspec list
   ```

3. **Call Gemini command**
   Build a comprehensive prompt for Gemini and call the command:
   ```bash
   gemini /openspec:proposal "
   ## User Request
   <user's description>

   ## Change ID
   <change-id>

   ## Existing Specs
   $(openspec list --specs)

   ## Active Changes
   $(openspec list)

   ## Instructions
   Read openspec/project.md and openspec/AGENTS.md for conventions.
   Explore the codebase to understand patterns.
   Generate complete proposal with FILE markers.
   " -o text > /tmp/gemini-proposal-<change-id>.txt
   ```

4. **Parse output and create files**
   ```bash
   cat /tmp/gemini-proposal-<change-id>.txt | \
     .claude/skills/openspec-proposal/scripts/parse-and-create-files.sh "<change-id>"
   ```

5. **Validate**
   ```bash
   openspec validate <change-id> --strict
   ```

   If validation fails:
   - Show errors to user
   - Identify common issues (scenario format, MODIFIED requirements)
   - Offer to fix manually or re-run Gemini with clarifications

6. **Present results**
   - Read and summarize `openspec/changes/<change-id>/proposal.md`
   - Count tasks from `tasks.md`
   - List generated spec deltas
   - Show validation status
   - Ask: "Should I proceed with implementation?" (use `/openspec:apply`)

**Reference**
- Gemini command: `.gemini/commands/openspec/proposal.toml`
- Gemini system prompt: `GEMINI.md` (OpenSpec Instructions section)
- Parser script: `.claude/skills/openspec-proposal/scripts/parse-and-create-files.sh`
- Validation: `openspec validate --strict`

**Cost Comparison**
- Gemini 2.0 Flash: $0.075/M tokens (1M context limit)
- Claude Sonnet: $3/M tokens (200K context limit)
- **40x cheaper** for proposal generation with 5x more context
<!-- OPENSPEC:END -->
