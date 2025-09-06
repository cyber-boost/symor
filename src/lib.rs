use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult,
    Watcher,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, fs, path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant, SystemTime},
};
pub mod versioning;
pub mod monitoring;
pub mod config;
pub mod errors;
pub mod performance;
pub mod tui;
#[cfg(test)]
mod tests;
const DEBOUNCE_DELAY: Duration = Duration::from_millis(100);
pub struct Mirror {
    src: PathBuf,
    targets: Vec<PathBuf>,
    rx: Receiver<NotifyResult<Event>>,
    _watcher: RecommendedWatcher,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymorConfig {
    pub home_dir: PathBuf,
    pub versioning: VersioningConfig,
    pub linking: LinkingConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    pub enabled: bool,
    pub max_versions: usize,
    pub compression: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkingConfig {
    pub link_type: String,
    pub preserve_permissions: bool,
}
impl Default for SymorConfig {
    fn default() -> Self {
        Self {
            home_dir: get_default_home_dir(),
            versioning: VersioningConfig {
                enabled: true,
                max_versions: 10,
                compression: 6,
            },
            linking: LinkingConfig {
                link_type: "copy".to_string(),
                preserve_permissions: true,
            },
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub id: String,
    pub timestamp: SystemTime,
    pub size: u64,
    pub hash: String,
    pub path: PathBuf,
    #[serde(default)]
    pub backup_path: Option<PathBuf>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedItem {
    pub id: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub recursive: bool,
    pub versions: Vec<FileVersion>,
    pub created_at: SystemTime,
    pub last_modified: SystemTime,
}
pub struct SymorManager {
    config: SymorConfig,
    watched_items: HashMap<String, WatchedItem>,
    change_detector: versioning::detector::ChangeDetector,
    version_storage: versioning::storage::VersionStorage,
    restore_engine: versioning::restore::RestoreEngine,
}
pub fn get_default_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".symor")
    } else if let Ok(user) = std::env::var("USERPROFILE") {
        PathBuf::from(user).join(".symor")
    } else {
        PathBuf::from("/tmp/.symor")
    }
}
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", timestamp)
}
impl Mirror {
    pub fn new(src: impl Into<PathBuf>, targets: Vec<PathBuf>) -> Result<Self> {
        let src = src.into();
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .context("failed to initialise file‑watcher")?;
        watcher
            .watch(&src, RecursiveMode::NonRecursive)
            .with_context(|| format!("cannot watch source file {:?}", src))?;
        Ok(Self {
            src,
            targets,
            rx,
            _watcher: watcher,
        })
    }
    fn sync_once(&self) -> Result<()> {
        let data = fs::read(&self.src)
            .with_context(|| format!("cannot read source file {:?}", self.src))?;
        for tgt in &self.targets {
            if let Some(parent) = tgt.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("cannot create directory {:?}", parent))?;
            }
            let tmp = tgt.with_extension("tmp-sync");
            fs::write(&tmp, &data)
                .with_context(|| format!("cannot write temporary file {:?}", tmp))?;
            fs::rename(&tmp, tgt)
                .with_context(|| format!("cannot atomically replace {:?}", tgt))?;
        }
        Ok(())
    }
    pub fn run(self) -> Result<()> {
        self.sync_once().with_context(|| "initial sync failed")?;
        info!("Watching {:?} → {} target(s)", self.src, self.targets.len());
        let mut pending = false;
        let mut last_event: Option<Event> = None;
        let mut debounce_deadline = Instant::now();
        loop {
            let timeout = if pending {
                debounce_deadline.checked_duration_since(Instant::now())
            } else {
                None
            };
            match self
                .rx
                .recv_timeout(timeout.unwrap_or_else(|| Duration::from_secs(u64::MAX)))
            {
                Ok(Ok(ev)) => {
                    debug!("raw notify event: {:?}", ev);
                    if Self::is_interesting(&ev) {
                        pending = true;
                        last_event = Some(ev);
                        debounce_deadline = Instant::now() + DEBOUNCE_DELAY;
                    }
                }
                Ok(Err(e)) => {
                    warn!("watcher error: {e:?}");
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if pending {
                        match self.sync_once() {
                            Ok(_) => {
                                if let Some(ev) = &last_event {
                                    info!("synced after {:?}", ev.kind);
                                } else {
                                    info!("synced");
                                }
                            }
                            Err(e) => error!("sync failed: {e:?}"),
                        }
                        pending = false;
                        last_event = None;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    error!("watcher thread terminated unexpectedly");
                    break;
                }
            }
        }
        Ok(())
    }
    fn is_interesting(event: &Event) -> bool {
        matches!(
            event.kind, EventKind::Modify(_) | EventKind::Create(_) |
            EventKind::Remove(_) | EventKind::Any
        )
    }
}
impl SymorManager {
    pub fn new() -> Result<Self> {
        let config = SymorConfig::default();
        let watched_items = HashMap::new();
        Self::setup_directory_structure(&config.home_dir)?;
        let change_detector = versioning::detector::ChangeDetector::new();
        let version_storage = versioning::storage::VersionStorage::new();
        let restore_engine = versioning::restore::RestoreEngine::new()?;
        let manager = Self {
            config,
            watched_items,
            change_detector,
            version_storage,
            restore_engine,
        };
        Ok(manager)
    }
    pub fn setup_directory_structure(home_dir: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir_all(home_dir)?;
        let mut perms = fs::metadata(home_dir)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(home_dir, perms)?;
        let backups_dir = home_dir.join("backups");
        fs::create_dir_all(&backups_dir)?;
        let mut backup_perms = fs::metadata(&backups_dir)?.permissions();
        backup_perms.set_mode(0o700);
        fs::set_permissions(&backups_dir, backup_perms)?;
        let temp_dir = home_dir.join("temp");
        fs::create_dir_all(&temp_dir)?;
        let mut temp_perms = fs::metadata(&temp_dir)?.permissions();
        temp_perms.set_mode(0o700);
        fs::set_permissions(&temp_dir, temp_perms)?;
        let logs_dir = home_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;
        let mut logs_perms = fs::metadata(&logs_dir)?.permissions();
        logs_perms.set_mode(0o700);
        fs::set_permissions(&logs_dir, logs_perms)?;
        let config_path = home_dir.join("config.json");
        if config_path.exists() {
            let mut config_perms = fs::metadata(&config_path)?.permissions();
            config_perms.set_mode(0o600);
            fs::set_permissions(&config_path, config_perms)?;
        }
        let mirror_path = home_dir.join("mirror.json");
        if mirror_path.exists() {
            let mut mirror_perms = fs::metadata(&mirror_path)?.permissions();
            mirror_perms.set_mode(0o600);
            fs::set_permissions(&mirror_path, mirror_perms)?;
        }
        info!(
            "Created symor directory structure with secure permissions at {:?}", home_dir
        );
        Ok(())
    }
    pub fn load_config(&mut self) -> Result<()> {
        let config_path = self.config.home_dir.join("config.json");
        if config_path.exists() {
            let config_data = fs::read_to_string(&config_path)?;
            let loaded_config: SymorConfig = serde_json::from_str(&config_data)?;
            self.config = loaded_config;
        }
        Ok(())
    }
    pub fn save_config(&self) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let config_path = self.config.home_dir.join("config.json");
        let config_data = serde_json::to_string_pretty(&self.config)?;
        fs::write(&config_path, config_data)?;
        let mut perms = fs::metadata(&config_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&config_path, perms)?;
        Ok(())
    }
    pub fn watch(&mut self, path: PathBuf, recursive: bool) -> Result<String> {
        let id = generate_id();
        let is_directory = path.is_dir();
        let watched_item = WatchedItem {
            id: id.clone(),
            path: path.clone(),
            is_directory,
            recursive,
            versions: Vec::new(),
            created_at: SystemTime::now(),
            last_modified: SystemTime::now(),
        };
        self.watched_items.insert(id.clone(), watched_item);
        self.save_watched_items()?;
        if self.config.versioning.enabled {
            self.create_backup(&id)?;
        }
        if let Some(item) = self.watched_items.get(&id) {
            if item.path.exists() {
                self.change_detector.scan_file(&item.path)?;
            }
        }
        info!("Now watching: {:?} (ID: {})", path, id);
        Ok(id)
    }
    pub fn list_watched(&self, detailed: bool) -> Result<()> {
        if self.watched_items.is_empty() {
            println!("No files or directories are currently being watched.");
            return Ok(());
        }
        println!("Watched items:");
        println!("==============");
        for (id, item) in &self.watched_items {
            println!("ID: {}", id);
            println!("Path: {:?}", item.path);
            println!("Type: {}", if item.is_directory { "Directory" } else { "File" });
            println!("Recursive: {}", item.recursive);
            if detailed {
                println!("Created: {:?}", item.created_at);
                println!("Last Modified: {:?}", item.last_modified);
                println!("Versions: {}", item.versions.len());
                if !item.versions.is_empty() {
                    println!(
                        "Latest version: {:?}", item.versions.last().unwrap().timestamp
                    );
                }
            }
            println!();
        }
        Ok(())
    }
    pub fn get_info(&self, path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)?;
        println!("Path: {:?}", path);
        println!("Type: {}", if metadata.is_dir() { "Directory" } else { "File" });
        println!("Size: {} bytes", metadata.len());
        println!("Permissions: {:?}", metadata.permissions());
        println!("Modified: {:?}", metadata.modified() ?);
        for (id, item) in &self.watched_items {
            if item.path == path {
                println!("Watched: Yes (ID: {})", id);
                println!("Recursive: {}", item.recursive);
                println!("Versions: {}", item.versions.len());
                break;
            }
        }
        Ok(())
    }
    fn save_watched_items(&self) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mirror_path = self.config.home_dir.join("mirror.json");
        let mirror_data = serde_json::to_string_pretty(&self.watched_items)?;
        fs::write(&mirror_path, mirror_data)?;
        let mut perms = fs::metadata(&mirror_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&mirror_path, perms)?;
        Ok(())
    }
    pub fn load_watched_items(&mut self) -> Result<()> {
        let mirror_path = self.config.home_dir.join("mirror.json");
        if mirror_path.exists() {
            let mirror_data = fs::read_to_string(mirror_path)?;
            self.watched_items = serde_json::from_str(&mirror_data)?;
        }
        Ok(())
    }
    pub fn install_binary(&self, force: bool) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let bin_name = "sym";
        let install_dir = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            PathBuf::from("/usr/local/bin")
        } else if cfg!(target_os = "windows") {
            std::env::var("USERPROFILE")
                .map(|p| PathBuf::from(p).join("bin"))
                .unwrap_or_else(|_| PathBuf::from("C:\\bin"))
        } else {
            return Err(anyhow::anyhow!("Unsupported platform for installation"));
        };
        let install_path = install_dir.join(bin_name);
        if install_path.exists() && !force {
            println!("sym is already installed at {:?}", install_path);
            println!("Use --force to overwrite existing installation");
            return Ok(());
        }
        fs::create_dir_all(&install_dir)?;
        fs::copy(&current_exe, &install_path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&install_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&install_path, perms)?;
        }
        println!("Successfully installed sym to {:?}", install_path);
        Ok(())
    }
    pub fn uninstall_binary(&self) -> Result<()> {
        let bin_name = "sym";
        let possible_paths = vec![
            PathBuf::from("/usr/local/bin").join(bin_name), PathBuf::from("/usr/bin")
            .join(bin_name), std::env::var("CARGO_HOME").map(| p | PathBuf::from(p)
            .join("bin").join(bin_name)).unwrap_or_else(| _ |
            PathBuf::from("~/.cargo/bin").join(bin_name)),
        ];
        let mut uninstalled = false;
        for path in possible_paths {
            if path.exists() {
                fs::remove_file(&path)?;
                println!("Removed sym from {:?}", path);
                uninstalled = true;
            }
        }
        if !uninstalled {
            println!("sym binary not found in standard locations");
        }
        Ok(())
    }
    pub fn remove_data(&self) -> Result<()> {
        if self.config.home_dir.exists() {
            fs::remove_dir_all(&self.config.home_dir)?;
            println!("Removed symor data directory: {:?}", self.config.home_dir);
        }
        Ok(())
    }
    pub fn config(&self) -> &SymorConfig {
        &self.config
    }
    pub fn watched_items(&self) -> &HashMap<String, WatchedItem> {
        &self.watched_items
    }
    pub fn watched_items_mut(&mut self) -> &mut HashMap<String, WatchedItem> {
        &mut self.watched_items
    }
    pub fn change_detector(&self) -> &versioning::detector::ChangeDetector {
        &self.change_detector
    }
    pub fn change_detector_mut(&mut self) -> &mut versioning::detector::ChangeDetector {
        &mut self.change_detector
    }
    pub fn version_storage(&self) -> &versioning::storage::VersionStorage {
        &self.version_storage
    }
    pub fn restore_engine(&self) -> &versioning::restore::RestoreEngine {
        &self.restore_engine
    }
    pub fn save_watched_items_public(&self) -> Result<()> {
        self.save_watched_items()
    }
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut SymorConfig),
    {
        updater(&mut self.config);
        self.save_config()?;
        Ok(())
    }
    pub fn create_backup(&mut self, item_id: &str) -> Result<()> {
        let item = self
            .watched_items
            .get_mut(item_id)
            .ok_or_else(|| anyhow::anyhow!("Watched item not found: {}", item_id))?;
        if !item.path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", item.path));
        }
        let content = fs::read(&item.path)?;
        let size = content.len() as u64;
        let hash = format!("{:x}", md5::compute(& content));
        let version_id = generate_id();
        let metadata = self
            .version_storage
            .store_version(&item.path, &content, &version_id)?;
        let version = FileVersion {
            id: version_id.clone(),
            timestamp: SystemTime::now(),
            size,
            hash,
            path: item.path.clone(),
            backup_path: Some(metadata.id.clone().into()),
        };
        item.versions.push(version);
        if item.versions.len() > self.config.versioning.max_versions {
            let to_remove = item.versions.len() - self.config.versioning.max_versions;
            for version in item.versions.drain(0..to_remove) {
                let _ = self.version_storage.delete_version(&version.id);
            }
        }
        item.last_modified = SystemTime::now();
        self.save_watched_items()?;
        info!("Created backup for file (version: {})", version_id);
        Ok(())
    }
    pub fn restore_file(
        &self,
        file_id: &str,
        version_id: &str,
        target_path: &Path,
    ) -> Result<()> {
        let item = self
            .watched_items
            .get(file_id)
            .ok_or_else(|| anyhow::anyhow!("Watched item not found: {}", file_id))?;
        let version = item
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version not found: {}", version_id))?;
        match self.version_storage.retrieve_version(version_id) {
            Ok((content, _)) => {
                let options = versioning::restore::RestoreOptions {
                    preserve_permissions: self.config.linking.preserve_permissions,
                    create_backup: true,
                    backup_suffix: ".pre-restore".to_string(),
                    atomic_restore: true,
                };
                self.restore_engine.restore_file(target_path, &content, &options)?;
                info!("Successfully restored file using version storage system");
            }
            Err(_) => {
                let backup_path = version
                    .backup_path
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "No backup path available for version: {}", version_id
                        )
                    })?;
                if !backup_path.exists() {
                    return Err(
                        anyhow::anyhow!("Backup file not found: {:?}", backup_path),
                    );
                }
                let content = fs::read(backup_path)?;
                let options = versioning::restore::RestoreOptions {
                    preserve_permissions: self.config.linking.preserve_permissions,
                    create_backup: true,
                    backup_suffix: ".pre-restore".to_string(),
                    atomic_restore: true,
                };
                self.restore_engine.restore_file(target_path, &content, &options)?;
                info!("Successfully restored file using legacy backup system");
            }
        }
        info!("Restored {:?} to {:?}", version.path, target_path);
        Ok(())
    }
    pub fn list_versions(&self, item_id: &str) -> Result<()> {
        let item = self
            .watched_items
            .get(item_id)
            .ok_or_else(|| anyhow::anyhow!("Watched item not found: {}", item_id))?;
        if item.versions.is_empty() {
            println!("No versions found for item: {}", item_id);
            return Ok(());
        }
        println!("Versions for: {:?}", item.path);
        println!("==============");
        for (i, version) in item.versions.iter().enumerate() {
            println!("{}. Version ID: {}", i + 1, version.id);
            println!("   Timestamp: {:?}", version.timestamp);
            println!("   Size: {} bytes", version.size);
            println!("   Hash: {}", & version.hash[..8]);
            println!(
                "   Backup: {:?}", version.backup_path.as_ref().unwrap_or(&
                PathBuf::from("N/A"))
            );
            println!();
        }
        Ok(())
    }
    pub fn generate_file_id(&self, path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}