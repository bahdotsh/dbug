use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("dbug").unwrap();
    cmd.arg("version");
    cmd.assert().success().stdout(predicate::str::contains("dbug v0.1.0"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("dbug").unwrap();
    cmd.arg("help");
    cmd.assert().success().stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn test_build_nonexistent_project() {
    let mut cmd = Command::cargo_bin("dbug").unwrap();
    cmd.args(["build", "/tmp/nonexistent_project"]);
    cmd.assert().failure();
}

#[test]
fn test_build_example_project() {
    // This test assumes that the examples/simple_app directory exists
    let project_path = Path::new("examples").join("simple_app");
    
    if !project_path.exists() {
        panic!("Example project not found: {}", project_path.display());
    }
    
    let mut cmd = Command::cargo_bin("dbug").unwrap();
    cmd.args(["build", &project_path.to_string_lossy()]);
    cmd.assert().success();
}

// Disabled for now because it requires proper implementation of the debugger
#[ignore]
#[test]
fn test_debug_flow() {
    // Set up a temporary directory with a simple Rust project
    let temp_dir = tempdir().unwrap();
    let project_dir = temp_dir.path().join("test_project");
    fs::create_dir(&project_dir).unwrap();
    
    // Create Cargo.toml
    fs::write(
        project_dir.join("Cargo.toml"),
        r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
dbug = { path = "../.." }
        "#,
    ).unwrap();
    
    // Create src directory
    fs::create_dir(project_dir.join("src")).unwrap();
    
    // Create main.rs with a debug point
    fs::write(
        project_dir.join("src").join("main.rs"),
        r#"
use dbug::prelude::*;

#[dbug]
fn main() {
    let x = 42;
    dbug::break_here!();
    println!("x = {}", x);
}
        "#,
    ).unwrap();
    
    // Build the project with dbug
    let mut build_cmd = Command::cargo_bin("dbug").unwrap();
    build_cmd.args(["build", &project_dir.to_string_lossy()]);
    build_cmd.assert().success();
    
    // Run the project and verify it hits the breakpoint
    let mut run_cmd = Command::cargo_bin("dbug").unwrap();
    run_cmd.args(["run", &project_dir.to_string_lossy()]);
    run_cmd.assert().success().stdout(predicate::str::contains("Breakpoint at"));
} 