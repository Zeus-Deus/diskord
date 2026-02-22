# Diskord
A fast, intuitive, and universal TUI storage manager for the Omarchy OS (Arch Linux + Hyprland). 

Diskord serves as a visual, 1-click alternative to terminal commands like `df -h`, functioning similarly to the macOS or Windows storage settings, but natively inside a terminal using Rust and Ratatui. 

It fully adapts to your current Omarchy theme automatically, seamlessly integrates with native Linux Trashing, and handles `pkexec` privilege escalations correctly for system caches.

## Features
- **4-Tab Architecture**: Check system junk, developer caches, apps, and use a deep scanner to drill down into large folders.
- **Session Trash**: Delete items with the deep scanner and easily undo/restore them right away from the Session Trash tab before committing to a permanent delete.
- **Theme-aware**: Dynamically parses `~/.config/omarchy/current/theme/colors.toml` to blend in perfectly with your setup.
- **Root/System Safety**: Protects you from accidentally trashing root files outside your home directory, prompting securely if you want to permanently obliterate them.

## Manual Installation

To install Diskord onto your Omarchy system seamlessly, so that it acts like a native, floating GUI app when launched from Walker (`SUPER + SPACE`), follow these exact steps:

### 1. Build and Install the Binary
Ensure you have the Rust toolchain installed. Clone this repository and run:
```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/diskord ~/.local/bin/
```
*(Make sure `~/.local/bin` is in your `$PATH`, which it is by default on Omarchy)*

### 2. Add to Walker (App Launcher)
Create a `.desktop` file so Diskord appears in Walker when you search for it.
```bash
mkdir -p ~/.local/share/applications
cat << 'EOF' > ~/.local/share/applications/diskord.desktop
[Desktop Entry]
Name=Diskord
Comment=Omarchy Storage Manager
GenericName=Storage Settings
Exec=ghostty --class=diskord -e diskord
Icon=drive-harddisk
Type=Application
Terminal=false
Categories=System;Settings;Utility;
Keywords=disk;space;storage;clean;omarchy;
EOF
```
*Note: This configuration explicitly uses `ghostty`. If you use another terminal on Omarchy like `alacritty`, simply swap `ghostty --class=diskord` with `alacritty --class diskord`.*

### 3. Make the Window Float and Center
By default, your terminal might tile. To make Diskord float perfectly in the center of your screen like a native app, add a window rule to your Hyprland configuration.

Open `~/.config/hypr/windows.conf` (or `hyprland.conf`) and add the following line:
```ini
# Diskord Storage Manager
windowrule = float on, center on, size 1000 700, match:class ^(diskord)$
```

## AUR Package Notice
If you are reading this and installing via the AUR (e.g., `yay -S diskord`), the `.desktop` file and binary are likely placed into `/usr/share/applications/` and `/usr/bin/` automatically for you! You may only need to add the Hyprland window rule mentioned in Step 3 to ensure it floats. 

## Keybindings
- `h` / `l` or `Tab`: Switch Tabs / Navigate in and out of folders in Deep Scanner
- `j` / `k` or `Up` / `Down`: Navigate lists
- `Space`: Select items
- `Enter`: Execute Clean / Move to Trash
- `u`: Undo Trashing (in Session Trash Tab)
- `q` / `Esc`: Quit
