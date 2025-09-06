# Symor
![Symor Logo](https://github.com/cyber-boost/symor/blob/main/symor.png?raw=true)


**Real-time file mirroring and version control for the modern developer**

Symor is a powerful command-line tool that provides real-time file mirroring with built-in version control, performance monitoring, and an intuitive terminal interface. Keep your important files synchronized across multiple locations while maintaining complete version history.

## ✨ Key Features

- **🔄 Real-time File Mirroring** - Mirror files to multiple targets with instant synchronization
- **📚 Version Control** - Complete file history with compressed storage and restore capabilities
- **👀 File Watching** - Automatic change detection and versioning for monitored files
- **📊 Performance Monitoring** - Built-in metrics, statistics, and system health monitoring
- **🖥️ Interactive TUI** - Terminal-based user interface for advanced operations (W.I.P.)
- **🧹 Smart Cleanup** - Automated maintenance with configurable retention policies
- **⚡ Thread-Safe** - Concurrent operations with proper synchronization
- **🌍 Cross-Platform** - Works on Linux, macOS, and Windows

## 🚀 Quick Start

### Installation

```bash
# Install symor binary to your system PATH
sym install
```

### Basic Usage

```bash
# Mirror a file to multiple locations
sym mirror source.txt /backup/source.txt /sync/source.txt

# Start watching a file for version control
sym watch important-config.json

# Check status of all operations
sym status

# View interactive dashboard
sym tui
```

## 📖 Usage Examples

### File Mirroring Workflow

```bash
# Create mirrors for a configuration file
sym mirror ~/.bashrc ~/backups/bashrc ~/cloud/bashrc

# Add another mirror target later
sym add-target ~/.bashrc ~/external-drive/bashrc

# Check sync status
sym status ~/.bashrc --verbose

# Force synchronization
sym sync ~/.bashrc --force

# Remove a mirror relationship
sym unmirror ~/.bashrc ~/cloud/bashrc
```

### Version Control Workflow

```bash
# Start tracking a file
sym watch project/config.yaml

# View file history
sym history <file_id>

# Restore from a previous version
sym restore <file_id> <version_id> ./restored-config.yaml

# Check for conflicts
sym conflicts
```

### Maintenance & Monitoring

```bash
# View system statistics
sym stats --detailed

# Clean up old versions (dry run first)
sym clean --dry-run
sym clean --keep 5

# Verify system integrity
sym check

# View all tracked files
sym list --detailed
```

## 📋 Command Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `sym mirror <source> <target...>` | Mirror a file to multiple targets |
| `sym watch <path> [--recursive]` | Add file/directory to version control |
| `sym status [path] [--verbose]` | Show synchronization status |
| `sym list [--detailed]` | List all watched files and history |
| `sym sync [path] [--force]` | Manually trigger synchronization |

### Version Control

| Command | Description |
|---------|-------------|
| `sym history <file_id> [--limit <n>]` | Display version history |
| `sym restore <file_id> <version_id> <target>` | Restore from version history |
| `sym conflicts` | List current conflicts |

### Management

| Command | Description |
|---------|-------------|
| `sym unmirror <source> [target]` | Remove mirror relationships |
| `sym unwatch <path>` | Stop watching a file/directory |
| `sym add-target <source> <target>` | Add new mirror target |
| `sym clean [--dry-run] [--keep <n>]` | Clean up old versions |

### Monitoring & Interface

| Command | Description |
|---------|-------------|
| `sym tui [--refresh-rate <seconds>]` | Interactive terminal interface |
| `sym stats [--detailed] [--period <seconds>]` | Performance statistics |
| `sym check [path]` | Verify integrity |
| `sym info <path>` | Detailed file information |

## ⚙️ Configuration

### Settings Management

```bash
# View current settings
sym settings show

# Configure versioning
sym settings versioning --enabled true --max-versions 50 --compression 6

# Configure linking behavior
sym settings linking --link-type hard --preserve-permissions true

# Set custom home directory
sym settings home /custom/symor/path
```

### Configuration Options

- **Versioning**: Control version retention, compression levels
- **Linking**: Choose between hard/soft links, permission handling
- **Storage**: Custom home directory, cleanup policies
- **Monitoring**: Refresh rates, notification settings

## 🛠️ Advanced Features

### Interactive TUI

Launch the terminal user interface for real-time monitoring:

```bash
sym tui --refresh-rate 1
```

Features include:
- Live status updates
- Performance metrics
- File tree navigation
- Conflict resolution
- System health monitoring

### Performance Monitoring

Track system performance and file operations:

```bash
# Current statistics
sym stats

# Detailed system information
sym stats --detailed

# Metrics for specific time period
sym stats --period 300  # Last 5 minutes
```

### Conflict Resolution

Handle synchronization conflicts:

```bash
# List current conflicts
sym conflicts

# Check system integrity
sym check

# Manual sync with force flag
sym sync --force
```

## 🔧 System Requirements

- **Operating Systems**: Linux, macOS, Windows
- **Storage**: Varies based on file size and version retention
- **Memory**: Minimal overhead for monitoring operations
- **Permissions**: Read/write access to source and target locations

## 📊 Use Cases

### Development Workflows
- Sync configuration files across development environments
- Maintain version history of critical project files
- Backup important scripts and configurations

### System Administration
- Mirror configuration files to multiple servers
- Version control for system configurations
- Automated backup with cleanup policies

### Content Management
- Sync documents across multiple locations
- Version tracking for important files
- Conflict detection and resolution

## 🚨 Important Notes

- **Atomic Operations**: All file operations are atomic with rollback capabilities
- **Thread Safety**: Designed for concurrent operations
- **Compression**: Uses gzip compression for efficient storage
- **Cross-Platform**: Consistent behavior across operating systems

## 📈 Performance

Symor is designed for efficiency:
- **Low Memory Footprint**: Minimal resource usage during monitoring
- **Compressed Storage**: Efficient version storage with gzip
- **Optimized I/O**: Smart change detection and batched operations
- **Concurrent Safe**: Thread-safe operations for high-performance environments

## 🗑️ Uninstallation

```bash
# Uninstall symor (keep data)
sym rip

# Uninstall and remove all data
sym rip --keep-data false
```

## 📚 Getting Help

```bash
# General help
sym --help

# Command-specific help
sym mirror --help
sym settings --help

# Version information
sym --version

# Verbose output for debugging
sym status -v
sym sync -vv  # Extra verbose
```

---

**Symor** - Keeping your files in sync, one mirror at a time. 🪞