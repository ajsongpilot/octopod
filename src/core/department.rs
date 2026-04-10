use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub workspace: u8,
}

impl Department {
    pub fn new(id: &str, name: &str, workspace: u8) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            workspace,
        }
    }
}
