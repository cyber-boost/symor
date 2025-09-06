# Symor Commands Reference

## Main Commands
sym mirror <source> <target...>
sym list [--detailed]
sym info <path>
sym install [--force]
sym watch <path> [--recursive]
sym restore <file_id> <version_id> <target>
sym status [path] [--verbose]
sym unmirror <source> [target]
sym history <file_id> [--limit <n>]
sym clean [--dry-run] [--file <id>] [--keep <n>]
sym unwatch <path>
sym sync [path] [--force]
sym stats [--detailed] [--period <seconds>]
sym tui [--refresh-rate <seconds>]
sym check [path]
sym conflicts
sym add-target <source> <target>
sym settings <subcommand>
sym rip [--keep-data]

## Status & Monitoring Commands
sym status [path] [--verbose]
sym stats [--detailed] [--period <seconds>]
sym check [path]
sym conflicts

## Mirror Management Commands
sym unmirror <source> [target]
sym sync [path] [--force]
sym add-target <source> <target>

## Watch Management Commands
sym unwatch <path>

## Version History Commands
sym history <file_id> [--limit <count>]

## Maintenance Commands
sym clean [--dry-run] [--file <file_id>] [--keep <count>]
sym tui [--refresh-rate <seconds>]

## Settings Subcommands
sym settings show
sym settings versioning [--enabled <bool>] [--max-versions <num>] [--compression <level>]
sym settings linking [--link-type <type>] [--preserve-permissions <bool>]
sym settings home <path>

## Command Descriptions

### Core Commands
- `sym mirror` - Mirror a file to many targets with real-time synchronization
- `sym list` - List all watched files, directories, and their version history
- `sym info` - Display detailed metadata and status information for files/directories
- `sym install` - Install sym binary to system PATH for global access
- `sym watch` - Add file/directory to version control monitoring
- `sym restore` - Restore file from version history to specified location
- `sym settings` - Manage symor settings and configuration
- `sym rip` - Uninstall sym and optionally remove all data

### Status & Monitoring
- `sym status` - Show current synchronization status and pending operations
- `sym stats` - Display performance statistics and system metrics
- `sym check` - Verify integrity of mirrors and links
- `sym conflicts` - List current conflicts needing resolution

### Mirror Management
- `sym unmirror` - Remove mirror relationships for a source file
- `sym sync` - Manually trigger synchronization for watched files
- `sym add-target` - Add new mirror target to existing source

### New Command Options

#### Unmirror Command
- `sym unmirror <source>` - Remove all mirror relationships for source file
- `sym unmirror <source> <target>` - Remove specific target from mirror relationship

### Watch Management
- `sym unwatch` - Stop watching a file or directory

### Version History
- `sym history` - Display version history for a watched file

### Maintenance & Cleanup
- `sym clean` - Clean up old versions and temporary files

## Settings Subcommand Descriptions
- `sym settings show` - Show current settings
- `sym settings versioning` - Set versioning options (enabled, max-versions, compression)
- `sym settings linking` - Set linking options (link-type, preserve-permissions)
- `sym settings home` - Set custom home directory path
- `sym settings init` - Initialize/reset directory structure and permissions

## New Command Options

### Status Command
- `sym status [path]` - Show status for specific path or all watched items
- `sym status --verbose` - Include detailed file information and system metrics

### History Command
- `sym history <file_id>` - Show complete version history for a file
- `sym history <file_id> --limit 5` - Show only the 5 most recent versions

### Clean Command
- `sym clean` - Clean all watched files (keep 10 versions each)
- `sym clean --dry-run` - Preview what would be cleaned
- `sym clean --file <id>` - Clean only specific file
- `sym clean --keep 20` - Keep 20 versions per file instead of default 10

### Sync Command
- `sym sync` - Sync all watched files
- `sym sync /path/to/file` - Sync specific file
- `sym sync --force` - Force sync even if no changes detected

### Stats Command
- `sym stats` - Show current performance statistics
- `sym stats --detailed` - Include system information
- `sym stats --period 60` - Show metrics for last 60 seconds

### TUI Command
- `sym tui` - Start interactive interface (default 2s refresh)
- `sym tui --refresh-rate 5` - Set refresh rate to 5 seconds

## Usage Examples

### Version Control Workflow
```bash
sym watch /path/to/important.txt
sym status --verbose
sym history <file_id> --limit 3
sym restore <file_id> <version_id> /path/to/restored.txt
```

### Maintenance Workflow
```bash
sym clean --dry-run                    # Preview cleanup
sym clean --keep 5                    # Aggressive cleanup
sym stats --detailed                  # Monitor performance
```

### Sync Management
```bash
sym sync --force                      # Force sync all
sym unmirror source.txt target.txt    # Remove mirror relationship
sym unwatch /path/to/file.txt         # Stop watching
sym status                            # Check current state
```

## Global Options
- `-v, --verbose` - Turn on verbose logging (multiple -v increase verbosity)
- `-h, --help` - Print help
- `-V, --version` - Print version

## Key Features

### Core Functionality
- **Real-time File Monitoring**: Automatic change detection and versioning
- **Compressed Storage**: Gzip compression for efficient version storage
- **Atomic Operations**: Safe file operations with rollback capabilities
- **Version Control**: Complete file history with restore capabilities

### Advanced Features
- **Performance Monitoring**: Built-in metrics and statistics tracking
- **Interactive TUI**: Terminal-based user interface for advanced operations
- **Maintenance Tools**: Automated cleanup and optimization
- **Status Monitoring**: Real-time sync status and system health

### System Features
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Thread-Safe**: Concurrent operations with proper synchronization
- **Configurable**: Extensive customization options
- **Production-Ready**: Comprehensive error handling and logging
