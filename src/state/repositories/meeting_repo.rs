use crate::state::entities::Meeting;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct MeetingRepository {
    pool: SqlitePool,
}

impl MeetingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, meeting: &Meeting) -> Result<Meeting> {
        sqlx::query(
            r#"INSERT INTO meetings (id, company_id, title, meeting_type_id, status_id, scheduled_at, duration_minutes, agenda_json, notes, outcomes_json, related_initiative_id, related_roadmap_id, created_by, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&meeting.id)
        .bind(&meeting.company_id)
        .bind(&meeting.title)
        .bind(meeting.meeting_type)
        .bind(meeting.status)
        .bind(meeting.scheduled_at)
        .bind(meeting.duration_minutes)
        .bind(&meeting.agenda_json)
        .bind(&meeting.notes)
        .bind(&meeting.outcomes_json)
        .bind(&meeting.related_initiative_id)
        .bind(&meeting.related_roadmap_id)
        .bind(&meeting.created_by)
        .bind(meeting.created_at)
        .bind(meeting.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(meeting.clone())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Meeting>> {
        let meeting = sqlx::query_as::<_, Meeting>("SELECT * FROM meetings WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(meeting)
    }

    pub async fn find_by_company(&self, company_id: &str) -> Result<Vec<Meeting>> {
        let meetings = sqlx::query_as::<_, Meeting>(
            "SELECT * FROM meetings WHERE company_id = ? ORDER BY scheduled_at DESC",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(meetings)
    }

    pub async fn find_upcoming(&self, company_id: &str, limit: i64) -> Result<Vec<Meeting>> {
        let now = chrono::Utc::now().to_rfc3339();
        let meetings = sqlx::query_as::<_, Meeting>(
            r#"SELECT * FROM meetings 
               WHERE company_id = ? AND scheduled_at > ? AND status_id = 1
               ORDER BY scheduled_at ASC LIMIT ?"#,
        )
        .bind(company_id)
        .bind(now)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(meetings)
    }

    pub async fn find_by_initiative(&self, initiative_id: &str) -> Result<Vec<Meeting>> {
        let meetings = sqlx::query_as::<_, Meeting>(
            "SELECT * FROM meetings WHERE related_initiative_id = ? ORDER BY scheduled_at DESC",
        )
        .bind(initiative_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(meetings)
    }

    pub async fn find_by_department_participation(
        &self,
        department_id: &str,
    ) -> Result<Vec<Meeting>> {
        let meetings = sqlx::query_as::<_, Meeting>(
            r#"SELECT m.* FROM meetings m
               INNER JOIN meeting_participants mp ON m.id = mp.meeting_id
               WHERE mp.department_id = ?
               ORDER BY m.scheduled_at DESC"#,
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(meetings)
    }

    pub async fn update(&self, meeting: &Meeting) -> Result<Meeting> {
        let now = chrono::Utc::now();
        sqlx::query(
            r#"UPDATE meetings SET 
                title = ?, meeting_type_id = ?, status_id = ?, scheduled_at = ?,
                duration_minutes = ?, agenda_json = ?, notes = ?, outcomes_json = ?,
                related_initiative_id = ?, related_roadmap_id = ?, updated_at = ?
               WHERE id = ?"#,
        )
        .bind(&meeting.title)
        .bind(meeting.meeting_type)
        .bind(meeting.status)
        .bind(meeting.scheduled_at)
        .bind(meeting.duration_minutes)
        .bind(&meeting.agenda_json)
        .bind(&meeting.notes)
        .bind(&meeting.outcomes_json)
        .bind(&meeting.related_initiative_id)
        .bind(&meeting.related_roadmap_id)
        .bind(now)
        .bind(&meeting.id)
        .execute(&self.pool)
        .await?;

        Ok(meeting.clone())
    }
}
