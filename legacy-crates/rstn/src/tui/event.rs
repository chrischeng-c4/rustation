//! Event handling for the TUI application

use crate::tui::claude_stream::ClaudeStreamMessage;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Worktree type classification
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorktreeType {
    /// Not in a git repository
    NotGit,
    /// Main repository (not a worktree)
    MainRepository,
    /// Feature worktree with parsed number and name
    FeatureWorktree { number: String, name: String },
}

/// Terminal events
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal tick (for animations/updates)
    Tick,
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Paste event (clipboard content)
    Paste(String),
    /// Command output line
    CommandOutput(String),
    /// Command completed with all output
    CommandDone {
        success: bool,
        lines: Vec<String>,
    },
    /// Spec phase completed, awaiting user review
    SpecPhaseCompleted {
        phase: String,
        success: bool,
        output: Vec<String>,
    },
    /// Git information updated
    GitInfoUpdated {
        branch: Option<String>,
        worktree_path: Option<PathBuf>,
        worktree_count: usize,
        worktree_type: WorktreeType,
        is_git_repo: bool,
        error: Option<String>,
    },
    /// Claude streaming JSON message received (real-time output)
    ClaudeStream(ClaudeStreamMessage),
    /// Claude command completed
    ClaudeCompleted {
        phase: String,
        success: bool,
        session_id: Option<String>,
    },
    /// MCP tool called: rstn_report_status
    McpStatus {
        status: String,
        prompt: Option<String>,
        message: Option<String>,
    },
    /// MCP tool called: rstn_complete_task
    McpTaskCompleted {
        task_id: String,
        success: bool,
        message: String,
    },
    /// Commit workflow started
    CommitStarted,
    /// Security scan blocked commit
    CommitBlocked {
        scan: crate::SecurityScanResult,
    },
    /// Ready to commit with generated message
    CommitReady {
        message: String,
        warnings: Vec<crate::SecurityWarning>,
        sensitive_files: Vec<crate::SensitiveFile>,
    },
    /// Commit groups ready for user review
    CommitGroupsReady {
        groups: Vec<crate::CommitGroup>,
        warnings: Vec<crate::SecurityWarning>,
        sensitive_files: Vec<crate::SensitiveFile>,
    },
    /// Single commit group completed successfully (Feature 050)
    CommitGroupCompleted,
    /// Single commit group failed (Feature 050)
    CommitGroupFailed {
        error: String,
    },
    /// Intelligent commit failed before entering review mode (Feature 050)
    IntelligentCommitFailed {
        error: String,
    },
    /// Commit execution completed
    CommitCompleted {
        success: bool,
        output: String,
    },
    /// Commit workflow error
    CommitError {
        error: String,
    },
    /// Specify workflow events (Feature 051)
    SpecifyGenerationStarted,
    SpecifyGenerationCompleted {
        spec: String,
        number: String,
        name: String,
    },
    SpecifyGenerationFailed {
        error: String,
    },
    SpecifySaved {
        path: String,
    },
    /// Task execution completed (Feature 056)
    TaskExecutionCompleted {
        task_id: String,
        success: bool,
        output: String,
    },
}

/// Event handler that runs in a separate thread
pub struct EventHandler {
    /// Event sender
    sender: mpsc::Sender<Event>,
    /// Event receiver
    receiver: mpsc::Receiver<Event>,
    /// Handler thread
    _handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler_sender = sender.clone();

        let handler = thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(tick_rate);

                if event::poll(timeout).expect("failed to poll events") {
                    match event::read().expect("failed to read event") {
                        CrosstermEvent::Key(e) => {
                            if handler_sender.send(Event::Key(e)).is_err() {
                                break;
                            }
                        }
                        CrosstermEvent::Mouse(e) => {
                            if handler_sender.send(Event::Mouse(e)).is_err() {
                                break;
                            }
                        }
                        CrosstermEvent::Resize(w, h) => {
                            if handler_sender.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        CrosstermEvent::Paste(text) => {
                            if handler_sender.send(Event::Paste(text)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if handler_sender.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self {
            sender,
            receiver,
            _handler: handler,
        }
    }

    /// Get a clone of the sender for sending custom events
    pub fn sender(&self) -> mpsc::Sender<Event> {
        self.sender.clone()
    }

    /// Receive the next event
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}
