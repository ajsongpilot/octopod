use crate::state::{
    backup::{BackupConfig, BackupManager},
    decision_file::DecisionFileManager,
    entities::*,
    initiative_file::InitiativeFileManager,
    message_bus::MessageBus,
    repositories::*,
    task_file::TaskFileManager,
    Database, DatabaseConfig,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// High-level state manager that coordinates all repositories
#[derive(Debug, Clone)]
pub struct StateManager {
    db: Database,
    pool: SqlitePool,

    // Repositories
    companies: CompanyRepository,
    departments: DepartmentRepository,
    decisions: DecisionRepository,
    conversations: ConversationRepository,
    messages: MessageRepository,
    tasks: TaskRepository,
    roadmaps: RoadmapRepository,
    initiatives: InitiativeRepository,
    meetings: MeetingRepository,
    meeting_participants: MeetingParticipantRepository,
    agent_sessions: AgentSessionRepository,
    activities: ActivityRepository,

    // Message bus for inter-department communication
    message_bus: MessageBus,

    // Backup manager
    backup_manager: BackupManager,

    // Task file manager
    task_file_manager: TaskFileManager,

    // Initiative file manager
    initiative_file_manager: InitiativeFileManager,

    // Decision file manager
    decision_file_manager: DecisionFileManager,

    // Current company context (loaded from config)
    current_company_id: Arc<RwLock<Option<String>>>,
}

impl StateManager {
    /// Initialize the state manager
    pub async fn init(config: DatabaseConfig) -> Result<Self> {
        let db = Database::init(config).await?;
        let pool = db.pool().clone();

        let backup_manager = BackupManager::new(BackupConfig::default());
        let message_bus = MessageBus::new(pool.clone());

        let project_dir = db
            .path()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let task_file_manager = TaskFileManager::new(&project_dir)?;
        let initiative_file_manager = InitiativeFileManager::new(&project_dir)?;
        let decision_file_manager = DecisionFileManager::new(&project_dir)?;

        let tasks_repo = TaskRepository::new(pool.clone());
        let backfilled =
            StateManager::backfill_task_files_static_inner(&tasks_repo, &task_file_manager).await?;
        if backfilled > 0 {
            info!("Backfilled {} task files", backfilled);
        }

        Ok(Self {
            db: db.clone(),
            pool: pool.clone(),
            companies: CompanyRepository::new(pool.clone()),
            departments: DepartmentRepository::new(pool.clone()),
            decisions: DecisionRepository::new(pool.clone()),
            conversations: ConversationRepository::new(pool.clone()),
            messages: MessageRepository::new(pool.clone()),
            tasks: TaskRepository::new(pool.clone()),
            roadmaps: RoadmapRepository::new(pool.clone()),
            initiatives: InitiativeRepository::new(pool.clone()),
            meetings: MeetingRepository::new(pool.clone()),
            meeting_participants: MeetingParticipantRepository::new(pool.clone()),
            agent_sessions: AgentSessionRepository::new(pool.clone()),
            activities: ActivityRepository::new(pool.clone()),
            message_bus,
            backup_manager,
            task_file_manager,
            initiative_file_manager,
            decision_file_manager,
            current_company_id: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize state for a project
    pub async fn init_for_project(project_dir: &Path) -> Result<Self> {
        let config = DatabaseConfig::for_project(project_dir);
        Self::init(config).await
    }

    /// Initialize state in user config directory
    pub async fn init_for_user() -> Result<Self> {
        let config = DatabaseConfig::for_user()?;
        Self::init(config).await
    }

    /// Set the current company context
    pub async fn set_company(&self, company_id: String) {
        let mut current = self.current_company_id.write().await;
        *current = Some(company_id);
    }

    /// Get the current company ID
    pub async fn current_company(&self) -> Option<String> {
        self.current_company_id.read().await.clone()
    }

    /// Require a company to be set
    async fn require_company(&self) -> Result<String> {
        self.current_company()
            .await
            .context("No company set. Call set_company() first.")
    }

    // ==================== Company Operations ====================

    pub async fn create_company(&self, name: impl Into<String>) -> Result<Company> {
        let company = Company::new(name);
        self.companies.create(&company).await
    }

    pub async fn get_company(&self, id: &str) -> Result<Option<Company>> {
        self.companies.find_by_id(id).await
    }

    pub async fn get_first_company(&self) -> Result<Option<Company>> {
        self.companies.find_first().await
    }

    // ==================== Department Operations ====================

    pub async fn create_department(
        &self,
        name: impl Into<String>,
        slug: impl Into<String>,
        workspace: i64,
    ) -> Result<Department> {
        let company_id = self.require_company().await?;
        let dept = Department::new(company_id, name, slug, workspace);
        self.departments.create(&dept).await
    }

    pub async fn get_department(&self, _id: &str) -> Result<Option<Department>> {
        // Note: This would need a find_by_id method in DepartmentRepository
        // For now, we'd need to add it
        todo!("Implement get_department")
    }

    pub async fn get_department_by_slug(&self, slug: &str) -> Result<Option<Department>> {
        self.departments.find_by_slug(slug).await
    }

    pub async fn list_departments(&self) -> Result<Vec<Department>> {
        // If company is set, return departments for that company
        // Otherwise, return all departments (for initialization purposes)
        match self.current_company().await {
            Some(company_id) => self.departments.find_by_company(&company_id).await,
            None => {
                // No company set - get all departments from any company
                // This allows initialization code to find and set the company
                sqlx::query_as::<_, Department>("SELECT * FROM departments LIMIT 100")
                    .fetch_all(&self.pool)
                    .await
                    .context("Failed to list all departments")
            }
        }
    }

    // ==================== Decision Operations ====================

    pub async fn create_decision(&self, title: impl Into<String>) -> Result<Decision> {
        let company_id = self.require_company().await?;
        let mut decision = Decision::new(company_id, title);

        let file_path = self.decision_file_manager.create_decision_file(&decision)?;
        decision.file_path = Some(file_path);

        self.decisions.create(&decision).await
    }

    pub async fn create_decision_with_severity(
        &self,
        title: impl Into<String>,
        severity: DecisionSeverity,
    ) -> Result<Decision> {
        let company_id = self.require_company().await?;
        let mut decision = Decision::new(company_id, title).with_severity(severity);

        let file_path = self.decision_file_manager.create_decision_file(&decision)?;
        decision.file_path = Some(file_path);

        self.decisions.create(&decision).await
    }

    pub async fn create_initiative_decision(
        &self,
        initiative_id: &str,
        title: impl Into<String>,
        severity: DecisionSeverity,
    ) -> Result<Decision> {
        let company_id = self.require_company().await?;
        let mut decision = Decision::new(company_id, title)
            .with_severity(severity)
            .with_initiative(initiative_id);

        let file_path = self.decision_file_manager.create_decision_file(&decision)?;
        decision.file_path = Some(file_path);

        self.decisions.create(&decision).await
    }

    pub async fn get_decision(&self, id: &str) -> Result<Option<Decision>> {
        self.decisions.find_by_id(id).await
    }

    pub async fn get_pending_decisions(&self, limit: i64) -> Result<Vec<Decision>> {
        let company_id = self.require_company().await?;
        self.decisions.get_pending(&company_id, limit).await
    }

    pub async fn get_high_severity_decisions(&self) -> Result<Vec<Decision>> {
        let company_id = self.require_company().await?;
        self.decisions.get_high_severity_pending(&company_id).await
    }

    pub async fn approve_decision(
        &self,
        decision_id: &str,
        approved_by: impl Into<String>,
        notes: Option<&str>,
    ) -> Result<Decision> {
        self.decisions
            .approve(decision_id, approved_by.into().as_str(), notes)
            .await
    }

    pub async fn reject_decision(
        &self,
        decision_id: &str,
        rejected_by: impl Into<String>,
        notes: Option<&str>,
    ) -> Result<Decision> {
        self.decisions
            .reject(decision_id, rejected_by.into().as_str(), notes)
            .await
    }

    pub async fn list_pending_decisions(&self, limit: i64) -> Result<Vec<Decision>> {
        let company_id = self.require_company().await?;
        self.decisions.get_pending(&company_id, limit).await
    }

    pub async fn get_decision_stats(&self) -> Result<DecisionStats> {
        let company_id = self.require_company().await?;
        self.decisions.get_stats(&company_id).await
    }

    // ==================== Task Operations ====================

    pub async fn create_task(
        &self,
        department_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Result<Task> {
        let company_id = self.require_company().await?;
        let mut task = Task::new(company_id, department_id, title);

        let file_path = self.task_file_manager.create_task_file(&task)?;
        task.file_path = Some(file_path);

        self.tasks.create(&task).await
    }

    pub async fn create_task_for_initiative(
        &self,
        department_id: &str,
        initiative_id: &str,
        title: &str,
    ) -> Result<Task> {
        let company_id = self.require_company().await?;
        let mut task = Task::new(company_id, department_id, title);
        task.initiative_id = Some(initiative_id.to_string());

        let file_path = self.task_file_manager.create_task_file(&task)?;
        task.file_path = Some(file_path);

        self.tasks.create(&task).await
    }

    pub async fn get_initiative_tasks(&self, initiative_id: &str) -> Result<Vec<Task>> {
        self.tasks.find_by_initiative(initiative_id).await
    }

    pub async fn get_department_tasks_for_initiative(
        &self,
        department_id: &str,
        initiative_id: &str,
    ) -> Result<Vec<Task>> {
        self.tasks
            .find_by_department_and_initiative(department_id, initiative_id)
            .await
    }

    pub async fn get_department_tasks(&self, department_slug: &str) -> Result<Vec<Task>> {
        // First resolve slug to UUID
        let dept = self.departments.find_by_slug(department_slug).await?;
        if let Some(dept) = dept {
            self.tasks.find_by_department(&dept.id).await
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>> {
        self.tasks.find_by_id(task_id).await
    }

    pub async fn update_task(&self, task: &Task) -> Result<Task> {
        self.tasks.update(task).await
    }

    pub async fn open_task_editor(&self, task: &Task) -> Result<()> {
        self.task_file_manager.open_in_editor(task)
    }

    pub fn open_initiative_editor(&self, initiative: &Initiative) -> Result<()> {
        self.initiative_file_manager.open_in_editor(initiative)
    }

    pub fn open_initiative_editor_by_id(&self, initiative_id: &str) -> Result<()> {
        self.initiative_file_manager.open_by_id(initiative_id)
    }

    pub fn get_task_content(&self, task: &Task) -> Result<String> {
        self.task_file_manager.read_task_file(task)
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        self.tasks.delete(task_id).await
    }

    pub async fn get_department_tasks_by_status(
        &self,
        department_id: &str,
        status: TaskStatus,
    ) -> Result<Vec<Task>> {
        self.tasks
            .find_by_department_and_status(department_id, status)
            .await
    }

    pub async fn get_unassigned_department_tasks(
        &self,
        department_id: &str,
        status: TaskStatus,
    ) -> Result<Vec<Task>> {
        self.tasks
            .find_unassigned_by_department(department_id, status)
            .await
    }

    pub async fn get_department_decisions(
        &self,
        department_slug: &str,
        status: Option<DecisionStatus>,
    ) -> Result<Vec<Decision>> {
        let company_id = self.require_company().await?;

        // Resolve slug to UUID
        let dept = self.departments.find_by_slug(department_slug).await?;
        let dept_id = match dept {
            Some(d) => d.id,
            None => return Ok(vec![]),
        };

        let filters = DecisionFilters::new()
            .with_department(&dept_id)
            .with_status(status.unwrap_or(DecisionStatus::Pending));
        self.decisions
            .list(&company_id, filters, Pagination::new(1, 100))
            .await
            .map(|r| r.items)
    }

    // ==================== Message Bus Operations ====================

    /// Send a message to a conversation (creates conversation if it doesn't exist)
    pub async fn send_message(
        &self,
        conversation_id: impl Into<String>,
        from_agent: Option<String>,
        to_agent: Option<String>,
        content: impl Into<String>,
        msg_type: MessageType,
    ) -> Result<Message> {
        let conversation_id = conversation_id.into();
        let content = content.into();

        // Ensure conversation exists
        self.ensure_conversation_exists(&conversation_id).await?;

        self.message_bus
            .send(conversation_id, from_agent, to_agent, content, msg_type)
            .await
    }

    /// Ensure a conversation exists, creating it if necessary
    async fn ensure_conversation_exists(&self, conversation_id: &str) -> Result<()> {
        // Check if conversation exists
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM conversations WHERE id = ?")
                .bind(conversation_id)
                .fetch_one(&self.pool)
                .await?;

        if exists == 0 {
            // Get company ID
            let company_id = self.require_company().await?;

            // Create conversation with the specific ID
            use crate::state::entities::{Conversation, ConversationType};
            let mut conversation = Conversation::new(company_id, ConversationType::Channel);
            conversation.id = conversation_id.to_string(); // Use the provided ID
            self.conversations.create(&conversation).await?;

            tracing::info!("Created conversation: {}", conversation_id);
        }

        Ok(())
    }

    /// Send a broadcast message to all departments
    pub async fn broadcast_message(
        &self,
        from_agent: Option<String>,
        content: impl Into<String>,
        msg_type: MessageType,
    ) -> Result<Message> {
        self.message_bus
            .broadcast_to_all(from_agent, content, msg_type)
            .await
    }

    /// Send a direct message between two agents/departments
    pub async fn send_direct_message(
        &self,
        from_agent: Option<String>,
        to_agent: Option<String>,
        content: impl Into<String>,
        msg_type: MessageType,
    ) -> Result<Message> {
        self.message_bus
            .send_dm(from_agent, to_agent, content, msg_type)
            .await
    }

    /// Subscribe to real-time messages for a conversation
    pub async fn subscribe_to_conversation(
        &self,
        conversation_id: impl Into<String>,
    ) -> tokio::sync::broadcast::Receiver<Message> {
        self.message_bus.subscribe(conversation_id).await
    }

    /// Get messages for a conversation
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Message>> {
        self.messages
            .get_conversation_messages(conversation_id, pagination)
            .await
    }

    /// Get recent messages across all conversations
    pub async fn get_recent_messages(&self, limit: i64) -> Result<Vec<Message>> {
        let company_id = self.require_company().await?;
        self.messages.get_recent(&company_id, limit).await
    }

    // ==================== Backup Operations ====================

    pub async fn backup(&self) -> Result<std::path::PathBuf> {
        self.backup_manager.backup(self.db.path()).await
    }

    pub async fn restore(&self, backup_path: &Path) -> Result<()> {
        // Close current database connection
        self.db.close().await;

        // Restore from backup
        self.backup_manager
            .restore(backup_path, self.db.path())
            .await?;

        // Reconnect to database
        // Note: This would need to reinitialize the Database object
        Ok(())
    }

    pub async fn list_backups(&self) -> Result<Vec<crate::state::backup::BackupInfo>> {
        self.backup_manager.list_backups().await
    }

    /// Backfill task files for existing tasks that don't have them.
    /// This creates markdown files and updates the database.
    /// Returns the number of tasks that were backfilled.
    pub async fn backfill_task_files(&self) -> Result<usize> {
        StateManager::backfill_task_files_static_inner(&self.tasks, &self.task_file_manager).await
    }

    async fn backfill_task_files_static_inner(
        tasks_repo: &TaskRepository,
        task_file_manager: &TaskFileManager,
    ) -> Result<usize> {
        let tasks_without_files = tasks_repo.find_without_file_path().await?;
        let count = tasks_without_files.len();

        if count == 0 {
            return Ok(0);
        }

        info!("Backfilling {} tasks with markdown files", count);

        for task in tasks_without_files {
            let file_path = task_file_manager.create_task_file(&task)?;
            let mut updated_task = task.clone();
            updated_task.file_path = Some(file_path);
            tasks_repo.update(&updated_task).await?;
            info!("Created file for task {}", task.id);
        }

        info!("Successfully backfilled {} task files", count);
        Ok(count)
    }

    // ==================== Roadmap & Initiative Operations ====================

    pub fn roadmaps(&self) -> &RoadmapRepository {
        &self.roadmaps
    }

    pub fn initiatives(&self) -> &InitiativeRepository {
        &self.initiatives
    }

    pub fn meetings(&self) -> &MeetingRepository {
        &self.meetings
    }

    pub fn meeting_participants(&self) -> &MeetingParticipantRepository {
        &self.meeting_participants
    }

    pub fn agent_sessions(&self) -> &AgentSessionRepository {
        &self.agent_sessions
    }

    pub fn activities(&self) -> &ActivityRepository {
        &self.activities
    }

    pub fn tasks(&self) -> &TaskRepository {
        &self.tasks
    }

    pub fn initiative_file_manager(&self) -> &InitiativeFileManager {
        &self.initiative_file_manager
    }

    pub fn decision_file_manager(&self) -> &DecisionFileManager {
        &self.decision_file_manager
    }

    pub fn get_initiative_title_from_file(&self, initiative_id: &str) -> Option<String> {
        self.initiative_file_manager
            .get_title_from_file(initiative_id)
            .ok()
            .flatten()
    }

    pub fn open_decision_editor_by_id(&self, decision_id: &str) -> Result<()> {
        self.decision_file_manager.open_by_id(decision_id)
    }

    pub async fn create_roadmap(
        &self,
        name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Roadmap> {
        let company_id = self.require_company().await?;
        let roadmap = Roadmap::new(&company_id, name, period_start, period_end);
        self.roadmaps.create(&roadmap).await
    }

    pub async fn get_active_roadmap(&self) -> Result<Option<Roadmap>> {
        let company_id = self.require_company().await?;
        self.roadmaps.find_active(&company_id).await
    }

    pub async fn create_initiative(
        &self,
        roadmap_id: &str,
        department_id: &str,
        title: &str,
    ) -> Result<Initiative> {
        let mut initiative = Initiative::new(roadmap_id, department_id, title);

        let file_path = self
            .initiative_file_manager
            .create_initiative_file(&initiative)?;
        initiative.file_path = Some(file_path);

        self.initiatives.create(&initiative).await
    }

    pub async fn create_initiative_with_id(
        &self,
        roadmap_id: &str,
        department_id: &str,
        title: &str,
        initiative_id: &str,
    ) -> Result<Initiative> {
        let mut initiative =
            Initiative::new_with_id(roadmap_id, department_id, title, initiative_id);

        let file_path = self
            .initiative_file_manager
            .create_initiative_file(&initiative)?;
        initiative.file_path = Some(file_path);

        self.initiatives.create(&initiative).await
    }

    pub async fn create_initiative_with_severity(
        &self,
        roadmap_id: &str,
        department_id: &str,
        title: &str,
        severity: DecisionSeverity,
        stakeholders: Vec<String>,
    ) -> Result<Initiative> {
        let mut initiative = Initiative::new(roadmap_id, department_id, title)
            .with_severity(severity)
            .with_stakeholders(stakeholders);

        let file_path = self
            .initiative_file_manager
            .create_initiative_file(&initiative)?;
        initiative.file_path = Some(file_path);

        self.initiatives.create(&initiative).await
    }

    pub async fn update_initiative(&self, initiative: &Initiative) -> Result<Initiative> {
        self.initiatives.update(initiative).await
    }

    pub async fn transition_initiative_to_proposed(
        &self,
        initiative_id: &str,
    ) -> Result<Initiative> {
        let mut initiative = self
            .initiatives
            .find_by_id(initiative_id)
            .await?
            .context("Initiative not found")?;

        let previous_status = Some(initiative.status.as_str().to_string());
        initiative.transition_to(InitiativeStatus::Proposed);

        // If HIGH severity, create a CEO decision
        if initiative.severity.requires_approval() {
            let company_id = self.require_company().await?;
            let mut decision = Decision::new(
                &company_id,
                format!("Approve initiative: {}", initiative.title),
            );
            decision.severity = initiative.severity;
            decision.initiative_id = Some(initiative.id.clone());
            decision.description = initiative.description.clone();
            decision.department_id = Some(initiative.department_id.clone());

            let decision = self.decisions.create(&decision).await?;
            let decision_id = decision.id.clone();
            initiative.pending_decision_id = Some(decision_id.clone());

            // Notify CEO about the new decision
            let _ = self
                .message_bus
                .notify_decision_for_approval(
                    None,
                    &decision_id,
                    initiative.severity.as_str(),
                    &initiative.title,
                )
                .await;
        }

        self.initiatives.update(&initiative).await?;

        // Broadcast initiative status change
        let _ = self
            .message_bus
            .broadcast_initiative_update(
                None,
                &initiative.id,
                &initiative.title,
                initiative.status.as_str(),
                previous_status.as_deref(),
            )
            .await;

        Ok(initiative)
    }

    pub async fn approve_initiative_decision(
        &self,
        initiative_id: &str,
        decision_id: &str,
    ) -> Result<Initiative> {
        let mut initiative = self
            .initiatives
            .find_by_id(initiative_id)
            .await?
            .context("Initiative not found")?;

        // Approve the decision
        self.decisions.approve(decision_id, "ceo", None).await?;

        let previous_status = Some(initiative.status.as_str().to_string());
        // Transition to Approved
        initiative.transition_to(InitiativeStatus::Approved);
        initiative.pending_decision_id = None;

        self.initiatives.update(&initiative).await?;

        // Broadcast initiative status change
        let _ = self
            .message_bus
            .broadcast_initiative_update(
                None,
                &initiative.id,
                &initiative.title,
                initiative.status.as_str(),
                previous_status.as_deref(),
            )
            .await;

        Ok(initiative)
    }

    pub async fn reject_initiative_decision(
        &self,
        initiative_id: &str,
        decision_id: &str,
    ) -> Result<Initiative> {
        let mut initiative = self
            .initiatives
            .find_by_id(initiative_id)
            .await?
            .context("Initiative not found")?;

        // Reject the decision
        self.decisions.reject(decision_id, "ceo", None).await?;

        let previous_status = Some(initiative.status.as_str().to_string());
        // Transition to Cancelled
        initiative.transition_to(InitiativeStatus::Cancelled);
        initiative.pending_decision_id = None;

        self.initiatives.update(&initiative).await?;

        // Broadcast initiative status change
        let _ = self
            .message_bus
            .broadcast_initiative_update(
                None,
                &initiative.id,
                &initiative.title,
                initiative.status.as_str(),
                previous_status.as_deref(),
            )
            .await;

        Ok(initiative)
    }

    pub async fn start_initiative(&self, initiative_id: &str) -> Result<Initiative> {
        let mut initiative = self
            .initiatives
            .find_by_id(initiative_id)
            .await?
            .context("Initiative not found")?;

        let previous_status = Some(initiative.status.as_str().to_string());
        initiative.transition_to(InitiativeStatus::Active);
        self.initiatives.update(&initiative).await?;

        // Broadcast initiative status change
        let _ = self
            .message_bus
            .broadcast_initiative_update(
                None,
                &initiative.id,
                &initiative.title,
                initiative.status.as_str(),
                previous_status.as_deref(),
            )
            .await;

        Ok(initiative)
    }

    pub async fn complete_initiative(&self, initiative_id: &str) -> Result<Initiative> {
        let mut initiative = self
            .initiatives
            .find_by_id(initiative_id)
            .await?
            .context("Initiative not found")?;

        let previous_status = Some(initiative.status.as_str().to_string());
        initiative.transition_to(InitiativeStatus::Completed);
        self.initiatives.update(&initiative).await?;

        // Broadcast initiative status change
        let _ = self
            .message_bus
            .broadcast_initiative_update(
                None,
                &initiative.id,
                &initiative.title,
                initiative.status.as_str(),
                previous_status.as_deref(),
            )
            .await;

        Ok(initiative)
    }

    pub async fn create_meeting(
        &self,
        title: &str,
        meeting_type: MeetingType,
        scheduled_at: DateTime<Utc>,
    ) -> Result<Meeting> {
        let company_id = self.require_company().await?;
        let meeting = Meeting::new(&company_id, title, meeting_type, scheduled_at);
        self.meetings.create(&meeting).await
    }

    pub async fn add_meeting_participant(
        &self,
        meeting_id: &str,
        department_id: &str,
        role: &str,
    ) -> Result<MeetingParticipant> {
        let participant = MeetingParticipant::new(meeting_id, department_id, role);
        self.meeting_participants.create(&participant).await
    }

    pub async fn get_pending_meeting_invitations(
        &self,
        department_id: &str,
    ) -> Result<Vec<MeetingParticipant>> {
        self.meeting_participants
            .find_pending_invitations(department_id)
            .await
    }

    pub async fn request_stakeholder_review(&self, initiative_id: &str) -> Result<Meeting> {
        let initiative = self
            .initiatives
            .find_by_id(initiative_id)
            .await?
            .context("Initiative not found")?;

        let stakeholder_depts: Vec<String> = initiative
            .stakeholder_depts_json
            .as_ref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        let meeting = self
            .create_meeting(
                &format!("Stakeholder Review: {}", initiative.title),
                MeetingType::StakeholderReview,
                Utc::now(),
            )
            .await?;

        for dept_slug in &stakeholder_depts {
            if let Some(dept) = self.get_department_by_slug(dept_slug).await? {
                self.add_meeting_participant(&meeting.id, &dept.id, "required")
                    .await?;
                self.send_message(
                    &format!("dept-{}", dept_slug),
                    None,
                    None,
                    &format!(
                        "Request for stakeholder review: {} - please review and provide feedback",
                        initiative.title
                    ),
                    MessageType::Notification,
                )
                .await?;
            }
        }

        info!(
            "Created stakeholder review meeting: {} with {} participants",
            meeting.title,
            stakeholder_depts.len()
        );
        Ok(meeting)
    }

    // ==================== Utility ====================

    pub fn database(&self) -> &Database {
        &self.db
    }

    pub async fn health_check(&self) -> Result<bool> {
        self.db.health_check().await
    }

    /// Clean up resources
    pub async fn shutdown(&self) {
        self.db.close().await;
    }
}

pub use crate::state::repositories::decision_repo::DecisionStats;
pub use crate::state::repositories::Pagination;
