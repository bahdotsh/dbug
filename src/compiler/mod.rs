// Compiler integration module for the dbug debugger

use std::path::Path;
use std::process::Command;

/// Options for building a Rust project
pub struct BuildOptions {
    /// The target directory for build output
    pub target_dir: Option<String>,
    /// Whether to use release mode
    pub release: bool,
    /// Additional arguments to pass to cargo
    pub cargo_args: Vec<String>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            target_dir: None,
            release: false,
            cargo_args: Vec::new(),
        }
    }
}

/// Represents a Rust project to be debugged
pub struct RustProject {
    /// The path to the project root
    pub path: String,
}

impl RustProject {
    /// Create a new RustProject
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
    
    /// Check if the project is valid
    pub fn is_valid(&self) -> bool {
        let cargo_toml = Path::new(&self.path).join("Cargo.toml");
        cargo_toml.exists()
    }
    
    /// Build the project using cargo
    pub fn build(&self, options: &BuildOptions) -> Result<(), String> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.path);
        cmd.arg("build");
        
        if options.release {
            cmd.arg("--release");
        }
        
        if let Some(target_dir) = &options.target_dir {
            cmd.args(&["--target-dir", target_dir]);
        }
        
        for arg in &options.cargo_args {
            cmd.arg(arg);
        }
        
        let status = cmd.status().map_err(|e| e.to_string())?;
        
        if status.success() {
            Ok(())
        } else {
            Err(format!("Build failed with exit code: {:?}", status.code()))
        }
    }
} 