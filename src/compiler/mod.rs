// Compiler integration module for the dbug debugger

use std::path::Path;
use std::process::Command;
use std::collections::HashMap;

/// Options for building a Rust project
#[allow(dead_code)]
pub struct BuildOptions {
    /// The target directory for build output
    pub target_dir: Option<String>,
    /// Whether to use release mode
    pub release: bool,
    /// Additional arguments to pass to cargo
    pub cargo_args: Vec<String>,
    /// Compiler flags to pass to rustc
    pub rustc_flags: Vec<String>,
    /// Environment variables to set during compilation
    pub env_vars: HashMap<String, String>,
}


/// Represents a Rust project to be debugged
#[allow(dead_code)]
pub struct RustProject {
    /// The path to the project root
    pub path: String,
}

impl RustProject {
    /// Create a new RustProject
    #[allow(dead_code)]
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
    
    /// Check if the project is valid
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        let cargo_toml = Path::new(&self.path).join("Cargo.toml");
        cargo_toml.exists()
    }
    
    /// Build the project using cargo
    #[allow(dead_code)]
    pub fn build(&self, options: &BuildOptions) -> Result<(), String> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.path);
        cmd.arg("build");
        
        if options.release {
            cmd.arg("--release");
        }
        
        if let Some(target_dir) = &options.target_dir {
            cmd.args(["--target-dir", target_dir]);
        }
        
        // Add any additional cargo arguments
        for arg in &options.cargo_args {
            cmd.arg(arg);
        }
        
        // Add rustc flags
        if !options.rustc_flags.is_empty() {
            // Join all rustc flags into a single string
            let rustc_flags = options.rustc_flags.join(" ");
            cmd.args(["-Z", "unstable-options", "--config", &format!("build.rustflags=[{:?}]", rustc_flags)]);
        }
        
        // Set environment variables
        for (key, value) in &options.env_vars {
            cmd.env(key, value);
        }
        
        // Run the build command
        let status = cmd.status().map_err(|e| e.to_string())?;
        
        if status.success() {
            Ok(())
        } else {
            Err(format!("Build failed with exit code: {:?}", status.code()))
        }
    }
    
    /// Clean the project
    #[allow(dead_code)]
    pub fn clean(&self, target_dir: Option<&str>) -> Result<(), String> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.path);
        cmd.arg("clean");
        
        if let Some(dir) = target_dir {
            cmd.args(["--target-dir", dir]);
        }
        
        let status = cmd.status().map_err(|e| e.to_string())?;
        
        if status.success() {
            Ok(())
        } else {
            Err(format!("Clean failed with exit code: {:?}", status.code()))
        }
    }
    
    /// Get the output directory for build artifacts
    #[allow(dead_code)]
    pub fn get_target_dir(&self, custom_dir: Option<&str>, release: bool) -> String {
        let base_dir = match custom_dir {
            Some(dir) => dir.to_string(),
            None => format!("{}/target", self.path),
        };
        
        if release {
            format!("{}/release", base_dir)
        } else {
            format!("{}/debug", base_dir)
        }
    }
} 