use crate::state::entities::Department;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct DepartmentRepository {
    pool: SqlitePool,
}

impl DepartmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, dept: &Department) -> Result<Department> {
        sqlx::query(
            "INSERT INTO departments (id, company_id, name, slug, description, workspace, config_json, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&dept.id)
        .bind(&dept.company_id)
        .bind(&dept.name)
        .bind(&dept.slug)
        .bind(&dept.description)
        .bind(dept.workspace)
        .bind(&dept.config_json)
        .bind(dept.created_at)
        .bind(dept.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(dept.clone())
    }

    pub async fn find_by_company(&self, company_id: &str) -> Result<Vec<Department>> {
        let departments = sqlx::query_as::<_, Department>(
            "SELECT * FROM departments WHERE company_id = ? ORDER BY workspace",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(departments)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Department>> {
        let department =
            sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE slug = ? LIMIT 1")
                .bind(slug)
                .fetch_optional(&self.pool)
                .await?;
        Ok(department)
    }
}
