use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use super::opencode;

#[async_trait]
pub trait CodingAgent: Send + Sync {
    fn name(&self) -> &str;
    async fn is_available(&self) -> Result<bool>;
    async fn spawn_session(&self, repo: &Path, task: &str) -> Result<CodingSession>;
}

#[derive(Debug, Clone)]
pub struct CodingSession {
    pub id: String,
}

pub struct CodingAgentFactory;

impl CodingAgentFactory {
    pub fn create(agent_type: CodingAgentType) -> Arc<dyn CodingAgent> {
        match agent_type {
            CodingAgentType::Opencode => Arc::new(opencode::OpencodeAgent::new()),
            _ => todo!("Other coding agents not yet implemented"),
        }
    }

    pub fn create_default() -> Arc<dyn CodingAgent> {
        Arc::new(opencode::OpencodeAgent::new())
    }
}

pub enum CodingAgentType {
    Opencode,
    Aider,
    ClaudeCode,
    Continue,
    Custom(String),
}
