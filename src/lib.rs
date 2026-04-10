pub mod agent;
pub mod backends;
pub mod cli;
pub mod coding;
pub mod core;
pub mod departments;
pub mod git;
pub mod platform;
pub mod state;
pub mod tui;

pub use core::company::Company;
pub use core::config::Config;
pub use core::department::Department;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONFIG_PATH: &str = "~/.config/octopod";
pub const COMPANY_CONFIG_DIR: &str = ".octopod";
