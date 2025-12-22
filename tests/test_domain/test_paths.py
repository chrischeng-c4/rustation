"""Tests for domain paths."""

from __future__ import annotations

import os
from pathlib import Path
from unittest.mock import patch

from rstn.domain import paths


class TestRstnHome:
    """Tests for rstn_home function."""

    def test_default_home(self) -> None:
        """Test default home directory."""
        with patch.dict(os.environ, {}, clear=True):
            # Clear XDG_DATA_HOME
            os.environ.pop("XDG_DATA_HOME", None)
            home = paths.rstn_home()
            assert home == Path.home() / ".rstn"

    def test_xdg_data_home(self) -> None:
        """Test XDG_DATA_HOME override."""
        with patch.dict(os.environ, {"XDG_DATA_HOME": "/custom/data"}):
            home = paths.rstn_home()
            assert home == Path("/custom/data/rstn")


class TestRstnDirectories:
    """Tests for rstn directory functions."""

    def test_sessions_dir(self) -> None:
        """Test sessions directory."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_DATA_HOME", None)
            sessions = paths.rstn_sessions_dir()
            assert sessions == Path.home() / ".rstn" / "sessions"

    def test_logs_dir(self) -> None:
        """Test logs directory."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_DATA_HOME", None)
            logs = paths.rstn_logs_dir()
            assert logs == Path.home() / ".rstn" / "logs"

    def test_mcp_config_path(self) -> None:
        """Test MCP config path."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_DATA_HOME", None)
            mcp = paths.rstn_mcp_config_path()
            assert mcp == Path.home() / ".rstn" / "mcp-session.json"

    def test_settings_path(self) -> None:
        """Test settings path."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_DATA_HOME", None)
            settings = paths.rstn_settings_path()
            assert settings == Path.home() / ".rstn" / "settings.json"

    def test_tmp_dir(self) -> None:
        """Test tmp directory."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_DATA_HOME", None)
            tmp = paths.rstn_tmp_dir()
            assert tmp == Path.home() / ".rstn" / "tmp"


class TestPromptsDir:
    """Tests for prompts directory functions."""

    def test_prompts_dir_default(self) -> None:
        """Test default prompts directory."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_CONFIG_HOME", None)
            prompts = paths.rstn_prompts_dir()
            assert prompts == Path.home() / ".config" / "rstn" / "prompts"

    def test_prompts_dir_xdg(self) -> None:
        """Test XDG_CONFIG_HOME override."""
        with patch.dict(os.environ, {"XDG_CONFIG_HOME": "/custom/config"}):
            prompts = paths.rstn_prompts_dir()
            assert prompts == Path("/custom/config/rstn/prompts")


class TestProjectPaths:
    """Tests for project-relative path functions."""

    def test_project_rstn_dir(self) -> None:
        """Test project .rstn directory."""
        root = Path("/project")
        rstn_dir = paths.project_rstn_dir(root)
        assert rstn_dir == Path("/project/.rstn")

    def test_project_specs_dir(self) -> None:
        """Test project specs directory."""
        root = Path("/project")
        specs = paths.project_specs_dir(root)
        assert specs == Path("/project/specs")

    def test_project_specify_dir(self) -> None:
        """Test project .specify directory."""
        root = Path("/project")
        specify = paths.project_specify_dir(root)
        assert specify == Path("/project/.specify")

    def test_project_prompts_dir(self) -> None:
        """Test project prompts directory."""
        root = Path("/project")
        prompts = paths.project_prompts_dir(root)
        assert prompts == Path("/project/.rstn/prompts")


class TestFeaturePaths:
    """Tests for feature-specific path functions."""

    def test_features_catalog_path(self) -> None:
        """Test features catalog path."""
        root = Path("/project")
        catalog = paths.features_catalog_path(root)
        assert catalog == Path("/project/specs/features.json")

    def test_feature_dir(self) -> None:
        """Test feature directory path."""
        root = Path("/project")
        feature = paths.feature_dir(root, "042-worktree-management")
        assert feature == Path("/project/specs/042-worktree-management")

    def test_spec_path(self) -> None:
        """Test spec.md path."""
        root = Path("/project")
        spec = paths.spec_path(root, "042-worktree-management")
        assert spec == Path("/project/specs/042-worktree-management/spec.md")

    def test_plan_path(self) -> None:
        """Test plan.md path."""
        root = Path("/project")
        plan = paths.plan_path(root, "042-worktree-management")
        assert plan == Path("/project/specs/042-worktree-management/plan.md")


class TestPathsIntegration:
    """Integration tests for paths module."""

    def test_all_rstn_paths_under_home(self) -> None:
        """Test all rstn paths are under rstn_home."""
        with patch.dict(os.environ, {}, clear=True):
            os.environ.pop("XDG_DATA_HOME", None)
            home = paths.rstn_home()
            assert paths.rstn_sessions_dir().is_relative_to(home)
            assert paths.rstn_logs_dir().is_relative_to(home)
            assert paths.rstn_mcp_config_path().is_relative_to(home)
            assert paths.rstn_settings_path().is_relative_to(home)
            assert paths.rstn_tmp_dir().is_relative_to(home)

    def test_all_project_paths_under_root(self) -> None:
        """Test all project paths are under project root."""
        root = Path("/project")
        assert paths.project_rstn_dir(root).is_relative_to(root)
        assert paths.project_specs_dir(root).is_relative_to(root)
        assert paths.project_specify_dir(root).is_relative_to(root)
        assert paths.features_catalog_path(root).is_relative_to(root)

    def test_feature_paths_under_specs(self) -> None:
        """Test all feature paths are under specs directory."""
        root = Path("/project")
        specs = paths.project_specs_dir(root)
        feature_name = "001-test-feature"
        assert paths.feature_dir(root, feature_name).is_relative_to(specs)
        assert paths.spec_path(root, feature_name).is_relative_to(specs)
        assert paths.plan_path(root, feature_name).is_relative_to(specs)
