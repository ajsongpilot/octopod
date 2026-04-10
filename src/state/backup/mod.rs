use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Directory to store backups
    pub backup_dir: PathBuf,
    /// Maximum number of backups to keep (0 = unlimited)
    pub max_backups: usize,
    /// Compress backups
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from(".octopod/backups"),
            max_backups: 10,
            compress: true,
        }
    }
}

/// Backup manager
#[derive(Debug, Clone)]
pub struct BackupManager {
    config: BackupConfig,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Self {
        Self { config }
    }

    /// Create a backup of the database
    pub async fn backup(&self, db_path: &Path) -> Result<PathBuf> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.config.backup_dir).await?;

        // Generate backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = if self.config.compress {
            format!("octopod_backup_{}.db.gz", timestamp)
        } else {
            format!("octopod_backup_{}.db", timestamp)
        };

        let backup_path = self.config.backup_dir.join(&backup_filename);

        info!("Creating database backup: {:?}", backup_path);

        // Copy database file
        if self.config.compress {
            self.backup_compressed(db_path, &backup_path).await?;
        } else {
            fs::copy(db_path, &backup_path).await?;
        }

        // Clean up old backups if needed
        if self.config.max_backups > 0 {
            self.cleanup_old_backups().await?;
        }

        info!("Backup created successfully: {:?}", backup_path);
        Ok(backup_path)
    }

    /// Create a compressed backup
    async fn backup_compressed(&self, source: &Path, dest: &Path) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::fs::File;
        use std::io::Write;

        let source_data = fs::read(source).await?;

        let file = File::create(dest)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&source_data)?;
        encoder.finish()?;

        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();

        if !self.config.backup_dir.exists() {
            return Ok(backups);
        }

        let mut entries = fs::read_dir(&self.config.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            if path
                .extension()
                .map(|e| e == "db" || e == "gz")
                .unwrap_or(false)
            {
                let filename = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                backups.push(BackupInfo {
                    path,
                    filename,
                    size_bytes: metadata.len(),
                    created_at: metadata
                        .created()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)),
                });
            }
        }

        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Restore from a backup
    pub async fn restore(&self, backup_path: &Path, db_path: &Path) -> Result<()> {
        info!("Restoring database from: {:?}", backup_path);

        // Verify backup exists
        if !backup_path.exists() {
            anyhow::bail!("Backup file does not exist: {:?}", backup_path);
        }

        // Create a backup of current database before restoring
        if db_path.exists() {
            let pre_restore_backup = db_path.with_extension("db.pre-restore");
            fs::copy(db_path, &pre_restore_backup)
                .await
                .context("Failed to create pre-restore backup")?;
            info!("Created pre-restore backup: {:?}", pre_restore_backup);
        }

        // Restore the backup
        if backup_path.extension().map(|e| e == "gz").unwrap_or(false) {
            self.restore_compressed(backup_path, db_path).await?;
        } else {
            fs::copy(backup_path, db_path).await?;
        }

        info!("Database restored successfully");
        Ok(())
    }

    /// Restore from compressed backup
    async fn restore_compressed(&self, source: &Path, dest: &Path) -> Result<()> {
        use flate2::read::GzDecoder;

        use std::io::Read;

        let compressed_data = fs::read(source).await?;
        let mut decoder = GzDecoder::new(&compressed_data[..]);
        let mut decoded_data = Vec::new();
        decoder.read_to_end(&mut decoded_data)?;

        fs::write(dest, decoded_data).await?;

        Ok(())
    }

    /// Clean up old backups, keeping only the most recent N
    async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups().await?;

        if backups.len() > self.config.max_backups {
            let to_delete = &backups[self.config.max_backups..];

            for backup in to_delete {
                warn!("Removing old backup: {:?}", backup.path);
                fs::remove_file(&backup.path).await.ok();
            }
        }

        Ok(())
    }

    /// Export database to JSON for portability
    pub async fn export_to_json(&self, _db: &super::Database, export_path: &Path) -> Result<()> {
        use serde_json::json;

        let export_data = json!({
            "export_version": "1.0",
            "exported_at": Utc::now().to_rfc3339(),
            "tables": {}
        });

        // This would query each table and serialize to JSON
        // For now, this is a placeholder - would need to implement actual export logic
        info!("Exporting database to JSON: {:?}", export_path);

        let json_string = serde_json::to_string_pretty(&export_data)?;
        fs::write(export_path, json_string).await?;

        Ok(())
    }

    /// Get backup directory path
    pub fn backup_dir(&self) -> &Path {
        &self.config.backup_dir
    }
}

/// Backup metadata
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl BackupInfo {
    pub fn size_human_readable(&self) -> String {
        let bytes = self.size_bytes as f64;
        if bytes < 1024.0 {
            format!("{} B", bytes as u64)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
