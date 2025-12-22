"""Service domain types.

All types are Pydantic models for JSON serialization.
"""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field


class ServiceStatus(str, Enum):
    """Status of a service."""

    RUNNING = "running"
    STOPPED = "stopped"
    UNKNOWN = "unknown"


class ServiceInfo(BaseModel):
    """Information about a monitored service."""

    model_config = {"frozen": True}

    name: str = Field(description="Service name")
    status: ServiceStatus = Field(description="Service status")
    pid: int | None = Field(default=None, description="Process ID if running")
    command: str | None = Field(default=None, description="Command line if available")
    port: int | None = Field(default=None, description="Port number if applicable")
