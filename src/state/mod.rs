use anyhow::{Context, Result};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

pub mod backup;
pub mod decision_file;
pub mod entities;
pub mod initiative_file;
pub mod manager;
pub mod message_bus;
pub mod migrations;
pub mod repositories;
pub mod task_file;

pub use decision_file::DecisionFileManager;
pub use entities::*;
pub use initiative_file::InitiativeFileManager;
pub use manager::StateManager;
pub use message_bus::MessageBus;
pub use repositories::*;
pub use task_file::TaskFileManager;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the database file
    pub db_path: PathBuf,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Enable WAL mode for better concurrency
    pub enable_wal: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("octopod.db"),
            max_connections: 5,
            enable_wal: true,
        }
    }
}

impl DatabaseConfig {
    /// Create config with database in project directory
    pub fn for_project(project_dir: &Path) -> Self {
        Self {
            db_path: project_dir.join(".octopod").join("state.db"),
            max_connections: 5,
            enable_wal: true,
        }
    }

    /// Create config with database in user's config directory
    pub fn for_user() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("octopod");

        Ok(Self {
            db_path: config_dir.join("octopod.db"),
            max_connections: 5,
            enable_wal: true,
        })
    }
}

/// Main database manager
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
    config: DatabaseConfig,
}

impl Database {
    /// Initialize the database, creating it if it doesn't exist
    pub async fn init(config: DatabaseConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Create database if it doesn't exist
        if !Sqlite::database_exists(&config.db_path.to_string_lossy())
            .await
            .unwrap_or(false)
        {
            info!("Creating new database at {:?}", config.db_path);
            Sqlite::create_database(&config.db_path.to_string_lossy()).await?;
        }

        // Build connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&format!("sqlite:{}", config.db_path.display()))
            .await
            .context("Failed to connect to database")?;

        let db = Self { pool, config };

        // Run migrations
        db.run_migrations().await?;

        // Enable WAL mode if configured
        if db.config.enable_wal {
            sqlx::query("PRAGMA journal_mode = WAL")
                .execute(&db.pool)
                .await?;
            sqlx::query("PRAGMA synchronous = NORMAL")
                .execute(&db.pool)
                .await?;
        }

        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db.pool)
            .await?;

        info!("Database initialized successfully");
        Ok(db)
    }

    /// Run all pending migrations
    async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations");

        // Create migrations table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Get current version
        let current_version: i64 =
            sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _migrations")
                .fetch_one(&self.pool)
                .await?;

        debug!("Current database version: {}", current_version);

        // Get all migrations
        let migrations = migrations::get_migrations();

        // Apply pending migrations in order
        for migration in migrations {
            if migration.version > current_version {
                info!(
                    "Applying migration {}: {}",
                    migration.version, migration.name
                );

                // Execute migration in a transaction
                let mut tx = self.pool.begin().await?;

                // Run the migration
                sqlx::query(migration.up_sql).execute(&mut *tx).await?;

                // Record the migration
                sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
                    .bind(migration.version)
                    .bind(migration.name)
                    .execute(&mut *tx)
                    .await?;

                tx.commit().await?;

                info!("Migration {} applied successfully", migration.version);
            }
        }

        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get database file path
    pub fn path(&self) -> &Path {
        &self.config.db_path
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<bool> {
        let result: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&self.pool).await?;
        Ok(result.0 == 1)
    }

    /// Get database statistics
    pub async fn stats(&self) -> Result<DatabaseStats> {
        // Get table sizes
        let table_sizes: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT name, pgsize 
            FROM sqlite_dbpage, pragma_table_list 
            WHERE sqlite_dbpage.pgno = 1 
            AND pragma_table_list.name NOT LIKE 'sqlite_%'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // Get migration count
        let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _migrations")
            .fetch_one(&self.pool)
            .await?;

        // Get database file size
        let metadata = tokio::fs::metadata(&self.config.db_path).await?;
        let file_size = metadata.len();

        Ok(DatabaseStats {
            file_size_bytes: file_size,
            migration_count: migration_count as u32,
            table_sizes,
        })
    }
}

/// Database statistics
#[derive(Debug)]
pub struct DatabaseStats {
    pub file_size_bytes: u64,
    pub migration_count: u32,
    pub table_sizes: Vec<(String, i64)>,
}

/// Migration definition
#[derive(Debug)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up_sql: &'static str,
    pub down_sql: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_init() {
        let temp_dir = TempDir::new().unwrap();
        let config = DatabaseConfig {
            db_path: temp_dir.path().join("test.db"),
            max_connections: 1,
            enable_wal: false,
        };

        let db = Database::init(config).await.unwrap();
        assert!(db.health_check().await.unwrap());
    }
}
