"""State definition for Prompt Claude workflow.

Tracks the state of a conversation with Claude Code, including
accumulated output, session ID, and cost tracking.
"""

from __future__ import annotations

from pydantic import BaseModel, Field


class PromptClaudeData(BaseModel):
    """Data for Prompt Claude workflow."""

    model_config = {"frozen": False}

    prompt: str = Field(description="The user's initial prompt")
    
    # Streaming Output
    output: str = Field(default="", description="Accumulated output text from Claude")
    
    # Session Context
    claude_session_id: str | None = Field(
        default=None, 
        description="The UUID of the Claude Code session"
    )
    
    # Configuration
    mcp_config_path: str | None = Field(
        default=None,
        description="Path to the session-specific MCP config file"
    )
    
    # Metrics
    cost_usd: float = Field(
        default=0.0,
        description="Total cost in USD for this interaction"
    )
    
    def append_output(self, delta: str) -> PromptClaudeData:
        """Append text delta to output."""
        return self.model_copy(update={"output": self.output + delta})

    def with_session_id(self, session_id: str) -> PromptClaudeData:
        """Update session ID."""
        return self.model_copy(update={"claude_session_id": session_id})
    
    def with_cost(self, cost: float) -> PromptClaudeData:
        """Update cost."""
        return self.model_copy(update={"cost_usd": cost})
