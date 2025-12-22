"""Tests for features catalog management."""

from __future__ import annotations

import json
from pathlib import Path

from rstn.domain.specify.catalog import (
    add_feature_to_catalog,
    create_catalog_update_effects,
    parse_catalog,
    serialize_catalog,
    update_feature_status,
)
from rstn.domain.specify.types import (
    CatalogEntry,
    FeaturesCatalog,
    NewFeature,
    SpecStatus,
)
from rstn.effect import WriteFile


class TestParseCatalog:
    """Tests for parse_catalog function."""

    def test_parse_empty_string(self) -> None:
        """Test parsing empty string."""
        catalog = parse_catalog("")
        assert catalog.features == []

    def test_parse_whitespace_only(self) -> None:
        """Test parsing whitespace-only string."""
        catalog = parse_catalog("   \n\t  ")
        assert catalog.features == []

    def test_parse_empty_catalog(self) -> None:
        """Test parsing catalog with no features."""
        content = '{"features": []}'
        catalog = parse_catalog(content)
        assert catalog.features == []

    def test_parse_single_feature(self) -> None:
        """Test parsing catalog with one feature."""
        content = json.dumps({
            "features": [
                {
                    "number": "001",
                    "name": "test-feature",
                    "status": "draft",
                    "description": "A test feature",
                    "created_at": "2024-01-01T00:00:00",
                }
            ]
        })
        catalog = parse_catalog(content)

        assert len(catalog.features) == 1
        assert catalog.features[0].number == "001"
        assert catalog.features[0].name == "test-feature"
        assert catalog.features[0].status == SpecStatus.DRAFT
        assert catalog.features[0].description == "A test feature"

    def test_parse_multiple_features(self) -> None:
        """Test parsing catalog with multiple features."""
        content = json.dumps({
            "features": [
                {"number": "001", "name": "first"},
                {"number": "002", "name": "second", "status": "implemented"},
            ]
        })
        catalog = parse_catalog(content)

        assert len(catalog.features) == 2
        assert catalog.features[0].number == "001"
        assert catalog.features[1].status == SpecStatus.IMPLEMENTED

    def test_parse_default_status(self) -> None:
        """Test parsing feature without status uses draft."""
        content = json.dumps({
            "features": [{"number": "001", "name": "test"}]
        })
        catalog = parse_catalog(content)

        assert catalog.features[0].status == SpecStatus.DRAFT


class TestSerializeCatalog:
    """Tests for serialize_catalog function."""

    def test_serialize_empty(self) -> None:
        """Test serializing empty catalog."""
        catalog = FeaturesCatalog()
        result = serialize_catalog(catalog)

        data = json.loads(result)
        assert data["features"] == []

    def test_serialize_single_feature(self) -> None:
        """Test serializing single feature."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(
                    number="001",
                    name="test",
                    status=SpecStatus.DRAFT,
                    description="Test desc",
                    created_at="2024-01-01T00:00:00",
                )
            ]
        )
        result = serialize_catalog(catalog)

        data = json.loads(result)
        assert len(data["features"]) == 1
        assert data["features"][0]["number"] == "001"
        assert data["features"][0]["status"] == "draft"

    def test_serialize_pretty(self) -> None:
        """Test pretty serialization."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="test")]
        )
        result = serialize_catalog(catalog, pretty=True)

        # Pretty format has newlines and indentation
        assert "\n" in result
        assert "  " in result

    def test_serialize_compact(self) -> None:
        """Test compact serialization."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="test")]
        )
        result = serialize_catalog(catalog, pretty=False)

        # Compact format has no newlines
        assert "\n" not in result

    def test_serialize_roundtrip(self) -> None:
        """Test serialize then parse returns equivalent catalog."""
        original = FeaturesCatalog(
            features=[
                CatalogEntry(
                    number="001",
                    name="first",
                    status=SpecStatus.DRAFT,
                    description="First feature",
                ),
                CatalogEntry(
                    number="002",
                    name="second",
                    status=SpecStatus.IMPLEMENTED,
                ),
            ]
        )

        json_str = serialize_catalog(original)
        restored = parse_catalog(json_str)

        assert len(restored.features) == 2
        assert restored.features[0].number == "001"
        assert restored.features[1].status == SpecStatus.IMPLEMENTED


class TestAddFeatureToCatalog:
    """Tests for add_feature_to_catalog function."""

    def test_add_to_empty(self) -> None:
        """Test adding to empty catalog."""
        catalog = FeaturesCatalog()
        feature = NewFeature(
            number="001",
            name="test",
            description="Test feature",
            full_name="001-test",
        )

        updated = add_feature_to_catalog(catalog, feature)

        assert len(updated.features) == 1
        assert updated.features[0].number == "001"
        assert updated.features[0].name == "test"
        assert updated.features[0].status == SpecStatus.DRAFT
        assert updated.features[0].created_at is not None

    def test_add_to_existing(self) -> None:
        """Test adding to existing catalog."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="first")]
        )
        feature = NewFeature(
            number="002",
            name="second",
            description="Second feature",
            full_name="002-second",
        )

        updated = add_feature_to_catalog(catalog, feature)

        assert len(updated.features) == 2
        assert updated.features[0].number == "001"
        assert updated.features[1].number == "002"

    def test_add_preserves_existing(self) -> None:
        """Test adding preserves existing entries."""
        original_entry = CatalogEntry(
            number="001",
            name="first",
            status=SpecStatus.IMPLEMENTED,
            description="Original",
        )
        catalog = FeaturesCatalog(features=[original_entry])
        feature = NewFeature(
            number="002",
            name="second",
            description="New",
            full_name="002-second",
        )

        updated = add_feature_to_catalog(catalog, feature)

        # Original should be unchanged
        assert updated.features[0] == original_entry


class TestUpdateFeatureStatus:
    """Tests for update_feature_status function."""

    def test_update_existing_status(self) -> None:
        """Test updating existing feature status."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="test", status=SpecStatus.DRAFT)
            ]
        )

        updated = update_feature_status(catalog, "001", SpecStatus.IMPLEMENTED)

        assert updated.features[0].status == SpecStatus.IMPLEMENTED

    def test_update_preserves_other_fields(self) -> None:
        """Test updating status preserves other fields."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(
                    number="001",
                    name="test",
                    status=SpecStatus.DRAFT,
                    description="My description",
                    created_at="2024-01-01",
                )
            ]
        )

        updated = update_feature_status(catalog, "001", SpecStatus.IN_PROGRESS)

        assert updated.features[0].name == "test"
        assert updated.features[0].description == "My description"
        assert updated.features[0].created_at == "2024-01-01"

    def test_update_nonexistent_unchanged(self) -> None:
        """Test updating nonexistent feature leaves catalog unchanged."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="test", status=SpecStatus.DRAFT)
            ]
        )

        updated = update_feature_status(catalog, "999", SpecStatus.IMPLEMENTED)

        assert updated.features[0].status == SpecStatus.DRAFT

    def test_update_one_of_many(self) -> None:
        """Test updating one feature among many."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="first", status=SpecStatus.DRAFT),
                CatalogEntry(number="002", name="second", status=SpecStatus.DRAFT),
                CatalogEntry(number="003", name="third", status=SpecStatus.DRAFT),
            ]
        )

        updated = update_feature_status(catalog, "002", SpecStatus.READY)

        assert updated.features[0].status == SpecStatus.DRAFT
        assert updated.features[1].status == SpecStatus.READY
        assert updated.features[2].status == SpecStatus.DRAFT


class TestCreateCatalogUpdateEffects:
    """Tests for create_catalog_update_effects function."""

    def test_creates_write_effect(self) -> None:
        """Test creating write effect."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="test")]
        )
        path = Path("/project/specs/features.json")

        effects = create_catalog_update_effects(catalog, path)

        assert len(effects) == 1
        assert isinstance(effects[0], WriteFile)
        assert effects[0].path == path

    def test_effect_content_is_json(self) -> None:
        """Test effect content is valid JSON."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="001", name="test")]
        )
        path = Path("/project/specs/features.json")

        effects = create_catalog_update_effects(catalog, path)
        write_effect = effects[0]

        # Should be valid JSON
        assert isinstance(write_effect, WriteFile)
        data = json.loads(write_effect.contents)
        assert "features" in data

    def test_effect_content_matches_catalog(self) -> None:
        """Test effect content matches catalog."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="042", name="my-feature", status=SpecStatus.DRAFT)
            ]
        )
        path = Path("/project/specs/features.json")

        effects = create_catalog_update_effects(catalog, path)
        write_effect = effects[0]

        assert isinstance(write_effect, WriteFile)
        data = json.loads(write_effect.contents)
        assert data["features"][0]["number"] == "042"
        assert data["features"][0]["name"] == "my-feature"
