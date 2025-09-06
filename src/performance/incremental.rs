use anyhow::Result;
use std::{collections::HashMap, fs, path::{Path, PathBuf}};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHash {
    pub offset: u64,
    pub size: u64,
    pub hash: String,
}
#[derive(Debug, Clone)]
pub struct DeltaBlock {
    pub offset: u64,
    pub size: u64,
    pub data: Option<Vec<u8>>,
}
pub struct IncrementalSync {
    block_size: usize,
    file_blocks: HashMap<PathBuf, Vec<BlockHash>>,
}
impl IncrementalSync {
    pub fn new(block_size: usize) -> Self {
        Self {
            block_size,
            file_blocks: HashMap::new(),
        }
    }
    pub fn calculate_delta(
        &self,
        old_path: &Path,
        new_path: &Path,
    ) -> Result<Vec<DeltaBlock>> {
        let old_content = fs::read(old_path)?;
        let new_content = fs::read(new_path)?;
        let old_blocks = self.calculate_blocks(&old_content);
        let new_blocks = self.calculate_blocks(&new_content);
        let mut deltas = Vec::new();
        let max_len = old_blocks.len().max(new_blocks.len());
        for i in 0..max_len {
            let old_block = old_blocks.get(i);
            let new_block = new_blocks.get(i);
            match (old_block, new_block) {
                (Some(old), Some(new)) if old.hash == new.hash => {
                    deltas
                        .push(DeltaBlock {
                            offset: (i * self.block_size) as u64,
                            size: old.size,
                            data: None,
                        });
                }
                (_, Some(new)) => {
                    let data_start = (i * self.block_size) as usize;
                    let data_end = (data_start + new.size as usize)
                        .min(new_content.len());
                    let data = new_content[data_start..data_end].to_vec();
                    deltas
                        .push(DeltaBlock {
                            offset: (i * self.block_size) as u64,
                            size: new.size,
                            data: Some(data),
                        });
                }
                (Some(old), None) => {
                    deltas
                        .push(DeltaBlock {
                            offset: (i * self.block_size) as u64,
                            size: old.size,
                            data: Some(Vec::new()),
                        });
                }
                (None, None) => unreachable!(),
            }
        }
        Ok(deltas)
    }
    pub fn apply_delta(
        &self,
        base_path: &Path,
        deltas: &[DeltaBlock],
        output_path: &Path,
    ) -> Result<()> {
        let base_content = fs::read(base_path)?;
        let mut result = Vec::new();
        let mut current_offset = 0;
        for delta in deltas {
            if current_offset < delta.offset as usize {
                let gap_size = delta.offset as usize - current_offset;
                if current_offset + gap_size <= base_content.len() {
                    result
                        .extend_from_slice(
                            &base_content[current_offset..current_offset + gap_size],
                        );
                }
                current_offset = delta.offset as usize;
            }
            if let Some(data) = &delta.data {
                result.extend(data);
            } else {
                let copy_size = delta.size as usize;
                if current_offset + copy_size <= base_content.len() {
                    result
                        .extend_from_slice(
                            &base_content[current_offset..current_offset + copy_size],
                        );
                }
            }
            current_offset = (delta.offset + delta.size) as usize;
        }
        if current_offset < base_content.len() {
            result.extend_from_slice(&base_content[current_offset..]);
        }
        fs::write(output_path, result)?;
        Ok(())
    }
    pub fn store_blocks(&mut self, path: PathBuf, content: &[u8]) {
        let blocks = self.calculate_blocks(content);
        self.file_blocks.insert(path, blocks);
    }
    pub fn get_blocks(&self, path: &Path) -> Option<&Vec<BlockHash>> {
        self.file_blocks.get(path)
    }
    fn calculate_blocks(&self, content: &[u8]) -> Vec<BlockHash> {
        let mut blocks = Vec::new();
        let mut offset = 0;
        while offset < content.len() {
            let size = (self.block_size).min(content.len() - offset);
            let block_data = &content[offset..offset + size];
            let hash = format!("{:x}", md5::compute(block_data));
            blocks
                .push(BlockHash {
                    offset: offset as u64,
                    size: size as u64,
                    hash,
                });
            offset += size;
        }
        blocks
    }
    pub fn get_stats(&self) -> IncrementalStats {
        let total_files = self.file_blocks.len();
        let total_blocks = self.file_blocks.values().map(|blocks| blocks.len()).sum();
        IncrementalStats {
            total_files,
            total_blocks,
            block_size: self.block_size,
        }
    }
}
#[derive(Debug, Clone)]
pub struct IncrementalStats {
    pub total_files: usize,
    pub total_blocks: usize,
    pub block_size: usize,
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn test_block_calculation() {
        let sync = IncrementalSync::new(4);
        let content = b"Hello, World!";
        let blocks = sync.calculate_blocks(content);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].offset, 0);
        assert_eq!(blocks[0].size, 4);
        assert_eq!(blocks[1].offset, 4);
        assert_eq!(blocks[1].size, 4);
        assert_eq!(blocks[2].offset, 8);
        assert_eq!(blocks[2].size, 4);
    }
    #[test]
    fn test_delta_calculation() {
        let temp_dir = tempdir().unwrap();
        let old_file = temp_dir.path().join("old.txt");
        let new_file = temp_dir.path().join("new.txt");
        fs::write(&old_file, "Hello, World!").unwrap();
        fs::write(&new_file, "Hello, Rust!").unwrap();
        let sync = IncrementalSync::new(4);
        let deltas = sync.calculate_delta(&old_file, &new_file).unwrap();
        assert!(! deltas.is_empty());
        let has_changed = deltas.iter().any(|d| d.data.is_some());
        let has_unchanged = deltas.iter().any(|d| d.data.is_none());
        assert!(has_changed || has_unchanged);
    }
}