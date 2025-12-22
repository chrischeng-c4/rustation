"""Specify domain operations for rstn.

Provides spec generation workflow including:
- Feature number allocation
- Feature name generation
- Directory setup
- Catalog management
- Workflow orchestration

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.specify.catalog import (
    add_feature_to_catalog,
    create_catalog_update_effects,
    parse_catalog,
)
from rstn.domain.specify.directory_setup import (
    create_feature_directory_effects,
    get_feature_paths,
)
from rstn.domain.specify.name_generator import (
    generate_feature_name,
    normalize_feature_name,
)
from rstn.domain.specify.number_allocator import (
    allocate_feature_number,
    format_feature_number,
    parse_feature_number,
)
from rstn.domain.specify.orchestrator import (
    create_spec_generation_effects,
)
from rstn.domain.specify.types import (
    FeaturesCatalog,
    NewFeature,
    SpecifyConfig,
    SpecResult,
)

__all__ = [
    # Types
    "FeaturesCatalog",
    "NewFeature",
    "SpecifyConfig",
    "SpecResult",
    # Number allocation
    "allocate_feature_number",
    "format_feature_number",
    "parse_feature_number",
    # Name generation
    "generate_feature_name",
    "normalize_feature_name",
    # Directory setup
    "get_feature_paths",
    "create_feature_directory_effects",
    # Catalog
    "parse_catalog",
    "add_feature_to_catalog",
    "create_catalog_update_effects",
    # Orchestration
    "create_spec_generation_effects",
]
