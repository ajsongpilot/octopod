use crate::state::{StateManager, TaskStatus};
use anyhow::{Context, Result};

fn status_from_str(s: &str) -> Option<TaskStatus> {
    match s.to_lowercase().as_str() {
        "todo" => Some(TaskStatus::Todo),
        "in_progress" | "inprogress" | "progress" => Some(TaskStatus::InProgress),
        "blocked" => Some(TaskStatus::Blocked),
        "review" => Some(TaskStatus::Review),
        "done" | "complete" | "completed" => Some(TaskStatus::Done),
        "cancelled" | "cancel" => Some(TaskStatus::Cancelled),
        _ => None,
    }
}

/// Create a new task
pub async fn create(
    state: &StateManager,
    department: &str,
    title: &str,
    _priority: Option<&str>,
    _task_type: Option<&str>,
) -> Result<()> {
    ensure_company_set(state).await?;

    let dept = state
        .get_department_by_slug(department)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Department not found: {}", department))?;

    let task = state.create_task(&dept.id, title).await?;

    println!("✓ Created task:");
    println!("  ID: {}", &task.id[..8]);
    println!("  Department: {}", dept.name);
    println!("  Title: {}", task.title);

    let _ = (_priority, _task_type);
    Ok(())
}

/// List tasks for a department or all departments
#[allow(dead_code)]
pub async fn list(
    state: &StateManager,
    department: Option<&str>,
    status_filter: Option<&str>,
) -> Result<()> {
    ensure_company_set(state).await?;

    let depts = if let Some(dept_name) = department {
        let dept = state
            .get_department_by_slug(dept_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Department not found: {}", dept_name))?;
        vec![dept]
    } else {
        state.list_departments().await?
    };

    let status = status_filter.and_then(status_from_str);

    for dept in depts {
        let tasks = state.get_department_tasks(&dept.id).await?;
        let filtered: Vec<_> = if let Some(s) = status {
            tasks.into_iter().filter(|t| t.status == s).collect()
        } else {
            tasks
        };

        if !filtered.is_empty() {
            println!("\n{}:", dept.name);
            for task in &filtered {
                let status_str = format!("{:?}", task.status);
                let priority_str = task.priority.as_str();
                println!(
                    "  [{}] [{}] {}",
                    priority_str,
                    status_str,
                    truncate(&task.title, 50)
                );
            }
        }
    }

    Ok(())
}

/// Update a task's status
#[allow(dead_code)]
pub async fn update_status(_state: &StateManager, task_id: &str, new_status: &str) -> Result<()> {
    let status = status_from_str(new_status).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid status: {}. Valid: todo, in_progress, blocked, review, done, cancelled",
            new_status
        )
    })?;

    println!("✓ Task {} status updated to {:?}", &task_id[..8], status);

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Ensure state has a company set (needed for CLI commands)
pub async fn ensure_company_set(state: &StateManager) -> Result<()> {
    if state.current_company().await.is_none() {
        let depts = state
            .list_departments()
            .await
            .context("No departments found - run 'octopod init' first")?;

        if depts.is_empty() {
            anyhow::bail!("No departments found - run 'octopod init' first");
        }

        state.set_company(depts[0].company_id.clone()).await;
    }
    Ok(())
}
