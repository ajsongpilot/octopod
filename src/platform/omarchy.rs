use super::types::{Platform, WindowHandle};
use anyhow::Result;
use async_trait::async_trait;

pub struct OmarchyPlatform;

impl Default for OmarchyPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl OmarchyPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Platform for OmarchyPlatform {
    fn name(&self) -> &str {
        "omarchy"
    }

    fn is_available(&self) -> bool {
        which::which("hyprctl").is_ok()
    }

    fn supports_workspaces(&self) -> bool {
        true
    }

    async fn spawn_in_workspace(&self, _command: &str, _workspace: u8) -> Result<WindowHandle> {
        todo!("Spawn in Hyprland workspace")
    }
}
