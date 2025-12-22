"""Tests for AppState.

State tests are MANDATORY following State-First Architecture:
1. Round-trip serialization (JSON + YAML)
2. State transitions
3. Invariants validation
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
import yaml
from rstn.state import AppState


class TestAppStateRoundTrip:
    """Test round-trip serialization."""

    def test_json_round_trip(self) -> None:
        """State can be serialized to JSON and deserialized back."""
        state = AppState(
            version="0.1.0",
            running=True,
            session_id="test-session",
            project_root="/test/path",
        )

        # Serialize to JSON
        json_str = state.model_dump_json()
        loaded = AppState.model_validate_json(json_str)

        assert state == loaded

    def test_yaml_round_trip(self) -> None:
        """State can be serialized to YAML and deserialized back."""
        state = AppState(
            version="0.1.0",
            running=False,
            session_id="yaml-session",
            project_root=None,
        )

        # Serialize to YAML (via JSON to ensure type safety)
        json_str = state.model_dump_json()
        data_dict = json.loads(json_str)
        yaml_str = yaml.dump(data_dict)

        # Deserialize from YAML
        data = yaml.safe_load(yaml_str)
        loaded = AppState.model_validate(data)

        assert state == loaded

    def test_file_persistence_json(self, tmp_path: Path) -> None:
        """State can be saved to and loaded from JSON file."""
        state = AppState(
            version="0.1.0",
            running=True,
            session_id="file-test",
        )

        json_file = tmp_path / "state.json"
        state.save_to_file(json_file)

        assert json_file.exists()

        loaded = AppState.load_from_file(json_file)
        assert state == loaded

    def test_file_persistence_yaml(self, tmp_path: Path) -> None:
        """State can be saved to and loaded from YAML file."""
        state = AppState(
            version="0.2.0",
            running=False,
            project_root="/yaml/path",
        )

        yaml_file = tmp_path / "state.yaml"
        state.save_to_file(yaml_file)

        assert yaml_file.exists()

        loaded = AppState.load_from_file(yaml_file)
        assert state == loaded

    def test_unsupported_file_extension(self, tmp_path: Path) -> None:
        """Saving to unsupported extension raises ValueError."""
        state = AppState()
        bad_file = tmp_path / "state.txt"

        with pytest.raises(ValueError, match="Unsupported file extension"):
            state.save_to_file(bad_file)

    def test_load_nonexistent_file(self, tmp_path: Path) -> None:
        """Loading from nonexistent file raises FileNotFoundError."""
        nonexistent = tmp_path / "does_not_exist.json"

        with pytest.raises(FileNotFoundError):
            AppState.load_from_file(nonexistent)


class TestAppStateDefaults:
    """Test default values."""

    def test_default_state(self) -> None:
        """AppState can be created with defaults."""
        state = AppState()

        assert state.version == "0.1.0"
        assert state.running is True
        assert state.session_id is None
        assert state.project_root is None

    def test_partial_initialization(self) -> None:
        """AppState can be created with partial fields."""
        state = AppState(session_id="test")

        assert state.version == "0.1.0"  # default
        assert state.running is True  # default
        assert state.session_id == "test"  # custom
        assert state.project_root is None  # default


class TestAppStateInvariants:
    """Test state invariants."""

    def test_valid_state_invariants(self) -> None:
        """Valid state passes invariant checks."""
        state = AppState(
            version="1.0.0",
            session_id="valid-session",
            project_root="/valid/path",
        )

        # Should not raise
        state.assert_invariants()

    def test_empty_version_fails(self) -> None:
        """Empty version violates invariants."""
        # Pydantic won't allow empty version due to Field constraints,
        # but we can test manual creation
        state = AppState()
        state.version = ""

        with pytest.raises(AssertionError, match="Version cannot be empty"):
            state.assert_invariants()

    def test_empty_session_id_fails(self) -> None:
        """Empty session_id string violates invariants."""
        state = AppState(session_id="   ")  # whitespace only

        with pytest.raises(AssertionError, match="Session ID cannot be empty"):
            state.assert_invariants()

    def test_empty_project_root_fails(self) -> None:
        """Empty project_root string violates invariants."""
        state = AppState(project_root="")

        with pytest.raises(AssertionError, match="Project root cannot be empty"):
            state.assert_invariants()

    def test_none_values_allowed(self) -> None:
        """None values for optional fields are allowed."""
        state = AppState(
            session_id=None,
            project_root=None,
        )

        # Should not raise
        state.assert_invariants()


class TestAppStateTransitions:
    """Test state transitions (builder pattern)."""

    def test_with_session_id(self) -> None:
        """with_session_id creates new state with updated session."""
        state = AppState()
        assert state.session_id is None

        new_state = state.with_session_id("new-session")

        # Original unchanged (immutability)
        assert state.session_id is None
        # New state updated
        assert new_state.session_id == "new-session"
        # Other fields preserved
        assert new_state.version == state.version
        assert new_state.running == state.running

    def test_with_project_root(self) -> None:
        """with_project_root creates new state with updated root."""
        state = AppState()
        assert state.project_root is None

        new_state = state.with_project_root("/new/root")

        # Original unchanged
        assert state.project_root is None
        # New state updated
        assert new_state.project_root == "/new/root"
        # Other fields preserved
        assert new_state.version == state.version

    def test_chained_transitions(self) -> None:
        """State transitions can be chained."""
        state = (
            AppState()
            .with_session_id("chained-session")
            .with_project_root("/chained/root")
        )

        assert state.session_id == "chained-session"
        assert state.project_root == "/chained/root"
        assert state.version == "0.1.0"  # default preserved
