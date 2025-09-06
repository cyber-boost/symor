use anyhow::Result;
use std::time::Duration;
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, delay: Duration },
    Fallback { alternative_action: String },
    Skip,
    Fail,
}
pub struct ErrorRecovery {
    strategies: std::collections::HashMap<String, RecoveryStrategy>,
}
impl ErrorRecovery {
    pub fn new() -> Self {
        let mut strategies = std::collections::HashMap::new();
        strategies
            .insert(
                "FileNotFound".to_string(),
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    delay: Duration::from_millis(100),
                },
            );
        strategies
            .insert(
                "PermissionDenied".to_string(),
                RecoveryStrategy::Fallback {
                    alternative_action: "Try with elevated permissions".to_string(),
                },
            );
        strategies
            .insert(
                "NetworkError".to_string(),
                RecoveryStrategy::Retry {
                    max_attempts: 5,
                    delay: Duration::from_secs(1),
                },
            );
        Self { strategies }
    }
    pub fn get_strategy(&self, error_code: &str) -> RecoveryStrategy {
        self.strategies.get(error_code).cloned().unwrap_or(RecoveryStrategy::Fail)
    }
    pub fn set_strategy(&mut self, error_code: String, strategy: RecoveryStrategy) {
        self.strategies.insert(error_code, strategy);
    }
    pub async fn execute_recovery<T, F>(
        &self,
        error_code: &str,
        operation: F,
    ) -> Result<T>
    where
        F: FnMut() -> Result<T> + Send + Sync,
        T: Send + Sync,
    {
        let strategy = self.get_strategy(error_code);
        match strategy {
            RecoveryStrategy::Retry { max_attempts, delay } => {
                self.execute_retry(operation, max_attempts, delay).await
            }
            RecoveryStrategy::Fallback { alternative_action } => {
                Err(anyhow::anyhow!("Fallback required: {}", alternative_action))
            }
            RecoveryStrategy::Skip => {
                Err(anyhow::anyhow!("Operation skipped due to error"))
            }
            RecoveryStrategy::Fail => {
                Err(anyhow::anyhow!("Operation failed without recovery option"))
            }
        }
    }
    async fn execute_retry<T, F>(
        &self,
        mut operation: F,
        max_attempts: u32,
        delay: Duration,
    ) -> Result<T>
    where
        F: FnMut() -> Result<T> + Send + Sync,
        T: Send + Sync,
    {
        let mut last_error = None;
        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_attempts {
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
}
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub success: bool,
    pub attempts: u32,
    pub final_error: Option<String>,
    pub recovery_strategy: String,
}
impl RecoveryResult {
    pub fn success(attempts: u32, strategy: &str) -> Self {
        Self {
            success: true,
            attempts,
            final_error: None,
            recovery_strategy: strategy.to_string(),
        }
    }
    pub fn failure(attempts: u32, error: &str, strategy: &str) -> Self {
        Self {
            success: false,
            attempts,
            final_error: Some(error.to_string()),
            recovery_strategy: strategy.to_string(),
        }
    }
}
pub struct AutoRecovery {
    error_recovery: ErrorRecovery,
    enabled: bool,
}
impl AutoRecovery {
    pub fn new() -> Self {
        Self {
            error_recovery: ErrorRecovery::new(),
            enabled: true,
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub async fn recover<T, F>(&self, error_code: &str, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync,
        T: Send + Sync,
    {
        if !self.enabled {
            return operation();
        }
        self.error_recovery.execute_recovery(error_code, operation).await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    #[tokio::test]
    async fn test_retry_recovery() {
        let recovery = ErrorRecovery::new();
        let attempt_count = AtomicU32::new(0);
        let result: Result<String, _> = recovery
            .execute_recovery(
                "FileNotFound",
                || {
                    let count = attempt_count.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(anyhow::anyhow!("File not found"))
                    } else {
                        Ok("success".to_string())
                    }
                },
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }
    #[tokio::test]
    async fn test_fallback_recovery() {
        let recovery = ErrorRecovery::new();
        let result: Result<String, _> = recovery
            .execute_recovery(
                "PermissionDenied",
                || Err(anyhow::anyhow!("Permission denied")),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Fallback required"));
    }
}