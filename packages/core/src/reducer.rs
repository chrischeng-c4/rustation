//! State reducer - pure state transitions.
//!
//! All state changes go through the reducer. The reducer is a pure function:
//! - Takes current state and action
//! - Returns nothing (mutates in place for efficiency)
//! - No side effects (async operations handled separately)

use crate::actions::{Action, DockerServiceData, JustCommandData, TaskStatusData};
use crate::app_state::{
    AppError, AppState, DockerServiceInfo, JustCommandInfo, ProjectState, RecentProject,
    ServiceStatus, ServiceType, TaskStatus,
};
use crate::persistence;

/// Apply an action to the state.
///
/// This function handles synchronous state mutations only.
/// Async operations (Docker calls, etc.) are handled by the dispatcher
/// which calls this reducer after async operations complete.
pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        // ====================================================================
        // Project Management
        // ====================================================================
        Action::OpenProject { path } => {
            // Check if already open
            if state.projects.iter().any(|p| p.path == path) {
                // Switch to existing project
                if let Some(idx) = state.projects.iter().position(|p| p.path == path) {
                    state.active_project_index = idx;
                }
                return;
            }

            // Create new project
            let mut project = ProjectState::new(path.clone());

            // Load and apply persisted project state (only if path exists on disk)
            // This prevents loading stale state for test paths that don't exist
            if std::path::Path::new(&path).exists() {
                if let Ok(Some(persisted)) = persistence::load_project(&path) {
                    persisted.apply_to(&mut project);
                }
            }

            state.projects.push(project);
            state.active_project_index = state.projects.len() - 1;

            // Update recent_projects (only for real paths)
            if std::path::Path::new(&path).exists() {
                update_recent_projects(state, &path);
            }
        }

        Action::CloseProject { index } => {
            if index < state.projects.len() {
                // Save project state before closing (only for real paths)
                let project = &state.projects[index];
                if std::path::Path::new(&project.path).exists() {
                    let _ = persistence::save_project(project);
                }

                state.projects.remove(index);

                // Adjust active index
                if state.projects.is_empty() {
                    state.active_project_index = 0;
                } else if state.active_project_index >= state.projects.len() {
                    state.active_project_index = state.projects.len() - 1;
                } else if index < state.active_project_index {
                    state.active_project_index -= 1;
                }
            }
        }

        Action::SwitchProject { index } => {
            if index < state.projects.len() {
                state.active_project_index = index;
            }
        }

        Action::SetFeatureTab { tab } => {
            if let Some(project) = state.active_project_mut() {
                project.active_tab = tab;
                // Save project state when tab changes (only for real paths)
                if std::path::Path::new(&project.path).exists() {
                    let _ = persistence::save_project(project);
                }
            }
        }

        // ====================================================================
        // Docker Actions (operate on active project)
        // ====================================================================
        Action::CheckDockerAvailability => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.is_loading = true;
            }
        }

        Action::SetDockerAvailable { available } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.docker_available = Some(available);
                project.dockers.is_loading = false;
            }
        }

        Action::RefreshDockerServices => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.is_loading = true;
            }
        }

        Action::SetDockerServices { services } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.services = services.into_iter().map(|s| s.into()).collect();
                project.dockers.is_loading = false;
            }
        }

        Action::StartDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(service) = project
                    .dockers
                    .services
                    .iter_mut()
                    .find(|s| s.id == service_id)
                {
                    service.status = ServiceStatus::Starting;
                }
            }
        }

        Action::StopDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(service) = project
                    .dockers
                    .services
                    .iter_mut()
                    .find(|s| s.id == service_id)
                {
                    service.status = ServiceStatus::Stopping;
                }
            }
        }

        Action::RestartDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(service) = project
                    .dockers
                    .services
                    .iter_mut()
                    .find(|s| s.id == service_id)
                {
                    service.status = ServiceStatus::Starting;
                }
            }
        }

        Action::SelectDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.selected_service_id = service_id;
                project.dockers.logs.clear();
            }
        }

        Action::FetchDockerLogs { .. } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.is_loading_logs = true;
            }
        }

        Action::SetDockerLogs { logs } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.logs = logs;
                project.dockers.is_loading_logs = false;
            }
        }

        Action::CreateDatabase { .. } => {
            // Async trigger - no immediate state change
        }

        Action::CreateVhost { .. } => {
            // Async trigger - no immediate state change
        }

        Action::SetDockerLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.is_loading = is_loading;
            }
        }

        Action::SetDockerLogsLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                project.dockers.is_loading_logs = is_loading;
            }
        }

        // ====================================================================
        // Tasks Actions (operate on active project)
        // ====================================================================
        Action::LoadJustfileCommands { .. } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.is_loading = true;
                project.tasks.error = None;
            }
        }

        Action::SetJustfileCommands { commands } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.commands = commands.into_iter().map(|c| c.into()).collect();
                project.tasks.is_loading = false;
            }
        }

        Action::RunJustCommand { name, .. } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.active_command = Some(name.clone());
                project.tasks.task_statuses.insert(name, TaskStatus::Running);
                project.tasks.output.clear();
                project.is_modified = true;
            }
        }

        Action::SetTaskStatus { name, status } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.task_statuses.insert(name, status.into());
                // Clear modified flag if task completed
                if matches!(status, TaskStatusData::Success | TaskStatusData::Error) {
                    project.is_modified = false;
                }
            }
        }

        Action::SetActiveCommand { name } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.active_command = name;
            }
        }

        Action::AppendTaskOutput { line } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.output.push(line);
            }
        }

        Action::ClearTaskOutput => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.output.clear();
            }
        }

        Action::SetTasksLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.is_loading = is_loading;
            }
        }

        Action::SetTasksError { error } => {
            if let Some(project) = state.active_project_mut() {
                project.tasks.error = error;
                project.tasks.is_loading = false;
            }
        }

        // ====================================================================
        // Settings Actions (operate on global settings)
        // ====================================================================
        Action::SetTheme { theme } => {
            state.global_settings.theme = theme;
        }

        Action::SetProjectPath { path } => {
            state.global_settings.default_project_path = path;
        }

        // ====================================================================
        // Error Handling
        // ====================================================================
        Action::SetError {
            code,
            message,
            context,
        } => {
            state.error = Some(AppError {
                code,
                message,
                context,
            });
        }

        Action::ClearError => {
            state.error = None;
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Update recent_projects list when opening a project
fn update_recent_projects(state: &mut AppState, path: &str) {
    // Remove existing entry if present (we'll re-add it at the top)
    state.recent_projects.retain(|p| p.path != path);

    // Get project name from path
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    // Add to front of recent projects
    state.recent_projects.insert(
        0,
        RecentProject {
            path: path.to_string(),
            name,
            last_opened: chrono::Utc::now().to_rfc3339(),
        },
    );

    // Keep only last 10 recent projects
    const MAX_RECENT: usize = 10;
    state.recent_projects.truncate(MAX_RECENT);
}

// ============================================================================
// Conversions from Action data types to State data types
// ============================================================================

impl From<DockerServiceData> for DockerServiceInfo {
    fn from(data: DockerServiceData) -> Self {
        Self {
            id: data.id,
            name: data.name,
            image: data.image,
            status: match data.status.as_str() {
                "running" => ServiceStatus::Running,
                "starting" => ServiceStatus::Starting,
                "stopping" => ServiceStatus::Stopping,
                "error" => ServiceStatus::Error,
                _ => ServiceStatus::Stopped,
            },
            port: data.port,
            service_type: match data.service_type.as_str() {
                "Database" => ServiceType::Database,
                "MessageBroker" => ServiceType::MessageBroker,
                "Cache" => ServiceType::Cache,
                _ => ServiceType::Other,
            },
        }
    }
}

impl From<JustCommandData> for JustCommandInfo {
    fn from(data: JustCommandData) -> Self {
        Self {
            name: data.name,
            description: data.description,
            recipe: data.recipe,
        }
    }
}

impl From<TaskStatusData> for TaskStatus {
    fn from(data: TaskStatusData) -> Self {
        match data {
            TaskStatusData::Idle => TaskStatus::Idle,
            TaskStatusData::Running => TaskStatus::Running,
            TaskStatusData::Success => TaskStatus::Success,
            TaskStatusData::Error => TaskStatus::Error,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{FeatureTab, Theme};

    /// Helper to create a state with one project for testing
    fn state_with_project() -> AppState {
        let mut state = AppState::default();
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/project".to_string(),
            },
        );
        state
    }

    // ========================================================================
    // Project Management Tests
    // ========================================================================

    #[test]
    fn test_reduce_open_project() {
        let mut state = AppState::default();
        assert!(state.projects.is_empty());

        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/project".to_string(),
            },
        );

        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0].path, "/test/project");
        assert_eq!(state.projects[0].name, "project");
        assert_eq!(state.active_project_index, 0);
    }

    #[test]
    fn test_reduce_open_multiple_projects() {
        let mut state = AppState::default();

        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/a".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/b".to_string(),
            },
        );

        assert_eq!(state.projects.len(), 2);
        assert_eq!(state.active_project_index, 1); // Active is last opened
    }

    #[test]
    fn test_reduce_open_existing_project_switches() {
        let mut state = AppState::default();

        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/a".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/b".to_string(),
            },
        );
        assert_eq!(state.active_project_index, 1);

        // Opening existing project switches to it
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/a".to_string(),
            },
        );
        assert_eq!(state.projects.len(), 2); // No duplicate
        assert_eq!(state.active_project_index, 0);
    }

    #[test]
    fn test_reduce_close_project() {
        let mut state = AppState::default();

        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/a".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/b".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/c".to_string(),
            },
        );
        assert_eq!(state.active_project_index, 2);

        // Close active project
        reduce(&mut state, Action::CloseProject { index: 2 });
        assert_eq!(state.projects.len(), 2);
        assert_eq!(state.active_project_index, 1);

        // Close project before active
        reduce(&mut state, Action::CloseProject { index: 0 });
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.active_project_index, 0);
    }

    #[test]
    fn test_reduce_switch_project() {
        let mut state = AppState::default();

        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/a".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/b".to_string(),
            },
        );
        assert_eq!(state.active_project_index, 1);

        reduce(&mut state, Action::SwitchProject { index: 0 });
        assert_eq!(state.active_project_index, 0);
    }

    #[test]
    fn test_reduce_set_feature_tab() {
        let mut state = state_with_project();
        assert_eq!(state.active_project().unwrap().active_tab, FeatureTab::Tasks);

        reduce(
            &mut state,
            Action::SetFeatureTab {
                tab: FeatureTab::Dockers,
            },
        );
        assert_eq!(
            state.active_project().unwrap().active_tab,
            FeatureTab::Dockers
        );
    }

    // ========================================================================
    // Docker Actions Tests (with project context)
    // ========================================================================

    #[test]
    fn test_reduce_docker_availability() {
        let mut state = state_with_project();

        reduce(&mut state, Action::CheckDockerAvailability);
        assert!(state.active_project().unwrap().dockers.is_loading);

        reduce(&mut state, Action::SetDockerAvailable { available: true });
        let project = state.active_project().unwrap();
        assert_eq!(project.dockers.docker_available, Some(true));
        assert!(!project.dockers.is_loading);
    }

    #[test]
    fn test_reduce_docker_services() {
        let mut state = state_with_project();

        reduce(&mut state, Action::RefreshDockerServices);
        assert!(state.active_project().unwrap().dockers.is_loading);

        reduce(
            &mut state,
            Action::SetDockerServices {
                services: vec![DockerServiceData {
                    id: "pg-1".to_string(),
                    name: "PostgreSQL".to_string(),
                    image: "postgres:16".to_string(),
                    status: "running".to_string(),
                    port: Some(5432),
                    service_type: "Database".to_string(),
                }],
            },
        );

        let project = state.active_project().unwrap();
        assert!(!project.dockers.is_loading);
        assert_eq!(project.dockers.services.len(), 1);
        assert_eq!(project.dockers.services[0].name, "PostgreSQL");
    }

    #[test]
    fn test_reduce_start_stop_service() {
        let mut state = state_with_project();
        state
            .active_project_mut()
            .unwrap()
            .dockers
            .services
            .push(DockerServiceInfo {
                id: "pg-1".to_string(),
                name: "PostgreSQL".to_string(),
                image: "postgres:16".to_string(),
                status: ServiceStatus::Stopped,
                port: Some(5432),
                service_type: ServiceType::Database,
            });

        reduce(
            &mut state,
            Action::StartDockerService {
                service_id: "pg-1".to_string(),
            },
        );
        assert_eq!(
            state.active_project().unwrap().dockers.services[0].status,
            ServiceStatus::Starting
        );

        reduce(
            &mut state,
            Action::StopDockerService {
                service_id: "pg-1".to_string(),
            },
        );
        assert_eq!(
            state.active_project().unwrap().dockers.services[0].status,
            ServiceStatus::Stopping
        );
    }

    // ========================================================================
    // Tasks Actions Tests (with project context)
    // ========================================================================

    #[test]
    fn test_reduce_justfile_commands() {
        let mut state = state_with_project();

        reduce(
            &mut state,
            Action::LoadJustfileCommands {
                path: "/some/path".to_string(),
            },
        );
        assert!(state.active_project().unwrap().tasks.is_loading);

        reduce(
            &mut state,
            Action::SetJustfileCommands {
                commands: vec![JustCommandData {
                    name: "test".to_string(),
                    description: Some("Run tests".to_string()),
                    recipe: "cargo test".to_string(),
                }],
            },
        );

        let project = state.active_project().unwrap();
        assert!(!project.tasks.is_loading);
        assert_eq!(project.tasks.commands.len(), 1);
        assert_eq!(project.tasks.commands[0].name, "test");
    }

    #[test]
    fn test_reduce_run_command_sets_modified() {
        let mut state = state_with_project();

        reduce(
            &mut state,
            Action::RunJustCommand {
                name: "test".to_string(),
                cwd: "/some/dir".to_string(),
            },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.tasks.active_command, Some("test".to_string()));
        assert_eq!(
            project.tasks.task_statuses.get("test"),
            Some(&TaskStatus::Running)
        );
        assert!(project.is_modified);
    }

    #[test]
    fn test_reduce_task_output() {
        let mut state = state_with_project();

        reduce(
            &mut state,
            Action::AppendTaskOutput {
                line: "line 1".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AppendTaskOutput {
                line: "line 2".to_string(),
            },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.tasks.output.len(), 2);

        reduce(&mut state, Action::ClearTaskOutput);
        assert!(state.active_project().unwrap().tasks.output.is_empty());
    }

    // ========================================================================
    // Settings & Error Tests
    // ========================================================================

    #[test]
    fn test_reduce_settings() {
        let mut state = AppState::default();
        assert_eq!(state.global_settings.theme, Theme::System);

        reduce(&mut state, Action::SetTheme { theme: Theme::Dark });
        assert_eq!(state.global_settings.theme, Theme::Dark);

        reduce(
            &mut state,
            Action::SetProjectPath {
                path: Some("/home/user/projects".to_string()),
            },
        );
        assert_eq!(
            state.global_settings.default_project_path,
            Some("/home/user/projects".to_string())
        );
    }

    #[test]
    fn test_reduce_error_handling() {
        let mut state = AppState::default();
        assert!(state.error.is_none());

        reduce(
            &mut state,
            Action::SetError {
                code: "DOCKER_ERROR".to_string(),
                message: "Docker not running".to_string(),
                context: Some("start_service".to_string()),
            },
        );

        assert!(state.error.is_some());
        let error = state.error.as_ref().unwrap();
        assert_eq!(error.code, "DOCKER_ERROR");

        reduce(&mut state, Action::ClearError);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_actions_noop_without_project() {
        let mut state = AppState::default();

        // These should not crash when no project exists
        reduce(&mut state, Action::CheckDockerAvailability);
        reduce(&mut state, Action::RefreshDockerServices);
        reduce(
            &mut state,
            Action::AppendTaskOutput {
                line: "test".to_string(),
            },
        );

        // State unchanged
        assert!(state.projects.is_empty());
    }
}
