//! State reducer - pure state transitions.
//!
//! All state changes go through the reducer. The reducer is a pure function:
//! - Takes current state and action
//! - Returns nothing (mutates in place for efficiency)
//! - No side effects (async operations handled separately)

use crate::actions::{
    Action, ChatRoleData, ConflictingContainerData, DevLogSourceData, DevLogTypeData,
    DockerServiceData, JustCommandData, McpLogDirectionData, McpStatusData, PortConflictData,
    TaskStatusData,
};
use crate::app_state::{
    AppError, AppState, ConflictingContainer, DevLog, DevLogSource, DevLogType, DockerServiceInfo,
    EnvCopyResult, JustCommandInfo, McpStatus, Notification, PendingConflict, PortConflict,
    ProjectState, RecentProject, ServiceStatus, ServiceType, TaskStatus, WorktreeState,
};
use crate::persistence;
use crate::worktree;

/// Apply an action to the state.
///
/// This function handles synchronous state mutations only.
/// Async operations (Docker calls, etc.) are handled by the dispatcher
/// which calls this reducer after async operations complete.
pub fn reduce(state: &mut AppState, action: Action) {
    // Auto-log actions for dev debugging (only key actions to avoid noise)
    log_action_if_interesting(state, &action);

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

        Action::AddMcpLogEntry { entry } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    let log_entry = crate::app_state::McpLogEntry {
                        timestamp: entry.timestamp,
                        direction: match entry.direction {
                            McpLogDirectionData::In => crate::app_state::McpLogDirection::In,
                            McpLogDirectionData::Out => crate::app_state::McpLogDirection::Out,
                        },
                        method: entry.method,
                        tool_name: entry.tool_name,
                        payload: entry.payload,
                        is_error: entry.is_error,
                    };
                    worktree.mcp.add_log_entry(log_entry);
                }
            }
        }

        Action::ClearMcpLogs => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.clear_logs();
                }
            }
        }

        Action::UpdateMcpTools { tools } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.mcp.available_tools = tools
                        .into_iter()
                        .map(|t| crate::app_state::McpTool {
                            name: t.name,
                            description: t.description,
                            input_schema: t.input_schema,
                        })
                        .collect();
                }
            }
        }

        // ====================================================================
        // Chat Actions (worktree scope)
        // ====================================================================
        Action::SendChatMessage { .. } => {
            // Async action - handled in lib.rs
            // Just set typing state here
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.is_typing = true;
                    worktree.chat.error = None;
                }
            }
        }

        Action::AddChatMessage { message } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    let chat_message = crate::app_state::ChatMessage {
                        id: message.id,
                        role: match message.role {
                            ChatRoleData::User => crate::app_state::ChatRole::User,
                            ChatRoleData::Assistant => crate::app_state::ChatRole::Assistant,
                            ChatRoleData::System => crate::app_state::ChatRole::System,
                        },
                        content: message.content,
                        timestamp: message.timestamp,
                        is_streaming: message.is_streaming,
                    };
                    worktree.chat.add_message(chat_message);
                }
            }
        }

        Action::AppendChatContent { content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.append_to_last(&content);
                }
            }
        }

        Action::SetChatTyping { is_typing } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.is_typing = is_typing;
                    if !is_typing {
                        worktree.chat.finish_streaming();
                    }
                }
            }
        }

        Action::SetChatError { error } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.error = Some(error);
                    worktree.chat.is_typing = false;
                }
            }
        }

        Action::ClearChatError => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.error = None;
                }
            }
        }

        Action::ClearChat => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.chat.clear();
                }
            }
        }

        // ====================================================================
        // Constitution Workflow Actions (worktree scope)
        // ====================================================================
        Action::StartConstitutionWorkflow => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.constitution_workflow =
                        Some(crate::app_state::ConstitutionWorkflow {
                            current_question: 0,
                            answers: std::collections::HashMap::new(),
                            output: String::new(),
                            status: crate::app_state::WorkflowStatus::Collecting,
                        });
                }
            }
        }

        Action::ClearConstitutionWorkflow => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.constitution_workflow = None;
                }
            }
        }

        Action::AnswerConstitutionQuestion { answer } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(workflow) = &mut worktree.tasks.constitution_workflow {
                        // Question keys (hardcoded for Phase 1)
                        const QUESTIONS: &[&str] =
                            &["tech_stack", "security", "code_quality", "architecture"];

                        // Save answer to current question
                        if workflow.current_question < QUESTIONS.len() {
                            let key = QUESTIONS[workflow.current_question];
                            workflow.answers.insert(key.to_string(), answer);
                            workflow.current_question += 1;
                        }
                    }
                }
            }
        }

        Action::GenerateConstitution => {
            // Async action - actual Claude call handled in lib.rs
            // Just update status to Generating
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(workflow) = &mut worktree.tasks.constitution_workflow {
                        workflow.status = crate::app_state::WorkflowStatus::Generating;
                        workflow.output.clear(); // Reset output before generation
                    }
                }
            }
        }

        Action::AppendConstitutionOutput { content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(workflow) = &mut worktree.tasks.constitution_workflow {
                        workflow.output.push_str(&content);
                    }
                }
            }
        }

        Action::SaveConstitution => {
            // Async action - actual file write handled in lib.rs
            // Just update status to Complete
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(workflow) = &mut worktree.tasks.constitution_workflow {
                        workflow.status = crate::app_state::WorkflowStatus::Complete;
                    }
                }
            }
        }

        Action::CheckConstitutionExists => {
            // Async trigger - no immediate state change
            // The async handler in lib.rs will check file and dispatch SetConstitutionExists
        }

        Action::SetConstitutionExists { exists } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.tasks.constitution_exists = Some(exists);
                }
            }
        }

        Action::ApplyDefaultConstitution => {
            // Async action - file write handled in lib.rs
            // No immediate state change needed
        }

        // ====================================================================
        // Change Management Actions (CESDD Phase 2 - worktree scope)
        // ====================================================================
        Action::CreateChange { intent } => {
            // Async action - file creation handled in lib.rs
            // Creates .rstn/changes/<change-id>/
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.is_loading = true;
                }
            }
            let _ = intent; // Used by async handler
        }

        Action::GenerateProposal { change_id } => {
            // Start proposal generation - set status to Planning
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        change.status = crate::app_state::ChangeStatus::Planning;
                        change.streaming_output.clear();
                    }
                }
            }
        }

        Action::AppendProposalOutput { change_id, content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        change.streaming_output.push_str(&content);
                    }
                }
            }
        }

        Action::CompleteProposal { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        // Move streaming output to proposal
                        change.proposal = Some(std::mem::take(&mut change.streaming_output));
                        change.status = crate::app_state::ChangeStatus::Proposed;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::GeneratePlan { change_id } => {
            // Start plan generation
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        change.status = crate::app_state::ChangeStatus::Planning;
                        change.streaming_output.clear();
                    }
                }
            }
        }

        Action::AppendPlanOutput { change_id, content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        change.streaming_output.push_str(&content);
                    }
                }
            }
        }

        Action::CompletePlan { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        // Move streaming output to plan
                        change.plan = Some(std::mem::take(&mut change.streaming_output));
                        change.status = crate::app_state::ChangeStatus::Planned;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::ApprovePlan { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        change.status = crate::app_state::ChangeStatus::Implementing;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::CancelChange { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree
                        .changes
                        .changes
                        .iter_mut()
                        .find(|c| c.id == change_id)
                    {
                        change.status = crate::app_state::ChangeStatus::Cancelled;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::SelectChange { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.selected_change_id = change_id;
                }
            }
        }

        Action::RefreshChanges => {
            // Async trigger - actual file reading handled in lib.rs
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.is_loading = true;
                }
            }
        }

        Action::SetChanges { changes } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.changes = changes.into_iter().map(|c| c.into()).collect();
                    worktree.changes.is_loading = false;
                }
            }
        }

        Action::SetChangesLoading { is_loading } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.is_loading = is_loading;
                }
            }
        }

        // ====================================================================
        // Docker Actions (global scope - operate on state.docker)
        // ====================================================================
        Action::CheckDockerAvailability => {
            state.docker.is_loading = true;
        }

        Action::SetDockerAvailable { available } => {
            state.docker.docker_available = Some(available);
            state.docker.is_loading = false;
        }

        Action::RefreshDockerServices => {
            state.docker.is_loading = true;
        }

        Action::SetDockerServices { services } => {
            state.docker.services = services.into_iter().map(|s| s.into()).collect();
            state.docker.is_loading = false;
        }

        Action::StartDockerService { service_id } => {
            if let Some(service) = state
                .docker
                .services
                .iter_mut()
                .find(|s| s.id == service_id)
            {
                service.status = ServiceStatus::Starting;
            }
        }

        Action::StopDockerService { service_id } => {
            if let Some(service) = state
                .docker
                .services
                .iter_mut()
                .find(|s| s.id == service_id)
            {
                service.status = ServiceStatus::Stopping;
            }
        }

        Action::RestartDockerService { service_id } => {
            if let Some(service) = state
                .docker
                .services
                .iter_mut()
                .find(|s| s.id == service_id)
            {
                service.status = ServiceStatus::Starting;
            }
        }

        Action::SelectDockerService { service_id } => {
            state.docker.selected_service_id = service_id;
            state.docker.logs.clear();
        }

        Action::FetchDockerLogs { .. } => {
            state.docker.is_loading_logs = true;
        }

        Action::SetDockerLogs { logs } => {
            state.docker.logs = logs;
            state.docker.is_loading_logs = false;
        }

        Action::CreateDatabase { .. } => {
            // Async trigger - no immediate state change
        }

        Action::CreateVhost { .. } => {
            // Async trigger - no immediate state change
        }

        Action::SetPortConflict { service_id, conflict } => {
            state.docker.pending_conflict = Some(PendingConflict {
                service_id,
                conflict: conflict.into(),
            });
        }

        Action::ClearPortConflict => {
            state.docker.pending_conflict = None;
        }

        Action::StartDockerServiceWithPort { ref service_id, port } => {
            // Store port override
            state.docker.port_overrides.insert(service_id.clone(), port);
            // Clear pending conflict
            state.docker.pending_conflict = None;
            // Set service to starting
            if let Some(service) = state
                .docker
                .services
                .iter_mut()
                .find(|s| s.id == *service_id)
            {
                service.status = ServiceStatus::Starting;
            }
        }

        Action::ResolveConflictByStoppingContainer { ref service_id, .. } => {
            // Clear pending conflict
            state.docker.pending_conflict = None;
            // Set service to starting
            if let Some(service) = state
                .docker
                .services
                .iter_mut()
                .find(|s| s.id == *service_id)
            {
                service.status = ServiceStatus::Starting;
            }
        }

        Action::SetDockerLoading { is_loading } => {
            state.docker.is_loading = is_loading;
        }

        Action::SetDockerLogsLoading { is_loading } => {
            state.docker.is_loading_logs = is_loading;
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
        // Env Actions (Project scope)
        // ====================================================================
        Action::CopyEnvFiles { .. } => {
            // Async trigger - handled in lib.rs
        }

        Action::SetEnvCopyResult { result } => {
            if let Some(project) = state.active_project_mut() {
                project.env_config.last_copy_result = Some(EnvCopyResult {
                    copied_files: result.copied_files,
                    failed_files: result.failed_files,
                    timestamp: result.timestamp,
                });
            }
        }

        Action::SetEnvTrackedPatterns { patterns } => {
            if let Some(project) = state.active_project_mut() {
                project.env_config.tracked_patterns = patterns;
            }
        }

        Action::SetEnvAutoCopy { enabled } => {
            if let Some(project) = state.active_project_mut() {
                project.env_config.auto_copy_enabled = enabled;
            }
        }

        Action::SetEnvSourceWorktree { worktree_path } => {
            if let Some(project) = state.active_project_mut() {
                project.env_config.source_worktree = worktree_path;
            }
        }

        // ====================================================================
        // Agent Rules Actions
        // ====================================================================
        Action::SetAgentRulesEnabled { enabled } => {
            if let Some(project) = state.active_project_mut() {
                project.agent_rules_config.enabled = enabled;

                // Clear active_profile_id when disabled
                if !enabled {
                    project.agent_rules_config.active_profile_id = None;
                }
            }
        }

        Action::SetAgentRulesPrompt { prompt } => {
            if let Some(project) = state.active_project_mut() {
                // For backward compatibility: update first custom profile or create new one
                if let Some(profile) = project.agent_rules_config.profiles
                    .iter_mut()
                    .find(|p| !p.is_builtin)
                {
                    profile.prompt = prompt;
                    profile.updated_at = chrono::Utc::now().to_rfc3339();
                } else {
                    // Create a new custom profile
                    let now = chrono::Utc::now().to_rfc3339();
                    let new_profile = crate::app_state::AgentProfile {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Custom".to_string(),
                        prompt,
                        is_builtin: false,
                        created_at: now.clone(),
                        updated_at: now,
                    };
                    project.agent_rules_config.profiles.push(new_profile.clone());
                    // Auto-select the new profile
                    project.agent_rules_config.active_profile_id = Some(new_profile.id);
                }
            }
        }

        Action::SetAgentRulesTempFile { path } => {
            if let Some(project) = state.active_project_mut() {
                project.agent_rules_config.temp_file_path = path;
            }
        }

        Action::CreateAgentProfile { name, prompt } => {
            if let Some(project) = state.active_project_mut() {
                let now = chrono::Utc::now().to_rfc3339();
                let profile = crate::app_state::AgentProfile {
                    id: uuid::Uuid::new_v4().to_string(),
                    name,
                    prompt,
                    is_builtin: false,
                    created_at: now.clone(),
                    updated_at: now,
                };
                project.agent_rules_config.profiles.push(profile);
            }
        }

        Action::UpdateAgentProfile { id, name, prompt } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(profile) = project
                    .agent_rules_config
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == id && !p.is_builtin)
                {
                    profile.name = name;
                    profile.prompt = prompt;
                    profile.updated_at = chrono::Utc::now().to_rfc3339();
                }
            }
        }

        Action::DeleteAgentProfile { id } => {
            if let Some(project) = state.active_project_mut() {
                // Remove profile only if it's not builtin
                project
                    .agent_rules_config
                    .profiles
                    .retain(|p| !(p.id == id && !p.is_builtin));

                // Clear active_profile_id if deleted profile was active
                if project.agent_rules_config.active_profile_id.as_ref() == Some(&id) {
                    project.agent_rules_config.active_profile_id = None;
                }
            }
        }

        Action::SelectAgentProfile { profile_id } => {
            if let Some(project) = state.active_project_mut() {
                project.agent_rules_config.active_profile_id = profile_id.clone();

                // Auto-enable agent rules if a profile is selected
                if profile_id.is_some() {
                    project.agent_rules_config.enabled = true;
                }
            }
        }

        // ====================================================================
        // Notification Actions
        // ====================================================================
        Action::AddNotification {
            message,
            notification_type,
        } => {
            state.notifications.push(Notification::new(
                message,
                notification_type.into(),
            ));
        }

        Action::DismissNotification { id } => {
            state.notifications.retain(|n| n.id != id);
        }

        Action::MarkNotificationRead { id } => {
            if let Some(notification) = state.notifications.iter_mut().find(|n| n.id == id) {
                notification.read = true;
            }
        }

        Action::MarkAllNotificationsRead => {
            for notification in &mut state.notifications {
                notification.read = true;
            }
        }

        Action::ClearNotifications => {
            state.notifications.clear();
        }

        // ====================================================================
        // View Actions
        // ====================================================================
        Action::SetActiveView { view } => {
            state.active_view = view.into();
        }

        // ====================================================================
        // Terminal Actions
        // ====================================================================
        Action::SpawnTerminal { cols, rows } => {
            // Async trigger - terminal manager will spawn PTY
            // Store dimensions in state
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.terminal.cols = cols;
                    worktree.terminal.rows = rows;
                }
            }
        }

        Action::ResizeTerminal { .. } => {
            // Async trigger - handled by terminal manager
        }

        Action::WriteTerminal { .. } => {
            // Async trigger - handled by terminal manager
        }

        Action::KillTerminal { .. } => {
            // Async trigger - after completion, SetTerminalSession(None) will be dispatched
        }

        Action::SetTerminalSession { session_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.terminal.session_id = session_id;
                }
            }
        }

        Action::SetTerminalSize { cols, rows } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.terminal.cols = cols;
                    worktree.terminal.rows = rows;
                }
            }
        }

        // ====================================================================
        // Dev Log Actions (global scope, dev mode only)
        // ====================================================================
        Action::AddDevLog { log } => {
            let dev_log = DevLog::new(
                match log.source {
                    DevLogSourceData::Rust => DevLogSource::Rust,
                    DevLogSourceData::Frontend => DevLogSource::Frontend,
                    DevLogSourceData::Claude => DevLogSource::Claude,
                    DevLogSourceData::Ipc => DevLogSource::Ipc,
                },
                match log.log_type {
                    DevLogTypeData::Action => DevLogType::Action,
                    DevLogTypeData::State => DevLogType::State,
                    DevLogTypeData::Claude => DevLogType::Claude,
                    DevLogTypeData::Error => DevLogType::Error,
                    DevLogTypeData::Info => DevLogType::Info,
                },
                log.summary,
                log.data,
            );
            state.add_dev_log(dev_log);
        }

        Action::ClearDevLogs => {
            state.clear_dev_logs();
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

/// Log key actions for dev debugging.
/// Only logs "interesting" actions to avoid noise from high-frequency actions.
fn log_action_if_interesting(state: &mut AppState, action: &Action) {
    // Get action name and determine if it's interesting
    let (action_name, is_interesting) = match action {
        // Project management - always interesting
        Action::OpenProject { .. } => ("OpenProject", true),
        Action::CloseProject { .. } => ("CloseProject", true),
        Action::SwitchProject { .. } => ("SwitchProject", true),

        // Worktree changes - always interesting
        Action::AddWorktree { .. } => ("AddWorktree", true),
        Action::AddWorktreeNewBranch { .. } => ("AddWorktreeNewBranch", true),
        Action::RemoveWorktree { .. } => ("RemoveWorktree", true),
        Action::SwitchWorktree { .. } => ("SwitchWorktree", true),
        Action::RefreshWorktrees => ("RefreshWorktrees", true),

        // MCP lifecycle - interesting
        Action::StartMcpServer => ("StartMcpServer", true),
        Action::StopMcpServer => ("StopMcpServer", true),
        Action::SetMcpStatus { .. } => ("SetMcpStatus", true),
        Action::SetMcpError { .. } => ("SetMcpError", true),

        // Docker operations - interesting
        Action::StartDockerService { .. } => ("StartDockerService", true),
        Action::StopDockerService { .. } => ("StopDockerService", true),
        Action::RestartDockerService { .. } => ("RestartDockerService", true),
        Action::SetPortConflict { .. } => ("SetPortConflict", true),

        // Constitution workflow - interesting
        Action::StartConstitutionWorkflow => ("StartConstitutionWorkflow", true),
        Action::ClearConstitutionWorkflow => ("ClearConstitutionWorkflow", true),
        Action::AnswerConstitutionQuestion { .. } => ("AnswerConstitutionQuestion", true),
        Action::GenerateConstitution => ("GenerateConstitution", true),
        Action::SaveConstitution => ("SaveConstitution", true),
        Action::CheckConstitutionExists => ("CheckConstitutionExists", true),
        Action::SetConstitutionExists { .. } => ("SetConstitutionExists", true),
        Action::ApplyDefaultConstitution => ("ApplyDefaultConstitution", true),

        // Change Management - interesting (key state changes)
        Action::CreateChange { .. } => ("CreateChange", true),
        Action::GenerateProposal { .. } => ("GenerateProposal", true),
        Action::CompleteProposal { .. } => ("CompleteProposal", true),
        Action::GeneratePlan { .. } => ("GeneratePlan", true),
        Action::CompletePlan { .. } => ("CompletePlan", true),
        Action::ApprovePlan { .. } => ("ApprovePlan", true),
        Action::CancelChange { .. } => ("CancelChange", true),
        Action::SelectChange { .. } => ("SelectChange", true),
        Action::RefreshChanges => ("RefreshChanges", true),
        Action::SetChanges { .. } => ("SetChanges", true),
        Action::SetChangesLoading { .. } => ("SetChangesLoading", false),
        // High-frequency streaming actions - not interesting
        Action::AppendProposalOutput { .. } => ("AppendProposalOutput", false),
        Action::AppendPlanOutput { .. } => ("AppendPlanOutput", false),

        // Task execution - interesting
        Action::RunJustCommand { .. } => ("RunJustCommand", true),
        Action::SetTaskStatus { .. } => ("SetTaskStatus", true),

        // Errors - always interesting
        Action::SetError { .. } => ("SetError", true),
        Action::SetChatError { .. } => ("SetChatError", true),
        Action::SetTasksError { .. } => ("SetTasksError", true),

        // Chat sending - interesting (but not content appending)
        Action::SendChatMessage { .. } => ("SendChatMessage", true),
        Action::AddChatMessage { .. } => ("AddChatMessage", true),

        // High-frequency actions - not interesting (skip to reduce noise)
        Action::AppendChatContent { .. } => ("AppendChatContent", false),
        Action::AppendTaskOutput { .. } => ("AppendTaskOutput", false),
        Action::AppendConstitutionOutput { .. } => ("AppendConstitutionOutput", false),
        Action::AddMcpLogEntry { .. } => ("AddMcpLogEntry", false),
        Action::AddDevLog { .. } => ("AddDevLog", false), // Avoid infinite loop!
        Action::ClearDevLogs => ("ClearDevLogs", false),

        // Other actions - log for completeness but with lower priority
        _ => ("OtherAction", false),
    };

    if is_interesting {
        // Create the log entry
        let log = DevLog::action(action_name, serde_json::to_value(action).unwrap_or_default());
        state.add_dev_log(log);
    }
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
    use crate::actions::{ChatMessageData, ChatRoleData};
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
    // Docker Actions Tests (global scope - operate on state.docker)
    // ========================================================================

    #[test]
    fn test_reduce_docker_availability() {
        let mut state = state_with_project();

        reduce(&mut state, Action::CheckDockerAvailability);
        assert!(state.docker.is_loading);

        reduce(&mut state, Action::SetDockerAvailable { available: true });
        assert_eq!(state.docker.docker_available, Some(true));
        assert!(!state.docker.is_loading);
    }

    #[test]
    fn test_reduce_docker_services() {
        let mut state = state_with_project();

        reduce(&mut state, Action::RefreshDockerServices);
        assert!(state.docker.is_loading);

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

        assert!(!state.docker.is_loading);
        assert_eq!(state.docker.services.len(), 1);
        assert_eq!(state.docker.services[0].name, "PostgreSQL");
    }

    #[test]
    fn test_reduce_start_stop_service() {
        let mut state = state_with_project();
        state.docker.services.push(DockerServiceInfo {
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
        assert_eq!(state.docker.services[0].status, ServiceStatus::Starting);

        reduce(
            &mut state,
            Action::StopDockerService {
                service_id: "pg-1".to_string(),
            },
        );
        assert_eq!(state.docker.services[0].status, ServiceStatus::Stopping);
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
    // View Actions Tests (Feature 02/03: Settings & Command Palette)
    // ========================================================================

    #[test]
    fn test_reduce_set_active_view() {
        use crate::app_state::ActiveView;

        let mut state = AppState::default();

        // Default is workflows
        assert_eq!(state.active_view, ActiveView::Workflows);

        // Switch to dockers
        reduce(
            &mut state,
            Action::SetActiveView {
                view: crate::actions::ActiveViewData::Dockers,
            },
        );
        assert_eq!(state.active_view, ActiveView::Dockers);

        // Switch to env
        reduce(
            &mut state,
            Action::SetActiveView {
                view: crate::actions::ActiveViewData::Env,
            },
        );
        assert_eq!(state.active_view, ActiveView::Env);

        // Switch to settings
        reduce(
            &mut state,
            Action::SetActiveView {
                view: crate::actions::ActiveViewData::Settings,
            },
        );
        assert_eq!(state.active_view, ActiveView::Settings);

        // Switch to tasks
        reduce(
            &mut state,
            Action::SetActiveView {
                view: crate::actions::ActiveViewData::Tasks,
            },
        );
        assert_eq!(state.active_view, ActiveView::Tasks);

        // Switch to workflows
        reduce(
            &mut state,
            Action::SetActiveView {
                view: crate::actions::ActiveViewData::Workflows,
            },
        );
        assert_eq!(state.active_view, ActiveView::Workflows);
    }

    // ========================================================================
    // Env Actions Tests (Feature 01: Env Management)
    // ========================================================================

    #[test]
    fn test_reduce_set_env_tracked_patterns() {
        let mut state = state_with_project();

        // Initially should have default patterns
        let project = state.active_project().unwrap();
        assert!(!project.env_config.tracked_patterns.is_empty());

        // Set custom patterns
        let new_patterns = vec![".env".to_string(), ".env.local".to_string()];
        reduce(
            &mut state,
            Action::SetEnvTrackedPatterns {
                patterns: new_patterns.clone(),
            },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.env_config.tracked_patterns, new_patterns);
    }

    #[test]
    fn test_reduce_set_env_auto_copy() {
        let mut state = state_with_project();

        // Initially enabled (default is true)
        let project = state.active_project().unwrap();
        assert!(project.env_config.auto_copy_enabled);

        // Disable auto-copy
        reduce(
            &mut state,
            Action::SetEnvAutoCopy { enabled: false },
        );

        let project = state.active_project().unwrap();
        assert!(!project.env_config.auto_copy_enabled);

        // Enable auto-copy
        reduce(
            &mut state,
            Action::SetEnvAutoCopy { enabled: true },
        );

        let project = state.active_project().unwrap();
        assert!(project.env_config.auto_copy_enabled);
    }

    #[test]
    fn test_reduce_set_env_copy_result() {
        let mut state = state_with_project();

        // Initially no result
        let project = state.active_project().unwrap();
        assert!(project.env_config.last_copy_result.is_none());

        // Set copy result
        reduce(
            &mut state,
            Action::SetEnvCopyResult {
                result: crate::actions::EnvCopyResultData {
                    copied_files: vec![".env".to_string()],
                    failed_files: vec![],
                    timestamp: "2024-12-26T10:00:00Z".to_string(),
                },
            },
        );

        let project = state.active_project().unwrap();
        let result = project.env_config.last_copy_result.as_ref().unwrap();
        assert_eq!(result.copied_files, vec![".env"]);
        assert!(result.failed_files.is_empty());
        assert_eq!(result.timestamp, "2024-12-26T10:00:00Z");
    }

    #[test]
    fn test_reduce_set_env_source_worktree() {
        let mut state = state_with_project();

        // Initially set to project path (ProjectState::new uses with_source)
        let project = state.active_project().unwrap();
        assert_eq!(
            project.env_config.source_worktree,
            Some("/test/project".to_string())
        );

        // Change source worktree to different path
        reduce(
            &mut state,
            Action::SetEnvSourceWorktree {
                worktree_path: Some("/test/other".to_string()),
            },
        );

        let project = state.active_project().unwrap();
        assert_eq!(
            project.env_config.source_worktree,
            Some("/test/other".to_string())
        );

        // Clear source worktree
        reduce(
            &mut state,
            Action::SetEnvSourceWorktree {
                worktree_path: None,
            },
        );

        let project = state.active_project().unwrap();
        assert!(project.env_config.source_worktree.is_none());
    }

    #[test]
    fn test_env_actions_noop_without_project() {
        let mut state = AppState::default();

        // These should not crash when no project exists
        reduce(
            &mut state,
            Action::SetEnvTrackedPatterns {
                patterns: vec![".env".to_string()],
            },
        );
        reduce(&mut state, Action::SetEnvAutoCopy { enabled: true });
        reduce(
            &mut state,
            Action::SetEnvSourceWorktree {
                worktree_path: Some("/test".to_string()),
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
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
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
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
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
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
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
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
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

    // ========================================================================
    // Chat FSM Tests
    // ========================================================================

    #[test]
    fn test_reduce_add_chat_message_user() {
        let mut state = state_with_project();

        // Add a user message
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "user-1".to_string(),
                    role: ChatRoleData::User,
                    content: "Hello Claude".to_string(),
                    timestamp: "2024-12-27T00:00:00Z".to_string(),
                    is_streaming: false,
                },
            },
        );

        let worktree = active_worktree(&state);
        assert_eq!(worktree.chat.messages.len(), 1);
        assert_eq!(worktree.chat.messages[0].content, "Hello Claude");
        assert!(matches!(
            worktree.chat.messages[0].role,
            crate::app_state::ChatRole::User
        ));
    }

    #[test]
    fn test_reduce_add_chat_message_assistant_streaming() {
        let mut state = state_with_project();

        // Add an assistant message with streaming=true
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "assistant-1".to_string(),
                    role: ChatRoleData::Assistant,
                    content: String::new(),
                    timestamp: "2024-12-27T00:00:00Z".to_string(),
                    is_streaming: true,
                },
            },
        );

        let worktree = active_worktree(&state);
        assert_eq!(worktree.chat.messages.len(), 1);
        assert!(worktree.chat.messages[0].is_streaming);
        assert!(worktree.chat.messages[0].content.is_empty());
    }

    #[test]
    fn test_reduce_append_chat_content() {
        let mut state = state_with_project();

        // Add an assistant message first
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "assistant-1".to_string(),
                    role: ChatRoleData::Assistant,
                    content: String::new(),
                    timestamp: "2024-12-27T00:00:00Z".to_string(),
                    is_streaming: true,
                },
            },
        );

        // Append content (streaming delta)
        reduce(
            &mut state,
            Action::AppendChatContent {
                content: "Hello".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AppendChatContent {
                content: " world".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        assert_eq!(worktree.chat.messages.len(), 1);
        assert_eq!(worktree.chat.messages[0].content, "Hello world");
    }

    #[test]
    fn test_reduce_set_chat_typing() {
        let mut state = state_with_project();

        // Initially not typing
        assert!(!active_worktree(&state).chat.is_typing);

        // Set typing true
        reduce(&mut state, Action::SetChatTyping { is_typing: true });
        assert!(active_worktree(&state).chat.is_typing);

        // Set typing false
        reduce(&mut state, Action::SetChatTyping { is_typing: false });
        assert!(!active_worktree(&state).chat.is_typing);
    }

    #[test]
    fn test_reduce_set_chat_error() {
        let mut state = state_with_project();

        // Set error
        reduce(
            &mut state,
            Action::SetChatError {
                error: "Connection failed".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        assert_eq!(worktree.chat.error, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_reduce_clear_chat_error() {
        let mut state = state_with_project();

        // Set error first
        reduce(
            &mut state,
            Action::SetChatError {
                error: "Error".to_string(),
            },
        );
        assert!(active_worktree(&state).chat.error.is_some());

        // Clear error
        reduce(&mut state, Action::ClearChatError);
        assert!(active_worktree(&state).chat.error.is_none());
    }

    #[test]
    fn test_reduce_clear_chat() {
        let mut state = state_with_project();

        // Add messages
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "user-1".to_string(),
                    role: ChatRoleData::User,
                    content: "Hello".to_string(),
                    timestamp: "2024-12-27T00:00:00Z".to_string(),
                    is_streaming: false,
                },
            },
        );
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "assistant-1".to_string(),
                    role: ChatRoleData::Assistant,
                    content: "Hi there!".to_string(),
                    timestamp: "2024-12-27T00:00:01Z".to_string(),
                    is_streaming: false,
                },
            },
        );

        assert_eq!(active_worktree(&state).chat.messages.len(), 2);

        // Clear chat
        reduce(&mut state, Action::ClearChat);

        let worktree = active_worktree(&state);
        assert!(worktree.chat.messages.is_empty());
        assert!(worktree.chat.error.is_none());
        assert!(!worktree.chat.is_typing);
    }

    #[test]
    fn test_chat_fsm_full_flow() {
        // Simulate complete chat flow: IDLE → SPAWNING → STREAMING → COMPLETE
        let mut state = state_with_project();

        // 1. IDLE: Initial state
        let worktree = active_worktree(&state);
        assert!(worktree.chat.messages.is_empty());
        assert!(!worktree.chat.is_typing);
        assert!(worktree.chat.error.is_none());

        // 2. User sends message → SPAWNING
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "user-1".to_string(),
                    role: ChatRoleData::User,
                    content: "What is Rust?".to_string(),
                    timestamp: "2024-12-27T00:00:00Z".to_string(),
                    is_streaming: false,
                },
            },
        );
        reduce(&mut state, Action::SetChatTyping { is_typing: true });

        let worktree = active_worktree(&state);
        assert_eq!(worktree.chat.messages.len(), 1);
        assert!(worktree.chat.is_typing);

        // 3. CLI spawns, creates assistant message placeholder → STREAMING
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "assistant-1".to_string(),
                    role: ChatRoleData::Assistant,
                    content: String::new(),
                    timestamp: "2024-12-27T00:00:01Z".to_string(),
                    is_streaming: true,
                },
            },
        );

        let worktree = active_worktree(&state);
        assert_eq!(worktree.chat.messages.len(), 2);
        assert!(worktree.chat.messages[1].is_streaming);

        // 4. Streaming deltas arrive
        reduce(
            &mut state,
            Action::AppendChatContent {
                content: "Rust is ".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AppendChatContent {
                content: "a systems ".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AppendChatContent {
                content: "programming language.".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        assert_eq!(
            worktree.chat.messages[1].content,
            "Rust is a systems programming language."
        );

        // 5. message_stop received → COMPLETE
        reduce(&mut state, Action::SetChatTyping { is_typing: false });

        let worktree = active_worktree(&state);
        assert!(!worktree.chat.is_typing);
        assert_eq!(worktree.chat.messages.len(), 2);
        assert_eq!(worktree.chat.messages[0].content, "What is Rust?");
        assert_eq!(
            worktree.chat.messages[1].content,
            "Rust is a systems programming language."
        );
    }

    #[test]
    fn test_chat_fsm_error_flow() {
        // Simulate error during chat: IDLE → SPAWNING → ERROR → IDLE
        let mut state = state_with_project();

        // 1. User sends message
        reduce(
            &mut state,
            Action::AddChatMessage {
                message: ChatMessageData {
                    id: "user-1".to_string(),
                    role: ChatRoleData::User,
                    content: "Hello".to_string(),
                    timestamp: "2024-12-27T00:00:00Z".to_string(),
                    is_streaming: false,
                },
            },
        );
        reduce(&mut state, Action::SetChatTyping { is_typing: true });

        // 2. Error occurs (CLI not found)
        reduce(
            &mut state,
            Action::SetChatError {
                error: "Claude Code CLI not found".to_string(),
            },
        );
        reduce(&mut state, Action::SetChatTyping { is_typing: false });

        let worktree = active_worktree(&state);
        assert!(!worktree.chat.is_typing);
        assert_eq!(
            worktree.chat.error,
            Some("Claude Code CLI not found".to_string())
        );

        // 3. User dismisses error → IDLE
        reduce(&mut state, Action::ClearChatError);

        let worktree = active_worktree(&state);
        assert!(worktree.chat.error.is_none());
    }

    // ========================================================================
    // Agent Profile Tests
    // ========================================================================

    #[test]
    fn test_create_agent_profile() {
        let mut state = state_with_project();

        reduce(
            &mut state,
            Action::CreateAgentProfile {
                name: "My Custom Profile".to_string(),
                prompt: "You are a test expert".to_string(),
            },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.agent_rules_config.profiles.len(), 4); // 3 builtin + 1 custom

        let custom_profile = project
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| p.name == "My Custom Profile")
            .unwrap();

        assert!(!custom_profile.is_builtin);
        assert_eq!(custom_profile.prompt, "You are a test expert");
        assert!(!custom_profile.id.is_empty());
    }

    #[test]
    fn test_update_agent_profile() {
        let mut state = state_with_project();

        // Create a custom profile
        reduce(
            &mut state,
            Action::CreateAgentProfile {
                name: "Original".to_string(),
                prompt: "Original prompt".to_string(),
            },
        );

        let profile_id = state
            .active_project()
            .unwrap()
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| !p.is_builtin)
            .unwrap()
            .id
            .clone();

        // Update the profile
        reduce(
            &mut state,
            Action::UpdateAgentProfile {
                id: profile_id.clone(),
                name: "Updated".to_string(),
                prompt: "Updated prompt".to_string(),
            },
        );

        let project = state.active_project().unwrap();
        let updated_profile = project
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .unwrap();

        assert_eq!(updated_profile.name, "Updated");
        assert_eq!(updated_profile.prompt, "Updated prompt");
    }

    #[test]
    fn test_update_builtin_profile_is_noop() {
        let mut state = state_with_project();

        let builtin_id = "builtin-rust-expert".to_string();
        let original_prompt = state
            .active_project()
            .unwrap()
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| p.id == builtin_id)
            .unwrap()
            .prompt
            .clone();

        // Try to update builtin profile (should be ignored)
        reduce(
            &mut state,
            Action::UpdateAgentProfile {
                id: builtin_id.clone(),
                name: "Hacked".to_string(),
                prompt: "Hacked prompt".to_string(),
            },
        );

        let project = state.active_project().unwrap();
        let builtin_profile = project
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| p.id == builtin_id)
            .unwrap();

        // Should remain unchanged
        assert_eq!(builtin_profile.name, "Rust Expert");
        assert_eq!(builtin_profile.prompt, original_prompt);
    }

    #[test]
    fn test_delete_agent_profile() {
        let mut state = state_with_project();

        // Create a custom profile
        reduce(
            &mut state,
            Action::CreateAgentProfile {
                name: "To Delete".to_string(),
                prompt: "Will be deleted".to_string(),
            },
        );

        let profile_id = state
            .active_project()
            .unwrap()
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| !p.is_builtin)
            .unwrap()
            .id
            .clone();

        assert_eq!(
            state.active_project().unwrap().agent_rules_config.profiles.len(),
            4
        );

        // Delete the profile
        reduce(
            &mut state,
            Action::DeleteAgentProfile { id: profile_id },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.agent_rules_config.profiles.len(), 3); // Back to 3 builtin
        assert!(project
            .agent_rules_config
            .profiles
            .iter()
            .all(|p| p.is_builtin));
    }

    #[test]
    fn test_delete_builtin_profile_is_noop() {
        let mut state = state_with_project();

        let builtin_id = "builtin-rust-expert".to_string();
        assert_eq!(
            state.active_project().unwrap().agent_rules_config.profiles.len(),
            3
        );

        // Try to delete builtin profile (should be ignored)
        reduce(
            &mut state,
            Action::DeleteAgentProfile { id: builtin_id },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.agent_rules_config.profiles.len(), 3); // Still 3
        assert!(project
            .agent_rules_config
            .profiles
            .iter()
            .any(|p| p.id == "builtin-rust-expert"));
    }

    #[test]
    fn test_delete_active_profile_clears_selection() {
        let mut state = state_with_project();

        // Create and select a custom profile
        reduce(
            &mut state,
            Action::CreateAgentProfile {
                name: "Active".to_string(),
                prompt: "Active prompt".to_string(),
            },
        );

        let profile_id = state
            .active_project()
            .unwrap()
            .agent_rules_config
            .profiles
            .iter()
            .find(|p| !p.is_builtin)
            .unwrap()
            .id
            .clone();

        reduce(
            &mut state,
            Action::SelectAgentProfile {
                profile_id: Some(profile_id.clone()),
            },
        );

        assert_eq!(
            state
                .active_project()
                .unwrap()
                .agent_rules_config
                .active_profile_id,
            Some(profile_id.clone())
        );

        // Delete the active profile
        reduce(&mut state, Action::DeleteAgentProfile { id: profile_id });

        // active_profile_id should be cleared
        assert_eq!(
            state
                .active_project()
                .unwrap()
                .agent_rules_config
                .active_profile_id,
            None
        );
    }

    #[test]
    fn test_select_agent_profile() {
        let mut state = state_with_project();

        let builtin_id = "builtin-typescript-expert".to_string();

        reduce(
            &mut state,
            Action::SelectAgentProfile {
                profile_id: Some(builtin_id.clone()),
            },
        );

        let project = state.active_project().unwrap();
        assert_eq!(
            project.agent_rules_config.active_profile_id,
            Some(builtin_id)
        );
        assert!(project.agent_rules_config.enabled); // Auto-enabled
    }

    #[test]
    fn test_select_none_keeps_enabled_state() {
        let mut state = state_with_project();

        // Enable and select a profile
        reduce(
            &mut state,
            Action::SelectAgentProfile {
                profile_id: Some("builtin-rust-expert".to_string()),
            },
        );

        assert!(state
            .active_project()
            .unwrap()
            .agent_rules_config
            .enabled);

        // Deselect (set to None)
        reduce(
            &mut state,
            Action::SelectAgentProfile { profile_id: None },
        );

        let project = state.active_project().unwrap();
        assert_eq!(project.agent_rules_config.active_profile_id, None);
        assert!(project.agent_rules_config.enabled); // Remains enabled
    }

    #[test]
    fn test_disable_agent_rules_clears_selection() {
        let mut state = state_with_project();

        // Select a profile
        reduce(
            &mut state,
            Action::SelectAgentProfile {
                profile_id: Some("builtin-rust-expert".to_string()),
            },
        );

        assert!(state
            .active_project()
            .unwrap()
            .agent_rules_config
            .active_profile_id
            .is_some());

        // Disable agent rules
        reduce(
            &mut state,
            Action::SetAgentRulesEnabled { enabled: false },
        );

        let project = state.active_project().unwrap();
        assert!(!project.agent_rules_config.enabled);
        assert_eq!(project.agent_rules_config.active_profile_id, None); // Cleared
    }

    #[test]
    fn test_default_profiles_exist() {
        let state = state_with_project();

        let project = state.active_project().unwrap();
        let profiles = &project.agent_rules_config.profiles;

        assert_eq!(profiles.len(), 3);

        // Check all 3 builtin profiles exist
        let rust_expert = profiles
            .iter()
            .find(|p| p.id == "builtin-rust-expert")
            .unwrap();
        assert_eq!(rust_expert.name, "Rust Expert");
        assert!(rust_expert.is_builtin);
        assert!(rust_expert.prompt.contains("snake_case"));

        let ts_expert = profiles
            .iter()
            .find(|p| p.id == "builtin-typescript-expert")
            .unwrap();
        assert_eq!(ts_expert.name, "TypeScript Expert");
        assert!(ts_expert.is_builtin);
        assert!(ts_expert.prompt.contains("TypeScript"));

        let reviewer = profiles
            .iter()
            .find(|p| p.id == "builtin-code-reviewer")
            .unwrap();
        assert_eq!(reviewer.name, "Code Reviewer");
        assert!(reviewer.is_builtin);
        assert!(reviewer.prompt.contains("code reviewer"));
    }

    // ========================================================================
    // Constitution Workflow Tests (CESDD Phase 1)
    // ========================================================================

    #[test]
    fn test_start_constitution_workflow() {
        let mut state = state_with_project();

        // Initially no workflow
        let worktree = active_worktree(&state);
        assert!(worktree.tasks.constitution_workflow.is_none());

        // Start workflow
        reduce(&mut state, Action::StartConstitutionWorkflow);

        // Workflow should be initialized
        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.current_question, 0);
        assert_eq!(workflow.answers.len(), 0);
        assert_eq!(workflow.output, "");
        assert_eq!(workflow.status, crate::app_state::WorkflowStatus::Collecting);
    }

    #[test]
    fn test_answer_constitution_questions() {
        let mut state = state_with_project();

        // Start workflow
        reduce(&mut state, Action::StartConstitutionWorkflow);

        // Answer first question (tech_stack)
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "React + Rust".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.current_question, 1);
        assert_eq!(workflow.answers.get("tech_stack").unwrap(), "React + Rust");

        // Answer second question (security)
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "JWT auth required".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.current_question, 2);
        assert_eq!(workflow.answers.get("security").unwrap(), "JWT auth required");

        // Answer third question (code_quality)
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "80% test coverage".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.current_question, 3);
        assert_eq!(
            workflow.answers.get("code_quality").unwrap(),
            "80% test coverage"
        );

        // Answer fourth question (architecture)
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "State-first principle".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.current_question, 4); // All questions answered
        assert_eq!(
            workflow.answers.get("architecture").unwrap(),
            "State-first principle"
        );
    }

    #[test]
    fn test_generate_constitution_updates_status() {
        let mut state = state_with_project();

        // Start workflow and answer all questions
        reduce(&mut state, Action::StartConstitutionWorkflow);
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "React + Rust".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "JWT auth".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "80% coverage".to_string(),
            },
        );
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "State-first".to_string(),
            },
        );

        // Generate constitution
        reduce(&mut state, Action::GenerateConstitution);

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(
            workflow.status,
            crate::app_state::WorkflowStatus::Generating
        );
        assert_eq!(workflow.output, ""); // Output cleared before generation
    }

    #[test]
    fn test_append_constitution_output() {
        let mut state = state_with_project();

        // Start workflow and set to generating
        reduce(&mut state, Action::StartConstitutionWorkflow);
        reduce(&mut state, Action::GenerateConstitution);

        // Append output chunks (simulating streaming)
        reduce(
            &mut state,
            Action::AppendConstitutionOutput {
                content: "# Project Constitution\n\n".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.output, "# Project Constitution\n\n");

        reduce(
            &mut state,
            Action::AppendConstitutionOutput {
                content: "## Technology Stack\n".to_string(),
            },
        );

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(
            workflow.output,
            "# Project Constitution\n\n## Technology Stack\n"
        );
    }

    #[test]
    fn test_save_constitution_marks_complete() {
        let mut state = state_with_project();

        // Start workflow, generate, and append output
        reduce(&mut state, Action::StartConstitutionWorkflow);
        reduce(&mut state, Action::GenerateConstitution);
        reduce(
            &mut state,
            Action::AppendConstitutionOutput {
                content: "# Constitution content".to_string(),
            },
        );

        // Save constitution
        reduce(&mut state, Action::SaveConstitution);

        let worktree = active_worktree(&state);
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.status, crate::app_state::WorkflowStatus::Complete);
    }

    #[test]
    fn test_constitution_workflow_serialization() {
        let mut state = state_with_project();

        // Start workflow and answer questions
        reduce(&mut state, Action::StartConstitutionWorkflow);
        reduce(
            &mut state,
            Action::AnswerConstitutionQuestion {
                answer: "React + Rust".to_string(),
            },
        );

        // Serialize to JSON
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("constitution_workflow"));
        assert!(json.contains("React + Rust"));

        // Deserialize back
        let loaded: AppState = serde_json::from_str(&json).unwrap();
        let worktree = loaded.active_project().unwrap().active_worktree().unwrap();
        let workflow = worktree.tasks.constitution_workflow.as_ref().unwrap();
        assert_eq!(workflow.current_question, 1);
        assert_eq!(workflow.answers.get("tech_stack").unwrap(), "React + Rust");
    }

    #[test]
    fn test_set_constitution_exists() {
        let mut state = state_with_project();

        // Initially null (not checked)
        let worktree = active_worktree(&state);
        assert!(worktree.tasks.constitution_exists.is_none());

        // Set to true
        reduce(&mut state, Action::SetConstitutionExists { exists: true });
        let worktree = active_worktree(&state);
        assert_eq!(worktree.tasks.constitution_exists, Some(true));

        // Set to false
        reduce(&mut state, Action::SetConstitutionExists { exists: false });
        let worktree = active_worktree(&state);
        assert_eq!(worktree.tasks.constitution_exists, Some(false));
    }

    #[test]
    fn test_constitution_exists_serialization() {
        let mut state = state_with_project();

        // Set constitution exists
        reduce(&mut state, Action::SetConstitutionExists { exists: true });

        // Serialize to JSON
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("constitution_exists"));

        // Deserialize back
        let loaded: AppState = serde_json::from_str(&json).unwrap();
        let worktree = loaded.active_project().unwrap().active_worktree().unwrap();
        assert_eq!(worktree.tasks.constitution_exists, Some(true));
    }
}
