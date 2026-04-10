use crate::state::entities::Roadmap;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct RoadmapRepository {
    pool: SqlitePool,
}

impl RoadmapRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, roadmap: &Roadmap) -> Result<Roadmap> {
        let status_str = match roadmap.status {
            crate::state::RoadmapStatus::Draft => "draft",
            crate::state::RoadmapStatus::Planning => "planning",
            crate::state::RoadmapStatus::Active => "active",
            crate::state::RoadmapStatus::Completed => "completed",
            crate::state::RoadmapStatus::Archived => "archived",
        };
        sqlx::query(
            r#"INSERT INTO roadmaps (id, company_id, name, description, period_start, period_end, status_id, goals_json, created_by, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&roadmap.id)
        .bind(&roadmap.company_id)
        .bind(&roadmap.name)
        .bind(&roadmap.description)
        .bind(roadmap.period_start)
        .bind(roadmap.period_end)
        .bind(status_str)
        .bind(&roadmap.goals_json)
        .bind(&roadmap.created_by)
        .bind(roadmap.created_at)
        .bind(roadmap.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(roadmap.clone())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Roadmap>> {
        let roadmap = sqlx::query_as::<_, Roadmap>("SELECT * FROM roadmaps WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(roadmap)
    }

    pub async fn find_by_company(&self, company_id: &str) -> Result<Vec<Roadmap>> {
        let roadmaps = sqlx::query_as::<_, Roadmap>(
            "SELECT * FROM roadmaps WHERE company_id = ? ORDER BY period_start DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(roadmaps)
    }

    pub async fn find_active(&self, company_id: &str) -> Result<Option<Roadmap>> {
        let roadmap = sqlx::query_as::<_, Roadmap>(
            "SELECT * FROM roadmaps WHERE company_id = ? AND status_id = 3 ORDER BY period_start DESC LIMIT 1"
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(roadmap)
    }

    pub async fn update(&self, roadmap: &Roadmap) -> Result<Roadmap> {
        let now = chrono::Utc::now();
        let status_str = match roadmap.status {
            crate::state::RoadmapStatus::Draft => "draft",
            crate::state::RoadmapStatus::Planning => "planning",
            crate::state::RoadmapStatus::Active => "active",
            crate::state::RoadmapStatus::Completed => "completed",
            crate::state::RoadmapStatus::Archived => "archived",
        };
        sqlx::query(
            r#"UPDATE roadmaps SET 
                name = ?, description = ?, period_start = ?, period_end = ?,
                status_id = ?, goals_json = ?, updated_at = ?
               WHERE id = ?"#,
        )
        .bind(&roadmap.name)
        .bind(&roadmap.description)
        .bind(roadmap.period_start)
        .bind(roadmap.period_end)
        .bind(status_str)
        .bind(&roadmap.goals_json)
        .bind(now)
        .bind(&roadmap.id)
        .execute(&self.pool)
        .await?;

        Ok(roadmap.clone())
    }
}
