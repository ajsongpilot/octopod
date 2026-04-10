use crate::state::entities::MeetingParticipant;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct MeetingParticipantRepository {
    pool: SqlitePool,
}

impl MeetingParticipantRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, participant: &MeetingParticipant) -> Result<MeetingParticipant> {
        sqlx::query(
            r#"INSERT INTO meeting_participants (id, meeting_id, department_id, agent_id, role, status, response_message, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&participant.id)
        .bind(&participant.meeting_id)
        .bind(&participant.department_id)
        .bind(&participant.agent_id)
        .bind(&participant.role)
        .bind(&participant.status)
        .bind(&participant.response_message)
        .bind(participant.created_at)
        .execute(&self.pool)
        .await?;

        Ok(participant.clone())
    }

    pub async fn find_by_meeting(&self, meeting_id: &str) -> Result<Vec<MeetingParticipant>> {
        let participants = sqlx::query_as::<_, MeetingParticipant>(
            "SELECT * FROM meeting_participants WHERE meeting_id = ?",
        )
        .bind(meeting_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    pub async fn find_by_department(&self, department_id: &str) -> Result<Vec<MeetingParticipant>> {
        let participants = sqlx::query_as::<_, MeetingParticipant>(
            "SELECT * FROM meeting_participants WHERE department_id = ?",
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    pub async fn find_pending_invitations(
        &self,
        department_id: &str,
    ) -> Result<Vec<MeetingParticipant>> {
        let participants = sqlx::query_as::<_, MeetingParticipant>(
            r#"SELECT mp.* FROM meeting_participants mp
               INNER JOIN meetings m ON mp.meeting_id = m.id
               WHERE mp.department_id = ? AND mp.status = 'invited' AND m.status_id = 1
               ORDER BY m.scheduled_at ASC"#,
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    pub async fn update_response(
        &self,
        participant_id: &str,
        status: &str,
        response_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE meeting_participants SET status = ?, response_message = ? WHERE id = ?"#,
        )
        .bind(status)
        .bind(response_message)
        .bind(participant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
