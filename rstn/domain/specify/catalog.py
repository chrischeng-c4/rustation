"""Features catalog management.

Pure functions for parsing and updating the catalog.
Effect creators for catalog file operations.
"""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

from rstn.domain.specify.types import (
    CatalogEntry,
    FeaturesCatalog,
    NewFeature,
    SpecStatus,
)
from rstn.effect import AppEffect, WriteFile


def parse_catalog(content: str) -> FeaturesCatalog:
    """Parse features catalog from JSON.

    Pure function - no I/O.

    Args:
        content: JSON content

    Returns:
        Parsed catalog
    """
    if not content.strip():
        return FeaturesCatalog()

    data = json.loads(content)
    features = []

    for entry in data.get("features", []):
        features.append(
            CatalogEntry(
                number=entry["number"],
                name=entry["name"],
                status=SpecStatus(entry.get("status", "draft")),
                description=entry.get("description", ""),
                created_at=entry.get("created_at"),
            )
        )

    return FeaturesCatalog(features=features)


def serialize_catalog(catalog: FeaturesCatalog, pretty: bool = True) -> str:
    """Serialize catalog to JSON.

    Pure function - no I/O.

    Args:
        catalog: Features catalog
        pretty: Whether to pretty-print

    Returns:
        JSON string
    """
    data = {
        "features": [
            {
                "number": f.number,
                "name": f.name,
                "status": f.status.value,
                "description": f.description,
                "created_at": f.created_at,
            }
            for f in catalog.features
        ]
    }

    if pretty:
        return json.dumps(data, indent=2)
    return json.dumps(data)


def add_feature_to_catalog(
    catalog: FeaturesCatalog,
    feature: NewFeature,
) -> FeaturesCatalog:
    """Add a new feature to the catalog.

    Pure function - no I/O.

    Args:
        catalog: Current catalog
        feature: Feature to add

    Returns:
        Updated catalog
    """
    new_entry = CatalogEntry(
        number=feature.number,
        name=feature.name,
        status=SpecStatus.DRAFT,
        description=feature.description,
        created_at=datetime.now().isoformat(),
    )

    return FeaturesCatalog(features=[*catalog.features, new_entry])


def update_feature_status(
    catalog: FeaturesCatalog,
    number: str,
    status: SpecStatus,
) -> FeaturesCatalog:
    """Update a feature's status.

    Pure function - no I/O.

    Args:
        catalog: Current catalog
        number: Feature number
        status: New status

    Returns:
        Updated catalog
    """
    features = []
    for f in catalog.features:
        if f.number == number:
            features.append(
                CatalogEntry(
                    number=f.number,
                    name=f.name,
                    status=status,
                    description=f.description,
                    created_at=f.created_at,
                )
            )
        else:
            features.append(f)

    return FeaturesCatalog(features=features)


def create_catalog_update_effects(
    catalog: FeaturesCatalog,
    catalog_path: Path,
) -> list[AppEffect]:
    """Create effects to update catalog file.

    Effect creator - returns effects, doesn't execute.

    Args:
        catalog: Updated catalog
        catalog_path: Path to catalog file

    Returns:
        List of effects to execute
    """
    content = serialize_catalog(catalog, pretty=True)
    return [
        WriteFile(
            path=catalog_path,
            contents=content,
        )
    ]
