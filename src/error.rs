use reqwest::{Error as ReqwestError, StatusCode};
use serde_json::Error as SerdeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeliusError {
    #[error("Bad request to {path}: {text}")]
    BadRequest { path: String, text: String },

    #[error("Internal server error: {code} - {text}")]
    InternalError { code: StatusCode, text: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Too many requests made to {path}")]
    RateLimitExceeded { path: String },

    #[error("Unauthorized access to {path}: {text}")]
    Unauthorized { path: String, text: String },

    #[error("Unknown error has occurred: HTTP {code} - {text}")]
    Unknown { code: StatusCode, text: String },
}

// Handy type alias
pub type Result<T> = std::result::Result<T, HeliusError>;
