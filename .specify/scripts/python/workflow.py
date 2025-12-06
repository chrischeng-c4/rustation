"""Orchestrates the full spec-kit workflow."""

import logging
import subprocess
from pathlib import Path

from .claude_client import ClaudeClient
from .exceptions import WorkflowError
from .feature_tracker import FeatureTracker
from .github_client import GitHubClient
from .models import ClaudeResult, Feature, FeatureStatus
from .session_manager import SessionManager

logger = logging.getLogger(__name__)


class WorkflowOrchestrator:
    """
    Orchestrates the full spec-kit workflow.

    Workflow phases:
    1. Discover features from GitHub
    2. Specify (create spec.md)
    3. Clarify (resolve ambiguities)
    4. Plan (create plan.md)
    5. Tasks (create tasks.md)
    6. Analyze (validate consistency)
    7. Implement (execute tasks)
    8. Review (verify PR matches spec)
    """

    def __init__(
        self,
        repo_root: Path,
        github: GitHubClient,
        claude: ClaudeClient,
        tracker: FeatureTracker,
        sessions: SessionManager,
        stop_on_error: bool = True,
    ):
        self.repo_root = repo_root
        self.github = github
        self.claude = claude
        self.tracker = tracker
        self.sessions = sessions
        self.stop_on_error = stop_on_error

    def _handle_result(
        self,
        result: ClaudeResult,
        phase: str,
        feature: Feature,
    ) -> bool:
        """Handle result from Claude invocation. Returns True if successful."""
        if not result.success:
            logger.error(f"[{feature.branch}] {phase} failed: {result.error_message}")
            if self.stop_on_error:
                raise WorkflowError(f"{phase} failed for {feature.branch}")
            return False

        logger.info(
            f"[{feature.branch}] {phase} completed in {result.duration_seconds:.1f}s"
        )
        if result.artifacts:
            logger.info(f"  Created: {', '.join(result.artifacts)}")

        self.sessions.update_last_used(feature.branch)
        return True

    # === Git Operations ===

    def _has_uncommitted_changes(self) -> bool:
        """Check if there are uncommitted changes."""
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            capture_output=True,
            text=True,
            cwd=self.repo_root,
        )
        return bool(result.stdout.strip())

    def _stash_changes(self) -> bool:
        """Stash uncommitted changes. Returns True if stash was created."""
        if not self._has_uncommitted_changes():
            return False

        result = subprocess.run(
            ["git", "stash", "push", "-m", "speckit-auto-stash"],
            capture_output=True,
            text=True,
            cwd=self.repo_root,
        )
        if result.returncode == 0:
            logger.info("Stashed uncommitted changes before branch checkout")
            return True
        logger.warning(f"Failed to stash changes: {result.stderr}")
        return False

    def _pop_stash(self) -> None:
        """Pop the most recent stash if it was created by speckit."""
        result = subprocess.run(
            ["git", "stash", "list", "-1"],
            capture_output=True,
            text=True,
            cwd=self.repo_root,
        )
        if "speckit-auto-stash" in result.stdout:
            subprocess.run(
                ["git", "stash", "pop"],
                cwd=self.repo_root,
            )
            logger.info("Restored stashed changes")

    def _checkout_branch(self, feature: Feature) -> None:
        """Checkout or create the feature branch."""
        # Handle dirty git state by stashing changes
        had_stash = self._stash_changes()

        try:
            # Check if branch exists
            result = subprocess.run(
                ["git", "branch", "--list", feature.branch],
                capture_output=True,
                text=True,
                cwd=self.repo_root,
            )

            if feature.branch in result.stdout:
                # Branch exists, checkout
                subprocess.run(
                    ["git", "checkout", feature.branch],
                    check=True,
                    cwd=self.repo_root,
                )
            else:
                # Create new branch from main
                subprocess.run(
                    ["git", "checkout", "-b", feature.branch, "main"],
                    check=True,
                    cwd=self.repo_root,
                )
        finally:
            # Restore stash if we created one
            if had_stash:
                self._pop_stash()

    def _commit_and_push(self, feature: Feature, message: str) -> None:
        """Commit all changes and push to remote."""
        subprocess.run(
            ["git", "add", "-A"],
            check=True,
            cwd=self.repo_root,
        )

        # Check if there are changes to commit
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            capture_output=True,
            text=True,
            cwd=self.repo_root,
        )

        if result.stdout.strip():
            subprocess.run(
                ["git", "commit", "-m", message],
                check=True,
                cwd=self.repo_root,
            )

            subprocess.run(
                ["git", "push", "-u", "origin", feature.branch],
                check=True,
                cwd=self.repo_root,
            )

    # === Individual Phase Methods ===

    def run_specify(self, feature: Feature) -> bool:
        """Run specification phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.specify")

        self._checkout_branch(feature)

        result = self.claude.specify(
            feature.description,
            session_id=feature.session_id,
        )

        if self._handle_result(result, "specify", feature):
            feature.status = FeatureStatus.SPECIFIED
            self._sync_spec_to_github(feature)
            self._commit_and_push(feature, f"docs({feature.number:03d}): add spec.md")
            return True
        return False

    def run_clarify(self, feature: Feature) -> bool:
        """Run clarification phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.clarify")

        result = self.claude.clarify(session_id=feature.session_id)

        if self._handle_result(result, "clarify", feature):
            feature.status = FeatureStatus.CLARIFIED
            self._commit_and_push(
                feature, f"docs({feature.number:03d}): update spec with clarifications"
            )
            return True
        return False

    def run_plan(self, feature: Feature) -> bool:
        """Run planning phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.plan")

        result = self.claude.plan(session_id=feature.session_id)

        if self._handle_result(result, "plan", feature):
            feature.status = FeatureStatus.PLANNED
            self._sync_plan_to_github(feature)
            self._commit_and_push(feature, f"docs({feature.number:03d}): add plan.md")
            return True
        return False

    def run_tasks(self, feature: Feature) -> bool:
        """Run task generation phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.tasks")

        result = self.claude.tasks(session_id=feature.session_id)

        if self._handle_result(result, "tasks", feature):
            feature.status = FeatureStatus.TASKED
            self._create_user_story_issues(feature)
            self._commit_and_push(feature, f"docs({feature.number:03d}): add tasks.md")
            return True
        return False

    def run_analyze(self, feature: Feature) -> bool:
        """Run analysis phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.analyze")

        result = self.claude.analyze(session_id=feature.session_id)
        return self._handle_result(result, "analyze", feature)

    def run_checklist(self, feature: Feature) -> bool:
        """Run checklist phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.checklist")

        result = self.claude.checklist(session_id=feature.session_id)

        if self._handle_result(result, "checklist", feature):
            self._commit_and_push(
                feature, f"docs({feature.number:03d}): add checklist"
            )
            return True
        return False

    def run_implement(self, feature: Feature) -> bool:
        """Run implementation phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.implement")

        result = self.claude.implement(session_id=feature.session_id)

        if self._handle_result(result, "implement", feature):
            feature.status = FeatureStatus.IMPLEMENTING
            self._commit_and_push(
                feature, f"feat({feature.number:03d}): implement {feature.name}"
            )
            self._create_pr(feature)
            return True
        return False

    def run_review(self, feature: Feature) -> bool:
        """Run review phase for a feature."""
        logger.info(f"[{feature.branch}] Running /speckit.review")

        result = self.claude.review(
            pr_number=feature.pr_number,
            session_id=feature.session_id,
        )

        if self._handle_result(result, "review", feature):
            feature.status = FeatureStatus.REVIEWING
            return True
        return False

    # === GitHub Sync Methods ===

    def _sync_spec_to_github(self, feature: Feature) -> None:
        """Sync spec.md content to GitHub issue."""
        if not feature.issue_number or not feature.spec_dir:
            return

        spec_file = feature.spec_dir / "spec.md"
        if not spec_file.exists():
            return

        spec_content = spec_file.read_text()

        # Add spec as a comment
        comment = f"## Specification\n\n{spec_content}"
        self.github.add_issue_comment(feature.issue_number, comment)
        logger.info(f"  Synced spec.md to issue #{feature.issue_number}")

    def _sync_plan_to_github(self, feature: Feature) -> None:
        """Sync plan.md content to GitHub issue comment."""
        if not feature.issue_number or not feature.spec_dir:
            return

        plan_file = feature.spec_dir / "plan.md"
        if not plan_file.exists():
            return

        plan_content = plan_file.read_text()
        comment = f"## Implementation Plan\n\n{plan_content}"
        self.github.add_issue_comment(feature.issue_number, comment)
        logger.info(f"  Synced plan.md to issue #{feature.issue_number}")

    def _create_user_story_issues(self, feature: Feature) -> None:
        """Create sub-issues for each user story from tasks.md."""
        if not feature.issue_number or not feature.spec_dir:
            return

        tasks_file = feature.spec_dir / "tasks.md"
        if not tasks_file.exists():
            return

        # Parse user stories from tasks.md
        tasks = self.tracker.parse_tasks_from_file(tasks_file)

        # Group tasks by user story
        stories: dict[str, list] = {}
        for task in tasks:
            if task.user_story:
                if task.user_story not in stories:
                    stories[task.user_story] = []
                stories[task.user_story].append(task)

        # Create sub-issue for each user story (if not already exists)
        existing_stories = {us.id for us in feature.user_stories}

        for story_id, story_tasks in stories.items():
            if story_id in existing_stories:
                continue

            # Get story description from first task
            description = f"Tasks for {story_id}"
            if story_tasks:
                # Try to find description from phase header in tasks.md
                description = story_tasks[0].description.split(" in ")[0]

            body = f"Parent: #{feature.issue_number}\n\n"
            body += "## Tasks\n\n"
            for task in story_tasks:
                status = "[x]" if task.completed else "[ ]"
                body += f"- {status} {task.id}: {task.description}\n"

            try:
                issue_num = self.github.create_issue(
                    title=f"{story_id}: {description}",
                    body=body,
                    labels=["user-story", f"feature-{feature.number:03d}"],
                )
                logger.info(f"  Created sub-issue #{issue_num} for {story_id}")
            except Exception as e:
                logger.warning(f"  Failed to create sub-issue for {story_id}: {e}")

    def _create_pr(self, feature: Feature) -> None:
        """Create a pull request for the feature."""
        if not feature.issue_number:
            return

        # Check if PR already exists
        existing_pr = self.github.get_pr_for_branch(feature.branch)
        if existing_pr:
            feature.pr_number = existing_pr.number
            logger.info(f"  PR already exists: #{existing_pr.number}")
            return

        title = f"feat({feature.number:03d}): {feature.name}"
        body = f"""## Summary
Implements Feature {feature.number}: {feature.name}

Closes #{feature.issue_number}

## Changes
See commits for details.

## Test Plan
- [ ] All tests pass
- [ ] Manual testing completed
"""

        try:
            pr_num = self.github.create_pr(
                title=title,
                body=body,
                head=feature.branch,
                base="main",
            )
            feature.pr_number = pr_num
            logger.info(f"  Created PR #{pr_num}")
        except Exception as e:
            logger.warning(f"  Failed to create PR: {e}")

    # === Full Workflow ===

    def run_full_workflow(
        self,
        feature: Feature,
        start_phase: str | None = None,
        end_phase: str | None = None,
    ) -> bool:
        """
        Run the complete spec-kit workflow for a feature.

        Args:
            feature: The feature to process
            start_phase: Phase to start from (skip earlier phases)
            end_phase: Phase to stop at (skip later phases)
        """
        phases = [
            ("specify", self.run_specify),
            ("clarify", self.run_clarify),
            ("plan", self.run_plan),
            ("tasks", self.run_tasks),
            ("analyze", self.run_analyze),
            ("checklist", self.run_checklist),
            ("implement", self.run_implement),
            ("review", self.run_review),
        ]

        started = start_phase is None

        for phase_name, phase_func in phases:
            # Skip until we reach start_phase
            if not started:
                if phase_name == start_phase:
                    started = True
                else:
                    continue

            # Skip phases the feature has already completed
            if feature.status == FeatureStatus.COMPLETE:
                continue

            # Run the phase
            logger.info(f"[{feature.branch}] Starting phase: {phase_name}")
            success = phase_func(feature)

            if not success and self.stop_on_error:
                return False

            # Stop if we've reached end_phase
            if end_phase and phase_name == end_phase:
                break

        return True

    def create_feature_issue(self, feature_num: int, description: str) -> Feature:
        """Create a new feature issue on GitHub."""
        import re

        def slugify(text: str) -> str:
            text = text.lower()
            text = re.sub(r"[^a-z0-9]+", "-", text)
            return text.strip("-")

        title = f"Feature {feature_num}: {description}"
        body = f"""Feature {feature_num} specification

See branch: {feature_num:03d}-{slugify(description)}

Status: Specification in progress
"""

        issue_num = self.github.create_issue(title, body, labels=["feature"])

        branch = f"{feature_num:03d}-{slugify(description)}"
        return Feature(
            number=feature_num,
            name=description,
            description=description,
            issue_number=issue_num,
            branch=branch,
            spec_dir=self.repo_root / "specs" / branch,
            status=FeatureStatus.ISSUE_CREATED,
            session_id=self.sessions.get_or_create_session(branch),
        )

    def merge_and_close(self, feature: Feature) -> bool:
        """Merge PR and close all related issues."""
        if not feature.pr_number:
            logger.error(f"[{feature.branch}] No PR to merge")
            return False

        try:
            # Merge the PR
            self.github.merge_pr(feature.pr_number)
            logger.info(f"[{feature.branch}] Merged PR #{feature.pr_number}")

            # Close the main issue
            if feature.issue_number:
                self.github.close_issue(feature.issue_number)
                logger.info(f"[{feature.branch}] Closed issue #{feature.issue_number}")

            # Close user story issues
            for us in feature.user_stories:
                if us.issue_number:
                    self.github.close_issue(us.issue_number)
                    logger.info(f"[{feature.branch}] Closed issue #{us.issue_number}")

            feature.status = FeatureStatus.COMPLETE
            return True

        except Exception as e:
            logger.error(f"[{feature.branch}] Failed to merge: {e}")
            return False
