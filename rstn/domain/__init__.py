"""Domain logic for rstn TUI.

This module contains all business logic for spec-driven development workflows.
Domain functions are either:
1. Pure functions (no I/O) - for analysis and transformation
2. Effect creators (return list[AppEffect]) - for operations requiring I/O

Key principle: Domain functions MUST NOT do direct I/O.
"""

from __future__ import annotations

from rstn.domain.errors import DomainError
from rstn.domain.paths import (
    rstn_home,
    rstn_logs_dir,
    rstn_mcp_config_path,
    rstn_sessions_dir,
    rstn_settings_path,
    rstn_tmp_dir,
)

__all__ = [
    # Errors
    "DomainError",
    # Paths
    "rstn_home",
    "rstn_sessions_dir",
    "rstn_logs_dir",
    "rstn_mcp_config_path",
    "rstn_settings_path",
    "rstn_tmp_dir",
]
