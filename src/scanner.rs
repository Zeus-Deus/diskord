use jwalk::WalkDir;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

pub fn scan_directory(path: &Path) -> Vec<DirEntry> {
    if !path.exists() {
        return Vec::new();
    }

    let mut direct_children = HashMap::new();
    let root_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for entry in WalkDir::new(&root_path).skip_hidden(false) {
        if let Ok(entry) = entry
            && let Ok(metadata) = entry.metadata()
                && metadata.is_file() {
                    let file_size = metadata.len();

                    let file_path = entry.path();
                    if let Ok(rel) = file_path.strip_prefix(&root_path) {
                        if let Some(first_component) = rel.components().next() {
                            let mut child_path = root_path.clone();
                            child_path.push(first_component);

                            let entry_size = direct_children.entry(child_path).or_insert(0);
                            *entry_size += file_size;
                        } else {
                            // The file is directly in the root_path
                            let entry_size = direct_children.entry(file_path.clone()).or_insert(0);
                            *entry_size += file_size;
                        }
                    }
                }
    }

    let mut results = Vec::new();
    for (child_path, size) in direct_children {
        let is_dir = child_path.is_dir();
        let name = child_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        results.push(DirEntry {
            path: child_path,
            name,
            size,
            is_dir,
        });
    }

    // Sort by size descending
    results.sort_by(|a, b| b.size.cmp(&a.size));

    // Only return top 50
    results.into_iter().take(50).collect()
}
