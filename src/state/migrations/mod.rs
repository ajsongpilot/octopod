use super::Migration;

/// Get all migrations in order
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "initial_schema",
            up_sql: include_str!("./001_initial_schema.sql"),
            down_sql: None,
        },
        Migration {
            version: 2,
            name: "add_indexes",
            up_sql: include_str!("./002_add_indexes.sql"),
            down_sql: None,
        },
        Migration {
            version: 3,
            name: "add_acceptance_criteria",
            up_sql: include_str!("./003_add_acceptance_criteria.sql"),
            down_sql: None,
        },
        Migration {
            version: 4,
            name: "add_file_path",
            up_sql: include_str!("./004_add_file_path.sql"),
            down_sql: None,
        },
        Migration {
            version: 5,
            name: "backfill_task_files",
            up_sql: include_str!("./005_backfill_task_files.sql"),
            down_sql: None,
        },
        Migration {
            version: 6,
            name: "add_roadmaps_meetings",
            up_sql: include_str!("./006_add_roadmaps_meetings.sql"),
            down_sql: None,
        },
        Migration {
            version: 7,
            name: "add_decision_severity",
            up_sql: include_str!("./007_add_decision_severity.sql"),
            down_sql: None,
        },
        Migration {
            version: 8,
            name: "update_initiative_status",
            up_sql: include_str!("008_update_initiative_status.sql"),
            down_sql: None,
        },
        Migration {
            version: 9,
            name: "add_initiative_message_types",
            up_sql: include_str!("009_add_initiative_message_types.sql"),
            down_sql: None,
        },
        Migration {
            version: 10,
            name: "add_agent_sessions",
            up_sql: include_str!("010_add_agent_sessions.sql"),
            down_sql: None,
        },
        Migration {
            version: 11,
            name: "add_decisions_deleted_at",
            up_sql: include_str!("011_add_decisions_deleted_at.sql"),
            down_sql: None,
        },
        Migration {
            version: 12,
            name: "add_initiative_file_path",
            up_sql: include_str!("012_add_initiative_file_path.sql"),
            down_sql: None,
        },
        Migration {
            version: 13,
            name: "add_decision_file_path",
            up_sql: include_str!("013_add_decision_file_path.sql"),
            down_sql: None,
        },
        Migration {
            version: 14,
            name: "fix_initiative_status_id",
            up_sql: include_str!("014_fix_initiative_status_id.sql"),
            down_sql: None,
        },
        Migration {
            version: 15,
            name: "fix_roadmap_status_id",
            up_sql: include_str!("015_fix_roadmap_status_id.sql"),
            down_sql: None,
        },
        Migration {
            version: 16,
            name: "convert_initiative_status_to_text",
            up_sql: include_str!("016_convert_initiative_status_to_text.sql"),
            down_sql: None,
        },
    ]
}

/// Get the latest migration version
pub fn latest_version() -> i64 {
    get_migrations().last().map(|m| m.version).unwrap_or(0)
}
