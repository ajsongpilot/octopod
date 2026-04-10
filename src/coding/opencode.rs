use super::types::{CodingAgent, CodingSession};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

pub struct OpencodeAgent;

impl Default for OpencodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl OpencodeAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CodingAgent for OpencodeAgent {
    fn name(&self) -> &str {
        "opencode"
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(which::which("opencode").is_ok())
    }

    async fn spawn_session(&self, _repo: &Path, _task: &str) -> Result<CodingSession> {
        todo!("Spawn opencode session")
    }
}
