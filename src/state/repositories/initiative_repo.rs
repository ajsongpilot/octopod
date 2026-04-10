use crate::state::entities::{Initiative, InitiativeStatus};
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct InitiativeRepository {
    pool: SqlitePool,
}

impl InitiativeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, initiative: &Initiative) -> Result<Initiative> {
        let status_id = match initiative.status {
            crate::state::InitiativeStatus::Draft => 1,
            crate::state::InitiativeStatus::Proposed => 2,
            crate::state::InitiativeStatus::StakeholderReview => 3,
            crate::state::InitiativeStatus::Approved => 4,
            crate::state::InitiativeStatus::Active => 5,
            crate::state::InitiativeStatus::Completed => 6,
            crate::state::InitiativeStatus::Cancelled => 7,
            crate::state::InitiativeStatus::Closed => 8,
            crate::state::InitiativeStatus::Archived => 9,
        };
        sqlx::query(
            r#"INSERT INTO initiatives (id, roadmap_id, department_id, title, description, status_id, priority, severity, stakeholder_depts_json, pending_decision_id, created_by, created_at, updated_at, file_path)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&initiative.id)
        .bind(&initiative.roadmap_id)
        .bind(&initiative.department_id)
        .bind(&initiative.title)
        .bind(&initiative.description)
        .bind(status_id)
        .bind(format!("{:?}", initiative.priority).to_lowercase())
        .bind(format!("{:?}", initiative.severity).to_lowercase())
        .bind(&initiative.stakeholder_depts_json)
        .bind(&initiative.pending_decision_id)
        .bind(&initiative.created_by)
        .bind(initiative.created_at)
        .bind(initiative.updated_at)
        .bind(&initiative.file_path)
        .execute(&self.pool)
        .await?;

        Ok(initiative.clone())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Initiative>> {
        let initiative = sqlx::query_as::<_, Initiative>("SELECT * FROM initiatives WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(initiative)
    }

    pub async fn find_by_roadmap(&self, roadmap_id: &str) -> Result<Vec<Initiative>> {
        let initiatives = sqlx::query_as::<_, Initiative>(
            "SELECT * FROM initiatives WHERE roadmap_id = ? ORDER BY priority ASC, created_at DESC",
        )
        .bind(roadmap_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(initiatives)
    }

    pub async fn find_by_department(&self, department_id: &str) -> Result<Vec<Initiative>> {
        let initiatives = sqlx::query_as::<_, Initiative>(
            "SELECT * FROM initiatives WHERE department_id = ? ORDER BY priority ASC, created_at DESC"
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(initiatives)
    }

    pub async fn find_pending_stakeholder_review(
        &self,
        department_id: &str,
    ) -> Result<Vec<Initiative>> {
        let initiatives = sqlx::query_as::<_, Initiative>(
            r#"SELECT * FROM initiatives 
               WHERE stakeholder_depts_json LIKE ? AND status_id = 2
               ORDER BY priority ASC, created_at DESC"#,
        )
        .bind(format!("%{}%", department_id))
        .fetch_all(&self.pool)
        .await?;

        Ok(initiatives)
    }

    pub async fn find_by_status(&self, status: InitiativeStatus) -> Result<Vec<Initiative>> {
        let initiatives = sqlx::query_as::<_, Initiative>(
            "SELECT * FROM initiatives WHERE status_id = ? ORDER BY priority ASC, created_at DESC",
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(initiatives)
    }

    pub async fn find_active(&self) -> Result<Vec<Initiative>> {
        let initiatives = sqlx::query_as::<_, Initiative>(
            "SELECT * FROM initiatives WHERE status_id IN (4, 5) ORDER BY priority ASC, created_at DESC"  // approved=4, active=5
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(initiatives)
    }

    pub async fn find_awaiting_ceo_decision(&self) -> Result<Vec<Initiative>> {
        let initiatives = sqlx::query_as::<_, Initiative>(
            r#"SELECT * FROM initiatives 
               WHERE pending_decision_id IS NOT NULL 
               AND status_id IN (2, 3)  -- proposed=2, stakeholder_review=3
               ORDER BY severity DESC, priority ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(initiatives)
    }

    pub async fn update(&self, initiative: &Initiative) -> Result<Initiative> {
        let now = chrono::Utc::now();
        let status_id = match initiative.status {
            crate::state::InitiativeStatus::Draft => 1,
            crate::state::InitiativeStatus::Proposed => 2,
            crate::state::InitiativeStatus::StakeholderReview => 3,
            crate::state::InitiativeStatus::Approved => 4,
            crate::state::InitiativeStatus::Active => 5,
            crate::state::InitiativeStatus::Completed => 6,
            crate::state::InitiativeStatus::Cancelled => 7,
            crate::state::InitiativeStatus::Closed => 8,
            crate::state::InitiativeStatus::Archived => 9,
        };
        sqlx::query(
            r#"UPDATE initiatives SET 
                title = ?, description = ?, status_id = ?, priority = ?,
                severity = ?, stakeholder_depts_json = ?, pending_decision_id = ?,
                updated_at = ?, file_path = ?
               WHERE id = ?"#,
        )
        .bind(&initiative.title)
        .bind(&initiative.description)
        .bind(status_id)
        .bind(format!("{:?}", initiative.priority).to_lowercase())
        .bind(format!("{:?}", initiative.severity).to_lowercase())
        .bind(&initiative.stakeholder_depts_json)
        .bind(&initiative.pending_decision_id)
        .bind(now)
        .bind(&initiative.file_path)
        .bind(&initiative.id)
        .execute(&self.pool)
        .await?;

        Ok(initiative.clone())
    }
}
