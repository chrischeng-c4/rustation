"""Core application state.

AppState is the root of all application state. It must be fully serializable
to JSON/YAML at all times (State-First Architecture principle).
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml
from pydantic import BaseModel, Field

from rstn.state.dashboard import DashboardState
from rstn.state.settings import SettingsState
from rstn.state.types import UiState, ViewType, WorkflowId
from rstn.state.workflow import WorkflowState
from rstn.state.worktree import WorktreeViewState


class AppState(BaseModel):
    """Root application state.

    All state must be JSON/YAML serializable. No closures, thread handles,
    or non-serializable types allowed.

    Phase 2: Full state system (12 fields < 15 limit)
    """

    model_config = {"frozen": False}  # Allow updates for state transitions

    # Core metadata
    version: str = Field(default="0.1.0", description="Application version")
    running: bool = Field(default=True, description="Whether the app is running")

    # Views
    current_view: ViewType = Field(
        default=ViewType.WORKTREE, description="Currently active view"
    )
    worktree_view: WorktreeViewState = Field(
        default_factory=WorktreeViewState, description="Worktree view state"
    )
    dashboard_view: DashboardState = Field(
        default_factory=DashboardState, description="Dashboard view state"
    )
    settings_view: SettingsState = Field(
        default_factory=SettingsState, description="Settings view state"
    )

    # Workflows
    active_workflows: dict[WorkflowId, WorkflowState[str]] = Field(
        default_factory=dict, description="Active workflows by ID"
    )

    # UI state
    ui_state: UiState = Field(default_factory=UiState, description="Global UI state")

    # Session
    error_message: str | None = Field(default=None, description="Current error message")
    session_id: str | None = Field(default=None, description="Current session ID")
    project_root: str | None = Field(default=None, description="Project root directory")

    # Settings (denormalized for convenience)
    mouse_enabled: bool = Field(default=True, description="Whether mouse input is enabled")

    def save_to_file(self, path: str | Path) -> None:
        """Save state to JSON or YAML file.

        Args:
            path: File path to save to (.json or .yaml/.yml)

        Raises:
            ValueError: If file extension is not .json or .yaml/.yml
        """
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        if path.suffix == ".json":
            with path.open("w", encoding="utf-8") as f:
                f.write(self.model_dump_json(indent=2))
        elif path.suffix in {".yaml", ".yml"}:
            # Convert to dict using JSON mode to ensure all types are serializable
            json_str = self.model_dump_json()
            data: dict[str, Any] = json.loads(json_str)

            with path.open("w", encoding="utf-8") as f:
                yaml.dump(
                    data,
                    f,
                    default_flow_style=False,
                    allow_unicode=True,
                )
        else:
            raise ValueError(f"Unsupported file extension: {path.suffix}")

    @classmethod
    def load_from_file(cls, path: str | Path) -> AppState:
        """Load state from JSON or YAML file.

        Args:
            path: File path to load from (.json or .yaml/.yml)

        Returns:
            Loaded AppState instance

        Raises:
            ValueError: If file extension is not .json or .yaml/.yml
            FileNotFoundError: If file does not exist
        """
        path = Path(path)

        if path.suffix == ".json":
            with path.open("r", encoding="utf-8") as f:
                data: dict[str, Any] = json.load(f)
        elif path.suffix in {".yaml", ".yml"}:
            with path.open("r", encoding="utf-8") as f:
                data = yaml.safe_load(f)
        else:
            raise ValueError(f"Unsupported file extension: {path.suffix}")

        return cls.model_validate(data)

    def assert_invariants(self) -> None:
        """Assert state invariants.

        Raises:
            AssertionError: If any invariant is violated
        """
        # Version must not be empty
        assert self.version, "Version cannot be empty"

        # If session_id is set, it must not be empty
        if self.session_id is not None:
            assert self.session_id.strip(), "Session ID cannot be empty string"

        # If project_root is set, it must not be empty
        if self.project_root is not None:
            assert self.project_root.strip(), "Project root cannot be empty string"

        # Check sub-state invariants
        self.worktree_view.assert_invariants()
        self.dashboard_view.assert_invariants()
        self.settings_view.assert_invariants()

        # Check workflow invariants
        for workflow_id, workflow in self.active_workflows.items():
            assert (
                workflow.id == workflow_id
            ), f"Workflow ID mismatch: {workflow.id} != {workflow_id}"
            workflow.assert_invariants()

    def with_session_id(self, session_id: str) -> AppState:
        """Create a new state with updated session_id.

        Args:
            session_id: New session ID

        Returns:
            New AppState instance with updated session_id
        """
        return self.model_copy(update={"session_id": session_id})

    def with_project_root(self, project_root: str) -> AppState:
        """Create a new state with updated project_root.

        Args:
            project_root: New project root path

        Returns:
            New AppState instance with updated project_root
        """
        return self.model_copy(update={"project_root": project_root})
