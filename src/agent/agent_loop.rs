use crate::agent::ai_client::OpenCodeClient;
use crate::state::{
    entities::{
        ActivityLogEntry, Agent, Initiative, Meeting, MeetingParticipant, Roadmap, Task, TaskStatus,
    },
    StateManager,
};
use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub agent: Agent,
    pub department_id: String,
    pub department_slug: String,
}

impl AgentContext {
    pub fn new(agent: Agent, department_slug: String) -> Self {
        Self {
            department_id: agent.department_id.clone(),
            department_slug,
            agent,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AgentDecision {
    PickUpTask(String),
    RespondToMessage(String),
    AttendMeeting(String),
    UpdateTaskStatus {
        task_id: String,
        status: TaskStatus,
    },
    CreateSubtask {
        parent_id: String,
        title: String,
    },
    RequestRefinement {
        initiative_id: String,
    },
    RequestStakeholderReview {
        initiative_id: String,
        departments: Vec<String>,
    },
    Wait,
    Shutdown,
}

pub struct AgentLoop {
    state: StateManager,
    context: Arc<RwLock<Option<AgentContext>>>,
}

impl AgentLoop {
    pub fn new(state: StateManager) -> Self {
        Self {
            state,
            context: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_context(&self, context: AgentContext) {
        let mut ctx = self.context.write().await;
        *ctx = Some(context);
    }

    pub async fn get_context(&self) -> Option<AgentContext> {
        let ctx = self.context.read().await;
        ctx.clone()
    }

    async fn log_activity(
        &self,
        action: &str,
        target_type: Option<&str>,
        target_id: Option<&str>,
        details: Option<&str>,
    ) -> Result<()> {
        let ctx = match self.get_context().await {
            Some(ctx) => ctx,
            None => return Ok(()),
        };

        let company_id = match self.state.current_company().await {
            Some(id) => id,
            None => return Ok(()),
        };
        let entry = ActivityLogEntry {
            id: 0,
            company_id,
            timestamp: Utc::now(),
            actor: ctx.agent.name.clone(),
            actor_type: "ironclaw".to_string(),
            action: action.to_string(),
            target_type: target_type.map(|s| s.to_string()),
            target_id: target_id.map(|s| s.to_string()),
            details_json: details.map(|s| s.to_string()),
        };

        self.state.activities().log(&entry).await?;
        Ok(())
    }

    pub async fn run_loop(&self) -> Result<()> {
        info!("Starting agent loop");

        'agent_loop: loop {
            let decision = self.decide_what_to_do().await?;

            match decision {
                AgentDecision::Shutdown => {
                    info!("Agent shutting down");
                    break 'agent_loop;
                }
                AgentDecision::Wait => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
                _ => {
                    self.execute_decision(decision).await?;
                }
            }
        }

        Ok(())
    }

    async fn decide_what_to_do(&self) -> Result<AgentDecision> {
        let ctx = match self.get_context().await {
            Some(ctx) => ctx,
            None => return Ok(AgentDecision::Wait),
        };

        if let Some(pending_invitation) = self
            .check_pending_meeting_invitations(&ctx.department_id)
            .await?
        {
            return Ok(AgentDecision::AttendMeeting(pending_invitation));
        }

        if let Some(msg) = self.check_for_mentions(&ctx.agent.id).await? {
            return Ok(AgentDecision::RespondToMessage(msg));
        }

        if let Some((_, task_id)) = self.pick_up_available_task(&ctx.department_id).await? {
            return Ok(AgentDecision::PickUpTask(task_id));
        }

        Ok(AgentDecision::Wait)
    }

    async fn execute_decision(&self, decision: AgentDecision) -> Result<()> {
        match decision {
            AgentDecision::PickUpTask(task_id) => {
                self.work_on_task(&task_id).await?;
            }
            AgentDecision::UpdateTaskStatus { task_id, status } => {
                self.update_task_status(&task_id, status).await?;
            }
            AgentDecision::AttendMeeting(meeting_id) => {
                self.handle_meeting_response(&meeting_id, "accepted", None)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn pick_up_available_task(
        &self,
        department_id: &str,
    ) -> Result<Option<(Task, String)>> {
        let tasks = self
            .state
            .get_unassigned_department_tasks(department_id, TaskStatus::Todo)
            .await?;

        if let Some(task) = tasks.into_iter().next() {
            let mut updated_task = task.clone();
            updated_task.status = TaskStatus::InProgress;
            let ctx = self.get_context().await.context("Agent context not set")?;
            updated_task.assigned_to = Some(ctx.agent.id.clone());
            self.state.tasks().update(&updated_task).await?;
            info!(
                "Picked up task: {} (dept: {}, initiative: {:?})",
                task.title, department_id, task.initiative_id
            );
            return Ok(Some((updated_task, task.id)));
        }

        Ok(None)
    }

    pub async fn work_on_task(&self, task_id: &str) -> Result<()> {
        info!("Working on task: {}", task_id);

        let task = self.state.tasks().find_by_id(task_id).await?;
        if let Some(task) = task {
            let ctx = self.get_context().await.context("Agent context not set")?;

            if let Some(client) = OpenCodeClient::from_config()? {
                let project_dir = std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| ".".to_string());

                match client
                    .spawn_task(task_id, &task.title, &project_dir, &ctx.department_slug)
                    .await
                {
                    Ok(pid) => {
                        info!("Spawned opencode for task {} with PID {}", task_id, pid);

                        let _ = self
                            .log_activity(
                                "spawned_agent",
                                Some("task"),
                                Some(task_id),
                                Some(&format!("Agent started working on: {}", task.title)),
                            )
                            .await;

                        let agent_session = crate::state::entities::AgentSession::new(
                            task_id,
                            &ctx.department_id,
                            "pending_capture",
                        );
                        self.state.agent_sessions().create(&agent_session).await?;

                        let agent_session_id = agent_session.id.clone();
                        let captured_pid = pid;
                        let task_id_clone = task_id.to_string();

                        let state = self.state.clone();
                        tokio::spawn(async move {
                            if let Some(session_id) = client
                                .capture_session_id(&task_id_clone)
                                .await
                                .ok()
                                .flatten()
                            {
                                if let Err(e) = state
                                    .agent_sessions()
                                    .update_session_id(&agent_session_id, &session_id)
                                    .await
                                {
                                    warn!("Failed to update session ID: {}", e);
                                }
                                if let Err(e) = state
                                    .agent_sessions()
                                    .update_process_id(&agent_session_id, captured_pid)
                                    .await
                                {
                                    warn!("Failed to update process ID: {}", e);
                                }
                                info!("Captured session {} for task {}", session_id, task_id_clone);
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Failed to spawn opencode: {}", e);
                    }
                }
            } else {
                warn!("opencode not available - skipping AI task execution");
            }

            if task.status == TaskStatus::InProgress {
                let mut updated_task = task.clone();
                updated_task.status = TaskStatus::Review;
                self.state.tasks().update(&updated_task).await?;
                info!("Task {} moved to Review", task_id);
            }
        }

        Ok(())
    }

    pub async fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<()> {
        let task = self.state.tasks().find_by_id(task_id).await?;
        if let Some(mut task) = task {
            task.status = status;
            self.state.tasks().update(&task).await?;
            info!("Task {} status updated to {:?}", task_id, status);
        }
        Ok(())
    }

    pub async fn check_pending_meeting_invitations(
        &self,
        department_id: &str,
    ) -> Result<Option<String>> {
        let invitations = self
            .state
            .meeting_participants()
            .find_pending_invitations(department_id)
            .await?;

        if let Some(invitation) = invitations.into_iter().next() {
            return Ok(Some(invitation.meeting_id));
        }

        Ok(None)
    }

    pub async fn handle_meeting_response(
        &self,
        meeting_id: &str,
        status: &str,
        message: Option<&str>,
    ) -> Result<()> {
        let participants = self
            .state
            .meeting_participants()
            .find_by_meeting(meeting_id)
            .await?;
        let ctx = self.get_context().await;
        let my_department_id = ctx.map(|c| c.department_id.clone()).unwrap_or_default();

        if let Some(participant) = participants
            .iter()
            .find(|p| p.department_id == my_department_id)
        {
            self.state
                .meeting_participants()
                .update_response(&participant.id, status, message)
                .await?;
            info!("Meeting {} response: {}", meeting_id, status);
        }

        Ok(())
    }

    pub async fn check_for_mentions(&self, _agent_id: &str) -> Result<Option<String>> {
        // TODO: Implement message mention detection
        Ok(None)
    }

    pub async fn request_stakeholder_review(
        &self,
        initiative_id: &str,
        stakeholder_departments: Vec<String>,
    ) -> Result<Meeting> {
        let initiative = self.state.initiatives().find_by_id(initiative_id).await?;

        if let Some(initiative) = initiative {
            let _ctx = self.get_context().await.context("Agent context not set")?;

            let meeting = self
                .state
                .create_meeting(
                    &format!("Stakeholder Review: {}", initiative.title),
                    crate::state::entities::MeetingType::StakeholderReview,
                    Utc::now(),
                )
                .await?;

            for dept_slug in &stakeholder_departments {
                if let Some(dept) = self.state.get_department_by_slug(dept_slug).await? {
                    let participant = MeetingParticipant::new(&meeting.id, &dept.id, "required");
                    self.state
                        .meeting_participants()
                        .create(&participant)
                        .await?;
                }
            }

            info!("Created stakeholder review meeting: {}", meeting.title);
            return Ok(meeting);
        }

        anyhow::bail!("Initiative not found: {}", initiative_id)
    }
}

#[allow(async_fn_in_trait)]
pub trait AgentTools {
    fn state(&self) -> &StateManager;

    async fn list_tasks(
        &self,
        department_id: &str,
        status: Option<TaskStatus>,
    ) -> Result<Vec<Task>>;

    async fn create_task(
        &self,
        department_id: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<Task>;

    async fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<()>;

    async fn send_message(&self, to_department: &str, content: &str) -> Result<()>;

    async fn get_initiatives(&self, department_id: &str) -> Result<Vec<Initiative>>;

    async fn get_roadmap(&self, company_id: &str) -> Result<Option<Roadmap>>;

    async fn create_initiative(
        &self,
        roadmap_id: &str,
        department_id: &str,
        title: &str,
        stakeholders: Vec<String>,
    ) -> Result<Initiative>;
}

impl AgentTools for AgentLoop {
    fn state(&self) -> &StateManager {
        &self.state
    }

    async fn list_tasks(
        &self,
        department_id: &str,
        status: Option<TaskStatus>,
    ) -> Result<Vec<Task>> {
        match status {
            Some(s) => {
                self.state
                    .get_department_tasks_by_status(department_id, s)
                    .await
            }
            None => self.state.get_department_tasks(department_id).await,
        }
    }

    async fn create_task(
        &self,
        department_id: &str,
        title: &str,
        _description: Option<&str>,
    ) -> Result<Task> {
        let task = self.state.create_task(department_id, title).await?;
        Ok(task)
    }

    async fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<()> {
        if let Some(mut task) = self.state().tasks().find_by_id(task_id).await? {
            task.status = status;
            self.state().tasks().update(&task).await?;
        }
        Ok(())
    }

    async fn send_message(&self, to_department: &str, content: &str) -> Result<()> {
        let conversation_id = format!("dept-{}", to_department);
        self.state()
            .send_message(
                &conversation_id,
                None,
                None,
                content,
                crate::state::entities::MessageType::Chat,
            )
            .await?;
        Ok(())
    }

    async fn get_initiatives(&self, department_id: &str) -> Result<Vec<Initiative>> {
        self.state()
            .initiatives()
            .find_by_department(department_id)
            .await
    }

    async fn get_roadmap(&self, company_id: &str) -> Result<Option<Roadmap>> {
        self.state().roadmaps().find_active(company_id).await
    }

    async fn create_initiative(
        &self,
        roadmap_id: &str,
        department_id: &str,
        title: &str,
        stakeholders: Vec<String>,
    ) -> Result<Initiative> {
        let mut initiative = Initiative::new(roadmap_id, department_id, title);
        initiative.stakeholder_depts_json = Some(serde_json::to_string(&stakeholders)?);
        self.state().initiatives().create(&initiative).await?;
        Ok(initiative)
    }
}
