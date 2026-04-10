use crate::state::entities::AgentSession;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AgentSessionRepository {
    pool: SqlitePool,
}

impl AgentSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, session: &AgentSession) -> Result<AgentSession> {
        sqlx::query(
            "INSERT INTO agent_sessions (id, task_id, department_id, session_id, process_id, status, created_at, ended_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&session.id)
        .bind(&session.task_id)
        .bind(&session.department_id)
        .bind(&session.session_id)
        .bind(session.process_id)
        .bind(&session.status)
        .bind(session.created_at)
        .bind(session.ended_at)
        .execute(&self.pool)
        .await?;

        Ok(session.clone())
    }

    pub async fn find_by_task(&self, task_id: &str) -> Result<Option<AgentSession>> {
        let session = sqlx::query_as::<_, AgentSession>(
            "SELECT * FROM agent_sessions WHERE task_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(session)
    }

    pub async fn find_by_department(&self, department_id: &str) -> Result<Vec<AgentSession>> {
        let sessions = sqlx::query_as::<_, AgentSession>(
            "SELECT * FROM agent_sessions WHERE department_id = ? ORDER BY created_at DESC",
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }

    pub async fn find_active_by_department(
        &self,
        department_id: &str,
    ) -> Result<Vec<AgentSession>> {
        let sessions = sqlx::query_as::<_, AgentSession>(
            "SELECT * FROM agent_sessions WHERE department_id = ? AND status = 'active' ORDER BY created_at DESC"
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }

    pub async fn update_status(&self, id: &str, status: &str) -> Result<()> {
        if status == "stopped" || status == "completed" {
            sqlx::query(
                "UPDATE agent_sessions SET status = ?, ended_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query("UPDATE agent_sessions SET status = ? WHERE id = ?")
                .bind(status)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn update_session_id(&self, id: &str, session_id: &str) -> Result<()> {
        sqlx::query("UPDATE agent_sessions SET session_id = ? WHERE id = ?")
            .bind(session_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_process_id(&self, id: &str, process_id: u32) -> Result<()> {
        sqlx::query("UPDATE agent_sessions SET process_id = ? WHERE id = ?")
            .bind(process_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
