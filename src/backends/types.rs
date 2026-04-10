use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use super::ironclaw;

#[async_trait]
pub trait ClawBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn is_available(&self) -> Result<bool>;
    async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentHandle>;
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub department: String,
}

#[derive(Debug, Clone)]
pub struct AgentHandle {
    pub id: String,
    pub pid: u32,
}

pub struct BackendFactory;

impl BackendFactory {
    pub fn create(backend_type: BackendType) -> Arc<dyn ClawBackend> {
        match backend_type {
            BackendType::Ironclaw => Arc::new(ironclaw::IronclawBackend::new()),
            BackendType::Openclaw => todo!("OpenClaw backend not yet implemented"),
        }
    }

    pub fn create_default() -> Arc<dyn ClawBackend> {
        Arc::new(ironclaw::IronclawBackend::new())
    }
}

pub enum BackendType {
    Ironclaw,
    Openclaw,
}
