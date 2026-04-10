use anyhow::Result;

pub struct WorkspaceManager;

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn switch_to(&self, _workspace: u8) -> Result<()> {
        todo!("Switch workspace implementation")
    }
}
