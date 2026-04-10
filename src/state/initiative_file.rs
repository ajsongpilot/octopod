use crate::state::Initiative;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InitiativeFileManager {
    initiatives_dir: PathBuf,
}

impl InitiativeFileManager {
    pub fn new(project_dir: &Path) -> Result<Self> {
        // If project_dir is already .octopod, don't add another .octopod prefix
        let initiatives_dir = if project_dir
            .file_name()
            .map(|s| s == ".octopod")
            .unwrap_or(false)
        {
            project_dir.join("initiatives")
        } else {
            project_dir.join(".octopod").join("initiatives")
        };
        if !initiatives_dir.exists() {
            fs::create_dir_all(&initiatives_dir)
                .context("Failed to create initiatives directory")?;
        }
        Ok(Self { initiatives_dir })
    }

    pub fn create_initiative_file(&self, initiative: &Initiative) -> Result<String> {
        let file_path = self.initiatives_dir.join(format!("{}.md", initiative.id));

        let content = format!(
            r#"---
id: {id}
roadmap: {roadmap_id}
owner: {department_id}
status: {status}
priority: {priority}
severity: {severity}
created_at: {created_at}
---

# {title}

## Executive Summary

[Brief 2-3 sentence overview of this initiative. What are we doing and why does it matter?]

## Problem Statement

**The Problem:** [What pain point or opportunity are we addressing?]

**Impact:** [Who is affected and how? What happens if we don't address this?]

**Evidence:** [Any data or feedback that validates this problem exists]

## Goals

What are we trying to achieve? Goals should be outcome-oriented, not task-oriented.

- [ ] **Goal 1:** [Specific, measurable outcome]
  - Why: [Reason this goal matters]
  - Success criteria: [How we know we've achieved it]

- [ ] **Goal 2:** [Specific, measurable outcome]
  - Why: [Reason this goal matters]
  - Success criteria: [How we know we've achieved it]

- [ ] **Goal 3:** [Specific, measurable outcome]
  - Why: [Reason this goal matters]
  - Success criteria: [How we know we've achieved it]

## Key Results

How do we measure success? Key results should be quantifiable and time-bound.

- [ ] **KR 1:** [Measurable result with specific target]
  - Metric: [What we're measuring]
  - Target: [Specific number/percentage]
  - Deadline: [When this needs to be achieved]

- [ ] **KR 2:** [Measurable result with specific target]
  - Metric: [What we're measuring]
  - Target: [Specific number/percentage]
  - Deadline: [When this needs to be achieved]

- [ ] **KR 3:** [Measurable result with specific target]
  - Metric: [What we're measuring]
  - Target: [Specific number/percentage]
  - Deadline: [When this needs to be achieved]

## Timeline & Milestones

| Milestone | Target Date | Status | Notes |
|-----------|-------------|--------|-------|
| Planning Complete | | Not Started | |
| Development Complete | | Not Started | |
| Testing Complete | | Not Started | |
| Launch | | Not Started | |

## Stakeholder Departments

Which departments need to be involved or approve?

| Department | Role | Approval Needed |
|------------|-------|-----------------|
| | | [ ] Yes [ ] No |
| | | [ ] Yes [ ] No |

## Risks & Dependencies

| Risk/Dependency | Impact | Mitigation |
|----------------|--------|------------|
| | | |

## Resources Needed

- **Budget:** [Estimated cost]
- **Team:** [Who is needed]
- **Tools/Systems:** [What is required]

## Success Metrics

How will we know this initiative succeeded?

1. [Primary success metric with target]
2. [Secondary success metric with target]
3. [Tertiary success metric with target]

## Notes

[Any additional context, questions, or considerations]

---
*Created: {created_at}*
*Initiative ID: {id}*
"#,
            id = initiative.id,
            roadmap_id = initiative.roadmap_id,
            department_id = initiative.department_id,
            status = format!("{:?}", initiative.status).to_lowercase(),
            priority = format!("{:?}", initiative.priority).to_lowercase(),
            severity = format!("{:?}", initiative.severity).to_lowercase(),
            created_at = initiative.created_at.to_rfc3339(),
            title = initiative.title,
        );

        fs::write(&file_path, content).context("Failed to write initiative file")?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn read_initiative_file(&self, initiative: &Initiative) -> Result<String> {
        let file_path = self.initiatives_dir.join(format!("{}.md", initiative.id));
        let content = fs::read_to_string(&file_path).context("Failed to read initiative file")?;
        Ok(content)
    }

    pub fn get_title_from_file(&self, initiative_id: &str) -> Result<Option<String>> {
        let file_path = self.get_file_path(initiative_id);
        if !file_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&file_path).context("Failed to read initiative file")?;

        for line in content.lines() {
            if line.starts_with("# ") {
                let title = line.trim_start_matches("# ").to_string();
                return Ok(Some(title));
            }
        }
        Ok(None)
    }

    pub fn update_initiative_file(&self, initiative: &Initiative, content: &str) -> Result<()> {
        let file_path = self.initiatives_dir.join(format!("{}.md", initiative.id));
        fs::write(&file_path, content).context("Failed to update initiative file")?;
        Ok(())
    }

    pub fn get_file_path(&self, initiative_id: &str) -> PathBuf {
        self.initiatives_dir.join(format!("{}.md", initiative_id))
    }

    pub fn open_by_id(&self, initiative_id: &str) -> Result<()> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let file_path = self.get_file_path(initiative_id);

        let cmd = format!(
            "tmux new-window -n 'initiative-edit' '{} {}'",
            editor,
            file_path.display()
        );
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .context("Failed to spawn editor in tmux")?;
        Ok(())
    }

    pub fn open_in_editor(&self, initiative: &Initiative) -> Result<()> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let file_path = self.get_file_path(&initiative.id);

        let cmd = format!(
            "tmux new-window -n 'initiative-edit' '{} {}'",
            editor,
            file_path.display()
        );
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .context("Failed to spawn editor in tmux")?;
        Ok(())
    }
}
