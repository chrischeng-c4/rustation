//! Explorer view - File browser with Git status
//!
//! Based on desktop/src/renderer/src/features/explorer/ExplorerPage.tsx

use gpui::*;
use rstn_ui::{EmptyState, MaterialTheme, PageHeader, Themed};

/// Git status indicator
#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Unmodified,
}

impl GitStatus {
    /// Get status indicator text
    pub fn indicator(&self) -> &'static str {
        match self {
            GitStatus::Untracked => "??",
            GitStatus::Modified => "M",
            GitStatus::Added => "A",
            GitStatus::Deleted => "D",
            GitStatus::Renamed => "R",
            GitStatus::Unmodified => "",
        }
    }

    /// Get status color
    pub fn color(&self) -> Rgba {
        match self {
            GitStatus::Untracked => rgb(0x9E9E9E),  // Grey
            GitStatus::Modified => rgb(0xFFC107),   // Amber
            GitStatus::Added => rgb(0x4CAF50),      // Green
            GitStatus::Deleted => rgb(0xF44336),    // Red
            GitStatus::Renamed => rgb(0x2196F3),    // Blue
            GitStatus::Unmodified => rgb(0x666666), // Dark grey
        }
    }
}

/// File entry in the explorer
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub git_status: GitStatus,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

impl FileEntry {
    /// Render file entry row in table
    pub fn render_row(&self, theme: &MaterialTheme, is_selected: bool) -> Div {
        let bg = if is_selected {
            theme.secondary.container
        } else {
            theme.background.paper
        };

        div()
            .flex()
            .items_center()
            .px(theme.spacing(2.0))
            .py(theme.spacing(1.0))
            .bg(bg)
            .hover(|style| style.bg(theme.surface.container))
            .cursor_pointer()
            .border_b_1()
            .border_color(theme.border.divider)
            .child(
                // Git status indicator
                div()
                    .w(px(32.0))
                    .text_xs()
                    .text_color(self.git_status.color())
                    .font_weight(FontWeight::BOLD)
                    .child(self.git_status.indicator()),
            )
            .child(
                // File icon
                div()
                    .w(px(32.0))
                    .text_lg()
                    .child(if self.is_dir { "📁" } else { "📄" }),
            )
            .child(
                // File name
                div()
                    .flex_1()
                    .text_sm()
                    .child(self.name.clone()),
            )
            .children(
                // File size
                self.size.map(|size| {
                    div()
                        .w(px(100.0))
                        .text_xs()
                        .text_color(theme.text.secondary)
                        .child(format_size(size))
                })
            )
            .children(
                // Modified time
                self.modified.as_ref().map(|time| {
                    div()
                        .w(px(100.0))
                        .text_xs()
                        .text_color(theme.text.secondary)
                        .child(time.clone())
                })
            )
    }
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// File tree node for tree view
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<TreeNode>,
    pub git_status: GitStatus,
}

impl TreeNode {
    /// Render tree node
    pub fn render(&self, theme: &MaterialTheme, depth: usize, is_selected: bool) -> Div {
        let indent = px((depth as f32) * 20.0);
        let bg = if is_selected {
            theme.secondary.container
        } else {
            rgb(0x00000000) // Transparent
        };

        div()
            .flex()
            .items_center()
            .pl(indent)
            .pr(theme.spacing(1.0))
            .py(theme.spacing(0.5))
            .bg(bg)
            .hover(|style| style.bg(theme.surface.container))
            .cursor_pointer()
            .child(
                // Expand/collapse arrow
                div()
                    .w(px(20.0))
                    .text_xs()
                    .child(if self.is_dir {
                        if self.is_expanded {
                            "▼"
                        } else {
                            "▶"
                        }
                    } else {
                        ""
                    }),
            )
            .child(
                // Git status
                div()
                    .w(px(24.0))
                    .text_xs()
                    .text_color(self.git_status.color())
                    .font_weight(FontWeight::BOLD)
                    .child(self.git_status.indicator()),
            )
            .child(
                // File/folder icon
                div()
                    .w(px(24.0))
                    .text_sm()
                    .child(if self.is_dir { "📁" } else { "📄" }),
            )
            .child(
                // Name
                div()
                    .flex_1()
                    .text_sm()
                    .child(self.name.clone()),
            )
    }
}

/// File tree view component
pub struct FileTreeView {
    root: TreeNode,
    theme: MaterialTheme,
}

impl FileTreeView {
    pub fn new(root: TreeNode, theme: MaterialTheme) -> Self {
        Self { root, theme }
    }

    /// Render tree recursively
    fn render_tree(&self, node: &TreeNode, depth: usize, selected_path: Option<&str>) -> Vec<Div> {
        let mut result = vec![node.render(
            &self.theme,
            depth,
            selected_path == Some(&node.path),
        )];

        if node.is_expanded && node.is_dir {
            for child in &node.children {
                result.extend(self.render_tree(child, depth + 1, selected_path));
            }
        }

        result
    }

    pub fn render(&self, selected_path: Option<&str>) -> Div {
        div()
            .flex()
            .flex_col()
            .h_full()
            .overflow_hidden()
            .bg(self.theme.background.paper)
            .border_r_1()
            .border_color(self.theme.border.divider)
            .children(self.render_tree(&self.root, 0, selected_path))
    }
}

/// File table view component
pub struct FileTableView {
    files: Vec<FileEntry>,
    selected_index: Option<usize>,
    theme: MaterialTheme,
}

impl FileTableView {
    pub fn new(files: Vec<FileEntry>, selected_index: Option<usize>, theme: MaterialTheme) -> Self {
        Self {
            files,
            selected_index,
            theme,
        }
    }

    pub fn render(&self) -> Div {
        div()
            .flex()
            .flex_col()
            .h_full()
            .overflow_hidden()
            .bg(self.theme.background.default)
            .child(
                // Table header
                div()
                    .flex()
                    .items_center()
                    .px(self.theme.spacing(2.0))
                    .py(self.theme.spacing(1.0))
                    .bg(self.theme.background.paper)
                    .border_b_1()
                    .border_color(self.theme.border.divider)
                    .child(
                        div()
                            .w(px(32.0))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(self.theme.text.secondary)
                            .child("Git"),
                    )
                    .child(
                        div()
                            .w(px(32.0))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(self.theme.text.secondary)
                            .child(""),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(self.theme.text.secondary)
                            .child("Name"),
                    )
                    .child(
                        div()
                            .w(px(100.0))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(self.theme.text.secondary)
                            .child("Size"),
                    )
                    .child(
                        div()
                            .w(px(150.0))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(self.theme.text.secondary)
                            .child("Modified"),
                    ),
            )
            .children(
                self.files
                    .iter()
                    .enumerate()
                    .map(|(i, file)| file.render_row(&self.theme, self.selected_index == Some(i))),
            )
    }
}

/// File preview panel
pub struct DetailPanel {
    file: Option<FileEntry>,
    content_preview: Option<String>,
    theme: MaterialTheme,
}

impl DetailPanel {
    pub fn new(file: Option<FileEntry>, content_preview: Option<String>, theme: MaterialTheme) -> Self {
        Self {
            file,
            content_preview,
            theme,
        }
    }

    pub fn render(&self) -> Div {
        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(self.theme.background.paper)
            .border_l_1()
            .border_color(self.theme.border.divider)
            .children(if let Some(file) = &self.file {
                vec![
                    // File info header
                    div()
                        .flex()
                        .flex_col()
                        .p(self.theme.spacing(2.0))
                        .border_b_1()
                        .border_color(self.theme.border.divider)
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .mb(self.theme.spacing(1.0))
                                .child(file.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme.text.secondary)
                                .child(file.path.clone()),
                        ),
                    // Content preview
                    div()
                        .flex_1()
                        .p(self.theme.spacing(2.0))
                        .overflow_hidden()
                        .font_family("monospace")
                        .text_xs()
                        .children(if let Some(content) = &self.content_preview {
                            vec![div().child(content.clone())]
                        } else {
                            vec![div()
                                .text_color(self.theme.text.disabled)
                                .child("Preview not available")]
                        }),
                ]
            } else {
                vec![EmptyState::new(
                    "📄",
                    "No File Selected",
                    "Select a file to view details",
                    self.theme.clone(),
                )
                .render(None::<Div>)]
            })
    }
}

/// Explorer page view
///
/// Layout based on OLD_UI_ANALYSIS.md:
/// ```
/// ┌───────────────────────────────────────────┐
/// │ PathBreadcrumbs                          │
/// ├───────────┬──────────────┬────────────────┤
/// │ Tree View │ File Table   │ Detail Panel   │
/// │ (25%)     │ (50%)        │ (25%)          │
/// │           │              │                │
/// │ Folders   │ Files List   │ Preview        │
/// │ ...       │ - file1.txt  │ ...            │
/// │           │ - file2.rs   │                │
/// └───────────┴──────────────┴────────────────┘
/// ```
pub struct ExplorerView {
    current_path: String,
    tree_root: TreeNode,
    files: Vec<FileEntry>,
    selected_file_index: Option<usize>,
    theme: MaterialTheme,
}

impl ExplorerView {
    pub fn new(
        current_path: String,
        tree_root: TreeNode,
        files: Vec<FileEntry>,
        theme: MaterialTheme,
    ) -> Self {
        Self {
            current_path,
            tree_root,
            files,
            selected_file_index: None,
            theme,
        }
    }

    pub fn render(&self, _window: &mut Window, _cx: &mut App) -> Div {
        let page_header = PageHeader::new(
            "Explorer",
            Some("Browse files with Git status"),
            self.theme.clone(),
        );

        // Get selected file for preview
        let selected_file = self
            .selected_file_index
            .and_then(|idx| self.files.get(idx).cloned());

        let tree_view = FileTreeView::new(self.tree_root.clone(), self.theme.clone());
        let table_view = FileTableView::new(
            self.files.clone(),
            self.selected_file_index,
            self.theme.clone(),
        );
        let detail_panel = DetailPanel::new(selected_file, None, self.theme.clone());

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                page_header.render(Some(
                    div()
                        .secondary_button(&self.theme)
                        .child("Refresh"),
                )),
            )
            .child(
                // Breadcrumbs
                div()
                    .flex()
                    .items_center()
                    .px(self.theme.spacing(2.0))
                    .py(self.theme.spacing(1.0))
                    .bg(self.theme.background.paper)
                    .border_b_1()
                    .border_color(self.theme.border.divider)
                    .text_sm()
                    .text_color(self.theme.text.secondary)
                    .child(self.current_path.clone()),
            )
            .child(
                // Main content: Tree + Table + Detail
                div()
                    .flex()
                    .flex_1()
                    .child(
                        // Tree view (25%)
                        div()
                            .flex()
                            .flex_col()
                            .w_1_4()
                            .child(tree_view.render(None)),
                    )
                    .child(
                        // File table (50%)
                        div()
                            .flex()
                            .flex_col()
                            .w_1_2()
                            .child(table_view.render()),
                    )
                    .child(
                        // Detail panel (25%)
                        div()
                            .flex()
                            .flex_col()
                            .w_1_4()
                            .child(detail_panel.render()),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_indicator() {
        assert_eq!(GitStatus::Modified.indicator(), "M");
        assert_eq!(GitStatus::Added.indicator(), "A");
        assert_eq!(GitStatus::Deleted.indicator(), "D");
        assert_eq!(GitStatus::Untracked.indicator(), "??");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry {
            path: "/test/file.rs".to_string(),
            name: "file.rs".to_string(),
            is_dir: false,
            git_status: GitStatus::Modified,
            size: Some(1024),
            modified: Some("2024-01-11".to_string()),
        };

        assert_eq!(entry.name, "file.rs");
        assert!(!entry.is_dir);
        assert_eq!(entry.git_status, GitStatus::Modified);
    }

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_dir: true,
            is_expanded: false,
            children: vec![],
            git_status: GitStatus::Unmodified,
        };

        assert!(node.is_dir);
        assert!(!node.is_expanded);
        assert_eq!(node.children.len(), 0);
    }
}
