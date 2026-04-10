use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use super::omarchy;

#[async_trait]
pub trait Platform: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn supports_workspaces(&self) -> bool;
    async fn spawn_in_workspace(&self, command: &str, workspace: u8) -> Result<WindowHandle>;
}

#[derive(Debug, Clone)]
pub struct WindowHandle {
    pub id: String,
}

pub struct PlatformFactory;

impl PlatformFactory {
    pub fn create(platform_type: PlatformType) -> Arc<dyn Platform> {
        match platform_type {
            PlatformType::Omarchy => Arc::new(omarchy::OmarchyPlatform::new()),
            PlatformType::Generic => todo!("Generic platform not yet implemented"),
        }
    }

    pub fn create_default() -> Arc<dyn Platform> {
        Arc::new(omarchy::OmarchyPlatform::new())
    }

    pub fn detect() -> PlatformType {
        if Self::has_hyprland() {
            PlatformType::Omarchy
        } else {
            PlatformType::Generic
        }
    }

    fn has_hyprland() -> bool {
        std::process::Command::new("which")
            .arg("hyprctl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

pub enum PlatformType {
    Omarchy,
    Generic,
}
