"""Prompt management operations.

Pure functions for building prompts.
Effect creators for prompt file operations.
"""

from __future__ import annotations

from pathlib import Path

from rstn.domain.prompts.types import PromptContext, SpecPhase
from rstn.effect import AppEffect, ReadFile

# Base prompt templates for each phase
PROMPT_TEMPLATES: dict[SpecPhase, str] = {
    SpecPhase.SPECIFY: """You are helping create a feature specification.

Project: {project_root}
Feature: {feature_name}

Generate a comprehensive spec document that includes:
1. Feature overview and goals
2. User stories and acceptance criteria
3. Technical requirements
4. Edge cases and error handling
5. Testing requirements

{additional_context}
""",
    SpecPhase.CLARIFY: """You are clarifying a feature specification.

Spec Path: {spec_path}
Feature: {feature_name}

Review the spec and generate clarifying questions for any ambiguous areas.
Focus on:
1. Unclear requirements
2. Missing edge cases
3. Undefined behaviors
4. Integration concerns

{additional_context}
""",
    SpecPhase.PLAN: """You are creating an implementation plan.

Spec Path: {spec_path}
Feature: {feature_name}

Create a detailed implementation plan that includes:
1. Implementation phases
2. File changes required
3. Dependencies
4. Testing strategy
5. Risk assessment

{additional_context}
""",
    SpecPhase.IMPLEMENT: """You are implementing a feature.

Plan Path: {plan_path}
Feature: {feature_name}

Follow the implementation plan and:
1. Create/modify files as specified
2. Write tests for new functionality
3. Update documentation
4. Follow project conventions

{additional_context}
""",
    SpecPhase.REVIEW: """You are reviewing implementation.

Feature: {feature_name}

Review the implementation for:
1. Correctness
2. Test coverage
3. Documentation
4. Code quality
5. Security concerns

{additional_context}
""",
}


def get_prompt_path(phase: SpecPhase, prompts_dir: Path) -> Path:
    """Get path to prompt file for a phase.

    Pure function - no I/O.

    Args:
        phase: Workflow phase
        prompts_dir: Directory containing prompt files

    Returns:
        Path to prompt file
    """
    return prompts_dir / f"{phase.value}.md"


def build_prompt(
    context: PromptContext,
    custom_template: str | None = None,
) -> str:
    """Build a prompt from context.

    Pure function - no I/O.

    Args:
        context: Prompt context
        custom_template: Optional custom template to use

    Returns:
        Formatted prompt string
    """
    template = custom_template or PROMPT_TEMPLATES.get(context.phase, "")

    # Build additional context string
    additional = ""
    for key, value in context.additional_context.items():
        additional += f"\n{key}: {value}"

    # Format template
    return template.format(
        project_root=context.project_root,
        feature_name=context.feature_name or "Unknown",
        feature_number=context.feature_number or "",
        spec_path=context.spec_path or "",
        plan_path=context.plan_path or "",
        additional_context=additional,
    )


def create_prompt_load_effects(
    phase: SpecPhase,
    prompts_dir: Path,
) -> list[AppEffect]:
    """Create effects to load a prompt file.

    Effect creator - returns effects, doesn't execute.

    Args:
        phase: Workflow phase
        prompts_dir: Directory containing prompt files

    Returns:
        List of effects to execute
    """
    prompt_path = get_prompt_path(phase, prompts_dir)
    return [
        ReadFile(path=prompt_path),
    ]


def create_prompt_context(
    phase: SpecPhase,
    project_root: Path,
    feature_name: str | None = None,
    feature_number: str | None = None,
    spec_path: Path | None = None,
    plan_path: Path | None = None,
    **kwargs: str,
) -> PromptContext:
    """Create a prompt context.

    Pure function - no I/O.

    Args:
        phase: Workflow phase
        project_root: Project root directory
        feature_name: Optional feature name
        feature_number: Optional feature number
        spec_path: Optional spec file path
        plan_path: Optional plan file path
        **kwargs: Additional context key-value pairs

    Returns:
        Prompt context
    """
    return PromptContext(
        phase=phase,
        project_root=project_root,
        feature_name=feature_name,
        feature_number=feature_number,
        spec_path=spec_path,
        plan_path=plan_path,
        additional_context=kwargs,
    )
