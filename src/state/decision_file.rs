use crate::state::Decision;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DecisionFileManager {
    decisions_dir: PathBuf,
}

impl DecisionFileManager {
    pub fn new(project_dir: &Path) -> Result<Self> {
        // If project_dir is already .octopod, don't add another .octopod prefix
        let decisions_dir = if project_dir
            .file_name()
            .map(|s| s == ".octopod")
            .unwrap_or(false)
        {
            project_dir.join("decisions")
        } else {
            project_dir.join(".octopod").join("decisions")
        };
        if !decisions_dir.exists() {
            fs::create_dir_all(&decisions_dir).context("Failed to create decisions directory")?;
        }
        Ok(Self { decisions_dir })
    }

    pub fn create_decision_file(&self, decision: &Decision) -> Result<String> {
        let file_path = self.decisions_dir.join(format!("{}.md", decision.id));

        let content = format!(
            r#"---
id: {id}
title: {title}
department: {department}
severity: {severity}
status: {status}
requester: {requester}
created_at: {created_at}
---

# {title}

## Agent's Context

[What problem does this decision solve? Why is it needed now?]

## Background & Data

[Any relevant data, context, or analysis provided by the requesting agent]

## Options Considered

1. **Option A**: [Description]
2. **Option B**: [Description]
3. **Option C**: [Description]

## Recommendation

[The agent's recommendation with reasoning]

---

## CEO Weigh-In

### Analysis

[CEO's analysis, tradeoffs considered, risks identified]

### Conditions

[If approved, what conditions must be met? Any follow-up required?]

### Sign-off

**Decision:** Approved / Rejected / Deferred / Escalated

**CEO Notes:**

[Detailed notes from CEO review]

**Reviewed:** {reviewed_at}
"#,
            id = decision.id,
            title = decision.title,
            department = decision.department_id.as_deref().unwrap_or("N/A"),
            severity = format!("{:?}", decision.severity).to_lowercase(),
            status = format!("{:?}", decision.status).to_lowercase(),
            requester = decision.requested_by.as_deref().unwrap_or("N/A"),
            created_at = decision.created_at.to_rfc3339(),
            reviewed_at = chrono::Utc::now().format("%Y-%m-%d"),
        );

        fs::write(&file_path, content).context("Failed to write decision file")?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn read_decision_file(&self, decision: &Decision) -> Result<String> {
        let file_path = self.decisions_dir.join(format!("{}.md", decision.id));
        let content = fs::read_to_string(&file_path).context("Failed to read decision file")?;
        Ok(content)
    }

    pub fn update_decision_file(&self, decision: &Decision, content: &str) -> Result<()> {
        let file_path = self.decisions_dir.join(format!("{}.md", decision.id));
        fs::write(&file_path, content).context("Failed to update decision file")?;
        Ok(())
    }

    pub fn get_file_path(&self, decision_id: &str) -> PathBuf {
        self.decisions_dir.join(format!("{}.md", decision_id))
    }

    pub fn open_by_id(&self, decision_id: &str) -> Result<()> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let file_path = self.get_file_path(decision_id);

        let cmd = format!(
            "tmux new-window -n 'decision-edit' -t octopod:1 {} {}",
            editor,
            file_path.display()
        );
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .spawn()
            .context("Failed to spawn editor in tmux")?;
        Ok(())
    }

    pub fn open_in_editor(&self, decision: &Decision) -> Result<()> {
        self.open_by_id(&decision.id)
    }
}
