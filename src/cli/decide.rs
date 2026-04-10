use crate::state::StateManager;
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
pub enum DecisionCommands {
    /// Propose a new initiative (creates initiative + decision if HIGH severity)
    Propose {
        /// Roadmap ID (or use default)
        #[arg(short, long)]
        roadmap: Option<String>,
        /// Department owning this initiative
        #[arg(short, long)]
        department: String,
        /// Initiative title
        title: String,
        /// Severity: low, medium, high
        #[arg(short, long, default_value = "medium")]
        severity: String,
        /// Stakeholder departments (comma-separated)
        #[arg(short, long)]
        stakeholders: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Log a decision made by an agent (auto-proceeds if LOW/MEDIUM)
    Decide {
        /// Decision description
        title: String,
        /// Severity: low, medium, high
        #[arg(short, long, default_value = "medium")]
        severity: String,
        /// Department making the decision
        #[arg(short, long)]
        department: Option<String>,
    },
    /// Request stakeholder review for an initiative
    Review {
        /// Initiative ID
        initiative_id: String,
    },
    /// Transition an initiative to proposed (creates CEO decision if HIGH)
    Submit {
        /// Initiative ID
        initiative_id: String,
    },
    /// Start working on an approved initiative
    Start {
        /// Initiative ID
        initiative_id: String,
    },
    /// Complete an initiative
    Complete {
        /// Initiative ID
        initiative_id: String,
    },
}

pub async fn run(decision_cmd: DecisionCommands) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_dir = find_project_dir(&current_dir).unwrap_or(current_dir);
    let state = StateManager::init_for_project(&project_dir).await?;

    match decision_cmd {
        DecisionCommands::Propose {
            roadmap,
            department,
            title,
            severity,
            stakeholders,
            description: _,
        } => {
            let roadmap_id = if let Some(r) = roadmap {
                r
            } else if let Some(active) = state.get_active_roadmap().await? {
                active.id
            } else {
                anyhow::bail!(
                    "No active roadmap. Create one with 'octopod init' first or specify --roadmap"
                );
            };

            let dept = state
                .get_department_by_slug(&department)
                .await?
                .context("Department not found")?;

            let severity = match severity.to_lowercase().as_str() {
                "low" => crate::state::DecisionSeverity::Low,
                "high" => crate::state::DecisionSeverity::High,
                _ => crate::state::DecisionSeverity::Medium,
            };

            let stakeholder_vec: Vec<String> = stakeholders
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let initiative = state
                .create_initiative_with_severity(
                    &roadmap_id,
                    &dept.id,
                    &title,
                    severity,
                    stakeholder_vec,
                )
                .await?;

            println!(
                "Created initiative: {} ({})",
                initiative.title,
                &initiative.id[..8]
            );

            if severity == crate::state::DecisionSeverity::High {
                println!("HIGH severity - CEO approval required");
            }

            Ok(())
        }

        DecisionCommands::Decide {
            title,
            severity,
            department,
        } => {
            let severity = match severity.to_lowercase().as_str() {
                "low" => crate::state::DecisionSeverity::Low,
                "high" => crate::state::DecisionSeverity::High,
                _ => crate::state::DecisionSeverity::Medium,
            };

            let company_id = state
                .current_company()
                .await
                .context("No company context")?;

            let _decision = if let Some(dept) = department {
                let dept_db = state.get_department_by_slug(&dept).await?;
                if let Some(d) = dept_db {
                    let mut decision = crate::state::Decision::new(&company_id, &title);
                    decision.severity = severity;
                    decision.department_id = Some(d.id);
                    decision
                } else {
                    anyhow::bail!("Department not found: {}", dept);
                }
            } else {
                let mut decision = crate::state::Decision::new(&company_id, &title);
                decision.severity = severity;
                decision
            };

            if severity.requires_approval() {
                println!("HIGH severity decision - will require CEO approval");
                println!("Creating decision: {}", title);
            } else {
                println!("Logged decision (auto-approved): {}", title);
            }

            state
                .create_decision_with_severity(&title, severity)
                .await?;
            println!("Decision logged successfully");

            Ok(())
        }

        DecisionCommands::Review { initiative_id } => {
            let meeting = state.request_stakeholder_review(&initiative_id).await?;
            println!("Created stakeholder review meeting: {}", meeting.title);
            println!("Meeting ID: {}", meeting.id);
            Ok(())
        }

        DecisionCommands::Submit { initiative_id } => {
            let initiative = state
                .transition_initiative_to_proposed(&initiative_id)
                .await?;
            println!("Initiative '{}' submitted for review", initiative.title);

            if initiative.pending_decision_id.is_some() {
                println!("HIGH severity - CEO decision created, awaiting approval");
            } else {
                println!("Auto-approved (LOW/MEDIUM severity)");
            }
            Ok(())
        }

        DecisionCommands::Start { initiative_id } => {
            let initiative = state.start_initiative(&initiative_id).await?;
            println!("Initiative '{}' is now ACTIVE", initiative.title);
            Ok(())
        }

        DecisionCommands::Complete { initiative_id } => {
            let initiative = state.complete_initiative(&initiative_id).await?;
            println!("Initiative '{}' is now COMPLETED", initiative.title);
            Ok(())
        }
    }
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
