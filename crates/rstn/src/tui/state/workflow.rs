use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Rich content node representing a discrete event in the workflow timeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowNode {
    /// Input from the user
    UserPrompt(String),
    
    /// Response from the assistant (text content)
    AssistantResponse(String),
    
    /// File modification (Unified Diff)
    Diff {
        file: String,
        diff: String,
    },
    
    /// Tool usage summary
    ToolUse {
        tool: String,
        input: String,
    },
    
    /// Error message
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolStatus {
    Running,
    Completed,
    Failed,
}

/// Generic container for any workflow state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowState<T> {
    /// The unique ID of the agent session (e.g., Claude session ID)
    pub agent_session_id: Option<String>,
    
    /// The timeline of events
    pub history: Vec<WorkflowNode>,
    
    /// The current status of the FSM
    pub status: T,
}

impl<T: Default> Default for WorkflowState<T> {
    fn default() -> Self {
        Self {
            agent_session_id: None,
            history: Vec::new(),
            status: T::default(),
        }
    }
}
