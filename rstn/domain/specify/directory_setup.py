"""Feature directory setup.

Pure functions for determining feature paths.
Effect creators for directory creation.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.specify.types import NewFeature
from rstn.effect import AppEffect, CreateDirectory, WriteFile


class FeaturePaths:
    """Paths for a feature."""

    def __init__(self, feature_dir: Path):
        """Initialize feature paths.

        Args:
            feature_dir: Root directory for the feature
        """
        self.root = feature_dir
        self.spec = feature_dir / "spec.md"
        self.plan = feature_dir / "plan.md"
        self.tasks = feature_dir / "tasks.md"
        self.clarify = feature_dir / "clarify.md"
        self.artifacts = feature_dir / "artifacts"


def get_feature_paths(
    project_root: Path,
    specs_dir: str,
    full_name: str,
) -> FeaturePaths:
    """Get paths for a feature.

    Pure function - no I/O.

    Args:
        project_root: Project root directory
        specs_dir: Specs directory name (relative to project root)
        full_name: Full feature name (e.g., "042-worktree-management")

    Returns:
        Feature paths
    """
    feature_dir = project_root / specs_dir / full_name
    return FeaturePaths(feature_dir)


def create_feature_directory_effects(
    paths: FeaturePaths,
    feature: NewFeature,
) -> list[AppEffect]:
    """Create effects to set up feature directory.

    Effect creator - returns effects, doesn't execute.

    Args:
        paths: Feature paths
        feature: Feature information

    Returns:
        List of effects to execute
    """
    effects: list[AppEffect] = []

    # Create main directory
    effects.append(CreateDirectory(path=paths.root, exist_ok=True))

    # Create artifacts subdirectory
    effects.append(CreateDirectory(path=paths.artifacts, exist_ok=True))

    # Create initial spec.md with template
    spec_content = _generate_spec_template(feature)
    effects.append(WriteFile(path=paths.spec, contents=spec_content))

    return effects


def _generate_spec_template(feature: NewFeature) -> str:
    """Generate initial spec template.

    Pure function - no I/O.

    Args:
        feature: Feature information

    Returns:
        Spec template content
    """
    return f"""# {feature.full_name}

## Overview

{feature.description}

## Goals

<!-- What are the main goals of this feature? -->

## User Stories

<!-- Describe the user stories -->

### As a user...

## Technical Requirements

<!-- List technical requirements -->

## Acceptance Criteria

<!-- Define clear acceptance criteria -->

## Edge Cases

<!-- Document edge cases -->

## Testing Requirements

<!-- Define testing requirements -->

## Notes

<!-- Additional notes -->
"""
