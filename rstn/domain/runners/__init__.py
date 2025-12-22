"""Script runner domain operations for rstn.

Provides script execution including:
- Bash script execution
- Cargo command execution
- Python script execution

All functions are effect creators that return effects for execution.
"""

from __future__ import annotations

from rstn.domain.runners.bash import (
    create_bash_command_effects,
    create_bash_script_effects,
    parse_bash_output,
)
from rstn.domain.runners.cargo import (
    create_cargo_effects,
    parse_cargo_json_output,
)
from rstn.domain.runners.python import (
    create_python_script_effects,
    create_uv_run_effects,
    parse_python_output,
)
from rstn.domain.runners.types import (
    RunnerResult,
    ScriptConfig,
)

__all__ = [
    # Types
    "RunnerResult",
    "ScriptConfig",
    # Bash functions
    "create_bash_command_effects",
    "create_bash_script_effects",
    "parse_bash_output",
    # Cargo functions
    "create_cargo_effects",
    "parse_cargo_json_output",
    # Python functions
    "create_python_script_effects",
    "create_uv_run_effects",
    "parse_python_output",
]
