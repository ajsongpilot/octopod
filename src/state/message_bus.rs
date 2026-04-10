use crate::state::entities::{
    DecisionMessageMetadata, InitiativeMessageMetadata, MeetingRequestMetadata, Message,
    MessageType, TaskAssignmentMetadata,
};
use crate::state::repositories::message_repo::MessageRepository;
use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};

/// Message bus for inter-department communication
#[derive(Debug, Clone)]
pub struct MessageBus {
    repo: MessageRepository,
    /// Broadcast channels for real-time updates
    /// Key: conversation_id, Value: broadcast sender
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
}

impl MessageBus {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: MessageRepository::new(pool),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Send a message
    ///
    /// # Arguments
    /// * `conversation_id` - The conversation/channel (e.g., "general", "product-engineering")
    /// * `from_agent` - Who is sending (None for system/CEO messages)
    /// * `to_agent` - Who should receive (None = broadcast to conversation)
    /// * `content` - The message content
    /// * `msg_type` - Type of message
    pub async fn send(
        &self,
        conversation_id: impl Into<String>,
        from_agent: Option<String>,
        to_agent: Option<String>,
        content: impl Into<String>,
        msg_type: MessageType,
    ) -> Result<Message> {
        let conversation_id = conversation_id.into();
        let content = content.into();

        // Create message
        let mut message = Message::new(&conversation_id, content);
        message.from_agent_id = from_agent;
        message.to_agent_id = to_agent;
        message.message_type = msg_type;

        // Save to database
        self.repo.create(&message).await?;

        // Broadcast to any listening clients
        if let Err(e) = self.broadcast(&conversation_id, &message).await {
            error!("Failed to broadcast message: {}", e);
            // Don't fail the send if broadcast fails
        }

        info!(
            "Message sent: {} -> {} (conversation: {})",
            message.from_agent_id.as_deref().unwrap_or("system"),
            message.to_agent_id.as_deref().unwrap_or("all"),
            conversation_id
        );

        Ok(message)
    }

    /// Send a broadcast message to all departments
    pub async fn broadcast_to_all(
        &self,
        from_agent: Option<String>,
        content: impl Into<String>,
        msg_type: MessageType,
    ) -> Result<Message> {
        self.send("general", from_agent, None, content, msg_type)
            .await
    }

    /// Send a direct message between two departments
    pub async fn send_dm(
        &self,
        from_agent: Option<String>,
        to_agent: Option<String>,
        content: impl Into<String>,
        msg_type: MessageType,
    ) -> Result<Message> {
        let conversation_id = match (&from_agent, &to_agent) {
            (Some(from), Some(to)) => format!("dm-{}-{}", from, to),
            _ => "dm-unknown".to_string(),
        };

        self.send(conversation_id, from_agent, to_agent, content, msg_type)
            .await
    }

    /// Get messages for a conversation
    pub async fn get_messages(&self, conversation_id: &str, limit: i64) -> Result<Vec<Message>> {
        use crate::state::repositories::Pagination;

        let pagination = Pagination::new(1, limit as u32);
        let result = self
            .repo
            .get_conversation_messages(conversation_id, pagination)
            .await?;

        Ok(result.items)
    }

    /// Subscribe to real-time messages for a conversation
    pub async fn subscribe(
        &self,
        conversation_id: impl Into<String>,
    ) -> broadcast::Receiver<Message> {
        let conversation_id = conversation_id.into();
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(&conversation_id) {
            sender.subscribe()
        } else {
            // Create new channel
            drop(channels);
            let (tx, rx) = broadcast::channel(100);
            let mut channels = self.channels.write().await;
            channels.insert(conversation_id, tx);
            rx
        }
    }

    /// Broadcast message to all subscribers
    async fn broadcast(&self, conversation_id: &str, message: &Message) -> Result<()> {
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(conversation_id) {
            // Ignore send errors (receiver may have dropped)
            let _ = sender.send(message.clone());
        }

        Ok(())
    }

    /// Get recent messages across all conversations for activity feed
    pub async fn get_recent_activity(&self, company_id: &str, limit: i64) -> Result<Vec<Message>> {
        self.repo.get_recent(company_id, limit).await
    }

    /// Send a decision proposal message
    pub async fn send_decision_proposal(
        &self,
        conversation_id: &str,
        from_agent: Option<String>,
        decision_id: &str,
        severity: &str,
        title: &str,
    ) -> Result<Message> {
        let metadata = DecisionMessageMetadata {
            decision_id: decision_id.to_string(),
            severity: severity.to_string(),
            title: title.to_string(),
        };
        let content = format!("Decision proposal: {} (severity: {})", title, severity);

        let mut message = Message::new(conversation_id, content);
        message.from_agent_id = from_agent;
        message.message_type = MessageType::DecisionProposal;
        message.metadata_json = Some(serde_json::to_string(&metadata)?);

        self.repo.create(&message).await?;
        self.broadcast(conversation_id, &message).await?;

        Ok(message)
    }

    /// Send an initiative update message
    pub async fn send_initiative_update(
        &self,
        conversation_id: &str,
        from_agent: Option<String>,
        initiative_id: &str,
        initiative_title: &str,
        status: &str,
        previous_status: Option<&str>,
    ) -> Result<Message> {
        let metadata = InitiativeMessageMetadata {
            initiative_id: initiative_id.to_string(),
            initiative_title: initiative_title.to_string(),
            status: status.to_string(),
            previous_status: previous_status.map(|s| s.to_string()),
        };
        let content = format!(
            "Initiative '{}' status changed: {} -> {}",
            initiative_title,
            previous_status.unwrap_or("none"),
            status
        );

        let mut message = Message::new(conversation_id, content);
        message.from_agent_id = from_agent;
        message.message_type = MessageType::InitiativeUpdate;
        message.metadata_json = Some(serde_json::to_string(&metadata)?);

        self.repo.create(&message).await?;
        self.broadcast(conversation_id, &message).await?;

        Ok(message)
    }

    /// Send a task assignment message
    pub async fn send_task_assignment(
        &self,
        conversation_id: &str,
        from_agent: Option<String>,
        task_id: &str,
        task_title: &str,
        department_id: &str,
        assigned_to: Option<&str>,
    ) -> Result<Message> {
        let metadata = TaskAssignmentMetadata {
            task_id: task_id.to_string(),
            task_title: task_title.to_string(),
            department_id: department_id.to_string(),
            assigned_to: assigned_to.map(|s| s.to_string()),
        };
        let assignee = assigned_to.unwrap_or("unassigned");
        let content = format!("Task assigned: '{}' -> {}", task_title, assignee);

        let mut message = Message::new(conversation_id, content);
        message.from_agent_id = from_agent;
        message.message_type = MessageType::TaskAssignment;
        message.metadata_json = Some(serde_json::to_string(&metadata)?);

        self.repo.create(&message).await?;
        self.broadcast(conversation_id, &message).await?;

        Ok(message)
    }

    /// Send a meeting request message
    pub async fn send_meeting_request(
        &self,
        conversation_id: &str,
        from_agent: Option<String>,
        meeting_id: &str,
        meeting_title: &str,
        meeting_type: &str,
        requested_by: &str,
    ) -> Result<Message> {
        let metadata = MeetingRequestMetadata {
            meeting_id: meeting_id.to_string(),
            meeting_title: meeting_title.to_string(),
            meeting_type: meeting_type.to_string(),
            requested_by: requested_by.to_string(),
        };
        let content = format!("Meeting request: {} ({})", meeting_title, meeting_type);

        let mut message = Message::new(conversation_id, content);
        message.from_agent_id = from_agent;
        message.message_type = MessageType::MeetingRequest;
        message.metadata_json = Some(serde_json::to_string(&metadata)?);

        self.repo.create(&message).await?;
        self.broadcast(conversation_id, &message).await?;

        Ok(message)
    }

    /// Broadcast initiative update to all stakeholders
    pub async fn broadcast_initiative_update(
        &self,
        from_agent: Option<String>,
        initiative_id: &str,
        initiative_title: &str,
        status: &str,
        previous_status: Option<&str>,
    ) -> Result<Message> {
        self.send_initiative_update(
            "initiatives",
            from_agent,
            initiative_id,
            initiative_title,
            status,
            previous_status,
        )
        .await
    }

    /// Notify about a decision needing CEO approval
    pub async fn notify_decision_for_approval(
        &self,
        from_agent: Option<String>,
        decision_id: &str,
        severity: &str,
        title: &str,
    ) -> Result<Message> {
        self.send_decision_proposal("ceo-queue", from_agent, decision_id, severity, title)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        // This would need a test database setup
        // Skipping for now as we'd need to mock the repo
    }
}
