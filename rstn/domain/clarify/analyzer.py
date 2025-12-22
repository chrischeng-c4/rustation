"""Spec coverage analysis.

Pure functions for analyzing spec content.
"""

from __future__ import annotations

from rstn.domain.clarify.types import Category, CoverageStatus, SpecCoverage

# Keywords for each category
CATEGORY_KEYWORDS: dict[Category, list[str]] = {
    Category.OVERVIEW: ["overview", "summary", "introduction", "description"],
    Category.GOALS: ["goal", "objective", "purpose", "aim"],
    Category.USER_STORIES: ["user story", "as a user", "as an", "persona"],
    Category.TECHNICAL_REQUIREMENTS: ["requirement", "must", "shall", "technical"],
    Category.ACCEPTANCE_CRITERIA: ["acceptance", "criteria", "verify", "validation"],
    Category.EDGE_CASES: ["edge case", "corner case", "boundary", "limit"],
    Category.ERROR_HANDLING: ["error", "exception", "failure", "invalid"],
    Category.TESTING: ["test", "testing", "unit test", "integration test"],
    Category.SECURITY: ["security", "auth", "permission", "access"],
    Category.PERFORMANCE: ["performance", "latency", "throughput", "scale"],
}


def analyze_spec_coverage(spec_content: str) -> list[SpecCoverage]:
    """Analyze spec content for category coverage.

    Pure function - no I/O.

    Args:
        spec_content: Spec document content

    Returns:
        List of coverage analysis for each category
    """
    content_lower = spec_content.lower()
    results: list[SpecCoverage] = []

    for category, keywords in CATEGORY_KEYWORDS.items():
        matches = _find_keyword_matches(content_lower, keywords)

        if len(matches) >= 3:
            status = CoverageStatus.COVERED
            notes = f"Found {len(matches)} references"
        elif len(matches) >= 1:
            status = CoverageStatus.PARTIAL
            notes = f"Found {len(matches)} references, may need more detail"
        else:
            status = CoverageStatus.MISSING
            notes = "No coverage found"

        # Extract excerpts (first 2 matches)
        excerpts = _extract_excerpts(spec_content, keywords[:2])

        results.append(
            SpecCoverage(
                category=category,
                status=status,
                notes=notes,
                excerpts=excerpts[:2],
            )
        )

    return results


def _find_keyword_matches(content: str, keywords: list[str]) -> list[str]:
    """Find keyword matches in content."""
    matches = []
    for keyword in keywords:
        if keyword.lower() in content:
            matches.append(keyword)
    return matches


def _extract_excerpts(content: str, keywords: list[str]) -> list[str]:
    """Extract excerpts containing keywords."""
    excerpts = []
    lines = content.split("\n")

    for line in lines:
        line_lower = line.lower()
        for keyword in keywords:
            if keyword.lower() in line_lower:
                excerpt = line.strip()[:200]  # Limit length
                if excerpt:
                    excerpts.append(excerpt)
                break

    return excerpts


def get_coverage_summary(coverages: list[SpecCoverage]) -> dict[str, int]:
    """Get summary of coverage analysis.

    Pure function - no I/O.

    Args:
        coverages: List of coverage analyses

    Returns:
        Dict with counts by status
    """
    summary = {
        "covered": 0,
        "partial": 0,
        "missing": 0,
    }

    for coverage in coverages:
        if coverage.status == CoverageStatus.COVERED:
            summary["covered"] += 1
        elif coverage.status == CoverageStatus.PARTIAL:
            summary["partial"] += 1
        else:
            summary["missing"] += 1

    return summary


def get_uncovered_categories(coverages: list[SpecCoverage]) -> list[Category]:
    """Get list of uncovered or partially covered categories.

    Pure function - no I/O.

    Args:
        coverages: List of coverage analyses

    Returns:
        List of categories needing attention
    """
    return [
        c.category
        for c in coverages
        if c.status in (CoverageStatus.MISSING, CoverageStatus.PARTIAL)
    ]
