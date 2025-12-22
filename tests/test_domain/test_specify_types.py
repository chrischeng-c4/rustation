"""Tests for specify domain types."""

from __future__ import annotations

from pathlib import Path

import pytest
from rstn.domain.specify.types import (
    CatalogEntry,
    FeaturesCatalog,
    NewFeature,
    SpecifyConfig,
    SpecResult,
    SpecStatus,
)


class TestSpecStatus:
    """Tests for SpecStatus enum."""

    def test_spec_status_values(self) -> None:
        """Test all status values."""
        assert SpecStatus.DRAFT.value == "draft"
        assert SpecStatus.IN_PROGRESS.value == "in_progress"
        assert SpecStatus.READY.value == "ready"
        assert SpecStatus.IMPLEMENTED.value == "implemented"
        assert SpecStatus.ARCHIVED.value == "archived"

    def test_spec_status_is_string_enum(self) -> None:
        """Test SpecStatus is a string enum."""
        for status in SpecStatus:
            assert isinstance(status.value, str)
            assert status == status.value


class TestNewFeature:
    """Tests for NewFeature model."""

    def test_new_feature_creation(self) -> None:
        """Test creating new feature."""
        feature = NewFeature(
            number="042",
            name="worktree-management",
            description="Manage git worktrees",
            full_name="042-worktree-management",
        )
        assert feature.number == "042"
        assert feature.name == "worktree-management"
        assert feature.description == "Manage git worktrees"
        assert feature.full_name == "042-worktree-management"

    def test_new_feature_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        feature = NewFeature(
            number="001",
            name="test-feature",
            description="Test",
            full_name="001-test-feature",
        )
        json_str = feature.model_dump_json()
        restored = NewFeature.model_validate_json(json_str)
        assert restored == feature

    def test_new_feature_immutable(self) -> None:
        """Test feature is immutable (frozen)."""
        feature = NewFeature(
            number="001",
            name="test",
            description="Test",
            full_name="001-test",
        )
        with pytest.raises(Exception):
            feature.name = "new-name"  # type: ignore


class TestSpecifyConfig:
    """Tests for SpecifyConfig model."""

    def test_specify_config_creation(self) -> None:
        """Test creating config."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("specs/features.json"),
        )
        assert config.project_root == Path("/project")
        assert config.specs_dir == Path("specs")
        assert config.catalog_path == Path("specs/features.json")
        assert config.start_number == 1  # Default

    def test_specify_config_custom_start_number(self) -> None:
        """Test custom start number."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("specs/features.json"),
            start_number=100,
        )
        assert config.start_number == 100

    def test_specify_config_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("specs/features.json"),
            start_number=42,
        )
        data = config.model_dump(mode="json")
        restored = SpecifyConfig.model_validate(data)
        assert restored.start_number == config.start_number


class TestSpecResult:
    """Tests for SpecResult model."""

    def test_spec_result_success(self) -> None:
        """Test successful result."""
        feature = NewFeature(
            number="001",
            name="test",
            description="Test",
            full_name="001-test",
        )
        result = SpecResult(
            success=True,
            feature=feature,
            spec_path=Path("/project/specs/001-test/spec.md"),
        )
        assert result.success is True
        assert result.feature == feature
        assert result.spec_path == Path("/project/specs/001-test/spec.md")
        assert result.error is None

    def test_spec_result_failure(self) -> None:
        """Test failed result."""
        result = SpecResult(
            success=False,
            error="Feature number already exists",
        )
        assert result.success is False
        assert result.feature is None
        assert result.spec_path is None
        assert result.error == "Feature number already exists"

    def test_spec_result_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        result = SpecResult(
            success=False,
            error="Test error",
        )
        json_str = result.model_dump_json()
        restored = SpecResult.model_validate_json(json_str)
        assert restored == result


class TestCatalogEntry:
    """Tests for CatalogEntry model."""

    def test_catalog_entry_creation(self) -> None:
        """Test creating catalog entry."""
        entry = CatalogEntry(
            number="042",
            name="worktree-management",
            description="Manage worktrees",
        )
        assert entry.number == "042"
        assert entry.name == "worktree-management"
        assert entry.status == SpecStatus.DRAFT  # Default
        assert entry.description == "Manage worktrees"
        assert entry.created_at is None

    def test_catalog_entry_with_status(self) -> None:
        """Test entry with custom status."""
        entry = CatalogEntry(
            number="001",
            name="completed-feature",
            status=SpecStatus.IMPLEMENTED,
            created_at="2024-01-01T00:00:00Z",
        )
        assert entry.status == SpecStatus.IMPLEMENTED
        assert entry.created_at == "2024-01-01T00:00:00Z"

    def test_catalog_entry_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        entry = CatalogEntry(
            number="042",
            name="test",
            status=SpecStatus.IN_PROGRESS,
            description="Description",
            created_at="2024-01-01T00:00:00Z",
        )
        json_str = entry.model_dump_json()
        restored = CatalogEntry.model_validate_json(json_str)
        assert restored == entry


class TestFeaturesCatalog:
    """Tests for FeaturesCatalog model."""

    def test_empty_catalog(self) -> None:
        """Test empty catalog."""
        catalog = FeaturesCatalog()
        assert catalog.features == []
        assert catalog.next_number == 1

    def test_catalog_with_features(self) -> None:
        """Test catalog with features."""
        entries = [
            CatalogEntry(number="001", name="first"),
            CatalogEntry(number="002", name="second"),
            CatalogEntry(number="005", name="fifth"),
        ]
        catalog = FeaturesCatalog(features=entries)
        assert len(catalog.features) == 3
        assert catalog.next_number == 6  # max(1,2,5) + 1

    def test_catalog_find_by_number(self) -> None:
        """Test finding feature by number."""
        entries = [
            CatalogEntry(number="001", name="first"),
            CatalogEntry(number="002", name="second"),
        ]
        catalog = FeaturesCatalog(features=entries)
        found = catalog.find_by_number("002")
        assert found is not None
        assert found.name == "second"

    def test_catalog_find_by_number_not_found(self) -> None:
        """Test finding non-existent number."""
        catalog = FeaturesCatalog(features=[CatalogEntry(number="001", name="first")])
        assert catalog.find_by_number("999") is None

    def test_catalog_find_by_name(self) -> None:
        """Test finding feature by name."""
        entries = [
            CatalogEntry(number="001", name="first-feature"),
            CatalogEntry(number="002", name="second-feature"),
        ]
        catalog = FeaturesCatalog(features=entries)
        found = catalog.find_by_name("second-feature")
        assert found is not None
        assert found.number == "002"

    def test_catalog_find_by_name_not_found(self) -> None:
        """Test finding non-existent name."""
        catalog = FeaturesCatalog(features=[CatalogEntry(number="001", name="first")])
        assert catalog.find_by_name("nonexistent") is None

    def test_catalog_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        entries = [
            CatalogEntry(number="001", name="first", status=SpecStatus.DRAFT),
            CatalogEntry(number="002", name="second", status=SpecStatus.IMPLEMENTED),
        ]
        catalog = FeaturesCatalog(features=entries)
        json_str = catalog.model_dump_json()
        restored = FeaturesCatalog.model_validate_json(json_str)
        assert len(restored.features) == 2
        assert restored.features[0].number == "001"
        assert restored.features[1].status == SpecStatus.IMPLEMENTED

    def test_catalog_immutable(self) -> None:
        """Test catalog is immutable (frozen)."""
        catalog = FeaturesCatalog()
        with pytest.raises(Exception):
            catalog.features = []  # type: ignore
