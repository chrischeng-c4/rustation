use serde::{Deserialize, Serialize};

/// Finite State Machine for Prompt Claude workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptClaudeStatus {
    /// Waiting for user trigger
    Idle,
    
    /// User is typing the initial prompt (Large Editor)
    Inputting {
        input: String,
        cursor: usize,
    },
    
    /// Workflow is running (Streaming / Tool Calling)
    Executing,
    
    /// Blocked on user input (MCP needs_input)
    InteractionRequired {
        prompt: String,
        input: String,
        cursor: usize,
    },
    
    /// AI finished, waiting for user's next move (Footer Chat)
    AwaitingFollowUp {
        input: String,
        cursor: usize,
    },
}

impl Default for PromptClaudeStatus {
    fn default() -> Self {
        Self::Idle
    }
}
