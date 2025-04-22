// Utility functions for the dbug debugger

use std::path::{Path, PathBuf};
use std::fs;
use std::process::exit;

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

/// Find the correct executable path for a project, considering workspace structure
pub fn find_executable_path(project_path: &str, project_name: &str, release: bool) -> PathBuf {
    let target_dir = if release { "release" } else { "debug" };
    
    // Convert to absolute path for consistency
    let abs_project_path = if Path::new(project_path).is_absolute() {
        PathBuf::from(project_path)
    } else {
        // Convert relative path to absolute
        std::env::current_dir()
            .map(|p| p.join(project_path))
            .unwrap_or_else(|_| PathBuf::from(project_path))
    };
    
    // List of possible locations for the executable, in order of preference:
    let possible_paths = vec![
        // 1. Workspace root's target directory (most common for workspace members)
        abs_project_path.join("../..").join("target").join(target_dir).join(project_name),
        
        // 2. Parent directory's target directory (alternative workspace structure)
        abs_project_path.join("..").join("target").join(target_dir).join(project_name),
        
        // 3. Project's own target directory (standalone projects)
        abs_project_path.join("target").join(target_dir).join(project_name)
    ];
    
    // Check each path and return the first one that exists
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }
    
    // If none of the paths exist yet, return the most likely one based on workspace detection
    if is_workspace_member(&abs_project_path) {
        // For workspace members, prefer the workspace root target
        possible_paths[0].clone()
    } else {
        // For standalone projects, use the project's own target
        possible_paths[2].clone()
    }
}

/// Check if a project is part of a workspace
fn is_workspace_member(project_path: &Path) -> bool {
    // Check if this directory contains a Cargo.toml that references a workspace
    let manifest_path = project_path.join("Cargo.toml");
    if manifest_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            // If this Cargo.toml defines a workspace, it's a workspace root
            if content.contains("[workspace]") {
                return true;
            }
            
            // If it has a workspace key, it's a workspace member
            if content.contains("workspace = ") {
                return true;
            }
        }
    }
    
    // Check if any parent directory contains a Cargo.toml with a workspace definition
    let parent = project_path.parent();
    if let Some(parent_path) = parent {
        let parent_cargo = parent_path.join("Cargo.toml");
        if parent_cargo.exists() {
            if let Ok(content) = std::fs::read_to_string(&parent_cargo) {
                if content.contains("[workspace]") {
                    return true;
                }
            }
        }
    }
    
    false
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