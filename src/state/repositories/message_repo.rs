use crate::state::entities::Message;
use crate::state::repositories::{PaginatedResult, Pagination};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Repository for message operations
#[derive(Debug, Clone)]
pub struct MessageRepository {
    pool: SqlitePool,
}

impl MessageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new message
    pub async fn create(&self, message: &Message) -> Result<Message> {
        tracing::debug!(
            "Creating message: id={}, conversation_id={}, from_agent_id={:?}, content_len={}",
            message.id,
            message.conversation_id,
            message.from_agent_id,
            message.content.len()
        );

        sqlx::query(
            r#"
            INSERT INTO messages (
                id, conversation_id, from_agent_id, to_agent_id,
                content, message_type, metadata_json, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.from_agent_id)
        .bind(&message.to_agent_id)
        .bind(&message.content)
        .bind(message.message_type)
        .bind(&message.metadata_json)
        .bind(message.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("SQL error creating message: {:?}", e);
            anyhow::anyhow!("Database error: {}", e)
        })?;

        Ok(message.clone())
    }

    /// Get messages for a conversation
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages 
            WHERE conversation_id = ? 
            AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(conversation_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&self.pool)
        .await
        .context("Failed to get conversation messages")?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = ? AND deleted_at IS NULL",
        )
        .bind(conversation_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(messages, total, &pagination))
    }

    /// Get recent messages for a company (for activity feed)
    pub async fn get_recent(&self, company_id: &str, limit: i64) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT m.* FROM messages m
            JOIN conversations c ON m.conversation_id = c.id
            WHERE c.company_id = ?
            AND m.deleted_at IS NULL
            ORDER BY m.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(company_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get recent messages")?;

        Ok(messages)
    }
}
