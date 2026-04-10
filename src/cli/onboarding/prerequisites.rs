use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Prerequisite {
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub install_instructions: &'static str,
    pub is_required: bool,
}

impl Clone for Prerequisite {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            command: self.command,
            args: self.args,
            install_instructions: self.install_instructions,
            is_required: self.is_required,
        }
    }
}

pub struct PrerequisiteCheck {
    pub prerequisite: Prerequisite,
    pub is_installed: bool,
    pub version: Option<String>,
}

/// Get common binary search paths that might not be in PATH
fn get_binary_paths() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        home.join(".cargo/bin"),
        home.join(".local/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
    ]
}

/// Find the full path to a binary
fn find_binary(name: &str) -> Option<PathBuf> {
    // First try which (checks PATH)
    if let Ok(path) = which::which(name) {
        return Some(path);
    }

    // Then check common locations
    for dir in get_binary_paths() {
        let full_path = dir.join(name);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    None
}

impl Prerequisite {
    pub fn check(&self) -> Result<PrerequisiteCheck> {
        // Find the binary (in PATH or common locations)
        let binary_path = find_binary(self.command);

        let output = if let Some(path) = binary_path {
            Command::new(&path).args(self.args).output()
        } else {
            // Command not found anywhere
            return Ok(PrerequisiteCheck {
                prerequisite: self.clone(),
                is_installed: false,
                version: None,
            });
        };

        let (is_installed, version) = match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version_line = version_str.lines().next();
                (true, version_line.map(|s| s.trim().to_string()))
            }
            _ => (false, None),
        };

        Ok(PrerequisiteCheck {
            prerequisite: self.clone(),
            is_installed,
            version,
        })
    }
}

pub fn get_prerequisites() -> Vec<Prerequisite> {
    vec![
        // Core system tools
        Prerequisite {
            name: "tmux",
            command: "tmux",
            args: &["-V"],
            install_instructions: "sudo pacman -S tmux  # On Arch/Omarchy\nsudo apt install tmux  # On Debian/Ubuntu\nbrew install tmux  # On macOS",
            is_required: true,
        },
        Prerequisite {
            name: "git",
            command: "git",
            args: &["--version"],
            install_instructions: "sudo pacman -S git  # On Arch/Omarchy\nsudo apt install git  # On Debian/Ubuntu\nbrew install git  # On macOS",
            is_required: true,
        },
        // Window manager (for Omarchy/Linux)
        Prerequisite {
            name: "Hyprland",
            command: "hyprctl",
            args: &["version"],
            install_instructions: "sudo pacman -S hyprland  # On Arch/Omarchy\n# Or use your distribution's package manager",
            is_required: false, // Optional for generic platform
        },
        // AI/LLM Tools
        Prerequisite {
            name: "ironclaw",
            command: "ironclaw",
            args: &["--version"],
            install_instructions: "cargo install ironclaw\n# Or download from: https://github.com/nearai/ironclaw/releases",
            is_required: true,
        },
        Prerequisite {
            name: "opencode",
            command: "opencode",
            args: &["--version"],
            install_instructions: "curl -fsSL https://opencode.ai/install | bash\n# Or on Omarchy: paru -S opencode",
            is_required: true,
        },
        // Note: tdl is a tmux command/keybinding, not a standalone binary
        // It's only available inside tmux sessions, so we skip the standalone check
        // Prerequisite {
        //     name: "tdl",
        //     command: "tdl",
        //     args: &["--version"],
        //     install_instructions: "# tdl is a tmux keybinding, configured in ~/.tmux.conf",
        //     is_required: false,
        // },
        // Git forges
        Prerequisite {
            name: "GitHub CLI (gh)",
            command: "gh",
            args: &["--version"],
            install_instructions: "sudo pacman -S github-cli  # On Arch/Omarchy\n# Or see: https://github.com/cli/cli#installation",
            is_required: false, // Optional - can use GitLab instead
        },
        Prerequisite {
            name: "GitLab CLI (glab)",
            command: "glab",
            args: &["--version"],
            install_instructions: "# See: https://gitlab.com/gitlab-org/cli#installation",
            is_required: false, // Optional - can use GitHub instead
        },
        // Development tools
        Prerequisite {
            name: "cargo",
            command: "cargo",
            args: &["--version"],
            install_instructions: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            is_required: false, // Recommended but not strictly required
        },
    ]
}

pub async fn check_all_prerequisites() -> Result<Vec<PrerequisiteCheck>> {
    let prerequisites = get_prerequisites();
    let mut checks = Vec::new();

    for prereq in prerequisites {
        let check = prereq.check()?;
        checks.push(check);
    }

    Ok(checks)
}

pub fn has_required_tools(checks: &[PrerequisiteCheck]) -> bool {
    checks
        .iter()
        .filter(|c| c.prerequisite.is_required)
        .all(|c| c.is_installed)
}
