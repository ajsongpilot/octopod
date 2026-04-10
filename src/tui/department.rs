use crate::state::{MessageType, Priority, StateManager, TaskStatus};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info};

const STATUS_COLORS: [(TaskStatus, Color); 5] = [
    (TaskStatus::Todo, Color::Gray),
    (TaskStatus::InProgress, Color::Blue),
    (TaskStatus::Blocked, Color::Red),
    (TaskStatus::Review, Color::Yellow),
    (TaskStatus::Done, Color::Green),
];

const STATUS_NAMES: [(TaskStatus, &str); 5] = [
    (TaskStatus::Todo, "TODO"),
    (TaskStatus::InProgress, "IN PROG"),
    (TaskStatus::Blocked, "BLOCKED"),
    (TaskStatus::Review, "REVIEW"),
    (TaskStatus::Done, "DONE"),
];

/// Field being edited in task detail
#[derive(Clone, PartialEq)]
pub enum TaskEditField {
    Title,
    Description,
    AcceptanceCriteria,
    Status,
    Priority,
}

/// List filter state
pub struct ListFilter {
    pub status: Option<TaskStatus>,
    pub search: String,
    pub search_mode: bool,
}

/// OpenCode agent session info for display
#[derive(Clone)]
pub struct AgentSessionInfo {
    pub session_id: String,
    pub title: String,
    pub task_id: Option<String>,
    pub pid: Option<u32>,
    pub is_running: bool,
    pub updated: i64,
    pub directory: String,
}

impl AgentSessionInfo {
    fn parse_from_opencode(title: &str, session_id: &str, updated: i64, pid: Option<u32>, is_running: bool, directory: &str) -> Option<Self> {
        if !title.starts_with("octopod:") {
            return None;
        }
        
        let parts: Vec<&str> = title.split(':').collect();
        if parts.len() < 3 {
            return None;
        }
        
        let task_id = parts.get(1).map(|s| s.to_string());
        let task_title = parts[2..].join(":");
        
        Some(Self {
            session_id: session_id.to_string(),
            title: task_title,
            task_id,
            pid,
            is_running,
            updated,
            directory: directory.to_string(),
        })
    }
}

/// Department TUI App
pub struct DepartmentApp {
    pub should_quit: bool,
    pub department_id: String,
    pub department_name: String,
    pub active_view: usize,
    pub selected_task_index: usize,
    pub selected_column: usize, // 0=TODO, 1=IN PROG, 2=BLOCKED, 3=REVIEW, 4=DONE
    pub selected_agent_index: usize,
    pub agent_scroll_offset: usize,
    pub input: String,
    pub creating_task: bool,
    pub new_task_title: String,
    pub viewing_task: Option<String>,
    pub selected_field: Option<TaskEditField>,
    pub editing_field: Option<TaskEditField>,
    pub edit_buffer: String,
    pub list_filter: ListFilter,
    pub messages: Vec<crate::state::Message>,
    pub tasks: Vec<crate::state::Task>,
    pub decisions: Vec<crate::state::Decision>,
    pub state: StateManager,
    pub message_receiver: Option<broadcast::Receiver<crate::state::Message>>,
    pub error_message: Option<String>,
    pub show_help: bool,
    pub show_logs: bool,
    pub show_logs_for_session: Option<String>,
    pub cached_sessions: Vec<AgentSessionInfo>,
    pub sessions_cache_time: std::time::Instant,
}

#[allow(dead_code)]
impl DepartmentApp {
    pub async fn new(
        department_id: String,
        department_name: String,
        state: StateManager,
    ) -> Result<Self> {
        let conversation_id = format!("dept-{}", department_id);
        let message_receiver = Some(state.subscribe_to_conversation(&conversation_id).await);

        let messages = state
            .get_conversation_messages(&conversation_id, crate::state::Pagination::new(1, 50))
            .await
            .map(|r| r.items)
            .unwrap_or_default();

        let tasks = state
            .get_department_tasks(&department_id)
            .await
            .unwrap_or_default();

        let decisions = state
            .get_department_decisions(&department_id, None)
            .await
            .unwrap_or_default();

        info!(
            "Department TUI initialized for {} with {} messages, {} tasks, {} decisions",
            department_name,
            messages.len(),
            tasks.len(),
            decisions.len()
        );

        Ok(Self {
            should_quit: false,
            department_id,
            department_name,
            active_view: 0,
            selected_task_index: 0,
            selected_column: 0,
            selected_agent_index: 0,
            input: String::new(),
            creating_task: false,
            new_task_title: String::new(),
            viewing_task: None,
            selected_field: None,
            editing_field: None,
            edit_buffer: String::new(),
            list_filter: ListFilter {
                status: None,
                search: String::new(),
                search_mode: false,
            },
            messages,
            tasks,
            decisions,
            state,
            message_receiver,
            error_message: None,
            show_help: false,
            show_logs: false,
            show_logs_for_session: None,
            agent_scroll_offset: 0,
            cached_sessions: Vec::new(),
            sessions_cache_time: std::time::Instant::now(),
        })
    }

    fn get_sessions_cached(&mut self) -> &Vec<AgentSessionInfo> {
        // Only refresh if cache is older than 2 seconds
        if self.sessions_cache_time.elapsed() > Duration::from_secs(2) {
            self.cached_sessions = self.fetch_sessions();
            self.sessions_cache_time = std::time::Instant::now();
        }
        &self.cached_sessions
    }

    fn fetch_sessions(&self) -> Vec<AgentSessionInfo> {
        use std::process::Command;
        
        Command::new("opencode")
            .args(["session", "list", "--format", "json"])
            .output()
            .map(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                serde_json::from_str::<Vec<serde_json::Value>>(&output_str)
                    .ok()
                    .map(|sessions| {
                        sessions
                            .into_iter()
                            .filter_map(|s| {
                                let title = s.get("title")?.as_str()?.to_string();
                                let session_id = s.get("id")?.as_str()?.to_string();
                                let updated = s.get("updated")?.as_i64().unwrap_or(0);
                                let directory = s.get("directory")?.as_str()?.to_string();
                                AgentSessionInfo::parse_from_opencode(&title, &session_id, updated, None, true, &directory)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        let mut last_tick = std::time::Instant::now();
        let tick_rate = Duration::from_millis(100);

        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Some(ref mut receiver) = self.message_receiver {
                while let Ok(message) = receiver.try_recv() {
                    if !self.messages.iter().any(|m| m.id == message.id) {
                        self.messages.push(message);
                    }
                }
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code).await;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    async fn handle_key(&mut self, key: KeyCode) {
        // Handle viewing task detail
        if self.viewing_task.is_some() {
            match key {
                KeyCode::Esc => {
                    self.viewing_task = None;
                    self.error_message = None;
                }
                KeyCode::Char('e') => {
                    self.open_task_in_editor().await;
                }
                KeyCode::Char('s') => {
                    self.cycle_task_status().await;
                }
                KeyCode::Char('p') => {
                    self.cycle_task_priority().await;
                }
                KeyCode::Char('d') => {
                    self.request_decision().await;
                }
                _ => {}
            }
            return;
        }

        // Handle creating new task
        if self.creating_task {
            match key {
                KeyCode::Esc => {
                    self.creating_task = false;
                    self.new_task_title.clear();
                }
                KeyCode::Char(c) => {
                    self.new_task_title.push(c);
                }
                KeyCode::Backspace => {
                    self.new_task_title.pop();
                }
                KeyCode::Enter if !self.new_task_title.is_empty() => {
                    self.create_task().await;
                }
                _ => {}
            }
            return;
        }

        // Normal mode
        match key {
            KeyCode::Esc if self.show_help => {
                self.show_help = false;
            }
            KeyCode::Esc if self.active_view == 1 && self.list_filter.search_mode => {
                self.list_filter.search_mode = false;
                self.list_filter.search.clear();
            }
            KeyCode::Esc if self.show_logs => {
                self.show_logs = false;
                self.show_logs_for_session = None;
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('q') => {
                // Kill the current tmux window and exit
                let _ = std::process::Command::new("tmux")
                    .args(["kill-window"])
                    .spawn();
                std::process::exit(0);
            }
            KeyCode::Tab => {
                let was_agents = self.active_view == 4;
                self.active_view = (self.active_view + 1) % 5;
                // Refresh cache when switching to agents tab
                if self.active_view == 4 && !was_agents {
                    self.cached_sessions = self.fetch_sessions();
                    self.sessions_cache_time = std::time::Instant::now();
                }
            }
            // Arrow keys only for navigation in board view - letters go to chat input
            KeyCode::Left => {
                if self.active_view == 0 {
                    self.selected_column = self.selected_column.saturating_sub(1);
                    self.selected_task_index = 0;
                }
            }
            KeyCode::Right => {
                if self.active_view == 0 {
                    self.selected_column = (self.selected_column + 1).min(4);
                    self.selected_task_index = 0;
                }
            }
            KeyCode::Up => {
                // Exit search mode when navigating
                if self.list_filter.search_mode {
                    self.list_filter.search_mode = false;
                    self.list_filter.search.clear();
                }
                if self.active_view == 0 {
                    if self.selected_task_index > 0 {
                        self.selected_task_index -= 1;
                    }
                } else if self.active_view == 1
                    && self.selected_task_index > 0 {
                        self.selected_task_index -= 1;
                    } else if self.active_view == 4
                    && self.selected_agent_index > 0 {
                        self.selected_agent_index -= 1;
                        if self.selected_agent_index < self.agent_scroll_offset {
                            self.agent_scroll_offset = self.selected_agent_index;
                        }
                }
            }
            KeyCode::Down => {
                // Exit search mode when navigating
                if self.list_filter.search_mode {
                    self.list_filter.search_mode = false;
                    self.list_filter.search.clear();
                }
                if self.active_view == 0 {
                    let col_tasks = self.get_tasks_in_column(self.selected_column);
                    if self.selected_task_index < col_tasks.len().saturating_sub(1) {
                        self.selected_task_index += 1;
                    }
                } else if self.active_view == 1
                    && self.selected_task_index < self.tasks.len().saturating_sub(1) {
                        self.selected_task_index += 1;
                    } else if self.active_view == 4 {
                    // Navigation in agents view - will be validated against session count in render
                    self.selected_agent_index += 1;
                }
            }
            KeyCode::Char('x') if self.active_view == 0 => {
                self.delete_selected_task().await;
            }
            KeyCode::Char('x') if self.active_view == 4 => {
                self.kill_selected_agent().await;
                self.cached_sessions = self.fetch_sessions();
                self.sessions_cache_time = std::time::Instant::now();
                // Clamp selected index to valid range
                if self.selected_agent_index >= self.cached_sessions.len() {
                    self.selected_agent_index = self.cached_sessions.len().saturating_sub(1);
                }
            }
            KeyCode::Char('v') if self.active_view == 4 => {
                self.show_logs_for_selected();
            }
            KeyCode::Enter if self.active_view == 4 && !self.cached_sessions.is_empty() => {
                self.resume_selected_session();
            }
            KeyCode::Char('s') if self.active_view == 4 => {
                // Spawn a new agent manually
                self.spawn_agent_manually().await;
            }
            KeyCode::Char('r') if self.active_view == 4 => {
                // Refresh session cache
                self.cached_sessions = self.fetch_sessions();
                self.sessions_cache_time = std::time::Instant::now();
                // Clamp selected index
                if self.selected_agent_index >= self.cached_sessions.len() {
                    self.selected_agent_index = self.cached_sessions.len().saturating_sub(1);
                }
            }
            KeyCode::Enter
                if (self.active_view == 0 || self.active_view == 1) && !self.tasks.is_empty() =>
            {
                self.view_task_detail();
            }
            KeyCode::Char('n') if self.active_view == 0 => {
                self.creating_task = true;
                self.new_task_title.clear();
            }
            KeyCode::Char('/') if self.active_view == 1 && !self.list_filter.search_mode => {
                // Start search in list view
                self.list_filter.search_mode = true;
                self.list_filter.search = String::new();
            }
            KeyCode::Char('f') if self.active_view == 1 => {
                // Cycle through filters
                self.cycle_filter();
            }
            KeyCode::Char(c) if self.active_view == 1 && self.list_filter.search_mode => {
                self.list_filter.search.push(c);
            }
            KeyCode::Backspace if self.active_view == 1 && self.list_filter.search_mode => {
                if self.list_filter.search.is_empty() {
                    self.list_filter.search_mode = false;
                } else {
                    self.list_filter.search.pop();
                }
            }
            KeyCode::Char(c) if self.active_view == 3 => {
                self.input.push(c);
            }
            KeyCode::Backspace if self.active_view == 3 => {
                self.input.pop();
            }
            KeyCode::Enter if self.active_view == 3 && !self.input.is_empty() => {
                self.send_message().await;
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }
            _ => {}
        }
    }

    fn get_task_fields() -> Vec<&'static str> {
        vec![
            "Title",
            "Description",
            "Acceptance Criteria",
            "Status",
            "Priority",
            "Request Decision",
        ]
    }

    fn cycle_selected_field(&mut self) {
        let fields = Self::get_task_fields();
        if let Some(ref current) = self.selected_field {
            let idx = fields
                .iter()
                .position(|f| f == &Self::field_to_string(current))
                .unwrap_or(0);
            let next_idx = (idx + 1) % fields.len();
            if next_idx == fields.len() - 1 {
                self.selected_field = Some(TaskEditField::Status); // Special handling for last field
            } else {
                self.selected_field = Some(Self::string_to_field(fields[next_idx]));
            }
        } else {
            self.selected_field = Some(TaskEditField::Title);
        }
    }

    fn cycle_selected_field_backward(&mut self) {
        let fields = Self::get_task_fields();
        if self.selected_field.is_none() {
            self.selected_field = Some(TaskEditField::Priority);
        } else {
            let idx = fields
                .iter()
                .position(|f| f == &Self::field_to_string(self.selected_field.as_ref().unwrap()))
                .unwrap_or(0);
            if idx > 0 {
                self.selected_field = Some(Self::string_to_field(fields[idx - 1]));
            }
        }
    }

    fn cycle_filter(&mut self) {
        let statuses = [
            None,
            Some(TaskStatus::Todo),
            Some(TaskStatus::InProgress),
            Some(TaskStatus::Review),
            Some(TaskStatus::Done),
            Some(TaskStatus::Blocked),
        ];

        let current = &self.list_filter.status;
        let current_idx = statuses.iter().position(|s| s == current).unwrap_or(0);
        let next_idx = (current_idx + 1) % statuses.len();
        self.list_filter.status = statuses[next_idx];
    }

    async fn cycle_task_status(&mut self) {
        if let Some(ref task_id) = self.viewing_task {
            if let Some(task) = self.tasks.iter_mut().find(|t| &t.id == task_id) {
                task.status = match task.status {
                    TaskStatus::Todo => TaskStatus::InProgress,
                    TaskStatus::InProgress => TaskStatus::Review,
                    TaskStatus::Review => TaskStatus::Done,
                    TaskStatus::Done => TaskStatus::Blocked,
                    TaskStatus::Blocked => TaskStatus::Todo,
                    TaskStatus::Cancelled => TaskStatus::Todo,
                };
                if let Err(e) = self.state.update_task(task).await {
                    self.error_message = Some(format!("Failed to update status: {}", e));
                }
            }
        }
    }

    fn get_tasks_in_column(&self, column: usize) -> Vec<&crate::state::Task> {
        let status = match column {
            0 => TaskStatus::Todo,
            1 => TaskStatus::InProgress,
            2 => TaskStatus::Blocked,
            3 => TaskStatus::Review,
            4 => TaskStatus::Done,
            _ => TaskStatus::Todo,
        };
        self.tasks.iter().filter(|t| t.status == status).collect()
    }

    async fn delete_selected_task(&mut self) {
        let col_tasks = self.get_tasks_in_column(self.selected_column);
        if let Some(task) = col_tasks.get(self.selected_task_index) {
            let task_id = task.id.clone();
            match self.state.delete_task(&task_id).await {
                Ok(_) => {
                    self.tasks.retain(|t| t.id != task_id);
                    if self.selected_task_index > 0 {
                        self.selected_task_index -= 1;
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to delete: {}", e));
                }
            }
        }
    }

    async fn kill_selected_agent(&mut self) {
        use std::process::Command;
        
        let sessions = self.fetch_sessions();
        
        if let Some(session) = sessions.get(self.selected_agent_index) {
            if session.session_id.is_empty() {
                self.error_message = Some("Cannot kill - no session ID".to_string());
                return;
            }
            
            // Use opencode CLI to delete the session
            let result = Command::new("opencode")
                .args(["session", "delete", &session.session_id])
                .output();
            
            match result {
                Ok(output) if output.status.success() => {
                    info!("Deleted session {}", session.session_id);
                }
                Ok(output) => {
                    let _stderr = String::from_utf8_lossy(&output.stderr);
                    // If session delete fails, try to kill the process anyway
                    let title_pattern = format!("octopod:{}:", session.task_id.as_deref().unwrap_or(""));
                    let _ = Command::new("ps")
                        .args(["aux"])
                        .output()
                        .map(|output| {
                            String::from_utf8_lossy(&output.stdout).lines()
                                .filter(|line| line.contains("opencode"))
                                .filter(|line| !line.contains("grep"))
                                .filter(|line| line.contains(&title_pattern))
                                .filter_map(|line| line.split_whitespace().nth(1))
                                .filter_map(|s| s.parse::<u32>().ok())
                                .for_each(|pid| {
                                    let _ = Command::new("kill").arg(pid.to_string()).output();
                                })
                        });
                    info!("Deleted session and killed process for {}", session.session_id);
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to delete session: {}", e));
                }
            }
        }
    }

    fn show_logs_for_selected(&mut self) {
        use std::process::Command;
        
        let sessions: Vec<AgentSessionInfo> = Command::new("opencode")
            .args(["session", "list", "--format", "json"])
            .output()
            .map(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                serde_json::from_str::<Vec<serde_json::Value>>(&output_str)
                    .ok()
                    .map(|sessions| {
                        sessions
                            .into_iter()
                            .filter_map(|s| {
                                let title = s.get("title")?.as_str()?.to_string();
                                let session_id = s.get("id")?.as_str()?.to_string();
                                let updated = s.get("updated")?.as_i64().unwrap_or(0);
                                let directory = s.get("directory")?.as_str()?.to_string();
                                AgentSessionInfo::parse_from_opencode(&title, &session_id, updated, None, true, &directory)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        if let Some(session) = sessions.get(self.selected_agent_index) {
            if !session.session_id.is_empty() {
                self.show_logs = true;
                self.show_logs_for_session = Some(session.session_id.clone());
            }
        }
    }

    async fn spawn_agent_manually(&mut self) {
        use std::process::Command;
        
        let task_title = format!("Manual agent in {}", self.department_name);
        let task_id = format!("manual_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("xxx"));
        let title = format!("octopod:task_{}:{}", task_id, task_title);
        
        let session_name = format!("{}-agent", self.department_name.to_lowercase());
        let cmd = format!("opencode run --title '{}' 'Work on: {}'", title, task_title);
        
        let project_dir = std::env::current_dir().unwrap_or_default().display().to_string();
        
        Command::new("tmux")
            .args(["new-session", "-d", "-s", &session_name, "-c", &project_dir, "--", "bash", "-c", &cmd])
            .spawn()
            .ok();
        
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        self.cached_sessions = self.fetch_sessions();
        self.sessions_cache_time = std::time::Instant::now();
        
        self.error_message = Some(format!(
            "Agent '{}' running in tmux session '{}'.\nSwitch to it with: tmux switch-client -t {}",
            task_title, session_name, session_name
        ));
    }

    fn resume_selected_session(&mut self) {
        if let Some(session) = self.cached_sessions.get(self.selected_agent_index) {
            let session_age = chrono::DateTime::from_timestamp(session.updated / 1000, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            self.error_message = Some(format!(
                "Session: '{}'\nDirectory: {}\nLast active: {}\n\nTo resume: cd to that directory and run opencode",
                session.title,
                session.directory,
                session_age
            ));
        }
    }

    async fn cycle_task_priority(&mut self) {
        if let Some(ref task_id) = self.viewing_task {
            if let Some(task) = self.tasks.iter_mut().find(|t| &t.id == task_id) {
                task.priority = match task.priority {
                    Priority::P0 => Priority::P1,
                    Priority::P1 => Priority::P2,
                    Priority::P2 => Priority::P3,
                    Priority::P3 => Priority::P0,
                };
                if let Err(e) = self.state.update_task(task).await {
                    self.error_message = Some(format!("Failed to update priority: {}", e));
                }
            }
        }
    }

    fn field_to_string(field: &TaskEditField) -> &'static str {
        match field {
            TaskEditField::Title => "Title",
            TaskEditField::Description => "Description",
            TaskEditField::AcceptanceCriteria => "Acceptance Criteria",
            TaskEditField::Status => "Status",
            TaskEditField::Priority => "Priority",
        }
    }

    fn string_to_field(s: &str) -> TaskEditField {
        match s {
            "Title" => TaskEditField::Title,
            "Description" => TaskEditField::Description,
            "Acceptance Criteria" => TaskEditField::AcceptanceCriteria,
            "Status" => TaskEditField::Status,
            "Priority" => TaskEditField::Priority,
            _ => TaskEditField::Title,
        }
    }

    fn view_task_detail(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_task_index) {
            self.viewing_task = Some(task.id.clone());
            self.editing_field = None;
            self.edit_buffer.clear();
        }
    }

    fn start_edit_current_field(&mut self) {
        if let Some(ref task_id) = self.viewing_task {
            if let Some(task) = self.tasks.iter().find(|t| &t.id == task_id) {
                let field = self.selected_field.clone().unwrap_or(TaskEditField::Title);
                self.editing_field = Some(field.clone());
                self.edit_buffer = match field {
                    TaskEditField::Title => task.title.clone(),
                    TaskEditField::Description => task.description.clone().unwrap_or_default(),
                    TaskEditField::AcceptanceCriteria => {
                        task.acceptance_criteria.clone().unwrap_or_default()
                    }
                    TaskEditField::Status => format!("{:?}", task.status),
                    TaskEditField::Priority => task.priority.as_str().to_string(),
                };
            }
        }
    }

    async fn save_task_edit(&mut self) {
        if let (Some(ref task_id), Some(ref field)) = (&self.viewing_task, &self.editing_field) {
            if let Some(task) = self.tasks.iter_mut().find(|t| &t.id == task_id) {
                match field {
                    TaskEditField::Title => {
                        task.title = self.edit_buffer.clone();
                    }
                    TaskEditField::Description => {
                        task.description = Some(self.edit_buffer.clone());
                    }
                    TaskEditField::AcceptanceCriteria => {
                        task.acceptance_criteria = Some(self.edit_buffer.clone());
                    }
                    TaskEditField::Status => {
                        // Cycle through status
                        task.status = match task.status {
                            TaskStatus::Todo => TaskStatus::InProgress,
                            TaskStatus::InProgress => TaskStatus::Review,
                            TaskStatus::Review => TaskStatus::Done,
                            TaskStatus::Done => TaskStatus::Blocked,
                            TaskStatus::Blocked => TaskStatus::Todo,
                            TaskStatus::Cancelled => TaskStatus::Todo,
                        };
                    }
                    TaskEditField::Priority => {
                        // Cycle through priority
                        task.priority = match task.priority {
                            Priority::P0 => Priority::P1,
                            Priority::P1 => Priority::P2,
                            Priority::P2 => Priority::P3,
                            Priority::P3 => Priority::P0,
                        };
                    }
                }

                if let Err(e) = self.state.update_task(task).await {
                    self.error_message = Some(format!("Failed to update: {}", e));
                }
            }
        }
        self.editing_field = None;
        self.edit_buffer.clear();
    }

    async fn request_decision(&mut self) {
        if let Some(ref task_id) = self.viewing_task {
            if let Some(task) = self.tasks.iter().find(|t| &t.id == task_id) {
                let decision = match self
                    .state
                    .create_decision(format!("Decision for: {}", task.title))
                    .await
                {
                    Ok(d) => d,
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create decision: {}", e));
                        return;
                    }
                };

                // Link decision to task
                if let Some(task) = self.tasks.iter_mut().find(|t| &t.id == task_id) {
                    task.related_decision_id = Some(decision.id.clone());
                    let _ = self.state.update_task(task).await;
                }
            }
        }
        self.viewing_task = None;
        self.editing_field = None;
    }

    async fn open_task_in_editor(&mut self) {
        if let Some(ref task_id) = self.viewing_task {
            if let Some(task) = self.tasks.iter().find(|t| &t.id == task_id) {
                match &task.file_path {
                    Some(path) => match self.state.open_task_editor(task).await {
                        Ok(_) => {
                            self.error_message = Some(format!("Opened {} in $EDITOR", path));
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed: {}", e));
                        }
                    },
                    None => {
                        self.error_message =
                            Some("No file. Task file_path is None - recreate the task".to_string());
                    }
                }
            }
        }
    }

    async fn send_message(&mut self) {
        if self.input.is_empty() {
            return;
        }

        let content = self.input.clone();
        let conversation_id = format!("dept-{}", self.department_id);

        match self
            .state
            .send_message(&conversation_id, None, None, &content, MessageType::Chat)
            .await
        {
            Ok(message) => {
                self.messages.push(message);
                self.input.clear();
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to send: {}", e));
            }
        }
    }

    async fn create_task(&mut self) {
        let title = self.new_task_title.clone();
        let dept = match self.state.get_department_by_slug(&self.department_id).await {
            Ok(Some(d)) => d,
            Ok(None) => {
                self.error_message = Some("Department not found".to_string());
                return;
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
                return;
            }
        };

        match self.state.create_task(&dept.id, &title).await {
            Ok(task) => {
                self.tasks.push(task);
                self.creating_task = false;
                self.new_task_title.clear();
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to create task: {}", e));
            }
        }
    }

    fn ui(&self, frame: &mut Frame) {
        let size = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
            ])
            .split(size);

        self.render_header(frame, chunks[0]);

        // Show task detail overlay if viewing a task
        if let Some(ref task_id) = self.viewing_task {
            if let Some(task) = self.tasks.iter().find(|t| &t.id == task_id) {
                self.render_task_detail(frame, chunks[1], chunks[2], task);
            }
        } else {
            self.render_tabs(frame, chunks[1]);
            match self.active_view {
                0 => self.render_board_tab(frame, chunks[2]),
                1 => self.render_list_tab(frame, chunks[2]),
                2 => self.render_activity_tab(frame, chunks[2]),
                3 => self.render_chat_tab(frame, chunks[2]),
                4 => self.render_agents_tab(frame, chunks[2]),
                _ => {}
            }
        }
        self.render_status_bar(frame, chunks[3]);

        // Show help overlay
        if self.show_help {
            self.render_help_overlay(frame, size);
        }

        // Show logs overlay
        if self.show_logs {
            self.render_logs_overlay(frame, size);
        }
    }

    fn render_help_overlay(&self, frame: &mut Frame, size: Rect) {
        let help_text = "OCTOPOD SHORTCUTS

NAVIGATION
  Tab         Cycle views (Board/List/Activity/Chat/Agents)
  Left / Right Move left/right (Board view)
  Up / Down    Navigate up/down

TASKS
  Enter       View task detail
  n           Create new task
  x           Delete selected task

IN TASK DETAIL
  e           Open in editor
  s           Cycle task status
  p           Cycle task priority
  d           Request decision
  Esc         Close detail

LIST VIEW
  /           Start search
  f           Cycle filters

CHAT
  Type + Enter  Send message
  (All letter keys work in chat view)

AGENTS VIEW
  View daemon and opencode process status

GENERAL
  ?           Toggle this help
  q           Quit

Press ? or Esc to close";

        let width = 55.min(size.width.saturating_sub(4));
        let height = 30.min(size.height.saturating_sub(2));
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

        frame.render_widget(paragraph, area);
    }

    fn render_logs_overlay(&self, frame: &mut Frame, size: Rect) {
        use std::process::Command;
        
        let session_id = match &self.show_logs_for_session {
            Some(id) => id.clone(),
            None => return,
        };
        
        // Get session info
        let (title, created, updated) = Command::new("opencode")
            .args(["session", "list", "--format", "json"])
            .output()
            .map(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let sessions: Vec<serde_json::Value> = serde_json::from_str(&output_str).unwrap_or_default();
                let session = sessions.iter().find(|s| s.get("id").and_then(|v| v.as_str()) == Some(&session_id));
                let title = session.and_then(|s| s.get("title").and_then(|v| v.as_str())).unwrap_or("Unknown").to_string();
                let created = session.and_then(|s| s.get("created").and_then(|v| v.as_i64())).unwrap_or(0);
                let updated = session.and_then(|s| s.get("updated").and_then(|v| v.as_i64())).unwrap_or(0);
                (title, created, updated)
            })
            .unwrap_or_else(|_| ("Unknown".to_string(), 0, 0));

        let width = 70.min(size.width.saturating_sub(4));
        let height = 15;
        let left = (size.width - width) / 2;
        let top = (size.height - height) / 2;

        let area = Rect::new(left, top, width, height);

        let block = Block::default()
            .title(format!(" Session Info: {} ", truncate(&title, 40)))
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let created_str = if created > 0 {
            chrono::DateTime::from_timestamp(created / 1000, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };
        
        let updated_str = if updated > 0 {
            chrono::DateTime::from_timestamp(updated / 1000, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        let log_text = format!(
            "Session ID: {}\nTitle: {}\nCreated: {}\nLast Updated: {}\n\nTo view full session history:\n  opencode -s {}\n\nPress [Enter] to resume this session\nin a new window, or [Esc] to close",
            truncate(&session_id, 30),
            truncate(&title, 50),
            created_str,
            updated_str,
            session_id
        );

        let paragraph = Paragraph::new(log_text)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, inner);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let pending_decisions = self
            .decisions
            .iter()
            .filter(|d| d.status == crate::state::DecisionStatus::Pending)
            .count();

        let header_text = format!(
            "🐙 {} Department | {} tasks | {} pending decisions",
            self.department_name,
            self.tasks.len(),
            pending_decisions
        );

        let style = Style::default()
            .fg(Color::Rgb(255, 127, 80))
            .add_modifier(Modifier::BOLD);

        let paragraph = Paragraph::new(header_text)
            .style(style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(style));
        frame.render_widget(paragraph, area);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let views = ["Board", "List", "Activity", "Chat", "Agents"];
        let active = views[self.active_view];

        let text = if self.creating_task {
            "Enter task title...  |  [Enter] create  [Esc] cancel".to_string()
        } else if self.active_view == 0 {
            format!(
                "{}  |  [n] new  [←→/hl] columns  [↑↓] move  [x] delete  [q] quit",
                active
            )
        } else if self.active_view == 1 {
            let search_hint = if self.list_filter.search_mode {
                format!(" Searching: {}  [Esc] cancel", self.list_filter.search)
            } else {
                String::new()
            };
            format!(
                "{}  |  [f] filter  [/] search  [Enter] view  [↑↓] move  [q] quit{}",
                active, search_hint
            )
        } else {
            format!("{}  |  [tab] switch  [q] quit", active)
        };

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles = vec!["Board", "List", "Activity", "Chat", "Agents"];
        let tabs = Tabs::new(titles)
            .select(self.active_view)
            .style(Style::default().fg(Color::Rgb(180, 180, 180)))
            .highlight_style(
                Style::default()
                    .fg(Color::Rgb(255, 127, 80))
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | ");
        frame.render_widget(tabs, area);
    }

    fn render_board_tab(&self, frame: &mut Frame, area: Rect) {
        let columns: Vec<(TaskStatus, &str, Vec<&crate::state::Task>)> = STATUS_NAMES
            .iter()
            .map(|(status, name)| {
                let tasks: Vec<&crate::state::Task> =
                    self.tasks.iter().filter(|t| t.status == *status).collect();
                (*status, *name, tasks)
            })
            .collect();

        let total_tasks: usize = columns.iter().map(|(_, _, t)| t.len()).sum();

        let board_area = if self.creating_task {
            let constraints = [Constraint::Min(1), Constraint::Length(3)];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            if total_tasks == 0 {
                let text = Paragraph::new("No tasks yet. Press [n] to create one.")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                frame.render_widget(text, chunks[0]);
            } else {
                self.render_board_columns(frame, chunks[0], &columns);
            }

            let input_text = format!("New task: {}", self.new_task_title);
            let input = Paragraph::new(input_text)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Rgb(255, 127, 80)))
                        .title("Create Task (Enter to confirm)"),
                );
            frame.render_widget(input, chunks[1]);

            return;
        } else if total_tasks == 0 {
            let text = Paragraph::new("No tasks yet. Press [n] to create one.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        } else {
            area
        };

        self.render_board_columns(frame, board_area, &columns);
    }

    fn render_board_columns(
        &self,
        frame: &mut Frame,
        area: Rect,
        columns: &[(TaskStatus, &str, Vec<&crate::state::Task>)],
    ) {
        let constraints: Vec<Constraint> = STATUS_NAMES
            .iter()
            .map(|_| Constraint::Percentage(20))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        for (i, (status, name, tasks)) in columns.iter().enumerate() {
            self.render_board_column(frame, chunks[i], *status, name, tasks, i);
        }
    }

    fn render_board_column(
        &self,
        frame: &mut Frame,
        area: Rect,
        status: TaskStatus,
        name: &str,
        tasks: &[&crate::state::Task],
        column_index: usize,
    ) {
        let color = STATUS_COLORS
            .iter()
            .find(|(s, _)| *s == status)
            .map(|(_, c)| *c)
            .unwrap_or(Color::Gray);

        let border_style = Style::default().fg(color);

        let title = if tasks.is_empty() {
            name.to_string()
        } else {
            format!("{} ({})", name, tasks.len())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title)
            .title_style(border_style.add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if tasks.is_empty() {
            return;
        }

        let task_height = 4i64;
        let max_visible = (inner.height as i64 / task_height).max(1) as usize;
        let visible_tasks = tasks.iter().take(max_visible).enumerate();

        for (i, task) in visible_tasks {
            let is_selected = column_index == self.selected_column && i == self.selected_task_index;
            let task_area = Rect::new(
                inner.x,
                inner.y + (i as u16 * task_height as u16),
                inner.width,
                (task_height - 1) as u16,
            );
            self.render_task_card(frame, task_area, task, is_selected);
        }
    }

    fn render_task_card(
        &self,
        frame: &mut Frame,
        area: Rect,
        task: &crate::state::Task,
        selected: bool,
    ) {
        let priority_str = match task.priority {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
        };

        let priority_color = match task.priority {
            Priority::P0 => Color::Red,
            Priority::P1 => Color::Yellow,
            Priority::P2 => Color::Blue,
            Priority::P3 => Color::Gray,
        };

        let assignee = task.assigned_to.as_deref().unwrap_or("unassigned");

        let content = format!(
            "[{}] {}\n   #{} | {}",
            priority_str,
            truncate(&task.title, 30),
            &task.id[..8],
            assignee
        );

        let mut style = Style::default().fg(Color::White);
        if selected {
            style = Style::default().fg(Color::Black).bg(Color::White);
        }

        let paragraph = Paragraph::new(content)
            .style(style)
            .block(
                Block::default()
                    .border_style(Style::default().fg(priority_color))
                    .borders(Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
                    .border_type(ratatui::widgets::BorderType::Thick),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_list_tab(&self, frame: &mut Frame, area: Rect) {
        let filter_text = if self.list_filter.search_mode {
            format!("🔍 Search: {}", self.list_filter.search)
        } else if let Some(ref status) = self.list_filter.status {
            format!("Filter: {:?} | [f] change | [/] search", status)
        } else {
            "Filter: all | [f] filter | [/] search".to_string()
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        let filter_para = Paragraph::new(filter_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(filter_para, chunks[0]);

        let filtered_tasks: Vec<&crate::state::Task> = self
            .tasks
            .iter()
            .filter(|t| {
                if let Some(ref status) = self.list_filter.status {
                    if t.status != *status {
                        return false;
                    }
                }
                if !self.list_filter.search.is_empty()
                    && !t
                        .title
                        .to_lowercase()
                        .contains(&self.list_filter.search.to_lowercase())
                    {
                        return false;
                    }
                true
            })
            .collect();

        if filtered_tasks.is_empty() {
            let text = if self.tasks.is_empty() {
                "No tasks. Press [n] to create one.".to_string()
            } else {
                "No tasks match filter. Press [/] to clear search.".to_string()
            };
            let para = Paragraph::new(text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(para, chunks[1]);
            return;
        }

        let items: Vec<ListItem> = filtered_tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let priority_str = task.priority.as_str();
                let status_str = format!("{:?}", task.status);
                let content = format!(
                    "[{}] [{}] {:<40} {}",
                    priority_str,
                    status_str,
                    truncate(&task.title, 40),
                    if i == self.selected_task_index {
                        " <-- selected"
                    } else {
                        ""
                    }
                );
                ListItem::new(content).style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_style(Style::default().fg(Color::Rgb(255, 127, 80)));

        frame.render_widget(list, chunks[1]);
    }

    fn render_activity_tab(&self, frame: &mut Frame, area: Rect) {
        let mut items: Vec<ListItem> = Vec::new();

        for decision in self
            .decisions
            .iter()
            .filter(|d| d.status == crate::state::DecisionStatus::Pending)
        {
            let status_str = match decision.status {
                crate::state::DecisionStatus::Pending => "[!] DECISION NEEDED",
                crate::state::DecisionStatus::Approved => "[✓] APPROVED",
                crate::state::DecisionStatus::Rejected => "[✗] REJECTED",
                _ => "[?]",
            };
            let content = format!(
                "{} {} - {}",
                status_str,
                truncate(&decision.title, 40),
                decision.priority.as_str()
            );
            items.push(ListItem::new(content).style(Style::default().fg(Color::Yellow)));
        }

        for msg in self.messages.iter().rev().take(20) {
            let from = msg.from_agent_id.as_deref().unwrap_or("System");
            let time = msg.created_at.format("%H:%M").to_string();
            let content = format!("[{}] {}: {}", time, from, truncate(&msg.content, 50));
            items.push(ListItem::new(content).style(Style::default().fg(Color::White)));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Activity"),
            )
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(list, area);
    }

    fn render_task_detail(
        &self,
        frame: &mut Frame,
        _tabs_area: Rect,
        content_area: Rect,
        task: &crate::state::Task,
    ) {
        let selected_style = Style::default()
            .fg(Color::Rgb(255, 127, 80))
            .add_modifier(Modifier::BOLD);

        let normal_style = Style::default().fg(Color::White);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with title, status, priority
                Constraint::Min(10),   // File content
                Constraint::Length(1), // Nav footer
            ])
            .split(content_area);

        // Header: Title | Status | Priority
        let header = format!(
            "{}  |  Status: {:?}  |  Priority: {}",
            task.title,
            task.status,
            task.priority.as_str()
        );
        let header_para = Paragraph::new(header)
            .style(selected_style)
            .block(Block::default().borders(Borders::ALL).title("Task"));
        frame.render_widget(header_para, chunks[0]);

        // File content
        let file_content = self
            .state
            .get_task_content(task)
            .unwrap_or_else(|_| "No file content. Press [e] to create it.".to_string());
        let content_lines: Vec<ListItem> = file_content
            .lines()
            .take(50)
            .map(ListItem::new)
            .collect();

        let list = List::new(content_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("File (press e to edit)"),
            )
            .style(normal_style);

        frame.render_widget(list, chunks[1]);

        // Footer with file path and nav
        let file_path = task.file_path.as_deref().unwrap_or("(no file)");
        let footer = if let Some(ref err) = self.error_message {
            format!(
                "{}  |  [e] open in $EDITOR  [d] request decision  [Esc] back",
                err
            )
        } else {
            format!("File: {}  |  [e] open in $EDITOR  [s] cycle status  [p] cycle priority  [d] request decision  [Esc] back", truncate(file_path, 40))
        };

        let footer_para = Paragraph::new(footer)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(footer_para, chunks[2]);
    }

    fn render_chat_tab(&self, frame: &mut Frame, area: Rect) {
        let footer_height = if self.error_message.is_some() { 4 } else { 3 };
        let input_height = footer_height;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(input_height)])
            .split(area);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .rev()
            .map(|msg| {
                let from = msg.from_agent_id.as_deref().unwrap_or("System");
                let content = format!(
                    "[{}] {}: {}",
                    msg.created_at.format("%H:%M"),
                    from,
                    msg.content
                );
                ListItem::new(content).style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(messages)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Department Chat"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(list, chunks[0]);

        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if self.error_message.is_some() { 1 } else { 0 }),
                Constraint::Length(3),
            ])
            .split(chunks[1]);

        if let Some(ref error) = self.error_message {
            let error_widget =
                Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
            frame.render_widget(error_widget, input_chunks[0]);
        }

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Message (Enter to send)"),
            );
        frame.render_widget(input, input_chunks[1]);
    }

    fn render_agents_tab(&self, frame: &mut Frame, area: Rect) {
        use std::process::Command;
        
        let daemon_session = format!("octopod-{}-daemon", self.department_id);
        
        let is_daemon_running = Command::new("tmux")
            .args(["has-session", "-t", &daemon_session])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        // Use cached sessions (refreshed when switching to this tab)
        let sessions = &self.cached_sessions;

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Running Agents - {}", self.department_name));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if !is_daemon_running && sessions.is_empty() {
            let msg = Paragraph::new("No agents running.\n\nStart an agent with: octopod agent loop <dept>\nor use [s] in CEO Dashboard to spawn")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(msg, inner);
            return;
        }

        let mut items = vec![];
        
        // Daemon row (not selectable)
        if is_daemon_running {
            items.push(ListItem::new(format!("● Daemon: {} - Running", daemon_session))
                .style(Style::default().fg(Color::Green)));
        } else {
            items.push(ListItem::new(format!("○ Daemon: {} - Stopped", daemon_session))
                .style(Style::default().fg(Color::DarkGray)));
        }

        items.push(ListItem::new("").style(Style::default()));

        if !sessions.is_empty() {
            let visible_rows = ((inner.height as usize).saturating_sub(8)).max(1);
            let session_count = sessions.len();
            
            // Clamp selected index to valid range
            let selected = self.selected_agent_index.min(session_count.saturating_sub(1));
            
            // Calculate visible range
            let start_idx = selected.saturating_sub(visible_rows.saturating_sub(1));
            let end_idx = (start_idx + visible_rows).min(session_count);
            
            items.push(ListItem::new(format!("{} Octopod Agents:", session_count))
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            
            for (i, session) in sessions.iter().enumerate().take(end_idx).skip(start_idx) {
                let is_selected = i == selected;
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Rgb(255, 127, 80))
                } else {
                    Style::default().fg(Color::White)
                };
                
                let status = if session.is_running { "running" } else { "stopped" };
                let task_id = session.task_id.as_deref().unwrap_or("?");
                let time_ago = time_ago(session.updated);
                
                items.push(ListItem::new(format!("  {}{} [task:{}] {} {}", 
                    if is_selected { ">" } else { " " },
                    truncate(&session.title, 30),
                    truncate(task_id, 10),
                    status,
                    time_ago
                )).style(style));
            }
            
            if session_count > visible_rows {
                items.push(ListItem::new(format!("  Showing {}-{} of {}", start_idx + 1, end_idx, session_count))
                    .style(Style::default().fg(Color::DarkGray)));
            }
            
            items.push(ListItem::new("").style(Style::default()));
            items.push(ListItem::new("[s] spawn  [↑↓] navigate  [x] kill  [Enter] resume  [v] info")
                .style(Style::default().fg(Color::DarkGray)));
        } else {
            items.push(ListItem::new("No octopod agents active")
                .style(Style::default().fg(Color::DarkGray)));
            items.push(ListItem::new("").style(Style::default()));
            items.push(ListItem::new("[s] spawn agent manually")
                .style(Style::default().fg(Color::DarkGray)));
        }

        let list = List::new(items)
            .block(Block::default());

        frame.render_widget(list, inner);
    }
}

fn time_ago(timestamp: i64) -> String {
    if timestamp == 0 {
        return String::new();
    }
    let now = chrono::Utc::now().timestamp_millis();
    let diff = now - timestamp;
    
    if diff < 60000 {
        "<1m ago".to_string()
    } else if diff < 3600000 {
        format!("{}m ago", diff / 60000)
    } else if diff < 86400000 {
        format!("{}h ago", diff / 3600000)
    } else {
        format!("{}d ago", diff / 86400000)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Run department TUI
pub async fn run_department_tui(department_id: String, department_name: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_dir = find_project_dir(&current_dir).unwrap_or(current_dir);

    let state = StateManager::init_for_project(&project_dir).await?;

    let company_set = match state.list_departments().await {
        Ok(depts) if !depts.is_empty() => {
            state.set_company(depts[0].company_id.clone()).await;
            info!("Set company to: {}", depts[0].company_id);
            true
        }
        Ok(_) => {
            error!("No departments found in database at: {:?}", project_dir);
            false
        }
        Err(e) => {
            error!(
                "Failed to list departments: {} (project_dir: {:?})",
                e, project_dir
            );
            false
        }
    };

    if !company_set {
        anyhow::bail!("Failed to initialize company. Make sure you're running from a project with 'octopod init' completed.");
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DepartmentApp::new(department_id, department_name, state).await?;
    let result = app.run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        error!("Department TUI error: {:?}", e);
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
