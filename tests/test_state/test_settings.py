"""Tests for SettingsState."""

from __future__ import annotations

import pytest
from rstn.state.settings import SettingsState, Theme


class TestTheme:
    """Test Theme enum."""

    def test_theme_values(self) -> None:
        """Theme has expected values."""
        assert Theme.DARK == "dark"  # type: ignore
        assert Theme.LIGHT == "light"  # type: ignore
        assert Theme.AUTO == "auto"  # type: ignore


class TestSettingsStateCreation:
    """Test SettingsState creation."""

    def test_default_settings_state(self) -> None:
        """Create default settings state."""
        settings = SettingsState()

        assert settings.theme == Theme.DARK
        assert settings.mouse_enabled is True
        assert settings.auto_save is True
        assert settings.log_level == "INFO"

    def test_settings_state_with_custom_values(self) -> None:
        """Create settings state with custom values."""
        settings = SettingsState(
            theme=Theme.LIGHT,
            mouse_enabled=False,
            auto_save=False,
            log_level="DEBUG",
        )

        assert settings.theme == Theme.LIGHT
        assert settings.mouse_enabled is False
        assert settings.auto_save is False
        assert settings.log_level == "DEBUG"

    def test_settings_state_all_themes(self) -> None:
        """Settings can use all theme values."""
        dark = SettingsState(theme=Theme.DARK)
        light = SettingsState(theme=Theme.LIGHT)
        auto = SettingsState(theme=Theme.AUTO)

        assert dark.theme == Theme.DARK
        assert light.theme == Theme.LIGHT
        assert auto.theme == Theme.AUTO


class TestWithTheme:
    """Test with_theme() method."""

    def test_with_theme_changes_theme(self) -> None:
        """with_theme() changes theme."""
        settings = SettingsState(theme=Theme.DARK)
        updated = settings.with_theme(Theme.LIGHT)

        # Original unchanged
        assert settings.theme == Theme.DARK
        # New state updated
        assert updated.theme == Theme.LIGHT

    def test_with_theme_preserves_other_fields(self) -> None:
        """with_theme() preserves other fields."""
        settings = SettingsState(
            mouse_enabled=False, auto_save=False, log_level="DEBUG"
        )
        updated = settings.with_theme(Theme.LIGHT)

        assert updated.mouse_enabled is False
        assert updated.auto_save is False
        assert updated.log_level == "DEBUG"

    def test_with_theme_all_values(self) -> None:
        """with_theme() works with all theme values."""
        settings = SettingsState()

        dark = settings.with_theme(Theme.DARK)
        light = settings.with_theme(Theme.LIGHT)
        auto = settings.with_theme(Theme.AUTO)

        assert dark.theme == Theme.DARK
        assert light.theme == Theme.LIGHT
        assert auto.theme == Theme.AUTO


class TestWithMouse:
    """Test with_mouse() method."""

    def test_with_mouse_enables(self) -> None:
        """with_mouse() enables mouse support."""
        settings = SettingsState(mouse_enabled=False)
        updated = settings.with_mouse(True)

        assert settings.mouse_enabled is False
        assert updated.mouse_enabled is True

    def test_with_mouse_disables(self) -> None:
        """with_mouse() disables mouse support."""
        settings = SettingsState(mouse_enabled=True)
        updated = settings.with_mouse(False)

        assert settings.mouse_enabled is True
        assert updated.mouse_enabled is False

    def test_with_mouse_preserves_other_fields(self) -> None:
        """with_mouse() preserves other fields."""
        settings = SettingsState(theme=Theme.LIGHT, auto_save=False, log_level="ERROR")
        updated = settings.with_mouse(False)

        assert updated.theme == Theme.LIGHT
        assert updated.auto_save is False
        assert updated.log_level == "ERROR"


class TestAutoSave:
    """Test auto_save field."""

    def test_auto_save_default(self) -> None:
        """auto_save defaults to True."""
        settings = SettingsState()
        assert settings.auto_save is True

    def test_auto_save_can_be_set(self) -> None:
        """auto_save can be set to False."""
        settings = SettingsState(auto_save=False)
        assert settings.auto_save is False

    def test_auto_save_can_be_updated(self) -> None:
        """auto_save can be updated via model_copy."""
        settings = SettingsState(auto_save=True)
        updated = settings.model_copy(update={"auto_save": False})

        assert settings.auto_save is True
        assert updated.auto_save is False


class TestLogLevel:
    """Test log_level field."""

    def test_log_level_default(self) -> None:
        """log_level defaults to INFO."""
        settings = SettingsState()
        assert settings.log_level == "INFO"

    def test_log_level_can_be_set(self) -> None:
        """log_level can be set to any string."""
        settings = SettingsState(log_level="DEBUG")
        assert settings.log_level == "DEBUG"

    def test_log_level_all_valid_levels(self) -> None:
        """All standard log levels are valid."""
        levels = ["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]

        for level in levels:
            settings = SettingsState(log_level=level)
            assert settings.log_level == level
            settings.assert_invariants()  # Should not raise

    def test_log_level_case_insensitive_validation(self) -> None:
        """log_level validation is case-insensitive."""
        # Lowercase should still pass validation
        settings = SettingsState(log_level="debug")
        settings.assert_invariants()  # Should not raise

        settings = SettingsState(log_level="info")
        settings.assert_invariants()

    def test_log_level_can_be_updated(self) -> None:
        """log_level can be updated via model_copy."""
        settings = SettingsState(log_level="INFO")
        updated = settings.model_copy(update={"log_level": "DEBUG"})

        assert settings.log_level == "INFO"
        assert updated.log_level == "DEBUG"


class TestSettingsStateInvariants:
    """Test settings state invariants."""

    def test_valid_settings_invariants(self) -> None:
        """Valid settings pass invariant checks."""
        settings = SettingsState(
            theme=Theme.DARK,
            mouse_enabled=True,
            auto_save=True,
            log_level="INFO",
        )

        # Should not raise
        settings.assert_invariants()

    def test_invariant_valid_log_levels(self) -> None:
        """Valid log levels pass invariant checks."""
        valid_levels = ["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]

        for level in valid_levels:
            settings = SettingsState(log_level=level)
            settings.assert_invariants()  # Should not raise

    def test_invariant_invalid_log_level(self) -> None:
        """Invalid log level violates invariants."""
        settings = SettingsState(log_level="INVALID")

        with pytest.raises(
            AssertionError, match="Log level must be one of"
        ):
            settings.assert_invariants()

    def test_invariant_empty_log_level(self) -> None:
        """Empty log level violates invariants."""
        settings = SettingsState(log_level="")

        with pytest.raises(AssertionError, match="Log level must be one of"):
            settings.assert_invariants()

    def test_invariant_lowercase_valid(self) -> None:
        """Lowercase log levels are valid (case-insensitive check)."""
        settings = SettingsState(log_level="debug")
        settings.assert_invariants()  # Should not raise

        settings = SettingsState(log_level="warning")
        settings.assert_invariants()


class TestSettingsStateSerialization:
    """Test settings state serialization."""

    def test_settings_state_serialization(self) -> None:
        """SettingsState can be serialized."""
        settings = SettingsState(
            theme=Theme.LIGHT,
            mouse_enabled=False,
            auto_save=False,
            log_level="DEBUG",
        )

        json_str = settings.model_dump_json()
        loaded = SettingsState.model_validate_json(json_str)

        assert loaded.theme == settings.theme
        assert loaded.mouse_enabled == settings.mouse_enabled
        assert loaded.auto_save == settings.auto_save
        assert loaded.log_level == settings.log_level

    def test_settings_state_with_defaults(self) -> None:
        """SettingsState with defaults can be serialized."""
        settings = SettingsState()

        json_str = settings.model_dump_json()
        loaded = SettingsState.model_validate_json(json_str)

        assert loaded.theme == Theme.DARK
        assert loaded.mouse_enabled is True
        assert loaded.auto_save is True
        assert loaded.log_level == "INFO"

    def test_settings_state_dict_round_trip(self) -> None:
        """SettingsState can round-trip through dict."""
        settings = SettingsState(
            theme=Theme.AUTO, mouse_enabled=False, log_level="ERROR"
        )

        data = settings.model_dump()
        loaded = SettingsState.model_validate(data)

        assert loaded.theme == settings.theme
        assert loaded.mouse_enabled == settings.mouse_enabled
        assert loaded.log_level == settings.log_level

    def test_theme_enum_serialization(self) -> None:
        """Theme enum serializes by value."""
        settings = SettingsState(theme=Theme.LIGHT)

        data = settings.model_dump()
        assert data["theme"] == "light"  # Serialized as string value

        loaded = SettingsState.model_validate(data)
        assert loaded.theme == Theme.LIGHT


class TestSettingsStateImmutability:
    """Test settings state immutability."""

    def test_methods_return_new_instance(self) -> None:
        """Methods return new instances."""
        settings = SettingsState()

        updated1 = settings.with_theme(Theme.LIGHT)
        updated2 = settings.with_mouse(False)

        # All should be different instances
        assert updated1 is not settings
        assert updated2 is not settings

    def test_original_unchanged_after_updates(self) -> None:
        """Original settings unchanged after updates."""
        settings = SettingsState(
            theme=Theme.DARK, mouse_enabled=True, auto_save=True
        )

        settings.with_theme(Theme.LIGHT)
        settings.with_mouse(False)

        # Original should be unchanged
        assert settings.theme == Theme.DARK
        assert settings.mouse_enabled is True
        assert settings.auto_save is True


class TestSettingsStateChaining:
    """Test chaining settings state updates."""

    def test_chain_all_updates(self) -> None:
        """Test chaining all update methods."""
        settings = (
            SettingsState()
            .with_theme(Theme.LIGHT)
            .with_mouse(False)
            .model_copy(update={"auto_save": False, "log_level": "DEBUG"})
        )

        assert settings.theme == Theme.LIGHT
        assert settings.mouse_enabled is False
        assert settings.auto_save is False
        assert settings.log_level == "DEBUG"
        settings.assert_invariants()

    def test_chain_theme_changes(self) -> None:
        """Test chaining multiple theme changes."""
        settings = (
            SettingsState()
            .with_theme(Theme.LIGHT)
            .with_theme(Theme.AUTO)
            .with_theme(Theme.DARK)
        )

        assert settings.theme == Theme.DARK
        settings.assert_invariants()

    def test_chain_preserves_invariants(self) -> None:
        """Chained updates preserve invariants."""
        settings = (
            SettingsState(log_level="INFO")
            .with_theme(Theme.LIGHT)
            .with_mouse(False)
            .model_copy(update={"log_level": "WARNING"})
        )

        settings.assert_invariants()  # Should not raise
        assert settings.theme == Theme.LIGHT
        assert settings.mouse_enabled is False
        assert settings.log_level == "WARNING"


class TestSettingsStateEdgeCases:
    """Test edge cases for SettingsState."""

    def test_same_value_update(self) -> None:
        """Updating with same value creates new instance."""
        settings = SettingsState(theme=Theme.DARK)
        updated = settings.with_theme(Theme.DARK)

        # Even with same value, should return new instance
        assert updated is not settings
        assert updated.theme == Theme.DARK

    def test_toggle_mouse_repeatedly(self) -> None:
        """Can toggle mouse setting repeatedly."""
        settings = SettingsState()

        toggled1 = settings.with_mouse(False)
        toggled2 = toggled1.with_mouse(True)
        toggled3 = toggled2.with_mouse(False)

        assert toggled1.mouse_enabled is False
        assert toggled2.mouse_enabled is True
        assert toggled3.mouse_enabled is False

    def test_all_field_combinations(self) -> None:
        """Test with various field combinations."""
        settings = SettingsState(
            theme=Theme.AUTO,
            mouse_enabled=False,
            auto_save=False,
            log_level="CRITICAL",
        )

        assert settings.theme == Theme.AUTO
        assert settings.mouse_enabled is False
        assert settings.auto_save is False
        assert settings.log_level == "CRITICAL"
        settings.assert_invariants()
