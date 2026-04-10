use thiserror::Error;

#[derive(Error, Debug)]
pub enum OctopodError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Department not found: {0}")]
    DepartmentNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, OctopodError>;
