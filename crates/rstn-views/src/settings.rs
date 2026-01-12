//! Settings view - Configuration interface
//!
//! Provides UI for managing rustation settings:
//! - General settings (theme, language)
//! - Project settings (default directory, git config)
//! - MCP server configuration
//! - Claude Code integration settings

use gpui::*;
use rstn_ui::MaterialTheme;

/// Configuration category
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsCategory {
    General,
    Project,
    MCP,
    ClaudeCode,
}

impl SettingsCategory {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Project => "Project",
            SettingsCategory::MCP => "MCP Server",
            SettingsCategory::ClaudeCode => "Claude Code",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingsCategory::General => "⚙️",
            SettingsCategory::Project => "📁",
            SettingsCategory::MCP => "🔌",
            SettingsCategory::ClaudeCode => "🤖",
        }
    }
}

/// Setting item component
pub struct SettingItem {
    pub label: String,
    pub description: String,
    pub value: String,
    pub theme: MaterialTheme,
}

impl SettingItem {
    pub fn new(
        label: impl Into<String>,
        description: impl Into<String>,
        value: impl Into<String>,
        theme: MaterialTheme,
    ) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
            value: value.into(),
            theme,
        }
    }

    pub fn render(&self, _window: &mut Window, _cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .p(self.theme.spacing(2.0))
            .bg(self.theme.background.paper)
            .rounded(self.theme.shape.border_radius)
            .mb(self.theme.spacing(1.5))
            .child(
                // Label
                div()
                    .text_base()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.theme.text.primary)
                    .mb(self.theme.spacing(0.5))
                    .child(self.label.clone()),
            )
            .child(
                // Description
                div()
                    .text_sm()
                    .text_color(self.theme.text.secondary)
                    .mb(self.theme.spacing(1.0))
                    .child(self.description.clone()),
            )
            .child(
                // Value input placeholder
                div()
                    .px(self.theme.spacing(1.5))
                    .py(self.theme.spacing(1.0))
                    .bg(self.theme.background.default)
                    .border_1()
                    .border_color(self.theme.border.divider)
                    .rounded(self.theme.shape.border_radius_sm)
                    .text_sm()
                    .text_color(self.theme.text.primary)
                    .child(self.value.clone()),
            )
    }
}

/// Settings category panel
pub struct SettingsCategoryPanel {
    pub category: SettingsCategory,
    pub items: Vec<SettingItem>,
    pub theme: MaterialTheme,
}

impl SettingsCategoryPanel {
    pub fn new(category: SettingsCategory, items: Vec<SettingItem>, theme: MaterialTheme) -> Self {
        Self {
            category,
            items,
            theme,
        }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .child(
                // Category header
                div()
                    .flex()
                    .items_center()
                    .mb(self.theme.spacing(2.0))
                    .child(
                        div()
                            .text_xl()
                            .mr(self.theme.spacing(1.0))
                            .child(self.category.icon()),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(self.theme.text.primary)
                            .child(self.category.label()),
                    ),
            )
            .children(
                // Setting items
                self.items
                    .iter()
                    .map(|item| item.render(window, cx)),
            )
    }
}

/// Main Settings view
pub struct SettingsView {
    pub active_category: SettingsCategory,
    pub theme_setting: String,
    pub default_project_path: String,
    pub current_project_path: String,
    pub mcp_port: String,
    pub mcp_config_path: String,
    pub theme: MaterialTheme,
}

impl SettingsView {
    pub fn new(
        theme_setting: String,
        default_project_path: String,
        current_project_path: String,
        mcp_port: String,
        mcp_config_path: String,
        theme: MaterialTheme,
    ) -> Self {
        Self {
            active_category: SettingsCategory::General,
            theme_setting,
            default_project_path,
            current_project_path,
            mcp_port,
            mcp_config_path,
            theme,
        }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        let categories = vec![
            SettingsCategory::General,
            SettingsCategory::Project,
            SettingsCategory::MCP,
            SettingsCategory::ClaudeCode,
        ];

        // Create sample settings for the active category
        let items = self.get_settings_items();
        let panel = SettingsCategoryPanel::new(self.active_category.clone(), items, self.theme.clone());

        div()
            .flex()
            .size_full()
            .child(
                // Left sidebar - category list
                div()
                    .flex()
                    .flex_col()
                    .w(px(240.0))
                    .h_full()
                    .bg(self.theme.background.default)
                    .border_r_1()
                    .border_color(self.theme.border.divider)
                    .p(self.theme.spacing(2.0))
                    .children(
                        categories.into_iter().map(|cat| {
                            let is_active = cat == self.active_category;
                            let mut item = div()
                                .flex()
                                .items_center()
                                .px(self.theme.spacing(1.5))
                                .py(self.theme.spacing(1.0))
                                .mb(self.theme.spacing(0.5))
                                .rounded(self.theme.shape.border_radius_sm)
                                .child(
                                    div()
                                        .text_base()
                                        .mr(self.theme.spacing(1.0))
                                        .child(cat.icon()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if is_active {
                                            self.theme.secondary.on_secondary_container
                                        } else {
                                            self.theme.text.secondary
                                        })
                                        .child(cat.label()),
                                );

                            if is_active {
                                item = item.bg(self.theme.secondary.container);
                            }

                            item
                        }),
                    ),
            )
            .child(
                // Right panel - settings content
                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .p(self.theme.spacing(3.0))
                    .child(panel.render(window, cx)),
            )
    }

    fn get_settings_items(&self) -> Vec<SettingItem> {
        match self.active_category {
            SettingsCategory::General => vec![
                SettingItem::new(
                    "Theme",
                    "Choose color theme (System/Light/Dark)",
                    &self.theme_setting,
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Language",
                    "Select interface language",
                    "English (US)",
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Font Size",
                    "Editor and UI font size",
                    "14px",
                    self.theme.clone(),
                ),
            ],
            SettingsCategory::Project => vec![
                SettingItem::new(
                    "Current Project",
                    "Currently opened project path",
                    &self.current_project_path,
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Default Directory",
                    "Default directory for new projects",
                    &self.default_project_path,
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Git User Name",
                    "Git commit author name",
                    "Your Name",
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Git User Email",
                    "Git commit author email",
                    "your.email@example.com",
                    self.theme.clone(),
                ),
            ],
            SettingsCategory::MCP => vec![
                SettingItem::new(
                    "Server Port",
                    "MCP HTTP server port",
                    &self.mcp_port,
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Config Path",
                    "Path to mcp-session.json configuration file",
                    &self.mcp_config_path,
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Auto Start",
                    "Start MCP server on launch",
                    "Enabled",
                    self.theme.clone(),
                ),
            ],
            SettingsCategory::ClaudeCode => vec![
                SettingItem::new(
                    "CLI Path",
                    "Path to Claude CLI executable",
                    "/usr/local/bin/claude",
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Model",
                    "Default Claude model to use",
                    "claude-sonnet-4-5",
                    self.theme.clone(),
                ),
                SettingItem::new(
                    "Max Tokens",
                    "Maximum context window size",
                    "200000",
                    self.theme.clone(),
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_category_labels() {
        assert_eq!(SettingsCategory::General.label(), "General");
        assert_eq!(SettingsCategory::Project.label(), "Project");
        assert_eq!(SettingsCategory::MCP.label(), "MCP Server");
        assert_eq!(SettingsCategory::ClaudeCode.label(), "Claude Code");
    }

    #[test]
    fn test_settings_view_creation() {
        let theme = MaterialTheme::dark();
        let view = SettingsView::new(theme);
        assert_eq!(view.active_category, SettingsCategory::General);
    }

    #[test]
    fn test_get_settings_items() {
        let theme = MaterialTheme::dark();
        let view = SettingsView::new(theme);
        let items = view.get_settings_items();
        assert_eq!(items.len(), 3); // General category has 3 items
    }
}
