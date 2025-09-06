use anyhow::Result;
use std::{
    fs, path::{Path, PathBuf},
    time::SystemTime,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[derive(Debug, Clone)]
pub struct RestoreOptions {
    pub preserve_permissions: bool,
    pub create_backup: bool,
    pub backup_suffix: String,
    pub atomic_restore: bool,
}
impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            preserve_permissions: true,
            create_backup: false,
            backup_suffix: ".backup".to_string(),
            atomic_restore: true,
        }
    }
}
pub struct RestoreEngine {
    temp_dir: PathBuf,
}
impl RestoreEngine {
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("symor-restore");
        fs::create_dir_all(&temp_dir)?;
        Ok(Self { temp_dir })
    }
    pub fn restore_file(
        &self,
        target_path: &Path,
        content: &[u8],
        options: &RestoreOptions,
    ) -> Result<RestoreResult> {
        let original_metadata = if options.preserve_permissions {
            target_path.metadata().ok()
        } else {
            None
        };
        let backup_path = if options.create_backup && target_path.exists() {
            Some(target_path.with_extension(&options.backup_suffix))
        } else {
            None
        };
        if let Some(ref backup_path) = backup_path {
            fs::copy(target_path, backup_path)?;
        }
        let result = if options.atomic_restore {
            self.atomic_restore(target_path, content)?
        } else {
            self.direct_restore(target_path, content)?
        };
        if let (Some(metadata), true) = (
            original_metadata,
            options.preserve_permissions,
        ) {
            if let Ok(mut perms) = fs::metadata(target_path).map(|m| m.permissions()) {
                #[cfg(unix)]
                {
                    perms.set_mode(metadata.permissions().mode());
                }
                let _ = fs::set_permissions(target_path, perms);
            }
        }
        Ok(result)
    }
    fn atomic_restore(
        &self,
        target_path: &Path,
        content: &[u8],
    ) -> Result<RestoreResult> {
        let temp_filename = format!(
            "restore_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap().as_nanos()
        );
        let temp_path = self.temp_dir.join(temp_filename);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, target_path)?;
        Ok(RestoreResult {
            success: true,
            bytes_written: content.len() as u64,
            temp_file_used: true,
            backup_created: false,
        })
    }
    fn direct_restore(
        &self,
        target_path: &Path,
        content: &[u8],
    ) -> Result<RestoreResult> {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target_path, content)?;
        Ok(RestoreResult {
            success: true,
            bytes_written: content.len() as u64,
            temp_file_used: false,
            backup_created: false,
        })
    }
    pub fn batch_restore(
        &self,
        operations: Vec<RestoreOperation>,
        options: &RestoreOptions,
    ) -> Result<BatchRestoreResult> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        let total_operations = operations.len();
        for operation in operations {
            match self.restore_file(&operation.target_path, &operation.content, options)
            {
                Ok(result) => {
                    results.push(Ok(result));
                    success_count += 1;
                }
                Err(e) => {
                    results.push(Err(e));
                    failure_count += 1;
                }
            }
        }
        Ok(BatchRestoreResult {
            total_operations,
            success_count,
            failure_count,
            results,
        })
    }
    pub fn validate_restore(
        &self,
        target_path: &Path,
        content: &[u8],
    ) -> Result<RestoreValidation> {
        let mut issues = Vec::new();
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                if let Some(grandparent) = parent.parent() {
                    if !grandparent.exists()
                        || grandparent.metadata()?.permissions().readonly()
                    {
                        issues.push(ValidationIssue::CannotCreateParentDirectory);
                    }
                }
            } else if parent.metadata()?.permissions().readonly() {
                issues.push(ValidationIssue::ParentDirectoryNotWritable);
            }
        }
        if target_path.exists() {
            if target_path.metadata()?.permissions().readonly() {
                issues.push(ValidationIssue::TargetFileNotWritable);
            }
        }
        let required_space = content.len() as u64;
        if let Some(parent) = target_path.parent() {
            if let Ok(metadata) = parent.metadata() {
                if metadata.len() < required_space {
                    issues.push(ValidationIssue::InsufficientDiskSpace);
                }
            }
        }
        Ok(RestoreValidation {
            can_proceed: issues.is_empty(),
            issues,
            estimated_space_required: required_space,
        })
    }
    pub fn cleanup_temp_files(&self) -> Result<usize> {
        let mut cleaned_count = 0;
        if self.temp_dir.exists() {
            for entry in fs::read_dir(&self.temp_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().unwrap_or_default().as_secs() > 3600 {
                            fs::remove_file(&path)?;
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }
        Ok(cleaned_count)
    }
}
#[derive(Debug, Clone)]
pub struct RestoreOperation {
    pub target_path: PathBuf,
    pub content: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub success: bool,
    pub bytes_written: u64,
    pub temp_file_used: bool,
    pub backup_created: bool,
}
#[derive(Debug)]
pub struct BatchRestoreResult {
    pub total_operations: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub results: Vec<Result<RestoreResult, anyhow::Error>>,
}
#[derive(Debug, Clone)]
pub struct RestoreValidation {
    pub can_proceed: bool,
    pub issues: Vec<ValidationIssue>,
    pub estimated_space_required: u64,
}
#[derive(Debug, Clone)]
pub enum ValidationIssue {
    CannotCreateParentDirectory,
    ParentDirectoryNotWritable,
    TargetFileNotWritable,
    InsufficientDiskSpace,
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn test_atomic_restore() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");
        let content = b"Hello, restored world!";
        let engine = RestoreEngine::new().unwrap();
        let options = RestoreOptions::default();
        let result = engine.restore_file(&target_path, content, &options).unwrap();
        assert!(result.success);
        assert_eq!(result.bytes_written, content.len() as u64);
        assert!(result.temp_file_used);
        let restored_content = fs::read(&target_path).unwrap();
        assert_eq!(restored_content, content);
    }
    #[test]
    fn test_restore_validation() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");
        let content = b"Test content";
        let engine = RestoreEngine::new().unwrap();
        let validation = engine.validate_restore(&target_path, content).unwrap();
        assert!(validation.can_proceed);
        assert!(validation.issues.is_empty());
    }
    #[test]
    fn test_batch_restore() {
        let temp_dir = tempdir().unwrap();
        let operations = vec![
            RestoreOperation { target_path : temp_dir.path().join("file1.txt"), content :
            b"Content 1".to_vec(), }, RestoreOperation { target_path : temp_dir.path()
            .join("file2.txt"), content : b"Content 2".to_vec(), },
        ];
        let engine = RestoreEngine::new().unwrap();
        let options = RestoreOptions::default();
        let result = engine.batch_restore(operations, &options).unwrap();
        assert_eq!(result.total_operations, 2);
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failure_count, 0);
    }
}