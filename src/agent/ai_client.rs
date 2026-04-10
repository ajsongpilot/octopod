use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct OpenCodeClient {
    _timeout_secs: u64,
}

impl OpenCodeClient {
    pub fn new(timeout_secs: u64) -> Self {
        Self { _timeout_secs: timeout_secs }
    }

    pub fn from_config() -> Result<Option<Self>> {
        if !Self::is_available() {
            return Ok(None);
        }
        Ok(Some(Self::new(300)))
    }

    pub fn is_available() -> bool {
        Command::new("which")
            .arg("opencode")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn read_cortex_file(path: &Path) -> String {
        std::fs::read_to_string(path)
            .map(|s| format!("\n\n{}:\n{}\n", path.file_name().unwrap().to_string_lossy(), s))
            .unwrap_or_default()
    }

    fn build_context_prompt(project_dir: &str, department_slug: &str) -> String {
        let base = Path::new(project_dir).join(".octopod").join("cortex");
        
        let company_context = Self::read_cortex_file(&base.join("company").join("OVERVIEW.md"));
        
        let dept_context = Self::read_cortex_file(&base.join(department_slug).join("CONTEXT.md"));
        
        if company_context.is_empty() && dept_context.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n# Company & Department Context{}\n\nPlease read the context above and apply it when working on tasks.",
                company_context
            )
        }
    }

    pub async fn spawn_task(
        &self,
        task_id: &str,
        task_title: &str,
        project_dir: &str,
        department_slug: &str,
    ) -> Result<u32> {
        let title = format!("octopod:task_{}:{}", task_id, task_title);
        
        let context_prompt = Self::build_context_prompt(project_dir, department_slug);
        
        let prompt = format!(
            "You are working on this task: {}.{}Focus on completing it.",
            task_title,
            context_prompt
        );
        
        let child = tokio::process::Command::new("opencode")
            .args(["run", "--title", &title])
            .arg(prompt)
            .current_dir(project_dir)
            .spawn()
            .context("Failed to spawn opencode")?;

        let pid = child.id().context("Failed to get PID")?;
        info!("Spawned opencode task {} with PID {}", task_id, pid);
        
        Ok(pid)
    }

    pub async fn capture_session_id(&self, task_id: &str) -> Result<Option<String>> {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let output = Command::new("opencode")
            .args(["session", "list", "--format", "json"])
            .output()
            .context("Failed to list sessions")?;

        let sessions: Vec<SessionInfo> = serde_json::from_slice(&output.stdout)
            .context("Failed to parse sessions")?;

        let pattern = format!("octopod:task_{}:", task_id);
        let session = sessions.iter().find(|s| s.title.starts_with(&pattern));
        
        Ok(session.map(|s| s.id.clone()))
    }

    pub async fn list_octopod_sessions(&self, project_dir: &str) -> Result<Vec<SessionInfo>> {
        let output = Command::new("opencode")
            .args(["session", "list", "--format", "json"])
            .output()
            .context("Failed to list sessions")?;

        let all_sessions: Vec<SessionInfo> = serde_json::from_slice(&output.stdout)
            .context("Failed to parse sessions")?;

        let octopod_sessions: Vec<SessionInfo> = all_sessions
            .into_iter()
            .filter(|s| s.title.starts_with("octopod:"))
            .filter(|s| s.directory == project_dir)
            .collect();

        Ok(octopod_sessions)
    }

    pub fn kill_process(pid: u32) -> Result<()> {
        Command::new("kill")
            .arg(pid.to_string())
            .output()
            .context("Failed to kill process")?;
        Ok(())
    }

    pub fn is_process_running(pid: u32) -> bool {
        Command::new("ps")
            .args(["-p", &pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub updated: i64,
    pub created: i64,
    pub directory: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawSession {
    id: String,
    title: String,
    #[serde(rename = "updated")]
    updated: i64,
    #[serde(rename = "created")]
    created: i64,
    #[serde(rename = "projectId")]
    _project_id: String,
    directory: String,
}

impl From<RawSession> for SessionInfo {
    fn from(raw: RawSession) -> Self {
        Self {
            id: raw.id,
            title: raw.title,
            updated: raw.updated,
            created: raw.created,
            directory: raw.directory,
        }
    }
}
