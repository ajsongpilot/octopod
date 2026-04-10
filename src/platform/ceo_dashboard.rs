use anyhow::{Context, Result};
use std::process::Command;

/// Spawn the CEO Dashboard as a tmux multi-pane layout in workspace 1
pub async fn spawn_ceo_dashboard() -> Result<()> {
    let session_name = "octopod-ceo";

    // Check if already exists
    let session_exists = Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if session_exists {
        // Just attach to existing session
        return attach_to_dashboard(session_name).await;
    }

    // Create new tmux session with multi-pane layout
    // Layout:
    // +------------------+------------------+
    // | Departments      |   Activity Feed  |
    // | (list view)      |   (scrollback)   |
    // |                  |                  |
    // +------------------+------------------+
    // |        Issues Board / Logs          |
    // |         (bottom panel)              |
    // +-------------------------------------+

    // Create base session
    Command::new("tmux")
        .args(["new-session", "-d", "-s", session_name, "-n", "Dashboard"])
        .spawn()
        .context("Failed to create CEO dashboard session")?;

    // Split into left/right (departments | activity)
    Command::new("tmux")
        .args(["split-window", "-h", "-t", &format!("{}:0.0", session_name)])
        .output()?;

    // Split bottom panel for issues/logs
    Command::new("tmux")
        .args(["split-window", "-v", "-t", &format!("{}:0.0", session_name)])
        .output()?;

    // Resize bottom panel to be smaller (30% height)
    Command::new("tmux")
        .args([
            "resize-pane",
            "-t",
            &format!("{}:0.2", session_name),
            "-y",
            "30%",
        ])
        .output()?;

    // Set up pane titles
    Command::new("tmux")
        .args([
            "select-pane",
            "-t",
            &format!("{}:0.0", session_name),
            "-T",
            "Departments",
        ])
        .output()?;

    Command::new("tmux")
        .args([
            "select-pane",
            "-t",
            &format!("{}:0.1", session_name),
            "-T",
            "Activity",
        ])
        .output()?;

    Command::new("tmux")
        .args([
            "select-pane",
            "-t",
            &format!("{}:0.2", session_name),
            "-T",
            "Issues",
        ])
        .output()?;

    // Start the dashboard app in the departments pane (pane 0.0)
    Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}:0.0", session_name),
            "octopod dashboard-ui",
            "C-m",
        ])
        .output()?;

    // Show logs in activity pane initially (pane 0.1)
    Command::new("tmux")
        .args([
            "send-keys",
            "-t", &format!("{}:0.1", session_name),
            "tail -f /tmp/octopod-ceo.log 2>/dev/null || echo 'Activity feed will appear here...' && read",
            "C-m",
        ])
        .output()?;

    // Show recent git issues in bottom pane (pane 0.2)
    Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}:0.2", session_name),
            "gh issue list --limit 10 2>/dev/null || echo 'Connect GitHub to see issues' && read",
            "C-m",
        ])
        .output()?;

    // Attach to the session
    attach_to_dashboard(session_name).await?;

    Ok(())
}

async fn attach_to_dashboard(session_name: &str) -> Result<()> {
    // Check if we're already in a tmux session
    let in_tmux = std::env::var("TMUX").is_ok();

    if in_tmux {
        // Switch to the dashboard session
        Command::new("tmux")
            .args(["switch-client", "-t", session_name])
            .spawn()?;
    } else {
        // Attach to the session
        Command::new("tmux")
            .args(["attach-session", "-t", session_name])
            .spawn()?;
    }

    Ok(())
}

/// Create a simple dashboard UI for the left pane (departments list)
/// This runs inside tmux, showing real-time status
pub async fn run_dashboard_ui() -> Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::time::Duration;

    enable_raw_mode()?;

    println!("🐙 Octopod CEO Dashboard - Departments\n");
    println!("Press 'q' to exit, 's' to spawn, arrows to navigate\n");
    println!("─────────────────────────────────────\n");

    let departments = [
        ("Super+2", "Product", "Roadmap, PRDs", "⏹ Stopped"),
        ("Super+3", "Engineering", "Feature dev", "⏹ Stopped"),
        ("Super+4", "QA", "Testing", "⏹ Stopped"),
        ("Super+5", "Finance", "Budgeting", "⏹ Stopped"),
        ("Super+6", "Legal", "Contracts", "⏹ Stopped"),
        ("Super+7", "DevOps", "Infrastructure", "⏹ Stopped"),
        ("Super+8", "Marketing", "Campaigns", "⏹ Stopped"),
        ("Super+9", "Sales", "Revenue", "⏹ Stopped"),
    ];

    let mut selected = 0;

    loop {
        // Clear screen and redraw
        print!("\x1b[2J\x1b[H"); // Clear screen and move cursor to top
        println!("🐙 Octopod CEO Dashboard - Departments\n");
        println!("Press 'q' to exit, 's' to spawn, arrows to navigate\n");
        println!("─────────────────────────────────────\n");

        for (i, (ws, name, desc, status)) in departments.iter().enumerate() {
            let marker = if i == selected { ">>> " } else { "    " };
            let color = match *status {
                "▶ Running" => "\x1b[32m",   // Green
                "🔄 Starting" => "\x1b[33m", // Yellow
                _ => "\x1b[90m",             // Gray
            };
            let reset = "\x1b[0m";
            println!(
                "{}{} {:8} {:12} {:15} {}{}",
                marker, color, ws, name, desc, status, reset
            );
        }

        // Check for input with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => selected = (selected + 1) % departments.len(),
                    KeyCode::Up => {
                        selected = if selected == 0 {
                            departments.len() - 1
                        } else {
                            selected - 1
                        }
                    }
                    KeyCode::Char('s') => {
                        let dept_names = [
                            "product",
                            "engineering",
                            "qa",
                            "finance",
                            "legal",
                            "devops",
                            "marketing",
                            "sales",
                        ];
                        let dept_name = dept_names[selected];
                        let _ = Command::new("octopod").args(["spawn", dept_name]).spawn();
                        println!("\nSpawning {}...", dept_name);
                        std::thread::sleep(Duration::from_millis(500));
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}

/// Switch focus between dashboard panes
pub fn switch_dashboard_pane(direction: &str) -> Result<()> {
    let session_name = "octopod-ceo";

    let target = match direction {
        "left" => "-L",
        "right" => "-R",
        "up" => "-U",
        "down" => "-D",
        _ => "-L",
    };

    Command::new("tmux")
        .args(["select-pane", "-t", session_name, target])
        .spawn()?;

    Ok(())
}
