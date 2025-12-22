"""Prompts domain operations for rstn.

Provides system prompt management including:
- Prompt templates for different phases
- Prompt loading and caching
- Effect creators for prompt operations

All functions are either pure (for analysis) or return effects (for I/O).
"""

from __future__ import annotations

from rstn.domain.prompts.manager import (
    build_prompt,
    create_prompt_load_effects,
    get_prompt_path,
)
from rstn.domain.prompts.types import PromptContext, SpecPhase

__all__ = [
    # Types
    "PromptContext",
    "SpecPhase",
    # Manager functions
    "build_prompt",
    "get_prompt_path",
    "create_prompt_load_effects",
]
