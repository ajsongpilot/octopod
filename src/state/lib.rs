//! # Octopod State Management
//!
//! This module provides comprehensive database management for Octopod.
//!
//! ## Architecture
//!
//! ```
//! ┌─────────────────────────────────────────────┐
//! │           StateManager                      │
//! │  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │  Database   │  │  BackupManager      │  │
//! │  │  (SQLite)   │  │  (compressed .gz)   │  │
//! │  └──────┬──────┘  └─────────────────────┘  │
//! │         │                                   │
//! │  ┌──────┴──────────────────────────────┐   │
//! │  │         Repositories                 │   │
//! │  │  - CompanyRepository                 │   │
//! │  │  - DepartmentRepository              │   │
//! │  │  - AgentRepository                   │   │
//! │  │  - DecisionRepository                │   │
//! │  │  - MessageRepository                 │   │
//! │  │  - TaskRepository                    │   │
//! │  │  - ActivityRepository                │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Migration System**: Versioned schema migrations with rollback support
//! - **Backup/Restore**: Automatic compressed backups with retention
//! - **Entity Models**: Type-safe Rust structs for all database entities
//! - **Repository Pattern**: Clean CRUD operations for each entity type
//! - **Pagination**: Built-in pagination for list queries
//! - **WAL Mode**: Write-Ahead Logging for better concurrency
//! - **Foreign Keys**: Referential integrity with cascade rules
//!
//! ## Usage Example
//!
//! ```rust
//! use octopod::state::{StateManager, DatabaseConfig, Decision, Priority};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize state manager for a project
//!     let state = StateManager::init_for_project(
//!         std::path::Path::new("/home/user/myproject")
//!     ).await?;
//!
//!     // Create a company
//!     let company = state.create_company("My Startup").await?;
//!     state.set_company(company.id).await;
//!
//!     // Create departments
//!     let product = state.create_department("Product", "product", 2).await?;
//!     let engineering = state.create_department("Engineering", "engineering", 3).await?;
//!
//!     // Create a decision
//!     let mut decision = Decision::new(&company.id, "Should we use Rust?");
//!     decision.description = Some("Engineering team needs to decide on tech stack");
//!     decision.priority = Priority::P1;
//!     
//!     let decision = state.decisions.create(&decision).await?;
//!     println!("Created decision: {}", decision.id);
//!
//!     // Approve the decision
//!     let approved = state.approve_decision(&decision.id, "ceo", Some("Approved!")).await?;
//!     println!("Decision status: {:?}", approved.status);
//!
//!     // List pending decisions
//!     let pending = state.list_pending_decisions(10).await?;
//!     println!("Pending decisions: {}", pending.len());
//!
//!     // Send a message
//!     let msg = state.send_message("general", "Hello team!").await?;
//!     
//!     // Get recent activity
//!     let recent = state.get_recent_messages(50).await?;
//!
//!     // Create backup
//!     let backup_path = state.backup().await?;
//!     println!("Backup created: {:?}", backup_path);
//!
//!     // List backups
//!     let backups = state.list_backups().await?;
//!
//!     // Restore from backup
//!     // state.restore(&backup_path).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Database Schema
//!
//! The database includes these tables:
//!
//! - **companies**: Company/project information
//! - **departments**: Departments (Product, Engineering, etc.)
//! - **agents**: Individual AI agents
//! - **decisions**: CEO decision tracking
//! - **conversations**: Chat channels
//! - **messages**: Chat messages
//! - **tasks**: Work items and assignments
//! - **activity_log**: Audit trail
//!
//! ## Backup Strategy
//!
//! Backups are stored in `.octopod/backups/` with automatic rotation:
//! - Named: `octopod_backup_YYYYMMDD_HHMMSS.db.gz`
//! - Compressed with gzip
//! - Configurable retention (default: keep 10)
//! - Pre-restore backups created automatically
//!
//! ## Migrations
//!
//! Migrations are stored in `src/state/migrations/`:
//! - `001_initial_schema.sql` - Core tables
//! - `002_add_indexes.sql` - Performance indexes
//!
//! The migration system:
//! - Tracks applied migrations in `_migrations` table
//! - Runs automatically on database init
//! - Supports transactions (all-or-nothing)
//! - Versioned for rollback support
//!
//! ## Query Performance
//!
//! Indexes are created for common query patterns:
//! - Decisions by company, status, priority
//! - Messages by conversation, timestamp
//! - Tasks by department, status
//! - Agents by department, status
//!
//! WAL mode is enabled for better concurrency between readers and writers.

pub use entities::*;
pub use manager::StateManager;
pub use repositories::{PaginatedResult, Pagination};
