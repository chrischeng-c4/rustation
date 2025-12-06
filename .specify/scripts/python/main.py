#!/usr/bin/env python3
"""
Spec-Kit Workflow Automation CLI

Usage:
    python -m speckit discover              # List all features from GitHub
    python -m speckit status [FEATURE]      # Show feature status
    python -m speckit run FEATURE           # Run full workflow for feature
    python -m speckit run-all               # Run workflow for all open features
    python -m speckit specify FEATURE       # Run specify phase only
    python -m speckit clarify FEATURE       # Run clarify phase only
    python -m speckit plan FEATURE          # Run plan phase only
    python -m speckit tasks FEATURE         # Run tasks phase only
    python -m speckit analyze FEATURE       # Run analyze phase only
    python -m speckit checklist FEATURE     # Run checklist phase only
    python -m speckit implement FEATURE     # Run implement phase only
    python -m speckit review FEATURE        # Run review phase only
    python -m speckit merge FEATURE         # Merge PR and close issues
    python -m speckit create NUMBER DESC    # Create new feature issue
"""

import argparse
import logging
import subprocess
import sys
from pathlib import Path

from .claude_client import ClaudeClient
from .config import Config
from .exceptions import SpecKitError
from .feature_tracker import FeatureTracker
from .github_client import GitHubClient
from .session_manager import SessionManager
from .workflow import WorkflowOrchestrator

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
)
logger = logging.getLogger(__name__)


def get_repo_root() -> Path:
    """Find repository root."""
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return Path(result.stdout.strip())
    return Path.cwd()


def create_orchestrator(repo_root: Path, model: str = "haiku") -> WorkflowOrchestrator:
    """Create and configure the workflow orchestrator."""
    config = Config(repo_root)

    github = GitHubClient(repo_root)
    claude = ClaudeClient(
        repo_root,
        permission_mode=config.permission_mode,
        timeout=config.claude_timeout,
        model=model,
    )
    sessions = SessionManager(repo_root / ".specify/scripts/python/sessions.json")
    tracker = FeatureTracker(repo_root, github, sessions)

    return WorkflowOrchestrator(
        repo_root,
        github,
        claude,
        tracker,
        sessions,
        stop_on_error=config.stop_on_error,
    )


def cmd_discover(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
    """List all features from GitHub."""
    features = orchestrator.tracker.discover_features_from_github()

    print(f"\n{'#':>3}  {'Branch':<30}  {'Issue':>6}  {'Status':<15}")
    print("-" * 60)

    for f in features:
        issue = f"#{f.issue_number}" if f.issue_number else "-"
        print(f"{f.number:>3}  {f.branch:<30}  {issue:>6}  {f.status.value:<15}")

    print(f"\nTotal: {len(features)} features")

    # Show open features
    open_features = [f for f in features if f.status.value != "complete"]
    if open_features:
        print(f"Open: {len(open_features)} features")


def cmd_status(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
    """Show detailed status for a feature."""
    feature = orchestrator.tracker.get_feature_by_number(args.feature)
    if not feature:
        print(f"Feature {args.feature} not found")
        sys.exit(1)

    print(f"\nFeature {feature.number}: {feature.name}")
    print(f"  Branch: {feature.branch}")
    print(
        f"  Issue: #{feature.issue_number}" if feature.issue_number else "  Issue: (none)"
    )
    print(f"  Status: {feature.status.value}")
    print(f"  Spec Dir: {feature.spec_dir}")
    print(f"  Session: {feature.session_id[:8]}...")

    if feature.user_stories:
        print("\n  User Stories:")
        for us in feature.user_stories:
            print(f"    {us.id}: {us.description} (#{us.issue_number})")

    # Show spec files
    if feature.spec_dir and feature.spec_dir.exists():
        print("\n  Files:")
        for f in sorted(feature.spec_dir.iterdir()):
            if f.is_file():
                print(f"    {f.name}")


def cmd_run(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
    """Run full workflow for a feature."""
    feature = orchestrator.tracker.get_feature_by_number(args.feature)
    if not feature:
        print(f"Feature {args.feature} not found")
        sys.exit(1)

    success = orchestrator.run_full_workflow(
        feature,
        start_phase=args.start,
        end_phase=args.end,
    )

    sys.exit(0 if success else 1)


def cmd_run_all(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
    """Run workflow for all open features."""
    features = orchestrator.tracker.get_open_features()

    if not features:
        print("No open features found")
        return

    print(f"Found {len(features)} open features")

    for feature in features:
        print(f"\n{'='*60}")
        print(f"Processing Feature {feature.number}: {feature.name}")
        print(f"{'='*60}")

        try:
            success = orchestrator.run_full_workflow(feature)
            if not success:
                print(f"Feature {feature.number} failed")
                if orchestrator.stop_on_error:
                    sys.exit(1)
        except Exception as e:
            print(f"Feature {feature.number} error: {e}")
            if orchestrator.stop_on_error:
                sys.exit(1)

    print("\nAll features processed")


def cmd_run_until_done(
    args: argparse.Namespace, orchestrator: WorkflowOrchestrator
) -> None:
    """Run workflow repeatedly until all features are complete."""
    import time

    iteration = 0
    max_iterations = args.max_iterations or 100  # Safety limit

    while iteration < max_iterations:
        iteration += 1
        print(f"\n{'#'*60}")
        print(f"# Iteration {iteration}")
        print(f"{'#'*60}")

        # Refresh feature list each iteration
        features = orchestrator.tracker.get_open_features()

        if not features:
            print("\nAll features complete!")
            return

        print(f"Found {len(features)} open features")

        for feature in features:
            print(f"\n{'='*60}")
            print(f"Processing Feature {feature.number}: {feature.name}")
            print(f"  Current status: {feature.status.value}")
            print(f"{'='*60}")

            try:
                success = orchestrator.run_full_workflow(feature)
                if not success:
                    print(f"Feature {feature.number} failed at status: {feature.status.value}")
                    if orchestrator.stop_on_error:
                        sys.exit(1)
            except Exception as e:
                print(f"Feature {feature.number} error: {e}")
                if orchestrator.stop_on_error:
                    sys.exit(1)

        # Brief pause between iterations
        if args.delay:
            print(f"\nWaiting {args.delay}s before next iteration...")
            time.sleep(args.delay)

    print(f"\nReached max iterations ({max_iterations})")
    remaining = orchestrator.tracker.get_open_features()
    if remaining:
        print(f"Still {len(remaining)} features remaining")
        sys.exit(1)


def cmd_create(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
    """Create a new feature issue."""
    feature = orchestrator.create_feature_issue(args.number, args.description)
    print(f"Created Feature {feature.number}: {feature.name}")
    print(f"  Issue: #{feature.issue_number}")
    print(f"  Branch: {feature.branch}")


def cmd_merge(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
    """Merge PR and close issues for a feature."""
    feature = orchestrator.tracker.get_feature_by_number(args.feature)
    if not feature:
        print(f"Feature {args.feature} not found")
        sys.exit(1)

    # Try to find PR if not set
    if not feature.pr_number:
        pr = orchestrator.github.get_pr_for_branch(feature.branch)
        if pr:
            feature.pr_number = pr.number

    success = orchestrator.merge_and_close(feature)
    sys.exit(0 if success else 1)


def cmd_phase(phase: str):
    """Create a command handler for a specific phase."""

    def handler(args: argparse.Namespace, orchestrator: WorkflowOrchestrator) -> None:
        feature = orchestrator.tracker.get_feature_by_number(args.feature)
        if not feature:
            print(f"Feature {args.feature} not found")
            sys.exit(1)

        phase_method = getattr(orchestrator, f"run_{phase}")
        success = phase_method(feature)
        sys.exit(0 if success else 1)

    return handler


def main() -> None:
    parser = argparse.ArgumentParser(description="Spec-Kit Workflow Automation")
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose output",
    )
    parser.add_argument(
        "--model",
        choices=["haiku", "sonnet", "opus"],
        default="haiku",
        help="Claude model to use (default: haiku)",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # discover command
    subparsers.add_parser("discover", help="List all features from GitHub")

    # status command
    status_parser = subparsers.add_parser("status", help="Show feature status")
    status_parser.add_argument("feature", type=int, help="Feature number")

    # run command
    run_parser = subparsers.add_parser("run", help="Run full workflow")
    run_parser.add_argument("feature", type=int, help="Feature number")
    run_parser.add_argument("--start", help="Start from phase")
    run_parser.add_argument("--end", help="Stop at phase")

    # run-all command
    subparsers.add_parser("run-all", help="Run workflow for all open features")

    # run-until-done command
    until_done_parser = subparsers.add_parser(
        "run-until-done", help="Run workflow repeatedly until all features complete"
    )
    until_done_parser.add_argument(
        "--max-iterations",
        type=int,
        default=100,
        help="Maximum iterations (default: 100)",
    )
    until_done_parser.add_argument(
        "--delay",
        type=int,
        default=5,
        help="Delay between iterations in seconds (default: 5)",
    )

    # create command
    create_parser = subparsers.add_parser("create", help="Create new feature issue")
    create_parser.add_argument("number", type=int, help="Feature number")
    create_parser.add_argument("description", help="Feature description")

    # merge command
    merge_parser = subparsers.add_parser("merge", help="Merge PR and close issues")
    merge_parser.add_argument("feature", type=int, help="Feature number")

    # Individual phase commands
    for phase in [
        "specify",
        "clarify",
        "plan",
        "tasks",
        "analyze",
        "checklist",
        "implement",
        "review",
    ]:
        phase_parser = subparsers.add_parser(phase, help=f"Run {phase} phase")
        phase_parser.add_argument("feature", type=int, help="Feature number")

    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    repo_root = get_repo_root()
    orchestrator = create_orchestrator(repo_root, model=args.model)

    try:
        if args.command == "discover":
            cmd_discover(args, orchestrator)
        elif args.command == "status":
            cmd_status(args, orchestrator)
        elif args.command == "run":
            cmd_run(args, orchestrator)
        elif args.command == "run-all":
            cmd_run_all(args, orchestrator)
        elif args.command == "run-until-done":
            cmd_run_until_done(args, orchestrator)
        elif args.command == "create":
            cmd_create(args, orchestrator)
        elif args.command == "merge":
            cmd_merge(args, orchestrator)
        else:
            # Individual phase commands
            cmd_phase(args.command)(args, orchestrator)

    except SpecKitError as e:
        logger.error(str(e))
        sys.exit(1)
    except KeyboardInterrupt:
        print("\nAborted")
        sys.exit(130)


if __name__ == "__main__":
    main()
