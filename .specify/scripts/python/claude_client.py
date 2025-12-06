"""Handles Claude CLI invocations."""

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

from .exceptions import ClaudeError
from .models import ClaudeResult


def _find_claude_cli() -> str:
    """Find the Claude CLI executable path."""
    # 1. Check environment variable override
    if env_path := os.environ.get("CLAUDE_CLI_PATH"):
        if Path(env_path).exists():
            return env_path

    # 2. Check common install locations
    common_paths = [
        Path.home() / ".claude" / "local" / "claude",
        Path.home() / ".local" / "bin" / "claude",
        Path("/usr/local/bin/claude"),
    ]

    for path in common_paths:
        if path.exists():
            return str(path)

    # 3. Fall back to 'claude' in PATH
    return "claude"


class ClaudeClient:
    """Handles Claude CLI invocations."""

    def __init__(
        self,
        repo_root: Path,
        permission_mode: str = "acceptEdits",
        timeout: int = 600,  # 10 minutes default
        model: str = "haiku",  # Default to haiku for speed/cost
    ):
        self.repo_root = repo_root
        self.permission_mode = permission_mode
        self.timeout = timeout
        self.model = model
        self._verified = False
        self._claude_path: str | None = None

    def _get_claude_path(self) -> str:
        """Get the Claude CLI path (cached)."""
        if self._claude_path is None:
            self._claude_path = _find_claude_cli()
        return self._claude_path

    def _verify_claude_cli(self) -> None:
        """Verify claude CLI is installed (lazy verification)."""
        if self._verified:
            return

        claude_path = self._get_claude_path()
        try:
            subprocess.run(
                [claude_path, "--version"],
                capture_output=True,
                text=True,
                check=True,
            )
            self._verified = True
        except FileNotFoundError:
            raise ClaudeError(
                f"Claude CLI not found at '{claude_path}'. "
                "Set CLAUDE_CLI_PATH env var or install with: npm install -g @anthropic-ai/claude-code"
            )

    def invoke(
        self,
        prompt: str,
        session_id: str | None = None,
        continue_session: bool = False,
        output_format: str = "json",
    ) -> ClaudeResult:
        """
        Invoke Claude CLI with a prompt.

        Args:
            prompt: The prompt to send (can include /slash commands)
            session_id: Specific session ID to use/resume
            continue_session: Whether to continue most recent session
            output_format: "json", "text", or "stream-json"

        Returns:
            ClaudeResult with parsed output
        """
        # Lazy verification - only check when actually invoking
        self._verify_claude_cli()

        claude_path = self._get_claude_path()
        args = [claude_path, "-p"]
        args.extend(["--model", self.model])
        args.extend(["--output-format", output_format])
        args.extend(["--permission-mode", self.permission_mode])

        if session_id:
            args.extend(["--resume", session_id])
        elif continue_session:
            args.append("--continue")

        # Add the prompt last
        args.append(prompt)

        start_time = time.time()

        try:
            result = subprocess.run(
                args,
                capture_output=True,
                text=True,
                cwd=self.repo_root,
                timeout=self.timeout,
            )
        except subprocess.TimeoutExpired:
            return ClaudeResult(
                success=False,
                output="",
                error_message=f"Claude CLI timed out after {self.timeout}s",
            )

        duration = time.time() - start_time

        # Parse output based on format
        if output_format == "json":
            return self._parse_json_output(result, duration)
        else:
            return ClaudeResult(
                success=result.returncode == 0,
                output=result.stdout,
                duration_seconds=duration,
                error_message=result.stderr if result.returncode != 0 else None,
            )

    def _parse_json_output(
        self,
        result: subprocess.CompletedProcess,
        duration: float,
    ) -> ClaudeResult:
        """Parse JSON output from Claude CLI."""
        if result.returncode != 0:
            return ClaudeResult(
                success=False,
                output=result.stdout,
                duration_seconds=duration,
                error_message=result.stderr,
            )

        try:
            data = json.loads(result.stdout)
        except json.JSONDecodeError:
            # Sometimes output is plain text even with --output-format json
            return ClaudeResult(
                success=True,
                output=result.stdout,
                duration_seconds=duration,
            )

        # Extract artifacts from tool_use messages
        artifacts = self._extract_artifacts(data)

        # Extract session ID if present
        session_id = data.get("session_id")

        # Extract cost if present
        cost = data.get("usage", {}).get("cost_usd")

        # Get the result text
        output_text = data.get("result", "")
        if not output_text and "messages" in data:
            # Try to extract from last assistant message
            for msg in reversed(data.get("messages", [])):
                if msg.get("role") == "assistant":
                    output_text = msg.get("content", "")
                    break

        return ClaudeResult(
            success=True,
            output=output_text,
            json_output=data,
            session_id=session_id,
            artifacts=artifacts,
            cost_usd=cost,
            duration_seconds=duration,
        )

    def _extract_artifacts(self, data: dict[str, Any]) -> list[str]:
        """Extract file paths from tool_use results."""
        artifacts = []

        messages = data.get("messages", [])
        for msg in messages:
            if msg.get("role") != "assistant":
                continue

            content = msg.get("content", [])
            if isinstance(content, str):
                continue

            for block in content:
                if block.get("type") != "tool_use":
                    continue

                tool_name = block.get("name", "")
                tool_input = block.get("input", {})

                if tool_name == "Write":
                    file_path = tool_input.get("file_path")
                    if file_path:
                        artifacts.append(file_path)
                elif tool_name == "Edit":
                    file_path = tool_input.get("file_path")
                    if file_path:
                        artifacts.append(file_path)

        return list(set(artifacts))  # Deduplicate

    # === Spec-Kit Command Helpers ===

    def specify(
        self,
        feature_description: str,
        session_id: str | None = None,  # Unused, kept for API compatibility
    ) -> ClaudeResult:
        """Run /speckit.specify command. Starts a new conversation."""
        return self.invoke(
            f"/speckit.specify {feature_description}",
            # Don't pass session_id - start fresh conversation
        )

    def clarify(
        self,
        session_id: str | None = None,  # Unused
    ) -> ClaudeResult:
        """Run /speckit.clarify command with web search for fish/POSIX reference."""
        prompt = "/speckit.clarify Use web search to look up fish shell and POSIX shell semantics for reference"
        return self.invoke(
            prompt,
            continue_session=True,  # Continue from specify
        )

    def plan(self, session_id: str | None = None) -> ClaudeResult:
        """Run /speckit.plan command."""
        return self.invoke(
            "/speckit.plan",
            continue_session=True,
        )

    def tasks(self, session_id: str | None = None) -> ClaudeResult:
        """Run /speckit.tasks command."""
        return self.invoke(
            "/speckit.tasks",
            continue_session=True,
        )

    def analyze(self, session_id: str | None = None) -> ClaudeResult:
        """Run /speckit.analyze command."""
        return self.invoke(
            "/speckit.analyze",
            continue_session=True,
        )

    def checklist(self, session_id: str | None = None) -> ClaudeResult:
        """Run /speckit.checklist command."""
        return self.invoke(
            "/speckit.checklist",
            continue_session=True,
        )

    def implement(self, session_id: str | None = None) -> ClaudeResult:
        """Run /speckit.implement command."""
        return self.invoke(
            "/speckit.implement",
            continue_session=True,
        )

    def review(
        self,
        pr_number: int | None = None,
        session_id: str | None = None,  # Unused
    ) -> ClaudeResult:
        """Run /speckit.review command."""
        prompt = f"/speckit.review {pr_number}" if pr_number else "/speckit.review"
        return self.invoke(
            prompt,
            continue_session=True,
        )
