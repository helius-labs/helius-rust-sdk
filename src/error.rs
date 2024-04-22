use reqwest::{Error as ReqwestError, StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeliusError {
    #[error("Bad request to {path}: {text}")]
    BadRequest { path: String, text: String },

    #[error("Internal server error: {code} - {text}")]
    InternalError { code: StatusCode, text: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    Network(ReqwestError),

    #[error("Not found: {text}")]
    NotFound { text: String },

    #[error("Too many requests made to {path}")]
    RateLimitExceeded { path: String },

    #[error("Serialization / Deserialization error: {0}")]
    SerdeJson(ReqwestError),

    #[error("Unauthorized access to {path}: {text}")]
    Unauthorized { path: String, text: String },

    #[error("Unknown error has occurred: HTTP {code} - {text}")]
    Unknown { code: StatusCode, text: String },
}

impl HeliusError {
    pub fn from_response_status(status: StatusCode, path: String, text: String) -> Self {
        match status {
            StatusCode::BAD_REQUEST => HeliusError::BadRequest { path, text },
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => HeliusError::Unauthorized { path, text },
            StatusCode::NOT_FOUND => HeliusError::NotFound { text },
            StatusCode::INTERNAL_SERVER_ERROR => HeliusError::InternalError { code: status, text },
            StatusCode::TOO_MANY_REQUESTS => HeliusError::RateLimitExceeded { path },
            _ => HeliusError::Unknown { code: status, text },
        }
    }
}

// Handy type alias
pub type Result<T> = std::result::Result<T, HeliusError>;
