use crate::state::entities::{Task, TaskStatus};
use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct TaskRepository {
    pool: SqlitePool,
}

impl TaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task: &Task) -> Result<Task> {
        sqlx::query(
            "INSERT INTO tasks (id, company_id, department_id, initiative_id, assigned_to, created_by, title, description, acceptance_criteria, task_type, status, priority, parent_task_id, related_decision_id, file_path, github_issue_number, estimated_hours, actual_hours, created_at, updated_at, started_at, completed_at, deadline_at, deleted_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&task.id)
        .bind(&task.company_id)
        .bind(&task.department_id)
        .bind(&task.initiative_id)
        .bind(&task.assigned_to)
        .bind(&task.created_by)
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.acceptance_criteria)
        .bind(task.task_type)
        .bind(task.status)
        .bind(task.priority)
        .bind(&task.parent_task_id)
        .bind(&task.related_decision_id)
        .bind(&task.file_path)
        .bind(task.github_issue_number)
        .bind(task.estimated_hours)
        .bind(task.actual_hours)
        .bind(task.created_at)
        .bind(task.updated_at)
        .bind(task.started_at)
        .bind(task.completed_at)
        .bind(task.deadline_at)
        .bind(task.deleted_at)
        .execute(&self.pool)
        .await?;

        Ok(task.clone())
    }

    pub async fn find_by_department(&self, department_id: &str) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE department_id = ? AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(department_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn find_by_department_and_status(
        &self,
        department_id: &str,
        status: TaskStatus,
    ) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE department_id = ? AND status = ? AND deleted_at IS NULL ORDER BY priority ASC, created_at DESC"
        )
        .bind(department_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn find_unassigned_by_department(
        &self,
        department_id: &str,
        status: TaskStatus,
    ) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE department_id = ? AND status = ? AND assigned_to IS NULL AND deleted_at IS NULL ORDER BY priority ASC, created_at DESC"
        )
        .bind(department_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Task>> {
        let task =
            sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ? AND deleted_at IS NULL")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(task)
    }

    pub async fn update(&self, task: &Task) -> Result<Task> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE tasks SET 
                title = ?,
                description = ?,
                acceptance_criteria = ?,
                task_type = ?,
                status = ?,
                priority = ?,
                initiative_id = ?,
                assigned_to = ?,
                parent_task_id = ?,
                related_decision_id = ?,
                file_path = ?,
                estimated_hours = ?,
                actual_hours = ?,
                started_at = ?,
                completed_at = ?,
                updated_at = ?
               WHERE id = ?"#,
        )
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.acceptance_criteria)
        .bind(task.task_type)
        .bind(task.status)
        .bind(task.priority)
        .bind(&task.initiative_id)
        .bind(&task.assigned_to)
        .bind(&task.parent_task_id)
        .bind(&task.related_decision_id)
        .bind(&task.file_path)
        .bind(task.estimated_hours)
        .bind(task.actual_hours)
        .bind(task.started_at)
        .bind(task.completed_at)
        .bind(now)
        .bind(&task.id)
        .execute(&self.pool)
        .await?;

        Ok(task.clone())
    }

    pub async fn delete(&self, task_id: &str) -> Result<()> {
        sqlx::query("UPDATE tasks SET deleted_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn find_without_file_path(&self) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE file_path IS NULL AND deleted_at IS NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tasks)
    }

    pub async fn find_by_initiative(&self, initiative_id: &str) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE initiative_id = ? AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(initiative_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(tasks)
    }

    pub async fn find_by_department_and_initiative(
        &self,
        department_id: &str,
        initiative_id: &str,
    ) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE department_id = ? AND initiative_id = ? AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(department_id)
        .bind(initiative_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(tasks)
    }
}
