use crate::store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git is not installed or not on PATH")]
    GitNotInstalled,
    #[error("git {operation} failed: {message}")]
    GitFailed { operation: String, message: String },
    #[error("invalid GitHub repository: {0}")]
    InvalidRepository(String),
    #[error("could not read a token from the gh CLI: {0}")]
    GhCliUnavailable(String),
}
