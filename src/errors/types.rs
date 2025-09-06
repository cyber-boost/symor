use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymorError {
    pub code: ErrorCode,
    pub message: String,
    pub context: HashMap<String, String>,
    pub timestamp: SystemTime,
    pub recovery_suggestion: Option<String>,
}
impl SymorError {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            context: HashMap::new(),
            timestamp: SystemTime::now(),
            recovery_suggestion: None,
        }
    }
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.recovery_suggestion = Some(suggestion);
        self
    }
}
impl std::fmt::Display for SymorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}
impl std::error::Error for SymorError {}
/// Error codes for different types of errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    FileNotFound,
    PermissionDenied,
    DiskFull,
    InvalidPath,
    VersionNotFound,
    VersionCorrupted,
    StorageFull,
    InvalidConfiguration,
    MissingConfiguration,
    NetworkError,
    ConnectionTimeout,
    InternalError,
    UnknownError,
}
/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub target: Option<String>,
    pub additional_info: HashMap<String, String>,
}
impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            target: None,
            additional_info: HashMap::new(),
        }
    }
    pub fn with_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info.insert(key.to_string(), value.to_string());
        self
    }
}