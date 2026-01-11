use crate::actions::Action;
use crate::app_state::AppState;

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::CreateChange { .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.is_loading = true;
                }
            }
        }

        Action::GenerateProposal { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Planning;
                        change.streaming_output.clear();
                    }
                }
            }
        }

        Action::AppendProposalOutput { change_id, content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.streaming_output.push_str(&content);
                    }
                }
            }
        }

        Action::CompleteProposal { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        let proposal_content = std::mem::take(&mut change.streaming_output);
                        change.proposal = Some(proposal_content.clone());
                        change.status = crate::app_state::ChangeStatus::Proposed;
                        change.updated_at = chrono::Utc::now().to_rfc3339();

                        let session_id = uuid::Uuid::new_v4().to_string();
                        let now = chrono::Utc::now().to_rfc3339();
                        let session = crate::app_state::ReviewSession {
                            id: session_id.clone(),
                            workflow_node_id: format!("proposal-{}", change_id),
                            status: crate::app_state::ReviewStatus::Reviewing,
                            content: crate::app_state::ReviewContent {
                                content_type: crate::app_state::ReviewContentType::Proposal,
                                content: proposal_content,
                                file_changes: vec![],
                            },
                            policy: crate::app_state::ReviewPolicy::AlwaysReview,
                            comments: vec![],
                            iteration: 1,
                            created_at: now.clone(),
                            updated_at: now,
                        };
                        change.proposal_review_session_id = Some(session_id.clone());
                        worktree.tasks.review_gate.sessions.insert(session_id.clone(), session);
                        worktree.tasks.review_gate.active_session_id = Some(session_id);
                    }
                }
            }
        }

        Action::GeneratePlan { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Planning;
                        change.streaming_output.clear();
                    }
                }
            }
        }

        Action::AppendPlanOutput { change_id, content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.streaming_output.push_str(&content);
                    }
                }
            }
        }

        Action::CompletePlan { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        let plan_content = std::mem::take(&mut change.streaming_output);
                        change.plan = Some(plan_content.clone());
                        change.status = crate::app_state::ChangeStatus::Planned;
                        change.updated_at = chrono::Utc::now().to_rfc3339();

                        let session_id = uuid::Uuid::new_v4().to_string();
                        let now = chrono::Utc::now().to_rfc3339();
                        let session = crate::app_state::ReviewSession {
                            id: session_id.clone(),
                            workflow_node_id: format!("plan-{}", change_id),
                            status: crate::app_state::ReviewStatus::Reviewing,
                            content: crate::app_state::ReviewContent {
                                content_type: crate::app_state::ReviewContentType::Plan,
                                content: plan_content,
                                file_changes: vec![],
                            },
                            policy: crate::app_state::ReviewPolicy::AlwaysReview,
                            comments: vec![],
                            iteration: 1,
                            created_at: now.clone(),
                            updated_at: now,
                        };
                        change.plan_review_session_id = Some(session_id.clone());
                        worktree.tasks.review_gate.sessions.insert(session_id.clone(), session);
                        worktree.tasks.review_gate.active_session_id = Some(session_id);
                    }
                }
            }
        }

        Action::ApprovePlan { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Planned;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::ExecutePlan { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Implementing;
                        change.streaming_output.clear();
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::AppendImplementationOutput { change_id, content } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.streaming_output.push_str(&content);
                    }
                }
            }
        }

        Action::CompleteImplementation { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Done;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::FailImplementation { change_id, .. } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Failed;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::CancelChange { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
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

        Action::SetContextValidationResult { result } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    worktree.changes.validation_result = Some(result.into());
                }
            }
        }

        Action::ValidateContextFile { .. } => {
            // Async action, handled in lib.rs
        }

        Action::AddContextFile { change_id, path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        if !change.context_files.contains(&path) {
                            change.context_files.push(path);
                            change.updated_at = chrono::Utc::now().to_rfc3339();
                        }
                    }
                }
            }
        }

        Action::RemoveContextFile { change_id, path } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.context_files.retain(|p| p != &path);
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::ClearContextFiles { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.context_files.clear();
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        Action::SetChangeArchived { change_id } => {
            if let Some(project) = state.active_project_mut() {
                if let Some(worktree) = project.active_worktree_mut() {
                    if let Some(change) = worktree.changes.changes.iter_mut().find(|c| c.id == change_id) {
                        change.status = crate::app_state::ChangeStatus::Archived;
                        change.updated_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }
        _ => {}
    }
}
