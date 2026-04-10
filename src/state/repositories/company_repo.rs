use crate::state::entities::Company;
use anyhow::{Context, Result};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct CompanyRepository {
    pool: SqlitePool,
}

impl CompanyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, company: &Company) -> Result<Company> {
        sqlx::query(
            "INSERT INTO companies (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&company.id)
        .bind(&company.name)
        .bind(&company.description)
        .bind(company.created_at)
        .bind(company.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to create company")?;

        Ok(company.clone())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Company>> {
        let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(company)
    }

    pub async fn find_first(&self) -> Result<Option<Company>> {
        let company = sqlx::query_as::<_, Company>("SELECT * FROM companies LIMIT 1")
            .fetch_optional(&self.pool)
            .await?;
        Ok(company)
    }
}
