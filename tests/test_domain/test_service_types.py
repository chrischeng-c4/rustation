"""Tests for service domain types."""

from __future__ import annotations

import pytest
from rstn.domain.service.types import (
    ServiceInfo,
    ServiceStatus,
)


class TestServiceStatus:
    """Tests for ServiceStatus enum."""

    def test_status_values(self) -> None:
        """Test all status values."""
        assert ServiceStatus.RUNNING.value == "running"
        assert ServiceStatus.STOPPED.value == "stopped"
        assert ServiceStatus.UNKNOWN.value == "unknown"

    def test_status_is_string_enum(self) -> None:
        """Test ServiceStatus is a string enum."""
        for status in ServiceStatus:
            assert isinstance(status.value, str)


class TestServiceInfo:
    """Tests for ServiceInfo model."""

    def test_running_service(self) -> None:
        """Test running service info."""
        info = ServiceInfo(
            name="rstn-mcp",
            status=ServiceStatus.RUNNING,
            pid=12345,
            command="python -m rstn.mcp",
            port=8080,
        )
        assert info.name == "rstn-mcp"
        assert info.status == ServiceStatus.RUNNING
        assert info.pid == 12345
        assert info.command == "python -m rstn.mcp"
        assert info.port == 8080

    def test_stopped_service(self) -> None:
        """Test stopped service info."""
        info = ServiceInfo(
            name="rstn-mcp",
            status=ServiceStatus.STOPPED,
        )
        assert info.name == "rstn-mcp"
        assert info.status == ServiceStatus.STOPPED
        assert info.pid is None
        assert info.command is None
        assert info.port is None

    def test_unknown_status(self) -> None:
        """Test unknown status service."""
        info = ServiceInfo(
            name="unknown-service",
            status=ServiceStatus.UNKNOWN,
        )
        assert info.status == ServiceStatus.UNKNOWN

    def test_service_info_serialization(self) -> None:
        """Test JSON serialization round-trip."""
        info = ServiceInfo(
            name="test",
            status=ServiceStatus.RUNNING,
            pid=999,
            port=3000,
        )
        json_str = info.model_dump_json()
        restored = ServiceInfo.model_validate_json(json_str)
        assert restored == info

    def test_service_info_immutable(self) -> None:
        """Test info is immutable (frozen)."""
        info = ServiceInfo(
            name="test",
            status=ServiceStatus.STOPPED,
        )
        with pytest.raises(Exception):
            info.status = ServiceStatus.RUNNING  # type: ignore
