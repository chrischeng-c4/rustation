//! Integrated PTY Terminal for worktree-scoped terminal sessions.
//!
//! Uses portable-pty to spawn shell sessions and stream I/O.

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

// ============================================================================
// Terminal State (serializable part)
// ============================================================================

/// Terminal state stored in WorktreeState (serializable).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TerminalState {
    /// Active session ID (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Terminal dimensions.
    #[serde(default)]
    pub cols: u16,
    #[serde(default)]
    pub rows: u16,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            session_id: None,
            cols: 80,
            rows: 24,
        }
    }
}

// ============================================================================
// Terminal Session (non-serializable PTY handle)
// ============================================================================

/// Active terminal session with PTY handle.
pub struct TerminalSession {
    /// Unique session ID.
    pub id: String,
    /// Worktree ID this session belongs to.
    pub worktree_id: String,
    /// Working directory.
    pub cwd: String,
    /// PTY pair (master + child).
    pty_pair: PtyPair,
    /// Writer to send input to PTY.
    writer: Box<dyn Write + Send>,
    /// Channel to stop the reader task.
    stop_tx: Option<mpsc::Sender<()>>,
}

impl TerminalSession {
    /// Resize the terminal.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.pty_pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize PTY: {}", e))
    }

    /// Write data to the terminal (user input).
    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(data)
            .map_err(|e| format!("Failed to write to PTY: {}", e))?;
        self.writer
            .flush()
            .map_err(|e| format!("Failed to flush PTY: {}", e))
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        // Signal the reader task to stop
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.try_send(());
        }
        // The PTY will be closed when pty_pair is dropped
    }
}

// ============================================================================
// Terminal Manager
// ============================================================================

/// Callback type for terminal output.
pub type OutputCallback = Arc<dyn Fn(String, Vec<u8>) + Send + Sync>;

/// Manager for all terminal sessions.
pub struct TerminalManager {
    /// Active sessions by session ID.
    sessions: RwLock<HashMap<String, TerminalSession>>,
    /// Output callback (session_id, data).
    output_callback: RwLock<Option<OutputCallback>>,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            output_callback: RwLock::new(None),
        }
    }

    /// Set the output callback for streaming PTY output.
    pub async fn set_output_callback(&self, callback: OutputCallback) {
        let mut cb = self.output_callback.write().await;
        *cb = Some(callback);
    }

    /// Spawn a new terminal session.
    pub async fn spawn(
        &self,
        worktree_id: String,
        cwd: String,
        cols: u16,
        rows: u16,
    ) -> Result<String, String> {
        let session_id = uuid::Uuid::new_v4().to_string();

        // Create PTY
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Determine shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "powershell.exe".to_string()
            } else {
                "/bin/zsh".to_string()
            }
        });

        // Build command
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&cwd);

        // Set up environment
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Spawn child process
        let _child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Get writer for input
        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

        // Get reader for output
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

        // Create stop channel
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        // Spawn reader task
        let session_id_clone = session_id.clone();
        // Clone the callback Arc if set
        let output_callback = {
            let cb = self.output_callback.blocking_read();
            cb.clone()
        };

        tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                // Check for stop signal (non-blocking)
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        let sid = session_id_clone.clone();

                        // Call output callback if set
                        if let Some(ref callback) = output_callback {
                            callback(sid, data);
                        }
                    }
                    Err(e) => {
                        // Check if it's just a would-block or interrupted
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::Interrupted
                        {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        // Real error, stop reading
                        tracing::warn!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });

        // Store session
        let session = TerminalSession {
            id: session_id.clone(),
            worktree_id,
            cwd,
            pty_pair,
            writer,
            stop_tx: Some(stop_tx),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Resize a terminal session.
    pub async fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
        session.resize(cols, rows)
    }

    /// Write data to a terminal session.
    pub async fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
        session.write(data)
    }

    /// Kill a terminal session.
    pub async fn kill(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        sessions
            .remove(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
        Ok(())
    }

    /// Kill all sessions for a worktree.
    pub async fn kill_worktree_sessions(&self, worktree_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, s| s.worktree_id != worktree_id);
    }

    /// Kill all sessions.
    pub async fn kill_all(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
    }

    /// Check if a session exists.
    pub async fn has_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(session_id)
    }

    /// Get session info for a worktree.
    pub async fn get_worktree_session(&self, worktree_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .find(|s| s.worktree_id == worktree_id)
            .map(|s| s.id.clone())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_default() {
        let state = TerminalState::new();
        assert!(state.session_id.is_none());
        assert_eq!(state.cols, 80);
        assert_eq!(state.rows, 24);
    }

    #[test]
    fn test_terminal_state_serialization() {
        let state = TerminalState {
            session_id: Some("test-123".to_string()),
            cols: 120,
            rows: 40,
        };

        let json = serde_json::to_string(&state).unwrap();
        let loaded: TerminalState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, loaded);
    }

    #[tokio::test]
    async fn test_terminal_manager_new() {
        let manager = TerminalManager::new();
        assert!(!manager.has_session("nonexistent").await);
    }

    // Note: Full PTY tests require a real terminal environment
    // and are better suited for integration tests
}
