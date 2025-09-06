use anyhow::{Context, Result};
use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use serde::{Deserialize, Serialize};
use std::{
    fs, path::{Path, PathBuf},
    time::SystemTime, io::{Read, Write},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub id: String,
    pub original_path: PathBuf,
    pub timestamp: SystemTime,
    pub size: u64,
    pub compressed_size: u64,
    pub hash: String,
    pub compression_level: u8,
}
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub compression_level: u8,
    pub max_versions_per_file: usize,
    pub storage_path: PathBuf,
}
impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            compression_level: 6,
            max_versions_per_file: 10,
            storage_path: PathBuf::from(".symor/versions"),
        }
    }
}
pub struct VersionStorage {
    config: StorageConfig,
}
impl VersionStorage {
    pub fn new() -> Self {
        Self::with_config(StorageConfig::default())
    }
    pub fn with_config(config: StorageConfig) -> Self {
        Self { config }
    }
    pub fn store_version(
        &self,
        file_path: &Path,
        content: &[u8],
        version_id: &str,
    ) -> Result<VersionMetadata> {
        fs::create_dir_all(&self.config.storage_path)?;
        let storage_path = self.get_storage_path(version_id);
        let compressed_data = self.compress_data(content)?;
        let temp_path = storage_path.with_extension("tmp");
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&temp_path, &compressed_data)?;
        fs::rename(&temp_path, &storage_path)?;
        let metadata = VersionMetadata {
            id: version_id.to_string(),
            original_path: file_path.to_path_buf(),
            timestamp: SystemTime::now(),
            size: content.len() as u64,
            compressed_size: compressed_data.len() as u64,
            hash: format!("{:x}", md5::compute(content)),
            compression_level: self.config.compression_level,
        };
        self.save_metadata(&metadata)?;
        Ok(metadata)
    }
    pub fn retrieve_version(
        &self,
        version_id: &str,
    ) -> Result<(Vec<u8>, VersionMetadata)> {
        let storage_path = self.get_storage_path(version_id);
        let compressed_data = fs::read(&storage_path)
            .with_context(|| {
                format!("Failed to read version file: {:?}", storage_path)
            })?;
        let decompressed_data = self.decompress_data(&compressed_data)?;
        let metadata = self.load_metadata(version_id)?;
        Ok((decompressed_data, metadata))
    }
    pub fn delete_version(&self, version_id: &str) -> Result<()> {
        let storage_path = self.get_storage_path(version_id);
        let metadata_path = self.get_metadata_path(version_id);
        let _ = fs::remove_file(&storage_path);
        let _ = fs::remove_file(&metadata_path);
        Ok(())
    }
    pub fn list_versions(&self, file_path: &Path) -> Result<Vec<VersionMetadata>> {
        let mut versions = Vec::new();
        let metadata_dir = self.config.storage_path.join("metadata");
        if !metadata_dir.exists() {
            return Ok(versions);
        }
        for entry in fs::read_dir(&metadata_dir)? {
            let entry = entry?;
            let metadata_path = entry.path();
            if let Ok(metadata) = self.load_metadata_from_path(&metadata_path) {
                if metadata.original_path == file_path {
                    versions.push(metadata);
                }
            }
        }
        versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(versions)
    }
    pub fn cleanup_old_versions(&self, file_path: &Path) -> Result<usize> {
        let versions = self.list_versions(file_path)?;
        let mut deleted_count = 0;
        if versions.len() > self.config.max_versions_per_file {
            let to_delete = versions.len() - self.config.max_versions_per_file;
            for version in versions.iter().rev().take(to_delete) {
                self.delete_version(&version.id)?;
                deleted_count += 1;
            }
        }
        Ok(deleted_count)
    }
    pub fn get_stats(&self) -> Result<StorageStats> {
        let mut total_versions = 0;
        let mut total_original_size = 0;
        let mut total_compressed_size = 0;
        let metadata_dir = self.config.storage_path.join("metadata");
        if metadata_dir.exists() {
            for entry in fs::read_dir(&metadata_dir)? {
                let entry = entry?;
                if let Ok(metadata) = self.load_metadata_from_path(&entry.path()) {
                    total_versions += 1;
                    total_original_size += metadata.size;
                    total_compressed_size += metadata.compressed_size;
                }
            }
        }
        Ok(StorageStats {
            total_versions,
            total_original_size,
            total_compressed_size,
            compression_ratio: if total_original_size > 0 {
                total_compressed_size as f64 / total_original_size as f64
            } else {
                0.0
            },
        })
    }
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(
            Vec::new(),
            Compression::new(self.config.compression_level as u32),
        );
        encoder.write_all(data)?;
        encoder.finish().context("Failed to compress data")
    }
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
    fn get_storage_path(&self, version_id: &str) -> PathBuf {
        self.config.storage_path.join("data").join(format!("{}.gz", version_id))
    }
    fn get_metadata_path(&self, version_id: &str) -> PathBuf {
        self.config.storage_path.join("metadata").join(format!("{}.json", version_id))
    }
    fn save_metadata(&self, metadata: &VersionMetadata) -> Result<()> {
        let metadata_dir = self.config.storage_path.join("metadata");
        fs::create_dir_all(&metadata_dir)?;
        let metadata_path = self.get_metadata_path(&metadata.id);
        let json_data = serde_json::to_string_pretty(metadata)?;
        fs::write(&metadata_path, json_data)?;
        Ok(())
    }
    fn load_metadata(&self, version_id: &str) -> Result<VersionMetadata> {
        let metadata_path = self.get_metadata_path(version_id);
        let json_data = fs::read_to_string(&metadata_path)?;
        let metadata: VersionMetadata = serde_json::from_str(&json_data)?;
        Ok(metadata)
    }
    fn load_metadata_from_path(&self, path: &Path) -> Result<VersionMetadata> {
        let json_data = fs::read_to_string(path)?;
        let metadata: VersionMetadata = serde_json::from_str(&json_data)?;
        Ok(metadata)
    }
}
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_versions: usize,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub compression_ratio: f64,
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn test_version_storage() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("versions");
        let config = StorageConfig {
            storage_path,
            ..Default::default()
        };
        let storage = VersionStorage::with_config(config);
        let test_content = b"Hello, World! This is test content.";
        let test_path = PathBuf::from("test.txt");
        let version_id = "test-version-1";
        let metadata = storage
            .store_version(&test_path, test_content, version_id)
            .unwrap();
        assert_eq!(metadata.id, version_id);
        assert_eq!(metadata.size, test_content.len() as u64);
        assert_eq!(metadata.original_path, test_path);
        let (retrieved_content, retrieved_metadata) = storage
            .retrieve_version(version_id)
            .unwrap();
        assert_eq!(retrieved_content, test_content);
        assert_eq!(retrieved_metadata.id, version_id);
        let versions = storage.list_versions(&test_path).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].id, version_id);
    }
    #[test]
    fn test_compression() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("versions");
        let config = StorageConfig {
            storage_path,
            compression_level: 9,
            ..Default::default()
        };
        let storage = VersionStorage::with_config(config);
        let test_content = vec![b'A'; 10000];
        let test_path = PathBuf::from("compressible.txt");
        let version_id = "compress-test";
        let metadata = storage
            .store_version(&test_path, &test_content, version_id)
            .unwrap();
        assert!(metadata.compressed_size < metadata.size);
        assert!(metadata.compression_level == 9);
    }
}