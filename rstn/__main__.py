"""Main entry point for rstn binary.

Provides CLI interface using Click framework.
All commands share business logic with TUI through reduce() pattern.
"""

from __future__ import annotations


def main() -> None:
    """Main entry point.

    Runs the CLI application using Click.
    """
    from rstn.cli import run_cli

    run_cli()


if __name__ == "__main__":
    main()
