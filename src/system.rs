use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::Disks;

pub struct DiskUsage {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub mount_point: String,
}

#[derive(Clone, Debug)]
pub struct TrashedItem {
    pub original_path: PathBuf,
    pub trash_file_path: PathBuf,
    pub trash_info_path: PathBuf,
    pub is_root: bool,
}

pub fn move_to_trash(original_path: &Path) -> Result<TrashedItem, String> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    let is_root = !original_path.starts_with(&home_dir);

    if is_root {
        // We do permanent deletion immediately if it's root
        let status = Command::new("pkexec")
            .arg("rm")
            .arg("-rf")
            .arg(original_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => Ok(TrashedItem {
                original_path: original_path.to_path_buf(),
                trash_file_path: PathBuf::new(),
                trash_info_path: PathBuf::new(),
                is_root: true,
            }),
            _ => Err("Failed to pkexec rm -rf".to_string()),
        }
    } else {
        let mut trash_dir =
            dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
        trash_dir.push("Trash");

        let files_dir = trash_dir.join("files");
        let info_dir = trash_dir.join("info");

        std::fs::create_dir_all(&files_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&info_dir).map_err(|e| e.to_string())?;

        let file_name = original_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let mut safe_name = file_name.clone();
        let mut counter = 1;

        while files_dir.join(&safe_name).exists()
            || info_dir.join(format!("{}.trashinfo", safe_name)).exists()
        {
            safe_name = format!("{}_{}", file_name, counter);
            counter += 1;
        }

        let trash_file_path = files_dir.join(&safe_name);
        let trash_info_path = info_dir.join(format!("{}.trashinfo", safe_name));

        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        let info_content = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            original_path.display(),
            date_str
        );

        std::fs::write(&trash_info_path, info_content).map_err(|e| e.to_string())?;

        if let Err(e) = std::fs::rename(original_path, &trash_file_path) {
            let _ = std::fs::remove_file(&trash_info_path);
            return Err(format!("Failed to move to trash: {}", e));
        }

        Ok(TrashedItem {
            original_path: original_path.to_path_buf(),
            trash_file_path,
            trash_info_path,
            is_root: false,
        })
    }
}

pub fn restore_trash_item(item: &TrashedItem) -> Result<(), String> {
    if item.is_root {
        return Err("Cannot restore permanently deleted root files".to_string());
    }

    if let Some(parent) = item.original_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    std::fs::rename(&item.trash_file_path, &item.original_path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&item.trash_info_path);

    Ok(())
}

pub fn perm_delete_trash_item(item: &TrashedItem) -> Result<(), String> {
    if item.is_root {
        return Ok(()); // already gone
    }

    if item.trash_file_path.is_dir() {
        let _ = std::fs::remove_dir_all(&item.trash_file_path);
    } else {
        let _ = std::fs::remove_file(&item.trash_file_path);
    }
    let _ = std::fs::remove_file(&item.trash_info_path);

    Ok(())
}

pub fn get_disks() -> Vec<DiskUsage> {
    let disks = Disks::new_with_refreshed_list();
    let mut result = Vec::new();

    for disk in &disks {
        let mount_point = disk.mount_point().to_string_lossy().to_string();

        if mount_point == "/" || mount_point == "/home" {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            result.push(DiskUsage {
                name: disk.name().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                mount_point,
            });
        }
    }

    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result
}

// System Junk Analyzers
pub fn get_pacman_cache_size() -> u64 {
    // pacman cache usually lives here on Arch
    get_dir_size_with_du("/var/cache/pacman/pkg")
}

pub fn get_yay_cache_size() -> u64 {
    let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("~/.cache"));
    path.push("yay");
    get_dir_size_with_du(&path.to_string_lossy())
}

pub fn get_journal_size() -> u64 {
    get_dir_size_with_du("/var/log/journal")
}

pub fn get_trash_size() -> u64 {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    path.push("Trash");
    get_dir_size_with_du(&path.to_string_lossy())
}

pub fn get_orphaned_packages() -> (u64, usize) {
    let output = Command::new("pacman")
        .arg("-Qtdq")
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::os::unix::process::ExitStatusExt::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.lines().filter(|l| !l.trim().is_empty()).count();

    if count == 0 {
        return (0, 0);
    }

    // Exact size measurement by calling pacman -Qi and awk
    let size_output = Command::new("bash")
        .arg("-c")
        .arg("pacman -Qi $(pacman -Qtdq) | awk '/Installed Size/ {print $4, $5}'")
        .output();

    let mut total_bytes: u64 = 0;
    if let Ok(out) = size_output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let val: f64 = parts[0].parse().unwrap_or(0.0);
                let unit = parts[1];
                let bytes = match unit {
                    "KiB" => val * 1024.0,
                    "MiB" => val * 1024.0 * 1024.0,
                    "GiB" => val * 1024.0 * 1024.0 * 1024.0,
                    _ => val,
                };
                total_bytes += bytes as u64;
            }
        }
    }

    (total_bytes, count)
}

pub fn clean_orphaned_packages() -> bool {
    let status = Command::new("pkexec")
        .arg("bash")
        .arg("-c")
        .arg("pacman -Rns $(pacman -Qtdq) --noconfirm")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    std::thread::sleep(std::time::Duration::from_millis(50));
    status.map(|s| s.success()).unwrap_or(false)
}

// Developer Tools Detectors

pub fn get_docker_size() -> u64 {
    // Check if docker is running and get size
    let output = Command::new("docker")
        .arg("system")
        .arg("df")
        .arg("--format")
        .arg("{{.Size}}") // Unfortunately format {{.Size}} gives human readable like "1.2GB"
        .output();

    if let Ok(out) = output
        && out.status.success() {
            // For simplicity, we just use du on /var/lib/docker if we have permission,
            // otherwise we'll parse the human readable string if possible, or just return 0 if permission denied.
            return get_dir_size_with_du("/var/lib/docker");
        }
    0
}

pub fn clean_docker() -> bool {
    // Clean all unused containers, networks, images (both dangling and unreferenced), and optionally, volumes.
    let status = Command::new("pkexec")
        .arg("bash")
        .arg("-c")
        .arg("docker system prune -a --volumes -f")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    std::thread::sleep(std::time::Duration::from_millis(50));
    status.map(|s| s.success()).unwrap_or(false)
}

pub fn get_cargo_cache_size() -> u64 {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    path.push(".cargo");
    let mut total = 0;

    let mut registry = path.clone();
    registry.push("registry");
    total += get_dir_size_with_du(&registry.to_string_lossy());

    let mut git = path.clone();
    git.push("git");
    total += get_dir_size_with_du(&git.to_string_lossy());

    total
}

pub fn clean_cargo_cache() -> bool {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    path.push(".cargo");

    let mut registry = path.clone();
    registry.push("registry");
    let _ = std::fs::remove_dir_all(&registry);
    let _ = std::fs::create_dir_all(&registry);

    let mut git = path.clone();
    git.push("git");
    let _ = std::fs::remove_dir_all(&git);
    let _ = std::fs::create_dir_all(&git);

    std::thread::sleep(std::time::Duration::from_millis(50));
    true
}

pub fn get_npm_cache_size() -> u64 {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    path.push(".npm");
    path.push("_cacache");
    get_dir_size_with_du(&path.to_string_lossy())
}

pub fn clean_npm_cache() -> bool {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    path.push(".npm");
    path.push("_cacache");

    let _ = std::fs::remove_dir_all(&path);
    let _ = std::fs::create_dir_all(&path);

    std::thread::sleep(std::time::Duration::from_millis(50));
    true
}
pub fn get_steam_size() -> u64 {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    path.push("Steam");
    get_dir_size_with_du(&path.to_string_lossy())
}

pub fn get_flatpak_size() -> u64 {
    let mut total = get_dir_size_with_du("/var/lib/flatpak/app");
    let mut user_path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    user_path.push("flatpak/app");
    total += get_dir_size_with_du(&user_path.to_string_lossy());
    total
}

pub fn clean_pacman_cache() -> bool {
    // Using pkexec to run 'bash -c "yes | pacman -Scc"' ensures that
    // ALL prompts are automatically answered 'yes'.
    let status = Command::new("pkexec")
        .arg("bash")
        .arg("-c")
        .arg("yes | pacman -Scc")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Sleep briefly to ensure filesystem stats have updated before we check du -sb again
    std::thread::sleep(std::time::Duration::from_millis(50));

    status.map(|s| s.success()).unwrap_or(false)
}

pub fn clean_yay_cache() -> bool {
    let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("~/.cache"));
    path.push("yay");

    if path.exists() {
        // Just remove the directory contents to be safe
        let result = std::fs::remove_dir_all(&path);
        // recreate empty dir
        let _ = std::fs::create_dir_all(&path);

        // Wait a tiny bit for FS to catch up before we recalculate
        std::thread::sleep(std::time::Duration::from_millis(50));

        result.is_ok()
    } else {
        true
    }
}

pub fn vacuum_journal() -> bool {
    let status = Command::new("pkexec")
        .arg("journalctl")
        .arg("--vacuum-time=14d")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    std::thread::sleep(std::time::Duration::from_millis(50));

    status.map(|s| s.success()).unwrap_or(false)
}

pub fn empty_trash() -> bool {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"));
    path.push("Trash");

    if path.exists() {
        std::fs::remove_dir_all(&path).is_ok() && std::fs::create_dir(&path).is_ok()
    } else {
        true
    }
}

// Helper: Use 'du' since normal user can't always traverse root directories (/var/cache, /var/log)
fn get_dir_size_with_du(path: &str) -> u64 {
    if !std::path::Path::new(path).exists() {
        return 0;
    }

    let output = Command::new("du").arg("-sb").arg(path).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(first_word) = stdout.split_whitespace().next()
            && let Ok(size) = first_word.parse::<u64>() {
                return size;
            }
    }

    // Fallback: Just return 0 if 'du' fails (e.g. permission denied)
    // To be perfectly accurate we'd run pkexec, but we don't want password prompts just on startup.
    // Instead we rely on the fact that these are somewhat readable on Arch by default,
    // or we'll accept displaying 0 until elevated.
    0
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
