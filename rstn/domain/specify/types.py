"""Specify domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum
from pathlib import Path

from pydantic import BaseModel, Field


class SpecStatus(str, Enum):
    """Status of a specification."""

    DRAFT = "draft"
    IN_PROGRESS = "in_progress"
    READY = "ready"
    IMPLEMENTED = "implemented"
    ARCHIVED = "archived"


class NewFeature(BaseModel):
    """A new feature to be specified."""

    model_config = {"frozen": True}

    number: str = Field(description="Feature number (e.g., '042')")
    name: str = Field(description="Feature name (kebab-case)")
    description: str = Field(description="Feature description")
    full_name: str = Field(description="Full name (e.g., '042-feature-name')")


class SpecifyConfig(BaseModel):
    """Configuration for spec generation."""

    model_config = {"frozen": True}

    project_root: Path = Field(description="Project root directory")
    specs_dir: Path = Field(description="Specs directory (relative to project root)")
    catalog_path: Path = Field(description="Features catalog path")
    start_number: int = Field(default=1, description="Starting feature number")


class SpecResult(BaseModel):
    """Result of spec generation."""

    model_config = {"frozen": True}

    success: bool = Field(description="Whether generation succeeded")
    feature: NewFeature | None = Field(default=None, description="Generated feature")
    spec_path: Path | None = Field(default=None, description="Path to spec file")
    error: str | None = Field(default=None, description="Error message if failed")


class CatalogEntry(BaseModel):
    """Entry in the features catalog."""

    model_config = {"frozen": True}

    number: str = Field(description="Feature number")
    name: str = Field(description="Feature name")
    status: SpecStatus = Field(default=SpecStatus.DRAFT, description="Feature status")
    description: str = Field(default="", description="Feature description")
    created_at: str | None = Field(default=None, description="Creation timestamp")


class FeaturesCatalog(BaseModel):
    """Catalog of all features."""

    model_config = {"frozen": True}

    features: list[CatalogEntry] = Field(
        default_factory=list, description="All features"
    )

    @property
    def next_number(self) -> int:
        """Get the next available feature number."""
        if not self.features:
            return 1
        max_num = max(int(f.number) for f in self.features)
        return max_num + 1

    def find_by_number(self, number: str) -> CatalogEntry | None:
        """Find feature by number."""
        for f in self.features:
            if f.number == number:
                return f
        return None

    def find_by_name(self, name: str) -> CatalogEntry | None:
        """Find feature by name."""
        for f in self.features:
            if f.name == name:
                return f
        return None
