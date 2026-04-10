use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::core::config::{BackendConfig, CodingConfig, Config, PlatformConfig};

pub mod banner;
pub mod prerequisites;
pub mod ui;

use ui::TerminalUI;

pub async fn run() -> Result<()> {
    let ui = TerminalUI::new();

    // Step 1: Welcome
    ui.clear_screen()?;
    banner::print_welcome();
    ui.wait_for_key()?;

    // Step 2: Prerequisites Check
    ui.clear_screen()?;
    ui.print_header("PREREQUISITES CHECK");

    let checks = prerequisites::check_all_prerequisites().await?;
    let mut missing_required = Vec::new();
    let mut missing_optional = Vec::new();

    for check in &checks {
        if check.is_installed {
            let version = check.version.as_deref().unwrap_or("unknown");
            ui.print_success(&format!("{} ({})", check.prerequisite.name, version));
        } else if check.prerequisite.is_required {
            ui.print_error(&format!("{} - NOT INSTALLED", check.prerequisite.name));
            missing_required.push(check);
        } else {
            ui.print_warning(&format!(
                "{} - NOT INSTALLED (optional)",
                check.prerequisite.name
            ));
            missing_optional.push(check);
        }
    }

    // Handle missing required tools
    if !missing_required.is_empty() {
        ui.print_divider();
        ui.print_error("Missing required tools!");
        println!("\nThe following tools are required to use Octopod:\n");

        for check in &missing_required {
            println!("  📦 {}", check.prerequisite.name);
            println!("  Install with:\n");
            ui.print_code_block(check.prerequisite.install_instructions);
            println!();
        }

        let install_now = ui.prompt_yes_no("Would you like to install missing tools now?", true)?;

        if install_now {
            println!("\n🚀 Installing missing tools...\n");

            for check in &missing_required {
                println!("Installing {}...", check.prerequisite.name);
                println!("Running: {}\n", check.prerequisite.install_instructions);
                // Note: In production, we'd actually run these commands
                // For safety, we just display them
            }

            ui.print_warning("Please run the commands above in a separate terminal, then run 'octopod onboard' again.");
            return Ok(());
        } else {
            ui.print_warning("Cannot continue without required tools. Please install them and run 'octopod onboard' again.");
            return Ok(());
        }
    }

    ui.print_divider();
    ui.print_success("All required tools are installed!");
    ui.wait_for_key()?;

    // Step 3: Configuration
    ui.clear_screen()?;
    ui.print_header("CONFIGURATION");

    let config = configure_octopod(&ui).await?;
    save_global_config(&config).await?;

    // Step 4: Completion
    ui.clear_screen()?;
    ui.print_header("SETUP COMPLETE!");

    println!(
        "\n  {}🎉{} Octopod is ready to use!\n",
        banner::INK_ACCENT,
        banner::RESET
    );

    println!(
        "  {}Your configuration:{}",
        banner::INK_SECONDARY,
        banner::RESET
    );
    println!(
        "    {}•{} Backend: {}{}",
        banner::INK_PRIMARY,
        banner::RESET,
        banner::INK_ACCENT,
        config.backend.coordinator
    );
    println!(
        "    {}•{} Coding Agent: {}{}",
        banner::INK_PRIMARY,
        banner::RESET,
        banner::INK_ACCENT,
        config.coding.agent
    );
    println!(
        "    {}•{} Platform: {}{}",
        banner::INK_PRIMARY,
        banner::RESET,
        banner::INK_ACCENT,
        config.platform.platform_type
    );

    println!("\n  {}Next steps:{}", banner::INK_SECONDARY, banner::RESET);

    println!(
        "\n  {}Octopod supports two project structures:{}",
        banner::INK_SECONDARY,
        banner::RESET
    );

    println!(
        "\n  {}Option 1:{} Monolith (single codebase)",
        banner::INK_ACCENT,
        banner::RESET
    );
    println!(
        "    {}cd /path/to/your-monolith-repo{}",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!("    {}octopod init{}", banner::INK_ACCENT, banner::RESET);

    println!(
        "\n  {}Option 2:{} Multi-repo (services in subdirectories)",
        banner::INK_ACCENT,
        banner::RESET
    );
    println!(
        "    {}mkdir my-company && cd my-company{}",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}/ Clone your service repos here{}",
        banner::INK_SECONDARY,
        banner::RESET
    );
    println!(
        "    {}git clone git@github.com:org/auth-service.git{}",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}git clone git@github.com:org/api-gateway.git{}",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}/ ... more services ...{}",
        banner::INK_SECONDARY,
        banner::RESET
    );
    println!(
        "    {}octopod init{}  / Run at the parent level",
        banner::INK_ACCENT,
        banner::RESET
    );

    println!(
        "\n  {}Multi-Agent Features:{}",
        banner::INK_SECONDARY,
        banner::RESET
    );
    println!(
        "    {}•{} Department agents communicate via message bus",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}•{} Tasks automatically route between departments",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}•{} CEO Dashboard monitors all agent activity",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}•{} Ironclaw provides coordination and recommendations",
        banner::INK_PRIMARY,
        banner::RESET
    );

    println!(
        "\n  {}Then launch your company:{}",
        banner::INK_SECONDARY,
        banner::RESET
    );
    println!(
        "    {}octopod spawn --all{}",
        banner::INK_ACCENT,
        banner::RESET
    );
    println!(
        "    {}octopod{}              / Start CEO Dashboard",
        banner::INK_ACCENT,
        banner::RESET
    );

    println!(
        "\n  {}Agent Backends Supported:{}",
        banner::INK_SECONDARY,
        banner::RESET
    );
    println!(
        "    {}•{} Opencode (primary coding agent)",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}•{} Ironclaw (coordinator/advisor)",
        banner::INK_PRIMARY,
        banner::RESET
    );
    println!(
        "    {}•{} Extensible: Claude Code, Aider, Custom (future)",
        banner::INK_PRIMARY,
        banner::RESET
    );

    println!(
        "\n  {}Use 'octopod --help' for more commands.{}",
        banner::INK_SECONDARY,
        banner::RESET
    );

    ui.wait_for_key()?;

    Ok(())
}

async fn configure_octopod(ui: &TerminalUI) -> Result<Config> {
    // Using Ironclaw (only supported backend for now)
    ui.print_info("Using Ironclaw as the coordinator backend");
    let backend = BackendConfig {
        coordinator: "ironclaw".to_string(),
    };

    // Using Opencode (only supported coding agent for now)
    ui.print_info("Using Opencode as the coding agent");
    let coding = CodingConfig {
        agent: "opencode".to_string(),
    };

    // Platform selection
    let has_hyprland = which::which("hyprctl").is_ok();

    let platform = if has_hyprland {
        ui.print_success("Hyprland detected - using workspace integration");
        PlatformConfig {
            platform_type: "omarchy".to_string(),
        }
    } else {
        ui.print_warning("Hyprland not detected - using generic mode");
        PlatformConfig {
            platform_type: "generic".to_string(),
        }
    };

    // OpenRouter Configuration for AI-powered init
    ui.print_info("");
    ui.print_info("AI Configuration (for intelligent project discovery)");

    // Check environment first
    let mut openrouter_api_key = std::env::var("OPENROUTER_API_KEY").ok();
    let mut openrouter_base_url = std::env::var("OPENROUTER_BASE_URL").ok();
    let mut openrouter_model = std::env::var("OPENROUTER_MODEL").ok();

    if openrouter_api_key.is_some() {
        ui.print_success("OPENROUTER_API_KEY found in environment");
        ui.print_info("This will be used during 'octopod init' to analyze your codebase");
        ui.print_info("and automatically generate company context for your AI team.");
    } else {
        ui.print_info("To enable AI-powered project discovery during 'octopod init',");
        ui.print_info("provide your OpenRouter credentials:");
        println!();

        let key = ui.prompt("OpenRouter API Key (optional, press Enter to skip):")?;
        if !key.is_empty() {
            openrouter_api_key = Some(key);

            let base_url = ui.prompt("OpenRouter Base URL [https://openrouter.ai/api/v1]:")?;
            openrouter_base_url = if base_url.is_empty() {
                Some("https://openrouter.ai/api/v1".to_string())
            } else {
                Some(base_url)
            };

            let model = ui.prompt("Default Model [openai/gpt-4o-mini]:")?;
            openrouter_model = if model.is_empty() {
                Some("openai/gpt-4o-mini".to_string())
            } else {
                Some(model)
            };
        }
    }

    Ok(Config {
        backend,
        coding,
        platform,
        openrouter_api_key,
        openrouter_base_url,
        openrouter_model,
    })
}

async fn save_global_config(config: &Config) -> Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("octopod");

    fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");
    let config_str = toml::to_string_pretty(config)?;

    fs::write(&config_path, config_str)?;

    println!("  Configuration saved to: {}", config_path.display());

    Ok(())
}
