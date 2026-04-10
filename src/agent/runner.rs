use crate::state::{entities::Agent, StateManager};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct AgentSchedule {
    pub department_id: String,
    pub department_slug: String,
    pub interval_secs: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub enabled: bool,
}

impl AgentSchedule {
    pub fn new(department_slug: &str, interval_secs: u64) -> Self {
        let next = Utc::now() + Duration::seconds(interval_secs as i64);
        Self {
            department_id: String::new(),
            department_slug: department_slug.to_string(),
            interval_secs,
            last_run: None,
            next_run: Some(next),
            enabled: true,
        }
    }

    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(next) = self.next_run {
            return Utc::now() >= next;
        }
        true
    }

    pub fn mark_run(&mut self) {
        self.last_run = Some(Utc::now());
        self.next_run = Some(Utc::now() + Duration::seconds(self.interval_secs as i64));
    }
}

#[derive(Debug, Clone)]
pub struct RunningAgent {
    pub department_slug: String,
    pub started_at: DateTime<Utc>,
    pub status: AgentRunStatus,
}

#[derive(Debug, Clone)]
pub enum AgentRunStatus {
    Starting,
    Running,
    Completed { success: bool, message: String },
    Error(String),
}

pub struct AgentRunner {
    state: StateManager,
    schedules: Arc<RwLock<HashMap<String, AgentSchedule>>>,
    running_agents: Arc<RwLock<HashMap<String, RunningAgent>>>,
}

impl AgentRunner {
    pub fn new(state: StateManager) -> Self {
        Self {
            state,
            schedules: Arc::new(RwLock::new(HashMap::new())),
            running_agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_schedule(&self, department_slug: &str, interval_secs: u64) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        let dept = self
            .state
            .get_department_by_slug(department_slug)
            .await?
            .context("Department not found")?;

        let mut schedule = AgentSchedule::new(department_slug, interval_secs);
        schedule.department_id = dept.id;
        schedules.insert(department_slug.to_string(), schedule);

        info!(
            "Added schedule for {}: every {} seconds",
            department_slug, interval_secs
        );
        Ok(())
    }

    pub async fn remove_schedule(&self, department_slug: &str) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        schedules.remove(department_slug);
        info!("Removed schedule for {}", department_slug);
        Ok(())
    }

    pub async fn enable_schedule(&self, department_slug: &str) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        if let Some(schedule) = schedules.get_mut(department_slug) {
            schedule.enabled = true;
            schedule.next_run = Some(Utc::now());
            info!("Enabled schedule for {}", department_slug);
        }
        Ok(())
    }

    pub async fn disable_schedule(&self, department_slug: &str) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        if let Some(schedule) = schedules.get_mut(department_slug) {
            schedule.enabled = false;
            info!("Disabled schedule for {}", department_slug);
        }
        Ok(())
    }

    pub async fn list_schedules(&self) -> Vec<AgentSchedule> {
        let schedules = self.schedules.read().await;
        schedules.values().cloned().collect()
    }

    pub async fn list_running_agents(&self) -> Vec<RunningAgent> {
        let running = self.running_agents.read().await;
        running.values().cloned().collect()
    }

    async fn run_agent(&self, department_slug: &str) -> Result<()> {
        let dept = self
            .state
            .get_department_by_slug(department_slug)
            .await?
            .context("Department not found")?;

        let agent = Agent::new(&dept.id, format!("{}-agent", department_slug));
        let context =
            crate::agent::agent_loop::AgentContext::new(agent, department_slug.to_string());

        let loop_ = crate::agent::agent_loop::AgentLoop::new(self.state.clone());
        loop_.set_context(context).await;

        info!("Starting agent loop for {}", department_slug);
        loop_.run_loop().await?;

        Ok(())
    }

    pub async fn run_once(&self, department_slug: &str) -> Result<()> {
        let mut running = self.running_agents.write().await;
        running.insert(
            department_slug.to_string(),
            RunningAgent {
                department_slug: department_slug.to_string(),
                started_at: Utc::now(),
                status: AgentRunStatus::Starting,
            },
        );
        drop(running);

        {
            let mut running = self.running_agents.write().await;
            if let Some(agent) = running.get_mut(department_slug) {
                agent.status = AgentRunStatus::Running;
            }
        }

        match self.run_agent(department_slug).await {
            Ok(()) => {
                let mut running = self.running_agents.write().await;
                if let Some(agent) = running.get_mut(department_slug) {
                    agent.status = AgentRunStatus::Completed {
                        success: true,
                        message: "Agent loop completed normally".to_string(),
                    };
                }
                info!("Agent for {} completed successfully", department_slug);
            }
            Err(e) => {
                let mut running = self.running_agents.write().await;
                if let Some(agent) = running.get_mut(department_slug) {
                    agent.status = AgentRunStatus::Error(e.to_string());
                }
                error!("Agent for {} failed: {}", department_slug, e);
            }
        }

        Ok(())
    }

    pub async fn run_scheduled(&self) {
        info!("Starting agent scheduler");

        let tick_interval = tokio::time::Duration::from_secs(10);
        let mut ticker = interval(tick_interval);

        loop {
            ticker.tick().await;

            let schedules_to_run: Vec<String> = {
                let schedules = self.schedules.read().await;
                schedules
                    .values()
                    .filter(|s| s.should_run())
                    .map(|s| s.department_slug.clone())
                    .collect()
            };

            for dept_slug in schedules_to_run {
                {
                    let mut schedules = self.schedules.write().await;
                    if let Some(schedule) = schedules.get_mut(&dept_slug) {
                        schedule.mark_run();
                    }
                }

                let runner = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = runner.run_once(&dept_slug).await {
                        warn!("Agent run for {} failed: {}", dept_slug, e);
                    }
                });
            }
        }
    }

    pub async fn run_forever(&self, department_slug: &str, interval_secs: u64) {
        info!(
            "Starting agent {} with {}s interval",
            department_slug, interval_secs
        );

        let mut ticker = interval(tokio::time::Duration::from_secs(interval_secs));

        loop {
            ticker.tick().await;

            info!("Running agent for {}", department_slug);
            if let Err(e) = self.run_once(department_slug).await {
                error!("Agent for {} failed: {}", department_slug, e);
            }
        }
    }

    pub async fn wait_for_completion(&self, timeout_secs: u64) -> Result<()> {
        let start = Utc::now();
        let timeout = Duration::seconds(timeout_secs as i64);

        loop {
            let running = {
                let agents = self.running_agents.read().await;
                agents
                    .values()
                    .filter(|a| {
                        matches!(a.status, AgentRunStatus::Running | AgentRunStatus::Starting)
                    })
                    .count()
            };

            if running == 0 {
                return Ok(());
            }

            if Utc::now() - start > timeout {
                anyhow::bail!("Timeout waiting for agents to complete");
            }

            sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

impl Clone for AgentRunner {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            schedules: self.schedules.clone(),
            running_agents: self.running_agents.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_schedule_new() {
        let schedule = AgentSchedule::new("engineering", 300);
        assert_eq!(schedule.department_slug, "engineering");
        assert_eq!(schedule.interval_secs, 300);
        assert!(schedule.enabled);
        assert!(schedule.last_run.is_none());
        assert!(schedule.next_run.is_some());
    }

    #[test]
    fn test_agent_schedule_mark_run() {
        let mut schedule = AgentSchedule::new("engineering", 300);
        let original_next = schedule.next_run;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        schedule.mark_run();
        
        assert!(schedule.last_run.is_some());
        assert!(schedule.next_run.is_some());
        assert!(schedule.next_run > original_next);
    }

    #[test]
    fn test_agent_run_status_variants() {
        let starting = AgentRunStatus::Starting;
        let running = AgentRunStatus::Running;
        let completed = AgentRunStatus::Completed { 
            success: true, 
            message: "Done".to_string() 
        };
        let error = AgentRunStatus::Error("Failed".to_string());
        
        match starting {
            AgentRunStatus::Starting => {},
            _ => panic!("Expected Starting"),
        }
        
        match running {
            AgentRunStatus::Running => {},
            _ => panic!("Expected Running"),
        }
        
        match completed {
            AgentRunStatus::Completed { success, message } => {
                assert!(success);
                assert_eq!(message, "Done");
            },
            _ => panic!("Expected Completed"),
        }
        
        match error {
            AgentRunStatus::Error(msg) => {
                assert_eq!(msg, "Failed");
            },
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_agent_schedule_partial_clone() {
        let schedule = AgentSchedule::new("engineering", 300);
        let schedule_clone = AgentSchedule {
            department_id: schedule.department_id.clone(),
            department_slug: schedule.department_slug.clone(),
            interval_secs: schedule.interval_secs,
            last_run: schedule.last_run,
            next_run: schedule.next_run,
            enabled: schedule.enabled,
        };
        
        assert_eq!(schedule.department_slug, schedule_clone.department_slug);
        assert_eq!(schedule.interval_secs, schedule_clone.interval_secs);
    }
}
