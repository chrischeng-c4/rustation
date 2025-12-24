"""Resources module for rstn.

Provides access to bundled resources like system prompts.
"""

from __future__ import annotations

from importlib import resources
from pathlib import Path


def get_system_prompt_path() -> Path:
    """Get path to bundled CLAUDE.md system prompt.

    Returns:
        Path to the bundled CLAUDE.md file
    """
    # Use importlib.resources to get the path to the bundled resource
    # This works with both installed packages and development mode
    ref = resources.files("rstn.resources").joinpath("CLAUDE.md")
    # For files in the package, we can get the path directly
    return Path(str(ref))


__all__ = ["get_system_prompt_path"]
