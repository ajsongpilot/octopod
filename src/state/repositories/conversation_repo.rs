use crate::state::entities::Conversation;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct ConversationRepository {
    pool: SqlitePool,
}

impl ConversationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, conversation: &Conversation) -> Result<Conversation> {
        sqlx::query(
            "INSERT INTO conversations (id, company_id, department_id, title, conversation_type, parent_message_id, created_at, archived_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&conversation.id)
        .bind(&conversation.company_id)
        .bind(&conversation.department_id)
        .bind(&conversation.title)
        .bind(conversation.conversation_type)
        .bind(&conversation.parent_message_id)
        .bind(conversation.created_at)
        .bind(conversation.archived_at)
        .execute(&self.pool)
        .await?;

        Ok(conversation.clone())
    }
}
