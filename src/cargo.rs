//! Cargo integration for the dbug debugger
//!
//! This module provides functionality for integrating with Cargo,
//! including building projects with instrumentation and running cargo commands.

use crate::errors::{DbugError, DbugResult};
use std::process::{Command, Stdio};

/// Run a Cargo command
pub fn run_cargo_command(project_path: &str, subcommand: &str, args: &[&str]) -> DbugResult<()> {
    println!("Running cargo {} for {}", subcommand, project_path);

    let status = Command::new("cargo")
        .current_dir(project_path)
        .arg(subcommand)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| DbugError::CompilationError(format!("Failed to execute cargo: {}", e)))?;

    if !status.success() {
        return Err(DbugError::CompilationError(format!(
            "Cargo {} failed with exit code: {}",
            subcommand,
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Build a project with instrumentation
pub fn build_with_instrumentation(project_path: &str, release: bool) -> DbugResult<()> {
    // Add custom environment variables to signal to proc macros that we're in debug mode
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_path)
        .arg("build")
        .env("DBUG_BUILD", "1");

    if release {
        cmd.arg("--release");
    }

    let status = cmd
        .status()
        .map_err(|e| DbugError::CompilationError(format!("Failed to build project: {}", e)))?;

    if !status.success() {
        return Err(DbugError::CompilationError(format!(
            "Build failed with exit code: {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Clean a project
pub fn clean_project(project_path: &str) -> DbugResult<()> {
    run_cargo_command(project_path, "clean", &[])
}

/// Check if a path contains a valid Cargo project
pub fn is_cargo_project(project_path: &str) -> bool {
    let cargo_toml_path = std::path::Path::new(project_path).join("Cargo.toml");
    cargo_toml_path.exists()
}

/// Get the name of a Cargo project
pub fn get_project_name(project_path: &str) -> DbugResult<String> {
    let cargo_toml_path = std::path::Path::new(project_path).join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Err(DbugError::NotARustProject);
    }

    let content = std::fs::read_to_string(cargo_toml_path)
        .map_err(|e| DbugError::CompilationError(format!("Failed to read Cargo.toml: {}", e)))?;

    // Simple parsing to extract the package name
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            if let Some(name) = line.split('=').nth(1) {
                return Ok(name.trim().trim_matches('"').to_string());
            }
        }
    }

    // If we couldn't find the name, use the directory name
    let path = std::path::Path::new(project_path);
    if let Some(dir_name) = path.file_name() {
        if let Some(name) = dir_name.to_str() {
            return Ok(name.to_string());
        }
    }

    Err(DbugError::CompilationError(
        "Could not determine project name".to_string(),
    ))
}
