"""Plan artifact writing.

Effect creators for writing plan artifacts.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.plan.types import PlanArtifact, PlanResult
from rstn.effect import AppEffect, CreateDirectory, WriteFile


def create_plan_write_effects(
    plan_content: str,
    output_path: Path,
) -> list[AppEffect]:
    """Create effects to write main plan file.

    Effect creator - returns effects, doesn't execute.

    Args:
        plan_content: Plan content to write
        output_path: Output path

    Returns:
        List of effects to execute
    """
    return [
        CreateDirectory(path=output_path.parent, exist_ok=True),
        WriteFile(path=output_path, contents=plan_content),
    ]


def create_artifact_write_effects(
    artifacts: list[PlanArtifact],
) -> list[AppEffect]:
    """Create effects to write plan artifacts.

    Effect creator - returns effects, doesn't execute.

    Args:
        artifacts: List of artifacts to write

    Returns:
        List of effects to execute
    """
    effects: list[AppEffect] = []

    # Collect unique parent directories
    dirs_to_create = set()
    for artifact in artifacts:
        dirs_to_create.add(artifact.path.parent)

    # Create directories first
    for dir_path in dirs_to_create:
        effects.append(CreateDirectory(path=dir_path, exist_ok=True))

    # Write artifact files
    for artifact in artifacts:
        effects.append(
            WriteFile(
                path=artifact.path,
                contents=artifact.content,
            )
        )

    return effects


def build_plan_result(
    artifacts: list[PlanArtifact],
    success: bool = True,
    error: str | None = None,
) -> PlanResult:
    """Build plan result from artifacts.

    Pure function - no I/O.

    Args:
        artifacts: Generated artifacts
        success: Whether generation succeeded
        error: Error message if failed

    Returns:
        Plan result
    """
    plan_path = None
    for artifact in artifacts:
        if artifact.kind.value == "plan":
            plan_path = artifact.path
            break

    return PlanResult(
        success=success,
        plan_path=plan_path,
        artifacts=artifacts,
        error=error,
    )
