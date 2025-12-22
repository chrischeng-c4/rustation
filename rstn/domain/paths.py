"""XDG-compliant path management for rstn.

All paths are derived from XDG base directories or sensible defaults.
These are pure functions - no I/O, just path computation.
"""

from __future__ import annotations

import os
from pathlib import Path

__all__ = [
    "rstn_home",
    "rstn_sessions_dir",
    "rstn_logs_dir",
    "rstn_mcp_config_path",
    "rstn_settings_path",
    "rstn_tmp_dir",
    "rstn_prompts_dir",
    "project_rstn_dir",
    "project_specs_dir",
    "project_specify_dir",
]


def rstn_home() -> Path:
    """Get rstn home directory.

    Uses XDG_DATA_HOME or defaults to ~/.rstn
    """
    xdg_data = os.environ.get("XDG_DATA_HOME")
    if xdg_data:
        return Path(xdg_data) / "rstn"
    return Path.home() / ".rstn"


def rstn_sessions_dir() -> Path:
    """Get sessions directory.

    Stores session state files.
    """
    return rstn_home() / "sessions"


def rstn_logs_dir() -> Path:
    """Get logs directory.

    Stores application logs.
    """
    return rstn_home() / "logs"


def rstn_mcp_config_path() -> Path:
    """Get MCP session config path.

    Used for Claude CLI MCP integration.
    """
    return rstn_home() / "mcp-session.json"


def rstn_settings_path() -> Path:
    """Get settings file path.

    Stores user preferences.
    """
    return rstn_home() / "settings.json"


def rstn_tmp_dir() -> Path:
    """Get temporary directory.

    For pastes, temp files, etc.
    """
    return rstn_home() / "tmp"


def rstn_prompts_dir() -> Path:
    """Get user prompts override directory.

    User can override built-in prompts here.
    """
    xdg_config = os.environ.get("XDG_CONFIG_HOME")
    if xdg_config:
        return Path(xdg_config) / "rstn" / "prompts"
    return Path.home() / ".config" / "rstn" / "prompts"


def project_rstn_dir(project_root: Path) -> Path:
    """Get project-local rstn directory.

    Contains project-specific configuration.
    """
    return project_root / ".rstn"


def project_specs_dir(project_root: Path) -> Path:
    """Get project specs directory.

    Contains feature specifications.
    """
    return project_root / "specs"


def project_specify_dir(project_root: Path) -> Path:
    """Get project .specify directory.

    Contains templates and scripts for spec-driven development.
    """
    return project_root / ".specify"


def project_prompts_dir(project_root: Path) -> Path:
    """Get project prompts override directory.

    Project can override user/built-in prompts here.
    """
    return project_rstn_dir(project_root) / "prompts"


def features_catalog_path(project_root: Path) -> Path:
    """Get features catalog path.

    JSON file tracking all features.
    """
    return project_specs_dir(project_root) / "features.json"


def feature_dir(project_root: Path, feature_name: str) -> Path:
    """Get feature directory path.

    Args:
        project_root: Project root directory
        feature_name: Feature full name (e.g., "042-worktree-management")

    Returns:
        Path to feature directory
    """
    return project_specs_dir(project_root) / feature_name


def spec_path(project_root: Path, feature_name: str) -> Path:
    """Get spec file path.

    Args:
        project_root: Project root directory
        feature_name: Feature full name

    Returns:
        Path to spec.md file
    """
    return feature_dir(project_root, feature_name) / "spec.md"


def plan_path(project_root: Path, feature_name: str) -> Path:
    """Get plan file path.

    Args:
        project_root: Project root directory
        feature_name: Feature full name

    Returns:
        Path to plan.md file
    """
    return feature_dir(project_root, feature_name) / "plan.md"
