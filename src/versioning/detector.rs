use anyhow::{Context, Result};
use md5;
use std::{
    collections::HashMap, path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Moved,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: SystemTime,
    pub old_hash: Option<String>,
    pub new_hash: String,
    pub size: Option<u64>,
}
#[derive(Debug, Clone)]
pub struct ChangeDetectorConfig {
    pub debounce_delay: Duration,
    pub hash_algorithm: HashAlgorithm,
    pub ignore_patterns: Vec<String>,
}
impl Default for ChangeDetectorConfig {
    fn default() -> Self {
        Self {
            debounce_delay: Duration::from_millis(100),
            hash_algorithm: HashAlgorithm::MD5,
            ignore_patterns: vec![
                "*.tmp".to_string(), "*.swp".to_string(), ".git/**".to_string(),
                "target/**".to_string(),
            ],
        }
    }
}
#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    MD5,
}
pub struct ChangeDetector {
    last_hashes: HashMap<PathBuf, String>,
    config: ChangeDetectorConfig,
    pending_changes: HashMap<PathBuf, FileChangeEvent>,
    last_activity: SystemTime,
}
impl ChangeDetector {
    pub fn new() -> Self {
        Self::with_config(ChangeDetectorConfig::default())
    }
    pub fn with_config(config: ChangeDetectorConfig) -> Self {
        Self {
            last_hashes: HashMap::new(),
            config,
            pending_changes: HashMap::new(),
            last_activity: SystemTime::now(),
        }
    }
    pub fn scan_file(&mut self, path: &Path) -> Result<Option<FileChangeEvent>> {
        if !self.should_process_file(path) {
            return Ok(None);
        }

        // Handle directories - track existence without trying to hash
        if path.is_dir() {
            let exists = path.exists();
            let was_tracked = self.last_hashes.contains_key(path);

            match (was_tracked, exists) {
                (false, true) => {
                    // Directory was created
                    self.last_hashes.insert(path.to_path_buf(), "directory".to_string());
                    return Ok(Some(FileChangeEvent {
                        path: path.to_path_buf(),
                        change_type: ChangeType::Created,
                        timestamp: SystemTime::now(),
                        old_hash: None,
                        new_hash: "directory".to_string(),
                        size: None,
                    }));
                }
                (true, false) => {
                    // Directory was deleted
                    self.last_hashes.remove(path);
                    return Ok(Some(FileChangeEvent {
                        path: path.to_path_buf(),
                        change_type: ChangeType::Deleted,
                        timestamp: SystemTime::now(),
                        old_hash: Some("directory".to_string()),
                        new_hash: "".to_string(),
                        size: None,
                    }));
                }
                _ => return Ok(None), // No change
            }
        }

        let current_hash = self.calculate_file_hash(path)?;
        let previous_hash = self.last_hashes.get(path);
        let change_event = match (previous_hash, path.exists()) {
            (None, true) => {
                self.last_hashes.insert(path.to_path_buf(), current_hash.clone());
                Some(FileChangeEvent {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Created,
                    timestamp: SystemTime::now(),
                    old_hash: None,
                    new_hash: current_hash,
                    size: path.metadata().ok().map(|m| m.len()),
                })
            }
            (Some(old_hash), true) if old_hash != &current_hash => {
                let old_hash_clone = old_hash.clone();
                self.last_hashes.insert(path.to_path_buf(), current_hash.clone());
                Some(FileChangeEvent {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Modified,
                    timestamp: SystemTime::now(),
                    old_hash: Some(old_hash_clone),
                    new_hash: current_hash,
                    size: path.metadata().ok().map(|m| m.len()),
                })
            }
            (Some(_), false) => {
                self.last_hashes.remove(path);
                Some(FileChangeEvent {
                    path: path.to_path_buf(),
                    change_type: ChangeType::Deleted,
                    timestamp: SystemTime::now(),
                    old_hash: None,
                    new_hash: String::new(),
                    size: None,
                })
            }
            _ => None,
        };
        if change_event.is_some() {
            self.last_activity = SystemTime::now();
        }
        Ok(change_event)
    }
    pub fn scan_files(&mut self, paths: &[PathBuf]) -> Result<Vec<FileChangeEvent>> {
        let mut changes = Vec::new();
        for path in paths {
            if let Some(change) = self.scan_file(path)? {
                changes.push(change);
            }
        }
        Ok(changes)
    }
    fn should_process_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.config.ignore_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return false;
            }
        }
        true
    }
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            let mut current_pos = 0;
            for (i, part) in pattern_parts.iter().enumerate() {
                if i == 0 {
                    if !path.starts_with(part) {
                        return false;
                    }
                    current_pos = part.len();
                } else if i == pattern_parts.len() - 1 {
                    if !path.ends_with(part) {
                        return false;
                    }
                } else {
                    if let Some(pos) = path[current_pos..].find(part) {
                        current_pos += pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            path.contains(pattern)
        }
    }
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        match self.config.hash_algorithm {
            HashAlgorithm::MD5 => {
                let content = std::fs::read(path)
                    .with_context(|| format!("Failed to read file: {:?}", path))?;
                Ok(format!("{:x}", md5::compute(& content)))
            }
        }
    }
    pub fn last_activity(&self) -> SystemTime {
        self.last_activity
    }
    pub fn clear_hashes(&mut self) {
        self.last_hashes.clear();
    }
    pub fn stats(&self) -> ChangeDetectorStats {
        ChangeDetectorStats {
            tracked_files: self.last_hashes.len(),
            pending_changes: self.pending_changes.len(),
            last_activity: self.last_activity,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ChangeDetectorStats {
    pub tracked_files: usize,
    pub pending_changes: usize,
    pub last_activity: SystemTime,
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    #[test]
    fn test_file_creation_detection() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut detector = ChangeDetector::new();
        assert!(detector.scan_file(& file_path).unwrap().is_none());
        fs::write(&file_path, "Hello, World!").unwrap();
        let change = detector.scan_file(&file_path).unwrap().unwrap();
        assert_eq!(change.change_type, ChangeType::Created);
        assert_eq!(change.path, file_path);
        assert!(change.old_hash.is_none());
    }
    #[test]
    fn test_file_modification_detection() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut detector = ChangeDetector::new();
        fs::write(&file_path, "Hello").unwrap();
        detector.scan_file(&file_path).unwrap();
        fs::write(&file_path, "Hello, World!").unwrap();
        let change = detector.scan_file(&file_path).unwrap().unwrap();
        assert_eq!(change.change_type, ChangeType::Modified);
        assert!(change.old_hash.is_some());
    }
    #[test]
    fn test_ignore_patterns() {
        let mut detector = ChangeDetector::new();
        assert!(! detector.should_process_file(Path::new("target/debug/binary")));
        assert!(! detector.should_process_file(Path::new("file.tmp")));
        assert!(detector.should_process_file(Path::new("src/main.rs")));
    }
}