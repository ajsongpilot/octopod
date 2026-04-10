use crate::state::StateManager;
use anyhow::{Context, Result};

pub async fn init_db() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Check if .octopod exists
    if !current_dir.join(".octopod").exists() {
        anyhow::bail!("No .octopod directory found. Run 'octopod init' first.");
    }

    println!("🗄️  Initializing database...");

    let state = StateManager::init_for_project(&current_dir)
        .await
        .context("Failed to initialize database")?;

    // Check if we already have departments
    match state.list_departments().await {
        Ok(depts) if !depts.is_empty() => {
            println!(
                "✓ Database already initialized with {} departments",
                depts.len()
            );
        }
        _ => {
            // Create default company and departments
            let company = state
                .create_company("My Company")
                .await
                .context("Failed to create company")?;

            state.set_company(company.id.clone()).await;

            let departments = [
                ("product", "Product", 2i64, "Product Management"),
                ("engineering", "Engineering", 3, "Software Engineering"),
                ("qa", "QA", 4, "Quality Assurance"),
                ("devops", "DevOps", 5, "DevOps & Infrastructure"),
                ("marketing", "Marketing", 6, "Marketing & Growth"),
                ("sales", "Sales", 7, "Sales & Business Development"),
                ("finance", "Finance", 8, "Finance & Accounting"),
                ("legal", "Legal", 9, "Legal & Compliance"),
            ];

            for (slug, name, workspace, _desc) in departments {
                state
                    .create_department(name, slug, workspace)
                    .await
                    .with_context(|| format!("Failed to create department: {}", name))?;
            }

            println!(
                "✓ Created company and {} departments in database",
                departments.len()
            );
        }
    }

    // Show database info
    let db = state.database();
    println!("\n📊 Database Info:");
    println!("  Location: {}", db.path().display());

    if let Ok(stats) = db.stats().await {
        println!("  File size: {} bytes", stats.file_size_bytes);
        println!("  Migrations: {}", stats.migration_count);
    }

    Ok(())
}

pub async fn backup() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join(".octopod").exists() {
        anyhow::bail!("No .octopod directory found.");
    }

    println!("💾 Creating backup...");

    let state = StateManager::init_for_project(&current_dir).await?;
    let backup_path = state.backup().await?;

    println!("✓ Backup created: {}", backup_path.display());

    Ok(())
}

pub async fn list_backups() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join(".octopod").exists() {
        anyhow::bail!("No .octopod directory found.");
    }

    let state = StateManager::init_for_project(&current_dir).await?;
    let backups = state.list_backups().await?;

    if backups.is_empty() {
        println!("No backups found.");
    } else {
        println!("📦 Backups:");
        for backup in backups {
            println!("  {} - {}", backup.filename, backup.size_human_readable());
        }
    }

    Ok(())
}
