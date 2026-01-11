//! State reducer - organized into submodules.

pub mod chat;
pub mod docker;
pub mod mcp;
pub mod notifications;
pub mod project;
pub mod tasks;
pub mod worktree;
pub mod terminal;
pub mod settings;
pub mod explorer;
pub mod dev_log;
pub mod file_viewer;
pub mod a2ui;
pub mod changes;
pub mod context;
pub mod constitution;
pub mod review_gate;
pub mod env;
pub mod conversions;

#[cfg(test)]
mod tests;

use crate::actions::Action;
use crate::app_state::{AppState, RecentProject};

/// Update recent_projects list when opening a project
pub fn update_recent_projects(state: &mut AppState, path: &str) {
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

/// Apply an action to the state.
pub fn reduce(state: &mut AppState, action: Action) {
    // Auto-log actions for dev debugging
    dev_log::log_action_if_interesting(state, &action);

    match action {
        Action::OpenProject { .. }
        | Action::CloseProject { .. }
        | Action::SwitchProject { .. }
        | Action::SetFeatureTab { .. } => {
            project::reduce(state, action);
        }

        Action::SwitchWorktree { .. }
        | Action::RefreshWorktrees
        | Action::SetWorktrees { .. }
        | Action::AddWorktree { .. }
        | Action::AddWorktreeNewBranch { .. }
        | Action::RemoveWorktree { .. }
        | Action::FetchBranches
        | Action::SetBranches { .. }
        | Action::SetBranchesLoading { .. } => {
            worktree::reduce(state, action);
        }

        Action::StartMcpServer
        | Action::StopMcpServer
        | Action::SetMcpStatus { .. }
        | Action::SetMcpPort { .. }
        | Action::SetMcpConfigPath { .. }
        | Action::SetMcpError { .. }
        | Action::AddMcpLogEntry { .. }
        | Action::ClearMcpLogs
        | Action::UpdateMcpTools { .. } => {
            mcp::reduce(state, action);
        }

        Action::SendChatMessage { .. }
        | Action::AddChatMessage { .. }
        | Action::AppendChatContent { .. }
        | Action::SetChatTyping { .. }
        | Action::SetChatError { .. }
        | Action::ClearChatError
        | Action::ClearChat => {
            chat::reduce(state, action);
        }

        Action::CheckDockerAvailability
        | Action::SetDockerAvailable { .. }
        | Action::RefreshDockerServices
        | Action::SetDockerServices { .. }
        | Action::StartDockerService { .. }
        | Action::StopDockerService { .. }
        | Action::RestartDockerService { .. }
        | Action::SelectDockerService { .. }
        | Action::FetchDockerLogs { .. }
        | Action::SetDockerLogs { .. }
        | Action::CreateDatabase { .. }
        | Action::CreateVhost { .. }
        | Action::SetDockerConnectionString { .. }
        | Action::SetPortConflict { .. }
        | Action::ClearPortConflict
        | Action::StartDockerServiceWithPort { .. }
        | Action::ResolveConflictByStoppingContainer { .. }
        | Action::SetDockerLoading { .. }
        | Action::SetDockerLogsLoading { .. } => {
            docker::reduce(state, action);
        }

        Action::LoadJustfileCommands { .. }
        | Action::RefreshJustfile
        | Action::SetJustfileCommands { .. }
        | Action::RunJustCommand { .. }
        | Action::SetTaskStatus { .. }
        | Action::SetActiveCommand { .. }
        | Action::AppendTaskOutput { .. }
        | Action::ClearTaskOutput
        | Action::SetTasksLoading { .. }
        | Action::SetTasksError { .. } => {
            tasks::reduce(state, action);
        }

        Action::AddNotification { .. }
        | Action::DismissNotification { .. }
        | Action::MarkNotificationRead { .. }
        | Action::MarkAllNotificationsRead
        | Action::ClearNotifications => {
            notifications::reduce(state, action);
        }

        Action::SetActiveView { .. } => {
            state.active_view = if let Action::SetActiveView { view } = action {
                view.into()
            } else {
                state.active_view
            };
        }

        Action::SpawnTerminal { .. }
        | Action::ResizeTerminal { .. }
        | Action::WriteTerminal { .. }
        | Action::KillTerminal { .. }
        | Action::SetTerminalSession { .. }
        | Action::SetTerminalSize { .. } => {
            terminal::reduce(state, action);
        }

        Action::SetTheme { .. }
        | Action::SetProjectPath { .. } => {
            settings::reduce(state, action);
        }

        Action::ExploreDir { .. }
        | Action::SetExplorerEntries { .. }
        | Action::SetFileComments { .. }
        | Action::NavigateBack
        | Action::NavigateForward
        | Action::NavigateUp
        | Action::SelectFile { .. }
        | Action::SetExplorerSort { .. }
        | Action::SetExplorerFilter { .. }
        | Action::CreateFile { .. }
        | Action::RenameFile { .. }
        | Action::DeleteFile { .. }
        | Action::RevealInOS { .. }
        | Action::AddFileComment { .. }
        | Action::DeleteFileComment { .. }
        | Action::OpenFileTab { .. }
        | Action::PinTab { .. }
        | Action::CloseTab { .. }
        | Action::SwitchTab { .. }
        | Action::ExpandDirectory { .. }
        | Action::CollapseDirectory { .. }
        | Action::SetDirectoryCache { .. } => {
            explorer::reduce(state, action);
        }

        Action::AddDevLog { .. }
        | Action::ClearDevLogs => {
            dev_log::reduce(state, action);
        }

        Action::ToggleLogPanel { .. }
        | Action::CloseLogPanel
        | Action::SetLogPanelWidth { .. } => {
            if let Action::ToggleLogPanel { panel_type } = action {
                let panel_type: crate::app_state::LogPanelType = panel_type.into();
                if state.ui_layout.active_panel == Some(panel_type) {
                    state.ui_layout.active_panel = None;
                    state.ui_layout.panel_expanded = false;
                } else {
                    state.ui_layout.active_panel = Some(panel_type);
                    state.ui_layout.panel_expanded = true;
                }
            } else if let Action::CloseLogPanel = action {
                state.ui_layout.active_panel = None;
                state.ui_layout.panel_expanded = false;
            } else if let Action::SetLogPanelWidth { width } = action {
                state.ui_layout.panel_width = width;
            }
        }

        Action::ReadFile { .. }
        | Action::SetFileContent { .. }
        | Action::SetFileLoading { .. }
        | Action::ReadBinaryFile { .. }
        | Action::SetBinaryFileContent { .. } => {
            file_viewer::reduce(state, action);
        }

        Action::SetA2UIPayload { .. } => {
            a2ui::reduce(state, action);
        }

        Action::CreateChange { .. }
        | Action::GenerateProposal { .. }
        | Action::AppendProposalOutput { .. }
        | Action::CompleteProposal { .. }
        | Action::GeneratePlan { .. }
        | Action::AppendPlanOutput { .. }
        | Action::CompletePlan { .. }
        | Action::ApprovePlan { .. }
        | Action::ExecutePlan { .. }
        | Action::AppendImplementationOutput { .. }
        | Action::CompleteImplementation { .. }
        | Action::FailImplementation { .. }
        | Action::CancelChange { .. }
        | Action::SelectChange { .. }
        | Action::RefreshChanges
        | Action::SetChanges { .. }
        | Action::SetChangesLoading { .. }
        | Action::AddContextFile { .. }
        | Action::RemoveContextFile { .. }
        | Action::ClearContextFiles { .. }
        | Action::StartProposalReview { .. }
        | Action::StartPlanReview { .. }
        | Action::SetChangeArchived { .. }
        | Action::ValidateContextFile { .. }
        | Action::SetContextValidationResult { .. } => {
            changes::reduce(state, action);
        }

        Action::LoadContext
        | Action::SetContext { .. }
        | Action::SetContextLoading { .. }
        | Action::InitializeContext
        | Action::RefreshContext
        | Action::UpdateContextFile { .. }
        | Action::CheckContextExists
        | Action::SetContextInitialized { .. }
        | Action::GenerateContext
        | Action::AppendGenerateContextOutput { .. }
        | Action::CompleteGenerateContext
        | Action::FailGenerateContext { .. }
        | Action::SyncContext { .. }
        | Action::AppendContextSyncOutput { .. }
        | Action::CompleteContextSync { .. }
        | Action::ArchiveChange { .. } => {
            context::reduce(state, action);
        }

        Action::StartConstitutionWorkflow
        | Action::ClearConstitutionWorkflow
        | Action::AnswerConstitutionQuestion { .. }
        | Action::GenerateConstitution
        | Action::AppendConstitutionOutput { .. }
        | Action::SaveConstitution
        | Action::SetConstitutionError { .. }
        | Action::CheckConstitutionExists
        | Action::SetConstitutionExists { .. }
        | Action::ApplyDefaultConstitution
        | Action::ReadConstitution
        | Action::SetConstitutionContent { .. }
        | Action::SetClaudeMdExists { .. }
        | Action::ReadClaudeMd
        | Action::SetClaudeMdContent { .. }
        | Action::ImportClaudeMd
        | Action::SkipClaudeMdImport
        | Action::SetUseClaudeMdReference { .. }
        | Action::SetConstitutionMode { .. }
        | Action::SelectConstitutionPreset { .. }
        | Action::CreateConstitutionPreset { .. }
        | Action::UpdateConstitutionPreset { .. }
        | Action::DeleteConstitutionPreset { .. }
        | Action::SetConstitutionPresetTempFile { .. } => {
            constitution::reduce(state, action);
        }

        Action::StartReview { .. }
        | Action::AddReviewComment { .. }
        | Action::ResolveReviewComment { .. }
        | Action::SubmitReviewFeedback { .. }
        | Action::ApproveReview { .. }
        | Action::RejectReview { .. }
        | Action::UpdateReviewContent { .. }
        | Action::SetReviewStatus { .. }
        | Action::SetReviewGateLoading { .. }
        | Action::SetReviewGateError { .. }
        | Action::SetActiveReviewSession { .. }
        | Action::ClearReviewSession { .. } => {
            review_gate::reduce(state, action);
        }

        Action::CopyEnvFiles { .. }
        | Action::SetEnvCopyResult { .. }
        | Action::SetEnvTrackedPatterns { .. }
        | Action::SetEnvAutoCopy { .. }
        | Action::SetEnvSourceWorktree { .. }
        | Action::SetAgentRulesEnabled { .. }
        | Action::SetAgentRulesPrompt { .. }
        | Action::SetAgentRulesTempFile { .. }
        | Action::CreateAgentProfile { .. }
        | Action::UpdateAgentProfile { .. }
        | Action::DeleteAgentProfile { .. }
        | Action::SelectAgentProfile { .. } => {
            env::reduce(state, action);
        }

        Action::SetError { .. }
        | Action::ClearError => {
            if let Action::SetError { code, message, context } = action {
                state.error = Some(crate::app_state::AppError {
                    code,
                    message,
                    context,
                });
            } else if let Action::ClearError = action {
                state.error = None;
            }
        }
    }
}
