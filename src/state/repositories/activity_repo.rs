use crate::state::entities::ActivityLogEntry;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct ActivityRepository {
    pool: SqlitePool,
}

impl ActivityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn log(&self, entry: &ActivityLogEntry) -> Result<ActivityLogEntry> {
        sqlx::query(
            "INSERT INTO activity_log (company_id, timestamp, actor, actor_type, action, target_type, target_id, details_json) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&entry.company_id)
        .bind(entry.timestamp)
        .bind(&entry.actor)
        .bind(&entry.actor_type)
        .bind(&entry.action)
        .bind(&entry.target_type)
        .bind(&entry.target_id)
        .bind(&entry.details_json)
        .execute(&self.pool)
        .await?;

        Ok(entry.clone())
    }

    pub async fn get_recent(&self, company_id: &str, limit: i64) -> Result<Vec<ActivityLogEntry>> {
        let entries = sqlx::query_as::<_, ActivityLogEntry>(
            "SELECT * FROM activity_log WHERE company_id = ? ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(company_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }
}
