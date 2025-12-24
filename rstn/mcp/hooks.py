"""Hook configuration and execution for rstn MCP.

Loads project-specific hook configurations from .rstn/hooks.yaml
and executes them on request.
"""

from __future__ import annotations

import asyncio
import os
import time
from pathlib import Path

from rstn.logging import get_logger
from rstn.mcp.types import HookConfig, HookDefinition, HookResult

log = get_logger("rstn.mcp.hooks")


def load_hook_config(project_root: Path) -> HookConfig:
    """Load hook configuration from project.

    Looks for .rstn/hooks.yaml or .rstn/hooks.json

    Args:
        project_root: Project root directory

    Returns:
        HookConfig (empty if no config found)
    """
    yaml_path = project_root / ".rstn" / "hooks.yaml"
    json_path = project_root / ".rstn" / "hooks.json"

    if yaml_path.exists():
        try:
            import yaml

            with yaml_path.open() as f:
                data = yaml.safe_load(f) or {}
                return HookConfig.model_validate(data)
        except ImportError:
            log.warning("PyYAML not installed, cannot load hooks.yaml")
        except Exception as e:
            log.error("Failed to load hooks.yaml", error=str(e))

    if json_path.exists():
        try:
            return HookConfig.model_validate_json(json_path.read_text())
        except Exception as e:
            log.error("Failed to load hooks.json", error=str(e))

    return HookConfig()


async def run_hook(
    hook: HookDefinition,
    args: list[str],
    project_root: Path,
) -> HookResult:
    """Execute a hook command.

    Args:
        hook: Hook definition
        args: Additional arguments
        project_root: Project root for cwd

    Returns:
        HookResult with output and exit code
    """
    cwd = Path(hook.cwd) if hook.cwd else project_root
    full_command = f"{hook.command} {' '.join(args)}".strip()

    log.info("Running hook", command=full_command, cwd=str(cwd))

    start_time = time.monotonic()

    try:
        # Merge environment
        env = {**os.environ, **hook.env}

        proc = await asyncio.create_subprocess_shell(
            full_command,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=cwd,
            env=env,
        )

        try:
            stdout_bytes, stderr_bytes = await asyncio.wait_for(
                proc.communicate(),
                timeout=hook.timeout_secs,
            )
        except TimeoutError:
            proc.kill()
            await proc.wait()
            duration = time.monotonic() - start_time
            log.warning("Hook timed out", command=full_command, timeout=hook.timeout_secs)
            return HookResult(
                hook_name=hook.command.split()[0],
                exit_code=-1,
                stdout="",
                stderr=f"Hook timed out after {hook.timeout_secs}s",
                duration_secs=duration,
            )

        duration = time.monotonic() - start_time

        result = HookResult(
            hook_name=hook.command.split()[0],
            exit_code=proc.returncode or 0,
            stdout=stdout_bytes.decode(errors="replace"),
            stderr=stderr_bytes.decode(errors="replace"),
            duration_secs=duration,
        )

        log.info(
            "Hook completed",
            command=full_command,
            exit_code=result.exit_code,
            duration=f"{duration:.2f}s",
        )

        return result

    except Exception as e:
        duration = time.monotonic() - start_time
        log.error("Hook failed", command=full_command, error=str(e))
        return HookResult(
            hook_name=hook.command.split()[0],
            exit_code=-1,
            stdout="",
            stderr=f"Hook execution failed: {e}",
            duration_secs=duration,
        )
