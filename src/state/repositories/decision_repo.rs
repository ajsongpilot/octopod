use crate::state::entities::{Decision, DecisionSeverity, DecisionStatus, Priority};
use crate::state::repositories::{PaginatedResult, Pagination};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Repository for decision operations
#[derive(Debug, Clone)]
pub struct DecisionRepository {
    pool: SqlitePool,
}

impl DecisionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new decision
    pub async fn create(&self, decision: &Decision) -> Result<Decision> {
        sqlx::query(
            r#"
            INSERT INTO decisions (
                id, company_id, title, description, department_id, requested_by,
                priority, severity, status, context_json, initiative_id, created_at, updated_at, file_path
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&decision.id)
        .bind(&decision.company_id)
        .bind(&decision.title)
        .bind(&decision.description)
        .bind(&decision.department_id)
        .bind(&decision.requested_by)
        .bind(format!("{:?}", decision.priority).to_lowercase())
        .bind(format!("{:?}", decision.severity).to_lowercase())
        .bind(format!("{:?}", decision.status).to_lowercase())
        .bind(&decision.context_json)
        .bind(&decision.initiative_id)
        .bind(decision.created_at)
        .bind(decision.updated_at)
        .bind(&decision.file_path)
        .execute(&self.pool)
        .await
        .context("Failed to create decision")?;

        Ok(decision.clone())
    }

    /// Find decision by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Decision>> {
        let decision = sqlx::query_as::<_, Decision>(
            r#"
            SELECT * FROM decisions WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find decision")?;

        Ok(decision)
    }

    /// Update a decision
    pub async fn update(&self, decision: &Decision) -> Result<Decision> {
        sqlx::query(
            r#"
            UPDATE decisions SET
                title = ?,
                description = ?,
                department_id = ?,
                requested_by = ?,
                priority = ?,
                severity = ?,
                status = ?,
                context_json = ?,
                approved_by = ?,
                decision_notes = ?,
                initiative_id = ?,
                resolved_at = ?,
                file_path = ?
            WHERE id = ?
            "#,
        )
        .bind(&decision.title)
        .bind(&decision.description)
        .bind(&decision.department_id)
        .bind(&decision.requested_by)
        .bind(format!("{:?}", decision.priority).to_lowercase())
        .bind(format!("{:?}", decision.severity).to_lowercase())
        .bind(format!("{:?}", decision.status).to_lowercase())
        .bind(&decision.context_json)
        .bind(&decision.approved_by)
        .bind(&decision.decision_notes)
        .bind(&decision.initiative_id)
        .bind(decision.resolved_at)
        .bind(&decision.file_path)
        .bind(&decision.id)
        .execute(&self.pool)
        .await
        .context("Failed to update decision")?;

        Ok(decision.clone())
    }

    /// List decisions for a company with optional filters
    pub async fn list(
        &self,
        company_id: &str,
        filters: DecisionFilters,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Decision>> {
        let mut query =
            String::from("SELECT * FROM decisions WHERE company_id = ? AND deleted_at IS NULL");

        if let Some(status) = filters.status {
            query.push_str(&format!(" AND status = '{}'", status.as_str()));
        }

        if let Some(department_id) = filters.department_id {
            query.push_str(&format!(" AND department_id = '{}'", department_id));
        }

        if let Some(priority) = filters.priority {
            query.push_str(&format!(" AND priority = '{}'", priority.as_str()));
        }

        if let Some(severity) = filters.severity {
            query.push_str(&format!(" AND severity = '{}'", severity.as_str()));
        }

        // Add ordering
        query.push_str(" ORDER BY created_at DESC");

        // Add pagination
        query.push_str(&format!(
            " LIMIT {} OFFSET {}",
            pagination.limit(),
            pagination.offset()
        ));

        let decisions: Vec<Decision> = sqlx::query_as(&query)
            .bind(company_id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to list decisions")?;

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM decisions WHERE company_id = ? AND deleted_at IS NULL{}",
            if filters.status.is_some() {
                " AND status = ?"
            } else {
                ""
            }
        );

        let total: i64 = sqlx::query_scalar(&count_query)
            .bind(company_id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to count decisions")?;

        Ok(PaginatedResult::new(decisions, total, &pagination))
    }

    /// Get pending decisions for a company
    pub async fn get_pending(&self, company_id: &str, limit: i64) -> Result<Vec<Decision>> {
        let decisions = sqlx::query_as::<_, Decision>(
            &format!(
                r#"
                SELECT * FROM decisions 
                WHERE company_id = ? 
                AND status = 'pending'
                AND deleted_at IS NULL
                ORDER BY 
                    CASE priority 
                        WHEN 'p0' THEN 0 
                        WHEN 'p1' THEN 1 
                        WHEN 'p2' THEN 2 
                        WHEN 'p3' THEN 3 
                    END,
                    created_at DESC
                LIMIT {}
                "#,
                limit
            ),
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get pending decisions")?;

        Ok(decisions)
    }

    /// Get high-severity pending decisions (CEO queue)
    pub async fn get_high_severity_pending(&self, company_id: &str) -> Result<Vec<Decision>> {
        let decisions = sqlx::query_as::<_, Decision>(
            r#"
            SELECT * FROM decisions 
            WHERE company_id = ? 
            AND status = 'pending'
            AND severity = 'high'
            AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get high-severity decisions")?;

        Ok(decisions)
    }

    /// Approve a decision
    pub async fn approve(
        &self,
        decision_id: &str,
        approved_by: &str,
        notes: Option<&str>,
    ) -> Result<Decision> {
        sqlx::query(
            r#"
            UPDATE decisions SET
                status = 'approved',
                approved_by = ?,
                decision_notes = ?,
                resolved_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(approved_by)
        .bind(notes)
        .bind(decision_id)
        .execute(&self.pool)
        .await
        .context("Failed to approve decision")?;

        self.find_by_id(decision_id)
            .await?
            .context("Decision not found after approval")
    }

    /// Reject a decision
    pub async fn reject(
        &self,
        decision_id: &str,
        rejected_by: &str,
        notes: Option<&str>,
    ) -> Result<Decision> {
        sqlx::query(
            r#"
            UPDATE decisions SET
                status = 'rejected',
                approved_by = ?,
                decision_notes = ?,
                resolved_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(rejected_by)
        .bind(notes)
        .bind(decision_id)
        .execute(&self.pool)
        .await
        .context("Failed to reject decision")?;

        self.find_by_id(decision_id)
            .await?
            .context("Decision not found after rejection")
    }

    /// Soft delete a decision
    pub async fn delete(&self, id: &str) -> Result<bool> {
        let result =
            sqlx::query("UPDATE decisions SET deleted_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
                .context("Failed to delete decision")?;

        Ok(result.rows_affected() > 0)
    }

    /// Get decision statistics for a company
    pub async fn get_stats(&self, company_id: &str) -> Result<DecisionStats> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM decisions WHERE company_id = ? AND deleted_at IS NULL",
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        let pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM decisions WHERE company_id = ? AND status = 'pending' AND deleted_at IS NULL"
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        let approved: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM decisions WHERE company_id = ? AND status = 'approved' AND deleted_at IS NULL"
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        let rejected: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM decisions WHERE company_id = ? AND status = 'rejected' AND deleted_at IS NULL"
        )
        .bind(company_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DecisionStats {
            total,
            pending,
            approved,
            rejected,
        })
    }
}

/// Filters for decision queries
#[derive(Debug, Clone, Default)]
pub struct DecisionFilters {
    pub status: Option<DecisionStatus>,
    pub department_id: Option<String>,
    pub priority: Option<Priority>,
    pub severity: Option<DecisionSeverity>,
}

impl DecisionFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: DecisionStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_department(mut self, department_id: impl Into<String>) -> Self {
        self.department_id = Some(department_id.into());
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_severity(mut self, severity: DecisionSeverity) -> Self {
        self.severity = Some(severity);
        self
    }
}

/// Decision statistics
#[derive(Debug, Clone)]
pub struct DecisionStats {
    pub total: i64,
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
}
