#[cfg(test)]
mod tests {
    use crate::{SymorManager, versioning};
    use std::fs;
    use tempfile::tempdir;
    #[test]
    fn test_full_versioning_workflow() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let restored_file = temp_dir.path().join("restored.txt");
        fs::write(&test_file, "Hello, World!").unwrap();
        let mut manager = SymorManager::new().unwrap();
        let file_id = manager.watch(test_file.clone(), false).unwrap();
        fs::write(&test_file, "Hello, Updated World!").unwrap();
        manager.create_backup(&file_id).unwrap();
        manager.list_versions(&file_id).unwrap();
        fs::write(&test_file, "Restored content").unwrap();
        manager.create_backup(&file_id).unwrap();
        let test_version_id = "test-version";
        let _ = manager.restore_file(&file_id, test_version_id, &restored_file);
        let restored_content = fs::read_to_string(&restored_file).unwrap();
        assert_eq!(restored_content, "Hello, Updated World!");
    }
    #[test]
    fn test_change_detection_integration() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("detect.txt");
        fs::write(&test_file, "Initial content").unwrap();
        let mut manager = SymorManager::new().unwrap();
        let file_id = manager.watch(test_file.clone(), false).unwrap();
        fs::write(&test_file, "Modified content").unwrap();
        let changes = manager.change_detector.scan_file(&test_file).unwrap();
        assert!(changes.is_some());
        let change = changes.unwrap();
        assert_eq!(change.change_type, versioning::detector::ChangeType::Modified);
    }
    #[test]
    fn test_version_storage_integration() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("storage_test.txt");
        fs::write(&test_file, "Test content for storage").unwrap();
        let mut manager = SymorManager::new().unwrap();
        let file_id = manager.watch(test_file.clone(), false).unwrap();
        manager.create_backup(&file_id).unwrap();
        let stats = manager.version_storage.get_stats().unwrap();
        assert!(stats.total_versions >= 1);
        assert!(stats.total_original_size > 0);
    }
    #[test]
    fn test_restore_engine_integration() {
        let temp_dir = tempdir().unwrap();
        let original_file = temp_dir.path().join("original.txt");
        let backup_file = temp_dir.path().join("backup.txt");
        let content = b"Content to be restored";
        fs::write(&original_file, content).unwrap();
        let mut manager = SymorManager::new().unwrap();
        let options = versioning::restore::RestoreOptions {
            preserve_permissions: false,
            create_backup: true,
            backup_suffix: ".bak".to_string(),
            atomic_restore: true,
        };
        manager.restore_engine.restore_file(&backup_file, content, &options).unwrap();
        let restored_content = fs::read(&backup_file).unwrap();
        assert_eq!(restored_content, content);
    }
    #[test]
    fn test_error_recovery_integration() {
        use crate::errors::recovery::ErrorRecovery;
        let recovery = ErrorRecovery::new();
        let mut attempt_count = 0;
        let result: Result<String, _> = tokio_test::block_on(
            recovery
                .execute_recovery(
                    "FileNotFound",
                    || {
                        attempt_count += 1;
                        if attempt_count < 2 {
                            Err(anyhow::anyhow!("File not found"))
                        } else {
                            Ok("success".to_string())
                        }
                    },
                ),
        );
        assert!(result.is_ok());
        assert_eq!(attempt_count, 2);
    }
    #[test]
    fn test_notification_system_integration() {
        use crate::monitoring::notifications::{
            NotificationSystem, FileChangeNotification, NotificationLevel,
        };
        let notification_system = NotificationSystem::new();
        let notification = FileChangeNotification {
            path: std::path::PathBuf::from("/test/path"),
            change_type: "modified".to_string(),
            timestamp: std::time::SystemTime::now(),
            level: NotificationLevel::Info,
        };
        notification_system.notify_file_change(notification.clone()).unwrap();
        let received = notification_system.receive_notification().unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().path, notification.path);
    }
    #[test]
    fn test_configuration_templates_integration() {
        use crate::config::templates::{TemplateManager, ConfigOverrides};
        let mut template_manager = TemplateManager::new();
        template_manager.load_builtin_templates().unwrap();
        let templates = template_manager.list_templates();
        assert!(! templates.is_empty());
        let overrides = ConfigOverrides {
            max_versions: Some(50),
            compression: Some(6),
            link_type: Some("copy".to_string()),
        };
        let config = template_manager
            .create_from_template("development", &overrides)
            .unwrap();
        assert_eq!(config.versioning.max_versions, 50);
        assert_eq!(config.versioning.compression, 6);
        assert_eq!(config.linking.link_type, "copy");
    }
    #[test]
    fn test_incremental_sync_integration() {
        use crate::performance::incremental::IncrementalSync;
        let temp_dir = tempdir().unwrap();
        let old_file = temp_dir.path().join("old.txt");
        let new_file = temp_dir.path().join("new.txt");
        fs::write(&old_file, "Hello, World!").unwrap();
        fs::write(&new_file, "Hello, Updated World!").unwrap();
        let sync = IncrementalSync::new(4);
        let deltas = sync.calculate_delta(&old_file, &new_file).unwrap();
        assert!(! deltas.is_empty());
    }
    #[test]
    fn test_parallel_processing_integration() {
        use crate::performance::parallel::{ParallelProcessor, ProcessResult};
        let temp_dir = tempdir().unwrap();
        let files = vec![
            temp_dir.path().join("file1.txt"), temp_dir.path().join("file2.txt"),
            temp_dir.path().join("file3.txt"),
        ];
        for file in &files {
            fs::write(file, "test content").unwrap();
        }
        let processor = ParallelProcessor::new(2);
        let results = processor
            .process_files_parallel(
                files.clone(),
                |path| {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(| r : & ProcessResult | r.success));
    }
    #[test]
    fn test_end_to_end_workflow() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");
        fs::write(&source_file, "Initial content").unwrap();
        let mut manager = SymorManager::new().unwrap();
        let file_id = manager.watch(source_file.clone(), false).unwrap();
        fs::write(&source_file, "Updated content").unwrap();
        manager.create_backup(&file_id).unwrap();
        manager.list_versions(&file_id).unwrap();
        let test_version_id = "test-version";
        let _ = manager.restore_file(&file_id, test_version_id, &target_file);
        let target_content = fs::read_to_string(&target_file).unwrap();
        assert_eq!(target_content, "Updated content");
        manager.get_info(&source_file).unwrap();
        manager.list_watched(false).unwrap();
    }
}