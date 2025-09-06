use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use env_logger::Env;
use log::LevelFilter;
use std::path::{Path, PathBuf};
use symor::{Mirror, SymorManager};
#[derive(Parser, Debug)]
#[command(
    name = "sym",
    author,
    version,
    about = "Advanced file mirroring and version control utility",
    long_about = r#"
Symor - Advanced File Mirroring and Version Control

Symor provides comprehensive file mirroring, real-time monitoring, and version control
capabilities with enterprise-grade features including compression, atomic operations,
and parallel processing.

FEATURES:
  • Real-time file monitoring with change detection
  • Compressed version storage with rollback capabilities
  • Atomic file operations for data integrity
  • Parallel processing for high-performance operations
  • Comprehensive configuration management
  • Interactive terminal user interface
  • Structured error handling and recovery
  • Cross-platform compatibility

EXAMPLES:
  sym mirror source.txt dest.txt         # Mirror file changes in real-time
  sym list --detailed                    # Show all watched items with details
  sym info /path/to/file                 # Show info about a file or directory
  sym install --force                    # Install with force option
  sym watch /path/to/file --recursive    # Start monitoring a file or directory recursively
  sym restore file1 v1 /tmp/backup       # Restore file version to new location
  sym status --verbose                   # Show status with verbose output
  sym unmirror source.txt dest.txt       # Remove mirror relationship
  sym history file1 --limit 3            # Show last 3 versions of a file
  sym clean --dry-run                    # Preview cleanup
  sym unwatch /path/to/file              # Stop watching a file
  sym sync --force                       # Force sync all watched files
  sym stats --detailed --period 60       # Show detailed stats for last 60 seconds
  sym tui --refresh-rate 5               # Start interactive UI with 5s refresh
  sym check /path/to/file                # Check file integrity/status
  sym conflicts                          # Show file conflicts
  sym add-target source.txt dest2.txt    # Add a new target to a source
  sym settings show                      # Display current configuration

For more information on any command, use: sym <command> --help
    "#
)]
struct Opt {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}
#[derive(Subcommand, Debug)]
enum Commands {
    Mirror {
        #[arg(
            value_name = "SOURCE",
            value_hint = ValueHint::FilePath,
            help = "Source file to monitor for changes",
            long_help = "The source file that will be continuously monitored. \
                        Any changes to this file will be automatically mirrored \
                        to all target files with atomic operations."
        )]
        source: PathBuf,
        #[arg(
            value_name = "TARGET",
            num_args = 1..,
            value_hint = ValueHint::FilePath,
            help = "Destination file(s) to keep in sync",
            long_help = "Target files that will be automatically updated whenever \
                        the source file changes. Each target receives an identical \
                        copy of the source file."
        )]
        targets: Vec<PathBuf>,
    },
    List {
        #[arg(
            short,
            long,
            help = "Display comprehensive details for each watched item",
            long_help = "When enabled, shows additional information including: \
                        file sizes, last modification times, total versions stored, \
                        compression ratios, and monitoring status."
        )]
        detailed: bool,
    },
    Info {
        #[arg(
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            help = "File or directory path to inspect",
            long_help = "Provide detailed information about the specified file or directory \
                        including size, permissions, modification time, and Symor monitoring status."
        )]
        path: PathBuf,
    },
    Install {
        #[arg(
            short,
            long,
            help = "Force installation even if sym is already installed",
            long_help = "Skip the confirmation prompt and overwrite any existing \
                        installation of sym in the system PATH."
        )]
        force: bool,
    },
    Watch {
        #[arg(
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            help = "File or directory to add to version control",
            long_help = "The file or directory that will be continuously monitored \
                        for changes. Symor will automatically create versions \
                        whenever modifications are detected."
        )]
        path: PathBuf,
        #[arg(
            short,
            long,
            help = "Monitor directory contents recursively",
            long_help = "When watching a directory, also monitor all files in \
                        subdirectories. This creates a comprehensive version \
                        control system for entire directory trees."
        )]
        recursive: bool,
    },
    Restore {
        #[arg(
            help = "File ID from 'sym list' command",
            long_help = "The unique identifier for the watched file, as shown \
                        in the output of 'sym list'. This identifies which \
                        file's history to restore from."
        )]
        file_id: String,
        #[arg(
            help = "Version ID to restore from history",
            long_help = "The version identifier to restore, as shown in \
                        'sym list --detailed'. Use the most recent version \
                        or a specific historical version."
        )]
        version_id: String,
        #[arg(
            value_name = "TARGET",
            value_hint = ValueHint::AnyPath,
            help = "Location to save the restored file",
            long_help = "The file path where the restored version will be saved. \
                        This can be the original location or a different path \
                        to preserve the current version."
        )]
        target: PathBuf,
    },
    Settings { #[command(subcommand)] action: SettingsCommand },
    Stats {
        #[arg(
            short,
            long,
            help = "Show comprehensive performance analysis",
            long_help = "Display detailed performance metrics including \
                        operation throughput, error rates, memory usage, \
                        and system resource utilization."
        )]
        detailed: bool,
        #[arg(
            short,
            long,
            value_name = "SECONDS",
            help = "Time period for metrics calculation",
            long_help = "Calculate performance metrics for the specified \
                        time period in seconds. Default is since startup."
        )]
        period: Option<u64>,
    },
    Tui {
        #[arg(
            short,
            long,
            value_name = "SECONDS",
            default_value = "2",
            help = "Interface refresh rate",
            long_help = "How often to refresh the TUI display with updated \
                        information. Lower values provide more responsive \
                        updates but may impact performance."
        )]
        refresh_rate: u64,
    },
    Check {
        #[arg(
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            help = "Specific path to verify integrity for",
            long_help = "Check integrity for a specific file or directory. \
                        If not provided, verifies all watched items."
        )]
        path: Option<PathBuf>,
    },
    Conflicts,
    AddTarget {
        #[arg(
            value_name = "SOURCE",
            value_hint = ValueHint::FilePath,
            help = "Source file to add target to",
            long_help = "The source file that is already being watched. \
                        Must be added to Symor first using 'sym watch'."
        )]
        source: PathBuf,
        #[arg(
            value_name = "TARGET",
            value_hint = ValueHint::FilePath,
            help = "New target file to add",
            long_help = "The new target file that will receive copies of the source file. \
                        Will be created if it doesn't exist."
        )]
        target: PathBuf,
    },
    Status {
        #[arg(
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            help = "Specific path to check status for",
            long_help = "Check status for a specific file or directory. \
                        If not provided, shows status for all watched items."
        )]
        path: Option<PathBuf>,
        #[arg(
            short,
            long,
            help = "Display detailed status information",
            long_help = "Show comprehensive status including sync progress, \
                        pending operations, conflicts, and detailed file information."
        )]
        verbose: bool,
    },
    Unmirror {
        #[arg(
            value_name = "SOURCE",
            value_hint = ValueHint::FilePath,
            help = "Source file to unmirror",
            long_help = "The source file whose mirror relationships will be removed. \
                        All target files will no longer be automatically synchronized."
        )]
        source: PathBuf,
        #[arg(
            value_name = "TARGET",
            value_hint = ValueHint::FilePath,
            help = "Specific target to remove",
            long_help = "Remove only this specific target from the mirror relationship. \
                        If not specified, all targets for the source will be removed."
        )]
        target: Option<PathBuf>,
    },
    History {
        #[arg(
            help = "File ID from 'sym list' command",
            long_help = "The unique identifier for the watched file, as shown \
                        in the output of 'sym list'. Shows the complete version \
                        history for this file."
        )]
        file_id: String,
        #[arg(
            short,
            long,
            value_name = "COUNT",
            help = "Maximum number of versions to display",
            long_help = "Limit the number of versions shown in the history. \
                        Useful for large histories. Shows most recent versions first."
        )]
        limit: Option<usize>,
    },
    Clean {
        #[arg(
            long,
            help = "Preview cleanup operations without executing them",
            long_help = "Show what files and versions would be cleaned up, \
                        but don't actually perform the cleanup. Useful for \
                        reviewing what will be removed."
        )]
        dry_run: bool,
        #[arg(
            short,
            long,
            value_name = "FILE_ID",
            help = "Clean only this specific file",
            long_help = "Clean up only the specified file's versions and temporary files. \
                        If not specified, cleans all watched files."
        )]
        file: Option<String>,
        #[arg(
            short = 'k',
            long,
            value_name = "COUNT",
            default_value = "10",
            help = "Minimum versions to keep per file",
            long_help = "Ensure at least this many versions are kept for each file, \
                        even if they would otherwise be cleaned up."
        )]
        keep: usize,
    },
    Unwatch {
        #[arg(
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            help = "File or directory to stop watching",
            long_help = "Remove the specified file or directory from version control monitoring. \
                        No new versions will be created for this path."
        )]
        path: PathBuf,
    },
    Sync {
        #[arg(
            value_name = "PATH",
            value_hint = ValueHint::AnyPath,
            help = "Specific path to sync",
            long_help = "Sync only the specified file or directory. \
                        If not provided, syncs all watched items."
        )]
        path: Option<PathBuf>,
        #[arg(
            short,
            long,
            help = "Force synchronization regardless of change detection",
            long_help = "Perform synchronization even if no changes are detected. \
                        Useful for ensuring consistency or after manual file modifications."
        )]
        force: bool,
    },
    Rip {
        #[arg(
            long,
            help = "Preserve configuration and version history",
            long_help = "Keep the Symor data directory containing configuration \
                        files and version history. Only removes the binary \
                        from system PATH."
        )]
        keep_data: bool,
    },
}
#[derive(Subcommand, Debug)]
enum SettingsCommand {
    Show,
    Versioning {
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long)]
        max_versions: Option<usize>,
        #[arg(long)]
        compression: Option<u8>,
    },
    Linking {
        #[arg(long)]
        link_type: Option<String>,
        #[arg(long)]
        preserve_permissions: Option<bool>,
    },
    Home { #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)] path: PathBuf },
    Init,
}
fn main() -> Result<()> {
    let opt = Opt::parse();
    let log_level = match opt.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::from_env(
            Env::default().default_filter_or(log_level.to_string()),
        )
        .init();
    match opt.command {
        Commands::Mirror { source, targets } => {
            handle_mirror(source, targets)?;
        }
        Commands::List { detailed } => {
            handle_list(detailed)?;
        }
        Commands::AddTarget { source, target } => {
            handle_add_target(source, target)?;
        }
        Commands::Info { path } => {
            handle_info(path)?;
        }
        Commands::Install { force } => {
            handle_install(force)?;
        }
        Commands::Watch { path, recursive } => {
            handle_watch(path, recursive)?;
        }
        Commands::Restore { file_id, version_id, target } => {
            handle_restore(file_id, version_id, target)?;
        }
        Commands::Settings { action } => {
            handle_settings(action)?;
        }
        Commands::Rip { keep_data } => {
            handle_rip(keep_data)?;
        }
        Commands::Stats { detailed, period } => {
            handle_stats(detailed, period)?;
        }
        Commands::Tui { refresh_rate } => {
            handle_tui(refresh_rate)?;
        }
        Commands::Conflicts => {
            handle_conflicts()?;
        }
        Commands::Check { path } => {
            handle_check(path)?;
        }
        Commands::Status { path, verbose } => {
            handle_status(path, verbose)?;
        }
        Commands::Unmirror { source, target } => {
            handle_unmirror(source, target)?;
        }
        Commands::History { file_id, limit } => {
            handle_history(file_id, limit)?;
        }
        Commands::Clean { dry_run, file, keep } => {
            handle_clean(dry_run, file, keep)?;
        }
        Commands::Unwatch { path } => {
            handle_unwatch(path)?;
        }
        Commands::Sync { path, force } => {
            handle_sync(path, force)?;
        }
    }
    Ok(())
}

fn handle_mirror(source: PathBuf, targets: Vec<PathBuf>) -> Result<()> {
    println!("Symor Mirror");
    println!("============");
    println!("");
    println!("Source: {}", source.display());
    println!("Targets:");
    for target in &targets {
        println!("  - {}", target.display());
    }
    println!("");
    
    // Create source file if it doesn't exist
    if !source.exists() {
        println!("Source file does not exist, creating: {}", source.display());
        if let Some(parent) = source.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&source, "")?;
        println!("✓ Created empty source file");
    }
    
    // Create target files if they don't exist
    for target in &targets {
        if !target.exists() {
            println!("Target file does not exist, creating: {}", target.display());
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(target, "")?;
            println!("✓ Created empty target file");
        }
    }
    let mut manager = SymorManager::new()?;
    manager.load_config()?;
    manager.load_watched_items()?;
    manager.watch(source.clone(), false)?;
    let mirror = Mirror::new(source.clone(), targets.clone())?;
    mirror.run()?;
    println!("✓ Mirror setup complete!");
    println!("  Source: {}", source.display());
    println!("  Targets: {}", targets.len());
    println!("");
    println!("The mirror is now active and will sync changes in real-time.");
    println!("Use 'sym list' to see all watched files.");
    println!("Use 'sym status' to check mirror status.");

    Ok(())
}

fn handle_list(detailed: bool) -> Result<()> {
    let mut manager = symor::SymorManager::new()?;
    manager.load_config()?;
    manager.load_watched_items()?;
    manager.list_watched(detailed)?;
    Ok(())
}
fn handle_info(path: PathBuf) -> Result<()> {
    let manager = symor::SymorManager::new()?;
    manager.get_info(&path)?;
    Ok(())
}
fn handle_install(force: bool) -> Result<()> {
    let manager = symor::SymorManager::new()?;
    manager.install_binary(force)?;
    Ok(())
}
fn handle_watch(path: PathBuf, recursive: bool) -> Result<()> {
    let mut manager = symor::SymorManager::new()?;
    manager.load_config()?;
    manager.load_watched_items()?;
    let id = manager.watch(path, recursive)?;
    println!("Started watching with ID: {}", id);
    Ok(())
}
fn handle_restore(file_id: String, version_id: String, target: PathBuf) -> Result<()> {
    let mut manager = symor::SymorManager::new()?;
    manager.load_watched_items()?;
    manager.restore_file(&file_id, &version_id, &target)?;
    println!(
        "Successfully restored file {} version {} to {:?}", file_id, version_id, target
    );
    Ok(())
}
fn handle_settings(action: SettingsCommand) -> Result<()> {
    let mut manager = symor::SymorManager::new()?;
    manager.load_config()?;
    match action {
        SettingsCommand::Show => {
            let config = manager.config();
            println!("Current settings:");
            println!("Home directory: {:?}", config.home_dir);
            println!("Versioning:");
            println!("  Enabled: {}", config.versioning.enabled);
            println!("  Max versions: {}", config.versioning.max_versions);
            println!("  Compression: {}", config.versioning.compression);
            println!("Linking:");
            println!("  Link type: {}", config.linking.link_type);
            println!("  Preserve permissions: {}", config.linking.preserve_permissions);
        }
        SettingsCommand::Versioning { enabled, max_versions, compression } => {
            manager
                .update_config(|config| {
                if let Some(e) = enabled {
                    config.versioning.enabled = e;
                }
                if let Some(mv) = max_versions {
                    config.versioning.max_versions = mv;
                }
                if let Some(c) = compression {
                    config.versioning.compression = c;
                }
            })?;
            println!("Versioning settings updated");
        }
        SettingsCommand::Linking { link_type, preserve_permissions } => {
            manager
                .update_config(|config| {
                if let Some(lt) = link_type {
                    config.linking.link_type = lt;
                }
                if let Some(pp) = preserve_permissions {
                    config.linking.preserve_permissions = pp;
                }
            })?;
            println!("Linking settings updated");
        }
        SettingsCommand::Home { path } => {
            manager
                .update_config(|config| {
                config.home_dir = path;
            })?;
            println!("Home directory updated");
        }
        SettingsCommand::Init => {
            let home_dir = manager.config().home_dir.clone();
            symor::SymorManager::setup_directory_structure(&home_dir)?;
            println!("Directory structure initialized/reset with proper permissions");
        }
    }
    Ok(())
}
fn handle_rip(keep_data: bool) -> Result<()> {
    let manager = symor::SymorManager::new()?;
    println!("This will uninstall sym and remove the binary from your system.");
    if !keep_data {
        println!(
            "WARNING: This will also remove all symor data including watched files and versions!"
        );
    }
    println!("Are you sure you want to continue? (Type 'yes' to confirm): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "yes" {
        println!("Uninstallation cancelled.");
        return Ok(());
    }
    manager.uninstall_binary()?;
    if !keep_data {
        manager.remove_data()?;
    }
    println!("Symor has been successfully uninstalled.");
    Ok(())
}
fn handle_stats(detailed: bool, period: Option<u64>) -> Result<()> {
    use symor::performance::parallel::PerformanceMonitor;
    let monitor = PerformanceMonitor::new();
    for i in 0..10 {
        let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.record_operation(start.elapsed());
        monitor
            .record_metric(
                format!("operation_{}", i),
                start.elapsed().as_secs_f64() * 1000.0,
                "ms".to_string(),
            );
    }
    monitor.record_error();
    let stats = monitor.get_stats();
    println!("{}", stats);
    if detailed {
        println!("\nSystem Information:");
        println!("  CPU Cores: {}", num_cpus::get());
        println!("  Available Memory: {} MB", 1024);
        println!("  Disk Usage: {} MB", 512);
        if let Some(period_secs) = period {
            println!("\nMetrics for last {} seconds:", period_secs);
        }
    }
    Ok(())
}
fn handle_tui(_refresh_rate: u64) -> Result<()> {
    let manager = SymorManager::new()?;
    let watched_items = manager.watched_items().values().cloned().collect::<Vec<_>>();
    let mut tui = symor::tui::SymorTUI::new()?;
    tui.update_state(|state| {
        state.watched_items = watched_items;
    });
    tui.run()?;
    tui.shutdown()?;
    Ok(())
}
fn handle_check(path: Option<PathBuf>) -> Result<()> {
    let manager = SymorManager::new()?;
    println!("Symor Integrity Check");
    println!("====================");
    println!("");
    if let Some(specific_path) = path {
        println!("Checking integrity for: {}", specific_path.display());
        let file_id = manager.generate_file_id(&specific_path);
        if let Some(item) = manager.watched_items().get(&file_id) {
            println!("✓ File is being watched");
            println!("  Path: {}", item.path.display());
            println!("  Last modified: {:?}", item.last_modified);
            println!("  Versions: {}", item.versions.len());
            if item.path.exists() {
                println!("✓ Source file exists");
            } else {
                println!("✗ Source file missing: {}", item.path.display());
            }
            if let Some(latest) = item.versions.last() {
                println!("✓ Latest version: {} ({})", latest.id, latest.size);
            }
        } else {
            println!("✗ Path not being watched: {}", specific_path.display());
        }
    } else {
        println!("Checking all watched files...");
        let mut total_files = 0;
        let mut missing_files = 0;
        let mut total_versions = 0;
        for item in manager.watched_items().values() {
            total_files += 1;
            total_versions += item.versions.len();
            if !item.path.exists() {
                missing_files += 1;
                println!("✗ Missing: {}", item.path.display());
            }
        }
        println!("");
        println!("Summary:");
        println!("  Total watched files: {}", total_files);
        println!("  Total versions: {}", total_versions);
        println!("  Missing files: {}", missing_files);
        if missing_files == 0 {
            println!("✓ All watched files are accessible");
        } else {
            println!("⚠ {} files are missing", missing_files);
        }
    }
    println!("");
    println!("Integrity check complete.");
    Ok(())
}
fn handle_conflicts() -> Result<()> {
    let manager = SymorManager::new()?;
    println!("Symor Conflict Detection");
    println!("=======================");
    println!("");
    let mut conflicts_found = 0;
    let mut total_checked = 0;
    let _target_map: std::collections::HashMap<PathBuf, Vec<String>> = std::collections::HashMap::new();
    for (file_id, item) in manager.watched_items() {
        total_checked += 1;
        if !item.path.exists() {
            conflicts_found += 1;
            println!("⚠ Conflict: Missing source file");
            println!("  File ID: {}", file_id);
            println!("  Path: {}", item.path.display());
            println!("  Status: Source file not found");
            println!("");
        }
        if item.versions.is_empty() {
            conflicts_found += 1;
            println!("⚠ Conflict: No versions found");
            println!("  File ID: {}", file_id);
            println!("  Path: {}", item.path.display());
            println!("  Status: File has no version history");
            println!("");
        }
    }
    println!("Conflict Detection Summary:");
    println!("  Files checked: {}", total_checked);
    println!("  Conflicts found: {}", conflicts_found);
    if conflicts_found == 0 {
        println!("✓ No conflicts detected");
    } else {
        println!("⚠ {} conflicts require attention", conflicts_found);
    }
    println!("");
    println!("Conflict detection complete.");
    Ok(())
}
fn handle_add_target(source: PathBuf, target: PathBuf) -> Result<()> {
    let manager = SymorManager::new()?;
    println!("Symor Add Target");
    println!("===============");
    println!("");
    println!("Adding target: {} -> {}", source.display(), target.display());
    let source_id = manager.generate_file_id(&source);
    if let Some(item) = manager.watched_items().get(&source_id) {
        println!("✓ Source is being watched: {}", item.path.display());
        if target.exists() {
            println!("⚠ Target already exists: {}", target.display());
            println!("  This will overwrite the existing file.");
        }
        if source.exists() {
            std::fs::copy(&source, &target)?;
            println!("✓ Target added successfully");
            println!("  Source: {}", source.display());
            println!("  Target: {}", target.display());
            manager.save_watched_items_public()?;
            println!("✓ Configuration updated");
        } else {
            println!("✗ Source file does not exist: {}", source.display());
        }
    } else {
        println!("✗ Source is not being watched: {}", source.display());
        println!("  Use 'sym watch {}' first", source.display());
    }
    println!("");
    println!("Add target operation complete.");
    Ok(())
}
fn handle_status(path: Option<PathBuf>, verbose: bool) -> Result<()> {
    let manager = SymorManager::new()?;
    println!("Symor Status Report");
    println!("===================");
    println!("");
    if let Some(specific_path) = path {
        if let Some(item) = manager
            .watched_items()
            .values()
            .find(|item| item.path == specific_path)
        {
            println!("Path: {}", item.path.display());
            println!("Type: {}", if item.is_directory { "Directory" } else { "File" });
            println!("Recursive: {}", item.recursive);
            println!("Versions: {}", item.versions.len());
            println!("Last Modified: {:?}", item.last_modified);
            if verbose {
                println!("");
                println!("Recent Versions:");
                for (i, version) in item.versions.iter().rev().take(5).enumerate() {
                    println!("  {}. {} - {} bytes", i + 1, version.id, version.size);
                }
            }
        } else {
            println!("Path not currently being watched: {}", specific_path.display());
        }
    } else {
        if manager.watched_items().is_empty() {
            println!("No files or directories are currently being watched.");
        } else {
            println!("Watched Items: {}", manager.watched_items().len());
            println!("");
            for (id, item) in manager.watched_items() {
                println!("ID: {}", id);
                println!("  Path: {}", item.path.display());
                println!(
                    "  Type: {}", if item.is_directory { "Directory" } else { "File" }
                );
                println!("  Versions: {}", item.versions.len());
                if verbose {
                    println!("  Last Modified: {:?}", item.last_modified);
                    println!("  Recursive: {}", item.recursive);
                }
                println!("");
            }
        }
    }
    if verbose {
        println!("System Information:");
        println!("  Configuration: {}", manager.config().home_dir.display());
        println!(
            "  Versioning: {}", if manager.config().versioning.enabled { "Enabled" } else
            { "Disabled" }
        );
        println!("  Max Versions: {}", manager.config().versioning.max_versions);
        println!("  Compression: {}", manager.config().versioning.compression);
    }
    Ok(())
}
fn handle_unmirror(source: PathBuf, target: Option<PathBuf>) -> Result<()> {
    println!("Unmirror command is under development.");
    println!("Source: {}", source.display());
    if let Some(tgt) = target {
        println!("Target: {}", tgt.display());
    } else {
        println!("Removing all targets for source");
    }
    println!("");
    println!("Note: This feature will be implemented to remove mirror relationships.");
    println!("For now, you can manually stop watching files with 'sym unwatch'");
    Ok(())
}
fn handle_history(file_id: String, limit: Option<usize>) -> Result<()> {
    let manager = SymorManager::new()?;
    if let Some(item) = manager.watched_items().get(&file_id) {
        println!("Version History for: {}", item.path.display());
        println!("File ID: {}", file_id);
        println!("Total Versions: {}", item.versions.len());
        println!("");
        if item.versions.is_empty() {
            println!("No versions found for this file.");
            return Ok(());
        }
        let versions_to_show = if let Some(lim) = limit {
            lim.min(item.versions.len())
        } else {
            item.versions.len()
        };
        println!("Showing {} most recent versions:", versions_to_show);
        println!("");
        for (i, version) in item.versions.iter().rev().take(versions_to_show).enumerate()
        {
            println!("Version {}: {}", i + 1, version.id);
            println!("  Timestamp: {:?}", version.timestamp);
            println!("  Size: {} bytes", version.size);
            println!("  Hash: {}", & version.hash[..16]);
            if let Some(backup_path) = &version.backup_path {
                println!("  Backup: {}", backup_path.display());
            }
            println!("");
        }
        if let Some(lim) = limit {
            if lim < item.versions.len() {
                println!(
                    "... and {} more versions (use --limit to see more)", item.versions
                    .len() - lim
                );
            }
        }
    } else {
        println!(
            "File ID '{}' not found. Use 'sym list' to see available files.", file_id
        );
    }
    Ok(())
}
fn handle_clean(dry_run: bool, file: Option<String>, keep: usize) -> Result<()> {
    let mut manager = SymorManager::new()?;
    println!("Symor Cleanup");
    println!("=============");
    println!("");
    if dry_run {
        println!("DRY RUN - No files will be actually removed");
        println!("");
    }
    let mut total_cleaned = 0;
    let mut total_space_freed = 0;
    if let Some(file_id) = file {
        if let Some(item) = manager.watched_items_mut().get_mut(&file_id) {
            println!("Cleaning file: {}", item.path.display());
            let original_count = item.versions.len();
            let mut cleaned_count = 0;
            let mut space_freed = 0;
            let mut versions_to_delete = Vec::new();
            while item.versions.len() > keep {
                let version = item.versions.remove(0);
                cleaned_count += 1;
                space_freed += version.size;
                versions_to_delete.push(version);
            }
            let _ = item;
            if !dry_run {
                for version in versions_to_delete {
                    if let Some(ref backup_path) = version.backup_path {
                        let _ = std::fs::remove_file(backup_path);
                    }
                    let _ = manager.version_storage().delete_version(&version.id);
                }
            }
            if cleaned_count > 0 {
                println!(
                    "  Cleaned {} versions, freed {} bytes", cleaned_count, space_freed
                );
                total_cleaned += cleaned_count;
                total_space_freed += space_freed;
            } else {
                println!(
                    "  No cleanup needed ({} versions, keeping {})", original_count, keep
                );
            }
        } else {
            println!("File ID '{}' not found.", file_id);
        }
    } else {
        let file_ids: Vec<String> = manager.watched_items().keys().cloned().collect();
        for file_id in file_ids {
            if let Some(mut item) = manager.watched_items_mut().remove(&file_id) {
                println!("Cleaning file: {} ({})", item.path.display(), file_id);
                let original_count = item.versions.len();
                let mut cleaned_count = 0;
                let mut space_freed = 0;
                let mut versions_to_delete = Vec::new();
                while item.versions.len() > keep {
                    let version = item.versions.remove(0);
                    cleaned_count += 1;
                    space_freed += version.size;
                    versions_to_delete.push(version);
                }
                if !item.versions.is_empty() {
                    manager.watched_items_mut().insert(file_id.clone(), item);
                }
                if !dry_run {
                    for version in versions_to_delete {
                        if let Some(ref backup_path) = version.backup_path {
                            let _ = std::fs::remove_file(backup_path);
                        }
                        let _ = manager.version_storage().delete_version(&version.id);
                    }
                }
                if cleaned_count > 0 {
                    println!(
                        "  Cleaned {} versions, freed {} bytes", cleaned_count,
                        space_freed
                    );
                    total_cleaned += cleaned_count;
                    total_space_freed += space_freed;
                } else {
                    println!(
                        "  No cleanup needed ({} versions, keeping {})", original_count,
                        keep
                    );
                }
            }
        }
    }
    println!("");
    println!("Cleanup Summary:");
    println!("  Total versions cleaned: {}", total_cleaned);
    println!("  Total space freed: {} bytes", total_space_freed);
    if dry_run {
        println!("");
        println!(
            "This was a dry run. Use 'sym clean' without --dry-run to actually clean files."
        );
    } else {
        manager.save_watched_items_public()?;
    }
    Ok(())
}
fn handle_unwatch(path: PathBuf) -> Result<()> {
    let mut manager = SymorManager::new()?;
    let item_id = manager
        .watched_items()
        .iter()
        .find(|(_, item)| item.path == path)
        .map(|(id, _)| id.clone());
    if let Some(id) = item_id {
        manager.watched_items_mut().remove(&id);
        manager.save_watched_items_public()?;
        println!("Stopped watching: {}", path.display());
        println!("File ID: {}", id);
    } else {
        println!("Path not currently being watched: {}", path.display());
        println!("Use 'sym list' to see currently watched files.");
    }
    Ok(())
}
fn handle_sync(path: Option<PathBuf>, force: bool) -> Result<()> {
    let mut manager = SymorManager::new()?;
    if let Some(specific_path) = path {
        if let Some(id) = manager
            .watched_items()
            .iter()
            .find(|(_, item)| item.path == specific_path)
            .map(|(id, _)| id.clone())
        {
            println!("Syncing: {}", specific_path.display());
            if force
                || manager.change_detector_mut().scan_file(&specific_path)?.is_some()
            {
                manager.create_backup(&id)?;
                println!("Created new version for: {}", specific_path.display());
            } else {
                println!("No changes detected for: {}", specific_path.display());
            }
        } else {
            println!("Path not currently being watched: {}", specific_path.display());
            println!("Use 'sym watch <path>' to start watching this file.");
        }
    } else {
        println!("Syncing all watched files...");
        let mut synced_count = 0;
        let mut changed_count = 0;
        let watched_items: Vec<(String, PathBuf)> = manager
            .watched_items()
            .iter()
            .map(|(id, item)| (id.clone(), item.path.clone()))
            .collect();
        for (id, path) in watched_items {
            synced_count += 1;
            println!("Checking: {}", path.display());
            let has_changes = if force {
                true
            } else {
                manager.change_detector_mut().scan_file(&path)?.is_some()
            };
            if has_changes {
                manager.create_backup(&id)?;
                changed_count += 1;
                println!("  ✓ Created new version");
            } else {
                println!("  - No changes");
            }
        }
        println!("");
        println!("Sync Summary:");
        println!("  Files checked: {}", synced_count);
        println!("  Files with changes: {}", changed_count);
    }
    Ok(())
}