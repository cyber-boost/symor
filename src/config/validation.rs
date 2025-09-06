use anyhow::Result;
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}
pub struct ConfigValidator;
impl ConfigValidator {
    pub fn new() -> Self {
        Self
    }
    pub fn validate_config(&self, config: &crate::SymorConfig) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        self.validate_versioning_config(&config.versioning, &mut errors, &mut warnings);
        self.validate_linking_config(&config.linking, &mut errors, &mut warnings);
        self.validate_home_directory(&config.home_dir, &mut errors, &mut warnings);
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
    fn validate_versioning_config(
        &self,
        config: &crate::VersioningConfig,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if config.max_versions == 0 {
            errors
                .push(ValidationError {
                    field: "versioning.max_versions".to_string(),
                    message: "Maximum versions cannot be zero".to_string(),
                    suggestion: Some(
                        "Set max_versions to a value greater than 0".to_string(),
                    ),
                });
        }
        if config.max_versions > 1000 {
            warnings
                .push(ValidationWarning {
                    field: "versioning.max_versions".to_string(),
                    message: "Very high max_versions may impact performance".to_string(),
                    suggestion: Some(
                        "Consider reducing max_versions to 100-200".to_string(),
                    ),
                });
        }
        if config.compression > 9 {
            errors
                .push(ValidationError {
                    field: "versioning.compression".to_string(),
                    message: "Compression level must be between 0-9".to_string(),
                    suggestion: Some(
                        "Set compression to a value between 0-9".to_string(),
                    ),
                });
        }
    }
    fn validate_linking_config(
        &self,
        config: &crate::LinkingConfig,
        errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) {
        let valid_link_types = ["hard", "soft", "copy"];
        if !valid_link_types.contains(&config.link_type.as_str()) {
            errors
                .push(ValidationError {
                    field: "linking.link_type".to_string(),
                    message: format!("Invalid link type: {}", config.link_type),
                    suggestion: Some(format!("Use one of: {:?}", valid_link_types)),
                });
        }
    }
    fn validate_home_directory(
        &self,
        home_dir: &std::path::Path,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if !home_dir.exists() {
            warnings
                .push(ValidationWarning {
                    field: "home_dir".to_string(),
                    message: "Home directory does not exist".to_string(),
                    suggestion: Some(
                        "Directory will be created automatically".to_string(),
                    ),
                });
        } else if !home_dir.is_dir() {
            errors
                .push(ValidationError {
                    field: "home_dir".to_string(),
                    message: "Home directory path exists but is not a directory"
                        .to_string(),
                    suggestion: Some(
                        "Choose a different path for home directory".to_string(),
                    ),
                });
        }
    }
    pub fn validate_and_fix_config(
        &self,
        config: &mut crate::SymorConfig,
    ) -> Result<ValidationResult> {
        let result = self.validate_config(config);
        if config.versioning.max_versions == 0 {
            config.versioning.max_versions = 10;
        }
        if config.versioning.compression > 9 {
            config.versioning.compression = 9;
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_config_validation() {
        let validator = ConfigValidator::new();
        let config = crate::SymorConfig {
            home_dir: PathBuf::from("/tmp/symor"),
            versioning: crate::VersioningConfig {
                enabled: true,
                max_versions: 0,
                compression: 10,
            },
            linking: crate::LinkingConfig {
                link_type: "invalid".to_string(),
                preserve_permissions: true,
            },
        };
        let result = validator.validate_config(&config);
        assert!(! result.is_valid);
        assert!(result.errors.len() >= 3);
    }
}