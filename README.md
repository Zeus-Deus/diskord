# Diskord

<p align="center">
  <img src="screenshots/main.png" alt="Diskord Storage Manager running on Omarchy with Hyprland floating rules" />
</p>

A fast, intuitive, and universal TUI storage manager for Omarchy (Arch Linux + Hyprland). 

Diskord serves as a visual, 1-click alternative to terminal commands like `df -h`, functioning similarly to the macOS or Windows storage settings, but natively inside a terminal using Rust and Ratatui. 

It fully adapts to your current Omarchy theme automatically, seamlessly integrates with native Linux Trashing, and handles `pkexec` privilege escalations correctly for system caches.

## Features
- **4-Tab Architecture**: Check system junk, developer caches, apps, and use a deep scanner to drill down into large folders.
- **Session Trash**: Delete items with the deep scanner and easily undo/restore them right away from the Session Trash tab before committing to a permanent delete.
- **Theme-aware**: Dynamically parses `~/.config/omarchy/current/theme/colors.toml` to blend in perfectly with your setup.
- **Root/System Safety**: Protects you from accidentally trashing root files outside your home directory, prompting securely if you want to permanently obliterate them.

## Installation (Recommended)

Diskord is available on the Arch User Repository (AUR). You can install it using `yay` or your preferred AUR helper.

```bash
yay -S diskord
```

### Make the Window Float and Center
By default, your terminal might tile. To make Diskord float perfectly in the center of your screen like a native app when launched from Walker (`SUPER + SPACE`), add a window rule to your Hyprland configuration.

Open `~/.config/hypr/windows.conf` (or `hyprland.conf`) and add the following line:
```ini
# Diskord Storage Manager
windowrule = float on, center on, size 1000 700, match:initial_class org.omarchy.Diskord
```

---

## Manual Installation (From Source)

If you prefer to build Diskord yourself, follow these steps:

### 1. Build and Install the Binary
Ensure you have the Rust toolchain installed. Clone this repository and run:
```bash
cargo build --release
mkdir -p ~/.local/bin ~/.local/share/icons/hicolor/scalable/apps/
cp target/release/diskord ~/.local/bin/
cp assets/diskord.svg ~/.local/share/icons/hicolor/scalable/apps/
```

### 2. Add to Walker (App Launcher)
Copy the included `.desktop` file to your local applications folder:
```bash
mkdir -p ~/.local/share/applications
cp diskord.desktop ~/.local/share/applications/
```

Then, follow the **Make the Window Float and Center** instructions above.

## Keybindings
- `h` / `l` or `Tab`: Switch Tabs / Navigate in and out of folders in Deep Scanner
- `j` / `k` or `Up` / `Down`: Navigate lists
- `Space`: Select items
- `Enter`: Execute Clean / Move to Trash
- `u`: Undo Trashing (in Session Trash Tab)
- `q` / `Esc`: Quit
