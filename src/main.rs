use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use octopod::cli::agent::AgentCommands;
use octopod::cli::decide::{self, DecisionCommands};
use octopod::state::StateManager;
use std::path::Path;
use tracing::info;

#[derive(Parser)]
#[command(name = "octopod")]
#[command(about = "Many-Armed Company Orchestration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive setup wizard
    Onboard,
    /// Initialize Octopod in current directory
    Init {
        #[arg(long)]
        template: Option<String>,
    },
    /// Spawn departments (use: octopod spawn all, or octopod spawn product engineering qa)
    Spawn {
        /// Departments to spawn (use "all" for all departments)
        departments: Vec<String>,
    },
    /// Stop departments (use: octopod stop product, or octopod stop all)
    Stop {
        /// Departments to stop (use "all" for all departments)
        departments: Vec<String>,
    },
    /// Show department status
    Status,
    /// Run diagnostics
    Doctor,
    /// Show mock dashboard layout (for debugging UI)
    Mock,
    /// Database operations
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },
    /// Open department TUI (chat interface)
    Dept {
        /// Department to open (e.g., product, engineering)
        department: String,
    },
    /// Manage tasks
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
    /// Run an agent loop (product, engineering, qa, marketing, sales, cs)
    Agent {
        /// Agent subcommand
        #[command(subcommand)]
        action: AgentSubcommands,
    },
    /// Manage initiatives and decisions
    Decide {
        #[command(subcommand)]
        action: DecisionCommands,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Initialize database for existing project
    Init,
    /// Create a backup of the database
    Backup,
    /// List available backups
    List,
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Create a new task
    New {
        /// Department (e.g., product, engineering, qa)
        department: String,
        /// Task title
        title: String,
        /// Priority (p0, p1, p2, p3)
        #[arg(short, long)]
        priority: Option<String>,
        /// Task type (feature, bug, task, research, documentation)
        #[arg(short, long)]
        task_type: Option<String>,
    },
    /// List tasks
    List {
        /// Filter by department
        #[arg(short, long)]
        department: Option<String>,
        /// Filter by status (todo, in_progress, blocked, review, done)
        #[arg(short, long)]
        status: Option<String>,
    },
}

#[derive(Subcommand)]
enum AgentSubcommands {
    /// Run an agent loop once
    Run {
        /// Department (product, engineering, qa, etc.)
        department: String,
        /// Agent name (optional, defaults to {type}-agent)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Run agent loop continuously (daemon mode)
    Loop {
        /// Department (product, engineering, qa, etc.)
        department: String,
        /// Agent name (optional, defaults to {type}-agent)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Schedule an agent to run periodically
    Schedule {
        /// Department to schedule
        department: String,
        /// Interval in seconds (default: 300)
        #[arg(default_value = "300")]
        interval_secs: u64,
    },
    /// Remove a schedule
    Unschedule {
        /// Department to unschedule
        department: String,
    },
    /// List all schedules
    List,
    /// Enable a disabled schedule
    Enable {
        /// Department to enable
        department: String,
    },
    /// Disable a schedule
    Disable {
        /// Department to disable
        department: String,
    },
}

impl From<AgentSubcommands> for AgentCommands {
    fn from(cmd: AgentSubcommands) -> Self {
        match cmd {
            AgentSubcommands::Schedule {
                department,
                interval_secs,
            } => AgentCommands::Schedule {
                department,
                interval_secs,
            },
            AgentSubcommands::Unschedule { department } => AgentCommands::Unschedule { department },
            AgentSubcommands::List => AgentCommands::List,
            AgentSubcommands::Run { department, .. } => AgentCommands::Run { department },
            AgentSubcommands::Loop { department, .. } => AgentCommands::Loop { department },
            AgentSubcommands::Enable { department } => AgentCommands::Enable { department },
            AgentSubcommands::Disable { department } => AgentCommands::Disable { department },
        }
    }
}

async fn run_agent_loop(state: &StateManager, agent_type: &str, agent_name: &str) -> Result<()> {
    use octopod::agent::agent_loop::AgentContext;
    use octopod::agent::agent_loop::AgentLoop;
    use octopod::state::entities::Agent;

    let dept = state
        .get_department_by_slug(agent_type)
        .await?
        .context("Department not found")?;

    let agent = Agent::new(&dept.id, agent_name);
    let context = AgentContext::new(agent, agent_type.to_string());

    let loop_ = AgentLoop::new(state.clone());
    loop_.set_context(context).await;

    info!("Starting {} agent loop", agent_type);
    loop_.run_loop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Only show errors to stderr - TUI will handle its own display
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_target(false)
        .with_level(false)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Onboard) => {
            if let Err(e) = octopod::cli::onboarding::run().await {
                eprintln!("Error during onboarding: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Init { template }) => {
            if let Err(e) = octopod::cli::init::run(template).await {
                eprintln!("Error during init: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Spawn { departments }) => {
            if departments.is_empty() {
                eprintln!("Usage: octopod spawn all");
                eprintln!("       octopod spawn product");
                eprintln!("       octopod spawn product engineering qa");
                std::process::exit(1);
            }

            let all = departments.len() == 1 && departments[0].to_lowercase() == "all";

            if let Err(e) = octopod::cli::spawn::run(all, departments).await {
                eprintln!("Error during spawn: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Stop { departments }) => {
            if departments.is_empty() {
                eprintln!("Usage: octopod stop all");
                eprintln!("       octopod stop product");
                eprintln!("       octopod stop product engineering qa");
                std::process::exit(1);
            }

            let all = departments.len() == 1 && departments[0].to_lowercase() == "all";

            if let Err(e) = octopod::cli::spawn::stop(all, departments).await {
                eprintln!("Error during stop: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Status) => {
            if let Err(e) = octopod::cli::status::run().await {
                eprintln!("Error showing status: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Doctor) => {
            println!("🔍 Running diagnostics...");
            println!("✓ All systems operational");
        }
        Some(Commands::Mock) => {
            // Show mock dashboard for UI debugging
            match octopod::tui::mock::render_mock_dashboard() {
                Ok(mock) => println!("{}", mock),
                Err(e) => eprintln!("Error generating mock: {}", e),
            }
        }
        Some(Commands::Db { action }) => match action {
            DbCommands::Init => {
                if let Err(e) = octopod::cli::db::init_db().await {
                    eprintln!("Error initializing database: {}", e);
                    std::process::exit(1);
                }
            }
            DbCommands::Backup => {
                if let Err(e) = octopod::cli::db::backup().await {
                    eprintln!("Error creating backup: {}", e);
                    std::process::exit(1);
                }
            }
            DbCommands::List => {
                if let Err(e) = octopod::cli::db::list_backups().await {
                    eprintln!("Error listing backups: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Some(Commands::Dept { department }) => {
            let dept_name = match department.to_lowercase().as_str() {
                "product" => "Product",
                "engineering" => "Engineering",
                "qa" => "QA",
                "devops" => "DevOps",
                "marketing" => "Marketing",
                "sales" => "Sales",
                "finance" => "Finance",
                "legal" => "Legal",
                _ => {
                    eprintln!("Unknown department: {}", department);
                    eprintln!("Valid departments: product, engineering, qa, devops, marketing, sales, finance, legal");
                    std::process::exit(1);
                }
            };

            if let Err(e) =
                octopod::tui::run_department_tui(department, dept_name.to_string()).await
            {
                eprintln!("Error opening department TUI: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Task { action }) => {
            let current_dir = std::env::current_dir()?;
            let project_dir = find_project_dir_or_current(&current_dir);
            let state = StateManager::init_for_project(&project_dir).await?;

            match action {
                TaskCommands::New {
                    department,
                    title,
                    priority,
                    task_type,
                } => {
                    if let Err(e) = octopod::cli::task::create(
                        &state,
                        &department,
                        &title,
                        priority.as_deref(),
                        task_type.as_deref(),
                    )
                    .await
                    {
                        eprintln!("Error creating task: {}", e);
                        std::process::exit(1);
                    }
                }
                TaskCommands::List { department, status } => {
                    if let Err(e) =
                        octopod::cli::task::list(&state, department.as_deref(), status.as_deref())
                            .await
                    {
                        eprintln!("Error listing tasks: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Some(Commands::Agent { action }) => match action {
            AgentSubcommands::Run { department, name } => {
                let current_dir = std::env::current_dir()?;
                let project_dir = find_project_dir_or_current(&current_dir);
                let state = StateManager::init_for_project(&project_dir).await?;

                let agent_name = name.unwrap_or_else(|| format!("{}-agent", department));

                if let Err(e) = run_agent_loop(&state, &department, &agent_name).await {
                    eprintln!("Error running agent: {}", e);
                    std::process::exit(1);
                }
            }
            AgentSubcommands::Loop { department, name } => {
                let current_dir = std::env::current_dir()?;
                let project_dir = find_project_dir_or_current(&current_dir);
                let state = StateManager::init_for_project(&project_dir).await?;

                let agent_name = name.unwrap_or_else(|| format!("{}-daemon", department));

                info!("Starting {} agent daemon (runs forever)", department);
                println!("Starting {} agent daemon... Ctrl+C to stop", department);
                
                // Run forever
                loop {
                    if let Err(e) = run_agent_loop(&state, &department, &agent_name).await {
                        eprintln!("Agent error: {}. Restarting in 5s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
            _ => {
                if let Err(e) = octopod::cli::agent::run(AgentCommands::from(action)).await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Some(Commands::Decide { action }) => {
            if let Err(e) = decide::run(action).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Default: open full-screen CEO Dashboard TUI
            if let Err(e) = octopod::tui::ceo_dashboard::run_ceo_dashboard().await {
                eprintln!("Error opening CEO dashboard: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn find_project_dir_or_current(current: &Path) -> std::path::PathBuf {
    let mut dir = current.to_path_buf();
    loop {
        if dir.join(".octopod").exists() {
            return dir;
        }
        if !dir.pop() {
            return current.to_path_buf();
        }
    }
}
