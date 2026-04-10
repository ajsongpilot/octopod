use anyhow::Result;

/// Renders the CEO Dashboard as text for debugging
/// Run: octopod dashboard --mock
pub fn render_mock_dashboard() -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║               🐙 Octopod - CEO Dashboard                 ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\n\n");

    // Department table
    output.push_str("┌────────────────────────────────────────────────────────┐\n");
    output.push_str("│ Departments                                            │\n");
    output.push_str("├──────────┬───────────────┬──────────────────┬──────────┤\n");
    output.push_str("│ Workspace│ Department    │ Description      │ Status   │\n");
    output.push_str("├──────────┼───────────────┼──────────────────┼──────────┤\n");

    let departments = [
        (1, "CEO Dashboard", "This dashboard", "▶ Running", "green"),
        (2, "Product", "Roadmap & PRDs", "⏹ Stopped", "gray"),
        (3, "Engineering", "Feature dev", "⏹ Stopped", "gray"),
        (4, "QA", "Testing", "⏹ Stopped", "gray"),
        (5, "Finance", "Budgeting", "⏹ Stopped", "gray"),
        (6, "Legal", "Contracts", "⏹ Stopped", "gray"),
        (7, "DevOps", "Infrastructure", "⏹ Stopped", "gray"),
        (8, "Marketing", "Campaigns", "⏹ Stopped", "gray"),
        (9, "Sales", "Revenue", "⏹ Stopped", "gray"),
    ];

    for (workspace, name, desc, status, _color) in &departments {
        output.push_str(&format!(
            "│ Super+{:2} │ {:13} │ {:16} │ {:8} │\n",
            workspace, name, desc, status
        ));
    }

    output.push_str("└──────────┴───────────────┴──────────────────┴──────────┘\n\n");

    // Activity Feed section (placeholder)
    output.push_str("┌────────────────────────────────────────────────────────┐\n");
    output.push_str("│ Recent Activity                                        │\n");
    output.push_str("├────────────────────────────────────────────────────────┤\n");
    output.push_str("│ 10:23 AM - Engineering: Started work on auth feature   │\n");
    output.push_str("│ 10:15 AM - Product: Created PRD for user profiles      │\n");
    output.push_str("│ 09:45 AM - QA: Found bug #127 in signup flow          │\n");
    output.push_str("└────────────────────────────────────────────────────────┘\n\n");

    // Issues section (placeholder)
    output.push_str("┌────────────────────────────────────────────────────────┐\n");
    output.push_str("│ Open Issues                                            │\n");
    output.push_str("├────────────────────────────────────────────────────────┤\n");
    output.push_str("│ #128  P0  Engineering  Auth token refresh failing     │\n");
    output.push_str("│ #127  P1  QA           Signup flow broken on mobile   │\n");
    output.push_str("│ #125  P2  Product      Clarify user profile fields    │\n");
    output.push_str("└────────────────────────────────────────────────────────┘\n\n");

    // Footer
    output.push_str("┌────────────────────────────────────────────────────────┐\n");
    output.push_str("│ [s]pawn [k]ill [a]ll [↑↓]nav [q]uit                   │\n");
    output.push_str("└────────────────────────────────────────────────────────┘\n");

    Ok(output)
}

/// Export current dashboard state to a file for debugging
pub fn export_dashboard_state(app: &crate::tui::app::App) -> Result<String> {
    let mut output = String::new();

    output.push_str("=== Octopod CEO Dashboard State ===\n\n");
    output.push_str(&format!("Selected Department: {}\n", app.selected_dept));
    output.push_str(&format!("Should Quit: {}\n\n", app.should_quit));

    output.push_str("Departments:\n");
    output.push_str("────────────\n");

    for (i, dept) in app.departments.iter().enumerate() {
        let marker = if i == app.selected_dept {
            ">>> "
        } else {
            "    "
        };
        output.push_str(&format!(
            "{}Super+{}: {:15} - {:?}\n",
            marker, dept.workspace, dept.name, dept.status
        ));
    }

    output.push('\n');
    output.push_str(&format!("Spawn Requests: {:?}\n", app.spawn_requests));
    output.push_str(&format!("Kill Requests: {:?}\n", app.kill_requests));

    Ok(output)
}
