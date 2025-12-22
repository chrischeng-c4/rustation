"""Tests for runners domain types."""

from __future__ import annotations

from pathlib import Path

import pytest
from rstn.domain.runners.types import (
    RunnerResult,
    ScriptConfig,
)


class TestScriptConfig:
    """Tests for ScriptConfig model."""

    def test_config_minimal(self) -> None:
        """Test minimal script config."""
        config = ScriptConfig(cwd=Path("/project"))
        assert config.cwd == Path("/project")
        assert config.timeout_secs == 120  # Default
        assert config.env == {}  # Default
        assert config.capture_output is True  # Default

    def test_config_full(self) -> None:
        """Test full script config."""
        config = ScriptConfig(
            cwd=Path("/project/scripts"),
            timeout_secs=300,
            env={"DEBUG": "1", "RUST_BACKTRACE": "1"},
            capture_output=False,
        )
        assert config.cwd == Path("/project/scripts")
        assert config.timeout_secs == 300
        assert config.env == {"DEBUG": "1", "RUST_BACKTRACE": "1"}
        assert config.capture_output is False

    def test_config_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        config = ScriptConfig(
            cwd=Path("/project"),
            timeout_secs=60,
            env={"FOO": "bar"},
        )
        data = config.model_dump(mode="json")
        restored = ScriptConfig.model_validate(data)
        assert restored.timeout_secs == config.timeout_secs
        assert restored.env == config.env

    def test_config_immutable(self) -> None:
        """Test config is immutable (frozen)."""
        config = ScriptConfig(cwd=Path("/project"))
        with pytest.raises(Exception):
            config.timeout_secs = 60  # type: ignore


class TestRunnerResult:
    """Tests for RunnerResult model."""

    def test_success_result(self) -> None:
        """Test successful runner result."""
        result = RunnerResult(
            success=True,
            exit_code=0,
            stdout="Build succeeded\n",
            duration_ms=1500.5,
        )
        assert result.success is True
        assert result.exit_code == 0
        assert result.stdout == "Build succeeded\n"
        assert result.stderr == ""  # Default
        assert result.duration_ms == 1500.5

    def test_failure_result(self) -> None:
        """Test failed runner result."""
        result = RunnerResult(
            success=False,
            exit_code=1,
            stdout="",
            stderr="Error: compilation failed\n",
            duration_ms=2500.0,
        )
        assert result.success is False
        assert result.exit_code == 1
        assert result.stderr == "Error: compilation failed\n"

    def test_result_with_exit_code(self) -> None:
        """Test various exit codes."""
        # Exit code 0 = success
        result0 = RunnerResult(success=True, exit_code=0)
        assert result0.exit_code == 0

        # Exit code 1 = general error
        result1 = RunnerResult(success=False, exit_code=1)
        assert result1.exit_code == 1

        # Exit code 137 = killed by signal
        result137 = RunnerResult(success=False, exit_code=137)
        assert result137.exit_code == 137

    def test_result_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        result = RunnerResult(
            success=True,
            exit_code=0,
            stdout="OK",
            stderr="",
            duration_ms=100.0,
        )
        json_str = result.model_dump_json()
        restored = RunnerResult.model_validate_json(json_str)
        assert restored == result

    def test_result_immutable(self) -> None:
        """Test result is immutable (frozen)."""
        result = RunnerResult(success=True, exit_code=0)
        with pytest.raises(Exception):
            result.success = False  # type: ignore


class TestRunnersIntegration:
    """Integration tests for runner types."""

    def test_config_and_result_workflow(self) -> None:
        """Test using config and result together."""
        # Configure a script run
        config = ScriptConfig(
            cwd=Path("/project"),
            timeout_secs=60,
            env={"RUST_BACKTRACE": "1"},
        )

        # Simulate a result
        result = RunnerResult(
            success=True,
            exit_code=0,
            stdout="All tests passed",
            duration_ms=45000.0,
        )

        # Check timeout wasn't exceeded
        assert (result.duration_ms or 0) < config.timeout_secs * 1000

    def test_multiple_results(self) -> None:
        """Test tracking multiple runner results."""
        results = [
            RunnerResult(success=True, exit_code=0, stdout="Step 1 OK"),
            RunnerResult(success=True, exit_code=0, stdout="Step 2 OK"),
            RunnerResult(success=False, exit_code=1, stderr="Step 3 failed"),
        ]

        success_count = sum(1 for r in results if r.success)
        failure_count = sum(1 for r in results if not r.success)

        assert success_count == 2
        assert failure_count == 1

        # Overall success is False if any failed
        overall_success = all(r.success for r in results)
        assert overall_success is False
