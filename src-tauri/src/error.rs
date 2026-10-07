use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Disk info error: {0}")]
    DiskInfo(String),

    #[error("Safety violation: {0}")]
    SafetyViolation(String),

    #[error("Category error: {0}")]
    Category(String),

    #[error("Scan error: {0}")]
    Scan(String),

    #[error("Clean error: {0}")]
    Clean(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("History error: {0}")]
    History(String),

    #[error("{0}")]
    Custom(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
