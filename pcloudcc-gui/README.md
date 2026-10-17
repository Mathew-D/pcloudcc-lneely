# pCloudCC GUI

A lightweight Rust desktop GUI for managing the pCloudCC command-line client.

## Features

- **Process Management**: Start and stop the pCloudCC daemon
- **Status Monitoring**: View daemon status and pending transfers
- **Configuration**: Configure username, mount point, cache size, and logging
- **Sync Folder Management**: List, pause, and resume sync folders
- **Crypto Support**: Unlock and lock crypto folders
- **Background Mode**: Minimize to background (close to hide, Ctrl+Shift+P to restore)

## Running on Linux

The GUI requires either Wayland or X11 display server. If you're on Wayland and encounter errors, you can force X11 backend:

```bash
# Force X11 backend
export WINIT_UNIX_BACKEND=x11
./target/release/pcloudcc-gui

# Or use the provided script
./run.sh
```

If you still encounter issues, install Wayland libraries:
```bash
# Arch Linux
sudo pacman -S wayland

# Debian/Ubuntu
sudo apt install libwayland-dev
```

## Requirements

- Rust 1.70 or later
- pCloudCC binary (built from the parent directory)
- Linux (with X11 or Wayland support)

## Building

```bash
cd pcloudcc-gui
cargo build --release
```

The compiled binary will be at `target/release/pcloudcc-gui`.

## Running

```bash
./target/release/pcloudcc-gui
```

## Usage

1. **Configuration Tab**: Set up your pCloud credentials and preferences
   - Enter your pCloud username
   - Optionally set a custom mount point
   - Configure cache size (default: 5GB)
   - Set log path and level
   - Enable password saving if desired

2. **Status Tab**: Control the daemon
   - Click "Start pCloudCC" to launch the daemon
   - Click "Refresh Status" to view current status
   - Click "Check Pending" to see pending transfers
   - Click "Stop pCloudCC" to gracefully shut down

3. **Sync Folders Tab**: Manage synchronization
   - List configured sync folders
   - Pause/resume syncing

4. **Crypto Tab**: Manage encrypted folders
   - Enter crypto password to unlock
   - Lock crypto folder when done

5. **Background Mode**: Minimize to background
   - Click "Minimize to Background" or close the window to hide
   - Press `Ctrl+Shift+P` to restore the window
   - Click "Quit" to fully exit the application

## Architecture

The GUI manages the pCloudCC binary as a separate process rather than porting its C++ code to Rust. This approach:

- Preserves the low resource consumption of the CLI client (~31 MiB vs ~343 MiB for official GUI)
- Leverages the existing daemon mode and command interface
- Provides a lightweight GUI wrapper without reimplementing the sync engine

Communication with the daemon uses the `pcloudcc -k` command interface with piped input.

## Dependencies

- `eframe` / `egui`: Immediate mode GUI framework
- `tokio`: Async runtime for process management
- `nix`: Unix signal handling for graceful shutdown
- `serde` / `serde_json`: Configuration serialization
- `dirs`: Cross-platform directory paths

## License

Same license as the parent pCloudCC project.
