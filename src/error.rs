use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum MartyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP server error: {0}")]
    Http(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, MartyError>;
