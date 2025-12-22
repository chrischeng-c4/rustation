"""Service monitoring operations.

Pure functions for parsing process output.
Effect creators for service monitoring.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.service.types import ServiceInfo, ServiceStatus
from rstn.effect import AppEffect, RunBashCommand


def parse_pgrep_output(output: str, service_name: str) -> ServiceInfo:
    """Parse pgrep output to determine service status.

    Pure function - no I/O.

    Args:
        output: Output from pgrep command
        service_name: Name of the service

    Returns:
        Service information
    """
    lines = output.strip().split("\n")
    if not lines or not lines[0]:
        return ServiceInfo(
            name=service_name,
            status=ServiceStatus.STOPPED,
        )

    # Try to parse PID from first line
    try:
        pid = int(lines[0].strip())
        return ServiceInfo(
            name=service_name,
            status=ServiceStatus.RUNNING,
            pid=pid,
        )
    except ValueError:
        return ServiceInfo(
            name=service_name,
            status=ServiceStatus.UNKNOWN,
        )


def parse_lsof_output(output: str, service_name: str, port: int) -> ServiceInfo:
    """Parse lsof output to check service on port.

    Pure function - no I/O.

    Args:
        output: Output from lsof command
        service_name: Name of the service
        port: Port number

    Returns:
        Service information
    """
    lines = output.strip().split("\n")
    if len(lines) < 2:  # Header + at least one result
        return ServiceInfo(
            name=service_name,
            status=ServiceStatus.STOPPED,
            port=port,
        )

    # Parse lsof output (skip header)
    for line in lines[1:]:
        parts = line.split()
        if len(parts) >= 2:
            try:
                pid = int(parts[1])
                return ServiceInfo(
                    name=service_name,
                    status=ServiceStatus.RUNNING,
                    pid=pid,
                    port=port,
                    command=parts[0] if parts else None,
                )
            except ValueError:
                continue

    return ServiceInfo(
        name=service_name,
        status=ServiceStatus.STOPPED,
        port=port,
    )


def create_service_check_effects(
    service_name: str,
    process_pattern: str | None = None,
    cwd: Path | None = None,
) -> list[AppEffect]:
    """Create effects to check a service status.

    Effect creator - returns effects, doesn't execute.

    Args:
        service_name: Name of the service
        process_pattern: Pattern to search for (defaults to service_name)
        cwd: Working directory

    Returns:
        List of effects to execute
    """
    pattern = process_pattern or service_name
    working_dir = cwd or Path.cwd()

    return [
        RunBashCommand(
            command=f"pgrep -f '{pattern}'",
            cwd=working_dir,
            effect_id=f"service_check_{service_name}",
        )
    ]


def create_service_port_check_effects(
    service_name: str,
    port: int,
    cwd: Path | None = None,
) -> list[AppEffect]:
    """Create effects to check if service is listening on port.

    Effect creator - returns effects, doesn't execute.

    Args:
        service_name: Name of the service
        port: Port to check
        cwd: Working directory

    Returns:
        List of effects to execute
    """
    working_dir = cwd or Path.cwd()

    return [
        RunBashCommand(
            command=f"lsof -i :{port} -P -n",
            cwd=working_dir,
            effect_id=f"service_port_{service_name}_{port}",
        )
    ]


def create_service_list_effects(
    service_patterns: list[str],
    cwd: Path | None = None,
) -> list[AppEffect]:
    """Create effects to list multiple services.

    Effect creator - returns effects, doesn't execute.

    Args:
        service_patterns: List of service patterns to check
        cwd: Working directory

    Returns:
        List of effects to execute
    """
    working_dir = cwd or Path.cwd()
    effects: list[AppEffect] = []

    for pattern in service_patterns:
        effects.append(
            RunBashCommand(
                command=f"pgrep -f '{pattern}'",
                cwd=working_dir,
                effect_id=f"service_check_{pattern}",
            )
        )

    return effects
