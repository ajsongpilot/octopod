use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Company entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Company {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Department entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub slug: String, // product, engineering, etc.
    pub description: Option<String>,
    pub workspace: i64,
    pub config_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Department {
    pub fn new(
        company_id: impl Into<String>,
        name: impl Into<String>,
        slug: impl Into<String>,
        workspace: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.into(),
            name: name.into(),
            slug: slug.into(),
            description: None,
            workspace,
            config_json: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Agent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Error,
    Offline,
}

/// Agent entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: String,
    pub department_id: String,
    pub name: String,
    pub role: Option<String>,
    pub personality: Option<String>,
    pub model: Option<String>,
    pub config_json: Option<String>,
    pub status: AgentStatus,
    pub current_task_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

impl Agent {
    pub fn new(department_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            department_id: department_id.into(),
            name: name.into(),
            role: None,
            personality: None,
            model: None,
            config_json: None,
            status: AgentStatus::Idle,
            current_task_id: None,
            created_at: Utc::now(),
            last_seen_at: None,
        }
    }
}

/// Priority levels for decisions and tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum Priority {
    P0, // Critical
    P1, // High
    P2, // Medium
    P3, // Low
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
        }
    }

    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "p0" | "critical" => Some(Priority::P0),
            "p1" | "high" => Some(Priority::P1),
            "p2" | "medium" => Some(Priority::P2),
            "p3" | "low" => Some(Priority::P3),
            _ => None,
        }
    }
}

/// Decision status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum DecisionStatus {
    Pending,
    Approved,
    Rejected,
    Escalated,
    Cancelled,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionStatus::Pending => "pending",
            DecisionStatus::Approved => "approved",
            DecisionStatus::Rejected => "rejected",
            DecisionStatus::Escalated => "escalated",
            DecisionStatus::Cancelled => "cancelled",
        }
    }
}

/// Decision severity - how important is this decision to escalate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum DecisionSeverity {
    Low,    // Operational, contained to one department - auto-proceed
    Medium, // Cross-department, significant - auto-proceed with logging
    High,   // External launch, feature removal, strategic - CEO approval required
}

impl DecisionSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionSeverity::Low => "low",
            DecisionSeverity::Medium => "medium",
            DecisionSeverity::High => "high",
        }
    }

    pub fn requires_approval(&self) -> bool {
        matches!(self, DecisionSeverity::High)
    }
}

/// Decision entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Decision {
    pub id: String,
    pub company_id: String,
    pub title: String,
    pub description: Option<String>,
    pub department_id: Option<String>,
    pub requested_by: Option<String>,
    pub priority: Priority,
    pub severity: DecisionSeverity,
    pub status: DecisionStatus,
    pub context_json: Option<String>,
    pub approved_by: Option<String>,
    pub decision_notes: Option<String>,
    pub initiative_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
}

impl Decision {
    pub fn new(company_id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.into(),
            title: title.into(),
            description: None,
            department_id: None,
            requested_by: None,
            priority: Priority::P2,
            severity: DecisionSeverity::Medium,
            status: DecisionStatus::Pending,
            context_json: None,
            approved_by: None,
            decision_notes: None,
            initiative_id: None,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            file_path: None,
        }
    }

    pub fn with_severity(mut self, severity: DecisionSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_initiative(mut self, initiative_id: impl Into<String>) -> Self {
        self.initiative_id = Some(initiative_id.into());
        self
    }

    pub fn approve(&mut self, approved_by: impl Into<String>, notes: Option<String>) {
        self.status = DecisionStatus::Approved;
        self.approved_by = Some(approved_by.into());
        self.decision_notes = notes;
        self.resolved_at = Some(Utc::now());
    }

    pub fn reject(&mut self, rejected_by: impl Into<String>, notes: Option<String>) {
        self.status = DecisionStatus::Rejected;
        self.approved_by = Some(rejected_by.into()); // Using same field for rejector
        self.decision_notes = notes;
        self.resolved_at = Some(Utc::now());
    }

    pub fn escalate(&mut self) {
        self.status = DecisionStatus::Escalated;
    }
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum MessageType {
    Chat,
    DecisionRequest,
    DecisionResponse,
    DecisionProposal,
    InitiativeUpdate,
    TaskAssignment,
    MeetingRequest,
    Command,
    Notification,
    File,
    Error,
}

/// Conversation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum ConversationType {
    Channel,
    Dm,
    Thread,
}

/// Conversation entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: String,
    pub company_id: String,
    pub department_id: Option<String>,
    pub title: Option<String>,
    pub conversation_type: ConversationType,
    pub parent_message_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl Conversation {
    pub fn new(company_id: impl Into<String>, conversation_type: ConversationType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.into(),
            department_id: None,
            title: None,
            conversation_type,
            parent_message_id: None,
            created_at: Utc::now(),
            archived_at: None,
        }
    }
}

/// Message entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub from_agent_id: Option<String>,
    pub to_agent_id: Option<String>,
    pub content: String,
    pub message_type: MessageType,
    pub metadata_json: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(conversation_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.into(),
            from_agent_id: None,
            to_agent_id: None,
            content: content.into(),
            message_type: MessageType::Chat,
            metadata_json: None,
            deleted_at: None,
            deleted_by: None,
            created_at: Utc::now(),
        }
    }

    pub fn from_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.from_agent_id = Some(agent_id.into());
        self
    }

    pub fn to_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.to_agent_id = Some(agent_id.into());
        self
    }

    pub fn message_type(mut self, msg_type: MessageType) -> Self {
        self.message_type = msg_type;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata_json = Some(serde_json::to_string(&metadata).unwrap_or_default());
        self
    }
}

/// Metadata for decision-related messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMessageMetadata {
    pub decision_id: String,
    pub severity: String,
    pub title: String,
}

/// Metadata for initiative-related messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiativeMessageMetadata {
    pub initiative_id: String,
    pub initiative_title: String,
    pub status: String,
    pub previous_status: Option<String>,
}

/// Metadata for task assignment messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignmentMetadata {
    pub task_id: String,
    pub task_title: String,
    pub department_id: String,
    pub assigned_to: Option<String>,
}

/// Metadata for meeting request messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRequestMetadata {
    pub meeting_id: String,
    pub meeting_title: String,
    pub meeting_type: String,
    pub requested_by: String,
}

/// Task type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum TaskType {
    Feature,
    Bug,
    Task,
    Research,
    Documentation,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Review,
    Done,
    Cancelled,
}

/// Task entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: String,
    pub company_id: String,
    pub department_id: String,
    pub initiative_id: Option<String>,
    pub assigned_to: Option<String>,
    pub created_by: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: Priority,
    pub parent_task_id: Option<String>,
    pub related_decision_id: Option<String>,
    pub file_path: Option<String>,
    pub github_issue_number: Option<i64>,
    pub estimated_hours: Option<i64>,
    pub actual_hours: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deadline_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(
        company_id: impl Into<String>,
        department_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.into(),
            department_id: department_id.into(),
            initiative_id: None,
            assigned_to: None,
            created_by: None,
            title: title.into(),
            description: None,
            acceptance_criteria: None,
            task_type: TaskType::Task,
            status: TaskStatus::Todo,
            priority: Priority::P2,
            parent_task_id: None,
            related_decision_id: None,
            file_path: None,
            github_issue_number: None,
            estimated_hours: None,
            actual_hours: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            deadline_at: None,
            deleted_at: None,
        }
    }
}

/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityLogEntry {
    pub id: i64,
    pub company_id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub actor_type: String,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub details_json: Option<String>,
}

/// Initiative status - lifecycle for initiatives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum InitiativeStatus {
    Draft,             // Initial creation, not yet proposed
    Proposed,          // Submitted for review, awaiting stakeholder input
    StakeholderReview, // Active review with stakeholders
    Approved,          // Approved but not yet started
    Active,            // In progress
    Completed,         // Done
    Cancelled,         // Cancelled
    Closed,            // Closed without completion (e.g., scrapped, deferred)
    Archived,          // Archived
}

impl InitiativeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InitiativeStatus::Draft => "draft",
            InitiativeStatus::Proposed => "proposed",
            InitiativeStatus::StakeholderReview => "stakeholder_review",
            InitiativeStatus::Approved => "approved",
            InitiativeStatus::Active => "active",
            InitiativeStatus::Completed => "completed",
            InitiativeStatus::Cancelled => "cancelled",
            InitiativeStatus::Closed => "closed",
            InitiativeStatus::Archived => "archived",
        }
    }

    pub fn can_transition_to(&self, next: &InitiativeStatus) -> bool {
        matches!(
            (self, next),
            (InitiativeStatus::Draft, InitiativeStatus::Proposed)
                | (InitiativeStatus::Draft, InitiativeStatus::Closed)
                | (InitiativeStatus::Draft, InitiativeStatus::Cancelled)
                | (
                    InitiativeStatus::Proposed,
                    InitiativeStatus::StakeholderReview
                )
                | (InitiativeStatus::Proposed, InitiativeStatus::Approved)
                | (InitiativeStatus::Proposed, InitiativeStatus::Closed)
                | (InitiativeStatus::Proposed, InitiativeStatus::Cancelled)
                | (
                    InitiativeStatus::StakeholderReview,
                    InitiativeStatus::Approved
                )
                | (
                    InitiativeStatus::StakeholderReview,
                    InitiativeStatus::Closed
                )
                | (
                    InitiativeStatus::StakeholderReview,
                    InitiativeStatus::Cancelled
                )
                | (InitiativeStatus::Approved, InitiativeStatus::Active)
                | (InitiativeStatus::Approved, InitiativeStatus::Closed)
                | (InitiativeStatus::Approved, InitiativeStatus::Cancelled)
                | (InitiativeStatus::Active, InitiativeStatus::Completed)
                | (InitiativeStatus::Active, InitiativeStatus::Closed)
                | (InitiativeStatus::Active, InitiativeStatus::Cancelled)
                | (InitiativeStatus::Completed, InitiativeStatus::Archived)
                | (InitiativeStatus::Closed, InitiativeStatus::Archived)
                | (InitiativeStatus::Cancelled, InitiativeStatus::Archived)
        )
    }

    pub fn requires_ceo_decision(&self) -> bool {
        matches!(
            self,
            InitiativeStatus::Proposed | InitiativeStatus::StakeholderReview
        )
    }
}

/// Roadmap status (stored as TEXT: draft, planning, active, completed, archived)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum RoadmapStatus {
    Draft,
    Planning,
    Active,
    Completed,
    Archived,
}

/// Roadmap entity - represents a planning period (quarter, year, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Roadmap {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    #[sqlx(rename = "status_id")]
    pub status: RoadmapStatus,
    pub goals_json: Option<String>, // JSON array of goal strings
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Roadmap {
    pub fn new(
        company_id: impl Into<String>,
        name: impl Into<String>,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.into(),
            name: name.into(),
            description: None,
            period_start,
            period_end,
            status: RoadmapStatus::Draft,
            goals_json: None,
            created_by: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Initiative entity - a group of related tasks under a roadmap
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Initiative {
    pub id: String,
    pub roadmap_id: String,
    pub department_id: String,
    pub title: String,
    pub description: Option<String>,
    #[sqlx(rename = "status_id")]
    pub status: InitiativeStatus,
    pub priority: Priority,
    pub severity: DecisionSeverity, // How important is this initiative
    pub stakeholder_depts_json: Option<String>, // JSON array of dept slugs that need to review
    pub pending_decision_id: Option<String>, // Link to CEO decision if HIGH severity
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub file_path: Option<String>, // Path to markdown file
}

impl Initiative {
    pub fn new(
        roadmap_id: impl Into<String>,
        department_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            roadmap_id: roadmap_id.into(),
            department_id: department_id.into(),
            title: title.into(),
            description: None,
            status: InitiativeStatus::Draft,
            priority: Priority::P2,
            severity: DecisionSeverity::Medium,
            stakeholder_depts_json: None,
            pending_decision_id: None,
            created_by: None,
            created_at: now,
            updated_at: now,
            file_path: None,
        }
    }

    pub fn new_with_id(
        roadmap_id: impl Into<String>,
        department_id: impl Into<String>,
        title: impl Into<String>,
        id: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            roadmap_id: roadmap_id.into(),
            department_id: department_id.into(),
            title: title.into(),
            description: None,
            status: InitiativeStatus::Draft,
            priority: Priority::P2,
            severity: DecisionSeverity::Medium,
            stakeholder_depts_json: None,
            pending_decision_id: None,
            created_by: None,
            created_at: now,
            updated_at: now,
            file_path: None,
        }
    }

    pub fn with_severity(mut self, severity: DecisionSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_stakeholders(mut self, depts: Vec<String>) -> Self {
        self.stakeholder_depts_json = serde_json::to_string(&depts).ok();
        self
    }

    pub fn get_stakeholders(&self) -> Vec<String> {
        self.stakeholder_depts_json
            .as_ref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default()
    }

    pub fn transition_to(&mut self, new_status: InitiativeStatus) -> bool {
        if self.status.can_transition_to(&new_status) {
            self.status = new_status;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
}

/// Meeting type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum MeetingType {
    Planning,
    Refinement,
    StakeholderReview,
    SprintRetro,
    OneOnOne,
    AllHands,
    AdHoc,
}

/// Meeting status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum MeetingStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

/// Meeting entity - structured meeting with participants
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Meeting {
    pub id: String,
    pub company_id: String,
    pub title: String,
    pub meeting_type: MeetingType,
    pub status: MeetingStatus,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: i64,
    pub agenda_json: Option<String>,   // JSON array of agenda items
    pub notes: Option<String>,         // Notes taken during/after meeting
    pub outcomes_json: Option<String>, // JSON array of outcomes/decisions
    pub related_initiative_id: Option<String>, // Optional link to initiative
    pub related_roadmap_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Meeting {
    pub fn new(
        company_id: impl Into<String>,
        title: impl Into<String>,
        meeting_type: MeetingType,
        scheduled_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.into(),
            title: title.into(),
            meeting_type,
            status: MeetingStatus::Scheduled,
            scheduled_at,
            duration_minutes: 60,
            agenda_json: None,
            notes: None,
            outcomes_json: None,
            related_initiative_id: None,
            related_roadmap_id: None,
            created_by: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Meeting participant entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeetingParticipant {
    pub id: String,
    pub meeting_id: String,
    pub department_id: String,
    pub agent_id: Option<String>,
    pub role: String, // "facilitator", "participant", "observer", "required", "optional"
    pub status: String, // "invited", "accepted", "declined", "tentative"
    pub response_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl MeetingParticipant {
    pub fn new(
        meeting_id: impl Into<String>,
        department_id: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.into(),
            department_id: department_id.into(),
            agent_id: None,
            role: role.into(),
            status: "invited".to_string(),
            response_message: None,
            created_at: Utc::now(),
        }
    }
}

/// Agent session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum AgentSessionStatus {
    Active,
    Completed,
    Stopped,
}

impl AgentSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentSessionStatus::Active => "active",
            AgentSessionStatus::Completed => "completed",
            AgentSessionStatus::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for AgentSessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Agent session - tracks opencode sessions spawned by octopod agents
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentSession {
    pub id: String,
    pub task_id: String,
    pub department_id: String,
    pub session_id: String,
    pub process_id: Option<i64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl AgentSession {
    pub fn new(
        task_id: impl Into<String>,
        department_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.into(),
            department_id: department_id.into(),
            session_id: session_id.into(),
            process_id: None,
            status: "active".to_string(),
            created_at: Utc::now(),
            ended_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_severity_as_str() {
        assert_eq!(DecisionSeverity::Low.as_str(), "low");
        assert_eq!(DecisionSeverity::Medium.as_str(), "medium");
        assert_eq!(DecisionSeverity::High.as_str(), "high");
    }

    #[test]
    fn test_decision_severity_requires_approval() {
        assert!(!DecisionSeverity::Low.requires_approval());
        assert!(!DecisionSeverity::Medium.requires_approval());
        assert!(DecisionSeverity::High.requires_approval());
    }

    #[test]
    fn test_decision_new() {
        let decision = Decision::new("company-1", "Test Decision");
        assert_eq!(decision.company_id, "company-1");
        assert_eq!(decision.title, "Test Decision");
        assert_eq!(decision.severity, DecisionSeverity::Medium);
        assert_eq!(decision.status, DecisionStatus::Pending);
        assert!(decision.description.is_none());
        assert!(decision.approved_by.is_none());
    }

    #[test]
    fn test_decision_with_severity() {
        let decision = Decision::new("company-1", "Test").with_severity(DecisionSeverity::High);
        assert_eq!(decision.severity, DecisionSeverity::High);
    }

    #[test]
    fn test_decision_with_initiative() {
        let decision = Decision::new("company-1", "Test").with_initiative("initiative-1");
        assert_eq!(decision.initiative_id, Some("initiative-1".to_string()));
    }

    #[test]
    fn test_decision_approve() {
        let mut decision = Decision::new("company-1", "Test");
        decision.approve("ceo-1", Some("LGTM".to_string()));
        assert_eq!(decision.status, DecisionStatus::Approved);
        assert_eq!(decision.approved_by, Some("ceo-1".to_string()));
        assert_eq!(decision.decision_notes, Some("LGTM".to_string()));
        assert!(decision.resolved_at.is_some());
    }

    #[test]
    fn test_decision_reject() {
        let mut decision = Decision::new("company-1", "Test");
        decision.reject("ceo-1", Some("Not ready".to_string()));
        assert_eq!(decision.status, DecisionStatus::Rejected);
        assert_eq!(decision.approved_by, Some("ceo-1".to_string()));
        assert_eq!(decision.decision_notes, Some("Not ready".to_string()));
        assert!(decision.resolved_at.is_some());
    }

    #[test]
    fn test_initiative_status_as_str() {
        assert_eq!(InitiativeStatus::Draft.as_str(), "draft");
        assert_eq!(InitiativeStatus::Proposed.as_str(), "proposed");
        assert_eq!(
            InitiativeStatus::StakeholderReview.as_str(),
            "stakeholder_review"
        );
        assert_eq!(InitiativeStatus::Approved.as_str(), "approved");
        assert_eq!(InitiativeStatus::Active.as_str(), "active");
        assert_eq!(InitiativeStatus::Completed.as_str(), "completed");
        assert_eq!(InitiativeStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(InitiativeStatus::Archived.as_str(), "archived");
    }

    #[test]
    fn test_initiative_status_can_transition() {
        use InitiativeStatus::*;

        // Valid transitions from Draft
        assert!(Draft.can_transition_to(&Proposed));
        assert!(Draft.can_transition_to(&Cancelled));
        assert!(!Draft.can_transition_to(&Active));
        assert!(!Draft.can_transition_to(&Completed));

        // Valid transitions from Proposed
        assert!(Proposed.can_transition_to(&StakeholderReview));
        assert!(Proposed.can_transition_to(&Approved));
        assert!(Proposed.can_transition_to(&Cancelled));
        assert!(!Proposed.can_transition_to(&Active));

        // Valid transitions from StakeholderReview
        assert!(StakeholderReview.can_transition_to(&Approved));
        assert!(StakeholderReview.can_transition_to(&Cancelled));

        // Valid transitions from Approved
        assert!(Approved.can_transition_to(&Active));
        assert!(Approved.can_transition_to(&Cancelled));

        // Valid transitions from Active
        assert!(Active.can_transition_to(&Completed));
        assert!(Active.can_transition_to(&Cancelled));

        // Valid transitions from Completed
        assert!(Completed.can_transition_to(&Archived));

        // Valid transitions from Cancelled
        assert!(Cancelled.can_transition_to(&Archived));
    }

    #[test]
    fn test_initiative_status_requires_ceo_decision() {
        assert!(!InitiativeStatus::Draft.requires_ceo_decision());
        assert!(InitiativeStatus::Proposed.requires_ceo_decision());
        assert!(InitiativeStatus::StakeholderReview.requires_ceo_decision());
        assert!(!InitiativeStatus::Approved.requires_ceo_decision());
        assert!(!InitiativeStatus::Active.requires_ceo_decision());
    }

    #[test]
    fn test_initiative_new() {
        let initiative = Initiative::new("roadmap-1", "dept-1", "Test Initiative");
        assert_eq!(initiative.roadmap_id, "roadmap-1");
        assert_eq!(initiative.department_id, "dept-1");
        assert_eq!(initiative.title, "Test Initiative");
        assert_eq!(initiative.status, InitiativeStatus::Draft);
        assert!(initiative.description.is_none());
    }

    #[test]
    fn test_initiative_transition_to() {
        let mut initiative = Initiative::new("roadmap-1", "dept-1", "Test");

        initiative.transition_to(InitiativeStatus::Proposed);
        assert_eq!(initiative.status, InitiativeStatus::Proposed);

        initiative.transition_to(InitiativeStatus::Approved);
        assert_eq!(initiative.status, InitiativeStatus::Approved);

        initiative.transition_to(InitiativeStatus::Active);
        assert_eq!(initiative.status, InitiativeStatus::Active);
    }

    #[test]
    fn test_initiative_invalid_transition() {
        let mut initiative = Initiative::new("roadmap-1", "dept-1", "Test");

        // Can't go from Draft directly to Active
        initiative.transition_to(InitiativeStatus::Active);
        // Should still be Draft because invalid transition
        assert_eq!(initiative.status, InitiativeStatus::Draft);
    }

    #[test]
    fn test_priority_as_str() {
        assert_eq!(Priority::P0.as_str(), "P0");
        assert_eq!(Priority::P1.as_str(), "P1");
        assert_eq!(Priority::P2.as_str(), "P2");
        assert_eq!(Priority::P3.as_str(), "P3");
    }

    #[test]
    fn test_priority_from_str() {
        assert_eq!(Priority::from_str_lossy("p0"), Some(Priority::P0));
        assert_eq!(Priority::from_str_lossy("critical"), Some(Priority::P0));
        assert_eq!(Priority::from_str_lossy("high"), Some(Priority::P1));
        assert_eq!(Priority::from_str_lossy("medium"), Some(Priority::P2));
        assert_eq!(Priority::from_str_lossy("low"), Some(Priority::P3));
        assert_eq!(Priority::from_str_lossy("invalid"), None);
    }

    #[test]
    fn test_task_status_debug() {
        assert_eq!(format!("{:?}", TaskStatus::Todo), "Todo");
        assert_eq!(format!("{:?}", TaskStatus::InProgress), "InProgress");
        assert_eq!(format!("{:?}", TaskStatus::Blocked), "Blocked");
        assert_eq!(format!("{:?}", TaskStatus::Review), "Review");
        assert_eq!(format!("{:?}", TaskStatus::Done), "Done");
        assert_eq!(format!("{:?}", TaskStatus::Cancelled), "Cancelled");
    }

    #[test]
    fn test_task_type_debug() {
        assert_eq!(format!("{:?}", TaskType::Feature), "Feature");
        assert_eq!(format!("{:?}", TaskType::Bug), "Bug");
        assert_eq!(format!("{:?}", TaskType::Task), "Task");
        assert_eq!(format!("{:?}", TaskType::Research), "Research");
        assert_eq!(format!("{:?}", TaskType::Documentation), "Documentation");
    }

    #[test]
    fn test_company_new() {
        let company = Company::new("Acme Corp");
        assert_eq!(company.name, "Acme Corp");
        assert!(!company.id.is_empty());
        assert!(company.created_at <= Utc::now());
    }

    #[test]
    fn test_department_new() {
        let dept = Department::new("company-1", "Engineering", "engineering", 3);
        assert_eq!(dept.company_id, "company-1");
        assert_eq!(dept.name, "Engineering");
        assert_eq!(dept.slug, "engineering");
        assert_eq!(dept.workspace, 3);
    }

    #[test]
    fn test_agent_new() {
        let agent = Agent::new("dept-1", "engineering-agent");
        assert_eq!(agent.department_id, "dept-1");
        assert_eq!(agent.name, "engineering-agent");
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_message_new() {
        let msg = Message::new("channel-1", "Hello, world!");
        assert_eq!(msg.conversation_id, "channel-1");
        assert_eq!(msg.content, "Hello, world!");
        assert_eq!(msg.message_type, MessageType::Chat);
    }

    #[test]
    fn test_message_with_metadata() {
        let msg =
            Message::new("channel-1", "Test").with_metadata(serde_json::json!({"key": "value"}));
        assert!(msg.metadata_json.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&msg.metadata_json.unwrap()).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_decision_message_metadata() {
        let meta = DecisionMessageMetadata {
            decision_id: "dec-1".to_string(),
            severity: "high".to_string(),
            title: "Test Decision".to_string(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: DecisionMessageMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.decision_id, "dec-1");
        assert_eq!(parsed.severity, "high");
    }

    #[test]
    fn test_initiative_message_metadata() {
        let meta = InitiativeMessageMetadata {
            initiative_id: "init-1".to_string(),
            initiative_title: "Test Initiative".to_string(),
            status: "active".to_string(),
            previous_status: Some("approved".to_string()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: InitiativeMessageMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.initiative_id, "init-1");
        assert_eq!(parsed.previous_status, Some("approved".to_string()));
    }

    #[test]
    fn test_roadmap_new() {
        let roadmap = Roadmap::new(
            "company-1",
            "Q1 2026",
            Utc::now(),
            Utc::now() + chrono::Duration::days(90),
        );
        assert_eq!(roadmap.company_id, "company-1");
        assert_eq!(roadmap.name, "Q1 2026");
        assert_eq!(roadmap.status, RoadmapStatus::Draft);
    }

    #[test]
    fn test_meeting_new() {
        let meeting = Meeting::new(
            "company-1",
            "Sprint Planning",
            MeetingType::Planning,
            Utc::now(),
        );
        assert_eq!(meeting.title, "Sprint Planning");
        assert_eq!(meeting.meeting_type, MeetingType::Planning);
    }

    #[test]
    fn test_meeting_participant_new() {
        let participant = MeetingParticipant::new("meeting-1", "dept-1", "required");
        assert_eq!(participant.meeting_id, "meeting-1");
        assert_eq!(participant.department_id, "dept-1");
        assert_eq!(participant.role, "required");
        assert_eq!(participant.status, "invited");
    }
}
