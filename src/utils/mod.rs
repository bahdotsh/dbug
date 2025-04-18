// Utility functions for the dbug debugger

use std::path::{Path, PathBuf};
use std::fs;

/// Find all Rust source files in a directory
pub fn find_rust_files(dir: &str) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    find_rust_files_recursive(Path::new(dir), &mut result)?;
    Ok(result)
}

/// Recursively find all Rust source files in a directory
fn find_rust_files_recursive(dir: &Path, result: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", dir.display()));
    }
    
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_dir() {
            find_rust_files_recursive(&path, result)?;
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                result.push(path);
            }
        }
    }
    
    Ok(())
}

/// Check if a path is a Rust file
#[allow(dead_code)]
pub fn is_rust_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        extension == "rs"
    } else {
        false
    }
}

/// Format a path relative to the base directory
#[allow(dead_code)]
pub fn format_path(path: &Path, base_dir: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(base_dir) {
        relative.display().to_string()
    } else {
        path.display().to_string()
    }
}

/// Get a timestamp string
#[allow(dead_code)]
pub fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    now.to_string()
} 