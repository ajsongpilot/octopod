use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::cli::onboarding::banner::{INK_ACCENT, INK_PRIMARY, INK_SECONDARY, INK_SUCCESS, RESET};
use crate::core::config::Config;

fn prompt(message: &str) -> Result<String> {
    print!("{}{}{}: ", INK_ACCENT, message, RESET);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub async fn run(_template: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let octopod_dir = current_dir.join(".octopod");

    // Check if already initialized
    if octopod_dir.exists() {
        println!("⚠️  Octopod is already initialized in this directory.");
        return Ok(());
    }

    println!("{}🐙 Initializing Octopod...{}", INK_ACCENT, RESET);
    println!();

    // Load global config to check for API keys
    let global_config = Config::load().ok();
    let has_openrouter = global_config
        .as_ref()
        .and_then(|c| c.openrouter_api_key.as_ref())
        .is_some();

    if has_openrouter {
        println!(
            "{}✓{} OpenRouter API key found in ~/.config/octopod/config.toml",
            INK_SUCCESS, RESET
        );
        println!("   {}AI-powered discovery enabled{} - I'll analyze your codebase to understand your project", INK_SECONDARY, RESET);
        println!();
    } else {
        println!(
            "{}ℹ{}  OpenRouter API key not configured",
            INK_SECONDARY, RESET
        );
        println!(
            "   Run {}octopod onboard{} to set up AI-powered discovery",
            INK_ACCENT, RESET
        );
        println!(
            "   Or set {}OPENROUTER_API_KEY{} environment variable",
            INK_SECONDARY, RESET
        );
        println!();
    }

    // Phase 1: Project Analysis
    println!(
        "{}🔍{} Phase 1: Analyzing your project...",
        INK_PRIMARY, RESET
    );
    let project_context = gather_project_context(&current_dir).await?;

    if !project_context.detections.is_empty() {
        println!("   {}Detected:{}", INK_SECONDARY, RESET);
        for detection in &project_context.detections {
            println!("     {}•{}{}", INK_PRIMARY, RESET, detection);
        }
    }

    // Phase 2: AI Discovery (if API key available)
    let discovery_results = if has_openrouter {
        println!();
        println!("{}🤖{} Phase 2: AI Discovery", INK_ACCENT, RESET);
        println!(
            "   {}Using AI to understand your codebase...{}",
            INK_SECONDARY, RESET
        );
        run_ai_discovery(&project_context).await?
    } else {
        println!();
        println!("{}📝{} Phase 2: Manual Discovery", INK_ACCENT, RESET);
        println!(
            "   {}No AI assistance available - I'll ask you about your project{}",
            INK_SECONDARY, RESET
        );
        run_manual_discovery(&project_context).await?
    };

    // Phase 3: Interactive Refinement
    println!();
    println!(
        "{}✨{} Phase 3: Let's refine your company profile",
        INK_ACCENT, RESET
    );
    let company_profile = refine_company_profile(discovery_results).await?;

    // Create structure
    println!();
    println!("{}🛠️{}  Creating Octopod structure...", INK_PRIMARY, RESET);
    create_directory_structure(&octopod_dir)?;
    create_config_files(&octopod_dir, &company_profile).await?;
    create_cortex_from_discovery(&octopod_dir, &company_profile).await?;
    create_department_configs(&octopod_dir).await?;

    // Initialize database
    println!("{}🗄️{}  Initializing database...", INK_PRIMARY, RESET);
    init_database(&current_dir).await?;

    // Success
    println!();
    println!(
        "{}✓{} Octopod initialized successfully!",
        INK_SUCCESS, RESET
    );
    println!();
    println!(
        "{}📁{} Created: {}",
        INK_PRIMARY,
        RESET,
        octopod_dir.display()
    );
    println!();
    println!("{}Your AI team now knows:{}", INK_SECONDARY, RESET);
    println!(
        "  {}•{} Company: {}",
        INK_PRIMARY, RESET, company_profile.name
    );
    println!(
        "  {}•{} What you're building: {}",
        INK_PRIMARY, RESET, company_profile.elevator_pitch
    );
    println!(
        "  {}•{} Tech stack: {}",
        INK_PRIMARY, RESET, company_profile.tech_stack
    );
    println!();
    println!("{}Next steps:{}", INK_SECONDARY, RESET);
    println!(
        "  {}1.{} Review .octopod/cortex/company/ for the full profile",
        INK_PRIMARY, RESET
    );
    println!(
        "  {}2.{} Edit any files to add more context",
        INK_PRIMARY, RESET
    );
    println!(
        "  {}3.{} Run {}octopod{} to open the CEO Dashboard",
        INK_PRIMARY, RESET, INK_ACCENT, RESET
    );
    println!(
        "  {}4.{} Use {}octopod dept <name>{} to open a department",
        INK_PRIMARY, RESET, INK_ACCENT, RESET
    );
    println!(
        "  {}5.{} Create tasks with {}octopod task new <dept> \"title\"{}",
        INK_PRIMARY, RESET, INK_ACCENT, RESET
    );
    println!(
        "  {}6.{} Run {}octopod spawn --all{} to spawn AI agents",
        INK_PRIMARY, RESET, INK_ACCENT, RESET
    );

    Ok(())
}

#[derive(Debug)]
struct ProjectContext {
    detections: Vec<String>,
    readme_content: Option<String>,
}

async fn gather_project_context(dir: &Path) -> Result<ProjectContext> {
    let mut detections = Vec::new();

    // Check for Git repository
    if dir.join(".git").exists() {
        detections.push("Git repository".to_string());
    }

    // Read README if it exists
    let readme_content = if let Ok(content) = fs::read_to_string(dir.join("README.md")) {
        detections.push("README.md found".to_string());
        Some(content)
    } else {
        None
    };

    // Detect tech stack
    if dir.join("Cargo.toml").exists() {
        detections.push("Rust project".to_string());
    }
    if dir.join("package.json").exists() {
        detections.push("Node.js project".to_string());
    }
    if dir.join("requirements.txt").exists() {
        detections.push("Python project".to_string());
    }
    if dir.join("pyproject.toml").exists() {
        detections.push("Modern Python project".to_string());
    }
    if dir.join("go.mod").exists() {
        detections.push("Go project".to_string());
    }

    // Detect frontend
    if dir.join("src").exists() {
        let src_dir = dir.join("src");
        if src_dir.join("App.tsx").exists() || src_dir.join("App.jsx").exists() {
            detections.push("React frontend detected".to_string());
        }
    }

    // Docker
    if dir.join("Dockerfile").exists() {
        detections.push("Docker containerization".to_string());
    }
    if dir.join("docker-compose.yml").exists() {
        detections.push("Docker Compose setup".to_string());
    }

    Ok(ProjectContext {
        detections,
        readme_content,
    })
}

#[derive(Debug)]
struct DiscoveryResults {
    suggested_name: String,
    suggested_description: String,
    tech_stack: String,
    key_features: Vec<String>,
    target_users: String,
    elevator_pitch: String,
}

async fn run_ai_discovery(context: &ProjectContext) -> Result<DiscoveryResults> {
    // For now, we'll simulate what the AI would do
    // In the real implementation, this would call the LLM

    println!("   Analyzing your codebase...");

    // Extract insights from README if available
    let suggested_name = if let Some(readme) = &context.readme_content {
        // Extract first line or title
        readme
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("# "))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "My Project".to_string())
    } else {
        "My Project".to_string()
    };

    println!("   💡 Based on your code, I suggest:");
    println!("      Company name: {}", suggested_name);

    // Ask clarifying questions based on what we found
    println!();
    println!("🎯 Let me ask a few questions to understand your business better:");
    println!();

    let name = prompt(&format!("Company name [{}]:", suggested_name))?;
    let name = if name.is_empty() {
        suggested_name
    } else {
        name
    };

    let description = prompt("What does your company do in one sentence?")?;

    println!();
    println!(
        "   I see you're building with: {}",
        context.detections.join(", ")
    );
    let tech = prompt("Tech stack (e.g., Rust + React + PostgreSQL):")?;

    println!();
    println!("   Understanding your users helps the Product team:");
    let users = prompt("Who are your target users? (e.g., developers, SMBs, consumers)")?;

    println!();
    println!("   The Marketing team wants to know:");
    let features_input = prompt("Main features/capabilities (comma-separated):")?;
    let key_features: Vec<String> = features_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let elevator_pitch = format!(
        "{} helps {} by providing {}.",
        name,
        users,
        key_features.join(", ")
    );

    Ok(DiscoveryResults {
        suggested_name: name,
        suggested_description: description,
        tech_stack: tech,
        key_features,
        target_users: users,
        elevator_pitch,
    })
}

async fn run_manual_discovery(context: &ProjectContext) -> Result<DiscoveryResults> {
    println!("   Tell me about your project:");
    println!();

    let name = prompt("Company/project name")?;
    let name = if name.is_empty() {
        "My Company".to_string()
    } else {
        name
    };

    let description = prompt("What does it do? (one sentence)")?;
    let tech = prompt(&format!("Tech stack [{}]:", context.detections.join(", ")))?;
    let users = prompt("Target users?")?;
    let features = prompt("Key features (comma-separated):")?;

    let key_features: Vec<String> = features
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let elevator_pitch = format!("{} helps {} with {}.", name, users, key_features.join(", "));

    Ok(DiscoveryResults {
        suggested_name: name,
        suggested_description: description,
        tech_stack: tech,
        key_features,
        target_users: users,
        elevator_pitch,
    })
}

#[derive(Debug)]
struct CompanyProfile {
    name: String,
    description: String,
    tech_stack: String,
    target_users: String,
    elevator_pitch: String,
    key_features: Vec<String>,
}

async fn refine_company_profile(discovery: DiscoveryResults) -> Result<CompanyProfile> {
    println!();
    println!("📋 Here's what I've learned about your company:");
    println!();
    println!("   Name: {}", discovery.suggested_name);
    println!("   Description: {}", discovery.suggested_description);
    println!("   Tech: {}", discovery.tech_stack);
    println!("   Users: {}", discovery.target_users);
    println!("   Features: {}", discovery.key_features.join(", "));
    println!();

    let ok = prompt("Does this look right? [Y/n]")?;

    if ok.to_lowercase() == "n" {
        println!("   No problem! Let's adjust...");
        // Could loop back here to re-ask questions
    }

    Ok(CompanyProfile {
        name: discovery.suggested_name,
        description: discovery.suggested_description,
        tech_stack: discovery.tech_stack,
        target_users: discovery.target_users,
        elevator_pitch: discovery.elevator_pitch,
        key_features: discovery.key_features,
    })
}

fn create_directory_structure(base: &Path) -> Result<()> {
    let dirs = [
        "",
        "cortex",
        "cortex/company",
        "cortex/product",
        "cortex/engineering",
        "cortex/qa",
        "cortex/devops",
        "cortex/marketing",
        "cortex/sales",
        "cortex/finance",
        "cortex/legal",
        "agents",
        "state",
        "ironclaw-configs",
    ];

    for dir in &dirs {
        let path = base.join(dir);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }

    Ok(())
}

async fn init_database(project_dir: &Path) -> Result<()> {
    use crate::state::StateManager;

    // Initialize state manager (this creates the database and runs migrations)
    let state = StateManager::init_for_project(project_dir)
        .await
        .context("Failed to initialize database")?;

    // Check if we already have a company
    let companies = state.list_departments().await;
    if companies.is_ok() && !companies?.is_empty() {
        println!("   Database already initialized with company data");
        return Ok(());
    }

    // Create default company
    let company = state
        .create_company("My Company")
        .await
        .context("Failed to create company in database")?;

    state.set_company(company.id.clone()).await;

    // Create default departments
    let departments = [
        ("product", "Product", 2i64, "Product Management"),
        ("engineering", "Engineering", 3, "Software Engineering"),
        ("qa", "QA", 4, "Quality Assurance"),
        ("devops", "DevOps", 5, "DevOps & Infrastructure"),
        ("marketing", "Marketing", 6, "Marketing & Growth"),
        ("sales", "Sales", 7, "Sales & Business Development"),
        ("finance", "Finance", 8, "Finance & Accounting"),
        ("legal", "Legal", 9, "Legal & Compliance"),
    ];

    for (slug, name, workspace, _description) in departments {
        state
            .create_department(name, slug, workspace)
            .await
            .with_context(|| format!("Failed to create department: {}", name))?;
    }

    println!(
        "   Created company and {} departments in database",
        departments.len()
    );

    Ok(())
}

async fn create_config_files(base: &Path, profile: &CompanyProfile) -> Result<()> {
    let key_features_str = profile.key_features.join("\", \"");

    let config_content = format!(
        r#"[company]
name = "{}"
description = "{}"

[project]
tech_stack = "{}"
target_users = "{}"
key_features = ["{}"]
elevator_pitch = "{}"

[cortex]
auto_commit = true

[departments]
enabled = ["product", "engineering", "qa", "devops", "marketing", "sales", "finance", "legal"]

[communication]
inter_dept_chat = true
decision_logging = true
"#,
        profile.name,
        profile.description,
        profile.tech_stack,
        profile.target_users,
        key_features_str,
        profile.elevator_pitch
    );

    fs::write(base.join("config.toml"), config_content).context("Failed to write config.toml")?;

    // .gitignore
    let gitignore_content = r#"# Octopod state files (not for git)
state/
*.log

# Temporary files
.tmp/
"#;

    fs::write(base.join(".gitignore"), gitignore_content).context("Failed to write .gitignore")?;

    Ok(())
}

async fn create_cortex_from_discovery(base: &Path, profile: &CompanyProfile) -> Result<()> {
    // Company overview
    let company_overview = format!(
        r#"# {} - Company Overview

## Elevator Pitch
{}

## What We Do
{}

## Technology Stack
{}

## Target Users
{}

## Key Features
{}

## Company Values
- Quality over speed
- Customer-first mindset  
- Security and privacy by default
- Collaborative decision making

## For AI Agents
This document contains the core context about our company. All departments should reference this when making decisions or answering questions.
"#,
        profile.name,
        profile.elevator_pitch,
        profile.description,
        profile.tech_stack,
        profile.target_users,
        profile
            .key_features
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    );

    fs::write(base.join("cortex/company/OVERVIEW.md"), company_overview)?;

    // Product context
    let product_context = format!(
        r#"# Product Context

## Product Overview
{}

## Target Market
{}

## Key Capabilities
{}

## Technical Foundation
Built with {}

## Product Principles
1. Solve real customer problems
2. Prioritize user experience
3. Maintain high quality standards
4. Iterate based on feedback

## For Product Team
When defining features or roadmap items, always reference the company OVERVIEW.md and ensure alignment with our elevator pitch.
"#,
        profile.description,
        profile.target_users,
        profile
            .key_features
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n"),
        profile.tech_stack
    );

    fs::write(base.join("cortex/product/CONTEXT.md"), product_context)?;

    // Engineering context
    let eng_context = format!(
        r#"# Engineering Context

## Technology Stack
{}

## Architecture Notes
- Refer to codebase structure for implementation details
- Follow existing patterns in the project
- Maintain consistency with current tech stack

## Development Priorities
1. Code quality and maintainability
2. Security best practices
3. Performance optimization
4. Comprehensive testing

## Integration Points
This project uses: {}

## For Engineering Team
Always check the Product CONTEXT.md before implementing features. Understand the "why" before the "how".
"#,
        profile.tech_stack, profile.tech_stack
    );

    fs::write(base.join("cortex/engineering/CONTEXT.md"), eng_context)?;

    // Marketing context
    let marketing_context = format!(
        r#"# Marketing Context

## Target Audience
{}

## Value Proposition
{}

## Key Messages
{}

## Competitive Positioning
We help {users} with {features}.

## For Marketing Team
All campaigns should reinforce our core value proposition. Reference the company OVERVIEW.md for consistent messaging.
"#,
        profile.target_users,
        profile.elevator_pitch,
        profile
            .key_features
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n"),
        users = profile.target_users,
        features = profile.key_features.join(", ")
    );

    fs::write(base.join("cortex/marketing/CONTEXT.md"), marketing_context)?;

    Ok(())
}

async fn create_department_configs(base: &Path) -> Result<()> {
    let departments = [
        (
            "product",
            "Product Manager",
            vec![
                "product_management",
                "user_research",
                "requirements_analysis",
            ],
        ),
        (
            "engineering",
            "Software Engineer",
            vec!["software_development", "architecture", "code_review"],
        ),
        (
            "qa",
            "QA Engineer",
            vec!["test_planning", "bug_reporting", "automation"],
        ),
        (
            "devops",
            "DevOps Engineer",
            vec!["infrastructure", "ci_cd", "monitoring"],
        ),
        (
            "marketing",
            "Marketing Manager",
            vec!["campaigns", "content_creation", "analytics"],
        ),
        (
            "sales",
            "Sales Representative",
            vec!["prospecting", "demos", "closing"],
        ),
        (
            "finance",
            "Finance Manager",
            vec!["budgeting", "forecasting", "reporting"],
        ),
        (
            "legal",
            "Legal Counsel",
            vec!["contracts", "compliance", "risk_management"],
        ),
    ];

    // Create ironclaw configs directory
    fs::create_dir_all(base.join("ironclaw-configs"))?;

    for (id, role, skills) in departments {
        let skills_str = skills.join("\n- ");
        let config = format!(
            r#"# {} Department Configuration
id = "{}"
name = "{}"
role = "{}"
model = "openai/gpt-4o-mini"

[skills]
- {}

# Important: Always check cortex/company/OVERVIEW.md for company context
# and your department's CONTEXT.md for specific guidance.

tone = "professional"
auto_approve_tools = false
"#,
            role, id, id, role, skills_str
        );

        fs::write(base.join(format!("agents/{}.toml", id)), config)?;

        // Create ironclaw-compatible config for this department
        create_ironclaw_config(base, id, role, skills).await?;
    }

    Ok(())
}

async fn create_ironclaw_config(
    base: &Path,
    dept_id: &str,
    role: &str,
    skills: Vec<&str>,
) -> Result<()> {
    let skills_list = skills.join(", ");

    // Create a department-specific ironclaw config
    // Only enable CLI channel to avoid port conflicts
    let config = format!(
        r#"# Ironclaw Config for Octopod {dept_id} Department
# Auto-generated by octopod init

onboard_completed = true

[agent]
name = "{dept_id}"
max_parallel_jobs = 3
job_timeout_secs = 3600
use_planning = true
auto_approve_tools = false

[channels]
cli_enabled = true
gateway_enabled = false
http_enabled = false
signal_enabled = false
wasm_channels_enabled = false

[heartbeat]
enabled = false

# Department: {dept_id}
# Role: {role}
# Skills: {skills_list}
"#,
        dept_id = dept_id,
        role = role,
        skills_list = skills_list
    );

    fs::write(
        base.join(format!("ironclaw-configs/{}.toml", dept_id)),
        config,
    )?;
    Ok(())
}
