pub mod app;
pub mod ceo_dashboard;
pub mod components;
pub mod dashboard;
pub mod department;
pub mod mock;
pub mod themes;

pub use ceo_dashboard::run_ceo_dashboard;
pub use department::run_department_tui;
