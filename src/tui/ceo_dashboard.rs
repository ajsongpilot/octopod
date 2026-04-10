use anyhow::{Context, Result};
use chrono::Datelike;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Cell, Paragraph, Row, Table, Tabs, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    path::Path,
    time::{Duration, Instant},
};
use tracing::{error, info};

use crate::{
    platform::spawn_manager::{is_daemon_running, is_department_running, kill_daemon, kill_department, spawn_agent_daemon, spawn_department},
    state::{Priority as DbPriority, StateManager},
};

/// UI Department representation (maps to DB department)
#[derive(Clone)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub status: DepartmentStatus,
    pub daemon_running: bool,
    pub workspace: u8,
    pub description: String,
    pub db_department_id: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DepartmentStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// UI Decision representation (maps to DB decision)
#[derive(Clone)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub department: String,
    pub priority: Priority,
    pub severity: DecisionSeverity,
    pub status: DecisionStatus,
    pub description: String,
    pub db_decision_id: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Priority {
    P0, // Critical
    P1, // High
    P2, // Medium
    P3, // Low
}

impl From<DbPriority> for Priority {
    fn from(p: DbPriority) -> Self {
        match p {
            DbPriority::P0 => Priority::P0,
            DbPriority::P1 => Priority::P1,
            DbPriority::P2 => Priority::P2,
            DbPriority::P3 => Priority::P3,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DecisionStatus {
    Pending,
    Approved,
    Rejected,
}

impl From<crate::state::DecisionStatus> for DecisionStatus {
    fn from(s: crate::state::DecisionStatus) -> Self {
        match s {
            crate::state::DecisionStatus::Pending => DecisionStatus::Pending,
            crate::state::DecisionStatus::Approved => DecisionStatus::Approved,
            crate::state::DecisionStatus::Rejected => DecisionStatus::Rejected,
            _ => DecisionStatus::Pending,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DecisionSeverity {
    Low,
    Medium,
    High,
}

impl From<crate::state::DecisionSeverity> for DecisionSeverity {
    fn from(s: crate::state::DecisionSeverity) -> Self {
        match s {
            crate::state::DecisionSeverity::Low => DecisionSeverity::Low,
            crate::state::DecisionSeverity::Medium => DecisionSeverity::Medium,
            crate::state::DecisionSeverity::High => DecisionSeverity::High,
        }
    }
}

#[derive(Clone)]
pub struct Activity {
    pub timestamp: String,
    pub department: String,
    pub message: String,
}

#[derive(Clone)]
pub struct Initiative {
    pub id: String,
    pub title: String,
    pub description: String,
    pub department: String,
    pub department_id: String,
    pub priority: Priority,
    pub severity: DecisionSeverity,
    pub status: InitiativeStatus,
    pub stakeholder_depts: Vec<String>,
    pub pending_decision_id: Option<String>,
    pub db_initiative_id: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum InitiativeStatus {
    Draft,
    Proposed,
    StakeholderReview,
    Approved,
    Active,
    Completed,
    Closed,
    Cancelled,
    Archived,
}

impl From<crate::state::InitiativeStatus> for InitiativeStatus {
    fn from(s: crate::state::InitiativeStatus) -> Self {
        match s {
            crate::state::InitiativeStatus::Draft => InitiativeStatus::Draft,
            crate::state::InitiativeStatus::Proposed => InitiativeStatus::Proposed,
            crate::state::InitiativeStatus::StakeholderReview => {
                InitiativeStatus::StakeholderReview
            }
            crate::state::InitiativeStatus::Approved => InitiativeStatus::Approved,
            crate::state::InitiativeStatus::Active => InitiativeStatus::Active,
            crate::state::InitiativeStatus::Completed => InitiativeStatus::Completed,
            crate::state::InitiativeStatus::Closed => InitiativeStatus::Closed,
            crate::state::InitiativeStatus::Cancelled => InitiativeStatus::Cancelled,
            crate::state::InitiativeStatus::Archived => InitiativeStatus::Archived,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RoadmapStatus {
    Draft,
    Planning,
    Active,
    Completed,
    Archived,
}

impl From<crate::state::RoadmapStatus> for RoadmapStatus {
    fn from(s: crate::state::RoadmapStatus) -> Self {
        match s {
            crate::state::RoadmapStatus::Draft => RoadmapStatus::Draft,
            crate::state::RoadmapStatus::Planning => RoadmapStatus::Planning,
            crate::state::RoadmapStatus::Active => RoadmapStatus::Active,
            crate::state::RoadmapStatus::Completed => RoadmapStatus::Completed,
            crate::state::RoadmapStatus::Archived => RoadmapStatus::Archived,
        }
    }
}

#[derive(Clone)]
pub struct Roadmap {
    pub id: String,
    pub name: String,
    pub description: String,
    pub period: String,
    pub status: RoadmapStatus,
    pub db_roadmap_id: String,
}

pub struct DashboardApp {
    pub should_quit: bool,
    pub active_tab: usize,
    pub departments: Vec<Department>,
    pub selected_dept: usize,
    pub decisions: Vec<Decision>,
    pub selected_decision: usize,
    pub decision_view: DecisionViewMode,
    pub activities: Vec<Activity>,
    pub roadmaps: Vec<Roadmap>,
    pub initiatives: Vec<Initiative>,
    pub selected_initiative: usize,
    pub initiative_view: InitiativeViewMode,
    pub last_tick: Instant,
    pub state: StateManager,
    pub company_id: String,
    pub error_message: Option<String>,
    pub show_help: bool,
    pub show_kill_confirmation: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DecisionViewMode {
    Pending,      // Default - show pending decisions
    HighPriority, // CEO Queue - HIGH severity pending
    Log,          // All decisions
}

#[derive(Clone, PartialEq, Debug)]
pub enum InitiativeViewMode {
    All,       // Show all initiatives
    Active,    // Show active initiatives (in progress)
    Completed, // Show completed/closed/cancelled
}

impl DashboardApp {
    pub async fn new(project_dir: &Path) -> Result<Self> {
        info!("Initializing CEO Dashboard with database");

        // Initialize state manager
        let state = StateManager::init_for_project(project_dir)
            .await
            .context("Failed to initialize database")?;

        // Get or create company
        let company = match state.get_first_company().await? {
            Some(c) => c,
            None => {
                info!("Creating default company");
                state.create_company("Default Company").await?
            }
        };

        state.set_company(company.id.clone()).await;

        // Get or create departments
        let mut departments = match state.list_departments().await {
            Ok(depts) if !depts.is_empty() => {
                info!("Loaded {} departments from database", depts.len());
                depts
                    .into_iter()
                    .map(|d| Department {
                        id: d.slug.clone(),
                        name: d.name.clone(),
                        status: DepartmentStatus::Stopped,
                        daemon_running: false,
                        workspace: d.workspace as u8,
                        description: d.description.clone().unwrap_or_default(),
                        db_department_id: d.id,
                    })
                    .collect()
            }
            _ => {
                info!("Creating default departments");
                // Create default departments
                let default_depts = vec![
                    ("product", "Product", 2, "Roadmap & PRDs"),
                    ("engineering", "Engineering", 3, "Feature development"),
                    ("qa", "QA", 4, "Testing & Quality"),
                    ("finance", "Finance", 5, "Budgeting & Finance"),
                    ("legal", "Legal", 6, "Contracts & Legal"),
                    ("devops", "DevOps", 7, "Infrastructure"),
                    ("marketing", "Marketing", 8, "Campaigns & Marketing"),
                    ("sales", "Sales", 9, "Sales & Revenue"),
                ];

                let mut depts = Vec::new();
                for (slug, name, workspace, desc) in default_depts {
                    let db_dept = state
                        .create_department(name, slug, workspace as i64)
                        .await?;
                    depts.push(Department {
                        id: slug.to_string(),
                        name: name.to_string(),
                        status: DepartmentStatus::Stopped,
                        daemon_running: false,
                        workspace: workspace as u8,
                        description: desc.to_string(),
                        db_department_id: db_dept.id,
                    });
                }
                depts
            }
        };

        // Add CEO dashboard as first department
        departments.insert(
            0,
            Department {
                id: "ceo".to_string(),
                name: "CEO Dashboard".to_string(),
                status: DepartmentStatus::Running,
                daemon_running: false,
                workspace: 1,
                description: "This dashboard".to_string(),
                db_department_id: "ceo".to_string(),
            },
        );

        // Load decisions from database
        let decisions = match state.list_pending_decisions(100).await {
            Ok(db_decisions) => {
                info!("Loaded {} decisions from database", db_decisions.len());
                db_decisions
                    .into_iter()
                    .map(|d| Decision {
                        id: d.id[..8].to_string(), // Short ID for display
                        title: d.title.clone(),
                        department: d
                            .department_id
                            .as_ref()
                            .and_then(|id| {
                                departments.iter().find(|dept| &dept.db_department_id == id)
                            })
                            .map(|dept| dept.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string()),
                        priority: d.priority.into(),
                        severity: d.severity.into(),
                        status: d.status.into(),
                        description: d.description.unwrap_or_default(),
                        db_decision_id: d.id,
                    })
                    .collect()
            }
            Err(e) => {
                error!("Failed to load decisions: {}", e);
                Vec::new()
            }
        };

        // Load activities from messages
        let activities = match state.get_recent_messages(20).await {
            Ok(messages) => {
                info!("Loaded {} messages for activity feed", messages.len());
                messages
                    .into_iter()
                    .map(|m| Activity {
                        timestamp: m.created_at.format("%H:%M").to_string(),
                        department: "General".to_string(), // TODO: Map to department
                        message: m.content,
                    })
                    .collect()
            }
            Err(e) => {
                error!("Failed to load messages: {}", e);
                Vec::new()
            }
        };

        // Load roadmaps
        let roadmaps = match state.roadmaps().find_by_company(&company.id).await {
            Ok(db_roadmaps) => {
                info!("Loaded {} roadmaps from database", db_roadmaps.len());
                db_roadmaps
                    .into_iter()
                    .map(|r| Roadmap {
                        id: r.id[..8].to_string(),
                        name: r.name.clone(),
                        description: r.description.clone().unwrap_or_default(),
                        period: format!(
                            "{} - {}",
                            r.period_start.format("%Y-%m-%d"),
                            r.period_end.format("%Y-%m-%d")
                        ),
                        status: RoadmapStatus::from(r.status),
                        db_roadmap_id: r.id,
                    })
                    .collect()
            }
            Err(e) => {
                error!("Failed to load roadmaps: {}", e);
                Vec::new()
            }
        };

        // Load initiatives
        let mut initiatives = Vec::new();
        for roadmap in &roadmaps {
            match state
                .initiatives()
                .find_by_roadmap(&roadmap.db_roadmap_id)
                .await
            {
                Ok(db_initiatives) => {
                    for i in db_initiatives {
                        let stakeholder_depts: Vec<String> = i
                            .stakeholder_depts_json
                            .as_ref()
                            .and_then(|j| serde_json::from_str(j).ok())
                            .unwrap_or_default();

                        let dept_name = departments
                            .iter()
                            .find(|d| d.db_department_id == i.department_id)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        initiatives.push(Initiative {
                            id: i.id[..8].to_string(),
                            title: i.title.clone(),
                            description: i.description.clone().unwrap_or_default(),
                            department: dept_name,
                            department_id: i.department_id.clone(),
                            priority: Priority::from(i.priority),
                            severity: DecisionSeverity::from(i.severity),
                            status: InitiativeStatus::from(i.status),
                            stakeholder_depts,
                            pending_decision_id: i.pending_decision_id.clone(),
                            db_initiative_id: i.id,
                        });
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to load initiatives for roadmap {}: {}",
                        roadmap.name, e
                    );
                }
            }
        }

        Ok(Self {
            should_quit: false,
            active_tab: 0,
            departments,
            selected_dept: 0,
            decisions,
            selected_decision: 0,
            decision_view: DecisionViewMode::Pending,
            activities,
            roadmaps,
            initiatives,
            selected_initiative: 0,
            initiative_view: InitiativeViewMode::All,
            last_tick: Instant::now(),
            state,
            company_id: company.id,
            error_message: None,
            show_help: false,
            show_kill_confirmation: false,
        })
    }

    pub fn on_tick(&mut self) {
        // Update department statuses from tmux
        for dept in &mut self.departments {
            if dept.id == "ceo" {
                continue;
            }

            let is_running = is_department_running(&dept.id, &dept.name);
            let daemon_running = is_daemon_running(&dept.id);
            dept.daemon_running = daemon_running;
            
            match (&dept.status, is_running) {
                (DepartmentStatus::Stopped, true) => dept.status = DepartmentStatus::Running,
                (DepartmentStatus::Starting, true) => dept.status = DepartmentStatus::Running,
                (DepartmentStatus::Running, false) => dept.status = DepartmentStatus::Stopped,
                _ => {}
            }
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        // Toggle help with ?
        if key == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return;
        }
        
        // If help is showing, only Esc or ? closes it
        if self.show_help {
            if key == KeyCode::Esc {
                self.show_help = false;
            }
            return;
        }
        
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') => self.active_tab = 0,
            KeyCode::Char('2') => self.active_tab = 1,
            KeyCode::Char('3') => self.active_tab = 2,
            KeyCode::Char('4') => self.active_tab = 3,
            KeyCode::Tab => self.active_tab = (self.active_tab + 1) % 4,
            KeyCode::Down => match self.active_tab {
                0 => self.selected_dept = (self.selected_dept + 1) % self.departments.len(),
                1 => {
                    self.selected_decision =
                        (self.selected_decision + 1) % self.visible_decisions().len().max(1)
                }
                3 => {
                    self.selected_initiative =
                        (self.selected_initiative + 1) % self.visible_initiatives().len().max(1)
                }
                _ => {}
            },
            KeyCode::Up => match self.active_tab {
                0 => {
                    if self.selected_dept == 0 {
                        self.selected_dept = self.departments.len() - 1;
                    } else {
                        self.selected_dept -= 1;
                    }
                }
                1 => {
                    let len = self.visible_decisions().len();
                    if self.selected_decision == 0 {
                        self.selected_decision = len.saturating_sub(1);
                    } else {
                        self.selected_decision -= 1;
                    }
                }
                3 => {
                    if self.visible_initiatives().is_empty() {
                        self.selected_initiative = 0;
                    } else if self.selected_initiative == 0 {
                        self.selected_initiative = self.visible_initiatives().len() - 1;
                    } else {
                        self.selected_initiative -= 1;
                    }
                }
                _ => {}
            },
            KeyCode::Char('s') => self.spawn_selected(),
            KeyCode::Char('c') => self.kill_selected(),
            KeyCode::Char('k') => {
                if self.active_tab == 0 {
                    self.show_kill_confirmation = true;
                }
            }
            KeyCode::Char('y') if self.show_kill_confirmation => {
                self.confirm_kill_daemon();
            }
            KeyCode::Char('n') if self.show_kill_confirmation => {
                self.cancel_kill_daemon();
            }
            KeyCode::Char('n') => self.create_test_decision_sync(),
            KeyCode::Char('x') => self.reject_decision_sync(),
            KeyCode::Char('p') => {
                if self.active_tab == 1 {
                    self.decision_view = DecisionViewMode::Pending;
                } else {
                    self.create_roadmap_sync();
                }
            }
            KeyCode::Char('Q') => {
                if self.active_tab == 1 {
                    self.decision_view = DecisionViewMode::HighPriority;
                }
            }
            KeyCode::Char('l') => {
                if self.active_tab == 1 {
                    self.decision_view = DecisionViewMode::Log;
                }
            }
            KeyCode::Char('i') => self.create_initiative_sync(),
            KeyCode::Char('e') => {
                if self.active_tab == 1 {
                    self.edit_decision_sync();
                } else if self.active_tab == 3 {
                    self.edit_initiative_sync();
                }
            }
            KeyCode::Char('d') => {
                if self.active_tab == 3 {
                    self.draft_with_ironclaw_sync();
                }
            }
            KeyCode::Char('a') => {
                if self.active_tab == 3 {
                    self.ask_agent_sync();
                }
            }
            KeyCode::Char('v') => {
                if self.active_tab == 3 {
                    // Cycle through: All -> Active -> Done
                    match self.initiative_view {
                        InitiativeViewMode::All => self.initiative_view = InitiativeViewMode::Active,
                        InitiativeViewMode::Active => self.initiative_view = InitiativeViewMode::Completed,
                        InitiativeViewMode::Completed => self.initiative_view = InitiativeViewMode::All,
                    }
                }
            }
            KeyCode::Char('w') => {
                if self.active_tab == 3 {
                    self.initiative_view = InitiativeViewMode::Active;
                }
            }
            KeyCode::Char('r') => self.refresh_tab(),
            _ => {}
        }
    }

    fn refresh_tab(&mut self) {
        self.error_message = Some("Press 'q' to quit and restart to see changes.".to_string());
    }

    fn spawn_selected(&mut self) {
        if self.active_tab != 0 {
            return;
        }
        if let Some(dept) = self.departments.get_mut(self.selected_dept) {
            if dept.id == "ceo" {
                return;
            }

            dept.status = DepartmentStatus::Starting;
            dept.daemon_running = true;

            let dept_id = dept.id.clone();
            let name = dept.name.clone();
            let workspace = dept.workspace;

            tokio::spawn(async move {
                if let Err(e) = spawn_department(&dept_id, &name, workspace).await {
                    eprintln!("\r\nError spawning {} TUI: {}\r", name, e);
                }
                if let Err(e) = spawn_agent_daemon(&dept_id) {
                    eprintln!("\r\nError spawning {} agent: {}\r", name, e);
                }
            });
        }
    }

    fn kill_selected(&mut self) {
        if self.active_tab != 0 {
            return;
        }
        if let Some(dept) = self.departments.get(self.selected_dept) {
            if dept.id == "ceo" {
                return;
            }

            tokio::spawn({
                let dept_id = dept.id.clone();
                let dept_name = dept.name.clone();
                async move {
                    let _ = kill_department(&dept_id, &dept_name).await;
                }
            });
        }
    }

    fn kill_agent_selected(&mut self) {
        if self.active_tab != 0 {
            return;
        }
        if let Some(dept) = self.departments.get(self.selected_dept) {
            if dept.id == "ceo" {
                return;
            }

            let dept_id = dept.id.clone();
            
            // Kill daemon tmux session
            let _ = kill_daemon(&dept_id);
            info!("Killed daemon for {}", dept.name);
            
            // Kill all opencode processes for this department
            let _title_pattern = "octopod:task_:".to_string();
            if let Ok(output) = std::process::Command::new("ps")
                .args(["aux"])
                .output()
            {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if line.contains("opencode") && !line.contains("grep") && line.contains(&dept_id) {
                        if let Some(pid_str) = line.split_whitespace().nth(1) {
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                let _ = std::process::Command::new("kill")
                                    .arg(pid.to_string())
                                    .output();
                                info!("Killed process {}", pid);
                            }
                        }
                    }
                }
            }

            if let Some(d) = self.departments.get_mut(self.selected_dept) {
                d.daemon_running = false;
            }
        }
    }
    
    fn confirm_kill_daemon(&mut self) {
        self.show_kill_confirmation = false;
        self.kill_agent_selected();
    }
    
    fn cancel_kill_daemon(&mut self) {
        self.show_kill_confirmation = false;
    }

    fn visible_decisions(&self) -> Vec<&Decision> {
        match self.decision_view {
            DecisionViewMode::Pending => self
                .decisions
                .iter()
                .filter(|d| matches!(d.status, DecisionStatus::Pending))
                .collect(),
            DecisionViewMode::HighPriority => self
                .decisions
                .iter()
                .filter(|d| {
                    matches!(d.status, DecisionStatus::Pending)
                        && matches!(d.severity, DecisionSeverity::High)
                })
                .collect(),
            DecisionViewMode::Log => self.decisions.iter().collect(),
        }
    }

    fn visible_initiatives(&self) -> Vec<&Initiative> {
        match self.initiative_view {
            InitiativeViewMode::All => self.initiatives.iter().collect(),
            InitiativeViewMode::Active => self
                .initiatives
                .iter()
                .filter(|i| {
                    matches!(
                        i.status,
                        InitiativeStatus::Draft
                            | InitiativeStatus::Proposed
                            | InitiativeStatus::StakeholderReview
                            | InitiativeStatus::Approved
                            | InitiativeStatus::Active
                    )
                })
                .collect(),
            InitiativeViewMode::Completed => self
                .initiatives
                .iter()
                .filter(|i| {
                    matches!(
                        i.status,
                        InitiativeStatus::Completed | InitiativeStatus::Closed | InitiativeStatus::Cancelled
                    )
                })
                .collect(),
        }
    }

    fn reject_decision_sync(&mut self) {
        if self.active_tab != 1 {
            return;
        }
        let visible = self.visible_decisions();
        if visible.is_empty() {
            return;
        }

        if let Some(decision) = visible.get(self.selected_decision) {
            let db_id = decision.db_decision_id.clone();
            let state = self.state.clone();
            let db_id_for_ui = db_id.clone();

            tokio::spawn(async move {
                match state
                    .reject_decision(&db_id, "ceo", Some("Rejected via CEO Dashboard"))
                    .await
                {
                    Ok(_) => info!("Decision {} rejected", db_id),
                    Err(e) => error!("Failed to reject decision: {}", e),
                }
            });

            // Update UI immediately
            if let Some(d) = self
                .decisions
                .iter_mut()
                .find(|d| d.db_decision_id == db_id_for_ui)
            {
                d.status = DecisionStatus::Rejected;
            }
        }
    }

    fn edit_decision_sync(&mut self) {
        if self.active_tab != 1 {
            return;
        }
        let visible = self.visible_decisions();
        if visible.is_empty() {
            self.error_message = Some("No decision selected to edit.".to_string());
            return;
        }

        let decision_id = visible[self.selected_decision].db_decision_id.clone();

        match self.state.open_decision_editor_by_id(&decision_id) {
            Ok(_) => {
                info!("Opened decision {} in editor", decision_id);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open editor: {}", e));
                error!("Failed to open decision editor: {}", e);
            }
        }
    }

    fn create_test_decision_sync(&mut self) {
        let state = self.state.clone();

        tokio::spawn(async move {
            match state.create_decision("Test decision from dashboard").await {
                Ok(decision) => info!("Created test decision: {}", decision.id),
                Err(e) => error!("Failed to create decision: {}", e),
            }
        });

        self.error_message =
            Some("Test decision created! Restart dashboard to see it.".to_string());
    }

    fn create_roadmap_sync(&mut self) {
        use crate::state::entities::Roadmap as DbRoadmap;
        
        let company_id = self.company_id.clone();
        let now = chrono::Utc::now();
        let quarter = ((now.month0() / 3) + 1) as i32;
        let year = now.year();
        let quarter_start = chrono::NaiveDate::from_ymd_opt(year, (quarter * 3 - 2) as u32, 1).unwrap();
        let quarter_end = chrono::NaiveDate::from_ymd_opt(year, (quarter * 3 + 1) as u32, 1).unwrap() - chrono::Duration::days(1);
        let roadmap_name = format!("Q{} {} Planning", quarter, year);
        let qs = quarter_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let qe = quarter_end.and_hms_opt(23, 59, 59).unwrap().and_utc();
        
        // Generate roadmap upfront - use same ID for both UI and DB
        let roadmap_id = uuid::Uuid::new_v4().to_string();
        let db_roadmap = DbRoadmap {
            id: roadmap_id.clone(),
            company_id: company_id.clone(),
            name: roadmap_name.clone(),
            description: None,
            period_start: qs,
            period_end: qe,
            status: crate::state::RoadmapStatus::Draft,
            goals_json: None,
            created_by: None,
            created_at: now,
            updated_at: now,
        };
        
        // Add to UI immediately with the same ID
        self.roadmaps.push(Roadmap {
            id: roadmap_id[..8].to_string(),
            name: roadmap_name.clone(),
            description: String::new(),
            period: format!("{} - {}", quarter_start.format("%Y-%m-%d"), quarter_end.format("%Y-%m-%d")),
            status: RoadmapStatus::Draft,
            db_roadmap_id: roadmap_id.clone(),
        });
        
        // Save to DB in background thread (can't use block_on in async context)
        let state = self.state.clone();
        let roadmap_name_for_msg = roadmap_name.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(state.roadmaps().create(&db_roadmap)) {
                Ok(_) => info!("Created roadmap: {}", roadmap_name_for_msg),
                Err(e) => error!("Failed to create roadmap: {}", e),
            }
        });
        
        self.error_message = Some("Roadmap created!".to_string());
    }

    fn create_initiative_sync(&mut self) {
        if self.roadmaps.is_empty() {
            self.error_message = Some("Create a roadmap first (press 'p')!".to_string());
            return;
        }

        // Use the most recently created roadmap (last in list)
        let roadmap_id = self.roadmaps.last().unwrap().db_roadmap_id.clone();
        
        // Find first non-CEO department
        let (dept_id, dept_name) = self
            .departments
            .iter()
            .find(|d| d.id != "ceo")
            .map(|d| (d.db_department_id.clone(), d.name.clone()))
            .unwrap_or_else(|| ("".to_string(), "Unknown".to_string()));

        // Generate initiative ID upfront - use same ID for both UI and DB
        let initiative_id = uuid::Uuid::new_v4().to_string();
        
        // Immediately add to UI with the same ID
        self.initiatives.push(Initiative {
            id: initiative_id[..8].to_string(),
            title: "New Initiative".to_string(),
            description: String::new(),
            department: dept_name,
            department_id: dept_id.clone(),
            priority: Priority::P2,
            severity: DecisionSeverity::Medium,
            status: InitiativeStatus::Draft,
            stakeholder_depts: Vec::new(),
            pending_decision_id: None,
            db_initiative_id: initiative_id.clone(),
        });

        // Save to DB in background thread with same ID
        let state = self.state.clone();
        let rid = roadmap_id.clone();
        let did = dept_id.clone();
        let iid = initiative_id.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(state.create_initiative_with_id(&rid, &did, "New Initiative", &iid)) {
                Ok(i) => info!("Created initiative: {} ({})", i.title, i.id),
                Err(e) => error!("Failed to create initiative: {}", e),
            }
        });

        self.error_message = Some("Initiative created!".to_string());
    }

    fn edit_initiative_sync(&mut self) {
        if self.active_tab != 3 {
            return;
        }
        let visible = self.visible_initiatives();
        if visible.is_empty() {
            self.error_message = Some("No initiative to edit.".to_string());
            return;
        }

        let initiative = visible[self.selected_initiative];
        let initiative_id = initiative.db_initiative_id.clone();

        // Sync title from markdown file (if it exists and has a different title)
        if let Some(md_title) = self.state.get_initiative_title_from_file(&initiative_id) {
            if md_title != initiative.title {
                // Find and update in the full list
                if let Some(full_init) = self.initiatives.iter_mut().find(|i| i.db_initiative_id == initiative_id) {
                    full_init.title = md_title.clone();
                    info!("Synced initiative title: {}", md_title);
                }
            }
        }

        match self.state.open_initiative_editor_by_id(&initiative_id) {
            Ok(_) => {
                info!("Opened initiative {} in editor", initiative_id);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open editor: {}", e));
                error!("Failed to open initiative editor: {}", e);
            }
        }
    }

    fn draft_with_ironclaw_sync(&mut self) {
        if self.active_tab != 3 {
            return;
        }
        let visible = self.visible_initiatives();
        if visible.is_empty() {
            self.error_message = Some("No initiative to draft.".to_string());
            return;
        }

        let initiative = visible[self.selected_initiative];
        let initiative_id = initiative.db_initiative_id.clone();
        let initiative_title = initiative.title.clone();

        // Get the markdown file path
        let file_path = self.state.initiative_file_manager().get_file_path(&initiative_id);
        
        if !file_path.exists() {
            self.error_message = Some("No markdown file found. Create initiative first.".to_string());
            return;
        }

        self.error_message = Some("Asking Ironclaw to help draft requirements...".to_string());

        // Read current content
        let current_content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                self.error_message = Some(format!("Failed to read file: {}", e));
                return;
            }
        };

        // Build prompt for Ironclaw
        let prompt = format!(
            r#"Help me draft requirements for the initiative "{}". 

Review the current initiative markdown below and improve the Problem Statement, Goals, and Key Results sections. Make them specific, measurable, and actionable.

Current content:
{}

Please provide an improved version with:
1. A clear PROBLEM STATEMENT that explains why this initiative matters
2. 3-5 specific GOALS that are outcome-oriented
3. 3-5 KEY RESULTS that are measurable (use format: KR1: [Measurable outcome])

Only output the improved markdown content - do not include any preamble or explanation."#,
            initiative_title,
            current_content
        );

        // Invoke ironclaw in a background thread
        std::thread::spawn(move || {
            let output = std::process::Command::new("ironclaw")
                .args(["run", "--message", &prompt])
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        let ironclaw_response = String::from_utf8_lossy(&result.stdout);
                        // Only use response if it's substantial (not an error or empty)
                        if !ironclaw_response.trim().is_empty() 
                            && !ironclaw_response.contains("error")
                            && ironclaw_response.len() > 100 {
                            if let Err(e) = std::fs::write(&file_path, ironclaw_response.as_ref()) {
                                error!("Failed to write ironclaw response: {}", e);
                            } else {
                                info!("Ironclaw drafted requirements for initiative");
                            }
                        } else {
                            // Fallback: just open in editor if response is empty/error
                            if let Err(e) = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(format!("tmux new-window -n 'initiative-draft' '$EDITOR {}'", file_path.display()))
                                .spawn()
                            {
                                error!("Failed to spawn editor: {}", e);
                            }
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        error!("Ironclaw failed: {}", stderr);
                    }
                }
                Err(e) => {
                    error!("Failed to run ironclaw: {}. Is ironclaw installed?", e);
                }
            }
        });

        self.error_message = Some("Ironclaw is drafting... Check tmux window for results.".to_string());
    }

    fn ask_agent_sync(&mut self) {
        if self.active_tab != 3 {
            return;
        }
        let visible = self.visible_initiatives();
        if visible.is_empty() {
            self.error_message = Some("No initiative to discuss.".to_string());
            return;
        }

        let initiative = visible[self.selected_initiative];
        let initiative_id = initiative.db_initiative_id.clone();
        let initiative_title = initiative.title.clone();
        let file_path = self.state.initiative_file_manager().get_file_path(&initiative_id);

        if !file_path.exists() {
            self.error_message = Some("No markdown file found.".to_string());
            return;
        }

        // Read the initiative content (validates file is readable before opening tmux)
        let _initiative_content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                self.error_message = Some(format!("Failed to read initiative: {}", e));
                return;
            }
        };

        self.error_message = Some("Opening Ironclaw chat... Check tmux window.".to_string());

        // Create an interactive tmux window for chatting with Ironclaw
        let script = format!(r#"#!/bin/bash
set -e

ORIGINAL_FILE="{file_path}"
BACKUP_FILE="/tmp/octopod-initiative-backup-$BASHPID.md"

clear
echo "=============================================="
echo "Ironclaw Initiative Chat"
echo "=============================================="
echo "Initiative: {title}"
echo "File: $ORIGINAL_FILE"
echo "=============================================="
echo ""
echo "Commands:"
echo "  Type 'exit' or 'q' to quit"
echo "  Type 'y' to approve Ironclaw's suggested changes"
echo ""
echo "--- Initiative Content ---"
cat "$ORIGINAL_FILE"
echo ""
echo "--- End Initiative Content ---"
echo ""

# Create backup before any changes
cp "$ORIGINAL_FILE" "$BACKUP_FILE"

while true; do
    echo -n "You: "
    read -r user_input
    if [ -z "$user_input" ]; then
        continue
    fi
    if [ "$user_input" = "exit" ] || [ "$user_input" = "quit" ] || [ "$user_input" = "q" ]; then
        echo "Goodbye!"
        rm -f "$BACKUP_FILE"
        break
    fi
    
    # Restore backup before each interaction
    cp "$BACKUP_FILE" "$ORIGINAL_FILE"
    echo ""
    echo "Ironclaw is thinking..."
    
    # Run ironclaw and capture the response
    response=$(ironclaw run --message "Context: You are helping draft an initiative. Here is the initiative file at $ORIGINAL_FILE:

$(cat '$ORIGINAL_FILE')

---

User question: $user_input" 2>&1)
    
    echo "$response"
    echo ""
    
    # Check if ironclaw wants to make changes and prompt for approval
    if echo "$response" | grep -q "apply_patch requires approval"; then
        echo "=============================================="
        echo "Ironclaw wants to update the initiative file!"
        echo "=============================================="
        
        # Extract the patched file path more robustly
        # Look for 'path:' followed by a file path (handles backticks, spaces, etc.)
        tmpfile=""
        if echo "$response" | grep -q 'path:'; then
            # Extract the line with path:, then extract the actual path
            # Handle cases like: `path: /tmp/file.md` or path: /tmp/file.md
            tmpfile=$(echo "$response" | grep 'path:' | head -1 | sed -E 's/.*path: *[`]?([^ `]*)[`]?.*/\1/' | tr -d '`')
        fi
        
        if [ -z "$tmpfile" ]; then
            echo "WARNING: Could not extract patch file path from response."
            echo "Ironclaw may need manual file editing tools enabled."
            echo ""
        elif [ ! -f "$tmpfile" ]; then
            echo "WARNING: Patch file does not exist: $tmpfile"
            echo ""
        else
            echo "--- Diff Preview ---"
            diff -u "$ORIGINAL_FILE" "$tmpfile" || true
            echo "--- End Diff ---"
            echo ""
            echo "Type 'y' and press Enter to approve these changes,"
            echo "or just press Enter to skip."
            read -r approve
            if [ "$approve" = "y" ] || [ "$approve" = "Y" ]; then
                cp "$tmpfile" "$ORIGINAL_FILE"
                echo "Applied Ironclaw's changes!"
            else
                echo "Changes skipped. Your file is unchanged."
            fi
        fi
    fi
done
"#, title = initiative_title, file_path = file_path.display());

        let script_path = format!("/tmp/octopod-ironclaw-chat-{}.sh", &initiative_id[..8]);
        if let Err(e) = std::fs::write(&script_path, script) {
            self.error_message = Some(format!("Failed to create script: {}", e));
            return;
        }

        // Make script executable
        let _ = std::process::Command::new("chmod")
            .args(["+x", &script_path])
            .output();

        // Open in tmux
        let window_name = format!("initiative-chat-{}", &initiative_id[..8]);
        std::thread::spawn(move || {
            let _ = std::process::Command::new("tmux")
                .args(["new-window", "-n", &window_name, &script_path])
                .spawn();
        });
    }
}

pub async fn run_ceo_dashboard() -> Result<()> {
    // Set environment variable so spawn_department knows not to spawn terminals
    std::env::set_var("OCTOPOD_IN_TUI", "1");

    // Rename tmux window to "CEO Dashboard" if inside tmux
    if std::env::var("TMUX").is_ok() {
        let _ = std::process::Command::new("tmux")
            .args(["rename-window", "CEO Dashboard"])
            .output();
    }

    // Find project directory (look for .octopod)
    let current_dir = std::env::current_dir()?;
    let project_dir = find_project_dir(&current_dir).unwrap_or(current_dir);

    info!("Starting CEO Dashboard in project: {:?}", project_dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = DashboardApp::new(&project_dir).await?;
    let res = run_app(&mut terminal, app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Dashboard error: {:?}", err);
    }

    Ok(())
}

fn find_project_dir(start: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".octopod").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: DashboardApp) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    // Header
    let header = Paragraph::new(" Octopod CEO Dashboard ")
        .style(
            Style::default()
                .fg(Color::Rgb(255, 127, 80))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226))),
        );
    f.render_widget(header, chunks[0]);

    // Tabs
    let titles = vec!["Departments", "Decisions", "Activity", "Planning"];
    let tabs = Tabs::new(titles)
        .select(app.active_tab)
        .style(Style::default().fg(Color::Rgb(180, 180, 180)))
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(255, 127, 80))
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[1]);

    // Content based on active tab
    match app.active_tab {
        0 => render_departments_tab(f, app, chunks[2]),
        1 => render_decisions_tab(f, app, chunks[2]),
        2 => render_activity_tab(f, app, chunks[2]),
        3 => render_planning_tab(f, app, chunks[2]),
        _ => {}
    }

        // Footer
    let help: String = match app.active_tab {
        0 => "[s]pawn [c]lose [k]ill daemon [↑↓]nav [?]help [q]uit".to_string(),
        1 => {
            let view_hint = match app.decision_view {
                DecisionViewMode::Pending => "[p]ending [Q]CEO Queue [l]og",
                DecisionViewMode::HighPriority => "[p]ending [Q]CEO Queue [l]og",
                DecisionViewMode::Log => "[p]ending [Q]CEO Queue [l]og",
            };
            format!("[a]pprove [x]reject [e]dit [↑↓]nav {} [?]help [q]uit", view_hint)
        }
        2 => "[↑↓]scroll [1/2/3/4]tabs [?]help [q]uit".to_string(),
        3 => "[p]lan [i]nit [d]raft [a]sk [e]dit [v]iew [w]ip [r]efresh [↑↓]nav [?]help [q]uit".to_string(),
        _ => "[Esc]quit".to_string(),
    };
    let footer = Paragraph::new(help)
        .style(Style::default().fg(Color::Rgb(180, 180, 180)))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226))),
        );
    f.render_widget(footer, chunks[3]);

    // Show help overlay if enabled
    if app.show_help {
        render_help_overlay(f, app);
    }

    // Show kill confirmation modal if enabled
    if app.show_kill_confirmation {
        render_kill_confirmation(f, app);
    }
}

fn render_help_overlay(f: &mut Frame, _app: &DashboardApp) {
    let size = f.size();

    let help_text = "OCTOPOD CEO DASHBOARD

NAVIGATION
  Tab / 1-4    Switch tabs
  Up / Down    Navigate

DEPARTMENTS TAB
  s            Spawn selected department
  c            Close department TUI
  k            Kill daemon

DECISIONS TAB
  a            Approve selected decision
  x            Reject selected decision
  e            Edit selected decision (markdown)
  p            Show pending decisions
  Q            Show CEO Queue (HIGH only)
  l            Show decision log

PLANNING TAB
  p            Create roadmap
  i            Create initiative
  d            Draft with Ironclaw (one-shot)
  a            Ask agent about initiative (interactive chat)
  e            Edit selected initiative
  v            Cycle view (all -> active -> done)
  w            Show work-in-progress only
  r            Refresh

GENERAL
  ?            Toggle this help
  q / Esc      Quit";

    let width = 60.min(size.width.saturating_sub(4));
    let height = 28.min(size.height.saturating_sub(2));
    let left = (size.width - width) / 2;
    let top = (size.height - height) / 2;

    let area = Rect::new(left, top, width, height);

    let block = Block::default()
        .title(" Help ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .block(block);

    f.render_widget(paragraph, area);
}

fn render_kill_confirmation(f: &mut Frame, app: &DashboardApp) {
    use std::process::Command;
    
    let size = f.size();
    
    // Get the selected department name
    let dept_name = app.departments
        .get(app.selected_dept)
        .map(|d| d.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Query opencode sessions for this department to show what will be killed
    let sessions_to_kill: Vec<String> = Command::new("opencode")
        .args(["session", "list", "--format", "json"])
        .output()
        .map(|output| {
            let output_str = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str::<Vec<serde_json::Value>>(&output_str)
                .ok()
                .map(|sessions| {
                    sessions
                        .into_iter()
                        .filter(|s| {
                            if let Some(title) = s.get("title").and_then(|t| t.as_str()) {
                                title.starts_with("octopod:") && title.contains(&dept_name)
                            } else {
                                false
                            }
                        })
                        .filter_map(|s| s.get("title").and_then(|t| t.as_str()).map(|t| t.to_string()))
                        .collect()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let session_count = sessions_to_kill.len();
    
    let width = 60.min(size.width.saturating_sub(4));
    let height = if session_count > 0 { 12 } else { 8 };
    let left = (size.width - width) / 2;
    let top = (size.height - height) / 2;

    let area = Rect::new(left, top, width, height);

    let block = Block::default()
        .title(" ⚠️ Kill Daemon ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![
        format!("Kill daemon for {}?", dept_name),
        "".to_string(),
    ];
    
    if session_count > 0 {
        lines.push(format!("This will kill {} opencode session(s):", session_count));
        for session in sessions_to_kill.iter().take(3) {
            lines.push(format!("  • {}", truncate(session, 50)));
        }
        if session_count > 3 {
            lines.push(format!("  ... and {} more", session_count - 3));
        }
    } else {
        lines.push("No active sessions.".to_string());
    }
    
    lines.push("".to_string());
    lines.push("Sessions will persist in opencode and can be resumed.".to_string());
    lines.push("".to_string());
    lines.push("[y] Yes, kill all  [n] or [Esc] Cancel".to_string());

    let paragraph = Paragraph::new(lines.join("\n"))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, inner);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn render_departments_tab(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let rows: Vec<Row> = app
        .departments
        .iter()
        .enumerate()
        .map(|(i, dept)| {
            let style = if i == app.selected_dept {
                Style::default()
                    .bg(Color::Rgb(60, 60, 80))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let tui_status = match &dept.status {
                DepartmentStatus::Stopped => ("Stopped", Color::Gray),
                DepartmentStatus::Starting => ("Starting", Color::Yellow),
                DepartmentStatus::Running => ("Running", Color::Green),
                DepartmentStatus::Error(_) => ("Error", Color::Red),
            };
            
            // CEO has no agent
            let (daemon_status_text, daemon_color) = if dept.id == "ceo" {
                ("N/A", Color::DarkGray)
            } else if dept.daemon_running {
                ("Running", Color::Green)
            } else {
                ("Stopped", Color::Gray)
            };

            let hotkey = if i == app.selected_dept { "[s]" } else { "   " };

            Row::new(vec![
                Cell::from(hotkey).style(Style::default().fg(Color::Rgb(138, 43, 226))),
                Cell::from(dept.name.clone()),
                Cell::from(tui_status.0).style(Style::default().fg(tui_status.1)),
                Cell::from(daemon_status_text).style(Style::default().fg(daemon_color)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec!["", "Department", "TUI", "Agent Daemon"]).style(
                Style::default()
                    .fg(Color::Rgb(255, 127, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .widths(&[
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(12),
        ])
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(table, area);
}

fn render_decisions_tab(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let visible: Vec<&Decision> = app.visible_decisions();

    let view_title = match app.decision_view {
        DecisionViewMode::Pending => "Pending Decisions",
        DecisionViewMode::HighPriority => "HIGH SEVERITY - CEO Queue",
        DecisionViewMode::Log => "Decision Log (All)",
    };

    if visible.is_empty() {
        let msg = match app.decision_view {
            DecisionViewMode::Pending => {
                "No pending decisions. [n] create test | [Q] CEO Queue | [l] Log"
            }
            DecisionViewMode::HighPriority => {
                "No high-severity pending. All clear! [p] Pending | [l] Log"
            }
            DecisionViewMode::Log => "No decisions yet. [p] Pending | [Q] CEO Queue",
        };
        let para = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(view_title));
        f.render_widget(para, area);
        return;
    }

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, decision)| {
            let style = if i == app.selected_decision {
                Style::default()
                    .bg(Color::Rgb(60, 60, 80))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let priority_color = match decision.priority {
                Priority::P0 => Color::Red,
                Priority::P1 => Color::Yellow,
                Priority::P2 => Color::Rgb(255, 127, 80),
                Priority::P3 => Color::Gray,
            };

            let severity_color = match decision.severity {
                DecisionSeverity::High => Color::Red,
                DecisionSeverity::Medium => Color::Yellow,
                DecisionSeverity::Low => Color::Gray,
            };

            let severity_str = match decision.severity {
                DecisionSeverity::High => "🔴 HIGH",
                DecisionSeverity::Medium => "🟡 MED",
                DecisionSeverity::Low => "⚪ LOW",
            };

            let status_str = match decision.status {
                DecisionStatus::Pending => "⏳ Pending",
                DecisionStatus::Approved => "Approved",
                DecisionStatus::Rejected => "Rejected",
            };

            Row::new(vec![
                Cell::from(decision.id.clone()),
                Cell::from(decision.title.clone()),
                Cell::from(decision.department.clone()),
                Cell::from(severity_str).style(Style::default().fg(severity_color)),
                Cell::from(format!("{:?}", decision.priority))
                    .style(Style::default().fg(priority_color)),
                Cell::from(status_str),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec![
                "ID", "Decision", "Dept", "Severity", "Priority", "Status",
            ])
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 127, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .widths(&[
            Constraint::Length(8),
            Constraint::Length(35),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ])
        .block(Block::default().borders(Borders::ALL).title(view_title));

    f.render_widget(table, area);
}

fn render_activity_tab(f: &mut Frame, app: &DashboardApp, area: Rect) {
    if app.activities.is_empty() {
        let msg = Paragraph::new("No recent activity.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, area);
        return;
    }

    let text: Vec<String> = app
        .activities
        .iter()
        .map(|a| format!("{} - {}: {}", a.timestamp, a.department, a.message))
        .collect();

    let paragraph = Paragraph::new(text.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Activity"),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_planning_tab(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Roadmap header
            Constraint::Min(5),    // Initiatives list
        ])
        .split(area);

    // Roadmap section
    let roadmap_header = if app.roadmaps.is_empty() {
        Paragraph::new("No roadmaps yet. Press [p] to create one.")
            .style(Style::default().fg(Color::Rgb(180, 180, 180)))
    } else {
        let roadmap = &app.roadmaps[0];
        Paragraph::new(format!(
            "{} | {} | Status: {:?}",
            roadmap.name, roadmap.period, roadmap.status
        ))
        .style(
            Style::default()
                .fg(Color::Rgb(255, 127, 80))
                .add_modifier(Modifier::BOLD),
        )
    };
    f.render_widget(
        roadmap_header.block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current Roadmap"),
        ),
        chunks[0],
    );

    // Initiatives section
    if app.initiatives.is_empty() {
        let msg = Paragraph::new("No initiatives yet. Press [i] to create one.\n\nInitiatives are cross-department goals. Each initiative has a primary department owner and stakeholder departments that need to review/approve.")
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Initiatives"));
        f.render_widget(msg, chunks[1]);
        return;
    }

    let rows: Vec<Row> = app
        .initiatives
        .iter()
        .enumerate()
        .map(|(i, initiative)| {
            let style = if i == app.selected_initiative {
                Style::default()
                    .bg(Color::Rgb(60, 60, 80))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let priority_color = match initiative.priority {
                Priority::P0 => Color::Red,
                Priority::P1 => Color::Rgb(255, 127, 80),
                Priority::P2 => Color::Yellow,
                Priority::P3 => Color::Gray,
            };

            let status_str = match initiative.status {
                InitiativeStatus::Draft => "Draft",
                InitiativeStatus::Proposed => "Proposed",
                InitiativeStatus::StakeholderReview => "Review",
                InitiativeStatus::Approved => "Approved",
                InitiativeStatus::Active => "Active",
                InitiativeStatus::Completed => "Done",
                InitiativeStatus::Closed => "Closed",
                InitiativeStatus::Cancelled => "Cancelled",
                InitiativeStatus::Archived => "Archived",
            };

            let stakeholders = if initiative.stakeholder_depts.is_empty() {
                "None".to_string()
            } else {
                initiative.stakeholder_depts.join(", ")
            };

            Row::new(vec![
                Cell::from(initiative.id.clone()),
                Cell::from(initiative.title.clone()),
                Cell::from(initiative.department.clone()),
                Cell::from(format!("{:?}", initiative.priority))
                    .style(Style::default().fg(priority_color)),
                Cell::from(status_str),
                Cell::from(stakeholders),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec![
                "ID",
                "Initiative",
                "Owner",
                "Priority",
                "Status",
                "Stakeholders",
            ])
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 127, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .widths(&[
            Constraint::Length(8),
            Constraint::Length(25),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(20),
        ])
        .block(Block::default().borders(Borders::ALL).title("Initiatives"));

    f.render_widget(table, chunks[1]);
}
