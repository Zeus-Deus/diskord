use std::path::PathBuf;
use std::process::Command;
use sysinfo::Disks;

pub struct DiskUsage {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub mount_point: String,
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

// System Junk Cleaners
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
        if let Some(first_word) = stdout.split_whitespace().next() {
            if let Ok(size) = first_word.parse::<u64>() {
                return size;
            }
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
