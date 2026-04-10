use crate::state::Task;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TaskFileManager {
    tasks_dir: PathBuf,
}

impl TaskFileManager {
    pub fn new(project_dir: &Path) -> Result<Self> {
        // If project_dir is already .octopod, don't add another .octopod prefix
        let tasks_dir = if project_dir
            .file_name()
            .map(|s| s == ".octopod")
            .unwrap_or(false)
        {
            project_dir.join("tasks")
        } else {
            project_dir.join(".octopod").join("tasks")
        };
        if !tasks_dir.exists() {
            fs::create_dir_all(&tasks_dir).context("Failed to create tasks directory")?;
        }
        Ok(Self { tasks_dir })
    }

    pub fn create_task_file(&self, task: &Task) -> Result<String> {
        let file_path = self.tasks_dir.join(format!("{}.md", task.id));

        let content = format!(
            r#"---
id: {id}
department: {department}
created_at: {created_at}
---

# {title}

## User Story

**As a** [who needs this]
**I want** [what they need]
**So that** [why they need it]

## Description

[Describe the feature or task in detail]

## Acceptance Criteria

- [ ] Criterion 1: [what must be true]
- [ ] Criterion 2: [what must be true]
- [ ] Criterion 3: [what must be true]

## Technical Notes

[Any technical considerations, API changes, database migrations, etc.]

## Test Cases (Gherkin)

```gherkin
Feature: {title}

  Scenario: User can complete the primary flow
    Given [setup]
    And [more setup]
    When [action]
    Then [expected outcome]
```
"#,
            id = task.id,
            title = task.title,
            department = task.department_id,
            created_at = task.created_at.to_rfc3339()
        );

        fs::write(&file_path, content).context("Failed to write task file")?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn read_task_file(&self, task: &Task) -> Result<String> {
        if let Some(ref path) = task.file_path {
            let content = fs::read_to_string(path).context("Failed to read task file")?;
            Ok(content)
        } else {
            anyhow::bail!("Task has no file path");
        }
    }

    pub fn update_task_file(&self, task: &Task, content: &str) -> Result<()> {
        if let Some(ref path) = task.file_path {
            fs::write(path, content).context("Failed to update task file")?;
            Ok(())
        } else {
            anyhow::bail!("Task has no file path");
        }
    }

    pub fn get_file_path(&self, task_id: &str) -> PathBuf {
        self.tasks_dir.join(format!("{}.md", task_id))
    }

    pub fn open_in_editor(&self, task: &Task) -> Result<()> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

        if let Some(ref path) = task.file_path {
            // Open in tmux new window so it detaches from TUI's terminal
            let cmd = format!("tmux new-window -n 'octopod-edit' '{} {}'", editor, path);
            std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .spawn()
                .context("Failed to spawn editor in tmux")?;
            Ok(())
        } else {
            anyhow::bail!("Task has no file path");
        }
    }
}

pub fn parse_frontmatter(content: &str) -> Result<(Option<TaskMetadata>, &str)> {
    let mut metadata = TaskMetadata::default();
    let mut lines = content.lines();

    if let Some(first) = lines.next() {
        if first.trim() != "---" {
            return Ok((None, content));
        }
    } else {
        return Ok((None, content));
    }

    let mut frontmatter_end = 0;
    for (i, line) in content.lines().enumerate().skip(1) {
        if line.trim() == "---" {
            frontmatter_end = i;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "title" => metadata.title = Some(value.to_string()),
                "status" => metadata.status = Some(value.to_string()),
                "priority" => metadata.priority = Some(value.to_string()),
                "assignee" => metadata.assignee = Some(value.to_string()),
                _ => {}
            }
        }
    }

    let body = &content[content
        .lines()
        .skip(frontmatter_end + 1)
        .map(|l| l.len() + 1)
        .sum()..];

    Ok((Some(metadata), body.trim()))
}

#[derive(Default)]
pub struct TaskMetadata {
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
}
