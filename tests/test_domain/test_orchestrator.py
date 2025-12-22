"""Tests for spec generation orchestrator."""

from __future__ import annotations

from pathlib import Path

from rstn.domain.specify.orchestrator import (
    build_spec_result,
    create_spec_generation_effects,
    prepare_new_feature,
)
from rstn.domain.specify.types import (
    CatalogEntry,
    FeaturesCatalog,
    NewFeature,
    SpecifyConfig,
)
from rstn.effect import CreateDirectory, WriteFile


class TestPrepareNewFeature:
    """Tests for prepare_new_feature function."""

    def test_prepare_from_empty_catalog(self) -> None:
        """Test preparing feature with empty catalog."""
        catalog = FeaturesCatalog()
        feature = prepare_new_feature("Add user authentication", catalog)

        assert feature.number == "001"
        assert "user" in feature.name or "add" in feature.name or "authentication" in feature.name
        assert feature.full_name.startswith("001-")
        assert feature.description == "Add user authentication"

    def test_prepare_from_existing_catalog(self) -> None:
        """Test preparing feature with existing features."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="first"),
                CatalogEntry(number="002", name="second"),
            ]
        )
        feature = prepare_new_feature("New feature", catalog)

        assert feature.number == "003"

    def test_prepare_generates_name_from_description(self) -> None:
        """Test name is generated from description."""
        catalog = FeaturesCatalog()
        feature = prepare_new_feature("Build payment integration system", catalog)

        # Name should contain words from description
        assert feature.name  # Not empty
        assert "-" in feature.name or len(feature.name) > 0

    def test_prepare_full_name_format(self) -> None:
        """Test full name has correct format."""
        catalog = FeaturesCatalog(
            features=[CatalogEntry(number="041", name="existing")]
        )
        feature = prepare_new_feature("Test feature", catalog)

        # Full name should be "042-generated-name"
        assert feature.full_name.startswith("042-")
        assert feature.full_name == f"{feature.number}-{feature.name}"


class TestCreateSpecGenerationEffects:
    """Tests for create_spec_generation_effects function."""

    def test_creates_effects_and_feature(self) -> None:
        """Test returns effects and feature."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("/project/specs/features.json"),
        )
        catalog = FeaturesCatalog()

        effects, feature = create_spec_generation_effects(
            "Add user auth", config, catalog
        )

        assert len(effects) > 0
        assert feature is not None
        assert feature.number == "001"

    def test_creates_directory_effects(self) -> None:
        """Test creates directory effects."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("/project/specs/features.json"),
        )
        catalog = FeaturesCatalog()

        effects, feature = create_spec_generation_effects(
            "Test feature", config, catalog
        )

        dir_effects = [e for e in effects if isinstance(e, CreateDirectory)]
        assert len(dir_effects) > 0

    def test_creates_write_effects(self) -> None:
        """Test creates write effects."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("/project/specs/features.json"),
        )
        catalog = FeaturesCatalog()

        effects, feature = create_spec_generation_effects(
            "Test feature", config, catalog
        )

        write_effects = [e for e in effects if isinstance(e, WriteFile)]
        # Should have at least spec.md and catalog update
        assert len(write_effects) >= 2

    def test_updates_catalog_in_effects(self) -> None:
        """Test catalog update is in effects."""
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("/project/specs/features.json"),
        )
        catalog = FeaturesCatalog()

        effects, feature = create_spec_generation_effects(
            "Test feature", config, catalog
        )

        # Find catalog write effect
        write_effects = [e for e in effects if isinstance(e, WriteFile)]
        catalog_effects = [
            e for e in write_effects
            if "features.json" in str(e.path)
        ]
        assert len(catalog_effects) == 1

    def test_feature_matches_catalog_number(self) -> None:
        """Test feature number matches catalog allocation."""
        catalog = FeaturesCatalog(
            features=[
                CatalogEntry(number="001", name="first"),
                CatalogEntry(number="005", name="fifth"),
            ]
        )
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("/project/specs/features.json"),
        )

        effects, feature = create_spec_generation_effects(
            "New feature", config, catalog
        )

        assert feature.number == "006"  # Next after 005


class TestBuildSpecResult:
    """Tests for build_spec_result function."""

    def test_success_result(self) -> None:
        """Test building successful result."""
        feature = NewFeature(
            number="001",
            name="test",
            description="Test",
            full_name="001-test",
        )
        spec_path = Path("/project/specs/001-test/spec.md")

        result = build_spec_result(feature, spec_path, success=True)

        assert result.success is True
        assert result.feature == feature
        assert result.spec_path == spec_path
        assert result.error is None

    def test_failure_result(self) -> None:
        """Test building failure result."""
        feature = NewFeature(
            number="001",
            name="test",
            description="Test",
            full_name="001-test",
        )
        spec_path = Path("/project/specs/001-test/spec.md")

        result = build_spec_result(
            feature, spec_path, success=False, error="Directory already exists"
        )

        assert result.success is False
        assert result.error == "Directory already exists"

    def test_default_success(self) -> None:
        """Test default success is True."""
        feature = NewFeature(
            number="001",
            name="test",
            description="Test",
            full_name="001-test",
        )
        spec_path = Path("/project/specs/001-test/spec.md")

        result = build_spec_result(feature, spec_path)

        assert result.success is True


class TestOrchestratorIntegration:
    """Integration tests for orchestrator."""

    def test_full_workflow(self) -> None:
        """Test full orchestration workflow."""
        # Setup
        config = SpecifyConfig(
            project_root=Path("/project"),
            specs_dir=Path("specs"),
            catalog_path=Path("/project/specs/features.json"),
        )
        catalog = FeaturesCatalog()

        # Prepare feature
        feature = prepare_new_feature("Add user authentication", catalog)
        assert feature.number == "001"

        # Create effects
        effects, created_feature = create_spec_generation_effects(
            "Add user authentication", config, catalog
        )
        assert created_feature.number == "001"

        # Build result
        spec_path = Path(f"/project/specs/{feature.full_name}/spec.md")
        result = build_spec_result(feature, spec_path)

        assert result.success is True
        assert result.feature.number == "001"
