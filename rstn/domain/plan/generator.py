"""Plan generation.

Pure functions for generating plan content.
Effect creators for plan generation workflow.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.plan.types import (
    ArtifactKind,
    PlanArtifact,
    PlanConfig,
    PlanContext,
)
from rstn.effect import AppEffect, RunClaudeCli


def generate_plan_content(
    context: PlanContext,
    config: PlanConfig,
) -> str:
    """Generate plan content from context.

    Pure function - no I/O.

    This generates the initial plan structure. Full content
    is generated via Claude CLI.

    Args:
        context: Plan context
        config: Plan configuration

    Returns:
        Plan template content
    """
    lines = [
        "# Implementation Plan",
        "",
        "## Overview",
        "",
        "<!-- Overview of the implementation approach -->",
        "",
        "## Phases",
        "",
        "### Phase 1: Foundation",
        "",
        "<!-- First implementation phase -->",
        "",
        "### Phase 2: Core Implementation",
        "",
        "<!-- Second implementation phase -->",
        "",
        "### Phase 3: Integration",
        "",
        "<!-- Third implementation phase -->",
        "",
        "## Files to Modify",
        "",
        "<!-- List of files to create/modify -->",
        "",
        "## Dependencies",
        "",
        "<!-- External dependencies -->",
        "",
        "## Testing Strategy",
        "",
        "<!-- How to test the implementation -->",
        "",
        "## Risks",
        "",
        "<!-- Implementation risks -->",
        "",
    ]

    return "\n".join(lines)


def generate_tasks_content(
    context: PlanContext,
    config: PlanConfig,
) -> str:
    """Generate tasks breakdown from context.

    Pure function - no I/O.

    Args:
        context: Plan context
        config: Plan configuration

    Returns:
        Tasks template content
    """
    lines = [
        "# Implementation Tasks",
        "",
        "## Task List",
        "",
        "- [ ] Task 1: Setup",
        "- [ ] Task 2: Core implementation",
        "- [ ] Task 3: Testing",
        "- [ ] Task 4: Documentation",
        "",
        "## Task Details",
        "",
        "### Task 1: Setup",
        "",
        "<!-- Task details -->",
        "",
    ]

    return "\n".join(lines)


def create_plan_generation_effects(
    context: PlanContext,
    config: PlanConfig,
) -> list[AppEffect]:
    """Create effects for plan generation via Claude CLI.

    Effect creator - returns effects, doesn't execute.

    Args:
        context: Plan context
        config: Plan configuration

    Returns:
        List of effects to execute
    """
    prompt = _build_plan_generation_prompt(context)

    return [
        RunClaudeCli(
            prompt=prompt,
            output_format="text",
            timeout_secs=300,  # 5 minutes for plan generation
            cwd=config.project_root,
            workflow_id="plan_generation",
        )
    ]


def _build_plan_generation_prompt(context: PlanContext) -> str:
    """Build the plan generation prompt.

    Pure function - no I/O.
    """
    lines = [
        "Generate an implementation plan for the following feature specification.",
        "",
        "## Specification",
        "",
        context.spec_content,
        "",
    ]

    if context.clarify_content:
        lines.extend(
            [
                "## Clarifications",
                "",
                context.clarify_content,
                "",
            ]
        )

    if context.codebase_context:
        lines.extend(
            [
                "## Codebase Context",
                "",
                context.codebase_context,
                "",
            ]
        )

    lines.extend(
        [
            "## Requirements",
            "",
            "Generate a comprehensive implementation plan that includes:",
            "1. Overview of the approach",
            "2. Implementation phases (3-5 phases)",
            "3. Files to create/modify",
            "4. Dependencies",
            "5. Testing strategy",
            "6. Risks and mitigations",
            "",
            "Use Mermaid diagrams where helpful.",
        ]
    )

    return "\n".join(lines)


def create_plan_artifacts(
    plan_content: str,
    tasks_content: str,
    output_dir: Path,
) -> list[PlanArtifact]:
    """Create plan artifacts from generated content.

    Pure function - no I/O.

    Args:
        plan_content: Generated plan content
        tasks_content: Generated tasks content
        output_dir: Output directory

    Returns:
        List of plan artifacts
    """
    return [
        PlanArtifact(
            kind=ArtifactKind.PLAN,
            path=output_dir / "plan.md",
            content=plan_content,
        ),
        PlanArtifact(
            kind=ArtifactKind.TASKS,
            path=output_dir / "tasks.md",
            content=tasks_content,
        ),
    ]
