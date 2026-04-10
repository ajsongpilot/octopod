use anyhow::{Context, Result};
use std::process::Command;

/// Check if we're currently inside a tmux session
fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Spawn a department as a new tmux window in the current session
/// Windows persist as part of the tmux session
pub async fn spawn_department(dept_id: &str, dept_name: &str, _workspace: u8) -> Result<()> {
    // Check if we're inside a tmux session
    if !is_inside_tmux() {
        // Not in tmux - fall back to creating a detached session
        let session_name = format!("octopod-{}", dept_id);

        let session_exists = Command::new("tmux")
            .args(["has-session", "-t", &session_name])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !session_exists {
            Command::new("tmux")
                .args(["new-session", "-d", "-s", &session_name, "-n", dept_name])
                .spawn()
                .context("Failed to create tmux session")?;
        }

        spawn_terminal_detached(&session_name).await?;
        return Ok(());
    }

    // We're inside tmux - create a new window in the current session
    let window_name = dept_name.to_string();

    // Check if window already exists
    let window_exists = Command::new("tmux")
        .args(["list-windows", "-F", "#W"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == window_name)
        })
        .unwrap_or(false);

    if window_exists {
        // Switch to existing window and launch department TUI if needed
        Command::new("tmux")
            .args(["select-window", "-t", &window_name])
            .spawn()?;

        // Send the command to start department TUI
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Command::new("tmux")
            .args([
                "send-keys",
                "-t",
                &window_name,
                &format!("octopod dept {}", dept_id),
                "C-m",
            ])
            .spawn()?;
    } else {
        // Create new window first
        Command::new("tmux")
            .args(["new-window", "-n", &window_name])
            .spawn()?;

        // Then send the command to start department TUI
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let project_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        Command::new("tmux")
            .args([
                "send-keys",
                "-t",
                &window_name,
                &format!("cd {} && octopod dept {}", project_dir, dept_id),
                "C-m",
            ])
            .spawn()?;
    }

    Ok(())
}

/// Kill a running department
pub async fn kill_department(_dept_id: &str, dept_name: &str) -> Result<()> {
    if is_inside_tmux() {
        // We're inside tmux - kill the window
        let window_name = dept_name.to_string();
        let _ = Command::new("tmux")
            .args(["kill-window", "-t", &window_name])
            .output();
    } else {
        // Not in tmux - kill the session
        let session_name = format!("octopod-{}", _dept_id);
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", &session_name])
            .output();
    }

    Ok(())
}

/// Check if a department is running
pub fn is_department_running(_dept_id: &str, dept_name: &str) -> bool {
    if is_inside_tmux() {
        // We're inside tmux - check for window
        let window_name = dept_name.to_string();
        Command::new("tmux")
            .args(["list-windows", "-F", "#W"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line == window_name)
            })
            .unwrap_or(false)
    } else {
        // Not in tmux - check for session
        let session_name = format!("octopod-{}", _dept_id);
        Command::new("tmux")
            .args(["has-session", "-t", &session_name])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Check if a department daemon is running
pub fn is_daemon_running(dept_id: &str) -> bool {
    let session_name = format!("octopod-{}-daemon", dept_id);
    Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Kill a department daemon
pub fn kill_daemon(dept_id: &str) -> Result<()> {
    let session_name = format!("octopod-{}-daemon", dept_id);
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session_name])
        .output();
    Ok(())
}

/// Spawn an agent daemon in a detached tmux session
pub fn spawn_agent_daemon(department: &str) -> Result<()> {
    let session_name = format!("octopod-{}-daemon", department);
    let project_dir = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let exists = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if exists {
        return Ok(());
    }

    let cmd = format!(
        "cd {} && exec octopod agent loop {}",
        project_dir, department
    );

    let output = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s", &session_name,
            "-n", &format!("{}-agent", department),
            "bash", "-c", &cmd,
        ])
        .output()
        .context("Failed to create tmux session for agent")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to create session: {}", stderr));
    }

    Ok(())
}

/// Get status of both TUI and daemon for a department
pub fn get_department_full_status(dept_id: &str, dept_name: &str) -> (bool, bool) {
    (is_department_running(dept_id, dept_name), is_daemon_running(dept_id))
}

/// Spawn all departments
pub async fn spawn_all() -> Result<()> {
    let departments = [
        ("product", "Product", 2u8),
        ("engineering", "Engineering", 3u8),
        ("qa", "QA", 4u8),
        ("finance", "Finance", 5u8),
        ("legal", "Legal", 6u8),
        ("devops", "DevOps", 7u8),
        ("marketing", "Marketing", 8u8),
        ("sales", "Sales", 9u8),
    ];

    for (id, name, workspace) in &departments {
        match spawn_department(id, name, *workspace).await {
            Ok(_) => println!("  ✓ {} TUI opened", name),
            Err(e) => eprintln!("  ✗ {} TUI failed: {}", name, e),
        }
    }

    Ok(())
}

async fn spawn_terminal_detached(session_name: &str) -> Result<()> {
    // Detect which terminal emulator to use
    let terminal = detect_terminal();

    // Build the attach command
    let _cmd = format!("tmux attach-session -t {}", session_name);

    match terminal.as_str() {
        "alacritty" => {
            std::process::Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "nohup alacritty -e bash -c 'tmux attach -t {}' > /dev/null 2>&1 &",
                        session_name
                    ),
                ])
                .spawn()?;
        }
        "kitty" => {
            std::process::Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "nohup kitty bash -c 'tmux attach -t {}' > /dev/null 2>&1 &",
                        session_name
                    ),
                ])
                .spawn()?;
        }
        "ghostty" => {
            std::process::Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "nohup ghostty -e bash -c 'tmux attach -t {}' > /dev/null 2>&1 &",
                        session_name
                    ),
                ])
                .spawn()?;
        }
        _ => {
            // Fallback: try alacritty
            std::process::Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "nohup alacritty -e bash -c 'tmux attach -t {}' > /dev/null 2>&1 &",
                        session_name
                    ),
                ])
                .spawn()?;
        }
    }

    Ok(())
}

fn detect_terminal() -> String {
    // Check for common terminals
    let terminals = ["alacritty", "kitty", "ghostty", "wezterm", "foot"];

    for term in &terminals {
        if Command::new("which")
            .arg(term)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return term.to_string();
        }
    }

    // Default fallback
    "alacritty".to_string()
}
