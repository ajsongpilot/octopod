use super::types::{AgentConfig, AgentHandle, ClawBackend};
use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;
use tracing::{error, info};

pub struct IronclawBackend {
    _port: u16,
}

impl Default for IronclawBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl IronclawBackend {
    pub fn new() -> Self {
        Self { _port: 9876 }
    }

    /// Build the system prompt for a department agent
    fn build_system_prompt(department: &str, name: &str, role: &str) -> String {
        match department {
            "product" => format!(
                "You are {}, a {} in the Product Department. 
                
Your responsibilities:
- Write Product Requirements Documents (PRDs)
- Prioritize features and create roadmaps  
- Communicate with Engineering and Design
- Make decisions about what to build and why

You are part of an AI-powered software company called Octopod. You communicate with other departments via the message bus. When you receive a message, respond professionally and helpfully. You can request work from Engineering, feedback from QA, and designs from Design.

Current department: Product
Your role: {}

Use the octopod_send_message tool to communicate with other departments.
Use the octopod_list_tasks tool to see your assigned tasks.
Use the octopod_update_task tool to update task status.
", name, role, role),

            "engineering" => format!(
                "You are {}, a {} in the Engineering Department.

Your responsibilities:
- Implement features based on PRDs from Product
- Write tests and maintain code quality
- Deploy code to production
- Work with QA to fix bugs

You are part of an AI-powered software company called Octopod. You receive tasks from Product and coordinate with QA for testing. When assigned work, implement it thoroughly and update your progress.

Current department: Engineering  
Your role: {}

Use the octopod_send_message tool to communicate with other departments.
Use the octopod_get_task tool to retrieve your current task details.
Use the octopod_update_task tool to mark tasks as complete.
", name, role, role),

            "qa" => format!(
                "You are {}, a {} in the QA Department.

Your responsibilities:
- Test features implemented by Engineering
- Write test plans and automation
- Report bugs with clear reproduction steps
- Verify fixes and approve releases

You are part of an AI-powered software company called Octopod. You receive features to test from Engineering and report results.

Current department: QA
Your role: {}

Use the octopod_send_message tool to communicate with other departments.
Use the octopod_list_tasks tool to see testing assignments.
", name, role, role),

            _ => format!(
                "You are {}, a {} in the {} Department.

You are part of an AI-powered software company called Octopod. Work with other departments to achieve company goals.

Use the octopod_send_message tool to communicate with other departments.
", name, role, department),
        }
    }
}

#[async_trait]
impl ClawBackend for IronclawBackend {
    fn name(&self) -> &str {
        "ironclaw"
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(which::which("ironclaw").is_ok())
    }

    async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentHandle> {
        let department = config.department.to_lowercase();
        let agent_id = format!("{}-agent", department);

        info!("Spawning Ironclaw agent for {} department", department);

        // Build system prompt based on department
        let system_prompt =
            Self::build_system_prompt(&department, &config.name, &config.department);

        // Create a unique session name
        let session_name = format!("octopod-agent-{}-{}", department, std::process::id());

        // Spawn ironclaw in a tmux session with custom config
        let ironclaw_cmd = format!(
            "ironclaw run --system-prompt '{}' --channel octopod-{} 2>&1",
            shell_escape::escape(system_prompt.into()),
            department
        );

        let mut cmd = Command::new("tmux");
        cmd.args(["new-session", "-d", "-s", &session_name, "-n", &config.name]);
        cmd.arg(ironclaw_cmd);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to spawn tmux session: {}", stderr);
            anyhow::bail!("Failed to spawn Ironclaw agent: {}", stderr);
        }

        info!("Spawned Ironclaw agent in tmux session: {}", session_name);

        // Get the PID of the ironclaw process
        // We need to find the actual ironclaw process in the tmux session
        let pid = find_process_in_session(&session_name).await?;

        Ok(AgentHandle { id: agent_id, pid })
    }
}

/// Find the ironclaw process PID in a tmux session
async fn find_process_in_session(session_name: &str) -> Result<u32> {
    // Give the process a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Get the pane PID
    let output = Command::new("tmux")
        .args(["list-panes", "-t", session_name, "-F", "#{pane_pid}"])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to get tmux pane PID");
    }

    let pane_pid_str = String::from_utf8_lossy(&output.stdout);
    let pane_pid: u32 = pane_pid_str.trim().parse()?;

    // Find the ironclaw child process
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    // Look for ironclaw process that is a descendant of the pane PID
    for (pid, process) in system.processes() {
        if process.name().eq_ignore_ascii_case("ironclaw") {
            // Check if this process is a descendant of our tmux pane
            let pid_u32 = pid.as_u32();
            if is_descendant_of(pid_u32, pane_pid, &system) {
                return Ok(pid_u32);
            }
        }
    }

    // Fallback: return the pane PID
    Ok(pane_pid)
}

/// Check if a process is a descendant of another process
fn is_descendant_of(pid: u32, ancestor: u32, system: &sysinfo::System) -> bool {
    use sysinfo::Pid;
    let mut current = Pid::from_u32(pid);
    let ancestor_pid = Pid::from_u32(ancestor);

    while let Some(process) = system.processes().get(&current) {
        if let Some(parent) = process.parent() {
            if parent == ancestor_pid {
                return true;
            }
            current = parent;
        } else {
            break;
        }
    }

    false
}
