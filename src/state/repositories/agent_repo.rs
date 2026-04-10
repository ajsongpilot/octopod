use crate::state::entities::Agent;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AgentRepository {
    pool: SqlitePool,
}

impl AgentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, agent: &Agent) -> Result<Agent> {
        sqlx::query(
            "INSERT INTO agents (id, department_id, name, role, personality, model, config_json, status, current_task_id, created_at, last_seen_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&agent.id)
        .bind(&agent.department_id)
        .bind(&agent.name)
        .bind(&agent.role)
        .bind(&agent.personality)
        .bind(&agent.model)
        .bind(&agent.config_json)
        .bind(agent.status)
        .bind(&agent.current_task_id)
        .bind(agent.created_at)
        .bind(agent.last_seen_at)
        .execute(&self.pool)
        .await?;

        Ok(agent.clone())
    }

    pub async fn find_by_department(&self, department_id: &str) -> Result<Vec<Agent>> {
        let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE department_id = ?")
            .bind(department_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(agents)
    }
}
