use crate::actions::{
    DockerServiceData, JustCommandData, TaskStatusData, McpStatusData, 
    PortConflictData, ConflictingContainerData, FileEntryData, CommentData,
    ReviewPolicyData, ReviewContentTypeData, ReviewFileActionData, ReviewStatusData,
};
use crate::app_state::{
    DockerServiceInfo, ServiceStatus, ServiceType, JustCommandInfo, TaskStatus,
    McpStatus, PortConflict, ConflictingContainer, FileEntry, Comment,
    ReviewPolicy, ReviewContentType, ReviewFileAction, ReviewStatus,
};

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

impl From<FileEntryData> for FileEntry {
    fn from(data: FileEntryData) -> Self {
        Self {
            name: data.name,
            path: data.path,
            kind: data.kind.into(),
            size: data.size,
            permissions: data.permissions,
            updated_at: data.updated_at,
            comment_count: data.comment_count,
            git_status: data.git_status.map(|s| s.into()),
        }
    }
}

impl From<CommentData> for Comment {
    fn from(data: CommentData) -> Self {
        Self {
            id: data.id,
            content: data.content,
            author: data.author,
            created_at: data.created_at,
            line_number: data.line_number,
        }
    }
}

impl From<ReviewPolicyData> for ReviewPolicy {
    fn from(data: ReviewPolicyData) -> Self {
        match data {
            ReviewPolicyData::AutoApprove => ReviewPolicy::AutoApprove,
            ReviewPolicyData::AgentDecides => ReviewPolicy::AgentDecides,
            ReviewPolicyData::AlwaysReview => ReviewPolicy::AlwaysReview,
        }
    }
}

impl From<ReviewContentTypeData> for ReviewContentType {
    fn from(data: ReviewContentTypeData) -> Self {
        match data {
            ReviewContentTypeData::Plan => ReviewContentType::Plan,
            ReviewContentTypeData::Proposal => ReviewContentType::Proposal,
            ReviewContentTypeData::Code => ReviewContentType::Code,
            ReviewContentTypeData::Artifact => ReviewContentType::Artifact,
        }
    }
}

impl From<ReviewFileActionData> for ReviewFileAction {
    fn from(data: ReviewFileActionData) -> Self {
        match data {
            ReviewFileActionData::Create => ReviewFileAction::Create,
            ReviewFileActionData::Modify => ReviewFileAction::Modify,
            ReviewFileActionData::Delete => ReviewFileAction::Delete,
        }
    }
}

impl From<ReviewStatusData> for ReviewStatus {
    fn from(data: ReviewStatusData) -> Self {
        match data {
            ReviewStatusData::Pending => ReviewStatus::Pending,
            ReviewStatusData::Reviewing => ReviewStatus::Reviewing,
            ReviewStatusData::Iterating => ReviewStatus::Iterating,
            ReviewStatusData::Approved => ReviewStatus::Approved,
            ReviewStatusData::Rejected => ReviewStatus::Rejected,
        }
    }
}