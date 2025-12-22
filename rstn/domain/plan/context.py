"""Plan context loading.

Pure functions for building plan context.
Effect creators for loading context files.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.plan.types import PlanConfig, PlanContext
from rstn.effect import AppEffect, ReadFile


def build_plan_context(
    spec_content: str,
    clarify_content: str | None = None,
    existing_plan: str | None = None,
    codebase_context: str = "",
) -> PlanContext:
    """Build plan context from loaded content.

    Pure function - no I/O.

    Args:
        spec_content: Spec document content
        clarify_content: Optional clarification content
        existing_plan: Optional existing plan content
        codebase_context: Optional codebase context

    Returns:
        Plan context
    """
    return PlanContext(
        spec_content=spec_content,
        clarify_content=clarify_content,
        existing_plan=existing_plan,
        codebase_context=codebase_context,
    )


def create_context_load_effects(config: PlanConfig) -> list[AppEffect]:
    """Create effects to load plan context.

    Effect creator - returns effects, doesn't execute.

    Args:
        config: Plan configuration

    Returns:
        List of effects to execute
    """
    effects: list[AppEffect] = []

    # Load spec
    effects.append(ReadFile(path=config.spec_path))

    # Try to load clarify file if it exists
    clarify_path = config.spec_path.parent / "clarify.md"
    effects.append(ReadFile(path=clarify_path))

    # Try to load existing plan
    plan_path = config.output_dir / "plan.md"
    effects.append(ReadFile(path=plan_path))

    return effects


def get_context_file_paths(config: PlanConfig) -> dict[str, Path]:
    """Get paths for context files.

    Pure function - no I/O.

    Args:
        config: Plan configuration

    Returns:
        Dict mapping file type to path
    """
    return {
        "spec": config.spec_path,
        "clarify": config.spec_path.parent / "clarify.md",
        "plan": config.output_dir / "plan.md",
        "tasks": config.output_dir / "tasks.md",
    }


def extract_codebase_context(file_contents: dict[Path, str]) -> str:
    """Extract relevant codebase context from file contents.

    Pure function - no I/O.

    Args:
        file_contents: Dict mapping file paths to contents

    Returns:
        Summarized codebase context
    """
    if not file_contents:
        return ""

    lines = ["## Relevant Codebase Files", ""]

    for path, content in sorted(file_contents.items()):
        lines.append(f"### {path}")
        # Truncate long content
        preview = content[:500] + "..." if len(content) > 500 else content
        lines.append(f"```\n{preview}\n```")
        lines.append("")

    return "\n".join(lines)
