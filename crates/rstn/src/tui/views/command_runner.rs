//! Command runner view with interactive output

use crate::tui::views::{View, ViewAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// Available commands in the menu
#[derive(Debug, Clone)]
pub struct CommandMenuItem {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [&'static str],
}

const COMMAND_MENU: &[CommandMenuItem] = &[
    CommandMenuItem {
        name: "test",
        description: "Run all tests",
        args: &[],
    },
    CommandMenuItem {
        name: "test",
        description: "Run library tests only",
        args: &["--lib"],
    },
    CommandMenuItem {
        name: "test",
        description: "Run integration tests only",
        args: &["--integration"],
    },
    CommandMenuItem {
        name: "build",
        description: "Build debug",
        args: &[],
    },
    CommandMenuItem {
        name: "build",
        description: "Build release",
        args: &["--release"],
    },
    CommandMenuItem {
        name: "check",
        description: "Fast compilation check",
        args: &[],
    },
    CommandMenuItem {
        name: "lint",
        description: "Run clippy lints",
        args: &[],
    },
    CommandMenuItem {
        name: "lint",
        description: "Run clippy with auto-fix",
        args: &["--fix"],
    },
    CommandMenuItem {
        name: "fmt",
        description: "Format code",
        args: &[],
    },
    CommandMenuItem {
        name: "fmt",
        description: "Check formatting",
        args: &["--check"],
    },
    CommandMenuItem {
        name: "ci",
        description: "Run all CI checks",
        args: &[],
    },
    CommandMenuItem {
        name: "doctor",
        description: "Run diagnostics",
        args: &[],
    },
    CommandMenuItem {
        name: "spec",
        description: "Show spec status",
        args: &["status"],
    },
    CommandMenuItem {
        name: "spec",
        description: "List all features",
        args: &["list"],
    },
];

/// Focus area in the command runner
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandFocus {
    Menu,
    Output,
}

/// Command runner view state
pub struct CommandRunner {
    /// Currently selected menu item
    pub menu_state: ListState,
    /// Command output lines
    pub output_lines: Vec<OutputLine>,
    /// Is a command currently running?
    pub running: bool,
    /// Current command name (if running)
    pub current_command: Option<String>,
    /// Output scroll position
    pub output_scroll: usize,
    /// Current focus area
    pub focus: CommandFocus,
    /// Spinner animation frame
    spinner_frame: usize,
}

/// Output line with optional styling
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub text: String,
    pub line_type: OutputLineType,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputLineType {
    Normal,
    Success,
    Error,
    Warning,
    Info,
    Command, // For displaying executed commands
}

impl CommandRunner {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            menu_state: state,
            output_lines: vec![OutputLine {
                text: "Select a command from the menu and press Enter to run.".to_string(),
                line_type: OutputLineType::Info,
            }],
            running: false,
            current_command: None,
            output_scroll: 0,
            focus: CommandFocus::Menu,
            spinner_frame: 0,
        }
    }

    /// Check if a command is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Start a command (called from app after receiving RunCommand action)
    pub fn start_command(&mut self, name: &str, args: &[String]) {
        self.running = true;
        let args_str = if args.is_empty() {
            String::new()
        } else {
            format!(" {}", args.join(" "))
        };
        self.current_command = Some(format!("{}{}", name, args_str));
        self.output_lines.clear();
        self.output_lines.push(OutputLine {
            text: format!("Running: rscli {}{}", name, args_str),
            line_type: OutputLineType::Info,
        });
        self.output_lines.push(OutputLine {
            text: "─".repeat(60),
            line_type: OutputLineType::Normal,
        });
        self.output_scroll = 0;
    }

    /// Add output line
    pub fn add_output(&mut self, line: String) {
        let line_type = if line.contains("error") || line.contains("FAILED") {
            OutputLineType::Error
        } else if line.contains("warning") {
            OutputLineType::Warning
        } else if line.contains("ok") || line.contains("PASSED") || line.contains("Finished") {
            OutputLineType::Success
        } else {
            OutputLineType::Normal
        };
        self.output_lines.push(OutputLine {
            text: line,
            line_type,
        });

        // Auto-scroll to bottom
        if self.output_lines.len() > 20 {
            self.output_scroll = self.output_lines.len().saturating_sub(20);
        }
    }

    /// Mark command as finished
    pub fn command_finished(&mut self, success: bool) {
        self.running = false;
        self.output_lines.push(OutputLine {
            text: "─".repeat(60),
            line_type: OutputLineType::Normal,
        });
        self.output_lines.push(OutputLine {
            text: if success {
                "✓ Command completed successfully".to_string()
            } else {
                "✗ Command failed".to_string()
            },
            line_type: if success {
                OutputLineType::Success
            } else {
                OutputLineType::Error
            },
        });
    }

    /// Get only the focused pane content as text
    pub fn get_focused_pane_text(&self) -> String {
        match self.focus {
            CommandFocus::Menu => {
                // Return currently selected command
                if let Some(i) = self.menu_state.selected() {
                    if let Some(cmd) = COMMAND_MENU.get(i) {
                        return format!("{} - {}", cmd.name, cmd.description);
                    }
                }
                String::new()
            }
            CommandFocus::Output => {
                // Return output lines
                self.output_lines
                    .iter()
                    .map(|line| line.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    /// Get all output as a single string for copying
    pub fn get_output_text(&self) -> String {
        self.output_lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get styled output with ANSI color codes
    pub fn get_styled_output(&self) -> String {
        self.output_lines
            .iter()
            .map(|line| {
                let color_code = match line.line_type {
                    OutputLineType::Success => "\x1b[32m",   // Green
                    OutputLineType::Error => "\x1b[31m",     // Red
                    OutputLineType::Warning => "\x1b[33m",   // Yellow
                    OutputLineType::Info => "\x1b[36m",      // Cyan
                    OutputLineType::Command => "\x1b[35;1m", // Magenta bold
                    OutputLineType::Normal => "",
                };
                let reset = if color_code.is_empty() { "" } else { "\x1b[0m" };
                format!("{}{}{}", color_code, line.text, reset)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Switch to next pane (Menu <-> Output)
    pub fn next_pane(&mut self) {
        self.focus = match self.focus {
            CommandFocus::Menu => CommandFocus::Output,
            CommandFocus::Output => CommandFocus::Menu,
        };
    }

    fn handle_menu_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.menu_state.selected().unwrap_or(0);
                let new_i = if i == 0 {
                    COMMAND_MENU.len() - 1
                } else {
                    i - 1
                };
                self.menu_state.select(Some(new_i));
                ViewAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.menu_state.selected().unwrap_or(0);
                let new_i = (i + 1) % COMMAND_MENU.len();
                self.menu_state.select(Some(new_i));
                ViewAction::None
            }
            KeyCode::Enter => {
                if !self.running {
                    if let Some(i) = self.menu_state.selected() {
                        let cmd = &COMMAND_MENU[i];
                        self.focus = CommandFocus::Output;
                        return ViewAction::RunCommand {
                            name: cmd.name.to_string(),
                            args: cmd.args.iter().map(|s| s.to_string()).collect(),
                        };
                    }
                }
                ViewAction::None
            }
            KeyCode::Char('c') if !self.running => {
                // Clear output
                self.output_lines.clear();
                self.output_lines.push(OutputLine {
                    text: "Output cleared.".to_string(),
                    line_type: OutputLineType::Info,
                });
                ViewAction::None
            }
            _ => ViewAction::None,
        }
    }

    fn handle_output_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.output_scroll = self.output_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.output_scroll < self.output_lines.len().saturating_sub(1) {
                    self.output_scroll += 1;
                }
            }
            KeyCode::PageUp => {
                self.output_scroll = self.output_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.output_scroll =
                    (self.output_scroll + 10).min(self.output_lines.len().saturating_sub(1));
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.output_scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.output_scroll = self.output_lines.len().saturating_sub(1);
            }
            _ => {}
        }
        ViewAction::None
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == CommandFocus::Menu;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Commands ")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let items: Vec<ListItem> = COMMAND_MENU
            .iter()
            .map(|cmd| {
                let args_str = if cmd.args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", cmd.args.join(" "))
                };
                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        format!("{}{}", cmd.name, args_str),
                        Style::default().fg(Color::Cyan),
                    )]),
                    Line::from(vec![Span::styled(
                        format!("  {}", cmd.description),
                        Style::default().fg(Color::DarkGray),
                    )]),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.menu_state.clone());
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == CommandFocus::Output;
        let title = if self.running {
            let spinner = ['⠋', '⠙', '⠹', '⠸'][self.spinner_frame];
            format!(" Output {} Running... ", spinner)
        } else {
            " Output ".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let lines: Vec<Line> = self
            .output_lines
            .iter()
            .skip(self.output_scroll)
            .map(|line| {
                let style = match line.line_type {
                    OutputLineType::Normal => Style::default(),
                    OutputLineType::Success => Style::default().fg(Color::Green),
                    OutputLineType::Error => Style::default().fg(Color::Red),
                    OutputLineType::Warning => Style::default().fg(Color::Yellow),
                    OutputLineType::Info => Style::default().fg(Color::Cyan),
                    OutputLineType::Command => Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                };
                Line::from(Span::styled(&line.text, style))
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl View for CommandRunner {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Split into menu (left) and output (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        self.render_menu(frame, chunks[0]);
        self.render_output(frame, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> ViewAction {
        match key.code {
            // Switch focus with h/l
            KeyCode::Char('h') | KeyCode::Left if self.focus == CommandFocus::Output => {
                self.focus = CommandFocus::Menu;
                return ViewAction::None;
            }
            KeyCode::Char('l') | KeyCode::Right if self.focus == CommandFocus::Menu => {
                self.focus = CommandFocus::Output;
                return ViewAction::None;
            }
            _ => {}
        }

        match self.focus {
            CommandFocus::Menu => self.handle_menu_key(key),
            CommandFocus::Output => self.handle_output_key(key),
        }
    }

    fn tick(&mut self) {
        if self.running {
            self.spinner_frame = (self.spinner_frame + 1) % 4;
        }
    }
}
