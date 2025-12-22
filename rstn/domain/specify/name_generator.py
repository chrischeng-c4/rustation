"""Feature name generation.

Pure functions for generating and normalizing feature names.
"""

from __future__ import annotations

import re


def normalize_feature_name(name: str) -> str:
    """Normalize feature name to kebab-case.

    Pure function - no I/O.

    Args:
        name: Raw feature name

    Returns:
        Normalized kebab-case name
    """
    # Convert to lowercase
    name = name.lower()

    # Replace spaces, underscores with hyphens
    name = re.sub(r"[\s_]+", "-", name)

    # Remove any non-alphanumeric characters except hyphens
    name = re.sub(r"[^a-z0-9-]", "", name)

    # Collapse multiple hyphens
    name = re.sub(r"-+", "-", name)

    # Remove leading/trailing hyphens
    name = name.strip("-")

    return name


def generate_feature_name(description: str, max_words: int = 4) -> str:
    """Generate feature name from description.

    Pure function - no I/O.

    Extracts key words from description and creates kebab-case name.

    Args:
        description: Feature description
        max_words: Maximum words to include

    Returns:
        Generated kebab-case name
    """
    # Normalize first
    normalized = normalize_feature_name(description)

    # Split into words
    words = normalized.split("-")

    # Filter out common stop words
    stop_words = {"a", "an", "the", "to", "for", "and", "or", "in", "on", "at", "of"}
    words = [w for w in words if w and w not in stop_words]

    # Take up to max_words
    words = words[:max_words]

    return "-".join(words)


def build_full_feature_name(number: str, name: str) -> str:
    """Build full feature name (number-name).

    Pure function - no I/O.

    Args:
        number: Feature number (e.g., "042")
        name: Feature name (e.g., "worktree-management")

    Returns:
        Full name (e.g., "042-worktree-management")
    """
    return f"{number}-{name}"


def validate_feature_name(name: str) -> bool:
    """Validate feature name format.

    Pure function - no I/O.

    Args:
        name: Feature name to validate

    Returns:
        True if valid kebab-case name
    """
    if not name:
        return False

    # Must be kebab-case: lowercase letters, numbers, hyphens
    # Must not start or end with hyphen
    pattern = r"^[a-z0-9]+(-[a-z0-9]+)*$"
    return bool(re.match(pattern, name))
