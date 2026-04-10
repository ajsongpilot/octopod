use crate::platform::spawn_manager::is_department_running;
use anyhow::Result;

pub async fn run() -> Result<()> {
    let departments = [
        ("ceo", "CEO Dashboard", 1u8),
        ("product", "Product", 2u8),
        ("engineering", "Engineering", 3u8),
        ("qa", "QA", 4u8),
        ("finance", "Finance", 5u8),
        ("legal", "Legal", 6u8),
        ("devops", "DevOps", 7u8),
        ("marketing", "Marketing", 8u8),
        ("sales", "Sales", 9u8),
    ];

    println!("🐙 Department Status\n");

    for (id, name, workspace) in &departments {
        let status = if *id == "ceo" {
            "▶ Running (this terminal)"
        } else if is_department_running(id, name) {
            "▶ Running"
        } else {
            "⏹ Stopped"
        };

        let status_color = if status.contains("Running") {
            "\x1b[32m" // Green
        } else {
            "\x1b[90m" // Gray
        };
        let reset = "\x1b[0m";

        println!(
            "  Super+{}: {:15} {}{}",
            workspace, name, status_color, status
        );
        print!("{}", reset);
    }

    println!();

    Ok(())
}
