use crate::agent::runner::AgentRunner;
use crate::state::StateManager;
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub enum AgentCommands {
    /// Schedule an agent to run periodically
    Schedule {
        /// Department (e.g., product, engineering, qa)
        department: String,
        /// Interval in seconds between runs
        #[arg(default_value = "300")]
        interval_secs: u64,
    },
    /// Remove a schedule
    Unschedule {
        /// Department to unschedule
        department: String,
    },
    /// List all schedules and their status
    List,
    /// Run an agent once immediately
    Run {
        /// Department to run
        department: String,
    },
    /// Run agent loop continuously (never exits)
    Loop {
        /// Department to run
        department: String,
    },
    /// Enable a disabled schedule
    Enable {
        /// Department to enable
        department: String,
    },
    /// Disable a schedule (without removing it)
    Disable {
        /// Department to disable
        department: String,
    },
}

pub async fn run(agent_cmd: AgentCommands) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_dir = find_project_dir(&current_dir).unwrap_or(current_dir);
    let state = StateManager::init_for_project(&project_dir).await?;
    let runner = AgentRunner::new(state);

    match agent_cmd {
        AgentCommands::Schedule {
            department,
            interval_secs,
        } => {
            runner.add_schedule(&department, interval_secs).await?;
            println!(
                "Scheduled {} agent to run every {} seconds",
                department, interval_secs
            );
        }
        AgentCommands::Unschedule { department } => {
            runner.remove_schedule(&department).await?;
            println!("Removed schedule for {}", department);
        }
        AgentCommands::List => {
            let schedules = runner.list_schedules().await;
            if schedules.is_empty() {
                println!("No agent schedules configured");
            } else {
                println!(
                    "{:<15} {:<10} {:<20} {:<20}",
                    "DEPARTMENT", "INTERVAL", "LAST RUN", "NEXT RUN"
                );
                println!("{}", "-".repeat(65));
                for schedule in schedules {
                    let status = if schedule.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    let last_run = schedule
                        .last_run
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "never".to_string());
                    let next_run = schedule
                        .next_run
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "never".to_string());
                    println!(
                        "{:<15} {:<10} {:<20} {:<20} ({})",
                        schedule.department_slug,
                        schedule.interval_secs,
                        last_run,
                        next_run,
                        status
                    );
                }
            }
        }
        AgentCommands::Run { department } => {
            println!("Running {} agent once...", department);
            runner.run_once(&department).await?;
            println!("Agent run completed");
        }
        AgentCommands::Loop { department } => {
            // This is handled directly in main.rs via run_agent_loop()
            // This match arm exists to satisfy the compiler
            println!("Agent daemon mode for {} is handled in main", department);
        }
        AgentCommands::Enable { department } => {
            runner.enable_schedule(&department).await?;
            println!("Enabled schedule for {}", department);
        }
        AgentCommands::Disable { department } => {
            runner.disable_schedule(&department).await?;
            println!("Disabled schedule for {}", department);
        }
    }

    Ok(())
}

fn find_project_dir(start: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".octopod").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}
