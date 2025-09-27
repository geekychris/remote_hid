use thiserror::Error;

/// Common errors that can occur in the remote HID system
#[derive(Error, Debug)]
pub enum RemoteHidError {
    #[error("Authentication error: {0}")]
    Authentication(#[from] crate::auth::AuthError),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Session error: {0}")]
    Session(String),
    
    #[error("HID operation error: {0}")]
    HidOperation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, RemoteHidError>;