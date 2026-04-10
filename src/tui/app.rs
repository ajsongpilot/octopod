use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::platform::spawn_manager::{is_department_running, kill_department, spawn_department};

pub struct App {
    pub should_quit: bool,
    pub departments: Vec<Department>,
    pub selected_dept: usize,
    pub last_tick: Instant,
    pub spawn_requests: Vec<String>, // Department IDs to spawn
    pub kill_requests: Vec<String>,  // Department IDs to kill
}

#[derive(Clone)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub status: DepartmentStatus,
    pub workspace: u8,
    pub description: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DepartmentStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
    Paused,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let departments = vec![
            Department {
                id: "ceo".to_string(),
                name: "CEO Dashboard (You are here)".to_string(),
                status: DepartmentStatus::Running,
                workspace: 1,
                description: "This dashboard - already active".to_string(),
            },
            Department {
                id: "product".to_string(),
                name: "Product".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 2,
                description: "Roadmap, PRDs, requirements".to_string(),
            },
            Department {
                id: "engineering".to_string(),
                name: "Engineering".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 3,
                description: "Feature implementation".to_string(),
            },
            Department {
                id: "qa".to_string(),
                name: "QA".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 4,
                description: "Testing and validation".to_string(),
            },
            Department {
                id: "finance".to_string(),
                name: "Finance".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 5,
                description: "Budgeting and financial planning".to_string(),
            },
            Department {
                id: "legal".to_string(),
                name: "Legal".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 6,
                description: "Contracts and compliance".to_string(),
            },
            Department {
                id: "devops".to_string(),
                name: "DevOps".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 7,
                description: "Infrastructure and deployment".to_string(),
            },
            Department {
                id: "marketing".to_string(),
                name: "Marketing".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 8,
                description: "Campaigns and growth".to_string(),
            },
            Department {
                id: "sales".to_string(),
                name: "Sales".to_string(),
                status: DepartmentStatus::Stopped,
                workspace: 9,
                description: "Revenue and customer acquisition".to_string(),
            },
        ];

        Self {
            should_quit: false,
            departments,
            selected_dept: 0,
            last_tick: Instant::now(),
            spawn_requests: Vec::new(),
            kill_requests: Vec::new(),
        }
    }

    pub fn on_tick(&mut self) {
        // Check actual tmux session status for each department
        for dept in &mut self.departments {
            // Skip CEO - it's always running (this dashboard)
            if dept.id == "ceo" {
                continue;
            }

            let is_running = is_department_running(&dept.id, &dept.name);
            match (&dept.status, is_running) {
                (DepartmentStatus::Stopped, true) => {
                    dept.status = DepartmentStatus::Running;
                }
                (DepartmentStatus::Starting, true) => {
                    dept.status = DepartmentStatus::Running;
                }
                (DepartmentStatus::Running, false) => {
                    dept.status = DepartmentStatus::Stopped;
                }
                (DepartmentStatus::Error(_), true) => {
                    dept.status = DepartmentStatus::Running;
                }
                _ => {}
            }
        }
    }

    pub fn process_spawn_requests(&mut self) -> Vec<(String, String, u8)> {
        let mut to_spawn = Vec::new();
        for dept_id in self.spawn_requests.drain(..) {
            if let Some(dept) = self.departments.iter_mut().find(|d| d.id == dept_id) {
                dept.status = DepartmentStatus::Starting;
                to_spawn.push((dept.id.clone(), dept.name.clone(), dept.workspace));
            }
        }
        to_spawn
    }

    pub fn process_kill_requests(&mut self) -> Vec<(String, String)> {
        let mut to_kill = Vec::new();
        for dept_id in self.kill_requests.drain(..) {
            if let Some(dept) = self.departments.iter_mut().find(|d| d.id == dept_id) {
                dept.status = DepartmentStatus::Stopped;
                to_kill.push((dept.id.clone(), dept.name.clone()));
            }
        }
        to_kill
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('s') => self.spawn_selected(),
            KeyCode::Char('k') => self.kill_selected(),
            KeyCode::Down => {
                self.selected_dept = (self.selected_dept + 1) % self.departments.len();
            }
            KeyCode::Up => {
                if self.selected_dept == 0 {
                    self.selected_dept = self.departments.len() - 1;
                } else {
                    self.selected_dept -= 1;
                }
            }
            KeyCode::Char('a') => self.spawn_all(),
            _ => {}
        }
    }

    fn spawn_selected(&mut self) {
        if let Some(dept) = self.departments.get(self.selected_dept) {
            // Skip CEO dashboard - it's already running
            if dept.id == "ceo" {
                return;
            }
            match &dept.status {
                DepartmentStatus::Stopped | DepartmentStatus::Error(_) => {
                    self.spawn_requests.push(dept.id.clone());
                }
                _ => {}
            }
        }
    }

    fn kill_selected(&mut self) {
        if let Some(dept) = self.departments.get(self.selected_dept) {
            // Skip CEO dashboard - can't kill the dashboard you're using
            if dept.id == "ceo" {
                return;
            }
            match dept.status {
                DepartmentStatus::Running | DepartmentStatus::Starting => {
                    self.kill_requests.push(dept.id.clone());
                }
                _ => {}
            }
        }
    }

    fn spawn_all(&mut self) {
        for dept in &self.departments {
            if dept.id != "ceo" {
                match &dept.status {
                    DepartmentStatus::Stopped | DepartmentStatus::Error(_) => {
                        self.spawn_requests.push(dept.id.clone());
                    }
                    _ => {}
                }
            }
        }
    }
}

pub async fn run_dashboard() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Dashboard error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::Hide)?;
        terminal.draw(|f| ui(f, &app))?;
        execute!(std::io::stdout(), crossterm::cursor::Show)?;

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

        // Process spawn and kill requests
        let spawn_requests = app.process_spawn_requests();
        for (id, name, workspace) in spawn_requests {
            if let Err(e) = spawn_department(&id, &name, workspace).await {
                // Update status to error
                if let Some(dept) = app.departments.iter_mut().find(|d| d.id == id) {
                    dept.status = DepartmentStatus::Error(e.to_string());
                }
            }
        }

        let kill_requests = app.process_kill_requests();
        for (id, name) in kill_requests {
            if let Err(e) = kill_department(&id, &name).await {
                eprintln!("Failed to kill {}: {}", id, e);
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

fn ui(f: &mut Frame, app: &App) {
    // Clear entire frame first to avoid border artifacts
    f.render_widget(Clear, f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Header
    let header = Paragraph::new("🐙 Octopod - CEO Dashboard")
        .style(
            Style::default()
                .fg(Color::Rgb(255, 127, 80))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226))),
        );
    f.render_widget(header, chunks[0]);

    // Department list
    let departments: Vec<Row> = app
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

            let status_str = match &dept.status {
                DepartmentStatus::Stopped => "⏹ Stopped",
                DepartmentStatus::Starting => "🔄 Starting...",
                DepartmentStatus::Running => "▶ Running",
                DepartmentStatus::Error(msg) => &format!("⚠ {}", msg),
                DepartmentStatus::Paused => "⏸ Paused",
            };

            let status_color = match &dept.status {
                DepartmentStatus::Stopped => Color::Gray,
                DepartmentStatus::Starting => Color::Yellow,
                DepartmentStatus::Running => Color::Green,
                DepartmentStatus::Error(_) => Color::Red,
                DepartmentStatus::Paused => Color::Yellow,
            };

            Row::new(vec![
                Cell::from(format!("Super+{}", dept.workspace))
                    .style(Style::default().fg(Color::Rgb(138, 43, 226))),
                Cell::from(dept.name.clone()).style(Style::default().fg(Color::Rgb(255, 255, 255))),
                Cell::from(dept.description.clone())
                    .style(Style::default().fg(Color::Rgb(180, 180, 180))),
                Cell::from(status_str.to_string()).style(Style::default().fg(status_color)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(departments)
        .header(
            Row::new(vec!["Workspace", "Department", "Description", "Status"])
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 127, 80))
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(35),
            Constraint::Length(15),
        ])
        .block(
            Block::default()
                .title("Departments")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226))),
        );

    f.render_widget(table, chunks[1]);

    // Footer with help
    let help_text = "[s]pawn [k]ill [a]ll [↑↓]nav [q]uit";
    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Rgb(180, 180, 180)))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(138, 43, 226))),
        );
    f.render_widget(footer, chunks[2]);
}
