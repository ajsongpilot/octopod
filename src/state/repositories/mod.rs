use anyhow::Result;

pub mod activity_repo;
pub mod agent_repo;
pub mod agent_session_repo;
pub mod company_repo;
pub mod conversation_repo;
pub mod decision_repo;
pub mod department_repo;
pub mod initiative_repo;
pub mod meeting_participant_repo;
pub mod meeting_repo;
pub mod message_repo;
pub mod roadmap_repo;
pub mod task_repo;

pub use activity_repo::ActivityRepository;
pub use agent_repo::AgentRepository;
pub use agent_session_repo::AgentSessionRepository;
pub use company_repo::CompanyRepository;
pub use conversation_repo::ConversationRepository;
pub use decision_repo::{DecisionFilters, DecisionRepository};
pub use department_repo::DepartmentRepository;
pub use initiative_repo::InitiativeRepository;
pub use meeting_participant_repo::MeetingParticipantRepository;
pub use meeting_repo::MeetingRepository;
pub use message_repo::MessageRepository;
pub use roadmap_repo::RoadmapRepository;
pub use task_repo::TaskRepository;

/// Common trait for all repositories
#[async_trait::async_trait]
pub trait Repository<T> {
    async fn create(&self, entity: &T) -> Result<T>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn update(&self, entity: &T) -> Result<T>;
    async fn delete(&self, id: &str) -> Result<bool>;
}

/// Pagination parameters
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 50,
        }
    }
}

impl Pagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
        }
    }

    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.per_page) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }
}

/// Paginated result
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: &Pagination) -> Self {
        let total_pages = ((total as f64) / (pagination.per_page as f64)).ceil() as u32;
        Self {
            items,
            total,
            page: pagination.page,
            per_page: pagination.per_page,
            total_pages: total_pages.max(1),
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}
