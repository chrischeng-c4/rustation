//! Dockers view - Docker container management
//!
//! Based on desktop/src/renderer/src/features/dockers/DockersPage.tsx

use gpui::*;
use rstn_core::state::{DockerService, ServiceStatus, ServiceType};
use rstn_ui::{EmptyState, MaterialTheme, PageHeader, Themed};

/// Docker service card component
pub struct ServiceCard {
    service: DockerService,
    theme: MaterialTheme,
}

impl ServiceCard {
    pub fn new(service: DockerService, theme: MaterialTheme) -> Self {
        Self { service, theme }
    }

    /// Get status color based on service state
    fn status_color(&self) -> Rgba {
        match self.service.status {
            ServiceStatus::Running => rgb(0x4CAF50),  // Green
            ServiceStatus::Stopped => rgb(0x9E9E9E),  // Grey
            ServiceStatus::Starting => rgb(0xFFC107), // Amber
            ServiceStatus::Error => rgb(0xF44336),    // Red
        }
    }

    /// Get status text
    fn status_text(&self) -> &'static str {
        match self.service.status {
            ServiceStatus::Running => "Running",
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::Starting => "Starting...",
            ServiceStatus::Error => "Error",
        }
    }

    /// Get service type icon
    fn service_icon(&self) -> &'static str {
        match self.service.service_type {
            ServiceType::Database => "🗄️",
            ServiceType::MessageBroker => "📨",
            ServiceType::Cache => "⚡",
            ServiceType::Other => "📦",
        }
    }

    pub fn render(&self) -> Div {
        div()
            .card(&self.theme)
            .mb(self.theme.spacing(1.5))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb(self.theme.spacing(1.0))
                    .child(
                        // Service name with icon
                        div()
                            .flex()
                            .items_center()
                            .gap(self.theme.spacing(1.0))
                            .child(
                                div()
                                    .text_xl()
                                    .child(self.service_icon()),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.service.name.clone()),
                            ),
                    )
                    .child(
                        // Status badge
                        div()
                            .px(self.theme.spacing(1.0))
                            .py(self.theme.spacing(0.5))
                            .bg(self.status_color())
                            .rounded(self.theme.shape.border_radius_xs)
                            .text_xs()
                            .text_color(rgb(0xFFFFFF))
                            .child(self.status_text()),
                    ),
            )
            .child(
                // Image info
                div()
                    .flex()
                    .flex_col()
                    .gap(self.theme.spacing(0.5))
                    .mb(self.theme.spacing(1.0))
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.text.secondary)
                            .child(format!("Image: {}", self.service.image)),
                    )
                    .children(
                        self.service.port.map(|port| {
                            div()
                                .text_sm()
                                .text_color(self.theme.text.secondary)
                                .child(format!("Port: {}", port))
                        })
                    )
                    .children(
                        self.service.project_group.as_ref().map(|group| {
                            div()
                                .text_sm()
                                .text_color(self.theme.text.secondary)
                                .child(format!("Group: {}", group))
                        })
                    ),
            )
            .child(
                // Action buttons
                div()
                    .flex()
                    .gap(self.theme.spacing(1.0))
                    .child(
                        div()
                            .primary_button(&self.theme)
                            .child(match self.service.status {
                                ServiceStatus::Running => "Stop",
                                ServiceStatus::Stopped => "Start",
                                ServiceStatus::Starting => "Starting...",
                                ServiceStatus::Error => "Restart",
                            }),
                    )
                    .child(
                        div()
                            .secondary_button(&self.theme)
                            .child("Logs"),
                    )
                    .child(
                        div()
                            .secondary_button(&self.theme)
                            .child("Remove"),
                    ),
            )
    }
}

/// Service group section
pub struct ServiceGroup {
    title: String,
    services: Vec<DockerService>,
    theme: MaterialTheme,
}

impl ServiceGroup {
    pub fn new(title: String, services: Vec<DockerService>, theme: MaterialTheme) -> Self {
        Self {
            title,
            services,
            theme,
        }
    }

    pub fn render(&self, _window: &mut Window, _cx: &mut App) -> Div {
        div()
            .flex()
            .flex_col()
            .mb(self.theme.spacing(3.0))
            .child(
                // Group title
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .mb(self.theme.spacing(1.5))
                    .child(self.title.clone()),
            )
            .children(
                self.services
                    .iter()
                    .map(|service| ServiceCard::new(service.clone(), self.theme.clone()).render()),
            )
    }
}

/// Dockers page view
///
/// Displays Docker services grouped by project
pub struct DockersView {
    services: Vec<DockerService>,
    theme: MaterialTheme,
}

impl DockersView {
    pub fn new(services: Vec<DockerService>, theme: MaterialTheme) -> Self {
        Self { services, theme }
    }

    /// Group services by project_group
    fn group_services(&self) -> Vec<ServiceGroup> {
        use std::collections::HashMap;

        let mut groups: HashMap<String, Vec<DockerService>> = HashMap::new();

        for service in &self.services {
            let group_name = service
                .project_group
                .clone()
                .unwrap_or_else(|| "Other".to_string());
            groups.entry(group_name).or_default().push(service.clone());
        }

        let mut result: Vec<_> = groups
            .into_iter()
            .map(|(title, services)| ServiceGroup::new(title, services, self.theme.clone()))
            .collect();

        // Sort by title
        result.sort_by(|a, b| a.title.cmp(&b.title));
        result
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        let page_header = PageHeader::new(
            "Docker Services",
            Some("Manage containerized services"),
            self.theme.clone(),
        );

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                page_header.render(Some(
                    div()
                        .flex()
                        .gap(self.theme.spacing(1.0))
                        .child(
                            div()
                                .secondary_button(&self.theme)
                                .child("Refresh"),
                        )
                        .child(
                            div()
                                .primary_button(&self.theme)
                                .child("New Service"),
                        ),
                )),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p(self.theme.spacing(2.0))
                    .children(if self.services.is_empty() {
                        vec![EmptyState::new(
                            "🐳",
                            "No Services",
                            "No Docker services found",
                            self.theme.clone(),
                        )
                        .render(Some(
                            div()
                                .primary_button(&self.theme)
                                .child("Scan Services"),
                        ))]
                    } else {
                        self.group_services()
                            .into_iter()
                            .map(|group| group.render(window, cx))
                            .collect()
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_card_creation() {
        let service = DockerService {
            id: "test-id".to_string(),
            name: "postgres".to_string(),
            image: "postgres:15".to_string(),
            status: ServiceStatus::Running,
            port: Some(5432),
            service_type: ServiceType::Database,
            project_group: Some("rstn".to_string()),
            is_rstn_managed: true,
        };

        let theme = MaterialTheme::dark();
        let card = ServiceCard::new(service.clone(), theme);
        assert_eq!(card.service.name, "postgres");
        assert_eq!(card.status_text(), "Running");
    }

    #[test]
    fn test_dockers_view_grouping() {
        let services = vec![
            DockerService {
                id: "1".to_string(),
                name: "postgres".to_string(),
                image: "postgres:15".to_string(),
                status: ServiceStatus::Running,
                port: Some(5432),
                service_type: ServiceType::Database,
                project_group: Some("rstn".to_string()),
                is_rstn_managed: true,
            },
            DockerService {
                id: "2".to_string(),
                name: "redis".to_string(),
                image: "redis:7".to_string(),
                status: ServiceStatus::Running,
                port: Some(6379),
                service_type: ServiceType::Cache,
                project_group: Some("rstn".to_string()),
                is_rstn_managed: true,
            },
        ];

        let theme = MaterialTheme::dark();
        let view = DockersView::new(services, theme);
        let groups = view.group_services();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].title, "rstn");
        assert_eq!(groups[0].services.len(), 2);
    }
}
