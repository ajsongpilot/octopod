//! Agent abstraction layer
//!
//! This module provides traits and types for integrating different AI agents
//! (Ironclaw, Opencode, Claude Code, Aider, etc.) without tying Octopod
//! to any specific implementation.
//!
//! Design principles:
//! - Agents are external processes managed by Octopod
//! - Communication via standardized interface (stdin/stdout, HTTP, or files)
//! - No agent-specific code in core Octopod logic
//! - Agent identities stored in Octopod database, passed to agents as config

pub mod agent_loop;
pub mod ai_client;
pub mod runner;

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Agent backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentBackend {
    /// Ironclaw - NEAR AI's secure agent runtime
    Ironclaw,
    /// Opencode - Multi-agent coding assistant
    Opencode,
    /// Claude Code - Anthropic's coding agent
    ClaudeCode,
    /// Aider - GPT-powered coding assistant
    Aider,
    /// Custom - User-defined agent
    Custom,
}

impl AgentBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentBackend::Ironclaw => "ironclaw",
            AgentBackend::Opencode => "opencode",
            AgentBackend::ClaudeCode => "claude-code",
            AgentBackend::Aider => "aider",
            AgentBackend::Custom => "custom",
        }
    }

    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ironclaw" => Some(AgentBackend::Ironclaw),
            "opencode" => Some(AgentBackend::Opencode),
            "claude-code" | "claude" => Some(AgentBackend::ClaudeCode),
            "aider" => Some(AgentBackend::Aider),
            "custom" => Some(AgentBackend::Custom),
            _ => None,
        }
    }
}

/// Agent identity/personality
#[derive(Debug, Clone)]
pub struct AgentIdentity {
    /// Unique agent ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Role/title (e.g., "Senior Product Manager")
    pub role: String,
    /// Department affiliation
    pub department: String,
    /// Personality description
    pub personality: String,
    /// System prompt/context
    pub system_prompt: String,
    /// Allowed tools/capabilities
    pub capabilities: Vec<String>,
    /// Backend-specific configuration
    pub backend_config: HashMap<String, String>,
}

impl AgentIdentity {
    /// Create a department agent with default capabilities
    pub fn for_department(department: &str, name: &str, role: &str) -> Self {
        let dept_lower = department.to_lowercase();

        let (capabilities, system_prompt) = match dept_lower.as_str() {
            "product" => (
                vec!["read".to_string(), "write".to_string(), "github".to_string()],
                format!(
                    "You are {}, a Product Manager. Your job is to write PRDs, prioritize features, and communicate with Engineering and Design.",
                    name
                ),
            ),
            "engineering" => (
                vec!["read".to_string(), "write".to_string(), "bash".to_string(), "github".to_string()],
                format!(
                    "You are {}, a Software Engineer. Your job is to implement features, write tests, and maintain code quality.",
                    name
                ),
            ),
            "qa" => (
                vec!["read".to_string(), "write".to_string(), "bash".to_string(), "github".to_string()],
                format!(
                    "You are {}, a QA Engineer. Your job is to test features, find bugs, and ensure quality.",
                    name
                ),
            ),
            _ => (
                vec!["read".to_string(), "write".to_string()],
                format!("You are {}, working in {}.", name, department),
            ),
        };

        Self {
            id: format!("{}-agent", dept_lower),
            name: name.to_string(),
            role: role.to_string(),
            department: department.to_string(),
            personality: format!(
                "Professional {} focused on quality and collaboration.",
                role
            ),
            system_prompt,
            capabilities,
            backend_config: HashMap::new(),
        }
    }
}

/// Agent process handle
#[derive(Debug)]
pub struct AgentHandle {
    pub id: String,
    pub process_id: Option<u32>,
    pub backend: AgentBackend,
}

/// Trait for agent backend implementations
#[async_trait]
pub trait AgentBackendImpl: Send + Sync {
    /// Backend name
    fn name(&self) -> &str;

    /// Check if the backend binary is available
    async fn is_available(&self) -> Result<bool>;

    /// Spawn an agent with the given identity
    async fn spawn(&self, identity: AgentIdentity, workspace: u8) -> Result<AgentHandle>;

    /// Kill a running agent
    async fn kill(&self, handle: &AgentHandle) -> Result<()>;

    /// Check if agent is still running
    async fn is_running(&self, handle: &AgentHandle) -> Result<bool>;
}

/// Agent status
#[derive(Debug, Clone)]
pub enum AgentStatus {
    Idle,
    Working { task: String },
    Error(String),
    Offline,
}

/// Opencode backend implementation
pub struct OpencodeBackend;

impl Default for OpencodeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl OpencodeBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AgentBackendImpl for OpencodeBackend {
    fn name(&self) -> &str {
        "opencode"
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(which::which("opencode").is_ok())
    }

    async fn spawn(&self, identity: AgentIdentity, workspace: u8) -> Result<AgentHandle> {
        use crate::platform::spawn_manager::spawn_department;

        spawn_department(
            &identity.department.to_lowercase(),
            &identity.name,
            workspace,
        )
        .await?;

        Ok(AgentHandle {
            id: identity.id,
            process_id: None,
            backend: AgentBackend::Opencode,
        })
    }

    async fn kill(&self, handle: &AgentHandle) -> Result<()> {
        use crate::platform::spawn_manager::kill_department;
        let dept_id = handle.id.replace("-agent", "");
        let dept_name = if dept_id.is_empty() {
            dept_id.clone()
        } else {
            let mut chars = dept_id.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        };
        kill_department(&dept_id, &dept_name).await
    }

    async fn is_running(&self, handle: &AgentHandle) -> Result<bool> {
        use crate::platform::spawn_manager::is_department_running;
        let dept_id = handle.id.replace("-agent", "");
        let dept_name = if dept_id.is_empty() {
            dept_id.clone()
        } else {
            let mut chars = dept_id.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        };
        Ok(is_department_running(&dept_id, &dept_name))
    }
}
