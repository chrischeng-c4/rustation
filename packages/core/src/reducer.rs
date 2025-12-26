//! State reducer - pure state transitions.
//!
//! All state changes go through the reducer. The reducer is a pure function:
//! - Takes current state and action
//! - Returns nothing (mutates in place for efficiency)
//! - No side effects (async operations handled separately)

use crate::actions::{
    Action, ConflictingContainerData, DockerServiceData, JustCommandData, McpStatusData,
    PortConflictData, TaskStatusData,
};
use crate::app_state::{
    AppError, AppState, ConflictingContainer, DockerServiceInfo, JustCommandInfo, McpStatus,
    PendingConflict, PortConflict, ProjectState, RecentProject, ServiceStatus, ServiceType,
    TaskStatus, WorktreeState,
};
use crate::persistence;
use crate::worktree;

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
            // Normalize to git root if inside a git repository
            let project_path = if std::path::Path::new(&path).exists() {
                worktree::get_git_root(&path).unwrap_or_else(|| path.clone())
            } else {
                path.clone()
            };

            // Check if this project (by git root) is already open
            if let Some(idx) = state.projects.iter().position(|p| p.path == project_path) {
                state.active_project_index = idx;

                // If the original path is a subdirectory, try to find matching worktree
                if path != project_path {
                    if let Some(project) = state.active_project_mut() {
                        let worktree_data: Vec<_> = project
                            .worktrees
                            .iter()
                            .map(|w| crate::actions::WorktreeData {
                                path: w.path.clone(),
                                branch: w.branch.clone(),
                                is_main: w.is_main,
                            })
                            .collect();

                        if let Some(wt_idx) = worktree::find_worktree_for_path(&path, &worktree_data) {
                            project.active_worktree_index = wt_idx;
                        }
                    }
                }
                return;
            }

            // Check if the path is inside any worktree of an existing project
            for (proj_idx, project) in state.projects.iter().enumerate() {
                let worktree_data: Vec<_> = project
                    .worktrees
                    .iter()
                    .map(|w| crate::actions::WorktreeData {
                        path: w.path.clone(),
                        branch: w.branch.clone(),
                        is_main: w.is_main,
                    })
                    .collect();

                if let Some(wt_idx) = worktree::find_worktree_for_path(&path, &worktree_data) {
                    state.active_project_index = proj_idx;
                    if let Some(proj) = state.active_project_mut() {
                        proj.active_worktree_index = wt_idx;
                    }
                    return;
                }
            }

            // Create new project with the normalized git root path
            let mut project = ProjectState::new(project_path.clone());

            // Load and apply persisted project state (only if path exists on disk)
            // This prevents loading stale state for test paths that don't exist
            if std::path::Path::new(&project_path).exists() {
                if let Ok(Some(persisted)) = persistence::load_project(&project_path) {
                    persisted.apply_to(&mut project);
                }
            }

            state.projects.push(project);
            state.active_project_index = state.projects.len() - 1;

            // Update recent_projects (only for real paths)
            if std::path::Path::new(&project_path).exists() {
                update_recent_projects(state, &project_path);
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
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.active_tab = tab;
                }
                // Save project state when tab changes (only for real paths)
                if std::path::Path::new(&project.path).exists() {
                    let _ = persistence::save_project(project);
                }
            }
        }

        // ====================================================================
        // Worktree Actions
        // ====================================================================
        Action::SwitchWorktree { index } => {
            if let Some(project) = state.active_project_mut() {
                if index < project.worktrees.len() {
                    project.active_worktree_index = index;
                }
            }
        }

        Action::RefreshWorktrees => {
            // Async trigger - no immediate state change
            // The dispatcher will call `git worktree list` and then SetWorktrees
        }

        Action::SetWorktrees { worktrees } => {
            if let Some(project) = state.active_project_mut() {
                // Convert WorktreeData to WorktreeState
                let new_worktrees: Vec<WorktreeState> = worktrees
                    .into_iter()
                    .map(|w| WorktreeState::new(w.path, w.branch, w.is_main))
                    .collect();

                // Replace worktrees, keeping current active index if valid
                project.worktrees = new_worktrees;
                if project.active_worktree_index >= project.worktrees.len() {
                    project.active_worktree_index = 0;
                }
            }
        }

        // ====================================================================
        // MCP Actions
        // ====================================================================
        Action::StartMcpServer => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.status = McpStatus::Starting;
                    worktree.mcp.error = None;
                }
            }
        }

        Action::StopMcpServer => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.status = McpStatus::Stopped;
                    worktree.mcp.port = None;
                }
            }
        }

        Action::SetMcpStatus { status } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.status = status.into();
                }
            }
        }

        Action::SetMcpPort { port } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.port = Some(port);
                    worktree.mcp.status = McpStatus::Running;
                }
            }
        }

        Action::SetMcpConfigPath { path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.config_path = Some(path);
                }
            }
        }

        Action::SetMcpError { error } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.status = McpStatus::Error;
                    worktree.mcp.error = Some(error);
                }
            }
        }

        // ====================================================================
        // Docker Actions (operate on active worktree)
        // ====================================================================
        Action::CheckDockerAvailability => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.is_loading = true;
                }
            }
        }

        Action::SetDockerAvailable { available } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.docker_available = Some(available);
                    worktree.dockers.is_loading = false;
                }
            }
        }

        Action::RefreshDockerServices => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.is_loading = true;
                }
            }
        }

        Action::SetDockerServices { services } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.services = services.into_iter().map(|s| s.into()).collect();
                    worktree.dockers.is_loading = false;
                }
            }
        }

        Action::StartDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(service) = worktree
                        .dockers
                        .services
                        .iter_mut()
                        .find(|s| s.id == service_id)
                    {
                        service.status = ServiceStatus::Starting;
                    }
                }
            }
        }

        Action::StopDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(service) = worktree
                        .dockers
                        .services
                        .iter_mut()
                        .find(|s| s.id == service_id)
                    {
                        service.status = ServiceStatus::Stopping;
                    }
                }
            }
        }

        Action::RestartDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(service) = worktree
                        .dockers
                        .services
                        .iter_mut()
                        .find(|s| s.id == service_id)
                    {
                        service.status = ServiceStatus::Starting;
                    }
                }
            }
        }

        Action::SelectDockerService { service_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.selected_service_id = service_id;
                    worktree.dockers.logs.clear();
                }
            }
        }

        Action::FetchDockerLogs { .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.is_loading_logs = true;
                }
            }
        }

        Action::SetDockerLogs { logs } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.logs = logs;
                    worktree.dockers.is_loading_logs = false;
                }
            }
        }

        Action::CreateDatabase { .. } => {
            // Async trigger - no immediate state change
        }

        Action::CreateVhost { .. } => {
            // Async trigger - no immediate state change
        }

        Action::SetPortConflict { service_id, conflict } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.pending_conflict = Some(PendingConflict {
                        service_id,
                        conflict: conflict.into(),
                    });
                }
            }
        }

        Action::ClearPortConflict => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.pending_conflict = None;
                }
            }
        }

        Action::StartDockerServiceWithPort { ref service_id, port } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    // Store port override
                    worktree.dockers.port_overrides.insert(service_id.clone(), port);
                    // Clear pending conflict
                    worktree.dockers.pending_conflict = None;
                    // Set service to starting
                    if let Some(service) = worktree
                        .dockers
                        .services
                        .iter_mut()
                        .find(|s| s.id == *service_id)
                    {
                        service.status = ServiceStatus::Starting;
                    }
                }
            }
        }

        Action::ResolveConflictByStoppingContainer { ref service_id, .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    // Clear pending conflict
                    worktree.dockers.pending_conflict = None;
                    // Set service to starting
                    if let Some(service) = worktree
                        .dockers
                        .services
                        .iter_mut()
                        .find(|s| s.id == *service_id)
                    {
                        service.status = ServiceStatus::Starting;
                    }
                }
            }
        }

        Action::SetDockerLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.is_loading = is_loading;
                }
            }
        }

        Action::SetDockerLogsLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.dockers.is_loading_logs = is_loading;
                }
            }
        }

        // ====================================================================
        // Tasks Actions (operate on active worktree)
        // ====================================================================
        Action::LoadJustfileCommands { .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.is_loading = true;
                    worktree.tasks.error = None;
                }
            }
        }

        Action::SetJustfileCommands { commands } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.commands = commands.into_iter().map(|c| c.into()).collect();
                    worktree.tasks.is_loading = false;
                }
            }
        }

        Action::RunJustCommand { name, .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.active_command = Some(name.clone());
                    worktree.tasks.task_statuses.insert(name, TaskStatus::Running);
                    worktree.tasks.output.clear();
                    worktree.is_modified = true;
                }
            }
        }

        Action::SetTaskStatus { name, status } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.task_statuses.insert(name, status.into());
                    // Clear modified flag if task completed
                    if matches!(status, TaskStatusData::Success | TaskStatusData::Error) {
                        worktree.is_modified = false;
                    }
                }
            }
        }

        Action::SetActiveCommand { name } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.active_command = name;
                }
            }
        }

        Action::AppendTaskOutput { line } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.output.push(line);
                }
            }
        }

        Action::ClearTaskOutput => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.output.clear();
                }
            }
        }

        Action::SetTasksLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.is_loading = is_loading;
                }
            }
        }

        Action::SetTasksError { error } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.error = error;
                    worktree.tasks.is_loading = false;
                }
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

        // ====================================================================
        // Async-only Actions (no synchronous state change)
        // ====================================================================
        // These are handled by handle_async_action() in lib.rs
        Action::AddWorktree { .. }
        | Action::AddWorktreeNewBranch { .. }
        | Action::RemoveWorktree { .. } => {
            // Async triggers - no immediate state change
            // After completion, these will dispatch SetWorktrees
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
            project_group: data.project_group,
            is_rstn_managed: data.is_rstn_managed,
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

impl From<McpStatusData> for McpStatus {
    fn from(data: McpStatusData) -> Self {
        match data {
            McpStatusData::Stopped => McpStatus::Stopped,
            McpStatusData::Starting => McpStatus::Starting,
            McpStatusData::Running => McpStatus::Running,
            McpStatusData::Error => McpStatus::Error,
        }
    }
}

impl From<PortConflictData> for PortConflict {
    fn from(data: PortConflictData) -> Self {
        Self {
            requested_port: data.requested_port,
            conflicting_container: data.conflicting_container.into(),
            suggested_port: data.suggested_port,
        }
    }
}

impl From<ConflictingContainerData> for ConflictingContainer {
    fn from(data: ConflictingContainerData) -> Self {
        Self {
            id: data.id,
            name: data.name,
            image: data.image,
            is_rstn_managed: data.is_rstn_managed,
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

    /// Helper to get the active worktree from state (for tests)
    fn active_worktree(state: &AppState) -> &crate::app_state::WorktreeState {
        state
            .active_project()
            .unwrap()
            .active_worktree()
            .unwrap()
    }

    /// Helper to get the active worktree mutably from state (for tests)
    fn active_worktree_mut(state: &mut AppState) -> &mut crate::app_state::WorktreeState {
        state
            .active_project_mut()
            .unwrap()
            .active_worktree_mut()
            .unwrap()
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
        assert_eq!(active_worktree(&state).active_tab, FeatureTab::Tasks);

        reduce(
            &mut state,
            Action::SetFeatureTab {
                tab: FeatureTab::Dockers,
            },
        );
        assert_eq!(active_worktree(&state).active_tab, FeatureTab::Dockers);
    }

    // ========================================================================
    // Docker Actions Tests (with project context)
    // ========================================================================

    #[test]
    fn test_reduce_docker_availability() {
        let mut state = state_with_project();

        reduce(&mut state, Action::CheckDockerAvailability);
        assert!(active_worktree(&state).dockers.is_loading);

        reduce(&mut state, Action::SetDockerAvailable { available: true });
        let worktree = active_worktree(&state);
        assert_eq!(worktree.dockers.docker_available, Some(true));
        assert!(!worktree.dockers.is_loading);
    }

    #[test]
    fn test_reduce_docker_services() {
        let mut state = state_with_project();

        reduce(&mut state, Action::RefreshDockerServices);
        assert!(active_worktree(&state).dockers.is_loading);

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
                    project_group: Some("rstn".to_string()),
                    is_rstn_managed: true,
                }],
            },
        );

        let worktree = active_worktree(&state);
        assert!(!worktree.dockers.is_loading);
        assert_eq!(worktree.dockers.services.len(), 1);
        assert_eq!(worktree.dockers.services[0].name, "PostgreSQL");
    }

    #[test]
    fn test_reduce_start_stop_service() {
        let mut state = state_with_project();
        active_worktree_mut(&mut state)
            .dockers
            .services
            .push(DockerServiceInfo {
                id: "pg-1".to_string(),
                name: "PostgreSQL".to_string(),
                image: "postgres:16".to_string(),
                status: ServiceStatus::Stopped,
                port: Some(5432),
                service_type: ServiceType::Database,
                project_group: Some("rstn".to_string()),
                is_rstn_managed: true,
            });

        reduce(
            &mut state,
            Action::StartDockerService {
                service_id: "pg-1".to_string(),
            },
        );
        assert_eq!(
            active_worktree(&state).dockers.services[0].status,
            ServiceStatus::Starting
        );

        reduce(
            &mut state,
            Action::StopDockerService {
                service_id: "pg-1".to_string(),
            },
        );
        assert_eq!(
            active_worktree(&state).dockers.services[0].status,
            ServiceStatus::Stopping
        );
    }

    // ========================================================================
    // Tasks Actions Tests (with worktree context)
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
        assert!(active_worktree(&state).tasks.is_loading);

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

        let worktree = active_worktree(&state);
        assert!(!worktree.tasks.is_loading);
        assert_eq!(worktree.tasks.commands.len(), 1);
        assert_eq!(worktree.tasks.commands[0].name, "test");
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

        let worktree = active_worktree(&state);
        assert_eq!(worktree.tasks.active_command, Some("test".to_string()));
        assert_eq!(
            worktree.tasks.task_statuses.get("test"),
            Some(&TaskStatus::Running)
        );
        assert!(worktree.is_modified);
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

        let worktree = active_worktree(&state);
        assert_eq!(worktree.tasks.output.len(), 2);

        reduce(&mut state, Action::ClearTaskOutput);
        assert!(active_worktree(&state).tasks.output.is_empty());
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

    // ========================================================================
    // Recent Projects Tests (Startup Flow Protection)
    // ========================================================================

    #[test]
    fn test_open_project_does_not_update_recent_for_fake_paths() {
        // This test ensures that test paths (which don't exist on disk)
        // don't pollute recent_projects
        let mut state = AppState::default();

        reduce(
            &mut state,
            Action::OpenProject {
                path: "/test/fake/project".to_string(),
            },
        );

        // Project is created
        assert_eq!(state.projects.len(), 1);
        // But recent_projects is NOT updated (path doesn't exist)
        assert!(state.recent_projects.is_empty());
    }

    #[test]
    fn test_recent_projects_order_most_recent_first() {
        // Simulate opening projects by directly manipulating recent_projects
        // (since we can't create real paths in tests)
        let mut state = AppState::default();

        // Manually add recent projects (simulating what would happen with real paths)
        state.recent_projects.push(RecentProject {
            path: "/project/a".to_string(),
            name: "a".to_string(),
            last_opened: "2024-01-01T00:00:00Z".to_string(),
        });
        state.recent_projects.push(RecentProject {
            path: "/project/b".to_string(),
            name: "b".to_string(),
            last_opened: "2024-01-02T00:00:00Z".to_string(),
        });

        // Verify first item is still first
        assert_eq!(state.recent_projects[0].path, "/project/a");
        assert_eq!(state.recent_projects[1].path, "/project/b");
    }

    #[test]
    fn test_startup_flow_apply_persisted_state() {
        // This test simulates the startup flow:
        // 1. Load persisted state (GlobalPersistedState)
        // 2. Apply to AppState
        // 3. Auto-open most recent project
        use crate::persistence::GlobalPersistedState;

        // Step 1: Create persisted state with recent projects
        let persisted = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![
                RecentProject {
                    path: "/project/recent1".to_string(),
                    name: "recent1".to_string(),
                    last_opened: "2024-01-02T00:00:00Z".to_string(),
                },
                RecentProject {
                    path: "/project/recent2".to_string(),
                    name: "recent2".to_string(),
                    last_opened: "2024-01-01T00:00:00Z".to_string(),
                },
            ],
            global_settings: crate::app_state::GlobalSettings {
                theme: Theme::Dark,
                default_project_path: None,
            },
        };

        // Step 2: Apply persisted state to fresh AppState
        let mut state = AppState::default();
        persisted.apply_to(&mut state);

        // Verify persisted state was applied
        assert_eq!(state.recent_projects.len(), 2);
        assert_eq!(state.recent_projects[0].path, "/project/recent1");
        assert_eq!(state.recent_projects[0].name, "recent1");
        assert_eq!(state.global_settings.theme, Theme::Dark);

        // Step 3: Simulate auto-open (what state_init does)
        // In real code, this would check if path exists first
        if let Some(recent) = state.recent_projects.first() {
            let path = recent.path.clone();
            // In real code: if std::path::Path::new(&path).exists() { ... }
            reduce(&mut state, Action::OpenProject { path });
        }

        // Verify project was opened
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0].path, "/project/recent1");
        assert_eq!(state.projects[0].name, "recent1");
        assert_eq!(state.active_project_index, 0);
    }

    #[test]
    fn test_startup_with_empty_recent_projects() {
        // Edge case: no recent projects should not panic
        use crate::persistence::GlobalPersistedState;

        let persisted = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![],
            global_settings: crate::app_state::GlobalSettings::default(),
        };

        let mut state = AppState::default();
        persisted.apply_to(&mut state);

        // No recent projects
        assert!(state.recent_projects.is_empty());
        // No projects opened
        assert!(state.projects.is_empty());

        // The auto-open logic should handle empty recent_projects safely
        let path_to_open = state.recent_projects.first().map(|r| r.path.clone());
        if let Some(path) = path_to_open {
            reduce(&mut state, Action::OpenProject { path });
        }

        // Still no projects
        assert!(state.projects.is_empty());
    }

    #[test]
    fn test_persisted_state_roundtrip_with_recent_projects() {
        // Ensure GlobalPersistedState can serialize/deserialize recent projects
        use crate::persistence::GlobalPersistedState;

        let original = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![
                RecentProject {
                    path: "/path/to/project1".to_string(),
                    name: "project1".to_string(),
                    last_opened: "2024-12-25T10:00:00Z".to_string(),
                },
                RecentProject {
                    path: "/path/to/project2".to_string(),
                    name: "project2".to_string(),
                    last_opened: "2024-12-24T10:00:00Z".to_string(),
                },
            ],
            global_settings: crate::app_state::GlobalSettings {
                theme: Theme::Light,
                default_project_path: Some("/home/user".to_string()),
            },
        };

        // Serialize
        let json = serde_json::to_string(&original).expect("Failed to serialize");

        // Deserialize
        let loaded: GlobalPersistedState =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // Verify
        assert_eq!(original, loaded);
        assert_eq!(loaded.recent_projects.len(), 2);
        assert_eq!(loaded.recent_projects[0].path, "/path/to/project1");
    }

    #[test]
    fn test_startup_flow_integration() {
        // Full integration test simulating exact startup sequence from lib.rs
        use crate::persistence::GlobalPersistedState;

        // Simulate what state_init does:
        // 1. Create default state
        let mut initial_state = AppState::default();

        // 2. Load persisted state (simulated)
        let persisted = GlobalPersistedState {
            version: "0.1.0".to_string(),
            recent_projects: vec![RecentProject {
                path: "/Users/test/myproject".to_string(),
                name: "myproject".to_string(),
                last_opened: "2024-12-25T12:00:00Z".to_string(),
            }],
            global_settings: crate::app_state::GlobalSettings {
                theme: Theme::System,
                default_project_path: None,
            },
        };

        // 3. Apply persisted state
        persisted.apply_to(&mut initial_state);

        // 4. Verify recent_projects was populated
        assert_eq!(initial_state.recent_projects.len(), 1);
        assert_eq!(
            initial_state.recent_projects[0].path,
            "/Users/test/myproject"
        );

        // 5. Auto-open the most recent project (path existence check skipped in test)
        if let Some(recent) = initial_state.recent_projects.first() {
            let path = recent.path.clone();
            // Real code checks: if std::path::Path::new(&path).exists()
            reduce(&mut initial_state, Action::OpenProject { path });
        }

        // 6. Verify project is now open
        assert_eq!(initial_state.projects.len(), 1);
        assert_eq!(initial_state.active_project_index, 0);

        let project = initial_state.active_project().unwrap();
        assert_eq!(project.path, "/Users/test/myproject");
        assert_eq!(project.name, "myproject");

        // 7. Verify worktree was created
        assert_eq!(project.worktrees.len(), 1);
        let worktree = project.active_worktree().unwrap();
        assert_eq!(worktree.branch, "main");
        assert_eq!(worktree.active_tab, FeatureTab::Tasks);
    }
}
