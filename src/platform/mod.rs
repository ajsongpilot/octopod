pub mod ceo_dashboard;
pub mod omarchy;
pub mod spawn_manager;
pub mod types;

pub use ceo_dashboard::{run_dashboard_ui, spawn_ceo_dashboard};
pub use spawn_manager::{is_department_running, kill_department, spawn_all, spawn_department};
pub use types::{Platform, PlatformFactory, PlatformType};
