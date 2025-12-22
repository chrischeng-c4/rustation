"""Settings view state.

State for application settings and preferences.
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field


class Theme(str, Enum):
    """UI theme options."""

    DARK = "dark"
    LIGHT = "light"
    AUTO = "auto"


class SettingsState(BaseModel):
    """Settings view state.

    User preferences and configuration.
    """

    model_config = {"frozen": False}

    theme: Theme = Field(default=Theme.DARK, description="UI theme")
    mouse_enabled: bool = Field(default=True, description="Enable mouse support")
    auto_save: bool = Field(default=True, description="Auto-save state on changes")
    log_level: str = Field(default="INFO", description="Logging level")

    def with_theme(self, theme: Theme) -> SettingsState:
        """Update theme setting.

        Args:
            theme: New theme

        Returns:
            New SettingsState with updated theme
        """
        return self.model_copy(update={"theme": theme})

    def with_mouse(self, enabled: bool) -> SettingsState:
        """Update mouse support setting.

        Args:
            enabled: Whether to enable mouse

        Returns:
            New SettingsState with updated mouse setting
        """
        return self.model_copy(update={"mouse_enabled": enabled})

    def assert_invariants(self) -> None:
        """Assert settings state invariants.

        Raises:
            AssertionError: If any invariant is violated
        """
        # Log level must be valid
        valid_levels = {"DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"}
        assert (
            self.log_level.upper() in valid_levels
        ), f"Log level must be one of {valid_levels}"
