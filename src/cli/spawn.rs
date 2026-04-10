use crate::platform::spawn_manager::{kill_department, spawn_all, spawn_department};
use anyhow::{Context, Result};
use std::process::Command;

/// Spawn departments with agent daemons
///
/// Usage:
///   octopod spawn all
///   octopod spawn product
///   octopod spawn product engineering qa
pub async fn run(all: bool, departments: Vec<String>) -> Result<()> {
    let valid_depts = [
        "product",
        "engineering",
        "qa",
        "devops",
        "marketing",
        "sales",
        "finance",
        "legal",
    ];

    if all {
        println!("🐙 Spawning all departments with agents...");
        spawn_all().await?;
        spawn_all_agents().await?;
    } else {
        // Validate and spawn specific departments
        let invalid: Vec<_> = departments
            .iter()
            .filter(|d| !valid_depts.contains(&d.to_lowercase().as_str()))
            .collect();

        if !invalid.is_empty() {
            eprintln!("Error: Invalid department(s): {:?}", invalid);
            eprintln!("Valid departments are: {}", valid_depts.join(", "));
            std::process::exit(1);
        }

        println!("🐙 Spawning: {:?} with agent daemons", departments);
        for dept_id in &departments {
            // Map department ID to name and workspace
            let (name, workspace) = match dept_id.as_str() {
                "product" => ("Product", 2u8),
                "engineering" => ("Engineering", 3u8),
                "qa" => ("QA", 4u8),
                "finance" => ("Finance", 5u8),
                "legal" => ("Legal", 6u8),
                "devops" => ("DevOps", 7u8),
                "marketing" => ("Marketing", 8u8),
                "sales" => ("Sales", 9u8),
                _ => continue,
            };

            match spawn_department(dept_id, name, workspace).await {
                Ok(_) => println!("  ✓ {} TUI opened", name),
                Err(e) => eprintln!("  ✗ {} TUI failed: {}", name, e),
            }

            // Start agent daemon
            match spawn_agent_daemon(dept_id) {
                Ok(_) => println!("  ✓ {} agent daemon started", name),
                Err(e) => eprintln!("  ✗ {} agent daemon failed: {}", name, e),
            }
        }
    }

    println!("\n💡 Use 'octopod' to open the CEO Dashboard");
    println!("💡 Use 'Ctrl+b 0-9' to switch between department windows in tmux");

    Ok(())
}

/// Spawn agent daemons for all departments
async fn spawn_all_agents() -> Result<()> {
    let departments = ["product", "engineering", "qa", "devops", "marketing", "sales", "finance", "legal"];
    println!("\n🤖 Starting agent daemons...");
    for dept in departments {
        match spawn_agent_daemon(dept) {
            Ok(_) => println!("  ✓ {} agent daemon", dept),
            Err(e) => eprintln!("  ✗ {} agent daemon failed: {}", dept, e),
        }
    }
    Ok(())
}

/// Spawn an agent daemon in a detached tmux session
fn spawn_agent_daemon(department: &str) -> Result<()> {
    let session_name = format!("octopod-{}-daemon", department);
    let project_dir = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    // Check if session already exists
    let exists = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if exists {
        println!("  ({} already running)", department);
        return Ok(()); // Already running
    }

    // Build the full command
    let cmd = format!(
        "cd {} && exec octopod agent loop {}",
        project_dir, department
    );

    // Create detached session with command directly
    // Use bash -c to run the command
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
        eprintln!("DEBUG: tmux new-session failed: {}", stderr);
        return Err(anyhow::anyhow!("Failed to create session: {}", stderr));
    }

    Ok(())
}

/// Stop departments
///
/// Usage:
///   octopod stop all
///   octopod stop product
pub async fn stop(all: bool, departments: Vec<String>) -> Result<()> {
    if all {
        println!("Stopping all departments...");
        // TODO: Implement stop_all
        println!("   (Not yet implemented)");
    } else {
        println!("Stopping: {:?}", departments);
        for dept_id in &departments {
            // Capitalize first letter for dept name
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

            if let Err(e) = kill_department(dept_id, &dept_name).await {
                eprintln!("Failed to stop {}: {}", dept_id, e);
            } else {
                println!("Stopped {}", dept_id);
            }
        }
    }

    Ok(())
}
