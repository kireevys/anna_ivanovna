use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ApiError {
    Network(String),
    Http(u16, String),
    Parse(String),
    InvalidUrl(String),
    Serialization(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {msg}"),
            ApiError::Http(code, msg) => write!(f, "HTTP {code}: {msg}"),
            ApiError::Parse(msg) => write!(f, "Parse error: {msg}"),
            ApiError::InvalidUrl(msg) => write!(f, "Invalid URL: {msg}"),
            ApiError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl From<ApiError> for String {
    fn from(err: ApiError) -> Self {
        err.to_string()
    }
}
