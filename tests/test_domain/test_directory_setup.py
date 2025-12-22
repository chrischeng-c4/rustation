"""Tests for feature directory setup."""

from __future__ import annotations

from pathlib import Path

from rstn.domain.specify.directory_setup import (
    FeaturePaths,
    create_feature_directory_effects,
    get_feature_paths,
)
from rstn.domain.specify.types import NewFeature
from rstn.effect import CreateDirectory, WriteFile


class TestFeaturePaths:
    """Tests for FeaturePaths class."""

    def test_feature_paths_creation(self) -> None:
        """Test creating feature paths."""
        feature_dir = Path("/project/specs/042-worktree")
        paths = FeaturePaths(feature_dir)

        assert paths.root == Path("/project/specs/042-worktree")
        assert paths.spec == Path("/project/specs/042-worktree/spec.md")
        assert paths.plan == Path("/project/specs/042-worktree/plan.md")
        assert paths.tasks == Path("/project/specs/042-worktree/tasks.md")
        assert paths.clarify == Path("/project/specs/042-worktree/clarify.md")
        assert paths.artifacts == Path("/project/specs/042-worktree/artifacts")

    def test_feature_paths_relative(self) -> None:
        """Test with relative paths."""
        feature_dir = Path("specs/001-feature")
        paths = FeaturePaths(feature_dir)

        assert paths.root == Path("specs/001-feature")
        assert paths.spec == Path("specs/001-feature/spec.md")

    def test_all_paths_under_root(self) -> None:
        """Test all paths are under root directory."""
        feature_dir = Path("/project/specs/001-test")
        paths = FeaturePaths(feature_dir)

        assert paths.spec.is_relative_to(paths.root)
        assert paths.plan.is_relative_to(paths.root)
        assert paths.tasks.is_relative_to(paths.root)
        assert paths.clarify.is_relative_to(paths.root)
        assert paths.artifacts.is_relative_to(paths.root)


class TestGetFeaturePaths:
    """Tests for get_feature_paths function."""

    def test_basic_paths(self) -> None:
        """Test getting basic feature paths."""
        paths = get_feature_paths(
            project_root=Path("/project"),
            specs_dir="specs",
            full_name="042-worktree-management",
        )

        assert paths.root == Path("/project/specs/042-worktree-management")
        assert paths.spec == Path("/project/specs/042-worktree-management/spec.md")

    def test_custom_specs_dir(self) -> None:
        """Test with custom specs directory."""
        paths = get_feature_paths(
            project_root=Path("/project"),
            specs_dir="features",
            full_name="001-auth",
        )

        assert paths.root == Path("/project/features/001-auth")

    def test_nested_project_root(self) -> None:
        """Test with nested project root."""
        paths = get_feature_paths(
            project_root=Path("/home/user/projects/my-project"),
            specs_dir="docs/specs",
            full_name="100-feature",
        )

        assert paths.root == Path("/home/user/projects/my-project/docs/specs/100-feature")


class TestCreateFeatureDirectoryEffects:
    """Tests for create_feature_directory_effects function."""

    def test_creates_directory_effects(self) -> None:
        """Test creating directory effects."""
        paths = FeaturePaths(Path("/project/specs/001-test"))
        feature = NewFeature(
            number="001",
            name="test",
            description="Test feature",
            full_name="001-test",
        )

        effects = create_feature_directory_effects(paths, feature)

        # Should have effects for directories and spec file
        assert len(effects) >= 3

        # Check for directory creation effects
        dir_effects = [e for e in effects if isinstance(e, CreateDirectory)]
        assert len(dir_effects) >= 2  # root and artifacts

        # Check for spec file write effect
        write_effects = [e for e in effects if isinstance(e, WriteFile)]
        assert len(write_effects) >= 1

    def test_spec_file_content(self) -> None:
        """Test spec file content is included."""
        paths = FeaturePaths(Path("/project/specs/042-worktree"))
        feature = NewFeature(
            number="042",
            name="worktree",
            description="Manage worktrees",
            full_name="042-worktree",
        )

        effects = create_feature_directory_effects(paths, feature)

        # Find the WriteFile effect for spec
        write_effects = [e for e in effects if isinstance(e, WriteFile)]
        spec_effect = next(
            (e for e in write_effects if "spec.md" in str(e.path)),
            None
        )

        assert spec_effect is not None
        assert "042-worktree" in spec_effect.contents
        assert "Manage worktrees" in spec_effect.contents

    def test_directory_paths(self) -> None:
        """Test directory paths in effects."""
        paths = FeaturePaths(Path("/project/specs/001-feature"))
        feature = NewFeature(
            number="001",
            name="feature",
            description="Description",
            full_name="001-feature",
        )

        effects = create_feature_directory_effects(paths, feature)
        dir_effects = [e for e in effects if isinstance(e, CreateDirectory)]

        # Should include root directory
        root_effect = next(
            (e for e in dir_effects if e.path == paths.root),
            None
        )
        assert root_effect is not None
        assert root_effect.exist_ok is True

        # Should include artifacts directory
        artifacts_effect = next(
            (e for e in dir_effects if e.path == paths.artifacts),
            None
        )
        assert artifacts_effect is not None

    def test_effect_order(self) -> None:
        """Test effects are in logical order."""
        paths = FeaturePaths(Path("/project/specs/001-test"))
        feature = NewFeature(
            number="001",
            name="test",
            description="Test",
            full_name="001-test",
        )

        effects = create_feature_directory_effects(paths, feature)

        # First effects should be directory creation
        assert isinstance(effects[0], CreateDirectory)


class TestFeaturePathsIntegration:
    """Integration tests for directory setup."""

    def test_full_workflow(self) -> None:
        """Test full directory setup workflow."""
        # Get paths
        paths = get_feature_paths(
            project_root=Path("/project"),
            specs_dir="specs",
            full_name="042-worktree-management",
        )

        # Create feature
        feature = NewFeature(
            number="042",
            name="worktree-management",
            description="Manage git worktrees",
            full_name="042-worktree-management",
        )

        # Get effects
        effects = create_feature_directory_effects(paths, feature)

        # Verify paths are consistent
        write_effects = [e for e in effects if isinstance(e, WriteFile)]
        assert any(e.path == paths.spec for e in write_effects)
