"""Spec generation orchestration.

Coordinates the spec generation workflow.
Effect creators for the full workflow.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.specify.catalog import add_feature_to_catalog, create_catalog_update_effects
from rstn.domain.specify.directory_setup import create_feature_directory_effects, get_feature_paths
from rstn.domain.specify.name_generator import build_full_feature_name, generate_feature_name
from rstn.domain.specify.number_allocator import allocate_feature_number
from rstn.domain.specify.types import FeaturesCatalog, NewFeature, SpecifyConfig, SpecResult
from rstn.effect import AppEffect


def prepare_new_feature(
    description: str,
    catalog: FeaturesCatalog,
) -> NewFeature:
    """Prepare a new feature from description.

    Pure function - no I/O.

    Args:
        description: Feature description
        catalog: Current features catalog

    Returns:
        New feature ready for creation
    """
    number = allocate_feature_number(catalog)
    name = generate_feature_name(description)
    full_name = build_full_feature_name(number, name)

    return NewFeature(
        number=number,
        name=name,
        description=description,
        full_name=full_name,
    )


def create_spec_generation_effects(
    description: str,
    config: SpecifyConfig,
    catalog: FeaturesCatalog,
) -> tuple[list[AppEffect], NewFeature]:
    """Create effects for full spec generation workflow.

    Effect creator - returns effects, doesn't execute.

    Args:
        description: Feature description
        config: Specify configuration
        catalog: Current features catalog

    Returns:
        Tuple of (effects to execute, new feature)
    """
    # Prepare the new feature
    feature = prepare_new_feature(description, catalog)

    # Get paths
    paths = get_feature_paths(
        config.project_root,
        str(config.specs_dir),
        feature.full_name,
    )

    effects: list[AppEffect] = []

    # 1. Create directory structure and initial spec
    effects.extend(create_feature_directory_effects(paths, feature))

    # 2. Update catalog
    updated_catalog = add_feature_to_catalog(catalog, feature)
    effects.extend(create_catalog_update_effects(updated_catalog, config.catalog_path))

    return effects, feature


def build_spec_result(
    feature: NewFeature,
    spec_path: Path,
    success: bool = True,
    error: str | None = None,
) -> SpecResult:
    """Build spec generation result.

    Pure function - no I/O.

    Args:
        feature: Generated feature
        spec_path: Path to spec file
        success: Whether generation succeeded
        error: Error message if failed

    Returns:
        Spec result
    """
    return SpecResult(
        success=success,
        feature=feature,
        spec_path=spec_path,
        error=error,
    )
