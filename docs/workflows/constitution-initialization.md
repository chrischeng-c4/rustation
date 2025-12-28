# Constitution Initialization Workflow

**Status**: ✅ Implemented (Phase 1 of CESDD)

## Overview

The Constitution Initialization workflow helps you create a project-specific **constitution file** (`.rstn/constitution.md`) that defines governance rules for AI-assisted development. This constitution serves as the foundation for AI agents working on your project.

## What is a Project Constitution?

A project constitution is a **living document** that defines:

- **Technology Stack**: What frameworks, languages, and tools your project uses
- **Security Requirements**: What security rules all code must follow
- **Code Quality Standards**: Testing, linting, and documentation requirements
- **Architectural Constraints**: Design patterns and principles your codebase follows

This document guides AI assistants (like Claude Code) to make decisions aligned with your project's standards.

## How to Use

### 1. Open Tasks Tab

Navigate to the **Tasks** tab in rstn.

### 2. Select "Initialize Constitution"

Click on the **"Initialize Constitution"** command in the commands list. This is a special workflow command (like Claude Code).

### 3. Answer Guided Questions

The workflow will ask you 4 questions:

1. **Technology Stack**
   - Example: "React + Rust (napi-rs), TypeScript, Tauri"
   - Hint: List primary frameworks and languages

2. **Security Requirements**
   - Example: "JWT auth required, no SQL injection, sanitize all user input"
   - Hint: What security rules MUST all code follow?

3. **Code Quality Standards**
   - Example: "80% test coverage, ESLint clean, TypeScript strict mode"
   - Hint: Testing, linting, and documentation requirements

4. **Architectural Constraints**
   - Example: "State-first principle, no singletons, reducer pattern"
   - Hint: Design patterns and architectural decisions

**Tips**:
- Be specific and actionable
- Think about what you'd tell a new developer joining the project
- Focus on "MUST" rules, not "SHOULD" preferences

### 4. Generate Constitution

After answering all 4 questions, click **"Generate Constitution"**.

Claude Code will generate a comprehensive constitution document based on your answers. You'll see the output streaming in real-time.

### 5. Review & Use

Once complete, your constitution is saved to:

```
<project-root>/.rstn/constitution.md
```

You can:
- ✅ View the generated constitution in the UI
- ✅ Edit `.rstn/constitution.md` manually to refine rules
- ✅ Commit it to version control so all team members follow the same standards

## Constitution File Structure

The generated constitution follows this structure:

```markdown
# Project Constitution

## Technology Stack
{detailed rules based on your tech_stack answer}

## Security Requirements
{detailed rules based on your security answer}

## Code Quality Standards
{detailed rules based on your code_quality answer}

## Architectural Constraints
{detailed rules based on your architecture answer}
```

Each section contains:
- **Authoritative rules** using "MUST" / "MUST NOT" language
- **Specific examples** based on your project
- **Rationale** for each rule

## Example Constitution

Based on answers:
- Tech Stack: "React + Rust (napi-rs)"
- Security: "JWT auth required"
- Code Quality: "80% test coverage"
- Architecture: "State-first principle"

Claude generates:

```markdown
# Project Constitution

## Technology Stack

This project MUST use:
- **Frontend**: React with TypeScript
- **Backend**: Rust with napi-rs bindings
- **State Management**: Reducer pattern (state-first)

All new features MUST follow the React + Rust integration pattern established in `packages/core/`.

## Security Requirements

All code MUST:
- Use JWT tokens for authentication
- Validate and sanitize all user input
- Never expose API keys or secrets in frontend code

## Code Quality Standards

All pull requests MUST:
- Achieve minimum 80% test coverage
- Pass `cargo clippy` with no warnings
- Pass `eslint` with no errors
- Include unit tests for new functions

## Architectural Constraints

This project follows **State-First Architecture**:
- All state MUST be serializable (JSON/YAML)
- UI MUST be a pure function of state: UI = render(State)
- Business logic MUST live in Rust backend, NOT React components
- State changes MUST go through `dispatch(action) → reducer → new state`
```

## Next Steps (Future Phases)

After initializing your constitution, future CESDD workflows will use it:

- **Phase 2 (Specify)**: Generate feature proposals that respect the constitution
- **Phase 3 (Plan)**: Create implementation plans aligned with the constitution
- **Phase 4 (Implement)**: Execute plans with constitution-aware AI agents

## Troubleshooting

### "Constitution saved to .rstn/constitution.md" doesn't appear

- Ensure you have write permissions in the project directory
- Check the terminal for error messages
- Verify `.rstn/` directory was created

### Constitution content seems generic

- Provide more specific answers to the questions
- You can manually edit `.rstn/constitution.md` after generation
- Re-run the workflow with better answers

### Can I use this without Claude Code CLI?

No. The Constitution Initialization workflow requires Claude Code CLI to be installed and available in your PATH.

Install from: https://docs.claude.com/claude-code

## Related

- [CESDD Architecture](../../kb/architecture/08-workflow-cesdd.md) - Full CESDD system design
- [State-First Principle](../../kb/architecture/02-state-first-principle.md) - Why state serialization matters
