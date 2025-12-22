"""Feature number allocation.

Pure functions for allocating and formatting feature numbers.
"""

from __future__ import annotations

from rstn.domain.specify.types import FeaturesCatalog


def allocate_feature_number(catalog: FeaturesCatalog) -> str:
    """Allocate the next feature number.

    Pure function - no I/O.

    Args:
        catalog: Current features catalog

    Returns:
        Next available feature number (zero-padded)
    """
    next_num = catalog.next_number
    return format_feature_number(next_num)


def format_feature_number(number: int) -> str:
    """Format feature number as zero-padded string.

    Pure function - no I/O.

    Args:
        number: Feature number

    Returns:
        Zero-padded string (e.g., "042")
    """
    return f"{number:03d}"


def parse_feature_number(number_str: str) -> int:
    """Parse feature number from string.

    Pure function - no I/O.

    Args:
        number_str: Feature number string (e.g., "042")

    Returns:
        Integer feature number

    Raises:
        ValueError: If string is not a valid number
    """
    return int(number_str)


def validate_feature_number(number: str, catalog: FeaturesCatalog) -> bool:
    """Check if feature number is valid and available.

    Pure function - no I/O.

    Args:
        number: Feature number to check
        catalog: Current features catalog

    Returns:
        True if valid and available
    """
    try:
        num = parse_feature_number(number)
        if num <= 0:
            return False
    except ValueError:
        return False

    # Check not already used
    return catalog.find_by_number(number) is None
